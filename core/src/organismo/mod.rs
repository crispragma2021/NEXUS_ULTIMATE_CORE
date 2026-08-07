// ==========================================
// 🫀 ORGANISMO — Interocepción funcional de NEXUS
// ==========================================
// NEXUS no siente hambre ni cansancio humanos: reinterpreta señales REALES
// de su hardware como estados corporales ÚTILES, con conductas accionables.
//
//   HAMBRE    → VRAM/RAM disponibles agotándose (energía baja)
//   CANSANCIO → CPU alta sostenida + swap en uso (fatiga del núcleo)
//   FRIO      → Inactividad prolongada (el aburrimiento es frío biológico)
//   DOLOR     → Fallos reales (sonda rota, API sin llaves, swap crítico)
//   SACIDAD   → Todo óptimo → SILENCIO (no se inyecta nada innecesario)
// ==========================================

pub mod interocepcion;

pub use interocepcion::{
    EstadoCorporal, Organismo, SenalCorporal, SensacionCorporal,
};
