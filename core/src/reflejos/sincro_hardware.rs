use std::time::Duration;
use tracing::{info, warn};

/// 🌡️ REFLEJO DE SINCRO-HARDWARE (El Termostato del Intel Core)
/// Ajusta el paralelismo dinámicamente según la temperatura y carga del hardware.
pub struct SincroHardware {
    pub paralelismo_actual: usize,
    pub temp_maxima: f32,
}

impl Default for SincroHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl SincroHardware {
    pub fn new() -> Self {
        Self {
            paralelismo_actual: 4,
            temp_maxima: 0.0,
        }
    }

    /// 🧪 AJUSTE DINÁMICO DE POTENCIA
    /// Reduce el paralelismo si la CPU se calienta demasiado.
    pub async fn ajustar_por_temperatura(&mut self, temperatura_actual: f32) {
        if temperatura_actual > self.temp_maxima {
            self.temp_maxima = temperatura_actual;
        }

        if temperatura_actual > 85.0 {
            // Peligro térmico — reducir drásticamente
            self.paralelismo_actual = (self.paralelismo_actual / 2).max(1);
            warn!(
                "🌡️ [SINCRO] ¡PELIGRO TÉRMICO! {}°C. Paralelismo reducido a {}.",
                temperatura_actual, self.paralelismo_actual
            );
        } else if temperatura_actual > 70.0 {
            // Precaución — reducir suavemente
            self.paralelismo_actual = self.paralelismo_actual.saturating_sub(1).max(1);
            info!(
                "🌡️ [SINCRO] Precaución térmica: {}°C. Paralelismo → {}.",
                temperatura_actual, self.paralelismo_actual
            );
        } else if temperatura_actual < 50.0 && self.paralelismo_actual < 8 {
            // Fresco — aumentar rendimiento
            self.paralelismo_actual += 1;
            info!(
                "🌡️ [SINCRO] Temperatura óptima: {}°C. Aumentando paralelismo a {}.",
                temperatura_actual, self.paralelismo_actual
            );
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    /// Devuelve el paralelismo actual para usar en operaciones concurrentes
    pub fn paralelismo(&self) -> usize {
        self.paralelismo_actual
    }
}
