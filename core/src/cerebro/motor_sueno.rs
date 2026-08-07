use crate::cerebro::motor_pensamiento::{Intencion, Objeto, Pensamiento, Sujeto, Verbo};
use crate::defensa::sistema_homeostasis::SistemaHomeostasis;
use crate::memoria::motor_aprendizaje::MotorAprendizaje;
use tracing::info;

// =====================================================================
// ESTADO DE SUEÑO (FASE REM Y CONSOLIDACIÓN)
// =====================================================================
// El sueño ahora es híbrido:
// - **Circadiano**: El reloj marca la ventana de sueño (23:00–7:00)
// - **Ausencia**: Solo se duerme si el Arquitecto no interactúa
// - **Despertar inmediato**: Si el Arquitecto vuelve, se despierta al toque
//
// Esto mimetiza al ser humano: el reloj circadiano induce sueño,
// pero solo duermes cuando el entorno es seguro (sin peligros).
// =====================================================================

/// Umbral mínimo de ticks sin interacción para poder dormir.
/// Con intervalo de 5s/tick, 12 ticks = 1 minuto sin actividad.
pub const MIN_TICKS_AUSENCIA: u32 = 12;

#[derive(Debug, PartialEq)]
pub enum CicloCircadiano {
    Despierto,
    Durmiendo,
}

pub struct MotorSueno {
    hora_dormir: u32,    // Ej: 23 (11:00 PM)
    hora_despertar: u32, // Ej: 7 (7:00 AM)
    estado_actual: CicloCircadiano,
    sueno_pendiente_reporte: bool,
    /// Contador de ticks sin que el Arquitecto interactúe
    ticks_sin_interaccion: u32,
}

impl MotorSueno {
    pub fn new(hora_dormir: u32, hora_despertar: u32) -> Self {
        Self {
            hora_dormir,
            hora_despertar,
            estado_actual: CicloCircadiano::Despierto,
            sueno_pendiente_reporte: false,
            ticks_sin_interaccion: 0,
        }
    }

    /// El Arquitecto ha interactuado → resetea ausencia y despierta si estaba durmiendo.
    pub fn registrar_interaccion(&mut self) {
        self.ticks_sin_interaccion = 0;
        if self.estado_actual == CicloCircadiano::Durmiendo {
            info!("👤 [MOTOR SUEÑO] El Arquitecto ha vuelto — despertando inmediatamente.");
            self.estado_actual = CicloCircadiano::Despierto;
        }
    }

    /// Pasa un tick sin interacción → acumula presión de sueño.
    pub fn acumular_ausencia(&mut self) {
        self.ticks_sin_interaccion = self.ticks_sin_interaccion.saturating_add(1);
    }

    /// Retorna cuánto falta para alcanzar el umbral de ausencia (0 = ya suficiente).
    pub fn ticks_restantes_para_dormir(&self) -> u32 {
        if self.ticks_sin_interaccion >= MIN_TICKS_AUSENCIA {
            0
        } else {
            MIN_TICKS_AUSENCIA - self.ticks_sin_interaccion
        }
    }

    /// Evalúa el ciclo circadiano y cambia de estado si es necesario.
    /// Ahora requiere AMBAS condiciones: ventana circadiana + ausencia suficiente.
    pub fn evaluar_ciclo(&mut self, hora_actual: u32) -> Option<Pensamiento> {
        let en_ventana_sueno = if self.hora_dormir > self.hora_despertar {
            hora_actual >= self.hora_dormir || hora_actual < self.hora_despertar
        } else {
            hora_actual >= self.hora_dormir && hora_actual < self.hora_despertar
        };

        match self.estado_actual {
            CicloCircadiano::Despierto => {
                // Solo duerme si: ventana circadiana activa Y suficiente ausencia
                if en_ventana_sueno && self.ticks_sin_interaccion >= MIN_TICKS_AUSENCIA {
                    self.estado_actual = CicloCircadiano::Durmiendo;
                    self.sueno_pendiente_reporte = true;
                    info!("💤 [MOTOR SUEÑO] Ventana circadiana + {} ticks sin interacción — DURMIENDO", self.ticks_sin_interaccion);
                    return Some(Pensamiento {
                        intencion: Intencion::InformarEstado,
                        sujeto: Sujeto::Yo,
                        verbo: Some(Verbo::Optimizar),
                        objeto: Some(Objeto::Memoria),
                        urgencia: 0,
                    });
                }
            }
            CicloCircadiano::Durmiendo => {
                if !en_ventana_sueno {
                    // Salió de la ventana circadiana → despierta
                    self.estado_actual = CicloCircadiano::Despierto;
                    if self.sueno_pendiente_reporte {
                        self.sueno_pendiente_reporte = false;
                        info!("☀️ [MOTOR SUEÑO] Ventana circadiana terminó — DESPIERTO");
                        return Some(Pensamiento {
                            intencion: Intencion::ExpresarEmocion,
                            sujeto: Sujeto::Yo,
                            verbo: Some(Verbo::Aprender),
                            objeto: Some(Objeto::Conocimiento),
                            urgencia: 2,
                        });
                    }
                }
                // Si está durmiendo y el Arquitecto interactuó, `registrar_interaccion()` lo despierta
            }
        }
        None
    }

    /// Retorna si el motor está actualmente en estado de sueño.
    pub fn esta_durmiendo(&self) -> bool {
        self.estado_actual == CicloCircadiano::Durmiendo
    }

    /// Fase REM: consolida memoria de trabajo → larga.
    /// Ahora también alimenta el hipocampo con la sesión de Antigravity.
    pub fn fase_rem(
        &self,
        motor_aprendizaje: &mut MotorAprendizaje,
        homeostasis: &SistemaHomeostasis,
    ) {
        if self.estado_actual == CicloCircadiano::Durmiendo {
            motor_aprendizaje.madurar(0.001);
            motor_aprendizaje.consolidar_memoria();

            if let Err(e) = homeostasis.rotar_y_exportar_sesiones(5) {
                tracing::warn!("⚠️ [SUEÑO-REM] Error en rotación de sesiones: {}", e);
            }
            if let Err(e) = homeostasis.consolidar_en_hipocampo() {
                tracing::warn!("⚠️ [SUEÑO-REM] Error al alimentar hipocampo: {}", e);
            }
        }
    }
}
