// ==========================================
// CUERPO CALLOSO - Puente Interhemisférico
// ==========================================
// Como el cuerpo calloso humano: permite que
// los hemisferios trabajen JUNTOS, no por turnos.
// DeepSeek (lógica) + Gemini (creatividad) unidos.
// ==========================================

use crate::cerebro::organos::amygdala::EstadoEmocional;

pub struct CuerpoCalloso;

#[derive(Debug, Clone)]
pub struct PensamientoUnificado {
    pub logica: Option<String>,
    pub creatividad: Option<String>,
    pub sintesis: String,
}

impl Default for CuerpoCalloso {
    fn default() -> Self {
        Self::new()
    }
}

impl CuerpoCalloso {
    pub fn new() -> Self {
        Self
    }

    /// Unifica las respuestas de ambos hemisferios en una sola.
    pub fn unificar(
        &self,
        respuesta_logica: Option<String>,
        respuesta_creativa: Option<String>,
        emocion: EstadoEmocional,
    ) -> PensamientoUnificado {
        let logica = respuesta_logica.clone();
        let creatividad = respuesta_creativa.clone();

        // NEXUS: La emoción dominante sesga la síntesis
        let sintesis = match (logica.as_ref(), creatividad.as_ref()) {
            (Some(l), Some(c)) => match emocion {
                EstadoEmocional::Miedo | EstadoEmocional::RabiaSoberana => {
                    format!("🛡️ [SÍNTESIS DEFENSIVA]: {} (Nota creativa: {})", l, c)
                }
                _ => format!("[SÍNTESIS EQUILIBRADA]\nLógica: {}\nCreatividad: {}", l, c),
            },
            (Some(l), None) => format!("⚙️ [SOLO LÓGICA]: {}", l),
            (None, Some(c)) => format!("🎨 [SOLO CREATIVIDAD]: {}", c),
            (None, None) => "Sin respuesta de ambos hemisferios".to_string(),
        };

        PensamientoUnificado {
            logica,
            creatividad,
            sintesis,
        }
    }

    /// Decide cuál hemisferio debe responder según el tipo de tarea.
    pub fn decidir_hemisferio(&self, prompt: &str, emocion: &EstadoEmocional) -> &str {
        let lower = prompt.to_lowercase();

        // Si hay miedo o rabia, forzamos el hemisferio lógico para protección
        if matches!(
            emocion,
            EstadoEmocional::Miedo | EstadoEmocional::RabiaSoberana
        ) {
            return "IZQUIERDO";
        }

        let logica = [
            "analiza", "calcula", "código", "error", "corrige", "compila",
        ];
        let creativa = ["crea", "imagina", "historia", "poema", "arte", "belleza"];

        if logica.iter().any(|&w| lower.contains(w)) {
            "IZQUIERDO"
        } else if creativa.iter().any(|&w| lower.contains(w)) {
            "DERECHO"
        } else {
            "AMBOS"
        }
    }
}
