//! Observatorio Personal en Tiempo Real (E1).
//!
//! Demonio que monitorea fuentes definidas por el usuario (precios, noticias,
//! ofertas) y notifica cambios vía canales configurables.
//!
//! ```
//! [ SQLite: observatory_sources ]
//!   └── url, selector, campo, frecuencia, último_valor
//!         │ (tokio::interval por fuente)
//!         ▼
//! [ Fetcher + Cleaner ]  (rápido, sin LLM)
//!         │
//!         ▼ (diferencia detectada)
//! [ Notifier ]
//!   ├── LogNotifier (siempre)
//!   ├── TelegramNotifier (teloxide, opcional)
//!   └── SQLite: registro de cambios
//! ```

use crate::scraping::pipeline::cleaner;
use crate::scraping::pipeline::fetcher::Fetcher;
use crate::scraping::pipeline::schemas::{Strategy, TaskSchema};
use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use teloxide::prelude::*; // Trae el trait `Requester` (send_message)

/// Fuente monitoreada.
#[derive(Debug, Clone)]
pub struct ObservatorySource {
    pub id: i64,
    pub url: String,
    /// Selector CSS para extracción focalizada (opcional).
    pub selector: Option<String>,
    /// Nombre del campo (para el mensaje de notificación).
    pub field_name: String,
    /// Frecuencia de check en minutos.
    pub check_interval_min: u64,
    /// Último valor observado.
    pub last_value: Option<String>,
    /// Última vez revisado (ISO 8601).
    pub last_checked: Option<String>,
}

/// Notificador abstracto.
pub trait Notifier: Send + Sync {
    fn notify(&self, message: &str);
}

/// Notificador a log (siempre activo).
pub struct LogNotifier;

impl Notifier for LogNotifier {
    fn notify(&self, message: &str) {
        tracing::info!("🔭 [OBSERVATORIO] {message}");
    }
}

/// Notificador Telegram (teloxide).
pub struct TelegramNotifier {
    bot: Option<teloxide::Bot>,
    chat_id: Option<i64>,
}

impl TelegramNotifier {
    pub fn new(token: Option<String>, chat_id: Option<i64>) -> Self {
        let bot = token.map(teloxide::Bot::new);
        Self { bot, chat_id }
    }
}

impl Notifier for TelegramNotifier {
    fn notify(&self, message: &str) {
        let Some(bot) = &self.bot else { return };
        let Some(chat_id) = self.chat_id else { return };
        let text = message.to_string();
        let bot = bot.clone();
        tokio::spawn(async move {
            let _ = bot
                .send_message(teloxide::types::ChatId(chat_id), text)
                .await;
        });
    }
}

/// Observatorio: gestión de fuentes y ciclo de monitorización.
pub struct Observatory {
    conn: Mutex<Connection>,
    fetcher: Arc<Fetcher>,
    notifiers: Vec<Box<dyn Notifier>>,
}

impl Observatory {
    pub fn open(path: &Path, fetcher: Arc<Fetcher>) -> Result<Self> {
        let conn = Connection::open(path)?;
        let obs = Self {
            conn: Mutex::new(conn),
            fetcher,
            notifiers: vec![Box::new(LogNotifier)],
        };
        obs.init()?;
        Ok(obs)
    }

    fn init(&self) -> Result<()> {
        self.conn.lock().unwrap().execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS observatory_sources (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                url               TEXT NOT NULL,
                selector          TEXT,
                field_name        TEXT NOT NULL DEFAULT 'valor',
                check_interval_min INTEGER NOT NULL DEFAULT 5,
                last_value        TEXT,
                last_checked      TEXT,
                created_at        TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS observatory_changes (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id   INTEGER NOT NULL REFERENCES observatory_sources(id) ON DELETE CASCADE,
                old_value   TEXT,
                new_value   TEXT,
                changed_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        )?;
        Ok(())
    }

    /// Añade un notificador adicional (Telegram, etc.).
    pub fn add_notifier(&mut self, n: Box<dyn Notifier>) {
        self.notifiers.push(n);
    }

    /// Registra una fuente a monitorear.
    pub fn add_source(
        &self,
        url: &str,
        selector: Option<&str>,
        field_name: &str,
        interval_min: u64,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO observatory_sources (url, selector, field_name, check_interval_min)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![url, selector, field_name, interval_min],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Lista todas las fuentes.
    pub fn list_sources(&self) -> Result<Vec<ObservatorySource>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, selector, field_name, check_interval_min, last_value, last_checked
             FROM observatory_sources ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, Option<String>>(5)?,
                r.get::<_, Option<String>>(6)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, url, selector, field_name, interval, last_value, last_checked) = row?;
            out.push(ObservatorySource {
                id,
                url,
                selector,
                field_name,
                check_interval_min: interval as u64,
                last_value,
                last_checked,
            });
        }
        Ok(out)
    }

    /// Comprueba una fuente: scrapea, extrae valor, compara y notifica si cambió.
    pub async fn check_source(&self, source: &ObservatorySource) -> Result<bool> {
        let task = TaskSchema {
            task_id: format!("obs-{}", source.id),
            url: source.url.clone(),
            strategy: Strategy::Http,
            selectors: None,
            output_schema: None,
            timeout_seconds: 30,
            max_retries: 1,
            respect_robots_txt: true,
            rate_limit_delay_ms: 2000,
            user_agent: "NexusObservatory/1.0".into(),
            metadata: None,
        };

        // Fetch + clean (sin LLM, rápido y gratuito).
        let out = self.fetcher.fetch(&task).await?;
        let markdown = cleaner::clean(&out.html, &[]);

        // Extraer el valor: si hay selector, usar texto focalizado; si no, el
        // primer párrafo o el texto completo truncado.
        let new_value = self.extract_value(&out.html, &markdown, source.selector.as_deref());

        // Comparar con el último valor.
        let changed = match &source.last_value {
            Some(prev) => prev != &new_value,
            None => true, // primera observación → registrar
        };

        if changed {
            self.notify_change(source, source.last_value.as_deref(), &new_value);
            self.update_source_value(source.id, &new_value)?;
        }

        Ok(changed)
    }

    /// Extrae un valor de la página (selector CSS focalizado o texto crudo).
    fn extract_value(&self, html: &str, markdown: &str, selector: Option<&str>) -> String {
        if let Some(sel) = selector {
            if let Ok(sel_parsed) = scraper::Selector::parse(sel) {
                let document = scraper::Html::parse_document(html);
                if let Some(el) = document.select(&sel_parsed).next() {
                    let text = el.text().collect::<String>();
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        return trimmed.to_string();
                    }
                }
            }
        }
        // Fallback: primer párrafo del markdown o texto completo truncado.
        for line in markdown.lines() {
            let t = line.trim();
            if !t.is_empty() && t.len() > 10 {
                return t.chars().take(200).collect();
            }
        }
        markdown.chars().take(200).collect()
    }

    fn notify_change(&self, source: &ObservatorySource, old: Option<&str>, new: &str) {
        let old = old.unwrap_or("(nuevo)");
        let msg = format!(
            "{} cambió en {}: \"{}\" → \"{}\"",
            source.field_name, source.url, old, new
        );
        for n in &self.notifiers {
            n.notify(&msg);
        }
    }

    fn update_source_value(&self, source_id: i64, new_value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // Registrar cambio.
        let _ = conn.execute(
            "UPDATE observatory_sources SET last_value = ?2, last_checked = datetime('now')
             WHERE id = ?1",
            rusqlite::params![source_id, new_value],
        )?;
        // Insertar registro en historial (con valor previo).
        let _ = conn.execute(
            "INSERT INTO observatory_changes (source_id, new_value)
             SELECT id, ?2 FROM observatory_sources WHERE id = ?1",
            rusqlite::params![source_id, new_value],
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrega_y_lista_fuentes() {
        let fetcher = Arc::new(Fetcher::new(None).unwrap());
        let obs = Observatory::open_in_memory_for_test(fetcher);
        obs.add_source("https://example.com/precio", Some(".price"), "precio", 5)
            .unwrap();
        let sources = obs.list_sources().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].field_name, "precio");
    }

    #[test]
    fn extrae_valor_por_selector() {
        let fetcher = Arc::new(Fetcher::new(None).unwrap());
        let obs = Observatory::open_in_memory_for_test(fetcher);
        let html = r#"<html><body><span class="price">$1,299.99</span></body></html>"#;
        let md = cleaner::clean(html, &[]);
        let v = obs.extract_value(html, &md, Some(".price"));
        assert_eq!(v, "$1,299.99");
    }

    #[test]
    fn extrae_valor_fallback_sin_selector() {
        let fetcher = Arc::new(Fetcher::new(None).unwrap());
        let obs = Observatory::open_in_memory_for_test(fetcher);
        let html = r#"<html><body><p>Este es el contenido principal del artículo largo.</p></body></html>"#;
        let md = cleaner::clean(html, &[]);
        let v = obs.extract_value(html, &md, None);
        assert!(v.contains("contenido principal"));
    }
}

impl Observatory {
    /// Helper para tests: abre en memoria.
    pub fn open_in_memory_for_test(fetcher: Arc<Fetcher>) -> Self {
        let conn = Connection::open_in_memory().unwrap();
        let obs = Self {
            conn: Mutex::new(conn),
            fetcher,
            notifiers: vec![Box::new(LogNotifier)],
        };
        obs.init().unwrap();
        obs
    }
}
