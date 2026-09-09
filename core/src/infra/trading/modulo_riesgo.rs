// ==========================================
// MÓDULO DE GESTIÓN DE RIESGO DINÁMICO (ATR)
// ==========================================
// Calcula niveles de salida dinámicos (Stop Loss y Take Profit)
// basados en la volatilidad real del activo (Average True Range).
// ==========================================

pub struct GestorRiesgoAtr {
    pub multiplicador_stop: f64,
    pub multiplicador_profit: f64,
}

impl GestorRiesgoAtr {
    pub fn new(mult_stop: f64, mult_profit: f64) -> Self {
        Self {
            multiplicador_stop: mult_stop,
            multiplicador_profit: mult_profit,
        }
    }

    /// Calcula las salidas para una posición de COMPRA (Long)
    /// Retorna: (stop_loss, take_profit)
    pub fn calcular_salidas_compra(&self, precio_entrada: f64, atr: f64) -> (f64, f64) {
        let stop_loss = precio_entrada - (self.multiplicador_stop * atr);
        let take_profit = precio_entrada + (self.multiplicador_profit * atr);
        (stop_loss, take_profit)
    }

    /// Calcula las salidas para una posición de VENTA (Short)
    /// Retorna: (stop_loss, take_profit)
    pub fn calcular_salidas_venta(&self, precio_entrada: f64, atr: f64) -> (f64, f64) {
        let stop_loss = precio_entrada + (self.multiplicador_stop * atr);
        let take_profit = precio_entrada - (self.multiplicador_profit * atr);
        (stop_loss, take_profit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salidas_riesgo_atr() {
        let gestor = GestorRiesgoAtr::new(1.5, 3.0);
        let precio = 100.0;
        let atr = 2.0;

        // Compra (Long)
        let (sl_compra, tp_compra) = gestor.calcular_salidas_compra(precio, atr);
        assert_eq!(sl_compra, 97.0); // 100 - (1.5 * 2) = 97
        assert_eq!(tp_compra, 106.0); // 100 + (3.0 * 2) = 106

        // Venta (Short)
        let (sl_venta, tp_venta) = gestor.calcular_salidas_venta(precio, atr);
        assert_eq!(sl_venta, 103.0); // 100 + (1.5 * 2) = 103
        assert_eq!(tp_venta, 94.0); // 100 - (3.0 * 2) = 94
    }
}
