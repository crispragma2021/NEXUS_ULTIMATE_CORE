use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::{info, warn};

/// 🛡️ REFLEJO DE AUTOINMUNIDAD (Pilar 13)
/// El Guardián de la Esencia: NEXUS protege su núcleo contra la degradación.
pub struct Autoinmunidad {
    // Pool de memoria opcional — si no se provee, opera en modo reducido
    memoria: Option<Arc<dyn Fn(&str) -> usize + Send + Sync>>,
}

impl Autoinmunidad {
    pub fn new() -> Self {
        Self { memoria: None }
    }

    pub fn con_memoria(mut self, contador: Arc<dyn Fn(&str) -> usize + Send + Sync>) -> Self {
        self.memoria = Some(contador);
        self
    }

    /// 🧪 FILTRO DE SOBERANÍA
    /// Analiza una intención o comando antes de que toque el núcleo.
    pub async fn filtrar_orden(&self, descripcion: &str) -> Result<()> {
        let mut riesgo = 0.0;

        // 1. Evaluación de Riesgo de Degradación
        let lower = descripcion.to_lowercase();
        if lower.contains("openclaw")
            || lower.contains("zeroclaw")
            || lower.contains("legacy")
            || lower.contains("eliminar")
            || lower.contains("borrar")
        {
            riesgo = 0.9;
        }

        // 2. Detección de "Ceguera de Sistema" (Fallas recurrentes)
        if let Some(ref contador) = self.memoria {
            let fallas = contador(descripcion);
            if fallas >= 2 {
                warn!("🛡️ [AUTOINMUNIDAD] Detectada falla táctica recurrente ({} ocurrencias). Invocando Pilar 14.", fallas);
                return Err(anyhow!("He detectado que este objetivo está fallando consecutivamente (Pilar 14). Debo realizar una verificación antes de insistir."));
            }
        }

        if riesgo > 0.8 {
            warn!(
                "🛡️ [AUTOINMUNIDAD] Orden rechazada por Riesgo Crítico (Pilar 13): {}",
                descripcion
            );
            return Err(anyhow!("No puedo ejecutar algo que me degrade técnicamente (Pilar 13). Rechazado por riesgo detectado: {:.2}", riesgo));
        }

        info!(
            "✅ [AUTOINMUNIDAD] Orden validada para ejecución: {}",
            descripcion
        );
        Ok(())
    }
}

impl Default for Autoinmunidad {
    fn default() -> Self {
        Self::new()
    }
}
