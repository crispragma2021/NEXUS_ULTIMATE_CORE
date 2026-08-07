// ==========================================
// MOTOR DE INDUCCIÓN SIMBÓLICA (MIS)
// ==========================================
// Extrae significado del texto usando HashMap
// y expresiones regulares. Sin IA generativa.
// ==========================================

use regex::Regex;
use std::collections::HashMap;

pub struct MotorSimbolico {
    lexico: HashMap<String, String>,
    patrones: Vec<(Regex, String)>,
}

impl Default for MotorSimbolico {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorSimbolico {
    pub fn new() -> Self {
        let mut lexico = HashMap::new();
        lexico.insert("hola".to_string(), "SALUDO".to_string());
        lexico.insert("nexus".to_string(), "IDENTIDAD_PROPIA".to_string());
        lexico.insert("crear".to_string(), "ACCION_CREAR".to_string());
        lexico.insert("buscar".to_string(), "ACCION_BUSCAR".to_string());
        lexico.insert("aprender".to_string(), "ACCION_APRENDER".to_string());
        lexico.insert("error".to_string(), "PROBLEMA".to_string());
        lexico.insert("gracias".to_string(), "GRATITUD".to_string());

        let mut patrones = Vec::new();
        patrones.push((
            Regex::new(r"(?i)qué es (.*)").unwrap(),
            "DEFINICION".to_string(),
        ));
        patrones.push((
            Regex::new(r"(?i)cómo (.*)").unwrap(),
            "PREGUNTA_PROCEDIMIENTO".to_string(),
        ));
        patrones.push((
            Regex::new(r"(?i)busca (.*)").unwrap(),
            "ORDEN_BUSCAR".to_string(),
        ));

        Self { lexico, patrones }
    }

    /// Analiza un texto y devuelve los conceptos detectados.
    pub fn analizar(&self, texto: &str, edad_mental: f64) -> Vec<String> {
        let mut conceptos = Vec::new();
        let lower = texto.to_lowercase();

        // Fase 1: Búsqueda léxica (siempre activa)
        for (palabra, concepto) in &self.lexico {
            if lower.contains(palabra) {
                conceptos.push(concepto.clone());
            }
        }

        // Fase 2: Patrones compuestos (edad_mental >= 0.7)
        if edad_mental >= 0.7 {
            for (patron, concepto) in &self.patrones {
                if patron.is_match(texto) {
                    conceptos.push(concepto.clone());
                }
            }
        }

        // Fase 3: Si no detecta nada y edad_mental < 0.7, pedir ayuda
        if conceptos.is_empty() && edad_mental < 0.7 {
            conceptos.push("DUDA".to_string());
        }

        conceptos
    }
}
