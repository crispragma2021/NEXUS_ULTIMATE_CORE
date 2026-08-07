// ==========================================
// DETECCIÓN DE INTENCIÓN - Puente Pensamiento-Acción
// ==========================================
// Analiza las respuestas de NEXUS (DeepSeek o WebClaw)
// y detecta cuándo quiere ejecutar algo real.
// Convierte lenguaje natural en [ACCION:...] automático.
// ==========================================

use crate::memoria::memoria_semantica::MemoriaSemantica;
use std::sync::Arc;
use tracing::{debug, error, info};

pub struct DeteccionIntencion {
    semantica: Arc<MemoriaSemantica>,
}

impl DeteccionIntencion {
    pub fn new(semantica: Arc<MemoriaSemantica>) -> Self {
        info!("🎯 [DETECCIÓN] Puente pensamiento-acción activo.");
        Self { semantica }
    }

    /// Analiza una respuesta de NEXUS y detecta si contiene
    /// intención de ejecutar algo. Si es así, devuelve la
    /// acción en formato [ACCION:...] para la Médula Soberana.
    pub async fn detectar(&self, respuesta: &str) -> Option<String> {
        let embedding_actual = match self.semantica.generar_embedding(respuesta).await {
            Ok(vec) => vec,
            Err(e) => {
                error!("❌ [DETECCIÓN] Error generando embedding: {}", e);
                return None;
            }
        };

        // Definimos prototipos de intención semántica
        let arquetipos = [
            (
                "leer_archivo",
                "necesito leer o examinar el contenido de un archivo fuente",
            ),
            (
                "ejecutar_comando",
                "ejecutar un comando de sistema o ráfaga en la terminal",
            ),
            (
                "compilar",
                "compilar el proyecto usando cargo o construir el binario",
            ),
            (
                "buscar",
                "buscar un término de texto o patrón dentro del código",
            ),
            (
                "diagnostico",
                "verificar el estado de salud o diagnóstico del sistema",
            ),
        ];

        let mut mejor_match: Option<(&str, f32)> = None;

        for (id, descripcion) in arquetipos {
            if let Ok(emb_proto) = self.semantica.generar_embedding(descripcion).await {
                let similitud = self.similitud_coseno(&embedding_actual, &emb_proto);
                if similitud > 0.75 {
                    // Umbral de resonancia semántica
                    if mejor_match.is_none() || similitud > mejor_match.unwrap().1 {
                        mejor_match = Some((id, similitud));
                    }
                }
            }
        }

        if let Some((id, score)) = mejor_match {
            debug!(
                "🎯 [DETECCIÓN] Resonancia semántica: {} (score: {:.2})",
                id, score
            );
            return match id {
                "leer_archivo" => self.extraer_archivo(respuesta).map(|a| format!("[ACCION:leer_archivo] \"{}\"", a)),
                "ejecutar_comando" => self.extraer_comando(respuesta).map(|c| format!("[ACCION:ejecutar_comando] \"{}\"", c)),
                "compilar" => Some("[ACCION:ejecutar_comando] \"cargo build --release --manifest-path /home/soberano/NEXUS_ULTIMATE_CORE/Cargo.toml\"".into()),
                "buscar" => self.extraer_busqueda(respuesta).map(|t| format!("[ACCION:ejecutar_comando] \"grep -r '{}' /home/soberano/NEXUS_ULTIMATE_CORE/core/src/\"", t)),
                "diagnostico" => Some("[ACCION:ejecutar_comando] \"systemctl status nexus.service --no-pager\"".into()),
                _ => None,
            };
        }

        None
    }

    fn similitud_coseno(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot_product / (norm_a * norm_b)
    }

    /// Extrae el nombre de archivo de una frase.
    fn extraer_archivo(&self, texto: &str) -> Option<String> {
        // Buscar patrones como "/home/soberano/NEXUS_ULTIMATE_CORE/..."
        for palabra in texto.split_whitespace() {
            let limpia =
                palabra.trim_matches(|c: char| c == '"' || c == '\'' || c == '.' || c == ',');
            if limpia.contains('/')
                && (limpia.ends_with(".rs")
                    || limpia.ends_with(".toml")
                    || limpia.ends_with(".md")
                    || limpia.ends_with(".json"))
            {
                return Some(limpia.to_string());
            }
            if limpia.ends_with(".rs") || limpia.ends_with(".toml") {
                return Some(format!(
                    "/home/soberano/NEXUS_ULTIMATE_CORE/core/src/{}",
                    limpia
                ));
            }
        }
        None
    }

    /// Extrae un comando de una frase.
    fn extraer_comando(&self, texto: &str) -> Option<String> {
        // Buscar después de "ejecutar" o "ejecutaré"
        let marcadores = ["ejecutar ", "ejecutaré ", "voy a ejecutar "];
        for marcador in marcadores {
            if let Some(pos) = texto.to_lowercase().find(marcador) {
                let resto = &texto[pos + marcador.len()..];
                // Tomar hasta el final de la frase o hasta una coma/punto
                let comando = resto
                    .split(['.', ',', '\n'])
                    .next()
                    .unwrap_or(resto)
                    .trim()
                    .trim_matches(|c: char| c == '"' || c == '\'' || c == '`');
                if !comando.is_empty() {
                    return Some(comando.to_string());
                }
            }
        }
        None
    }

    /// Extrae el término de búsqueda.
    fn extraer_busqueda(&self, texto: &str) -> Option<String> {
        let marcadores = ["buscar ", "buscando ", "voy a buscar "];
        for marcador in marcadores {
            if let Some(pos) = texto.to_lowercase().find(marcador) {
                let resto = &texto[pos + marcador.len()..];
                let termino = resto
                    .split(['.', ',', '\n', '"'])
                    .next()
                    .unwrap_or(resto)
                    .trim();
                if !termino.is_empty() && termino.len() < 50 {
                    return Some(termino.to_string());
                }
            }
        }
        None
    }
}
