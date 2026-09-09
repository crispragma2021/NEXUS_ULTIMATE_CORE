// ==========================================
// MEMORIA EPISÓDICA - Estructura (SQLite → nexus_memoria.db)
// ==========================================
// Guarda relaciones entre eventos, causa-efecto.
// Equivalente a "recordar experiencias personales".
// AHORA apunta a nexus_memoria.db unificada.
// ==========================================

use anyhow::{Context, Result};
use chrono::Local;
use rusqlite::{params, Connection};
use tracing::{error, info};

const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";

pub struct MemoriaEpisodica {
    db_path: std::path::PathBuf,
}

impl MemoriaEpisodica {
    pub fn new() -> Result<Self> {
        // ✅ AHORA apunta a nexus_memoria.db en vez de data/memoria_episodica.db
        let db_path = crate::infra::paths::resolve_path(NEXUS_MEMORIA_DB);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Inicializar esquema en SQLite (solo si no existe)
        let conn = Connection::open(&db_path)
            .context("No se pudo abrir nexus_memoria.db para memoria episódica")?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS eventos (
                id   INTEGER PRIMARY KEY AUTOINCREMENT,
                nombre TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS relaciones (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                origen    TEXT NOT NULL,
                destino   TEXT NOT NULL,
                tipo      TEXT NOT NULL,
                timestamp TEXT NOT NULL
            );",
        )
        .context("No se pudo inicializar el esquema de relaciones en nexus_memoria.db")?;

        info!("🕸️ MemoriaEpisodica — Relaciones en nexus_memoria.db");
        Ok(Self { db_path })
    }

    // Registrar una relación entre eventos
    pub fn registrar_relacion(&self, evento_a: &str, evento_b: &str, relacion: &str) -> Result<()> {
        let conn = Connection::open(&self.db_path).context("No se pudo abrir nexus_memoria.db")?;

        let ahora = Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO relaciones (origen, destino, tipo, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![evento_a, evento_b, relacion, ahora],
        )
        .context("Fallo al registrar relación en nexus_memoria.db")?;

        info!(
            "🔗 Relación registrada: {} --{}-> {}",
            evento_a, relacion, evento_b
        );
        Ok(())
    }

    /// Busca la cadena de eventos que precedieron a un evento específico
    pub fn buscar_cadena_causal(&self, evento_final: &str) -> Result<Vec<(String, String)>> {
        let conn = Connection::open(&self.db_path).context("No se pudo abrir nexus_memoria.db")?;

        let mut stmt = conn.prepare(
            "SELECT origen, timestamp FROM relaciones WHERE destino = ?1 ORDER BY timestamp",
        )?;

        let cadena: Vec<(String, String)> = stmt
            .query_map(params![evento_final], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| {
                r.map_err(|e| error!("Error leyendo relación causal: {}", e))
                    .ok()
            })
            .collect();

        Ok(cadena)
    }
}
