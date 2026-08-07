// ==========================================
// COOKIE CLAW - EXTRACTOR DE COOKIES LOCALES
// ==========================================

use rusqlite::Connection;
use std::path::PathBuf;
use tracing::info;

pub struct CookieClaw;

impl Default for CookieClaw {
    fn default() -> Self {
        Self::new()
    }
}

impl CookieClaw {
    pub fn new() -> Self {
        Self
    }

    // Extraer cookies de Brave - devuelve String
    pub fn extract_chrome_cookies(&self) -> Result<String, String> {
        let brave_path =
            PathBuf::from(env!("HOME")).join(".config/BraveSoftware/Brave-Browser/Default/Cookies");

        if !brave_path.exists() {
            return Err("No se encontró Brave".to_string());
        }

        let conn = Connection::open(brave_path).map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT value FROM cookies WHERE host_key LIKE '%google.com%' AND name = '__Secure-1PSID'"
        ).map_err(|e| e.to_string())?;

        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let cookie: String = row.get(0).map_err(|e| e.to_string())?;
            info!("🍪 Cookie __Secure-1PSID extraída de Brave");
            return Ok(cookie);
        }

        Err("No se encontró __Secure-1PSID en Brave".to_string())
    }
}
