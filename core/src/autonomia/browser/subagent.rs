use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::info;
use crate::infra::navegador_soberano::{fetch_html_native, BrowserPool};

pub struct BrowserSubagent {
    pub port: u16,
    pool: Option<Arc<BrowserPool>>,
}

impl BrowserSubagent {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            pool: BrowserPool::new_auto(),
        }
    }

    pub async fn execute_task(&mut self, url: &str, task: &str) -> Result<()> {
        info!("🌐 [NEXUS-BROWSER-SUBAGENT] Navegando a {} para tarea: {}", url, task);

        if let Some(ref pool) = self.pool {
            let page = pool.acquire(None).await?;
            let _ = page
                .evaluate_on_new_document(crate::defensa::camuflaje_omega::STEALTH_PAYLOAD)
                .await;
            page.goto(url).await?;
            info!("✅ [NEXUS-BROWSER-SUBAGENT] Pestaña cargada vía BrowserPool soberano.");
            Ok(())
        } else {
            let (status, _html) = fetch_html_native(url, Some(2000)).await?;
            info!("✅ [NEXUS-BROWSER-SUBAGENT] Fetch nativo completado con status HTTP {}", status);
            Ok(())
        }
    }
}
