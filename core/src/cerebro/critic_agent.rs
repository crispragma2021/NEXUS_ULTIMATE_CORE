// 🦾 [PUENTE OMEGA] cerebro/critic_agent.rs → valores/critic_agent.rs
// El CriticAgent ahora reside en el sistema de valores (valores/)
// Este puente mantiene compatibilidad con imports legacy.

pub use crate::valores::critic_agent::{AuditResult, CriticAgent};
