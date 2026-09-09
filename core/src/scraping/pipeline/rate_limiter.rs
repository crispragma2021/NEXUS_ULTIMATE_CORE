//! Rate limiter por dominio (F5.3).
//!
//! Controla el ritmo de peticiones al mismo dominio usando la tabla
//! `rate_limit_state` de SQLite (spec §6.2):
//! - Retraso mínimo entre peticiones al mismo dominio: 2,000 ms (configurable).
//! - Máximo de peticiones concurrentes globales: 3.
//! - Backoff exponencial si el dominio devuelve 429/403.

use crate::scraping::pipeline::db::PipelineDb;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Retraso mínimo por defecto entre peticiones al mismo dominio (ms).
pub const DEFAULT_MIN_DELAY_MS: u64 = 2000;

/// Máximo de peticiones concurrentes globales.
pub const DEFAULT_MAX_CONCURRENT: usize = 3;

/// Rate limiter por dominio.
pub struct RateLimiter {
    db: Option<Arc<PipelineDb>>,
    min_delay: Duration,
    /// Conteo global de peticiones en vuelo.
    in_flight: std::sync::atomic::AtomicUsize,
    max_concurrent: usize,
    /// Backoff por dominio (fallos consecutivos → retraso extra).
    backoff_ms: std::sync::Mutex<std::collections::HashMap<String, u64>>,
}

impl RateLimiter {
    pub fn new(db: Option<Arc<PipelineDb>>) -> Self {
        Self {
            db,
            min_delay: Duration::from_millis(DEFAULT_MIN_DELAY_MS),
            in_flight: std::sync::atomic::AtomicUsize::new(0),
            max_concurrent: DEFAULT_MAX_CONCURRENT,
            backoff_ms: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn with_min_delay(mut self, ms: u64) -> Self {
        self.min_delay = Duration::from_millis(ms);
        self
    }

    /// Espera hasta que haya un hueco de concurrencia global y respete el
    /// delay por dominio. Se llama ANTES de hacer una petición.
    pub async fn acquire(&self, domain: &str) {
        // 1. Límite global de concurrencia.
        loop {
            let current = self.in_flight.load(std::sync::atomic::Ordering::SeqCst);
            if current < self.max_concurrent {
                if self
                    .in_flight
                    .compare_exchange(
                        current,
                        current + 1,
                        std::sync::atomic::Ordering::SeqCst,
                        std::sync::atomic::Ordering::SeqCst,
                    )
                    .is_ok()
                {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // 2. Delay mínimo por dominio (leído de DB si está disponible).
        let last = self.last_request_epoch_ms(domain).await;
        let backoff = self
            .backoff_ms
            .lock()
            .unwrap()
            .get(domain)
            .copied()
            .unwrap_or(0);
        let now = unix_ms();
        let elapsed = now.saturating_sub(last);
        let needed = self.min_delay.as_millis() as u64 + backoff;
        if elapsed < needed {
            tokio::time::sleep(Duration::from_millis(needed - elapsed)).await;
        }
    }

    /// Libera el hueco de concurrencia tras la petición.
    pub fn release(&self, domain: &str, success: bool) {
        self.in_flight
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        if success {
            self.backoff_ms.lock().unwrap().remove(domain);
        }
    }

    /// Registra un fallo (429/403) → incrementa backoff exponencial.
    pub fn note_failure(&self, domain: &str) {
        let mut map = self.backoff_ms.lock().unwrap();
        let current = map.get(domain).copied().unwrap_or(0);
        // Backoff exponencial: 2s, 4s, 8s, ... cap a 120s.
        let next = if current == 0 {
            2000
        } else {
            (current * 2).min(120_000)
        };
        map.insert(domain.to_string(), next);
    }

    /// Timestamp (ms epoch) de la última petición al dominio, desde SQLite.
    async fn last_request_epoch_ms(&self, domain: &str) -> u64 {
        if let Some(db) = &self.db {
            if let Ok((last, _failures)) = db.get_rate_limit(domain) {
                // Formato SQLite: "YYYY-MM-DD HH:MM:SS" (UTC).
                if let Ok(ts) = parse_sqlite_datetime_ms(&last) {
                    return ts;
                }
            }
        }
        // Si no hay DB o registro, permitir inmediato (0).
        0
    }
}

/// Convierte "YYYY-MM-DD HH:MM:SS" (UTC) a epoch millis.
fn parse_sqlite_datetime_ms(s: &str) -> Result<u64> {
    // Parse simple sin dependencias externas (formato fijo de SQLite).
    let mut parts = s.split(['-', ' ', ':']).filter(|p| !p.is_empty());
    let y: i64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("bad year"))?
        .parse()?;
    let mo: i64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("bad month"))?
        .parse()?;
    let d: i64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("bad day"))?
        .parse()?;
    let h: i64 = parts.next().unwrap_or("0").parse()?;
    let mi: i64 = parts.next().unwrap_or("0").parse()?;
    let se: i64 = parts.next().unwrap_or("0").parse()?;

    // Días desde epoch (aproximación civil; suficiente para rate limiting).
    let days = days_from_civil(y, mo, d);
    let secs = days * 86_400 + h * 3600 + mi * 60 + se;
    Ok((secs as u64) * 1000)
}

/// Días desde 1970-01-01 (algoritmo civil de Howard Hinnant).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_fecha_sqlite() {
        let ms = parse_sqlite_datetime_ms("1970-01-01 00:00:00").unwrap();
        assert_eq!(ms, 0);
        let ms = parse_sqlite_datetime_ms("1970-01-02 00:00:00").unwrap();
        assert_eq!(ms, 86_400_000);
    }

    #[test]
    fn days_from_civil_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1970, 1, 2), 1);
    }

    #[tokio::test]
    async fn backoff_exponencial_se_acumula() {
        let rl = RateLimiter::default();
        rl.note_failure("example.com");
        rl.note_failure("example.com");
        let map = rl.backoff_ms.lock().unwrap();
        assert_eq!(map.get("example.com").copied(), Some(4000));
    }

    #[tokio::test]
    async fn release_limpiar_backoff() {
        let rl = RateLimiter::default();
        rl.note_failure("example.com");
        rl.release("example.com", true);
        let map = rl.backoff_ms.lock().unwrap();
        assert!(map.get("example.com").is_none());
    }

    // Verificación rápida: min_delay es 2s por defecto.
    #[test]
    fn delay_por_defecto_es_2s() {
        let rl = RateLimiter::default();
        assert_eq!(rl.min_delay, Duration::from_millis(2000));
    }
}
