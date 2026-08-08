// ============================================================================
// nexus_futures::simulacion — PAPER TRADING de Binance Futures USDT-M
// ============================================================================
// Simulador completo de futuros SIN API (sin fapi.binance.com):
//   - Balance virtual USDT (default $10,000)
//   - Posiciones LONG/SHORT con leverage (1-125)
//   - Órdenes MARKET/LIMIT/STOP/TP con SL/TP nativos evaluados contra el feed
//   - PnL realizado + no realizado, comisiones, trades, historial
//   - Liquidación forzada si el precio cruza el nivel de quiebra
//
// Reutiliza los DTOs de types.rs (FuturesAccountInfo, FuturesPosition,
// FuturesOrderResponse, FuturesTrade...) para que los endpoints REST del
// portal funcionen de forma idéntica a la API real.
//
// El precio se alimenta con `actualizar_precio()` desde el feed global
// (procesar_tick_mercado en main.rs): mismo precio que ve el spot.
// ============================================================================

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Utc;

use super::types::*;

/// Comisión taker por operación (0.04% = binance futures taker)
const COMISION_TASA: f64 = 0.0004;

#[derive(Debug, Clone)]
pub struct SimPosition {
    pub symbol: String,
    pub side: PositionSide, // Long | Short
    pub quantity: f64,
    pub entry_price: f64,
    pub leverage: u32,
    pub unrealized_pnl: f64,
    pub liquidation_price: f64,
    pub mark_price: f64,
}

#[derive(Debug, Clone)]
pub struct SimOrder {
    pub order_id: u64,
    pub symbol: String,
    pub side: OrderSide,
    pub position_side: Option<PositionSide>,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub reduce_only: bool,
    pub close_position: bool,
    pub status: String, // "NEW" | "FILLED" | "CANCELED"
}

#[derive(Debug, Clone)]
pub struct SimState {
    pub wallet_balance: f64,
    pub available_balance: f64,
    pub initial_balance: f64,
    pub leverage_map: HashMap<String, u32>,
    pub positions: HashMap<String, SimPosition>,
    pub open_orders: Vec<SimOrder>,
    pub trades: Vec<FuturesTrade>,
    pub mark_prices: HashMap<String, f64>,
    pub last_order_id: u64,
    pub last_trade_id: u64,
    pub pnl_realizado_total: f64,
}

pub struct FuturesSimulator {
    inner: Mutex<SimState>,
}

impl Default for FuturesSimulator {
    fn default() -> Self {
        Self::new(10_000.0)
    }
}

impl FuturesSimulator {
    /// Crea el simulador con un balance inicial virtual en USDT.
    pub fn new(balance_inicial: f64) -> Self {
        Self {
            inner: Mutex::new(SimState {
                wallet_balance: balance_inicial,
                available_balance: balance_inicial,
                initial_balance: balance_inicial,
                leverage_map: HashMap::new(),
                positions: HashMap::new(),
                open_orders: Vec::new(),
                trades: Vec::new(),
                mark_prices: HashMap::new(),
                last_order_id: 100_000,
                last_trade_id: 0,
                pnl_realizado_total: 0.0,
            }),
        }
    }

    /// Resetea el simulador con un nuevo balance inicial.
    pub fn reset(&self, balance_inicial: f64) {
        let mut st = self.inner.lock().unwrap();
        *st = SimState {
            wallet_balance: balance_inicial,
            available_balance: balance_inicial,
            initial_balance: balance_inicial,
            leverage_map: st.leverage_map.clone(),
            positions: HashMap::new(),
            open_orders: Vec::new(),
            trades: Vec::new(),
            mark_prices: st.mark_prices.clone(),
            last_order_id: 100_000,
            last_trade_id: 0,
            pnl_realizado_total: 0.0,
        };
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Feed de precios — llamado desde el procesador de ticks del portal
    // ═════════════════════════════════════════════════════════════════════════

    /// Alimenta el simulador con el último precio del símbolo y evalúa
    /// SL/TP + órdenes límite + liquidación. Retorna los trades ejecutados
    /// por este tick (si hubo cierres).
    pub fn actualizar_precio(&self, symbol: &str, price: f64) -> Vec<FuturesTrade> {
        let mut ejecutados: Vec<FuturesTrade> = Vec::new();
        if price <= 0.0 {
            return ejecutados;
        }

        let symbol_up = symbol.to_uppercase();
        let mut st = self.inner.lock().unwrap();
        st.mark_prices.insert(symbol_up.clone(), price);

        // ── 1) Cerrar posiciones cuyo SL/TP se cruzó ──
        // (order_id, symbol, precio_cierre) — se procesan TODOS los triggers
        let mut a_cerrar: Vec<(u64, String, f64)> = Vec::new();
        for ord in st.open_orders.iter() {
            if ord.symbol != symbol_up || ord.status != "NEW" {
                continue;
            }
            let Some(sp) = ord.stop_price else { continue };
            let Some(pos) = st.positions.get(&symbol_up) else { continue };

            match ord.order_type {
                OrderType::StopMarket => {
                    // SL: LONG se cierra si precio <= sp; SHORT si precio >= sp
                    let dispara = match pos.side {
                        PositionSide::Long => price <= sp,
                        PositionSide::Short => price >= sp,
                        _ => false,
                    };
                    if dispara {
                        a_cerrar.push((ord.order_id, symbol_up.clone(), sp));
                    }
                }
                OrderType::TakeProfitMarket => {
                    let dispara = match pos.side {
                        PositionSide::Long => price >= sp,
                        PositionSide::Short => price <= sp,
                        _ => false,
                    };
                    if dispara {
                        a_cerrar.push((ord.order_id, symbol_up.clone(), sp));
                    }
                }
                _ => {}
            }
        }

        for (oid, sym, precio_cierre) in a_cerrar {
            // Marcar SOLO la orden que disparó como FILLED
            if let Some(ord) = st.open_orders.iter_mut().find(|o| o.order_id == oid && o.status == "NEW") {
                ord.status = "FILLED".to_string();
            }
            // Si aún queda posición abierta, cerrarla (evita doble cierre
            // cuando SL y TP se cruzan en el mismo tick)
            if st.positions.contains_key(&sym) {
                if let Some(trade) = self.cerrar_posicion_interno(&mut st, &sym, precio_cierre) {
                    ejecutados.push(trade);
                }
            }
        }

        // ── 2) Ejecutar órdenes LÍMITE si el precio alcanzó el límite ──
        let limite_filled: Vec<usize> = st
            .open_orders
            .iter()
            .enumerate()
            .filter(|(_, ord)| {
                ord.symbol == symbol_up && ord.status == "NEW" && ord.order_type == OrderType::Limit
            })
            .filter(|(_, ord)| {
                if let Some(p) = ord.price {
                    match ord.side {
                        OrderSide::Buy => price <= p,
                        OrderSide::Sell => price >= p,
                    }
                } else {
                    false
                }
            })
            .map(|(i, _)| i)
            .collect();

        if !limite_filled.is_empty() {
            let idx = limite_filled[0];
            let ord = st.open_orders[idx].clone();
            st.open_orders[idx].status = "FILLED".to_string();
            // Apertura de posición vía límite:
            let (sym, side, ps, qty, price_l) = (
                ord.symbol.clone(),
                ord.side.clone(),
                ord.position_side.clone(),
                ord.quantity,
                ord.price.unwrap_or(price),
            );
            let lev = st.leverage_map.get(&sym).copied().unwrap_or(5);
            self.abrir_posicion_interno(&mut st, &sym, side, ps, qty, price_l, lev);
        }

        // ── 3) Actualizar PnL flotante + liqudación de posiciones abiertas ──
        let mut liquidadas: Vec<String> = Vec::new();
        for (sym, pos) in st.positions.iter_mut() {
            pos.mark_price = price;
            pos.unrealized_pnl = pnl_float(pos.side.clone(), pos.entry_price, price, pos.quantity);
            if pos.leverage > 0 && price > 0.0 {
                pos.liquidation_price = match pos.side {
                    PositionSide::Long => pos.entry_price * (1.0 - 1.0 / pos.leverage as f64 - 0.005),
                    PositionSide::Short => pos.entry_price * (1.0 + 1.0 / pos.leverage as f64 + 0.005),
                    _ => pos.entry_price,
                };
                let liquidado = match pos.side {
                    PositionSide::Long => price <= pos.liquidation_price,
                    PositionSide::Short => price >= pos.liquidation_price,
                    _ => false,
                };
                if liquidado {
                    liquidadas.push(sym.clone());
                }
            }
        }
        for sym in liquidadas {
            if let Some(trade) = self.cerrar_posicion_interno(&mut st, &sym, price) {
                ejecutados.push(trade);
            }
        }

        // ── 4) Recalcular balances ──
        st.pnl_realizado_total = st.trades.iter().map(|t| t.realized_pnl.parse::<f64>().unwrap_or(0.0)).sum();
        let margen_usado: f64 = st
            .positions
            .values()
            .map(|p| (p.entry_price * p.quantity) / p.leverage.max(1) as f64)
            .sum();
        st.wallet_balance = st.initial_balance + st.pnl_realizado_total;
        st.available_balance = (st.wallet_balance - margen_usado).max(0.0);

        ejecutados
    }

    /// Fuerza el cierre de una posición al precio dado. Retorna el trade si cerró algo.
    fn cerrar_posicion_interno(&self, st: &mut SimState, symbol: &str, precio_cierre: f64) -> Option<FuturesTrade> {
        let sym = symbol.to_uppercase();
        let pos = st.positions.remove(&sym)?;

        let pnl = pnl_float(pos.side.clone(), pos.entry_price, precio_cierre, pos.quantity);
        let notional = precio_cierre * pos.quantity;
        let comision = notional * COMISION_TASA;
        let pnl_neto = pnl - comision;

        // Lado del trade de cierre: opuesto a la posición
        let close_side = match pos.side {
            PositionSide::Long => OrderSide::Sell,
            _ => OrderSide::Buy,
        };
        let buyer = matches!(close_side, OrderSide::Buy);

        st.last_trade_id += 1;
        let trade = FuturesTrade {
            symbol: sym.clone(),
            id: st.last_trade_id,
            order_id: st.last_order_id,
            side: serde_json::to_string(&close_side).unwrap_or_default().trim_matches('"').to_string(),
            price: precio_cierre.to_string(),
            qty: pos.quantity.to_string(),
            realized_pnl: pnl_neto.to_string(),
            margin: format!("{:.4}", (pos.entry_price * pos.quantity) / pos.leverage.max(1) as f64),
            commission: comision.to_string(),
            commission_asset: "USDT".to_string(),
            time: Utc::now().timestamp_millis() as u64,
            position_side: serde_json::to_string(&pos.side).unwrap_or_default().trim_matches('"').to_string(),
            buyer,
            maker: false,
        };
        st.trades.push(trade.clone());
        st.pnl_realizado_total += pnl_neto;
        st.last_order_id += 1;

        // Limpiar órdenes de SL/TP del símbolo
        st.open_orders.retain(|o| o.symbol != sym || o.status != "NEW");

        Some(trade)
    }

    /// Reduce (cierra parcialmente) una posición existente. Retorna el trade.
    fn reducir_posicion_interno(&self, st: &mut SimState, symbol: &str, qty: f64, precio_cierre: f64) -> Option<FuturesTrade> {
        let sym = symbol.to_uppercase();
        let pos = st.positions.get_mut(&sym)?;
        let cerrar_qty = qty.min(pos.quantity);
        if cerrar_qty <= 0.0 {
            return None;
        }

        let pnl = pnl_float(pos.side.clone(), pos.entry_price, precio_cierre, cerrar_qty);
        let notional = precio_cierre * cerrar_qty;
        let comision = notional * COMISION_TASA;
        let pnl_neto = pnl - comision;

        let close_side = match pos.side {
            PositionSide::Long => OrderSide::Sell,
            _ => OrderSide::Buy,
        };
        let buyer = matches!(close_side, OrderSide::Buy);

        st.last_trade_id += 1;
        let trade = FuturesTrade {
            symbol: sym.clone(),
            id: st.last_trade_id,
            order_id: st.last_order_id,
            side: serde_json::to_string(&close_side).unwrap_or_default().trim_matches('"').to_string(),
            price: precio_cierre.to_string(),
            qty: cerrar_qty.to_string(),
            realized_pnl: pnl_neto.to_string(),
            margin: format!("{:.4}", (pos.entry_price * cerrar_qty) / pos.leverage.max(1) as f64),
            commission: comision.to_string(),
            commission_asset: "USDT".to_string(),
            time: Utc::now().timestamp_millis() as u64,
            position_side: serde_json::to_string(&pos.side).unwrap_or_default().trim_matches('"').to_string(),
            buyer,
            maker: false,
        };
        st.trades.push(trade.clone());
        st.pnl_realizado_total += pnl_neto;
        st.last_order_id += 1;

        pos.quantity -= cerrar_qty;
        if pos.quantity.abs() < 1e-9 {
            st.positions.remove(&sym);
        }

        Some(trade)
    }

    /// Abre una posición (o suma si ya hay una del mismo lado). No valida margen.
    fn abrir_posicion_interno(
        &self,
        st: &mut SimState,
        symbol: &str,
        side: OrderSide,
        ps: Option<PositionSide>,
        qty: f64,
        price: f64,
        leverage: u32,
    ) -> u64 {
        let sym = symbol.to_uppercase();
        let lev = leverage.clamp(1, 125);
        let pos_side = match ps {
            Some(p) => p,
            None => match side {
                OrderSide::Buy => PositionSide::Long,
                OrderSide::Sell => PositionSide::Short,
            },
        };
        // En one-way (BOTH), Buy → Long, Sell → Short
        let pos_side = match pos_side {
            PositionSide::Both => match side {
                OrderSide::Buy => PositionSide::Long,
                OrderSide::Sell => PositionSide::Short,
            },
            other => other,
        };

        let notional = price * qty;
        let comision = notional * COMISION_TASA;
        let margen = notional / lev as f64;
        if st.available_balance < margen + comision {
            // Sin margen suficiente: rechazar (retorna 0 = error implícito)
            return 0;
        }

        st.last_order_id += 1;
        let order_id = st.last_order_id;

        if let Some(existente) = st.positions.get_mut(&sym) {
            // Misma dirección → sumar; dirección opuesta → cerrar/reducir
            if existente.side == pos_side {
                let qty_total = existente.quantity + qty;
                existente.entry_price = (existente.entry_price * existente.quantity + price * qty) / qty_total;
                existente.quantity = qty_total;
            } else {
                // Cierre parcial/total
                let cerrar_qty = qty.min(existente.quantity);
                let pnl = pnl_float(existente.side.clone(), existente.entry_price, price, cerrar_qty);
                let com = price * cerrar_qty * COMISION_TASA;
                st.pnl_realizado_total += pnl - com;
                st.last_trade_id += 1;
                st.trades.push(FuturesTrade {
                    symbol: sym.clone(),
                    id: st.last_trade_id,
                    order_id,
                    side: serde_json::to_string(&side).unwrap_or_default().trim_matches('"').to_string(),
                    price: price.to_string(),
                    qty: cerrar_qty.to_string(),
                    realized_pnl: (pnl - com).to_string(),
                    margin: format!("{:.4}", margen),
                    commission: com.to_string(),
                    commission_asset: "USDT".to_string(),
                    time: Utc::now().timestamp_millis() as u64,
                    position_side: serde_json::to_string(&existente.side).unwrap_or_default().trim_matches('"').to_string(),
                    buyer: matches!(side, OrderSide::Buy),
                    maker: false,
                });
                existente.quantity -= cerrar_qty;
                if existente.quantity.abs() < 1e-9 {
                    st.positions.remove(&sym);
                }
            }
        } else {
            st.positions.insert(
                sym.clone(),
                SimPosition {
                    symbol: sym.clone(),
                    side: pos_side,
                    quantity: qty,
                    entry_price: price,
                    leverage: lev,
                    unrealized_pnl: 0.0,
                    liquidation_price: 0.0,
                    mark_price: price,
                },
            );
        }

        // Actualizar balances tras apertura
        st.wallet_balance = st.initial_balance + st.pnl_realizado_total;
        let margen_usado: f64 = st
            .positions
            .values()
            .map(|p| (p.entry_price * p.quantity) / p.leverage.max(1) as f64)
            .sum();
        st.available_balance = (st.wallet_balance - margen_usado).max(0.0);

        order_id
    }

    fn leverage(&self, symbol: &str) -> u32 {
        self.inner
            .lock()
            .unwrap()
            .leverage_map
            .get(&symbol.to_uppercase())
            .copied()
            .unwrap_or(5)
    }

    // ═════════════════════════════════════════════════════════════════════════
    // API pública (misma firma que FuturesClient → intercambiable)
    // ═════════════════════════════════════════════════════════════════════════

    /// GET /fapi/v2/account — balance + posiciones del simulador
    pub async fn account_info(&self) -> anyhow::Result<FuturesAccountInfo> {
        let st = self.inner.lock().unwrap();
        let available = format!("{:.4}", st.available_balance);
        let wallet = format!("{:.4}", st.wallet_balance);
        let unreal = format!("{:.4}", st.positions.values().map(|p| p.unrealized_pnl).sum::<f64>());
        let margin_balance = format!("{:.4}", st.wallet_balance + st.positions.values().map(|p| p.unrealized_pnl).sum::<f64>());
        let pos_margin: f64 = st
            .positions
            .values()
            .map(|p| (p.entry_price * p.quantity) / p.leverage.max(1) as f64)
            .sum();

        let asset = FuturesAsset {
            asset: "USDT".to_string(),
            wallet_balance: wallet.clone(),
            unrealized_profit: unreal.clone(),
            margin_balance: margin_balance.clone(),
            maint_margin: "0.0000".to_string(),
            initial_margin: format!("{:.4}", pos_margin),
            position_initial_margin: format!("{:.4}", pos_margin),
            open_order_initial_margin: "0.0000".to_string(),
            cross_wallet_balance: wallet.clone(),
            cross_unpnl: unreal.clone(),
            available_balance: available.clone(),
            max_withdraw_amount: available.clone(),
            margin_available: true,
            update_time: Utc::now().timestamp_millis() as u64,
        };

        let positions: Vec<FuturesPosition> = st
            .positions
            .values()
            .map(|p| FuturesPosition {
                symbol: p.symbol.clone(),
                initial_margin: format!("{:.4}", (p.entry_price * p.quantity) / p.leverage.max(1) as f64),
                maint_margin: format!("{:.4}", (p.entry_price * p.quantity) / p.leverage.max(1) as f64 * 0.005),
                unrealized_profit: p.unrealized_pnl.to_string(),
                position_initial_margin: format!("{:.4}", (p.entry_price * p.quantity) / p.leverage.max(1) as f64),
                open_order_initial_margin: "0.0000".to_string(),
                leverage: p.leverage.to_string(),
                entry_price: p.entry_price.to_string(),
                max_notional: "250000.0".to_string(),
                position_side: serde_json::to_string(&p.side).unwrap_or_default().trim_matches('"').to_string(),
                position_amt: p.quantity.to_string(),
                notional: format!("{:.4}", p.mark_price * p.quantity),
                isolated_wallet: "0.0000".to_string(),
                mark_price: p.mark_price.to_string(),
                liquidation_price: p.liquidation_price.to_string(),
                update_time: Utc::now().timestamp_millis() as u64,
                break_even_price: p.entry_price.to_string(),
                isolated: false,
                percentage: Some(if (p.entry_price * p.quantity) > 0.0 {
                    p.unrealized_pnl / ((p.entry_price * p.quantity) / p.leverage.max(1) as f64) * 100.0
                } else {
                    0.0
                }),
            })
            .collect();

        Ok(FuturesAccountInfo {
            fee_tier: 1,
            can_trade: true,
            can_deposit: true,
            can_withdraw: true,
            total_initial_margin: format!("{:.4}", pos_margin),
            total_maint_margin: format!("{:.4}", pos_margin * 0.005),
            total_wallet_balance: wallet,
            total_unrealized_profit: unreal.clone(),
            total_margin_balance: margin_balance,
            total_position_initial_margin: format!("{:.4}", pos_margin),
            total_open_order_initial_margin: "0.0000".to_string(),
            total_cross_wallet_balance: format!("{:.4}", st.wallet_balance),
            total_cross_unpnl: unreal,
            available_balance: available.clone(),
            max_withdraw_amount: available,
            assets: vec![asset],
            positions,
        })
    }

    /// GET /fapi/v2/positionRisk — posiciones abiertas
    pub async fn positions(&self, symbol: Option<&str>) -> anyhow::Result<Vec<FuturesPosition>> {
        let st = self.inner.lock().unwrap();
        let mut out: Vec<FuturesPosition> = st
            .positions
            .values()
            .map(|p| FuturesPosition {
                symbol: p.symbol.clone(),
                initial_margin: format!("{:.4}", (p.entry_price * p.quantity) / p.leverage.max(1) as f64),
                maint_margin: format!("{:.4}", (p.entry_price * p.quantity) / p.leverage.max(1) as f64 * 0.005),
                unrealized_profit: p.unrealized_pnl.to_string(),
                position_initial_margin: format!("{:.4}", (p.entry_price * p.quantity) / p.leverage.max(1) as f64),
                open_order_initial_margin: "0.0000".to_string(),
                leverage: p.leverage.to_string(),
                entry_price: p.entry_price.to_string(),
                max_notional: "250000.0".to_string(),
                position_side: serde_json::to_string(&p.side).unwrap_or_default().trim_matches('"').to_string(),
                position_amt: p.quantity.to_string(),
                notional: format!("{:.4}", p.mark_price * p.quantity),
                isolated_wallet: "0.0000".to_string(),
                mark_price: p.mark_price.to_string(),
                liquidation_price: p.liquidation_price.to_string(),
                update_time: Utc::now().timestamp_millis() as u64,
                break_even_price: p.entry_price.to_string(),
                isolated: false,
                percentage: Some(if (p.entry_price * p.quantity) > 0.0 {
                    p.unrealized_pnl / ((p.entry_price * p.quantity) / p.leverage.max(1) as f64) * 100.0
                } else {
                    0.0
                }),
            })
            .collect();
        if let Some(sym) = symbol {
            out.retain(|p| p.symbol == sym.to_uppercase());
        }
        Ok(out)
    }

    /// POST /fapi/v1/leverage — cambio de leverage (1-125)
    pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> anyhow::Result<LeverageResponse> {
        let lev = leverage.clamp(1, 125);
        {
            let mut st = self.inner.lock().unwrap();
            st.leverage_map.insert(symbol.to_uppercase(), lev);
            if let Some(pos) = st.positions.get_mut(&symbol.to_uppercase()) {
                pos.leverage = lev;
            }
        }
        Ok(LeverageResponse {
            symbol: symbol.to_uppercase(),
            leverage: lev,
            max_notional_value: "250000.0".to_string(),
        })
    }

    /// POST /fapi/v1/marginType — ISOLATED | CROSSED (simulado)
    pub async fn set_margin_type(&self, _symbol: &str, _margin_type: &str) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({"code": 200, "msg": "simulated margin type"}))
    }

    /// POST /fapi/v1/positionSide/dual — modo hedge (simulado)
    pub async fn set_position_mode(&self, _dual: bool) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::json!({"code": 200, "msg": "simulated position mode"}))
    }

    /// POST /fapi/v1/order — colocar orden simulada
    pub async fn place_order(&self, req: &FuturesOrderRequest) -> anyhow::Result<FuturesOrderResponse> {
        let sym = req.symbol.to_uppercase();
        let precio_actual = self.inner.lock().unwrap().mark_prices.get(&sym).copied().unwrap_or(0.0);
        let mut st = self.inner.lock().unwrap();
        let lev = st.leverage_map.get(&sym).copied().unwrap_or(5);

        // Cierre completo (closePosition=true)
        if req.close_position.unwrap_or(false) {
            let precio_cierre = precio_actual;
            st.last_order_id += 1;
            let oid = st.last_order_id;
            drop(st);
            let mut st2 = self.inner.lock().unwrap();
            let trade = self.cerrar_posicion_interno(&mut st2, &sym, precio_cierre);
            drop(st2);
            return Ok(resp_simulada(oid, &sym, req, precio_cierre, if trade.is_some() { "FILLED" } else { "NEW" }));
        }

        match req.order_type {
            OrderType::Market => {
                if req.reduce_only.unwrap_or(false) {
                    // reduceOnly: cerrar la cantidad indicada contra la posición
                    let qty_cierre = req.quantity.abs();
                    drop(st);
                    let mut st2 = self.inner.lock().unwrap();
                    let trade = self.reducir_posicion_interno(&mut st2, &sym, qty_cierre, precio_actual);
                    st2.last_order_id += 1;
                    let oid = st2.last_order_id;
                    drop(st2);
                    return Ok(resp_simulada(
                        oid,
                        &sym,
                        req,
                        precio_actual,
                        if trade.is_some() { "FILLED" } else { "NEW" },
                    ));
                }
                // Apertura
                drop(st);
                let oid = self.abrir_posicion_interno(
                    &mut self.inner.lock().unwrap(),
                    &sym,
                    req.side.clone(),
                    req.position_side.clone(),
                    req.quantity,
                    precio_actual,
                    lev,
                );
                if oid == 0 {
                    return Ok(resp_simulada(0, &sym, req, precio_actual, "EXPIRED"));
                }
                return Ok(resp_simulada(oid, &sym, req, precio_actual, "FILLED"));
            }
            OrderType::Limit => {
                // Si el límite ya se cruza, ejecuta ahora; si no, queda en book
                let price = req.price.unwrap_or(precio_actual);
                let cruza = match req.side {
                    OrderSide::Buy => precio_actual <= price,
                    OrderSide::Sell => precio_actual >= price,
                };
                if cruza && !req.reduce_only.unwrap_or(false) {
                    drop(st);
                    let oid = self.abrir_posicion_interno(
                        &mut self.inner.lock().unwrap(),
                        &sym,
                        req.side.clone(),
                        req.position_side.clone(),
                        req.quantity,
                        price,
                        lev,
                    );
                    if oid == 0 {
                        return Ok(resp_simulada(0, &sym, req, price, "EXPIRED"));
                    }
                    return Ok(resp_simulada(oid, &sym, req, price, "FILLED"));
                }
                // Pendiente en el book
                st.last_order_id += 1;
                let oid = st.last_order_id;
                st.open_orders.push(SimOrder {
                    order_id: oid,
                    symbol: sym.clone(),
                    side: req.side.clone(),
                    position_side: req.position_side.clone(),
                    order_type: OrderType::Limit,
                    quantity: req.quantity,
                    price: Some(price),
                    stop_price: None,
                    reduce_only: req.reduce_only.unwrap_or(false),
                    close_position: false,
                    status: "NEW".to_string(),
                });
                return Ok(resp_simulada(oid, &sym, req, price, "NEW"));
            }
            OrderType::StopMarket | OrderType::TakeProfitMarket => {
                // SL/TP como órdenes pendientes que se evalúan en actualizar_precio
                st.last_order_id += 1;
                let oid = st.last_order_id;
                st.open_orders.push(SimOrder {
                    order_id: oid,
                    symbol: sym.clone(),
                    side: req.side.clone(),
                    position_side: req.position_side.clone(),
                    order_type: req.order_type.clone(),
                    quantity: req.quantity,
                    price: None,
                    stop_price: req.stop_price,
                    reduce_only: req.reduce_only.unwrap_or(true),
                    close_position: req.close_position.unwrap_or(false),
                    status: "NEW".to_string(),
                });
                return Ok(resp_simulada(oid, &sym, req, req.stop_price.unwrap_or(precio_actual), "NEW"));
            }
            _ => {
                // Otros tipos: se registran como NEW sin ejecutar
                st.last_order_id += 1;
                let oid = st.last_order_id;
                return Ok(resp_simulada(oid, &sym, req, precio_actual, "NEW"));
            }
        }
    }

    /// DELETE /fapi/v1/allOpenOrders — cancelar todas las órdenes de un símbolo
    pub async fn cancel_all_orders(&self, symbol: &str) -> anyhow::Result<serde_json::Value> {
        let mut st = self.inner.lock().unwrap();
        let before = st.open_orders.len();
        st.open_orders.retain(|o| o.symbol != symbol.to_uppercase() || o.status != "NEW");
        Ok(serde_json::json!({"code": 200, "msg": format!("canceladas {} órdenes", before - st.open_orders.len())}))
    }

    /// GET /fapi/v1/openOrders — órdenes abiertas
    pub async fn open_orders(&self, symbol: Option<&str>) -> anyhow::Result<Vec<FuturesOrderResponse>> {
        let st = self.inner.lock().unwrap();
        let mut out: Vec<FuturesOrderResponse> = st
            .open_orders
            .iter()
            .filter(|o| o.status == "NEW")
            .filter(|o| symbol.map(|s| o.symbol == s.to_uppercase()).unwrap_or(true))
            .map(|o| FuturesOrderResponse {
                order_id: o.order_id,
                symbol: o.symbol.clone(),
                status: "NEW".to_string(),
                client_order_id: format!("sim-{}", o.order_id),
                price: o.price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()),
                avg_price: "0".to_string(),
                orig_qty: o.quantity.to_string(),
                executed_qty: "0".to_string(),
                cum_quote: "0".to_string(),
                time_in_force: "GTC".to_string(),
                order_type: serde_json::to_string(&o.order_type).unwrap_or_default().trim_matches('"').to_string(),
                reduce_only: o.reduce_only,
                close_position: o.close_position,
                side: serde_json::to_string(&o.side).unwrap_or_default().trim_matches('"').to_string(),
                position_side: o.position_side.as_ref().map(|ps| serde_json::to_string(ps).unwrap_or_default().trim_matches('"').to_string()).unwrap_or_else(|| "BOTH".to_string()),
                stop_price: o.stop_price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()),
                working_type: "MARK_PRICE".to_string(),
                price_protect: false,
                orig_type: serde_json::to_string(&o.order_type).unwrap_or_default().trim_matches('"').to_string(),
                activate_price: None,
                price_rate: None,
                update_time: Utc::now().timestamp_millis() as u64,
                working_time: Utc::now().timestamp_millis() as u64,
            })
            .collect();
        out.sort_by_key(|o| o.order_id);
        Ok(out)
    }

    /// GET /fapi/v2/userTrades — historial de trades simulados
    pub async fn trade_history_v2(&self, symbol: &str, limit: Option<u32>) -> anyhow::Result<Vec<FuturesTrade>> {
        let st = self.inner.lock().unwrap();
        let mut out: Vec<FuturesTrade> = st
            .trades
            .iter()
            .filter(|t| t.symbol == symbol.to_uppercase())
            .cloned()
            .collect();
        out.reverse();
        if let Some(l) = limit {
            out.truncate(l as usize);
        }
        Ok(out)
    }

    /// GET /fapi/v1/premiumIndex etc. — snapshot de mercado desde el feed local
    pub async fn market_snapshot(&self, symbol: &str) -> anyhow::Result<MarketSnapshot> {
        let sym = symbol.to_uppercase();
        let st = self.inner.lock().unwrap();
        let mark = st.mark_prices.get(&sym).copied().unwrap_or(0.0);
        let ts = Utc::now().timestamp_millis();
        drop(st);
        Ok(MarketSnapshot {
            symbol: sym,
            mark_price: mark,
            funding_rate: 0.0001,
            open_interest: 1_000_000.0,
            long_short_ratio: 1.05,
            bid: mark * 0.9998,
            ask: mark * 1.0002,
            bid_qty: 0.0,
            ask_qty: 0.0,
            cvd: 0.0,
            timestamp: ts,
        })
    }

    /// Resumen JSON del simulador para el dashboard
    pub fn estado_json(&self) -> serde_json::Value {
        let st = self.inner.lock().unwrap();
        serde_json::json!({
            "modo": "simulacion",
            "balance_inicial": st.initial_balance,
            "wallet_balance": st.wallet_balance,
            "available_balance": st.available_balance,
            "pnl_realizado_total": st.pnl_realizado_total,
            "posiciones_abiertas": st.positions.len(),
            "ordenes_abiertas": st.open_orders.iter().filter(|o| o.status == "NEW").count(),
            "trades_total": st.trades.len(),
            "leverage_por_simbolo": st.leverage_map,
            "mark_prices": st.mark_prices,
            "timestamp": Utc::now().timestamp_millis(),
        })
    }
}

fn resp_simulada(order_id: u64, symbol: &str, req: &FuturesOrderRequest, precio: f64, status: &str) -> FuturesOrderResponse {
    FuturesOrderResponse {
        order_id,
        symbol: symbol.to_uppercase(),
        status: status.to_string(),
        client_order_id: req.new_client_order_id.clone().unwrap_or_else(|| format!("sim-{}", order_id)),
        price: precio.to_string(),
        avg_price: if status == "FILLED" { precio.to_string() } else { "0".to_string() },
        orig_qty: req.quantity.to_string(),
        executed_qty: if status == "FILLED" { req.quantity.to_string() } else { "0".to_string() },
        cum_quote: if status == "FILLED" { format!("{:.4}", precio * req.quantity) } else { "0".to_string() },
        time_in_force: req.time_in_force.as_ref().map(|t| serde_json::to_string(t).unwrap_or_default().trim_matches('"').to_string()).unwrap_or_else(|| "GTC".to_string()),
        order_type: serde_json::to_string(&req.order_type).unwrap_or_default().trim_matches('"').to_string(),
        reduce_only: req.reduce_only.unwrap_or(false),
        close_position: req.close_position.unwrap_or(false),
        side: serde_json::to_string(&req.side).unwrap_or_default().trim_matches('"').to_string(),
        position_side: req.position_side.as_ref().map(|ps| serde_json::to_string(ps).unwrap_or_default().trim_matches('"').to_string()).unwrap_or_else(|| "BOTH".to_string()),
        stop_price: req.stop_price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string()),
        working_type: req.working_type.as_ref().map(|w| serde_json::to_string(w).unwrap_or_default().trim_matches('"').to_string()).unwrap_or_else(|| "MARK_PRICE".to_string()),
        price_protect: false,
        orig_type: serde_json::to_string(&req.order_type).unwrap_or_default().trim_matches('"').to_string(),
        activate_price: None,
        price_rate: None,
        update_time: Utc::now().timestamp_millis() as u64,
        working_time: Utc::now().timestamp_millis() as u64,
    }
}

/// PnL flotante de una posición según dirección
fn pnl_float(side: PositionSide, entry: f64, mark: f64, qty: f64) -> f64 {
    match side {
        PositionSide::Long => (mark - entry) * qty,
        PositionSide::Short => (entry - mark) * qty,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn abre_y_cierra_long_con_pnl() {
        let sim = FuturesSimulator::new(10_000.0);
        sim.actualizar_precio("BTCUSDT", 60_000.0);
        let req = FuturesOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Buy,
            position_side: Some(PositionSide::Long),
            order_type: OrderType::Market,
            quantity: 0.01,
            price: None,
            stop_price: None,
            trailing_delta: None,
            reduce_only: Some(false),
            post_only: None,
            close_position: None,
            time_in_force: None,
            working_type: None,
            new_client_order_id: None,
            price_precision: None,
            quantity_precision: None,
        };
        let resp = sim.place_order(&req).await.unwrap();
        assert_eq!(resp.status, "FILLED");

        let pos = sim.positions(Some("BTCUSDT")).await.unwrap();
        assert_eq!(pos.len(), 1);
        assert_eq!(pos[0].position_amt, "0.01");

        // Sube el precio → PnL positivo
        sim.actualizar_precio("BTCUSDT", 61_000.0);
        let pos = sim.positions(Some("BTCUSDT")).await.unwrap();
        let pnl: f64 = pos[0].unrealized_profit.parse().unwrap();
        assert!(pnl > 0.0);

        // Cierra la posición
        let close_req = FuturesOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: OrderSide::Sell,
            position_side: None,
            order_type: OrderType::Market,
            quantity: 0.01,
            price: None,
            stop_price: None,
            trailing_delta: None,
            reduce_only: Some(true),
            post_only: None,
            close_position: Some(true),
            time_in_force: None,
            working_type: None,
            new_client_order_id: None,
            price_precision: None,
            quantity_precision: None,
        };
        sim.place_order(&close_req).await.unwrap();
        let pos = sim.positions(Some("BTCUSDT")).await.unwrap();
        assert_eq!(pos.len(), 0);

        let estado = sim.estado_json();
        let pnl_total: f64 = estado["pnl_realizado_total"].as_f64().unwrap_or(0.0);
        assert!(pnl_total > 0.0, "PnL realizado debe ser positivo, fue {}", pnl_total);
    }
}
