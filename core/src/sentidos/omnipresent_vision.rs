use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{imageops::FilterType, DynamicImage};
use once_cell::sync::Lazy;
use std::io::Cursor;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use xcap::Monitor;

/// 👁️ EL OJO QUE TODO LO VE: Omnipresent Vision
/// Unifica la velocidad de xcap con la inteligencia de VisionOmega.
pub struct OmnipresentVision {
    pub activo: bool,
    pub ultimo_frame_b64: Option<String>,
    pub ultimo_texto_ocr: Option<String>,
    pub frames_capturados: u64,
}

static VISION_INSTANCE: Lazy<Arc<RwLock<OmnipresentVision>>> = Lazy::new(|| {
    Arc::new(RwLock::new(OmnipresentVision {
        activo: false,
        ultimo_frame_b64: None,
        ultimo_texto_ocr: None,
        frames_capturados: 0,
    }))
});

impl OmnipresentVision {
    /// Obtener la instancia global del Ojo
    pub fn instance() -> Arc<RwLock<Self>> {
        VISION_INSTANCE.clone()
    }

    /// Captura absoluta de todos los monitores y procesamiento inteligente
    pub async fn capturar_y_procesar() -> anyhow::Result<()> {
        let start = Instant::now();
        let monitors = Monitor::all().map_err(|e| anyhow::anyhow!("Fallo xcap: {}", e))?;

        if let Some(monitor) = monitors.first() {
            let image = monitor
                .capture_image()
                .map_err(|e| anyhow::anyhow!("Error captura: {}", e))?;

            // 1. Convertir a Base64 para el HUD
            let mut buffer = std::io::Cursor::new(Vec::new());
            image.write_to(&mut buffer, image::ImageFormat::Png)?;
            let b64 = STANDARD.encode(buffer.get_ref());

            // 2. Persistir temporalmente para OCR si es necesario
            let temp_path = "/tmp/nexus_vision_latest.png";
            image.save(temp_path)?;

            // 3. OCR Real (Legado de VisionOmega)
            let ocr_text = Self::ejecutar_ocr(temp_path).await;

            // 4. Actualizar Estado
            let mut eye = VISION_INSTANCE.write().await;
            eye.ultimo_frame_b64 = Some(b64);
            eye.ultimo_texto_ocr = ocr_text;
            eye.frames_capturados += 1;

            info!(
                "👁️ [VISION] Frame #{} procesado en {:?}",
                eye.frames_capturados,
                start.elapsed()
            );
        }

        Ok(())
    }

    /// Ejecuta Tesseract OCR sobre el frame capturado
    async fn ejecutar_ocr(path: &str) -> Option<String> {
        let output = Command::new("tesseract")
            .arg(path)
            .arg("stdout")
            .arg("-l")
            .arg("spa+eng") // Soporte bilingüe para el búnker
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let text = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
                None
            }
            _ => {
                warn!("⚠️ [VISION] Tesseract no respondió correctamente o no está instalado.");
                None
            }
        }
    }

    /// Activar el bucle de vigilancia visual
    pub async fn iniciar_vigilancia(fps: u64) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / fps));
        {
            let mut eye = VISION_INSTANCE.write().await;
            eye.activo = true;
        }

        tokio::spawn(async move {
            info!("👁️ [VISION] Bucle de vigilancia OMEGA iniciado.");
            loop {
                interval.tick().await;
                let activo = { VISION_INSTANCE.read().await.activo };
                if !activo {
                    break;
                }

                if let Err(e) = Self::capturar_y_procesar().await {
                    error!("❌ [VISION] Error en bucle: {}", e);
                }
            }
            info!("👁️ [VISION] Bucle de vigilancia finalizado.");
        });
    }

    pub async fn detener() {
        let mut eye = VISION_INSTANCE.write().await;
        eye.activo = false;
    }

    /// [FUSIÓN]: Captura y escala la imagen para modelos de visión locales (ex: 378x378 para Moondream)
    /// Succionado de VisionOmega. Ocurre 100% en RAM.
    pub async fn capturar_para_modelo_local(width: u32, height: u32) -> Option<Vec<u8>> {
        let monitors = Monitor::all().unwrap_or_default();
        if let Some(monitor) = monitors.first() {
            if let Ok(image) = monitor.capture_image() {
                let dynamic_img = DynamicImage::ImageRgba8(image);
                let resized = dynamic_img.resize_exact(width, height, FilterType::Lanczos3);

                let mut buffer = Vec::new();
                let mut cursor = Cursor::new(&mut buffer);
                if resized
                    .write_to(&mut cursor, image::ImageFormat::Png)
                    .is_ok()
                {
                    info!(
                        "👁️ [VISION] Imagen normalizada a {}x{} para inferencia local.",
                        width, height
                    );
                    return Some(buffer);
                }
            }
        }
        None
    }
}
