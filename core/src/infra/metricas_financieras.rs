// ==========================================
// MÉTRICAS FINANCIERAS OMEGA - Control de Riesgo
// ==========================================
// Calcula VaR, Sharpe, Kelly y Drawdown para
// asegurar la supervivencia del capital en el i7-12700F.
// ==========================================

use tracing::info;

pub struct CalculadoraRiesgo;

impl CalculadoraRiesgo {
    pub fn new() -> Self {
        info!("📈 [RIESGO] Módulo de Métricas Financieras inicializado.");
        Self
    }

    /// Calcula el Value at Risk (VaR) histórico al nivel de confianza dado (ej. 0.95).
    pub fn calcular_var(&self, retornos: &[f32], confianza: f32) -> f32 {
        if retornos.is_empty() {
            return 0.0;
        }
        let mut r = retornos.to_vec();
        r.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((1.0 - confianza) * r.len() as f32).floor() as usize;
        *r.get(idx).unwrap_or(&0.0)
    }

    /// Calcula el Sharpe Ratio (Retorno Ajustado a la Volatilidad).
    pub fn calcular_sharpe(&self, retornos: &[f32], tasa_libre_riesgo: f32) -> f32 {
        if retornos.is_empty() {
            return 0.0;
        }
        let n = retornos.len() as f32;
        let media: f32 = retornos.iter().sum::<f32>() / n;
        let varianza: f32 = retornos.iter().map(|x| (x - media).powi(2)).sum::<f32>() / n;
        let std_dev = varianza.sqrt();
        if std_dev == 0.0 {
            0.0
        } else {
            (media - tasa_libre_riesgo) / std_dev
        }
    }

    /// Criterio de Kelly para dimensionamiento de posición óptimo.
    /// b = ratio profit/loss, p = win_rate
    pub fn calcular_kelly(&self, win_rate: f32, profit_loss_ratio: f32) -> f32 {
        if profit_loss_ratio <= 0.0 {
            return 0.0;
        }
        win_rate - (1.0 - win_rate) / profit_loss_ratio
    }

    /// Calcula el Máximo Drawdown (La mayor caída histórica desde el pico).
    pub fn calcular_max_drawdown(&self, equidad_historica: &[f32]) -> f32 {
        let mut max_drawdown = 0.0;
        let mut pico = 0.0;
        for &valor in equidad_historica {
            if valor > pico {
                pico = valor;
            }
            let drawdown = if pico == 0.0 {
                0.0
            } else {
                (pico - valor) / pico
            };
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }
        max_drawdown
    }
}

impl Default for CalculadoraRiesgo {
    fn default() -> Self {
        Self::new()
    }
}
