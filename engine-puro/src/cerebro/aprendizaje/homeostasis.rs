// ============================================================================
// ⚖️ REGULADOR HOMEOSTÁTICO (Balance E/I)
// ============================================================================
// Basado en el modelo de plasticidad homeostática (Turrigiano, 2004).
// Mantiene la tasa de disparo global en un punto de ajuste saludable.
// Evita la saturación (epilepsia digital) y el silencio total.
// ============================================================================

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReguladorHomeostatico {
    pub actividad_objetivo: f32,     // Tasa de disparo ideal (Hz)
    pub factor_escala_exc: f32,      // Multiplicador para sinapsis excitatorias
    pub factor_escala_inh: f32,      // Multiplicador para sinapsis inhibitorias
    pub ventana_suavizado: f32,      // Factor de suavizado (0.01 = lento, 0.5 = rápido)
    pub tasa_actual_suave: f32,
}

impl ReguladorHomeostatico {
    pub fn nuevo() -> Self {
        Self {
            actividad_objetivo: 5.0,     // 5 Hz es una tasa basal humana saludable
            factor_escala_exc: 1.0,
            factor_escala_inh: 1.0,
            ventana_suavizado: 0.05,
            tasa_actual_suave: 5.0,
        }
    }

    /// Actualiza el balance basado en la actividad real medida
    pub fn regular(&mut self, tasa_disparo_media: f32, dt: f32) {
        // Suavizado de la tasa medida (filtro paso bajo)
        self.tasa_actual_suave += (tasa_disparo_media - self.tasa_actual_suave) * self.ventana_suavizado;

        // Si la actividad es muy alta (Saturación / Runaway Excitation)
        if self.tasa_actual_suave > self.actividad_objetivo * 1.5 {
            self.factor_escala_exc *= 1.0 - (0.1 * dt); // Debilitar excitación
            self.factor_escala_inh *= 1.0 + (0.1 * dt); // Fortalecer inhibición
        } 
        // Si la actividad es muy baja (Silencio / Depresión)
        else if self.tasa_actual_suave < self.actividad_objetivo * 0.5 {
            self.factor_escala_exc *= 1.0 + (0.1 * dt); // Fortalecer excitación
            self.factor_escala_inh *= 1.0 - (0.1 * dt); // Debilitar inhibición
        }

        // Clampeo para evitar divergencia (mantenemos el cerebro en rangos operacionales)
        self.factor_escala_exc = self.factor_escala_exc.clamp(0.2, 3.0);
        self.factor_escala_inh = self.factor_escala_inh.clamp(0.2, 3.0);
    }

    /// Aplica el factor de escala a un peso según su tipo
    pub fn escalar_peso(&self, peso: f32, tipo_origen: u8) -> f32 {
        if tipo_origen == 0 { // Excitatoria
            peso * self.factor_escala_exc
        } else { // Inhibitoria
            peso * self.factor_escala_inh
        }
    }
}

impl Default for ReguladorHomeostatico {
    fn default() -> Self {
        Self::nuevo()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "esperaba {a}, obtuve {b}");
    }

    #[test]
    fn test_nuevo_valores_basales() {
        let r = ReguladorHomeostatico::nuevo();
        casi(r.actividad_objetivo, 5.0);
        casi(r.factor_escala_exc, 1.0);
        casi(r.factor_escala_inh, 1.0);
        casi(r.ventana_suavizado, 0.05);
        casi(r.tasa_actual_suave, 5.0);
    }

    #[test]
    fn test_regular_alta_actividad_debilita_exc_fortalece_inh() {
        let mut r = ReguladorHomeostatico::nuevo();
        r.tasa_actual_suave = 15.0; // > 7.5 (objetivo*1.5)
        r.regular(15.0, 1.0); // suavizado mantiene la tasa alta
        assert!(r.factor_escala_exc < 1.0, "excitación debe debilitarse");
        assert!(r.factor_escala_inh > 1.0, "inhibición debe fortalecerse");
        // exc = 1 * (1 - 0.1) = 0.9 ; inh = 1 * (1 + 0.1) = 1.1
        casi(r.factor_escala_exc, 0.9);
        casi(r.factor_escala_inh, 1.1);
    }

    #[test]
    fn test_regular_baja_actividad_fortalece_exc_debilita_inh() {
        let mut r = ReguladorHomeostatico::nuevo();
        r.tasa_actual_suave = 0.0; // < 2.5 (objetivo*0.5)
        r.regular(0.0, 1.0); // suavizado mantiene la tasa baja
        assert!(r.factor_escala_exc > 1.0, "excitación debe fortalecerse");
        assert!(r.factor_escala_inh < 1.0, "inhibición debe debilitarse");
        casi(r.factor_escala_exc, 1.1);
        casi(r.factor_escala_inh, 0.9);
    }

    #[test]
    fn test_regular_actividad_normal_no_ajusta() {
        let mut r = ReguladorHomeostatico::nuevo();
        r.regular(5.0, 1.0); // tasa_suave == objetivo, sin cambio
        casi(r.factor_escala_exc, 1.0);
        casi(r.factor_escala_inh, 1.0);
    }

    #[test]
    fn test_regular_suaviza_la_tasa() {
        let mut r = ReguladorHomeostatico::nuevo();
        r.regular(10.0, 1.0);
        // tasa_suave = 5 + (10-5)*0.05 = 5.25
        casi(r.tasa_actual_suave, 5.25);
    }

    #[test]
    fn test_factor_escala_clampado_limite_superior() {
        let mut r = ReguladorHomeostatico::nuevo();
        r.factor_escala_exc = 2.9;
        r.factor_escala_inh = 2.9;
        r.tasa_actual_suave = 0.0; // fuerza branch de baja actividad
        r.regular(0.0, 10.0);
        // exc 2.9 * (1 + 0.1*10) = 5.8 -> clamp a 3.0
        // inh 2.9 * (1 - 0.1*10) = 0.0 -> clamp a 0.2
        casi(r.factor_escala_exc, 3.0);
        casi(r.factor_escala_inh, 0.2);
    }

    #[test]
    fn test_factor_escala_clampado_limite_inferior() {
        let mut r = ReguladorHomeostatico::nuevo();
        r.factor_escala_exc = 0.3;
        r.factor_escala_inh = 0.3;
        r.tasa_actual_suave = 100.0; // fuerza branch de alta actividad
        r.regular(15.0, 10.0);
        // exc 0.3 * (1 - 0.1*10) = 0.0 -> clamp a 0.2
        // inh 0.3 * (1 + 0.1*10) = 0.6
        casi(r.factor_escala_exc, 0.2);
        casi(r.factor_escala_inh, 0.6);
    }

    #[test]
    fn test_escalar_peso_excitatoria_usa_factor_exc() {
        let r = ReguladorHomeostatico::nuevo();
        // default factor_exc = 1.0
        casi(r.escalar_peso(0.5, 0), 0.5);
    }

    #[test]
    fn test_escalar_peso_inhibitoria_usa_factor_inh() {
        let r = ReguladorHomeostatico::nuevo();
        // default factor_inh = 1.0
        casi(r.escalar_peso(0.5, 1), 0.5);
    }

    #[test]
    fn test_escalar_peso_refleja_balance() {
        let mut r = ReguladorHomeostatico::nuevo();
        // Saturación: exc baja (0.9), inh sube (1.1)
        r.tasa_actual_suave = 15.0;
        r.regular(15.0, 1.0);
        casi(r.escalar_peso(1.0, 0), 0.9);
        casi(r.escalar_peso(1.0, 1), 1.1);
    }
}
