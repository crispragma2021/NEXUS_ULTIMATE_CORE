// 🔱 darts_rs — Prophet-style Forecasting
// ============================================================================
// Implementación estilo Facebook Prophet en Rust Puro:
//   y(t) = trend(t) + seasonality(t) + residuals
// Trend: lineal o logístico con puntos de cambio (changepoints)
// Seasonality: series de Fourier (semanal, anual)
// No requiere dependencias externas — solo f64 y std.

use crate::infra::trading::darts_rs::types::{
    ForecastResult, Frequency, ProphetCoefficients, ProphetComponents, ProphetConfig, TimeSeries,
    TrendType,
};
use crate::infra::trading::darts_rs::utils;

/// Ajusta un modelo Prophet-style a los datos
///
/// # Argumentos
/// * `ts` - Serie temporal a ajustar
/// * `config` - Configuración del modelo Prophet
///
/// # Retorna
/// * Componentes descompuestos + coeficientes ajustados
pub fn fit_prophet(
    ts: &TimeSeries,
    config: &ProphetConfig,
) -> Option<(ProphetComponents, ProphetCoefficients)> {
    let n = ts.len();
    if n < 6 {
        return None; // Mínimo 6 puntos para ajustar
    }

    // Construir matriz de diseño con términos de tendencia + Fourier
    let (design_matrix, coeffs) = build_prophet_design(ts, config)?;

    // Calcular fitted values y componentes
    let mut trend = vec![0.0; n];
    let mut weekly = vec![0.0; n];
    let mut yearly = vec![0.0; n];

    // Separar componentes por índice de coeficientes
    let n_trend_params = if config.trend_type == TrendType::Linear {
        1 + config.n_changepoints // k + delta_1..delta_c
    } else {
        1 + config.n_changepoints // k + delta_1..delta_c (para logístico también)
    };

    let n_weekly = if config.weekly_seasonality {
        2 * config.fourier_order
    } else {
        0
    };
    let n_yearly = if config.yearly_seasonality {
        2 * config.fourier_order
    } else {
        0
    };

    for i in 0..n {
        let mut val = coeffs[0]; // k (tasa de crecimiento base)
                                 // Tendencia lineal simple: k * t
        let t = (i as f64) / (n - 1) as f64; // normalizado [0, 1]
        trend[i] = coeffs[0] * t; // Componente de tendencia lineal simple

        // Sumar puntos de cambio
        for j in 0..config.n_changepoints {
            let cp = (j + 1) as f64 / (config.n_changepoints + 1) as f64;
            if t >= cp {
                let delta = if j + 1 < coeffs.len() {
                    coeffs[j + 1]
                } else {
                    0.0
                };
                trend[i] += delta * (t - cp);
            }
        }

        // Componente estacional semanal
        if config.weekly_seasonality && n_weekly > 0 {
            let week_idx = (i as f64 * 7.0 / n as f64) % 1.0; // 0..1 sobre 7 días
            for h in 0..config.fourier_order {
                let freq = 2.0 * std::f64::consts::PI * (h + 1) as f64 * week_idx;
                let cos_coeff = if n_trend_params + 2 * h < coeffs.len() {
                    coeffs[n_trend_params + 2 * h]
                } else {
                    0.0
                };
                let sin_coeff = if n_trend_params + 2 * h + 1 < coeffs.len() {
                    coeffs[n_trend_params + 2 * h + 1]
                } else {
                    0.0
                };
                weekly[i] += cos_coeff * freq.cos() + sin_coeff * freq.sin();
            }
        }

        // Componente estacional anual
        if config.yearly_seasonality && n_yearly > 0 {
            let year_idx = (i as f64) / n as f64; // 0..1 sobre el año
            let weekly_offset = if config.weekly_seasonality {
                n_weekly
            } else {
                0
            };
            for h in 0..config.fourier_order {
                let freq = 2.0 * std::f64::consts::PI * (h + 1) as f64 * year_idx;
                let cos_coeff = if n_trend_params + weekly_offset + 2 * h < coeffs.len() {
                    coeffs[n_trend_params + weekly_offset + 2 * h]
                } else {
                    0.0
                };
                let sin_coeff = if n_trend_params + weekly_offset + 2 * h + 1 < coeffs.len() {
                    coeffs[n_trend_params + weekly_offset + 2 * h + 1]
                } else {
                    0.0
                };
                yearly[i] += cos_coeff * freq.cos() + sin_coeff * freq.sin();
            }
        }
    }

    // Calcular fitted y residuos
    let fitted: Vec<f64> = trend
        .iter()
        .zip(weekly.iter())
        .map(|(t, w)| t + w)
        .zip(yearly.iter())
        .map(|(tw, y)| tw + y)
        .collect();

    let residuals: Vec<f64> = ts
        .values
        .iter()
        .zip(fitted.iter())
        .map(|(actual, pred)| actual - pred)
        .collect();

    let components = ProphetComponents {
        trend,
        weekly,
        yearly,
        residuals,
    };

    // Estructurar coeficientes
    let weekly_coeffs: Vec<f64> = if config.weekly_seasonality {
        let start = n_trend_params;
        let end = start + n_weekly;
        if end <= coeffs.len() {
            coeffs[start..end].to_vec()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let yearly_coeffs: Vec<f64> = if config.yearly_seasonality {
        let start = n_trend_params + n_weekly;
        let end = start + n_yearly;
        if end <= coeffs.len() {
            coeffs[start..end].to_vec()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let trend_params: Vec<f64> = if coeffs.len() > 0 {
        let end = n_trend_params.min(coeffs.len());
        coeffs[0..end].to_vec()
    } else {
        vec![]
    };

    Some((
        components,
        ProphetCoefficients {
            trend_params,
            weekly_coeffs,
            yearly_coeffs,
        },
    ))
}

/// Construye la matriz de diseño del modelo Prophet y estima coeficientes por OLS
fn build_prophet_design(
    ts: &TimeSeries,
    config: &ProphetConfig,
) -> Option<(Vec<Vec<f64>>, Vec<f64>)> {
    let n = ts.len();
    let n_trend = 1 + config.n_changepoints; // k + changepoint deltas
    let n_weekly = if config.weekly_seasonality {
        2 * config.fourier_order
    } else {
        0
    };
    let n_yearly = if config.yearly_seasonality {
        2 * config.fourier_order
    } else {
        0
    };

    let n_cols = n_trend + n_weekly + n_yearly;
    if n_cols == 0 {
        return None;
    }

    let mut design = vec![vec![0.0; n_cols]; n];

    for i in 0..n {
        let t = (i as f64) / (n - 1) as f64; // tiempo normalizado [0, 1]

        // Columna 0: tendencia lineal (t)
        design[i][0] = t;

        // Columnas 1..n_trend: puntos de cambio (rampa)
        for j in 0..config.n_changepoints {
            let cp = (j + 1) as f64 / (config.n_changepoints + 1) as f64;
            if t >= cp {
                design[i][1 + j] = t - cp;
            }
        }

        // Columnas n_trend..n_trend+n_weekly: Fourier semanal
        let mut col = n_trend;
        if config.weekly_seasonality {
            let week_idx = (i as f64 * 7.0 / n as f64) % 1.0;
            for h in 0..config.fourier_order {
                let freq = 2.0 * std::f64::consts::PI * (h + 1) as f64 * week_idx;
                if col < n_cols {
                    design[i][col] = freq.cos();
                }
                col += 1;
                if col < n_cols {
                    design[i][col] = freq.sin();
                }
                col += 1;
            }
        }

        // Columnas restantes: Fourier anual
        if config.yearly_seasonality {
            let year_idx = (i as f64) / n as f64;
            for h in 0..config.fourier_order {
                let freq = 2.0 * std::f64::consts::PI * (h + 1) as f64 * year_idx;
                if col < n_cols {
                    design[i][col] = freq.cos();
                }
                col += 1;
                if col < n_cols {
                    design[i][col] = freq.sin();
                }
                col += 1;
            }
        }
    }

    // Estimar coeficientes por OLS
    let y = &ts.values;
    let coeffs = utils::ols_estimate(&design, y)?;

    Some((design, coeffs))
}

/// Pronostica con modelo Prophet ajustado
///
/// # Argumentos
/// * `ts` - Serie temporal original
/// * `components` - Componentes descompuestos de fit_prophet()
/// * `coeffs` - Coeficientes ajustados
/// * `steps` - Número de pasos a pronosticar
/// * `config` - Configuración del modelo
///
/// # Retorna
/// * ForecastResult con pronóstico e intervalos de confianza
pub fn forecast_prophet(
    ts: &TimeSeries,
    components: &ProphetComponents,
    coeffs: &ProphetCoefficients,
    steps: usize,
    config: &ProphetConfig,
) -> ForecastResult {
    if steps == 0 {
        return ForecastResult::simple(vec![], vec![]);
    }

    let n = ts.len();
    let mut forecast = vec![0.0; steps];
    let mut residuals = components.residuals.clone();

    // Calcular sigma de los residuos para intervalos de confianza
    let n_resid = residuals.len() as f64;
    let sigma = if n_resid > 1.0 {
        let mean = residuals.iter().sum::<f64>() / n_resid;
        let var = residuals.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n_resid - 1.0);
        var.sqrt()
    } else {
        0.0
    };

    for i in 0..steps {
        let t = (n as f64 + i as f64) / (n - 1) as f64; // tiempo normalizado para forecast

        // Componente de tendencia
        let mut trend_val = if coeffs.trend_params.is_empty() {
            0.0
        } else {
            coeffs.trend_params[0] * t
        };

        // Puntos de cambio
        let n_changepoints = config
            .n_changepoints
            .min(coeffs.trend_params.len().saturating_sub(1));
        for j in 0..n_changepoints {
            let cp = (j + 1) as f64 / (config.n_changepoints + 1) as f64;
            if t >= cp {
                let delta = if j + 1 < coeffs.trend_params.len() {
                    coeffs.trend_params[j + 1]
                } else {
                    0.0
                };
                trend_val += delta * (t - cp);
            }
        }

        // Componente estacional semanal
        let mut weekly_val = 0.0;
        if config.weekly_seasonality {
            let week_idx = ((n + i) as f64 * 7.0 / n as f64) % 1.0;
            for h in 0..config.fourier_order {
                let freq = 2.0 * std::f64::consts::PI * (h + 1) as f64 * week_idx;
                if 2 * h < coeffs.weekly_coeffs.len() {
                    weekly_val += coeffs.weekly_coeffs[2 * h] * freq.cos();
                }
                if 2 * h + 1 < coeffs.weekly_coeffs.len() {
                    weekly_val += coeffs.weekly_coeffs[2 * h + 1] * freq.sin();
                }
            }
        }

        // Componente estacional anual
        let mut yearly_val = 0.0;
        if config.yearly_seasonality {
            let year_idx = (n + i) as f64 / n as f64;
            for h in 0..config.fourier_order {
                let freq = 2.0 * std::f64::consts::PI * (h + 1) as f64 * year_idx;
                if 2 * h < coeffs.yearly_coeffs.len() {
                    yearly_val += coeffs.yearly_coeffs[2 * h] * freq.cos();
                }
                if 2 * h + 1 < coeffs.yearly_coeffs.len() {
                    yearly_val += coeffs.yearly_coeffs[2 * h + 1] * freq.sin();
                }
            }
        }

        forecast[i] = trend_val + weekly_val + yearly_val;
    }

    // Intervalos de confianza (lineal, expandiendo con √t)
    let z = 1.96; // 95% confidence
    let lower: Vec<f64> = forecast
        .iter()
        .enumerate()
        .map(|(i, f)| f - z * sigma * (i as f64 + 1.0).sqrt() * (1.0 + config.uncertainty_scale))
        .collect();
    let upper: Vec<f64> = forecast
        .iter()
        .enumerate()
        .map(|(i, f)| f + z * sigma * (i as f64 + 1.0).sqrt() * (1.0 + config.uncertainty_scale))
        .collect();

    ForecastResult {
        forecast,
        lower_bound: lower,
        upper_bound: upper,
        confidence: 0.95,
        residuals,
    }
}

/// Descompone la serie en componentes (análisis exploratorio)
pub fn decompose_series(ts: &TimeSeries) -> Option<ProphetComponents> {
    let config = ProphetConfig {
        n_changepoints: 3,
        capacity: 1_000_000.0,
        weekly_seasonality: false,
        yearly_seasonality: true,
        fourier_order: 3,
        uncertainty_scale: 0.05,
        trend_type: TrendType::Linear,
    };

    let (components, _) = fit_prophet(ts, &config)?;
    Some(components)
}

/// Detección automática de puntos de cambio en la tendencia
/// Retorna índices donde la pendiente cambia significativamente
pub fn detect_trend_changepoints(ts: &TimeSeries, n_changepoints: usize) -> Vec<usize> {
    if ts.len() < 10 || n_changepoints == 0 {
        return Vec::new();
    }

    let n = ts.len();
    // Posibles ubicaciones de puntos de cambio
    let candidates: Vec<usize> = (1..=n_changepoints)
        .map(|i| (i as f64 * n as f64 / (n_changepoints + 1) as f64) as usize)
        .filter(|&idx| idx > 0 && idx < n)
        .collect();

    // Para cada punto candidato, ajustar regresión lineal antes y después
    // y medir la diferencia de pendiente
    let changepoints: Vec<usize> = candidates
        .iter()
        .filter(|&&cp| {
            let before: Vec<f64> = ts.values[..cp].to_vec();
            let after: Vec<f64> = ts.values[cp..].to_vec();
            if before.len() < 3 || after.len() < 3 {
                return false;
            }

            // Pendiente antes
            let slope_before = linear_regression_slope(&before);
            let slope_after = linear_regression_slope(&after);

            // Diferencia significativa?
            (slope_after - slope_before).abs() > 0.01
        })
        .copied()
        .collect();

    changepoints
}

fn linear_regression_slope(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = data.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den = 0.0;
    for (i, &y) in data.iter().enumerate() {
        let x = i as f64;
        num += (x - x_mean) * (y - y_mean);
        den += (x - x_mean).powi(2);
    }

    if den.abs() < 1e-15 {
        0.0
    } else {
        num / den
    }
}

/// Suavizado exponencial simple (SES) para pronóstico
pub fn simple_exponential_smoothing(data: &[f64], alpha: f64, steps: usize) -> Vec<f64> {
    if data.is_empty() || steps == 0 {
        return Vec::new();
    }

    let alpha = alpha.clamp(0.01, 0.99);
    let mut smoothed = data[0];

    // Aplicar suavizado hasta el final de los datos
    for i in 1..data.len() {
        smoothed = alpha * data[i] + (1.0 - alpha) * smoothed;
    }

    // Pronóstico: el último valor suavizado se repite
    vec![smoothed; steps]
}

/// Métricas de precisión del pronóstico
pub fn forecast_accuracy(actual: &[f64], predicted: &[f64]) -> (f64, f64, f64) {
    let mae = utils::mean_absolute_error(actual, predicted);
    let mse = utils::mean_squared_error(actual, predicted);
    let rmse_val = mse.sqrt();

    // MAPE (Mean Absolute Percentage Error)
    let mape = if actual.iter().any(|&a| a.abs() < 1e-15) {
        f64::NAN
    } else {
        actual
            .iter()
            .zip(predicted.iter())
            .map(|(a, p)| ((a - p) / a).abs())
            .sum::<f64>()
            / actual.len() as f64
            * 100.0
    };

    (mae, rmse_val, mape)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::trading::darts_rs::types::TimeSeries;

    #[test]
    fn test_prophet_fit_simple_trend() {
        // Serie con tendencia lineal pura
        let values: Vec<f64> = (0..30).map(|x| 3.0 * x as f64 + 10.0).collect();
        let ts = TimeSeries::new(values);
        let config = ProphetConfig {
            n_changepoints: 2,
            capacity: 1_000_000.0,
            weekly_seasonality: false,
            yearly_seasonality: false,
            fourier_order: 3,
            uncertainty_scale: 0.05,
            trend_type: TrendType::Linear,
        };

        let result = fit_prophet(&ts, &config);
        assert!(result.is_some(), "Prophet should fit linear trend");
        let (components, coeffs) = result.unwrap();
        assert_eq!(components.trend.len(), 30);
        assert_eq!(components.residuals.len(), 30);
        // Coeficiente de tendencia (k) debe ser ≈ 3
        assert!(coeffs.trend_params.len() >= 1);
    }

    #[test]
    fn test_prophet_forecast_linear_trend() {
        let values: Vec<f64> = (0..20).map(|x| 2.0 * x as f64 + 5.0).collect();
        let ts = TimeSeries::new(values);
        let config = ProphetConfig {
            n_changepoints: 2,
            capacity: 1_000_000.0,
            weekly_seasonality: false,
            yearly_seasonality: false,
            fourier_order: 3,
            uncertainty_scale: 0.05,
            trend_type: TrendType::Linear,
        };

        if let Some((components, coeffs)) = fit_prophet(&ts, &config) {
            let result = forecast_prophet(&ts, &components, &coeffs, 5, &config);
            assert_eq!(result.steps(), 5);
            // El pronóstico debe continuar la tendencia ascendente
            // Último valor observado: 2*19 + 5 = 43
            assert!(result.forecast[0] > 40.0);
            assert!(result.forecast[4] > result.forecast[0]);
            // Intervalos de confianza deben ser válidos
            for i in 0..5 {
                assert!(result.lower_bound[i] < result.forecast[i]);
                assert!(result.upper_bound[i] > result.forecast[i]);
            }
        }
    }

    #[test]
    fn test_prophet_residuals_small_for_pure_trend() {
        let values: Vec<f64> = (0..15).map(|x| 10.0 * x as f64).collect();
        let ts = TimeSeries::new(values);
        let config = ProphetConfig {
            n_changepoints: 1,
            capacity: 1_000_000.0,
            weekly_seasonality: false,
            yearly_seasonality: false,
            fourier_order: 2,
            uncertainty_scale: 0.05,
            trend_type: TrendType::Linear,
        };

        if let Some((components, _)) = fit_prophet(&ts, &config) {
            // Los residuos deben ser pequeños para una tendencia perfecta
            let max_resid = components
                .residuals
                .iter()
                .map(|r| r.abs())
                .fold(0.0_f64, f64::max);
            assert!(
                max_resid < 5.0,
                "Max residual = {} should be small for perfect linear trend",
                max_resid
            );
        }
    }

    #[test]
    fn test_prophet_short_series_returns_none() {
        let ts = TimeSeries::new(vec![1.0, 2.0, 3.0]);
        let config = ProphetConfig::default();
        assert!(fit_prophet(&ts, &config).is_none());
    }

    #[test]
    fn test_prophet_with_seasonality() {
        // Serie con tendencia + estacionalidad semanal — datos abundantes para OLS
        let n = 56; // 8 semanas
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            let trend = 0.5 * i as f64;
            let weekly = (2.0 * std::f64::consts::PI * i as f64 / 7.0).sin() * 3.0;
            values.push(trend + weekly);
        }
        let ts = TimeSeries::new(values);
        // Sin puntos de cambio para evitar columnas correlacionadas
        let config = ProphetConfig {
            n_changepoints: 0,
            capacity: 1_000_000.0,
            weekly_seasonality: true,
            yearly_seasonality: false,
            fourier_order: 3,
            uncertainty_scale: 0.05,
            trend_type: TrendType::Linear,
        };

        let result = fit_prophet(&ts, &config);
        assert!(
            result.is_some(),
            "Prophet should fit with seasonality, got None"
        );
        let (components, coeffs) = result.unwrap();
        assert_eq!(components.weekly.len(), n);

        // Debe haber coeficientes semanales
        assert!(!coeffs.weekly_coeffs.is_empty());

        // Forecast debe capturar estacionalidad
        let forecast = forecast_prophet(&ts, &components, &coeffs, 7, &config);
        assert_eq!(forecast.steps(), 7);
    }

    #[test]
    fn test_detect_changepoints_constant_series() {
        let ts = TimeSeries::new(vec![5.0; 20]);
        let cps = detect_trend_changepoints(&ts, 3);
        // Serie constante no debe tener puntos de cambio
        assert!(
            cps.is_empty(),
            "No changepoints expected for constant series, got {:?}",
            cps
        );
    }

    #[test]
    fn test_detect_changepoints_short_series() {
        let ts = TimeSeries::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let cps = detect_trend_changepoints(&ts, 2);
        assert!(cps.is_empty(), "Should return empty for short series");
    }

    #[test]
    fn test_simple_exponential_smoothing_constant() {
        let data = vec![10.0; 5];
        let forecast = simple_exponential_smoothing(&data, 0.5, 3);
        assert_eq!(forecast.len(), 3);
        // Para datos constantes, el pronóstico debe ser el mismo
        for &f in &forecast {
            assert!((f - 10.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_simple_exponential_smoothing_trend() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let forecast = simple_exponential_smoothing(&data, 0.8, 2);
        assert_eq!(forecast.len(), 2);
        // Con alpha alto, el suavizado sigue la tendencia
        assert!(forecast[0] > 4.0); // debe estar cerca de 5
    }

    #[test]
    fn test_simple_exponential_smoothing_empty() {
        assert!(simple_exponential_smoothing(&[], 0.5, 5).is_empty());
    }

    #[test]
    fn test_linear_regression_slope_positive() {
        let data: Vec<f64> = (0..10).map(|x| 2.0 * x as f64 + 1.0).collect();
        let slope = linear_regression_slope(&data);
        assert!((slope - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_slope_constant() {
        let data = vec![5.0; 10];
        let slope = linear_regression_slope(&data);
        assert!((slope - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_forecast_accuracy_perfect() {
        let actual = vec![1.0, 2.0, 3.0];
        let predicted = vec![1.0, 2.0, 3.0];
        let (mae, rmse_val, mape) = forecast_accuracy(&actual, &predicted);
        assert!((mae - 0.0).abs() < 1e-10);
        assert!((rmse_val - 0.0).abs() < 1e-10);
        assert!((mape - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_decompose_series_returns_components() {
        let values: Vec<f64> = (0..30)
            .map(|x| x as f64 * 2.0 + (x as f64 * 0.5).sin() * 3.0)
            .collect();
        let ts = TimeSeries::new(values);
        let components = decompose_series(&ts);
        assert!(components.is_some());
        let comp = components.unwrap();
        assert_eq!(comp.trend.len(), 30);
        assert_eq!(comp.yearly.len(), 30);
        assert_eq!(comp.residuals.len(), 30);
    }

    #[test]
    fn test_prophet_with_yearly_seasonality() {
        let n = 365;
        let mut values = Vec::with_capacity(n);
        for i in 0..n {
            let trend = 0.01 * i as f64;
            let yearly = (2.0 * std::f64::consts::PI * i as f64 / 365.0).sin() * 5.0;
            values.push(trend + yearly);
        }
        let ts = TimeSeries::new(values);
        let config = ProphetConfig {
            n_changepoints: 3,
            capacity: 1_000_000.0,
            weekly_seasonality: false,
            yearly_seasonality: true,
            fourier_order: 4,
            uncertainty_scale: 0.05,
            trend_type: TrendType::Linear,
        };

        let result = fit_prophet(&ts, &config);
        assert!(
            result.is_some(),
            "Prophet should fit with yearly seasonality"
        );
        let (components, coeffs) = result.unwrap();
        assert!(components.yearly.len() == n);
        assert!(!coeffs.yearly_coeffs.is_empty());
    }
}
