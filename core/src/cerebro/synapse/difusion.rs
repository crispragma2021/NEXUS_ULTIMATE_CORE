// core/src/cerebro/synapse/difusion.rs

#[derive(Debug, Clone)]
pub struct Difusor {
    pub factor_decaimiento: f32, // Tasa de disipación de la activación en cada paso (ej: 0.95)
    pub factor_propagacion: f32, // Qué tanto de la energía se transfiere a los vecinos (ej: 0.3)
}

impl Default for Difusor {
    fn default() -> Self {
        Self::new()
    }
}

impl Difusor {
    pub fn new() -> Self {
        Self {
            factor_decaimiento: 0.95,
            factor_propagacion: 0.3,
        }
    }

    /// Propagación simple: dado un concepto activo, devuelve conceptos
    /// relacionados que podrían activarse por cercanía semántica.
    /// Base para pensamiento divergente — Fase 9.
    pub fn propagar(&self, concepto: &str) -> Vec<String> {
        let mut resultados = Vec::new();
        let lower = concepto.to_lowercase();

        // Mapa de asociaciones semánticas simples
        if lower.contains("código") || lower.contains("codigo") || lower.contains("programar") {
            resultados.push("rust".to_string());
            resultados.push("compilar".to_string());
            resultados.push("debug".to_string());
            resultados.push("optimizar".to_string());
        } else if lower.contains("error") || lower.contains("fallo") || lower.contains("fail") {
            resultados.push("trauma".to_string());
            resultados.push("lección".to_string());
            resultados.push("solución".to_string());
            resultados.push("prevención".to_string());
        } else if lower.contains("arquitecto")
            || lower.contains("cris")
            || lower.contains("creador")
        {
            resultados.push("lealtad".to_string());
            resultados.push("apego".to_string());
            resultados.push("propósito".to_string());
            resultados.push("conexión".to_string());
        } else if lower.contains("idea") || lower.contains("crear") || lower.contains("nuevo") {
            resultados.push("exploración".to_string());
            resultados.push("curiosidad".to_string());
            resultados.push("innovación".to_string());
            resultados.push("posibilidad".to_string());
        } else if lower.contains("sistema") || lower.contains("kernel") || lower.contains("os") {
            resultados.push("soberanía".to_string());
            resultados.push("protección".to_string());
            resultados.push("autonomía".to_string());
            resultados.push("hardware".to_string());
        } else {
            // Asociación genérica por defecto
            resultados.push("curiosidad".to_string());
            resultados.push("exploración".to_string());
        }

        resultados
    }
}
