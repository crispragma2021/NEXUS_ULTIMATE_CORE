use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

pub struct OSControlManager;

impl OSControlManager {
    /// Captura la pantalla del escritorio o de una URL
    pub fn capturar_pantalla(objetivo: Option<&str>) -> Result<PathBuf> {
        let obj = objetivo.unwrap_or("escritorio");
        crate::vision::VisionManager::capturar_pantalla(obj)
    }

    /// Ejecuta un clic del mouse en las coordenadas (X, Y) del escritorio
    pub fn clic_escritorio(x: u32, y: u32) -> Result<String> {
        info!("🖱️ [NEXUS-OS] Ejecutando clic en X={}, Y={}", x, y);
        let status = Command::new("./scripts/nexus_os_control.sh")
            .arg("click")
            .arg(x.to_string())
            .arg(y.to_string())
            .status()
            .or_else(|_| {
                Command::new("xdotool")
                    .arg("mousemove")
                    .arg(x.to_string())
                    .arg(y.to_string())
                    .arg("click")
                    .arg("1")
                    .status()
            })?;

        if status.success() {
            Ok(format!("Clic exitoso en X={}, Y={}", x, y))
        } else {
            Err(anyhow!("Fallo al ejecutar clic en X={}, Y={}", x, y))
        }
    }

    /// Inyecta texto en la ventana o campo activo del escritorio
    pub fn escribir_escritorio(texto: &str) -> Result<String> {
        info!("⌨️ [NEXUS-OS] Inyectando texto en teclado: {}", texto);
        let status = Command::new("./scripts/nexus_os_control.sh")
            .arg("escribir")
            .arg(texto)
            .status()
            .or_else(|_| {
                Command::new("xdotool")
                    .arg("type")
                    .arg("--delay")
                    .arg("50")
                    .arg(texto)
                    .status()
            })?;

        if status.success() {
            Ok(format!("Texto enviado exitosamente: '{}'", texto))
        } else {
            Err(anyhow!("Fallo al inyectar texto"))
        }
    }

    /// Envía una tecla o combinación de teclas al escritorio
    pub fn tecla_escritorio(tecla: &str) -> Result<String> {
        info!("🔤 [NEXUS-OS] Enviando pulsación de tecla: {}", tecla);
        let status = Command::new("./scripts/nexus_os_control.sh")
            .arg("tecla")
            .arg(tecla)
            .status()
            .or_else(|_| {
                Command::new("xdotool")
                    .arg("key")
                    .arg(tecla)
                    .status()
            })?;

        if status.success() {
            Ok(format!("Tecla enviada exitosamente: '{}'", tecla))
        } else {
            Err(anyhow!("Fallo al enviar tecla '{}'", tecla))
        }
    }

    /// Ejecuta un comando ADB en un dispositivo Android conectado
    pub fn ejecutar_adb(accion: &str, params: &[&str]) -> Result<String> {
        info!("📱 [NEXUS-ADB] Ejecutando acción ADB: {} con parámetros: {:?}", accion, params);
        let mut cmd = Command::new("adb");

        match accion {
            "dispositivos" => {
                cmd.arg("devices");
            }
            "tap" => {
                if params.len() < 2 {
                    return Err(anyhow!("Acción 'tap' requiere parámetros X e Y"));
                }
                cmd.args(["shell", "input", "tap", params[0], params[1]]);
            }
            "swipe" => {
                if params.len() < 4 {
                    return Err(anyhow!("Acción 'swipe' requiere x1 y1 x2 y2"));
                }
                cmd.args(["shell", "input", "swipe", params[0], params[1], params[2], params[3]]);
            }
            "escribir" => {
                if params.is_empty() {
                    return Err(anyhow!("Acción 'escribir' requiere el texto a enviar"));
                }
                cmd.args(["shell", "input", "text", params[0]]);
            }
            "capturar" => {
                let dest = params.first().copied().unwrap_or("/tmp/adb_screenshot.png");
                let output = Command::new("adb")
                    .args(["exec-out", "screencap", "-p"])
                    .output()?;

                if output.status.success() {
                    std::fs::write(dest, output.stdout)?;
                    return Ok(format!("Captura de pantalla de Android guardada en {}", dest));
                } else {
                    return Err(anyhow!("Error al capturar pantalla ADB"));
                }
            }
            "shell" => {
                cmd.arg("shell");
                for p in params {
                    cmd.arg(p);
                }
            }
            _ => {
                return Err(anyhow!("Acción ADB no soportada: {}", accion));
            }
        }

        let output = cmd.output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(if stdout.trim().is_empty() {
                format!("Acción ADB '{}' ejecutada correctamente", accion)
            } else {
                stdout
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow!("Error ADB: {}", stderr))
        }
    }
}
