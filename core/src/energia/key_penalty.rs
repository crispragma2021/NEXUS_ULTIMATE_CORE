// ==========================================
// KEY PENALTY SYSTEM - Penalización Inteligente de Llaves
// ==========================================
// Migrado desde la lógica de clasificación de errores del ReliableProvider
// (legacy/nexusclaw/src/providers/reliable.rs)
//
// Cada vez que una célula (cuenta + llave) falla con 429, se le aplica
// una penalización temporal. Durante ese tiempo, el sistema la salta
// automáticamente, evitando perder tiempo en llaves agotadas.
// ==========================================

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Estado de una célula energética (combinación cuenta + llave)
#[derive(Debug, Clone)]
struct CellState {
    /// Cuenta a la que pertenece (índice en ZENITH_ACCOUNTS)
    account_idx: usize,
    /// Índice de la llave dentro de la cuenta
    key_idx: usize,
    /// Timestamp de cuando se levantará la penalización
    penalized_until: Option<Instant>,
    /// Contador de fallos consecutivos
    consecutive_failures: u32,
    /// Último código de error (429, 403, 500, etc.)
    last_error_code: Option<u16>,
    /// Cuántas veces ha sido penalizada esta célula
    penalty_count: u32,
}

/// Clasificación de errores (migrado de ReliableProvider)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClassification {
    /// 429 rate limit - recuperable, cambiar de llave
    RateLimited,
    /// 429 pero por plan/balance - NO recuperable con retry
    NonRetryableRateLimit,
    /// Error del cliente (auth, bad request, etc) - fatal
    NonRetryable,
    /// Error del servidor / red - recuperable con backoff
    Retryable,
}

impl ErrorClassification {
    /// Determina si el error puede resolverse cambiando de llave
    pub fn requires_key_rotation(&self) -> bool {
        matches!(self, ErrorClassification::RateLimited)
    }

    /// Determina si el error es fatal para este proveedor
    pub fn is_fatal_for_provider(&self) -> bool {
        matches!(
            self,
            ErrorClassification::NonRetryable | ErrorClassification::NonRetryableRateLimit
        )
    }
}

/// Gestor central de penalización de llaves
pub struct KeyPenaltySystem {
    cells: Mutex<HashMap<(usize, usize), CellState>>,
    /// Duración base de penalización (se multiplica por fallos consecutivos)
    base_penalty_duration: Duration,
    /// Penalización máxima (cap)
    max_penalty_duration: Duration,
    /// Penalización por defecto para 429
    rate_limit_penalty: Duration,
}

impl Default for KeyPenaltySystem {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyPenaltySystem {
    pub fn new() -> Self {
        info!("⚔️ [KEY_PENALTY] Sistema de penalización de llaves activado");
        Self {
            cells: Mutex::new(HashMap::new()),
            base_penalty_duration: Duration::from_secs(30),
            max_penalty_duration: Duration::from_secs(300), // 5 minutos máximo
            rate_limit_penalty: Duration::from_secs(60),    // 1 minuto por defecto para 429
        }
    }

    // ─── CLASIFICACIÓN DE ERRORES ─────────────────────────────────────────
    // Migrado de ReliableProvider (legacy/nexusclaw/src/providers/reliable.rs)
    // Original: fn is_non_retryable(), is_rate_limited(), is_non_retryable_rate_limit()

    /// Clasifica un código de estado HTTP y cuerpo de error
    pub fn classify_error(status: u16, body: &str) -> ErrorClassification {
        match status {
            429 => {
                if Self::is_non_retryable_rate_limit(body) {
                    ErrorClassification::NonRetryableRateLimit
                } else {
                    ErrorClassification::RateLimited
                }
            }
            408 | 502 | 503 | 504 => ErrorClassification::Retryable,
            400..=499 => ErrorClassification::NonRetryable,
            500..=599 => ErrorClassification::Retryable,
            _ => ErrorClassification::Retryable,
        }
    }

    /// Determina si un 429 es por plan/balance (no recuperable)
    fn is_non_retryable_rate_limit(body: &str) -> bool {
        let lower = body.to_lowercase();
        let business_hints = [
            "plan does not include",
            "doesn't include",
            "not include",
            "insufficient balance",
            "insufficient_balance",
            "insufficient quota",
            "insufficient_quota",
            "quota exhausted",
            "out of credits",
            "no available package",
            "package not active",
            "purchase package",
            "model not available for your plan",
            "billing",
        ];
        business_hints.iter().any(|hint| lower.contains(hint))
    }

    /// Verifica si un error es de autenticación (non-retryable)
    pub fn is_auth_error(body: &str) -> bool {
        let lower = body.to_lowercase();
        let auth_hints = [
            "invalid api key",
            "incorrect api key",
            "missing api key",
            "api key not set",
            "authentication failed",
            "auth failed",
            "unauthorized",
            "forbidden",
            "permission denied",
            "access denied",
            "invalid token",
            "api key not valid",
            "key not found",
        ];
        auth_hints.iter().any(|hint| lower.contains(hint))
    }

    // ─── GESTIÓN DE PENALIZACIÓN ──────────────────────────────────────────

    /// Registra un fallo en una célula y aplica penalización
    pub fn register_failure(&self, account_idx: usize, key_idx: usize, status: u16, body: &str) {
        let classification = Self::classify_error(status, body);
        let penalty_duration = match classification {
            ErrorClassification::RateLimited => self.rate_limit_penalty,
            ErrorClassification::NonRetryableRateLimit => {
                // Penalización más larga para errores de plan/balance
                Duration::from_secs(600) // 10 minutos
            }
            ErrorClassification::NonRetryable => {
                // Errores de auth - marcar como muerta permanentemente
                Duration::from_secs(86400) // 24 horas (hasta que el usuario la reactive)
            }
            ErrorClassification::Retryable => {
                Duration::from_secs(15) // 15 segundos para errores de servidor
            }
        };

        let mut cells = self.cells.lock().unwrap();
        let key = (account_idx, key_idx);
        let state = cells.entry(key).or_insert(CellState {
            account_idx,
            key_idx,
            penalized_until: None,
            consecutive_failures: 0,
            last_error_code: None,
            penalty_count: 0,
        });

        state.consecutive_failures += 1;
        state.last_error_code = Some(status);
        state.penalty_count += 1;

        // Penalización progresiva: multiplicar por fallos consecutivos
        let actual_duration = penalty_duration
            .mul_f64(state.consecutive_failures.min(10) as f64)
            .min(self.max_penalty_duration);

        state.penalized_until = Some(Instant::now() + actual_duration);

        warn!(
            "⚔️ [KEY_PENALTY] Célula [{}/{}] penalizada por {:.0}s (429×{})",
            account_idx + 1,
            key_idx + 1,
            actual_duration.as_secs_f64(),
            state.consecutive_failures,
        );
    }

    /// Registra una autenticación fallida (403, 401) - marca como muerta
    pub fn register_auth_failure(&self, account_idx: usize, key_idx: usize) {
        let mut cells = self.cells.lock().unwrap();
        let key = (account_idx, key_idx);
        let state = cells.entry(key).or_insert(CellState {
            account_idx,
            key_idx,
            penalized_until: None,
            consecutive_failures: 0,
            last_error_code: None,
            penalty_count: 0,
        });

        state.consecutive_failures = 99; // Marcar como prácticamente muerta
        state.last_error_code = Some(403);
        state.penalty_count += 1;
        state.penalized_until = Some(Instant::now() + Duration::from_secs(86400));

        warn!(
            "⚔️ [KEY_PENALTY] ⛔ Célula [{}/{}] MARCADA COMO MUERTA (auth failure)",
            account_idx + 1,
            key_idx + 1,
        );
    }

    /// Registra un éxito en una célula (resetea contador de fallos)
    pub fn register_success(&self, account_idx: usize, key_idx: usize) {
        let mut cells = self.cells.lock().unwrap();
        let key = (account_idx, key_idx);
        if let Some(state) = cells.get_mut(&key) {
            if state.consecutive_failures > 0 {
                debug!(
                    "⚔️ [KEY_PENALTY] Célula [{}/{}] exitosa - reseteando fallos",
                    account_idx + 1,
                    key_idx + 1,
                );
            }
            state.consecutive_failures = 0;
            state.penalized_until = None;
            state.last_error_code = None;
        }
    }

    /// Verifica si una célula está actualmente penalizada
    pub fn is_penalized(&self, account_idx: usize, key_idx: usize) -> bool {
        let cells = self.cells.lock().unwrap();
        if let Some(state) = cells.get(&(account_idx, key_idx)) {
            if let Some(until) = state.penalized_until {
                if Instant::now() < until {
                    return true;
                }
            }
        }
        false
    }

    /// Obtiene el tiempo restante de penalización para una célula (en segundos)
    pub fn remaining_penalty(&self, account_idx: usize, key_idx: usize) -> f64 {
        let cells = self.cells.lock().unwrap();
        if let Some(state) = cells.get(&(account_idx, key_idx)) {
            if let Some(until) = state.penalized_until {
                let remaining = until.saturating_duration_since(Instant::now());
                return remaining.as_secs_f64();
            }
        }
        0.0
    }

    /// Limpia células cuya penalización ya expiró
    pub fn sweep_expired(&self) -> usize {
        let mut cells = self.cells.lock().unwrap();
        let now = Instant::now();
        let before = cells.len();
        cells.retain(|_, state| {
            if let Some(until) = state.penalized_until {
                now < until
            } else {
                false // Sin penalización activa, la limpiamos
            }
        });
        let swept = before - cells.len();
        if swept > 0 {
            debug!(
                "⚔️ [KEY_PENALTY] Limpieza: {} células expiradas eliminadas",
                swept
            );
        }
        swept
    }

    /// Reporte de estado del sistema de penalización
    pub fn report(&self) -> String {
        let cells = self.cells.lock().unwrap();
        let active: Vec<_> = cells
            .values()
            .filter(|s| {
                s.penalized_until
                    .map(|u| Instant::now() < u)
                    .unwrap_or(false)
            })
            .collect();

        if active.is_empty() {
            return "⚔️ [KEY_PENALTY] 0 células penalizadas activamente.".to_string();
        }

        let mut report = format!("⚔️ [KEY_PENALTY] {} células penalizadas:\n", active.len());
        for state in &active {
            let remaining = state
                .penalized_until
                .map(|u| u.saturating_duration_since(Instant::now()).as_secs_f64())
                .unwrap_or(0.0);
            report.push_str(&format!(
                "  ⛔ [{}/{}] error={:?} restante={:.0}s fallos={}\n",
                state.account_idx + 1,
                state.key_idx + 1,
                state.last_error_code,
                remaining,
                state.consecutive_failures,
            ));
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_classify_429_rate_limit() {
        let classification = KeyPenaltySystem::classify_error(429, "Rate limit exceeded");
        assert_eq!(classification, ErrorClassification::RateLimited);
    }

    #[test]
    fn test_classify_429_non_retryable() {
        let classification = KeyPenaltySystem::classify_error(429, "quota exhausted");
        assert_eq!(classification, ErrorClassification::NonRetryableRateLimit);
    }

    #[test]
    fn test_classify_403() {
        let classification = KeyPenaltySystem::classify_error(403, "API key not valid");
        assert_eq!(classification, ErrorClassification::NonRetryable);
    }

    #[test]
    fn test_classify_503() {
        let classification = KeyPenaltySystem::classify_error(503, "Service unavailable");
        assert_eq!(classification, ErrorClassification::Retryable);
    }

    #[test]
    fn test_penalty_and_expiry() {
        let system = KeyPenaltySystem::new();
        system.register_failure(0, 0, 429, "rate limit");
        assert!(system.is_penalized(0, 0));
        assert!(system.remaining_penalty(0, 0) > 0.0);
    }

    #[test]
    fn test_success_resets_penalty() {
        let system = KeyPenaltySystem::new();
        system.register_failure(0, 0, 429, "rate limit");
        assert!(system.is_penalized(0, 0));
        system.register_success(0, 0);
        assert!(!system.is_penalized(0, 0));
    }

    #[test]
    fn test_auth_failure_permanent() {
        let system = KeyPenaltySystem::new();
        system.register_auth_failure(0, 0);
        assert!(system.is_penalized(0, 0));
        assert!(system.remaining_penalty(0, 0) > 80000.0); // ~24h en segundos
    }

    #[test]
    fn test_is_auth_error() {
        assert!(KeyPenaltySystem::is_auth_error("API key not valid"));
        assert!(KeyPenaltySystem::is_auth_error("invalid api key"));
        assert!(!KeyPenaltySystem::is_auth_error("rate limit exceeded"));
    }
}
