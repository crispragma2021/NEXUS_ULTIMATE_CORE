// ==========================================
// MÓDULO HFT - ARBITRAJE DE LATENCIA & LOB IMBALANCE
// ==========================================
// Monitorea desfases temporales entre feeds de datos de alta velocidad
// y calcula el desequilibrio en el libro de órdenes (Limit Order Book)
// para predecir la micro-dirección de los precios.
// ==========================================

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct FeedTick {
    pub precio: f64,
    pub timestamp: Instant,
    pub bid_volume: f64,
    pub ask_volume: f64,
}

#[derive(Debug)]
pub struct RastreadorLatencia {
    pub max_latencia_tolerable: Duration,
    pub desvio_minimo_precio: f64,
    pub ultimo_tick_rapido: Option<FeedTick>,
    pub ultimo_tick_lento: Option<FeedTick>,
}

impl RastreadorLatencia {
    pub fn new(max_latencia_ms: u64, desvio_minimo: f64) -> Self {
        Self {
            max_latencia_tolerable: Duration::from_millis(max_latencia_ms),
            desvio_minimo_precio: desvio_minimo,
            ultimo_tick_rapido: None,
            ultimo_tick_lento: None,
        }
    }

    /// Registra un nuevo tick del feed institucional rápido
    pub fn registrar_tick_rapido(&mut self, precio: f64, bid_vol: f64, ask_vol: f64) {
        self.ultimo_tick_rapido = Some(FeedTick {
            precio,
            timestamp: Instant::now(),
            bid_volume: bid_vol,
            ask_volume: ask_vol,
        });
    }

    /// Registra un nuevo tick del feed lento de ejecución del bróker
    pub fn registrar_tick_lento(&mut self, precio: f64, bid_vol: f64, ask_vol: f64) {
        self.ultimo_tick_lento = Some(FeedTick {
            precio,
            timestamp: Instant::now(),
            bid_volume: bid_vol,
            ask_volume: ask_vol,
        });
    }

    /// Calcula el desequilibrio en el libro de órdenes (LOB Imbalance)
    /// Retorna un valor entre -1.0 (máxima presión de venta) y 1.0 (máxima presión de compra)
    pub fn calcular_lob_imbalance(&self, bid_vol: f64, ask_vol: f64) -> f64 {
        let total_vol = bid_vol + ask_vol;
        if total_vol <= 0.0 {
            return 0.0;
        }
        (bid_vol - ask_vol) / total_vol
    }

    /// Evalúa si existe una oportunidad de arbitraje por latencia activa.
    /// Retorna:
    ///   - Some(true): Señal de compra (el feed rápido ya subió, el lento sigue abajo)
    ///   - Some(false): Señal de venta (el feed rápido ya bajó, el lento sigue arriba)
    ///   - None: Sin oportunidad clara o datos insuficientes.
    pub fn verificar_oportunidad(&self) -> Option<bool> {
        let rapido = self.ultimo_tick_rapido.as_ref()?;
        let lento = self.ultimo_tick_lento.as_ref()?;

        // Verificar obsolescencia del feed rápido (no debe superar el umbral de latencia)
        if rapido.timestamp.elapsed() > self.max_latencia_tolerable {
            return None; // Señal expirada/demasiado vieja
        }

        let diferencia_precio = rapido.precio - lento.precio;

        // Comprobar si el desvío supera el umbral mínimo configurado
        if diferencia_precio.abs() >= self.desvio_minimo_precio {
            let imbalance = self.calcular_lob_imbalance(rapido.bid_volume, rapido.ask_volume);

            if diferencia_precio > 0.0 && imbalance > 0.1 {
                // El precio real subió en el feed rápido y el libro de órdenes soporta la compra
                Some(true)
            } else if diferencia_precio < 0.0 && imbalance < -0.1 {
                // El precio real bajó en el feed rápido y el libro de órdenes soporta la venta
                Some(false)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lob_imbalance() {
        let rastreador = RastreadorLatencia::new(200, 0.05);
        // Balanceado
        assert_eq!(rastreador.calcular_lob_imbalance(1000.0, 1000.0), 0.0);
        // Presión de compra
        assert_eq!(rastreador.calcular_lob_imbalance(1500.0, 500.0), 0.5);
        // Presión de venta
        assert_eq!(rastreador.calcular_lob_imbalance(500.0, 1500.0), -0.5);
    }
}
