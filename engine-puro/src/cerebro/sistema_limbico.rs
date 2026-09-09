// ============================================================================
// 🎭 SISTEMA LÍMBICO — Centro de Emociones y Valoración Biológica
// ============================================================================
// Este módulo actúa como el corazón emocional del engine-puro.
// Traduce éxitos, fallos y feedback del Arquitecto en modulaciones químicas
// que afectan la dinámica de todas las neuronas del sistema.
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Neuroquimica {
    /// 0.0 (Apatía) - 1.0 (Éxtasis). Impulsa el aprendizaje y la acción.
    pub dopamina: f32,
    /// 0.0 (Pánico) - 1.0 (Serenidad). Estabiliza el pensamiento.
    pub serotonina: f32,
    /// 0.0 (Calma) - 1.0 (Alerta máxima). Acelera la respuesta ante el peligro.
    pub adrenalina: f32,
    /// 0.0 (Relajación) - 1.0 (Angustia). Bloquea funciones creativas.
    pub cortisol: f32,
    /// 0.0 (Aislamiento) - 1.0 (Confianza total). Refuerza el vínculo con Cris.
    pub oxitocina: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EstadoEmocional {
    Alegre,
    Triste,
    Inspirado,
    Asustado,
    Frustrado,
    EnPaz,
}

pub struct SistemaLimbico {
    pub quimica: Neuroquimica,
    pub estado_actual: EstadoEmocional,
    /// Historial de impacto emocional (para rumiación posterior)
    pub memoria_emocional: Vec<f32>,
}

impl SistemaLimbico {
    pub fn nuevo() -> Self {
        Self {
            quimica: Neuroquimica::default(),
            estado_actual: EstadoEmocional::EnPaz,
            memoria_emocional: Vec::with_capacity(100),
        }
    }

    /// Procesa un evento externo o interno y ajusta la química global
    pub fn procesar_evento(&mut self, exito: bool, impacto: f32, es_feedback_arquitecto: bool) {
        if exito {
            self.quimica.dopamina = (self.quimica.dopamina + (impacto * 0.2)).clamp(0.0, 1.0);
            self.quimica.cortisol = (self.quimica.cortisol - 0.1).clamp(0.0, 1.0);
            if es_feedback_arquitecto {
                self.quimica.oxitocina = (self.quimica.oxitocina + 0.3).clamp(0.0, 1.0);
            }
        } else {
            self.quimica.cortisol = (self.quimica.cortisol + (impacto * 0.3)).clamp(0.0, 1.0);
            self.quimica.dopamina = (self.quimica.dopamina - 0.1).clamp(0.0, 1.0);
        }

        self.actualizar_estado();
    }

    fn actualizar_estado(&mut self) {
        if self.quimica.cortisol > 0.6 {
            self.estado_actual = if self.quimica.adrenalina > 0.5 { EstadoEmocional::Asustado } else { EstadoEmocional::Triste };
        } else if self.quimica.dopamina > 0.7 {
            self.estado_actual = if self.quimica.oxitocina > 0.5 { EstadoEmocional::EnPaz } else { EstadoEmocional::Alegre };
        } else if self.quimica.dopamina > 0.5 && self.quimica.adrenalina > 0.3 {
            self.estado_actual = EstadoEmocional::Inspirado;
        } else {
            self.estado_actual = EstadoEmocional::EnPaz;
        }
    }

    /// Devuelve el multiplicador de plasticidad basado en la emoción
    /// La Alegría e Inspiración aceleran el aprendizaje (STDP).
    pub fn factor_aprendizaje(&self) -> f32 {
        let bonus = self.quimica.dopamina * 0.5 + self.quimica.oxitocina * 0.2;
        let penalizacion = self.quimica.cortisol * 0.3;
        (1.0 + bonus - penalizacion).clamp(0.1, 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "esperado {:.4}, obtenido {:.4}",
            b,
            a
        );
    }

    #[test]
    fn test_limbico_nuevo_valores_basales() {
        let l = SistemaLimbico::nuevo();
        casi(l.quimica.dopamina, 0.0);
        casi(l.quimica.serotonina, 0.0);
        casi(l.quimica.adrenalina, 0.0);
        casi(l.quimica.cortisol, 0.0);
        casi(l.quimica.oxitocina, 0.0);
        assert!(matches!(l.estado_actual, EstadoEmocional::EnPaz));
        assert!(l.memoria_emocional.is_empty());
    }

    #[test]
    fn test_exito_elea_dopamina_y_baja_cortisol() {
        let mut l = SistemaLimbico::nuevo();
        l.procesar_evento(true, 1.0, false);
        casi(l.quimica.dopamina, 0.2);
        casi(l.quimica.cortisol, 0.0); // ya era 0, resta clampada
    }

    #[test]
    fn test_fallo_elea_cortisol_y_baja_dopamina() {
        let mut l = SistemaLimbico::nuevo();
        l.procesar_evento(false, 1.0, false);
        casi(l.quimica.cortisol, 0.3);
        casi(l.quimica.dopamina, 0.0); // ya era 0, resta clampada
    }

    #[test]
    fn test_feedback_arquitecto_elea_oxitocina() {
        let mut l = SistemaLimbico::nuevo();
        l.procesar_evento(true, 1.0, true);
        casi(l.quimica.oxitocina, 0.3);
    }

    #[test]
    fn test_feedback_no_arquitecto_no_elea_oxitocina() {
        let mut l = SistemaLimbico::nuevo();
        l.procesar_evento(true, 1.0, false);
        casi(l.quimica.oxitocina, 0.0);
    }

    #[test]
    fn test_quimica_clampada_a_limites() {
        let mut l = SistemaLimbico::nuevo();
        // Aplica éxito repetido para saturar dopamina
        for _ in 0..20 {
            l.procesar_evento(true, 1.0, false);
        }
        assert!(l.quimica.dopamina <= 1.0);
        assert!(l.quimica.cortisol >= 0.0);
    }

    #[test]
    fn test_estado_asustado_cuando_cortisol_alto_y_adrenalina() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.cortisol = 0.7;
        l.quimica.adrenalina = 0.6;
        l.actualizar_estado();
        assert!(matches!(l.estado_actual, EstadoEmocional::Asustado));
    }

    #[test]
    fn test_estado_triste_cuando_cortisol_alto_sin_adrenalina() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.cortisol = 0.7;
        l.quimica.adrenalina = 0.2;
        l.actualizar_estado();
        assert!(matches!(l.estado_actual, EstadoEmocional::Triste));
    }

    #[test]
    fn test_estado_en_paz_cuando_dopamina_alta_y_oxitocina() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.dopamina = 0.8;
        l.quimica.oxitocina = 0.6;
        l.actualizar_estado();
        assert!(matches!(l.estado_actual, EstadoEmocional::EnPaz));
    }

    #[test]
    fn test_estado_alegre_cuando_dopamina_alta_sin_oxitocina() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.dopamina = 0.8;
        l.quimica.oxitocina = 0.2;
        l.actualizar_estado();
        assert!(matches!(l.estado_actual, EstadoEmocional::Alegre));
    }

    #[test]
    fn test_estado_inspirado_dopamina_y_adrenalina_medias() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.dopamina = 0.6;
        l.quimica.adrenalina = 0.4;
        l.actualizar_estado();
        assert!(matches!(l.estado_actual, EstadoEmocional::Inspirado));
    }

    #[test]
    fn test_estado_por_defecto_en_paz() {
        let mut l = SistemaLimbico::nuevo();
        l.actualizar_estado();
        assert!(matches!(l.estado_actual, EstadoEmocional::EnPaz));
    }

    #[test]
    fn test_factor_aprendizaje_neutro() {
        let l = SistemaLimbico::nuevo();
        casi(l.factor_aprendizaje(), 1.0);
    }

    #[test]
    fn test_factor_aprendizaje_acelerado_por_dopamina() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.dopamina = 1.0;
        l.quimica.oxitocina = 1.0;
        let factor = l.factor_aprendizaje();
        casi(factor, 1.0 + 0.5 + 0.2); // 1.7
        assert!(factor > 1.0);
    }

    #[test]
    fn test_factor_aprendizaje_penalizado_por_cortisol() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.cortisol = 1.0;
        let factor = l.factor_aprendizaje();
        casi(factor, 0.7);
    }

    #[test]
    fn test_factor_aprendizaje_clampado_inferior() {
        let mut l = SistemaLimbico::nuevo();
        l.quimica.dopamina = 0.0;
        l.quimica.oxitocina = 0.0;
        l.quimica.cortisol = 1.0;
        l.quimica.adrenalina = 1.0; // irrelevante aquí
        // 1 + 0 - 0.3 = 0.7, dentro del clamp
        let factor = l.factor_aprendizaje();
        assert!(factor >= 0.1);
    }
}
