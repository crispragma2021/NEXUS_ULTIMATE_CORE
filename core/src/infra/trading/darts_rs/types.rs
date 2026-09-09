// 🔱 darts_rs — Tipos de datos para forecasting
// ============================================================================

/// Serie temporal simple: vector de valores flotantes ordenados cronológicamente
#[derive(Debug, Clone)]
pub struct TimeSeries {
    pub values: Vec<f64>,
    pub freq: Frequency,
}

/// Frecuencia de muestreo de la serie temporal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Frequency {
    Minutely,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Custom(u32), // segundos por periodo
}

impl Default for Frequency {
    fn default() -> Self {
        Self::Daily
    }
}

impl TimeSeries {
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            freq: Frequency::Daily,
        }
    }

    pub fn with_frequency(values: Vec<f64>, freq: Frequency) -> Self {
        Self { values, freq }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn last(&self) -> Option<f64> {
        self.values.last().copied()
    }

    pub fn mean(&self) -> f64 {
        if self.values.is_empty() {
            return f64::NAN;
        }
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }

    pub fn variance(&self) -> f64 {
        let n = self.values.len();
        if n < 2 {
            return f64::NAN;
        }
        let mean = self.mean();
        self.values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    }
}

/// Resultado de un pronóstico
#[derive(Debug, Clone)]
pub struct ForecastResult {
    /// Valores pronosticados (punto medio / media)
    pub forecast: Vec<f64>,
    /// Límite inferior del intervalo de confianza
    pub lower_bound: Vec<f64>,
    /// Límite superior del intervalo de confianza
    pub upper_bound: Vec<f64>,
    /// Nivel de confianza usado (ej. 0.80, 0.95)
    pub confidence: f64,
    /// Residuos del ajuste (in-sample)
    pub residuals: Vec<f64>,
}

impl ForecastResult {
    pub fn new(
        forecast: Vec<f64>,
        lower_bound: Vec<f64>,
        upper_bound: Vec<f64>,
        confidence: f64,
        residuals: Vec<f64>,
    ) -> Self {
        Self {
            forecast,
            lower_bound,
            upper_bound,
            confidence,
            residuals,
        }
    }

    /// Genera un forecast simple sin intervalo de confianza
    pub fn simple(forecast: Vec<f64>, residuals: Vec<f64>) -> Self {
        let lower = forecast.iter().map(|v| v - 2.0).collect();
        let upper = forecast.iter().map(|v| v + 2.0).collect();
        Self {
            forecast,
            lower_bound: lower,
            upper_bound: upper,
            confidence: 0.0,
            residuals,
        }
    }

    pub fn steps(&self) -> usize {
        self.forecast.len()
    }
}

/// Configuración del modelo ARIMA
#[derive(Debug, Clone)]
pub struct ArimaConfig {
    /// Orden autoregresivo (p)
    pub p: usize,
    /// Orden de diferenciación (d)
    pub d: usize,
    /// Orden de media móvil (q)
    pub q: usize,
    /// Incluir constante (intercepto)
    pub include_constant: bool,
}

impl Default for ArimaConfig {
    fn default() -> Self {
        Self {
            p: 1,
            d: 0,
            q: 0,
            include_constant: true,
        }
    }
}

impl ArimaConfig {
    pub fn new(p: usize, d: usize, q: usize) -> Self {
        Self {
            p,
            d,
            q,
            include_constant: true,
        }
    }

    pub fn without_constant(mut self) -> Self {
        self.include_constant = false;
        self
    }
}

/// Configuración del modelo Prophet-style
#[derive(Debug, Clone)]
pub struct ProphetConfig {
    /// Número de puntos de cambio en la tendencia
    pub n_changepoints: usize,
    /// Amplitud de la tendencia logística (crecimiento máximo)
    pub capacity: f64,
    /// Incluir estacionalidad semanal (periodo=7)
    pub weekly_seasonality: bool,
    /// Incluir estacionalidad anual (periodo=365)
    pub yearly_seasonality: bool,
    /// Número de términos de Fourier para estacionalidad
    pub fourier_order: usize,
    /// Factor de incertidumbre (escala de los intervalos)
    pub uncertainty_scale: f64,
    /// Tipo de tendencia
    pub trend_type: TrendType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendType {
    Linear,
    Logistic,
    Flat,
}

impl Default for TrendType {
    fn default() -> Self {
        Self::Linear
    }
}

impl Default for ProphetConfig {
    fn default() -> Self {
        Self {
            n_changepoints: 5,
            capacity: 1_000_000.0,
            weekly_seasonality: true,
            yearly_seasonality: true,
            fourier_order: 6,
            uncertainty_scale: 0.05,
            trend_type: TrendType::Linear,
        }
    }
}

/// Componentes descompuestos del modelo Prophet
#[derive(Debug, Clone)]
pub struct ProphetComponents {
    pub trend: Vec<f64>,
    pub weekly: Vec<f64>,
    pub yearly: Vec<f64>,
    pub residuals: Vec<f64>,
}

/// Coeficientes del modelo Prophet ajustado
#[derive(Debug, Clone)]
pub struct ProphetCoefficients {
    pub trend_params: Vec<f64>,
    pub weekly_coeffs: Vec<f64>,
    pub yearly_coeffs: Vec<f64>,
}

/// Modelo ARIMA ajustado con coeficientes
#[derive(Debug, Clone)]
pub struct ArimaModel {
    pub config: ArimaConfig,
    pub ar_coeffs: Vec<f64>,
    pub ma_coeffs: Vec<f64>,
    pub constant: f64,
    pub residuals: Vec<f64>,
    pub sigma2: f64,
    pub log_likelihood: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_series_new() {
        let ts = TimeSeries::new(vec![1.0, 2.0, 3.0]);
        assert_eq!(ts.len(), 3);
        assert_eq!(ts.mean(), 2.0);
    }

    #[test]
    fn test_time_series_empty() {
        let ts = TimeSeries::new(vec![]);
        assert!(ts.is_empty());
        assert!(ts.mean().is_nan());
    }

    #[test]
    fn test_time_series_variance() {
        let ts = TimeSeries::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let var = ts.variance();
        assert!((var - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_forecast_result_simple() {
        let fr = ForecastResult::simple(vec![10.0, 20.0], vec![1.0, 2.0]);
        assert_eq!(fr.steps(), 2);
        assert_eq!(fr.forecast[0], 10.0);
    }

    #[test]
    fn test_arima_config_default() {
        let cfg = ArimaConfig::default();
        assert_eq!(cfg.p, 1);
        assert_eq!(cfg.d, 0);
        assert_eq!(cfg.q, 0);
    }

    #[test]
    fn test_prophet_config_default() {
        let cfg = ProphetConfig::default();
        assert_eq!(cfg.n_changepoints, 5);
        assert!(cfg.weekly_seasonality);
    }
}
