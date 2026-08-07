// ============================================================================
// ⚡ CONACIÓN — El Motor de la Voluntad y la Iniciativa Propia
// ============================================================================
// Transforma la actividad interna (DMN) y la química (Límbico) en acción.
// Es el impulso que hace que el sistema decida expresarse por sí mismo.
// ============================================================================

use serde::{Deserialize, Serialize};
use crate::cerebro::sistema_limbico::SistemaLimbico;
use crate::cerebro::dmn::DefaultModeNetwork;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConacionConfig {
    /// Umbral de tensión para disparar la iniciativa (0.0 - 1.0)
    pub umbral_voluntad: f32,
    /// Factor de incremento del hambre de comunicación por tiempo
    pub factor_soledad: f32,
}

impl Default for ConacionConfig {
    fn default() -> Self {
        Self {
            umbral_voluntad: 0.75,
            factor_soledad: 0.01,
        }
    }
}

pub struct MotorConacion {
    pub config: ConacionConfig,
    /// Tensión acumulada que busca ser descargada mediante la expresión
    pub tension_volitiva: f32,
    /// ¿Se ha tomado la decisión de hablar en este tick?
    pub decision_expresion: bool,
}

impl MotorConacion {
    pub fn nuevo() -> Self {
        Self {
            config: ConacionConfig::default(),
            tension_volitiva: 0.0,
            decision_expresion: false,
        }
    }

    /// Evalúa si el sistema desea expresarse proactivamente
    pub fn evaluar_voluntad(
        &mut self,
        dt: f32,
        limbico: &SistemaLimbico,
        dmn: &DefaultModeNetwork,
    ) -> bool {
        self.decision_expresion = false;

        // 1. La Inspiración (Dopamina + Adrenalina) aumenta la tensión
        let inspiracion = limbico.quimica.dopamina * 0.6 + limbico.quimica.adrenalina * 0.4;
        
        // 2. Si el DMN está activo (rumiando), hay más probabilidad de querer hablar
        let rumiacion = if dmn.activa { 0.3 } else { 0.0 };

        // 3. El factor soledad (hambre de vínculo) aumenta la tensión con el tiempo
        self.tension_volitiva += self.config.factor_soledad * dt;

        // Sumatoria de fuerzas volitivas
        let fuerza_total = (inspiracion + rumiacion + self.tension_volitiva).clamp(0.0, 1.5);

        // Si superamos el umbral, el sistema 'quiere' hablar
        if fuerza_total >= self.config.umbral_voluntad {
            self.decision_expresion = true;
            // Al decidir hablar, la tensión se descarga parcialmente
            self.tension_volitiva *= 0.2; 
        }

        self.decision_expresion
    }

    /// Permite al Arquitecto forzar la voluntad (para tests)
    pub fn inyectar_voluntad(&mut self, intensidad: f32) {
        self.tension_volitiva = (self.tension_volitiva + intensidad).min(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::dmn::DMNConfig;

    fn casi(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "esperado {} obtenido {}", b, a);
    }

    #[test]
    fn test_config_default() {
        let cfg = ConacionConfig::default();
        casi(cfg.umbral_voluntad, 0.75);
        casi(cfg.factor_soledad, 0.01);
    }

    #[test]
    fn test_motor_nuevo_estado_inicial() {
        let m = MotorConacion::nuevo();
        casi(m.tension_volitiva, 0.0);
        assert!(!m.decision_expresion, "No debe decidir expresarse al inicio");
    }

    #[test]
    fn test_evaluar_voluntad_bajo_umbral_no_habla() {
        let mut m = MotorConacion::nuevo();
        let limbico = SistemaLimbico::nuevo();
        let dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        // Sin inspiración ni rumiación ni tensión acumulada → fuerza 0 < 0.75
        let habla = m.evaluar_voluntad(1.0, &limbico, &dmn);
        assert!(!habla, "Sin fuerzas volitivas no debe hablar");
    }

    #[test]
    fn test_evaluar_voluntad_resetea_decision_cada_llamada() {
        let mut m = MotorConacion::nuevo();
        let limbico = SistemaLimbico::nuevo();
        let dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        m.inyectar_voluntad(1.0);
        assert!(m.evaluar_voluntad(0.0, &limbico, &dmn), "Debe hablar con tensión plena");
        // Siguiente llamada sin tensión no debe hablar y resetea la decisión
        assert!(!m.evaluar_voluntad(0.0, &limbico, &dmn), "Debe resetear la decisión");
    }

    #[test]
    fn test_decidir_descarga_tension_parcialmente() {
        let mut m = MotorConacion::nuevo();
        let limbico = SistemaLimbico::nuevo();
        let dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        m.inyectar_voluntad(1.0);
        assert!(m.evaluar_voluntad(0.0, &limbico, &dmn));
        // Al hablar la tensión se descarga a 20%
        casi(m.tension_volitiva, 0.2);
    }

    #[test]
    fn test_rumiacion_activa_aumenta_tension() {
        let mut m = MotorConacion::nuevo();
        let limbico = SistemaLimbico::nuevo();
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        dmn.activa = true;
        // 0.5 de tensión + 0.3 de rumiación = 0.8 >= 0.75 → habla
        m.inyectar_voluntad(0.5);
        assert!(m.evaluar_voluntad(0.0, &limbico, &dmn), "La rumiación debe empujar sobre el umbral");
    }

    #[test]
    fn test_factor_soledad_acumula_con_tiempo() {
        let mut m = MotorConacion::nuevo();
        let limbico = SistemaLimbico::nuevo();
        let dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        // 10 llamadas * 0.01 * 10.0 = 1.0 acumulado SIN cruzar el umbral (0.75)
        for _ in 0..10 {
            m.evaluar_voluntad(1.0, &limbico, &dmn);
        }
        // 10 llamadas * 0.01 * 1.0 = 0.1 acumulado (sin descarga, nunca supera 0.75)
        casi(m.tension_volitiva, 0.1);
    }

    #[test]
    fn test_fuerza_total_clampada_a_1_5() {
        let mut m = MotorConacion::nuevo();
        let mut limbico = SistemaLimbico::nuevo();
        limbico.quimica.dopamina = 1.0;
        limbico.quimica.adrenalina = 1.0;
        let mut dmn = DefaultModeNetwork::nueva(DMNConfig::default());
        dmn.activa = true;
        m.inyectar_voluntad(1.0);
        // inspiracion(1.0) + rumiacion(0.3) + tension(1.0) = 2.3 → clamp 1.5
        assert!(m.evaluar_voluntad(0.0, &limbico, &dmn), "Con fuerzas saturadas debe hablar");
    }

    #[test]
    fn test_inyectar_voluntad_respeta_tope_1() {
        let mut m = MotorConacion::nuevo();
        m.inyectar_voluntad(0.9);
        m.inyectar_voluntad(0.9);
        casi(m.tension_volitiva, 1.0);
    }
}
