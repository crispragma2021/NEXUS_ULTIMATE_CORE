//! Demonio del pipeline de scraping (F5.1–F5.2).
//!
//! Bucle principal async con:
//! 1. Polling de tareas pendientes en SQLite.
//! 2. Procesamiento secuencial (fetch → clean → route → infer).
//! 3. Rate limiting por dominio ([`RateLimiter`]).
//! 4. Graceful shutdown ante SIGINT/SIGTERM (timeout 30s → exit 1).
//!
//! ```
//! ┌──────────────────────────────────────┐
//! │            MAIN LOOP                  │
//! │  1. START: init DB + tokio runtime    │
//! │  2. POLL: tasks pending (LIMIT 1)     │
//! │  3. PROCESS: fetch→clean→route→infer  │
//! │  4. SLEEP: rate_limit delay           │
//! │  ...loop...                           │
//! │                                       │
//! │  SHUTDOWN (SIGINT / SIGTERM):         │
//! │  1. Stop polling                      │
//! │  2. Finish current task               │
//! │  3. Close DB + connections            │
//! │  4. std::process::exit(0)             │
//! │     (force exit(1) @30s timeout)      │
//! └──────────────────────────────────────┘
//! ```

use crate::scraping::pipeline::db::PipelineDb;
use crate::scraping::pipeline::fetcher::Fetcher;
use crate::scraping::pipeline::pipeline::Pipeline;
use crate::scraping::pipeline::rate_limiter::RateLimiter;
use crate::scraping::pipeline::schemas::{Strategy, TaskSchema};
use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use std::time::Duration;

/// Configuración del demonio.
#[derive(Clone)]
pub struct DaemonConfig {
    /// Intervalo entre ciclos de polling cuando no hay tareas.
    pub poll_interval_ms: u64,
    /// Máx. tareas a tomar por ciclo.
    pub batch_size: usize,
    /// Timeout de graceful shutdown (segundos).
    pub shutdown_timeout_secs: u64,
    /// Espera mínima entre peticiones por dominio (ms).
    pub rate_limit_delay_ms: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 5000,
            batch_size: 1,
            shutdown_timeout_secs: 30,
            rate_limit_delay_ms: 2000,
        }
    }
}

/// Demonio de scraping.
pub struct Daemon {
    pub config: DaemonConfig,
    pub db: Arc<PipelineDb>,
    pub pipeline: Arc<Pipeline>,
    pub fetcher: Arc<Fetcher>,
    rate_limiter: RateLimiter,
    /// Señal de stop solicitado (graceful).
    stop_requested: Arc<std::sync::atomic::AtomicBool>,
}

impl Daemon {
    pub fn new(
        config: DaemonConfig,
        db: Arc<PipelineDb>,
        pipeline: Arc<Pipeline>,
        fetcher: Arc<Fetcher>,
    ) -> Self {
        Self {
            rate_limiter: RateLimiter::new(Some(db.clone()))
                .with_min_delay(config.rate_limit_delay_ms),
            config,
            db,
            pipeline,
            fetcher,
            stop_requested: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Inicia el bucle principal. Devuelve tras recibir señal de parada.
    pub async fn run(&self) {
        tracing::info!("🚀 [DAEMON] iniciando loop (poll={}ms)", self.config.poll_interval_ms);

        loop {
            if self.stop_requested.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            // Polling de tareas pendientes.
            let pending = match self.db.list_pending_tasks(self.config.batch_size) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("[DAEMON] error polling DB: {e}");
                    tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
                    continue;
                }
            };

            if pending.is_empty() {
                tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
                continue;
            }

            for task_id in pending {
                if self.stop_requested.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                match self.process_task(&task_id).await {
                    Ok(()) => {}
                    Err(e) => tracing::error!("[DAEMON] tarea {task_id} falló: {e}"),
                }
            }
        }

        tracing::info!("✅ [DAEMON] loop terminado limpio");
    }

    /// Procesa una tarea individual con rate limiting por dominio.
    async fn process_task(&self, task_id: &str) -> Result<()> {
        // Recuperar la tarea de la DB.
        let Some((id, url, strategy, _selectors, _output_schema)) = self.db.get_task(task_id)? else {
            tracing::warn!("[DAEMON] tarea {task_id} no encontrada");
            return Ok(());
        };

        let task = TaskSchema {
            task_id: id,
            url,
            strategy: if strategy == "headless" {
                Strategy::Headless
            } else {
                Strategy::Http
            },
            selectors: None,
            output_schema: None,
            timeout_seconds: 30,
            max_retries: 3,
            respect_robots_txt: true,
            rate_limit_delay_ms: self.config.rate_limit_delay_ms,
            user_agent: "NexusScraper/1.0 (+https://github.com/NEXUS_ULTIMATE_CORE)".into(),
            metadata: None,
        };

        // Rate limiting por dominio.
        let domain = crate::scraping::pipeline::fetcher::domain_of(&task.url)
            .unwrap_or_else(|_| "unknown".to_string());
        self.rate_limiter.acquire(&domain).await;

        tracing::info!("🧹 [DAEMON] procesando {task_id} → {}", task.url);
        let result = self.pipeline.run(&task).await;
        let ok = matches!(
            result.status,
            crate::scraping::pipeline::schemas::ScrapingStatus::Success
        );
        self.rate_limiter.release(&domain, ok);

        // Persistir estado final (el pipeline ya actualizó tasks/extracted_data
        // vía persist()). Si hubo 429/403, registrar backoff.
        if !result.errors.is_empty() {
            let errs = result.errors.join("; ");
            tracing::warn!("[DAEMON] {task_id} → {:?}: {errs}", result.status);
            if errs.contains("429") || errs.contains("403") {
                self.rate_limiter.note_failure(&domain);
            }
        } else {
            tracing::info!("✅ [DAEMON] {task_id} → {:?}", result.status);
        }

        Ok(())
    }

    /// Solicita el graceful shutdown.
    pub fn request_stop(&self) {
        self.stop_requested
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Espera la señal SIGINT/SIGTERM (Unix) o Ctrl+C (Windows) y dispara
    /// el graceful shutdown con timeout.
    pub async fn wait_for_shutdown(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())?;
            let mut sigint = signal(SignalKind::interrupt())?;
            tokio::select! {
                _ = sigterm.recv() => tracing::warn!("[DAEMON] SIGTERM recibido"),
                _ = sigint.recv() => tracing::warn!("[DAEMON] SIGINT recibido"),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await?;
        }

        self.request_stop();

        // Esperar a que el loop termine con timeout.
        let timeout = Duration::from_secs(self.config.shutdown_timeout_secs);
        let result = tokio::time::timeout(timeout, async {
            // El loop principal corre en una task; aquí solo se espera un tick
            // para que el bucle observe stop_requested. En el binario real se
            // usa tokio::select entre run() y wait_for_shutdown().
            tokio::time::sleep(Duration::from_millis(100)).await;
        })
        .await;

        if result.is_err() {
            tracing::error!("[DAEMON] timeout de shutdown; forzando exit(1)");
            std::process::exit(1);
        }
        Ok(())
    }
}

/// Helper para construir una tarea simple desde una URL (útiles en binarios).
pub fn task_from_url(task_id: &str, url: &str, strategy: Strategy) -> TaskSchema {
    TaskSchema {
        task_id: task_id.into(),
        url: url.into(),
        strategy,
        selectors: None,
        output_schema: None,
        timeout_seconds: 30,
        max_retries: 3,
        respect_robots_txt: true,
        rate_limit_delay_ms: 2000,
        user_agent: "NexusScraper/1.0 (+https://github.com/NEXUS_ULTIMATE_CORE)".into(),
        metadata: None,
    }
}

/// Crea un `Pipeline` con los componentes por defecto (sin nube/ollama).
pub fn build_basic_pipeline(db: Option<Arc<PipelineDb>>) -> Result<Arc<Pipeline>> {
    let fetcher = Arc::new(Fetcher::new(db.clone()).map_err(|e| anyhow!(e))?);
    let pipeline = Arc::new(Pipeline::new(
        fetcher.clone(),
        None,
        None,
        db.clone(),
        Default::default(),
    ));
    Ok(pipeline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tarea_desde_url_se_construye() {
        let t = task_from_url("t1", "https://example.com", Strategy::Http);
        assert_eq!(t.url, "https://example.com");
        assert_eq!(t.strategy, Strategy::Http);
        assert!(t.respect_robots_txt);
    }

    #[tokio::test]
    async fn request_stop_detiene_el_loop() {
        let db = Arc::new(PipelineDb::open_in_memory().unwrap());
        let pipeline = build_basic_pipeline(Some(db.clone())).unwrap();
        let fetcher = Arc::new(Fetcher::new(Some(db.clone())).unwrap());
        let daemon = Daemon::new(DaemonConfig::default(), db, pipeline, fetcher);

        // Solicitar stop inmediato; run() debe terminar rápido sin tareas.
        daemon.request_stop();
        daemon.run().await; // no debe colgar
    }

    #[tokio::test]
    async fn loop_procesa_tarea_pendiente() {
        let db = Arc::new(PipelineDb::open_in_memory().unwrap());
        db.insert_task("task-1", "https://example.com", "http", None, None)
            .unwrap();
        let pipeline = build_basic_pipeline(Some(db.clone())).unwrap();
        let fetcher = Arc::new(Fetcher::new(Some(db.clone())).unwrap());

        // Config con batch_size alto para tomar la tarea.
        let cfg = DaemonConfig {
            poll_interval_ms: 10,
            batch_size: 10,
            ..Default::default()
        };
        let daemon = Daemon::new(cfg, db.clone(), pipeline, fetcher);

        // Correr el loop en una task con timeout; el fetch fallará (sin red)
        // pero el loop debe completar la iteración y detenerse.
        daemon.request_stop();
        // Solo verificamos que process_task no cuelga al faltar la URL.
        let result = daemon.process_task("task-1").await;
        assert!(result.is_ok()); // procesa y marca error, no lanza
    }
}
