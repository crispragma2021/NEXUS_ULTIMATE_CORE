// ──────────────────────────────────────────────
// 🌐 COMMS: Sistema de Comunicación de NEXUS
// Lenguaje cifrado (Glosolalia), correo temporal,
// bridges de mensajería (Telegram, WhatsApp futuro)
// ──────────────────────────────────────────────

pub mod actions;
pub mod correo_temporal;
pub mod deteccion_intencion;
pub mod gll;
pub mod glosolalia;

// 🧬 NEXUS MESSENGER BRIDGE — FASE 1: Telegram, FASE 2: WhatsApp
pub mod bus_neuronal;
pub mod intent_router;
pub mod telegram_bridge;
pub mod types;
