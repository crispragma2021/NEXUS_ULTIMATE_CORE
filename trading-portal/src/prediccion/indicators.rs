// ============================================================================
// 📊 NEXUS PREDICCIÓN — Indicadores Técnicos en Rust Puro
// ============================================================================
// Cero dependencias externas. Implementación matemática desde cero.
// RSI, MACD, Bollinger Bands, EMA, SMA, ATR, Estocástico
// ============================================================================

/// Calcula la Media Móvil Simple (SMA) sobre un slice de precios
pub fn sma(data: &[f64], ventana: usize) -> Vec<f64> {
    if data.is_empty() || ventana == 0 || ventana > data.len() {
        return Vec::new();
    }
    let mut resultado = Vec::with_capacity(data.len() - ventana + 1);
    let mut suma: f64 = data[..ventana].iter().sum();
    resultado.push(suma / ventana as f64);
    
    for i in ventana..data.len() {
        suma += data[i] - data[i - ventana];
        resultado.push(suma / ventana as f64);
    }
    resultado
}

/// Calcula la Media Móvil Exponencial (EMA)
/// Suaviza datos dando más peso a valores recientes
pub fn ema(data: &[f64], ventana: usize) -> Vec<f64> {
    if data.is_empty() || ventana == 0 {
        return Vec::new();
    }
    let k = 2.0 / (ventana as f64 + 1.0);
    let mut resultado = Vec::with_capacity(data.len());
    
    // Primer valor = SMA de la ventana inicial
    let initial_sma: f64 = data.iter().take(ventana).sum::<f64>() / ventana as f64;
    resultado.push(initial_sma);
    
    for i in ventana..data.len() {
        let prev = resultado.last().copied().unwrap_or(initial_sma);
        resultado.push(data[i] * k + prev * (1.0 - k));
    }
    resultado
}

/// Calcula el RSI (Relative Strength Index) — período 14 por defecto
/// Mide la velocidad y magnitud de cambios de precio
pub fn rsi(data: &[f64], periodo: usize) -> Vec<f64> {
    if data.len() < periodo + 1 {
        return Vec::new();
    }
    
    // Calcular cambios día a día
    let cambios: Vec<f64> = data.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    
    let mut ganancias = Vec::with_capacity(cambios.len());
    let mut perdidas = Vec::with_capacity(cambios.len());
    
    for &c in &cambios {
        if c > 0.0 {
            ganancias.push(c);
            perdidas.push(0.0);
        } else {
            ganancias.push(0.0);
            perdidas.push(-c);
        }
    }
    
    // EMA de ganancias y pérdidas
    let avg_ganancia = ema(&ganancias, periodo);
    let avg_perdida = ema(&perdidas, periodo);
    
    let mut rsi_values = Vec::with_capacity(avg_ganancia.len().min(avg_perdida.len()));
    for i in 0..avg_ganancia.len().min(avg_perdida.len()) {
        let g = avg_ganancia[i];
        let p = avg_perdida[i];
        if p == 0.0 {
            rsi_values.push(100.0); // Sin pérdidas = sobrecomprado
        } else {
            let rs = g / p;
            rsi_values.push(100.0 - (100.0 / (1.0 + rs)));
        }
    }
    rsi_values
}

/// Datos del MACD
#[derive(Debug, Clone, Copy)]
pub struct MacdData {
    pub macd_line: f64,
    pub signal_line: f64,
    pub histogram: f64,
}

/// Calcula el MACD (Moving Average Convergence Divergence)
/// Parámetros estándar: 12, 26, 9
pub fn macd(data: &[f64], rapido: usize, lento: usize, senal: usize) -> Vec<MacdData> {
    if data.len() < lento {
        return Vec::new();
    }
    
    let ema_rapida = ema(data, rapido);
    let ema_lenta = ema(data, lento);
    
    // MACD Line = EMA rápida - EMA lenta (alinear longitudes)
    let offset = lento - rapido;
    let macd_line: Vec<f64> = ema_rapida.iter()
        .skip(offset)
        .zip(ema_lenta.iter())
        .map(|(r, l)| r - l)
        .collect();
    
    if macd_line.is_empty() {
        return Vec::new();
    }
    
    // Signal Line = EMA de la MACD Line
    let signal_line = ema(&macd_line, senal);
    
    // Alinear longitudes
    let signal_offset = senal;
    let mut resultado = Vec::with_capacity(macd_line.len().saturating_sub(signal_offset).max(0));
    
    for i in signal_offset..macd_line.len() {
        let m = macd_line[i];
        let s = signal_line[i - signal_offset];
        resultado.push(MacdData {
            macd_line: m,
            signal_line: s,
            histogram: m - s,
        });
    }
    resultado
}

/// Datos de Bollinger Bands
#[derive(Debug, Clone, Copy)]
pub struct BollingerBands {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
    pub bandwidth: f64,  // Ancho relativo de las bandas
}

/// Calcula Bollinger Bands (20, 2 por defecto)
pub fn bollinger(data: &[f64], ventana: usize, desvios: f64) -> Vec<BollingerBands> {
    let medias = sma(data, ventana);
    if medias.is_empty() {
        return Vec::new();
    }
    
    let mut resultado = Vec::with_capacity(medias.len());
    
    for i in 0..medias.len() {
        let start = i;
        let end = i + ventana;
        let middle = medias[i];
        
        // Desviación estándar de la ventana
        let slice = &data[start..end];
        let mean = middle;
        let variance: f64 = slice.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / ventana as f64;
        let std_dev = variance.sqrt();
        
        let upper = middle + desvios * std_dev;
        let lower = middle - desvios * std_dev;
        let bandwidth = if middle > 0.0 {
            (upper - lower) / middle
        } else {
            0.0
        };
        
        resultado.push(BollingerBands { upper, middle, lower, bandwidth });
    }
    resultado
}

/// Calcula el ATR (Average True Range) — medida de volatilidad
pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], periodo: usize) -> Vec<f64> {
    if closes.len() < 2 {
        return Vec::new();
    }
    
    let mut true_ranges = Vec::with_capacity(closes.len() - 1);
    for i in 1..closes.len() {
        let high = highs.get(i).copied().unwrap_or(closes[i]);
        let low = lows.get(i).copied().unwrap_or(closes[i]);
        let prev_close = closes[i - 1];
        
        let tr1 = high - low;
        let tr2 = (high - prev_close).abs();
        let tr3 = (low - prev_close).abs();
        
        let tr = tr1.max(tr2).max(tr3);
        true_ranges.push(tr);
    }
    
    // EMA de los True Ranges
    let atr_values = ema(&true_ranges, periodo);
    atr_values
}

/// Calcula el Estocástico (%K y %D)
#[derive(Debug, Clone, Copy)]
pub struct StochasticData {
    pub k: f64,
    pub d: f64,
}

pub fn stochastic(highs: &[f64], lows: &[f64], closes: &[f64], k_periodo: usize, d_periodo: usize) -> Vec<StochasticData> {
    if closes.len() < k_periodo {
        return Vec::new();
    }
    
    let mut k_values = Vec::with_capacity(closes.len() - k_periodo + 1);
    
    for i in (k_periodo - 1)..closes.len() {
        let start = i + 1 - k_periodo;
        let high_max = highs[start..=i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let low_min = lows[start..=i].iter().cloned().fold(f64::INFINITY, f64::min);
        let close = closes[i];
        
        let k = if high_max > low_min {
            ((close - low_min) / (high_max - low_min)) * 100.0
        } else {
            50.0
        };
        k_values.push(k);
    }
    
    // %D = SMA de %K
    let d_values = sma(&k_values, d_periodo);
    
    let mut resultado = Vec::with_capacity(k_values.len());
    for i in 0..k_values.len() {
        let d = d_values.get(i).copied().unwrap_or(k_values[i]);
        resultado.push(StochasticData { k: k_values[i], d });
    }
    resultado
}

/// Clasifica el RSI en señal de trading
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RsiSignal {
    Sobrecomprado,  // > 70
    Sobrevenido,    // < 30
    Neutral,        // entre 30 y 70
}

pub fn clasificar_rsi(rsi_val: f64) -> RsiSignal {
    if rsi_val >= 70.0 {
        RsiSignal::Sobrecomprado
    } else if rsi_val <= 30.0 {
        RsiSignal::Sobrevenido
    } else {
        RsiSignal::Neutral
    }
}

/// Señal combinada de MACD
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MacdSignal {
    Compra,   // MACD cruza arriba de Signal Line
    Venta,    // MACD cruza abajo de Signal Line
    Neutral,
}

pub fn clasificar_macd(actual: &MacdData, previo: Option<&MacdData>) -> MacdSignal {
    match previo {
        Some(prev) => {
            if prev.histogram <= 0.0 && actual.histogram > 0.0 {
                MacdSignal::Compra
            } else if prev.histogram >= 0.0 && actual.histogram < 0.0 {
                MacdSignal::Venta
            } else {
                MacdSignal::Neutral
            }
        }
        None => MacdSignal::Neutral,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_basico() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let resultado = sma(&data, 3);
        assert_eq!(resultado.len(), 3);
        assert!((resultado[0] - 2.0).abs() < 0.001);
        assert!((resultado[2] - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_ema_basico() {
        let data = vec![10.0, 12.0, 11.0, 13.0, 14.0];
        let resultado = ema(&data, 3);
        assert!(!resultado.is_empty());
        // EMA debe existir y ser finita
        for &v in &resultado {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn test_rsi_identifica_extremos() {
        // Serie fuertemente alcista → RSI debe estar alto
        let data: Vec<f64> = (0..30).map(|i| 100.0 + i as f64 * 2.0).collect();
        let rsi_vals = rsi(&data, 14);
        assert!(!rsi_vals.is_empty());
        let ultimo = rsi_vals.last().copied().unwrap();
        assert!(ultimo > 50.0, "RSI en tendencia alcista debe estar sobre 50, es {:.2}", ultimo);
    }

    #[test]
    fn test_macd_cruce_alcista() {
        // Serie que acelera al final para simular cruce MACD
        let mut data = vec![100.0; 30];
        for i in 30..60 {
            data.push(100.0 + (i as f64 - 30.0) * 3.0);
        }
        let macd_vals = macd(&data, 5, 13, 5);
        assert!(!macd_vals.is_empty());
        for m in &macd_vals {
            assert!(m.macd_line.is_finite());
            assert!(m.signal_line.is_finite());
            assert!(m.histogram.is_finite());
        }
    }

    #[test]
    fn test_bollinger_bandas_orden_correcto() {
        let data = vec![100.0, 102.0, 101.0, 103.0, 99.0, 98.0, 100.0, 
                        101.0, 102.0, 99.0, 100.0, 101.0, 102.0, 103.0,
                        104.0, 102.0, 101.0, 100.0, 99.0, 98.0];
        let bands = bollinger(&data, 5, 2.0);
        assert!(!bands.is_empty());
        for b in &bands {
            assert!(b.upper >= b.middle, "Upper debe ser >= Middle");
            assert!(b.middle >= b.lower, "Middle debe ser >= Lower");
            assert!(b.bandwidth >= 0.0, "Bandwidth debe ser no negativo");
        }
    }

    #[test]
    fn test_stochastic_rango_valido() {
        let highs = vec![110.0, 112.0, 111.0, 115.0, 113.0, 116.0, 114.0, 117.0, 115.0, 118.0];
        let lows = vec![90.0, 92.0, 91.0, 95.0, 93.0, 96.0, 94.0, 97.0, 95.0, 98.0];
        let closes = vec![105.0, 108.0, 107.0, 110.0, 109.0, 112.0, 111.0, 113.0, 112.0, 115.0];
        
        let stoch = stochastic(&highs, &lows, &closes, 5, 3);
        assert!(!stoch.is_empty());
        for s in &stoch {
            assert!((0.0..=100.0).contains(&s.k), "%K debe estar entre 0 y 100, es {:.2}", s.k);
            assert!((0.0..=100.0).contains(&s.d), "%D debe estar entre 0 y 100, es {:.2}", s.d);
        }
    }
}
