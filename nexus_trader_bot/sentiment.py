"""
NEXUS TRADER BOT — Scraping de Noticias y Análisis de Sentimiento
Extrae titulares de fuentes de crypto, analiza sentimiento y calcula impacto.
"""
import urllib.request, urllib.error, ssl, re, time, json
from typing import List, Dict, Tuple
from datetime import datetime, timezone
from . import config

class MarketSentiment:
    """Analiza el sentimiento del mercado desde múltiples fuentes."""

    def __init__(self):
        self.ctx = ssl.create_default_context()
        self.ctx.check_hostname = False
        self.ctx.verify_mode = ssl.CERT_NONE
        self.cache = {}
        self.cache_ttl = 300  # 5 minutos

    def _fetch_text(self, url: str) -> str:
        """Obtiene texto crudo de una URL."""
        req = urllib.request.Request(url, headers={"User-Agent": config.USER_AGENT})
        try:
            with urllib.request.urlopen(req, context=self.ctx, timeout=10) as resp:
                return resp.read().decode('utf-8', errors='replace')
        except Exception as e:
            return ""

    def _extract_headlines(self, html: str) -> List[str]:
        """Extrae titulares del HTML usando patrones comunes."""
        headlines = []
        # Buscar texto en <h1>-<h3> y <title>
        for tag in ['h1', 'h2', 'h3', 'title']:
            matches = re.findall(f'<{tag}[^>]*>(.*?)</{tag}>', html, re.IGNORECASE | re.DOTALL)
            for m in matches:
                text = re.sub(r'<[^>]+>', '', m).strip()
                if 15 < len(text) < 200 and any(kw in text.lower() for kw in 
                    ['bitcoin', 'btc', 'ethereum', 'eth', 'solana', 'sol', 'crypto', 'binance',
                     'market', 'price', 'bull', 'bear', 'rally', 'crash', 'altcoin']):
                    headlines.append(text)
        
        # Buscar en meta tags
        meta_matches = re.findall(r'<meta[^>]*content=["\'](.*?)["\']', html, re.IGNORECASE)
        for m in meta_matches:
            if 15 < len(m) < 300 and any(kw in m.lower() for kw in 
                ['bitcoin', 'crypto', 'market', 'trading']):
                headlines.append(m)
        
        return list(set(headlines))[:20]  # máx 20 titulares únicos

    def _analyze_sentiment(self, text: str) -> Tuple[float, str]:
        """Analiza sentimiento de un texto. Retorna (score, label).
        Score: -1.0 (bearish) a +1.0 (bullish)
        """
        text_lower = text.lower()
        bullish_score = 0
        bearish_score = 0
        
        # Palabras bullish con pesos
        bullish_words = {
            'surge': 0.5, 'rally': 0.6, 'bullish': 0.7, 'breakout': 0.8,
            'accumulation': 0.4, 'upgrade': 0.5, 'partnership': 0.4, 'adoption': 0.5,
            'launch': 0.3, 'buy': 0.3, 'support': 0.3, 'green': 0.3,
            'recovery': 0.4, 'positive': 0.3, 'growth': 0.4, 'gain': 0.3,
            'moon': 0.5, 'pump': 0.4, 'ATH': 0.6, 'high': 0.2, 'up': 0.1,
        }
        bearish_words = {
            'crash': 0.7, 'dump': 0.6, 'bearish': 0.7, 'liquidation': 0.6,
            'fud': 0.5, 'hack': 0.8, 'ban': 0.6, 'regulation': 0.4,
            'sell': 0.3, 'resistance': 0.3, 'red': 0.3, 'drop': 0.4,
            'decline': 0.4, 'negative': 0.3, 'loss': 0.4, 'fall': 0.3,
            'danger': 0.5, 'warn': 0.4, 'crash': 0.7, 'low': 0.2, 'down': 0.1,
        }
        
        for word, weight in bullish_words.items():
            if word in text_lower:
                bullish_score += weight * text_lower.count(word)
        
        for word, weight in bearish_words.items():
            if word in text_lower:
                bearish_score += weight * text_lower.count(word)
        
        total = bullish_score + bearish_score
        if total == 0:
            return 0.0, "neutral"
        
        score = (bullish_score - bearish_score) / total
        
        if score > 0.3:
            label = "bullish"
        elif score < -0.3:
            label = "bearish"
        else:
            label = "neutral"
        
        return score, label

    def scrape_news(self) -> List[Dict]:
        """Scrapea noticias de múltiples fuentes y retorna titulares analizados."""
        all_headlines = []
        
        for source_url in config.NEWS_SOURCES:
            try:
                html = self._fetch_text(source_url)
                if html:
                    headlines = self._extract_headlines(html)
                    for h in headlines:
                        score, label = self._analyze_sentiment(h)
                        all_headlines.append({
                            "source": source_url,
                            "headline": h,
                            "sentiment_score": score,
                            "sentiment_label": label,
                            "timestamp": datetime.now(timezone.utc).isoformat(),
                        })
                time.sleep(config.REQUEST_DELAY)
            except Exception:
                continue
        
        return all_headlines

    def market_sentiment_score(self, headlines: List[Dict]) -> Dict:
        """Calcula score agregado de sentimiento del mercado."""
        if not headlines:
            return {"score": 0.0, "label": "neutral", "count": 0, "bullish_count": 0, "bearish_count": 0}
        
        scores = [h["sentiment_score"] for h in headlines if h["sentiment_score"] != 0]
        if not scores:
            return {"score": 0.0, "label": "neutral", "count": len(headlines), "bullish_count": 0, "bearish_count": 0}
        
        avg_score = sum(scores) / len(scores)
        bullish = sum(1 for s in scores if s > 0.3)
        bearish = sum(1 for s in scores if s < -0.3)
        
        if avg_score > 0.2:
            label = "bullish"
        elif avg_score < -0.2:
            label = "bearish"
        else:
            label = "neutral"
        
        return {
            "score": avg_score,
            "label": label,
            "count": len(headlines),
            "bullish_count": bullish,
            "bearish_count": bearish,
        }

    def get_funding_sentiment(self, funding_rate_data: List[Dict]) -> str:
        """Analiza funding rate para determinar sentimiento.
        Funding rate positivo alto = mercado sobre-comprado (bearish).
        Funding rate negativo alto = mercado sobre-vendido (bullish).
        """
        if not funding_rate_data:
            return "neutral"
        
        rates = [float(fr.get("lastFundingRate", 0)) for fr in funding_rate_data[-10:]]
        if not rates:
            return "neutral"
        
        avg_rate = sum(rates) / len(rates)
        
        # Binance: funding rate cada 8h, típicamente 0.01%
        if avg_rate > 0.0005:  # > 0.05%
            return "bearish"  # caro mantener longs
        elif avg_rate < -0.0005:  # < -0.05%
            return "bullish"  # caro mantener shorts
        else:
            return "neutral"
