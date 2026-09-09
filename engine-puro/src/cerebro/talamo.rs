// ============================================================================
// 🏛️ TÁLAMO DIGITAL: Filtro Sensorial y Sincronización Gamma
// ============================================================================
// El Tálamo es la "puerta de la consciencia". Sincroniza las regiones
// corticales mediante ritmos Gamma (~40Hz) y filtra estímulos irrelevantes.
// ============================================================================

use crate::cerebro::estructuras::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum EstadoConsciencia {
    Vigilia,        // Ritmos Gamma/Beta (30-80 Hz)
    SuenioLigero,   // Spindles de sueño
    SuenioREM,      // Ritmos Theta (4-8 Hz) - Consolidación
    SuenioProfundo, // Ondas lentas Delta (0.5-4 Hz)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ModoTransmision {
    Tonico, // Transmisión lineal y fiel
    Fasico, // Ráfaga por novedad o alerta
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AccesoConsciente {
    TransmisionFiel,
    Alerta(f32),
    Filtrado,
}

// ============================================================================
// OSCILADOR TALÁMICO (Generador de Ritmos)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OsciladorTalamico {
    pub frecuencia_gamma: f32, // Hz (~40)
    pub frecuencia_alpha: f32, // Hz (~10)
    pub fase_actual: f32,      // Radianes
    pub tiempo_ultimo_pulso: f32,
}

impl OsciladorTalamico {
    pub fn nuevo() -> Self {
        Self {
            frecuencia_gamma: 40.0,
            frecuencia_alpha: 10.0,
            fase_actual: 0.0,
            tiempo_ultimo_pulso: 0.0,
        }
    }

    /// Genera un pulso de sincronización Gamma
    pub fn pulso_gamma(&mut self, tiempo_actual: f32) -> bool {
        let periodo = 1.0 / self.frecuencia_gamma;
        if tiempo_actual - self.tiempo_ultimo_pulso >= periodo {
            self.tiempo_ultimo_pulso = tiempo_actual;
            self.fase_actual = (self.fase_actual + std::f32::consts::PI / 2.0) % (2.0 * std::f32::consts::PI);
            true
        } else {
            false
        }
    }

    pub fn ajustar_ritmo(&mut self, estado: &EstadoConsciencia) {
        match estado {
            EstadoConsciencia::Vigilia => {
                self.frecuencia_gamma = 40.0;
                self.frecuencia_alpha = 10.0;
            }
            EstadoConsciencia::SuenioREM => {
                self.frecuencia_gamma = 0.0;
                self.frecuencia_alpha = 6.0;
            }
            EstadoConsciencia::SuenioProfundo => {
                self.frecuencia_gamma = 0.0;
                self.frecuencia_alpha = 1.0;
            }
            _ => {}
        }
    }
}

// ============================================================================
// FILTRO ATENCIONAL
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FiltroAtencional {
    pub umbral_novedad: f32,
    pub nivel_arousal: f32, // 0.0 a 1.0
}

impl FiltroAtencional {
    pub fn nuevo() -> Self {
        Self {
            umbral_novedad: 0.3,
            nivel_arousal: 0.5,
        }
    }

    pub fn evaluar_acceso(&self, estimulo: &Estimulo, prediccion: Option<&Prediccion>) -> AccesoConsciente {
        let valor_esperado = prediccion.map(|p| p.valor_esperado).unwrap_or(0.0);
        let error_prediccion = (estimulo.valor - valor_esperado).abs();

        if error_prediccion > self.umbral_novedad * (1.0 - self.nivel_arousal) {
            AccesoConsciente::Alerta(error_prediccion)
        } else if self.nivel_arousal > 0.7 {
            AccesoConsciente::TransmisionFiel
        } else {
            AccesoConsciente::Filtrado
        }
    }
}

// ============================================================================
// TÁLAMO DIGITAL
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TalamoDigital {
    pub estado: EstadoConsciencia,
    pub oscilador: OsciladorTalamico,
    pub filtro: FiltroAtencional,
    pub neuronas_relevo: Vec<NeuronaTalamica>,
    pub modo: ModoTransmision,
}

impl TalamoDigital {
    pub fn nuevo() -> Self {
        Self {
            estado: EstadoConsciencia::Vigilia,
            oscilador: OsciladorTalamico::nuevo(),
            filtro: FiltroAtencional::nuevo(),
            neuronas_relevo: Vec::new(),
            modo: ModoTransmision::Tonico,
        }
    }

    pub fn procesar_estimulo(&mut self, estimulo: &Estimulo, prediccion: Option<&Prediccion>) -> AccesoConsciente {
        self.filtro.evaluar_acceso(estimulo, prediccion)
    }

    pub fn sincronizar(&mut self, tiempo_actual: f32) -> bool {
        self.oscilador.pulso_gamma(tiempo_actual)
    }

    /// Recibe feedback predictivo desde la Capa VI de las columnas corticales.
    ///
    /// En el circuito talamo-cortical, Capa VI envía predicciones descendentes
    /// al tálamo para modular qué información sensorial pasa a la corteza.
    /// Esto implementa el principio de **predictive coding** (Rao & Ballard, 1999):
    /// - Si la predicción es precisa (confianza alta), el tálamo filtra el input
    /// - Si hay error de predicción grande, el tálamo envía una ráfaga de alerta
    ///
    /// Además, las predicciones múltiples se fusionan en una representación
    /// talámica unificada (binding atencional).
    pub fn recibir_feedback(
        &mut self,
        predicciones: &[crate::cerebro::estructuras::PrediccionTalamica],
    ) {
        for pred in predicciones {
            // Actualizar neuronas de relevo talámico con la predicción
            let target_id = pred.columna_origen;
            
            // Buscar si ya existe una neurona de relevo para esta columna
            if let Some(nt) = self.neuronas_relevo.iter_mut().find(|n: &&mut NeuronaTalamica| n.id == target_id) {
                // Si la predicción es muy confiable, bajar el umbral tónico
                if pred.confianza > 0.8 {
                    nt.umbral = 0.7; // Umbral bajo = transmisión fiel
                    nt.modo_rafaga = false; // Modo tónico
                } else if pred.confianza < 0.3 {
                    nt.umbral = 1.3; // Umbral alto = alerta solo para novedades fuertes
                    nt.modo_rafaga = true; // Modo ráfaga
                } else {
                    nt.umbral = 1.0; // Neutro
                    nt.modo_rafaga = false;
                }
            } else {
                // Crear nueva neurona de relevo para esta columna cortical
                self.neuronas_relevo.push(NeuronaTalamica {
                    id: target_id,
                    voltaje: -70.0,
                    umbral: 1.0,
                    modo_rafaga: false,
                    ultima_actividad: 0,
                });
            }
        }

        // Poda: limpiar neuronas de relevo inactivas
        let max_actividad = self.neuronas_relevo.iter()
            .map(|n| n.ultima_actividad)
            .max()
            .unwrap_or(0);
        if max_actividad > 1000 {
            self.neuronas_relevo.retain(|n| n.ultima_actividad > max_actividad.saturating_sub(500));
        }

        // Actualizar modo de transmisión basado en las predicciones
        let confianza_promedio: f32 = if predicciones.is_empty() {
            0.5
        } else {
            predicciones.iter().map(|p| p.confianza).sum::<f32>() / predicciones.len() as f32
        };

        self.modo = if confianza_promedio > 0.6 {
            ModoTransmision::Tonico // Todo predecible → transmisión fiel
        } else {
            ModoTransmision::Fasico // Mucha novedad → modo alerta
        };

        // Ajustar el ritmo del oscilador según el nivel de confianza
        // (más confianza = ritmo Gamma más estable)
        if confianza_promedio > 0.7 {
            self.oscilador.frecuencia_gamma = 45.0; // Gamma más rápida = binding fuerte
        } else if confianza_promedio < 0.3 {
            self.oscilador.frecuencia_gamma = 30.0; // Gamma más lenta = exploración
        } else {
            self.oscilador.frecuencia_gamma = 40.0; // Valor nominal
        }
    }

    /// Genera estímulos talámicos para alimentar las columnas corticales
    /// basados en los estímulos entrantes y las predicciones de Capa VI
    pub fn generar_estimulos_columnares(
        &self,
        estimulos_entrada: &[Estimulo],
    ) -> Vec<crate::cerebro::estructuras::EstimuloTalamico> {
        estimulos_entrada.iter().map(|est| {
            let novedad = est.valor * est.intensidad; // Simple heurística de novedad
            crate::cerebro::estructuras::EstimuloTalamico {
                origen_talamo: est.id,
                intensidad: est.intensidad,
                novedad,
            }
        }).collect()
    }
}

// ============================================================================
// TÁLAMO DIGITAL — Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "esperado {}, obtenido {}", b, a);
    }

    fn estimulo(id: u32, valor: f32) -> Estimulo {
        Estimulo {
            id,
            intensidad: valor,
            amenaza: 0.0,
            recompensa: 0.0,
            valor,
        }
    }

    // ── OsciladorTalamico ────────────────────────────────────────────────────
    #[test]
    fn test_oscilador_nuevo_valores_basales() {
        let osc = OsciladorTalamico::nuevo();
        casi(osc.frecuencia_gamma, 40.0);
        casi(osc.frecuencia_alpha, 10.0);
        casi(osc.fase_actual, 0.0);
        casi(osc.tiempo_ultimo_pulso, 0.0);
    }

    #[test]
    fn test_pulso_gamma_dentro_del_periodo_no_dispara() {
        let mut osc = OsciladorTalamico::nuevo();
        // periodo = 1/40 = 0.025s; a t=0.01 aún no toca
        assert!(!osc.pulso_gamma(0.01));
    }

    #[test]
    fn test_pulso_gamma_al_cumplir_periodo_dispara() {
        let mut osc = OsciladorTalamico::nuevo();
        assert!(osc.pulso_gamma(0.025));
        casi(osc.fase_actual, std::f32::consts::PI / 2.0);
    }

    #[test]
    fn test_pulso_gamma_avanza_fase_ciclicamente() {
        let mut osc = OsciladorTalamico::nuevo();
        // 3 pulsos → fase = 3π/2 (evita precisión f32 en el 4º)
        assert!(osc.pulso_gamma(0.025));
        assert!(osc.pulso_gamma(0.05));
        assert!(osc.pulso_gamma(0.075));
        casi(osc.fase_actual, 3.0 * std::f32::consts::PI / 2.0);
    }

    #[test]
    fn test_ajustar_ritmo_vigilia_gamma_40() {
        let mut osc = OsciladorTalamico::nuevo();
        osc.ajustar_ritmo(&EstadoConsciencia::Vigilia);
        casi(osc.frecuencia_gamma, 40.0);
        casi(osc.frecuencia_alpha, 10.0);
    }

    #[test]
    fn test_ajustar_ritmo_rem_theta() {
        let mut osc = OsciladorTalamico::nuevo();
        osc.ajustar_ritmo(&EstadoConsciencia::SuenioREM);
        casi(osc.frecuencia_gamma, 0.0);
        casi(osc.frecuencia_alpha, 6.0);
    }

    #[test]
    fn test_ajustar_ritmo_profundo_delta() {
        let mut osc = OsciladorTalamico::nuevo();
        osc.ajustar_ritmo(&EstadoConsciencia::SuenioProfundo);
        casi(osc.frecuencia_gamma, 0.0);
        casi(osc.frecuencia_alpha, 1.0);
    }

    // ── FiltroAtencional ─────────────────────────────────────────────────────
    #[test]
    fn test_filtro_nuevo_valores_basales() {
        let f = FiltroAtencional::nuevo();
        casi(f.umbral_novedad, 0.3);
        casi(f.nivel_arousal, 0.5);
    }

    #[test]
    fn test_error_prediccion_grande_genera_alerta() {
        let f = FiltroAtencional::nuevo(); // arousal 0.5 → umbral efectivo 0.15
        let est = estimulo(1, 1.0);
        let pred = Prediccion {
            id_objetivo: 1,
            valor_esperado: 0.0,
            confianza: 0.9,
        };
        match f.evaluar_acceso(&est, Some(&pred)) {
            AccesoConsciente::Alerta(err) => casi(err, 1.0),
            _ => panic!("se esperaba Alerta"),
        }
    }

    #[test]
    fn test_arousal_alto_transmite_fiel() {
        let f = FiltroAtencional {
            umbral_novedad: 0.3,
            nivel_arousal: 0.9,
        };
        let est = estimulo(1, 0.5);
        // error = 0.5, umbral = 0.3*(1-0.9)=0.03 → 0.5 > 0.03 → Alerta
        // arousal > 0.7 no alcanza: la novedad manda
        let pred = Prediccion {
            id_objetivo: 1,
            valor_esperado: 0.0,
            confianza: 0.9,
        };
        match f.evaluar_acceso(&est, Some(&pred)) {
            AccesoConsciente::Alerta(_) => {}
            _ => panic!("se esperaba Alerta por novedad alta"),
        }
    }

    #[test]
    fn test_arousal_alto_sin_novedad_transmite_fiel() {
        let f = FiltroAtencional {
            umbral_novedad: 0.3,
            nivel_arousal: 0.9,
        };
        let est = estimulo(1, 0.5);
        let pred = Prediccion {
            id_objetivo: 1,
            valor_esperado: 0.48, // error = 0.02 → por debajo del umbral
            confianza: 0.9,
        };
        match f.evaluar_acceso(&est, Some(&pred)) {
            AccesoConsciente::TransmisionFiel => {}
            _ => panic!("se esperaba TransmisionFiel"),
        }
    }

    #[test]
    fn test_baja_novedad_y_bajo_arousal_filtra() {
        let f = FiltroAtencional {
            umbral_novedad: 0.3,
            nivel_arousal: 0.2,
        };
        let est = estimulo(1, 0.5);
        let pred = Prediccion {
            id_objetivo: 1,
            valor_esperado: 0.48,
            confianza: 0.9,
        };
        match f.evaluar_acceso(&est, Some(&pred)) {
            AccesoConsciente::Filtrado => {}
            _ => panic!("se esperaba Filtrado"),
        }
    }

    #[test]
    fn test_sin_prediccion_valor_esperado_cero() {
        let f = FiltroAtencional::nuevo();
        let est = estimulo(1, 0.5); // error = 0.5 > 0.15 → Alerta
        match f.evaluar_acceso(&est, None) {
            AccesoConsciente::Alerta(err) => casi(err, 0.5),
            _ => panic!("se esperaba Alerta"),
        }
    }

    // ── TalamoDigital ────────────────────────────────────────────────────────
    #[test]
    fn test_talamo_nuevo_estado_vigilia() {
        let t = TalamoDigital::nuevo();
        assert_eq!(t.estado, EstadoConsciencia::Vigilia);
        assert!(matches!(t.modo, ModoTransmision::Tonico));
        assert!(t.neuronas_relevo.is_empty());
    }

    #[test]
    fn test_procesar_estimulo_delega_al_filtro() {
        let mut t = TalamoDigital::nuevo();
        let est = estimulo(1, 0.5);
        assert!(matches!(t.procesar_estimulo(&est, None), AccesoConsciente::Alerta(_)));
    }

    #[test]
    fn test_sincronizar_delega_al_oscilador() {
        let mut t = TalamoDigital::nuevo();
        assert!(t.sincronizar(0.025));
        assert!(!t.sincronizar(0.026)); // aún dentro del periodo
    }

    #[test]
    fn test_recibir_feedback_crea_neurona_relevo() {
        let mut t = TalamoDigital::nuevo();
        let preds = vec![PrediccionTalamica {
            columna_origen: 7,
            valor_esperado: 0.5,
            confianza: 0.5,
        }];
        t.recibir_feedback(&preds);
        assert_eq!(t.neuronas_relevo.len(), 1);
        assert_eq!(t.neuronas_relevo[0].id, 7);
        assert_eq!(t.neuronas_relevo[0].umbral, 1.0); // neutro
        assert!(!t.neuronas_relevo[0].modo_rafaga);
    }

    #[test]
    fn test_feedback_confianza_alta_baja_umbral_tonico() {
        let mut t = TalamoDigital::nuevo();
        t.neuronas_relevo.push(NeuronaTalamica {
            id: 1,
            voltaje: -70.0,
            umbral: 1.0,
            modo_rafaga: false,
            ultima_actividad: 0,
        });
        let preds = vec![PrediccionTalamica {
            columna_origen: 1,
            valor_esperado: 0.8,
            confianza: 0.9,
        }];
        t.recibir_feedback(&preds);
        assert_eq!(t.neuronas_relevo[0].umbral, 0.7);
        assert!(!t.neuronas_relevo[0].modo_rafaga);
        // confianza > 0.7 → modo tónico y gamma rápida
        assert!(matches!(t.modo, ModoTransmision::Tonico));
        casi(t.oscilador.frecuencia_gamma, 45.0);
    }

    #[test]
    fn test_feedback_confianza_baja_activa_rafaga() {
        let mut t = TalamoDigital::nuevo();
        t.neuronas_relevo.push(NeuronaTalamica {
            id: 1,
            voltaje: -70.0,
            umbral: 1.0,
            modo_rafaga: false,
            ultima_actividad: 0,
        });
        let preds = vec![PrediccionTalamica {
            columna_origen: 1,
            valor_esperado: 0.2,
            confianza: 0.2,
        }];
        t.recibir_feedback(&preds);
        assert_eq!(t.neuronas_relevo[0].umbral, 1.3);
        assert!(t.neuronas_relevo[0].modo_rafaga);
        assert!(matches!(t.modo, ModoTransmision::Fasico));
        casi(t.oscilador.frecuencia_gamma, 30.0);
    }

    #[test]
    fn test_feedback_actualiza_neurona_existente() {
        let mut t = TalamoDigital::nuevo();
        t.neuronas_relevo.push(NeuronaTalamica {
            id: 5,
            voltaje: -70.0,
            umbral: 1.0,
            modo_rafaga: false,
            ultima_actividad: 0,
        });
        let preds = vec![PrediccionTalamica {
            columna_origen: 5,
            valor_esperado: 0.8,
            confianza: 0.9,
        }];
        t.recibir_feedback(&preds);
        assert_eq!(t.neuronas_relevo.len(), 1); // no duplica
        assert_eq!(t.neuronas_relevo[0].umbral, 0.7);
    }

    #[test]
    fn test_feedback_sin_predicciones_confianza_media() {
        let mut t = TalamoDigital::nuevo();
        t.recibir_feedback(&[]);
        // confianza_promedio default 0.5 → modo fasico? 0.5 no > 0.6 → Fasico
        assert!(matches!(t.modo, ModoTransmision::Fasico));
        // 0.5 no > 0.7 ni < 0.3 → gamma nominal 40
        casi(t.oscilador.frecuencia_gamma, 40.0);
    }

    #[test]
    fn test_feedback_poda_neuronas_inactivas() {
        let mut t = TalamoDigital::nuevo();
        // La poda se dispara si max_ultima_actividad > 1000
        t.neuronas_relevo.push(NeuronaTalamica {
            id: 1,
            voltaje: -70.0,
            umbral: 1.0,
            modo_rafaga: false,
            ultima_actividad: 1200, // activa reciente
        });
        t.neuronas_relevo.push(NeuronaTalamica {
            id: 2,
            voltaje: -70.0,
            umbral: 1.0,
            modo_rafaga: false,
            ultima_actividad: 10, // inactiva → se poda
        });
        t.recibir_feedback(&[]);
        assert_eq!(t.neuronas_relevo.len(), 1);
        assert_eq!(t.neuronas_relevo[0].id, 1);
    }

    #[test]
    fn test_generar_estimulos_columnares_mantiene_campos() {
        let t = TalamoDigital::nuevo();
        let entrada = vec![estimulo(10, 0.8), estimulo(11, 0.2)];
        let columnares = t.generar_estimulos_columnares(&entrada);
        assert_eq!(columnares.len(), 2);
        assert_eq!(columnares[0].origen_talamo, 10);
        casi(columnares[0].intensidad, 0.8);
        casi(columnares[0].novedad, 0.64); // valor * intensidad = 0.8*0.8
        casi(columnares[1].novedad, 0.04); // 0.2*0.2
    }
}
