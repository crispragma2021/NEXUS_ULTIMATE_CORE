// ──────────────────────────────────────────────
// 📱 TELEGRAM SCRAPER — OSINT en Telegram
// Busca usuarios, grupos y canales públicos de Telegram
// Usa scraping de t.me + consulta a API pública
// No requiere el bot token para búsqueda pública
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Resultado de búsqueda de usuario en Telegram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub username: String,
    pub url: String,
    pub exists: bool,
    pub nombre_mostrado: Option<String>,
    pub descripcion: Option<String>,
    pub miembros: Option<u32>,
    pub tipo: String, // "user", "group", "channel", "bot"
}

/// 📱 Scraper OSINT de Telegram
pub struct TelegramScraper {
    client: reqwest::Client,
}

impl Default for TelegramScraper {
    fn default() -> Self {
        Self::new()
    }
}

impl TelegramScraper {
    pub fn new() -> Self {
        info!("📱 [TELEGRAM] Inicializando scraper de Telegram...");

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// Busca un usuario/canal/grupo en Telegram via t.me
    pub async fn buscar_usuario(&self, username: &str) -> TelegramUser {
        info!("📱 [TELEGRAM] Buscando usuario: {}", username);

        let clean = username.trim_start_matches('@');
        let url = format!("https://t.me/{}", clean);

        match self.client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();

                if status.as_u16() == 200 {
                    // La página existe - intentar extraer metadatos
                    let html = resp.text().await.unwrap_or_default();

                    let nombre_mostrado = Self::extract_meta(&html, "og:title")
                        .or_else(|| Self::extract_html_title(&html));

                    let descripcion = Self::extract_meta(&html, "og:description");

                    // Determinar tipo
                    let tipo = if html.contains("tg://resolve?domain=") {
                        if html.contains("group") || html.contains("supergroup") {
                            "group".to_string()
                        } else if html.contains("channel") {
                            "channel".to_string()
                        } else if username.to_lowercase().ends_with("bot") {
                            "bot".to_string()
                        } else {
                            "user".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    };

                    // Estimar miembros desde la descripción
                    let miembros = descripcion
                        .as_ref()
                        .and_then(|d| Self::extract_member_count(d));

                    info!(
                        "📱 [TELEGRAM] Usuario '{}' encontrado ({}): {}",
                        username,
                        tipo,
                        nombre_mostrado.as_deref().unwrap_or("sin nombre")
                    );

                    TelegramUser {
                        username: clean.to_string(),
                        url,
                        exists: true,
                        nombre_mostrado,
                        descripcion,
                        miembros,
                        tipo,
                    }
                } else if status.as_u16() == 404 {
                    info!("📱 [TELEGRAM] Usuario '{}' no encontrado (404)", username);
                    TelegramUser {
                        username: clean.to_string(),
                        url,
                        exists: false,
                        nombre_mostrado: None,
                        descripcion: None,
                        miembros: None,
                        tipo: "not_found".to_string(),
                    }
                } else {
                    warn!(
                        "📱 [TELEGRAM] Status inesperado {} para '{}'",
                        status, username
                    );
                    TelegramUser {
                        username: clean.to_string(),
                        url,
                        exists: status.is_success(),
                        nombre_mostrado: None,
                        descripcion: None,
                        miembros: None,
                        tipo: "unknown".to_string(),
                    }
                }
            }
            Err(e) => {
                warn!("📱 [TELEGRAM] Error consultando '{}': {}", username, e);
                TelegramUser {
                    username: clean.to_string(),
                    url,
                    exists: false,
                    nombre_mostrado: None,
                    descripcion: None,
                    miembros: None,
                    tipo: "error".to_string(),
                }
            }
        }
    }

    /// Busca múltiples usuarios en paralelo
    pub async fn buscar_usuarios(&self, usernames: &[&str]) -> Vec<TelegramUser> {
        let mut handles = Vec::new();

        for username in usernames {
            let username = username.to_string();
            let handle = tokio::spawn(async move {
                // Creamos un scraper temporal para cada búsqueda
                let scraper = TelegramScraper::new();
                scraper.buscar_usuario(&username).await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(user) = handle.await {
                results.push(user);
            }
        }

        results
    }

    /// Busca un grupo/canal por query en TGStat (si está disponible)
    pub async fn buscar_grupo_tgstat(&self, query: &str) -> Vec<TelegramUser> {
        info!("📱 [TELEGRAM] Buscando en TGStat: {}", query);

        let url = format!("https://tgstat.ru/en/search?q={}", query);

        match self.client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let _html = resp.text().await.unwrap_or_default();
                    // TGStat requiere parsing más complejo - placeholder
                    warn!("📱 [TELEGRAM] TGStat parsing no implementado completamente");
                }
                Vec::new()
            }
            Err(e) => {
                warn!("📱 [TELEGRAM] Error consultando TGStat: {}", e);
                Vec::new()
            }
        }
    }

    /// Busca un username en múltiples fuentes de Telegram
    pub async fn buscar_completo(&self, username: &str) -> Vec<TelegramUser> {
        let mut results = Vec::new();

        // 1. Buscar en t.me directamente
        let user = self.buscar_usuario(username).await;
        results.push(user);

        // 2. Buscar variantes comunes
        let variants = [
            format!("{}_chat", username),
            format!("{}.chat", username),
            format!("{}group", username),
            format!("{}_group", username),
        ];

        for variant in &variants {
            let user = self.buscar_usuario(variant).await;
            if user.exists {
                results.push(user);
            }
        }

        results
    }

    // ─── Privados ────────────────────────────────

    fn extract_meta(html: &str, property: &str) -> Option<String> {
        let patterns = [
            format!(r#"property="{}" content=""#, property),
            format!(r#"property='{}' content='"#, property),
            format!(r#"name="{}" content=""#, property),
            format!(r#"name='{}' content='"#, property),
        ];

        for pattern in &patterns {
            if let Some(start) = html.find(pattern.as_str()) {
                let start = start + pattern.len();
                if let Some(end) = html[start..].find('"').or_else(|| html[start..].find('\'')) {
                    let value = &html[start..start + end];
                    if !value.is_empty() && value != "-" {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_html_title(html: &str) -> Option<String> {
        let patterns = ["<title>", "<title>"];
        for pattern in &patterns {
            if let Some(start) = html.find(pattern) {
                let start = start + pattern.len();
                if let Some(end) = html[start..].find("</title>") {
                    let title = &html[start..start + end];
                    let clean = title.trim();
                    if !clean.is_empty() {
                        return Some(clean.to_string());
                    }
                }
            }
        }
        None
    }

    fn extract_member_count(desc: &str) -> Option<u32> {
        // Pattern: "X members" or "X subscribers"
        let re = regex::Regex::new(r"(\d[\d\s,.]*)\s*(members|subscribers|miembros)").ok()?;
        if let Some(cap) = re.captures(desc) {
            let num_str = cap.get(1)?.as_str().replace(&[' ', ',', '.'][..], "");
            num_str.parse::<u32>().ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buscar_usuario_telegram() {
        let scraper = TelegramScraper::new();
        // Telegram oficial debería existir
        let result = scraper.buscar_usuario("telegram").await;
        // Debería al menos no crashear
    }

    #[test]
    fn test_extract_member_count() {
        assert_eq!(
            TelegramScraper::extract_member_count("1000 members"),
            Some(1000)
        );
        assert_eq!(
            TelegramScraper::extract_member_count("1,500 subscribers"),
            Some(1500)
        );
        assert_eq!(TelegramScraper::extract_member_count("No info"), None);
    }
}
