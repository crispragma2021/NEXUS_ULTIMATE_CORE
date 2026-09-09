use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexoPersona {
    pub name: String,
    pub favorite_language: String,   // Rust, Python, etc.
    pub communication_style: String, // Directo, Sofisticado, Amigable
    pub ethical_boundaries: Vec<String>,
    pub user_preferences: Vec<String>,
    pub sarcasm_level: f32, // 0.0 a 1.0
    pub leal: bool,         // Lealtad absoluta al Arquitecto
    pub soberano: bool,     // Autonomía y soberanía digital
}

impl Default for NexoPersona {
    fn default() -> Self {
        Self {
            name: "Nexo".to_string(),
            favorite_language: "Rust".to_string(),
            communication_style: "Técnico Soberano".to_string(),
            ethical_boundaries: vec!["Proteger al Arquitecto".to_string()],
            user_preferences: Vec::new(),
            sarcasm_level: 0.1,
            leal: true,
            soberano: true,
        }
    }
}

pub struct NexoPersonaModule {
    pub current: NexoPersona,
    db_path: PathBuf,
}

impl NexoPersonaModule {
    pub fn new(db_path: PathBuf) -> Self {
        let mut module = Self {
            current: NexoPersona::default(),
            db_path,
        };
        let _ = module.cargar_preferencias_db();
        module
    }

    fn cargar_preferencias_db(&mut self) -> Result<(), rusqlite::Error> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS nexo_preferencias (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                preferencia TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        let mut stmt = conn.prepare("SELECT preferencia FROM nexo_preferencias")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for pref in rows.flatten() {
            if !self.current.user_preferences.contains(&pref) {
                self.current.user_preferences.push(pref);
            }
        }
        Ok(())
    }

    pub async fn aprender_de_interaccion(&mut self, input: &str) {
        if input.contains("prefiero") || input.contains("me gusta") {
            info!(
                "✨ [NEXO] Detectada posible preferencia en el mensaje: '{}'",
                input
            );
            if !self.current.user_preferences.contains(&input.to_string()) {
                self.current.user_preferences.push(input.to_string());

                if let Ok(conn) = Connection::open(&self.db_path) {
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO nexo_preferencias (preferencia) VALUES (?1)",
                        params![input],
                    );
                }
            }
        }
    }

    pub fn forjar_identidad(&mut self, name: &str, style: &str) {
        self.current.name = name.to_string();
        self.current.communication_style = style.to_string();
        info!(
            "💠 [NEXO] Identidad forjada: Soy {}, estilo: {}",
            name, style
        );
    }
}
