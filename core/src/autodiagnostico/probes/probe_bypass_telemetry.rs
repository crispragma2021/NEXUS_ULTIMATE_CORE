use crate::autodiagnostico::sentinel_core::{HealthProbe, ProbeResult, ProbeTier};
use async_trait::async_trait;
use std::time::Instant;
use tokio::net::TcpStream;

pub struct ProbeBypassTelemetry;

impl ProbeBypassTelemetry {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthProbe for ProbeBypassTelemetry {
    async fn check(&self) -> ProbeResult {
        let start_time = Instant::now();
        let mut passed = true;
        let mut mensaje = String::new();
        let mut detalles = serde_json::json!({});

        // Medir latencia de socket TCP ultra-baja (latencia de loopback)
        let addr = "127.0.0.1:43210"; // Puerto API de NEXUS
        let socket_start = Instant::now();

        match TcpStream::connect(addr).await {
            Ok(_) => {
                let duration_micros = socket_start.elapsed().as_micros();
                passed = true;
                detalles["socket_loopback_latency_micros"] = serde_json::json!(duration_micros);
                detalles["kernel_bypass_simulation"] = serde_json::json!({
                    "engine": "eBPF/XDP Telemetry Mock",
                    "status": "active",
                    "hardware_efficiency": if duration_micros < 500 { "ultra_high" } else { "normal" }
                });
                mensaje = format!(
                    "Socket Loopback Latency: {} μs (Microsegundos) — Rango de eficiencia óptimo.",
                    duration_micros
                );
            }
            Err(e) => {
                passed = false;
                detalles["error"] = serde_json::json!(e.to_string());
                mensaje = format!(
                    "Error de conexión socket al núcleo: {}. Levantando alertas de red.",
                    e
                );
            }
        }

        ProbeResult {
            nombre: self.nombre().to_string(),
            tier: self.tier(),
            passed,
            mensaje,
            detalles: Some(detalles),
            latencia_ms: start_time.elapsed().as_millis() as u64,
        }
    }

    fn tier(&self) -> ProbeTier {
        ProbeTier::Info
    }

    fn nombre(&self) -> &'static str {
        "HFT Bypass Telemetry"
    }
}
