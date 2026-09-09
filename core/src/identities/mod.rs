// 🌱 NEXUS OMEGA — Módulo Sembrador de Identidades
// ============================================================
// Fachada pública del subsistema de identidades sintéticas.

pub mod browser_profile;
pub mod chrome_planter;
pub mod destroyer;
pub mod generator;
pub mod mail_factory;
pub mod rotator;
pub mod sembrador;
pub mod sms_activate;
pub mod storage;
pub mod types;

// Re-exportaciones públicas de la API de alto nivel
pub use browser_profile::BrowserProfileManager;
pub use chrome_planter::ChromePlanter;
pub use destroyer::IdentityDestroyer;
pub use generator::IdentityGenerator;
pub use mail_factory::MailFactory;
pub use rotator::IdentityRotator;
pub use sms_activate::SmsActivateClient;
pub use storage::IdentityStore;
pub use types::*;
