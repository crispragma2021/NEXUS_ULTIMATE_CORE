//! 🕸️ Módulo de scraping del pipeline NEXUS.
//!
//! Implementa las fases F0/F1 del plan [`plans/pipeline-spec.md`]:
//! - `pipeline::schemas` — contratos de datos (TaskSchema, ScrapingResult).
//! - `pipeline::db` — esquema SQLite (rusqlite) para persistencia.
//! - `pipeline::fetcher` — captura HTTP/headless con respeto a robots.txt.
//! - `pipeline::cleaner` — limpieza determinista HTML → Markdown.
//! - `pipeline::token_counter` — estimación de tokens.
//! - `pipeline::router` — enrutador de umbral (≤4k directo a nube, >4k Map-Reduce).

pub mod pipeline;
