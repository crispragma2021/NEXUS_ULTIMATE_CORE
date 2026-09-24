"""
NEXUS TRADER BOT — Ejecutor Principal
Orquesta: datos → indicadores → sentimiento → decisión → ejecución.
"""
import time, json, os, sys
from datetime import datetime, timezone
from typing import Dict, List, Optional

from . import config
from .api import BinanceAPI
from .strategy import DecisionEngine, Signal
from .indicators import calculate_all_indicators

DATA_DIR = os.path.join(os.path.dirname(__file__), 'data')

class NexusTrader:
    """El Corazón del Bot. Loop principal de trading."""

    def __init__(self):
        self.api = BinanceAPI()
        self.engine = DecisionEngine()
        self.symbols = config.WATCHLIST
        self.timeframes = ['5m', '15m', '1h', '4h']
        self.running = False
        self.last_signals: Dict[str, Signal] = {}
        self.active_positions: Dict[str, Dict] = {}
        self.stats = {
            "trades_total": 0,
            "wins": 0,
            "losses": 0,
            "pnl_total": 0.0,
            "daily_pnl": 0.0,
            "last_update": "",
            "balance_history": [],
        }
        os.makedirs(DATA_DIR, exist_ok=True)

    # ── Inicialización ──
    def initialize(self) -> bool:
        """Sincroniza tiempo, verifica conectividad, configura apalancamiento."""
        print("[NEXUS] Inicializando...")
        
        # Sincronizar tiempo
        offset = self.api.sync_time()
        print(f"[NEXUS] Time offset: {offset}ms")
        
        # Verificar conectividad
        if not self.api.ping():
            print("[ERROR] Sin conectividad con Binance")
            return False
        print("[OK] Conectividad Binance verificada")
        
        # Configurar apalancamiento para cada símbolo
        for symbol in self.symbols:
            try:
                status, data = self.api.set_leverage(symbol, config.MAX_LEVERAGE)
                if status == 200:
                    print(f"[OK] {symbol} leverage={config.MAX_LEVERAGE}x")
                else:
                    print(f"[WARN] {symbol} leverage: {data}")
                
                status, data = self.api.set_margin_type(symbol, "ISOLATED")
                if status == 200:
                    print(f"[OK] {symbol} margin=ISOLATED")
                elif "No need to change margin type" in str(data):
                    pass  # ya está aislado
                else:
                    print(f"[WARN] {symbol} margin: {data}")
            except Exception as e:
                print(f"[WARN] {symbol} setup: {e}")
        
        # Obtener balance inicial
        balance = self.api.get_balance()
        if "available" in balance:
            print(f"[OK] Balance: ${balance['available']:.2f} USDT disponible")
            self.stats["balance_history"].append({
                "time": datetime.now(timezone.utc).isoformat(),
                "balance": balance["available"]
            })
        
        print("[NEXUS] Inicialización completa.")
        return True

    # ── Ciclo Principal ──
    def run_once(self) -> Dict[str, Signal]:
        """Ejecuta un ciclo completo de análisis y trading."""
        signals = {}
        
        try:
            # 1. Obtener balance actual
            balance = self.api.get_balance()
            available = balance.get("available", 0)
            positions = self.api.get_positions()
            open_positions_count = len(positions)
            
            # Actualizar posiciones activas
            for pos in positions:
                sym = pos.get("symbol", "")
                amt = float(pos.get("positionAmt", 0))
                if abs(amt) > 0:
                    self.active_positions[sym] = {
                        "amount": amt,
                        "entry_price": float(pos.get("entryPrice", 0)),
                        "unrealized_pnl": float(pos.get("unrealizedProfit", 0)),
                    }
            
            # 2. Obtener funding rates
            funding_rates = {}
            for symbol in self.symbols:
                fr = self.api.get_funding_rate(symbol)
                if fr:
                    funding_rates[symbol] = fr
            
            # 3. Actualizar noticias (cada 5 minutos)
            if not hasattr(self, '_last_news_update') or time.time() - self._last_news_update > 300:
                print("[NEWS] Scrapeando noticias...")
                try:
                    self.engine.update_news()
                except Exception as e:
                    print(f"[WARN] News scrape: {e}")
                self._last_news_update = time.time()
            
            # 4. Analizar cada símbolo
            for symbol in self.symbols:
                # Obtener klines multi-timeframe
                klines_dict = {}
                for tf in self.timeframes:
                    klines = self.api.get_klines(symbol, tf, config.KLINE_LIMIT)
                    if klines:
                        klines_dict[tf] = klines
                
                if not klines_dict:
                    continue
                
                # Generar señal
                fr_data = funding_rates.get(symbol)
                signal = self.engine.analyze(symbol, klines_dict, fr_data)
                signals[symbol] = signal
                
                # Mostrar diagnóstico
                self._print_signal(symbol, signal, available, open_positions_count)
                
                # 5. Ejecutar si hay señal y podemos operar
                if signal.side in ("BUY", "SELL") and signal.confidence >= 0.35:
                    can_trade, reason = self.engine.can_trade(available, open_positions_count, self.stats["daily_pnl"])
                    if can_trade:
                        self._execute_signal(signal, available)
                    else:
                        print(f"  ⛔ {reason}")
            
            # 6. Gestionar trailing stops para posiciones existentes
            self._manage_positions()
            
            # 7. Actualizar estadísticas
            self.stats["last_update"] = datetime.now(timezone.utc).isoformat()
            if "available" in balance:
                self.stats["balance_history"].append({
                    "time": self.stats["last_update"],
                    "balance": balance["available"]
                })
            
        except Exception as e:
            print(f"[ERROR] run_once: {e}")
        
        self.last_signals = signals
        return signals

    # ── Mostrar señal ──
    def _print_signal(self, symbol: str, signal: Signal, balance: float, positions: int):
        """Imprime diagnóstico detallado de la señal."""
        indicators = signal.indicators
        if "error" in indicators:
            print(f"\n[{symbol}] ERROR: {indicators['error']}")
            return
        
        # Helper: indicators.get() puede retornar None si la clave existe con valor None
        # .get(key, default) solo aplica si la clave NO existe
        def _iv(key, default=0.0):
            val = indicators.get(key)
            return val if val is not None else default
        
        print(f"\n{'─' * 50}")
        print(f"  {symbol}  |  Balance: ${balance:.2f}  |  Posiciones: {positions}")
        print(f"{'─' * 50}")
        
        if indicators:
            print(f"  Precio: ${_iv('price'):.4f}")
            print(f"  RSI(14): {_iv('rsi'):.1f}  |  MACD: {_iv('macd_histogram'):.4f}")
            print(f"  BB: ${_iv('bb_lower'):.2f} - ${_iv('bb_upper'):.2f}")
            ema_f = _iv('ema_fast')
            ema_s = _iv('ema_slow')
            if ema_f and ema_s:
                print(f"  EMA({config.EMA_FAST}): ${ema_f:.2f}  |  EMA({config.EMA_SLOW}): ${ema_s:.2f}")
            support = _iv('support')
            resistance = _iv('resistance')
            if support:
                print(f"  Soporte: ${support:.2f}  |  Resistencia: ${resistance:.2f}" if resistance else f"  Soporte: ${support:.2f}")
            if indicators.get('patterns'):
                print(f"  Patrón: {', '.join(indicators['patterns'])}")
            if indicators.get('signals'):
                print(f"  📶 {', '.join(indicators['signals'])}")
        
        sent = self.engine.last_sentiment_score
        print(f"  📰 Sentimiento: {sent.get('label', 'N/A')} ({sent.get('score') or 0.0:.2f})")
        
        # Señal
        if signal.side == "BUY":
            print(f"  🟢 SEÑAL: {signal.side} (confianza: {signal.confidence:.1%})")
        elif signal.side == "SELL":
            print(f"  🔴 SEÑAL: {signal.side} (confianza: {signal.confidence:.1%})")
        else:
            print(f"  ⚪ SEÑAL: HOLD (score: {signal.confidence:.1%})")
        
        print(f"  Razón: {signal.reason}")
        if signal.stop_loss and signal.entry_price:
            print(f"  Stop-Loss: ${signal.stop_loss:.4f} ({abs(1 - signal.stop_loss/signal.entry_price)*100:.2f}%)")
        if signal.take_profit and signal.entry_price:
            print(f"  Take-Profit: ${signal.take_profit:.4f} ({abs(1 - signal.take_profit/signal.entry_price)*100:.2f}%)")

    # ── Ejecutar señal ──
    def _execute_signal(self, signal: Signal, available_balance: float):
        """Ejecuta una orden basada en la señal generada."""
        try:
            symbol = signal.symbol
            price = signal.entry_price
            if not price or price == 0:
                print(f"  [WARN] Sin precio para {symbol}")
                return
            
            # Calcular cantidad
            position_size = available_balance * config.MAX_POSITION_SIZE_PCT
            
            # Convertir a cantidad del token
            quantity = position_size / price
            
            # Redondear según LOT_SIZE del símbolo en Binance Futures
            # stepSize por símbolo (verificado contra exchangeInfo)
            lot_sizes = {
                "BTCUSDT":    (3, 0.001),
                "ETHUSDT":    (3, 0.001),
                "SOLUSDT":    (2, 0.01),
                "BNBUSDT":    (3, 0.001),
                "XRPUSDT":    (1, 1.0),
                "ADAUSDT":    (1, 1.0),
                "DOGEUSDT":   (0, 1.0),
                "LTCUSDT":    (3, 0.001),
                "AAVEUSDT":   (3, 0.001),
                "NEARUSDT":   (1, 0.1),
            }
            decimals, min_qty = lot_sizes.get(symbol, (2, 0.01))
            quantity = max(round(quantity, decimals), min_qty)
            
            # Validar cantidad mínima
            min_notional = 5.0  # $5 mínimo para futuros
            if quantity * price < min_notional:
                print(f"  ⛔ Cantidad mínima no alcanzada: ${quantity * price:.2f} < $5")
                return
            
            # Si es BUY: abrir LONG
            # Si es SELL: abrir SHORT
            print(f"  🚀 Ejecutando {signal.side} {quantity} {symbol}...")
            
            status, result = self.api.place_order(
                symbol=symbol,
                side=signal.side,
                quantity=quantity,
                order_type="MARKET",
            )
            
            if status == 200:
                order_id = result.get("orderId", "?")
                print(f"  ✅ Orden ejecutada: ID {order_id}")
                print(f"     Precio: ${float(result.get('avgPrice', price)):.4f}")
                print(f"     Cantidad: {quantity} {symbol}")
                
                # Colocar stop-loss como orden Algo (necesario para Multi-Assets)
                if signal.stop_loss:
                    sl_side = "SELL" if signal.side == "BUY" else "BUY"
                    sl_status, sl_result = self.api.place_algo_order(
                        symbol=symbol,
                        side=sl_side,
                        quantity=quantity,
                        order_type="STOP_LOSS",
                        stop_price=signal.stop_loss,
                        reduce_only=True,
                    )
                    if sl_status == 200:
                        print(f"  ✅ Stop-Loss colocado: ${signal.stop_loss:.4f} (ID {sl_result.get('orderId', '?')})")
                    else:
                        print(f"  ⚠️ Stop-Loss falló: {sl_result}")
                
                # Colocar take-profit como orden Algo
                if signal.take_profit:
                    tp_side = "SELL" if signal.side == "BUY" else "BUY"
                    tp_status, tp_result = self.api.place_algo_order(
                        symbol=symbol,
                        side=tp_side,
                        quantity=quantity,
                        order_type="TAKE_PROFIT",
                        stop_price=signal.take_profit,
                        reduce_only=True,
                    )
                    if tp_status == 200:
                        print(f"  ✅ Take-Profit colocado: ${signal.take_profit:.4f} (ID {tp_result.get('orderId', '?')})")
                    else:
                        print(f"  ⚠️ Take-Profit falló: {tp_result}")
                
                self.stats["trades_total"] += 1
            else:
                error_msg = result.get("msg", str(result))
                print(f"  ❌ Orden falló: {error_msg}")
                self.engine.record_loss()
                
        except Exception as e:
            print(f"  ❌ Error ejecutando orden: {e}")
            self.engine.record_loss()

    # ── Gestionar posiciones activas ──
    def _manage_positions(self):
        """Monitorea posiciones abiertas y actualiza trailing stops."""
        try:
            positions = self.api.get_positions()
            for pos in positions:
                symbol = pos.get("symbol", "")
                amt = float(pos.get("positionAmt", 0))
                if abs(amt) == 0:
                    continue
                
                entry = float(pos.get("entryPrice", 0))
                mark = float(pos.get("markPrice", 0))
                upnl = float(pos.get("unrealizedProfit", 0))
                
                if entry == 0:
                    continue
                
                # Calcular PnL %
                if amt > 0:  # LONG
                    pnl_pct = (mark - entry) / entry
                else:  # SHORT
                    pnl_pct = (entry - mark) / entry
                
                # Trailing stop: si ganancia > threshold, mover stop-loss
                if pnl_pct >= config.TRAILING_STOP_ACTIVATION and symbol in self.active_positions:
                    old_trailing = self.active_positions[symbol].get("trailing_activated", False)
                    if not old_trailing:
                        self.active_positions[symbol]["trailing_activated"] = True
                        print(f"  🎯 {symbol} Trailing stop activado (+{pnl_pct*100:.1f}%)")
                    
                    # Nueva distancia de stop
                    new_stop = mark * (1 - config.TRAILING_STOP_DISTANCE)
                    if amt < 0:  # SHORT
                        new_stop = mark * (1 + config.TRAILING_STOP_DISTANCE)
                    
                    self.active_positions[symbol]["trailing_stop"] = new_stop
                    print(f"  📍 {symbol} Trailing: ${new_stop:.4f}")
                
        except Exception as e:
            pass  # errores de gestión no críticos

    # ── Ejecutar ciclo continuo ──
    def run_forever(self, interval_seconds: int = 300):
        """Loop principal. Ejecuta análisis cada `interval_seconds`."""
        self.running = True
        print(f"\n{'=' * 50}")
        print(f"  NEXUS TRADER BOT — INICIADO")
        print(f"  Intervalo: {interval_seconds}s")
        print(f"  Símbolos: {', '.join(self.symbols)}")
        print(f"  Capital: ~$19.27 USDT")
        print(f"  Apalancamiento: {config.MAX_LEVERAGE}x")
        print(f"{'=' * 50}\n")
        
        while self.running:
            try:
                signals = self.run_once()
                
                # Mostrar resumen
                balance = self.api.get_balance()
                count_buy = sum(1 for s in signals.values() if s.side == "BUY")
                count_sell = sum(1 for s in signals.values() if s.side == "SELL")
                
                print(f"\n{'=' * 50}")
                print(f"  RESUMEN: BUY={count_buy} SELL={count_sell} HOLD={len(signals)-count_buy-count_sell}")
                avail = balance.get("available", 0)
                upnl = balance.get("unrealized_pnl", 0)
                total = avail + max(upnl, 0)
                print(f"  Balance: ${avail:.2f}  |  PnL No Realizado: ${upnl:.4f}  |  Total: ${total:.2f}")
                print(f"  Trades: {self.stats['trades_total']} | Wins: {self.stats['wins']} | Losses: {self.stats['losses']}")
                print(f"{'=' * 50}\n")
                
                # Guardar reporte
                self._save_report()
                
                # Esperar
                for remaining in range(interval_seconds, 0, -10):
                    if not self.running:
                        break
                    time.sleep(10)
                    
            except KeyboardInterrupt:
                print("\n[NEXUS] Deteniendo...")
                self.running = False
            except Exception as e:
                print(f"\n[ERROR] Ciclo principal: {e}")
                time.sleep(60)
    
    # ── Reporte ──
    def _save_report(self):
        """Guarda estado actual a JSON."""
        report = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "signals": {
                sym: {
                    "side": s.side,
                    "confidence": s.confidence,
                    "reason": s.reason,
                    "entry_price": s.entry_price,
                    "stop_loss": s.stop_loss,
                    "take_profit": s.take_profit,
                }
                for sym, s in self.last_signals.items()
            },
            "positions": self.active_positions,
            "stats": self.stats,
        }
        report_path = os.path.join(DATA_DIR, "latest_report.json")
        with open(report_path, "w") as f:
            json.dump(report, f, indent=2)
    
    def stop(self):
        self.running = False
