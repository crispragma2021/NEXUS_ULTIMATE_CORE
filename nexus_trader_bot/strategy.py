"""
NEXUS TRADER BOT — Estrategia y Decisión
Combina analysis técnico + sentimiento + funding rate.
Genera señales BUY/SELL/HOLD con confianza y razón.
"""
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass
from datetime import datetime, timezone
import time
from . import config
from .indicators import calculate_all_indicators
from .sentiment import MarketSentiment

@dataclass
class Signal:
    symbol: str
    side: str          # "BUY" | "SELL" | "HOLD"
    confidence: float   # 0.0 - 1.0
    reason: str
    indicators: Dict
    entry_price: Optional[float] = None
    stop_loss: Optional[float] = None
    take_profit: Optional[float] = None
    timestamp: str = ""
    
    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now(timezone.utc).isoformat()

class DecisionEngine:
    """Motor de decisión que consolida todos los análisis en señales de trading."""

    def __init__(self):
        self.sentiment = MarketSentiment()
        self.last_news = []
        self.last_sentiment_score = {"score": 0, "label": "neutral"}
        self.circuit_breaker_count = 0
        self.circuit_breaker_until = 0
        self.daily_loss = 0.0
        self.trades_today = 0
        self.last_signal = None

    def _check_circuit_breaker(self) -> bool:
        """True si el circuit breaker permite operar."""
        now = time.time()
        if self.circuit_breaker_count >= config.CIRCUIT_BREAKER_THRESHOLD:
            if now < self.circuit_breaker_until:
                return False
            else:
                self.circuit_breaker_count = 0
        return True

    def _score_multi_timeframe(self, klines_dict: Dict[str, List[Dict]]) -> Tuple[float, float, List[str]]:
        """Analiza múltiples timeframes y retorna (bullish_score, bearish_score, razones).
        
        Ponderación: 5m=1x, 15m=2x, 1h=3x, 4h=4x, 1d=5x
        """
        tf_weights = {"5m": 1, "15m": 2, "1h": 3, "4h": 4, "1d": 5}
        total_bullish = 0.0
        total_bearish = 0.0
        total_weight = 0.0
        all_reasons = []
        
        for tf, klines in klines_dict.items():
            if not klines or len(klines) < 50:
                continue
            
            weight = tf_weights.get(tf, 1)
            indicators = calculate_all_indicators(klines)
            
            if "error" in indicators:
                continue
            
            price = indicators["price"]
            rsi_val = indicators["rsi"]
            bb_pct = indicators["bb_percent_b"]
            macd_hist = indicators["macd_histogram"]
            ema_fast = indicators.get("ema_fast")
            ema_slow = indicators.get("ema_slow")
            support = indicators.get("support")
            resistance = indicators.get("resistance")
            
            tf_bullish = 0.0
            tf_bearish = 0.0
            tf_reasons = []
            
            # RSI
            if rsi_val < config.RSI_OVERSOLD:
                tf_bullish += 0.4 * (1 - rsi_val / config.RSI_OVERSOLD)
                tf_reasons.append(f"{tf} RSI oversold ({rsi_val:.0f})")
            elif rsi_val > config.RSI_OVERBOUGHT:
                tf_bearish += 0.4 * (rsi_val / 100 - config.RSI_OVERBOUGHT / 100) * 3.33
                tf_reasons.append(f"{tf} RSI overbought ({rsi_val:.0f})")
            
            # Bollinger %B
            if bb_pct < 0.05:
                tf_bullish += 0.3 * (1 - bb_pct / 0.05)
                tf_reasons.append(f"{tf} BB lower band touch")
            elif bb_pct > 0.95:
                tf_bearish += 0.3 * (bb_pct / 1.0 - 0.95) * 20
                tf_reasons.append(f"{tf} BB upper band touch")
            
            # MACD histogram momentum
            if macd_hist > 0:
                tf_bullish += 0.2 * min(macd_hist * 10, 1.0)
            elif macd_hist < 0:
                tf_bearish += 0.2 * min(abs(macd_hist) * 10, 1.0)
            
            # EMA alignment
            if ema_fast and ema_slow:
                if ema_fast > ema_slow:
                    tf_bullish += 0.2
                else:
                    tf_bearish += 0.2
            
            # Support/Resistance proximity
            if support and price > support:
                dist_to_support = (price - support) / price
                if dist_to_support < 0.01:
                    tf_bullish += 0.3  # rebote de soporte
        
            if resistance and price < resistance:
                dist_to_resistance = (resistance - price) / price
                if dist_to_resistance < 0.01:
                    tf_bearish += 0.3  # techo de resistencia
            
            total_bullish += tf_bullish * weight
            total_bearish += tf_bearish * weight
            total_weight += weight
            all_reasons.extend(tf_reasons)
        
        if total_weight == 0:
            return 0.0, 0.0, ["insufficient data"]
        
        return total_bullish / total_weight, total_bearish / total_weight, all_reasons

    def analyze(self, symbol: str, klines_dict: Dict[str, List[Dict]], 
                funding_rate_data: Optional[List[Dict]] = None) -> Signal:
        """Analiza un símbolo y genera señal de trading."""
        
        # 1. Análisis Técnico Multi-Timeframe
        bullish_score, bearish_score, tech_reasons = self._score_multi_timeframe(klines_dict)
        
        # 2. Sentimiento de noticias
        news_sentiment = self.last_sentiment_score.get("score", 0)
        
        # 3. Funding rate sentiment
        fund_sentiment = 0.0
        if funding_rate_data:
            fr_label = self.sentiment.get_funding_sentiment(funding_rate_data)
            if fr_label == "bullish":
                fund_sentiment = 0.2
            elif fr_label == "bearish":
                fund_sentiment = -0.2
        
        # 4. Score consolidado
        weights = {
            "technical": 0.60,
            "sentiment": 0.25,
            "funding": 0.15,
        }
        
        tech_score = (bullish_score - bearish_score)  # -1 a +1
        total_score = (
            tech_score * weights["technical"] +
            news_sentiment * weights["sentiment"] +
            fund_sentiment * weights["funding"]
        )
        
        # 5. Decisión final
        price = None
        if "5m" in klines_dict and klines_dict["5m"]:
            price = klines_dict["5m"][-1]["close"]
        
        indicators = calculate_all_indicators(klines_dict.get("5m", [])) if "5m" in klines_dict else {}
        
        # Inicializar valores por defecto
        side = "HOLD"
        confidence = 0.0
        stop_loss = None
        take_profit = None
        reason = f"HOLD (score=0.0)"
        
        if total_score > 0.25 and bullish_score > bearish_score:
            side = "BUY"
            confidence = min(abs(total_score), 1.0)
            
            if price:
                support = indicators.get("support")
                if support and support < price:
                    stop_loss = support * 0.995  # 0.5% debajo del soporte
                else:
                    stop_loss = price * (1 - config.STOP_LOSS_PCT)
                take_profit = price * (1 + config.TAKE_PROFIT_PCT)
            
            reason = f"BUY signal (conf={confidence:.1%}): " + "; ".join(tech_reasons[:3])
            
        elif total_score < -0.25 and bearish_score > bullish_score:
            side = "SELL"
            confidence = min(abs(total_score), 1.0)
            
            if price:
                resistance = indicators.get("resistance")
                if resistance and resistance > price:
                    stop_loss = resistance * 1.005
                else:
                    stop_loss = price * (1 + config.STOP_LOSS_PCT)
                take_profit = price * (1 - config.TAKE_PROFIT_PCT)
            
            reason = f"SELL signal (conf={confidence:.1%}): " + "; ".join(tech_reasons[:3])
        else:
            confidence = max(0.5 - abs(total_score), 0.0)
            reason = f"HOLD (score={total_score:.2f})"
        
        self.last_signal = Signal(
            symbol=symbol,
            side=side,
            confidence=confidence,
            reason=reason,
            indicators=indicators,
            entry_price=price,
            stop_loss=stop_loss,
            take_profit=take_profit,
        )
        
        return self.last_signal

    def can_trade(self, current_balance: float, open_positions: int, daily_pnl: float) -> Tuple[bool, str]:
        """Verifica si podemos operar dado el estado actual."""
        
        # Circuit breaker
        if not self._check_circuit_breaker():
            remaining = int(self.circuit_breaker_until - time.time())
            return False, f"Circuit breaker active ({remaining}s remaining)"
        
        # Límite de posiciones abiertas
        if open_positions >= config.MAX_OPEN_POSITIONS:
            return False, f"Max open positions ({config.MAX_OPEN_POSITIONS})"
        
        # Límite de pérdida diaria
        if self.daily_loss >= config.MAX_DAILY_LOSS_PCT * current_balance:
            return False, "Daily loss limit reached"
        
        # Balance mínimo
        if current_balance < config.MIN_ORDER_SIZE_USDT * 2:
            return False, f"Balance too low (${current_balance:.2f})"
        
        return True, "OK"

    def record_loss(self):
        """Registra una pérdida para el circuit breaker."""
        self.circuit_breaker_count += 1
        if self.circuit_breaker_count >= config.CIRCUIT_BREAKER_THRESHOLD:
            self.circuit_breaker_until = time.time() + config.CIRCUIT_BREAKER_TIMEOUT

    def record_win(self):
        """Reduce el contador del circuit breaker en racha ganadora."""
        if self.circuit_breaker_count > 0:
            self.circuit_breaker_count -= 1

    def update_news(self):
        """Actualiza el caché de noticias."""
        try:
            self.last_news = self.sentiment.scrape_news()
            self.last_sentiment_score = self.sentiment.market_sentiment_score(self.last_news)
        except Exception:
            pass
