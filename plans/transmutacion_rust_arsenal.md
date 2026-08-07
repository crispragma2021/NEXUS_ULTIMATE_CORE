# 🔱 PLAN DE TRANSMUTACIÓN RUST: Arsenal NEXUS

> **Objetivo:** Transmutar las librerías Python útiles de `venv_trading/` y `venv_osint/` a módulos Rust puros, integrados como submódulos nativos de NEXUS Core. Cero Python, máximo rendimiento.

---

## 📋 Índice de Transmutación

| Librería Python | Propósito | Módulo Rust Destino | Prioridad |
|----------------|-----------|-------------------|-----------|
| `ccxt` (997KB) | Conectar ~100 exchanges crypto | `core/src/arsenal/trading/ccxt_rs/` | 🔴 Alta |
| `darts` (2MB) | Predicción financiera (ARIMA, Prophet) | `core/src/arsenal/trading/darts_rs/` | 🔴 Alta |
| `torch` (1.2GB) | Deep Learning (trozos útiles) | `core/src/arsenal/ml/torch_rs/` (cranelift) | 🟡 Media |
| `aiohttp` (700KB) | HTTP asíncrono + WebSocket | `core/src/arsenal/net/aiohttp_rs/` (ya hay reqwest) | 🟢 Ya existe |
| `cloudscraper` (100KB) | Bypass Cloudflare | `core/src/arsenal/osint/cloudscraper_rs/` | 🔴 Alta |
| `finta` (100KB) | Indicadores financieros TA | `core/src/arsenal/trading/finta_rs/` | 🟡 Media |
| `holehe` (50KB) | OSINT email | `core/src/arsenal/osint/holehe_rs/` | 🟡 Media |
| `beautifulsoup` (500KB) | HTML parsing | `core/src/arsenal/osint/dom_rs/` | 🟢 Ya existe (scraper) |
| `networkx` (2MB) | Grafos de relaciones | `core/src/cerebro/synapse/` (ya existe) | 🟢 Ya existe |
| `flask` (1MB) | API web mínima | `core/src/infra/santuario_rs/` (ya hay axum) | 🟢 Ya existe |

---

## 🔴 Fase 1: ccxt-rs — Conector Crypto Nativo

### ¿Qué hace ccxt?
- API unificada para ~100 exchanges (Binance, Kraken, Coinbase, etc.)
- Órdenes, balances, velas, streams WebSocket

### Arquitectura Rust Propuesta

```
core/src/arsenal/trading/ccxt_rs/
├── mod.rs              # Re-export público
├── exchange/
│   ├── mod.rs          # Trait Exchange
│   ├── binance.rs      # Implementación Binance
│   ├── kraken.rs       # Implementación Kraken
│   └── coinbase.rs     # Implementación Coinbase
├── rest.rs             # Cliente HTTP genérico
├── ws.rs               # WebSocket unified
├── types.rs            # Tipos compartidos (Order, Ticker, OHLCV)
└── error.rs            # Error handling soberano
```

### Dependencias (ya existentes en Cargo.toml)
- `reqwest` → HTTP REST
- `tokio-tungstenite` → WebSocket
- `serde` / `serde_json` → parseo JSON

### Código ejemplo del trait `Exchange`:

```rust
// core/src/arsenal/trading/ccxt_rs/exchange/mod.rs
#[async_trait]
pub trait Exchange: Send + Sync {
    async fn fetch_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError>;
    async fn fetch_ohlcv(&self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<OHLCV>, ExchangeError>;
    async fn create_order(&self, symbol: &str, order: Order) -> Result<OrderResult, ExchangeError>;
    async fn fetch_balance(&self) -> Result<Balance, ExchangeError>;
    async fn watch_ticker(&self, symbol: &str) -> Pin<Box<dyn Stream<Item = Ticker> + Send>>;
}
```

---

## 🔴 Fase 2: darts-rs — Predicción Financiera Nativa

### ¿Qué hace darts?
- Modelos de series temporales: ARIMA, Exponential Smoothing, Prophet-style
- Forecast de precios, volúmenes, tendencias

### Arquitectura Rust

```
core/src/arsenal/trading/darts_rs/
├── mod.rs
├── arima.rs            # ARIMA puro (sin dependencias externas)
├── smoothing.rs        # Exponential Smoothing
├── prophet_lite.rs     # Prophet simplificado (descomposición estacional)
├── metrics.rs          # MSE, MAE, MAPE, SMAPE
└── types.rs            # TimeSeries, Forecast
```

### Implementación ARIMA (ya hay math en engine-puro)

```rust
// core/src/arsenal/trading/darts_rs/arima.rs
pub struct ARIMA {
    p: usize,  // autoregresivo
    d: usize,  // diferenciación
    q: usize,  // media móvil
    coefficients: Vec<f64>,
    residuals: Vec<f64>,
}

impl ARIMA {
    pub fn fit(&mut self, data: &[f64]) -> Result<(), DartsError>;
    pub fn predict(&self, steps: usize) -> Vec<f64>;
    pub fn forecast_intervals(&self, steps: usize, confidence: f64) -> Vec<(f64, f64)>;
}
```

### NOTA: `darts` original usa `torch` para Prophet y N-BEATS.
Para Prophet-style, implementamos descomposición estacional Fourier (puro math, sin torch).
Para N-BEATS, usamos el grafo sináptico de NEXUS Puro Engine (red neuronal emergente).

---

## 🔴 Fase 3: cloudscraper-rs — Bypass Cloudflare Nativo

### ¿Qué hace cloudscraper?
- Emula navegador real con TLS fingerprinting
- Resuelve challenge JS de Cloudflare
- Headers, cookies, user-agent rotación

### Arquitectura Rust

```
core/src/arsenal/osint/cloudscraper_rs/
├── mod.rs
├── tls_emulator.rs     # JA3 fingerprinting (ya hay en zeroclaw)
├── challenge.rs        # Resolución de challenges JS (via quickjs)
├── session.rs          # Manejo de cookies + sesiones
├── rotator.rs          # Rotación de fingerprints
└── error.rs
```

### NOTA: Ya tenemos `zeroclaw` en `bin/zeroclaw` que hace algo similar.
Este módulo lo integraría directamente, no desde cero.

---

## 🟡 Fase 4: finta-rs — Indicadores Financieros

### ¿Qué hace finta?
- ~80 indicadores técnicos (SMA, EMA, RSI, MACD, Bollinger, etc.)
- Análisis técnico sobre velas OHLCV

```
core/src/arsenal/trading/finta_rs/
├── mod.rs
├── trend.rs            # SMA, EMA, MACD, ADX
├── momentum.rs         # RSI, Stochastic, Williams %R
├── volatility.rs       # Bollinger, ATR, Keltner
├── volume.rs           # OBV, VWAP, MFI
└── types.rs            # Indicator, Signal
```

### Código ejemplo RSI (14 líneas de math puro):

```rust
pub fn rsi(prices: &[f64], period: usize) -> Vec<f64> {
    prices.windows(period + 1).map(|window| {
        let gains: f64 = window.windows(2).map(|w| (w[1] - w[0]).max(0.0)).sum();
        let losses: f64 = window.windows(2).map(|w| (w[0] - w[1]).max(0.0)).sum();
        let rs = gains / losses.max(f64::EPSILON);
        100.0 - (100.0 / (1.0 + rs))
    }).collect()
}
```

---

## 🟡 Fase 5: holehe-rs — OSINT Email Nativo

### ¿Qué hace holehe?
- Verifica si un email está registrado en ~120 servicios
- SMTP check + HTTP status code analysis

```
core/src/arsenal/osint/holehe_rs/
├── mod.rs
├── smtp_check.rs       # Verificación SMTP sin enviar email
├── services.rs         # Mapa de ~120 servicios con módulos
├── modules/
│   ├── adobe.rs
│   ├── facebook.rs
│   ├── github.rs
│   ├── twitter.rs
│   └── ... (generados desde JSON)
└── reporter.rs         # Reporte estructurado
```

---

## 📊 Prioridades de Implementación

| Fase | Módulo | Dependencias Externas | Esfuerzo | Valor |
|------|--------|----------------------|----------|-------|
| 1 | `ccxt_rs` (Binance + Kraken) | reqwest + serde | 3-4 días | 🏆 Trading real |
| 2 | `finta_rs` | math puro | 1-2 días | 📈 Indicadores |
| 3 | `cloudscraper_rs` | reqwest + quickjs | 2-3 días | 🕵️ Scraping |
| 4 | `darts_rs` (ARIMA + Smoothing) | math puro | 3-5 días | 🔮 Predicción |
| 5 | `holehe_rs` | reqwest | 2-3 días | 🕵️ OSINT |
| 6 | `ccxt_rs` (resto de exchanges) | reqwest | 2-3 días | 🏆 Más exchanges |
| 7 | `darts_rs` (Prophet + N-BEATS) | grafo sináptico | 5-7 días | 🔮 ML forecasting |

---

## 🔧 Reglas de Transmutación (Pilar 4)

1. **Cero dependencias externas nuevas** — solo `reqwest`, `serde`, `tokio` que ya existen
2. **Cero `unwrap()` y `expect()`** — error handling soberano con `Result<T, NEXUS_ERROR>`
3. **Cero Python** — ni siquiera para scripts de build
4. **Máximo rendimiento** — aprovechar el i7-12700F con `rayon` para parallelización
5. **Tests desde el día 1** — cada función con su test unitario
6. **Documentación `///`** — cada struct y función pública documentada

---

## 🚀 Siguiente Paso

¿Aprobado, Arquitecto? Si es así, cambio a modo CÓDIGO y empezamos con la **Fase 1: ccxt_rs** — el módulo de conexión a exchanges crypto en Rust puro. Haremos Binance primero (el más usado), luego Kraken y Coinbase.
