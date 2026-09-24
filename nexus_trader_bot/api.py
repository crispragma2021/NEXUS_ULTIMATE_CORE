"""
NEXUS TRADER BOT — API Binance (Futuros USDS-M)
Wrapper con firmas HMAC verificadas.
"""
import hmac, hashlib, time, json, ssl, urllib.request, urllib.parse
from typing import Dict, List, Optional, Tuple, Any
from . import config

class BinanceAPI:
    """Cliente API para Binance Futures (USDS-M) y Spot."""

    def __init__(self):
        self.api_key = config.API_KEY
        self.secret_key = config.SECRET_KEY
        self.spot_base = "https://api.binance.com"
        self.futures_base = "https://fapi.binance.com"
        self.ctx = ssl.create_default_context()
        self._server_time_offset = 0

    # ── Firma HMAC ──
    def _sign(self, params: Dict[str, str]) -> str:
        qs = '&'.join(f"{k}={v}" for k, v in sorted(params.items()))
        return hmac.new(self.secret_key.encode(), qs.encode(), hashlib.sha256).hexdigest()

    def _signed_request(self, base_url: str, endpoint: str,
                        params: Optional[Dict] = None, method: str = "GET") -> Tuple[int, Any]:
        if params is None:
            params = {}
        ts = str(int(time.time() * 1000) + self._server_time_offset)
        params["timestamp"] = ts
        params["recvWindow"] = "15000"
        params["signature"] = self._sign(params)
        qs = urllib.parse.urlencode(sorted(params.items()))
        url = f"{base_url}{endpoint}?{qs}"
        req = urllib.request.Request(url, method=method)
        req.add_header("X-MBX-APIKEY", self.api_key)
        try:
            with urllib.request.urlopen(req, context=self.ctx, timeout=15) as resp:
                return resp.status, json.loads(resp.read())
        except urllib.error.HTTPError as e:
            try:
                return e.code, json.loads(e.read())
            except:
                return e.code, {"error": e.read().decode()[:500]}
        except Exception as e:
            return 0, {"error": str(e)}

    def _public_get(self, url: str) -> Any:
        try:
            with urllib.request.urlopen(url, context=self.ctx, timeout=15) as resp:
                return json.loads(resp.read())
        except Exception as e:
            return {"error": str(e)}

    # ── Sincronización de tiempo ──
    def sync_time(self) -> int:
        data = self._public_get(f"{self.spot_base}/api/v3/time")
        server_ts = data.get("serverTime", 0)
        local_ts = int(time.time() * 1000)
        self._server_time_offset = server_ts - local_ts
        return self._server_time_offset

    # ── Klines (velas históricas) ──
    def get_klines(self, symbol: str, interval: str = "5m", limit: int = 500) -> List[Dict]:
        """Retorna velas OHLCV. Formato Binance: [time, open, high, low, close, volume, ...]"""
        url = f"{self.futures_base}/fapi/v1/klines?symbol={symbol}&interval={interval}&limit={limit}"
        data = self._public_get(url)
        if isinstance(data, list):
            result = []
            for k in data:
                result.append({
                    "time": k[0], "open": float(k[1]), "high": float(k[2]),
                    "low": float(k[3]), "close": float(k[4]), "volume": float(k[5]),
                    "close_time": k[6], "quote_volume": float(k[7]),
                    "trades": int(k[8]), "taker_buy_vol": float(k[9]),
                    "taker_buy_quote": float(k[10])
                })
            return result
        return []

    # ── Order Book ──
    def get_orderbook(self, symbol: str, limit: int = 20) -> Dict:
        url = f"{self.futures_base}/fapi/v1/depth?symbol={symbol}&limit={limit}"
        return self._public_get(url)

    # ── Funding Rate ──
    def get_funding_rate(self, symbol: str, limit: int = 50) -> List[Dict]:
        url = f"{self.futures_base}/fapi/v1/fundingRate?symbol={symbol}&limit={limit}"
        data = self._public_get(url)
        return data if isinstance(data, list) else []

    # ── Ticker 24h ──
    def get_ticker_24h(self, symbol: str) -> Dict:
        url = f"{self.futures_base}/fapi/v1/ticker/24hr?symbol={symbol}"
        return self._public_get(url)

    # ── Mark Price ──
    def get_mark_price(self, symbol: str) -> Dict:
        url = f"{self.futures_base}/fapi/v1/premiumIndex?symbol={symbol}"
        return self._public_get(url)

    # ── Posiciones abiertas ──
    def get_positions(self) -> List[Dict]:
        _, data = self._signed_request(self.futures_base, "/fapi/v2/account")
        if isinstance(data, dict) and "positions" in data:
            return [p for p in data["positions"] if float(p.get("positionAmt", 0)) != 0]
        return []

    # ── Balance ──
    def get_balance(self) -> Dict:
        _, data = self._signed_request(self.futures_base, "/fapi/v2/account")
        if isinstance(data, dict):
            return {
                "wallet_balance": float(data.get("totalWalletBalance", 0)),
                "unrealized_pnl": float(data.get("totalUnrealizedProfit", 0)),
                "margin_balance": float(data.get("totalMarginBalance", 0)),
                "available": float(data.get("availableBalance", 0)),
            }
        return {}

    # ── Órdenes abiertas ──
    def get_open_orders(self, symbol: Optional[str] = None) -> List[Dict]:
        params = {}
        if symbol:
            params["symbol"] = symbol
        _, data = self._signed_request(self.futures_base, "/fapi/v1/openOrders", params)
        return data if isinstance(data, list) else []

    # ── Crear orden (Futuros) ──
    def place_order(self, symbol: str, side: str, quantity: float,
                    order_type: str = "MARKET", price: Optional[float] = None,
                    stop_price: Optional[float] = None,
                    reduce_only: bool = False) -> Tuple[int, Any]:
        """Crea una orden en futuros USDS-M.
        
        Args:
            symbol: Par (ej: "SOLUSDT")
            side: "BUY" o "SELL"
            quantity: Cantidad en moneda base
            order_type: "MARKET", "LIMIT", "STOP_MARKET", "TAKE_PROFIT_MARKET"
            price: Precio (requerido para LIMIT)
            stop_price: Precio activación (para STOP/TAKE_PROFIT)
            reduce_only: Solo reducir posición
        """
        params = {
            "symbol": symbol,
            "side": side,
            "type": order_type,
            "quantity": str(quantity),
        }
        if price:
            params["price"] = str(price)
            params["timeInForce"] = "GTC"
        if stop_price:
            params["stopPrice"] = str(stop_price)
        if reduce_only:
            params["reduceOnly"] = "true"

        status, data = self._signed_request(
            self.futures_base, "/fapi/v1/order", params, method="POST"
        )
        return status, data

    # ── Crear orden de Algo (Stop-Loss / Take-Profit) ──
    def place_algo_order(self, symbol: str, side: str, quantity: float,
                         order_type: str = "STOP_LOSS",
                         stop_price: Optional[float] = None,
                         reduce_only: bool = False) -> Tuple[int, Any]:
        """Crea una orden de Algo (Stop-Loss / Take-Profit) en futuros USDS-M.
        
        Usa el endpoint /fapi/v1/algo/order necesario cuando la cuenta está
        en modo Multi-Assets (CROSS). Los tipos SOPORTADOS:
          - STOP_LOSS:   se activa como MARKET cuando el precio toca stopPrice
          - TAKE_PROFIT: se activa como MARKET cuando el precio toca stopPrice
        """
        params = {
            "symbol": symbol,
            "side": side,
            "quantity": str(quantity),
            "type": order_type,
        }
        if stop_price:
            params["stopPrice"] = str(stop_price)
        if reduce_only:
            params["reduceOnly"] = "true"
        
        status, data = self._signed_request(
            self.futures_base, "/fapi/v1/algo/order", params, method="POST"
        )
        return status, data

    # ── Cancelar orden ──
    def cancel_order(self, symbol: str, order_id: Optional[int] = None) -> Tuple[int, Any]:
        params = {"symbol": symbol}
        if order_id:
            params["orderId"] = str(order_id)
        return self._signed_request(
            self.futures_base, "/fapi/v1/order", params, method="DELETE"
        )

    # ── Cancelar TODAS las órdenes ──
    def cancel_all_orders(self, symbol: str) -> Tuple[int, Any]:
        params = {"symbol": symbol}
        return self._signed_request(
            self.futures_base, "/fapi/v1/allOpenOrders", params, method="DELETE"
        )

    # ── Setear modo de apalancamiento ──
    def set_leverage(self, symbol: str, leverage: int) -> Tuple[int, Any]:
        params = {"symbol": symbol, "leverage": str(leverage)}
        return self._signed_request(
            self.futures_base, "/fapi/v1/leverage", params, method="POST"
        )

    # ── Setear modo de margen (ISOLATED/CROSSED) ──
    def set_margin_type(self, symbol: str, margin_type: str = "ISOLATED") -> Tuple[int, Any]:
        params = {"symbol": symbol, "marginType": margin_type}
        return self._signed_request(
            self.futures_base, "/fapi/v1/marginType", params, method="POST"
        )

    def ping(self) -> bool:
        data = self._public_get(f"{self.spot_base}/api/v3/ping")
        return "error" not in data
