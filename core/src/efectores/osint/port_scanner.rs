// ──────────────────────────────────────────────
// 🔌 PORT SCANNER — Escaneo de puertos TCP básico
// Escanea puertos comunes vía TCP connect (tokio::net::TcpStream)
// No requiere nmap ni dependencias externas
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{info, warn};

/// Puerto abierto encontrado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPort {
    pub port: u16,
    pub protocol: String,
    pub service: String,
    pub state: String,
}

/// 🔌 Escáner de puertos TCP
pub struct PortScanner {
    timeout_secs: u64,
    concurrency: usize,
}

/// Puertos comunes con servicios asociados
const COMMON_PORTS: &[(u16, &str)] = &[
    (20, "FTP-data"),
    (21, "FTP"),
    (22, "SSH"),
    (23, "Telnet"),
    (25, "SMTP"),
    (53, "DNS"),
    (80, "HTTP"),
    (110, "POP3"),
    (111, "RPC"),
    (135, "MSRPC"),
    (139, "NetBIOS"),
    (143, "IMAP"),
    (443, "HTTPS"),
    (445, "SMB"),
    (465, "SMTPS"),
    (514, "Syslog"),
    (587, "SMTP Submission"),
    (593, "HTTP RPC"),
    (636, "LDAPS"),
    (993, "IMAPS"),
    (995, "POP3S"),
    (1080, "SOCKS5"),
    (1433, "MSSQL"),
    (1521, "Oracle DB"),
    (2049, "NFS"),
    (2082, "cPanel"),
    (2083, "cPanel SSL"),
    (2222, "DirectAdmin"),
    (2375, "Docker API"),
    (2376, "Docker API SSL"),
    (3306, "MySQL"),
    (3389, "RDP"),
    (3690, "SVN"),
    (4333, "MySQL Alt"),
    (4444, "Metasploit"),
    (5060, "SIP"),
    (5222, "XMPP"),
    (5432, "PostgreSQL"),
    (5900, "VNC"),
    (5901, "VNC 1"),
    (5984, "CouchDB"),
    (5985, "WinRM HTTP"),
    (5986, "WinRM HTTPS"),
    (6379, "Redis"),
    (8080, "HTTP Proxy"),
    (8081, "HTTP Alt"),
    (8443, "HTTPS Alt"),
    (8888, "HTTP Alt"),
    (9000, "Phoenix"),
    (9092, "Kafka"),
    (9200, "Elasticsearch"),
    (9300, "Elasticsearch Transport"),
    (9418, "Git"),
    (11211, "Memcached"),
    (27017, "MongoDB"),
    (27018, "MongoDB Web"),
];

impl Default for PortScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PortScanner {
    pub fn new() -> Self {
        Self {
            timeout_secs: 3,
            concurrency: 20,
        }
    }

    /// Configura timeout para cada conexión
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Configura concurrencia (máximo de conexiones paralelas)
    pub fn with_concurrency(mut self, max: usize) -> Self {
        self.concurrency = max;
        self
    }

    /// Escanea puertos comunes en una IP
    pub async fn scan_common(&self, ip: &str) -> Vec<OpenPort> {
        info!("🔌 [PORT-SCANNER] Escaneando puertos comunes en {}...", ip);

        let ports_to_scan: Vec<(u16, &str)> = COMMON_PORTS.to_vec();
        let results = self.scan_ports(ip, &ports_to_scan).await;

        info!(
            "🔌 [PORT-SCANNER] Escaneo completo en {}. {} puertos abiertos de {}",
            ip,
            results.len(),
            ports_to_scan.len()
        );

        results
    }

    /// Escanea puertos específicos
    pub async fn scan_ports(&self, ip: &str, ports: &[(u16, &str)]) -> Vec<OpenPort> {
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.concurrency));
        let mut handles = Vec::new();

        for (port, service) in ports {
            let ip = ip.to_string();
            let sem = semaphore.clone();
            let port = *port;
            let service = service.to_string();
            let timeout_secs = self.timeout_secs;

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await;
                scan_single_port(&ip, port, &service, timeout_secs).await
            });

            handles.push(handle);
        }

        let mut open_ports = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                if let Some(port) = result {
                    open_ports.push(port);
                }
            }
        }

        // Ordenar por número de puerto
        open_ports.sort_by(|a, b| a.port.cmp(&b.port));
        open_ports
    }

    /// Retorna la lista de puertos comunes (para referencia)
    pub fn common_ports_list() -> &'static [(u16, &'static str)] {
        COMMON_PORTS
    }
}

/// Escanea un solo puerto
async fn scan_single_port(
    ip: &str,
    port: u16,
    service: &str,
    timeout_secs: u64,
) -> Option<OpenPort> {
    let addr = format!("{}:{}", ip, port);

    match timeout(Duration::from_secs(timeout_secs), TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => {
            // Conexión exitosa → puerto abierto
            Some(OpenPort {
                port,
                protocol: "tcp".to_string(),
                service: service.to_string(),
                state: "open".to_string(),
            })
        }
        _ => None, // Puerto cerrado, filtrado o timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_scanner_creation() {
        let scanner = PortScanner::new();
        assert_eq!(scanner.timeout_secs, 3);
        assert_eq!(scanner.concurrency, 20);
    }

    #[test]
    fn test_common_ports_count() {
        assert_eq!(COMMON_PORTS.len(), 56);
    }

    #[test]
    fn test_common_ports_contains_ssh() {
        let ports = PortScanner::common_ports_list();
        assert!(ports.contains(&(22, "SSH")));
        assert!(ports.contains(&(80, "HTTP")));
        assert!(ports.contains(&(443, "HTTPS")));
        assert!(ports.contains(&(3306, "MySQL")));
    }

    #[tokio::test]
    async fn test_scan_common_ports_localhost() {
        let scanner = PortScanner::new();
        let results = scanner.scan_common("127.0.0.1").await;
        // No esperamos puertos abiertos específicos, solo que la función no crashee
        // Esto prueba que el scan funciona sin errores
    }
}
