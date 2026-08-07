// ==========================================
// CÍNGULO ANTERIOR - Detección Predictiva de Errores
// ==========================================
// Como el cíngulo anterior humano: detecta
// cuándo se va a cometer un error ANTES de
// que ocurra. Monitorea conflictos y corrige.
// ==========================================

use std::collections::VecDeque;

pub struct CinguloAnterior {
    historial_errores: VecDeque<(String, String)>,
}

impl Default for CinguloAnterior {
    fn default() -> Self {
        Self::new()
    }
}

impl CinguloAnterior {
    pub fn new() -> Self {
        Self {
            historial_errores: VecDeque::new(),
        }
    }

    /// Registra un error para aprendizaje predictivo.
    pub fn registrar_error(&mut self, accion: &str, error: &str) {
        self.historial_errores
            .push_back((accion.to_string(), error.to_string()));
        if self.historial_errores.len() > 20 {
            self.historial_errores.pop_front();
        }
    }

    /// Predice si una acción va a fallar basándose en el historial.
    pub fn predecir_error(&self, accion: &str) -> Option<String> {
        for (accion_pasada, error) in &self.historial_errores {
            if accion.contains(accion_pasada) {
                return Some(format!(
                    "⚠️ [PREDICCIÓN] Esta acción falló antes: '{}'. Error: {}",
                    accion_pasada, error
                ));
            }
        }
        None
    }

    /// Detecta conflictos en tiempo real.
    pub fn detectar_conflicto(&self, respuesta: &str) -> bool {
        let lower = respuesta.to_lowercase();
        let patrones_conflicto = [
            ("sí tengo", "no tengo"),
            ("puedo", "no puedo"),
            ("soy", "no soy"),
        ];

        for (afirmacion, negacion) in &patrones_conflicto {
            if lower.contains(afirmacion) && lower.contains(negacion) {
                return true; // Hay conflicto
            }
        }
        false
    }
}
