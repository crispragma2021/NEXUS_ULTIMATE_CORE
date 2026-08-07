// ==========================================
// VISION OMEGA — DEPRECADO (Stub de compatibilidad)
// ==========================================
// Este módulo era un duplicado de OmnipresentVision.
// Mantenido como stub para compatibilidad con código legacy que importa:
//   crate::sentidos::vision_omega::VisionOmega
//
// Usado por:
//   - CerebroNativo (ia_nativa.rs) como `self.vision.capturar_escritorio()`
//   - Constructor del Orquestador (constructor.rs)

use crate::sentidos::omnipresent_vision::OmnipresentVision;

/// 👁️ Vision Omega — Stub de compatibilidad
/// Re-exporta OmnipresentVision internamente como bridge legacy.
pub struct VisionOmega;

impl VisionOmega {
    pub fn new() -> Self {
        Self
    }

    /// Captura el escritorio completo usando OmnipresentVision internamente.
    /// Método de instancia para compatibilidad con CerebroNativo.
    pub async fn capturar_escritorio(&self) -> Option<Vec<u8>> {
        OmnipresentVision::capturar_para_modelo_local(1920, 1080).await
    }

    /// Captura con resolución personalizada.
    pub async fn capturar_con_resolucion(&self, width: u32, height: u32) -> Option<Vec<u8>> {
        OmnipresentVision::capturar_para_modelo_local(width, height).await
    }
}
