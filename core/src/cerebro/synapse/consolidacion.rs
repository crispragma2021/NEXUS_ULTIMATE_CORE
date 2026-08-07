// file:///home/soberano/NEXUS_ULTIMATE_CORE/core/src/cerebro/synapse/consolidacion.rs
// ============================================================================
// 🧬 CONSOLIDACIÓN SINÁPTICA — Monitor Cognitivo OMEGA
// ============================================================================
// Absorbido de: legacy/nexus-orquestador/src/chappie/consolidacion_sinaptica.rs
// Propósito: Monitoriza cambios en el workspace, genera sinapsis conceptuales
//            y persiste conexiones semánticas en SQLite.
// 🔱 Soberanía total: NexusEmbedder (768-dim) + heurística pura Rust.
// CERO dependencia en Ollama o servicios externos.
// ============================================================================

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use rusqlite::{params, Connection};
use serde_json;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::nexus_embedder::NexusEmbedder;

/// 🧬 Monitor Cognitivo: el subconsciente que consolida sinapsis mientras trabajas.
pub struct MonitorCognitivo {
    workspace_path: PathBuf,
    pub db_path: String,
}

impl MonitorCognitivo {
    pub fn new(workspace: &str, db_path: &str) -> Self {
        let monitor = Self {
            workspace_path: PathBuf::from(workspace),
            db_path: db_path.to_string(),
        };
        let _ = monitor.inicializar_tabla_sinapsis();
        monitor
    }

    fn conectar(&self) -> rusqlite::Result<Connection> {
        Connection::open(&self.db_path)
    }

    fn inicializar_tabla_sinapsis(&self) -> Result<()> {
        let conn = self.conectar()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sinapsis (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                sinapsis TEXT NOT NULL,
                embedding TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sinapsis_file_path ON sinapsis(file_path)",
            [],
        )?;
        Ok(())
    }

    /// Filtro estricto para excluir ruido en el watcher
    fn deberia_monitorear(&self, path: &Path) -> bool {
        if let Some(path_str) = path.to_str() {
            if path_str.contains("/target/")
                || path_str.contains("/.git/")
                || path_str.contains("/.cargo/")
                || path_str.contains("/node_modules/")
                || path_str.contains(".db")
                || path_str.contains(".lock")
                || path_str.contains(".corrupted")
            {
                return false;
            }
        }
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") | Some("toml") | Some("md") | Some("yaml") | Some("json") => true,
            _ => false,
        }
    }

    /// Inicia la monitorización en segundo plano de manera no bloqueante
    pub async fn encender_subconsciente(self: Arc<Self>) -> Result<()> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let monitor_clone = Arc::clone(&self);

        // Hilo 1: FS Watcher
        let workspace = self.workspace_path.clone();
        tokio::spawn(async move {
            let mut watcher =
                notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Modify(_)) {
                            let _ = tx.send(event);
                        }
                    }
                })
                .unwrap();

            let _ = watcher.watch(&workspace, RecursiveMode::Recursive);

            // Mantener el watcher vivo
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });

        // Hilo 2: Consolidación Cognitiva
        tokio::spawn(async move {
            info!("🧠 [MONITOR] Subconsciente sináptico encendido.");
            while let Some(event) = rx.recv().await {
                for path in event.paths {
                    if monitor_clone.deberia_monitorear(&path) {
                        let monitor = Arc::clone(&monitor_clone);
                        tokio::spawn(async move {
                            if let Err(e) = monitor.procesar_cambio(&path).await {
                                error!("❌ [MONITOR] Error procesando {}: {}", path.display(), e);
                            }
                        });
                    }
                }
            }
        });

        Ok(())
    }

    async fn procesar_cambio(&self, path: &Path) -> Result<()> {
        // 1. Extraer contexto del archivo (últimas 25 líneas)
        let content = tokio::fs::read_to_string(path).await?;
        let lines: Vec<&str> = content.lines().rev().take(25).collect();
        let context = lines.into_iter().rev().collect::<Vec<&str>>().join("\n");
        if context.trim().is_empty() {
            return Ok(());
        }

        // 2. Generar Sinapsis conceptual (heurística pura, sin Ollama)
        let relative_path = path
            .strip_prefix(&self.workspace_path)
            .unwrap_or(path)
            .to_string_lossy();
        let sinapsis = Self::generar_resumen_sinapsis(&relative_path, &context);

        // 3. Generar Embedding (NexusEmbedder soberano)
        let embedding = Self::generar_embedding_soberano(&sinapsis);

        // 4. Persistir en SQLite
        let embedding_json = serde_json::to_string(&embedding)?;
        let conn = self.conectar()?;
        conn.execute(
            "INSERT INTO sinapsis (file_path, sinapsis, embedding) VALUES (?1, ?2, ?3)",
            params![relative_path.to_string(), sinapsis, embedding_json],
        )?;

        // Limpieza periódica: mantener solo las últimas 30 sinapsis por archivo
        conn.execute(
            "DELETE FROM sinapsis WHERE id NOT IN (
                SELECT id FROM sinapsis WHERE file_path = ?1 ORDER BY created_at DESC LIMIT 30
            ) AND file_path = ?1",
            [relative_path.to_string()],
        )?;

        info!(
            "🧠 [SINAPSIS GRABADA] {} -> \"{}\"",
            relative_path,
            &sinapsis[..sinapsis.len().min(80)]
        );
        Ok(())
    }

    /// 🔱 Heurística pura Rust: extrae keywords del contexto y genera resumen descriptivo.
    /// Reemplaza la llamada Ollama a deepseek-r1:14b con análisis sintáctico local.
    fn generar_resumen_sinapsis(file_name: &str, code_context: &str) -> String {
        let line_count = code_context.lines().count();
        let lower = code_context.to_lowercase();

        // Detectar construcciones Rust clave
        let mut features: Vec<&str> = Vec::new();
        if lower.contains("fn ") {
            features.push("funciones");
        }
        if lower.contains("struct ") {
            features.push("structs");
        }
        if lower.contains("impl ") {
            features.push("implementaciones");
        }
        if lower.contains("use ") {
            features.push("imports");
        }
        if lower.contains("mod ") {
            features.push("módulos");
        }
        if lower.contains("pub ") {
            features.push("API pública");
        }
        if lower.contains("async ") {
            features.push("async");
        }
        if lower.contains("unsafe ") {
            features.push("unsafe");
        }
        if lower.contains("todo!") || lower.contains("fixme") {
            features.push("pendientes");
        }
        if lower.contains("#[test]") || lower.contains("#[cfg(test)]") {
            features.push("tests");
        }
        if lower.contains("error") || lower.contains("err") {
            features.push("manejo de errores");
        }
        if lower.contains("trace") || lower.contains("info") || lower.contains("warn") {
            features.push("logging");
        }
        if lower.contains("mut ") {
            features.push("mutabilidad");
        }
        if lower.contains("clone") || lower.contains("arc") {
            features.push("manejo de memoria");
        }

        // Extraer nombres de funciones/structs (patrón simple)
        let mut symbol_names: HashSet<&str> = HashSet::new();
        for line in code_context.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if let Some(name) = rest.split(['(', '<']).next() {
                    if name.len() > 1 && name.len() < 60 {
                        symbol_names.insert(name);
                    }
                }
            }
            if let Some(rest) = trimmed.strip_prefix("pub fn ") {
                if let Some(name) = rest.split(['(', '<']).next() {
                    if name.len() > 1 && name.len() < 60 {
                        symbol_names.insert(name);
                    }
                }
            }
            if let Some(rest) = trimmed.strip_prefix("struct ") {
                if let Some(name) = rest.split(['{', '<', '(', ';']).next() {
                    if name.len() > 1 && name.len() < 60 {
                        symbol_names.insert(name);
                    }
                }
            }
        }

        let extension = std::path::Path::new(file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let lang = match extension {
            "rs" => "Rust",
            "toml" => "TOML",
            "md" => "Markdown",
            "yaml" | "yml" => "YAML",
            "json" => "JSON",
            "ts" | "tsx" => "TypeScript",
            "js" | "jsx" => "JavaScript",
            "py" => "Python",
            _ => extension,
        };

        if !features.is_empty() {
            let feat_str = features.join(", ");
            if !symbol_names.is_empty() {
                let names: Vec<&str> = symbol_names.into_iter().collect();
                let names_str = names.join(", ");
                format!(
                    "Modificación de {} en '{}' ({}) afectando {}: {}",
                    lang, file_name, line_count, feat_str, names_str
                )
            } else {
                format!(
                    "Modificación de {} en '{}' ({}) afectando {}",
                    lang, file_name, line_count, feat_str
                )
            }
        } else if !symbol_names.is_empty() {
            let names: Vec<&str> = symbol_names.into_iter().collect();
            let names_str = names.join(", ");
            format!(
                "Edición de {} en '{}' ({}) con símbolos: {}",
                lang, file_name, line_count, names_str
            )
        } else {
            format!(
                "Edición activa de {} líneas de código {} en '{}'",
                line_count, lang, file_name
            )
        }
    }

    /// 🔱 NexusEmbedder soberano: embedding 768-dim sin dependencia externa.
    fn generar_embedding_soberano(text: &str) -> Vec<f32> {
        NexusEmbedder::generar(text, &[])
    }

    /// Recupera el contexto asociativo comparando semánticamente
    pub fn recuperar_contexto(&self, file_path: &str, limit: usize) -> Result<Vec<String>> {
        let relative_path = file_path
            .replace("/opt/NEXUS_ULTIMATE_CORE/", "")
            .replace("/home/soberano/NEXUS_ULTIMATE_CORE/", "");

        let conn = self.conectar()?;
        let mut stmt = conn.prepare(
            "SELECT sinapsis, embedding FROM sinapsis WHERE file_path = ?1 ORDER BY created_at DESC LIMIT 50",
        )?;

        let rows = stmt.query_map([relative_path.clone()], |row| {
            let sinapsis: String = row.get(0)?;
            let embedding_str: String = row.get(1)?;
            Ok((sinapsis, embedding_str))
        })?;

        let mut candidatas = Vec::new();
        for r in rows.flatten() {
            if let Ok(emb) = serde_json::from_str::<Vec<f32>>(&r.1) {
                candidatas.push((r.0, emb));
            }
        }

        if candidatas.is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = candidatas[0].1.clone();

        let mut puntuaciones: Vec<(String, f32)> = candidatas
            .into_iter()
            .map(|(sinapsis, emb)| {
                let sim = coseno_similaridad(&query_embedding, &emb);
                (sinapsis, sim)
            })
            .collect();

        puntuaciones.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut final_sinapsis = Vec::new();
        for p in puntuaciones {
            if !final_sinapsis.contains(&p.0) {
                final_sinapsis.push(p.0);
            }
            if final_sinapsis.len() >= limit {
                break;
            }
        }

        Ok(final_sinapsis)
    }
}

pub fn coseno_similaridad(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }
    let dot: f32 = v1.iter().zip(v2).map(|(a, b)| a * b).sum();
    let mag1: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
    let mag2: f32 = v2.iter().map(|b| b * b).sum::<f32>().sqrt();
    if mag1 == 0.0 || mag2 == 0.0 {
        return 0.0;
    }
    dot / (mag1 * mag2)
}
