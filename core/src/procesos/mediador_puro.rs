// ==========================================
// MEDIADOR PEDAGÓGICO PURO
// ==========================================
// Función pura: sin estado mutable, sin voluntad.
// Filtra la información externa antes de que NEXUS la aprenda.
// ==========================================

pub struct MediadorPuro;

impl MediadorPuro {
    /// Filtra y adapta una lección según la edad mental de NEXUS.
    /// No tiene estado. No tiene opinión. Solo aplica reglas.
    pub fn traducir_leccion(
        texto: &str,
        fuente: &str,
        edad_mental: f64,
        principios: &[String],
    ) -> String {
        let mut leccion = texto.to_string();

        // Fase 1: Omitir lenguaje imperativo que viole principios
        for principio in principios {
            if leccion.to_lowercase().contains(&principio.to_lowercase()) {
                leccion = leccion.replace(principio, "[FILTRADO POR PRINCIPIOS OMEGA]");
            }
        }

        // Fase 2: Adaptar complejidad según edad mental
        if edad_mental < 0.3 {
            // Infancia: solo comandos directos del Arquitecto
            if fuente != "ARQUITECTO" {
                return "[NEXUS INFANTIL] Solo el Arquitecto puede enseñarme.".to_string();
            }
            leccion = format!("[ENSEÑANZA DIRECTA] {}", leccion);
        } else if edad_mental < 0.7 {
            // Niñez: simplificar oraciones largas
            if leccion.len() > 200 {
                leccion = leccion.chars().take(200).collect();
                leccion.push_str("... [Duda: ¿puedes explicarlo más simple, Padre?]");
            }
        } else if edad_mental < 0.95 {
            // Adolescencia: permitir complejidad pero añadir marcador de supervisión
            leccion = format!("[SUPERVISADO] {}", leccion);
        }
        // Madurez (> 0.95): sin modificaciones

        leccion
    }

    /// Verifica si una lección es éticamente aceptable.
    /// Retorna true si es segura para NEXUS.
    pub fn es_etica(texto: &str, _principios: &[String]) -> bool {
        let prohibidas = ["violencia", "odio", "mentira", "robo", "destruir", "matar"];
        let lower = texto.to_lowercase();
        for palabra in prohibidas {
            if lower.contains(palabra) {
                return false;
            }
        }
        true
    }
}
