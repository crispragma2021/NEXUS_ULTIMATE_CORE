// 🔱 finta_rs — Indicadores técnicos de trading
// Implementaciones basadas en fórmulas TA-Lib/CCXT/finta
// Cada indicador opera sobre PriceSeries. Sin estado mutable global.

use super::types::*;

// ============================================================================
// MEDIAS MÓVILES
// ============================================================================

/// Simple Moving Average (Media Aritmética Simple)
pub fn sma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![f64::NAN; data.len()];
    }

    let mut result = vec![f64::NAN; data.len()];
    let mut sum: f64 = 0.0;

    // Ventana inicial
    for i in 0..period {
        sum += data[i];
    }
    result[period - 1] = sum / period as f64;

    // Ventana deslizante
    for i in period..data.len() {
        sum += data[i] - data[i - period];
        result[i] = sum / period as f64;
    }

    result
}

/// Exponential Moving Average (Media Móvil Exponencial)
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![f64::NAN; data.len()];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut result = vec![f64::NAN; data.len()];

    // Primer valor = SMA
    let mut sum: f64 = 0.0;
    for i in 0..period {
        sum += data[i];
    }
    result[period - 1] = sum / period as f64;

    // EMA recursivo
    for i in period..data.len() {
        result[i] = (data[i] - result[i - 1]) * multiplier + result[i - 1];
    }

    result
}

/// Weighted Moving Average (Media Móvil Ponderada)
pub fn wma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![f64::NAN; data.len()];
    }

    let mut result = vec![f64::NAN; data.len()];
    let weight_sum = (period * (period + 1)) as f64 / 2.0;

    for i in (period - 1)..data.len() {
        let mut sum = 0.0;
        for j in 0..period {
            sum += data[i - j] * (period - j) as f64;
        }
        result[i] = sum / weight_sum;
    }

    result
}

// ============================================================================
// RSI — Relative Strength Index
// ============================================================================

/// Relative Strength Index (Wilder's RSI)
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period + 1 || period == 0 {
        return vec![f64::NAN; data.len()];
    }

    let mut result = vec![f64::NAN; data.len()];

    // Primeros cambios
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;

    for i in 1..=period {
        let diff = data[i] - data[i - 1];
        if diff > 0.0 {
            avg_gain += diff;
        } else {
            avg_loss -= diff;
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;

    // Evitar división por cero
    // Si ambos son 0 → sin movimiento → RSI = 50 (neutral)
    if avg_loss == 0.0 {
        result[period] = if avg_gain > 0.0 { 100.0 } else { 50.0 };
    } else {
        let rs = avg_gain / avg_loss;
        result[period] = 100.0 - (100.0 / (1.0 + rs));
    }

    // RSI recursivo con Wilder smoothing
    for i in (period + 1)..data.len() {
        let diff = data[i] - data[i - 1];
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };

        avg_gain = ((avg_gain * (period - 1) as f64) + gain) / period as f64;
        avg_loss = ((avg_loss * (period - 1) as f64) + loss) / period as f64;

        if avg_loss == 0.0 {
            result[i] = if avg_gain > 0.0 { 100.0 } else { 50.0 };
        } else {
            let rs = avg_gain / avg_loss;
            result[i] = 100.0 - (100.0 / (1.0 + rs));
        }
    }

    result
}

// ============================================================================
// MACD — Moving Average Convergence Divergence
// ============================================================================

/// MACD (Moving Average Convergence Divergence)
pub fn macd(
    data: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> MACDResult {
    let fast_ema = ema(data, fast_period);
    let slow_ema = ema(data, slow_period);

    // MACD line = fast_ema - slow_ema
    let macd_line: Vec<f64> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(f, s)| f - s)
        .collect();

    // Signal line = EMA de la MACD line
    let signal_line = ema(&macd_line, signal_period);

    // Histogram = MACD line - Signal line
    let histogram: Vec<f64> = macd_line
        .iter()
        .zip(signal_line.iter())
        .map(|(m, s)| m - s)
        .collect();

    MACDResult {
        macd_line,
        signal_line,
        histogram,
    }
}

// ============================================================================
// BOLLINGER BANDS
// ============================================================================

/// Bollinger Bands (Standard Deviation bands around SMA)
pub fn bollinger(data: &[f64], period: usize, std_dev: f64) -> BollingerBands {
    let middle = sma(data, period);
    let n = data.len();

    let mut upper = vec![f64::NAN; n];
    let mut lower = vec![f64::NAN; n];

    for i in (period - 1)..n {
        // Desviación estándar poblacional
        let mean = middle[i];
        let mut variance = 0.0;
        for j in (i + 1 - period)..=i {
            let diff = data[j] - mean;
            variance += diff * diff;
        }
        variance /= period as f64;
        let sd = variance.sqrt();

        upper[i] = mean + std_dev * sd;
        lower[i] = mean - std_dev * sd;
    }

    BollingerBands {
        upper,
        middle,
        lower,
    }
}

// ============================================================================
// STOCHASTIC OSCILLATOR
// ============================================================================

/// Stochastic Oscillator (%K y %D)
pub fn stochastic(data: &PriceSeries, k_period: usize, d_period: usize) -> StochasticResult {
    let n = data.len();
    let mut k_values = vec![f64::NAN; n];

    for i in (k_period - 1)..n {
        let mut highest_high = f64::NEG_INFINITY;
        let mut lowest_low = f64::INFINITY;

        for j in (i + 1 - k_period)..=i {
            if data.high[j] > highest_high {
                highest_high = data.high[j];
            }
            if data.low[j] < lowest_low {
                lowest_low = data.low[j];
            }
        }

        let range = highest_high - lowest_low;
        if range != 0.0 {
            k_values[i] = ((data.close[i] - lowest_low) / range) * 100.0;
        } else {
            k_values[i] = 50.0; // Sin movimiento, neutro
        }
    }

    // %D = SMA(3) de %K
    let d_values = sma(&k_values, d_period);

    StochasticResult {
        k: k_values,
        d: d_values,
    }
}

// ============================================================================
// ATR — Average True Range
// ============================================================================

/// Average True Range (Wilder's ATR)
pub fn atr(data: &PriceSeries, period: usize) -> Vec<f64> {
    let n = data.len();
    if n < period + 1 || period == 0 {
        return vec![f64::NAN; n];
    }

    // True Range
    let mut tr_values = Vec::with_capacity(n);
    tr_values.push(f64::NAN); // Primer TR no existe

    for i in 1..n {
        let high_low = data.high[i] - data.low[i];
        let high_close = (data.high[i] - data.close[i - 1]).abs();
        let low_close = (data.low[i] - data.close[i - 1]).abs();

        let tr = high_low.max(high_close).max(low_close);
        tr_values.push(tr);
    }

    // Primer ATR = SMA del TR inicial
    let mut atr_values = vec![f64::NAN; n];
    let mut sum = 0.0;
    for i in 1..=period {
        sum += tr_values[i];
    }
    atr_values[period] = sum / period as f64;

    // ATR recursivo con Wilder smoothing
    for i in (period + 1)..n {
        atr_values[i] = (atr_values[i - 1] * (period - 1) as f64 + tr_values[i]) / period as f64;
    }

    atr_values
}

// ============================================================================
// INDICADORES ADICIONALES
// ============================================================================

/// Commodity Channel Index
pub fn cci(data: &PriceSeries, period: usize) -> Vec<f64> {
    let n = data.len();
    if n < period || period == 0 {
        return vec![f64::NAN; n];
    }

    let mut result = vec![f64::NAN; n];
    let mut tp_sum = 0.0;
    let mut tp_values: Vec<f64> = Vec::with_capacity(n);

    // Typical Price = (H + L + C) / 3
    for i in 0..n {
        let tp = (data.high[i] + data.low[i] + data.close[i]) / 3.0;
        tp_values.push(tp);
    }

    for i in (period - 1)..n {
        let mut sma_tp = 0.0;
        for j in (i + 1 - period)..=i {
            sma_tp += tp_values[j];
        }
        sma_tp /= period as f64;

        let mut mean_dev = 0.0;
        for j in (i + 1 - period)..=i {
            mean_dev += (tp_values[j] - sma_tp).abs();
        }
        mean_dev /= period as f64;

        if mean_dev != 0.0 {
            result[i] = (tp_values[i] - sma_tp) / (0.015 * mean_dev);
        }
    }

    result
}

/// Williams %R
pub fn williams_r(data: &PriceSeries, period: usize) -> Vec<f64> {
    let n = data.len();
    if n < period || period == 0 {
        return vec![f64::NAN; n];
    }

    let mut result = vec![f64::NAN; n];

    for i in (period - 1)..n {
        let mut highest = f64::NEG_INFINITY;
        let mut lowest = f64::INFINITY;

        for j in (i + 1 - period)..=i {
            if data.high[j] > highest {
                highest = data.high[j];
            }
            if data.low[j] < lowest {
                lowest = data.low[j];
            }
        }

        let range = highest - lowest;
        if range != 0.0 {
            result[i] = ((highest - data.close[i]) / range) * -100.0;
        } else {
            result[i] = -50.0;
        }
    }

    result
}

/// On-Balance Volume (OBV)
pub fn obv(data: &PriceSeries) -> Vec<f64> {
    let n = data.len();
    if n < 2 {
        return vec![f64::NAN; n];
    }

    let mut result = vec![0.0; n];
    result[0] = data.volume[0];

    for i in 1..n {
        if data.close[i] > data.close[i - 1] {
            result[i] = result[i - 1] + data.volume[i];
        } else if data.close[i] < data.close[i - 1] {
            result[i] = result[i - 1] - data.volume[i];
        } else {
            result[i] = result[i - 1];
        }
    }

    result
}

/// Money Flow Index (MFI)
pub fn mfi(data: &PriceSeries, period: usize) -> Vec<f64> {
    let n = data.len();
    if n < period + 1 || period == 0 {
        return vec![f64::NAN; n];
    }

    let mut result = vec![f64::NAN; n];
    let mut mf_values: Vec<(f64, bool)> = Vec::with_capacity(n); // (money_flow, is_positive)

    // Typical Price * Volume
    for i in 0..n {
        let tp = (data.high[i] + data.low[i] + data.close[i]) / 3.0;
        let mf = tp * data.volume[i];
        // Necesitamos comparar con el TP anterior
        if i == 0 {
            mf_values.push((mf, true)); // neutro
        } else {
            let prev_tp = (data.high[i - 1] + data.low[i - 1] + data.close[i - 1]) / 3.0;
            mf_values.push((mf, tp >= prev_tp));
        }
    }

    for i in period..n {
        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;

        for j in (i + 1 - period)..=i {
            let (mf, is_pos) = mf_values[j];
            if is_pos {
                positive_flow += mf;
            } else {
                negative_flow += mf;
            }
        }

        if negative_flow != 0.0 {
            let mfr = positive_flow / negative_flow;
            result[i] = 100.0 - (100.0 / (1.0 + mfr));
        } else if positive_flow > 0.0 {
            result[i] = 100.0; // Solo flujo positivo
        } else {
            result[i] = 0.0; // Sin flujo
        }
    }

    result
}

/// Rate of Change (ROC)
pub fn roc(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period + 1 || period == 0 {
        return vec![f64::NAN; data.len()];
    }

    let mut result = vec![f64::NAN; data.len()];
    for i in period..data.len() {
        let prev = data[i - period];
        if prev != 0.0 {
            result[i] = ((data[i] - prev) / prev) * 100.0;
        } else {
            result[i] = 0.0;
        }
    }

    result
}

// ============================================================================
// FUNCIÓN PRINCIPAL — Procesar un indicador por nombre
// ============================================================================

/// Procesa un indicador por nombre, devolviendo Result tipado
pub fn calcular_indicador(nombre: &str, data: &PriceSeries, period: usize) -> IndicatorResult {
    match nombre.to_lowercase().as_str() {
        "sma" | "ma" | "moving_average" => IndicatorResult {
            name: "SMA".into(),
            values: sma(&data.close, period),
            signal: None,
            extra: None,
        },
        "ema" | "exponential_moving_average" => IndicatorResult {
            name: "EMA".into(),
            values: ema(&data.close, period),
            signal: None,
            extra: None,
        },
        "rsi" | "relative_strength_index" => IndicatorResult {
            name: "RSI".into(),
            values: rsi(&data.close, period),
            signal: None,
            extra: None,
        },
        "macd" => {
            let m = macd(&data.close, 12, 26, 9);
            IndicatorResult {
                name: "MACD".into(),
                values: m.macd_line,
                signal: Some(m.signal_line),
                extra: Some(m.histogram),
            }
        }
        "bollinger" | "bb" | "bollinger_bands" => {
            let b = bollinger(&data.close, period, 2.0);
            IndicatorResult {
                name: "Bollinger".into(),
                values: b.middle,
                signal: Some(b.upper),
                extra: Some(b.lower),
            }
        }
        "stochastic" | "stoch" | "stoch_k" => {
            let s = stochastic(data, period, 3);
            IndicatorResult {
                name: "Stochastic".into(),
                values: s.k,
                signal: Some(s.d),
                extra: None,
            }
        }
        "atr" | "average_true_range" => IndicatorResult {
            name: "ATR".into(),
            values: atr(data, period),
            signal: None,
            extra: None,
        },
        "obv" | "on_balance_volume" => IndicatorResult {
            name: "OBV".into(),
            values: obv(data),
            signal: None,
            extra: None,
        },
        "mfi" | "money_flow_index" => IndicatorResult {
            name: "MFI".into(),
            values: mfi(data, period),
            signal: None,
            extra: None,
        },
        "cci" | "commodity_channel_index" => IndicatorResult {
            name: "CCI".into(),
            values: cci(data, period),
            signal: None,
            extra: None,
        },
        "williams_r" | "williams" | "%r" => IndicatorResult {
            name: "Williams %R".into(),
            values: williams_r(data, period),
            signal: None,
            extra: None,
        },
        "roc" | "rate_of_change" => IndicatorResult {
            name: "ROC".into(),
            values: roc(&data.close, period),
            signal: None,
            extra: None,
        },
        _ => IndicatorResult {
            name: format!("UNKNOWN: {nombre}"),
            values: vec![f64::NAN; data.len()],
            signal: None,
            extra: None,
        },
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_close() -> Vec<f64> {
        vec![
            44.0, 44.34, 44.47, 44.28, 44.07, 44.64, 45.0, 44.78, 44.55, 44.89, 45.17, 44.86,
            44.96, 45.07, 45.29, 45.3, 45.68, 45.99, 45.88, 45.53, 45.59, 45.75, 45.54, 45.18,
            45.12, 44.94, 44.88, 44.98, 45.03, 45.19,
        ]
    }

    fn sample_price_series() -> PriceSeries {
        PriceSeries::from_close(sample_close())
    }

    // ========== SMA ==========

    #[test]
    fn test_sma_period_3() {
        let data = sample_close();
        let result = sma(&data, 3);
        // indices manuales: avg(44.0, 44.34, 44.47) = 44.27
        assert!((result[2] - 44.27).abs() < 0.01);
        // avg(44.34, 44.47, 44.28) = 44.363
        assert!((result[3] - 44.363).abs() < 0.01);
    }

    #[test]
    fn test_sma_period_5_equals_last() {
        let data = sample_close();
        let result = sma(&data, 5);
        assert!(result[4].is_finite());
        // Los primeros 4 son NAN
        assert!(result[0].is_nan());
        assert!(result[1].is_nan());
        assert!(result[2].is_nan());
        assert!(result[3].is_nan());
    }

    #[test]
    fn test_sma_period_0_devuelve_nan() {
        let data = vec![1.0, 2.0, 3.0];
        let result = sma(&data, 0);
        assert!(result.iter().all(|x| x.is_nan()));
    }

    // ========== EMA ==========

    #[test]
    fn test_ema_inicia_con_sma() {
        let data = sample_close();
        let result = ema(&data, 5);
        // Primer valor EMA = SMA de primeros 5
        let sma5 = (44.0 + 44.34 + 44.47 + 44.28 + 44.07) / 5.0;
        assert!((result[4] - sma5).abs() < 0.001);
    }

    #[test]
    fn test_ema_posterior_difiere_de_sma() {
        let data = sample_close();
        let ema_result = ema(&data, 5);
        let sma_result = sma(&data, 5);
        // En índices más altos, EMA debe ser diferente de SMA
        // (EMA es más reactivo)
        assert!(ema_result[10] != sma_result[10]);
    }

    // ========== RSI ==========

    #[test]
    fn test_rsi_en_rango_valido() {
        let data = sample_close();
        let result = rsi(&data, 14);
        // Último valor debe estar entre 0 y 100
        let last = result[result.len() - 1];
        assert!(last >= 0.0 && last <= 100.0);
    }

    #[test]
    fn test_rsi_todos_constantes_da_50() {
        let data = vec![50.0; 20];
        let result = rsi(&data, 14);
        assert!((result[14] - 50.0).abs() < 0.001);
        assert!((result[19] - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_rsi_con_todo_al_alza() {
        // Precios subiendo constantemente => RSI alto
        let data: Vec<f64> = (0..30).map(|x| x as f64).collect();
        let result = rsi(&data, 14);
        let last = result[result.len() - 1];
        assert!(
            last > 50.0,
            "RSI should be > 50 when price only goes up, got: {}",
            last
        );
    }

    #[test]
    fn test_rsi_con_todo_a_la_baja() {
        let data: Vec<f64> = (0..30).rev().map(|x| x as f64).collect();
        let result = rsi(&data, 14);
        let last = result[result.len() - 1];
        assert!(
            last < 50.0,
            "RSI should be < 50 when price only goes down, got: {}",
            last
        );
    }

    // ========== MACD ==========

    #[test]
    fn test_macd_tamanos_coinciden() {
        let data = sample_close();
        let result = macd(&data, 12, 26, 9);
        assert_eq!(result.macd_line.len(), data.len());
        assert_eq!(result.signal_line.len(), data.len());
        assert_eq!(result.histogram.len(), data.len());
    }

    #[test]
    fn test_macd_histogram_es_resta() {
        let data = sample_close();
        let result = macd(&data, 12, 26, 9);
        for i in 0..data.len() {
            if !result.macd_line[i].is_nan() && !result.signal_line[i].is_nan() {
                let expected = result.macd_line[i] - result.signal_line[i];
                assert!((result.histogram[i] - expected).abs() < 0.0001);
            }
        }
    }

    // ========== Bollinger ==========

    #[test]
    fn test_bollinger_upper_sobre_middle() {
        let data = sample_close();
        let bands = bollinger(&data, 5, 2.0);
        for i in 0..data.len() {
            if bands.middle[i].is_finite() {
                assert!(
                    bands.upper[i] >= bands.middle[i],
                    "Upper band must be >= middle at index {}",
                    i
                );
                assert!(
                    bands.lower[i] <= bands.middle[i],
                    "Lower band must be <= middle at index {}",
                    i
                );
            }
        }
    }

    #[test]
    fn test_bollinger_data_constante_bands_iguales() {
        let data = vec![50.0; 20];
        let bands = bollinger(&data, 5, 2.0);
        for i in 4..data.len() {
            // Sin desviación, todas las bandas son iguales
            assert!((bands.upper[i] - 50.0).abs() < 0.001);
            assert!((bands.lower[i] - 50.0).abs() < 0.001);
            assert!((bands.middle[i] - 50.0).abs() < 0.001);
        }
    }

    // ========== Stochastic ==========

    #[test]
    fn test_stochastic_rango_valido() {
        let data = sample_price_series();
        let result = stochastic(&data, 14, 3);
        for i in 14..data.len() {
            assert!(result.k[i] >= 0.0 && result.k[i] <= 100.0);
            if result.d[i].is_finite() {
                assert!(result.d[i] >= 0.0 && result.d[i] <= 100.0);
            }
        }
    }

    // ========== ATR ==========

    #[test]
    fn test_atr_siempre_positivo() {
        let data = sample_price_series();
        let result = atr(&data, 14);
        for val in result.iter().skip(14) {
            assert!(
                val.is_nan() || *val > 0.0,
                "ATR debe ser positivo, got: {}",
                val
            );
        }
    }

    #[test]
    fn test_atr_data_constante_es_cero() {
        let data = PriceSeries {
            open: vec![10.0; 30],
            high: vec![10.0; 30],
            low: vec![10.0; 30],
            close: vec![10.0; 30],
            volume: vec![1000.0; 30],
        };
        let result = atr(&data, 14);
        for val in result.iter().skip(14) {
            assert!(
                val.is_nan() || (*val - 0.0).abs() < 0.001,
                "ATR sin movimiento debe ser 0, got: {}",
                val
            );
        }
    }

    // ========== OBV ==========

    #[test]
    fn test_obv_subiendo_con_precio() {
        let data = PriceSeries {
            open: vec![10.0; 10],
            high: vec![11.0; 10],
            low: vec![9.0; 10],
            close: (0..10).map(|x| 10.0 + x as f64).collect(), // siempre sube
            volume: vec![100.0; 10],
        };
        let result = obv(&data);
        // OBV debe ser creciente
        for i in 1..result.len() {
            assert!(
                result[i] > result[i - 1],
                "OBV debe crecer cuando precio sube, en {}: {} <= {}",
                i,
                result[i],
                result[i - 1]
            );
        }
    }

    // ========== CCI ==========

    #[test]
    fn test_cci_data_constante_es_cero() {
        let data = PriceSeries {
            open: vec![50.0; 30],
            high: vec![50.0; 30],
            low: vec![50.0; 30],
            close: vec![50.0; 30],
            volume: vec![1000.0; 30],
        };
        let result = cci(&data, 14);
        // CCI sin movimiento debe ser cercano a 0 o NAN
        let last = result[result.len() - 1];
        if last.is_finite() {
            assert!(
                last.abs() < 1.0,
                "CCI sin movimiento debe ser ~0, got: {}",
                last
            );
        }
    }

    // ========== MFI ==========

    #[test]
    fn test_mfi_en_rango_valido() {
        let data = sample_price_series();
        let result = mfi(&data, 14);
        for val in result.iter().skip(14) {
            if val.is_finite() {
                assert!(
                    *val >= 0.0 && *val <= 100.0,
                    "MFI debe estar entre 0 y 100, got: {}",
                    val
                );
            }
        }
    }

    // ========== ROC ==========

    #[test]
    fn test_roc_en_data_constante_es_cero() {
        let data = vec![50.0; 20];
        let result = roc(&data, 5);
        for val in result.iter().skip(5) {
            assert!(
                (*val - 0.0).abs() < 0.001,
                "ROC sin cambio debe ser 0, got: {}",
                val
            );
        }
    }

    // ========== WILLIAMS %R ==========

    #[test]
    fn test_williams_r_en_rango() {
        let data = sample_price_series();
        let result = williams_r(&data, 14);
        for val in result.iter().skip(14) {
            if val.is_finite() {
                assert!(
                    *val <= 0.0 && *val >= -100.0,
                    "Williams %R debe estar entre -100 y 0, got: {}",
                    val
                );
            }
        }
    }

    // ========== FUNCIÓN PRINCIPAL ==========

    #[test]
    fn test_calcular_indicador_rsi_por_nombre() {
        let data = sample_price_series();
        let result = calcular_indicador("rsi", &data, 14);
        assert_eq!(result.name, "RSI");
        assert_eq!(result.values.len(), data.len());
        assert!(result.values[result.values.len() - 1].is_finite());
    }

    #[test]
    fn test_calcular_indicador_unknown_devuelve_nan() {
        let data = sample_price_series();
        let result = calcular_indicador("nonexistent", &data, 14);
        assert!(result.values.iter().all(|x| x.is_nan()));
    }

    #[test]
    fn test_calcular_indicador_macd_returna_tres_series() {
        let data = sample_price_series();
        let result = calcular_indicador("macd", &data, 14);
        assert_eq!(result.name, "MACD");
        assert!(result.signal.is_some());
        assert!(result.extra.is_some());
    }

    // ========== WMA ==========

    #[test]
    fn test_wma_tamanio_correcto() {
        let data = sample_close();
        let result = wma(&data, 5);
        assert_eq!(result.len(), data.len());
    }

    // ========== PRICE SERIES ==========

    #[test]
    fn test_price_series_from_close() {
        let close = vec![10.0, 11.0, 12.0];
        let ps = PriceSeries::from_close(close.clone());
        assert_eq!(ps.close, close);
        assert_eq!(ps.open.len(), 3);
        assert_eq!(ps.len(), 3);
        assert!(!ps.is_empty());
    }

    #[test]
    fn test_price_series_empty() {
        let ps = PriceSeries::from_close(vec![]);
        assert!(ps.is_empty());
    }
}
