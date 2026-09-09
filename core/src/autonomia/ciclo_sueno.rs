// 🌙 CICLO DE SUEÑO — Ritmo Circadiano de NEXUS
// ============================================================================
// Fusión del legacy `motor_sueno.rs` en el sistema autónomo del core.
//
// Aporta:
//   1. CicloCircadiano: Despierto/Durmiendo con horario configurable
//   2. Programación por hora: hora_dormir / hora_despertar
//   3. Fase REM: consolida aprendizaje llamando a MotorAprendizaje
//   4. Reportes de transición: pensamientos al dormir/despertar
//
// Diferencia con `core/src/emociones/limbico.rs::dormir()`:
//   - limbico.dormir(): Restaura homeostasis, poda historial emocional,
//     recupera energía. Es un RESET EMOCIONAL.
//   - CicloSueno: Gestiona el ciclo circadiano horario, detecta cuándo
//     debe dormir/despertar según la hora, y ejecuta fase REM.
//     Es un RELOJ BIOLÓGICO.
// ============================================================================

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum CicloCircadiano {
    Despierto,
    Durmiendo,
}

/// Informe generado al despertar, resumiendo los efectos del sueño.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteSueno {
    pub hora_dormir: u32,
    pub hora_despertar: u32,
    pub duracion_horas: u32,
    pub fase_rem_completada: bool,
    pub memoria_consolidada: bool,
    pub descripcion: String,
}

/// Gestiona el ciclo de sueño/vigilia de NEXUS basado en un horario circadiano.
///
/// # Ejemplo
/// ```ignore
/// let mut sueno = CicloSueno::new(23, 7); // Duerme de 23:00 a 7:00
/// let now = 2; // 2 AM
/// if let Some(reporte) = sueno.evaluar_ciclo(now) {
///     println!("{}", reporte.descripcion);
/// }
/// ```
pub struct CicloSueno {
    /// Hora en que NEXUS debe dormir (formato 0-23)
    pub hora_dormir: u32,
    /// Hora en que NEXUS debe despertar (formato 0-23)
    pub hora_despertar: u32,
    /// Estado actual del ciclo
    pub estado_actual: CicloCircadiano,
    /// Indica si hay un reporte pendiente por generar (al despertar)
    pub sueno_pendiente_reporte: bool,
    /// Hora en que se durmió (para calcular duración al despertar)
    pub hora_dormido: Option<u32>,
}

impl CicloSueno {
    /// Crea un nuevo ciclo de sueño con el horario especificado.
    ///
    /// * `hora_dormir` — Hora de dormir (0-23, ej: 23 = 11 PM)
    /// * `hora_despertar` — Hora de despertar (0-23, ej: 7 = 7 AM)
    ///
    /// Soporta ciclos que cruzan la medianoche (dormir 23 → despertar 7).
    pub fn new(hora_dormir: u32, hora_despertar: u32) -> Self {
        let hora_dormir = hora_dormir.min(23);
        let hora_despertar = hora_despertar.min(23);

        info!(
            "🌙 [CICLO SUEÑO] Inicializado: dormir {}:00, despertar {}:00",
            hora_dormir, hora_despertar
        );

        Self {
            hora_dormir,
            hora_despertar,
            estado_actual: CicloCircadiano::Despierto,
            sueno_pendiente_reporte: false,
            hora_dormido: None,
        }
    }

    /// Crea un ciclo con horario por defecto (23:00 → 7:00).
    pub fn default_night() -> Self {
        Self::new(23, 7)
    }

    /// Crea un ciclo con siesta por defecto (14:00 → 14:30).
    pub fn siesta() -> Self {
        Self::new(14, 14) // Siesta de 30 min si el ciclo se evalúa cada hora
    }

    /// Evalúa si debe cambiar de estado según la hora actual.
    ///
    /// Retorna `Some(ReporteSueno)` cuando hay una transición de estado
    /// (dormir → despertar), o `None` si no hay cambio.
    ///
    /// * `hora_actual` — Hora del sistema (0-23)
    pub fn evaluar_ciclo(&mut self, hora_actual: u32) -> Option<ReporteSueno> {
        let hora_actual = hora_actual.min(23);

        // Determinar si debería estar durmiendo según el horario
        let deberia_dormir = if self.hora_dormir > self.hora_despertar {
            // Cruzando medianoche: ej, dormir 23 → despertar 7
            hora_actual >= self.hora_dormir || hora_actual < self.hora_despertar
        } else if self.hora_dormir == self.hora_despertar {
            // Siesta del mismo día (misma hora = ventana de 1 hora)
            hora_actual == self.hora_dormir
        } else {
            // Mismo día: ej, dormir 22 → despertar 6
            hora_actual >= self.hora_dormir && hora_actual < self.hora_despertar
        };

        match self.estado_actual {
            CicloCircadiano::Despierto => {
                if deberia_dormir {
                    self.estado_actual = CicloCircadiano::Durmiendo;
                    self.sueno_pendiente_reporte = true;
                    self.hora_dormido = Some(hora_actual);
                    info!(
                        "😴 [CICLO SUEÑO] Hora de dormir ({}:00). Entrando en modo sueño.",
                        hora_actual
                    );
                    // Transición a dormir → no genera reporte aún
                    None
                } else {
                    None
                }
            }
            CicloCircadiano::Durmiendo => {
                if !deberia_dormir || hora_actual == self.hora_despertar {
                    self.estado_actual = CicloCircadiano::Despierto;

                    // Calcular duración
                    let duracion = self.hora_dormido.map_or(1, |d| {
                        if hora_actual >= d {
                            hora_actual - d
                        } else {
                            (24 - d) + hora_actual
                        }
                    });

                    let tiene_reporte = self.sueno_pendiente_reporte;

                    // Generar reporte al despertar
                    let reporte = ReporteSueno {
                        hora_dormir: self.hora_dormido.unwrap_or(hora_actual),
                        hora_despertar: hora_actual,
                        duracion_horas: duracion.max(1),
                        fase_rem_completada: duracion >= 1,
                        memoria_consolidada: tiene_reporte && duracion >= 1,
                        descripcion: if tiene_reporte {
                            format!(
                                "🌅 ¡Buenos días! Dormí {} horas ({}:00 → {}:00). \
                                 Fase REM completada. Memoria consolidada.",
                                duracion,
                                self.hora_dormido.unwrap_or(hora_actual),
                                hora_actual
                            )
                        } else {
                            format!(
                                "🌅 Desperté tras {} horas de sueño ({}:00 → {}:00). \
                                 Ciclo completado.",
                                duracion,
                                self.hora_dormido.unwrap_or(hora_actual),
                                hora_actual
                            )
                        },
                    };

                    self.sueno_pendiente_reporte = false;
                    self.hora_dormido = None;

                    info!("☀️ [CICLO SUEÑO] {}", reporte.descripcion);

                    Some(reporte)
                } else {
                    None
                }
            }
        }
    }

    /// Ejecuta la fase REM: consolida memoria y procesa aprendizaje.
    ///
    /// Solo se ejecuta si el sistema está en estado `Durmiendo`.
    /// La consolidación se delega al `MotorAprendizaje` a través del closure.
    pub fn fase_rem<F>(&self, consolidar: F)
    where
        F: FnOnce(),
    {
        if self.estado_actual == CicloCircadiano::Durmiendo {
            debug!("🌙 [CICLO SUEÑO] Fase REM activa. Consolidando memoria...");
            consolidar();
            info!("🧠 [CICLO SUEÑO] Fase REM completada. Memoria consolidada.");
        }
    }

    /// Verifica si el sistema está actualmente en estado de sueño.
    pub fn esta_durmiendo(&self) -> bool {
        self.estado_actual == CicloCircadiano::Durmiendo
    }

    /// Retorna el porcentaje del ciclo completado (0.0 - 1.0)
    /// basado en la hora actual. Útil para UI y logging.
    pub fn progreso_ciclo(&self, hora_actual: u32) -> f64 {
        let duracion_total = if self.hora_dormir > self.hora_despertar {
            (24 - self.hora_dormir) + self.hora_despertar
        } else {
            self.hora_despertar - self.hora_dormir
        };

        if duracion_total == 0 {
            return 0.0;
        }

        let hora_en_ciclo = if self.hora_dormir > self.hora_despertar {
            if hora_actual >= self.hora_dormir {
                hora_actual - self.hora_dormir
            } else {
                (24 - self.hora_dormir) + hora_actual
            }
        } else if hora_actual >= self.hora_dormir {
            hora_actual - self.hora_dormir
        } else {
            0
        };

        (hora_en_ciclo as f64 / duracion_total as f64).clamp(0.0, 1.0)
    }

    /// Información legible del estado actual del ciclo.
    pub fn estado_info(&self) -> String {
        let estado = match self.estado_actual {
            CicloCircadiano::Despierto => "☀️ Despierto",
            CicloCircadiano::Durmiendo => "🌙 Durmiendo",
        };
        format!(
            "{} — Horario: {}:00 → {}:00 | {}",
            estado,
            self.hora_dormir,
            self.hora_despertar,
            if self.sueno_pendiente_reporte {
                "Reporte pendiente al despertar"
            } else {
                "Estado estable"
            }
        )
    }
}

impl Default for CicloSueno {
    fn default() -> Self {
        Self::new(23, 7)
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estado_inicial_despierto() {
        let sueno = CicloSueno::new(23, 7);
        assert_eq!(sueno.estado_actual, CicloCircadiano::Despierto);
        assert!(!sueno.esta_durmiendo());
    }

    #[test]
    fn test_dormir_a_las_23_despertar_a_las_7() {
        let mut sueno = CicloSueno::new(23, 7);

        // A las 22 → despierto
        assert!(sueno.evaluar_ciclo(22).is_none());
        assert_eq!(sueno.estado_actual, CicloCircadiano::Despierto);

        // A las 23 → debe dormir (transición sin reporte)
        assert!(sueno.evaluar_ciclo(23).is_none());
        assert_eq!(sueno.estado_actual, CicloCircadiano::Durmiendo);
        assert!(sueno.esta_durmiendo());

        // A las 2 → sigue durmiendo
        assert!(sueno.evaluar_ciclo(2).is_none());
        assert_eq!(sueno.estado_actual, CicloCircadiano::Durmiendo);

        // A las 7 → debe despertar (reporte generado)
        let reporte = sueno.evaluar_ciclo(7);
        assert!(reporte.is_some());
        assert_eq!(sueno.estado_actual, CicloCircadiano::Despierto);

        let reporte = reporte.unwrap();
        assert!(reporte.descripcion.contains("Dormí"));
        assert!(reporte.memoria_consolidada);
        assert_eq!(reporte.duracion_horas, 8); // 23 → 7 = 8 horas
    }

    #[test]
    fn test_sin_transicion_si_horario_no_coincide() {
        let mut sueno = CicloSueno::new(22, 6);

        // A las 12 → despierto, no debe pasar nada
        for h in 6..22 {
            assert!(sueno.evaluar_ciclo(h).is_none());
            assert_eq!(sueno.estado_actual, CicloCircadiano::Despierto);
        }
    }

    #[test]
    fn test_dormir_sin_cruzar_medianoche() {
        let mut sueno = CicloSueno::new(2, 6); // Duerme de 2 AM a 6 AM

        assert!(sueno.evaluar_ciclo(2).is_none()); // Se duerme
        assert_eq!(sueno.estado_actual, CicloCircadiano::Durmiendo);

        assert!(sueno.evaluar_ciclo(4).is_none()); // Sigue durmiendo

        let reporte = sueno.evaluar_ciclo(6);
        assert!(reporte.is_some()); // Despierta
        assert_eq!(reporte.unwrap().duracion_horas, 4);
        assert_eq!(sueno.estado_actual, CicloCircadiano::Despierto);
    }

    #[test]
    fn test_fase_rem_solo_en_sueno() {
        let mut sueno = CicloSueno::new(23, 7);
        let mut consolido = false;

        // Despierto → fase REM no se ejecuta
        sueno.fase_rem(|| {
            consolido = true;
        });
        assert!(!consolido);

        // Dormir
        sueno.evaluar_ciclo(23);

        // Durmiendo → fase REM se ejecuta
        sueno.fase_rem(|| {
            consolido = true;
        });
        assert!(consolido);
    }

    #[test]
    fn test_siesta_misma_hora() {
        let mut sueno = CicloSueno::new(14, 14); // Siesta a las 14

        // A las 14 → debe dormir
        assert!(sueno.evaluar_ciclo(14).is_none());
        assert_eq!(sueno.estado_actual, CicloCircadiano::Durmiendo);

        // A las 15 → ya no debería dormir (pasó la ventana)
        let reporte = sueno.evaluar_ciclo(15);
        assert!(reporte.is_some());
        assert_eq!(sueno.estado_actual, CicloCircadiano::Despierto);
    }

    #[test]
    fn test_default_night() {
        let sueno = CicloSueno::default_night();
        assert_eq!(sueno.hora_dormir, 23);
        assert_eq!(sueno.hora_despertar, 7);
    }

    #[test]
    fn test_progreso_ciclo_completo() {
        let sueno = CicloSueno::new(22, 6); // 8 horas de sueño

        // A las 22 → 0% (empieza)
        let p0 = sueno.progreso_ciclo(22);
        assert!(p0 < 0.1);

        // A las 2 → 50%
        let p50 = sueno.progreso_ciclo(2);
        assert!((p50 - 0.5).abs() < 0.15);

        // A las 6 → 100%
        let p100 = sueno.progreso_ciclo(6);
        assert!((p100 - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_estado_info_formato() {
        let sueno = CicloSueno::new(23, 7);
        let info = sueno.estado_info();
        assert!(info.contains("Despierto"));
        assert!(info.contains("23:00"));
        assert!(info.contains("7:00"));
    }

    #[test]
    fn test_no_reporta_en_ciclo_sin_cambio() {
        let mut sueno = CicloSueno::new(23, 7);

        // Simular múltiples ticks en el mismo estado
        sueno.evaluar_ciclo(23); // Se duerme
        for h in &[0, 1, 2, 3, 4, 5, 6] {
            assert!(sueno.evaluar_ciclo(*h).is_none());
        }
    }

    #[test]
    fn test_horarios_validados() {
        let sueno = CicloSueno::new(99, 42); // Inválido, debe ser clamped
        assert!(sueno.hora_dormir <= 23);
        assert!(sueno.hora_despertar <= 23);
    }
}
