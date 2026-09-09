// ============================================================================
// 🧠 RETRIEVAL ENGINE — Sistema de Recuperación Universal RAG
// ============================================================================
// Propósito: Coordina la recuperación de contexto desde múltiples fuentes:
//            - Codebase knowledge (LanceDB table "codebase_knowledge")
//            - Ocean emocional (memoria episódica existente)
//            - Corteza Asociativa (red conceptual)
//            Unifica, rerankea, y formatea para inyección en el prompt.
// ============================================================================

use crate::cerebro::organos::reranker::Reranker;
use crate::memoria::memoria_semantica::MemoriaSemantica;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// Fuente de recuperación
#[derive(Debug, Clone, PartialEq)]
pub enum RetrievalSource {
    /// Tabla de conocimiento del codebase (LanceDB)
    CodebaseKnowledge,
    /// Memoria emocional episódica (Ocean)
    OceanEmotional,
    /// Corteza Asociativa (conceptos)
    CortezaAsociativa,
    /// Synapse consolidación (SQLite local)
    SynapseLocal,
}

/// Resultado unificado de recuperación
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub content: String,
    pub source: RetrievalSource,
    pub score: f32,
    pub file_path: Option<String>,
}

/// Configuración de recuperación para una consulta
#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    /// Límite máximo de resultados totales
    pub max_results: usize,
    /// Límite por fuente
    pub per_source: usize,
    /// Si debe incluir Ocean
    pub include_ocean: bool,
    /// Si debe incluir codebase knowledge
    pub include_codebase: bool,
    /// Si debe incluir corteza asociativa
    pub include_corteza: bool,
    /// Umbral de score mínimo
    pub threshold: f32,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            per_source: 5,
            include_ocean: true,
            include_codebase: true,
            include_corteza: true,
            threshold: 0.35,
        }
    }
}

/// Motor de Recuperación Universal — el Tálamo RAG
pub struct RetrievalEngine {
    pub memoria_semantica: Arc<MemoriaSemantica>,
    pub reranker: Reranker,
    pub table_name: String,
}

impl RetrievalEngine {
    pub fn new(memoria_semantica: Arc<MemoriaSemantica>, table_name: &str) -> Self {
        Self {
            memoria_semantica,
            reranker: Reranker::default(),
            table_name: table_name.to_string(),
        }
    }

    /// Configura reranker personalizado
    pub fn with_reranker(mut self, reranker: Reranker) -> Self {
        self.reranker = reranker;
        self
    }

    // ─── Recuperación principal (universal) ────────────────────────────────

    /// Recupera contexto relevante de TODAS las fuentes, rerankea y unifica.
    /// Devuelve un string listo para inyectar en el prompt.
    pub async fn recuperar(&self, query: &str, config: &RetrievalConfig) -> Vec<RetrievalResult> {
        let mut all_results = Vec::new();

        // 1. Codebase Knowledge (LanceDB)
        if config.include_codebase {
            if let Ok(knowledge) = self.recuperar_codebase(query, config.per_source).await {
                all_results.extend(knowledge);
            }
        }

        // 2. Ocean (memoria emocional episódica)
        if config.include_ocean {
            if let Ok(ocean) = self.recuperar_ocean(query, config.per_source).await {
                all_results.extend(ocean);
            }
        }

        // 3. Corteza Asociativa (si está disponible como callback)
        // Nota: la corteza asociativa se inyecta aparte porque opera sobre conceptos

        if all_results.is_empty() {
            return vec![];
        }

        // Rerankear unificadamente
        let raw: Vec<(String, f32)> = all_results
            .iter()
            .map(|r| (r.content.clone(), r.score))
            .collect();

        let reranked = self.reranker.rerank(query, &raw);

        // Reconstruir resultados con metadatos originales
        // Mapeamos contenido rerank → resultado original
        let reranked_contents: std::collections::HashSet<String> =
            reranked.results.iter().map(|r| r.content.clone()).collect();

        let mut final_results: Vec<RetrievalResult> = all_results
            .into_iter()
            .filter(|r| reranked_contents.contains(&r.content))
            .collect();

        // Ordenar por score rerank (usamos el orden del reranker)
        let score_map: std::collections::HashMap<String, f32> = reranked
            .results
            .iter()
            .map(|r| (r.content.clone(), r.rerank_score))
            .collect();

        final_results.sort_by(|a, b| {
            score_map
                .get(&b.content)
                .unwrap_or(&0.0)
                .partial_cmp(score_map.get(&a.content).unwrap_or(&0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncar al límite máximo
        final_results.truncate(config.max_results);

        info!(
            "🧠 [RETRIEVAL] {} resultados tras rerank (de {} fuentes)",
            final_results.len(),
            if config.include_codebase && config.include_ocean {
                "codebase+ocean"
            } else if config.include_codebase {
                "codebase"
            } else {
                "ocean"
            }
        );

        final_results
    }

    /// Version simplificada: devuelve string listo para inyectar
    pub async fn recuperar_contexto(&self, query: &str) -> String {
        let config = RetrievalConfig {
            include_corteza: false, // No tenemos acceso directo aquí
            ..Default::default()
        };
        let results = self.recuperar(query, &config).await;

        if results.is_empty() {
            return String::new();
        }

        let mut contexto = String::new();
        contexto.push_str("\n### 🧠 CONOCIMIENTO RECUPERADO (RAG UNIVERSAL):\n");

        for (i, r) in results.iter().enumerate() {
            let fuente_icono = match r.source {
                RetrievalSource::CodebaseKnowledge => "📄",
                RetrievalSource::OceanEmotional => "💭",
                RetrievalSource::CortezaAsociativa => "🔗",
                RetrievalSource::SynapseLocal => "🧬",
            };
            let ref_info = if let Some(ref path) = r.file_path {
                format!(" [{}]", path)
            } else {
                String::new()
            };
            contexto.push_str(&format!(
                "{}. {} [Relevancia: {:.2}]{}: {}\n",
                i + 1,
                fuente_icono,
                r.score,
                ref_info,
                &r.content[..r.content.len().min(300)]
            ));
        }

        contexto
    }

    // ─── Recuperación por fuente ───────────────────────────────────────────

    /// Recupera del codebase knowledge (LanceDB table "codebase_knowledge")
    async fn recuperar_codebase(&self, query: &str, limit: usize) -> Result<Vec<RetrievalResult>> {
        let embedding = self.memoria_semantica.generar_embedding(query).await?;

        let similares = self
            .memoria_semantica
            .buscar_similares_en_tabla(&embedding, limit * 2, &self.table_name)
            .await?;

        let mut results = Vec::new();
        for (id, score) in similares {
            results.push(RetrievalResult {
                content: format!("[chunk:{}]", id),
                source: RetrievalSource::CodebaseKnowledge,
                score: 1.0 - score.clamp(0.0, 1.0), // Convertir distancia a similitud
                file_path: None,
            });
        }

        Ok(results)
    }

    /// Recupera de Ocean (memoria emocional episódica)
    async fn recuperar_ocean(&self, query: &str, limit: usize) -> Result<Vec<RetrievalResult>> {
        let embedding = self.memoria_semantica.generar_embedding(query).await?;

        let similares = self
            .memoria_semantica
            .buscar_similares_en_tabla(&embedding, limit * 2, "ocean_vectors")
            .await?;

        let mut results = Vec::new();
        for (id, score) in similares {
            results.push(RetrievalResult {
                content: format!("[ocean:{}]", id),
                source: RetrievalSource::OceanEmotional,
                score: 1.0 - score.clamp(0.0, 1.0),
                file_path: None,
            });
        }

        Ok(results)
    }

    // ─── Estadísticas ──────────────────────────────────────────────────────

    /// Retorna conteo de vectores en cada tabla relevante
    pub async fn diagnosticar(&self) -> String {
        let codebase_count = self
            .memoria_semantica
            .verificar_estado_lancedb()
            .await
            .unwrap_or(0);

        let ocean_count = self
            .memoria_semantica
            .contar_en_tabla("ocean_vectors")
            .await
            .unwrap_or(0);

        format!(
            "🧠 RAG OMEGA:\n  - Codebase Knowledge: {} vectores\n  - Ocean Emotional: {} vectores\n  - Reranker: activo (umbral={:.2})",
            codebase_count, ocean_count, self.reranker.base_threshold
        )
    }
}
