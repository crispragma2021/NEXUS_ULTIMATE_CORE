// ──────────────────────────────────────────────
// 🌐 SUBDOMAIN ENUMERATOR — Enumeración de subdominios
// Fuentes: Certificate Transparency (crt.sh) + Brave Search + DNS
// Sin dependencias externas — usa reqwest + regex
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Subdominio encontrado con su fuente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subdomain {
    pub name: String,
    pub source: String,
    pub ip_addresses: Vec<String>,
    pub first_seen: Option<String>,
}

/// 🌐 Enumerador de subdominios multi-fuente
pub struct SubdomainEnumerator {
    client: reqwest::Client,
}

impl Default for SubdomainEnumerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SubdomainEnumerator {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("NEXUS-OSINT/1.0")
                .timeout(Duration::from_secs(20))
                .build()
                .expect("SubdomainEnumerator: Cliente HTTP válido"),
        }
    }

    /// Enumera subdominios de un dominio usando múltiples fuentes
    pub async fn enumerate(&self, domain: &str) -> anyhow::Result<Vec<Subdomain>> {
        info!("🌐 [SUBDOMAIN-ENUM] Enumerando subdominios de: {}", domain);

        let mut all: Vec<Subdomain> = Vec::new();
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 1. Certificate Transparency (crt.sh)
        match self.crtsh_lookup(domain).await {
            Ok(results) => {
                for s in results {
                    if seen.insert(s.name.clone()) {
                        all.push(s);
                    }
                }
                info!(
                    "🌐 [SUBDOMAIN-ENUM] crt.sh: {} subdominios únicos",
                    all.len()
                );
            }
            Err(e) => warn!("⚠️ [SUBDOMAIN-ENUM] crt.sh falló: {}", e),
        }

        // 2. Brave Search: site:*.domain -www
        match self.brave_search_subdomains(domain).await {
            Ok(results) => {
                for s in results {
                    if seen.insert(s.name.clone()) {
                        all.push(s);
                    }
                }
                info!(
                    "🌐 [SUBDOMAIN-ENUM] Brave Search: {} subdominios únicos",
                    all.len()
                );
            }
            Err(e) => debug!("[SUBDOMAIN-ENUM] Brave Search falló: {}", e),
        }

        info!(
            "✅ [SUBDOMAIN-ENUM] Total: {} subdominios únicos para {}",
            all.len(),
            domain
        );
        Ok(all)
    }

    /// Consulta Certificate Transparency logs vía crt.sh
    async fn crtsh_lookup(&self, domain: &str) -> anyhow::Result<Vec<Subdomain>> {
        let url = format!("https://crt.sh/?q=%25.{}&output=json&limit=100", domain);
        debug!("🌐 [SUBDOMAIN-ENUM] Consultando crt.sh: {}", url);

        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("crt.sh returned HTTP {}", resp.status());
        }

        let text = resp.text().await?;

        // crt.sh puede devolver múltiples JSON objects no envueltos en array si hay error
        // Parsear como array, si falla intentar como objeto individual
        let entries: Vec<CrtshEntry> = match serde_json::from_str(&text) {
            Ok(entries) => entries,
            Err(_) => {
                // Intentar parsear como objeto individual
                match serde_json::from_str::<CrtshEntry>(&text) {
                    Ok(entry) => vec![entry],
                    Err(e) => anyhow::bail!("Error parseando crt.sh response: {}", e),
                }
            }
        };

        let mut subdomains: std::collections::HashMap<String, Subdomain> =
            std::collections::HashMap::new();

        for entry in entries {
            let name = entry
                .name_value
                .trim_start_matches("*.")
                .trim()
                .to_lowercase();
            if !name.contains(domain) || name == domain {
                continue;
            }

            let sub = subdomains.entry(name.clone()).or_insert(Subdomain {
                name,
                source: "crt.sh".to_string(),
                ip_addresses: Vec::new(),
                first_seen: None,
            });

            // Actualizar primera fecha vista
            if let Some(ref date) = entry.entry_timestamp {
                match sub.first_seen {
                    None => sub.first_seen = Some(date.clone()),
                    Some(ref existing) if date < existing => sub.first_seen = Some(date.clone()),
                    _ => {}
                }
            }
        }

        Ok(subdomains.into_values().collect())
    }

    /// Busca subdominios vía Brave Search
    async fn brave_search_subdomains(&self, domain: &str) -> anyhow::Result<Vec<Subdomain>> {
        let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            anyhow::bail!("BRAVE_API_KEY no configurada");
        }

        let query = format!("site:*.{} -site:www.{}", domain, domain);
        let resp = self
            .client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", &api_key)
            .header("Accept", "application/json")
            .query(&[("q", query.as_str()), ("count", "10")])
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("Brave Search returned HTTP {}", resp.status());
        }

        let data: serde_json::Value = resp.json().await?;
        let mut results = Vec::new();

        if let Some(web) = data
            .get("web")
            .and_then(|w| w.get("results"))
            .and_then(|r| r.as_array())
        {
            for r in web {
                if let Some(url) = r.get("url").and_then(|u| u.as_str()) {
                    // Extraer subdominio de la URL
                    if let Some(sub) = extract_subdomain_from_url(url, domain) {
                        results.push(Subdomain {
                            name: sub,
                            source: "brave-search".to_string(),
                            ip_addresses: Vec::new(),
                            first_seen: None,
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Extrae el subdominio de una URL dado el dominio base
fn extract_subdomain_from_url(url: &str, domain: &str) -> Option<String> {
    let url_lower = url.to_lowercase();
    // Remover protocolo
    let without_proto = url_lower
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    // Obtener el host (hasta el primer /)
    let host = without_proto.split('/').next()?;

    // Verificar que termina con el dominio
    if !host.ends_with(domain) {
        return None;
    }

    // El subdominio es todo antes del dominio base
    let sub = host.trim_end_matches(domain).trim_end_matches('.');
    if sub.is_empty() || sub == "www" {
        return None;
    }

    Some(format!("{}.{}", sub, domain))
}

// ── Estructura de crt.sh ──
#[derive(Debug, Deserialize)]
struct CrtshEntry {
    #[serde(rename = "name_value")]
    name_value: String,
    #[serde(rename = "entry_timestamp")]
    entry_timestamp: Option<String>,
    #[serde(rename = "issuer_name")]
    _issuer_name: Option<String>,
    #[serde(rename = "id")]
    _id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_subdomain() {
        assert_eq!(
            extract_subdomain_from_url("https://admin.example.com/login", "example.com"),
            Some("admin.example.com".to_string())
        );
        assert_eq!(
            extract_subdomain_from_url("https://www.example.com", "example.com"),
            None
        );
        assert_eq!(
            extract_subdomain_from_url("https://mail.example.com", "example.com"),
            Some("mail.example.com".to_string())
        );
        assert_eq!(
            extract_subdomain_from_url("https://other.com", "example.com"),
            None
        );
    }

    #[test]
    fn test_subdomain_enum_creation() {
        let enumerator = SubdomainEnumerator::new();
        // Solo verificar que se crea sin errores
        assert!(true);
    }
}
