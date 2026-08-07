//! Juez Evaluador (E3).
//!
//! Un LLM grande gratuito (vía [`CloudAdapter`]) supervisa la calidad de las
//! extracciones del SLM local. Evalúa 3 métricas:
//! 1. **Faithfulness** — ¿los datos extraídos están realmente en el HTML?
//! 2. **Completeness** — ¿faltan campos esperados?
//! 3. **Format compliance** — ¿JSON válido según el schema?
//!
//! Devuelve un score 0.0–1.0 y decide si la extracción se guarda, se marca
//! para retroalimentación, o se descarta (spec §3).

use crate::scraping::pipeline::cloud_adapter::CloudAdapter;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Umbrales del Juez (spec §3):
/// - Score ≥ `ACCEPT_THRESHOLD` → guardar en SQLite + vector DB.
/// - Score ≥ `RETRAIN_THRESHOLD` → guardar + registrar para retroalimentación.
/// - Score < `RETRAIN_THRESHOLD` → descartar + ajustar prompt.
pub const ACCEPT_THRESHOLD: f64 = 0.8;
pub const RETRAIN_THRESHOLD: f64 = 0.5;

/// Resultado de una evaluación del Juez.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JudgeVerdict {
    pub faithfulness: f64,
    pub completeness: f64,
    pub format_compliance: f64,
    pub score: f64,
    pub reason: String,
}

impl JudgeVerdict {
    /// Decisión según umbrales.
    pub fn decision(&self) -> JudgeDecision {
        if self.score >= ACCEPT_THRESHOLD {
            JudgeDecision::Accept
        } else if self.score >= RETRAIN_THRESHOLD {
            JudgeDecision::Retrain
        } else {
            JudgeDecision::Reject
        }
    }
}

/// Decisión del Juez.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeDecision {
    /// Guardar en SQLite + vector DB.
    Accept,
    /// Guardar + registrar para retroalimentación.
    Retrain,
    /// Descartar + ajustar prompt.
    Reject,
}

/// Prompt de sistema del Juez.
const JUDGE_SYSTEM_PROMPT: &str = r#"Eres un evaluador riguroso de extracciones de datos. Dado un HTML original y la extracción JSON producida, evalúa y devuelve ÚNICAMENTE un JSON con:
{"faithfulness": 0.0-1.0, "completeness": 0.0-1.0, "format_compliance": 0.0-1.0, "reason": "breve justificación"}
- faithfulness: qué proporción de los datos extraídos está realmente presente en el HTML.
- completeness: qué proporción de los campos esperados fue capturada.
- format_compliance: si la extracción es JSON válido y coherente con el esquema.
No añadas texto fuera del JSON."#;

/// El Juez: evalúa extracciones usando un LLM grande gratuito.
pub struct Judge {
    pub cloud: CloudAdapter,
    pub provider: &'static str,
}

impl Judge {
    pub fn new(cloud: CloudAdapter, provider: &'static str) -> Self {
        Self { cloud, provider }
    }

    /// Evalúa una extracción contra el HTML original.
    pub async fn evaluate(&self, html: &str, extraction: &Value) -> Result<JudgeVerdict> {
        let prompt = format!(
            "HTML ORIGINAL:\n{}\n\nEXTRACCIÓN JSON A EVALUAR:\n{}\n\n\
             Evalúa y devuelve el JSON de veredicto.",
            truncate(html, 8000),
            serde_json::to_string_pretty(extraction)?
        );

        // El Juez pide respuesta JSON pura.
        let (value, provider) = self.cloud.reason_json(&format!("{JUDGE_SYSTEM_PROMPT}\n\n{prompt}")).await?;

        // Parsear veredicto (con tolerancia a campos faltantes).
        let faithfulness = value.get("faithfulness").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let completeness = value.get("completeness").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let format_compliance = value.get("format_compliance").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let reason = value
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let score = (faithfulness + completeness + format_compliance) / 3.0;

        tracing::info!(
            "⚖️ [JUEZ] score={score:.2} (faith={faithfulness:.2}, comp={completeness:.2}, fmt={format_compliance:.2}) via {provider}"
        );

        Ok(JudgeVerdict {
            faithfulness,
            completeness,
            format_compliance,
            score,
            reason,
        })
    }
}

/// Trunca un texto a un máximo de caracteres conservando el inicio.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraping::pipeline::cloud_adapter::{CloudAdapter, MockProvider};

    #[test]
    fn decision_segun_umbrales() {
        let v = JudgeVerdict {
            faithfulness: 0.9,
            completeness: 0.9,
            format_compliance: 0.9,
            score: 0.9,
            reason: "ok".into(),
        };
        assert_eq!(v.decision(), JudgeDecision::Accept);

        let v = JudgeVerdict {
            faithfulness: 0.6,
            completeness: 0.6,
            format_compliance: 0.6,
            score: 0.6,
            reason: "medio".into(),
        };
        assert_eq!(v.decision(), JudgeDecision::Retrain);

        let v = JudgeVerdict {
            faithfulness: 0.3,
            completeness: 0.3,
            format_compliance: 0.3,
            score: 0.3,
            reason: "bajo".into(),
        };
        assert_eq!(v.decision(), JudgeDecision::Reject);
    }

    #[tokio::test]
    async fn juez_evalua_con_mock() {
        let cloud = CloudAdapter::new(vec![Box::new(MockProvider {
            name: "mock",
            should_fail: false,
            response: r#"{"faithfulness": 0.9, "completeness": 0.8, "format_compliance": 1.0, "reason": "correcto"}"#.into(),
        })]);
        let judge = Judge::new(cloud, "mock");
        let verdict = judge
            .evaluate("<html><body>producto</body></html>", &serde_json::json!({"item": "producto"}))
            .await
            .unwrap();
        assert!(verdict.score > 0.8);
        assert_eq!(verdict.decision(), JudgeDecision::Accept);
    }

    #[test]
    fn trunca_texto_largo() {
        let s = "a".repeat(10000);
        assert_eq!(truncate(&s, 100).len(), 100);
    }
}
