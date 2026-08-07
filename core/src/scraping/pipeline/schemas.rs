//! Contratos de datos del pipeline de scraping (F0.3).
//!
//! Espejo en Rust del JSON Schema de `plans/pipeline-spec.md` §2.1 y §2.2.
//! - [`TaskSchema`] — tarea de scraping emitida por el orquestador.
//! - [`ScrapingResult`] — resultado devuelto por el pipeline.
//! - [`Strategy`] — estrategia de captura (http vs headless).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Estrategia de captura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// reqwest: HTML estático.
    Http,
    /// chromiumoxide: SPA / renderizado JS.
    Headless,
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Http
    }
}

/// Selectores CSS opcionales para extracción focalizada.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Selectors {
    #[serde(default)]
    pub main_content: Option<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub target_elements: Vec<String>,
}

/// Tarea de scraping (Task Schema — spec §2.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskSchema {
    /// UUID v4 único para trazabilidad.
    pub task_id: String,
    /// URL objetivo del scraping.
    pub url: String,
    /// Estrategia de captura.
    #[serde(default)]
    pub strategy: Strategy,
    /// Selectores CSS opcionales.
    #[serde(default)]
    pub selectors: Option<Selectors>,
    /// Esquema JSON esperado del resultado (opcional → extracción libre).
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,
    /// Timeout de la operación en segundos.
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Número máximo de reintentos.
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Respetar robots.txt.
    #[serde(default = "default_true")]
    pub respect_robots_txt: bool,
    /// Retraso mínimo entre peticiones al mismo dominio (ms).
    #[serde(default = "default_rate_limit")]
    pub rate_limit_delay_ms: u64,
    /// User-Agent identificable con URL de contacto.
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
    /// Metadatos opcionales para trazabilidad.
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

fn default_timeout() -> u64 {
    30
}
fn default_max_retries() -> u32 {
    3
}
fn default_true() -> bool {
    true
}
fn default_rate_limit() -> u64 {
    2000
}
fn default_user_agent() -> String {
    "NexusScraper/1.0 (+https://github.com/NEXUS_ULTIMATE_CORE)".to_string()
}

impl TaskSchema {
    /// Valida los campos requeridos y los rangos definidos en el schema.
    pub fn validate(&self) -> Result<(), String> {
        if self.task_id.is_empty() {
            return Err("task_id no puede estar vacío".into());
        }
        if self.url.is_empty() || !self.url.starts_with("http") {
            return Err(format!("url inválida: {}", self.url));
        }
        if !(5..=120).contains(&self.timeout_seconds) {
            return Err("timeout_seconds debe estar entre 5 y 120".into());
        }
        if self.max_retries > 5 {
            return Err("max_retries no puede exceder 5".into());
        }
        if self.rate_limit_delay_ms < 500 {
            return Err("rate_limit_delay_ms debe ser >= 500".into());
        }
        Ok(())
    }
}

/// Estado de un resultado de scraping (spec §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrapingStatus {
    Success,
    Partial,
    Failed,
    BlockedByRobots,
    Timeout,
    ProviderExhausted,
}

impl ScrapingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScrapingStatus::Success => "success",
            ScrapingStatus::Partial => "partial",
            ScrapingStatus::Failed => "failed",
            ScrapingStatus::BlockedByRobots => "blocked_by_robots",
            ScrapingStatus::Timeout => "timeout",
            ScrapingStatus::ProviderExhausted => "provider_exhausted",
        }
    }
}

/// Tier(s) que procesaron un resultado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierUsed {
    Tier1Slm,
    Tier2Cloud,
    Tier1ThenTier2,
}

/// Proveedor de nube utilizado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudProvider {
    OpenRouter,
    Gemini,
    DeepSeek,
    SovereignWeb,
    Groq,
    Cerebras,
    GoogleAiStudio,
}

/// Resultado de scraping (ScrapingResult — spec §2.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScrapingResult {
    pub task_id: String,
    pub status: ScrapingStatus,
    /// URL final tras redirects.
    #[serde(default)]
    pub url_final: Option<String>,
    /// Contenido limpio en Markdown.
    #[serde(default)]
    pub cleaned_markdown: Option<String>,
    /// Conteo estimado de tokens.
    #[serde(default)]
    pub token_count: Option<u64>,
    /// Qué tier(s) procesaron este resultado.
    #[serde(default)]
    pub tier_used: Option<TierUsed>,
    /// Datos estructurados extraídos.
    #[serde(default)]
    pub extracted_data: Option<serde_json::Value>,
    /// Resumen generado por LLM/SLM (~200-500 tokens).
    #[serde(default)]
    pub summary: Option<String>,
    /// Ruta al .jsonl si se usó Map-Reduce.
    #[serde(default)]
    pub scratchpad_path: Option<String>,
    /// Proveedor de nube usado.
    #[serde(default)]
    pub cloud_provider_used: Option<CloudProvider>,
    /// Errores acumulados.
    #[serde(default)]
    pub errors: Vec<String>,
    /// Timings en milisegundos.
    #[serde(default)]
    pub timing_ms: Option<Timing>,
    /// Timestamp de creación (ISO 8601).
    pub created_at: String,
}

/// Desglose de tiempos de ejecución (spec §2.2 timing_ms).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Timing {
    pub fetch: Option<u64>,
    pub clean: Option<u64>,
    pub inference_total: Option<u64>,
}

impl ScrapingResult {
    /// Crea un resultado de éxito con valores por defecto.
    pub fn success(task_id: &str) -> Self {
        Self {
            task_id: task_id.to_string(),
            status: ScrapingStatus::Success,
            url_final: None,
            cleaned_markdown: None,
            token_count: None,
            tier_used: None,
            extracted_data: None,
            summary: None,
            scratchpad_path: None,
            cloud_provider_used: None,
            errors: Vec::new(),
            timing_ms: None,
            created_at: now_iso(),
        }
    }

    /// Crea un resultado de error.
    pub fn failed(task_id: &str, error: impl Into<String>) -> Self {
        let mut r = Self::success(task_id);
        r.status = ScrapingStatus::Failed;
        r.errors.push(error.into());
        r
    }
}

/// Mapea el nombre de un proveedor (de `CloudProvider::name()`) a la enum
/// [`CloudProvider`] del resultado.
pub fn cloud_provider_from_str(name: &str) -> Option<CloudProvider> {
    match name {
        "openrouter" => Some(CloudProvider::OpenRouter),
        "gemini" => Some(CloudProvider::Gemini),
        "deepseek" => Some(CloudProvider::DeepSeek),
        "sovereign_web" | "sovereign" | "sovereign/web" => Some(CloudProvider::SovereignWeb),
        "groq" => Some(CloudProvider::Groq),
        "cerebras" => Some(CloudProvider::Cerebras),
        "google_ai_studio" => Some(CloudProvider::GoogleAiStudio),
        _ => None,
    }
}

/// Devuelve el timestamp actual en formato ISO 8601 (UTC).
pub fn now_iso() -> String {
    use chrono::Utc;
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_schema_deserializa_json_valido() {
        let json = r#"{
            "task_id": "550e8400-e29b-41d4-a716-446655440000",
            "url": "https://example.com/producto",
            "strategy": "http",
            "timeout_seconds": 30,
            "max_retries": 3
        }"#;
        let task: TaskSchema = serde_json::from_str(json).unwrap();
        assert_eq!(task.strategy, Strategy::Http);
        assert_eq!(task.timeout_seconds, 30);
        assert!(task.validate().is_ok());
    }

    #[test]
    fn task_schema_valida_url_invalida() {
        let task = TaskSchema {
            task_id: "id-1".into(),
            url: "no-es-una-url".into(),
            strategy: Strategy::Http,
            selectors: None,
            output_schema: None,
            timeout_seconds: 30,
            max_retries: 3,
            respect_robots_txt: true,
            rate_limit_delay_ms: 2000,
            user_agent: "test".into(),
            metadata: None,
        };
        assert!(task.validate().is_err());
    }

    #[test]
    fn result_status_serializa_correctamente() {
        let r = ScrapingResult::success("task-1");
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"status\":\"success\""));
    }

    #[test]
    fn result_failed_acumula_errores() {
        let r = ScrapingResult::failed("task-1", "timeout");
        assert_eq!(r.status, ScrapingStatus::Failed);
        assert_eq!(r.errors.len(), 1);
    }
}
