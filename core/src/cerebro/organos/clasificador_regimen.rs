// ==========================================
// CLASIFICADOR DE RÉGIMEN DE MERCADO (ML/HEURÍSTICO)
// ==========================================
// Clasifica la estructura del mercado en tres estados dinámicos:
//   - TendenciaAlcista: Mercado subiendo con fuerza (Ideal para compras).
//   - TendenciaBajista: Mercado cayendo con fuerza (Ideal para shorts).
//   - RangoConsolidacion: Mercado lateral / rango (Evitar operar HFT).
// ==========================================

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RegimenMercado {
    TendenciaAlcista,
    TendenciaBajista,
    RangoConsolidacion,
}

pub struct ClasificadorRegimen;

impl ClasificadorRegimen {
    /// Clasifica el régimen actual del mercado
    /// Parámetros:
    ///   - precio_actual: Último precio registrado.
    ///   - ema_20: Media móvil exponencial de 20 periodos.
    ///   - bollinger_upper: Banda superior de Bollinger.
    ///   - bollinger_lower: Banda inferior de Bollinger.
    pub fn clasificar(
        precio_actual: f64,
        ema_20: f64,
        bollinger_upper: f64,
        bollinger_lower: f64,
    ) -> RegimenMercado {
        if ema_20.is_nan() || bollinger_upper.is_nan() || bollinger_lower.is_nan() {
            return RegimenMercado::RangoConsolidacion;
        }

        let amplitud_bandas = bollinger_upper - bollinger_lower;
        let umbral_consolidacion = ema_20 * 0.005; // 0.5% del precio como rango de compresión

        // 1. Verificar compresión de volatilidad (Squeeze / Consolidación)
        if amplitud_bandas < umbral_consolidacion {
            return RegimenMercado::RangoConsolidacion;
        }

        // 2. Clasificar según la posición del precio respecto a la EMA y las bandas
        let distancia_ema = precio_actual - ema_20;
        let umbral_tendencia = ema_20 * 0.001; // 0.1% de desviación mínima

        if distancia_ema > umbral_tendencia
            && precio_actual > (bollinger_upper - (amplitud_bandas * 0.2))
        {
            RegimenMercado::TendenciaAlcista
        } else if distancia_ema < -umbral_tendencia
            && precio_actual < (bollinger_lower + (amplitud_bandas * 0.2))
        {
            RegimenMercado::TendenciaBajista
        } else {
            RegimenMercado::RangoConsolidacion
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clasificador_regimen() {
        let ema = 100.0;
        let upper = 105.0;
        let lower = 95.0;

        // Caso 1: Tendencia Alcista (precio cerca de la banda superior)
        assert_eq!(
            ClasificadorRegimen::clasificar(104.5, ema, upper, lower),
            RegimenMercado::TendenciaAlcista
        );

        // Caso 2: Tendencia Bajista (precio cerca de la banda inferior)
        assert_eq!(
            ClasificadorRegimen::clasificar(95.5, ema, upper, lower),
            RegimenMercado::TendenciaBajista
        );

        // Caso 3: Consolidación / Rango (precio en la media móvil o bandas muy comprimidas)
        assert_eq!(
            ClasificadorRegimen::clasificar(100.2, ema, upper, lower),
            RegimenMercado::RangoConsolidacion
        );

        // Caso 4: Squeeze de Bollinger (amplitud menor al 0.5% de la EMA)
        assert_eq!(
            ClasificadorRegimen::clasificar(104.0, 100.0, 100.2, 99.8),
            RegimenMercado::RangoConsolidacion
        );
    }
}
