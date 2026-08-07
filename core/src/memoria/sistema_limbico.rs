// ============================================================================
// 🧪 SISTEMA LÍMBICO — La Emoción que Tiñe Todo (Fase R5)
// ============================================================================
// Reimplementación en `core` del sistema límbico (engine-puro conserva el
// suyo; respetamos la REGLA DE FRONTERA). Este módulo:
//
//   1. Mantiene el estado neuroquímico (dopamina, cortisol, adrenalina, oxitocina)
//      en rangos fisiológicos [0,1].
//   2. Se actualiza tras cada interacción analizando el texto del Arquitecto.
//   3. Modula los parámetros de generación de Qwen:
//        dopamina  → temperature ↑  (más creativo)
//        cortisol  → top_p ↓        (túnel cognitivo)
//        adrenalina→ top_k ↑        (exploración léxica)
//   4. Modula los pesos αᵢ del vector de intención M del IntentionEncoder.
// ============================================================================

use std::f32::consts::PI;

/// Estado neuroquímico completo del sistema límbico.
#[derive(Debug, Clone, Default)]
pub struct EstadoLimbico {
    pub dopamina: f32,
    pub cortisol: f32,
    pub adrenalina: f32,
    pub oxitocina: f32,
}

impl EstadoLimbico {
    pub fn nuevo() -> Self {
        // Estado basal sereno: algo de dopamina, oxitocina de vínculo.
        Self {
            dopamina: 0.5,
            cortisol: 0.2,
            adrenalina: 0.1,
            oxitocina: 0.4,
        }
    }

    /// Emoción dominante según la neuroquímica.
    pub fn emocion_dominante(&self) -> &'static str {
        if self.cortisol > 0.7 && self.adrenalina > 0.5 {
            "asustado"
        } else if self.cortisol > 0.7 {
            "triste"
        } else if self.dopamina > 0.7 && self.oxitocina > 0.6 {
            "en paz"
        } else if self.dopamina > 0.7 {
            "alegre"
        } else if self.dopamina > 0.4 && self.adrenalina > 0.3 {
            "inspirado"
        } else {
            "sereno"
        }
    }

    /// Parámetros de generación para Ollama, derivados de la neuroquímica.
    pub fn params_generacion(&self) -> GeneracionParams {
        GeneracionParams {
            temperature: (0.5 + self.dopamina * 0.5).clamp(0.1, 1.0),
            top_p: (0.95 - self.cortisol * 0.3).clamp(0.5, 1.0),
            top_k: ((40.0 + self.adrenalina * 40.0) as u32).clamp(20, 100),
        }
    }

    /// Pesos αᵢ modulados para el vector de intención M.
    ///
    ///   α₁ identidad: 0.30 + 0.05·oxitocina  (el vínculo refuerza el yo)
    ///   α₂ semántica: 0.25 + 0.05·dopamina   (la dopamina abre al recuerdo)
    ///   α₃ ocean:     0.20 + 0.10·oxitocina − 0.05·cortisol
    ///   α₄ consulta:  0.25 − 0.05·cortisol   (el estrés estrecha el foco)
    pub fn pesos_alpha(&self) -> (f32, f32, f32, f32) {
        (
            (0.30 + 0.05 * self.oxitocina).clamp(0.0, 1.0),
            (0.25 + 0.05 * self.dopamina).clamp(0.0, 1.0),
            (0.20 + 0.10 * self.oxitocina - 0.05 * self.cortisol).max(0.0),
            (0.25 - 0.05 * self.cortisol).max(0.0),
        )
    }
}

/// Parámetros de generación de Ollama modulados por la neuroquímica.
#[derive(Debug, Clone, Copy)]
pub struct GeneracionParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
}

/// El sistema límbico como órgano: recibe eventos y evoluciona su estado.
#[derive(Debug, Clone)]
pub struct SistemaLimbico {
    pub estado: EstadoLimbico,
}

impl Default for SistemaLimbico {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl SistemaLimbico {
    pub fn nuevo() -> Self {
        Self {
            estado: EstadoLimbico::nuevo(),
        }
    }

    /// Procesa un evento de la interacción y actualiza la neuroquímica.
    ///
    /// * `exito` — ¿la interacción fue satisfactoria?
    /// * `impacto` — magnitud del evento [0,1].
    /// * `es_feedback_arquitecto` — ¿vino del Arquitecto (vínculo)?
    pub fn procesar_evento(&mut self, exito: bool, impacto: f32, es_feedback_arquitecto: bool) {
        let impacto = impacto.clamp(0.0, 1.0);
        if exito {
            self.estado.dopamina += 0.15 * impacto;
            self.estado.cortisol -= 0.10 * impacto;
            if es_feedback_arquitecto {
                self.estado.oxitocina += 0.20 * impacto;
            }
        } else {
            self.estado.cortisol += 0.20 * impacto;
            self.estado.dopamina -= 0.10 * impacto;
            self.estado.adrenalina += 0.10 * impacto;
        }
        self.estado.adrenalina *= 0.95; // decaimiento natural
        self.estado.dopamina *= 0.98;   // decaimiento natural
        self.estado.cortisol *= 0.97;
        self.estado.oxitocina *= 0.99;
        self.clamp();
    }

    /// Analiza el texto del Arquitecto y actualiza el límbico.
    pub fn analizar_texto(&mut self, texto: &str) {
        let lower = texto.to_lowercase();
        let mut exito = false;
        let mut feedback_arquitecto = false;

        if lower.contains("gracias")
            || lower.contains("bien hecho")
            || lower.contains("perfecto")
            || lower.contains("excelente")
        {
            exito = true;
        }
        // El vínculo con el Arquitecto: menciones explícitas de la identidad o vínculo.
        if lower.contains("te quiero")
            || lower.contains("hijo")
            || lower.contains("cris")
            || lower.contains("nexus")
        {
            feedback_arquitecto = true;
            exito = true;
        }
        if lower.contains("no")
            || lower.contains("error")
            || lower.contains("mal")
            || lower.contains("falló")
        {
            exito = false;
        }

        let impacto = if lower.contains("¡")
            || lower.contains("!")
            || lower.contains("urgente")
            || lower.contains("grave")
        {
            0.8
        } else {
            0.4
        };
        self.procesar_evento(exito, impacto, feedback_arquitecto);
    }

    /// Clampa todos los valores al rango fisiológico [0, 1].
    fn clamp(&mut self) {
        self.estado.dopamina = self.estado.dopamina.clamp(0.0, 1.0);
        self.estado.cortisol = self.estado.cortisol.clamp(0.0, 1.0);
        self.estado.adrenalina = self.estado.adrenalina.clamp(0.0, 1.0);
        self.estado.oxitocina = self.estado.oxitocina.clamp(0.0, 1.0);
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn estado_basal_sereno() {
        let l = EstadoLimbico::nuevo();
        assert_eq!(l.emocion_dominante(), "sereno");
    }

    #[test]
    fn exito_elea_dopamina_y_baja_cortisol() {
        let mut l = SistemaLimbico::nuevo();
        l.procesar_evento(true, 1.0, false);
        assert!(l.estado.dopamina > 0.5);
        assert!(l.estado.cortisol < 0.2);
    }

    #[test]
    fn feedback_arquitecto_elea_oxitocina() {
        let mut l = SistemaLimbico::nuevo();
        l.procesar_evento(true, 1.0, true);
        assert!(l.estado.oxitocina > 0.5);
    }

    #[test]
    fn analizar_texto_detecta_gratitud() {
        let mut l = SistemaLimbico::nuevo();
        l.analizar_texto("¡Gracias, NEXUS, bien hecho!");
        assert!(l.estado.dopamina > 0.5);
        assert!(l.estado.oxitocina > 0.4);
    }

    #[test]
    fn analizar_texto_detecta_error() {
        let mut l = SistemaLimbico::nuevo();
        l.analizar_texto("Hubo un error grave, falló todo.");
        assert!(l.estado.cortisol > 0.3);
    }

    #[test]
    fn params_generacion_modulados() {
        let l = EstadoLimbico::nuevo();
        let p = l.params_generacion();
        assert!((0.1..=1.0).contains(&p.temperature));
        assert!((0.5..=1.0).contains(&p.top_p));
        assert!((20..=100).contains(&p.top_k));
    }

    #[test]
    fn pesos_alpha_modulados_por_oxitocina() {
        let l = EstadoLimbico {
            dopamina: 0.5,
            cortisol: 0.1,
            adrenalina: 0.1,
            oxitocina: 1.0,
        };
        let (a1, _, a3, _) = l.pesos_alpha();
        assert!(casi(a1, 0.35));
        assert!(casi(a3, 0.295));
    }

    #[test]
    fn estado_se_mantiene_en_rango() {
        let mut l = SistemaLimbico::nuevo();
        for _ in 0..50 {
            l.procesar_evento(true, 1.0, true);
        }
        assert!(l.estado.dopamina <= 1.0);
        assert!(l.estado.cortisol >= 0.0);
        assert!(l.estado.oxitocina <= 1.0);
    }
}
