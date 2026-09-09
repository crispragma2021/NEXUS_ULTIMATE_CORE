// 💖 SISTEMA LÍMBICO DE NEXUS
// Amígdala, memoria emocional (OCEAN), dopamina, sentimientos

pub mod alarma_sistemica;
pub mod limbico;
pub mod ocean;
pub mod pulso;
pub mod sentimiento;

pub mod apego;
pub use alarma_sistemica::{AlarmaSistemica, EstadoInstintivo};
pub use limbico::{EstadoEmocional, Metacognicion, SistemaLimbico};
pub use sentimiento::SentimientoSoberano;
