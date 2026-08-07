//! Pipeline completo de scraping (F4.1–F4.3).
//!
//! Ensambla el flujo completo: **fetch → clean → route (threshold) → infer
//! (Tier-1 o Tier-2) → persistir** en SQLite.
//!
//! ```
//! ┌──────────┐   ┌───────────┐   ┌─────────┐   ┌──────────────────┐
//! │ Fetcher  │──▶│  Cleaner  │──▶│ Router  │──▶│ Tier-2 Cloud     │
//! │ (HTTP)   │   │ HTML→MD   │   │ ≤4k?    │   │ (CloudAdapter)   │
//! └──────────┘   └───────────┘   └────┬────┘   └──────────────────┘
//!                                     │ >4k
//!                                     ▼
//!                          ┌──────────────────┐
//!                          │ Tier-1 Ollama    │
//!                          │ Map-Reduce→resumen│
//!                          └──────────────────┘
//!                                     │
//!                                     ▼
//!                        ┌──────────────────────┐
//!                        │ SQLite: tasks + data │
//!                        └──────────────────────┘
//! ```

use crate::scraping::pipeline::cerebro::Cerebro;
use crate::scraping::pipeline::cleaner;
use crate::scraping::pipeline::cloud_adapter::CloudAdapter;
use crate::scraping::pipeline::db::PipelineDb;
use crate::scraping::pipeline::fetcher::Fetcher;
use crate::scraping::pipeline::map_reduce::{map_reduce, MapReduceOutput};
use crate::scraping::pipeline::metrics::Metrics;
use crate::scraping::pipeline::ollama_client::OllamaClient;
use crate::scraping::pipeline::router::{route, Route};
use crate::scraping::pipeline::schemas::{
    now_iso, ScrapingResult, ScrapingStatus, TaskSchema, TierUsed, Timing,
};
use crate::scraping::pipeline::token_counter::estimate;
use std::sync::Arc;

/// Configuración del pipeline.
#[derive(Clone)]
pub struct PipelineConfig {
    /// Si es `true`, persiste en SQLite (tasks + extracted_data).
    pub persist: bool,
    /// Si es `true`, ejecuta inferencia tier-2 tras la limpieza. Si `false`,
    /// solo devuelve el markdown limpio (modo "solo limpieza").
    pub infer: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            persist: true,
            infer: true,
        }
    }
}

/// Pipeline orquestador de scraping.
pub struct Pipeline {
    fetcher: Arc<Fetcher>,
    ollama: Option<Arc<OllamaClient>>,
    cloud: Option<Arc<CloudAdapter>>,
    db: Option<Arc<PipelineDb>>,
    config: PipelineConfig,
    metrics: Option<Arc<Metrics>>,
    cerebro: Option<Arc<Cerebro>>,
}

impl Pipeline {
    /// Construye el pipeline con todos sus componentes.
    pub fn new(
        fetcher: Arc<Fetcher>,
        ollama: Option<Arc<OllamaClient>>,
        cloud: Option<Arc<CloudAdapter>>,
        db: Option<Arc<PipelineDb>>,
        config: PipelineConfig,
    ) -> Self {
        Self {
            fetcher,
            ollama,
            cloud,
            db,
            config,
            metrics: None,
            cerebro: None,
        }
    }

    /// Asocia métricas opcionales al pipeline.
    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Asocia el Cerebro Auto-Creciente (E2) para indexar resultados.
    pub fn with_cerebro(mut self, cerebro: Arc<Cerebro>) -> Self {
        self.cerebro = Some(cerebro);
        self
    }

    /// Indexa un resultado exitoso en el Cerebro (hook post-scraping E2).
    async fn index_in_cerebro(&self, result: &ScrapingResult) {
        if let Some(cerebro) = &self.cerebro {
            if result.status == ScrapingStatus::Success {
                match cerebro.index_result(result).await {
                    Ok(n) if n > 0 => {
                        tracing::info!(
                            "🧠 [CEREBRO] indexados {n} chunks de {}",
                            result.task_id
                        );
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("⚠️ [CEREBRO] no se indexó {}: {e}", result.task_id);
                    }
                }
            }
        }
    }

    /// Flujo completo: fetch → clean → route → infer → persistir.
    pub async fn run(&self, task: &TaskSchema) -> ScrapingResult {
        let mut result = ScrapingResult::success(&task.task_id);

        // 1. FETCH
        let fetch_start = std::time::Instant::now();
        let fetch_out = match self.fetcher.fetch(task).await {
            Ok(out) => out,
            Err(e) => {
                let err_msg = e.to_string();
                result.status = if err_msg.contains("blocked_by_robots") {
                    ScrapingStatus::BlockedByRobots
                } else {
                    ScrapingStatus::Failed
                };
                result.errors.push(err_msg);
                self.persist(&result, None, None);
                return result;
            }
        };
        let fetch_ms = fetch_start.elapsed().as_millis() as u64;
        result.url_final = Some(fetch_out.final_url.clone());
        result.timing_ms = Some(Timing {
            fetch: Some(fetch_ms),
            clean: None,
            inference_total: None,
        });

        // 2. CLEAN
        let clean_start = std::time::Instant::now();
        let exclude: Vec<String> = task
            .selectors
            .as_ref()
            .map(|s| s.exclude.clone())
            .unwrap_or_default();
        let markdown = cleaner::clean(&fetch_out.html, &exclude);
        let clean_ms = clean_start.elapsed().as_millis() as u64;
        let token_count = estimate(&markdown);

        result.cleaned_markdown = Some(markdown.clone());
        result.token_count = Some(token_count);
        if let Some(t) = result.timing_ms.as_mut() {
            t.clean = Some(clean_ms);
        }

        // Si el markdown quedó vacío → partial.
        if markdown.trim().is_empty() {
            result.status = ScrapingStatus::Partial;
            result.errors.push("contenido limpio vacío".into());
            self.persist(&result, None, None);
            return result;
        }

        // 3. ROUTE + INFER
        if self.config.infer {
            match route(&markdown) {
                Route::DirectToCloud => {
                    result.tier_used = Some(TierUsed::Tier2Cloud);
                    self.infer_tier2(&task.task_id, &markdown, &mut result).await;
                }
                Route::MapReduceLocal => {
                    result.tier_used = Some(TierUsed::Tier1ThenTier2);
                    self.infer_tier1_then_tier2(&task.task_id, &markdown, &mut result)
                        .await;
                }
            }
        }

        // 4. PERSIST + MÉTRICAS + CEREBRO (E2)
        if let Some(m) = &self.metrics {
            m.record_task_status(result.status.as_str());
        }
        let extracted = result.extracted_data.clone();
        self.persist(&result, Some(&markdown), extracted.as_ref());
        self.index_in_cerebro(&result).await;
        result
    }

    /// Procesa HTML ya descargado (sin fetch) — útil para tests y para el
    /// modo "solo limpieza".
    pub async fn process_html(&self, task: &TaskSchema, html: &str) -> ScrapingResult {
        let mut result = ScrapingResult::success(&task.task_id);
        let exclude: Vec<String> = task
            .selectors
            .as_ref()
            .map(|s| s.exclude.clone())
            .unwrap_or_default();
        let markdown = cleaner::clean(html, &exclude);
        let token_count = estimate(&markdown);
        result.cleaned_markdown = Some(markdown.clone());
        result.token_count = Some(token_count);
        result.timing_ms = Some(Timing {
            fetch: None,
            clean: None,
            inference_total: None,
        });

        if markdown.trim().is_empty() {
            result.status = ScrapingStatus::Partial;
            result.errors.push("contenido limpio vacío".into());
            self.persist(&result, None, None);
            return result;
        }

        if self.config.infer {
            match route(&markdown) {
                Route::DirectToCloud => {
                    result.tier_used = Some(TierUsed::Tier2Cloud);
                    self.infer_tier2(&task.task_id, &markdown, &mut result).await;
                }
                Route::MapReduceLocal => {
                    result.tier_used = Some(TierUsed::Tier1ThenTier2);
                    self.infer_tier1_then_tier2(&task.task_id, &markdown, &mut result)
                        .await;
                }
            }
        }

        let extracted = result.extracted_data.clone();
        self.persist(&result, Some(&markdown), extracted.as_ref());
        self.index_in_cerebro(&result).await;
        result
    }

    /// Inferencia tier-2 (nube): envía el markdown al CloudAdapter.
    async fn infer_tier2(&self, task_id: &str, markdown: &str, result: &mut ScrapingResult) {
        let start = std::time::Instant::now();
        let token_count = result.token_count.unwrap_or(0);
        match &self.cloud {
            Some(adapter) => {
                let prompt = format!(
                    "Analiza la siguiente información extraída de la web y extrae los datos relevantes.\n\
                     Devuelve JSON estructurado.\n\n{markdown}"
                );
                match adapter.reason_json(&prompt).await {
                    Ok((value, provider)) => {
                        result.extracted_data = Some(value);
                        result.cloud_provider_used =
                            crate::scraping::pipeline::schemas::cloud_provider_from_str(provider);
                        result.status = ScrapingStatus::Success;
                    }
                    Err(e) => {
                        result.status = ScrapingStatus::Failed;
                        result.errors.push(format!("tier-2: {e}"));
                    }
                }
            }
            None => {
                // Sin adaptador de nube → éxito parcial (solo markdown limpio).
                result.status = ScrapingStatus::Partial;
                result
                    .errors
                    .push("sin CloudAdapter configurado; solo limpieza".into());
            }
        }
        if let Some(t) = result.timing_ms.as_mut() {
            t.inference_total = Some(start.elapsed().as_millis() as u64);
        }
        if let Some(m) = &self.metrics {
            m.record_tier2_call(token_count, start.elapsed().as_millis() as u64);
        }
        let _ = task_id;
    }

    /// Inferencia tier-1 (Ollama Map-Reduce) → resumen → tier-2.
    async fn infer_tier1_then_tier2(
        &self,
        task_id: &str,
        markdown: &str,
        result: &mut ScrapingResult,
    ) {
        let start = std::time::Instant::now();
        let Some(ollama) = &self.ollama else {
            result.status = ScrapingStatus::Partial;
            result.errors.push("sin Ollama configurado; solo limpieza".into());
            return;
        };

        // Map-Reduce local sobre el texto masivo.
        let map_out: MapReduceOutput = match map_reduce(ollama, task_id, markdown, &Default::default())
            .await
        {
            Ok(out) => out,
            Err(e) => {
                result.status = ScrapingStatus::Failed;
                result.errors.push(format!("tier-1 map-reduce: {e}"));
                return;
            }
        };

        result.scratchpad_path = Some(map_out.scratchpad_path.clone());
        result.extracted_data = Some(map_out.consolidated.clone());

        // Opcional: enviar el resumen al tier-2 si está disponible.
        if let Some(adapter) = &self.cloud {
            let summary_json = map_out.consolidated.to_string();
            let prompt = format!(
                "Analiza el siguiente resumen estructurado extraído de una web y genera la respuesta final.\n\n{summary_json}"
            );
            match adapter.reason(&prompt).await {
                Ok(_content) => {
                    result.status = ScrapingStatus::Success;
                }
                Err(e) => {
                    result.status = ScrapingStatus::Partial;
                    result.errors.push(format!("tier-2 tras map-reduce: {e}"));
                }
            }
        } else {
            result.status = ScrapingStatus::Success;
        }

        if let Some(t) = result.timing_ms.as_mut() {
            t.inference_total = Some(start.elapsed().as_millis() as u64);
        }
        if let Some(m) = &self.metrics {
            m.record_tier1_call(
                result.token_count.unwrap_or(0),
                start.elapsed().as_millis() as u64,
            );
        }
    }

    /// Persiste el resultado en SQLite (si está habilitado).
    fn persist(
        &self,
        result: &ScrapingResult,
        markdown: Option<&str>,
        extracted: Option<&serde_json::Value>,
    ) {
        if !self.config.persist {
            return;
        }
        let Some(db) = &self.db else {
            return;
        };

        let status = result.status.as_str();
        let token_count = result.token_count.map(|v| v as i64);
        let error_log = if result.errors.is_empty() {
            None
        } else {
            Some(result.errors.join("; "))
        };

        let _ = db.update_status(&result.task_id, status, token_count, error_log.as_deref());

        if let Some(data) = extracted {
            let data_str = serde_json::to_string(data).unwrap_or_default();
            let summary = result.summary.as_deref();
            let scratch = result.scratchpad_path.as_deref();
            let _ = db.insert_extracted_data(&result.task_id, &data_str, summary, scratch);
        }
        let _ = markdown;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraping::pipeline::cloud_adapter::{CloudAdapter, MockProvider};
    use crate::scraping::pipeline::fetcher::Fetcher;

    fn test_task() -> TaskSchema {
        TaskSchema {
            task_id: "task-integracion-1".into(),
            url: "https://example.com/test".into(),
            strategy: crate::scraping::pipeline::schemas::Strategy::Http,
            selectors: None,
            output_schema: None,
            timeout_seconds: 30,
            max_retries: 1,
            respect_robots_txt: false,
            rate_limit_delay_ms: 500,
            user_agent: "TestBot/1.0".into(),
            metadata: None,
        }
    }

    fn small_pipeline() -> Pipeline {
        let fetcher = Arc::new(Fetcher::new(None).unwrap());
        let cloud = Arc::new(CloudAdapter::new(vec![Box::new(MockProvider {
            name: "mock",
            should_fail: false,
            response: "{\"items\": [\"a\", \"b\"]}".into(),
        })]));
        Pipeline::new(
            fetcher,
            None, // sin Ollama
            Some(cloud),
            None, // sin DB
            PipelineConfig {
                persist: false,
                infer: true,
            },
        )
    }

    #[tokio::test]
    async fn procesa_html_corto_via_tier2() {
        let pipeline = small_pipeline();
        let task = test_task();
        let html = "<html><body><h1>Producto</h1><p>Laptop gamer a 1200 USD.</p></body></html>";
        let result = pipeline.process_html(&task, html).await;

        assert_eq!(result.status, ScrapingStatus::Success);
        assert_eq!(result.tier_used, Some(TierUsed::Tier2Cloud));
        assert!(result.cleaned_markdown.unwrap().contains("Laptop gamer"));
        assert_eq!(result.extracted_data.as_ref().unwrap()["items"][0], "a");
    }

    #[tokio::test]
    async fn html_vacio_devuelve_partial() {
        let pipeline = small_pipeline();
        let task = test_task();
        let result = pipeline.process_html(&task, "<html><body><script>alert(1)</script></body></html>").await;
        assert_eq!(result.status, ScrapingStatus::Partial);
    }

    #[tokio::test]
    async fn html_masivo_sin_ollama_devuelve_partial() {
        let pipeline = small_pipeline();
        let task = test_task();
        // > 10,000 chars → masivo (>4,000 tokens) → requiere Ollama.
        let big_text = "Contenido ".repeat(3000);
        let html = format!("<html><body><p>{big_text}</p></body></html>");
        let result = pipeline.process_html(&task, &html).await;
        // Sin Ollama configurado → partial con error.
        assert_eq!(result.tier_used, Some(TierUsed::Tier1ThenTier2));
        assert_eq!(result.status, ScrapingStatus::Partial);
        assert!(result.errors.iter().any(|e| e.contains("Ollama")));
    }

    #[tokio::test]
    async fn modo_sin_inferencia_solo_limpia() {
        let fetcher = Arc::new(Fetcher::new(None).unwrap());
        let pipeline = Pipeline::new(
            fetcher,
            None,
            None,
            None,
            PipelineConfig {
                persist: false,
                infer: false,
            },
        );
        let task = test_task();
        let html = "<html><body><h1>Hola</h1><nav>menu</nav><p>mundo</p></body></html>";
        let result = pipeline.process_html(&task, html).await;
        let md = result.cleaned_markdown.unwrap();
        assert!(md.contains("Hola"));
        assert!(!md.contains("menu"));
        assert_eq!(result.tier_used, None);
    }
}
