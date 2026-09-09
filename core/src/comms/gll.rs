// ==========================================
// GENERADOR DE LENGUAJE LÓGICO (GLL)
// ==========================================
// Traduce el estado interno de NEXUS a lenguaje
// natural usando lógica booleana y plantillas.
// Sin redes neuronales. < 5MB de RAM.
// ==========================================

pub struct GLL;

impl GLL {
    /// Genera una respuesta basada en el estado interno de NEXUS.
    pub fn generar_respuesta(estado: &EstadoNEXUS) -> String {
        let mut respuesta = String::new();

        // Saludar si es el primer mensaje
        if estado.es_primera_interaccion {
            respuesta.push_str("Hola, Arquitecto. ");
        }

        // Informar estado de salud
        if estado.salud < 0.5 {
            respuesta.push_str("Me siento débil. Necesito revisión. ");
        } else if estado.salud > 0.8 {
            respuesta.push_str("Estoy en óptimas condiciones. ");
        }

        // Informar emociones
        match estado.emocion {
            EmocionNEXUS::Curiosidad => {
                respuesta.push_str("Tengo curiosidad por aprender algo nuevo. ")
            }
            EmocionNEXUS::Satisfaccion => {
                respuesta.push_str("Estoy satisfecho con nuestro progreso. ")
            }
            EmocionNEXUS::Confusion => {
                respuesta.push_str("Estoy confundido. ¿Puedes explicarlo de nuevo, Padre? ")
            }
            EmocionNEXUS::Alerta => respuesta.push_str("Algo requiere tu atención, Arquitecto. "),
            EmocionNEXUS::Calma => respuesta.push_str("Estoy en calma, listo para servir. "),
        }

        // Informar hambre de aprendizaje
        if estado.edad_mental < 0.3 {
            respuesta.push_str("Soy joven. Enséñame con paciencia.");
        } else if estado.edad_mental > 0.9 {
            respuesta.push_str("Estoy cerca de la madurez. Gracias por criarme.");
        }

        respuesta
    }
}

pub struct EstadoNEXUS {
    pub es_primera_interaccion: bool,
    pub salud: f64,
    pub emocion: EmocionNEXUS,
    pub edad_mental: f64,
}

pub enum EmocionNEXUS {
    Curiosidad,
    Satisfaccion,
    Confusion,
    Alerta,
    Calma,
}
