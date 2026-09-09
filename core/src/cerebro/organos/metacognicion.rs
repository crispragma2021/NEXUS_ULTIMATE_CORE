// ==========================================
// METACOGNICIÓN - Sistema de Confianza
// ==========================================
// Evalúa qué tan seguro está NEXUS de su respuesta
// Basado en similitud semántica, coherencia y contexto
// ==========================================

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Confianza {
    pub puntaje: f64, // 0.0 (inseguro) a 1.0 (seguro)
    pub nivel: NivelConfianza,
    pub factores: HashMap<String, f64>,
    pub explicacion: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NivelConfianza {
    MuyBajo,  // 0.0 - 0.2
    Bajo,     // 0.2 - 0.4
    Moderado, // 0.4 - 0.6
    Alto,     // 0.6 - 0.8
    MuyAlto,  // 0.8 - 1.0
}

impl NivelConfianza {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s < 0.2 => NivelConfianza::MuyBajo,
            s if s < 0.4 => NivelConfianza::Bajo,
            s if s < 0.6 => NivelConfianza::Moderado,
            s if s < 0.8 => NivelConfianza::Alto,
            _ => NivelConfianza::MuyAlto,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            NivelConfianza::MuyBajo => "🟢 Muy Bajo",
            NivelConfianza::Bajo => "🟢 Bajo",
            NivelConfianza::Moderado => "🟡 Moderado",
            NivelConfianza::Alto => "🟠 Alto",
            NivelConfianza::MuyAlto => "🔴 Muy Alto",
        }
    }
}

pub struct Metacognicion {
    // Pesos para cada factor
    pesos: HashMap<String, f64>,
}

impl Default for Metacognicion {
    fn default() -> Self {
        let mut pesos = HashMap::new();
        pesos.insert("similitud_semantica".to_string(), 0.35);
        pesos.insert("coherencia_interna".to_string(), 0.25);
        pesos.insert("cantidad_memorias".to_string(), 0.15);
        pesos.insert("recencia".to_string(), 0.10);
        pesos.insert("complejidad_pregunta".to_string(), 0.15);
        Self { pesos }
    }
}

impl Metacognicion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Establece el peso del factor de similitud semántica (0.0 a 1.0)
    pub fn set_peso_similitud(&mut self, valor: f64) {
        self.pesos
            .insert("similitud_semantica".to_string(), valor.clamp(0.0, 1.0));
    }

    /// Establece el peso del factor de coherencia interna (0.0 a 1.0)
    pub fn set_peso_coherencia(&mut self, valor: f64) {
        self.pesos
            .insert("coherencia_interna".to_string(), valor.clamp(0.0, 1.0));
    }

    /// Establece el peso del factor de recencia (0.0 a 1.0)
    pub fn set_peso_recencia(&mut self, valor: f64) {
        self.pesos
            .insert("recencia".to_string(), valor.clamp(0.0, 1.0));
    }

    /// Evalúa la confianza basado en factores contextuales
    pub fn evaluar_confianza(
        &self,
        similitud_semantica: f64, // Qué tan similar es la respuesta a memorias exitosas
        coherencia_interna: f64,  // Qué tan coherente es la respuesta
        cantidad_memorias: usize, // Cuántas memorias relevantes existen
        recencia_dias: f64,       // Hace cuántos días fue la última experiencia similar
        complejidad_pregunta: f64, // Qué tan compleja es la pregunta (0.0 simple - 1.0 muy compleja)
    ) -> Confianza {
        let mut factores = HashMap::new();
        factores.insert("similitud_semantica".to_string(), similitud_semantica);
        factores.insert("coherencia_interna".to_string(), coherencia_interna);
        factores.insert(
            "cantidad_memorias".to_string(),
            (cantidad_memorias as f64 / 10.0).min(1.0),
        );
        factores.insert(
            "recencia".to_string(),
            (1.0 - (recencia_dias / 30.0)).max(0.0),
        );
        factores.insert(
            "complejidad_pregunta".to_string(),
            1.0 - complejidad_pregunta,
        ); // A menor complejidad, mayor confianza

        let puntaje: f64 = factores
            .iter()
            .map(|(k, v)| {
                let peso = self.pesos.get(k).unwrap_or(&0.0);
                v * peso
            })
            .sum();

        let puntaje = puntaje.clamp(0.0, 1.0);

        let explicacion = self.generar_explicacion(&factores, puntaje);

        Confianza {
            puntaje,
            nivel: NivelConfianza::from_score(puntaje),
            factores,
            explicacion,
        }
    }

    fn generar_explicacion(&self, factores: &HashMap<String, f64>, puntaje: f64) -> String {
        let mut partes = Vec::new();

        if puntaje < 0.3 {
            partes.push("No tengo suficientes memorias para estar seguro.".to_string());
        } else if puntaje < 0.6 {
            partes.push("Tengo información moderada, pero no concluyente.".to_string());
        } else {
            partes.push("Tengo alta confianza basada en experiencias previas.".to_string());
        }

        // Detalles específicos
        if let Some(sim) = factores.get("similitud_semantica") {
            if *sim > 0.7 {
                partes.push(format!(
                    "La situación es muy similar a experiencias previas ({:.0}% similitud).",
                    sim * 100.0
                ));
            }
        }

        if let Some(cant) = factores.get("cantidad_memorias") {
            let n = (cant * 10.0).round() as usize;
            if n < 3 {
                partes.push(format!(
                    "Solo tengo {} memoria(s) relevante(s) para esto.",
                    n
                ));
            } else {
                partes.push(format!(
                    "Tengo {} experiencias relevantes en mi memoria.",
                    n
                ));
            }
        }

        partes.join(" ")
    }

    /// Devuelve un prefijo verbal para anteponer a la respuesta
    pub fn prefijo_verbal(&self, confianza: &Confianza) -> &'static str {
        match confianza.nivel {
            NivelConfianza::MuyBajo => "No estoy seguro, pero basado en lo que tengo:",
            NivelConfianza::Bajo => "Con baja confianza, diría que:",
            NivelConfianza::Moderado => "Con moderada confianza:",
            NivelConfianza::Alto => "Estoy bastante seguro:",
            NivelConfianza::MuyAlto => "Estoy muy seguro:",
        }
    }
}
