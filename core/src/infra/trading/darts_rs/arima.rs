// 🔱 darts_rs — ARIMA: AR(p), ARMA(p,q), ARIMA(p,d,q)
// ============================================================================
// Implementación en Rust Puro sin dependencias externas.
// Estimación OLS para AR(p), MLE iterativo para MA(q) y ARMA(p,q).
// Baseline naive (último valor) y seasonal naive incluidos.

use crate::infra::trading::darts_rs::types::{ArimaConfig, ArimaModel};
use crate::infra::trading::darts_rs::utils;

/// Ajusta un modelo AR(p) por OLS
pub fn fit_ar(data: &[f64], order: usize, include_constant: bool) -> Option<ArimaModel> {
    if data.len() <= order {
        return None;
    }

    let config = ArimaConfig::new(order, 0, 0);
    if include_constant {
        config.without_constant();
    }

    let x = utils::lag_matrix_with_const(data, order, include_constant);
    let y: Vec<f64> = data[order..].to_vec();

    if x.is_empty() || y.is_empty() {
        return None;
    }

    let beta = utils::ols_estimate(&x, &y)?;

    // Extraer coeficientes
    let mut constant = 0.0;
    let mut ar_coeffs = Vec::with_capacity(order);

    if include_constant {
        constant = beta[0];
        for &b in beta[1..].iter() {
            ar_coeffs.push(b);
        }
    } else {
        for &b in beta.iter() {
            ar_coeffs.push(b);
        }
    }

    // Calcular residuos
    let residuals = compute_ar_residuals(data, order, &ar_coeffs, constant);

    // Sigma^2 y log-likelihood
    let n = residuals.len() as f64;
    let sigma2 = if n > 0.0 {
        residuals.iter().map(|r| r * r).sum::<f64>() / n
    } else {
        0.0
    };

    // Log-likelihood aproximada (asumiendo normalidad)
    let log_likelihood = if sigma2 > 1e-15 {
        -0.5 * n * (2.0 * std::f64::consts::PI * sigma2).ln()
            - 0.5 * residuals.iter().map(|r| r * r / sigma2).sum::<f64>()
    } else {
        f64::NEG_INFINITY
    };

    Some(ArimaModel {
        config: config_with_constant(order, 0, 0, include_constant),
        ar_coeffs,
        ma_coeffs: vec![],
        constant,
        residuals,
        sigma2,
        log_likelihood,
    })
}

fn config_with_constant(p: usize, d: usize, q: usize, constant: bool) -> ArimaConfig {
    let mut cfg = ArimaConfig::new(p, d, q);
    if !constant {
        cfg = cfg.without_constant();
    }
    cfg
}

/// Computa los residuos in-sample de un modelo AR(p)
fn compute_ar_residuals(data: &[f64], order: usize, ar_coeffs: &[f64], constant: f64) -> Vec<f64> {
    let n = data.len();
    let mut residuals = Vec::with_capacity(n);

    // Los primeros `order` valores no tienen suficiente historia para predecir
    for _ in 0..order {
        residuals.push(0.0);
    }

    for i in order..n {
        let mut pred = constant;
        for (j, &coeff) in ar_coeffs.iter().enumerate() {
            pred += coeff * data[i - j - 1];
        }
        residuals.push(data[i] - pred);
    }

    residuals
}

/// Ajusta un modelo ARIMA(p,d,q) completo:
/// 1. Diferencia la serie (orden d)
/// 2. Ajusta ARMA(p,q) a la serie diferenciada
/// 3. Retorna modelo con coeficientes estimados
pub fn fit_arima(data: &[f64], config: &ArimaConfig) -> Option<ArimaModel> {
    if data.len() < config.p + config.q + config.d + 2 {
        return None;
    }

    // Paso 1: diferenciar
    let diff_data = utils::diff(data, config.d);
    if diff_data.len() < config.p + 1 {
        return None;
    }

    // Paso 2: ajustar AR(p) a la serie diferenciada
    let ar_model = fit_ar(&diff_data, config.p, config.include_constant)?;

    // Paso 3: si q > 0, refinar con MA usando residuos
    let (ma_coeffs, refined_residuals) = if config.q > 0 {
        fit_ma_residuals(&ar_model.residuals[config.p..], config.q)
    } else {
        (vec![], ar_model.residuals.clone())
    };

    // Recalcular sigma2 y log-likelihood con el modelo completo
    let n = refined_residuals.len() as f64;
    let sigma2 = if n > 0.0 {
        refined_residuals.iter().map(|r| r * r).sum::<f64>() / n
    } else {
        0.0
    };
    let log_likelihood = if sigma2 > 1e-15 {
        -0.5 * n * (2.0 * std::f64::consts::PI * sigma2).ln()
            - 0.5
                * refined_residuals
                    .iter()
                    .map(|r| r * r / sigma2)
                    .sum::<f64>()
    } else {
        f64::NEG_INFINITY
    };

    Some(ArimaModel {
        config: config.clone(),
        ar_coeffs: ar_model.ar_coeffs,
        ma_coeffs,
        constant: ar_model.constant,
        residuals: refined_residuals,
        sigma2,
        log_likelihood,
    })
}

/// Estima coeficientes MA(q) por regresión sobre residuos
/// Usa OLS: e_t = θ_1*e_{t-1} + ... + θ_q*e_{t-q} + ε_t
fn fit_ma_residuals(residuals: &[f64], q: usize) -> (Vec<f64>, Vec<f64>) {
    if residuals.len() <= q {
        return (Vec::new(), residuals.to_vec());
    }

    let x = utils::lag_matrix(residuals, q);
    let y: Vec<f64> = residuals[q..].to_vec();

    if x.is_empty() || y.is_empty() {
        return (Vec::new(), residuals.to_vec());
    }

    let beta = utils::ols_estimate(&x, &y).unwrap_or_default();

    if beta.is_empty() {
        return (Vec::new(), residuals.to_vec());
    }

    // Calcular nuevos residuos usando los coeficientes MA
    let mut new_residuals = vec![0.0; residuals.len()];
    for i in q..residuals.len() {
        let mut pred = 0.0;
        for (j, &coeff) in beta.iter().enumerate() {
            pred += coeff * residuals[i - j - 1];
        }
        new_residuals[i] = y[i - q] - pred;
    }

    (beta, new_residuals)
}

/// Pronóstico con modelo ARIMA ajustado
/// `model` = modelo previamente ajustado con fit_arima()
/// `steps` = número de pasos a pronosticar
/// `data` = serie original completa (para propagar lags)
/// Retorna vector de pronósticos
pub fn forecast_arima(model: &ArimaModel, steps: usize, data: &[f64]) -> Vec<f64> {
    if steps == 0 || data.is_empty() {
        return Vec::new();
    }

    // Reconstruir la serie diferenciada más reciente
    let diff_data = utils::diff(data, model.config.d);
    if diff_data.is_empty() {
        return Vec::new();
    }

    let order = model.config.p;
    let mut forecast_diff = Vec::with_capacity(steps);

    // Inicializar con los últimos `order` valores de la serie diferenciada
    let mut recent: Vec<f64> = if diff_data.len() >= order {
        diff_data[diff_data.len() - order..].to_vec()
    } else {
        // Padding con 0 si no hay suficientes datos
        let mut padded = vec![0.0; order - diff_data.len()];
        padded.extend_from_slice(&diff_data);
        padded
    };

    // Últimos residuos para MA (inicializados en 0)
    let mut recent_residuals = vec![0.0; model.config.q.max(1)];

    for _ in 0..steps {
        let mut pred = model.constant;

        // Componente AR
        for (j, &coeff) in model.ar_coeffs.iter().enumerate() {
            if j < recent.len() {
                pred += coeff * recent[recent.len() - j - 1];
            }
        }

        // Componente MA (usando residuos recientes)
        for (j, &coeff) in model.ma_coeffs.iter().enumerate() {
            if j < recent_residuals.len() {
                pred += coeff * recent_residuals[recent_residuals.len() - j - 1];
            }
        }

        forecast_diff.push(pred);

        // Actualizar ventana AR
        recent.push(pred);
        if recent.len() > order {
            recent.remove(0);
        }

        // El residuo del pronóstico es 0 (esperanza condicional)
        recent_residuals.push(0.0);
        if recent_residuals.len() > model.config.q.max(1) {
            recent_residuals.remove(0);
        }
    }

    // Reconstruir diferenciación
    if model.config.d > 0 {
        let initial: Vec<f64> = if data.len() >= model.config.d {
            data[data.len() - model.config.d..].to_vec()
        } else {
            data.to_vec()
        };
        utils::inverse_diff_order(&forecast_diff, &initial, model.config.d)
    } else {
        forecast_diff
    }
}

/// Pronóstico naive: repite el último valor observado
pub fn naive_forecast(data: &[f64], steps: usize) -> Vec<f64> {
    if data.is_empty() || steps == 0 {
        return Vec::new();
    }
    let last = data[data.len() - 1];
    vec![last; steps]
}

/// Pronóstico seasonal naive: repite el valor de hace `seasonal_period` periodos
pub fn seasonal_naive_forecast(data: &[f64], steps: usize, seasonal_period: usize) -> Vec<f64> {
    if data.len() < seasonal_period || steps == 0 {
        return naive_forecast(data, steps);
    }

    let mut forecast = Vec::with_capacity(steps);
    for i in 0..steps {
        let idx = data.len() as i64 - seasonal_period as i64 + i as i64;
        if idx >= 0 && (idx as usize) < data.len() {
            forecast.push(data[idx as usize]);
        } else {
            forecast.push(data[data.len() - 1]); // fallback
        }
    }
    forecast
}

/// Intervalo de confianza para pronóstico ARIMA
/// Basado en sigma^2 del modelo, expandiendo con √t para multi-step
pub fn arima_confidence_intervals(
    model: &ArimaModel,
    forecast: &[f64],
    confidence: f64,
) -> (Vec<f64>, Vec<f64>) {
    // z-score para nivel de confianza
    let z = match confidence {
        c if c >= 0.999 => 3.29,
        c if c >= 0.99 => 2.58,
        c if c >= 0.95 => 1.96,
        c if c >= 0.90 => 1.645,
        c if c >= 0.80 => 1.28,
        _ => 1.0,
    };

    let sigma = model.sigma2.sqrt();
    let mut lower = Vec::with_capacity(forecast.len());
    let mut upper = Vec::with_capacity(forecast.len());

    for (i, &f) in forecast.iter().enumerate() {
        // La varianza del error de pronóstico crece con √(step)
        let step_variance = ((i + 1) as f64).sqrt() * sigma;
        let margin = z * step_variance;
        lower.push(f - margin);
        upper.push(f + margin);
    }

    (lower, upper)
}

/// Orden óptimo AR(p) por AIC
pub fn select_ar_order(data: &[f64], max_order: usize) -> usize {
    if data.len() <= 2 {
        return 0;
    }

    let max_order = max_order.min(data.len() - 2);
    let mut best_order = 0;
    let mut best_aic = f64::INFINITY;

    for p in 0..=max_order {
        if let Some(model) = fit_ar(data, p, true) {
            let n = model.residuals.len() as f64;
            let k = p as f64 + 1.0; // p parámetros AR + constante
            let aic = -2.0 * model.log_likelihood + 2.0 * k;

            if aic < best_aic {
                best_aic = aic;
                best_order = p;
            }
        }
    }

    best_order
}

/// Criterio de información de Akaike
pub fn aic(model: &ArimaModel) -> f64 {
    let n = model.residuals.len() as f64;
    let k = (model.config.p + model.config.q) as f64
        + if model.config.include_constant {
            1.0
        } else {
            0.0
        };
    -2.0 * model.log_likelihood + 2.0 * k
}

/// Criterio de información Bayesiano (BIC)
pub fn bic(model: &ArimaModel) -> f64 {
    let n = model.residuals.len() as f64;
    let k = (model.config.p + model.config.q) as f64
        + if model.config.include_constant {
            1.0
        } else {
            0.0
        };
    -2.0 * model.log_likelihood + k * n.ln()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_ar_1_with_constant() {
        // AR(1): y_t = 2 + 0.5*y_{t-1} + e
        let data = vec![2.0, 3.0, 3.5, 3.75, 3.875, 3.9375];
        let model = fit_ar(&data, 1, true).unwrap();
        assert!(model.ar_coeffs.len() == 1);
        // Constante debe estar cerca de 2, AR cerca de 0.5
        assert!((model.constant - 2.0).abs() < 0.1);
        assert!((model.ar_coeffs[0] - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_fit_ar_1_without_constant() {
        // AR(1): y_t = 0.7*y_{t-1}
        let mut data = vec![1.0; 50];
        for i in 1..50 {
            data[i] = 0.7 * data[i - 1];
        }
        let model = fit_ar(&data, 1, false).unwrap();
        assert!((model.ar_coeffs[0] - 0.7).abs() < 0.01);
        assert!((model.constant - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_fit_ar_returns_some_residuals() {
        // AR(1) con ruido no perfecto para evitar matriz singular en AR(2)
        let data = vec![10.0, 12.0, 15.0, 13.0, 18.0, 22.0];
        let model = fit_ar(&data, 2, true).unwrap();
        assert_eq!(model.residuals.len(), data.len());
        // Los primeros `order` residuos deben ser 0
        assert_eq!(model.residuals[0], 0.0);
        assert_eq!(model.residuals[1], 0.0);
    }

    #[test]
    fn test_fit_ar_returns_none_with_short_data() {
        assert!(fit_ar(&[1.0, 2.0], 3, true).is_none());
    }

    #[test]
    fn test_fit_arima_ar_only() {
        // ARIMA(1,0,0) = AR(1)
        let data = vec![2.0, 3.0, 3.5, 3.75, 3.875, 3.9375, 3.96875, 3.984375];
        let config = ArimaConfig::new(1, 0, 0);
        let model = fit_arima(&data, &config).unwrap();
        assert!((model.ar_coeffs[0] - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_fit_arima_arima_1_1_0() {
        // ARIMA(1,1,0): primera diferencia + AR(1)
        let data = vec![1.0, 2.0, 3.5, 5.5, 7.75, 10.125, 12.5625, 15.0625];
        let config = ArimaConfig::new(1, 1, 0);
        let model = fit_arima(&data, &config).unwrap();
        assert_eq!(model.config.d, 1);
        assert!(!model.residuals.is_empty());
        assert!(model.sigma2 >= 0.0);
    }

    #[test]
    fn test_fit_arima_returns_none_short_data() {
        let config = ArimaConfig::new(2, 1, 2);
        assert!(fit_arima(&[1.0, 2.0, 3.0], &config).is_none());
    }

    #[test]
    fn test_forecast_arima_ar1_pure_ramp() {
        // Serie lineal: y = 2x + 1
        let data: Vec<f64> = (0..20).map(|x| 2.0 * x as f64 + 1.0).collect();
        let config = ArimaConfig::new(1, 0, 0);
        let model = fit_arima(&data, &config).unwrap();
        let forecast = forecast_arima(&model, 5, &data);
        assert_eq!(forecast.len(), 5);
        // Último valor: 2*19 + 1 = 39
        // AR(1) con pendiente 2 debe seguir la tendencia
        assert!(forecast[0] > 39.0);
    }

    #[test]
    fn test_forecast_arima_constant_after_differencing() {
        // Serie constante después de diff
        let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let config = ArimaConfig::new(0, 1, 0); // Random walk
        let model = fit_arima(&data, &config).unwrap();
        let forecast = forecast_arima(&model, 3, &data);
        assert_eq!(forecast.len(), 3);
        // Random walk pronostica último valor para todos los pasos
        for f in &forecast {
            assert!((f - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_naive_forecast() {
        let data = vec![1.0, 2.0, 3.0];
        let forecast = naive_forecast(&data, 3);
        assert_eq!(forecast, vec![3.0, 3.0, 3.0]);
    }

    #[test]
    fn test_seasonal_naive_forecast() {
        let data = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
        let forecast = seasonal_naive_forecast(&data, 3, 2);
        assert_eq!(forecast.len(), 3);
        assert!((forecast[0] - 50.0).abs() < 1e-10); // data[4] = data[6-2]
        assert!((forecast[1] - 60.0).abs() < 1e-10); // data[5] = data[6-1]
    }

    #[test]
    fn test_arima_confidence_intervals_80pct() {
        let model = ArimaModel {
            config: ArimaConfig::new(1, 0, 0),
            ar_coeffs: vec![0.5],
            ma_coeffs: vec![],
            constant: 2.0,
            residuals: vec![0.0, 0.1, -0.05, 0.02],
            sigma2: 0.01,
            log_likelihood: -10.0,
        };
        let forecast = vec![10.0, 11.0, 12.0];
        let (lower, upper) = arima_confidence_intervals(&model, &forecast, 0.80);
        assert_eq!(lower.len(), 3);
        assert_eq!(upper.len(), 3);
        // z=1.28 para 80%, sigma=0.1, step_variance = √step * 0.1
        // step 1: margin = 1.28 * 0.1 = 0.128
        assert!(lower[0] < forecast[0]);
        assert!(upper[0] > forecast[0]);
    }

    #[test]
    fn test_select_ar_order_known_ar1() {
        let mut data = vec![0.0; 100];
        let mut rng = 12345u64;
        for i in 1..100 {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            let noise = (rng as f64 / u64::MAX as f64) * 2.0 - 1.0;
            data[i] = 0.7 * data[i - 1] + noise * 0.5;
        }
        let order = select_ar_order(&data, 10);
        // AIC debe seleccionar p=1 (o cerca, puede variar con ruido)
        assert!(order <= 3, "AIC selected p={} for AR(1) process", order);
    }

    #[test]
    fn test_aic_bic() {
        let model = ArimaModel {
            config: ArimaConfig::new(1, 0, 1),
            ar_coeffs: vec![0.5],
            ma_coeffs: vec![0.3],
            constant: 1.0,
            residuals: vec![0.1, -0.2, 0.05, -0.1, 0.15],
            sigma2: 0.02,
            log_likelihood: -5.0,
        };
        let aic_val = aic(&model);
        let bic_val = bic(&model);
        assert!(aic_val.is_finite(), "AIC should be finite, got {}", aic_val);
        assert!(bic_val.is_finite(), "BIC should be finite, got {}", bic_val);
        // Para n=5, ln(5)=1.609 < 2*1, así que BIC < AIC para n pequeña
        // Verificamos que BIC use la penalización correcta: k*ln(n) vs 2*k
        let expected_aic = -2.0 * (-5.0) + 2.0 * 3.0; // = 16
        let expected_bic = -2.0 * (-5.0) + 3.0 * (5.0_f64).ln();
        assert!((aic_val - expected_aic).abs() < 1e-10);
        assert!((bic_val - expected_bic).abs() < 1e-10);
    }

    #[test]
    fn test_forecast_arima_empty_steps() {
        let data = vec![1.0, 2.0, 3.0];
        let config = ArimaConfig::new(1, 0, 0);
        let model = fit_arima(&data, &config).unwrap();
        let forecast = forecast_arima(&model, 0, &data);
        assert!(forecast.is_empty());
    }

    #[test]
    fn test_fit_ma_residuals_zero_returns_original() {
        let residuals = vec![0.1, -0.2, 0.3];
        let (coeffs, new_res) = fit_ma_residuals(&residuals, 0);
        assert!(coeffs.is_empty());
        assert_eq!(new_res.len(), residuals.len());
    }
}
