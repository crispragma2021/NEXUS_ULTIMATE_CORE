// ============================================================================
// 🧬 Módulo Léxico — Lenguaje Emergente Matemático
// ============================================================================
// Principio: las palabras emergen del campo de activación neuronal mediante
// softmax sobre pesos STDP. No hay listas fijas, no hay switches, no hay
// palabras preprogramadas en el código de generación.
//
// La semilla inicial (~64 tokens) se carga desde binario pero el vocabulario
// crece orgánicamente con el uso del cerebro via STDP léxico.
// ============================================================================

pub mod mediador;
pub mod asambleas;

pub use mediador::{MediadorConsciencia, MediadorInmutable, CorrienteConsciencia, EstadoMentalActivo, NeuroquimicaSnapshot};
