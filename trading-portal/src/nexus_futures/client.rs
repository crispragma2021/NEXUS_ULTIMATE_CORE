// ============================================================================
// nexus_futures::client — Cliente HMAC para Binance Futures USDT-M
// ============================================================================
// Firma idéntica a la del spot: HMAC-SHA256 + X-MBX-APIKEY.
// Endpoints: fapi.binance.com
// ============================================================================

use super::types::*;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

const BASE_URL: &str = "https://fapi.binance.com";
const RECV_WINDOW: u32 = 5000;

pub struct FuturesClient {
    api_key: String,
    secret_key: String,
    http: reqwest::Client,
}

impl FuturesClient {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
            http: reqwest::Client::new(),
        }
    }

    // ─── HMAC helpers ───────────────────────────────────────────────────────

    fn sign(&self, query: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC key valid");
        mac.update(query.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn timestamp_query(&self) -> String {
        format!("timestamp={}&recvWindow={}", Utc::now().timestamp_millis(), RECV_WINDOW)
    }

    fn signed_url(&self, endpoint: &str, params: &str) -> String {
        let full_params = if params.is_empty() {
            self.timestamp_query()
        } else {
            format!("{}&{}", params, self.timestamp_query())
        };
        let sig = self.sign(&full_params);
        format!("{}{}?{}&signature={}", BASE_URL, endpoint, full_params, sig)
    }

    // ─── Account / Balance ──────────────────────────────────────────────────

    /// GET /fapi/v2/account — Balance completo + posiciones abiertas
    pub async fn account_info(&self) -> anyhow::Result<FuturesAccountInfo> {
        let url = self.signed_url("/fapi/v2/account", "");
        let resp = self.http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures Account error: {}", body))
        }
    }

    /// GET /fapi/v2/balance — Solo balances de activos
    pub async fn balance(&self) -> anyhow::Result<Vec<FuturesAsset>> {
        let info = self.account_info().await?;
        Ok(info.assets)
    }

    /// GET /fapi/v2/positionRisk — Posiciones abiertas
    pub async fn positions(&self, symbol: Option<&str>) -> anyhow::Result<Vec<FuturesPosition>> {
        let params = match symbol {
            Some(sym) => format!("symbol={}", sym.to_uppercase()),
            None => String::new(),
        };
        let url = self.signed_url("/fapi/v2/positionRisk", &params);
        let resp = self.http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures PositionRisk error: {}", body))
        }
    }

    // ─── Leverage ───────────────────────────────────────────────────────────

    /// POST /fapi/v1/leverage — Cambiar leverage de un símbolo (1-125)
    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> anyhow::Result<LeverageResponse> {
        let params = format!("symbol={}&leverage={}", symbol.to_uppercase(), leverage);
        let url = self.signed_url("/fapi/v1/leverage", &params);
        let resp = self.http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures Leverage error: {}", body))
        }
    }

    // ─── Margin Type ────────────────────────────────────────────────────────

    /// POST /fapi/v1/marginType — ISOLATED | CROSSED
    pub async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> anyhow::Result<serde_json::Value> {
        let params = format!("symbol={}&marginType={}", symbol.to_uppercase(), margin_type.to_uppercase());
        let url = self.signed_url("/fapi/v1/marginType", &params);
        let resp = self.http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures MarginType error: {}", body))
        }
    }

    // ─── Position Mode (Hedge / One-way) ────────────────────────────────────

    /// POST /fapi/v1/positionSide/dual — true = hedge mode, false = one-way
    pub async fn set_position_mode(&self, dual: bool) -> anyhow::Result<serde_json::Value> {
        let params = format!("dualSidePosition={}", dual);
        let url = self.signed_url("/fapi/v1/positionSide/dual", &params);
        let resp = self.http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures PositionSide/Dual error: {}", body))
        }
    }

    // ─── Orders ─────────────────────────────────────────────────────────────

    /// POST /fapi/v1/order — Colocar orden en futures
    pub async fn place_order(&self, req: &FuturesOrderRequest) -> anyhow::Result<FuturesOrderResponse> {
        let mut params = format!(
            "symbol={}&side={}&type={}&quantity={}",
            req.symbol.to_uppercase(),
            serde_json::to_string(&req.side).unwrap().trim_matches('"'),
            serde_json::to_string(&req.order_type).unwrap().trim_matches('"'),
            req.quantity,
        );

        if let Some(ref ps) = req.position_side {
            params.push_str(&format!(
                "&positionSide={}",
                serde_json::to_string(ps).unwrap().trim_matches('"')
            ));
        }
        if let Some(price) = req.price {
            params.push_str(&format!("&price={}", price));
        }
        if let Some(stop) = req.stop_price {
            params.push_str(&format!("&stopPrice={}", stop));
        }
        if let Some(td) = req.trailing_delta {
            params.push_str(&format!("&trailingDelta={}", td));
        }
        if let Some(ro) = req.reduce_only {
            params.push_str(&format!("&reduceOnly={}", ro));
        }
        if let Some(po) = req.post_only {
            params.push_str(&format!("&postOnly={}", po));
        }
        if let Some(cp) = req.close_position {
            params.push_str(&format!("&closePosition={}", cp));
        }
        if let Some(ref tif) = req.time_in_force {
            params.push_str(&format!(
                "&timeInForce={}",
                serde_json::to_string(tif).unwrap().trim_matches('"')
            ));
        }
        if let Some(ref wt) = req.working_type {
            params.push_str(&format!(
                "&workingType={}",
                serde_json::to_string(wt).unwrap().trim_matches('"')
            ));
        }
        if let Some(ref cid) = req.new_client_order_id {
            params.push_str(&format!("&newClientOrderId={}", cid));
        }

        let url = self.signed_url("/fapi/v1/order", &params);
        let resp = self.http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures Order error: {}", body))
        }
    }

    /// DELETE /fapi/v1/order — Cancelar orden por ID o clientOrderId
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        client_order_id: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let mut params = format!("symbol={}", symbol.to_uppercase());
        if let Some(id) = order_id {
            params.push_str(&format!("&orderId={}", id));
        }
        if let Some(cid) = client_order_id {
            params.push_str(&format!("&origClientOrderId={}", cid));
        }
        let url = self.signed_url("/fapi/v1/order", &params);
        let resp = self.http
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures Cancel error: {}", body))
        }
    }

    /// DELETE /fapi/v1/allOpenOrders — Cancelar todas las órdenes abiertas
    pub async fn cancel_all_orders(&self, symbol: &str) -> anyhow::Result<serde_json::Value> {
        let params = format!("symbol={}", symbol.to_uppercase());
        let url = self.signed_url("/fapi/v1/allOpenOrders", &params);
        let resp = self.http
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures CancelAll error: {}", body))
        }
    }

    /// GET /fapi/v1/openOrders — Órdenes abiertas
    pub async fn open_orders(&self, symbol: Option<&str>) -> anyhow::Result<Vec<FuturesOrderResponse>> {
        let params = match symbol {
            Some(sym) => format!("symbol={}", sym.to_uppercase()),
            None => String::new(),
        };
        let url = self.signed_url("/fapi/v1/openOrders", &params);
        let resp = self.http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures OpenOrders error: {}", body))
        }
    }

    /// GET /fapi/v1/order — Estado de una orden específica
    pub async fn order_status(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        client_order_id: Option<&str>,
    ) -> anyhow::Result<FuturesOrderResponse> {
        let mut params = format!("symbol={}", symbol.to_uppercase());
        if let Some(id) = order_id {
            params.push_str(&format!("&orderId={}", id));
        }
        if let Some(cid) = client_order_id {
            params.push_str(&format!("&origClientOrderId={}", cid));
        }
        let url = self.signed_url("/fapi/v1/order", &params);
        let resp = self.http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures OrderStatus error: {}", body))
        }
    }

    // ─── Trade History ──────────────────────────────────────────────────────

    /// GET /fapi/v1/userTrades — Historial de trades ejecutados
    pub async fn trade_history(
        &self,
        symbol: &str,
        limit: Option<u32>,
        from_id: Option<u64>,
    ) -> anyhow::Result<Vec<FuturesTrade>> {
        let mut params = format!("symbol={}", symbol.to_uppercase());
        if let Some(l) = limit {
            params.push_str(&format!("&limit={}", l.min(1000)));
        }
        if let Some(fid) = from_id {
            params.push_str(&format!("&fromId={}", fid));
        }
        let url = self.signed_url("/fapi/v1/userTrades", &params);
        let resp = self.http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures UserTrades error: {}", body))
        }
    }

    /// GET /fapi/v2/userTrades — Historial de trades con posición information
    pub async fn trade_history_v2(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<Vec<FuturesTrade>> {
        let mut params = format!("symbol={}", symbol.to_uppercase());
        if let Some(l) = limit {
            params.push_str(&format!("&limit={}", l.min(1000)));
        }
        let url = self.signed_url("/fapi/v2/userTrades", &params);
        let resp = self.http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures UserTrades v2 error: {}", body))
        }
    }

    // ─── Listen Key (User Data Stream) ─────────────────────────────────────

    /// POST /fapi/v1/listenKey — Obtener listen key para WS de usuario
    pub async fn create_listen_key(&self) -> anyhow::Result<String> {
        let url = format!("{}/fapi/v1/listenKey", BASE_URL);
        let resp = self.http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if resp.status().is_success() {
            let lk: ListenKeyResponse = resp.json().await?;
            Ok(lk.listen_key)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures ListenKey error: {}", body))
        }
    }

    /// PUT /fapi/v1/listenKey — Extender listen key (cada 30 min)
    pub async fn keepalive_listen_key(&self, listen_key: &str) -> anyhow::Result<()> {
        let url = format!("{}/fapi/v1/listenKey", BASE_URL);
        let params = format!("listenKey={}", listen_key);
        let resp = self.http
            .put(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(params)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures ListenKey keepalive error: {}", body))
        }
    }

    /// DELETE /fapi/v1/listenKey — Cerrar stream de usuario
    pub async fn close_listen_key(&self, listen_key: &str) -> anyhow::Result<()> {
        let url = format!("{}/fapi/v1/listenKey", BASE_URL);
        let params = format!("listenKey={}", listen_key);
        let resp = self.http
            .delete(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(params)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures ListenKey close error: {}", body))
        }
    }

    // ─── Market Data (público, sin firma) ───────────────────────────────────

    /// GET /fapi/v1/exchangeInfo
    pub async fn exchange_info(&self) -> anyhow::Result<ExchangeInfo> {
        let url = format!("{}/fapi/v1/exchangeInfo", BASE_URL);
        let resp = self.http.get(&url).send().await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures ExchangeInfo error: {}", body))
        }
    }

    /// GET /fapi/v1/premiumIndex — Mark price + funding rate
    pub async fn premium_index(&self, symbol: Option<&str>) -> anyhow::Result<Vec<FundingRate>> {
        let url = match symbol {
            Some(sym) => format!("{}/fapi/v1/premiumIndex?symbol={}", BASE_URL, sym.to_uppercase()),
            None => format!("{}/fapi/v1/premiumIndex", BASE_URL),
        };
        let resp = self.http.get(&url).send().await?;
        if resp.status().is_success() {
            if symbol.is_some() {
                let item: FundingRate = resp.json().await?;
                Ok(vec![item])
            } else {
                Ok(resp.json().await?)
            }
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures PremiumIndex error: {}", body))
        }
    }

    /// GET /fapi/v1/openInterest
    pub async fn open_interest(&self, symbol: &str) -> anyhow::Result<OpenInterest> {
        let url = format!("{}/fapi/v1/openInterest?symbol={}", BASE_URL, symbol.to_uppercase());
        let resp = self.http.get(&url).send().await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures OpenInterest error: {}", body))
        }
    }

    /// GET /fapi/v1/topLongShortAccountRatio — Top trader long/short ratio
    pub async fn top_trader_ratio(
        &self,
        symbol: &str,
        period: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<Vec<TopTraderRatio>> {
        let mut params = format!("symbol={}&period={}", symbol.to_uppercase(), period);
        if let Some(l) = limit {
            params.push_str(&format!("&limit={}", l.min(500)));
        }
        let url = format!("{}/fapi/v1/topLongShortAccountRatio?{}", BASE_URL, params);
        let resp = self.http.get(&url).send().await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Futures TopTraderRatio error: {}", body))
        }
    }

    // ─── Convenience: snapshot de mercado para el JUEZ ──────────────────────

    /// Arma un MarketSnapshot con funding, OI, long/short ratio y mark price
    pub async fn market_snapshot(&self, symbol: &str) -> anyhow::Result<MarketSnapshot> {
        let sym = symbol.to_uppercase();

        // Obtener funding/mark (público)
        let fundings = self.premium_index(Some(&sym)).await.unwrap_or_default();
        let (mark_price, funding_rate) = if let Some(f) = fundings.first() {
            (
                f.mark_price.parse::<f64>().unwrap_or(0.0),
                f.funding_rate.parse::<f64>().unwrap_or(0.0),
            )
        } else {
            (0.0, 0.0)
        };

        // Open interest
        let oi = self.open_interest(&sym).await
            .map(|o| o.open_interest.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        // Long/short ratio último
        let ratio = self.top_trader_ratio(&sym, "5m", Some(1)).await
            .map(|r| r.first().and_then(|t| t.long_short_ratio.parse::<f64>().ok()).unwrap_or(1.0))
            .unwrap_or(1.0);

        Ok(MarketSnapshot {
            symbol: sym,
            mark_price,
            funding_rate,
            open_interest: oi,
            long_short_ratio: ratio,
            bid: 0.0,   // se llenan desde WS
            ask: 0.0,
            bid_qty: 0.0,
            ask_qty: 0.0,
            cvd: 0.0,
            timestamp: Utc::now().timestamp_millis(),
        })
    }
}
