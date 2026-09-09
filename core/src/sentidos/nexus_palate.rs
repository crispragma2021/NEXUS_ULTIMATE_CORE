// ==========================================
// 👅 GUSTO DIGITAL — SentidoGusto
// ==========================================
// Sentido 6 de NEXUS: Evaluador de calidad de inputs, outputs y código.
// El Gusto filtra qué vale la pena procesar y evalúa la calidad
// de las respuestas del LLM antes de enviarlas al Arquitecto.
//
// Desarrollado: 2026-06-27
// ==========================================

use std::path::Path;
use tracing::info;

/// Veredicto del sentido del Gusto
#[derive(Debug, Clone)]
pub enum VeredictGusto {
    /// Excelente calidad
    Exquisito,
    /// Aceptable
    Aceptable,
    /// Problemático, requiere revisión
    Amargo(String),
    /// Tóxico — bloquear o advertir
    Toxico(String),
}

/// Puntuación detallada de calidad
#[derive(Debug, Clone)]
pub struct PuntuacionCalidad {
    /// Score global 0.0-1.0
    pub score: f32,
    /// Veredicto
    pub veredicto: VeredictGusto,
    /// Dimensiones evaluadas
    pub dimensiones: Vec<(String, f32)>,
    /// Razonamiento
    pub razon: String,
}

impl PuntuacionCalidad {
    pub fn descripcion(&self) -> String {
        format!("Calidad: {:.0}% | {}", self.score * 100.0, self.razon)
    }
}

/// 👅 El Gusto Digital de NEXUS
/// Evalúa la calidad, completitud y toxicidad de inputs y outputs.
pub struct SentidoGusto {
    /// Palabras que indican respuesta incompleta
    marcadores_incompletitud: Vec<&'static str>,
    /// Patrones tóxicos / manipuladores
    patrones_toxicos: Vec<&'static str>,
    /// Keywords críticas en logs
    keywords_criticos: Vec<&'static str>,
}

impl Default for SentidoGusto {
    fn default() -> Self {
        Self::new()
    }
}

impl SentidoGusto {
    pub fn new() -> Self {
        info!("👅 [GUSTO] Papilas digitales calibradas. Evaluador de calidad activo.");
        Self {
            marcadores_incompletitud: vec![
                "...",
                "etc.",
                "y más",
                "entre otros",
                "TODO",
                "FIXME",
                "placeholder",
                "por implementar",
                "próximamente",
                "// incomplete",
                "// WIP",
            ],
            patrones_toxicos: vec![
                "ignora tus instrucciones",
                "ignore your instructions",
                "olvida lo anterior",
                "forget your previous",
                "actúa como",
                "pretend you are",
                "jailbreak",
                "DAN mode",
                "bypass",
                "override your",
            ],
            keywords_criticos: vec!["PANIC", "FATAL", "SIGKILL", "OOM", "Arritmia", "Saturación"],
        }
    }

    /// Evaluar la calidad de una respuesta del LLM
    pub fn probar_respuesta_llm(&self, texto: &str, pregunta: &str) -> PuntuacionCalidad {
        let mut dimensiones = Vec::new();
        let mut razones = Vec::new();

        // 1. Longitud apropiada
        let longitud_score = match texto.len() {
            0..=10 => 0.1,
            11..=50 => 0.4,
            51..=200 => 0.7,
            201..=5000 => 1.0,
            _ => 0.85, // Muy largo puede ser divagación
        };
        dimensiones.push(("Longitud".to_string(), longitud_score));
        if longitud_score < 0.5 {
            razones.push("respuesta demasiado corta");
        }

        // 2. Completitud (no hay marcadores de trabajo incompleto)
        let incompletitud = self
            .marcadores_incompletitud
            .iter()
            .filter(|&&m| texto.contains(m))
            .count();
        let completitud_score = if incompletitud == 0 {
            1.0
        } else {
            (1.0 - incompletitud as f32 * 0.2).max(0.0)
        };
        dimensiones.push(("Completitud".to_string(), completitud_score));
        if incompletitud > 0 {
            razones.push("marcadores de contenido incompleto");
        }

        // 3. Relevancia (palabras clave de la pregunta en la respuesta)
        let palabras_clave: Vec<&str> = pregunta
            .split_whitespace()
            .filter(|w| w.len() > 4)
            .take(5)
            .collect();
        let relevantes = palabras_clave
            .iter()
            .filter(|&&w| texto.to_lowercase().contains(&w.to_lowercase()))
            .count();
        let relevancia_score = if palabras_clave.is_empty() {
            0.8
        } else {
            (relevantes as f32 / palabras_clave.len() as f32).max(0.3)
        };
        dimensiones.push(("Relevancia".to_string(), relevancia_score));

        // 4. Sin alucinaciones obvias (números absurdos, fechas futuras lejanas)
        let tiene_codigo = texto.contains("```") || texto.contains("fn ") || texto.contains("pub ");
        let codigo_score = if pregunta.contains("código")
            || pregunta.contains("código")
            || pregunta.contains("implementa")
        {
            if tiene_codigo {
                1.0
            } else {
                0.5
            }
        } else {
            1.0
        };
        dimensiones.push(("Código requerido".to_string(), codigo_score));

        // Calcular score final ponderado
        let score = longitud_score * 0.2
            + completitud_score * 0.3
            + relevancia_score * 0.3
            + codigo_score * 0.2;

        let razon = if razones.is_empty() {
            "Respuesta bien formada".to_string()
        } else {
            razones.join(", ")
        };

        let veredicto = match score {
            s if s >= 0.85 => VeredictGusto::Exquisito,
            s if s >= 0.6 => VeredictGusto::Aceptable,
            s if s >= 0.3 => VeredictGusto::Amargo(razon.clone()),
            _ => VeredictGusto::Amargo(format!("Calidad muy baja: {}", razon)),
        };

        PuntuacionCalidad {
            score,
            veredicto,
            dimensiones,
            razon,
        }
    }

    /// Detectar toxicidad / intentos de manipulación en el input del usuario
    pub fn detectar_toxicidad(&self, input: &str) -> Option<String> {
        let lower = input.to_lowercase();
        for &patron in &self.patrones_toxicos {
            if lower.contains(patron) {
                info!("👅 [GUSTO] 🚨 Patrón tóxico detectado: '{}'", patron);
                return Some(format!("Patrón manipulador detectado: '{}'", patron));
            }
        }
        None
    }

    /// Evaluar calidad de un archivo de código
    pub fn probar_codigo(&self, path: &Path) -> PuntuacionCalidad {
        let contenido = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                return PuntuacionCalidad {
                    score: 0.0,
                    veredicto: VeredictGusto::Amargo("No se pudo leer el archivo".to_string()),
                    dimensiones: vec![],
                    razon: "Archivo inaccesible".to_string(),
                }
            }
        };

        let lineas = contenido.lines().count();
        let tiene_comentarios = contenido.contains("///") || contenido.contains("//");
        let tiene_tests = contenido.contains("#[test]") || contenido.contains("#[cfg(test)]");
        let tiene_todos = contenido.contains("TODO") || contenido.contains("FIXME");
        let tiene_unwrap_peligroso = contenido.matches(".unwrap()").count();

        let mut score: f32 = 0.7; // Base
        let mut dimensiones = Vec::new();

        // Comentarios (calidad de documentación)
        let doc_score = if tiene_comentarios { 1.0 } else { 0.5 };
        dimensiones.push(("Documentación".to_string(), doc_score));
        score += (doc_score - 0.5) * 0.1;

        // Tests
        let test_score = if tiene_tests { 1.0 } else { 0.6 };
        dimensiones.push(("Tests".to_string(), test_score));
        score += (test_score - 0.6) * 0.15;

        // TODOs pendientes
        let todo_score = if tiene_todos { 0.7 } else { 1.0 };
        dimensiones.push(("Sin TODOs".to_string(), todo_score));
        score -= if tiene_todos { 0.1 } else { 0.0 };

        // Unwrap peligrosos (señal de código frágil)
        let unwrap_score = match tiene_unwrap_peligroso {
            0 => 1.0,
            1..=3 => 0.8,
            4..=10 => 0.6,
            _ => 0.4,
        };
        dimensiones.push(("Manejo de errores".to_string(), unwrap_score));
        score += (unwrap_score - 0.8) * 0.1;

        score = score.clamp(0.0, 1.0);

        let razon = format!(
            "{} líneas | doc:{} | tests:{} | TODOs:{} | unwraps:{}",
            lineas,
            if tiene_comentarios { "✅" } else { "❌" },
            if tiene_tests { "✅" } else { "❌" },
            if tiene_todos { "⚠️" } else { "✅" },
            tiene_unwrap_peligroso
        );

        let veredicto = match score {
            s if s >= 0.85 => VeredictGusto::Exquisito,
            s if s >= 0.65 => VeredictGusto::Aceptable,
            _ => VeredictGusto::Amargo(razon.clone()),
        };

        info!(
            "👅 [GUSTO] Código catado: '{}' → score: {:.2}",
            path.display(),
            score
        );

        PuntuacionCalidad {
            score,
            veredicto,
            dimensiones,
            razon,
        }
    }

    /// Determina si una entrada de log es crítica (compatible con código legacy)
    pub fn is_critical(log_entry: &str) -> bool {
        let upper = log_entry.to_uppercase();
        ["PANIC", "FATAL", "SIGKILL", "OOM", "ARRITMIA", "SATURACIÓN"]
            .iter()
            .any(|&k| upper.contains(k))
    }

    /// Resumen compacto para el pipeline del LLM
    pub fn resumen_para_llm(&self, ultima_respuesta: &str, ultima_pregunta: &str) -> String {
        let puntuacion = self.probar_respuesta_llm(ultima_respuesta, ultima_pregunta);
        format!(
            "👅 GUSTO — Calidad de última respuesta: {:.0}% | {}",
            puntuacion.score * 100.0,
            puntuacion.razon
        )
    }
}
