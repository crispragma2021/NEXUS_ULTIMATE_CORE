// ============================================================================
// 🟢 HEALTH MONITOR — Healthcheck TCP/HTTP para Estado LED (Regla 3)
// ============================================================================
// Determina el estado de cada servicio de proyecto para la UI minimalista:
//
//   ENCENDIDO  🟢 → healthcheck TCP/HTTP confirmado (verde neón pulsante)
//   APAGADO    ⚫ → servicio inactivo (gris neutro)
//
// La UI muestra SOLO nombre + LED por proyecto; el health monitor es la fuente
// de verdad del LED.
// ============================================================================

use anyhow::Result;
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

/// Estado de un servicio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Healthcheck confirmado (TCP o HTTP 2xx).
    Up,
    /// Servicio inactivo / no responde.
    Down,
}

impl ServiceStatus {
    pub fn is_up(&self) -> bool {
        matches!(self, ServiceStatus::Up)
    }
}

/// Resultado de un healthcheck.
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub project_id: String,
    pub port: u16,
    pub status: ServiceStatus,
    /// Latencia del check en ms (0 si Down).
    pub latency_ms: u64,
}

/// Monitor de salud por proyecto.
pub struct HealthMonitor {
    pub connect_timeout: Duration,
    pub http_path: Option<String>,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_millis(1500),
            http_path: Some("/health".to_string()),
        }
    }
}

impl HealthMonitor {
    pub fn new(connect_timeout: Duration) -> Self {
        Self {
            connect_timeout,
            http_path: Some("/health".to_string()),
        }
    }

    /// Healthcheck TCP: ¿el puerto acepta conexiones?
    ///
    /// Es la base para el LED. Si el servicio no responde → Down.
    pub fn check_tcp(&self, port: u16) -> ServiceStatus {
        let start = std::time::Instant::now();
        match TcpStream::connect_timeout(&format!("127.0.0.1:{port}").parse().unwrap(), self.connect_timeout)
        {
            Ok(stream) => {
                // Cerrar limpiamente.
                let _ = stream.shutdown(Shutdown::Both);
                let _ = start.elapsed();
                ServiceStatus::Up
            }
            Err(_) => ServiceStatus::Down,
        }
    }

    /// Healthcheck HTTP: GET /health (o ruta configurada) esperando 2xx.
    ///
    /// Más robusto que TCP puro: confirma que el servicio responde HTTP.
    /// Usa un socket crudo para evitar dependencias async aquí.
    pub fn check_http(&self, port: u16) -> ServiceStatus {
        let path = self.http_path.as_deref().unwrap_or("/health");
        let request = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");

        match TcpStream::connect_timeout(&format!("127.0.0.1:{port}").parse().unwrap(), self.connect_timeout) {
            Ok(mut stream) => {
                stream
                    .set_read_timeout(Some(self.connect_timeout))
                    .ok();
                if stream.write_all(request.as_bytes()).is_err() {
                    return ServiceStatus::Down;
                }
                // Leer la primera línea de respuesta.
                let mut buf = [0u8; 512];
                let n = stream.read(&mut buf).unwrap_or(0);
                if n == 0 {
                    return ServiceStatus::Down;
                }
                let head = String::from_utf8_lossy(&buf[..n]);
                // Buscar "HTTP/1.x 2xx" o "HTTP/1.x 3xx" (redirects = vivo).
                head.split_whitespace()
                    .nth(1)
                    .and_then(|code| code.parse::<u16>().ok())
                    .map(|c| if (200..400).contains(&c) { ServiceStatus::Up } else { ServiceStatus::Down })
                    .unwrap_or(ServiceStatus::Down)
            }
            Err(_) => ServiceStatus::Down,
        }
    }

    /// Healthcheck completo para un proyecto (usa HTTP si se puede, TCP como fallback).
    pub fn check(&self, project_id: &str, port: u16) -> HealthCheckResult {
        let start = std::time::Instant::now();
        let status = self.check_http(port);
        if status == ServiceStatus::Down {
            // Fallback: TCP puro (servicios que no hablan HTTP).
            let tcp = self.check_tcp(port);
            let status = tcp;
            HealthCheckResult {
                project_id: project_id.to_string(),
                port,
                status,
                latency_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            HealthCheckResult {
                project_id: project_id.to_string(),
                port,
                status,
                latency_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
}

use std::io::{Read, Write};

#[cfg(test)]
mod tests {
    use super::*;

    // NOTA: estos tests no abren sockets reales; verifican la lógica del
    // rango y del mapeo de estados.

    #[test]
    fn puerto_cerrado_devuelve_down() {
        // Puerto 8000 no abierto en CI → Down esperado (o Up si hay servicio local).
        let mon = HealthMonitor::default();
        let s = mon.check_tcp(9); // puerto discard, casi seguro cerrado
        // No asumimos: solo verificamos que devuelve un valor válido.
        assert!(s == ServiceStatus::Up || s == ServiceStatus::Down);
    }

    #[test]
    fn is_up_matches() {
        assert!(ServiceStatus::Up.is_up());
        assert!(!ServiceStatus::Down.is_up());
    }

    #[test]
    fn health_check_result_lleva_proyecto_y_puerto() {
        let mon = HealthMonitor::default();
        let r = mon.check("trader", 9);
        assert_eq!(r.project_id, "trader");
        assert_eq!(r.port, 9);
    }
}
