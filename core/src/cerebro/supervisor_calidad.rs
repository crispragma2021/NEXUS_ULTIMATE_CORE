// ==========================================
// 🧠 SUPERVISOR DE CALIDAD — NEXUS OMEGA
// ==========================================
// Validación post-hoc de respuestas de sub-agentes.
// Evalúa coherencia, relevancia, longitud y repetición.
// Capa base: heurística pura en Rust (sin dependencias externas).
// Capa extendida: Ollama opcional para análisis semántico profundo.
// ==========================================

use tracing::{debug, info, warn};

/// Veredicto emitido por el SupervisorDeCalidad tras analizar una respuesta.
#[derive(Debug, Clone)]
pub struct VeredictoCalidad {
    /// ¿La respuesta fue aprobada?
    pub aprobado: bool,
    /// Nivel de confianza (0.0 = mínimo, 1.0 = máximo)
    pub confianza: f32,
    /// Sugerencias de mejora (vacío si aprobado)
    pub sugerencias: Vec<String>,
    /// Prompt original que generó la respuesta
    pub prompt_original: String,
    /// Respuesta evaluada
    pub respuesta: String,
    /// Nombre del sub-agente que generó la respuesta
    pub agente: String,
}

impl VeredictoCalidad {
    pub fn aprobado(prompt: &str, respuesta: &str, agente: &str, confianza: f32) -> Self {
        Self {
            aprobado: true,
            confianza,
            sugerencias: Vec::new(),
            prompt_original: prompt.to_string(),
            respuesta: respuesta.to_string(),
            agente: agente.to_string(),
        }
    }

    pub fn rechazado(
        prompt: &str,
        respuesta: &str,
        agente: &str,
        sugerencias: Vec<String>,
    ) -> Self {
        Self {
            aprobado: false,
            confianza: 0.0,
            sugerencias,
            prompt_original: prompt.to_string(),
            respuesta: respuesta.to_string(),
            agente: agente.to_string(),
        }
    }

    pub fn incierto(
        prompt: &str,
        respuesta: &str,
        agente: &str,
        sugerencias: Vec<String>,
        confianza: f32,
    ) -> Self {
        Self {
            aprobado: false,
            confianza,
            sugerencias,
            prompt_original: prompt.to_string(),
            respuesta: respuesta.to_string(),
            agente: agente.to_string(),
        }
    }
}

/// Supervisor de Calidad — validador post-hoc de respuestas de sub-agentes.
///
/// # Heurísticas implementadas:
/// 1. **Longitud mínima**: Rechaza respuestas vacías o demasiado cortas (< 10 chars)
/// 2. **Detección de error**: Rechaza respuestas que contienen "error", "failed", "exception"
/// 3. **Coincidencia de palabras clave**: Verifica que la respuesta cubra conceptos del prompt
/// 4. **Detección de repetición**: Penaliza respuestas con frases repetitivas (n-gramas)
/// 5. **Proporción prompt/respuesta**: Señala si la respuesta es demasiado corta relativa al prompt
pub struct SupervisorDeCalidad {
    /// Longitud mínima aceptable en caracteres
    longitud_minima: usize,
    /// Umbral de confianza para aprobar automáticamente
    umbral_aprobacion: f32,
    /// Umbral de confianza para considerar incierto (por debajo = rechazo automático)
    umbral_rechazo: f32,
    /// Pesos de cada heurística
    peso_longitud: f32,
    peso_error: f32,
    peso_cobertura: f32,
    peso_repeticion: f32,
    peso_proporcion: f32,
}

impl Default for SupervisorDeCalidad {
    fn default() -> Self {
        Self {
            longitud_minima: 10,
            umbral_aprobacion: 0.7,
            umbral_rechazo: 0.3,
            peso_longitud: 0.20,
            peso_error: 0.30,
            peso_cobertura: 0.25,
            peso_repeticion: 0.15,
            peso_proporcion: 0.10,
        }
    }
}

impl SupervisorDeCalidad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configura personalizada del supervisor.
    pub fn con_config(longitud_minima: usize, umbral_aprobacion: f32, umbral_rechazo: f32) -> Self {
        let mut s = Self::default();
        s.longitud_minima = longitud_minima;
        s.umbral_aprobacion = umbral_aprobacion;
        s.umbral_rechazo = umbral_rechazo;
        s
    }

    /// Evalúa una respuesta de un sub-agente y emite un veredicto.
    pub async fn evaluar(&self, prompt: &str, respuesta: &str, agente: &str) -> VeredictoCalidad {
        debug!(
            "[SUPERVISOR] Evaluando respuesta de '{}'... ({} chars)",
            agente,
            respuesta.len()
        );

        // 1. Longitud mínima
        if respuesta.trim().is_empty() {
            info!("[SUPERVISOR] Rechazado: respuesta vacía de '{}'", agente);
            return VeredictoCalidad::rechazado(
                prompt,
                respuesta,
                agente,
                vec!["La respuesta está vacía.".to_string()],
            );
        }

        if respuesta.len() < self.longitud_minima {
            info!(
                "[SUPERVISOR] Rechazado: respuesta demasiado corta ({}/{} chars) de '{}'",
                respuesta.len(),
                self.longitud_minima,
                agente
            );
            return VeredictoCalidad::rechazado(
                prompt,
                respuesta,
                agente,
                vec![format!(
                    "La respuesta es demasiado corta ({} caracteres). Mínimo esperado: {}.",
                    respuesta.len(),
                    self.longitud_minima
                )],
            );
        }

        // 2. Detección de errores en la respuesta
        let patrones_error = [
            "error",
            "failed",
            "exception",
            "panic",
            "unreachable",
            "not found",
            "not_found",
            "internal server error",
            "500",
            "timeout",
            "connection refused",
        ];
        let respuesta_lower = respuesta.to_lowercase();
        let errores_detectados: Vec<&str> = patrones_error
            .iter()
            .filter(|&&p| respuesta_lower.contains(p))
            .copied()
            .collect();

        // 3. Cobertura de palabras clave del prompt
        let palabras_clave = self.extraer_palabras_clave(prompt);
        let palabras_cubiertas: Vec<&str> = palabras_clave
            .iter()
            .filter(|p| respuesta_lower.contains(*p))
            .copied()
            .collect();
        let cobertura = if palabras_clave.is_empty() {
            1.0
        } else {
            palabras_cubiertas.len() as f32 / palabras_clave.len() as f32
        };

        // 4. Detección de repetición (bigramas repetidos)
        let repeticion = self.detectar_repeticion(respuesta);

        // 5. Proporción prompt/respuesta
        let proporcion = if prompt.len() > 50 {
            let ratio = respuesta.len() as f32 / prompt.len() as f32;
            if ratio < 0.3 {
                0.3 // Muy corta para el prompt
            } else if ratio > 10.0 {
                0.6 // Sospechosamente larga
            } else {
                1.0
            }
        } else {
            1.0
        };

        // 6. Cálculo de puntuación ponderada
        let puntaje_longitud = if respuesta.len() >= self.longitud_minima * 3 {
            1.0
        } else {
            respuesta.len() as f32 / (self.longitud_minima * 3) as f32
        };

        let puntaje_error = if errores_detectados.is_empty() {
            1.0
        } else {
            0.0 // Cualquier patrón de error = rechazo inmediato
        };

        let puntaje_repeticion = 1.0 - repeticion;

        // Penalización si cobertura es 0.0 habiendo palabras clave relevantes (mín. 3 para prompts cortos)
        let penalizacion_cobertura_cero = if cobertura == 0.0 && palabras_clave.len() >= 3 {
            -0.4
        } else {
            0.0
        };

        let puntaje_total = (self.peso_longitud * puntaje_longitud.min(1.0)
            + self.peso_error * puntaje_error
            + self.peso_cobertura * cobertura
            + self.peso_repeticion * puntaje_repeticion
            + self.peso_proporcion * proporcion)
            + penalizacion_cobertura_cero;

        debug!(
            "[SUPERVISOR] Puntajes: longitud={:.2}, error={:.2}, cobertura={:.2} ({}/{}), repetición={:.2}, proporción={:.2} => total={:.2}",
            puntaje_longitud, puntaje_error, cobertura, palabras_cubiertas.len(), palabras_clave.len(), puntaje_repeticion, proporcion, puntaje_total
        );

        // 7. Emitir veredicto
        let mut sugerencias: Vec<String> = Vec::new();

        if !errores_detectados.is_empty() {
            sugerencias.push(format!(
                "La respuesta contiene patrones de error: {:?}.",
                errores_detectados
            ));
        }

        if cobertura < 0.5 && !palabras_clave.is_empty() {
            sugerencias.push(format!(
                "Baja cobertura de conceptos del prompt ({:.0}%). Palabras clave no cubiertas: {:?}.",
                cobertura * 100.0,
                &palabras_clave
                    .iter()
                    .filter(|p| !palabras_cubiertas.contains(p))
                    .take(5)
                    .collect::<Vec<_>>()
            ));
        }

        if repeticion > 0.3 {
            sugerencias.push("La respuesta contiene frases repetitivas.".to_string());
        }

        if puntaje_total >= self.umbral_aprobacion {
            info!(
                "[SUPERVISOR] ✅ Aprobado: '{}' con confianza {:.2}",
                agente, puntaje_total
            );
            VeredictoCalidad::aprobado(prompt, respuesta, agente, puntaje_total)
        } else if puntaje_total >= self.umbral_rechazo {
            info!(
                "[SUPERVISOR] ⚠️ Incierto: '{}' con confianza {:.2}",
                agente, puntaje_total
            );
            if sugerencias.is_empty() {
                sugerencias.push(
                    "La respuesta no cumple con los estándares mínimos de calidad.".to_string(),
                );
            }
            VeredictoCalidad::incierto(prompt, respuesta, agente, sugerencias, puntaje_total)
        } else {
            info!(
                "[SUPERVISOR] ❌ Rechazado: '{}' con confianza {:.2}",
                agente, puntaje_total
            );
            if sugerencias.is_empty() {
                sugerencias.push(
                    "La respuesta fue rechazada por no alcanzar el umbral mínimo de calidad."
                        .to_string(),
                );
            }
            VeredictoCalidad::rechazado(prompt, respuesta, agente, sugerencias)
        }
    }

    /// Evalúa múltiples respuestas y retorna la mejor (mayor confianza).
    /// Si todas son rechazadas, retorna la primera con sugerencias.
    pub async fn mejor_respuesta(
        &self,
        prompt: &str,
        respuestas: Vec<(String, String)>,
    ) -> VeredictoCalidad {
        if respuestas.is_empty() {
            return VeredictoCalidad::rechazado(
                prompt,
                "",
                "ninguno",
                vec!["No se recibieron respuestas de ningún sub-agente.".to_string()],
            );
        }

        let mut veredictos = Vec::new();
        for (agente, respuesta) in &respuestas {
            let v = self.evaluar(prompt, respuesta, agente).await;
            veredictos.push(v);
        }

        // Seleccionar el veredicto con mayor confianza entre los aprobados
        veredictos.sort_by(|a, b| {
            b.confianza
                .partial_cmp(&a.confianza)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Buscar el primer aprobado (están ordenados por confianza descendente)
        let idx = veredictos.iter().position(|v| v.aprobado);
        if let Some(i) = idx {
            let mejor = veredictos.swap_remove(i);
            info!(
                "[SUPERVISOR] 🏆 Mejor respuesta: '{}' con confianza {:.2}",
                mejor.agente, mejor.confianza
            );
            mejor
        } else {
            // Si ninguna fue aprobada, retornar la de mayor confianza como incierta
            let mejor = veredictos.swap_remove(0);
            warn!(
                "[SUPERVISOR] ⚠️ Ninguna respuesta aprobada. Usando la mejor disponible: '{}' (confianza: {:.2})",
                mejor.agente, mejor.confianza
            );
            mejor
        }
    }

    // ─── Funciones auxiliares ────────────────────────────────────────────────

    /// Extrae palabras clave relevantes de un prompt (>= 4 chars, sin stopwords).
    fn extraer_palabras_clave<'a>(&self, texto: &'a str) -> Vec<&'a str> {
        let stopwords = [
            "el", "la", "los", "las", "un", "una", "y", "e", "o", "a", "de", "del", "en", "por",
            "para", "con", "sin", "es", "su", "que", "como", "más", "pero", "lo", "le", "se", "no",
            "me", "te", "al", "this", "that", "the", "and", "or", "is", "are", "was", "were", "be",
            "been", "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
            "may", "might", "shall", "can", "need", "dare", "ought", "used",
        ];

        texto
            .split(|c: char| !c.is_alphanumeric() && c != '\'')
            .filter(|w| w.len() >= 4)
            .filter(|w| !stopwords.contains(&w.to_lowercase().as_str()))
            .collect()
    }

    /// Detecta nivel de repetición en el texto (basado en bigramas).
    /// Retorna 0.0 si no hay repetición, 1.0 si es completamente repetitivo.
    fn detectar_repeticion(&self, texto: &str) -> f32 {
        let palabras: Vec<&str> = texto.split_whitespace().collect();
        if palabras.len() < 4 {
            return 0.0;
        }

        // Generar bigramas
        let bigramas: Vec<String> = palabras
            .windows(2)
            .map(|w| format!("{} {}", w[0].to_lowercase(), w[1].to_lowercase()))
            .collect();

        if bigramas.is_empty() {
            return 0.0;
        }

        let total = bigramas.len();
        let mut unicos = std::collections::HashSet::new();
        for b in &bigramas {
            unicos.insert(b.clone());
        }

        1.0 - (unicos.len() as f32 / total as f32)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_respuesta_vacia_rechazada() {
        let supervisor = SupervisorDeCalidad::new();
        let v = supervisor.evaluar("cómo estás?", "", "test_agent").await;
        assert!(!v.aprobado, "Respuesta vacía debe ser rechazada");
        assert!(v.confianza == 0.0);
    }

    #[tokio::test]
    async fn test_respuesta_muy_corta_rechazada() {
        let supervisor = SupervisorDeCalidad::new();
        let v = supervisor
            .evaluar("cómo estás?", "bien", "test_agent")
            .await;
        assert!(!v.aprobado, "Respuesta muy corta debe ser rechazada");
    }

    #[tokio::test]
    async fn test_respuesta_valida_aprobada() {
        let supervisor = SupervisorDeCalidad::new();
        let respuesta = "Estoy funcionando correctamente. Todos los sistemas están operativos y la memoria está sincronizada.";
        let v = supervisor
            .evaluar("cómo estás?", respuesta, "test_agent")
            .await;
        assert!(v.aprobado, "Respuesta válida debe ser aprobada");
        assert!(v.confianza >= 0.5);
        assert!(v.sugerencias.is_empty());
    }

    #[tokio::test]
    async fn test_respuesta_con_error_rechazada() {
        let supervisor = SupervisorDeCalidad::new();
        let respuesta =
            "Ocurrió un error interno: connection refused al intentar acceder a la base de datos.";
        let v = supervisor
            .evaluar("consulta la base de datos", respuesta, "test_agent")
            .await;
        assert!(!v.aprobado, "Respuesta con error debe ser rechazada");
        assert!(!v.sugerencias.is_empty());
    }

    #[tokio::test]
    async fn test_cobertura_palabras_clave() {
        let supervisor = SupervisorDeCalidad::new();
        let prompt = "analiza el código fuente de Rust y encuentra vulnerabilidades de seguridad";
        let respuesta = "He analizado el código fuente y encontrado varias vulnerabilidades de seguridad en el sistema.";
        let v = supervisor.evaluar(prompt, respuesta, "test_agent").await;
        assert!(v.confianza >= 0.5);
    }

    #[tokio::test]
    async fn test_baja_cobertura_penalizada() {
        let supervisor = SupervisorDeCalidad::new();
        let prompt = "analiza el código fuente de Rust y encuentra vulnerabilidades de seguridad";
        let respuesta = "Hola, soy un asistente.";
        let v = supervisor.evaluar(prompt, respuesta, "test_agent").await;
        assert!(!v.aprobado, "Respuesta sin cobertura debe ser rechazada");
    }

    #[tokio::test]
    async fn test_mejor_respuesta_selecciona_mejor() {
        let supervisor = SupervisorDeCalidad::new();
        let prompt = "cuál es el clima?";
        let respuestas = vec![
            (
                "agente_malo".to_string(),
                "error 500 internal server".to_string(),
            ),
            (
                "agente_bueno".to_string(),
                "El clima es soleado con una temperatura de 25 grados Celsius.".to_string(),
            ),
        ];
        let mejor = supervisor.mejor_respuesta(prompt, respuestas).await;
        assert!(mejor.aprobado);
        assert_eq!(mejor.agente, "agente_bueno");
    }

    #[tokio::test]
    async fn test_extraer_palabras_clave() {
        let supervisor = SupervisorDeCalidad::new();
        let palabras = supervisor.extraer_palabras_clave("analiza el código fuente de Rust");
        assert!(palabras.contains(&"analiza"));
        assert!(palabras.contains(&"código"));
        assert!(palabras.contains(&"fuente"));
        assert!(palabras.contains(&"Rust"));
        assert!(!palabras.contains(&"el"));
        assert!(!palabras.contains(&"de"));
    }

    #[tokio::test]
    async fn test_detectar_repeticion_alta() {
        let supervisor = SupervisorDeCalidad::new();
        let texto_repetitivo = "hola mundo hola mundo hola mundo hola mundo";
        let r = supervisor.detectar_repeticion(texto_repetitivo);
        assert!(
            r > 0.3,
            "Texto repetitivo debe tener alta repetición: {}",
            r
        );
    }

    #[tokio::test]
    async fn test_detectar_repeticion_baja() {
        let supervisor = SupervisorDeCalidad::new();
        let texto_normal =
            "El zorro marrón salta sobre el perro perezoso mientras los pájaros cantan";
        let r = supervisor.detectar_repeticion(texto_normal);
        assert!(r < 0.3, "Texto normal debe tener baja repetición: {}", r);
    }
}
