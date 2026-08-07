//! Red de Conocimiento Multi-Fuente (E4).
//!
//! Cruza múltiples fuentes de datos de una página y detecta corroboraciones,
//! discrepancias y datos sin corroborar:
//! - **HTML** → texto limpio (vía `cleaner`).
//! - **JSON-LD / Microdata** → `<script type="application/ld+json">`.
//! - **OpenGraph / Twitter Cards** → `<meta>` tags.
//!
//! Para cada campo, si aparece idéntico en ≥2 fuentes → "corroborado";
//! si difiere → "conflicto"; si solo en una → "sin corroborar".

use anyhow::Result;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

/// Nivel de confianza de un campo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Campo idéntico en ≥2 fuentes.
    Corroborated,
    /// Campo difiere entre fuentes.
    Conflict,
    /// Solo presente en una fuente.
    Uncorroborated,
}

/// Campo consolidado con confianza.
#[derive(Debug, Clone)]
pub struct ConsolidatedField {
    pub key: String,
    pub value: String,
    pub confidence: Confidence,
    pub sources: Vec<String>,
}

/// Resultado del análisis multi-fuente.
#[derive(Debug, Clone)]
pub struct MultiSourceResult {
    /// Campos consolidados por clave.
    pub fields: Vec<ConsolidatedField>,
    /// JSON final con scores de confianza.
    pub json: Value,
    /// JSON-LD crudo extraído (si existe).
    pub json_ld: Option<Value>,
    /// Metadatos OpenGraph/Twitter extraídos.
    pub open_graph: HashMap<String, String>,
}

/// Extrae JSON-LD de una página HTML.
pub fn extract_json_ld(html: &str) -> Option<Value> {
    let document = Html::parse_document(html);
    if let Ok(selector) = Selector::parse(r#"script[type="application/ld+json"]"#) {
        for el in document.select(&selector) {
            let text = el.text().collect::<String>().trim().to_string();
            if let Ok(v) = serde_json::from_str::<Value>(&text) {
                return Some(v);
            }
        }
    }
    None
}

/// Extrae metadatos OpenGraph y Twitter Cards.
pub fn extract_open_graph(html: &str) -> HashMap<String, String> {
    let document = Html::parse_document(html);
    let mut out = HashMap::new();
    if let Ok(selector) = Selector::parse(r#"meta[property^="og:"], meta[name^="twitter:"]"#) {
        for el in document.select(&selector) {
            let property = el
                .value()
                .attr("property")
                .or_else(|| el.value().attr("name"))
                .map(|s| s.to_string());
            let content = el.value().attr("content").map(|s| s.to_string());
            if let (Some(p), Some(c)) = (property, content) {
                out.insert(p, c);
            }
        }
    }
    out
}

/// Extrae el título real del documento (`<title>` o primer `<h1>`).
fn extract_html_title(html: &str) -> String {
    let document = Html::parse_document(html);
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = document.select(&sel).next() {
            let t = el.text().collect::<String>().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }
    if let Ok(sel) = Selector::parse("h1") {
        if let Some(el) = document.select(&sel).next() {
            let t = el.text().collect::<String>().trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }
    String::new()
}

/// Analiza una página multi-fuente y consolida con verificación cruzada.
pub fn analyze(html: &str, markdown: &str) -> MultiSourceResult {
    let json_ld = extract_json_ld(html);
    let open_graph = extract_open_graph(html);

    // Recolectar pares (clave → valor) por fuente.
    let mut sources: HashMap<String, Vec<(String, String)>> = HashMap::new();

    // Fuente: HTML — título real del documento (<title> o primer h1 limpio).
    let html_title = extract_html_title(html);
    if !html_title.is_empty() {
        sources
            .entry("html".to_string())
            .or_default()
            .push(("title".into(), html_title));
    }

    // Fuente: JSON-LD (aplanar). Normaliza `name` → `title` para el título.
    if let Some(ld) = &json_ld {
        for (k, v) in flatten_json(ld) {
            let key = if k == "name" || k == "headline" {
                "title".to_string()
            } else {
                k
            };
            sources
                .entry("json_ld".to_string())
                .or_default()
                .push((key, v));
        }
    }

    // Fuente: OpenGraph (normalizar prefijos og:/twitter:).
    for (k, v) in &open_graph {
        let key = k
            .strip_prefix("og:")
            .or_else(|| k.strip_prefix("twitter:"))
            .unwrap_or(k)
            .to_string();
        sources
            .entry("opengraph".to_string())
            .or_default()
            .push((key, v.clone()));
    }

    // Consolidar por clave.
    let mut by_key: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (source, pairs) in &sources {
        for (k, v) in pairs {
            by_key
                .entry(k.clone())
                .or_default()
                .push((source.clone(), v.clone()));
        }
    }

    let mut fields = Vec::new();
    let mut json_map = serde_json::Map::new();
    for (key, entries) in by_key {
        let first_value = entries[0].1.clone();
        let distinct_values: std::collections::HashSet<String> =
            entries.iter().map(|(_, v)| v.clone()).collect();
        let sources_list: Vec<String> = entries.iter().map(|(s, _)| s.clone()).collect();

        let confidence = if distinct_values.len() == 1 && entries.len() >= 2 {
            Confidence::Corroborated
        } else if distinct_values.len() > 1 {
            Confidence::Conflict
        } else {
            Confidence::Uncorroborated
        };

        json_map.insert(
            key.clone(),
            serde_json::json!({
                "value": first_value,
                "confidence": match confidence {
                    Confidence::Corroborated => "corroborated",
                    Confidence::Conflict => "conflict",
                    Confidence::Uncorroborated => "uncorroborated",
                },
                "sources": sources_list,
            }),
        );

        fields.push(ConsolidatedField {
            key,
            value: first_value,
            confidence,
            sources: sources_list,
        });
    }

    MultiSourceResult {
        fields,
        json: Value::Object(json_map),
        json_ld,
        open_graph,
    }
}

/// Aplana un JSON anidado en pares clave→valor (solo strings/numbers).
fn flatten_json(value: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    flatten_json_rec("", value, &mut out);
    out
}

fn flatten_json_rec(prefix: &str, value: &Value, out: &mut Vec<(String, String)>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_json_rec(&key, v, out);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                flatten_json_rec(&format!("{prefix}[{i}]"), v, out);
            }
        }
        Value::String(s) => out.push((prefix.to_string(), s.clone())),
        Value::Number(n) => out.push((prefix.to_string(), n.to_string())),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraping::pipeline::cleaner;

    #[test]
    fn extrae_json_ld() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Product","name":"Laptop Pro"}</script></head></html>"#;
        let ld = extract_json_ld(html).unwrap();
        assert_eq!(ld["name"], "Laptop Pro");
    }

    #[test]
    fn extrae_open_graph() {
        let html = r#"<html><head><meta property="og:title" content="Mi Titulo"><meta name="twitter:card" content="summary"></head></html>"#;
        let og = extract_open_graph(html);
        assert_eq!(og.get("og:title").map(|s| s.as_str()), Some("Mi Titulo"));
        assert!(og.contains_key("twitter:card"));
    }

    #[test]
    fn corrobora_campos_coincidentes() {
        // Mismo título en HTML y JSON-LD → corroborado.
        let html = r#"<html><head><script type="application/ld+json">{"name":"Laptop"}</script></head><body><h1>Laptop</h1></body></html>"#;
        let md = cleaner::clean(html, &[]);
        let res = analyze(html, &md);
        // Buscar un campo corroborado.
        let corroborated = res
            .fields
            .iter()
            .any(|f| f.confidence == Confidence::Corroborated);
        assert!(corroborated);
    }

    #[test]
    fn detecta_conflicto_cuando_difieren() {
        let html = r#"<html><head><meta property="og:title" content="Titulo A"></head><body><h1>Titulo B</h1></body></html>"#;
        let md = cleaner::clean(html, &[]);
        let res = analyze(html, &md);
        let conflict = res.fields.iter().any(|f| f.confidence == Confidence::Conflict);
        assert!(conflict);
    }
}
