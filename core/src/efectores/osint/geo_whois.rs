// ──────────────────────────────────────────────
// 🌍 GEO WHOIS — Consulta whois + geolocalización IP
// Whois vía comando del sistema (`whois`)
// Geolocalización vía ip-api.com (gratis, 45 req/min, no requiere API key)
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::{info, warn};

/// Información de geolocalización IP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoResult {
    pub ip: String,
    pub pais: String,
    pub region: String,
    pub ciudad: String,
    pub lat: f64,
    pub lon: f64,
    pub isp: String,
    pub org: String,
    pub asn: String,
    pub tiempo_zona: String,
}

/// Información whois de un dominio/IP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoisResult {
    pub target: String,
    pub raw: String,
    pub registrador: Option<String>,
    pub creado: Option<String>,
    pub expiracion: Option<String>,
    pub dns_servers: Vec<String>,
    pub emails: Vec<String>,
    pub organizacion: Option<String>,
}

/// 🌍 Geolocalizador + Whois
pub struct GeoWhois;

impl GeoWhois {
    pub fn new() -> Self {
        Self
    }

    /// Geolocaliza una dirección IP via ip-api.com
    pub async fn geo_ip(&self, ip: &str) -> anyhow::Result<GeoResult> {
        info!("🌍 [GEO] Geolocalizando IP: {}", ip);

        let url = format!("http://ip-api.com/json/{}?fields=status,country,regionName,city,lat,lon,isp,org,as,timezone", ip);

        let resp = reqwest::get(&url).await?;
        let data: serde_json::Value = resp.json().await?;

        if data.get("status").and_then(|s| s.as_str()) != Some("success") {
            anyhow::bail!("ip-api.com falló para IP: {}", ip);
        }

        let result = GeoResult {
            ip: ip.to_string(),
            pais: data
                .get("country")
                .and_then(|v| v.as_str())
                .unwrap_or("Desconocido")
                .to_string(),
            region: data
                .get("regionName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ciudad: data
                .get("city")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            lat: data.get("lat").and_then(|v| v.as_f64()).unwrap_or(0.0),
            lon: data.get("lon").and_then(|v| v.as_f64()).unwrap_or(0.0),
            isp: data
                .get("isp")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            org: data
                .get("org")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            asn: data
                .get("as")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            tiempo_zona: data
                .get("timezone")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        };

        info!("🌍 [GEO] IP '{}' → {} ({})", ip, result.ciudad, result.pais);
        Ok(result)
    }

    /// Consulta whois vía comando del sistema
    pub async fn whois(&self, target: &str) -> anyhow::Result<WhoisResult> {
        info!("🌍 [WHOIS] Consultando: {}", target);

        let output = Command::new("whois").arg(target).output().await?;

        if !output.status.success() {
            anyhow::bail!("whois falló para: {}", target);
        }

        let raw = String::from_utf8_lossy(&output.stdout).to_string();

        // Extraer campos relevantes del whois
        let registrador = Self::extract_whois_field(
            &raw,
            &[
                "Registrar:",
                "registrar:",
                "Registrar Organization:",
                "Registrar:",
                "Sponsoring Registrar:",
            ],
        );

        let creado = Self::extract_whois_field(
            &raw,
            &[
                "Creation Date:",
                "created:",
                "creation_date:",
                "Domain Creation Date:",
            ],
        );

        let expiracion = Self::extract_whois_field(
            &raw,
            &[
                "Registry Expiry Date:",
                "expire:",
                "expiration_date:",
                "Registrar Registration Expiration Date:",
            ],
        );

        let organizacion = Self::extract_whois_field(
            &raw,
            &[
                "OrgName:",
                "org:",
                "organization:",
                "Registrant Organization:",
                "Organization:",
            ],
        );

        // Extraer DNS servers
        let dns_servers: Vec<String> = raw
            .lines()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("name server:")
                    || lower.contains("nserver:")
                    || lower.contains("dns server:")
            })
            .filter_map(|l| {
                let parts: Vec<&str> = l.splitn(2, ':').collect();
                if parts.len() == 2 {
                    Some(parts[1].trim().trim_end_matches('.').to_string())
                } else {
                    None
                }
            })
            .filter(|s| !s.is_empty())
            .collect();

        // Extraer emails del whois
        let email_regex =
            regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        let emails: Vec<String> = email_regex
            .find_iter(&raw)
            .map(|m| m.as_str().to_string())
            .filter(|e| !e.contains("noreply") && !e.contains("no-reply"))
            .collect();

        let result = WhoisResult {
            target: target.to_string(),
            raw: raw.chars().take(2000).collect(), // Truncar a 2000 chars
            registrador,
            creado,
            expiracion,
            dns_servers,
            emails,
            organizacion,
        };

        info!(
            "🌍 [WHOIS] '{}' → Registrador: {:?}, Creado: {:?}",
            target, result.registrador, result.creado
        );

        Ok(result)
    }

    /// Geolocaliza + whois combinado para un dominio/IP
    pub async fn investigar_ip(&self, ip: &str) -> anyhow::Result<(GeoResult, WhoisResult)> {
        let geo = self.geo_ip(ip).await?;
        let whois = self.whois(ip).await?;
        Ok((geo, whois))
    }

    /// Verifica si whois está disponible en el sistema
    pub async fn is_whois_available() -> bool {
        Command::new("which")
            .arg("whois")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    // ─── Privados ────────────────────────────────

    fn extract_whois_field(raw: &str, patterns: &[&str]) -> Option<String> {
        for line in raw.lines() {
            let lower = line.to_lowercase();
            for pattern in patterns {
                let lower_pattern = pattern.to_lowercase();
                if lower.contains(&lower_pattern) {
                    // Extraer el valor después del separador
                    for sep in &[": ", ":\t"] {
                        if let Some(val) = line.split(sep).nth(1) {
                            let val = val.trim();
                            if !val.is_empty() && val != "-" {
                                return Some(val.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_whois_field() {
        let raw =
            "Registrar: GoDaddy.com, LLC\nCreation Date: 2020-01-15\nOrganization: Example Corp";
        assert_eq!(
            GeoWhois::extract_whois_field(raw, &["Registrar:", "registrar:"]),
            Some("GoDaddy.com, LLC".to_string())
        );
        assert_eq!(
            GeoWhois::extract_whois_field(raw, &["Creation Date:", "created:"]),
            Some("2020-01-15".to_string())
        );
    }
}
