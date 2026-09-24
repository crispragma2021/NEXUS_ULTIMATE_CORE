"""
NEXUS TRADER BOT — Configuración Central
"""
import json, os

CONFIG_PATH = os.path.join(os.path.dirname(__file__), '..', 'nexus_trader_config.json')
with open(CONFIG_PATH) as f:
    _cfg = json.load(f)

# ── Cuenta ──
API_KEY = _cfg['binance_api_key']
SECRET_KEY = _cfg['binance_secret_key']

# ── Mercado ──
WATCHLIST = _cfg.get('watchlist', ['BTCUSDT', 'ETHUSDT', 'SOLUSDT'])
TIMEFRAMES = ['5m', '15m', '1h', '4h', '1d']
KLINE_LIMIT = 500  # velas por timeframe

# ── Riesgo (ultra-conservador) ──
RISK = _cfg.get('risk', {})
MAX_POSITION_SIZE_PCT = 0.30      # 30% del capital por operación
STOP_LOSS_PCT = 0.02              # 2% stop-loss (menos agresivo)
TAKE_PROFIT_PCT = 0.06            # 6% take-profit (ratio 1:3)
MAX_LEVERAGE = 10                 # Apalancamiento 10x (configurado por el Arquitecto)
MAX_OPEN_POSITIONS = 1            # Solo 1 operación a la vez
MAX_DAILY_LOSS_PCT = 0.08         # 8% pérdida máxima diaria
TRAILING_STOP_ACTIVATION = 0.03   # 3% para activar trailing
TRAILING_STOP_DISTANCE = 0.015    # 1.5% distancia trailing

# ── Indicadores ──
RSI_PERIOD = 14
RSI_OVERSOLD = 30
RSI_OVERBOUGHT = 70
EMA_FAST = 9
EMA_SLOW = 21
MACD_FAST = 12
MACD_SLOW = 26
MACD_SIGNAL = 9
BB_PERIOD = 20
BB_STD = 2.0
SUPPORT_RESISTANCE_LOOKBACK = 100

# ── Sentimiento / Scraping ──
NEWS_SOURCES = [
    'https://www.coindesk.com/',
    'https://cointelegraph.com/',
    'https://cryptopanic.com/news/',
]
SENTIMENT_KEYWORDS_BULLISH = ['surge', 'rally', 'bullish', 'breakout', 'accumulation', 'upgrade', 'partnership', 'adoption']
SENTIMENT_KEYWORDS_BEARISH = ['crash', 'dump', 'bearish', 'liquidation', 'fud', 'hack', 'ban', 'regulation']

# ── Ejecución ──
CIRCUIT_BREAKER_THRESHOLD = 3     # 3 fallos consecutivos → pausa
CIRCUIT_BREAKER_TIMEOUT = 300     # 5 minutos de pausa
MIN_ORDER_SIZE_USDT = 5.0         # mínimo para futuros
SLIPPAGE_TOLERANCE = 0.001        # 0.1%

# ── HTTP ──
USER_AGENT = 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
REQUEST_DELAY = 1.5  # segundos entre requests
