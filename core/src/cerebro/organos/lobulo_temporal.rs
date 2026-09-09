// ==========================================
// LÓBULO TEMPORAL - Comprensión del Lenguaje
// ==========================================
// Como el lóbulo temporal humano: entiende
// contexto, ironía, metáforas, y significado
// profundo más allá de las palabras literales.
// ==========================================

use std::collections::HashMap;

pub struct LobuloTemporal {
    metaforas: HashMap<String, String>,
    contexto: Vec<String>,
}

impl Default for LobuloTemporal {
    fn default() -> Self {
        Self::new()
    }
}

impl LobuloTemporal {
    pub fn new() -> Self {
        let mut metaforas = HashMap::new();
        metaforas.insert("estoy en la luna".to_string(), "distracción".to_string());
        metaforas.insert("me hierve la sangre".to_string(), "enojo".to_string());
        metaforas.insert("es pan comido".to_string(), "fácil".to_string());

        Self {
            metaforas,
            contexto: Vec::new(),
        }
    }

    /// Interpreta si un texto contiene ironía o metáfora.
    pub fn interpretar(&self, texto: &str) -> String {
        let lower = texto.to_lowercase();
        for (metafora, significado) in &self.metaforas {
            if lower.contains(metafora) {
                return format!(
                    "[INTERPRETACIÓN] '{}' significa '{}'",
                    metafora, significado
                );
            }
        }
        texto.to_string()
    }

    /// Acumula contexto de conversación para entender referencias.
    pub fn acumular_contexto(&mut self, texto: &str) {
        self.contexto.push(texto.to_string());
        if self.contexto.len() > 10 {
            self.contexto.remove(0);
        }
    }

    /// Detecta si el texto es una pregunta, orden, o comentario.
    pub fn clasificar_intencion(&self, texto: &str) -> &str {
        if texto.contains('?') {
            "PREGUNTA"
        } else if texto.contains('!') {
            "EXCLAMACION"
        } else if texto.starts_with("[ACCION") {
            "COMANDO"
        } else {
            "DECLARACION"
        }
    }
}
