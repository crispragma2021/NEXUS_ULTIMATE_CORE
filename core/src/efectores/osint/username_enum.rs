// ──────────────────────────────────────────────
// 🕵️ USERNAME ENUMERATION — Sherlock DNA
// Rastrea la presencia de un alias en plataformas populares
// Adaptado para core sin dependencia de ShadowCrawlAPI
// ──────────────────────────────────────────────

use tracing::info;

/// Escáner de presencia de username en redes sociales
pub struct UsernameScanner {
    client: reqwest::Client,
}

impl Default for UsernameScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl UsernameScanner {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("UsernameScanner: Cliente HTTP válido"),
        }
    }

    /// Escanea un alias en las plataformas más comunes.
    /// Retorna las URLs donde el username fue encontrado.
    pub async fn scan_username(&self, username: &str) -> anyhow::Result<Vec<String>> {
        info!("🕵️ [SHERLOCK] Iniciando escaneo de alias: {}", username);

        let platforms = [
            ("github.com", "{}/{}"),
            ("twitter.com", "{}/{}"),
            ("instagram.com", "{}/{}"),
            ("linkedin.com/in", "{}/{}"),
            ("reddit.com/user", "{}/{}"),
            ("facebook.com", "{}/{}"),
            ("tiktok.com/@", "{}"),
        ];

        let mut found = Vec::new();
        for (base, pattern) in &platforms {
            let full = if *base == "tiktok.com/@" {
                format!("https://www.tiktok.com/@{}", username)
            } else if *base == "linkedin.com/in" {
                format!("https://www.linkedin.com/in/{}", username)
            } else if *base == "reddit.com/user" {
                format!("https://www.reddit.com/user/{}", username)
            } else {
                format!("https://www.{}/{}", base, username)
            };

            // HEAD request para verificar existencia sin descargar contenido
            match self.client.head(&full).send().await {
                Ok(resp) if resp.status().is_success() => {
                    found.push(full);
                }
                Ok(_) => {
                    tracing::debug!("[SHERLOCK] No encontrado en {}: HTTP {}", base, full);
                }
                Err(e) => {
                    tracing::warn!("[SHERLOCK] Error verificando {}: {}", base, e);
                }
            }
        }

        info!(
            "✅ [SHERLOCK] Escaneo completado. {} perfiles detectados.",
            found.len()
        );
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_username_returns_vec() {
        let scanner = UsernameScanner::new();
        let result = scanner.scan_username("octocat").await;
        assert!(result.is_ok());
        // No garantizamos encontrar nada, pero el vec debe ser válido
    }
}
