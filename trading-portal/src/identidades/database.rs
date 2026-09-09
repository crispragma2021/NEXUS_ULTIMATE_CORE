use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tracing::info;

use super::identity_factory::Identidad;

// ============================================================================
// BASE DE DATOS DE IDENTIDADES — Hipocampo Cognitivo Mejorado
// ============================================================================
// Reemplaza y mejora legacy/nexus-orquestador/src/hipocampo_cognitivo.rs
// Tabla unificada synthetic_identities con esquema enriquecido
// ============================================================================

pub struct IdentidadDb {
    conn: Mutex<Connection>,
}

impl IdentidadDb {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;

        // Crear tabla de identidades con esquema completo
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS synthetic_identities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                email TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                nombre TEXT NOT NULL DEFAULT '',
                apellido TEXT NOT NULL DEFAULT '',
                segundo_apellido TEXT DEFAULT '',
                fecha_nacimiento TEXT DEFAULT '',
                pais TEXT DEFAULT '',
                ciudad TEXT DEFAULT '',
                genero TEXT DEFAULT '',
                telefono TEXT DEFAULT '',
                recovery_email TEXT,
                tipo TEXT DEFAULT 'sintetico',
                estado TEXT DEFAULT 'creada',
                email_provider TEXT DEFAULT '',
                metadata_json TEXT,
                foto_url TEXT DEFAULT '',
                creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
                ultimo_uso DATETIME,
                veces_usada INTEGER DEFAULT 0,
                api_keys TEXT DEFAULT '[]',
                plataformas TEXT DEFAULT '[]'
            )",
        )?;

        // Índices para búsquedas rápidas
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_identities_estado ON synthetic_identities(estado);
             CREATE INDEX IF NOT EXISTS idx_identities_tipo ON synthetic_identities(tipo);
             CREATE INDEX IF NOT EXISTS idx_identities_provider ON synthetic_identities(email_provider);
             CREATE INDEX IF NOT EXISTS idx_identities_pais ON synthetic_identities(pais);",
        )?;

        info!("🗄️ [DB] Base de datos de identidades inicializada: {}", path);

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Registra una nueva identidad
    pub fn registrar_identidad(
        &self,
        email: &str,
        password: &str,
        recovery_email: Option<&str>,
        estado: &str,
        email_provider: &str,
        metadata: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();

        // Extraer campos del metadata
        let nombre = metadata["nombre"].as_str().unwrap_or("");
        let apellido = metadata["apellido"].as_str().unwrap_or("");
        let segundo = metadata["segundo_apellido"].as_str().unwrap_or("");
        let fecha_nac = metadata["fecha_nacimiento"].as_str().unwrap_or("");
        let pais = metadata["pais"].as_str().unwrap_or("");
        let ciudad = metadata["ciudad"].as_str().unwrap_or("");
        let genero = metadata["genero"].as_str().unwrap_or("");
        let telefono = metadata["telefono"].as_str().unwrap_or("");
        let tipo = metadata["tipo"].as_str().unwrap_or("sintetico");

        conn.execute(
            "INSERT OR REPLACE INTO synthetic_identities
             (email, password, nombre, apellido, segundo_apellido,
              fecha_nacimiento, pais, ciudad, genero, telefono,
              recovery_email, tipo, estado, email_provider, metadata_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                email,
                password,
                nombre,
                apellido,
                segundo,
                fecha_nac,
                pais,
                ciudad,
                genero,
                telefono,
                recovery_email,
                tipo,
                estado,
                email_provider,
                metadata.to_string()
            ],
        )?;

        Ok(())
    }

    /// Lista todas las identidades
    pub fn listar_identidades(&self, limit: usize) -> anyhow::Result<Vec<Identidad>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, password, nombre, apellido, segundo_apellido,
                    fecha_nacimiento, pais, ciudad, genero, telefono,
                    recovery_email, tipo, estado, email_provider, metadata_json,
                    creado_en, ultimo_uso
             FROM synthetic_identities
             ORDER BY creado_en DESC
             LIMIT ?",
        )?;

        let identidades = stmt
            .query_map(params![limit as i64], |row| {
                Ok(Identidad {
                    id: Some(row.get(0)?),
                    email: row.get(1)?,
                    password: row.get(2)?,
                    nombre: row.get(3)?,
                    apellido: row.get(4)?,
                    segundo_apellido: row.get(5)?,
                    fecha_nacimiento: row.get(6)?,
                    pais: row.get(7)?,
                    ciudad: row.get(8)?,
                    genero: row.get(9)?,
                    telefono: row.get(10)?,
                    recovery_email: row.get(11)?,
                    tipo: row.get(12)?,
                    estado: row.get(13)?,
                    email_provider: row.get(14)?,
                    metadata_json: row.get(15)?,
                    creado_en: row.get(16)?,
                    ultimo_uso: row.get(17)?,
                    foto_url: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(identidades)
    }

    /// Obtiene una identidad por email
    pub fn obtener_identidad(&self, email: &str) -> anyhow::Result<Option<Identidad>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, email, password, nombre, apellido, segundo_apellido,
                    fecha_nacimiento, pais, ciudad, genero, telefono,
                    recovery_email, tipo, estado, email_provider, metadata_json,
                    creado_en, ultimo_uso
             FROM synthetic_identities
             WHERE email = ?",
        )?;

        let mut rows = stmt.query_map(params![email], |row| {
            Ok(Identidad {
                id: Some(row.get(0)?),
                email: row.get(1)?,
                password: row.get(2)?,
                nombre: row.get(3)?,
                apellido: row.get(4)?,
                segundo_apellido: row.get(5)?,
                fecha_nacimiento: row.get(6)?,
                pais: row.get(7)?,
                ciudad: row.get(8)?,
                genero: row.get(9)?,
                telefono: row.get(10)?,
                recovery_email: row.get(11)?,
                tipo: row.get(12)?,
                estado: row.get(13)?,
                email_provider: row.get(14)?,
                metadata_json: row.get(15)?,
                creado_en: row.get(16)?,
                ultimo_uso: row.get(17)?,
                foto_url: None,
            })
        })?;

        match rows.next() {
            Some(Ok(identidad)) => Ok(Some(identidad)),
            _ => Ok(None),
        }
    }

    /// Actualiza estado de una identidad
    pub fn actualizar_estado(&self, email: &str, estado: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE synthetic_identities SET estado = ?1, ultimo_uso = CURRENT_TIMESTAMP,
             veces_usada = veces_usada + 1 WHERE email = ?2",
            params![estado, email],
        )?;
        Ok(())
    }

    /// Asigna API keys a una identidad (para cuentas Google/Gemini)
    pub fn asignar_api_key(&self, email: &str, api_key: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE synthetic_identities SET
             api_keys = CASE
                WHEN api_keys IS NULL OR api_keys = '[]' THEN json_array(?1)
                ELSE json_insert(api_keys, '$[#]', ?1)
             END
             WHERE email = ?2",
            params![api_key, email],
        )?;
        Ok(())
    }

    /// Reporte estadístico de identidades
    pub fn reporte_estadistico(&self) -> anyhow::Result<serde_json::Value> {
        let conn = self.conn.lock().unwrap();

        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM synthetic_identities", params![], |row| {
                row.get(0)
            })?;

        let por_estado: serde_json::Value = {
            let mut stmt = conn.prepare(
                "SELECT estado, COUNT(*) as cnt FROM synthetic_identities GROUP BY estado",
            )?;
            let rows = stmt
                .query_map(params![], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let mut map = serde_json::Map::new();
            for (estado, count) in rows {
                map.insert(estado, serde_json::Value::Number(count.into()));
            }
            serde_json::Value::Object(map)
        };

        let por_tipo: serde_json::Value = {
            let mut stmt = conn.prepare(
                "SELECT tipo, COUNT(*) as cnt FROM synthetic_identities GROUP BY tipo",
            )?;
            let rows = stmt
                .query_map(params![], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect::<Vec<_>>();

            let mut map = serde_json::Map::new();
            for (tipo, count) in rows {
                map.insert(tipo, serde_json::Value::Number(count.into()));
            }
            serde_json::Value::Object(map)
        };

        let activas: i64 = conn.query_row(
            "SELECT COUNT(*) FROM synthetic_identities WHERE estado = 'activa'",
            params![],
            |row| row.get(0),
        )?;

        Ok(serde_json::json!({
            "total": total,
            "activas": activas,
            "por_estado": por_estado,
            "por_tipo": por_tipo,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
