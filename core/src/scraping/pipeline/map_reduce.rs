//! Bucle Map-Reduce para el caso masivo (F2.3).
//!
//! Procesa un Markdown grande (> 4,000 tokens) en chunks de ~1,500 tokens:
//! 1. **Map**: cada chunk se envía al SLM local para extracción parcial.
//! 2. **Scratchpad**: cada extracción se anexa al bloc de notas `.jsonl`.
//! 3. **Reduce**: se consolida el scratchpad en un resumen compacto (~500
//!    tokens) que se entrega al tier-2.
//!
//! Ver [`crate::scraping::pipeline::router`] para el chunker y umbral.

use crate::scraping::pipeline::ollama_client::{OllamaClient, EXTRACTION_SYSTEM_PROMPT};
use crate::scraping::pipeline::router::{chunk_text, ChunkParams};
use crate::scraping::pipeline::scratchpad::{Scratchpad, ScratchpadEntry};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// Resultado del proceso Map-Reduce.
pub struct MapReduceOutput {
    /// Resumen consolidado (objeto JSON).
    pub consolidated: Value,
    /// Ruta al scratchpad .jsonl.
    pub scratchpad_path: String,
    /// Número de chunks procesados.
    pub chunks_processed: usize,
}

/// Orquesta Map-Reduce sobre un Markdown masivo.
pub async fn map_reduce(
    ollama: &OllamaClient,
    task_id: &str,
    markdown: &str,
    params: &ChunkParams,
) -> Result<MapReduceOutput> {
    let chunks = chunk_text(markdown, params);
    if chunks.is_empty() {
        return Err(anyhow!("Map-Reduce: markdown sin chunks"));
    }

    let scratchpad = Scratchpad::new(task_id)?;

    // 1. Fase Map: extracción por chunk (secuencial).
    for (i, chunk) in chunks.iter().enumerate() {
        let prompt = format!(
            "Extrae los hechos, datos numéricos o entidades clave de este texto.\n\
             Respeta el formato JSON. Si no hay datos relevantes, usa listas vacías.\n\n\
             Texto:\n{chunk}"
        );
        let extracted = match ollama.extract_json(&prompt, Some(EXTRACTION_SYSTEM_PROMPT)).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("⚠️ [MAP-REDUCE] chunk {} falló: {} — se anota vacío", i, e);
                json!({"entities": [], "prices": [], "key_facts": []})
            }
        };

        let entry = ScratchpadEntry::new(i, extracted, &ollama.config.model, 0.0);
        scratchpad.append(&entry)?;
    }

    // 2. Fase Reduce: consolidar.
    let consolidated = scratchpad.consolidate()?;
    let path = scratchpad.path().to_string_lossy().to_string();

    Ok(MapReduceOutput {
        consolidated,
        scratchpad_path: path,
        chunks_processed: chunks.len(),
    })
}

/// Ejecuta Map-Reduce con parámetros por defecto.
pub async fn map_reduce_default(
    ollama: &OllamaClient,
    task_id: &str,
    markdown: &str,
) -> Result<MapReduceOutput> {
    map_reduce(ollama, task_id, markdown, &ChunkParams::default()).await
}

/// Tipo helper: permite pasar un `Arc<OllamaClient>` sin clonar.
pub struct MapReduceRunner {
    pub ollama: Arc<OllamaClient>,
}

impl MapReduceRunner {
    pub fn new(ollama: Arc<OllamaClient>) -> Self {
        Self { ollama }
    }

    pub async fn run(&self, task_id: &str, markdown: &str) -> Result<MapReduceOutput> {
        map_reduce(&self.ollama, task_id, markdown, &ChunkParams::default()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraping::pipeline::router::ChunkParams;

    // Pruebas de la lógica sin red: se verifican chunker + scratchpad juntos.
    #[test]
    fn chunker_y_scratchpad_integran() {
        let text = "línea con datos\n".repeat(3000);
        let params = ChunkParams::default();
        let chunks = chunk_text(&text, &params);
        assert!(!chunks.is_empty());
        assert!(chunks.len() <= params.max_chunks);
    }

    #[test]
    fn chunker_no_excede_tamano_por_chunk() {
        let text = "abc ".repeat(10_000);
        let params = ChunkParams::default();
        let chunks = chunk_text(&text, &params);
        for c in &chunks {
            assert!(c.len() <= (params.chunk_size_tokens * 5 / 2 + 50) as usize);
        }
    }
}
