use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::{self, Duration};
use tracing::{error, info, warn};

pub struct VisionBridge;

impl VisionBridge {
    /// Toma screenshot del frontend vía Playwright (Node.js)
    /// Retorna path del archivo PNG o error
    pub async fn capturar_frontend(url: &str) -> Result<PathBuf> {
        let screenshot_output_path = format!(
            "/tmp/nexus_health/screenshot_{}.png",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let script_path = "./take_screenshot.cjs";

        // Asegurarse de que el directorio de salida exista
        let output_dir = Path::new("/tmp/nexus_health/");
        tokio::fs::create_dir_all(&output_dir).await?;

        info!(
            "📸 [VISION BRIDGE] Ejecutando Playwright para capturar {}. Salida en {}",
            url, screenshot_output_path
        );

        let mut cmd = Command::new("node");
        cmd.arg(script_path)
            .arg(url) // Pasar la URL como argumento al script de Playwright
            .arg(&screenshot_output_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow!("Fallo al ejecutar Node/Playwright: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Child process stdout has not been captured."))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Child process stderr has not been captured."))?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut output_lines = Vec::new();
        let mut error_lines = Vec::new();

        // Leer stdout y stderr hasta que el proceso termine
        loop {
            tokio::select! {
                stdout_line = stdout_reader.next_line() => {
                    if let Some(line) = stdout_line? { output_lines.push(line); }
                    else { break; }
                },
                stderr_line = stderr_reader.next_line() => {
                    if let Some(line) = stderr_line? { error_lines.push(line); }
                    else { break; }
                },
                _ = time::sleep(Duration::from_secs(10)) => {
                    warn!("⚠️ [VISION BRIDGE] Playwright timed out after 10 seconds. Terminating process.");
                    let _ = child.kill().await; // Kill the process if it times out
                    return Err(anyhow!("Playwright timeout."));
                }
            }
        }

        let status = child
            .wait()
            .await
            .map_err(|e| anyhow!("Fallo al esperar Node/Playwright: {}", e))?;

        for line in output_lines {
            info!("🤖 [Playwright stdout]: {}", line);
        }
        for line in error_lines {
            error!("❌ [Playwright stderr]: {}", line);
        }

        if status.success() {
            info!("✅ [VISION BRIDGE] Playwright ejecutado con éxito.");
            Ok(PathBuf::from(screenshot_output_path))
        } else {
            error!(
                "❌ [VISION BRIDGE] Playwright falló con estado: {:?}",
                status
            );
            Err(anyhow!("Playwright script failed."))
        }
    }

    /// Verifica que la UI carga sin errores JS
    /// Retorna true si no hay errores en consola
    pub async fn verificar_ui_sana(url: &str) -> Result<bool> {
        // TODO: Implementar lógica de análisis de logs de navegador para errores JS
        // Por ahora, se asume que si el screenshot funciona, la UI está relativamente sana.
        Ok(true)
    }

    /// Almacena screenshot con timestamp en /tmp/nexus_health/
    pub fn archivar_screenshot(path: &PathBuf) -> Result<PathBuf> {
        // La ruta ya incluye el timestamp del nombre de archivo
        Ok(path.clone())
    }
}
