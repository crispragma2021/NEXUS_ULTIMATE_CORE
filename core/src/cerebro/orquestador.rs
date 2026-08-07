// ==========================================
// ORQUESTADOR - CEREBRO COMPLETO DE NEXUS
// ==========================================
// Contenedor principal que divide responsabilidades en 3 submódulos:
//
//   constructor.rs  → struct Orquestador (46 fields) + new()
//   diagnostico.rs  → escanear_warnings, autodiagnosticar, diagnostico
//   pipeline.rs     → responder(), fallbacks, clasificar_tarea
//
// Uso externo: use crate::cerebro::orquestador::Orquestador;
// ==========================================

// Los submódulos se cargan desde archivos hermanos mediante #[path]
#[path = "constructor.rs"]
mod constructor;
#[path = "diagnostico.rs"]
mod diagnostico;
#[path = "pipeline.rs"]
mod pipeline;

// Re-export público para compatibilidad:
//   crate::cerebro::orquestador::Orquestador
pub use constructor::Orquestador;
