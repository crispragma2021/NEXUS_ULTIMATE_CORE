use crate::autodiagnostico::sentinel_core::{HealthProbe, ProbeResult, ProbeTier};
use async_trait::async_trait;
use std::path::Path;
use std::time::Instant;
use tokio::fs;

pub struct ProbeFilesystem;

impl ProbeFilesystem {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HealthProbe for ProbeFilesystem {
    async fn check(&self) -> ProbeResult {
        let start_time = Instant::now();
        let mut passed = true;
        let mut mensaje = String::new();
        let mut detalles = serde_json::json!({});

        let critical_paths = vec![
            "core/src/",
            "src-tauri/src/main.rs",
            "dist/",
            "data/nexus_memoria.db",
            "index.html",
        ];

        for path_str in critical_paths {
            let path = Path::new(path_str);
            if !path.exists() {
                passed = false;
                mensaje.push_str(&format!("Ruta crítica no encontrada: {}. ", path_str));
                detalles[&path_str.replace('/', "_")] = serde_json::json!({ "exists": false });
            } else {
                detalles[&path_str.replace('/', "_")] = serde_json::json!({ "exists": true });
            }
        }

        // Check write permissions for data, logs, tmp
        let writable_paths = vec!["data/", "logs/", "/tmp/"];
        for path_str in writable_paths {
            let path = Path::new(path_str);
            if !path.exists() {
                // Try to create to check writability indirectly
                if let Err(e) = fs::create_dir_all(&path).await {
                    passed = false;
                    mensaje.push_str(&format!(
                        "No se pudo crear/escribir en ruta: {}. {}. ",
                        path_str, e
                    ));
                    detalles[&format!("{}_writable", path_str.replace('/', "_"))] =
                        serde_json::json!({ "writable": false, "error": e.to_string() });
                }
            } else {
                // Try to create a dummy file and delete it
                let test_file = path.join("nexus_write_test.tmp");
                if let Err(e) = fs::write(&test_file, "test").await {
                    passed = false;
                    mensaje.push_str(&format!(
                        "No se pudo escribir en ruta: {}. {}. ",
                        path_str, e
                    ));
                    detalles[&format!("{}_writable", path_str.replace('/', "_"))] =
                        serde_json::json!({ "writable": false, "error": e.to_string() });
                } else {
                    let _ = fs::remove_file(&test_file).await;
                    detalles[&format!("{}_writable", path_str.replace('/', "_"))] =
                        serde_json::json!({ "writable": true });
                }
            }
        }

        if mensaje.is_empty() {
            mensaje.push_str("Todas las rutas críticas y permisos OK.");
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
        "Filesystem Integrity"
    }
}
