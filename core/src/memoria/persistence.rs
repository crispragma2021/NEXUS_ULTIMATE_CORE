use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

pub struct DatabaseManager {
    pub pool: Option<Arc<SqlitePool>>,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        println!(
            "📡 [DATABASE] Iniciando conexión nativa a SQLite: {}",
            database_url
        );

        let raw_path = database_url.strip_prefix("sqlite:").unwrap_or(database_url);
        let _guard = crate::brain::immune::memory_shield::MemoryShieldGuard::new(raw_path)?;

        let mut sys = sysinfo::System::new_all();
        sys.refresh_cpu_all();
        let total_threads = sys.cpus().len();
        let max_conn = total_threads.max(16) as u32;

        let pool = SqlitePoolOptions::new()
            .max_connections(max_conn) // Máxima ráfaga para hardware local
            .connect(database_url)
            .await
            .map_err(|e| anyhow!("Error al conectar a nexus_intelligence.db: {}", e))?;

        // Asegurar que las tablas existan (Pilar 3: Persistencia Soberana)
        Self::ensure_tables(&pool).await?;

        Ok(Self {
            pool: Some(Arc::new(pool)),
        })
    }

    async fn ensure_tables(pool: &SqlitePool) -> Result<()> {
        sqlx::query("CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, title TEXT, metadata TEXT, created_at TEXT);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS messages (id INTEGER PRIMARY KEY AUTOINCREMENT, session_id TEXT, role TEXT, content TEXT, metadata TEXT, timestamp TEXT);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS evolution_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, data TEXT, timestamp TEXT);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS error_logs (id INTEGER PRIMARY KEY AUTOINCREMENT, message TEXT, component TEXT, context TEXT, timestamp TEXT);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS kali_arsenal (id INTEGER PRIMARY KEY AUTOINCREMENT, tool TEXT, target TEXT, status TEXT, created_at TEXT);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS tool_experience (id INTEGER PRIMARY KEY AUTOINCREMENT, tool_name TEXT, status TEXT, details TEXT, timestamp DATETIME DEFAULT CURRENT_TIMESTAMP);")
            .execute(pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS lessons_sovereign (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT UNIQUE, content TEXT, priority INTEGER, created_at DATETIME DEFAULT CURRENT_TIMESTAMP);")
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn learn_lesson(&self, title: &str, content: &str, priority: i32) -> Result<()> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        if let Some(pool) = &self.pool {
            sqlx::query("INSERT OR REPLACE INTO lessons_sovereign (title, content, priority) VALUES (?, ?, ?)")
                .bind(title)
                .bind(content)
                .bind(priority)
                .execute(&**pool)
                .await?;
        }
        Ok(())
    }

    pub async fn get_lessons(&self) -> Result<Vec<(String, String)>> {
        if let Some(pool) = &self.pool {
            let rows: Vec<(String, String)> = sqlx::query_as(
                "SELECT title, content FROM lessons_sovereign ORDER BY priority DESC",
            )
            .fetch_all(&**pool)
            .await?;
            Ok(rows)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        if let Some(pool) = &self.pool {
            let row: Option<(String,)> = sqlx::query_as("SELECT value FROM config WHERE key = ?")
                .bind(key)
                .fetch_optional(&**pool)
                .await?;
            Ok(row.map(|(v,)| v))
        } else {
            Ok(None)
        }
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        if let Some(pool) = &self.pool {
            sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
                .bind(key)
                .bind(value)
                .execute(&**pool)
                .await?;
        }
        Ok(())
    }

    pub async fn create_session(&self, title: &str, metadata: Value) -> Result<Uuid> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        let id = Uuid::new_v4();
        if let Some(pool) = &self.pool {
            sqlx::query("INSERT INTO sessions (id, title, metadata, created_at) VALUES (?, ?, ?, datetime('now'))")
                .bind(id.to_string())
                .bind(title)
                .bind(metadata)
                .execute(&**pool)
                .await?;
        }
        Ok(id)
    }

    pub async fn store_message(
        &self,
        session_id: Uuid,
        role: &str,
        content: &str,
        metadata: Value,
    ) -> Result<()> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        if let Some(pool) = &self.pool {
            sqlx::query("INSERT INTO messages (session_id, role, content, metadata, timestamp) VALUES (?, ?, ?, ?, datetime('now'))")
                .bind(session_id.to_string())
                .bind(role)
                .bind(content)
                .bind(metadata)
                .execute(&**pool)
                .await?;
        }
        Ok(())
    }

    pub async fn log_evolution(&self, data: Value) -> Result<()> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        if let Some(pool) = &self.pool {
            sqlx::query("INSERT INTO evolution_logs (data, timestamp) VALUES (?, datetime('now'))")
                .bind(data)
                .execute(&**pool)
                .await?;
        }
        Ok(())
    }

    pub async fn record_error(
        &self,
        error_log: &str,
        component: &str,
        context: Value,
    ) -> Result<i32> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        if let Some(pool) = &self.pool {
            let result = sqlx::query("INSERT INTO error_logs (message, component, context, timestamp) VALUES (?, ?, ?, datetime('now'))")
                .bind(error_log)
                .bind(component)
                .bind(context)
                .execute(&**pool)
                .await?;
            Ok(result.last_insert_rowid() as i32)
        } else {
            Ok(0)
        }
    }

    pub async fn find_similar_errors(&self, component: &str) -> Result<Vec<String>> {
        if let Some(pool) = &self.pool {
            let rows: Vec<(String,)> = sqlx::query_as("SELECT message FROM error_logs WHERE component = ? ORDER BY timestamp DESC LIMIT 5")
                .bind(component)
                .fetch_all(&**pool)
                .await?;
            Ok(rows.into_iter().map(|(m,)| m).collect())
        } else {
            Ok(vec![])
        }
    }

    pub async fn insert_kali_tool(&self, tool: &str, target: &str) -> Result<()> {
        let _guard =
            crate::brain::immune::memory_shield::MemoryShieldGuard::new("nexus_intelligence.db")?;
        if let Some(pool) = &self.pool {
            sqlx::query("INSERT INTO kali_arsenal (tool, target, status, created_at) VALUES (?, ?, 'pending', datetime('now'))")
                .bind(tool)
                .bind(target)
                .execute(&**pool)
                .await?;
        }
        Ok(())
    }
}
