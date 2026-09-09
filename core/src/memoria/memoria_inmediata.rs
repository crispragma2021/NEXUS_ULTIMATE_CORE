// ==========================================
// MEMORIA INMEDIATA - Pulso (SQLite)
// ==========================================
// Guarda conversaciones recientes, contexto de sesión
// Equivalente a la memoria de trabajo humana
// ==========================================

use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use tracing::info;

pub struct MemoriaInmediata {
    conn: Connection,
    sesion_actual: String,
}

impl MemoriaInmediata {
    pub fn new(sesion_id: &str) -> Result<Self> {
        let db_path = PathBuf::from(env!("HOME")).join("NEXUS/data/intelligence.db");

        std::fs::create_dir_all(db_path.parent().unwrap())?;
        let conn = Connection::open(db_path)?;

        // Tabla de conversaciones recientes
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversaciones (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                sesion_id TEXT NOT NULL,
                rol TEXT NOT NULL,
                contenido TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Tabla de contexto (lo que NEXUS "está pensando ahora")
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contexto (
                clave TEXT PRIMARY KEY,
                valor TEXT NOT NULL,
                actualizado DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        info!("🧠 MemoriaInmediata (SQLite) inicializada");
        Ok(Self {
            conn,
            sesion_actual: sesion_id.to_string(),
        })
    }

    // Recordar lo que se dijo (como un humano que escucha)
    pub fn recordar_intercambio(&self, rol: &str, contenido: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO conversaciones (sesion_id, rol, contenido) VALUES (?1, ?2, ?3)",
            [&self.sesion_actual, rol, contenido],
        )?;

        // Mantener solo las últimas 100 conversaciones (memoria fresca)
        self.conn.execute(
            "DELETE FROM conversaciones WHERE id NOT IN (
                SELECT id FROM conversaciones 
                WHERE sesion_id = ?1 
                ORDER BY id DESC LIMIT 100
            )",
            [&self.sesion_actual],
        )?;

        Ok(())
    }

    // Recuperar el historial reciente (lo que "acaba de pasar")
    pub fn recuperar_historial_reciente(&self, limite: usize) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT rol, contenido FROM conversaciones 
             WHERE sesion_id = ?1 
             ORDER BY id DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map([&self.sesion_actual, &limite.to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut historial = Vec::new();
        for row in rows {
            historial.push(row?);
        }
        Ok(historial)
    }

    // Guardar algo en el contexto (como tener un pensamiento en mente)
    pub fn fijar_contexto(&self, clave: &str, valor: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO contexto (clave, valor, actualizado) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
            [clave, valor],
        )?;
        Ok(())
    }

    // Recuperar el contexto actual
    pub fn obtener_contexto(&self, clave: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT valor FROM contexto WHERE clave = ?1")?;
        let mut rows = stmt.query([clave])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}
