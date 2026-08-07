// ──────────────────────────────────────────────
// 🕵️ MOTOR DE DORKS — Reconocimiento OSINT
// Automatiza la generación y búsqueda de Google Dorks
// Adaptado para core sin dependencia de ShadowCrawlAPI
// ──────────────────────────────────────────────

use tracing::info;

/// Motor de búsqueda con Google Dorks sobre un dominio
pub struct DorkEngine {
    client: reqwest::Client,
}

impl Default for DorkEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DorkEngine {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("DorkEngine: Cliente HTTP válido"),
        }
    }

    /// Ejecuta una búsqueda de dorks sobre un dominio específico
    pub async fn scan_domain(&self, domain: &str) -> anyhow::Result<Vec<String>> {
        info!(
            "🕵️ [DORK-ENGINE] Iniciando reconocimiento sobre: {}",
            domain
        );

        let dorks = [
            format!("site:{} ext:php OR ext:asp OR ext:aspx", domain),
            format!("site:{} intitle:\"index of\"", domain),
            format!("site:{} inurl:admin OR inurl:login", domain),
            format!("site:{} ext:sql OR ext:db OR ext:bak", domain),
            format!("site:{} \"confidential\" OR \"internal use only\"", domain),
        ];

        let mut all_results = Vec::new();
        for dork in &dorks {
            match self.buscar_dork(dork).await {
                Ok(urls) => {
                    for url in urls {
                        all_results.push(format!("[DORK: {}] -> {}", dork, url));
                    }
                }
                Err(e) => {
                    tracing::warn!("❌ [DORK-ENGINE] Error en dork '{}': {}", dork, e);
                }
            }
        }

        info!(
            "✅ [DORK-ENGINE] Escaneo finalizado. {} resultados.",
            all_results.len()
        );
        Ok(all_results)
    }

    /// Ejecuta una búsqueda simple contra un motor de búsqueda público
    async fn buscar_dork(&self, query: &str) -> anyhow::Result<Vec<String>> {
        let url = format!("https://www.google.com/search?q={}", urlencoding(query));
        let resp = self.client.get(&url).send().await?;
        let html = resp.text().await?;

        // Extracción simple de URLs de resultados
        let mut urls = Vec::new();
        for cap in html.match_indices("https://") {
            let start = cap.0;
            let end = html[start..]
                .find('"')
                .map(|e| start + e)
                .unwrap_or(html.len());
            let found = &html[start..end];
            if !found.contains("google.com") && !found.contains("accounts.google") {
                urls.push(found.to_string());
                if urls.len() >= 10 {
                    break;
                }
            }
        }

        Ok(urls)
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            ':' | '/' | '"' => format!("%{:02X}", c as u8),
            _ => c.to_string(),
        })
        .collect()
}
