// 🔱 ccxt_rs — Error handling soberano
// Sin unwrap(), sin expect(). Jerarquía de errores tipada con thiserror.
// Cero dependencias externas — usamos std::error::Error + Display

use std::fmt;

/// Error unificado del arsenal de trading
#[derive(Debug)]
pub enum ExchangeError {
    /// Error de conexión HTTP/WebSocket
    Network {
        exchange: String,
        endpoint: String,
        source: String,
    },
    /// Error de autenticación (API key inválida, expirada, sin permisos)
    Authentication { exchange: String, reason: String },
    /// Error de rate limiting (429 Too Many Requests)
    RateLimit {
        exchange: String,
        retry_after_ms: u64,
    },
    /// Error de validación de parámetros (símbolo inválido, cantidad fuera de rango)
    BadRequest { exchange: String, reason: String },
    /// Error de parsing de respuesta (JSON malformado, campo faltante)
    Parse {
        exchange: String,
        raw: String,
        source: String,
    },
    /// Error de timeout
    Timeout {
        exchange: String,
        endpoint: String,
        duration_ms: u64,
    },
    /// Error de WebSocket
    WebSocket { exchange: String, reason: String },
    /// Error interno de NEXUS
    Internal { reason: String },
    /// Error del exchange (código HTTP 4xx/5xx)
    Exchange {
        exchange: String,
        code: u16,
        body: String,
    },
}

impl fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network {
                exchange,
                endpoint,
                source,
            } => {
                write!(f, "[{exchange}] Network error en {endpoint}: {source}")
            }
            Self::Authentication { exchange, reason } => {
                write!(f, "[{exchange}] Authentication error: {reason}")
            }
            Self::RateLimit {
                exchange,
                retry_after_ms,
            } => {
                write!(
                    f,
                    "[{exchange}] Rate limited. Retry after {retry_after_ms}ms"
                )
            }
            Self::BadRequest { exchange, reason } => {
                write!(f, "[{exchange}] Bad request: {reason}")
            }
            Self::Parse {
                exchange,
                raw,
                source,
            } => {
                write!(f, "[{exchange}] Parse error: {source}. Raw: {raw:.120}")
            }
            Self::Timeout {
                exchange,
                endpoint,
                duration_ms,
            } => {
                write!(
                    f,
                    "[{exchange}] Timeout after {duration_ms}ms on {endpoint}"
                )
            }
            Self::WebSocket { exchange, reason } => {
                write!(f, "[{exchange}] WebSocket error: {reason}")
            }
            Self::Internal { reason } => {
                write!(f, "Internal NEXUS error: {reason}")
            }
            Self::Exchange {
                exchange,
                code,
                body,
            } => {
                write!(f, "[{exchange}] HTTP {code}: {body:.200}")
            }
        }
    }
}

impl std::error::Error for ExchangeError {}

impl From<std::io::Error> for ExchangeError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal {
            reason: format!("IO error: {}", e),
        }
    }
}

/// Resultado especializado para operaciones de exchange
pub type ExchangeResult<T> = Result<T, ExchangeError>;
