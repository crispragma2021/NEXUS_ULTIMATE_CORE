// ============================================================================
// 💡 LÓBULO DE IMAGINACIÓN — Creatividad y Generación de Hipótesis (OMEGA)
// ============================================================================
// Absorbido de: legacy/nexus-orquestador/src/procesos/creatividad.rs
// Propósito: Generar conexiones creativas entre dominios para explorar
//            soluciones no pedidas (pensamiento divergente).
// ============================================================================

use rand::seq::SliceRandom;
use tracing::info;

#[derive(Debug, Clone)]
pub struct SemillaCreativa {
    pub dominio: String,
    pub concepto: String,
}

/// 💡 Lóbulo de Imaginación: genera hipótesis "¿Y si...?" conectando dominios.
pub struct LobuloImaginacion {
    pub semillas: Vec<SemillaCreativa>,
}

impl Default for LobuloImaginacion {
    fn default() -> Self {
        Self::new()
    }
}

impl LobuloImaginacion {
    pub fn new() -> Self {
        let mut s = Vec::new();
        s.push(SemillaCreativa {
            dominio: "Biología".to_string(),
            concepto: "Homeostasis (Autocuración)".to_string(),
        });
        s.push(SemillaCreativa {
            dominio: "Física".to_string(),
            concepto: "Entropía (Limpieza de código muerto)".to_string(),
        });
        s.push(SemillaCreativa {
            dominio: "Música".to_string(),
            concepto: "Armonía (Sincronización de hilos)".to_string(),
        });
        s.push(SemillaCreativa {
            dominio: "Psicología".to_string(),
            concepto: "Inconsciente (Procesos en segundo plano)".to_string(),
        });

        Self { semillas: s }
    }

    /// 💡 Genera una ráfaga de "¿Y si...?" para explorar lo no pedido.
    pub fn generar_hipotesis_loca(&self) {
        let mut rng = rand::thread_rng();
        if let Some(semilla) = self.semillas.choose(&mut rng) {
            info!(
                "💡 [CREATIVIDAD] ¿Y si aplicamos {} de {} al sistema actual?",
                semilla.concepto, semilla.dominio
            );
            info!("🎨 NEXUS imaginando una conexión no pedida por el Arquitecto...");
        }
    }

    /// ⚖️ Evalúa si una acción fue un acto de creatividad pura.
    pub fn evaluar_acto_creativo(fue_pedida: bool, impacto_inesperado: bool) -> f32 {
        let mut score = 0.0;
        if !fue_pedida {
            score += 0.5;
        }
        if impacto_inesperado {
            score += 0.5;
        }
        score
    }
}
