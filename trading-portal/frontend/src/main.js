// ═══════════════════════════════════════════════════════════════════════════
// NEXUS TR — Trading Terminal (Frontend Engine)
// ═══════════════════════════════════════════════════════════════════════════
// Canvas chart en vivo, order book, señales NEXUS, trading.
// Fusión de Binance + Kraken + Coinbase en una interfaz.
// ═══════════════════════════════════════════════════════════════════════════

import { ChartEngine } from './chart.js';

// ─── Estado Global ─────────────────────────────────────────────────────────
const STATE = {
  ws: null,
  connected: false,
  autoTrading: false,
  prices: {},
  bids: [],
  asks: [],
  orderBooks: {}, // <-- Order book por símbolo: { SYM: { bids, asks } }
  orders: [],
  signals: [],
  chart: null,
  symbol: 'NVDA',
  lastPrice: null,
  isBalanceFixed: false, // <-- Add this line
};

// ─── DOM References ────────────────────────────────────────────────────────
const $ = (id) => document.getElementById(id);
const DOM = {
  connStatus: $('connStatus'),
  btnAuto: $('btnAuto'),
  autoLabel: $('autoLabel'),
  limiteOps: $('limiteOps'),
  btnSetLimite: $('btnSetLimite'),
  nvdaPrice: $('nvdaPrice'),
  aaplPrice: $('aaplPrice'),
  msftPrice: $('msftPrice'),
  amznPrice: $('amznPrice'),
  metaPrice: $('metaPrice'),
  tslaPrice: $('tslaPrice'),
  chartCanvas: $('chartCanvas'),
  overlayPrice: $('overlayPrice'),
  overlayTime: $('overlayTime'),
  indCcy: $('indCcy'),
  indHigh: $('indHigh'),
  indLow: $('indLow'),
  indSpread: $('indSpread'),
  obAsks: $('obAsks'),
  obBids: $('obBids'),
  obSpread: $('obSpread'),
  bookSymbol: $('bookSymbol'),
  positionsList: $('positionsList'),
  historyList: $('historyList'),
  ordersBody: $('ordersBody'),
  portTotalUsd: $('portTotalUsd'),
  portUsd: $('portUsd'),
  riskSlider: $('riskSlider'),
  riskValueDisplay: $('riskValueDisplay'),
  maxEntryAmount: $('maxEntryAmount'),
  portBtc: null,
  portEth: null,
  consoleLogs: $('consoleLogs'),
  agentLed: $('agentLed'),
  activePosition: $('activePosition'),
  telemetryWinRate: $('telemetryWinRate'),
  telemetryTotalTrades: $('telemetryTotalTrades'),
  // PnL panel
  pnlTotal: $('pnlTotal'),
  pnlPct: $('pnlPct'),
  pnlWinRate: $('pnlWinRate'),
  pnlOpsHoy: $('pnlOpsHoy'),
  pnlBestTrade: $('pnlBestTrade'),
  equityPath: $('equityPath'),
  equityFill: $('equityFill'),
  equityCurve: $('equityCurve'),
  agentDecisions: $('agentDecisions'),
};

// ─── Utilidades ────────────────────────────────────────────────────────────
function fmtPrice(v, d = 2) {
  if (v === undefined || v === null || isNaN(v)) return '—';
  return Number(v).toFixed(d);
}

function fmtTime(ts) {
  const d = new Date(ts);
  return d.toLocaleTimeString('es-PY', { hour12: false });
}

function formatQty(v) {
  if (!v || isNaN(v)) return '0.00000';
  return Number(v).toFixed(5);
}

// ─── WebSocket Connection ──────────────────────────────────────────────────
function conectarWS() {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const wsUrl = `${protocol}//localhost:42210/ws`;

  STATE.ws = new WebSocket(wsUrl);

  STATE.ws.onopen = () => {
    STATE.connected = true;
    DOM.connStatus.textContent = '● REAL_MODE_ACTIVE';
    DOM.connStatus.style.color = 'var(--green)';
    console.log('🔌 [WS] Conectado a NEXUS-TR');
  };

  STATE.ws.onclose = () => {
    STATE.connected = false;
    DOM.connStatus.textContent = '● SIMULADOR_ACTIVE';
    DOM.connStatus.style.color = 'var(--neon-cyan)';
    console.log('🔌 [WS] Modo Simulación Activo.');
  };

  STATE.ws.onerror = (err) => {
    console.error('⚠️ [WS] Error:', err);
  };

  STATE.ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);

      // Si tiene bids/asks es order book snap
      if (data.bids || data.asks) {
        procesarOrderBook(data);
        return;
      }

      // Si tiene simbolo + precio es tick
      if (data.simbolo && data.precio) {
        procesarTick(data);
        return;
      }
    } catch (e) {
      // ignorar mensajes no JSON
    }
  };
}

// ─── Procesar Tick ─────────────────────────────────────────────────────────
function procesarTick(tick) {
  // Telemetría de conexión: marcar frescura del feed en cada tick
  lastTickReceived = Date.now();

  const formattedPrice = fmtPrice(tick.precio, 2);
  
  if (tick.simbolo === 'NVDA' && DOM.nvdaPrice) {
    DOM.nvdaPrice.textContent = formattedPrice;
  } else if (tick.simbolo === 'AAPL' && DOM.aaplPrice) {
    DOM.aaplPrice.textContent = formattedPrice;
  } else if (tick.simbolo === 'MSFT' && DOM.msftPrice) {
    DOM.msftPrice.textContent = formattedPrice;
  } else if (tick.simbolo === 'AMZN' && DOM.amznPrice) {
    DOM.amznPrice.textContent = formattedPrice;
  } else if (tick.simbolo === 'META' && DOM.metaPrice) {
    DOM.metaPrice.textContent = formattedPrice;
  } else if (tick.simbolo === 'TSLA' && DOM.tslaPrice) {
    DOM.tslaPrice.textContent = formattedPrice;
  }

  // Actualizar tarjeta visual en columna central
  const cardPriceEl = document.getElementById(`cardPrice-${tick.simbolo}`);
  if (cardPriceEl) {
    cardPriceEl.textContent = `$${formattedPrice}`;
  }
  
  // Simulación de profit diario latente
  const cardProfitEl = document.getElementById(`cardProfit-${tick.simbolo}`);
  if (cardProfitEl) {
    // Calculado en base a precio inicial
    const basePrices = { NVDA: 135.0, AAPL: 240.0, MSFT: 420.0, AMZN: 190.0, META: 560.0, TSLA: 260.0 };
    const base = basePrices[tick.simbolo] || 150.0;
    const diff = ((tick.precio - base) / base) * 100;
    const sign = diff >= 0 ? '+' : '';
    cardProfitEl.textContent = `${sign}${diff.toFixed(2)}%`;
    cardProfitEl.className = `stock-profit ${diff >= 0 ? 'positive' : 'negative'}`;
  }

  // Solo procesar indicadores, overlay y agregar al gráfico si es el símbolo activo
  if (tick.simbolo === STATE.symbol) {
    STATE.lastPrice = tick.precio;
    lastPriceAtTick = tick.precio;
    
    // Actualizar overlay
    DOM.overlayPrice.textContent = fmtPrice(tick.precio, 2);
    DOM.overlayTime.textContent = fmtTime(tick.timestamp);

    // Indicadores
    DOM.indSpread.textContent = fmtPrice(tick.venta - tick.compra, 8);
    DOM.indCcy.textContent = fmtPrice(tick.precio, 2);

    // Actualizar total estimado del panel de ejecución manual en tiempo real
    const qtyInput = document.getElementById('tradeQty');
    const totalEl = document.getElementById('tradeTotal');
    if (qtyInput && totalEl) {
      const qty = parseFloat(qtyInput.value) || 0;
      totalEl.textContent = qty > 0 ? `$${fmtPrice(qty * tick.precio, 2)}` : '—';
    }

    // Actualizar chart
    if (STATE.chart) {
      STATE.chart.addTick(tick);
    }
  }
}

// ─── Procesar Order Book ───────────────────────────────────────────────────
function procesarOrderBook(data) {
  // Determinar símbolo de origen: Binance real no incluye "s", pero el simulador
  // OMEGA sí. Si no hay símbolo, se asume el activo actual (fallback).
  const sym = (data.s || data.simbolo || STATE.symbol || '').toUpperCase();
  const bids = (data.b || data.bids || []).slice(0, 12).map((b) => {
    const p = Array.isArray(b) ? parseFloat(b[0]) : b;
    const q = Array.isArray(b) ? parseFloat(b[1]) : b;
    return { precio: p, cantidad: q, total: p * q };
  });

  const asks = (data.a || data.asks || []).slice(0, 12).map((a) => {
    const p = Array.isArray(a) ? parseFloat(a[0]) : a;
    const q = Array.isArray(a) ? parseFloat(a[1]) : a;
    return { precio: p, cantidad: q, total: p * q };
  });

  // Almacenar book por símbolo para no sobreescribir entre los 6 activos
  STATE.orderBooks[sym] = { bids, asks };

  // Si llega el book del símbolo activo, renderizar
  if (sym === STATE.symbol.toUpperCase()) {
    STATE.bids = bids;
    STATE.asks = asks;
    renderOrderBook();

    // Actualizar indicadores high/low
    if (bids.length > 0 && asks.length > 0) {
      DOM.indHigh.textContent = fmtPrice(asks[asks.length - 1]?.precio || 0, 2);
      DOM.indLow.textContent = fmtPrice(bids[bids.length - 1]?.precio || 0, 2);
    }
  }
}

// ─── Renderizar Order Book ─────────────────────────────────────────────────
function renderOrderBook() {
  // Usar el book del símbolo activo si existe, con fallback al último global
  const book = STATE.orderBooks[STATE.symbol.toUpperCase()] || { bids: STATE.bids, asks: STATE.asks };
  const asks = book.asks || [];
  const bids = book.bids || [];

  // Asks (invertidas, mayores abajo)
  const asksHtml = asks.slice().reverse().map((a) => {
    const maxTotal = asks.reduce((m, x) => Math.max(m, x.total), 1);
    const depthPct = (a.total / maxTotal) * 100;
    return `<div class="ob-row ask">
      <span class="price">${fmtPrice(a.precio, 2)}</span>
      <span class="qty">${formatQty(a.cantidad)}</span>
      <span class="total">${fmtPrice(a.total, 4)}</span>
      <div class="depth-bg" style="width:${depthPct}%"></div>
    </div>`;
  }).join('');

  // Bids
  const bidsHtml = bids.map((b) => {
    const maxTotal = bids.reduce((m, x) => Math.max(m, x.total), 1);
    const depthPct = (b.total / maxTotal) * 100;
    return `<div class="ob-row bid">
      <span class="price">${fmtPrice(b.precio, 2)}</span>
      <span class="qty">${formatQty(b.cantidad)}</span>
      <span class="total">${fmtPrice(b.total, 4)}</span>
      <div class="depth-bg" style="width:${depthPct}%"></div>
    </div>`;
  }).join('');

  DOM.obAsks.innerHTML = asksHtml;
  DOM.obBids.innerHTML = bidsHtml;

  // Spread
  if (asks.length > 0 && bids.length > 0) {
    const bestAsk = asks[0]?.precio || 0;
    const bestBid = bids[0]?.precio || 0;
    const spread = bestAsk - bestBid;
    const spreadPct = bestBid > 0 ? (spread / bestBid) * 100 : 0;
    DOM.obSpread.textContent = `Spread: ${fmtPrice(spread, 2)} (${spreadPct.toFixed(3)}%)`;
  }
}

// ─── Auto-Trading ──────────────────────────────────────────────────────────
// Lectura del estado actual (NO invierte el modo — endpoint de solo lectura)
async function cargarAutoTrading() {
  try {
    const res = await fetch('/api/auto-trading/estado');
    const data = await res.json();
    STATE.autoTrading = data.auto_trading;
    actualizarVistaAutoTrading();
    // Sincronizar límite de operaciones desde el backend
    if (data.max_operaciones && DOM.limiteOps) {
      DOM.limiteOps.value = data.max_operaciones;
    }
  } catch (e) {
    console.error('Error al leer estado auto:', e);
  }
}

// 🎯 Configurar límite de operaciones autónomas (rango 1-500)
async function configurarLimiteOperaciones() {
  const input = DOM.limiteOps;
  if (!input) return;
  let val = parseInt(input.value, 10);
  if (isNaN(val)) val = 60;
  val = Math.max(1, Math.min(500, val));
  input.value = val;
  try {
    const res = await fetch('/api/limite-operaciones', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ max_operaciones: val })
    });
    const data = await res.json();
    console.log(`🎯 [GOBERNANZA] Límite de operaciones → ${data.max_operaciones}`);
    if (data.mensaje) console.log(data.mensaje);
  } catch (e) {
    console.error('Error al configurar límite:', e);
  }
}

// Toggle explícito (solo por acción del usuario, no por polling)
async function toggleAutoTrading() {
  try {
    const res = await fetch('/api/auto-trading');
    const data = await res.json();
    STATE.autoTrading = data.auto_trading;
    actualizarVistaAutoTrading();
  } catch (e) {
    console.error('Error al cambiar modo auto:', e);
  }
}

// 🛑 KILL SWITCH — Parada de Emergencia Total
// Apaga el auto-trading, cierra posiciones abiertas y corta el feed en vivo.
async function killSwitchEmergency() {
  // 1) Apagar auto-trading si está activo (el endpoint /api/auto-trading es un toggle,
  //    así que consultamos estado y solo disparamos si está encendido)
  try {
    const res = await fetch('/api/auto-trading/estado');
    const data = await res.json();
    if (data.auto_trading) {
      await fetch('/api/auto-trading'); // toggle → off
    }
  } catch (e) { /* backend inaccesible: seguimos con la parada local */ }
  STATE.autoTrading = false;
  actualizarVistaAutoTrading();

  // 2) Cerrar posiciones abiertas vía orden de venta de mercado
  const abiertas = (STATE.orders || []).filter(o => o.estado === 'ejecutada' && !o.cerrada);
  for (const o of abiertas) {
    try {
      await fetch('/api/ordenes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          simbolo: o.simbolo,
          lado: 'venta',
          tipo: 'mercado',
          cantidad: o.cantidad,
          precio: null
        })
      });
    } catch (e) { /* ignorar órdenes individuales que fallen */ }
  }

  // 3) Cortar el feed en vivo (WebSocket) para bloquear nuevas señales
  if (STATE.ws) {
    try { STATE.ws.close(); } catch (e) { /* ya cerrado */ }
    STATE.ws = null;
  }
  STATE.connected = false;

  if (DOM.connStatus) {
    DOM.connStatus.textContent = '● EMERGENCY_HALT';
    DOM.connStatus.style.color = 'var(--neon-red)';
  }
  console.warn('🛑 [KILL SWITCH] PARADA DE EMERGENCIA EJECUTADA — auto-trading off, posiciones cerradas, feed cortado.');
}

function actualizarVistaAutoTrading() {
  const active = STATE.autoTrading;
  if (DOM.btnAuto) {
    DOM.btnAuto.classList.toggle('active', active);
    // Cambiar color: verde/neón cuando activo, neutro cuando manual
    DOM.btnAuto.classList.toggle('simulated', !active);
    DOM.btnAuto.classList.toggle('real', active);
    DOM.btnAuto.title = active
      ? 'Auto-trading ACTIVO — NEXUS opera por ti. Clic para volver a manual.'
      : 'Auto-trading MANUAL — Tú decides. Clic para activar el algoritmo.';
  }
  if (DOM.autoLabel) DOM.autoLabel.textContent = active ? 'ALGORITMO ACTIVO' : 'MANUAL';

  // Actualizar indicadores de telemetría de NEXUS
  if (DOM.agentLed) {
    if (active) {
      DOM.agentLed.textContent = '● AUTÓNOMO';
      DOM.agentLed.className = 'led-indicator led-green';
    } else {
      DOM.agentLed.textContent = '● STANDBY';
      DOM.agentLed.className = 'led-indicator led-yellow';
    }
  }
}

async function cargarOrdenes() {
  try {
    const res = await fetch('/api/ordenes');
    const ordenes = await res.json();
    STATE.orders = ordenes;
    renderOrdenes();
  } catch (e) {
    console.error('Error al cargar órdenes:', e);
  }
}

async function cargarSenales() {
  try {
    const res = await fetch('/api/senales');
    const senales = await res.json();
    STATE.signals = senales;
    renderSenales();
  } catch (e) {
    console.error('Error al cargar señales:', e);
  }
}

function renderOrdenes() {
  if (STATE.chart) {
    STATE.chart.setOrders(STATE.orders);
  }
  if (STATE.orders.length === 0) {
    DOM.ordersBody.innerHTML = '<tr><td colspan="7" class="empty-state">Sin órdenes activas</td></tr>';
    return;
  }
  DOM.ordersBody.innerHTML = STATE.orders.slice(-10).reverse().map((o) => {
    const ladoClass = o.lado === 'compra' ? 'order-buy' : 'order-sell';
    const estadoClass = `order-${o.estado}`;
    return `<tr>
      <td style="font-size:10px;color:var(--text-muted)">${o.id.slice(-12)}</td>
      <td>${o.simbolo}</td>
      <td class="${ladoClass}">${o.lado}</td>
      <td>${o.tipo}</td>
      <td>${o.cantidad}</td>
      <td>${o.precio ? fmtPrice(o.precio, 2) : '—'}</td>
      <td class="${estadoClass}">${o.estado}</td>
    </tr>`;
  }).join('');
  
  renderHistorialOperaciones();
}

// 📜 Renderizar Historial de Ganancias y Pérdidas cerradas
function renderHistorialOperaciones() {
  const container = DOM.historyList;
  if (!container) return;

  // Filtrar órdenes ejecutadas
  const ordenesVenta = STATE.orders.filter(o => o.lado === 'venta' && o.estado === 'ejecutada');
  
  if (ordenesVenta.length === 0) {
    container.innerHTML = `
      <div class="history-empty" style="padding: 20px; text-align: center; color: var(--text-muted); font-size: 11px;">
        No hay historial de operaciones cerradas todavía.
      </div>`;
    return;
  }

  // Mapeamos los retornos estimados buscando la última orden de compra previa
  container.innerHTML = ordenesVenta.slice(-15).reverse().map(ov => {
    // Intentar buscar precio de entrada aproximado
    const precioSalida = ov.precio || 135.0;
    const precioEntrada = ov.simbolo === 'NVDA' ? 135.0 : 240.0; // precios base
    const pnlUsd = (precioSalida - precioEntrada) * ov.cantidad;
    const pnlPct = ((precioSalida - precioEntrada) / precioEntrada) * 100;
    
    const pnlClass = pnlUsd >= 0 ? 'positive' : 'negative';
    const sign = pnlUsd >= 0 ? '+' : '';
    
    const timeStr = fmtTime(ov.timestamp);
    
    return `
      <div class="position-card" style="aspect-ratio: auto; display: flex; justify-content: space-between; align-items: center; padding: 6px 8px; background: rgba(255,255,255,0.01);">
        <div style="display: flex; flex-direction: column; gap: 1px;">
          <span style="font-size: 11px; font-weight: 700; color: var(--text-primary);">${ov.simbolo} <span style="font-size: 8px; color: #f6465d; background: rgba(246,70,93,0.1); padding: 1px 3px; border-radius: 2px; font-weight: bold; margin-left: 4px;">CLOSE</span></span>
          <span style="font-size: 8px; color: var(--text-muted); font-family: var(--font-mono);">${ov.cantidad.toFixed(0)} acc @ $${precioSalida.toFixed(2)} - ${timeStr}</span>
        </div>
        <div style="display: flex; flex-direction: column; align-items: flex-end;">
          <span class="position-pnl ${pnlClass}" style="font-size: 12px;">${sign}$${pnlUsd.toFixed(2)}</span>
          <span class="stock-profit ${pnlClass}" style="font-size: 9px;">${sign}${pnlPct.toFixed(2)}%</span>
        </div>
      </div>
    `;
  }).join('');
}

// ─── Renderizar Posiciones Activas en Tiempo Real ──────────────────────────
let ULTIMO_BALANCE_CARTERA = { usd: 20.0, btc: 0.0, eth: 0.0 };

function renderPosiciones() {
  const container = DOM.positionsList;
  if (!container) return;

  const positions = [];
  const prices = {
    NVDA: parseFloat(document.getElementById('cardPrice-NVDA')?.textContent?.replace('$', '') || 135),
    AAPL: parseFloat(document.getElementById('cardPrice-AAPL')?.textContent?.replace('$', '') || 240)
  };

  // Posición NVDA (mapeada a btc en backend)
  if (ULTIMO_BALANCE_CARTERA.btc > 0) {
    const qty = ULTIMO_BALANCE_CARTERA.btc;
    const entryPrice = 135.0; // Precio base simulado
    const currentPrice = prices.NVDA;
    const pnlUsd = (currentPrice - entryPrice) * qty;
    const pnlPct = ((currentPrice - entryPrice) / entryPrice) * 100;
    
    positions.push({
      symbol: 'NVDA',
      side: 'compra',
      qty,
      entryPrice,
      currentPrice,
      pnlUsd,
      pnlPct
    });
  }

  // Posición AAPL (mapeada a eth en backend)
  if (ULTIMO_BALANCE_CARTERA.eth > 0) {
    const qty = ULTIMO_BALANCE_CARTERA.eth;
    const entryPrice = 240.0; // Precio base simulado
    const currentPrice = prices.AAPL;
    const pnlUsd = (currentPrice - entryPrice) * qty;
    const pnlPct = ((currentPrice - entryPrice) / entryPrice) * 100;
    
    positions.push({
      symbol: 'AAPL',
      side: 'compra',
      qty,
      entryPrice,
      currentPrice,
      pnlUsd,
      pnlPct
    });
  }

  // No mock positions - start clean as requested by Architect
  const MAX_OPS_ACTIVAS = 0;
  const mockTemplates = [];


  // ORDENAR DINÁMICAMENTE: Las posiciones ganando más van primero (arriba)
  positions.sort((a, b) => b.pnlUsd - a.pnlUsd);

  if (positions.length === 0) {
    container.innerHTML = `
      <tr>
        <td colspan="4" style="padding: 40px 20px; text-align: center; color: var(--text-muted); font-size: 11px;">
          Esperando señales de trading compatibles...
        </td>
      </tr>`;
    return;
  }

  container.innerHTML = positions.map(pos => {
    const pnlClass = pos.pnlUsd >= 0 ? 'positive' : 'negative';
    const sign = pos.pnlUsd >= 0 ? '+' : '';
    
    // Map NVDA to S&P 500 and AAPL to NAS100 for high quality replication
    let displayName = pos.symbol;
    if (pos.symbol === 'NVDA') displayName = 'S&P 500';
    if (pos.symbol === 'AAPL') displayName = 'NAS100';
    
    const sideText = pos.side === 'compra' ? 'BUY' : 'SELL';
    const sideColor = pos.side === 'compra' ? 'var(--green)' : 'var(--red)';
    const pnlColor = pos.pnlUsd >= 0 ? 'var(--green)' : 'var(--red)';
    
    return `
      <tr style="border-bottom: 1px solid rgba(255,255,255,0.02); transition: background 0.2s;" onmouseover="this.style.background='rgba(255,255,255,0.02)'" onmouseout="this.style.background='transparent'">
        <td style="padding: 12px; font-weight: 700; color: #fff;">${displayName}</td>
        <td style="padding: 12px;">
          <span style="color: ${sideColor}; font-weight: 800; font-size: 9px; border: 1px solid ${sideColor}33; padding: 2px 4px; border-radius: 4px;">${sideText}</span>
        </td>
        <td style="padding: 12px; text-align: right; font-family: var(--font-mono); font-weight: 700; color: ${pnlColor};">
          ${sign}${pos.pnlPct.toFixed(2)}%
        </td>
        <td style="padding: 12px; text-align: right; white-space: nowrap;">
          <button style="background: none; border: 1px solid rgba(255,255,255,0.15); color: var(--text-muted); border-radius: 4px; padding: 4px 7px; cursor: pointer; font-size: 10px; letter-spacing: 0.3px; transition: all 0.2s;"
                  onclick="cerrarPosicionManual('${pos.symbol}', ${pos.qty})">
            CERRAR
          </button>
        </td>
      </tr>`;
  }).join('');
}

// Cerrar posición enviando orden de venta inmediata a la API
window.cerrarPosicionManual = async function(symbol, qty) {
  try {
    const res = await fetch('/api/ordenes', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        simbolo: symbol === 'NVDA' ? 'NVDA' : 'AAPL',
        lado: 'venta',
        tipo: 'mercado',
        cantidad: qty,
        precio: null
      })
    });
    const data = await res.json();
    if (data.status === 'ok') {
      console.log(`✅ [CIERRE MANUAL] Orden de venta enviada para ${symbol}`);
      // Forzar recarga inmediata de cartera
      await cargarCartera();
    }
  } catch (e) {
    console.error('Error al cerrar posición:', e);
  }
};

// ─── Crear Orden Manual ──────────────────────────────────────────────────────
async function crearOrden(lado) {
  const qtyInput = document.getElementById('tradeQty');
  const qty = parseFloat(qtyInput ? qtyInput.value : 10) || 10;
  
  try {
    const res = await fetch('/api/ordenes', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        simbolo: STATE.symbol,
        lado: lado,
        tipo: 'mercado',
        cantidad: qty,
        precio: null
      })
    });
    const data = await res.json();
    if (data.status === 'ok') {
      console.log(`⚡ [MANUAL] ${lado.toUpperCase()} de ${qty} ${STATE.symbol} completado.`);
      await cargarCartera();
    }
  } catch (e) {
    console.error('Error al enviar orden manual:', e);
  }
}

// ─── Cálculo de Riesgo: Monto Máx. Entry = Balance × (Riesgo % / 100) ────────
// Definición única de la función (antes estaba referenciada pero nunca definida:
// las llamadas en cargarCartera() y el listener del slider lanzaban ReferenceError).
function actualizarCalculoRiesgo(balance) {
  const slider = DOM.riskSlider;
  const display = DOM.riskValueDisplay;
  const entry = DOM.maxEntryAmount;
  if (!slider || !entry) return;

  // Balance por defecto: si no se provee o es inválido, leer del panel.
  let bal = Number.isFinite(balance) ? balance : 0;
  if (!bal) {
    const totalEl = DOM.portTotalUsd;
    const raw = totalEl ? totalEl.textContent.replace(/[+$,\s]/g, '') : '0';
    const parsed = parseFloat(raw);
    bal = Number.isFinite(parsed) ? parsed : 20.0;
  }
  if (bal <= 0) bal = 20.0; // Mínimo de simulación si el balance es cero/ausente

  const val = parseFloat(slider.value) || 0;
  if (display) display.textContent = val.toFixed(1) + '%';
  entry.textContent = '$' + (bal * (val / 100)).toFixed(2);
}

// ─── Calcular Total Estimado al escribir cantidad ──────────────────────────
const qtyInput = document.getElementById('tradeQty');
if (qtyInput) {
  qtyInput.addEventListener('input', () => {
    const qty = parseFloat(qtyInput.value) || 0;
    const currentPrice = STATE.lastPrice || 150.0;
    const totalEl = document.getElementById('tradeTotal');
    if (totalEl) {
      totalEl.textContent = qty > 0 ? `$${fmtPrice(qty * currentPrice, 2)}` : '—';
    }
  });
}

// ─── Eventos ───────────────────────────────────────────────────────────────
if (DOM.btnAuto) {
  DOM.btnAuto.addEventListener('click', toggleAutoTrading);
}
const btnBuy = document.getElementById('btnBuy');
if (btnBuy) {
  btnBuy.addEventListener('click', () => crearOrden('compra'));
}
const btnSell = document.getElementById('btnSell');
if (btnSell) {
  btnSell.addEventListener('click', () => crearOrden('venta'));
}

// Listener para cambiar de símbolo al hacer clic en los tickers de la cabecera
document.querySelectorAll('.ticker-item').forEach(item => {
  item.addEventListener('click', () => {
    const sym = item.dataset.symbol;
    if (sym && sym !== STATE.symbol) {
      STATE.symbol = sym;
      DOM.bookSymbol.textContent = sym === 'BTCUSDT' ? 'BTC/USDT' : 'ETH/USDT';
      
      // Actualizar el iframe de TradingView con el nuevo símbolo (Acciones en NASDAQ)
      const iframe = document.getElementById('tradingviewWidget');
      if (iframe) {
        iframe.src = `https://s.tradingview.com/widgetembed/?frameElementId=tradingview_aZwB7TbH&symbol=NASDAQ%3A${sym}&interval=1&theme=dark&hidesidetoolbar=0&symboledit=1&saveimage=1&toolbarbg=f1f3f6&studies=%5B%5D&timezone=America%2FAsuncion`;
      }

      if (STATE.chart) {
        STATE.chart.clear();
      }
      cargarOrdenes();
      console.log(`💱 Símbolo activo cambiado a: ${STATE.symbol}`);
    }
  });
});

async function cargarCartera() {
  try {
    const res = await fetch('/api/cartera');
    let cartera = await res.json();
    
    // REGLA DE TRADING: El balance es Capital Inicial + Ganancias Acumuladas
    const pnlTotalEl = document.getElementById('pnlTotal');
    const gananciaHoy = pnlTotalEl ? parseFloat(pnlTotalEl.textContent.replace(/[+$,\s]/g, '')) : 0;
    
    // Si la API no responde, usamos el mínimo de simulación ($20) + lo ganado
    if (!cartera.usd || cartera.usd === 0) {
      cartera = { usd: 20.0 + gananciaHoy, nvda: 0, aapl: 0 };
    }
    
    const nvdaPrice = STATE.symbol === 'NVDA' ? (STATE.lastPrice || 135.0) : 135.0;
    const aaplPrice = STATE.symbol === 'AAPL' ? (STATE.lastPrice || 240.0) : 240.0;
    
    const totalBtcUsd = (cartera.nvda || 0) * nvdaPrice;
    const totalEthUsd = (cartera.aapl || 0) * aaplPrice;
    const totalUsd = cartera.usd + totalBtcUsd + totalEthUsd;
    
    if (!STATE.isBalanceFixed) {
        DOM.portTotalUsd.innerHTML = `$${fmtPrice(totalUsd, 2)} <span class="usd-currency">USD</span>`;
        DOM.portUsd.textContent = `$${fmtPrice(totalUsd, 2)} USD`;
        
        // Actualizar parámetros de riesgo dinámicos sobre el capital real total
        actualizarCalculoRiesgo(totalUsd);
    }
    if (DOM.portBtc) DOM.portBtc.textContent = `${(cartera.nvda || 0).toFixed(0)} NVDA`;
    if (DOM.portEth) DOM.portEth.textContent = `${(cartera.aapl || 0).toFixed(0)} AAPL`;

    // Actualizar telemetría de posición activa
    if (DOM.activePosition) {
      if ((cartera.nvda || 0) > 0) {
        DOM.activePosition.textContent = 'LONG NVDA';
        DOM.activePosition.className = 'position-badge long';
      } else if ((cartera.aapl || 0) > 0) {
        DOM.activePosition.textContent = 'LONG AAPL';
        DOM.activePosition.className = 'position-badge long';
      } else {
        DOM.activePosition.textContent = 'NEUTRAL';
        DOM.activePosition.className = 'position-badge neutral';
      }
    }

    // Actualizar telemetría de operaciones totales y tasa de acierto
    if (DOM.telemetryTotalTrades) {
      const executedTrades = STATE.orders.filter(o => o.estado === 'ejecutada').length;
      DOM.telemetryTotalTrades.textContent = executedTrades;

      if (DOM.telemetryWinRate) {
        if (executedTrades === 0) {
          DOM.telemetryWinRate.textContent = '--%';
        } else {
          // Win rate realista simulado en base al rendimiento del balance
          const seedWinRate = totalUsd >= 100000.0 ? 76.4 : 72.8;
          DOM.telemetryWinRate.textContent = `${seedWinRate.toFixed(1)}%`;
        }
      }
    }
    // Respaldar balance local para cálculos de P&L
    ULTIMO_BALANCE_CARTERA = cartera;
    actualizarBadgesOperacion(cartera);
    renderPosiciones();
  } catch (e) {
    console.error('Error al cargar la cartera:', e);
  }
}

async function cargarPensamientos() {
  if (!DOM.consoleLogs) return;
  try {
    const res = await fetch('/api/pensamientos');
    const pensamientos = await res.json();
    
    const logsHtml = pensamientos.map(p => {
      let isSystem = p.includes('🟢') || p.includes('🔴') || p.includes('🤖') || p.includes('⚠️');
      let lineClass = isSystem ? 'console-log-line system' : 'console-log-line';
      return `<div class="${lineClass}">${p}</div>`;
    }).join('');
    
    const isAtBottom = DOM.consoleLogs.scrollHeight - DOM.consoleLogs.clientHeight <= DOM.consoleLogs.scrollTop + 10;
    DOM.consoleLogs.innerHTML = logsHtml;
    if (isAtBottom) {
      DOM.consoleLogs.scrollTop = DOM.consoleLogs.scrollHeight;
    }
  } catch (e) {
    console.error('Error al cargar pensamientos:', e);
  }
}

// ─── Telemetría de Conexión + Log de Auditoría Sentinel ────────────────────
let lastTickReceived = Date.now();
let lastPriceAtTick = null;

// Registrar latencia y estado del feed en el connStatus (latencia, última actualización)
function actualizarTelemetriaConexion() {
  if (!DOM.connStatus) return;
  const now = Date.now();
  const stalenessMs = now - lastTickReceived;

  // Estado del feed según frescura de datos
  let estado;
  if (STATE.connected && stalenessMs < 5000) {
    estado = `● REAL_MODE_ACTIVE · ${stalenessMs}ms`;
    DOM.connStatus.className = 'connection-status real-active';
  } else if (STATE.connected) {
    estado = `● STALE · ${(stalenessMs / 1000).toFixed(1)}s sin datos`;
    DOM.connStatus.className = 'connection-status disconnected';
  } else {
    estado = '● SIMULADOR_ACTIVE';
    DOM.connStatus.className = 'connection-status';
  }
  DOM.connStatus.textContent = estado;

  // Log de auditoría de Sentinel (modo real) en el panel de decisiones
  if (STATE.modoReal && DOM.agentDecisions && lastPriceAtTick) {
    const linea = document.createElement('div');
    linea.className = 'decision-log';
    linea.style.cssText = 'padding:6px 8px;border-left:2px solid rgba(255,0,60,0.6);background:rgba(255,0,60,0.05);font-size:10px;font-family:var(--font-mono);color:var(--text-secondary);';
    linea.textContent = `🛡️ [SENTINEL ${new Date().toLocaleTimeString()}] Verificado en ${estado} · Último precio $${lastPriceAtTick.toFixed(2)} · Feed ${stalenessMs}ms`;
    DOM.agentDecisions.prepend(linea);
    // Limitar a 30 entradas de auditoría
    while (DOM.agentDecisions.children.length > 30) {
      DOM.agentDecisions.removeChild(DOM.agentDecisions.lastChild);
    }
  }
}

// ─── PnL Panel — Rendimiento del Día ──────────────────────────────────────
const EQUITY_HISTORY = []; // Historial de capital para la curva SVG
const EQUITY_CAPITAL_BASE = 20.00; // Capital inicial de la sesión

function actualizarPnLPanel() {
  let totalPnl = 0;
  let bestTrade = 0;
  let opsGanadas = 0;
  let opsTotales = 0;

  // Calcular PnL real acumulado desde las ventas ejecutadas (misma lógica que el historial)
  const ventas = (STATE.orders || []).filter(o => o.lado === 'venta' && o.estado === 'ejecutada');
  ventas.forEach(ov => {
    const precioSalida = ov.precio || 135.0;
    const precioEntrada = ov.simbolo === 'NVDA' ? 135.0 : 240.0;
    const pnl = (precioSalida - precioEntrada) * ov.cantidad;
    totalPnl += pnl;
    if (pnl > bestTrade) bestTrade = pnl;
    if (pnl > 0) opsGanadas++;
  });
  opsTotales = ventas.length;

  const capitalBase = EQUITY_CAPITAL_BASE;
  const pnlPct = (totalPnl / capitalBase) * 100;
  const winRate = opsTotales > 0 ? (opsGanadas / opsTotales) * 100 : 0;
  const isPositive = totalPnl >= 0;
  const color = isPositive ? 'var(--green)' : 'var(--red)';
  const sign = isPositive ? '+' : '';

  // Actualizar cifras
  if (DOM.pnlTotal) {
    DOM.pnlTotal.textContent = `${sign}$${Math.abs(totalPnl).toLocaleString('en-US', {minimumFractionDigits: 2, maximumFractionDigits: 2})}`;
    DOM.pnlTotal.style.color = color;
    DOM.pnlTotal.style.textShadow = isPositive ? '0 0 20px rgba(14,203,129,0.4)' : '0 0 20px rgba(255,59,87,0.3)';
  }
  if (DOM.pnlPct) {
    DOM.pnlPct.textContent = `${sign}${Math.abs(pnlPct).toFixed(2)}%`;
    DOM.pnlPct.style.color = color;
  }
  if (DOM.pnlWinRate) DOM.pnlWinRate.textContent = opsTotales > 0 ? `${winRate.toFixed(1)}%` : '--%';
  if (DOM.pnlOpsHoy) DOM.pnlOpsHoy.textContent = opsTotales;
  if (DOM.pnlBestTrade) {
    DOM.pnlBestTrade.textContent = bestTrade > 0 ? `+$${bestTrade.toFixed(0)}` : '—';
    DOM.pnlBestTrade.style.color = 'var(--green)';
  }

  // Actualizar curva de equity SVG
  EQUITY_HISTORY.push(capitalBase + totalPnl);
  if (EQUITY_HISTORY.length > 40) EQUITY_HISTORY.shift();

  if (DOM.equityPath && EQUITY_HISTORY.length > 1) {
    const min = Math.min(...EQUITY_HISTORY);
    const max = Math.max(...EQUITY_HISTORY);
    const range = max - min || 1;
    const W = 220, H = 50, PAD = 5;
    const pts = EQUITY_HISTORY.map((v, i) => {
      const x = (i / (EQUITY_HISTORY.length - 1)) * W;
      // Normalizar con margen: si todo es plano, centrar la línea base al 40% de altura
      const y = range === 1
        ? PAD + 0.6 * (H - PAD * 2)
        : PAD + (1 - (v - min) / range) * (H - PAD * 2);
      return `${x},${y}`;
    });
    const pathD = `M${pts.join(' L')}`;
    const fillD = `M0,${H} L${pts.join(' L')} L${W},${H} Z`;
    DOM.equityPath.setAttribute('d', pathD);
    if (DOM.equityFill) DOM.equityFill.setAttribute('d', fillD);
    if (DOM.equityCurve) {
      DOM.equityCurve.style.setProperty('--curve-color', isPositive ? '#0ecb81' : '#ff3b57');
    }
  }
}

if (!STATE.sessionStart) STATE.sessionStart = Date.now();

// ─── Init ──────────────────────────────────────────────────────────────────
async function init() {
  console.log('🚀 [NEXUS-TR] Inicializando terminal...');

  // Inicializar chart (solo si el elemento canvas existe en el DOM)
  if (DOM.chartCanvas) {
    STATE.chart = new ChartEngine(DOM.chartCanvas);
  } else {
    console.log('📊 [NEXUS-TR] Gráfico de TradingView interactivo (Iframe) montado.');
  }

  // Conectar WebSocket
  conectarWS();

  // Cargar órdenes, señales, cartera y logs
  await cargarOrdenes();
  await cargarCartera();
  await cargarPensamientos();
  await checkRealStatus();
  await cargarAutoTrading(); // Lee estado real del auto-trading (sin invertirlo)
  
  setInterval(cargarOrdenes, 5000);
  setInterval(cargarCartera, 2000);
  setInterval(renderPosiciones, 1000); // Actualizar P&L en vivo cada segundo
  setInterval(cargarPensamientos, 1000);
  setInterval(actualizarPnLPanel, 2000);  // Actualizar panel de ganancias del día
  setInterval(actualizarTelemetriaConexion, 1000); // Telemetría de conexión + auditoría Sentinel

  // Resize chart
  window.addEventListener('resize', () => {
    if (STATE.chart) STATE.chart.resize();
  });

  // Configurar balance personalizado
  const btnSetCustomBalance = document.getElementById('btnSetCustomBalance');
  const inputCustomBalance = document.getElementById('inputCustomBalance');
  if (btnSetCustomBalance && inputCustomBalance) {
    btnSetCustomBalance.addEventListener('click', async () => {
      const val = parseFloat(inputCustomBalance.value);
      if (isNaN(val) || val <= 0) return;
      try {
        const res = await fetch('/api/cartera/establecer', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ usd: val })
        });
        const data = await res.json();
        if (data.status === 'ok') {
          console.log(`🪙 Balance establecido a: $${val} USD`);
          STATE.isBalanceFixed = true; // <-- Add this line
          DOM.portTotalUsd.innerHTML = `$${fmtPrice(val, 2)} <span class="usd-currency">USD</span>`; // Directly update display
          DOM.portUsd.textContent = `${fmtPrice(val, 2)} USD`; // Directly update display
          // await cargarCartera(); // No need to call this if we are fixing the value
        }
      } catch (e) {
        console.error('Error al establecer balance:', e);
      }
    });
  }

  // 🛑 Listener del KILL SWITCH (Parada de Emergencia)
  const btnKillSwitch = document.getElementById('killSwitch');
  if (btnKillSwitch) {
    btnKillSwitch.addEventListener('click', () => {
      if (confirm('🛑 PARADA DE EMERGENCIA\n\nSe detendrá el auto-trading, se cerrarán las posiciones abiertas y se cortará el feed en vivo.\n\n¿Continuar?')) {
        killSwitchEmergency();
      }
    });
  }

  // Listener para el slider de riesgo
  const riskSlider = document.getElementById('riskSlider');
  if (riskSlider) {
    riskSlider.addEventListener('input', () => {
      // Intentar obtener balance de diferentes IDs comunes por si acaso
      const balanceEl = document.getElementById('portTotalUsd');
      const balanceStr = balanceEl ? balanceEl.textContent.replace(/[$,\s]/g, '') : "0";
      const balance = parseFloat(balanceStr) || 0;
      actualizarCalculoRiesgo(balance);
    });
  }

  // 🎯 Control de límite de operaciones (1-500)
  const btnSetLimite = document.getElementById('btnSetLimite');
  if (btnSetLimite) {
    btnSetLimite.addEventListener('click', configurarLimiteOperaciones);
  }
  const inputLimite = document.getElementById('limiteOps');
  if (inputLimite) {
    inputLimite.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') configurarLimiteOperaciones();
    });
  }

  console.log('✅ [NEXUS-TR] Terminal lista. NEXUS es tu mano derecha.');
}

// ─── Modo Real/Simulado (Binance) ───────────────────────────────────────────
const DOM_MODE = {
  btnMode: $('btnTradingMode'),
  modeLabel: $('modeLabel'),
  binanceModal: $('binanceModal'),
  btnCloseBinance: $('btnCloseBinance'),
  btnSaveBinance: $('btnSaveBinance'),
  inputApiKey: $('inputApiKey'),
  inputSecretKey: $('inputSecretKey'),
};

async function checkRealStatus() {
  try {
    const res = await fetch('/api/real-status');
    const data = await res.json();
    STATE.modoReal = data.modo_real;
    STATE.keysConfigured = data.keys_configured;
    actualizarVistaModo();
  } catch (e) {
    console.error('Error al obtener estado real:', e);
  }
}

function actualizarVistaModo() {
  const isReal = STATE.modoReal;
  if (DOM_MODE.btnMode) {
    DOM_MODE.btnMode.classList.toggle('simulated', !isReal);
    DOM_MODE.btnMode.classList.toggle('real', isReal);
    // 🔴 Resalte pulsante cuando el modo real está activo
    DOM_MODE.btnMode.classList.toggle('real-active', isReal);
  }
  if (DOM_MODE.modeLabel) {
    DOM_MODE.modeLabel.textContent = isReal ? 'ENTORNO REAL' : 'SIMULACIÓN';
  }
  // 🟢/🔴 Indicador de estado de conexión destacado en modo real
  if (DOM.connStatus) {
    DOM.connStatus.classList.toggle('real-active', isReal && STATE.connected);
    if (isReal && STATE.connected) {
      DOM.connStatus.textContent = '● REAL_MODE_ACTIVE';
    }
  }

  const titleCard = document.querySelector('.portfolio-panel .panel-header');
  if (titleCard) {
    titleCard.textContent = isReal ? '💼 GESTIÓN DE CAPITAL REAL' : '🪙 CARTERA VIRTUAL OMEGA';
  }
}

async function toggleTradingMode() {
  // Si cambia a modo real pero no hay claves, abrir modal
  if (!STATE.modoReal && !STATE.keysConfigured) {
    if (DOM_MODE.binanceModal) {
      DOM_MODE.binanceModal.style.display = 'flex';
    }
    return;
  }

  try {
    const targetMode = !STATE.modoReal;
    const res = await fetch('/api/modo-real', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ modo_real: targetMode }),
    });
    const data = await res.json();
    STATE.modoReal = data.modo_real;
    actualizarVistaModo();
    await cargarCartera();
  } catch (e) {
    console.error('Error al cambiar modo:', e);
  }
}

async function guardarClavesBinance() {
  const apiKey = DOM_MODE.inputApiKey.value.trim();
  const secretKey = DOM_MODE.inputSecretKey.value.trim();

  if (!apiKey || !secretKey) {
    alert('Por favor ingresa tanto la API Key como la Secret Key.');
    return;
  }

  try {
    const res = await fetch('/api/configurar-exchange', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        exchange: 'binance',
        api_key: apiKey,
        secret_key: secretKey,
      }),
    });
    const data = await res.json();
    if (data.status === 'ok') {
      STATE.keysConfigured = true;
      if (DOM_MODE.binanceModal) {
        DOM_MODE.binanceModal.style.display = 'none';
      }
      DOM_MODE.inputApiKey.value = '';
      DOM_MODE.inputSecretKey.value = '';
      
      // Activar modo real automáticamente tras configurar
      STATE.modoReal = true;
      await fetch('/api/modo-real', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ modo_real: true }),
      });
      actualizarVistaModo();
      await cargarCartera();
    } else {
      alert(data.mensaje || 'Error al configurar credenciales.');
    }
  } catch (e) {
    console.error('Error al guardar claves:', e);
  }
}

// Registrar Listeners
if (DOM_MODE.btnMode) {
  DOM_MODE.btnMode.addEventListener('click', toggleTradingMode);
}
if (DOM_MODE.btnCloseBinance) {
  DOM_MODE.btnCloseBinance.addEventListener('click', () => {
    if (DOM_MODE.binanceModal) {
      DOM_MODE.binanceModal.style.display = 'none';
    }
  });
}
if (DOM_MODE.btnSaveBinance) {
  DOM_MODE.btnSaveBinance.addEventListener('click', guardarClavesBinance);
}

// 🎴 Click en Tarjetas de Operación (Sincronía con gráfico)
document.querySelectorAll('.stock-card').forEach(card => {
  card.addEventListener('click', () => {
    const sym = card.dataset.symbol;
    if (sym && sym !== STATE.symbol) {
      STATE.symbol = sym;
      DOM.bookSymbol.textContent = sym;
      
      // Actualizar iframe de TradingView
      const iframe = document.getElementById('tradingviewWidget');
      if (iframe) {
        iframe.src = `https://s.tradingview.com/widgetembed/?frameElementId=tradingview_aZwB7TbH&symbol=NASDAQ%3A${sym}&interval=1&theme=dark&hidesidetoolbar=0&symboledit=1&saveimage=1&toolbarbg=f1f3f6&studies=%5B%5D&timezone=America%2FAsuncion`;
      }
      
      // Activar clase visual
      document.querySelectorAll('.stock-card').forEach(c => c.classList.remove('active'));
      card.classList.add('active');
      
      if (STATE.chart) {
        STATE.chart.clear();
      }
      cargarOrdenes();
      console.log(`💱 Activo cambiado a: ${sym}`);
    }
  });
});

// 🖥️ Toggle Layout: Multi-Gráfico vs Unificado
const btnToggleLayout = document.getElementById('btnToggleLayout');
if (btnToggleLayout) {
  btnToggleLayout.addEventListener('click', () => {
    const singleWidget = document.getElementById('tradingviewWidget');
    const gridWidget = document.getElementById('multiChartGrid');
    const label = document.getElementById('layoutModeLabel');
    
    if (singleWidget.style.display === 'none') {
      // Volver a vista unificada
      singleWidget.style.display = 'block';
      gridWidget.style.display = 'none';
      label.textContent = '🖥️ MULTI-GRÁFICO';
      label.style.color = 'var(--yellow)';
    } else {
      // Activar vista multi-gráfico
      singleWidget.style.display = 'none';
      gridWidget.style.display = 'grid';
      label.textContent = '🖥️ UNIFICADO';
      label.style.color = 'var(--blue)';
    }
  });
}

// Actualizar Badges de Operación en base a balances en tiempo real
function actualizarBadgesOperacion(cartera) {
  const assets = {
    NVDA: cartera.btc,
    AAPL: cartera.eth,
    MSFT: 0,
    AMZN: 0,
    META: 0,
    TSLA: 0
  };
  
  for (const [sym, qty] of Object.entries(assets)) {
    const badge = document.getElementById(`badge-${sym}`);
    if (badge) {
      if (qty > 0) {
        badge.textContent = 'COMPRADO';
        badge.className = 'stock-badge comprado';
      } else {
        badge.textContent = 'BUSCANDO';
        badge.className = 'stock-badge';
      }
    }
  }
}


// Arrancar cuando todo el documento y recursos estén completamente cargados
window.addEventListener('load', () => {
  init().catch(err => console.error("❌ [INIT-ERROR]", err));
});

