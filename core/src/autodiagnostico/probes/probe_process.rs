use crate::autodiagnostico::sentinel_core::{HealthProbe, ProbeResult, ProbeTier};
use async_trait::async_trait;
use std::time::Instant;
use tokio::process::Command;

pub struct ProbeProcess;

impl ProbeProcess {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthProbe for ProbeProcess {
    async fn check(&self) -> ProbeResult {
        let start_time = Instant::now();
        let mut passed = true;
        let mut mensaje = String::new();
        let mut detalles = serde_json::json!({});

        // Ruta real del gestor de servicios (relativa al workspace).
        // Antes usaba ./scripts/service_manager.sh (ruta inexistente) y buscaba
        // servicios placeholder ("nexus-backend") que jamás existieron.
        let script_path = "scripts/services/service_manager.sh";
        let output = Command::new("bash")
            .arg(script_path)
            .arg("list")
            .output()
            .await;

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                let stderr = String::from_utf8_lossy(&o.stderr);
                detalles["stdout"] = serde_json::json!(stdout.to_string());
                detalles["stderr"] = serde_json::json!(stderr.to_string());

                // El gestor debe existir y responder. Verificamos que `list` devuelva
                // contenido (servicios ACTIVO/CORRIENDO) y exit status 0.
                if o.status.success() {
                    let activos = stdout
                        .lines()
                        .filter(|l| {
                            let upper = l.to_ascii_uppercase();
                            upper.contains("ACTIVO")
                                || upper.contains("RUNNING")
                                || upper.contains("CORRIENDO")
                        })
                        .count();
                    detalles["servicios_activos"] = serde_json::json!(activos);
                    if activos > 0 {
                        mensaje.push_str(&format!("{} servicios críticos activos. ", activos));
                    } else {
                        // Sin servicios registrados no es fallo: el gestor opera bien.
                        mensaje.push_str("Gestor de servicios operativo (sin servicios registrados). ");
                    }
                } else {
                    passed = false;
                    mensaje.push_str(&format!(
                        "service_manager.sh list devolvió status no exitoso. stderr: {}. ",
                        stderr.trim()
                    ));
                }
            }
            Err(e) => {
                passed = false;
                mensaje.push_str(&format!("Fallo al ejecutar service_manager.sh: {}. ", e));
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
        ProbeTier::Critical
    }

    fn nombre(&self) -> &'static str {
        "Critical Processes"
    }
}
