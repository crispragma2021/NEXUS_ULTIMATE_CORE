// 🔱 holehe_rs — Transmutación Rust Pura de holehe (OSINT email checker)
// Cero dependencias externas nuevas. Usa reqwest del arsenal existente.
// Verifica existencia de emails en 28+ servicios online.

pub mod checker;

pub use checker::*;
