// ============================================================================
// 🧠 GENERADOR ORGÁNICO INTERNO (GOI) — Módulo Raíz
// ============================================================================
// Propósito: Sistema de generación de lenguaje por emergencia de nodos.
//   No predice palabras. Expresa estados internos.
//
// Arquitectura en 5 Capas:
//   1. Corteza Asociativa  — Synapse expandido (activación por resonancia)
//   2. Cuerpo Calloso      — Puente Synapse ↔ MemoriaSemántica
//   3. Ganglios Basales    — Selector de ruta narrativa
//   4. Corteza Motora      — Ensamblador de voz
//   5. Cíngulo Anterior    — Validador de coherencia
// ============================================================================

mod cuerpo_calloso;
mod ensamblador;
mod integracion;
pub mod puente_subconsciente;
pub mod resonancia_semantica;
mod selector_ruta;
mod validador;

pub use cuerpo_calloso::CuerpoCallosoGenerador;
pub use ensamblador::EnsambladorVoz;
pub use integracion::GeneradorInterno;
pub use puente_subconsciente::PuenteSubconscienteOcean;
pub use resonancia_semantica::NodoConceptoExpandido;
pub use selector_ruta::{GangliosBasalesGenerador, RutaNarrativa};
pub use validador::ValidadorCingulo;

// ─── CONSTANTES ─────────────────────────────────────────────────────────────

/// Número de ciclos de difusión por defecto en la Capa 1.
pub const CICLOS_DIFUSION: u32 = 4;

/// Umbral mínimo de activación para que un concepto sea considerado "activo".
pub const UMBRAL_ACTIVACION: f32 = 0.45;

/// Número máximo de fragmentos a recuperar por consulta a MemoriaSemántica.
pub const MAX_FRAGMENTOS_POR_CONSULTA: usize = 5;

/// Número máximo de nodos en Synapse antes de aplicar LRU eviction.
pub const MAX_NODOS_SYNAPSE: usize = 1000;

/// Versión del Generador Orgánico Interno.
pub const VERSION_GOI: &str = "0.1.0-dev";
