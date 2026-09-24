// ==========================================
// 🔱 NEXUS OMEGA — Telegram Bridge (Legacy Compatibility)
// ==========================================
// Re-exporta send_alert desde el nuevo Bridge en comms::telegram_bridge.
// Mantiene compatibilidad con código legacy que importa:
//   crate::nexus_telegram::send_alert()
// o via infra:
//   crate::infra::nexus_telegram::send_alert()
// ==========================================

pub use crate::comms::telegram_bridge::send_alert;
