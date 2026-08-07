//! Métricas del pipeline (F7.2).
//!
//! Contadores e histogramas en memoria (atomics + mutex):
//! - `tasks_total` — tareas procesadas (por estado).
//! - `tier_usage_ratio` — proporción tier1 vs tier2.
//! - `errors_total` — errores acumulados (por categoría).
//! - `tokens_processed` — total de tokens procesados (por tier).
//! - `provider_circuit_opens` — aperturas de circuit breaker por proveedor.
//!
//! Se expone un snapshot JSON para logs y (opcionalmente) un endpoint /metrics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Contadores atómicos del pipeline.
pub struct Metrics {
    // ── Contadores (atomics) ─────────────────────────────────
    tasks_success: AtomicU64,
    tasks_partial: AtomicU64,
    tasks_failed: AtomicU64,
    tasks_blocked: AtomicU64,
    tasks_timeout: AtomicU64,
    tasks_provider_exhausted: AtomicU64,
    tier1_calls: AtomicU64,
    tier2_calls: AtomicU64,
    errors_total: AtomicU64,
    tokens_tier1: AtomicU64,
    tokens_tier2: AtomicU64,
    // ── Histogramas / agregados (Mutex) ─────────────────────
    /// Latencia de inferencia por tier (ms), acumulada.
    inference_latency_ms: Mutex<Vec<u64>>,
    /// Aperturas de circuit breaker por proveedor.
    circuit_opens: Mutex<HashMap<String, u64>>,
    /// Errores por categoría.
    errors_by_category: Mutex<HashMap<String, u64>>,
    // ── Inicio de proceso ────────────────────────────────────
    start: Instant,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            tasks_success: AtomicU64::new(0),
            tasks_partial: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            tasks_blocked: AtomicU64::new(0),
            tasks_timeout: AtomicU64::new(0),
            tasks_provider_exhausted: AtomicU64::new(0),
            tier1_calls: AtomicU64::new(0),
            tier2_calls: AtomicU64::new(0),
            errors_total: AtomicU64::new(0),
            tokens_tier1: AtomicU64::new(0),
            tokens_tier2: AtomicU64::new(0),
            inference_latency_ms: Mutex::new(Vec::new()),
            circuit_opens: Mutex::new(HashMap::new()),
            errors_by_category: Mutex::new(HashMap::new()),
            start: Instant::now(),
        }
    }

    // ── Registro de tareas ───────────────────────────────────

    pub fn record_task_success(&self) {
        self.tasks_success.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_task_partial(&self) {
        self.tasks_partial.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_task_failed(&self) {
        self.tasks_failed.fetch_add(1, Ordering::Relaxed);
        self.errors_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_task_blocked(&self) {
        self.tasks_blocked.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_task_timeout(&self) {
        self.tasks_timeout.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_task_provider_exhausted(&self) {
        self.tasks_provider_exhausted.fetch_add(1, Ordering::Relaxed);
    }

    /// Registra el estado de una tarea genérico por string.
    pub fn record_task_status(&self, status: &str) {
        match status {
            "success" => self.record_task_success(),
            "partial" => self.record_task_partial(),
            "failed" => self.record_task_failed(),
            "blocked_by_robots" => self.record_task_blocked(),
            "timeout" => self.record_task_timeout(),
            "provider_exhausted" => self.record_task_provider_exhausted(),
            _ => {}
        }
    }

    // ── Tier y tokens ────────────────────────────────────────

    pub fn record_tier1_call(&self, tokens: u64, latency_ms: u64) {
        self.tier1_calls.fetch_add(1, Ordering::Relaxed);
        self.tokens_tier1.fetch_add(tokens, Ordering::Relaxed);
        self.inference_latency_ms.lock().unwrap().push(latency_ms);
    }
    pub fn record_tier2_call(&self, tokens: u64, latency_ms: u64) {
        self.tier2_calls.fetch_add(1, Ordering::Relaxed);
        self.tokens_tier2.fetch_add(tokens, Ordering::Relaxed);
        self.inference_latency_ms.lock().unwrap().push(latency_ms);
    }

    // ── Errores y circuit breaker ────────────────────────────

    pub fn record_error(&self, category: &str) {
        self.errors_total.fetch_add(1, Ordering::Relaxed);
        let mut map = self.errors_by_category.lock().unwrap();
        *map.entry(category.to_string()).or_insert(0) += 1;
    }

    pub fn record_circuit_open(&self, provider: &str) {
        let mut map = self.circuit_opens.lock().unwrap();
        *map.entry(provider.to_string()).or_insert(0) += 1;
    }

    // ── Lecturas ─────────────────────────────────────────────

    pub fn tasks_total(&self) -> u64 {
        self.tasks_success.load(Ordering::Relaxed)
            + self.tasks_partial.load(Ordering::Relaxed)
            + self.tasks_failed.load(Ordering::Relaxed)
            + self.tasks_blocked.load(Ordering::Relaxed)
            + self.tasks_timeout.load(Ordering::Relaxed)
            + self.tasks_provider_exhausted.load(Ordering::Relaxed)
    }

    /// Proporción tier2 / total de llamadas de inferencia (0..1).
    pub fn tier_usage_ratio(&self) -> f64 {
        let t1 = self.tier1_calls.load(Ordering::Relaxed);
        let t2 = self.tier2_calls.load(Ordering::Relaxed);
        let total = t1 + t2;
        if total == 0 {
            0.0
        } else {
            t2 as f64 / total as f64
        }
    }

    pub fn tokens_processed_total(&self) -> u64 {
        self.tokens_tier1.load(Ordering::Relaxed) + self.tokens_tier2.load(Ordering::Relaxed)
    }

    /// Latencia media de inferencia (ms).
    pub fn avg_inference_latency_ms(&self) -> f64 {
        let lat = self.inference_latency_ms.lock().unwrap();
        if lat.is_empty() {
            0.0
        } else {
            lat.iter().sum::<u64>() as f64 / lat.len() as f64
        }
    }

    /// Uptime del proceso en segundos.
    pub fn uptime_secs(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    /// Snapshot JSON completo para logs / endpoint /metrics.
    pub fn snapshot_json(&self) -> serde_json::Value {
        let circuit: HashMap<String, u64> = self.circuit_opens.lock().unwrap().clone();
        let errors: HashMap<String, u64> = self.errors_by_category.lock().unwrap().clone();
        serde_json::json!({
            "uptime_secs": self.uptime_secs(),
            "tasks_total": self.tasks_total(),
            "tasks_success": self.tasks_success.load(Ordering::Relaxed),
            "tasks_partial": self.tasks_partial.load(Ordering::Relaxed),
            "tasks_failed": self.tasks_failed.load(Ordering::Relaxed),
            "tasks_blocked_by_robots": self.tasks_blocked.load(Ordering::Relaxed),
            "tasks_timeout": self.tasks_timeout.load(Ordering::Relaxed),
            "tasks_provider_exhausted": self.tasks_provider_exhausted.load(Ordering::Relaxed),
            "tier_usage_ratio": self.tier_usage_ratio(),
            "tier1_calls": self.tier1_calls.load(Ordering::Relaxed),
            "tier2_calls": self.tier2_calls.load(Ordering::Relaxed),
            "tokens_processed_total": self.tokens_processed_total(),
            "tokens_tier1": self.tokens_tier1.load(Ordering::Relaxed),
            "tokens_tier2": self.tokens_tier2.load(Ordering::Relaxed),
            "avg_inference_latency_ms": self.avg_inference_latency_ms(),
            "errors_total": self.errors_total.load(Ordering::Relaxed),
            "errors_by_category": errors,
            "provider_circuit_opens": circuit,
        })
    }

    /// Renderiza el snapshot en formato Prometheus-ish (texto plano).
    pub fn render_prometheus(&self) -> String {
        let s = self.snapshot_json();
        let mut out = String::new();
        let mut append = |key: &str, val: &serde_json::Value| {
            out.push_str(&format!("{key} {val}\n"));
        };
        if let Some(obj) = s.as_object() {
            for (k, v) in obj {
                if v.is_number() || v.is_boolean() {
                    append(k, v);
                }
            }
        }
        out
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registra_tareas_y_errores() {
        let m = Metrics::new();
        m.record_task_success();
        m.record_task_success();
        m.record_task_failed();
        assert_eq!(m.tasks_total(), 3);
        assert_eq!(m.errors_total.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn calcula_ratio_de_tiers() {
        let m = Metrics::new();
        m.record_tier1_call(100, 50);
        m.record_tier2_call(300, 100);
        m.record_tier2_call(300, 100);
        // 2 de 3 llamadas en tier2 → ratio 0.66
        assert!((m.tier_usage_ratio() - 0.666).abs() < 0.01);
        assert_eq!(m.tokens_processed_total(), 700);
    }

    #[test]
    fn snapshot_json_incluye_todas_las_claves() {
        let m = Metrics::new();
        m.record_circuit_open("openrouter");
        let snap = m.snapshot_json();
        assert!(snap["provider_circuit_opens"]["openrouter"].is_number());
        assert!(snap.get("tasks_total").is_some());
        assert!(snap.get("tier_usage_ratio").is_some());
    }

    #[test]
    fn latencia_media_correcta() {
        let m = Metrics::new();
        m.record_tier2_call(1, 100);
        m.record_tier2_call(1, 300);
        assert_eq!(m.avg_inference_latency_ms(), 200.0);
    }

    #[test]
    fn render_prometheus_plano() {
        let m = Metrics::new();
        let out = m.render_prometheus();
        assert!(out.contains("tasks_total"));
        assert!(out.contains("uptime_secs"));
    }
}
