use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn};
use crate::cerebro::reflex_arc::ReflexSignal;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

/// 🔮 GEMELO DIGITAL: Órgano de Predicción y Simulación Crítica
/// Permite a NEXUS proyectar el impacto de sus acciones en un entorno virtual
/// antes de comprometer el sistema real (Soberanía OMEGA).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictOutcome {
    Success,
    PartialSuccess(String),
    CatastrophicFailure(String),
    Unknown,
}

pub struct DigitalTwin {
    // Simulamos la estructura del proyecto en memoria para validar eliminaciones/movimientos
    pub project_root: String,
}

impl DigitalTwin {
    pub fn new(root: &str) -> Self {
        Self {
            project_root: root.to_string(),
        }
    }

    /// 🧠 SIMULACIÓN ESTRUCTURAL: ¿Qué pasa si borramos/modificamos este archivo?
    pub async fn simular_cambio_archivo(&self, path: &str, operation: &str) -> PredictOutcome {
        info!(
            "🔮 [GEMELO] Simulando operación '{}' en path: {}",
            operation, path
        );

        let target_path = Path::new(path);

        // Regla de Oro: No tocar el núcleo vital si no es una operación de evolución controlada
        if operation == "delete" && (path.contains("src/brain") || path.contains("src/infra")) {
            return PredictOutcome::CatastrophicFailure(
                "Intento de extirpación de órgano vital detectado en el gemelo.".to_string(),
            );
        }

        if !target_path.exists() && operation != "create" {
            return PredictOutcome::PartialSuccess(
                "El archivo no existe en la realidad, pero se asume su ausencia en el gemelo."
                    .to_string(),
            );
        }

        PredictOutcome::Success
    }

    /// 🛠️ SIMULACIÓN DE COMANDO: Análisis de impacto antes de la ejecución
    pub async fn predecir_impacto_comando(&self, cmd: &str) -> f32 {
        let mut confidence = 1.0;

        // Si el comando contiene patrones destructivos no autorizados
        if cmd.contains("rm -rf") || cmd.contains("mkfs") {
            warn!("🚨 [GEMELO] Comando de alta entropía detectado: {}", cmd);
            confidence *= 0.1;
        }

        // Si el comando afecta a la ruta maestra sin ser un comando de git nexus
        if cmd.contains("/") && !cmd.contains(&self.project_root) {
            confidence *= 0.5;
        }

        confidence
    }

    /// 🛡️ VEDO DE SEGURIDAD (Simulado): ¿Debería ejecutarse esta acción?
    pub async fn autorizacion_soberana(&self, cmd: &str) -> bool {
        let confidence = self.predecir_impacto_comando(cmd).await;

        if confidence < 0.3 {
            warn!(
                "🛑 [GEMELO] Veto de Simulación: Nivel de confianza insuficiente ({:.2})",
                confidence
            );
            return false;
        }

        info!(
            "✅ [GEMELO] Simulación validada. Confianza: {:.2}",
            confidence
        );
        true
    }
}

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
