// ============================================================================
// nexus_futures::futures_loop — Orquestador de trading autónomo en Futures
// ============================================================================
// Pipeline: WS aggTrade → CvdTracker → trigger por umbral → LLM (JSON)
// → FuturesClient.place_order (entrada + SL/TP nativos como órdenes separadas)
// ============================================================================

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};
use chrono::Utc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::backend::FuturesBackend;
use super::types::{
    FuturesOrderRequest, OrderSide, OrderType, PositionSide, TimeInForce, WorkingType,
};
use super::ws::{CvdTracker, FuturesMarketWs, MarketStream};

// ═══════════════════════════════════════════════════════════════════════════════
// Decisión del LLM
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTradingDecision {
    /// "LONG" | "SHORT" | "HOLD" | "CLOSE"
    pub accion: String,
    #[serde(default)]
    pub qty: Option<f64>,
    #[serde(default)]
    pub leverage: Option<u32>,
    #[serde(default)]
    pub sl: Option<f64>,
    #[serde(default)]
    pub tp: Option<f64>,
    #[serde(default)]
    pub razon: String,
}

impl LlmTradingDecision {
    /// Parsea JSON estricto tolerando fences markdown (```json ... ```)
    pub fn parse(texto: &str) -> Result<Self, String> {
        let limpio = texto
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        serde_json::from_str(limpio).map_err(|e| format!("LLM JSON inválido: {e} → {texto}"))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Backend LLM: Ollama local o OpenRouter
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum LlmBackend {
    /// http://localhost:11434 + modelo local
    Ollama { url: String, model: String },
    /// https://openrouter.ai/api/v1/chat/completions
    OpenRouter { api_key: String, model: String },
}

impl Default for LlmBackend {
    fn default() -> Self {
        LlmBackend::Ollama {
            url: "http://localhost:11434".to_string(),
            model: "qwen3:8b".to_string(),
        }
    }
}

impl LlmBackend {
    /// Autodetecta backend: si existe OPENROUTER_API_KEY usa Gemini 2.5 Flash,
    /// si no, intenta Ollama local.
    pub fn autodetect() -> Self {
        if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
            if !key.trim().is_empty() {
                return LlmBackend::OpenRouter {
                    api_key: key,
                    model: "google/gemini-2.5-flash".to_string(),
                };
            }
        }
        LlmBackend::default()
    }

    /// Envía un prompt y devuelve la respuesta textual del modelo.
    pub async fn generar(&self, prompt: &str) -> Result<String, String> {
        let http = reqwest::Client::new();
        match self {
            LlmBackend::Ollama { url, model } => {
                let body = serde_json::json!({
                    "model": model,
                    "prompt": prompt,
                    "stream": false,
                    "options": { "temperature": 0.1 }
                });
                let resp = http
                    .post(format!("{}/api/generate", url))
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Ollama HTTP error: {e}"))?;
                let json: Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("Ollama JSON error: {e}"))?;
                json.get("response")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Ollama: sin campo 'response'".to_string())
            }
            LlmBackend::OpenRouter { api_key, model } => {
                let body = serde_json::json!({
                    "model": model,
                    "messages": [
                        {"role": "system", "content": "Eres un trader cuantitativo. Responde SOLO JSON."},
                        {"role": "user", "content": prompt}
                    ],
                    "temperature": 0.1
                });
                let resp = http
                    .post("https://openrouter.ai/api/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("OpenRouter HTTP error: {e}"))?;
                let json: Value = resp
                    .json()
                    .await
                    .map_err(|e| format!("OpenRouter JSON error: {e}"))?;
                json.pointer("/choices/0/message/content")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("OpenRouter: sin contenido → {json}"))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Orquestador
// ═══════════════════════════════════════════════════════════════════════════════

pub struct FuturesOrchestrator {
    /// Backend intercambiable: API real o simulador paper trading
    pub client: Arc<dyn FuturesBackend>,
    pub symbol: String,
    pub delta_threshold: f64,
    pub check_interval_secs: u64,
    /// Cantidad por defecto si el LLM no la especifica
    pub qty_default: f64,
    /// Leverage por defecto (1-125)
    pub leverage_default: u32,
    pub llm: LlmBackend,
    /// 📡 Telemetría compartida publicada en vivo para el dashboard
    pub telemetry: Arc<Mutex<Value>>,
    /// true → paper trading (sin WS real de Binance, se usan los precios del feed local)
    pub simulado: bool,
}

impl FuturesOrchestrator {
    pub fn new(
        client: Arc<dyn FuturesBackend>,
        symbol: String,
        delta_threshold: f64,
        check_interval_secs: u64,
    ) -> Self {
        Self::with_telemetry(client, symbol, delta_threshold, check_interval_secs, Arc::new(Mutex::new(Value::Null)))
    }

    /// Constructor con estado de telemetría compartido (dashboard)
    pub fn with_telemetry(
        client: Arc<dyn FuturesBackend>,
        symbol: String,
        delta_threshold: f64,
        check_interval_secs: u64,
        telemetry: Arc<Mutex<Value>>,
    ) -> Self {
        Self {
            client,
            symbol,
            delta_threshold,
            check_interval_secs,
            qty_default: 0.001,
            leverage_default: 5,
            llm: LlmBackend::autodetect(),
            telemetry,
            simulado: false,
        }
    }

    /// Construye el orquestador a partir de un backend concreto (real o sim).
    pub fn con_backend<T: FuturesBackend + 'static>(
        client: std::sync::Arc<T>,
        symbol: String,
        delta_threshold: f64,
        check_interval_secs: u64,
    ) -> Self {
        Self::new(
            client as std::sync::Arc<dyn FuturesBackend>,
            symbol,
            delta_threshold,
            check_interval_secs,
        )
    }

    /// Marca el orquestador como paper trading (feed local, sin WS de Binance).
    pub fn modo_simulado(mut self, simulado: bool) -> Self {
        self.simulado = simulado;
        self
    }

    /// Inicia el bucle de trading autónomo:
    /// 1) WS aggTrade alimenta CvdTracker en segundo plano
    /// 2) Cada check_interval_secs evalúa el delta acumulado
    /// 3) Si cruza umbral → snapshot + LLM → orden con SL/TP nativos
    pub async fn run(&self) {
        println!("[FUTURES LOOP] Iniciando orquestador para {} (umbral CVD={})", self.symbol, self.delta_threshold);

        let (tx, mut rx) = mpsc::unbounded_channel::<Value>();
        let shutdown = Arc::new(AtomicBool::new(false));
        let cvd = Arc::new(Mutex::new(CvdTracker::new()));

        // Publicar telemetría inicial para el dashboard
        {
            let mut tel = self.telemetry.lock().await;
            *tel = serde_json::json!({
                "status": "running",
                "symbol": self.symbol,
                "umbral_cvd": self.delta_threshold,
                "intervalo_seg": self.check_interval_secs,
                "qty_default": self.qty_default,
                "leverage_default": self.leverage_default,
                "llm_backend": self.llm_label(),
                "cvd_delta": 0.0,
                "ultima_decision": "—",
                "ultima_razon": "Esperando acumulación de CVD...",
                "ultima_accion": "idle",
                "ts": Utc::now().timestamp_millis(),
            });
        }

        // ── Tarea 1: consumir stream y alimentar CvdTracker ──
        let cvd_feed = Arc::clone(&cvd);
        let _feed_task = tokio::spawn(async move {
            while let Some(val) = rx.recv().await {
                // Solo procesamos trades agresivos (aggTrade)
                if val.get("e").and_then(|v| v.as_str()) == Some("aggTrade") {
                    cvd_feed.lock().await.process_agg_trade(&val);
                }
            }
        });

        // ── Tarea 2: conectar WS de mercado (auto-reconnect) ──
        // En modo simulado el feed lo alimenta procesar_tick_mercado del portal,
        // así que aquí no abrimos WS real.
        let ws_symbol = self.symbol.clone();
        let ws_tx = tx;
        let ws_shutdown = Arc::clone(&shutdown);
        let simulado = self.simulado;
        let _ws_task = tokio::spawn(async move {
            if simulado {
                // En paper trading el CvdTracker se alimenta desde el snapshot
                // de precios del portal; dejamos el canal abierto por si se
                // quisiera inyectar ticks manualmente en el futuro.
                let _ = ws_shutdown.load(Ordering::Relaxed);
            } else {
                FuturesMarketWs::subscribe(
                    &ws_symbol,
                    vec![
                        MarketStream::AggTrade,
                        MarketStream::MarkPrice,
                        MarketStream::BookTicker,
                    ],
                    ws_tx,
                    ws_shutdown,
                )
                .await;
            }
        });

        // ── Loop principal: evaluación y disparo ──
        loop {
            sleep(Duration::from_secs(self.check_interval_secs)).await;

            let current_cvd = cvd.lock().await.delta;
            println!("[FUTURES LOOP] CVD delta ({}): {:.4}", self.symbol, current_cvd);

            // 📡 Publicar CVD en vivo al dashboard
            {
                let mut tel = self.telemetry.lock().await;
                if tel.is_null() {
                    *tel = serde_json::json!({});
                }
                tel["status"] = serde_json::json!("running");
                tel["symbol"] = serde_json::json!(self.symbol);
                tel["umbral_cvd"] = serde_json::json!(self.delta_threshold);
                tel["intervalo_seg"] = serde_json::json!(self.check_interval_secs);
                tel["qty_default"] = serde_json::json!(self.qty_default);
                tel["leverage_default"] = serde_json::json!(self.leverage_default);
                tel["llm_backend"] = serde_json::json!(self.llm_label());
                tel["cvd_delta"] = serde_json::json!(current_cvd);
                tel["ts"] = serde_json::json!(Utc::now().timestamp_millis());
            }

            // En paper trading no hay CVD (no hay WS real): decidimos siempre que
            // el simulador ya tenga un mark price alimentado por el feed local.
            let disparar = if self.simulado {
                match self.client.market_snapshot(&self.symbol).await {
                    Ok(s) => s.mark_price > 0.0,
                    Err(_) => false,
                }
            } else {
                current_cvd.abs() >= self.delta_threshold
            };

            if disparar {
                println!(
                    "[FUTURES TRIGGER] Disparo (simulado={}, CVD={:.4}, umbral={}). Consultando LLM...",
                    self.simulado, current_cvd, self.delta_threshold
                );
                match self.evaluar_y_ejecutar(current_cvd).await {
                    Ok(()) => {
                        cvd.lock().await.reset();
                    }
                    Err(e) => {
                        eprintln!("[FUTURES LOOP ERROR] Ciclo de decisión fallido: {e}");
                    }
                }
            }
        }

        #[allow(unreachable_code)]
        {
            shutdown.store(true, Ordering::Relaxed);
        }
    }

    /// Recopila contexto de mercado, consulta LLM y ejecuta entrada + SL/TP.
    async fn evaluar_y_ejecutar(&self, cvd_delta: f64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Snapshot de mercado (funding, OI, long/short ratio, mark price)
        let snapshot = self.client.market_snapshot(&self.symbol).await?;
        let posiciones = self.client.positions(Some(&self.symbol)).await?;
        let pos_abierta = posiciones
            .iter()
            .find(|p| p.position_amt.parse::<f64>().unwrap_or(0.0) != 0.0)
            .map(|p| format!("{} {}@{}", p.position_side, p.position_amt, p.entry_price))
            .unwrap_or_else(|| "ninguna".to_string());

        let prompt = format!(
            "Analiza este snapshot de Binance Futures y decide. Contexto:\n\
            - Símbolo: {}\n\
            - Mark price: {:.2}\n\
            - CVD delta acumulado: {:.4}\n\
            - Funding rate: {:.6}\n\
            - Open interest: {:.0}\n\
            - Long/Short ratio top traders: {:.3}\n\
            - Posición abierta actual: {}\n\n\
            Reglas de riesgo: leverage <= 10, nunca sobre-apalancar. Si ya hay posición, prefiere HOLD o CLOSE.\n\
            Responde EXCLUSIVAMENTE JSON estricto sin texto extra:\n\
            {{\"accion\":\"LONG|SHORT|HOLD|CLOSE\",\"qty\":{},\"leverage\":{},\"sl\":<precio_stop_loss>,\"tp\":<precio_take_profit>,\"razon\":\"motivo breve\"}}",
            self.symbol,
            snapshot.mark_price,
            cvd_delta,
            snapshot.funding_rate,
            snapshot.open_interest,
            snapshot.long_short_ratio,
            pos_abierta,
            self.qty_default,
            self.leverage_default,
        );

        println!("[FUTURES LLM] Consultando {} ...", self.llm_label());
        let respuesta = self.llm.generar(&prompt).await?;
        println!("[FUTURES LLM] Respuesta: {}", respuesta);

        let decision = LlmTradingDecision::parse(&respuesta)?;
        println!("[FUTURES DECISION] {} | {}", decision.accion, decision.razon);

        // 📡 Publicar la decisión del LLM en la telemetría compartida
        {
            let mut tel = self.telemetry.lock().await;
            if tel.is_null() {
                *tel = serde_json::json!({});
            }
            tel["status"] = serde_json::json!("running");
            tel["symbol"] = serde_json::json!(self.symbol);
            tel["cvd_delta"] = serde_json::json!(cvd_delta);
            tel["llm_backend"] = serde_json::json!(self.llm_label());
            tel["ultima_decision"] = serde_json::json!(decision.accion.to_uppercase());
            tel["ultima_razon"] = serde_json::json!(decision.razon);
            tel["ultima_qty"] = serde_json::json!(decision.qty.unwrap_or(self.qty_default));
            tel["ultimo_leverage"] = serde_json::json!(decision.leverage.unwrap_or(self.leverage_default).clamp(1, 125));
            tel["ultimo_sl"] = decision.sl.map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null);
            tel["ultimo_tp"] = decision.tp.map(|v| serde_json::json!(v)).unwrap_or(serde_json::Value::Null);
            tel["ultima_accion"] = serde_json::json!(match decision.accion.to_uppercase().as_str() {
                "LONG" => "open_long",
                "SHORT" => "open_short",
                "CLOSE" => "close",
                _ => "hold",
            });
            tel["ts"] = serde_json::json!(Utc::now().timestamp_millis());
        }

        let qty = decision.qty.unwrap_or(self.qty_default);
        let lev = decision.leverage.unwrap_or(self.leverage_default).clamp(1, 125);

        match decision.accion.to_uppercase().as_str() {
            "LONG" => {
                let _ = self.client.set_leverage(&self.symbol, lev).await;
                self.abrir_posicion(OrderSide::Buy, PositionSide::Long, qty, decision.sl, decision.tp).await?;
            }
            "SHORT" => {
                let _ = self.client.set_leverage(&self.symbol, lev).await;
                self.abrir_posicion(OrderSide::Sell, PositionSide::Short, qty, decision.sl, decision.tp).await?;
            }
            "CLOSE" => {
                self.cerrar_posicion().await?;
            }
            _ => {
                println!("[FUTURES EXEC] HOLD: sin órdenes.");
            }
        }
        Ok(())
    }

    /// Abre posición MARKET y coloca SL + TP nativos como órdenes separadas.
    async fn abrir_posicion(
        &self,
        side: OrderSide,
        ps: PositionSide,
        qty: f64,
        sl: Option<f64>,
        tp: Option<f64>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. Entrada a mercado
        let entrada = FuturesOrderRequest {
            symbol: self.symbol.clone(),
            side: side.clone(),
            position_side: Some(ps.clone()),
            order_type: OrderType::Market,
            quantity: qty,
            price: None,
            stop_price: None,
            trailing_delta: None,
            reduce_only: Some(false),
            post_only: None,
            close_position: None,
            time_in_force: None,
            working_type: Some(WorkingType::MarkPrice),
            new_client_order_id: None,
            price_precision: None,
            quantity_precision: None,
        };
        let resp = self.client.place_order(&entrada).await?;
        println!("[FUTURES EXEC] Entrada {:?} ok → OrderID {}", ps, resp.order_id);

        // El lado de cierre es el opuesto al de entrada
        let close_side = match side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        };

        // 2. Stop loss nativo (STOP_MARKET, reduceOnly)
        if let Some(sl_price) = sl {
            let sl_req = FuturesOrderRequest {
                symbol: self.symbol.clone(),
                side: close_side.clone(),
                position_side: Some(ps.clone()),
                order_type: OrderType::StopMarket,
                quantity: qty,
                price: None,
                stop_price: Some(sl_price),
                trailing_delta: None,
                reduce_only: Some(true),
                post_only: None,
                close_position: None,
                time_in_force: None,
                working_type: Some(WorkingType::MarkPrice),
                new_client_order_id: None,
                price_precision: None,
                quantity_precision: None,
            };
            let sl_resp = self.client.place_order(&sl_req).await?;
            println!("[FUTURES EXEC] SL @ {} ok → OrderID {}", sl_price, sl_resp.order_id);
        }

        // 3. Take profit nativo (TAKE_PROFIT_MARKET, reduceOnly)
        if let Some(tp_price) = tp {
            let tp_req = FuturesOrderRequest {
                symbol: self.symbol.clone(),
                side: close_side,
                position_side: Some(ps.clone()),
                order_type: OrderType::TakeProfitMarket,
                quantity: qty,
                price: None,
                stop_price: Some(tp_price),
                trailing_delta: None,
                reduce_only: Some(true),
                post_only: None,
                close_position: None,
                time_in_force: None,
                working_type: Some(WorkingType::MarkPrice),
                new_client_order_id: None,
                price_precision: None,
                quantity_precision: None,
            };
            let tp_resp = self.client.place_order(&tp_req).await?;
            println!("[FUTURES EXEC] TP @ {} ok → OrderID {}", tp_price, tp_resp.order_id);
        }

        Ok(())
    }

    /// Cierra la posición completa de un símbolo (closePosition=true).
    async fn cerrar_posicion(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let posiciones = self.client.positions(Some(&self.symbol)).await?;
        for p in posiciones {
            let amt: f64 = p.position_amt.parse().unwrap_or(0.0);
            if amt == 0.0 {
                continue;
            }
            // Si la posición es positiva (long), hay que vender para cerrar
            let side = if amt > 0.0 { OrderSide::Sell } else { OrderSide::Buy };
            let ps = PositionSide::Long; // closePosition con LONG cierra el lado long

            // En hedge mode LONG/SHORT son independientes; usamos la side detectada.
            let req = FuturesOrderRequest {
                symbol: self.symbol.clone(),
                side: side.clone(),
                position_side: Some(if side == OrderSide::Buy { PositionSide::Short } else { ps }),
                order_type: OrderType::Market,
                quantity: amt.abs(),
                price: None,
                stop_price: None,
                trailing_delta: None,
                reduce_only: Some(true),
                post_only: None,
                close_position: Some(true),
                time_in_force: None,
                working_type: Some(WorkingType::MarkPrice),
                new_client_order_id: None,
                price_precision: None,
                quantity_precision: None,
            };
            let resp = self.client.place_order(&req).await?;
            println!("[FUTURES EXEC] Cierre {} ok → OrderID {}", p.symbol, resp.order_id);
        }
        Ok(())
    }

    fn llm_label(&self) -> &'static str {
        match self.llm {
            LlmBackend::Ollama { .. } => "Ollama local",
            LlmBackend::OpenRouter { .. } => "OpenRouter (Gemini 2.5 Flash)",
        }
    }
}
