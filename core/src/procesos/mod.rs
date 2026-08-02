// 🔄 SISTEMA DE PROCESOS DE NEXUS
// Gobernador de recursos, sesiones, sistema inmune y telemetría

pub mod checkpoint;
pub mod fusion_selectiva;
pub mod limpiador_zombies;
pub mod mediador_puro;
pub mod resource_governor;
pub mod session;
pub mod sistema_inmune;
pub mod telemetry;

pub use sistema_inmune::{FaseInmune, SistemaInmune};
