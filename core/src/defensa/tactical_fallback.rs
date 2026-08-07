// ==========================================
// TACTICAL FALLBACK - Modo de emergencia
// ==========================================

use crate::infra::policy::TacticalFallback;
use reqwest::Client;
use std::time::Duration;

pub struct TacticalFallbackDaemon {
    pub config: TacticalFallback, // ← HACER PÚBLICO
    mode: String,
    client: Client,
    last_check: std::time::Instant,
}

impl TacticalFallbackDaemon {
    pub fn new(config: TacticalFallback) -> Self {
        Self {
            mode: "NORMAL".to_string(),
            client: Client::new(),
            last_check: std::time::Instant::now(),
            config,
        }
    }

    pub async fn verify_token(&self) -> bool {
        let url = &self.config.token_verification_url;

        match self
            .client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(e) => {
                tracing::warn!("⚠️ No se pudo verificar token: {}", e);
                false
            }
        }
    }

    pub async fn check_and_update_mode(&mut self) {
        let token_valid = self.verify_token().await;

        if !token_valid {
            let new_mode = self.config.fallback_mode.clone();
            if self.mode != new_mode {
                tracing::error!("🚨 TOKEN INVÁLIDO - Cambiando a modo: {}", new_mode);
                self.mode = new_mode;
            }
        } else if self.mode != "NORMAL" {
            tracing::info!("✅ Token verificado - Volviendo a modo NORMAL");
            self.mode = "NORMAL".to_string();
        }

        self.last_check = std::time::Instant::now();
    }

    pub fn needs_check(&self) -> bool {
        self.last_check.elapsed().as_secs() > self.config.token_check_interval_minutes * 60
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == "SENTINEL_READ_ONLY"
    }

    pub fn get_mode(&self) -> &str {
        &self.mode
    }
}
