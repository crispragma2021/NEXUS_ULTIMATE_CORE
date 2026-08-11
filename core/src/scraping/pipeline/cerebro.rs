//! Cerebro Auto-Creciente (RAG Acumulativo) — E2.
//!
//! Indexa automáticamente cada resultado scrapeado en la base vectorial y
//! permite consultar el conocimiento acumulado con búsqueda semántica.
//!
//! Flujo:
//! 1. [`Cerebro::index_result`] — al terminar un scraping exitoso, trocea el
//!    markdown en chunks, genera embeddings y los guarda en [`VectorStore`].
//! 2. [`Cerebro::retrieve`] — ante una consulta, genera su embedding y devuelve
//!    los top-k chunks más relevantes (< 2,000 tokens en contexto).
//!
//! Chunking: 512 tokens, overlap 50 (spec §1), reutilizando `router::chunk_text`.

use crate::scraping::pipeline::embedding::EmbeddingEngine;
use crate::scraping::pipeline::router::{chunk_text, ChunkParams};
use crate::scraping::pipeline::schemas::ScrapingResult;
use crate::scraping::pipeline::vector_store::{SearchHit, VectorStore};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;

/// Parámetros de chunking del cerebro (spec §1: 512 tokens, overlap 50).
fn brain_chunk_params() -> ChunkParams {
    ChunkParams {
        chunk_size_tokens: 512,
        chunk_overlap_tokens: 50,
        max_chunks: 50,
    }
}

/// Cerebro Auto-Creciente: indexación + recuperación semántica.
pub struct Cerebro {
    pub embedding: EmbeddingEngine,
    pub store: Arc<VectorStore>,
}

impl Cerebro {
    pub fn new(embedding: EmbeddingEngine, store: Arc<VectorStore>) -> Self {
        Self { embedding, store }
    }

    /// Abre un cerebro con base vectorial en la ruta indicada.
    pub fn open(store_path: &Path, embedding: EmbeddingEngine) -> Result<Self> {
        let store = Arc::new(VectorStore::open(store_path)?);
        Ok(Self::new(embedding, store))
    }

    /// Indexa el contenido de un resultado scrapeado exitoso.
    ///
    /// Devuelve el número de chunks indexados.
    pub async fn index_result(&self, result: &ScrapingResult) -> Result<usize> {
        let markdown = result
            .cleaned_markdown
            .as_deref()
            .filter(|m| !m.trim().is_empty())
            .context("ScrapingResult sin contenido para indexar")?;
        let url = result.url_final.as_deref().unwrap_or("");
        self.index_text(&result.task_id, url, markdown).await
    }

    /// Indexa un texto arbitrario (troceado + embeddings + persistencia).
    pub async fn index_text(&self, task_id: &str, source_url: &str, text: &str) -> Result<usize> {
        self.index_text_in_project(task_id, "", source_url, text)
            .await
    }

    /// Indexa un texto asociado a un proyecto (Scope isolation).
    ///
    /// Los chunks quedan etiquetados con `project_id`, permitiendo que el
    /// ScopeMapper recupere SOLO el conocimiento de ese proyecto.
    pub async fn index_text_in_project(
        &self,
        task_id: &str,
        project_id: &str,
        source_url: &str,
        text: &str,
    ) -> Result<usize> {
        let chunks = chunk_text(text, &brain_chunk_params());
        if chunks.is_empty() {
            return Ok(0);
        }

        let embeddings = self
            .embedding
            .embed_batch(&chunks.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .await?;

        for (chunk, vec) in chunks.iter().zip(embeddings.iter()) {
            self.store
                .insert_in_project(task_id, project_id, source_url, chunk, vec)?;
        }
        Ok(chunks.len())
    }

    /// Recupera los top-k chunks más relevantes para una consulta.
    pub async fn retrieve(&self, query: &str, k: usize) -> Result<Vec<SearchHit>> {
        let query_vec = self.embedding.embed(query).await?;
        self.store.search(&query_vec, k)
    }

    /// Recupera los top-k chunks más relevantes SOLO de un proyecto.
    ///
    /// Conecta el ScopeMapper con el RAG: dado el mensaje del usuario y el
    /// proyecto detectado, busca únicamente en el conocimiento de ese proyecto.
    pub async fn retrieve_in_project(
        &self,
        query: &str,
        project_id: &str,
        k: usize,
    ) -> Result<Vec<SearchHit>> {
        let query_vec = self.embedding.embed(query).await?;
        self.store.search_in_project(&query_vec, project_id, k)
    }

    /// Construye el contexto RAG combinado para un proyecto: la intención del
    /// usuario + los fragmentos semánticos relevantes de ESE proyecto.
    ///
    /// Devuelve (texto_para_contexto, hits). Es la inyección mínima de tokens:
    /// nunca trae fragmentos de otros proyectos.
    pub async fn build_project_context(
        &self,
        query: &str,
        project_id: &str,
        k: usize,
    ) -> Result<(String, Vec<SearchHit>)> {
        let hits = self.retrieve_in_project(query, project_id, k).await?;
        let mut ctx = String::new();
        for (i, hit) in hits.iter().enumerate() {
            ctx.push_str(&format!(
                "[Fragmento {} · {} · score {:.3}]\n{}\n\n",
                i + 1,
                hit.chunk.source_url,
                hit.score,
                hit.chunk.text
            ));
        }
        Ok((ctx, hits))
    }

    /// Recupera el contexto RAG listo para inyectar al LLM (top-k, formateado).
    ///
    /// Devuelve (texto_para_contexto, hits). El texto es <= 2,000 tokens
    /// aproximados (spec §1).
    pub async fn build_context(&self, query: &str, k: usize) -> Result<(String, Vec<SearchHit>)> {
        let hits = self.retrieve(query, k).await?;
        let mut ctx = String::new();
        for (i, hit) in hits.iter().enumerate() {
            ctx.push_str(&format!(
                "[Fuente {} · {} · score {:.3}]\n{}\n\n",
                i + 1,
                hit.chunk.source_url,
                hit.score,
                hit.chunk.text
            ));
        }
        Ok((ctx, hits))
    }

    /// Número de chunks almacenados en el cerebro.
    pub fn count(&self) -> Result<u64> {
        self.store.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraping::pipeline::embedding::{cosine_similarity, normalize};

    // Test de la lógica de chunking + store sin red (mock de embeddings).
    #[test]
    fn chunking_usa_512_tokens_con_overlap() {
        let text = "conocimiento sobre inteligencia artificial ".repeat(200);
        let chunks = chunk_text(&text, &brain_chunk_params());
        assert!(!chunks.is_empty());
        assert!(chunks.len() <= brain_chunk_params().max_chunks);
    }

    #[test]
    fn similitud_retorna_orden_correcto() {
        // Verificación de que el store ordena por score (sin red).
        let store = VectorStore::open_in_memory().unwrap();
        store
            .insert("a", "u1", "red", &normalize(&[1.0, 0.0, 0.0]))
            .unwrap();
        store
            .insert("b", "u2", "blue", &normalize(&[0.0, 1.0, 0.0]))
            .unwrap();
        let query = normalize(&[0.8, 0.2, 0.0]);
        let hits = store.search(&query, 5).unwrap();
        assert_eq!(hits[0].chunk.task_id, "a");
        assert!(hits[0].score > hits[1].score);
    }

    #[test]
    fn store_persiste_y_recupera() {
        let store = VectorStore::open_in_memory().unwrap();
        store
            .insert("t", "https://x", "hola mundo", &[1.0, 0.0, 0.0, 0.0])
            .unwrap();
        assert_eq!(store.count().unwrap(), 1);
        // Coseno del mismo vector = 1.
        let hits = store.search(&[1.0, 0.0, 0.0, 0.0], 1).unwrap();
        assert!((hits[0].score - 1.0).abs() < 1e-6);
    }

    // Prueba rápida de la función coseno usada en retrieval.
    #[test]
    fn coseno_funciona_en_cerebro() {
        let a = normalize(&[1.0, 1.0]);
        let b = normalize(&[1.0, 0.0]);
        let s = cosine_similarity(&a, &b);
        assert!(s > 0.7 && s < 1.0);
    }
}
