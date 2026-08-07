//! Scratchpad (Bloc de Notas) para el caso masivo (F2.4).
//!
//! Almacena las extracciones parciales del SLM local en un archivo `.jsonl`
//! y consolida el resultado en un resumen compacto (~500 tokens).
//!
//! Estructura de cada línea (spec §2.3):
//! ```json
//! {"chunk_index":0,"chunk_token_count":1480,"extracted":{...},"model":"...","tokens_per_second":42.3,"timestamp":"..."}
//! ```

use crate::scraping::pipeline::schemas::now_iso;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Entrada del scratchpad .jsonl (spec §2.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ScratchpadEntry {
    pub chunk_index: usize,
    #[serde(default)]
    pub chunk_token_count: Option<u64>,
    /// Datos extraídos por el SLM para este chunk (JSON).
    pub extracted: Value,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub tokens_per_second: f64,
    pub timestamp: String,
}

impl ScratchpadEntry {
    pub fn new(chunk_index: usize, extracted: Value, model: &str, tokens_per_second: f64) -> Self {
        Self {
            chunk_index,
            chunk_token_count: None,
            extracted,
            model: model.to_string(),
            tokens_per_second,
            timestamp: now_iso(),
        }
    }
}

/// Gestor del bloc de notas en disco.
pub struct Scratchpad {
    path: PathBuf,
}

impl Scratchpad {
    /// Crea un scratchpad en `./scratchpad/{task_id}.jsonl`.
    pub fn new(task_id: &str) -> Result<Self> {
        let dir = PathBuf::from("scratchpad");
        std::fs::create_dir_all(&dir).context("creando directorio scratchpad")?;
        Ok(Self {
            path: dir.join(format!("{task_id}.jsonl")),
        })
    }

    /// Ruta del archivo.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Anexa una entrada como línea JSON (incremental).
    pub fn append(&self, entry: &ScratchpadEntry) -> Result<()> {
        let line = serde_json::to_string(entry).context("serializando entrada")?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .context("abriendo scratchpad")?;
        writeln!(file, "{line}").context("escribiendo scratchpad")?;
        Ok(())
    }

    /// Lee todas las entradas del scratchpad.
    pub fn read_all(&self) -> Result<Vec<ScratchpadEntry>> {
        let content = std::fs::read_to_string(&self.path).unwrap_or_default();
        let mut entries = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: ScratchpadEntry = serde_json::from_str(line)
                .with_context(|| format!("parseando línea {} del scratchpad", idx + 1))?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// Consolida todas las entradas en un JSON único de resumen.
    ///
    /// Combina arrays (`entities`, `key_facts`) y deduplica. Devuelve un objeto
    /// compacto que se envía al tier-2 en lugar del texto completo.
    pub fn consolidate(&self) -> Result<Value> {
        let entries = self.read_all()?;
        let mut entities: Vec<Value> = Vec::new();
        let mut prices: Vec<Value> = Vec::new();
        let mut key_facts: Vec<Value> = Vec::new();

        for e in &entries {
            if let Some(arr) = e.extracted.get("entities").and_then(|v| v.as_array()) {
                entities.extend(arr.iter().cloned());
            }
            if let Some(arr) = e.extracted.get("prices").and_then(|v| v.as_array()) {
                prices.extend(arr.iter().cloned());
            }
            if let Some(arr) = e.extracted.get("key_facts").and_then(|v| v.as_array()) {
                key_facts.extend(arr.iter().cloned());
            }
        }

        Ok(json!({
            "total_chunks": entries.len(),
            "entities": dedupe(entities),
            "prices": dedupe(prices),
            "key_facts": dedupe(key_facts),
        }))
    }

    /// Elimina el archivo del scratchpad (limpieza de recursos, spec §2.3).
    pub fn cleanup(&self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Deduplica un vector de valores JSON conservando el orden.
fn dedupe(values: Vec<Value>) -> Vec<Value> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for v in values {
        if seen.insert(v.clone()) {
            out.push(v);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn tmp_task_id() -> String {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("test-task-{n}")
    }

    #[test]
    fn anexa_y_lee_entradas_incrementalmente() {
        let sp = Scratchpad::new(&tmp_task_id()).unwrap();
        sp.append(&ScratchpadEntry::new(
            0,
            json!({"entities": ["A"], "prices": [], "key_facts": ["f1"]}),
            "test",
            10.0,
        ))
        .unwrap();
        sp.append(&ScratchpadEntry::new(
            1,
            json!({"entities": ["B"], "prices": [], "key_facts": ["f2"]}),
            "test",
            12.0,
        ))
        .unwrap();

        let entries = sp.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].chunk_index, 0);
        assert_eq!(entries[1].chunk_index, 1);
        sp.cleanup();
    }

    #[test]
    fn consolida_y_deduplica() {
        let sp = Scratchpad::new(&tmp_task_id()).unwrap();
        for i in 0..3 {
            sp.append(&ScratchpadEntry::new(
                i,
                json!({"entities": ["X", "Y"], "prices": [{"item": "A", "price": 1.0}], "key_facts": ["k"]}),
                "test",
                10.0,
            ))
            .unwrap();
        }
        let consolidated = sp.consolidate().unwrap();
        assert_eq!(consolidated["total_chunks"], 3);
        assert_eq!(consolidated["entities"].as_array().unwrap().len(), 2); // dedup
        assert_eq!(consolidated["prices"].as_array().unwrap().len(), 1);
        sp.cleanup();
    }

    #[test]
    fn consolidate_vacio_devuelve_arrays_vacios() {
        let sp = Scratchpad::new(&tmp_task_id()).unwrap();
        let consolidated = sp.consolidate().unwrap();
        assert_eq!(consolidated["total_chunks"], 0);
        assert_eq!(consolidated["entities"].as_array().unwrap().len(), 0);
        sp.cleanup();
    }
}
