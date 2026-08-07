// ============================================================================
// 🎯 SCOPE MAPPER — Aislamiento de Contexto por Proyecto (Regla 1)
// ============================================================================
// Router en Rust que intercepta los mensajes del usuario hacia la IA.
//
// PRINCIPIO: si el usuario menciona un proyecto específico ("página trader"),
// se inyecta en la ventana de contexto ÚNICAMENTE los archivos, variables y
// logs de ESE proyecto. Queda PROHIBIDO cargar contexto global, métricas o
// historial de otros proyectos → ahorro de tokens.
//
// El System Prompt y las capacidades del agente se mantienen intactos; solo
// la DATA del proyecto se aísla por completo.
// ============================================================================

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// Un proyecto registrado en el orquestador.
#[derive(Debug, Clone)]
pub struct ProjectScope {
    pub id: String,
    pub name: String,
    /// Alias por los que se menciona el proyecto (ej. "trader", "página trader").
    pub aliases: Vec<String>,
    /// Archivos propios del proyecto (rutas relativas a la raíz).
    pub files: Vec<String>,
    /// Variables de entorno propias del proyecto.
    pub env_vars: HashMap<String, String>,
    /// Directorio de logs propio.
    pub log_dir: String,
}

impl ProjectScope {
    /// Construye el contexto aislado para la IA (solo datos de este proyecto).
    ///
    /// Genera un bloque de contexto que NO incluye datos de otros proyectos.
    pub fn build_context(&self) -> String {
        let mut ctx = String::new();
        ctx.push_str(&format!("### PROYECTO: {}\n", self.name));
        ctx.push_str(&format!("ID: {}\n", self.id));
        ctx.push_str(&format!("Logs: {}\n", self.log_dir));
        if !self.files.is_empty() {
            ctx.push_str("Archivos:\n");
            for f in &self.files {
                ctx.push_str(&format!("  - {f}\n"));
            }
        }
        if !self.env_vars.is_empty() {
            ctx.push_str("Variables (nombres solamente, no valores secretos):\n");
            for k in self.env_vars.keys() {
                ctx.push_str(&format!("  - {k}\n"));
            }
        }
        ctx
    }
}

/// Mapeador de enfoque: detecta qué proyecto menciona el usuario y aísla
/// su contexto. Persistente en SQLite.
pub struct ScopeMapper {
    conn: Mutex<Connection>,
}

impl ScopeMapper {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let m = Self {
            conn: Mutex::new(conn),
        };
        m.init()?;
        Ok(m)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let m = Self {
            conn: Mutex::new(conn),
        };
        m.init()?;
        Ok(m)
    }

    fn init(&self) -> Result<()> {
        self.conn.lock().unwrap().execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id        TEXT PRIMARY KEY,
                name      TEXT NOT NULL,
                log_dir   TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS project_aliases (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                alias     TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS project_files (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                path      TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    /// Registra un proyecto con sus alias y archivos.
    pub fn register_project(&self, project: &ProjectScope) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO projects (id, name, log_dir) VALUES (?1, ?2, ?3)",
            rusqlite::params![project.id, project.name, project.log_dir],
        )?;
        for alias in &project.aliases {
            conn.execute(
                "INSERT OR IGNORE INTO project_aliases (project_id, alias) VALUES (?1, ?2)",
                rusqlite::params![project.id, alias],
            )?;
        }
        for f in &project.files {
            conn.execute(
                "INSERT OR IGNORE INTO project_files (project_id, path) VALUES (?1, ?2)",
                rusqlite::params![project.id, f],
            )?;
        }
        Ok(())
    }

    /// Detecta qué proyecto (si alguno) menciona el mensaje del usuario.
    ///
    /// Busca los alias de cada proyecto dentro del texto. Devuelve `None` si
    /// no hay proyecto explícito (contexto global mínimo, sin data de proyectos).
    pub fn detect_project(&self, user_message: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let msg = user_message.to_lowercase();
        let mut stmt = conn.prepare(
            "SELECT project_id, alias FROM project_aliases ORDER BY LENGTH(alias) DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (project_id, alias) = row?;
            // Priorizar el alias más específico (más largo) ya detectado por ORDER BY.
            if msg.contains(&alias.to_lowercase()) {
                return Ok(Some(project_id));
            }
        }
        Ok(None)
    }

    /// Resuelve el contexto aislado para el mensaje dado.
    ///
    /// - Si detecta un proyecto → contexto solo de ese proyecto.
    /// - Si no → `None` (el orquestador usa SOLO el system prompt, sin data).
    ///
    /// Esta es la pieza clave del ahorro de tokens: NUNCA se carga data global.
    pub fn resolve_context(&self, user_message: &str) -> Result<Option<String>> {
        let Some(project_id) = self.detect_project(user_message)? else {
            return Ok(None);
        };
        self.build_scope(&project_id).map(|p| p.build_context()).map(Some)
    }

    /// Construye el `ProjectScope` completo desde la DB.
    pub fn build_scope(&self, project_id: &str) -> Result<ProjectScope> {
        let conn = self.conn.lock().unwrap();
        let (name, log_dir): (String, String) = conn
            .query_row(
                "SELECT name, log_dir FROM projects WHERE id = ?1",
                rusqlite::params![project_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| anyhow::anyhow!("proyecto no encontrado: {project_id}"))?;

        let mut aliases = Vec::new();
        {
            let mut stmt =
                conn.prepare("SELECT alias FROM project_aliases WHERE project_id = ?1")?;
            let rows = stmt.query_map(rusqlite::params![project_id], |r| r.get::<_, String>(0))?;
            for r in rows {
                aliases.push(r?);
            }
        }

        let mut files = Vec::new();
        {
            let mut stmt =
                conn.prepare("SELECT path FROM project_files WHERE project_id = ?1")?;
            let rows = stmt.query_map(rusqlite::params![project_id], |r| r.get::<_, String>(0))?;
            for r in rows {
                files.push(r?);
            }
        }

        Ok(ProjectScope {
            id: project_id.to_string(),
            name,
            aliases,
            files,
            env_vars: HashMap::new(), // se cargan desde .env del proyecto en producción
            log_dir,
        })
    }

    /// Lista todos los proyectos (para la UI de alta densidad).
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id FROM projects ORDER BY name")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mapper() -> ScopeMapper {
        let m = ScopeMapper::open_in_memory().unwrap();
        m.register_project(&ProjectScope {
            id: "p1".into(),
            name: "Página Trader".into(),
            aliases: vec!["trader".into(), "página trader".into()],
            files: vec!["projects/trader/main.rs".into(), "projects/trader/config.toml".into()],
            env_vars: HashMap::new(),
            log_dir: "logs/trader".into(),
        })
        .unwrap();
        m.register_project(&ProjectScope {
            id: "p2".into(),
            name: "Bot de Telegram".into(),
            aliases: vec!["telegram".into(), "bot telegram".into()],
            files: vec!["projects/telegram/main.rs".into()],
            env_vars: HashMap::new(),
            log_dir: "logs/telegram".into(),
        })
        .unwrap();
        m
    }

    #[test]
    fn detecta_proyecto_por_alias() {
        let m = sample_mapper();
        let id = m.detect_project("arregla la página trader por favor").unwrap().unwrap();
        assert_eq!(id, "p1");
    }

    #[test]
    fn prioriza_alias_mas_especifico() {
        let m = sample_mapper();
        // "página trader" es más específico que "trader".
        let id = m.detect_project("en la página trader falla el login").unwrap().unwrap();
        assert_eq!(id, "p1");
    }

    #[test]
    fn sin_proyecto_devuelve_none() {
        let m = sample_mapper();
        assert!(m.detect_project("hola, cómo estás?").unwrap().is_none());
    }

    #[test]
    fn contexto_aislado_solo_tiene_datos_del_proyecto() {
        let m = sample_mapper();
        let ctx = m.resolve_context("revisa el bot telegram").unwrap().unwrap();
        assert!(ctx.contains("Bot de Telegram"));
        assert!(ctx.contains("projects/telegram/main.rs"));
        // NO debe contener datos del otro proyecto.
        assert!(!ctx.contains("trader"));
        assert!(!ctx.contains("logs/trader"));
    }

    #[test]
    fn context_sin_proyecto_es_none() {
        let m = sample_mapper();
        assert!(m.resolve_context("hola").unwrap().is_none());
    }
}
