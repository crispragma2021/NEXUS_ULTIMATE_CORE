use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::info;

pub static METABOLISMO_ACTUAL: AtomicUsize = AtomicUsize::new(4);

/// 🧬 APLICA EL METABOLISMO GLOBAL
/// Configura el paralelismo de motores pesados (como Candle) según el estado del hardware.
pub fn aplicar_metabolismo(n: usize) {
    METABOLISMO_ACTUAL.store(n, Ordering::SeqCst);

    // Nota: Candle usa Rayon. Para cambios dinámicos, los módulos
    // deben usar rayon::ThreadPoolBuilder con el valor de METABOLISMO_ACTUAL.

    info!(
        "🧬 [METABOLISMO] Paralelismo global solicitado: {} hilos",
        n
    );
}

pub fn obtener_latencia_disco() -> f64 {
    // Retorna una latencia nominal por defecto para no romper la compatibilidad
    0.5
}
