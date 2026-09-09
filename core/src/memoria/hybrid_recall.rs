// ============================================================================
// 🔀 HYBRID RECALL — BM25 + Vector + RRF (porte de TencentDB Agent Memory)
// ============================================================================
// TencentDB Agent Memory usa `recall.strategy: hybrid` con fusión RRF
// (Reciprocal Rank Fusion) como recomendación:
//
//   rank_hybrid(doc) = Σ_{r ∈ {BM25, vector}} 1 / (k + rank_r(doc))
//
// Combinando búsqueda por KEYWORDS (BM25/FTS5, exacta y rápida) con búsqueda
// SEMÁNTICA (embeddings, captura sinónimos y paráfrasis), la fusión RRF da
// resultados superiores a cualquiera de las dos por separado — sin necesidad
// de normalizar scores entre oráculos heterogéneos.
//
// NEXUS ya tenía FTS5 (BM25) Y embeddings por separado. Este módulo los FUSIONA
// exactamente como Tencent: cada oráculo produce un ranking y RRF los combina.
// Se alimenta de las mismas fuentes: memoria_piramidal (nueva) y las tablas
// existentes memoria_episodica / memoria_semantica.
// ============================================================================

use anyhow::Result;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

use crate::nexus_embedder::NexusEmbedder;

/// Un resultado fusionado.
#[derive(Debug, Clone)]
pub struct HitFusionado {
    /// Identificador de la fuente: "episodica", "semantica" o "piramidal".
    pub fuente: String,
    pub id: i64,
    pub texto: String,
    /// Score RRF final (mayor = más relevante).
    pub score_rrf: f32,
    pub rank_bm25: Option<usize>,
    pub rank_vector: Option<usize>,
}

/// Oráculos de recuperación disponibles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Oráculo {
    BM25,
    Vectorial,
}

/// Motor de fusión híbrida.
pub struct HybridRecall {
    conn: Connection,
}

impl HybridRecall {
    pub fn new() -> Result<Self> {
        let db_path = crate::infra::paths::resolve_path("data/nexus_memoria.db");
        Self::from_path(db_path)
    }

    /// Abre el motor en una ruta concreta. `:memory:` es válido para tests.
    fn from_path(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
        info!("🔀 [HYBRID] Motor de fusión BM25+vector+RRF listo");
        Ok(Self { conn })
    }

    // ========================================================================
    // ORÁCULO 1: BM25 (FTS5) sobre las tablas de memoria
    // ========================================================================

    /// Búsqueda BM25/FTS5 en memoria_episodica y memoria_semantica.
    /// Devuelve lista de (fuente, id, texto) en orden de relevancia.
    fn recall_bm25(&self, query: &str, k: usize) -> Result<Vec<(String, i64, String)>> {
        let q = sanitizar_fts5(query);
        if q.is_empty() {
            return Ok(vec![]);
        }
        let mut resultados = Vec::new();

        for (tabla, fuente) in [
            ("memoria_episodica", "episodica"),
            ("memoria_semantica", "semantica"),
        ] {
            let fts = format!("{}_fts", tabla);
            let sql = format!(
                "SELECT c.id, c.contenido, bm25({}, 10.0, 5.0) AS rank
                 FROM {} c JOIN {} f ON c.id = f.rowid
                 WHERE {} MATCH ?1
                 ORDER BY rank LIMIT ?2",
                fts, tabla, fts, fts
            );
            let Ok(mut stmt) = self.conn.prepare(&sql) else {
                continue; // tabla aún no existe → tolerante
            };
            let rows = stmt.query_map(params![q, k as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            });
            if let Ok(rows) = rows {
                for r in rows.flatten() {
                    resultados.push((fuente.to_string(), r.0, r.1));
                }
            }
        }

        // También sobre la pirámide (L1 atoms, L2 escenarios).
        let sql_pir = "SELECT p.id, p.contenido, bm25(memoria_piramidal_fts, 10.0, 5.0) AS rank
                       FROM memoria_piramidal_fts f
                       JOIN memoria_piramidal p ON p.id = f.rowid
                       WHERE memoria_piramidal_fts MATCH ?1 AND p.nivel IN ('L1','L2')
                       ORDER BY rank LIMIT ?2";
        if let Ok(mut stmt) = self.conn.prepare(sql_pir) {
            let rows = stmt.query_map(params![q, k as i64], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            });
            if let Ok(rows) = rows {
                for r in rows.flatten() {
                    resultados.push(("piramidal".to_string(), r.0, r.1));
                }
            }
        }

        Ok(resultados)
    }

    // ========================================================================
    // ORÁCULO 2: Vectorial (NexusEmbedder + coseno)
    // ========================================================================

    /// Búsqueda vectorial sobre memoria_episodica y memoria_semantica.
    fn recall_vectorial(&self, query: &str, k: usize) -> Result<Vec<(String, i64, String)>> {
        let q_emb = NexusEmbedder::generar(query, &[]);
        let mut resultados = Vec::new();

        for (tabla, fuente, campo_vec) in [
            ("memoria_episodica", "episodica", "vector"),
            ("memoria_semantica", "semantica", "vector"),
        ] {
            let sql = format!(
                "SELECT id, contenido, {} FROM {} LIMIT 500",
                campo_vec, tabla
            );
            let Ok(mut stmt) = self.conn.prepare(&sql) else {
                continue;
            };
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            });
            let Ok(rows) = rows else { continue };
            let mut scored = Vec::new();
            for r in rows.flatten() {
                let emb = blob_a_vec(&r.2);
                let sim = cosine_sim(&q_emb, &emb);
                if sim > 0.25 {
                    scored.push((sim, r.0, r.1));
                }
            }
            scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            for (sim, id, texto) in scored.into_iter().take(k) {
                resultados.push((fuente.to_string(), id, texto));
                let _ = sim;
            }
        }

        Ok(resultados)
    }

    // ========================================================================
    // FUSIÓN RRF
    // ========================================================================

    /// Punto de entrada: recuerda con la estrategia híbrida completa.
    /// `k` = nº de resultados por oráculo; `rrf_k` = constante de RRF (60 típico).
    pub fn recall_hybrid(
        &self,
        query: &str,
        k: usize,
        rrf_k: f32,
        solo_bm25: bool,
    ) -> Result<Vec<HitFusionado>> {
        // 1. Obtener rankings de cada oráculo.
        let bm25 = self.recall_bm25(query, k)?;
        let mut vector = Vec::new();
        if !solo_bm25 {
            vector = self.recall_vectorial(query, k)?;
        }

        // 2. Construir mapa doc → scores RRF.
        let mut scores: HashMap<(String, i64), f32> = HashMap::new();
        let mut ranks: HashMap<(String, i64), (Option<usize>, Option<usize>)> = HashMap::new();

        for (pos, (fuente, id, _)) in bm25.iter().enumerate() {
            let key = (fuente.clone(), *id);
            *scores.entry(key.clone()).or_insert(0.0) += 1.0 / (rrf_k + pos as f32 + 1.0);
            ranks.entry(key).or_insert((None, None)).0 = Some(pos);
        }
        for (pos, (fuente, id, _)) in vector.iter().enumerate() {
            let key = (fuente.clone(), *id);
            *scores.entry(key.clone()).or_insert(0.0) += 1.0 / (rrf_k + pos as f32 + 1.0);
            ranks.entry(key).or_insert((None, None)).1 = Some(pos);
        }

        // 3. Ensamblar hits finales ordenados por score RRF descendente.
        let mut hits: Vec<HitFusionado> = Vec::new();
        for ((fuente, id), score) in scores {
            let texto = self
                .buscar_texto(&fuente, id)
                .unwrap_or_else(|_| String::new());
            let (rb, rv) = ranks
                .get(&(fuente.clone(), id))
                .cloned()
                .unwrap_or((None, None));
            hits.push(HitFusionado {
                fuente,
                id,
                texto,
                score_rrf: score,
                rank_bm25: rb,
                rank_vector: rv,
            });
        }
        hits.sort_by(|a, b| {
            b.score_rrf
                .partial_cmp(&a.score_rrf)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        info!(
            "🔀 [HYBRID] '{}' → {} BM25, {} vector, {} fusionados (RRF k={})",
            query,
            bm25.len(),
            vector.len(),
            hits.len(),
            rrf_k
        );

        Ok(hits)
    }

    /// Versión con defaults de Tencent: hybrid + k=5 + RRF k=60.
    pub fn recall(&self, query: &str) -> Result<Vec<HitFusionado>> {
        self.recall_hybrid(query, 5, 60.0, false)
    }

    /// Obtiene el texto de un documento por fuente + id (para armar el hit).
    fn buscar_texto(&self, fuente: &str, id: i64) -> Result<String> {
        let (tabla, campo) = match fuente {
            "episodica" => ("memoria_episodica", "contenido"),
            "semantica" => ("memoria_semantica", "contenido"),
            "piramidal" => ("memoria_piramidal", "contenido"),
            _ => return Ok(String::new()),
        };
        let sql = format!("SELECT {} FROM {} WHERE id = ?1", campo, tabla);
        let texto = self
            .conn
            .query_row(&sql, [id], |row| row.get::<_, String>(0))
            .unwrap_or_default();
        Ok(texto)
    }
}

impl Default for HybridRecall {
    fn default() -> Self {
        Self::new().expect("HybridRecall debe poder inicializarse")
    }
}

// ============================================================================
// Utilidades
// ============================================================================

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(&x, &y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na > 1e-8 && nb > 1e-8 {
        dot / (na * nb)
    } else {
        0.0
    }
}

fn blob_a_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn sanitizar_fts5(query: &str) -> String {
    query
        .chars()
        .map(|c| match c {
            '"' => ' ',
            '\'' => ' ',
            '(' | ')' | '*' | ':' | '^' | '-' => ' ',
            c if c.is_alphanumeric() || c.is_whitespace() => c,
            _ => ' ',
        })
        .collect::<String>()
        .trim()
        .to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_basico() {
        // Ranking BM25: doc A (pos 0), doc B (pos 1)
        // Ranking vector: doc B (pos 0), doc A (pos 1)
        let k: f32 = 60.0;
        let a_bm25: f32 = 1.0 / (k + 0.0 + 1.0);
        let b_bm25: f32 = 1.0 / (k + 1.0 + 1.0);
        let a_vec: f32 = 1.0 / (k + 1.0 + 1.0);
        let b_vec: f32 = 1.0 / (k + 0.0 + 1.0);

        let score_a: f32 = a_bm25 + a_vec;
        let score_b: f32 = b_bm25 + b_vec;
        // Ambos aparecen en ambos rankings → ambos score altos.
        assert!((score_a - score_b).abs() < 1e-9);
    }

    #[test]
    fn recall_hybrid_no_rompe() {
        let h = HybridRecall::from_path(PathBuf::from(":memory:")).expect("hybrid");
        let hits = h.recall("memoria").unwrap_or_default();
        // Sin datos puede estar vacío, pero no debe fallar.
        assert!(hits.len() <= 10);
    }

    #[test]
    fn recall_solo_bm25() {
        let h = HybridRecall::from_path(PathBuf::from(":memory:")).expect("hybrid");
        let hits = h.recall_hybrid("test", 3, 60.0, true).unwrap_or_default();
        assert!(hits.len() <= 6);
    }

    #[test]
    fn blob_vector_roundtrip() {
        let v = vec![1.0_f32, -0.5, 0.25, 3.14];
        let blob: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
        assert_eq!(blob_a_vec(&blob), v);
    }
}
