use rusqlite::{params, Connection};
use std::sync::Mutex;

use super::identity_factory::Identidad;

/// Base de datos de identidades sintéticas
pub struct IdentidadDb {
    conn: Mutex<Connection>,
}

impl IdentidadDb {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        // Crear directorio si no existe
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA busy_timeout=5000;
             PRAGMA foreign_keys=ON;",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS synthetic_identities (
                id              TEXT PRIMARY KEY,
                nombre          TEXT NOT NULL,
                apellido        TEXT NOT NULL,
                segundo_apellido TEXT,
                email           TEXT UNIQUE NOT NULL,
                password        TEXT NOT NULL,
                recovery_email  TEXT,
                fecha_nacimiento TEXT NOT NULL,
                pais            TEXT NOT NULL,
                ciudad          TEXT NOT NULL,
                genero          TEXT NOT NULL,
                telefono        TEXT,
                foto_url        TEXT,
                tipo            TEXT NOT NULL DEFAULT 'Sintetico',
                estado          TEXT NOT NULL DEFAULT 'Creada',
                email_provider  TEXT,
                metadata_json   TEXT,
                api_key_asignada TEXT,
                creado_en       TEXT NOT NULL DEFAULT (datetime('now')),
                ultimo_uso      TEXT
            )",
            [],
        )?;

        // Índices para búsquedas frecuentes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_identidades_estado ON synthetic_identities(estado)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_identidades_tipo ON synthetic_identities(tipo)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_identidades_email_provider ON synthetic_identities(email_provider)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_identidades_pais ON synthetic_identities(pais)",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Registra una nueva identidad en la base de datos
    pub fn registrar_identidad(
        &self,
        id: &str,
        nombre: &str,
        apellido: &str,
        segundo_apellido: Option<&str>,
        email: &str,
        password: &str,
        recovery_email: Option<&str>,
        fecha_nacimiento: &str,
        pais: &str,
        ciudad: &str,
        genero: &str,
        telefono: Option<&str>,
        foto_url: Option<&str>,
        tipo: &str,
        estado: &str,
        email_provider: Option<&str>,
        metadata_json: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO synthetic_identities (
                id, nombre, apellido, segundo_apellido, email, password,
                recovery_email, fecha_nacimiento, pais, ciudad, genero,
                telefono, foto_url, tipo, estado, email_provider, metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            params![
                id,
                nombre,
                apellido,
                segundo_apellido,
                email,
                password,
                recovery_email,
                fecha_nacimiento,
                pais,
                ciudad,
                genero,
                telefono,
                foto_url,
                tipo,
                estado,
                email_provider,
                metadata_json
            ],
        )?;
        Ok(())
    }

    /// Lista identidades con paginación
    pub fn listar_identidades(&self, limit: usize) -> anyhow::Result<Vec<Identidad>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, nombre, apellido, segundo_apellido, email, password,
                    recovery_email, fecha_nacimiento, pais, ciudad, genero,
                    telefono, foto_url, tipo, estado, email_provider, metadata_json,
                    creado_en, ultimo_uso
             FROM synthetic_identities
             ORDER BY creado_en DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(Identidad {
                id: row.get(0)?,
                nombre: row.get(1)?,
                apellido: row.get(2)?,
                segundo_apellido: row.get(3)?,
                email: row.get(4)?,
                password: row.get(5)?,
                recovery_email: row.get(6)?,
                fecha_nacimiento: row.get(7)?,
                pais: row.get(8)?,
                ciudad: row.get(9)?,
                genero: row.get(10)?,
                telefono: row.get(11)?,
                foto_url: row.get(12)?,
                tipo: row.get(13)?,
                estado: row.get(14)?,
                email_provider: row.get(15)?,
                metadata_json: row.get(16)?,
                creado_en: row.get(17)?,
                ultimo_uso: row.get(18)?,
            })
        })?;

        let mut identidades = Vec::new();
        for row in rows {
            identidades.push(row?);
        }
        Ok(identidades)
    }

    /// Obtiene una identidad por email
    pub fn obtener_identidad(&self, email: &str) -> anyhow::Result<Option<Identidad>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, nombre, apellido, segundo_apellido, email, password,
                    recovery_email, fecha_nacimiento, pais, ciudad, genero,
                    telefono, foto_url, tipo, estado, email_provider, metadata_json,
                    creado_en, ultimo_uso
             FROM synthetic_identities
             WHERE email = ?1",
        )?;

        let mut rows = stmt.query_map(params![email], |row| {
            Ok(Identidad {
                id: row.get(0)?,
                nombre: row.get(1)?,
                apellido: row.get(2)?,
                segundo_apellido: row.get(3)?,
                email: row.get(4)?,
                password: row.get(5)?,
                recovery_email: row.get(6)?,
                fecha_nacimiento: row.get(7)?,
                pais: row.get(8)?,
                ciudad: row.get(9)?,
                genero: row.get(10)?,
                telefono: row.get(11)?,
                foto_url: row.get(12)?,
                tipo: row.get(13)?,
                estado: row.get(14)?,
                email_provider: row.get(15)?,
                metadata_json: row.get(16)?,
                creado_en: row.get(17)?,
                ultimo_uso: row.get(18)?,
            })
        })?;

        match rows.next() {
            Some(Ok(identidad)) => Ok(Some(identidad)),
            _ => Ok(None),
        }
    }

    /// Actualiza el estado de una identidad
    pub fn actualizar_estado(&self, email: &str, estado: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE synthetic_identities SET estado = ?1, ultimo_uso = datetime('now') WHERE email = ?2",
            params![estado, email],
        )?;
        Ok(())
    }

    /// Asigna una API key de Gemini a una identidad
    pub fn asignar_api_key(&self, email: &str, api_key: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE synthetic_identities SET api_key_asignada = ?1 WHERE email = ?2",
            params![api_key, email],
        )?;
        Ok(())
    }

    /// Reporte estadístico de identidades
    pub fn reporte_estadistico(&self) -> anyhow::Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();

        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM synthetic_identities", [], |row| {
                row.get(0)
            })?;

        let por_tipo: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT tipo, COUNT(*) as cnt FROM synthetic_identities GROUP BY tipo ORDER BY cnt DESC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "tipo": row.get::<_, String>(0)?,
                    "cantidad": row.get::<_, i64>(1)?
                }))
            })?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let por_estado: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT estado, COUNT(*) as cnt FROM synthetic_identities GROUP BY estado ORDER BY cnt DESC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "estado": row.get::<_, String>(0)?,
                    "cantidad": row.get::<_, i64>(1)?
                }))
            })?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let por_pais: Vec<serde_json::Value> = {
            let mut stmt = conn.prepare(
                "SELECT pais, COUNT(*) as cnt FROM synthetic_identities GROUP BY pais ORDER BY cnt DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(serde_json::json!({
                    "pais": row.get::<_, String>(0)?,
                    "cantidad": row.get::<_, i64>(1)?
                }))
            })?;
            rows.filter_map(|r| r.ok()).collect()
        };

        Ok(serde_json::json!({
            "total": total,
            "por_tipo": por_tipo,
            "por_estado": por_estado,
            "por_pais": por_pais
        }))
    }
}
