use crate::brain::reflex_arc::ReflexSignal;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

/// ⚡ Simulador de Fiebre Interna (Protocolo BIST)
/// Inyecta una señal de calor crítico para validar el Arco Reflejo (Pilar 6).
pub async fn simular_fiebre_interna(reflex_tx: mpsc::Sender<ReflexSignal>) {
    println!("🧪 [DIAGNÓSTICO] Iniciando Protocolo BIST de Reflejos...");
    println!("🧪 [DIAGNÓSTICO] Inyectando 'Grito de Dolor' de 86°C en el Arco Reflejo...");

    // Inyección de señal de calor crítico
    if let Err(e) = reflex_tx.send(ReflexSignal::HeatSpike(86)).await {
        eprintln!("❌ [DIAGNÓSTICO] Error al inyectar señal: {}", e);
        return;
    }

    // El sistema debería reaccionar al instante
    println!(
        "🧪 [DIAGNÓSTICO] Verificación de Ejecución: El Ejecutivo debería estar respondiendo."
    );

    // Esperar 3 segundos como solicita el Arquitecto
    sleep(Duration::from_secs(3)).await;

    println!("🧪 [DIAGNÓSTICO] Restaurando estado normal sensorizado...");
    // Inyectar señal de restauración (40°C)
    let _ = reflex_tx.send(ReflexSignal::HeatSpike(40)).await;

    println!("🧪 [DIAGNÓSTICO] Simulacro Finalizado. Comprobar logs de Homeostasis.");
}
