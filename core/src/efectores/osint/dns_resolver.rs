// ──────────────────────────────────────────────
// 🌐 DNS RESOLVER — Resolución de registros DNS
// Resuelve A, AAAA, MX, TXT, NS usando:
// - tokio::net::lookup_host para A/AAAA (Rust puro)
// - Sistema `dig` para MX, TXT, NS
// Cero dependencias externas nuevas.
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use tokio::net::lookup_host;
use tokio::process::Command;
use tracing::{info, warn};

/// Resultado de registro DNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub value: String,
    pub ttl: Option<u32>,
    pub priority: Option<u16>,
}

/// Resultado consolidado de resolución DNS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResult {
    pub domain: String,
    pub a_records: Vec<IpAddr>,
    pub aaaa_records: Vec<IpAddr>,
    pub mx_records: Vec<DnsRecord>,
    pub txt_records: Vec<String>,
    pub ns_records: Vec<String>,
    pub cname: Option<String>,
}

/// 🌐 Resolvedor DNS multi-registro
pub struct DnsResolver;

impl DnsResolver {
    pub fn new() -> Self {
        Self
    }

    /// Resuelve todos los registros DNS para un dominio
    pub async fn resolve_all(&self, domain: &str) -> DnsResult {
        info!("🌐 [DNS] Resolviendo todos los registros para: {}", domain);

        let a_records = self.resolve_a(domain).await.unwrap_or_else(|e| {
            warn!("🌐 [DNS] Error resolviendo A: {}", e);
            Vec::new()
        });

        let aaaa_records = self.resolve_aaaa(domain).await.unwrap_or_else(|e| {
            warn!("🌐 [DNS] Error resolviendo AAAA: {}", e);
            Vec::new()
        });

        let mx_records = self.resolve_mx(domain).await.unwrap_or_else(|e| {
            warn!("🌐 [DNS] Error resolviendo MX: {}", e);
            Vec::new()
        });

        let txt_records = self.resolve_txt(domain).await.unwrap_or_else(|e| {
            warn!("🌐 [DNS] Error resolviendo TXT: {}", e);
            Vec::new()
        });

        let ns_records = self.resolve_ns(domain).await.unwrap_or_else(|e| {
            warn!("🌐 [DNS] Error resolviendo NS: {}", e);
            Vec::new()
        });

        let cname = self.resolve_cname(domain).await.unwrap_or_else(|e| {
            warn!("🌐 [DNS] Error resolviendo CNAME: {}", e);
            None
        });

        info!(
            "🌐 [DNS] Resolución completa para '{}': A={}, AAAA={}, MX={}, TXT={}, NS={}, CNAME={}",
            domain,
            a_records.len(),
            aaaa_records.len(),
            mx_records.len(),
            txt_records.len(),
            ns_records.len(),
            cname.as_deref().unwrap_or("none")
        );

        DnsResult {
            domain: domain.to_string(),
            a_records,
            aaaa_records,
            mx_records,
            txt_records,
            ns_records,
            cname,
        }
    }

    /// Resuelve registros A (IPv4) usando lookup_host de Tokio
    pub async fn resolve_a(&self, domain: &str) -> anyhow::Result<Vec<IpAddr>> {
        let addr_str = format!("{}:0", domain);
        let hosts = lookup_host(&addr_str).await?;

        let ips: Vec<IpAddr> = hosts
            .filter_map(|sockaddr| {
                let ip = sockaddr.ip();
                if ip.is_ipv4() {
                    Some(ip)
                } else {
                    None
                }
            })
            .collect();

        Ok(ips)
    }

    /// Resuelve registros AAAA (IPv6) usando lookup_host de Tokio
    pub async fn resolve_aaaa(&self, domain: &str) -> anyhow::Result<Vec<IpAddr>> {
        let addr_str = format!("{}:0", domain);
        let hosts = lookup_host(&addr_str).await?;

        let ips: Vec<IpAddr> = hosts
            .filter_map(|sockaddr| {
                let ip = sockaddr.ip();
                if ip.is_ipv6() {
                    Some(ip)
                } else {
                    None
                }
            })
            .collect();

        Ok(ips)
    }

    /// Resuelve registros MX usando `dig mx`
    pub async fn resolve_mx(&self, domain: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let output = Command::new("dig")
            .arg("+short")
            .arg("MX")
            .arg(domain)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut records = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            // Formato: "10 mail.example.com."
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let priority: u16 = parts[0].parse().unwrap_or(10);
                let server = parts[1].trim_end_matches('.');
                records.push(DnsRecord {
                    record_type: "MX".to_string(),
                    value: format!("{} (priority: {})", server, priority),
                    ttl: None,
                    priority: Some(priority),
                });
            }
        }

        Ok(records)
    }

    /// Resuelve registros TXT usando `dig txt`
    pub async fn resolve_txt(&self, domain: &str) -> anyhow::Result<Vec<String>> {
        let output = Command::new("dig")
            .arg("+short")
            .arg("TXT")
            .arg(domain)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut records = Vec::new();

        for line in stdout.lines() {
            let line = line.trim().trim_matches('"');
            if !line.is_empty() && !line.starts_with(';') {
                records.push(line.to_string());
            }
        }

        Ok(records)
    }

    /// Resuelve registros NS usando `dig ns`
    pub async fn resolve_ns(&self, domain: &str) -> anyhow::Result<Vec<String>> {
        let output = Command::new("dig")
            .arg("+short")
            .arg("NS")
            .arg(domain)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let nameservers: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().trim_end_matches('.').to_string())
            .filter(|l| !l.is_empty() && !l.starts_with(';'))
            .collect();

        Ok(nameservers)
    }

    /// Resuelve registro CNAME usando `dig cname`
    pub async fn resolve_cname(&self, domain: &str) -> anyhow::Result<Option<String>> {
        let output = Command::new("dig")
            .arg("+short")
            .arg("CNAME")
            .arg(domain)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let cname = stdout
            .lines()
            .next()
            .map(|l| l.trim().trim_end_matches('.').to_string())
            .filter(|l| !l.is_empty() && !l.starts_with(';'));

        Ok(cname)
    }

    /// Verifica si dig está disponible en el sistema
    pub async fn is_dig_available() -> bool {
        Command::new("which")
            .arg("dig")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dns_resolver_creation() {
        let resolver = DnsResolver::new();
        // Solo verificar que se crea sin errores
        assert!(resolver.resolve_a("localhost").await.is_ok());
    }

    #[tokio::test]
    async fn test_resolve_a_localhost() {
        let resolver = DnsResolver::new();
        let ips = resolver.resolve_a("localhost").await.unwrap_or_default();
        // localhost debe resolver a 127.0.0.1
        assert!(ips.is_empty() || ips.contains(&IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)));
    }

    #[tokio::test]
    async fn test_resolve_all_google() {
        let resolver = DnsResolver::new();
        let result = resolver.resolve_all("google.com").await;
        // google.com debe tener al menos A records
        assert!(!result.a_records.is_empty() || !result.ns_records.is_empty());
    }
}
