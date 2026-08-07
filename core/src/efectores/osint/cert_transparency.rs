// ──────────────────────────────────────────────
// 🔍 CERTIFICATE TRANSPARENCY — Consulta de logs CT
// Fuente: crt.sh (Certificate Transparency database)
// Revela: subdominios, fechas de emisión, CA emisora, SANs
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Certificado encontrado en CT logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertInfo {
    pub id: i64,
    pub issuer: String,
    pub common_name: String,
    pub name_value: String,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
    pub entry_timestamp: Option<String>,
    pub serial_number: Option<String>,
}

/// 🔍 Consultor de Certificate Transparency logs
pub struct CertTransparency {
    client: reqwest::Client,
}

impl Default for CertTransparency {
    fn default() -> Self {
        Self::new()
    }
}

impl CertTransparency {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("NEXUS-OSINT/1.0")
                .timeout(Duration::from_secs(15))
                .build()
                .expect("CertTransparency: Cliente HTTP válido"),
        }
    }

    /// Consulta certificados para un dominio (incluye wildcard)
    pub async fn lookup_domain(&self, domain: &str) -> anyhow::Result<Vec<CertInfo>> {
        info!(
            "🔍 [CERT-TRANSPARENCY] Consultando certificados para: {}",
            domain
        );
        self.query_crtsh(&format!("%.{}", domain)).await
    }

    /// Consulta certificados para un dominio exacto (sin wildcard)
    pub async fn lookup_exact(&self, domain: &str) -> anyhow::Result<Vec<CertInfo>> {
        info!(
            "🔍 [CERT-TRANSPARENCY] Consultando certificados exactos para: {}",
            domain
        );
        self.query_crtsh(domain).await
    }

    /// Query genérica a crt.sh
    async fn query_crtsh(&self, query: &str) -> anyhow::Result<Vec<CertInfo>> {
        let url = format!(
            "https://crt.sh/?q={}&output=json&limit=100",
            urlencoding(query)
        );

        debug!("🔍 [CERT-TRANSPARENCY] URL: {}", url);

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

        // crt.sh devuelve array JSON o un solo objeto en caso de error
        let raw_entries: Vec<CrtshCertEntry> = match serde_json::from_str(&text) {
            Ok(entries) => entries,
            Err(_) => match serde_json::from_str::<CrtshCertEntry>(&text) {
                Ok(entry) => vec![entry],
                Err(e) => anyhow::bail!("Error parseando crt.sh: {}", e),
            },
        };

        let mut certs = Vec::new();
        for entry in raw_entries {
            certs.push(CertInfo {
                id: entry.id,
                issuer: entry.issuer_name.unwrap_or_default(),
                common_name: entry.common_name.unwrap_or_default(),
                name_value: entry.name_value,
                not_before: entry.not_before,
                not_after: entry.not_after,
                entry_timestamp: entry.entry_timestamp,
                serial_number: entry.serial_number,
            });
        }

        info!(
            "🔍 [CERT-TRANSPARENCY] {} certificados encontrados para '{}'",
            certs.len(),
            query
        );
        Ok(certs)
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            '%' => "%25".to_string(),
            '.' => "%2E".to_string(),
            '*' => "%2A".to_string(),
            _ => c.to_string(),
        })
        .collect()
}

// ── Estructura crt.sh ──
#[derive(Debug, Deserialize)]
struct CrtshCertEntry {
    id: i64,
    #[serde(rename = "issuer_name")]
    issuer_name: Option<String>,
    #[serde(rename = "common_name")]
    common_name: Option<String>,
    #[serde(rename = "name_value")]
    name_value: String,
    #[serde(rename = "not_before")]
    not_before: Option<String>,
    #[serde(rename = "not_after")]
    not_after: Option<String>,
    #[serde(rename = "entry_timestamp")]
    entry_timestamp: Option<String>,
    #[serde(rename = "serial_number")]
    serial_number: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("*.example.com"), "%2A%2Eexample%2Ecom");
        assert_eq!(urlencoding("test"), "test");
    }

    #[test]
    fn test_cert_transparency_creation() {
        let ct = CertTransparency::new();
        // Solo verificar creación
        assert!(true);
    }
}
