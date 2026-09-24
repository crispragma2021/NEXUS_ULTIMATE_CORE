"""
NEXUS TRADER BOT — Indicadores Técnicos y Análisis
RSI, MACD, EMA, Bollinger Bands, Soportes/Resistencias, Patrones de velas.
"""
import numpy as np
from typing import List, Dict, Tuple, Optional
from . import config

def sma(data: List[float], period: int) -> np.ndarray:
    """Simple Moving Average."""
    if len(data) < period:
        return np.array([])
    arr = np.array(data)
    cumsum = np.cumsum(arr, dtype=float)
    cumsum[period:] = cumsum[period:] - cumsum[:-period]
    result = np.empty_like(arr)
    result[:period-1] = np.nan
    result[period-1:] = cumsum[period-1:] / period
    return result

def ema(data: List[float], period: int) -> np.ndarray:
    """Exponential Moving Average."""
    if len(data) < period:
        return np.array([])
    arr = np.array(data, dtype=float)
    multiplier = 2.0 / (period + 1)
    result = np.empty_like(arr)
    result[:period-1] = np.nan
    result[period-1] = np.mean(arr[:period])
    for i in range(period, len(arr)):
        result[i] = (arr[i] - result[i-1]) * multiplier + result[i-1]
    return result

def rsi(data: List[float], period: int = 14) -> np.ndarray:
    """Relative Strength Index."""
    if len(data) < period + 1:
        return np.array([])
    arr = np.array(data, dtype=float)
    deltas = np.diff(arr)
    gains = np.where(deltas > 0, deltas, 0.0)
    losses = np.where(deltas < 0, -deltas, 0.0)
    avg_gain = np.empty_like(data, dtype=float)
    avg_loss = np.empty_like(data, dtype=float)
    avg_gain[:period] = np.nan
    avg_loss[:period] = np.nan
    avg_gain[period] = np.mean(gains[:period])
    avg_loss[period] = np.mean(losses[:period])
    for i in range(period + 1, len(data)):
        avg_gain[i] = (avg_gain[i-1] * (period - 1) + gains[i-1]) / period
        avg_loss[i] = (avg_loss[i-1] * (period - 1) + losses[i-1]) / period
    rs = np.divide(avg_gain, avg_loss, where=avg_loss != 0, out=np.full_like(avg_gain, 100))
    rsi_vals = 100 - (100 / (1 + rs))
    rsi_vals[avg_loss == 0] = 100
    rsi_vals[avg_gain == 0] = 0
    return rsi_vals

def macd(data: List[float], fast: int = 12, slow: int = 26, signal: int = 9) -> Dict:
    """MACD: macd_line, signal_line, histogram."""
    ema_fast = ema(data, fast)
    ema_slow = ema(data, slow)
    macd_line = ema_fast - ema_slow
    signal_line = ema(list(macd_line[~np.isnan(macd_line)]), signal) if len(macd_line[~np.isnan(macd_line)]) > signal else np.array([])
    # Alinear signal_line con macd_line
    if len(signal_line) > 0:
        padded_signal = np.full_like(macd_line, np.nan)
        start = len(macd_line) - len(signal_line)
        if start >= 0:
            padded_signal[start:] = signal_line
        signal_line = padded_signal
    histogram = macd_line - signal_line
    return {
        "macd": macd_line,
        "signal": signal_line,
        "histogram": histogram
    }

def bollinger(data: List[float], period: int = 20, std_dev: float = 2.0) -> Dict:
    """Bollinger Bands: middle, upper, lower, bandwidth, %b."""
    if len(data) < period:
        return {"middle": np.array([]), "upper": np.array([]), "lower": np.array([]), "bandwidth": 0, "percent_b": 0.5}
    arr = np.array(data, dtype=float)
    middle = sma(data, period)
    rolling_std = np.empty_like(arr)
    rolling_std[:period-1] = np.nan
    for i in range(period-1, len(arr)):
        rolling_std[i] = np.std(arr[i-period+1:i+1])
    upper = middle + std_dev * rolling_std
    lower = middle - std_dev * rolling_std
    last_mid = middle[-1]
    last_up = upper[-1]
    last_low = lower[-1]
    bandwidth = (last_up - last_low) / last_mid if last_mid != 0 else 0
    percent_b = (data[-1] - last_low) / (last_up - last_low) if (last_up - last_low) > 0 else 0.5
    return {
        "middle": middle, "upper": upper, "lower": lower,
        "bandwidth": bandwidth, "percent_b": percent_b,
        "last_middle": last_mid, "last_upper": last_up, "last_lower": last_low
    }

def find_support_resistance(prices: List[float], lookback: int = 100, tolerance: float = 0.005) -> Dict:
    """Encuentra niveles clave de soporte y resistencia.
    
    Args:
        prices: Lista de precios OHLC (usamos close)
        lookback: Ventana de búsqueda
        tolerance: Tolerancia para agrupar niveles cercanos (0.5% default)
    """
    if len(prices) < 50:
        return {"support": None, "resistance": None, "supports": [], "resistances": []}
    
    arr = np.array(prices[-lookback:])
    peaks = []
    troughs = []
    
    # Detectar picos y valles locales
    for i in range(2, len(arr) - 2):
        if arr[i] > arr[i-1] and arr[i] > arr[i-2] and arr[i] > arr[i+1] and arr[i] > arr[i+2]:
            peaks.append(arr[i])
        if arr[i] < arr[i-1] and arr[i] < arr[i-2] and arr[i] < arr[i+1] and arr[i] < arr[i+2]:
            troughs.append(arr[i])
    
    # Agrupar niveles cercanos
    def cluster_levels(levels: List[float]) -> List[float]:
        if not levels:
            return []
        sorted_vals = sorted(levels)
        clusters = []
        current_cluster = [sorted_vals[0]]
        for v in sorted_vals[1:]:
            if abs(v - np.mean(current_cluster)) / np.mean(current_cluster) < tolerance:
                current_cluster.append(v)
            else:
                clusters.append(np.mean(current_cluster))
                current_cluster = [v]
        clusters.append(np.mean(current_cluster))
        return clusters
    
    resistance_levels = cluster_levels(peaks)
    support_levels = cluster_levels(troughs)
    
    current_price = prices[-1]
    
    # Resistencia más cercana POR ENCIMA
    nearest_resistance = min([r for r in resistance_levels if r > current_price], default=None)
    # Soporte más cercano POR DEBAJO
    nearest_support = max([s for s in support_levels if s < current_price], default=None)
    
    return {
        "support": nearest_support,
        "resistance": nearest_resistance,
        "supports": support_levels[-5:] if support_levels else [],  # últimos 5 niveles
        "resistances": resistance_levels[-5:] if resistance_levels else [],
    }

def volume_profile(klines: List[Dict], num_bins: int = 20) -> Dict:
    """Punto de control de volumen (VWAP de rango)."""
    if not klines:
        return {"poc": None, "vwap": None}
    
    prices = np.array([(k["high"] + k["low"]) / 2 for k in klines])
    volumes = np.array([k["volume"] for k in klines])
    
    vwap = np.average(prices, weights=volumes) if np.sum(volumes) > 0 else prices[-1]
    
    # Punto de Control (precio con mayor volumen)
    min_p, max_p = np.min(prices), np.max(prices)
    if max_p == min_p:
        return {"poc": prices[-1], "vwap": vwap}
    bin_width = (max_p - min_p) / num_bins
    bins = np.zeros(num_bins)
    for p, v in zip(prices, volumes):
        idx = min(int((p - min_p) / bin_width), num_bins - 1)
        bins[idx] += v
    poc_bin = np.argmax(bins)
    poc_price = min_p + (poc_bin + 0.5) * bin_width
    
    return {"poc": poc_price, "vwap": vwap}

def detect_candle_patterns(kline: Dict, prev_kline: Dict) -> List[str]:
    """Detecta patrones básicos de velas."""
    patterns = []
    o, h, l, c = kline["open"], kline["high"], kline["low"], kline["close"]
    po, pc = prev_kline["open"], prev_kline["close"]
    body = abs(c - o)
    upper_wick = h - max(o, c)
    lower_wick = min(o, c) - l
    total_range = h - l
    
    if total_range == 0:
        return patterns
    
    # Doji
    if body / total_range < 0.1:
        patterns.append("doji")
    
    # Martillo (hammer)
    if body / total_range > 0.1 and lower_wick > body * 2 and upper_wick < body * 0.3:
        patterns.append("hammer")
    
    # Estrella fugaz (shooting star)
    if body / total_range > 0.1 and upper_wick > body * 2 and lower_wick < body * 0.3:
        patterns.append("shooting_star")
    
    # Envolvente alcista
    if c > o and po > pc and o < pc and c > po:
        patterns.append("bullish_engulfing")
    
    # Envolvente bajista
    if c < o and pc > po and o > po and c < pc:
        patterns.append("bearish_engulfing")
    
    # Marubozu (vela sin mecha)
    if body / total_range > 0.95:
        if c > o:
            patterns.append("marubozu_bullish")
        else:
            patterns.append("marubozu_bearish")
    
    return patterns

def calculate_all_indicators(klines: List[Dict]) -> Dict:
    """Calcula TODOS los indicadores para una lista de velas."""
    close = [k["close"] for k in klines]
    high = [k["high"] for k in klines]
    low = [k["low"] for k in klines]
    volume = [k["volume"] for k in klines]
    
    if len(close) < 50:
        return {"error": "insufficient data"}
    
    # Indicadores
    rsi_vals = rsi(close, config.RSI_PERIOD)
    macd_vals = macd(close, config.MACD_FAST, config.MACD_SLOW, config.MACD_SIGNAL)
    bb = bollinger(close, config.BB_PERIOD, config.BB_STD)
    ema_fast_vals = ema(close, config.EMA_FAST)
    ema_slow_vals = ema(close, config.EMA_SLOW)
    sr = find_support_resistance(close, config.SUPPORT_RESISTANCE_LOOKBACK)
    vp = volume_profile(klines)
    
    # Últimos valores
    last_rsi = rsi_vals[-1] if len(rsi_vals) > 0 else 50.0
    last_macd = macd_vals["macd"][-1] if len(macd_vals.get("macd", [])) > 0 else 0
    last_signal = macd_vals["signal"][-1] if len(macd_vals.get("signal", [])) > 0 else 0
    last_hist = macd_vals["histogram"][-1] if len(macd_vals.get("histogram", [])) > 0 else 0
    last_ema_fast = ema_fast_vals[-1] if len(ema_fast_vals) > 0 else None
    last_ema_slow = ema_slow_vals[-1] if len(ema_slow_vals) > 0 else None
    last_price = close[-1]
    
    # Patrones de velas (últimas 2)
    patterns = []
    if len(klines) >= 2:
        patterns = detect_candle_patterns(klines[-1], klines[-2])
    
    # Señales de indicadores
    signals = []
    
    # RSI
    if last_rsi < config.RSI_OVERSOLD:
        signals.append(f"RSI oversold ({last_rsi:.1f})")
    elif last_rsi > config.RSI_OVERBOUGHT:
        signals.append(f"RSI overbought ({last_rsi:.1f})")
    
    # MACD crossover
    if len(macd_vals["macd"]) >= 2 and len(macd_vals["signal"]) >= 2:
        prev_macd = macd_vals["macd"][-2]
        prev_signal = macd_vals["signal"][-2]
        if not np.isnan(prev_macd) and not np.isnan(prev_signal):
            if prev_macd < prev_signal and last_macd > last_signal:
                signals.append("MACD bullish cross")
            elif prev_macd > prev_signal and last_macd < last_signal:
                signals.append("MACD bearish cross")
    
    # EMA crossover
    if last_ema_fast is not None and last_ema_slow is not None:
        if len(ema_fast_vals) >= 2 and len(ema_slow_vals) >= 2:
            prev_fast = ema_fast_vals[-2]
            prev_slow = ema_slow_vals[-2]
            if not np.isnan(prev_fast) and not np.isnan(prev_slow):
                if prev_fast < prev_slow and last_ema_fast > last_ema_slow:
                    signals.append("EMA golden cross")
                elif prev_fast > prev_slow and last_ema_fast < last_ema_slow:
                    signals.append("EMA death cross")
    
    # Bollinger squeeze (bandwidth bajo)
    if bb["bandwidth"] < 0.05:
        signals.append(f"BB squeeze ({bb['bandwidth']:.3f})")
    
    # Price near support/resistance
    if sr["support"] and abs(last_price - sr["support"]) / last_price < 0.01:
        signals.append(f"Near support ${sr['support']:.2f}")
    if sr["resistance"] and abs(last_price - sr["resistance"]) / last_price < 0.01:
        signals.append(f"Near resistance ${sr['resistance']:.2f}")
    
    return {
        "price": last_price,
        "rsi": float(last_rsi),
        "macd": float(last_macd),
        "macd_signal": float(last_signal),
        "macd_histogram": float(last_hist),
        "bb_upper": float(bb["last_upper"]),
        "bb_middle": float(bb["last_middle"]),
        "bb_lower": float(bb["last_lower"]),
        "bb_bandwidth": float(bb["bandwidth"]),
        "bb_percent_b": float(bb["percent_b"]),
        "ema_fast": float(last_ema_fast) if last_ema_fast else None,
        "ema_slow": float(last_ema_slow) if last_ema_slow else None,
        "support": float(sr["support"]) if sr["support"] else None,
        "resistance": float(sr["resistance"]) if sr["resistance"] else None,
        "vwap": float(vp["vwap"]) if vp["vwap"] else None,
        "poc": float(vp["poc"]) if vp["poc"] else None,
        "patterns": patterns,
        "signals": signals,
    }
