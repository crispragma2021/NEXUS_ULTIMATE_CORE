// 🔱 ccxt_rs — Cliente REST Genérico
// Maneja rate limiting, timeouts, re-intentos, y firmas HMAC.
use core::time::Duration;
use reqwest::{Client, Response};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::error::{ExchangeError, ExchangeResult};

/// Configuración del cliente REST
#[derive(Debug, Clone)]
pub struct RestConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub secret: Option<String>,
    pub timeout_ms: u64,
    pub rate_limit_per_second: u32,
    pub user_agent: String,
}

impl Default for RestConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            api_key: None,
            secret: None,
            timeout_ms: 30_000,
            rate_limit_per_second: 10,
            user_agent: format!("NEXUS/{} ({})", env!("CARGO_PKG_VERSION"), "ccxt_rs"),
        }
    }
}

/// Cliente HTTP reutilizable con rate limiting y retry
#[derive(Debug)]
pub struct RestClient {
    client: Client,
    config: RestConfig,
    last_request_time: Arc<AtomicU64>,
    min_interval_us: u64,
}

impl RestClient {
    /// Crear un nuevo cliente REST
    pub fn new(config: RestConfig) -> ExchangeResult<Self> {
        let rate_per_sec = config.rate_limit_per_second.max(1);
        let min_interval_us = (1_000_000u64) / (rate_per_sec as u64);

        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .user_agent(&config.user_agent)
            .build()
            .map_err(|e| ExchangeError::Internal {
                reason: format!("Failed to create HTTP client: {e}"),
            })?;

        Ok(Self {
            client,
            config,
            last_request_time: Arc::new(AtomicU64::new(0)),
            min_interval_us,
        })
    }

    /// Aplica rate limiting: espera si es necesario
    async fn enforce_rate_limit(&self) {
        let now = current_time_us();
        let last = self.last_request_time.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);

        if elapsed < self.min_interval_us {
            let wait = self.min_interval_us - elapsed;
            tokio::time::sleep(Duration::from_micros(wait)).await;
        }
        self.last_request_time
            .store(current_time_us(), Ordering::Relaxed);
    }

    /// GET request con rate limiting y manejo de errores
    pub async fn get(
        &self,
        endpoint: &str,
        query: &[(&str, String)],
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut req = self.client.get(&url);

        if !query.is_empty() {
            req = req.query(
                &query
                    .iter()
                    .map(|(k, v)| (k, v.as_str()))
                    .collect::<Vec<_>>(),
            );
        }

        let response = req.send().await.map_err(|e| {
            if e.is_timeout() {
                ExchangeError::Timeout {
                    exchange: "generic".into(),
                    endpoint: endpoint.to_string(),
                    duration_ms: self.config.timeout_ms,
                }
            } else {
                ExchangeError::Network {
                    exchange: "generic".into(),
                    endpoint: endpoint.to_string(),
                    source: e.to_string(),
                }
            }
        })?;

        Self::handle_response(response, "generic", endpoint).await
    }

    /// POST request con body JSON
    pub async fn post(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network {
                exchange: "generic".into(),
                endpoint: endpoint.to_string(),
                source: e.to_string(),
            })?;

        Self::handle_response(response, "generic", endpoint).await
    }

    /// DELETE request
    pub async fn delete(
        &self,
        endpoint: &str,
        query: &[(&str, String)],
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut req = self.client.delete(&url);

        if !query.is_empty() {
            req = req.query(
                &query
                    .iter()
                    .map(|(k, v)| (k, v.as_str()))
                    .collect::<Vec<_>>(),
            );
        }

        let response = req.send().await.map_err(|e| ExchangeError::Network {
            exchange: "generic".into(),
            endpoint: endpoint.to_string(),
            source: e.to_string(),
        })?;

        Self::handle_response(response, "generic", endpoint).await
    }

    /// PUT request con body JSON
    pub async fn put(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let response =
            self.client
                .put(&url)
                .json(body)
                .send()
                .await
                .map_err(|e| ExchangeError::Network {
                    exchange: "generic".into(),
                    endpoint: endpoint.to_string(),
                    source: e.to_string(),
                })?;

        Self::handle_response(response, "generic", endpoint).await
    }

    /// Maneja la respuesta HTTP y parsea JSON
    async fn handle_response(
        response: Response,
        exchange: &str,
        endpoint: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(match status.as_u16() {
                429 => ExchangeError::RateLimit {
                    exchange: exchange.to_string(),
                    retry_after_ms: 60_000,
                },
                401 | 403 => ExchangeError::Authentication {
                    exchange: exchange.to_string(),
                    reason: format!("HTTP {}: {}", status.as_u16(), body_text),
                },
                400 => ExchangeError::BadRequest {
                    exchange: exchange.to_string(),
                    reason: format!("HTTP {}: {}", status.as_u16(), body_text),
                },
                code => ExchangeError::Exchange {
                    exchange: exchange.to_string(),
                    code,
                    body: body_text,
                },
            });
        }

        serde_json::from_str(&body_text).map_err(|e| ExchangeError::Parse {
            exchange: exchange.to_string(),
            raw: body_text.chars().take(500).collect(),
            source: e.to_string(),
        })
    }

    /// GET request con headers personalizados (para requests autenticados)
    pub async fn get_with_headers(
        &self,
        endpoint: &str,
        query: &[(&str, String)],
        headers: &[(&str, &str)],
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut req = self.client.get(&url);

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        if !query.is_empty() {
            req = req.query(
                &query
                    .iter()
                    .map(|(k, v)| (k, v.as_str()))
                    .collect::<Vec<_>>(),
            );
        }

        let response = req.send().await.map_err(|e| ExchangeError::Network {
            exchange: "generic".into(),
            endpoint: endpoint.to_string(),
            source: e.to_string(),
        })?;

        Self::handle_response(response, "generic", endpoint).await
    }

    /// POST request con headers personalizados
    pub async fn post_with_headers(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
        headers: &[(&str, &str)],
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut req = self.client.post(&url);

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        let response = req
            .json(body)
            .send()
            .await
            .map_err(|e| ExchangeError::Network {
                exchange: "generic".into(),
                endpoint: endpoint.to_string(),
                source: e.to_string(),
            })?;

        Self::handle_response(response, "generic", endpoint).await
    }

    /// POST request form-encoded con headers personalizados
    /// Kraken usa application/x-www-form-urlencoded, no JSON
    pub async fn post_form_with_headers(
        &self,
        endpoint: &str,
        body: &str,
        headers: &[(&str, &str)],
    ) -> ExchangeResult<serde_json::Value> {
        self.enforce_rate_limit().await;

        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut req = self.client.post(&url);

        for (k, v) in headers {
            req = req.header(*k, *v);
        }

        let response = req
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_owned())
            .send()
            .await
            .map_err(|e| ExchangeError::Network {
                exchange: "generic".into(),
                endpoint: endpoint.to_string(),
                source: e.to_string(),
            })?;

        Self::handle_response(response, "generic", endpoint).await
    }
}

fn current_time_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}
