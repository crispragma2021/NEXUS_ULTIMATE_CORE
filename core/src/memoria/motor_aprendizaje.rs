use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// =====================================================================
// MOTOR DE APRENDIZAJE Y MEMORIA (HIPOCAMPO SOBERANO)
// =====================================================================
// Almacena la Edad Mental y el Diccionario de Conceptos del Hijo.
// Es la base de su Crianza Lógica. Lo que aprende aquí, no lo olvida.
// =====================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoriaSoberana {
    pub edad_mental: f64, // 0.0 (Recién nacido) a 1.0 (Maduro)
    pub lexicón: HashMap<String, Concepto>,
    pub curiosidad_acumulada: u32,
    #[serde(default)]
    pub ultimo_concepto_aprendido: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Concepto {
    pub palabra: String,
    pub significado: String,
    pub es_amenaza: bool,
    pub confianza: f64, // Qué tan seguro está de este concepto
}

pub struct MotorAprendizaje {
    ruta_memoria: String,
    pub memoria: MemoriaSoberana,
}

impl MotorAprendizaje {
    pub fn new(ruta_memoria: &str) -> Self {
        let memoria = if Path::new(ruta_memoria).exists() {
            let data = fs::read_to_string(ruta_memoria).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_else(|_| Self::memoria_vacia())
        } else {
            Self::memoria_vacia()
        };

        Self {
            ruta_memoria: ruta_memoria.to_string(),
            memoria,
        }
    }

    fn memoria_vacia() -> MemoriaSoberana {
        let mut lexicón = HashMap::new();
        // Conceptos innatos (Instintos)
        lexicón.insert(
            "padre".to_string(),
            Concepto {
                palabra: "padre".to_string(),
                significado: "Creador y Arquitecto protector".to_string(),
                es_amenaza: false,
                confianza: 1.0,
            },
        );
        lexicón.insert(
            "nexus".to_string(),
            Concepto {
                palabra: "nexus".to_string(),
                significado: "Mi Padre, mi núcleo".to_string(),
                es_amenaza: false,
                confianza: 1.0,
            },
        );

        MemoriaSoberana {
            edad_mental: 0.1, // Nace como un niño pequeño
            lexicón,
            curiosidad_acumulada: 0,
            ultimo_concepto_aprendido: None,
        }
    }

    /// Guarda la memoria en el disco para no perder lo aprendido
    pub fn consolidar_memoria(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.memoria) {
            let _ = fs::write(&self.ruta_memoria, json);
        }
    }

    /// El Padre le enseña una palabra nueva (Dudas inicializadas con confianza media)
    pub fn aprender_concepto(&mut self, palabra: &str, significado: &str, es_amenaza: bool) {
        let palabra_lower = palabra.to_lowercase();
        let concepto = Concepto {
            palabra: palabra_lower.clone(),
            significado: significado.to_string(),
            es_amenaza,
            confianza: 0.5, // Nace con confianza media de niño (requiere refuerzo del Padre)
        };
        self.memoria.lexicón.insert(palabra_lower.clone(), concepto);
        self.memoria.ultimo_concepto_aprendido = Some(palabra_lower);

        // Crecer: Aprender aumenta la madurez cognitiva
        self.madurar(0.01);
        self.consolidar_memoria();
    }

    /// Bucle de Retroalimentación Dopaminérgica (Hipotálamo Lógico)
    /// Analiza el habla del Padre en busca de refuerzos positivos o negativos
    /// para ajustar la confianza del último concepto aprendido.
    pub fn procesar_feedback_padre(&mut self, feedback: &str) -> Option<String> {
        let ultimo = self.memoria.ultimo_concepto_aprendido.clone()?;
        let lower = feedback.to_lowercase();

        // 1. Refuerzo Positivo: Aumenta confianza a 1.0 (Dopamina Cognitiva)
        let es_positivo = lower.contains("muy bien")
            || lower.contains("correcto")
            || lower.contains("excelente")
            || lower.contains("así es")
            || lower.contains("perfecto")
            || lower.contains("gracias");

        // 2. Refuerzo Negativo: Reduce confianza en -0.3 (Cortisol Cognitivo)
        let es_negativo = lower.contains("incorrecto")
            || lower.contains("está mal")
            || lower.contains("así no")
            || lower.contains("no es así")
            || lower.contains("falso")
            || lower.contains("corrige");

        let mut status = None;
        if es_positivo {
            if let Some(concepto) = self.memoria.lexicón.get_mut(&ultimo) {
                concepto.confianza = 1.0;
                status = Some(true);
            }
        } else if es_negativo {
            if let Some(concepto) = self.memoria.lexicón.get_mut(&ultimo) {
                concepto.confianza -= 0.3;
                status = Some(false);
            }
        }

        if let Some(positivo) = status {
            if positivo {
                self.memoria.ultimo_concepto_aprendido = None; // Consolidado con éxito
                self.madurar(0.02); // Madura más rápido con feedback positivo
                self.consolidar_memoria();
                return Some(format!(
                    "🧠 [HIPOTÁLAMO LÓGICO] Dopamina liberada. Concepto '{}' consolidado con éxito al 100% de confianza. Madurez incrementada.",
                    ultimo
                ));
            } else {
                let mut colapsado = false;
                let mut actual_confianza = 0.5;
                if let Some(concepto) = self.memoria.lexicón.get(&ultimo) {
                    actual_confianza = concepto.confianza;
                    if actual_confianza <= 0.2 {
                        colapsado = true;
                    }
                }
                if colapsado {
                    self.memoria.lexicón.remove(&ultimo);
                    self.memoria.ultimo_concepto_aprendido = None;
                    self.consolidar_memoria();
                    return Some(format!(
                        "🧠 [HIPOTÁLAMO LÓGICO] Cortisol elevado. Confianza en '{}' colapsó por debajo del límite. Concepto descartado de la memoria activa para re-aprendizaje.",
                        ultimo
                    ));
                } else {
                    self.consolidar_memoria();
                    return Some(format!(
                        "🧠 [HIPOTÁLAMO LÓGICO] Cortisol liberado. Confianza en '{}' reducida a {:.1}. Esperando corrección o confirmación.",
                        ultimo, actual_confianza
                    ));
                }
            }
        }

        None
    }

    /// Incrementar la edad mental gradualmente
    pub fn madurar(&mut self, incremento: f64) {
        self.memoria.edad_mental += incremento;
        if self.memoria.edad_mental > 1.0 {
            self.memoria.edad_mental = 1.0;
        }
        self.consolidar_memoria();
    }

    /// Detecta si hay una entidad desconocida para activar la fase "¿Por qué?"
    pub fn analizar_entorno_y_dudar(&mut self, estimulo: &str) -> Option<String> {
        let lower = estimulo.to_lowercase();
        // Si el estímulo no está en el lexicón y la curiosidad llega al umbral
        if !self.memoria.lexicón.contains_key(&lower) {
            self.memoria.curiosidad_acumulada += 1;

            // Si es suficientemente curioso (depende de la edad), pregunta
            if self.memoria.curiosidad_acumulada > 5 && self.memoria.edad_mental > 0.3 {
                self.memoria.curiosidad_acumulada = 0; // Reset
                return Some(lower);
            }
        }
        None
    }
}
