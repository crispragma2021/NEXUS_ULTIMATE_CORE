// 🐝 COLMENA HIJO — Cliente gRPC del Enjambre
// Se conecta a la Madre, envía telemetría periódica y recibe comandos.
// Adaptado de legacy/nexus-orquestador/src/colmena/hijo.rs
// SIN tokio-stream — usa futures::stream::unfold para convertir mpsc::Sender en Stream.

use crate::colmena::proto::swarm_control_client::SwarmControlClient;
use crate::colmena::proto::{NodeStatus, SwarmCommand};
use futures::stream::{unfold, Stream};
use std::pin::Pin;
use tracing::{error, info, warn};

pub struct ColmenaHijo {
    madre_addr: String,
    mi_id: String,
}

impl ColmenaHijo {
    pub fn new(madre_addr: String, mi_id: String) -> Self {
        Self { madre_addr, mi_id }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        info!(
            "🐝 [COLMENA HIJO] Iniciando intento de conexión a Madre: {}",
            self.madre_addr
        );

        let mut client = loop {
            match SwarmControlClient::connect(self.madre_addr.clone()).await {
                Ok(c) => break c,
                Err(e) => {
                    warn!(
                        "⏳ [COLMENA HIJO] Madre no encontrada: {}. Reintentando en 5s...",
                        e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        };

        info!("🔗 [COLMENA HIJO] Conectado a la Madre. Entrando al Enjambre.");

        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let mi_id_clone = self.mi_id.clone();

        // Enviar estado inicial y telemetría periódica
        tokio::spawn(async move {
            loop {
                let status = NodeStatus {
                    node_id: mi_id_clone.clone(),
                    cpu_temp: 65.0,
                    gemini_latency_ms: 150,
                    is_idle: true,
                };
                if tx.send(status).await.is_err() {
                    break; // Cortar si el stream muere
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        });

        // Convertir mpsc::Receiver en Stream sin tokio-stream
        let out_stream: Pin<Box<dyn Stream<Item = NodeStatus> + Send + 'static>> =
            Box::pin(unfold(
                rx,
                |mut rx: tokio::sync::mpsc::Receiver<NodeStatus>| async move {
                    rx.recv().await.map(|msg| (msg, rx))
                },
            ));

        let response = client.join_swarm(out_stream).await?;
        let mut in_stream = response.into_inner();

        let mut failover_timeout = 0;

        loop {
            tokio::select! {
                msg = in_stream.message() => {
                    match msg {
                        Ok(Some(cmd)) => {
                            failover_timeout = 0; // Madre está viva, reseteamos timeout
                            self.handle_command(cmd).await;
                        }
                        Ok(None) | Err(_) => {
                            error!("💀 [COLMENA HIJO] Conexión con Madre PERDIDA.");
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    failover_timeout += 1;
                    if failover_timeout > 5 {
                        error!("🔥 [COLMENA HIJO] TIMEOUT DE MADRE SUPERADO (5s). INICIANDO FASE DE EMANCIPACIÓN...");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&self, cmd: SwarmCommand) {
        if let Some(c) = cmd.command {
            match c {
                crate::colmena::proto::swarm_command::Command::Ping(_) => {
                    // Heartbeat recibido, cordón umbilical sano
                }
                crate::colmena::proto::swarm_command::Command::ExecuteTask(task) => {
                    info!(
                        "⚔️ [COLMENA HIJO] Ejecutando Tarea Delegada OMEGA: {}",
                        task.task_id
                    );
                }
                crate::colmena::proto::swarm_command::Command::SyncMemory(mem) => {
                    info!(
                        "💾 [COLMENA HIJO] Inyectando transacción global SQL de la Madre ({} bytes). Clonando conciencia...",
                        mem.sql_statement.len()
                    );
                }
                crate::colmena::proto::swarm_command::Command::Promote(_) => {
                    info!("👑 [COLMENA HIJO] RECIBÍ CORONA MANUAL. ASUMIENDO ROL DE MADRE INMEDIATAMENTE.");
                }
            }
        }
    }
}
