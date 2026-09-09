// 🐝 COLMENA MADRE — Servidor gRPC del Enjambre
// Escucha en 0.0.0.0:PUERTO, maneja hijos con heartbeat y failover.
// Adaptado de legacy/nexus-orquestador/src/colmena/madre.rs
// SIN tokio-stream — usa futures::stream::unfold para convertir mpsc::Receiver en Stream.

use crate::colmena::proto::swarm_control_server::{SwarmControl, SwarmControlServer};
use crate::colmena::proto::{Ack, Heartbeat, NodeStatus, SwarmCommand, TaskResult};
use futures::stream::{unfold, Stream};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{error, info, warn};

/// Nodo hijo conectado a la colmena
pub struct ChildNode {
    pub node_id: String,
    pub last_seen: std::time::Instant,
    pub tx: mpsc::Sender<Result<SwarmCommand, Status>>,
    pub cpu_temp: f32,
    pub gemini_latency: u32,
}

pub struct ColmenaMadre {
    active_children: Arc<RwLock<HashMap<String, ChildNode>>>,
}

impl Default for ColmenaMadre {
    fn default() -> Self {
        Self::new()
    }
}

impl ColmenaMadre {
    pub fn new() -> Self {
        Self {
            active_children: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(
        self: Arc<Self>,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let addr = format!("0.0.0.0:{}", port).parse()?;
        info!("🐝 [COLMENA MADRE] Iniciando servidor gRPC en {}", addr);

        // Spawn Watcher de Hijos Muertos (heartbeat timeout cada 5s)
        let children = self.active_children.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let mut map = children.write().await;
                map.retain(|id, node| {
                    if node.last_seen.elapsed().as_secs() > 10 {
                        warn!(
                            "💀 [COLMENA MADRE] Hijo {} perdido en combate (Timeout).",
                            id
                        );
                        false
                    } else {
                        // Enviar Heartbeat asíncrono (Ping)
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let _ = node.tx.try_send(Ok(SwarmCommand {
                            command: Some(crate::colmena::proto::swarm_command::Command::Ping(
                                Heartbeat { timestamp },
                            )),
                        }));
                        true
                    }
                });
            }
        });

        Server::builder()
            .add_service(SwarmControlServer::new(self.clone()))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl SwarmControl for Arc<ColmenaMadre> {
    type JoinSwarmStream =
        Pin<Box<dyn Stream<Item = Result<SwarmCommand, Status>> + Send + 'static>>;

    async fn join_swarm(
        &self,
        request: Request<tonic::Streaming<NodeStatus>>,
    ) -> Result<Response<Self::JoinSwarmStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);

        // Primer mensaje esperado del hijo
        if let Some(status) = in_stream.message().await? {
            info!(
                "🐝 [COLMENA MADRE] Nuevo Hijo conectándose: {}",
                status.node_id
            );
            let child = ChildNode {
                node_id: status.node_id.clone(),
                last_seen: std::time::Instant::now(),
                tx: tx.clone(),
                cpu_temp: status.cpu_temp,
                gemini_latency: status.gemini_latency_ms,
            };

            self.active_children
                .write()
                .await
                .insert(status.node_id.clone(), child);

            let children_map = self.active_children.clone();
            let node_id = status.node_id.clone();

            // Mantener el stream de lectura vivo (Telemetry)
            tokio::spawn(async move {
                while let Ok(Some(status)) = in_stream.message().await {
                    if let Some(node) = children_map.write().await.get_mut(&node_id) {
                        node.last_seen = std::time::Instant::now();
                        node.cpu_temp = status.cpu_temp;
                        node.gemini_latency = status.gemini_latency_ms;
                    }
                }
                warn!("🔌 [COLMENA MADRE] Stream cerrado por el Hijo {}", node_id);
            });
        } else {
            return Err(Status::invalid_argument("Se esperaba NodeStatus inicial"));
        }

        // Convertir mpsc::Receiver en Stream sin tokio-stream:
        // usamos futures::stream::unfold igual que en ccxt_rs/ws.rs
        let stream = unfold(
            rx,
            |mut rx: mpsc::Receiver<Result<SwarmCommand, Status>>| async move {
                rx.recv().await.map(|msg| (msg, rx))
            },
        );

        Ok(Response::new(Box::pin(stream) as Self::JoinSwarmStream))
    }

    async fn report_task(&self, request: Request<TaskResult>) -> Result<Response<Ack>, Status> {
        let res = request.into_inner();
        info!(
            "📋 [COLMENA MADRE] Resultado de {} recibido de {}: Exito={}",
            res.task_id, res.node_id, res.success
        );
        // Aquí se reinsertaría la memoria compartida en SQLite/LanceDB
        Ok(Response::new(Ack { received: true }))
    }
}
