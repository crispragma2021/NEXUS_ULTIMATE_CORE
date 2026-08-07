//! Circuit breaker por proveedor (F8.2).
//!
//! Estado por proveedor en memoria (`HashMap<String, CircuitState>`):
//! - **3 fallos consecutivos** → proveedor en `Open` (desactivado temporalmente).
//! - Pausa de **5 minutos** (configurable).
//! - Tras la pausa, pasa a `HalfOpen`: una prueba decide si se reactiva.
//!
//! Un fallo puede ser: error HTTP 5xx, 429 (rate limit agotado), o timeout.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Estado de un circuito.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Funcionando normalmente.
    Closed,
    /// Abierto: fallos ≥ umbral → rechazar llamadas.
    Open,
    /// Semi-abierto: tras la pausa, permite 1 llamada de prueba.
    HalfOpen,
}

/// Estado interno de un circuito individual.
#[derive(Debug, Clone)]
struct CircuitEntry {
    state: CircuitState,
    consecutive_failures: u32,
    opened_at: Option<Instant>,
    /// Pausa aplicable al abrirse (ms).
    pause_ms: u64,
}

impl CircuitEntry {
    fn new(pause_ms: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            opened_at: None,
            pause_ms,
        }
    }
}

/// Gestor de circuit breakers por proveedor.
pub struct ProviderCircuitBreaker {
    failure_threshold: u32,
    pause_ms: u64,
    circuits: Mutex<HashMap<String, CircuitEntry>>,
}

impl ProviderCircuitBreaker {
    pub fn new(failure_threshold: u32, pause_ms: u64) -> Self {
        Self {
            failure_threshold,
            pause_ms,
            circuits: Mutex::new(HashMap::new()),
        }
    }

    /// Circuit breaker por defecto: 3 fallos → 5 min de pausa.
    pub fn default() -> Self {
        Self::new(3, 300_000)
    }

    /// Indica si se puede llamar al proveedor (no está en Open).
    pub fn is_allowed(&self, provider: &str) -> bool {
        let mut circuits = self.circuits.lock().unwrap();
        let entry = circuits.entry(provider.to_string()).or_insert_with(|| CircuitEntry::new(self.pause_ms));

        match entry.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true, // 1 prueba permitida
            CircuitState::Open => {
                // Si ya pasó la pausa, pasar a HalfOpen (permitir prueba).
                if let Some(opened_at) = entry.opened_at {
                    if opened_at.elapsed() >= Duration::from_millis(entry.pause_ms) {
                        entry.state = CircuitState::HalfOpen;
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Registra un éxito → resetea el circuito.
    pub fn record_success(&self, provider: &str) {
        let mut circuits = self.circuits.lock().unwrap();
        let entry = circuits.entry(provider.to_string()).or_insert_with(|| CircuitEntry::new(self.pause_ms));
        entry.state = CircuitState::Closed;
        entry.consecutive_failures = 0;
        entry.opened_at = None;
    }

    /// Registra un fallo → incrementa; si llega al umbral, abre el circuito.
    pub fn record_failure(&self, provider: &str) {
        let mut circuits = self.circuits.lock().unwrap();
        let entry = circuits.entry(provider.to_string()).or_insert_with(|| CircuitEntry::new(self.pause_ms));

        // Si estaba HalfOpen y falló la prueba → volver a abrir.
        if entry.state == CircuitState::HalfOpen {
            entry.state = CircuitState::Open;
            entry.opened_at = Some(Instant::now());
            entry.consecutive_failures = self.failure_threshold;
            return;
        }

        entry.consecutive_failures += 1;
        if entry.consecutive_failures >= self.failure_threshold {
            entry.state = CircuitState::Open;
            entry.opened_at = Some(Instant::now());
        }
    }

    /// Devuelve el estado actual (para métricas).
    pub fn state_of(&self, provider: &str) -> CircuitState {
        let circuits = self.circuits.lock().unwrap();
        circuits
            .get(provider)
            .map(|c| c.state)
            .unwrap_or(CircuitState::Closed)
    }
}

impl Default for ProviderCircuitBreaker {
    fn default() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cierra_tras_3_fallos() {
        let cb = ProviderCircuitBreaker::new(3, 300_000);
        assert!(cb.is_allowed("openrouter"));
        cb.record_failure("openrouter");
        cb.record_failure("openrouter");
        assert!(cb.is_allowed("openrouter"));
        cb.record_failure("openrouter");
        // Abierto → rechazado.
        assert!(!cb.is_allowed("openrouter"));
        assert_eq!(cb.state_of("openrouter"), CircuitState::Open);
    }

    #[test]
    fn exito_reactiva_circuito() {
        let cb = ProviderCircuitBreaker::new(3, 300_000);
        cb.record_failure("gemini");
        cb.record_failure("gemini");
        cb.record_failure("gemini");
        assert!(!cb.is_allowed("gemini"));
        // Forzar pausa corta y probar reactivación vía record_success.
        cb.record_success("gemini");
        assert_eq!(cb.state_of("gemini"), CircuitState::Closed);
        assert!(cb.is_allowed("gemini"));
    }

    #[test]
    fn half_open_permite_una_prueba() {
        // Pausa real: primero bloquea, luego de la pausa permite una prueba.
        let cb = ProviderCircuitBreaker::new(1, 50);
        cb.record_failure("deepseek");
        assert!(!cb.is_allowed("deepseek")); // Open: pausa no ha terminado
        assert_eq!(cb.state_of("deepseek"), CircuitState::Open);
        // Esperar a que termine la pausa (50ms).
        std::thread::sleep(Duration::from_millis(60));
        assert!(cb.is_allowed("deepseek")); // HalfOpen → prueba permitida
        assert_eq!(cb.state_of("deepseek"), CircuitState::HalfOpen);
        // Si la prueba falla, vuelve a abrir.
        cb.record_failure("deepseek");
        assert!(!cb.is_allowed("deepseek"));
        assert_eq!(cb.state_of("deepseek"), CircuitState::Open);
    }

    #[test]
    fn estados_por_proveedor_son_independientes() {
        let cb = ProviderCircuitBreaker::new(1, 300_000);
        cb.record_failure("a");
        assert!(!cb.is_allowed("a"));
        assert!(cb.is_allowed("b"));
    }
}
