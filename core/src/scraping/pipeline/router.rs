//! Enrutador de umbral (Threshold Switch — spec §3.1).
//!
//! Decide el camino de procesamiento según el tamaño del Markdown limpio:
//!
//! | Condición | Acción | Frecuencia |
//! |-----------|--------|------------|
//! | ≤ 4,000 tokens | Directo a Tier-2 (nube, ExtractorOmega) | ~90% |
//! | > 4,000 tokens | Tier-1 (Ollama Map-Reduce) → resumen → Tier-2 | ~10% |
//!
//! Ver [`token_counter::THRESHOLD_TOKENS`].

use crate::scraping::pipeline::token_counter::{self, estimate};

/// Resultado de la decisión de enrutado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Markdown corto → directo a la nube.
    DirectToCloud,
    /// Markdown masivo → Map-Reduce local primero.
    MapReduceLocal,
}

/// Decide la ruta para un texto de Markdown limpio.
pub fn route(markdown: &str) -> Route {
    if token_counter::is_massive(markdown) {
        Route::MapReduceLocal
    } else {
        Route::DirectToCloud
    }
}

/// Parámetros del chunker (spec §3.2).
pub struct ChunkParams {
    pub chunk_size_tokens: u64,
    pub chunk_overlap_tokens: u64,
    pub max_chunks: usize,
}

impl Default for ChunkParams {
    fn default() -> Self {
        Self {
            chunk_size_tokens: 1500,
            chunk_overlap_tokens: 100,
            max_chunks: 20,
        }
    }
}

/// Trocea un texto en chunks respetando tamaño en tokens y solapamiento.
///
/// La división se hace por caracteres (proporcional a tokens) y trata de
/// cortar en límites de línea/párrafo cuando es posible.
pub fn chunk_text(markdown: &str, params: &ChunkParams) -> Vec<String> {
    let chars_per_chunk = (params.chunk_size_tokens * 5).div_ceil(2) as usize;
    let chars_overlap = (params.chunk_overlap_tokens * 5).div_ceil(2) as usize;

    // Trabajar sobre un Vec<char> para evitar slicing por bytes en UTF-8
    // multibyte (p. ej. caracteres acentuados).
    let chars: Vec<char> = markdown.chars().collect();
    let total = chars.len();

    let mut chunks = Vec::new();
    let mut start = 0usize;

    while start < total && chunks.len() < params.max_chunks {
        // Tomar ventana de tamaño chars_per_chunk.
        let mut end = (start + chars_per_chunk).min(total);
        // Retroceder hasta un salto de línea si no es el final.
        if end < total {
            if let Some(idx) = chars[start..end].iter().rposition(|&c| c == '\n') {
                if idx > 0 {
                    end = start + idx;
                }
            }
        }
        let chunk: String = chars[start..end].iter().collect();
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
        if end == total {
            break;
        }
        // Avanzar con solapamiento.
        start = end.saturating_sub(chars_overlap);
    }
    chunks
}

/// Suma el número de tokens estimados de un conjunto de chunks.
pub fn total_tokens(chunks: &[String]) -> u64 {
    chunks.iter().map(|c| estimate(c)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enruta_corto_a_nube() {
        let text = "a".repeat(2000);
        assert_eq!(route(&text), Route::DirectToCloud);
    }

    #[test]
    fn enruta_masivo_a_local() {
        let text = "a".repeat(20_000);
        assert_eq!(route(&text), Route::MapReduceLocal);
    }

    #[test]
    fn chunk_text_respeta_tamano_y_limite() {
        let text = "línea uno\n".repeat(500);
        let params = ChunkParams::default();
        let chunks = chunk_text(&text, &params);
        assert!(chunks.len() <= params.max_chunks);
        for c in &chunks {
            assert!(estimate(c) <= params.chunk_size_tokens + 200);
        }
        // El texto completo se recupera aproximadamente (considerando overlap).
        let joined: usize = chunks.iter().map(|c| c.len()).sum();
        assert!(joined >= text.len());
    }

    #[test]
    fn chunk_sin_exceder_max() {
        let text = "x".repeat(1_000_000);
        let params = ChunkParams::default();
        let chunks = chunk_text(&text, &params);
        assert_eq!(chunks.len(), params.max_chunks);
    }
}
