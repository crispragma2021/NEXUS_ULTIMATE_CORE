//! Motor de embeddings para el Cerebro Auto-Creciente (E2).
//!
//! Reutiliza el patrón validado del proyecto: embeddings vía **Ollama** con
//! `nomic-embed-text` (modelo ligero ~80 MB, corre 100% en CPU). Evita añadir
//! `fastembed`/Qdrant (infraestructura externa) manteniendo la soberanía local.
//!
//! Referencia: [`prueba_fuego_omega.rs`](../../bin/prueba_fuego_omega.rs).

use crate::scraping::pipeline::ollama_client::OLLAMA_URL;
use anyhow::{anyhow, Context, Result};
use serde_json::json;

/// Modelo de embedding por defecto (ligero, CPU).
pub const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text";

/// Cliente de embeddings vía Ollama.
#[derive(Clone)]
pub struct EmbeddingEngine {
    client: reqwest::Client,
    pub model: String,
}

impl EmbeddingEngine {
    pub fn new(model: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        Ok(Self {
            client,
            model: model.to_string(),
        })
    }

    /// Cliente por defecto con `nomic-embed-text`.
    pub fn default() -> Result<Self> {
        Self::new(DEFAULT_EMBED_MODEL)
    }

    /// Genera el vector de embedding de un texto.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let resp = self
            .client
            .post(format!("{OLLAMA_URL}/api/embeddings"))
            .json(&json!({
                "model": self.model,
                "prompt": text
            }))
            .send()
            .await
            .context("Error conectando con Ollama API (embeddings)")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama respondió HTTP {status}: {body}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .context("Error parseando respuesta de embeddings")?;
        let embedding = data
            .get("embedding")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Respuesta no contiene 'embedding'"))?;

        let vector: Vec<f32> = embedding
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();
        if vector.is_empty() {
            return Err(anyhow!("Embedding generado está vacío"));
        }
        Ok(vector)
    }

    /// Genera embeddings por lote (sin paralelismo excesivo).
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed(t).await?);
        }
        Ok(out)
    }
}

/// Normaliza un vector (L2) para similitud coseno eficiente.
pub fn normalize(v: &[f32]) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm == 0.0 {
        v.to_vec()
    } else {
        v.iter().map(|x| x / norm).collect()
    }
}

/// Similitud coseno entre dos vectores.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coseno_identico_es_uno() {
        let v = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn coseno_ortogonal_es_cero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn coseno_longitud_distinta_es_cero() {
        assert_eq!(cosine_similarity(&[1.0, 2.0], &[1.0]), 0.0);
    }

    #[test]
    fn normalize_unitario() {
        let v = vec![3.0, 4.0];
        let n = normalize(&v);
        let norm: f32 = n.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
}
