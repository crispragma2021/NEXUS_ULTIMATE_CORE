//! Estimación de tokens (F1.3).
//!
//! Heurística del plan: `chars.len() / 2.5` (aprox. ±15% para texto en
//! español/inglés mezclado). Suficiente para el enrutador de umbral, que
//! solo necesita distinguir "≤ 4,000 tokens" de "> 4,000 tokens".

/// Estima el número de tokens a partir de una cadena de texto.
///
/// Regla: `chars / 2.5`, redondeado hacia arriba. Acepta un texto ya limpio
/// (Markdown) o HTML crudo; el usuario del pipeline debe pasar el Markdown
/// limpio para obtener una estimación coherente con la especificación.
pub fn estimate(text: &str) -> u64 {
    let chars = text.chars().count() as u64;
    // chars / 2.5 == (chars * 2) / 5; redondeo a techo.
    (chars * 2).div_ceil(5)
}

/// Umbral de enrutamiento: el plan usa 4,000 tokens.
pub const THRESHOLD_TOKENS: u64 = 4000;

/// Decide si el texto es "corto" (directo a nube) o "masivo" (Map-Reduce local).
pub fn is_massive(text: &str) -> bool {
    estimate(text) > THRESHOLD_TOKENS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimacion_corresponde_a_division_por_2_5() {
        // 100 chars → 40 tokens
        let text = "a".repeat(100);
        assert_eq!(estimate(&text), 40);
    }

    #[test]
    fn umbral_4000_tokens_equivale_a_10000_chars() {
        // 10,000 chars / 2.5 = 4,000 tokens → en el borde (no masivo).
        let text = "a".repeat(10_000);
        assert!(!is_massive(&text));
        // 10,001 chars → 4,001 tokens → masivo.
        let text = "a".repeat(10_001);
        assert!(is_massive(&text));
    }

    #[test]
    fn texto_vacio_no_es_masivo() {
        assert!(!is_massive(""));
    }
}
