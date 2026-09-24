use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

pub struct VisionManager;

impl VisionManager {
    /// Captura pantalla de una URL o del escritorio actual
    pub fn capturar_pantalla(objetivo: &str) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let output_dir = Path::new("/tmp/nexus_health");
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let output_path = output_dir.join(format!("screenshot_{}.png", timestamp));

        if objetivo.starts_with("http://") || objetivo.starts_with("https://") {
            info!("📸 [NEXUS-VISION] Capturando URL vía Motor Unificado Playwright: {}", objetivo);
            let node_path = std::env::var("NODE_PATH")
                .unwrap_or_else(|_| "/home/nexus/.hermes/hermes-agent/node_modules".to_string());

            let status = Command::new("node")
                .env("NODE_PATH", &node_path)
                .arg("./scripts/nexus_browser_engine.cjs")
                .arg("navegar")
                .arg(objetivo)
                .arg(&output_path)
                .status()
                .or_else(|_| {
                    Command::new("node")
                        .env("NODE_PATH", &node_path)
                        .arg("./take_screenshot.cjs")
                        .arg(objetivo)
                        .arg(&output_path)
                        .status()
                })?;

            if !status.success() {
                return Err(anyhow!("Fallo al capturar URL con Playwright"));
            }
        } else {
            info!("📸 [NEXUS-VISION] Capturando Escritorio de Linux en Tiempo Real (Trigger Nativo GNOME / Wayland)...");
            
            // Disparar atajo de teclado nativo de GNOME para captura en tiempo real instante (Super + Print)
            let _ = Command::new("xdotool")
                .arg("key")
                .arg("Super+Print")
                .status();

            std::thread::sleep(std::time::Duration::from_millis(400));

            let pictures_dir = Path::new("/home/nexus/Imágenes/Capturas de pantalla");
            let mut latest_fallback: Option<PathBuf> = None;

            if pictures_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(pictures_dir) {
                    let mut latest_file: Option<(PathBuf, std::time::SystemTime)> = None;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
                            if let Ok(meta) = entry.metadata() {
                                if let Ok(modified) = meta.modified() {
                                    if latest_file.as_ref().map_or(true, |(_, last_mod)| modified > *last_mod) {
                                        latest_file = Some((path, modified));
                                    }
                                }
                            }
                        }
                    }
                    
                    if let Some((latest_path, mod_time)) = latest_file {
                        latest_fallback = Some(latest_path.clone());
                        if let Ok(elapsed) = mod_time.elapsed() {
                            // Si la captura de GNOME fue tomada recién (hace menos de 10s), usarla de inmediato
                            if elapsed.as_secs() < 10 {
                                info!("📸 [NEXUS-VISION] Captura real-time en tiempo real obtenida: {:?}", latest_path);
                                std::fs::copy(&latest_path, &output_path)?;
                                return Ok(output_path);
                            }
                        }
                    }
                }
            }

            // Fallback con ffmpeg / gnome-screenshot / grim
            let status = Command::new("ffmpeg")
                .arg("-y")
                .arg("-f")
                .arg("x11grab")
                .arg("-i")
                .arg(":0.0")
                .arg("-vframes")
                .arg("1")
                .arg(&output_path)
                .status()
                .or_else(|_| {
                    Command::new("gnome-screenshot")
                        .arg("-f")
                        .arg(&output_path)
                        .status()
                })
                .or_else(|_| {
                    Command::new("grim")
                        .arg(&output_path)
                        .status()
                });

            let success = status.map(|s| s.success()).unwrap_or(false);

            if !success || output_path.metadata().map(|m| m.len()).unwrap_or(0) < 15000 {
                if let Some(fallback_path) = latest_fallback {
                    info!("📸 [NEXUS-VISION] Usando fallback de captura nativa GNOME: {:?}", fallback_path);
                    std::fs::copy(&fallback_path, &output_path)?;
                    return Ok(output_path);
                }
                return Err(anyhow!("No se pudo encontrar una captura de pantalla válida"));
            }
        }

        Ok(output_path)
    }

    /// Prepara la imagen para ser analizada por el modelo multimodal
    pub fn preparar_base64(imagen_path: &Path) -> Result<String> {
        if !imagen_path.exists() {
            return Err(anyhow!("El archivo de imagen no existe: {:?}", imagen_path));
        }

        let bytes = std::fs::read(imagen_path)?;
        use base64::Engine;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(b64)
    }
}
