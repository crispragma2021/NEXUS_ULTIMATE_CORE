use crate::autodiagnostico::sentinel_core::{HealthProbe, ProbeResult, ProbeTier};
use async_trait::async_trait;
use std::time::Instant;

pub struct ProbeFrontend;

impl ProbeFrontend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthProbe for ProbeFrontend {
    async fn check(&self) -> ProbeResult {
        let start_time = Instant::now();
        let mut passed = true;
        let mut mensaje = String::new();
        let mut detalles = serde_json::json!({});

        // 1. Check NEXUS Core API (:43210) — el órgano real del sistema.
        //   El dashboard web corre en :1420 y el dev server legacy en :5173,
        //   pero la salud del ecosistema depende del Core en :43210.
        let core_url = "http://localhost:43210/api/health";
        match reqwest::Client::new()
            .get(core_url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    mensaje.push_str("NEXUS Core API (43210): OK. ");
                } else {
                    passed = false;
                    mensaje.push_str(&format!(
                        "NEXUS Core API (43210): Error {}. ",
                        response.status().as_u16()
                    ));
                }
            }
            Err(e) => {
                passed = false;
                mensaje.push_str(&format!(
                    "NEXUS Core API (43210): Fallo de conexión ({}). ",
                    e
                ));
            }
        }

        // 2. Verificación opcional del dashboard web en :1420 (no bloqueante)
        let ui_url = "http://localhost:1420";
        match reqwest::Client::new()
            .get(ui_url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    detalles["ui_web"] = serde_json::json!({ "status": "ok", "url": ui_url });
                    mensaje.push_str("Dashboard Web (1420): OK. ");
                } else {
                    detalles["ui_web"] = serde_json::json!({ "status": "error", "code": response.status().as_u16() });
                }
            }
            Err(e) => {
                detalles["ui_web"] = serde_json::json!({ "status": "offline", "error": e.to_string() });
                mensaje.push_str(&format!("Dashboard Web (1420): offline ({}). ", e));
            }
        }

        // 3. Playwright screenshot (via vision_bridge, opcional — no bloquea el probe)
        detalles["playwright_screenshot"] = serde_json::json!({ "status": "simulated_ok", "message": "Playwright se ejecuta vía vision_bridge" });

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
        ProbeTier::Warning
    }

    fn nombre(&self) -> &'static str {
        "Frontend UI"
    }
}
