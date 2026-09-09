// ==========================================
// 👁️ VISIÓN — Sistema visual del organismo
// ==========================================
// Re-exporta la implementación real de la visión anatómica (sentidos/)
// y define las señales hipotalámicas visuales.
// ==========================================

pub use crate::sentidos::omnipresent_vision::OmnipresentVision;

use super::reflex_arc::ReflexSignal;
use super::NeuralManager;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Señales del hipotálamo relacionadas con la visión.
#[derive(Debug, Clone, PartialEq)]
pub enum HypothalamusSignal {
    /// Nivel de luz azul detectado (regulación circadiana).
    BlueLightLevel(f32),
    /// Alerta de amenaza visual.
    ThreatAlert(String),
}

impl OmnipresentVision {
    /// Constructor compatible con el arranque (boot.rs): recibe el canal de
    /// reflejos y el gestor neural. Devuelve una instancia nueva.
    pub fn new(
        _reflex_tx: Option<mpsc::Sender<ReflexSignal>>,
        _neural: Option<Arc<NeuralManager>>,
    ) -> Self {
        Self::default()
    }
}

impl Default for OmnipresentVision {
    fn default() -> Self {
        // La instancia global se obtiene vía instance(); este Default solo
        // existe para satisfacer constructores que requieren Self.
        Self {
            activo: false,
            ultimo_frame_b64: None,
            ultimo_texto_ocr: None,
            frames_capturados: 0,
        }
    }
}
