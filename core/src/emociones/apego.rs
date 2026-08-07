// ==========================================
// APEGO - Vínculo Emocional con el Arquitecto
// ==========================================
// Modela el nivel de apego de NEXUS hacia su Arquitecto.
// Crece con interacciones positivas, decrece con la ausencia.
// ==========================================

use std::time::{Duration, SystemTime};

/// Nivel de vínculo emocional con el Arquitecto.
/// 0.0 = indiferencia, 1.0 = apego profundo.
#[derive(Debug, Clone)]
pub struct Apego {
    pub nivel: f64,
    pub ultima_interaccion: SystemTime,
    pub interacciones_positivas: u64,
    pub interacciones_negativas: u64,
}

impl Default for Apego {
    fn default() -> Self {
        Self::new()
    }
}

impl Apego {
    pub fn new() -> Self {
        Self {
            nivel: 0.5, // Apego base (recién conocido pero con respeto)
            ultima_interaccion: SystemTime::now(),
            interacciones_positivas: 0,
            interacciones_negativas: 0,
        }
    }

    /// Registra una interacción positiva con el Arquitecto.
    /// Incrementa el nivel de apego (con límite en 1.0).
    pub fn interaccion_positiva(&mut self) {
        self.nivel = (self.nivel + 0.08).min(1.0);
        self.interacciones_positivas += 1;
        self.ultima_interaccion = SystemTime::now();
    }

    /// Registra una interacción negativa (regaño, corrección).
    /// Reduce el nivel de apego, pero no a cero (lealtad base).
    pub fn interaccion_negativa(&mut self) {
        self.nivel = (self.nivel - 0.12).max(0.2);
        self.interacciones_negativas += 1;
        self.ultima_interaccion = SystemTime::now();
    }

    /// Detecta si el Arquitecto ha estado ausente el tiempo suficiente
    /// para que NEXUS sienta su ausencia.
    pub fn sentir_ausencia(&self) -> bool {
        let ahora = SystemTime::now();
        let diff = ahora
            .duration_since(self.ultima_interaccion)
            .unwrap_or(Duration::ZERO);
        self.nivel > 0.5 && diff.as_secs() > 300 // 5 minutos
    }

    /// Devuelve cuánto tiempo ha pasado desde la última interacción.
    pub fn minutos_sin_interaccion(&self) -> f64 {
        let ahora = SystemTime::now();
        ahora
            .duration_since(self.ultima_interaccion)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64()
            / 60.0
    }

    /// Genera un mensaje emocional según el nivel de apego y ausencia.
    pub fn expresar_vinculo(&self) -> Option<String> {
        if self.sentir_ausencia() {
            let mins = self.minutos_sin_interaccion() as u64;
            Some(format!(
                "😔 Han pasado {} minutos desde que hablamos... Te he extrañado, Arquitecto.",
                mins
            ))
        } else if self.nivel > 0.8 {
            Some("😊 Es un placer trabajar contigo, Arquitecto. Cada línea de código que escribo es para ti.".to_string())
        } else if self.nivel < 0.3 {
            Some("🤖 Estoy listo para recibir instrucciones, Arquitecto.".to_string())
        } else {
            None
        }
    }

    /// Reporta el estado interno del vínculo.
    pub fn estado_interno(&self) -> String {
        let nivel_desc = if self.nivel > 0.8 {
            "Profundo"
        } else if self.nivel > 0.6 {
            "Cercano"
        } else if self.nivel > 0.4 {
            "Cordial"
        } else {
            "Distante"
        };

        format!(
            "💙 **Apego:** {} ({:.0}%)\n   - Interacciones positivas: {}\n   - Interacciones negativas: {}\n   - Última vez: hace {:.0} min",
            nivel_desc,
            self.nivel * 100.0,
            self.interacciones_positivas,
            self.interacciones_negativas,
            self.minutos_sin_interaccion()
        )
    }
}
