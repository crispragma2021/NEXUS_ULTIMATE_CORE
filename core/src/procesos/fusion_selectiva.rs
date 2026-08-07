// ==========================================
// FUSIÓN SELECTIVA OMEGA - Evolución sin Olvido
// ==========================================
// ⚠️ ÓRGANO OMEGA: FUSIÓN SELECTIVA POR MEJORA ⚠️
// Migrado de legacy/nexus-orquestador/src/evolucion/fusion.rs
//
// "NEXUS no debe perder lo que ya tiene. Debe EVOLUCIONAR sin olvidar."
// - Arquitecto Director sobre Ryzen i7-12700F
//
// FusionSelectiva permite a NEXUS evaluar si una nueva capacidad
// (migración, refactor, característica) es superior, inferior,
// compatible o idéntica a la existente, decidiendo si absorberla,
// rechazarla o fusionarla.
// ==========================================

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Una "Capacidad" genérica que NEXUS evaluará antes de absorber.
/// Representa cualquier funcionalidad, módulo o comportamiento.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capacidad {
    /// Funcionalidad base (nombre/descripción)
    pub base: String,
    /// Mejora opcional específica
    pub mejora: Option<String>,
    /// Extra/información adicional
    pub extra: Option<String>,
    /// Versión o tag identificador
    pub version: String,
    /// Si soporta operaciones asíncronas
    pub complejidad_asincrona: bool,
    /// Si tiene aislamiento de memoria (Arc, Mutex, etc.)
    pub aislamiento_memoria: bool,
    /// Si implementa retry/fallback robusto
    pub robustez_retry: bool,
}

impl Capacidad {
    /// Crea una nueva capacidad con los campos esenciales
    pub fn new(base: &str, version: &str) -> Self {
        Self {
            base: base.to_string(),
            mejora: None,
            extra: None,
            version: version.to_string(),
            complejidad_asincrona: false,
            aislamiento_memoria: false,
            robustez_retry: false,
        }
    }

    /// Marca esta capacidad como asíncrona
    pub fn with_async(mut self) -> Self {
        self.complejidad_asincrona = true;
        self
    }

    /// Marca esta capacidad con aislamiento de memoria
    pub fn with_memory_isolation(mut self) -> Self {
        self.aislamiento_memoria = true;
        self
    }

    /// Marca esta capacidad con robustez de retry
    pub fn with_retry_robustness(mut self) -> Self {
        self.robustez_retry = true;
        self
    }

    /// Añade una mejora opcional
    pub fn with_mejora(mut self, mejora: &str) -> Self {
        self.mejora = Some(mejora.to_string());
        self
    }

    /// Añade un extra opcional
    pub fn with_extra(mut self, extra: &str) -> Self {
        self.extra = Some(extra.to_string());
        self
    }
}

/// Resultado de la comparación entre dos capacidades
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparacion {
    /// La nueva capacidad es superior - debe absorberla
    Superior,
    /// La nueva capacidad es inferior - debe rechazarla
    Inferior,
    /// Misma fortaleza técnica, implementación diferente - fusionar
    Compatible,
    /// Exactamente igual - ignorar
    Identica,
}

/// Motor de fusión evolutiva OMEGA
pub struct FusionSelectiva;

impl Default for FusionSelectiva {
    fn default() -> Self {
        Self::new()
    }
}

impl FusionSelectiva {
    pub fn new() -> Self {
        info!("🧬 [FUSIÓN SELECTIVA] Motor de evolución OMEGA activado");
        Self
    }

    /// Compara métricas y estructura de una nueva capacidad frente a la existente.
    /// Evalúa: asincronía, aislamiento de memoria, robustez de retry.
    fn comparar(&self, existente: &Capacidad, nueva: &Capacidad) -> Comparacion {
        let puntaje_existente = (existente.complejidad_asincrona as u8)
            + (existente.aislamiento_memoria as u8)
            + (existente.robustez_retry as u8);

        let puntaje_nueva = (nueva.complejidad_asincrona as u8)
            + (nueva.aislamiento_memoria as u8)
            + (nueva.robustez_retry as u8);

        if puntaje_nueva > puntaje_existente {
            return Comparacion::Superior;
        } else if puntaje_nueva < puntaje_existente {
            return Comparacion::Inferior;
        }

        // Si los puntajes base son idénticos pero el contenido cambia
        if existente.base == nueva.base
            && existente.mejora == nueva.mejora
            && existente.extra == nueva.extra
        {
            Comparacion::Identica
        } else {
            // Misma fuerza técnica, diferente implementación táctica -> Se fusionan
            Comparacion::Compatible
        }
    }

    /// El Acto de Evolución.
    /// - Superior: absorbe la mejora evolutiva
    /// - Inferior: rechaza, devuelve el original blindado
    /// - Compatible: fusiona caminos paralelos
    /// - Idéntica: ignora (no hace log para no ser charlatán)
    pub fn fusionar(&self, existente: &Capacidad, nueva: &Capacidad) -> Capacidad {
        match self.comparar(existente, nueva) {
            Comparacion::Superior => {
                info!("🧬 [FUSIÓN SELECTIVA] Propuesta SUPERIOR detectada. Absorbiendo mejora evolutiva.");
                Capacidad {
                    base: existente.base.clone(),
                    mejora: nueva
                        .mejora
                        .clone()
                        .or_else(|| Some("Mejora Absorbida Corticalmente".to_string())),
                    extra: existente.extra.clone(),
                    version: "Mejorada (NEXUS OMEGA)".to_string(),
                    complejidad_asincrona: nueva.complejidad_asincrona,
                    aislamiento_memoria: nueva.aislamiento_memoria,
                    robustez_retry: nueva.robustez_retry,
                }
            }
            Comparacion::Inferior => {
                warn!("🛡️ [FUSIÓN SELECTIVA] AMENAZA DE RECESIÓN DETECTADA. Propuesta INFERIOR rechazada (Lobotomía Evitada).");
                existente.clone()
            }
            Comparacion::Compatible => {
                info!("🧩 [FUSIÓN SELECTIVA] Propuesta COMPATIBLE detectada. Fusionando rutas paralelas sin borrar la base.");
                let merged_extra = format!("{:?} | {:?}", existente.extra, nueva.extra);
                Capacidad {
                    base: existente.base.clone(),
                    mejora: existente.mejora.clone(),
                    extra: Some(merged_extra),
                    version: "Fusionada (Híbrido)".to_string(),
                    complejidad_asincrona: existente.complejidad_asincrona,
                    aislamiento_memoria: existente.aislamiento_memoria,
                    robustez_retry: existente.robustez_retry,
                }
            }
            Comparacion::Identica => {
                // "Es lo mismo -> IGNORAR"
                // No emitimos log para no ser un charlatán (Regla del Juicio de Habla)
                existente.clone()
            }
        }
    }

    /// Evalúa rápidamente si una migración de legacy a core es segura
    pub fn evaluar_migracion(
        &self,
        nombre: &str,
        legacy: &Capacidad,
        core: &Capacidad,
    ) -> Comparacion {
        let resultado = self.comparar(legacy, core);
        match resultado {
            Comparacion::Superior => {
                info!(
                    "✅ [FUSIÓN] Migración de '{}' es SUPERIOR - proceder con reemplazo.",
                    nombre
                );
            }
            Comparacion::Inferior => {
                warn!(
                    "❌ [FUSIÓN] Migración de '{}' es INFERIOR - NO reemplazar.",
                    nombre
                );
            }
            Comparacion::Compatible => {
                info!(
                    "🔄 [FUSIÓN] Migración de '{}' es COMPATIBLE - fusionar.",
                    nombre
                );
            }
            Comparacion::Identica => {
                info!(
                    "⏭️ [FUSIÓN] Migración de '{}' es IDÉNTICA - mantener actual.",
                    nombre
                );
            }
        }
        resultado
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_superior_absorbe_mejora() {
        let fusion = FusionSelectiva::new();
        let existente = Capacidad::new("modulo_x", "1.0").with_async();
        let nueva = Capacidad::new("modulo_x", "2.0")
            .with_async()
            .with_retry_robustness()
            .with_mejora("retry con backoff exponencial");

        let result = fusion.fusionar(&existente, &nueva);
        assert_eq!(result.version, "Mejorada (NEXUS OMEGA)");
        assert!(result.robustez_retry);
        assert_eq!(result.mejora.unwrap(), "retry con backoff exponencial");
    }

    #[test]
    fn test_inferior_rechazada() {
        let fusion = FusionSelectiva::new();
        let existente = Capacidad::new("modulo_x", "1.0")
            .with_async()
            .with_retry_robustness();
        let nueva = Capacidad::new("modulo_x", "0.5"); // Sin async ni retry

        let result = fusion.fusionar(&existente, &nueva);
        assert_eq!(result.version, "1.0");
        assert!(result.complejidad_asincrona);
        assert!(result.robustez_retry);
    }

    #[test]
    fn test_compatible_fusiona_extra() {
        let fusion = FusionSelectiva::new();
        let existente = Capacidad::new("modulo_y", "1.0")
            .with_async()
            .with_extra("modo texto");
        let nueva = Capacidad::new("modulo_y", "1.1")
            .with_async()
            .with_extra("modo voz");

        let result = fusion.fusionar(&existente, &nueva);
        assert_eq!(result.version, "Fusionada (Híbrido)");
        assert!(result.extra.unwrap().contains("modo voz"));
    }

    #[test]
    fn test_identica_ignora() {
        let fusion = FusionSelectiva::new();
        let existente = Capacidad::new("modulo_z", "1.0");
        let nueva = Capacidad::new("modulo_z", "1.0");

        let result = fusion.fusionar(&existente, &nueva);
        assert_eq!(result.version, "1.0");
    }
}
