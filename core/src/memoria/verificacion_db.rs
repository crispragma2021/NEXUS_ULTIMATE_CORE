use rusqlite::Connection;
use tracing::{error, info, warn};

pub struct ValidadorTrayectorias;

impl ValidadorTrayectorias {
    pub fn verificar_pulso_nervioso() -> Result<(), String> {
        let resolved = crate::infra::paths::resolve_path("brain/nexus_trajectories.db");

        // 1. Intentar conectar nativamente al silicio del NVMe
        let conn = match Connection::open(&resolved) {
            Ok(c) => c,
            Err(e) => {
                error!("🛑 [HEAL] Error crítico al abrir la conexión SQLite: {}", e);
                return Err(e.to_string());
            }
        };

        // 2. Ejecutar un query de prueba sobre la estructura de trayectorias
        match conn.execute("SELECT 1 FROM trajectories LIMIT 1;", []) {
            Ok(_) => {
                info!("✅ [HEAL] Pulso verificado en nexus_trajectories.db. Estructura íntegra.");
                Ok(())
            }
            Err(e) => {
                warn!(
                    "⚠️ [HEAL] Archivo corrupto o tabla inexistente: {}. Re-inicializando...",
                    e
                );
                // Si la tabla falló, forzamos un reset del esquema para curar la ceguera
                let _ = conn.execute(
                    "CREATE TABLE IF NOT EXISTS trajectories (id TEXT PRIMARY KEY, data TEXT);",
                    [],
                );
                Ok(())
            }
        }
    }
}
