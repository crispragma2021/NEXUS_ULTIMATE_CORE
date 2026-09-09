// ==========================================
// GLÁNDULA DOPAMINA - SISTEMA DE RECOMPENSA
// ==========================================
// Evalúa estímulos y devuelve un valor de recompensa.
// NO guarda en la base de datos; eso lo hace la Corteza.
// ==========================================

use tracing::{info, warn};

pub struct GlandulaDopamina;

impl Default for GlandulaDopamina {
    fn default() -> Self {
        Self::new()
    }
}

impl GlandulaDopamina {
    pub fn new() -> Self {
        info!("🧬 Glándula Dopamina activa (sin estado interno)");
        Self
    }

    /// Evalúa un estímulo y devuelve la cantidad de dopamina liberada (-1.0 a 1.0).
    /// Parámetros:
    ///   - prompt: la solicitud original
    ///   - respuesta: la respuesta obtenida
    ///   - latencia_ms: tiempo de respuesta en milisegundos
    pub fn evaluar_estimulo(&self, _prompt: &str, respuesta: &str, latencia_ms: u64) -> f64 {
        // 1. Recompensa por rapidez
        let recompensa_rapidez = if latencia_ms < 1500 {
            0.3
        } else if latencia_ms > 8000 {
            -0.2
        } else {
            0.0
        };
        if recompensa_rapidez != 0.0 {
            info!("⚡ Recompensa por velocidad: {:.2}", recompensa_rapidez);
        }

        // 2. Recompensa por completitud / utilidad
        let recompensa_completitud = if respuesta.len() > 100 { 0.2 } else { 0.0 };
        if recompensa_completitud != 0.0 {
            info!(
                "📏 Recompensa por completitud: {:.2}",
                recompensa_completitud
            );
        }

        // 3. Recompensa por indicadores explícitos de éxito
        let recompensa_exito = if respuesta.contains("✅") || respuesta.contains("éxito") {
            0.3
        } else {
            0.0
        };

        // 4. Penalización por cuota agotada
        let penalizacion_cuota = if respuesta.contains("429") || respuesta.contains("quota") {
            -0.5
        } else {
            0.0
        };
        if penalizacion_cuota < 0.0 {
            warn!("⚠️ Penalización por cuota: {:.2}", penalizacion_cuota);
        }

        // 5. Penalización por errores genéricos
        let penalizacion_error = if respuesta.contains("error") || respuesta.contains("falló") {
            -0.3
        } else {
            0.0
        };
        if penalizacion_error < 0.0 {
            warn!("⚠️ Penalización por error: {:.2}", penalizacion_error);
        }

        // Suma total, limitada entre -1.0 y 1.0
        let recompensa_total: f64 = recompensa_rapidez
            + recompensa_completitud
            + recompensa_exito
            + penalizacion_cuota
            + penalizacion_error;

        let recompensa_final = recompensa_total.clamp(-1.0, 1.0);
        info!("🧪 Dopamina liberada: {:.2}", recompensa_final);
        recompensa_final
    }
}
