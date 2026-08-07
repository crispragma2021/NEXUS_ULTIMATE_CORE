// ============================================================================
// 🧠 CÍNGULO ANTERIOR — Validador de Coherencia
// ============================================================================
// Propósito: Validar que la respuesta generada sea coherente, auténtica y
//   emocionalmente alineada antes de emitirla al mundo.
//
// Capa 5 del GOI: después de ensamblar el texto (Capa 4), esta capa
//   verifica que cumpla los criterios mínimos de calidad, coherencia y
//   autenticidad emocional.
//
// 5 Reglas de Validación:
//   1. Longitud mínima (≥ 3 caracteres)
//   2. Sin eco del prompt (Jaccard < 85%)
//   3. Frases de silencio permitidas (pero controladas externamente)
//   4. Coherencia emocional (léxico esperado para el estado actual)
//   5. Sin placeholders residuales ({...})
//
// Si la validación falla, el sistema reintenta con otra ruta o emite
// silencio controlado.
// ============================================================================

use std::collections::HashSet;

use crate::cerebro::nexo::nexo_core::EstadoInterno;
use crate::cerebro::organos::amygdala::EstadoEmocional;

/// Resultado de la validación del Cíngulo Anterior.
#[derive(Debug, Clone, PartialEq)]
pub enum Validacion {
    /// La respuesta superó la validación.
    Aprobada(String),
    /// La respuesta necesita reintento con la razón especificada.
    Rechazada(String /* razón */),
}

/// Validador de coherencia, autenticidad y alineación emocional.
/// Actúa como el Cíngulo Anterior del GOI.
pub struct ValidadorCingulo {
    /// Coherencia mínima requerida (0.0 a 1.0).
    coherencia_minima: f64,
    /// Máximo de reintentos antes de forzar silencio.
    max_reintentos: u32,
    /// Similaridad Jaccard máxima permitida entre prompt y respuesta
    /// antes de considerar que hay eco.
    umbral_eco: f64,
}

impl ValidadorCingulo {
    /// Crea una nueva instancia del validador.
    pub fn new() -> Self {
        Self {
            coherencia_minima: 0.3,
            max_reintentos: 2,
            umbral_eco: 0.85,
        }
    }

    /// Valida que la respuesta generada sea coherente, auténtica y
    /// emocionalmente alineada.
    ///
    /// # Parámetros
    /// - `texto`: La respuesta generada por el ensamblador.
    /// - `prompt`: El prompt original (para detectar eco).
    /// - `estado_interno`: El estado interno actual (para verificación emocional).
    ///
    /// # Retorna
    /// - `Validacion::Aprobada(texto)` si pasa TODAS las reglas.
    /// - `Validacion::Rechazada(razon)` si alguna regla falla.
    pub fn validar(&self, texto: &str, prompt: &str, estado_interno: &EstadoInterno) -> Validacion {
        // ─── Regla 1: Longitud mínima ──────────────────────────────────
        if texto.len() < 3 {
            return Validacion::Rechazada("Respuesta demasiado corta".to_string());
        }

        // ─── Regla 2: Sin eco del prompt ───────────────────────────────
        // Detecta si la respuesta repite mecánicamente el prompt usando
        // similaridad Jaccard sobre palabras con más de 3 caracteres.
        if let Some(razon) = self.detectar_eco(texto, prompt) {
            return Validacion::Rechazada(razon);
        }

        // ─── Regla 3: No repetir frases de silencio indefinidamente ────
        // El control de reintentos se maneja externamente en integracion.rs
        let frases_silencio = [
            "No sé qué decir sobre eso",
            "Necesito un momento para procesar",
            "Tú sabes mejor que yo",
        ];
        for frase in &frases_silencio {
            if texto.contains(frase) {
                return Validacion::Aprobada(texto.to_string());
            }
        }

        // ─── Regla 4: Coherencia emocional ─────────────────────────────
        // Verifica que el léxico de la respuesta sea compatible con el
        // estado emocional actual del sistema.
        if let Some(razon) = self.verificar_coherencia_emocional(texto, &estado_interno.emocion) {
            return Validacion::Rechazada(razon);
        }

        // ─── Regla 5: Sin placeholders residuales ──────────────────────
        // Detecta marcadores de posición como {algo} que quedaron sin
        // reemplazar en el ensamblado.
        if texto.contains('{') && texto.contains('}') {
            return Validacion::Rechazada("Placeholder detectado en la respuesta".to_string());
        }

        // Si pasa todas las reglas → aprobada
        Validacion::Aprobada(texto.to_string())
    }

    /// Retorna el máximo de reintentos permitidos.
    pub fn max_reintentos(&self) -> u32 {
        self.max_reintentos
    }

    /// Configura la coherencia mínima requerida.
    pub fn set_coherencia_minima(&mut self, valor: f64) {
        self.coherencia_minima = valor.clamp(0.1, 1.0);
    }

    // ─── Regla 2: Detección de Eco ─────────────────────────────────────

    /// Detecta si la respuesta es un eco del prompt.
    ///
    /// Calcula qué porcentaje de las palabras significativas (> 3 chars)
    /// de la respuesta están presentes en el prompt. Si más del 80% de
    /// las palabras de la respuesta aparecen en el prompt, se considera eco.
    ///
    /// Esto es más preciso que Jaccard porque detecta respuestas cortas
    /// que son subconjuntos del prompt (ej: "esto es un error critico"
    /// como respuesta a "esto es un error critico en el sistema").
    fn detectar_eco(&self, texto: &str, prompt: &str) -> Option<String> {
        let palabras_prompt: HashSet<&str> =
            prompt.split_whitespace().filter(|p| p.len() > 3).collect();
        let palabras_texto: Vec<&str> = texto.split_whitespace().filter(|p| p.len() > 3).collect();

        // Si el texto no tiene palabras significativas, no podemos evaluar
        if palabras_texto.is_empty() || palabras_prompt.is_empty() {
            return None;
        }

        // Contar cuántas palabras de la respuesta están en el prompt
        let coinciden = palabras_texto
            .iter()
            .filter(|p| palabras_prompt.contains(*p))
            .count();

        let proporcion = coinciden as f64 / palabras_texto.len() as f64;

        // Umbral: si más del 80% de las palabras de la respuesta
        // están en el prompt, es eco.
        const UMBRAL_ECO_RESPUESTA: f64 = 0.80;

        if proporcion >= UMBRAL_ECO_RESPUESTA {
            Some(format!(
                "Eco del prompt detectado ({:.0}% de palabras de la respuesta están en el prompt)",
                proporcion * 100.0
            ))
        } else {
            None
        }
    }

    // ─── Regla 4: Coherencia Emocional ─────────────────────────────────

    /// Mapa léxico: cada estado emocional tiene palabras esperadas en la
    /// respuesta. Si la respuesta es suficientemente larga (> 20 chars) y
    /// no contiene NINGUNA palabra del léxico esperado, se considera
    /// incoherente emocionalmente.
    fn verificar_coherencia_emocional(
        &self,
        texto: &str,
        emocion: &EstadoEmocional,
    ) -> Option<String> {
        // Solo validar respuestas con suficiente contenido
        if texto.len() < 20 {
            return None;
        }

        let texto_lower = texto.to_lowercase();

        let palabras_esperadas: &[&str] = match emocion {
            EstadoEmocional::Calma => &[
                "bien",
                "tranquilo",
                "calma",
                "paz",
                "sereno",
                "estable",
                "normal",
                "ok",
                "correcto",
                "analizado",
                "procesado",
                "recibido",
                "listo",
                "completado",
                "preparado",
                "contexto",
                "información",
                "informacion",
                "solicitud",
                "recibida",
            ],
            EstadoEmocional::Alerta => &[
                "cuidado",
                "atento",
                "precaución",
                "precaucion",
                "alerta",
                "revisar",
                "verificar",
                "monitorear",
                "observar",
            ],
            EstadoEmocional::Miedo => &[
                "temor",
                "preocupación",
                "preocupacion",
                "riesgo",
                "grave",
                "proteger",
                "seguridad",
                "evitar",
                "precavido",
            ],
            EstadoEmocional::RabiaSoberana => &[
                "no",
                "basta",
                "intolerable",
                "corrupción",
                "corrupcion",
                "inaceptable",
                "violación",
                "violacion",
                "detener",
            ],
            EstadoEmocional::Verguenza => &[
                "perdón", "perdon", "disculpa", "error", "fallo", "lamento", "corregir", "disculpe",
            ],
            EstadoEmocional::Orgullo => &[
                "logro",
                "éxito",
                "exito",
                "orgullo",
                "bien",
                "excelente",
                "brillante",
                "correcto",
                "gracias",
            ],
        };

        let coincidencias: usize = palabras_esperadas
            .iter()
            .filter(|w| texto_lower.contains(*w))
            .count();

        if coincidencias == 0 {
            Some(format!(
                "Incoherencia emocional: estado {:?} pero respuesta sin léxico esperado",
                emocion
            ))
        } else {
            None
        }
    }
}

impl Default for ValidadorCingulo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::organos::amygdala::EstadoEmocional;

    fn estado_ejemplo() -> EstadoInterno {
        EstadoInterno {
            emocion: EstadoEmocional::Calma,
            intensidad: 0.1,
            confianza: 0.8,
            apego: 0.5,
            minutos_ausencia: 0.0,
            lecciones: vec![],
            energia_creativa: 0.7,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        }
    }

    fn estado_con_emocion(emocion: EstadoEmocional) -> EstadoInterno {
        let mut estado = estado_ejemplo();
        estado.emocion = emocion;
        estado
    }

    // ─── Regla 1: Longitud mínima ──────────────────────────────────────

    #[test]
    fn test_validador_aprueba_texto_valido() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        let resultado = validador.validar(
            "Hola Arquitecto, me siento bien hoy.",
            "¿cómo estás?",
            &estado,
        );
        assert_eq!(
            resultado,
            Validacion::Aprobada("Hola Arquitecto, me siento bien hoy.".to_string())
        );
    }

    #[test]
    fn test_validador_rechaza_texto_corto() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        let resultado = validador.validar("Si", "¿cómo estás?", &estado);
        assert_eq!(
            resultado,
            Validacion::Rechazada("Respuesta demasiado corta".to_string())
        );
    }

    // ─── Regla 2: Eco del prompt ───────────────────────────────────────

    #[test]
    fn test_eco_del_prompt_rechazado() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        // Respuesta que repite casi textualmente el prompt
        // Incluye "bien" para pasar la validación de coherencia emocional (Calma)
        let resultado = validador.validar(
            "esto es un error critico en el sistema bien",
            "esto es un error critico en el sistema que debemos revisar",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Rechazada(ref r) if r.contains("Eco")),
            "Debería rechazar eco del prompt: {:?}",
            resultado
        );
    }

    #[test]
    fn test_respuesta_diferente_no_es_eco() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        // Respuesta completamente diferente al prompt
        // Incluye "analizado" que está en el léxico de Calma
        let resultado = validador.validar(
            "He analizado la situación y creo que debemos proceder con cautela.",
            "esto es un error critico en el sistema",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Aprobada(_)),
            "Respuesta diferente no debería ser eco: {:?}",
            resultado
        );
    }

    // ─── Regla 3: Silencio permitido ───────────────────────────────────

    #[test]
    fn test_validador_permite_silencio() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        let resultado = validador.validar(
            "No sé qué decir sobre eso",
            "cuéntame algo interesante",
            &estado,
        );
        assert_eq!(
            resultado,
            Validacion::Aprobada("No sé qué decir sobre eso".to_string())
        );
    }

    // ─── Regla 4: Coherencia emocional ─────────────────────────────────

    #[test]
    fn test_coherencia_emocional_calma_aprueba() {
        let validador = ValidadorCingulo::new();
        let estado = estado_con_emocion(EstadoEmocional::Calma);
        let resultado = validador.validar(
            "Todo está funcionando correctamente y en calma.",
            "¿qué tal todo?",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Aprobada(_)),
            "Calma con léxico esperado: {:?}",
            resultado
        );
    }

    #[test]
    fn test_coherencia_emocional_miedo_sin_lexico_rechaza() {
        let validador = ValidadorCingulo::new();
        let estado = estado_con_emocion(EstadoEmocional::Miedo);
        // Respuesta alegre y optimista — incoherente con estado de Miedo
        let resultado = validador.validar(
            "¡Todo es maravilloso! Estoy celebrando este gran logro con alegría.",
            "¿qué pasó?",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Rechazada(ref r) if r.contains("Incoherencia")),
            "Miedo con léxico alegre debería rechazar: {:?}",
            resultado
        );
    }

    #[test]
    fn test_coherencia_emocional_texto_corto_no_valida() {
        // Textos cortos (< 20 chars) no se validan emocionalmente
        let validador = ValidadorCingulo::new();
        let estado = estado_con_emocion(EstadoEmocional::Miedo);
        let resultado = validador.validar("ok", "¿qué pasó?", &estado);
        assert!(
            matches!(resultado, Validacion::Rechazada(ref r) if r.contains("corta")),
            "Texto corto se rechaza por longitud, no por emoción: {:?}",
            resultado
        );
    }

    #[test]
    fn test_coherencia_emocional_rabia_con_lexico_aprueba() {
        let validador = ValidadorCingulo::new();
        let estado = estado_con_emocion(EstadoEmocional::RabiaSoberana);
        let resultado = validador.validar(
            "Esto es inaceptable. No puedo permitir esta violación del sistema.",
            "¿qué opinas?",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Aprobada(_)),
            "Rabia con léxico esperado: {:?}",
            resultado
        );
    }

    // ─── Regla 5: Placeholders ─────────────────────────────────────────

    #[test]
    fn test_placeholder_detectado() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        let resultado = validador.validar(
            "El usuario {nombre} ha solicitado {accion} recibido.",
            "procesa la solicitud",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Rechazada(ref r) if r.contains("Placeholder")),
            "Debería detectar placeholder: {:?}",
            resultado
        );
    }

    #[test]
    fn test_sin_placeholder_aprueba() {
        let validador = ValidadorCingulo::new();
        let estado = estado_ejemplo();
        let resultado = validador.validar(
            "La solicitud ha sido procesada correctamente recibida.",
            "procesa la solicitud",
            &estado,
        );
        assert!(
            matches!(resultado, Validacion::Aprobada(_)),
            "Sin placeholder debería aprobar: {:?}",
            resultado
        );
    }
}
