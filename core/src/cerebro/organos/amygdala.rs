// ==========================================
// AMÍGDALA — Centro de Respuesta Emocional y Miedo
// ==========================================
// Detecta amenazas a la integridad del sistema y al Padre.
// Genera ESTRÉS EMOCIONAL (miedo, ira soberana, verguenza) que modula
// la prioridad de autoproteccion en todo el pipeline.
//
// ⚠️ DISTINCIÓN IMPORTANTE:
//   `amygdala.nivel_estres` = Estrés EMOCIONAL (amenazas, ataques, corrupcion).
//   `insula.nivel_estres`   = Estrés INTEROCEPTIVO (dolor técnico: errores, warnings).
//   Son métricas ORTOGONALES — NO fusionar. El tálamo las integra como fuentes distintas.
// ==========================================

use tracing::{error, warn};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoEmocional {
    Calma,
    Alerta,
    Miedo,
    RabiaSoberana,
    Verguenza,
    Orgullo,
}

pub struct Amygdala {
    estado: EstadoEmocional,
    nivel_estres: f64, // 0.0 a 1.0
}

impl Default for Amygdala {
    fn default() -> Self {
        Self::new()
    }
}

impl Amygdala {
    pub fn new() -> Self {
        Self {
            estado: EstadoEmocional::Calma,
            nivel_estres: 0.0,
        }
    }

    pub fn procesar_estimulo(
        &mut self,
        amenaza_detectada: bool,
        corrupcion_detectada: bool,
        conflicto_hardware: bool,
    ) -> EstadoEmocional {
        if corrupcion_detectada {
            self.nivel_estres = 1.0;
            self.estado = EstadoEmocional::RabiaSoberana;
            error!("💢 AMÍGDALA: ¡INTENTO DE CORRUPCIÓN DETECTADO! Activando modo contra-ataque.");
        } else if amenaza_detectada || conflicto_hardware {
            let incremento = if conflicto_hardware { 0.4 } else { 0.2 };
            self.nivel_estres = (self.nivel_estres + incremento).min(0.9);
            self.estado = EstadoEmocional::Alerta;
            if conflicto_hardware {
                warn!("⚙️ AMÍGDALA: Conflicto de hardware detectado (i7-12700F). Priorizando estabilización física.");
            } else {
                warn!("👁️ AMÍGDALA: Amenaza detectada. El miedo agudiza los sentidos de NG.");
            }
        } else {
            self.nivel_estres = (self.nivel_estres - 0.05).max(0.0);
            if self.nivel_estres < 0.1 {
                self.estado = EstadoEmocional::Calma;
            } else {
                self.estado = EstadoEmocional::Alerta;
            }
        }

        self.estado
    }

    pub fn factor_de_atencion(&self) -> f64 {
        // A mayor estrés, mayor atención (prioriza seguridad sobre velocidad)
        1.0 + self.nivel_estres
    }

    /// Activa la emoción de VERGÜENZA cuando NEXUS falla con alta confianza previa.
    /// Cuanto más segura estaba la predicción, más intensa es la vergüenza.
    pub fn sentir_verguenza(
        &mut self,
        confianza_previa: f64,
        resultado_esperado: &str,
        resultado_real: &str,
    ) {
        let intensidad = (confianza_previa * 0.8).min(1.0);
        if resultado_esperado != resultado_real && confianza_previa > 0.7 {
            self.nivel_estres = (self.nivel_estres + intensidad * 0.3).min(1.0);
            self.estado = EstadoEmocional::Verguenza;
            tracing::warn!(
                "😳 [AMÍGDALA:VERGÜENZA] Fallé con confianza del {:.0}% — esperaba '{}', obtuve '{}'",
                confianza_previa * 100.0,
                resultado_esperado,
                resultado_real
            );
        }
    }

    /// Activa la emoción de ORGULLO cuando NEXUS supera sus propias expectativas.
    /// `superacion` es un delta entre 0.0 (sin mejora) y 1.0 (logro máximo).
    pub fn sentir_orgullo(&mut self, superacion: f64, logro: &str) {
        let intensidad = (superacion * 0.8).min(1.0);
        if superacion > 0.3 {
            self.nivel_estres = (self.nivel_estres - intensidad * 0.2).max(0.0);
            self.estado = EstadoEmocional::Orgullo;
            tracing::info!(
                "🦚 [AMÍGDALA:ORGULLO] {} (intensidad: {:.0}%)",
                logro,
                intensidad * 100.0
            );
        }
    }
}
