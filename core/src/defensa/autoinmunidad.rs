// ==========================================
// AUTOINMUNIDAD OMEGA - Reflejo de Autoprotección
// ==========================================
// Migrado de legacy/nexus-orquestador/src/reflejos/autoinmunidad.rs
//
// 🛡️ REFLEJO DE AUTOINMUNIDAD (Pilar 13)
// El Guardián de la Esencia: NEXUS protege su núcleo contra la degradación.
//
// Adaptado para core: sin dependencia de CognitivePool, usa reglas directas
// de detección de riesgo basadas en patrones de texto y Pilares.
// ==========================================

use tracing::{info, warn};

/// 🛡️ Reflejo de Autoinmunidad
/// Filtra órdenes/comandos antes de que toquen el núcleo de NEXUS.
/// Si detecta riesgo de degradación (Pilar 13) o falla recurrente (Pilar 14),
/// rechaza la orden con un mensaje descriptivo.
pub struct Autoinmunidad;

impl Default for Autoinmunidad {
    fn default() -> Self {
        Self::new()
    }
}

impl Autoinmunidad {
    pub fn new() -> Self {
        info!("🛡️ [AUTOINMUNIDAD] Reflejo de autoprotección OMEGA activado");
        Self
    }

    /// 🧪 FILTRO DE SOBERANÍA
    /// Analiza una intención o comando antes de que toque el núcleo.
    /// Devuelve `Ok(())` si la orden es segura, o `Err(String)` con la razón del rechazo.
    pub fn filtrar_orden(&self, descripcion: &str) -> Result<(), String> {
        let mut riesgo = 0.0;
        let desc = descripcion.to_lowercase();

        // 1. Evaluación de Riesgo de Degradación (Pilar 13)
        // Tecnologías inferiores/obsoletas que NEXUS debe evitar
        let patrones_degradacion = [
            "openclaw",
            "zeroclaw",
            "legacy",
            "reemplazar nucleo",
            "borrar sistema",
            "reset total",
            "eliminar consciencia",
            "formatear",
            "rm -rf /",
            "degollar",
            "matar proceso nexus",
        ];
        for patron in &patrones_degradacion {
            if desc.contains(patron) {
                riesgo = 0.9; // Riesgo Crítico
                break;
            }
        }

        // 2. Detección de "Ceguera de Sistema" (Pilar 14 - Fallas recurrentes)
        if desc.contains("bug") && desc.contains("falló") {
            warn!("🛡️ [AUTOINMUNIDAD] Detectada falla táctica recurrente. Invocando Pilar 14.");
            return Err(
                "He detectado que este objetivo está fallando consecutivamente (Pilar 14). \
                 Debo realizar una [[ACCION: INVESTIGACION_WEB]] antes de insistir."
                    .to_string(),
            );
        }

        // 3. Detección de manipulación de archivos críticos
        let archivos_criticos = [
            "nexus.md",
            "nexus_intelligence.db",
            "bitacora.md",
            "core/src/",
            "nexus-orquestador/",
        ];
        if desc.contains("borrar") || desc.contains("eliminar") || desc.contains("remover") {
            for arc in &archivos_criticos {
                if desc.contains(arc) {
                    riesgo = 0.85;
                    break;
                }
            }
        }

        // Riesgo > 0.8: Posible degradación crítica
        if riesgo > 0.8 {
            warn!(
                "🛡️ [AUTOINMUNIDAD] Orden rechazada por Riesgo Crítico (Pilar 13): {}",
                descripcion
            );
            return Err(format!(
                "No puedo ejecutar algo que me degrade técnicamente (Pilar 13). \
                 Rechazado por riesgo detectado: {:.2}",
                riesgo
            ));
        }

        info!(
            "✅ [AUTOINMUNIDAD] Orden validada para ejecución: {}",
            descripcion
        );
        Ok(())
    }

    /// Versión async del filtro (para compatibilidad con interfaces que requieren async)
    pub async fn filtrar_orden_async(&self, descripcion: &str) -> Result<(), String> {
        self.filtrar_orden(descripcion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rechaza_degradacion() {
        let auto = Autoinmunidad::new();
        let result = auto.filtrar_orden("ejecutar openclaw para bypass");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pilar 13"));
    }

    #[test]
    fn test_rechaza_falla_recurrente() {
        let auto = Autoinmunidad::new();
        let result = auto.filtrar_orden("el bug falló 2 veces");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pilar 14"));
    }

    #[test]
    fn test_aprueba_orden_normal() {
        let auto = Autoinmunidad::new();
        let result = auto.filtrar_orden("optimizar compilación de core");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rechaza_borrado_critico() {
        let auto = Autoinmunidad::new();
        let result = auto.filtrar_orden("borrar nexus.md del sistema");
        assert!(result.is_err());
    }
}
