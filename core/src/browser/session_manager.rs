// ============================================================================
// 🧬 NEXUS Session Manager — Persistencia de Sesiones SQLite
// ============================================================================
// Almacena cookies, localStorage, sessionStorage y metadatos de perfil
// por sesión de navegador. Permite reanudar sesiones autenticadas.
// ============================================================================

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Tipos de datos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieData {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
    pub expires: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageData {
    pub localStorage: Vec<StorageEntry>,
    pub sessionStorage: Vec<StorageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub profile_id: String,
    pub user_agent: String,
    pub viewport: (u32, u32),
    pub last_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub metadata: SessionMetadata,
    pub cookies: Vec<CookieData>,
    pub storage: StorageData,
}

// ---------------------------------------------------------------------------
// Session Manager
// ---------------------------------------------------------------------------

pub struct SessionManager {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

impl SessionManager {
    /// Abre (o crea) la base de datos SQLite de sesiones.
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| {
            let mut p =
                PathBuf::from(std::env::var("NEXUS_ROOT").unwrap_or_else(|_| "/tmp".into()));
            p.push("nexus_sessions.db");
            p
        });

        let conn =
            Connection::open(&path).context("No se pudo abrir la base de datos de sesiones")?;

        // Crear tablas si no existen
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                profile_id TEXT NOT NULL,
                user_agent TEXT NOT NULL,
                viewport_width INTEGER NOT NULL DEFAULT 1920,
                viewport_height INTEGER NOT NULL DEFAULT 1080,
                last_url TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS cookies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                name TEXT NOT NULL,
                value TEXT NOT NULL,
                domain TEXT NOT NULL,
                path TEXT NOT NULL DEFAULT '/',
                secure INTEGER NOT NULL DEFAULT 0,
                http_only INTEGER NOT NULL DEFAULT 0,
                same_site TEXT NOT NULL DEFAULT 'Lax',
                expires INTEGER,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS storage (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                store_type TEXT NOT NULL CHECK(store_type IN ('local', 'session')),
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_cookies_session ON cookies(session_id);
            CREATE INDEX IF NOT EXISTS idx_storage_session ON storage(session_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_profile ON sessions(profile_id);
            ",
        )
        .context("No se pudieron crear las tablas de sesiones")?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path: path,
        })
    }

    // -----------------------------------------------------------------------
    // CRUD de sesiones
    // -----------------------------------------------------------------------

    /// Crea una nueva sesión vacía y devuelve su ID.
    pub fn create_session(&self, metadata: SessionMetadata) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO sessions (id, profile_id, user_agent, viewport_width, viewport_height, last_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                metadata.profile_id,
                metadata.user_agent,
                metadata.viewport.0,
                metadata.viewport.1,
                metadata.last_url,
                metadata.created_at,
                metadata.updated_at,
            ],
        )?;

        Ok(id)
    }

    /// Guarda una sesión completa (cookies + storage).
    pub fn save_session(&self, session: &Session) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Upsert de la sesión
        conn.execute(
            "INSERT INTO sessions (id, profile_id, user_agent, viewport_width, viewport_height, last_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                last_url = excluded.last_url,
                updated_at = excluded.updated_at",
            params![
                session.id,
                session.metadata.profile_id,
                session.metadata.user_agent,
                session.metadata.viewport.0,
                session.metadata.viewport.1,
                session.metadata.last_url,
                session.metadata.created_at,
                session.metadata.updated_at,
            ],
        )?;

        // Limpiar cookies viejas y reinserter
        conn.execute(
            "DELETE FROM cookies WHERE session_id = ?1",
            params![session.id],
        )?;
        for cookie in &session.cookies {
            conn.execute(
                "INSERT INTO cookies (session_id, name, value, domain, path, secure, http_only, same_site, expires)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    session.id,
                    cookie.name,
                    cookie.value,
                    cookie.domain,
                    cookie.path,
                    cookie.secure as i32,
                    cookie.http_only as i32,
                    cookie.same_site,
                    cookie.expires,
                ],
            )?;
        }

        // Limpiar storage viejo y reinserter
        conn.execute(
            "DELETE FROM storage WHERE session_id = ?1",
            params![session.id],
        )?;
        for entry in &session.storage.localStorage {
            conn.execute(
                "INSERT INTO storage (session_id, store_type, key, value) VALUES (?1, 'local', ?2, ?3)",
                params![session.id, entry.key, entry.value],
            )?;
        }
        for entry in &session.storage.sessionStorage {
            conn.execute(
                "INSERT INTO storage (session_id, store_type, key, value) VALUES (?1, 'session', ?2, ?3)",
                params![session.id, entry.key, entry.value],
            )?;
        }

        Ok(())
    }

    /// Carga una sesión completa por ID.
    pub fn load_session(&self, session_id: &str) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, profile_id, user_agent, viewport_width, viewport_height, last_url, created_at, updated_at
             FROM sessions WHERE id = ?1",
        )?;

        let session_row = stmt.query_row(params![session_id], |row| {
            Ok(SessionMetadata {
                profile_id: row.get(1)?,
                user_agent: row.get(2)?,
                viewport: (row.get::<_, u32>(3)?, row.get::<_, u32>(4)?),
                last_url: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });

        match session_row {
            Ok(metadata) => {
                // Cargar cookies
                let mut cookie_stmt = conn.prepare(
                    "SELECT name, value, domain, path, secure, http_only, same_site, expires
                     FROM cookies WHERE session_id = ?1",
                )?;
                let cookies = cookie_stmt
                    .query_map(params![session_id], |row| {
                        Ok(CookieData {
                            name: row.get(0)?,
                            value: row.get(1)?,
                            domain: row.get(2)?,
                            path: row.get(3)?,
                            secure: row.get::<_, i32>(4)? != 0,
                            http_only: row.get::<_, i32>(5)? != 0,
                            same_site: row.get(6)?,
                            expires: row.get(7)?,
                        })
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();

                // Cargar storage
                let mut storage_stmt = conn
                    .prepare("SELECT store_type, key, value FROM storage WHERE session_id = ?1")?;

                let mut localStorage = Vec::new();
                let mut sessionStorage = Vec::new();

                let storage_rows = storage_stmt.query_map(params![session_id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?;

                for row in storage_rows.flatten() {
                    match row.0.as_str() {
                        "local" => localStorage.push(StorageEntry {
                            key: row.1,
                            value: row.2,
                        }),
                        "session" => sessionStorage.push(StorageEntry {
                            key: row.1,
                            value: row.2,
                        }),
                        _ => {}
                    }
                }

                Ok(Some(Session {
                    id: session_id.to_string(),
                    metadata,
                    cookies,
                    storage: StorageData {
                        localStorage,
                        sessionStorage,
                    },
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Lista todas las sesiones disponibles (sin datos pesados).
    pub fn list_sessions(&self) -> Result<Vec<(String, SessionMetadata)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, profile_id, user_agent, viewport_width, viewport_height, last_url, created_at, updated_at
             FROM sessions ORDER BY updated_at DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    SessionMetadata {
                        profile_id: row.get(1)?,
                        user_agent: row.get(2)?,
                        viewport: (row.get::<_, u32>(3)?, row.get::<_, u32>(4)?),
                        last_url: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    },
                ))
            })?
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>();

        Ok(sessions)
    }

    /// Elimina una sesión y todos sus datos asociados.
    pub fn delete_session(&self, session_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let affected = conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
        // CASCADE debería limpiar cookies y storage
        Ok(affected > 0)
    }

    /// Obtiene la ruta de la base de datos.
    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_load_session() {
        let manager = SessionManager::new(Some(PathBuf::from(":memory:"))).unwrap();

        let metadata = SessionMetadata {
            profile_id: "test-profile".into(),
            user_agent: "Mozilla/5.0 Test".into(),
            viewport: (1920, 1080),
            last_url: Some("https://example.com".into()),
            created_at: 1000,
            updated_at: 1000,
        };

        let id = manager.create_session(metadata.clone()).unwrap();

        let session = Session {
            id: id.clone(),
            metadata,
            cookies: vec![CookieData {
                name: "session_id".into(),
                value: "abc123".into(),
                domain: ".example.com".into(),
                path: "/".into(),
                secure: true,
                http_only: true,
                same_site: "Lax".into(),
                expires: Some(9999999999),
            }],
            storage: StorageData {
                localStorage: vec![StorageEntry {
                    key: "theme".into(),
                    value: "dark".into(),
                }],
                sessionStorage: vec![],
            },
        };

        manager.save_session(&session).unwrap();

        let loaded = manager.load_session(&id).unwrap().unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.cookies.len(), 1);
        assert_eq!(loaded.storage.localStorage.len(), 1);
        assert_eq!(loaded.cookies[0].name, "session_id");
        assert_eq!(loaded.storage.localStorage[0].value, "dark");

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
    }
}
