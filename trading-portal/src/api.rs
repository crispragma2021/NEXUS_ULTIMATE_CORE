use axum::{extract::*, response::*, Json};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, warn, error};
use crate::state::*;
use crate::error::AppError;
use crate::nexus_futures;
use crate::prediccion;
use axum::extract::ws::{WebSocketUpgrade, WebSocket, Message as AxumMessage};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use anyhow::{anyhow, Result};
type HmacSha256 = Hmac<Sha256>;

pub fn load_binance_keys() -> (Option<String>, Option<String>) {
    let mut api_key = None;
    let mut secret_key = None;
    if let Ok(content) = std::fs::read_to_string("/home/soberano/NEXUS_ULTIMATE_CORE/.env") {
        for line in content.lines() {
            let line_trimmed = line.trim();
            if line_trimmed.starts_with("BINANCE_API_KEY=") {
                let parts: Vec<&str> = line_trimmed.split('=').collect();
                if parts.len() >= 2 {
                    api_key = Some(parts[1].trim().trim_matches('"').trim_matches('\'').to_string());
                }
            } else if line_trimmed.starts_with("BINANCE_SECRET_KEY=") {
                let parts: Vec<&str> = line_trimmed.split('=').collect();
                if parts.len() >= 2 {
                    secret_key = Some(parts[1].trim().trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
    }
    (api_key, secret_key)
}

pub fn save_binance_keys(api_key: &str, secret_key: &str) -> std::io::Result<()> {
    let env_path = "/home/soberano/NEXUS_ULTIMATE_CORE/.env";
    let content = std::fs::read_to_string(env_path).unwrap_or_default();
    
    let mut lines: Vec<String> = content.lines()
        .filter(|l| {
            let lt = l.trim();
            !lt.starts_with("BINANCE_API_KEY=") && !lt.starts_with("BINANCE_SECRET_KEY=")
        })
        .map(|s| s.to_string())
        .collect();
        
    lines.push(format!("BINANCE_API_KEY={}", api_key));
    lines.push(format!("BINANCE_SECRET_KEY={}", secret_key));
    
    std::fs::write(env_path, lines.join("\n"))
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn firmar_hmac_sha256(key: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    bytes_to_hex(&result.into_bytes())
}

#[derive(Debug, Deserialize)]
struct BinanceAccountInfo {
    balances: Vec<BinanceBalance>,
}

#[derive(Debug, Deserialize)]
struct BinanceBalance {
    asset: String,
    free: String,
    locked: String,
}

#[derive(Debug, Deserialize)]
struct BinanceOrderResponse {
    #[serde(rename = "orderId")]
    order_id: u64,
    status: String,
    price: String,
    #[serde(rename = "executedQty")]
    executed_qty: String,
}

async fn query_binance_account(api_key: &str, secret_key: &str) -> Result<Cartera> {
    let client = reqwest::Client::new();
    let timestamp = Utc::now().timestamp_millis();
    let query_string = format!("timestamp={}&recvWindow=5000", timestamp);
    
    let signature = firmar_hmac_sha256(secret_key, &query_string);
    let url = format!(
        "https://api.binance.com/api/v3/account?{}&signature={}",
        query_string, signature
    );
    
    let resp = client.get(&url)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;
        
    if resp.status().is_success() {
        let info: BinanceAccountInfo = resp.json().await?;
        let mut usd = 0.0;
        let mut btc = 0.0;
        let mut eth = 0.0;
        
        for bal in info.balances {
            let free_val: f64 = bal.free.parse().unwrap_or(0.0);
            let locked_val: f64 = bal.locked.parse().unwrap_or(0.0);
            let total = free_val + locked_val;
            
            if bal.asset == "USDT" {
                usd = total;
            } else if bal.asset == "BTC" {
                btc = total;
            } else if bal.asset == "ETH" {
                eth = total;
            }
        }
        
        Ok(Cartera { usd, nvda: btc, aapl: eth })
    } else {
        let err_text = resp.text().await.unwrap_or_default();
        Err(anyhow!("Binance Account Query Error: {}", err_text))
    }
}

async fn colocar_orden_binance(
    api_key: &str,
    secret_key: &str,
    simbolo: &str,
    lado: &str,
    tipo: &str,
    cantidad: f64,
) -> Result<BinanceOrderResponse> {
    let client = reqwest::Client::new();
    let timestamp = Utc::now().timestamp_millis();
    
    let side_formatted = match lado.to_lowercase().as_str() {
        "compra" | "buy" => "BUY",
        _ => "SELL",
    };
    
    let type_formatted = match tipo.to_lowercase().as_str() {
        "limite" | "limit" => "LIMIT",
        _ => "MARKET",
    };
    
    let query_params = format!(
        "symbol={}&side={}&type={}&quantity={}&timestamp={}&recvWindow=5000",
        simbolo.to_uppercase(),
        side_formatted,
        type_formatted,
        cantidad,
        timestamp
    );
    
    let signature = firmar_hmac_sha256(secret_key, &query_params);
    let url = "https://api.binance.com/api/v3/order";
    
    let resp = client.post(url)
        .header("X-MBX-APIKEY", api_key)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!("{}&signature={}", query_params, signature))
        .send()
        .await?;
        
    if resp.status().is_success() {
        let order_res: BinanceOrderResponse = resp.json().await?;
        Ok(order_res)
    } else {
        let err_text = resp.text().await.unwrap_or_default();
        Err(anyhow!("Binance Order Error: {}", err_text))
    }
}

// ─── WebSocket proxy a Binance ────────────────────────────────────────────────

pub async fn conectar_binance_ws(simbolo: &str, tx: tokio::sync::mpsc::UnboundedSender<String>) {
    let stream_url = format!(
        "wss://stream.binance.com:9443/ws/{}@trade/{}@depth20@100ms",
        simbolo.to_lowercase(),
        simbolo.to_lowercase()
    );

    let connect_future = tokio_tungstenite::connect_async(&stream_url);
    let mut connect_success = false;
    let mut ws_stream_opt = None;

    match tokio::time::timeout(tokio::time::Duration::from_secs(2), connect_future).await {
        Ok(Ok((ws_stream, _response))) => {
            info!("📡 [BINANCE] Conectado a stream {}", simbolo);
            connect_success = true;
            ws_stream_opt = Some(ws_stream);
        }
        Ok(Err(e)) => {
            warn!("⚠️ [BINANCE] Error de conexión a {}: {}", simbolo, e);
        }
        Err(_) => {
            warn!("⚠️ [BINANCE] Tiempo de espera agotado al conectar a {}.", simbolo);
        }
    }

    if connect_success {
        if let Some(ws_stream) = ws_stream_opt {
            let (mut _write, mut read) = ws_stream.split();
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        // El stream de depth real de Binance (@depth20@100ms) no incluye
                        // el símbolo en el payload (solo lastUpdateId/bids/asks). Lo
                        // inyectamos para que el frontend asocie cada book a su activo.
                        if text.contains("\"bids\"") || text.contains("\"asks\"") {
                            if let Ok(mut v) = serde_json::from_str::<serde_json::Value>(&text) {
                                if v.get("s").is_none() {
                                    v["s"] = serde_json::Value::String(simbolo.to_uppercase());
                                    let _ = tx.send(v.to_string());
                                } else {
                                    let _ = tx.send(text);
                                }
                            } else {
                                let _ = tx.send(text);
                            }
                        } else {
                            let _ = tx.send(text);
                        }
                    }
                    Ok(tokio_tungstenite::tungstenite::Message::Ping(_)) => {}
                    Err(e) => {
                        warn!("⚠️ [BINANCE] Error en stream {}: {}", simbolo, e);
                        break;
                    }
                    _ => {}
                }
            }
            info!("🔌 [BINANCE] Stream {} desconectado, reconectando...", simbolo);
        }
    } else {
        warn!("🚨 [BINANCE] Activando simulador OMEGA local de mercado para {}...", simbolo);
        
        // Generador simulado local de alta fidelidad
        use rand::seq::SliceRandom;
        use rand::Rng;
        
        let mut precio_base = match simbolo.to_uppercase().as_str() {
            "NVDA" => 135.0,
            "AAPL" => 240.0,
            "MSFT" => 420.0,
            "AMZN" => 190.0,
            "META" => 560.0,
            "TSLA" => 260.0,
            _ => 150.0,
        };
        
        loop {
            let (var_pct, qty, bids_list, asks_list) = {
                let mut rng = rand::thread_rng();
                // Mayor volatilidad en el simulador para que los precios se muevan de forma visible
                let var_pct = *[-0.0015, -0.0008, 0.0, 0.0008, 0.0015].choose(&mut rng).unwrap_or(&0.0);
                let qty = rng.gen_range(0.01..2.5);
                
                // Variar el bid/ask de forma que el ratio compra/venta fluctúe para generar señales
                let bid_ratio = rng.gen_range(0.970..1.030);
                let bid = precio_base * 0.999 * (if bid_ratio < 1.0 { bid_ratio } else { 1.0 });
                let ask = precio_base * 1.001 * (if bid_ratio > 1.0 { bid_ratio } else { 1.0 });
                
                let mut bids_list = Vec::new();
                let mut asks_list = Vec::new();
                for idx in 1..=12 {
                    let step = idx as f64 * (precio_base * 0.0001);
                    bids_list.push(vec![
                        (bid - step).to_string(), 
                        rng.gen_range(0.1..5.0).to_string()
                    ]);
                    asks_list.push(vec![
                        (ask + step).to_string(), 
                        rng.gen_range(0.1..5.0).to_string()
                    ]);
                }
                (var_pct, qty, bids_list, asks_list)
            }; // rng es liberado aquí antes del await
            
            precio_base = precio_base * (1.0 + var_pct);
            let bid = precio_base * 0.9997;
            let ask = precio_base * 1.0003;
            
            // Formato idéntico al Trade Event de Binance
            let t_msg = serde_json::json!({
                "stream": simbolo,
                "s": simbolo.to_uppercase(),
                "p": precio_base.to_string(),
                "q": qty.to_string(),
                "T": Utc::now().timestamp_millis(),
                "b": bid.to_string(),
                "a": ask.to_string(),
            });
            
            if tx.send(t_msg.to_string()).is_err() {
                break;
            }
            
            let ob_msg = serde_json::json!({
                "stream": simbolo,
                "s": simbolo.to_uppercase(),
                "bids": bids_list,
                "asks": asks_list,
            });
            
            if tx.send(ob_msg.to_string()).is_err() {
                break;
            }
            
            let sleep_ms = if simbolo == "btcusdt" { 500 } else { 1000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(sleep_ms)).await;
        }
    }
}

// ─── Estrategia NEXUS v3.0 (ML + multi-fuente) ─────────────────────────────

async fn analizar_nexus(tick: &TickMercado, state: &Arc<AppState>) -> Option<SenalTrading> {
    let simbolo = &tick.simbolo;
    let precio = tick.precio;

    // Obtener o crear analizador para este símbolo
    let analisis = {
        let mut analizador = state.analizador.entry(simbolo.clone())
            .or_insert_with(|| prediccion::AnalizadorCompleto::new());

        // Ejecutar análisis completo
        let resultado = analizador.analizar(simbolo, precio, tick.compra, tick.venta, tick.volumen);

        // Retroalimentación si tenemos precio anterior
        if analizador.motor_ml.historico_precios.len() >= 2 {
            let len = analizador.motor_ml.historico_precios.len();
            let anterior = analizador.motor_ml.historico_precios[len - 2];
            analizador.retroalimentar(anterior, precio);
        }

        resultado
    };

    // Convertir análisis a señal de trading
    if !analisis.listo || analisis.confianza_final < 0.35 {
        return None;
    }

    match analisis.accion_final.as_str() {
        "COMPRA" => {
            let razonamiento = format!(
                "🧠 NEXUS-ML: Predice subida (conf:{:.1}%) | Prob:{:.2} | MLSenal:{:.2} | OrderFlow:{:.2}",
                analisis.confianza_final * 100.0,
                analisis.senal_fusionada.prob_subida,
                analisis.prediccion_ml.contribuciones.regresion_logistica,
                analisis.prediccion_ml.contribuciones.red_neuronal,
            );
            Some(SenalTrading {
                simbolo: simbolo.clone(),
                accion: String::from("compra"),
                confianza: analisis.confianza_final,
                precio_entrada: precio,
                precio_stop_loss: precio * 0.975,
                precio_take_profit: precio * 1.035,
                razonamiento,
                timestamp: Utc::now().timestamp_millis(),
            })
        }
        "VENTA" => {
            let razonamiento = format!(
                "🧠 NEXUS-ML: Predice caída (conf:{:.1}%) | Prob:{:.2} | MLSenal:{:.2} | OrderFlow:{:.2}",
                analisis.confianza_final * 100.0,
                analisis.senal_fusionada.prob_subida,
                analisis.prediccion_ml.contribuciones.regresion_logistica,
                analisis.prediccion_ml.contribuciones.red_neuronal,
            );
            Some(SenalTrading {
                simbolo: simbolo.clone(),
                accion: String::from("venta"),
                confianza: analisis.confianza_final,
                precio_entrada: precio,
                precio_stop_loss: precio * 1.025,
                precio_take_profit: precio * 0.965,
                razonamiento,
                timestamp: Utc::now().timestamp_millis(),
            })
        }
        _ => None,
    }
}

/// 📈 Registra una operación ejecutada y aplica el límite soberano configurado.
async fn registrar_operacion(state: &Arc<AppState>) -> bool {
    let max = *state.max_operaciones.write();
    let mut ops = state.operaciones_realizadas.write();
    *ops += 1;
    let alcanzado = *ops >= max;
    drop(ops);

    if alcanzado {
        *state.modo_auto.write() = false;
        let mut pensamientos = state.pensamientos.write();
        let msg = format!(
            "⏹️ [LÍMITE ALCANZADO] NEXUS ejecutó {} operaciones. Auto-trading desactivado.",
            max
        );
        warn!("{}", msg);
        pensamientos.push(msg);
    }
    alcanzado
}

async fn ejecutar_orden_automatica(state: Arc<AppState>, senal: &SenalTrading) {
    let modo_real = *state.modo_real.write();

    // 🔒 Gobernanza: límite configurable de operaciones autónomas (1-500)
    {
        let max = *state.max_operaciones.write();
        let ops = state.operaciones_realizadas.write();
        if *ops >= max {
            drop(ops);
            *state.modo_auto.write() = false;
            let mut pensamientos = state.pensamientos.write();
            let msg = format!(
                "⏹️ [LÍMITE ALCANZADO] NEXUS completó {} operaciones. Auto-trading desactivado.",
                max
            );
            warn!("{}", msg);
            pensamientos.push(msg);
            return;
        }
    }

    if modo_real {
        let (api_key, secret_key) = load_binance_keys();

        if api_key.is_none() || secret_key.is_none() {
            let msg = String::from("🚨 [TRADING REAL FALLIDO] No se configuraron las API keys de Binance.");
            warn!("{}", msg);
            state.pensamientos.write().push(msg);
            return;
        }

        let api_key = api_key.unwrap();
        let secret_key = secret_key.unwrap();

        let qty = if senal.simbolo == "BTCUSDT" { 0.00015 } else { 0.0035 };

        let msg_inicio = format!("📡 [BINANCE REAL] Enviando orden de {} de {} {}...", senal.accion, qty, senal.simbolo);
        info!("{}", msg_inicio);
        state.pensamientos.write().push(msg_inicio);

        match colocar_orden_binance(&api_key, &secret_key, &senal.simbolo, &senal.accion, "MARKET", qty).await {
            Ok(order_res) => {
                let precio_final = order_res.price.parse::<f64>().unwrap_or(senal.precio_entrada);
                let orden = Orden {
                    id: order_res.order_id.to_string(),
                    simbolo: senal.simbolo.clone(),
                    lado: senal.accion.clone(),
                    tipo: String::from("mercado (NEXUS REAL)"),
                    cantidad: qty,
                    precio: Some(if precio_final > 0.0 { precio_final } else { senal.precio_entrada }),
                    estado: String::from("ejecutada"),
                    timestamp: Utc::now().timestamp_millis(),
                    razon_nexus: Some(senal.razonamiento.clone()),
                };
                state.ordenes.write().push(orden);
                let msg_success = format!(
                    "🟢 [REAL EJECUTADA] {} {} ejecutado. ID: {}. Motivo: {}",
                    qty, senal.simbolo, order_res.order_id, senal.razonamiento
                );
                info!("{}", msg_success);
                state.pensamientos.write().push(msg_success);
                let _ = registrar_operacion(&state).await;
            }
            Err(e) => {
                let msg_err = format!("🔴 [REAL ERROR] Falló orden Binance: {}", e);
                error!("{}", msg_err);
                state.pensamientos.write().push(msg_err);
            }
        }

        if state.pensamientos.read().len() > 50 {
            state.pensamientos.write().remove(0);
        }
        return;
    }

    let mut do_reg = false;
    let mut msg_info = None;
    let mut msg_warn = None;

    {
        let mut cartera = state.cartera.write();
        let mut ordenes = state.ordenes.write();
        let mut pensamientos = state.pensamientos.write();

        let balance_usd = cartera.usd;
        let balance_nvda = cartera.nvda;
        let balance_aapl = cartera.aapl;

        let qty = if senal.accion == "compra" {
            let monto = (balance_usd * FRACCION_POR_OPERACION).max(10.0);
            (monto / senal.precio_entrada).max(0.0)
        } else if senal.simbolo == "NVDA" {
            balance_nvda
        } else if senal.simbolo == "AAPL" {
            balance_aapl
        } else {
            let monto = (balance_usd * FRACCION_POR_OPERACION).max(10.0);
            (monto / senal.precio_entrada).max(0.0)
        };

        if senal.accion == "compra" {
            let coste = qty * senal.precio_entrada;
            if balance_usd >= coste {
                cartera.usd -= coste;
                if senal.simbolo == "NVDA" {
                    cartera.nvda += qty;
                } else if senal.simbolo == "AAPL" {
                    cartera.aapl += qty;
                }
                let orden = Orden {
                    id: format!("nexus-auto-{}", Utc::now().timestamp_millis()),
                    simbolo: senal.simbolo.clone(),
                    lado: String::from("compra"),
                    tipo: String::from("mercado (NEXUS)"),
                    cantidad: qty,
                    precio: Some(senal.precio_entrada),
                    estado: String::from("ejecutada"),
                    timestamp: Utc::now().timestamp_millis(),
                    razon_nexus: Some(senal.razonamiento.clone()),
                };
                ordenes.push(orden);
                let msg = format!(
                    "🟢 [COMPRA EJECUTADA] {} {} a ${:.2}. Coste: ${:.2}. Motivo: {}",
                    qty, senal.simbolo, senal.precio_entrada, coste, senal.razonamiento
                );
                msg_info = Some(msg.clone());
                pensamientos.push(msg);
                do_reg = true;
            } else {
                let msg = format!(
                    "⚠️ [FONDOS INSUFICIENTES] NEXUS intentó comprar {} {} pero solo hay ${:.2} USD disponibles.", 
                    qty, senal.simbolo, balance_usd
                );
                msg_warn = Some(msg.clone());
                pensamientos.push(msg);
            }
        } else if senal.accion == "venta" {
            let disp = if senal.simbolo == "NVDA" { balance_nvda } else if senal.simbolo == "AAPL" { balance_aapl } else { qty };
            if disp >= qty {
                if senal.simbolo == "NVDA" {
                    cartera.nvda -= qty;
                } else if senal.simbolo == "AAPL" {
                    cartera.aapl -= qty;
                }
                let ganancias = qty * senal.precio_entrada;
                cartera.usd += ganancias;
                let orden = Orden {
                    id: format!("nexus-auto-{}", Utc::now().timestamp_millis()),
                    simbolo: senal.simbolo.clone(),
                    lado: String::from("venta"),
                    tipo: String::from("mercado (NEXUS)"),
                    cantidad: qty,
                    precio: Some(senal.precio_entrada),
                    estado: String::from("ejecutada"),
                    timestamp: Utc::now().timestamp_millis(),
                    razon_nexus: Some(senal.razonamiento.clone()),
                };
                ordenes.push(orden);
                let msg = format!(
                    "🔴 [VENTA EJECUTADA] {} {} a ${:.2}. Retorno: ${:.2}. Motivo: {}",
                    qty, senal.simbolo, senal.precio_entrada, ganancias, senal.razonamiento
                );
                msg_info = Some(msg.clone());
                pensamientos.push(msg);
                do_reg = true;
            } else {
                let msg = format!(
                    "⚠️ [SIN ACTIVOS] NEXUS intentó vender {} {} pero no hay saldo suficiente en cartera.", 
                    qty, senal.simbolo
                );
                msg_warn = Some(msg.clone());
                pensamientos.push(msg);
            }
        }

        if pensamientos.len() > 50 {
            pensamientos.remove(0);
        }
    }

    if let Some(m) = msg_info {
        info!("{}", m);
    }
    if let Some(m) = msg_warn {
        warn!("{}", m);
    }
    if do_reg {
        let _ = registrar_operacion(&state).await;
    }
}

/// GET /api/health — Health check
pub async fn api_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "estado": "operativo",
        "servicio": "nexus-tr",
        "version": "2.0.0",
        "timestamp": Utc::now().timestamp_millis()
    }))
}

pub async fn api_precio(state: axum::extract::State<AppStateArc>) -> Json<Vec<TickMercado>> {
    let ticks: Vec<TickMercado> = state.inner.precio_actual.iter().map(|r| r.value().clone()).collect();
    Json(ticks)
}

pub async fn api_ordenes(state: axum::extract::State<AppStateArc>) -> Json<Vec<Orden>> {
    let ordenes = state.inner.ordenes.write();
    Json(ordenes.clone())
}

pub async fn api_crear_orden(
    state: axum::extract::State<AppStateArc>,
    Json(req): Json<OrdenRequest>,
) -> Json<serde_json::Value> {
    let mut ordenes = state.inner.ordenes.write();
    let orden = Orden {
        id: format!("nexus-{}-{}", Utc::now().timestamp_millis(), ordenes.len()),
        simbolo: req.simbolo,
        lado: req.lado,
        tipo: req.tipo,
        cantidad: req.cantidad,
        precio: req.precio,
        estado: String::from("abierta"),
        timestamp: Utc::now().timestamp_millis(),
        razon_nexus: None,
    };
    ordenes.push(orden.clone());
    info!("📝 [ORDEN] Creada {} {} de {} {}", orden.lado, orden.cantidad, orden.simbolo, orden.id);
    Json(serde_json::json!({ "status": "ok", "orden": orden }))
}

pub async fn api_senales(state: axum::extract::State<AppStateArc>) -> Json<Vec<SenalTrading>> {
    let senales = state.inner.senales.write();
    Json(senales.clone())
}

pub async fn api_auto_trading(state: axum::extract::State<AppStateArc>) -> Json<serde_json::Value> {
    let mut modo = state.inner.modo_auto.write();
    *modo = !*modo;
    info!("🤖 [AUTO-TRADING] Modo automático: {}", if *modo { "ACTIVADO" } else { "DESACTIVADO" });
    Json(serde_json::json!({
        "status": "ok",
        "auto_trading": *modo,
        "mensaje": if *modo {
            "NEXUS está operando por ti. Confía en tu mano derecha."
        } else {
            "NEXUS en modo manual. Tú decides."
        }
    }))
}

/// GET /api/auto-trading/estado — Solo lectura (NO invierte el modo).
/// Usado por el watchdog y por el frontend para conocer el estado sin togglear.
pub async fn api_auto_trading_estado(state: axum::extract::State<AppStateArc>) -> Json<serde_json::Value> {
    let modo = *state.inner.modo_auto.write();
    let realizadas = *state.inner.operaciones_realizadas.write();
    let max = *state.inner.max_operaciones.write();
    Json(serde_json::json!({
        "status": "ok",
        "auto_trading": modo,
        "operaciones_realizadas": realizadas,
        "max_operaciones": max,
        "mensaje": if modo {
            "NEXUS está operando por ti. Confía en tu mano derecha."
        } else {
            "NEXUS en modo manual. Tú decides."
        }
    }))
}

/// GET/POST /api/limite-operaciones — Configura el límite de operaciones autónomas (rango 1-500)
pub async fn api_limite_operaciones(
    state: axum::extract::State<AppStateArc>,
    body: Option<axum::Json<serde_json::Value>>,
) -> Json<serde_json::Value> {
    if let Some(axum::Json(payload)) = body {
        if let Some(valor) = payload.get("max_operaciones").and_then(|v| v.as_u64()) {
            let nuevo = valor.clamp(LIMITE_MIN as u64, LIMITE_MAX as u64) as u32;
            let mut max = state.inner.max_operaciones.write();
            *max = nuevo;
            let msg = format!(
                "🎯 [GOBERNANZA] Límite de operaciones configurado a {} (rango permitido {}–{}).",
                nuevo, LIMITE_MIN, LIMITE_MAX
            );
            info!("{}", msg);
            let mut pensamientos = state.inner.pensamientos.write();
            pensamientos.push(msg);
            let realizadas = *state.inner.operaciones_realizadas.write();
            return Json(serde_json::json!({
                "status": "ok",
                "max_operaciones": nuevo,
                "operaciones_realizadas": realizadas,
                "mensaje": format!("Límite establecido en {} operaciones.", nuevo)
            }));
        }
        return Json(serde_json::json!({
            "status": "error",
            "mensaje": "Campo 'max_operaciones' (entero 1-500) requerido."
        }));
    }

    let max = *state.inner.max_operaciones.write();
    let realizadas = *state.inner.operaciones_realizadas.write();
    Json(serde_json::json!({
        "status": "ok",
        "max_operaciones": max,
        "operaciones_realizadas": realizadas,
        "min": LIMITE_MIN,
        "max": LIMITE_MAX,
    }))
}

pub async fn api_configurar_exchange(
    Json(req): Json<ApiKeyRequest>,
) -> Json<serde_json::Value> {
    info!("🔑 [EXCHANGE] Configurando {} con API key ...{}", req.exchange,
          &req.api_key[req.api_key.len().saturating_sub(8)..]);
    
    if req.exchange.to_lowercase() == "binance" {
        if let Err(e) = save_binance_keys(&req.api_key, &req.secret_key) {
            return Json(serde_json::json!({
                "status": "error",
                "mensaje": format!("Error al guardar las claves en .env: {}", e)
            }));
        }
        Json(serde_json::json!({
            "status": "ok",
            "exchange": "Binance",
            "mensaje": "Credenciales de Binance guardadas localmente en .env con éxito."
        }))
    } else {
        Json(serde_json::json!({
            "status": "error",
            "mensaje": format!("Exchange {} no soportado. Usa 'Binance'.", req.exchange)
        }))
    }
}

pub async fn api_cartera(state: axum::extract::State<AppStateArc>) -> Json<Cartera> {
    let modo_real = *state.inner.modo_real.write();
    if modo_real {
        let (api_key, secret_key) = load_binance_keys();
        if let (Some(api), Some(sec)) = (api_key, secret_key) {
            match query_binance_account(&api, &sec).await {
                Ok(real_cartera) => {
                    // Actualizar el estado en memoria para telemetría
                    let mut mem_cartera = state.inner.cartera.write();
                    *mem_cartera = real_cartera.clone();
                    return Json(real_cartera);
                }
                Err(e) => {
                    warn!("⚠️ [BINANCE REAL] Error al consultar balance, usando mock: {}", e);
                }
            }
        }
    }
    
    let cartera = state.inner.cartera.write();
    Json(cartera.clone())
}

#[derive(Debug, Deserialize)]
pub struct EstablecerBalanceRequest {
    usd: f64,
}

pub async fn api_establecer_balance(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<EstablecerBalanceRequest>,
) -> Json<serde_json::Value> {
    let mut cartera = state.inner.cartera.write();
    cartera.usd = body.usd;
    info!("🪙 [BALANCE MOCK] Establecido manualmente a: ${:.2} USD", cartera.usd);
    Json(serde_json::json!({
        "status": "ok",
        "usd": cartera.usd
    }))
}

pub async fn api_real_status(state: axum::extract::State<AppStateArc>) -> Json<serde_json::Value> {
    let modo_real = *state.inner.modo_real.write();
    let (api, secret) = load_binance_keys();
    Json(serde_json::json!({
        "status": "ok",
        "modo_real": modo_real,
        "keys_configured": api.is_some() && secret.is_some()
    }))
}

pub async fn api_modo_real(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut modo = state.inner.modo_real.write();
    if let Some(target) = body["modo_real"].as_bool() {
        *modo = target;
    } else {
        *modo = !*modo;
    }
    
    info!("💱 [MODO TRADING] Cambiado a modo real: {}", *modo);
    Json(serde_json::json!({
        "status": "ok",
        "modo_real": *modo,
        "mensaje": if *modo {
            "Operando en MODO REAL en la red principal de Binance."
        } else {
            "Operando en MODO SIMULADO con fondos ficticios."
        }
    }))
}

pub async fn api_pensamientos(state: axum::extract::State<AppStateArc>) -> Json<Vec<String>> {
    let pensamientos = state.inner.pensamientos.write();
    Json(pensamientos.clone())
}

/// GET /api/prediccion/reporte — Estado del motor de predicción ML
pub async fn api_prediccion_reporte(state: axum::extract::State<AppStateArc>) -> Json<serde_json::Value> {
    let mut reportes = std::collections::HashMap::new();
    for r in state.inner.analizador.iter() {
        reportes.insert(r.key().clone(), r.value().reporte());
    }
    Json(serde_json::json!({
        "status": "ok",
        "analizadores_activos": reportes.len(),
        "reportes": reportes,
        "timestamp": Utc::now().timestamp_millis(),
    }))
}

/// GET /api/prediccion/analizar — Fuerza un análisis manual de un símbolo
pub async fn api_prediccion_analizar(
    state: axum::extract::State<AppStateArc>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let simbolo = params.get("simbolo").map(|s| s.as_str()).unwrap_or("NVDA");
    let precio_actual = state.inner.precio_actual.get(simbolo).map(|r| r.value().clone());
    
    match precio_actual {
        Some(tick) => {
            let resultado = {
                let mut analizador = state.inner.analizador.entry(simbolo.to_string())
                    .or_insert_with(|| prediccion::AnalizadorCompleto::new());
                analizador.analizar(simbolo, tick.precio, tick.compra, tick.venta, tick.volumen)
            };
            Json(serde_json::json!({
                "status": "ok",
                "analisis": resultado,
            }))
        }
        None => Json(serde_json::json!({
            "status": "error",
            "mensaje": format!("No hay datos de precio para {}", simbolo),
        })),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🚀 API DE FUTURES — Binance Futures USDT-M
// ═══════════════════════════════════════════════════════════════════════════════

/// Devuelve el backend de futuros activo: simulador si está activo,
/// sino el cliente real (si está configurado).
async fn futures_backend_activo(
    state: &Arc<AppState>,
) -> Option<Arc<dyn nexus_futures::FuturesBackend>> {
    if *state.futures_sim_activo.read() {
        return state
            .futures_sim
            .read()
            .as_ref()
            .map(|s| s.clone() as Arc<dyn nexus_futures::FuturesBackend>);
    }
    state
        .futures_client
        .read()
        .as_ref()
        .map(|c| c.clone() as Arc<dyn nexus_futures::FuturesBackend>)
}

/// POST /api/futures/configurar — Inicializa el cliente de futures (modo real)
pub async fn api_futures_configurar(
    state: axum::extract::State<AppStateArc>,
) -> Json<serde_json::Value> {
    let (api_key, secret_key) = load_binance_keys();
    match (api_key, secret_key) {
        (Some(api), Some(sec)) => {
            let client = Arc::new(nexus_futures::FuturesClient::new(api, sec));
            let mut fc = state.inner.futures_client.write();
            *fc = Some(client);
            // Configurar la API real desactiva la simulación
            *state.inner.futures_sim_activo.write() = false;
            *state.inner.futures_modo.write() = true;
            info!("🚀 [FUTURES] Cliente inicializado en fapi.binance.com (modo real)");
            Json(serde_json::json!({
                "status": "ok",
                "mensaje": "Futures client conectado a fapi.binance.com. Largo/corto, leverage, SL/TP listos."
            }))
        }
        _ => Json(serde_json::json!({
            "status": "error",
            "mensaje": "API keys no configuradas. Usa POST /api/configurar-exchange primero."
        })),
    }
}

/// POST /api/futures/orden — Colocar orden (MARKET, LIMIT, STOP, TP, TRAILING)
pub async fn api_futures_orden(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c,
        None => {
            return Json(serde_json::json!({
                "status": "error",
                "mensaje": "Futures no configurado. Usa POST /api/futures/configurar o activa la simulación con /api/futures/simulacion."
            }));
        }
    };

    let symbol = body["symbol"].as_str().unwrap_or("BTCUSDT").to_string();
    let side_str = body["side"].as_str().unwrap_or("BUY");
    let side = nexus_futures::OrderSide::from_spanish(side_str);
    let position_side = body["positionSide"].as_str().map(|ps| {
        match ps.to_uppercase().as_str() {
            "LONG" => nexus_futures::PositionSide::Long,
            "SHORT" => nexus_futures::PositionSide::Short,
            _ => nexus_futures::PositionSide::Both,
        }
    });
    let order_type_str = body["type"].as_str().unwrap_or("MARKET");
    let order_type = match order_type_str.to_uppercase().as_str() {
        "LIMIT" => nexus_futures::OrderType::Limit,
        "STOP" | "STOP_MARKET" => nexus_futures::OrderType::StopMarket,
        "TAKE_PROFIT" | "TAKE_PROFIT_MARKET" => nexus_futures::OrderType::TakeProfitMarket,
        "TRAILING_STOP_MARKET" | "TRAILING" => nexus_futures::OrderType::TrailingStopMarket,
        _ => nexus_futures::OrderType::Market,
    };
    let quantity = body["quantity"].as_f64().unwrap_or(0.001);
    let price = body["price"].as_f64();
    let stop_price = body["stopPrice"].as_f64();
    let trailing_delta = body["trailingDelta"].as_f64();
    let reduce_only = body["reduceOnly"].as_bool();
    let close_position = body["closePosition"].as_bool().unwrap_or(false);
    let client_order_id = body["clientOrderId"].as_str().map(|s| s.to_string());
    let time_in_force = body["timeInForce"].as_str().map(|tif| {
        match tif.to_uppercase().as_str() {
            "IOC" => nexus_futures::TimeInForce::Ioc,
            "FOK" => nexus_futures::TimeInForce::Fok,
            "GTX" => nexus_futures::TimeInForce::Gtx,
            _ => nexus_futures::TimeInForce::Gtc,
        }
    });
    let working_type = body["workingType"].as_str().map(|wt| {
        match wt.to_uppercase().as_str() {
            "CONTRACT_PRICE" => nexus_futures::WorkingType::ContractPrice,
            _ => nexus_futures::WorkingType::MarkPrice,
        }
    });

    let req = nexus_futures::FuturesOrderRequest {
        symbol,
        side,
        position_side,
        order_type,
        quantity,
        price,
        stop_price,
        trailing_delta,
        reduce_only,
        post_only: body["postOnly"].as_bool(),
        close_position: Some(close_position),
        time_in_force,
        working_type,
        new_client_order_id: client_order_id,
        price_precision: body["pricePrecision"].as_u64().map(|v| v as u32),
        quantity_precision: body["quantityPrecision"].as_u64().map(|v| v as u32),
    };

    info!("📡 [FUTURES] Enviando orden: {:?}", req);

    match fc.place_order(&req).await {
        Ok(resp) => {
            info!("✅ [FUTURES] Orden colocada: {} {} — ID: {}", resp.side, resp.symbol, resp.order_id);
            {
                let mut ordenes = state.inner.ordenes.write();
                ordenes.push(Orden {
                    id: resp.order_id.to_string(),
                    simbolo: resp.symbol.clone(),
                    lado: resp.side.clone(),
                    tipo: format!("futures/{}", resp.order_type),
                    cantidad: resp.orig_qty.parse().unwrap_or(0.0),
                    precio: resp.price.parse().ok(),
                    estado: resp.status.clone(),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    razon_nexus: Some(format!("Futures: {} {}", resp.order_type, resp.position_side)),
                });
            }
            Json(serde_json::json!({ "status": "ok", "orden": resp }))
        }
        Err(e) => {
            error!("🔴 [FUTURES] Error: {}", e);
            Json(serde_json::json!({ "status": "error", "mensaje": e.to_string() }))
        }
    }
}

/// POST /api/futures/orden/close — Cerrar posición completa
pub async fn api_futures_cerrar_posicion(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let symbol = body["symbol"].as_str().unwrap_or("BTCUSDT");
    let position_side = body["positionSide"].as_str().map(|ps| {
        match ps.to_uppercase().as_str() {
            "LONG" => nexus_futures::PositionSide::Long,
            "SHORT" => nexus_futures::PositionSide::Short,
            _ => nexus_futures::PositionSide::Both,
        }
    });
    let req = nexus_futures::FuturesOrderRequest {
        symbol: symbol.to_string(),
        side: nexus_futures::OrderSide::Sell,
        position_side, order_type: nexus_futures::OrderType::Market,
        quantity: 0.0, price: None, stop_price: None, trailing_delta: None,
        reduce_only: Some(true), post_only: None, close_position: Some(true),
        time_in_force: None, working_type: None, new_client_order_id: None,
        price_precision: None, quantity_precision: None,
    };
    info!("🔒 [FUTURES] Cerrando posición {}...", symbol);
    match fc.place_order(&req).await {
        Ok(resp) => Json(serde_json::json!({"status":"ok","orden":resp,"mensaje":format!("Posición {} cerrada.",symbol)})),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// GET /api/futures/posiciones — Posiciones abiertas
pub async fn api_futures_posiciones(
    state: axum::extract::State<AppStateArc>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let symbol = params.get("symbol").map(|s| s.as_str());
    match fc.positions(symbol).await {
        Ok(positions) => {
            let activas: Vec<_> = positions.into_iter()
                .filter(|p| p.position_amt.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
                .collect();
            Json(serde_json::json!({"status":"ok","total":activas.len(),"posiciones":activas}))
        }
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// GET /api/futures/balance — Balance futures
pub async fn api_futures_balance(
    state: axum::extract::State<AppStateArc>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    match fc.account_info().await {
        Ok(info) => Json(serde_json::json!({
            "status":"ok","total_wallet_balance":info.total_wallet_balance,
            "available_balance":info.available_balance,"total_unrealized_profit":info.total_unrealized_profit,
            "total_margin_balance":info.total_margin_balance,"total_initial_margin":info.total_initial_margin,
            "total_maint_margin":info.total_maint_margin,"max_withdraw_amount":info.max_withdraw_amount,
            "assets":info.assets,
        })),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// POST /api/futures/leverage — Leverage 1-125
pub async fn api_futures_leverage(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let symbol = body["symbol"].as_str().unwrap_or("BTCUSDT");
    let leverage = body["leverage"].as_u64().unwrap_or(1).clamp(1, 125) as u32;
    match fc.set_leverage(symbol, leverage).await {
        Ok(resp) => Json(serde_json::json!({"status":"ok","leverage":resp})),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// POST /api/futures/modo-hedge — Modo hedge (dual side positions)
pub async fn api_futures_modo_hedge(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let dual = body["dual"].as_bool().unwrap_or(true);
    match fc.set_position_mode(dual).await {
        Ok(resp) => Json(serde_json::json!({"status":"ok","modo_hedge":dual,"respuesta":resp})),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// DELETE /api/futures/ordenes — Cancelar todas las órdenes abiertas
pub async fn api_futures_cancelar_todas(
    state: axum::extract::State<AppStateArc>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let symbol = params.get("symbol").map(|s| s.as_str()).unwrap_or("BTCUSDT");
    match fc.cancel_all_orders(symbol).await {
        Ok(resp) => Json(serde_json::json!({"status":"ok","mensaje":format!("Órdenes canceladas para {}",symbol),"respuesta":resp})),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// GET /api/futures/ordenes-abiertas
pub async fn api_futures_ordenes_abiertas(
    state: axum::extract::State<AppStateArc>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let symbol = params.get("symbol").map(|s| s.as_str());
    match fc.open_orders(symbol).await {
        Ok(orders) => Json(serde_json::json!({"status":"ok","total":orders.len(),"ordenes":orders})),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// GET /api/futures/trades/:symbol — Historial de trades + PnL
pub async fn api_futures_trades(
    state: axum::extract::State<AppStateArc>,
    axum::extract::Path(symbol): axum::extract::Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    let limit = params.get("limit").and_then(|l| l.parse::<u32>().ok()).unwrap_or(50);
    match fc.trade_history_v2(&symbol, Some(limit)).await {
        Ok(trades) => {
            let pnl_total: f64 = trades.iter().filter_map(|t| t.realized_pnl.parse::<f64>().ok()).sum();
            Json(serde_json::json!({"status":"ok","total":trades.len(),"pnl_total":pnl_total,"trades":trades}))
        }
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// GET /api/futures/snapshot/:symbol — Snapshot de mercado para el JUEZ
pub async fn api_futures_snapshot(
    state: axum::extract::State<AppStateArc>,
    axum::extract::Path(symbol): axum::extract::Path<String>,
) -> Json<serde_json::Value> {
    let fc = match futures_backend_activo(&state.inner).await {
        Some(c) => c, None => return Json(serde_json::json!({"status":"error","mensaje":"Futures no configurado o simulación inactiva."})),
    };
    match fc.market_snapshot(&symbol).await {
        Ok(snapshot) => Json(serde_json::json!({"status":"ok","snapshot":snapshot})),
        Err(e) => Json(serde_json::json!({"status":"error","mensaje":e.to_string()})),
    }
}

/// POST /api/futures/modo — Activar/desactivar modo futures
pub async fn api_futures_modo(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let mut modo = state.inner.futures_modo.write();
    if let Some(target) = body["futures_modo"].as_bool() { *modo = target; } else { *modo = !*modo; }
    info!("💱 [FUTURES] Modo futures: {}", if *modo { "ACTIVADO" } else { "DESACTIVADO" });
    Json(serde_json::json!({
        "status":"ok","futures_modo":*modo,
        "mensaje": if *modo { "Futures USDT-M activos: largo/corto, leverage, SL/TP." } else { "Futures desactivados. Operando en spot." }
    }))
}

/// GET /api/futures/loop/estado — Telemetría en vivo del orquestador futures
pub async fn api_futures_loop_estado(state: axum::extract::State<AppStateArc>) -> Json<serde_json::Value> {
    let activo = *state.inner.futures_loop_activo.read();
    let tel_arc = state.inner.futures_loop_telemetry.read().clone();
    let tel = match tel_arc {
        Some(ref t) => t.lock().await.clone(),
        None => serde_json::Value::Null,
    };
    let mut base = if tel.is_object() {
        tel
    } else {
        serde_json::json!({
            "status": if activo { "starting" } else { "stopped" },
            "symbol": "BTCUSDT",
            "cvd_delta": 0.0,
            "ultima_decision": "—",
            "ultima_razon": "Orquestador no iniciado todavía.",
            "ultima_accion": "idle",
        })
    };
    base["loop_activo"] = serde_json::json!(activo);
    base["ts"] = serde_json::json!(Utc::now().timestamp_millis());
    Json(base)
}

/// GET /api/energia/estado — Cadena energética maestra (estado de cada motor)
/// Orden: OpenRouter (PRIMARIO) → DeepSeek → Groq → Vertex → Gemini AI Studio (ÚLTIMO) → Ollama Local
pub async fn api_energia_estado() -> Json<serde_json::Value> {
    let env_path = "/home/soberano/NEXUS_ULTIMATE_CORE/.env";
    let mut claves: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    if let Ok(content) = std::fs::read_to_string(env_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(eq) = line.find('=') {
                let k = line[..eq].trim().to_string();
                let v = line[eq + 1..].trim();
                let v_limpio = v.trim_matches(|c| c == '"' || c == '\'');
                claves.insert(k, !v_limpio.is_empty());
            }
        }
    }
    let key_ok = |k: &str| claves.get(k).copied().unwrap_or(false);
    let motores = vec![
        serde_json::json!({"nombre":"OpenRouter","pos":"PRIMARIO","icono":"⚡","modelo":"Gemini 2.5 Flash","configurado": key_ok("OPENROUTER_API_KEY")}),
        serde_json::json!({"nombre":"DeepSeek","pos":"RESPALDO 1","icono":"🌊","modelo":"DeepSeek R1/V3","configurado": key_ok("DEEPSEEK_API_KEY")}),
        serde_json::json!({"nombre":"Groq LPU","pos":"RESPALDO 2","icono":"🧠","modelo":"Llama 70B","configurado": key_ok("GROQ_API_KEY")}),
        serde_json::json!({"nombre":"Vertex AI","pos":"RESPALDO 3","icono":"🏔️","modelo":"Gemini Pro","configurado": key_ok("VERTEX_TOKEN")}),
        serde_json::json!({"nombre":"Gemini AI Studio","pos":"ÚLTIMO RESPALDO","icono":"🔵","modelo":"Gemini 2.5","configurado": key_ok("GEMINI_API_KEY")}),
        serde_json::json!({"nombre":"Ollama Local","pos":"MODO OFFLINE","icono":"🤖","modelo":"qwen2.5:7b","configurado": key_ok("NEXUS_LOCAL_KEY") || key_ok("OLLAMA_KEEP_ALIVE")}),
    ];
    Json(serde_json::json!({
        "status": "ok",
        "motores": motores,
        "ts": Utc::now().timestamp_millis(),
    }))
}

/// POST /api/futures/loop — Iniciar/detener el orquestador autónomo
/// Body: {"symbol":"BTCUSDT","umbral_cvd":50.0,"intervalo_seg":2,"accion":"start"|"stop"}
/// El loop escucha aggTrade por WS, alimenta CvdTracker y cuando el delta cruza
/// el umbral consulta al LLM (Ollama local o Gemini 2.5 Flash vía OpenRouter).
pub async fn api_futures_loop(
    state: axum::extract::State<AppStateArc>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let accion = body["accion"].as_str().unwrap_or("start").to_string();
    if accion == "stop" {
        *state.inner.futures_loop_activo.write() = false;
        // Marcar la telemetría como detenida
        let tel_arc = state.inner.futures_loop_telemetry.read().clone();
        if let Some(tel) = tel_arc {
            let mut guard = tel.lock().await;
            if guard.is_object() {
                guard["status"] = serde_json::json!("stopped");
                guard["ultima_razon"] = serde_json::json!("Orquestador detenido manualmente.");
            }
        }
        return Json(serde_json::json!({
            "status":"ok",
            "mensaje":"Orquestador detenido. El loop se cancelará en el próximo ciclo."
        }));
    }

    if *state.inner.futures_loop_activo.read() {
        return Json(serde_json::json!({
            "status":"error",
            "mensaje":"El orquestador ya está corriendo. Usa {\"accion\":\"stop\"} para detenerlo."
        }));
    }

    // Backend activo: simulador (si está activo) o cliente real configurado
    let sim_activo = *state.inner.futures_sim_activo.read();
    let client = match futures_backend_activo(&state.inner).await {
        Some(c) => c,
        None => {
            return Json(serde_json::json!({
                "status":"error",
                "mensaje":"Futures no inicializado. Configura la API real o activa la simulación con /api/futures/simulacion."
            }));
        }
    };

    let symbol = body["symbol"].as_str().unwrap_or("BTCUSDT").to_uppercase();
    let umbral = body["umbral_cvd"].as_f64().unwrap_or(50.0);
    let intervalo = body["intervalo_seg"].as_u64().unwrap_or(2).max(1);

    *state.inner.futures_loop_activo.write() = true;
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let flag_loop = Arc::clone(&flag);
    let flag_state = Arc::clone(&flag);
    let run_client = client.clone();
    let symbol_run = symbol.clone();

    // 📡 Telemetría compartida del orquestador para el dashboard
    let telemetry = Arc::new(tokio::sync::Mutex::new(serde_json::Value::Null));
    *state.inner.futures_loop_telemetry.write() = Some(Arc::clone(&telemetry));

    tokio::spawn(async move {
        let orquestador = nexus_futures::FuturesOrchestrator::with_telemetry(
            run_client,
            symbol_run,
            umbral,
            intervalo,
            telemetry,
        )
        .modo_simulado(sim_activo);
        orquestador.run().await;
        flag_loop.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    // Tarea de vigilancia: si el loop termina solo, limpia el flag
    let state_watch = state.inner.clone();
    tokio::spawn(async move {
        while flag_state.load(std::sync::atomic::Ordering::Relaxed) {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
        *state_watch.futures_loop_activo.write() = false;
    });

    info!("🚀 [FUTURES LOOP] Orquestador iniciado para {} (umbral CVD={}, intervalo={}s)", symbol, umbral, intervalo);
    Json(serde_json::json!({
        "status":"ok",
        "symbol":symbol,
        "umbral_cvd":umbral,
        "intervalo_seg":intervalo,
        "mensaje":"Orquestador autónomo activo: WS aggTrade → CvdTracker → LLM → orden con SL/TP nativos."
    }))
}

// ─── Simulación (paper trading sin API) ──────────────────────────────────────

/// POST /api/futures/simulacion  {"accion":"start"|"stop"|"reset"|"estado","balance":10000}
/// GET  /api/futures/simulacion  → estado del simulador
pub async fn api_futures_simulacion(
    state: axum::extract::State<AppStateArc>,
    body: Option<Json<serde_json::Value>>,
) -> Json<serde_json::Value> {
    let body = body.map(|j| j.0).unwrap_or_else(|| serde_json::json!({}));
    let accion = body["accion"].as_str().unwrap_or("estado").to_lowercase();

    match accion.as_str() {
        "start" => {
            let balance = body["balance"].as_f64().unwrap_or(10_000.0).max(100.0);
            let sim = Arc::new(nexus_futures::FuturesSimulator::new(balance));
            *state.inner.futures_sim.write() = Some(Arc::clone(&sim));
            *state.inner.futures_sim_activo.write() = true;
            *state.inner.futures_modo.write() = true;
            info!("🧪 [FUTURES SIM] Simulación iniciada con balance ${:.2}", balance);
            Json(serde_json::json!({
                "status":"ok",
                "accion":"start",
                "balance":balance,
                "activo":true,
                "mensaje":"Simulación activa. El portal opera en PAPER TRADING sin tocar tu API real."
            }))
        }
        "stop" => {
            *state.inner.futures_sim_activo.write() = false;
            info!("🧪 [FUTURES SIM] Simulación detenida. Volviendo al backend real (si existe).");
            Json(serde_json::json!({
                "status":"ok",
                "accion":"stop",
                "activo":false,
                "mensaje":"Simulación detenida. Los endpoints vuelven al cliente real si estaba configurado."
            }))
        }
        "reset" => {
            let balance = body["balance"].as_f64().unwrap_or(10_000.0).max(100.0);
            let sim_opt = state.inner.futures_sim.write().clone();
            match sim_opt {
                Some(sim) => {
                    sim.reset(balance);
                    *state.inner.futures_sim_activo.write() = true;
                    *state.inner.futures_modo.write() = true;
                    info!("🧪 [FUTURES SIM] Reiniciada con balance ${:.2}", balance);
                    Json(serde_json::json!({
                        "status":"ok",
                        "accion":"reset",
                        "balance":balance,
                        "activo":true,
                        "mensaje":"Simulación reiniciada con balance limpio."
                    }))
                }
                None => Json(serde_json::json!({
                    "status":"error",
                    "accion":"reset",
                    "mensaje":"No hay simulador inicializado. Usa {\"accion\":\"start\"}."
                })),
            }
        }
        _ => {
            let estado = match state.inner.futures_sim.write().as_ref() {
                Some(sim) => sim.estado_json(),
                None => serde_json::json!({"error":"simulador_no_inicializado"}),
            };
            let activo = *state.inner.futures_sim_activo.write();
            let mut resp = estado;
            if resp.is_object() {
                resp["activo"] = serde_json::json!(activo);
                resp["backend_actual"] = serde_json::json!(if activo { "simulador" } else { "cliente_real" });
            }
            Json(resp)
        }
    }
}

// ─── WebSocket handler ────────────────────────────────────────────────────────

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    state: axum::extract::State<AppStateArc>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| manejar_ws(socket, state.inner.clone()))
}

/// Procesa cada tick del mercado global: actualiza precio_actual y alimenta el
/// motor ML SIEMPRE (Fase B). Las órdenes solo se ejecutan en modo auto.
pub async fn procesar_tick_mercado(state: &Arc<AppState>, msg: &str) {
    let Ok(data) = serde_json::from_str::<serde_json::Value>(msg) else { return };
    let Some(precio_str) = data.get("p").and_then(|v| v.as_str()) else { return };
    let Ok(precio) = precio_str.parse::<f64>() else { return };

    let simbolo_evento = data.get("s").and_then(|v| v.as_str()).unwrap_or("NVDA").to_string();
    let tick = TickMercado {
        simbolo: simbolo_evento.clone(),
        precio,
        volumen: data.get("q").and_then(|v| v.as_str())
            .and_then(|v| v.parse().ok()).unwrap_or(0.0),
        timestamp: data.get("T").and_then(|v| v.as_i64()).unwrap_or(0),
        compra: data.get("b").and_then(|v| v.as_str())
            .and_then(|v| v.parse().ok()).unwrap_or(precio * 0.999),
        venta: data.get("a").and_then(|v| v.as_str())
            .and_then(|v| v.parse().ok()).unwrap_or(precio * 1.001),
    };

    state.precio_actual.insert(simbolo_evento.clone(), tick.clone());

    // 🧪 Alimentar el simulador de futuros (paper trading) si está activo
    // Solo símbolos USDT tienen mercado de futuros en el simulador.
    if simbolo_evento.ends_with("USDT") {
        if let Some(sim) = state.futures_sim.write().clone() {
            let trades = sim.actualizar_precio(&simbolo_evento, precio);
            if !trades.is_empty() {
                info!(
                    "🧪 [FUTURES SIM] {} ejecutados: {} trade(s) @ ${:.2} (pnl_total=${:.2})",
                    simbolo_evento,
                    trades.len(),
                    precio,
                    sim.estado_json()["pnl_realizado_total"].as_f64().unwrap_or(0.0)
                );
            }
        }
    }

    // Alimentar el analizador ML SIEMPRE (independiente del modo auto)
    if let Some(senal) = analizar_nexus(&tick, state).await {
        let auto = *state.modo_auto.write();
        {
            let mut senales = state.senales.write();
            senales.push(senal.clone());
        }
        if auto {
            ejecutar_orden_automatica(state.clone(), &senal).await;
        }
    }
}

async fn manejar_ws(mut socket: WebSocket, state: Arc<AppState>) {
    info!("🔌 [WS] Cliente frontend conectado");
    let (mut sender, mut receiver) = socket.split();

    // Suscribirse al feed global de mercado (24/7, ya alimenta el motor ML)
    let mut broadcast_rx = state.mercado_broadcast.subscribe();

    let forward_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if sender.send(AxumMessage::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(AxumMessage::Text(text)) => info!("📩 [WS] Comando frontend: {}", text),
            Ok(AxumMessage::Close(_)) => break,
            Err(e) => { warn!("⚠️ [WS] Error: {}", e); break; }
            _ => {}
        }
    }

    forward_task.abort();
    info!("🔌 [WS] Cliente frontend desconectado");
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTRYPOINT
// ═══════════════════════════════════════════════════════════════════════════════

