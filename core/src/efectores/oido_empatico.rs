// ──────────────────────────────────────────────
// 👂 OÍDO EMPÁTICO: Detección de tono y pausas naturales
// Migrado desde legacy/nexus-orquestador/src/tentaculos/oido_empatico.rs
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Tono {
    Alegre,
    Frustrado,
    Urgente,
    Normal,
    Preocupado,
}

/// Detector de tono emocional en mensajes de texto
pub struct TonoDetector;

impl TonoDetector {
    pub fn analizar(&self, mensaje: &str) -> Tono {
        let m = mensaje.to_lowercase();
        if m.contains('!') || m.contains("ayuda") || m.contains("urgente") {
            Tono::Urgente
        } else if m.contains("mal")
            || m.contains("error")
            || m.contains("falla")
            || m.contains("no funciona")
        {
            Tono::Frustrado
        } else if m.contains("bien") || m.contains("genial") || m.contains("gracias") {
            Tono::Alegre
        } else {
            Tono::Normal
        }
    }
}

/// Pausa natural antes de responder, ajustada al tono detectado
pub struct Pausa;

impl Pausa {
    pub async fn ajustar(&self, tono: Tono) {
        let ms = match tono {
            Tono::Urgente => 100,
            Tono::Frustrado => 800, // Pausa de empatía/reflexión
            Tono::Alegre => 300,
            _ => 500,
        };
        info!(
            "👂 [PAUSA NATURAL] Esperando {}ms para responder con naturalidad...",
            ms
        );
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

/// Oído que siente el tono emocional y pausa naturalmente
pub struct OidoEmpatico {
    pub tono_detector: TonoDetector,
    pub pausa_natural: Pausa,
}

impl Default for OidoEmpatico {
    fn default() -> Self {
        Self::new()
    }
}

impl OidoEmpatico {
    pub fn new() -> Self {
        Self {
            tono_detector: TonoDetector,
            pausa_natural: Pausa,
        }
    }

    pub async fn escuchar_y_sentir(&self, mensaje: &str) -> anyhow::Result<Tono> {
        let tono = self.tono_detector.analizar(mensaje);
        info!("👂 [OÍDO EMPÁTICO] Tono detectado: {:?}", tono);
        self.pausa_natural.ajustar(tono.clone()).await;
        Ok(tono)
    }
}
