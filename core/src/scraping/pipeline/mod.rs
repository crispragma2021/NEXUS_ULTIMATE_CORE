//! 🕸️ Pipeline de scraping NEXUS (F0/F1).
//!
//! Sub-módulos:
//! - [`schemas`] — TaskSchema y ScrapingResult (serde, JSON).
//! - [`db`] — persistencia SQLite con `rusqlite`.
//! - [`fetcher`] — captura determinista.
//! - [`cleaner`] — limpieza HTML → Markdown.
//! - [`token_counter`] — estimación de tokens.
//! - [`router`] — enrutador de umbral.

pub mod cerebro;
pub mod cleaner;
pub mod cloud_adapter;
pub mod daemon;
pub mod db;
pub mod embedding;
pub mod episodic_memory;
pub mod fetcher;
pub mod judge;
pub mod logging;
pub mod map_reduce;
pub mod metrics;
pub mod multi_source;
pub mod observatory;
pub mod ollama_client;
pub mod pipeline;
pub mod provider_circuit;
pub mod rate_limiter;
pub mod router;
pub mod schemas;
pub mod scratchpad;
pub mod token_counter;
pub mod vector_store;

pub use schemas::{ScrapingResult, ScrapingStatus, Strategy, TaskSchema};
