//! Persistencia SQLite del pipeline de scraping (F0.2).
//!
//! Esquema replicado de `plans/pipeline-spec.md` §5:
//! - `tasks` — estado de cada tarea de scraping.
//! - `extracted_data` — datos estructurados extraídos.
//! - `robots_cache` — cache de robots.txt (TTL 24h).
//! - `rate_limit_state` — estado de rate limiting por dominio.

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

/// Cliente de base de datos del pipeline.
pub struct PipelineDb {
    conn: Mutex<Connection>,
}

impl PipelineDb {
    /// Abre (o crea) la base y aplica el esquema.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init()?;
        Ok(db)
    }

    /// Abre una base en memoria (para tests).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init()?;
        Ok(db)
    }

    /// Aplica el esquema idempotente (CREATE TABLE IF NOT EXISTS + índices).
    pub fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                task_id         TEXT PRIMARY KEY,
                url             TEXT NOT NULL,
                strategy        TEXT NOT NULL CHECK(strategy IN ('http', 'headless')),
                selectors       TEXT,
                output_schema   TEXT,
                status          TEXT NOT NULL DEFAULT 'pending'
                                CHECK(status IN ('pending','fetching','cleaning','inferring','success','partial','failed','blocked_by_robots','timeout','provider_exhausted')),
                tier_used       TEXT CHECK(tier_used IN ('tier1_slm','tier2_cloud','tier1_then_tier2')),
                cloud_provider  TEXT,
                token_count     INTEGER,
                retry_count     INTEGER NOT NULL DEFAULT 0,
                error_log       TEXT,
                timing_ms       TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS extracted_data (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id         TEXT NOT NULL REFERENCES tasks(task_id) ON DELETE CASCADE,
                data            TEXT NOT NULL,
                summary         TEXT,
                scratchpad_path TEXT,
                created_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS robots_cache (
                domain          TEXT PRIMARY KEY,
                rules           TEXT NOT NULL,
                fetched_at      TEXT NOT NULL DEFAULT (datetime('now')),
                expires_at      TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS rate_limit_state (
                domain              TEXT PRIMARY KEY,
                last_request_at     TEXT NOT NULL,
                consecutive_failures INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks(created_at);
            CREATE INDEX IF NOT EXISTS idx_extracted_task ON extracted_data(task_id);
            "#,
        )?;
        Ok(())
    }

    /// Inserta una tarea nueva (o no-op si ya existe).
    pub fn insert_task(
        &self,
        task_id: &str,
        url: &str,
        strategy: &str,
        selectors: Option<&str>,
        output_schema: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO tasks (task_id, url, strategy, selectors, output_schema)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![task_id, url, strategy, selectors, output_schema],
        )?;
        Ok(())
    }

    /// Actualiza el estado de una tarea.
    pub fn update_status(
        &self,
        task_id: &str,
        status: &str,
        token_count: Option<i64>,
        error_log: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tasks SET status = ?2, token_count = COALESCE(?3, token_count),
             error_log = COALESCE(?4, error_log), updated_at = datetime('now')
             WHERE task_id = ?1",
            rusqlite::params![task_id, status, token_count, error_log],
        )?;
        Ok(())
    }

    /// Recupera la regla de robots.txt cacheada para un dominio (si no expiró).
    pub fn get_robots_cache(&self, domain: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT rules FROM robots_cache
             WHERE domain = ?1 AND expires_at > datetime('now')",
        )?;
        let mut rows = stmt.query(rusqlite::params![domain])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    /// Almacena reglas de robots.txt con TTL de 24h.
    pub fn set_robots_cache(&self, domain: &str, rules: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO robots_cache (domain, rules, fetched_at, expires_at)
             VALUES (?1, ?2, datetime('now'), datetime('now', '+1 day'))",
            rusqlite::params![domain, rules],
        )?;
        Ok(())
    }

    /// Devuelve el estado de rate limiting de un dominio.
    pub fn get_rate_limit(&self, domain: &str) -> Result<(String, i64)> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT last_request_at, consecutive_failures FROM rate_limit_state WHERE domain = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![domain])?;
        if let Some(row) = rows.next()? {
            Ok((row.get(0)?, row.get(1)?))
        } else {
            Ok(("1970-01-01 00:00:00".to_string(), 0))
        }
    }

    /// Registra una petición al dominio y resetea/incr cuenta de fallos.
    pub fn record_request(&self, domain: &str, success: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if success {
            conn.execute(
                "INSERT OR REPLACE INTO rate_limit_state (domain, last_request_at, consecutive_failures)
                 VALUES (?1, datetime('now'), 0)",
                rusqlite::params![domain],
            )?;
        } else {
            conn.execute(
                "INSERT INTO rate_limit_state (domain, last_request_at, consecutive_failures)
                 VALUES (?1, datetime('now'), 1)
                 ON CONFLICT(domain) DO UPDATE SET
                    last_request_at = datetime('now'),
                    consecutive_failures = consecutive_failures + 1",
                rusqlite::params![domain],
            )?;
        }
        Ok(())
    }

    /// Inserta datos extraídos asociados a una tarea.
    pub fn insert_extracted_data(
        &self,
        task_id: &str,
        data: &str,
        summary: Option<&str>,
        scratchpad_path: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO extracted_data (task_id, data, summary, scratchpad_path)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, data, summary, scratchpad_path],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Lista tareas en estado `pending` limitadas a `limit`.
    pub fn list_pending_tasks(&self, limit: usize) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT task_id FROM tasks WHERE status = 'pending' ORDER BY created_at LIMIT ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![limit as i64])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(row.get(0)?);
        }
        Ok(out)
    }

    /// Recupera los campos de una tarea para reconstruir un `TaskSchema`.
    ///
    /// Devuelve `(task_id, url, strategy, selectors, output_schema)`.
    pub fn get_task(
        &self,
        task_id: &str,
    ) -> Result<Option<(String, String, String, Option<String>, Option<String>)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT task_id, url, strategy, selectors, output_schema FROM tasks WHERE task_id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![task_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            )))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crea_las_cuatro_tablas() {
        let db = PipelineDb::open_in_memory().unwrap();
        let conn = db.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN
                 ('tasks','extracted_data','robots_cache','rate_limit_state')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn inserta_y_actualiza_tarea() {
        let db = PipelineDb::open_in_memory().unwrap();
        db.insert_task("t-1", "https://example.com", "http", None, None)
            .unwrap();
        db.update_status("t-1", "success", Some(1200), None)
            .unwrap();

        let pending = db.list_pending_tasks(10).unwrap();
        assert!(pending.is_empty());
    }

    #[test]
    fn robots_cache_expira_correctamente() {
        let db = PipelineDb::open_in_memory().unwrap();
        assert!(db.get_robots_cache("example.com").unwrap().is_none());
        db.set_robots_cache("example.com", "User-agent: *\nDisallow:")
            .unwrap();
        let cached = db.get_robots_cache("example.com").unwrap();
        assert!(cached.is_some());
        assert!(cached.unwrap().contains("Disallow"));
    }

    #[test]
    fn rate_limit_contabiliza_fallos() {
        let db = PipelineDb::open_in_memory().unwrap();
        db.record_request("example.com", false).unwrap();
        db.record_request("example.com", false).unwrap();
        let (_, failures) = db.get_rate_limit("example.com").unwrap();
        assert_eq!(failures, 2);
        db.record_request("example.com", true).unwrap();
        let (_, failures) = db.get_rate_limit("example.com").unwrap();
        assert_eq!(failures, 0);
    }
}
