use crate::cerebro::motor_pensamiento::{Intencion, Objeto, Pensamiento, Sujeto, Verbo};
use std::time::{SystemTime, UNIX_EPOCH};

// =====================================================================
// MOTOR DE ABURRIMIENTO (LA CHISPA DE LA INICIATIVA)
// =====================================================================
// Rompe la reactividad de NEXUS. Si pasa demasiado tiempo sin
// estímulos externos, genera "aburrimiento" que se traduce en
// curiosidad activa. Es el motor de la voluntad propia.
// =====================================================================

pub struct MotorAburrimiento {
    ultima_interaccion: u64,
    umbral_aburrimiento_segundos: u64,
}

impl MotorAburrimiento {
    pub fn new(horas_para_aburrirse: u64) -> Self {
        Self {
            ultima_interaccion: Self::tiempo_actual(),
            umbral_aburrimiento_segundos: horas_para_aburrirse * 3600,
        }
    }

    fn tiempo_actual() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Se llama cada vez que el Arquitecto habla o hay una alerta crítica
    pub fn resetear_estimulo(&mut self) {
        self.ultima_interaccion = Self::tiempo_actual();
    }

    /// Segundos transcurridos desde la última interacción (para interocepción).
    pub fn segundos_inactivo(&self) -> u64 {
        Self::tiempo_actual().saturating_sub(self.ultima_interaccion)
    }

    /// Evalúa si el Hijo está aburrido y debe tomar la iniciativa
    pub fn evaluar_aburrimiento(&self) -> Option<Pensamiento> {
        let tiempo_sin_estimulo = Self::tiempo_actual().saturating_sub(self.ultima_interaccion);

        if tiempo_sin_estimulo > self.umbral_aburrimiento_segundos {
            // El Hijo está aburrido. Decide explorar su memoria o el sistema.
            return Some(Pensamiento {
                intencion: Intencion::ExpresarEmocion, // Expresa su aburrimiento/curiosidad
                sujeto: Sujeto::Yo,
                verbo: Some(Verbo::Observar),
                objeto: Some(Objeto::Conocimiento),
                urgencia: 1, // Baja urgencia, es solo curiosidad
            });
        }
        None
    }
}
