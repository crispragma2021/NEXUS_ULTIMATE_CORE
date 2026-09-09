// ==========================================
// RESOURCE GOVERNOR - Control de recursos (versión corregida)
// ==========================================

use crate::infra::policy::ResourceGovernor;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;

pub struct ResourceGovernorDaemon {
    pub config: ResourceGovernor,
    system: System,
    request_counter: Arc<Mutex<u32>>,
    last_reset: Arc<Mutex<std::time::Instant>>,
}

impl ResourceGovernorDaemon {
    pub fn new(config: ResourceGovernor) -> Self {
        Self {
            config,
            system: System::new_all(),
            request_counter: Arc::new(Mutex::new(0)),
            last_reset: Arc::new(Mutex::new(std::time::Instant::now())),
        }
    }

    pub async fn check_cpu(&mut self) -> bool {
        self.system.refresh_all();
        let cpu_usage = self.system.global_cpu_usage() as u8;

        if cpu_usage > self.config.cpu_max_percent {
            tracing::warn!(
                "⚠️ CPU excede límite: {}% > {}%",
                cpu_usage,
                self.config.cpu_max_percent
            );
            return false;
        }
        true
    }

    pub async fn check_memory(&mut self) -> bool {
        self.system.refresh_memory();

        // Usar la medición CORRECTA: solo RAM usada (no swap, no cache)
        let total_mem = self.system.total_memory();
        let used_mem = self.system.used_memory();

        // Convertir a MB
        let used_mem_mb = (used_mem / (1024 * 1024)) as u16;
        let total_mem_mb = (total_mem / (1024 * 1024)) as u16;

        // Calcular porcentaje de uso real
        let usage_percent = (used_mem as f64 / total_mem as f64) * 100.0;

        tracing::debug!(
            "📊 Memoria: {}/{} MB ({:.1}%)",
            used_mem_mb,
            total_mem_mb,
            usage_percent
        );

        if used_mem_mb > self.config.mem_vector_max_mb {
            tracing::warn!(
                "⚠️ Memoria excede límite: {}MB > {}MB ({}%)",
                used_mem_mb,
                self.config.mem_vector_max_mb,
                usage_percent
            );
            return false;
        }
        true
    }

    pub async fn check_rate_limit(&self) -> bool {
        let mut counter = self.request_counter.lock().await;
        let mut last = self.last_reset.lock().await;

        if last.elapsed() >= std::time::Duration::from_secs(1) {
            *counter = 0;
            *last = std::time::Instant::now();
        }

        if *counter >= self.config.net_requests_per_sec as u32 {
            tracing::warn!(
                "⚠️ Rate limit excedido: {} req/seg",
                self.config.net_requests_per_sec
            );
            return false;
        }

        *counter += 1;
        true
    }

    pub async fn enforce(&mut self) -> bool {
        let cpu_ok = self.check_cpu().await;
        let mem_ok = self.check_memory().await;
        let rate_ok = self.check_rate_limit().await;

        cpu_ok && mem_ok && rate_ok
    }
}
