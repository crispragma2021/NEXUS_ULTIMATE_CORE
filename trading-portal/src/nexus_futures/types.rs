// ============================================================================
// nexus_futures::types — DTOs para Binance Futures API (fapi.binance.com)
// ============================================================================
use serde::{Deserialize, Serialize};

// ─── Enums de trading ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn from_spanish(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "compra" | "buy" | "long" => OrderSide::Buy,
            _ => OrderSide::Sell,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Limit,
    Market,
    Stop,
    #[serde(rename = "STOP_MARKET")]
    StopMarket,
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfit,
    #[serde(rename = "TAKE_PROFIT_MARKET")]
    TakeProfitMarket,
    #[serde(rename = "TRAILING_STOP_MARKET")]
    TrailingStopMarket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Gtc, // Good-Til-Cancelled
    Ioc, // Immediate-Or-Cancel
    Fok, // Fill-Or-Kill
    Gtx, // Good-Til-Crossing (Post-Only)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    Both,
    Long,
    Short,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkingType {
    #[serde(rename = "CONTRACT_PRICE")]
    ContractPrice,
    #[serde(rename = "MARK_PRICE")]
    MarkPrice,
}

// ─── Request: colocar orden en futures ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderRequest {
    /// Símbolo (BTCUSDT, ETHUSDT, etc.)
    pub symbol: String,
    /// BUY | SELL
    pub side: OrderSide,
    /// LONG | SHORT | BOTH (siempre LONG o SHORT en hedge mode)
    pub position_side: Option<PositionSide>,
    /// LIMIT | MARKET | STOP | STOP_MARKET | TAKE_PROFIT | TAKE_PROFIT_MARKET | TRAILING_STOP_MARKET
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Cantidad en contratos (no en USD). 1 contrato = cantidad base del símbolo.
    pub quantity: f64,
    /// Precio límite (requerido para LIMIT/STOP/TAKE_PROFIT)
    pub price: Option<f64>,
    /// Precio de activación (requerido para STOP/STOP_MARKET/TAKE_PROFIT/TAKE_PROFIT_MARKET)
    pub stop_price: Option<f64>,
    /// Distancia de trailing stop en porcentaje (0.5 = 0.5%). Solo para TRAILING_STOP_MARKET.
    pub trailing_delta: Option<f64>,
    /// Si true, la orden solo reduce posición existente (no puede abrir nueva).
    pub reduce_only: Option<bool>,
    /// Si true, la orden se coloca como maker (Post-Only).
    pub post_only: Option<bool>,
    /// ClosePosition=true cierra toda la posición al precio de mercado. Ignora quantity/price/side.
    pub close_position: Option<bool>,
    /// GTC | IOC | FOK | GTX
    pub time_in_force: Option<TimeInForce>,
    /// CONTRACT_PRICE | MARK_PRICE (para stop/TP). Default MARK_PRICE.
    pub working_type: Option<WorkingType>,
    /// ID externo para tracking (opcional, máx 36 chars)
    pub new_client_order_id: Option<String>,
    /// Número de decimales del precio para redondeo de cantidad (default: usar exchange)
    pub price_precision: Option<u32>,
    /// Número de decimales de cantidad (default: usar exchange)
    pub quantity_precision: Option<u32>,
}

// ─── Response: orden colocada ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: String,
    pub price: String,
    #[serde(rename = "avgPrice")]
    pub avg_price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cumQuote")]
    pub cum_quote: String,
    #[serde(rename = "timeInForce")]
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "reduceOnly")]
    pub reduce_only: bool,
    #[serde(rename = "closePosition")]
    pub close_position: bool,
    pub side: String,
    #[serde(rename = "positionSide")]
    pub position_side: String,
    #[serde(rename = "stopPrice")]
    pub stop_price: String,
    #[serde(rename = "workingType")]
    pub working_type: String,
    #[serde(rename = "priceProtect")]
    pub price_protect: bool,
    #[serde(rename = "origType")]
    pub orig_type: String,
    #[serde(rename = "activatePrice")]
    pub activate_price: Option<String>,
    #[serde(rename = "priceRate")]
    pub price_rate: Option<String>,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "workingTime")]
    pub working_time: u64,
}

// ─── Response: posición abierta ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPosition {
    pub symbol: String,
    #[serde(rename = "initialMargin")]
    pub initial_margin: String,
    #[serde(rename = "maintMargin")]
    pub maint_margin: String,
    #[serde(rename = "unrealizedProfit")]
    pub unrealized_profit: String,
    #[serde(rename = "positionInitialMargin")]
    pub position_initial_margin: String,
    #[serde(rename = "openOrderInitialMargin")]
    pub open_order_initial_margin: String,
    pub leverage: String,
    #[serde(rename = "entryPrice")]
    pub entry_price: String,
    #[serde(rename = "maxNotional")]
    pub max_notional: String,
    #[serde(rename = "positionSide")]
    pub position_side: String,
    #[serde(rename = "positionAmt")]
    pub position_amt: String,
    #[serde(rename = "notional")]
    pub notional: String,
    #[serde(rename = "isolatedWallet")]
    pub isolated_wallet: String,
    #[serde(rename = "markPrice")]
    pub mark_price: String,
    #[serde(rename = "liquidationPrice")]
    pub liquidation_price: String,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "breakEvenPrice")]
    pub break_even_price: String,
    /// Margen aislado o cruzado
    #[serde(rename = "isolated")]
    pub isolated: bool,
    /// ROE porcentual ya calculado por Binance
    #[serde(rename = "percentage")]
    pub percentage: Option<f64>,
}

// ─── Response: balance de futuros ────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct FuturesAccountInfo {
    #[serde(rename = "feeTier")]
    pub fee_tier: u32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "totalInitialMargin")]
    pub total_initial_margin: String,
    #[serde(rename = "totalMaintMargin")]
    pub total_maint_margin: String,
    #[serde(rename = "totalWalletBalance")]
    pub total_wallet_balance: String,
    #[serde(rename = "totalUnrealizedProfit")]
    pub total_unrealized_profit: String,
    #[serde(rename = "totalMarginBalance")]
    pub total_margin_balance: String,
    #[serde(rename = "totalPositionInitialMargin")]
    pub total_position_initial_margin: String,
    #[serde(rename = "totalOpenOrderInitialMargin")]
    pub total_open_order_initial_margin: String,
    #[serde(rename = "totalCrossWalletBalance")]
    pub total_cross_wallet_balance: String,
    #[serde(rename = "totalCrossUnPnl")]
    pub total_cross_unpnl: String,
    #[serde(rename = "availableBalance")]
    pub available_balance: String,
    #[serde(rename = "maxWithdrawAmount")]
    pub max_withdraw_amount: String,
    pub assets: Vec<FuturesAsset>,
    pub positions: Vec<FuturesPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesAsset {
    pub asset: String,
    #[serde(rename = "walletBalance")]
    pub wallet_balance: String,
    #[serde(rename = "unrealizedProfit")]
    pub unrealized_profit: String,
    #[serde(rename = "marginBalance")]
    pub margin_balance: String,
    #[serde(rename = "maintMargin")]
    pub maint_margin: String,
    #[serde(rename = "initialMargin")]
    pub initial_margin: String,
    #[serde(rename = "positionInitialMargin")]
    pub position_initial_margin: String,
    #[serde(rename = "openOrderInitialMargin")]
    pub open_order_initial_margin: String,
    #[serde(rename = "crossWalletBalance")]
    pub cross_wallet_balance: String,
    #[serde(rename = "crossUnPnl")]
    pub cross_unpnl: String,
    #[serde(rename = "availableBalance")]
    pub available_balance: String,
    #[serde(rename = "maxWithdrawAmount")]
    pub max_withdraw_amount: String,
    #[serde(rename = "marginAvailable")]
    pub margin_available: bool,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
}

// ─── Response: leverage config ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageResponse {
    pub symbol: String,
    pub leverage: u32,
    #[serde(rename = "maxNotionalValue")]
    pub max_notional_value: String,
}

// ─── Response: trade history ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesTrade {
    pub symbol: String,
    pub id: u64,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "side")]
    pub side: String,
    pub price: String,
    pub qty: String,
    #[serde(rename = "realizedPnl")]
    pub realized_pnl: String,
    pub margin: String,
    pub commission: String,
    #[serde(rename = "commissionAsset")]
    pub commission_asset: String,
    pub time: u64,
    pub position_side: String,
    pub buyer: bool,
    pub maker: bool,
}

// ─── Response: exchange info (filters) ───────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SymbolInfo {
    pub symbol: String,
    #[serde(rename = "pair")]
    pub pair: String,
    #[serde(rename = "contractType")]
    pub contract_type: String,
    #[serde(rename = "deliveryDate")]
    pub delivery_date: String,
    #[serde(rename = "onboardDate")]
    pub onboard_date: u64,
    pub status: String,
    pub filters: Vec<serde_json::Value>,
    #[serde(rename = "orderType")]
    pub order_types: Vec<String>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Vec<String>,
    pub quote_asset: String,
    pub base_asset: String,
    pub price_precision: u32,
    pub quantity_precision: u32,
    #[serde(rename = "underlyingType")]
    pub underlying_type: String,
    #[serde(rename = "underlyingSubType")]
    pub underlying_sub_type: Vec<String>,
    #[serde(rename = "settlePlan")]
    pub settle_plan: u32,
    /// Factor de trigger protect (0.05 = 5%)
    #[serde(rename = "triggerProtect")]
    pub trigger_protect: String,
}

// ─── Response: listen key ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

// ─── Response: funding rate ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct FundingRate {
    pub symbol: String,
    #[serde(rename = "fundingRate")]
    pub funding_rate: String,
    #[serde(rename = "fundingTime")]
    pub funding_time: u64,
    #[serde(rename = "markPrice")]
    pub mark_price: String,
}

// ─── Response: open interest ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct OpenInterest {
    pub symbol: String,
    #[serde(rename = "openInterest")]
    pub open_interest: String,
    pub time: u64,
}

// ─── Response: top trader long/short ratio ───────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct TopTraderRatio {
    #[serde(rename = "longAccount")]
    pub long_account: String,
    #[serde(rename = "shortAccount")]
    pub short_account: String,
    #[serde(rename = "longShortRatio")]
    pub long_short_ratio: String,
    pub timestamp: u64,
}

// ─── Resumen unificado para el Orquestador/JUEZ ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub symbol: String,
    pub mark_price: f64,
    pub funding_rate: f64,
    pub open_interest: f64,
    pub long_short_ratio: f64,
    pub bid: f64,
    pub ask: f64,
    pub bid_qty: f64,
    pub ask_qty: f64,
    pub cvd: f64, // Cumulative Volume Delta (calculado por nosotros)
    pub timestamp: i64,
}
