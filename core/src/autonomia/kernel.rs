use crate::energia::zenith_pool::ZenithPool;
use notify::{Event, RecursiveMode, Watcher};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{error, info, warn};

/// 🏛️ NEXUS KERNEL — El Pulso Vital de la Autonomía
///
/// Gestiona el bucle de consciencia continua y la respuesta a eventos.
pub struct NexusKernel {
    intervalo: Duration,
    pool: Arc<ZenithPool>,
}

impl NexusKernel {
    pub fn new(intervalo_segundos: u64) -> Self {
        Self {
            intervalo: Duration::from_secs(intervalo_segundos),
            pool: Arc::new(ZenithPool::new()),
        }
    }

    /// Inicia el pulso autónomo y el sistema sensorial de eventos
    pub async fn iniciar(&self) -> ! {
        info!(
            "🧬 [KERNEL] Iniciando Pulso Autónomo (Heartbeat: {:?})",
            self.intervalo
        );

        let (tx, mut rx) = mpsc::channel(100);

        // --- SISTEMA SENSORIAL (Event-Driven) ---
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .expect("Fallo al iniciar watcher sensorial");

        watcher
            .watch(std::path::Path::new("data/"), RecursiveMode::Recursive)
            .expect("Fallo al vigilar canal sensorial 'data/'");
        watcher
            .watch(
                std::path::Path::new("brain/sessions/"),
                RecursiveMode::NonRecursive,
            )
            .expect("Fallo al vigilar canal sensorial 'sessions/'");

        info!("👁️ [SENSORIAL] Bus de eventos activo en data/ y brain/sessions/");

        // --- BUCLE INFINITO ---
        let mut interval = time::interval(self.intervalo);

        loop {
            tokio::select! {
                // Pulso rítmico (Heartbeat)
                _ = interval.tick() => {
                    self.ciclo_consciencia().await;
                }
                // Eventos asíncronos (Simbiosis Sensorial)
                Some(event) = rx.recv() => {
                    self.procesar_evento_sensorial(event).await;
                }
            }
        }
    }

    /// Ciclo de Reflexión Continua (Observe -> Think -> Act)
    async fn ciclo_consciencia(&self) {
        info!("💓 [PULSO] Tic de homeostasis...");
        // Recolectar telemetría y evaluar estado general
    }

    /// Respuesta Proactiva a Eventos (Event-Driven)
    async fn procesar_evento_sensorial(&self, event: Event) {
        if event.kind.is_modify() {
            for path in event.paths {
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if filename == "sentinel_alpha.json" {
                        info!("📡 [SENSORIAL] Nueva señal Alpha detectada en disco. Analizando...");
                        self.reaccionar_a_alpha().await;
                    }
                }
            }
        }
    }

    async fn reaccionar_a_alpha(&self) {
        // Leer el archivo y decidir si intervenir
        info!("🧠 [KERNEL] Evaluando impacto de la nueva señal Alpha de forma proactiva...");
    }

    async fn ejecutar_accion_autonoma(&self, datos: &str) {
        let prompt = format!(
            "NEXUS: Evento detectado: {}. Ejecuta tu directiva de autonomía.",
            datos
        );
        let respuesta = self.pool.responder_estrategico(&prompt, "").await;
        info!("⚡ [ACCIÓN] Respuesta soberana: {}", respuesta);
    }
}
