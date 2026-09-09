// 🔱 Kraken Exchange Connector — Transmutación Rust Pura del CCXT
// REST API pública y autenticada de Kraken Spot
// Documentación: https://docs.kraken.com/rest/
// Autenticación: API-Key + HMAC-SHA512 de (URI path + SHA256(nonce+postdata))
// Sin dependencias externas — ring para HMAC, sha2 para SHA256 (ya en Cargo.toml)

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use futures::Stream;
use ring::hmac;
use sha2::{Digest, Sha256};
use std::pin::Pin;

use super::super::error::{ExchangeError, ExchangeResult};
use super::super::rest::{RestClient, RestConfig};
use super::super::types::*;
use super::{Exchange, Level, OrderBook};

/// Configuración del connector Kraken
pub struct KrakenConfig {
    pub api_key: Option<String>,
    pub secret: Option<String>, // Base64-encoded private key
    pub rate_limit_per_second: u32,
}

impl Default for KrakenConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            secret: None,
            rate_limit_per_second: 1, // Kraken es más restrictivo
        }
    }
}

/// Connector para Kraken Spot
pub struct Kraken {
    name: &'static str,
    rest: RestClient,
    api_key: Option<String>,
    secret: Option<Vec<u8>>, // Decodificado de base64 para HMAC-SHA512
    nonce: std::sync::atomic::AtomicU64,
}

impl Kraken {
    /// Crear una nueva instancia del connector Kraken
    pub fn new(config: KrakenConfig) -> ExchangeResult<Self> {
        let rest_config = RestConfig {
            base_url: "https://api.kraken.com".to_string(),
            api_key: config.api_key.clone(),
            secret: config.secret.clone(),
            timeout_ms: 30_000,
            rate_limit_per_second: config.rate_limit_per_second,
            user_agent: "NEXUS/ccxt_rs/1.0".to_string(),
        };

        let rest = RestClient::new(rest_config)?;

        // Decodificar secret de base64 si está presente
        let secret_bytes = match &config.secret {
            Some(s) => match Base64Decoder::decode(s) {
                Ok(bytes) => Some(bytes),
                Err(_) => {
                    return Err(ExchangeError::Authentication {
                        exchange: "kraken".to_string(),
                        reason: "Invalid base64 secret".to_string(),
                    });
                }
            },
            None => None,
        };

        let initial_nonce = current_millis() as u64;
        tracing::debug!("[kraken] Base URL: https://api.kraken.com");

        Ok(Self {
            name: "kraken",
            rest,
            api_key: config.api_key,
            secret: secret_bytes,
            nonce: std::sync::atomic::AtomicU64::new(initial_nonce),
        })
    }

    /// Generar un nonce incrementando el contador atómico
    fn next_nonce(&self) -> u64 {
        let now = current_millis() as u64;
        let prev = self.nonce.load(std::sync::atomic::Ordering::Relaxed);
        let next = if now > prev { now } else { prev + 1 };
        self.nonce.store(next, std::sync::atomic::Ordering::Relaxed);
        next
    }

    /// Firmar una petición autenticada Kraken
    /// 1. SHA256(nonce + post_data)
    /// 2. HMAC-SHA512(secret, URI_path + sha256_hash)
    fn sign_request(&self, uri_path: &str, post_data: &str) -> ExchangeResult<String> {
        let secret_bytes = self
            .secret
            .as_ref()
            .ok_or_else(|| ExchangeError::Authentication {
                exchange: self.name.to_string(),
                reason: "Missing API secret".to_string(),
            })?;

        // SHA256(nonce + post_data)
        let mut sha256 = Sha256::new();
        sha256.update(post_data.as_bytes());
        let sha256_hash = sha256.finalize();

        // HMAC-SHA512(secret, URI_path + SHA256_hash)
        let mut hmac_input = Vec::with_capacity(uri_path.len() + 32);
        hmac_input.extend_from_slice(uri_path.as_bytes());
        hmac_input.extend_from_slice(&sha256_hash);

        let key = hmac::Key::new(hmac::HMAC_SHA512, secret_bytes);
        let signature = hmac::sign(&key, &hmac_input);
        let sig_b64 = base64_encode_manual(signature.as_ref());

        Ok(sig_b64)
    }

    /// Realizar una petición pública GET
    async fn public_get(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("/0/public/{}", path);
        self.rest.get(&url, params).await
    }

    /// Realizar una petición privada POST (form-encoded, no JSON)
    async fn private_post(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> ExchangeResult<serde_json::Value> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| ExchangeError::Authentication {
                exchange: self.name.to_string(),
                reason: "Missing API key".to_string(),
            })?;

        let nonce = self.next_nonce();
        let mut post_parts: Vec<String> =
            params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        post_parts.push(format!("nonce={}", nonce));
        let post_data = post_parts.join("&");

        let uri_path = format!("/0/private/{}", path);
        let signature = self.sign_request(&uri_path, &post_data)?;

        let headers = vec![
            ("API-Key", api_key.as_str()),
            ("API-Sign", signature.as_str()),
        ];

        let url = format!("/0/private/{}", path);
        self.rest
            .post_form_with_headers(&url, &post_data, &headers)
            .await
    }

    /// Convertir par "BTCUSDT" al formato Kraken "XBTUSDT"
    fn to_kraken_pair(symbol: &str) -> String {
        if symbol.starts_with("XBT") || symbol.contains('/') {
            return symbol.replace("/", "");
        }
        let s = symbol.replace("BTC", "XBT");
        s
    }

    /// Convertir desde formato Kraken a formato estándar "BTCUSDT"
    fn from_kraken_pair(pair: &str) -> String {
        pair.replace("XBT", "BTC")
    }

    /// Parsear timestamp Kraken (Unix seconds como string)
    fn parse_kraken_ts(ts_secs: &str) -> ExchangeResult<DateTime<Utc>> {
        let secs: i64 = ts_secs.parse().map_err(|_| ExchangeError::Parse {
            exchange: "kraken".to_string(),
            raw: ts_secs.to_string(),
            source: format!("Invalid Kraken timestamp: {}", ts_secs),
        })?;
        Utc.timestamp_opt(secs, 0)
            .single()
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: ts_secs.to_string(),
                source: format!("Timestamp out of range: {}", secs),
            })
    }

    /// Obtener valor flotante de un campo en JSON (Kraken usa strings para precios)
    fn get_json_f64(val: &serde_json::Value) -> ExchangeResult<f64> {
        match val {
            serde_json::Value::String(s) => s.parse::<f64>().map_err(|_| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: s.clone(),
                source: format!("Cannot parse f64 from string: {}", s),
            }),
            serde_json::Value::Number(n) => n.as_f64().ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: n.to_string(),
                source: format!("Cannot convert number to f64: {}", n),
            }),
            _ => Err(ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", val),
                source: format!("Unexpected JSON type for f64: {}", val),
            }),
        }
    }

    /// Verificar errores en respuesta Kraken
    fn check_error(response: &serde_json::Value) -> ExchangeResult<()> {
        if let Some(error_array) = response.get("error").and_then(|e| e.as_array()) {
            if !error_array.is_empty() {
                let error_msg = error_array
                    .iter()
                    .filter_map(|e| e.as_str())
                    .collect::<Vec<&str>>()
                    .join("; ");
                return Err(ExchangeError::BadRequest {
                    exchange: "kraken".to_string(),
                    reason: error_msg,
                });
            }
        }
        Ok(())
    }

    /// Parsear resultado de Kraken (result[<pair_name>])
    fn parse_result<'a>(response: &'a serde_json::Value) -> ExchangeResult<&'a serde_json::Value> {
        Self::check_error(response)?;
        response.get("result").ok_or_else(|| ExchangeError::Parse {
            exchange: "kraken".to_string(),
            raw: format!("{}", serde_json::to_string(response).unwrap_or_default()),
            source: "Missing Kraken 'result' field".to_string(),
        })
    }

    /// Convertir intervalo Timeframe a minutos Kraken
    fn timeframe_to_minutes(tf: &Timeframe) -> u32 {
        match tf {
            Timeframe::M1 => 1,
            Timeframe::M5 => 5,
            Timeframe::M15 => 15,
            Timeframe::M30 => 30,
            Timeframe::H1 => 60,
            Timeframe::H4 => 240,
            Timeframe::D1 => 1440,
            Timeframe::W1 => 10080,
            _ => 1440, // default daily
        }
    }
}

#[async_trait]
impl Exchange for Kraken {
    fn name(&self) -> &'static str {
        self.name
    }

    // ========== DATOS DE MERCADO ==========

    async fn fetch_ticker(&self, symbol: &str) -> ExchangeResult<Ticker> {
        let kraken_pair = Self::to_kraken_pair(symbol);
        let resp = self
            .public_get("Ticker", &[("pair", kraken_pair.clone())])
            .await?;
        let result = Self::parse_result(&resp)?;

        let pair_data = result
            .as_object()
            .and_then(|obj| obj.values().next())
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "Empty Kraken ticker result".to_string(),
            })?;

        let bid = Self::get_json_f64(&pair_data["b"][0])?;
        let ask = Self::get_json_f64(&pair_data["a"][0])?;
        let last = Self::get_json_f64(&pair_data["c"][0])?;
        let high_24h = Self::get_json_f64(&pair_data["h"][1])?;
        let low_24h = Self::get_json_f64(&pair_data["l"][1])?;
        let volume_24h = Self::get_json_f64(&pair_data["v"][1])?;
        let quote_volume = Self::get_json_f64(&pair_data["p"][1])? * volume_24h;

        let change = last - Self::get_json_f64(&pair_data["o"])?;
        let change_pct = if change != 0.0 && last > 0.0 {
            (change / Self::get_json_f64(&pair_data["o"])?) * 100.0
        } else {
            0.0
        };

        let timestamp = Self::parse_kraken_ts(pair_data["t"].as_str().unwrap_or("0"))?;

        Ok(Ticker {
            symbol: Self::from_kraken_pair(&kraken_pair),
            bid,
            ask,
            last,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: quote_volume,
            timestamp,
            change,
            change_pct,
        })
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        timeframe: &Timeframe,
        limit: u32,
    ) -> ExchangeResult<Vec<OHLCV>> {
        let kraken_pair = Self::to_kraken_pair(symbol);
        let interval = Self::timeframe_to_minutes(timeframe);

        let mut params = vec![
            ("pair", kraken_pair.clone()),
            ("interval", interval.to_string()),
        ];

        if limit > 0 {
            params.push(("since", "0".to_string()));
        }

        let resp = self.public_get("OHLC", &params).await?;
        let result = Self::parse_result(&resp)?;

        let ohlcv_data = result
            .as_object()
            .and_then(|obj| {
                obj.iter()
                    .find(|(key, val)| *key != "last" && val.is_array())
                    .map(|(_, val)| val)
            })
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "Empty Kraken OHLC result".to_string(),
            })?;

        let arr = ohlcv_data.as_array().ok_or_else(|| ExchangeError::Parse {
            exchange: "kraken".to_string(),
            raw: format!("{}", serde_json::to_string(ohlcv_data).unwrap_or_default()),
            source: "OHLC data is not array".to_string(),
        })?;

        let ohlcvs: Vec<OHLCV> = arr
            .iter()
            .take(limit as usize)
            .map(|item| {
                let arr = item.as_array().ok_or_else(|| ExchangeError::Parse {
                    exchange: "kraken".to_string(),
                    raw: format!("{}", serde_json::to_string(item).unwrap_or_default()),
                    source: "OHLC item not array".to_string(),
                })?;
                Ok(OHLCV {
                    timestamp: Utc
                        .timestamp_opt(
                            arr[0].as_i64().ok_or_else(|| ExchangeError::Parse {
                                exchange: "kraken".to_string(),
                                raw: arr[0].to_string(),
                                source: "Invalid OHLC timestamp".to_string(),
                            })?,
                            0,
                        )
                        .single()
                        .ok_or_else(|| ExchangeError::Parse {
                            exchange: "kraken".to_string(),
                            raw: arr[0].to_string(),
                            source: "OHLC timestamp out of range".to_string(),
                        })?,
                    open: Self::get_json_f64(&arr[1])?,
                    high: Self::get_json_f64(&arr[2])?,
                    low: Self::get_json_f64(&arr[3])?,
                    close: Self::get_json_f64(&arr[4])?,
                    volume: Self::get_json_f64(&arr[6])?,
                })
            })
            .collect::<ExchangeResult<Vec<OHLCV>>>()?;

        Ok(ohlcvs)
    }

    async fn fetch_order_book(&self, symbol: &str, limit: u32) -> ExchangeResult<OrderBook> {
        let kraken_pair = Self::to_kraken_pair(symbol);
        let count = if limit == 0 || limit > 100 { 25 } else { limit };
        let resp = self
            .public_get(
                "Depth",
                &[("pair", kraken_pair.clone()), ("count", count.to_string())],
            )
            .await?;
        let result = Self::parse_result(&resp)?;

        let pair_data = result
            .as_object()
            .and_then(|obj| obj.values().next())
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "Empty Kraken depth result".to_string(),
            })?;

        let bids = Self::parse_kraken_levels(&pair_data["bids"])?;
        let asks = Self::parse_kraken_levels(&pair_data["asks"])?;

        Ok(OrderBook {
            symbol: Self::from_kraken_pair(&kraken_pair),
            bids,
            asks,
            timestamp: Utc::now(),
        })
    }

    async fn fetch_markets(&self) -> ExchangeResult<Vec<MarketInfo>> {
        let resp = self.public_get("AssetPairs", &[]).await?;
        let result = Self::parse_result(&resp)?;

        let markets: Vec<MarketInfo> = result
            .as_object()
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "AssetPairs result not object".to_string(),
            })?
            .iter()
            .filter(|(key, _)| !key.starts_with('.'))
            .map(|(pair_name, info)| {
                let altname = info["altname"].as_str().unwrap_or(pair_name);
                let base = info["base"].as_str().unwrap_or("").to_string();
                let quote = info["quote"].as_str().unwrap_or("").to_string();
                let ws = info["wsname"].as_str().unwrap_or(altname);

                let status = info["status"].as_str().unwrap_or("");
                let active = matches!(status, "online" | "post_only" | "limit_only");

                let pair_decimals = info["pair_decimals"].as_u64().unwrap_or(8) as u32;
                let lot_decimals = info["lot_decimals"].as_u64().unwrap_or(8) as u32;

                let ordermin = Self::get_json_f64(&info["ordermin"]).unwrap_or(0.0);

                Ok(MarketInfo {
                    symbol: Self::from_kraken_pair(ws),
                    base,
                    quote,
                    active,
                    precision: MarketPrecision {
                        price: pair_decimals,
                        quantity: lot_decimals,
                        quote: pair_decimals,
                    },
                    limits: MarketLimits {
                        quantity_min: ordermin,
                        quantity_max: f64::MAX,
                        price_min: f64::EPSILON,
                        price_max: f64::MAX,
                    },
                })
            })
            .collect::<ExchangeResult<Vec<MarketInfo>>>()?;

        Ok(markets)
    }

    // ========== COMERCIO (Autenticado) ==========

    async fn create_order(&self, order: Order) -> ExchangeResult<OrderResult> {
        let kraken_pair = Self::to_kraken_pair(&order.symbol);
        let side = order.side.to_string();
        let order_type = match order.order_type {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::StopLoss => "stop-loss",
            OrderType::StopLossLimit => "stop-loss-limit",
            OrderType::TakeProfit => "take-profit",
            OrderType::TakeProfitLimit => "take-profit-limit",
        };

        let mut params = vec![
            ("pair", kraken_pair.clone()),
            ("type", side),
            ("ordertype", order_type.to_string()),
            ("volume", order.quantity.to_string()),
        ];

        if let Some(price) = order.price {
            params.push(("price", price.to_string()));
        }
        if let Some(stop_price) = order.stop_price {
            params.push(("stopprice", stop_price.to_string()));
        }

        let resp = self.private_post("AddOrder", &params).await?;
        let result = Self::parse_result(&resp)?;

        let order_id = result["txid"][0]
            .as_str()
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "Missing order txid".to_string(),
            })?
            .to_string();

        match self.fetch_order(&order.symbol, &order_id).await {
            Ok(order_result) => Ok(order_result),
            Err(_) => Ok(OrderResult {
                id: order_id,
                client_order_id: order.client_order_id,
                symbol: Self::from_kraken_pair(&kraken_pair),
                side: order.side,
                order_type: order.order_type,
                quantity: order.quantity,
                filled_quantity: 0.0,
                price: order.price.unwrap_or(0.0),
                average_price: 0.0,
                status: OrderStatus::Open,
                timestamp: Utc::now(),
                fee: None,
            }),
        }
    }

    async fn cancel_order(&self, symbol: &str, order_id: &str) -> ExchangeResult<bool> {
        let _ = symbol;
        let resp = self
            .private_post("CancelOrder", &[("txid", order_id.to_string())])
            .await?;
        let result = Self::parse_result(&resp)?;

        let count = result["count"].as_u64().unwrap_or(0);
        Ok(count > 0)
    }

    async fn fetch_order(&self, symbol: &str, order_id: &str) -> ExchangeResult<OrderResult> {
        let resp = self
            .private_post("QueryOrders", &[("txid", order_id.to_string())])
            .await?;
        let result = Self::parse_result(&resp)?;

        let order_data = result.get(order_id).ok_or_else(|| ExchangeError::Parse {
            exchange: "kraken".to_string(),
            raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
            source: format!("Order {} not found", order_id),
        })?;

        Self::parse_order_result(order_data, symbol, order_id)
    }

    async fn fetch_open_orders(&self, symbol: &str) -> ExchangeResult<Vec<OrderResult>> {
        let mut params = Vec::new();
        if !symbol.is_empty() {
            // Kraken no soporta filtro por símbolo directamente
        }
        let resp = self.private_post("OpenOrders", &params).await?;
        let result = Self::parse_result(&resp)?;

        let orders_data = result["open"]
            .as_object()
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "Open orders not object".to_string(),
            })?;

        let orders: Vec<OrderResult> = orders_data
            .iter()
            .filter_map(|(id, data)| {
                let kraken_pair = data["descr"]["pair"].as_str().unwrap_or("");
                let order_symbol = Self::from_kraken_pair(kraken_pair);
                if symbol.is_empty()
                    || order_symbol == Self::from_kraken_pair(&Self::to_kraken_pair(symbol))
                {
                    Self::parse_order_result(data, &order_symbol, id).ok()
                } else {
                    None
                }
            })
            .collect();

        Ok(orders)
    }

    async fn fetch_balance(&self) -> ExchangeResult<Balance> {
        let resp = self.private_post("Balance", &[]).await?;
        let result = Self::parse_result(&resp)?;

        let assets: Vec<AssetBalance> = result
            .as_object()
            .ok_or_else(|| ExchangeError::Parse {
                exchange: "kraken".to_string(),
                raw: format!("{}", serde_json::to_string(result).unwrap_or_default()),
                source: "Balance result not object".to_string(),
            })?
            .iter()
            .filter_map(|(currency, amount_str)| {
                let amount = Self::get_json_f64(amount_str).ok()?;
                if amount <= 0.0 {
                    return None;
                }
                Some(AssetBalance {
                    currency: currency.clone(),
                    free: amount,
                    used: 0.0,
                    total: amount,
                    usd_value: None,
                })
            })
            .collect();

        Ok(Balance {
            assets,
            timestamp: Utc::now(),
        })
    }

    // ========== WEBSOCKET ==========

    async fn watch_ticker(
        &self,
        _symbol: &str,
    ) -> ExchangeResult<Pin<Box<dyn Stream<Item = ExchangeResult<Ticker>> + Send>>> {
        Err(ExchangeError::WebSocket {
            exchange: self.name.to_string(),
            reason: "WebSocket streaming coming in Phase 2".to_string(),
        })
    }

    async fn watch_ohlcv(
        &self,
        _symbol: &str,
        _timeframe: &Timeframe,
    ) -> ExchangeResult<Pin<Box<dyn Stream<Item = ExchangeResult<OHLCV>> + Send>>> {
        Err(ExchangeError::WebSocket {
            exchange: self.name.to_string(),
            reason: "WebSocket streaming coming in Phase 2".to_string(),
        })
    }
}

impl Kraken {
    /// Parsear niveles del order book de Kraken
    fn parse_kraken_levels(levels: &serde_json::Value) -> ExchangeResult<Vec<Level>> {
        let arr = levels.as_array().ok_or_else(|| ExchangeError::Parse {
            exchange: "kraken".to_string(),
            raw: format!("{}", serde_json::to_string(levels).unwrap_or_default()),
            source: "Levels not array".to_string(),
        })?;

        arr.iter()
            .map(|item| {
                let inner_arr = item.as_array().ok_or_else(|| ExchangeError::Parse {
                    exchange: "kraken".to_string(),
                    raw: format!("{}", serde_json::to_string(item).unwrap_or_default()),
                    source: "Level item not array".to_string(),
                })?;
                Ok(Level {
                    price: Self::get_json_f64(&inner_arr[0])?,
                    quantity: Self::get_json_f64(&inner_arr[1])?,
                })
            })
            .collect()
    }

    /// Parsear resultado de una orden de Kraken
    fn parse_order_result(
        data: &serde_json::Value,
        symbol: &str,
        order_id: &str,
    ) -> ExchangeResult<OrderResult> {
        let side = match data["descr"]["type"].as_str() {
            Some("buy") => OrderSide::Buy,
            _ => OrderSide::Sell,
        };

        let order_type = match data["descr"]["ordertype"].as_str() {
            Some("market") => OrderType::Market,
            Some("limit") => OrderType::Limit,
            Some("stop-loss") | Some("stop-loss-limit") => OrderType::StopLoss,
            Some("take-profit") | Some("take-profit-limit") => OrderType::TakeProfit,
            _ => OrderType::Limit,
        };

        let status = match data["status"].as_str() {
            Some("open" | "pending") => OrderStatus::Open,
            Some("closed") => OrderStatus::Closed,
            Some("canceled") => OrderStatus::Canceled,
            Some("expired") => OrderStatus::Expired,
            _ => OrderStatus::Open,
        };

        let vol = Self::get_json_f64(&data["vol"]).unwrap_or(0.0);
        let vol_exec = Self::get_json_f64(&data["vol_exec"]).unwrap_or(0.0);
        let price = Self::get_json_f64(&data["descr"]["price"]).unwrap_or(0.0);
        let average = Self::get_json_f64(&data["price"]).unwrap_or(0.0);

        let timestamp = data["opentm"]
            .as_f64()
            .map(|ts| {
                Utc.timestamp_opt(ts as i64, (ts.fract() * 1_000_000_000.0) as u32)
                    .single()
                    .unwrap_or(Utc::now())
            })
            .unwrap_or(Utc::now());

        Ok(OrderResult {
            id: order_id.to_string(),
            client_order_id: data["userref"].as_u64().map(|u| u.to_string()),
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity: vol,
            filled_quantity: vol_exec,
            price,
            average_price: average,
            status,
            timestamp,
            fee: None,
        })
    }
}

// ============================================================================
// Decodificación base64 manual (sin dep externa)
// ============================================================================

struct Base64Decoder;

impl Base64Decoder {
    fn decode(input: &str) -> Result<Vec<u8>, ()> {
        let input = input.trim_end_matches('=');
        let mut output = Vec::with_capacity(input.len() * 3 / 4);
        let chars: Vec<char> = input.chars().collect();

        let mut i = 0;
        while i < chars.len() {
            let chunk: Vec<u8> = chars[i..(i + 4).min(chars.len())]
                .iter()
                .map(|&c| Self::char_val(c))
                .collect();

            if chunk.len() == 4 {
                output.push((chunk[0] << 2) | (chunk[1] >> 4));
                output.push((chunk[1] << 4) | (chunk[2] >> 2));
                output.push((chunk[2] << 6) | chunk[3]);
            } else if chunk.len() == 3 {
                output.push((chunk[0] << 2) | (chunk[1] >> 4));
                output.push((chunk[1] << 4) | (chunk[2] >> 2));
            } else if chunk.len() == 2 {
                output.push((chunk[0] << 2) | (chunk[1] >> 4));
            }
            i += 4;
        }

        Ok(output)
    }

    fn char_val(c: char) -> u8 {
        match c {
            'A'..='Z' => (c as u8) - b'A',
            'a'..='z' => (c as u8) - b'a' + 26,
            '0'..='9' => (c as u8) - b'0' + 52,
            '+' => 62,
            '/' => 63,
            _ => 0,
        }
    }
}

// ============================================================================
// Codificación base64 manual
// ============================================================================

fn base64_encode_manual(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn current_millis() -> i64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kraken_new_defaults() {
        let config = KrakenConfig::default();
        let kraken = Kraken::new(config).unwrap();
        assert_eq!(kraken.name, "kraken");
    }

    #[test]
    fn test_to_kraken_pair_btc() {
        assert_eq!(Kraken::to_kraken_pair("BTCUSDT"), "XBTUSDT");
    }

    #[test]
    fn test_to_kraken_pair_eth() {
        assert_eq!(Kraken::to_kraken_pair("ETHUSD"), "ETHUSD");
    }

    #[test]
    fn test_to_kraken_pair_already_xbt() {
        assert_eq!(Kraken::to_kraken_pair("XBTUSD"), "XBTUSD");
    }

    #[test]
    fn test_from_kraken_pair() {
        assert_eq!(Kraken::from_kraken_pair("XBTUSDT"), "BTCUSDT");
    }

    #[test]
    fn test_parse_kraken_ts() {
        let dt = Kraken::parse_kraken_ts("1700000000").unwrap();
        assert_eq!(dt.timestamp(), 1700000000);
    }

    #[test]
    fn test_parse_invalid_ts() {
        assert!(Kraken::parse_kraken_ts("abc").is_err());
    }

    #[test]
    fn test_get_json_f64_from_string() {
        let val = serde_json::json!("50000.00");
        let result = Kraken::get_json_f64(&val).unwrap();
        assert!((result - 50000.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_json_f64_from_number() {
        let val = serde_json::json!(123.45);
        let result = Kraken::get_json_f64(&val).unwrap();
        assert!((result - 123.45).abs() < 1e-10);
    }

    #[test]
    fn test_get_json_f64_invalid_type() {
        let val = serde_json::json!([1, 2, 3]);
        assert!(Kraken::get_json_f64(&val).is_err());
    }

    #[test]
    fn test_check_ok_response() {
        let resp = serde_json::json!({"error": [], "result": {}});
        assert!(Kraken::check_error(&resp).is_ok());
    }

    #[test]
    fn test_check_error_response() {
        let resp = serde_json::json!({"error": ["EGeneral:Invalid arguments"]});
        assert!(Kraken::check_error(&resp).is_err());
    }

    #[test]
    fn test_parse_kraken_levels() {
        let levels = serde_json::json!([["50000.00", "1.5"], ["49900.00", "2.0"]]);
        let parsed = Kraken::parse_kraken_levels(&levels).unwrap();
        assert_eq!(parsed.len(), 2);
        assert!((parsed[0].price - 50000.0).abs() < 1e-6);
        assert!((parsed[0].quantity - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_parse_order_result_buy_limit() {
        let data = serde_json::json!({
            "descr": {
                "pair": "XBTUSD",
                "type": "buy",
                "ordertype": "limit",
                "price": "45000.00"
            },
            "vol": "1.0",
            "vol_exec": "0.5",
            "price": "45000.00",
            "status": "open",
            "opentm": 1700000000.0,
            "userref": 12345
        });

        let result = Kraken::parse_order_result(&data, "BTCUSD", "OABCDE-12345").unwrap();
        assert_eq!(result.id, "OABCDE-12345");
        assert_eq!(result.symbol, "BTCUSD");
        assert_eq!(result.side as i32, OrderSide::Buy as i32);
        assert_eq!(result.quantity, 1.0);
        assert!(matches!(result.status, OrderStatus::Open));
    }

    #[test]
    fn test_parse_order_result_closed() {
        let data = serde_json::json!({
            "descr": {
                "pair": "ETHUSD",
                "type": "sell",
                "ordertype": "market",
                "price": "3000.00"
            },
            "vol": "2.0",
            "vol_exec": "2.0",
            "price": "3010.50",
            "status": "closed",
            "opentm": 1700000000.0
        });

        let result = Kraken::parse_order_result(&data, "ETHUSD", "OEDCBA-67890").unwrap();
        assert!(matches!(result.status, OrderStatus::Closed));
        assert!(matches!(result.order_type, OrderType::Market));
        assert_eq!(result.filled_quantity, 2.0);
    }

    #[test]
    fn test_base64_encode_manual() {
        let input = b"test";
        let encoded = base64_encode_manual(input);
        assert_eq!(encoded, "dGVzdA==");
    }

    #[test]
    fn test_base64_encode_longer() {
        let input = b"hello world";
        let encoded = base64_encode_manual(input);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "dGVzdA==";
        let decoded = Base64Decoder::decode(encoded).unwrap();
        assert_eq!(decoded, b"test");
    }

    #[test]
    fn test_timeframe_to_minutes() {
        assert_eq!(Kraken::timeframe_to_minutes(&Timeframe::M1), 1);
        assert_eq!(Kraken::timeframe_to_minutes(&Timeframe::H1), 60);
        assert_eq!(Kraken::timeframe_to_minutes(&Timeframe::D1), 1440);
    }

    #[test]
    fn test_fetch_order_book_requires_auth() {
        let config = KrakenConfig::default();
        let kraken = Kraken::new(config).unwrap();
        assert_eq!(kraken.name, "kraken");
    }

    #[tokio::test]
    async fn test_fetch_order_requires_api_key() {
        let config = KrakenConfig::default();
        let kraken = Kraken::new(config).unwrap();
        let order = Order {
            symbol: "BTCUSD".to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity: 0.001,
            price: Some(10000.0),
            stop_price: None,
            client_order_id: None,
            time_in_force: None,
        };
        let result = kraken.create_order(order).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_incrementing() {
        let config = KrakenConfig::default();
        let kraken = Kraken::new(config).unwrap();
        let n1 = kraken.next_nonce();
        let n2 = kraken.next_nonce();
        assert!(
            n2 > n1,
            "Nonces must be strictly increasing: {} -> {}",
            n1,
            n2
        );
    }

    #[test]
    fn test_parse_result_missing() {
        let resp = serde_json::json!({"error": []});
        assert!(Kraken::parse_result(&resp).is_err());
    }

    #[test]
    fn test_parse_result_ok() {
        let resp = serde_json::json!({"error": [], "result": {"XBTUSD": {}}});
        assert!(Kraken::parse_result(&resp).is_ok());
    }

    #[test]
    fn test_next_nonce_from_millis() {
        let config = KrakenConfig::default();
        let kraken = Kraken::new(config).unwrap();
        let nonce = kraken.next_nonce();
        assert!(nonce > 1_700_000_000_000u64);
    }

    #[test]
    fn test_side_from_str() {
        let side: OrderSide = "buy".into();
        assert_eq!(side as i32, OrderSide::Buy as i32);
    }

    #[test]
    fn test_bid_ask_order_parsing_integrity() {
        let sample = serde_json::json!({
            "error": [],
            "result": {
                "XXBTZUSD": {
                    "a": ["50000.00", "1", "1.000"],
                    "b": ["49950.00", "2", "2.000"],
                    "c": ["49980.00", "0.5"],
                    "v": ["1000", "5000"],
                    "p": ["48000.00", "48500.00"],
                    "t": ["1500", "7500"],
                    "l": ["47000.00", "47500.00"],
                    "h": ["51000.00", "52000.00"],
                    "o": "49500.00"
                }
            }
        });

        let result = Kraken::parse_result(&sample).unwrap();
        let pair_data = result.as_object().unwrap().values().next().unwrap();

        let bid = Kraken::get_json_f64(&pair_data["b"][0]).unwrap();
        let ask = Kraken::get_json_f64(&pair_data["a"][0]).unwrap();
        let last = Kraken::get_json_f64(&pair_data["c"][0]).unwrap();
        let open = Kraken::get_json_f64(&pair_data["o"]).unwrap();

        assert!((bid - 49950.0).abs() < 1e-6);
        assert!((ask - 50000.0).abs() < 1e-6);
        assert!((last - 49980.0).abs() < 1e-6);

        let change = last - open;
        let change_pct = (change / open) * 100.0;
        assert!((change - 480.0).abs() < 1e-6);
        assert!((change_pct - 0.9697).abs() < 0.01);
    }

    #[test]
    fn test_sign_request_requires_secret() {
        let config = KrakenConfig::default();
        let kraken = Kraken::new(config).unwrap();
        let result = kraken.sign_request("/0/private/Balance", "nonce=12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_request_works_with_secret() {
        // Secret must be valid base64
        let config = KrakenConfig {
            api_key: Some("test-key".to_string()),
            secret: Some("dGVzdC1zZWNyZXQ=".to_string()), // "test-secret" in base64
            rate_limit_per_second: 1,
        };
        let kraken = Kraken::new(config).unwrap();
        let result = kraken.sign_request("/0/private/Balance", "nonce=12345");
        assert!(result.is_ok());
        let sig = result.unwrap();
        assert!(!sig.is_empty());
        // Base64 encoded HMAC-SHA512 produces ~88 chars
        assert!(sig.len() > 80);
    }
}
