// ──────────────────────────────────────────────
// 📧 EMAIL HUNTER — Extracción de correos electrónicos
// Fuentes: búsqueda web + regex + validación de formato
// Sin dependencias externas
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Email encontrado con contexto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundEmail {
    pub email: String,
    pub source_url: String,
    pub context: String,
    pub source: String,
    pub validity: EmailValidity,
}

/// Validez estimada del email
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmailValidity {
    ValidFormat,
    InvalidFormat,
    LikelyDisposable,
    RoleAccount,
}

impl std::fmt::Display for EmailValidity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailValidity::ValidFormat => write!(f, "Formato válido"),
            EmailValidity::InvalidFormat => write!(f, "Formato inválido"),
            EmailValidity::LikelyDisposable => write!(f, "Posiblemente desechable"),
            EmailValidity::RoleAccount => write!(f, "Cuenta de rol"),
        }
    }
}

/// 📧 Buscador de correos electrónicos en fuentes OSINT
pub struct EmailHunter {
    client: reqwest::Client,
    disposable_domains: std::collections::HashSet<String>,
}

impl Default for EmailHunter {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailHunter {
    pub fn new() -> Self {
        // Dominios desechables conocidos
        let disposable: std::collections::HashSet<String> = [
            "mailinator.com",
            "guerrillamail.com",
            "tempmail.com",
            "10minutemail.com",
            "throwaway.com",
            "sharklasers.com",
            "trashmail.com",
            "yopmail.com",
            "burner.com",
            "temp-mail.org",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            client: reqwest::Client::builder()
                .user_agent("NEXUS-OSINT/1.0")
                .timeout(Duration::from_secs(15))
                .build()
                .expect("EmailHunter: Cliente HTTP válido"),
            disposable_domains: disposable,
        }
    }

    /// Busca emails relacionados con una consulta (nombre, dominio, etc.)
    pub async fn search_emails(&self, query: &str) -> anyhow::Result<Vec<FoundEmail>> {
        info!("📧 [EMAIL-HUNTER] Buscando emails para: '{}'", query);

        let mut results = Vec::new();

        // Intentar Brave Search
        if let Ok(emails) = self.search_via_brave(query).await {
            results.extend(emails);
        }

        // Intentar WebSearch (Exa/Tavily) si disponible
        if let Ok(emails) = self.search_via_web(query).await {
            results.extend(emails);
        }

        info!(
            "📧 [EMAIL-HUNTER] {} emails encontrados para '{}'",
            results.len(),
            query
        );
        Ok(results)
    }

    /// Busca emails vía Brave Search API
    async fn search_via_brave(&self, query: &str) -> anyhow::Result<Vec<FoundEmail>> {
        let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            anyhow::bail!("BRAVE_API_KEY no configurada");
        }

        // Query optimizada para encontrar emails
        let search_query = format!("{} \"@\" email OR mail OR contact", query);

        let resp = self
            .client
            .get("https://api.search.brave.com/res/v1/web/search")
            .header("X-Subscription-Token", &api_key)
            .header("Accept", "application/json")
            .query(&[("q", search_query.as_str()), ("count", "10")])
            .send()
            .await?;

        if !resp.status().is_success() {
            anyhow::bail!("Brave Search returned HTTP {}", resp.status());
        }

        let data: serde_json::Value = resp.json().await?;
        let mut emails = Vec::new();

        if let Some(web) = data
            .get("web")
            .and_then(|w| w.get("results"))
            .and_then(|r| r.as_array())
        {
            for r in web {
                let snippet = r.get("description").and_then(|d| d.as_str()).unwrap_or("");
                let url = r.get("url").and_then(|u| u.as_str()).unwrap_or("");
                let title = r.get("title").and_then(|t| t.as_str()).unwrap_or("");

                // Extraer emails del snippet y título
                let text = format!("{} {}", title, snippet);
                for found in self.extract_emails(&text) {
                    let validity = self.validate_email(&found);
                    emails.push(FoundEmail {
                        email: found,
                        source_url: url.to_string(),
                        context: snippet.chars().take(100).collect(),
                        source: "brave-search".to_string(),
                        validity,
                    });
                }
            }
        }

        Ok(emails)
    }

    /// Busca emails vía WebSearch (Exa/Tavily)
    async fn search_via_web(&self, query: &str) -> anyhow::Result<Vec<FoundEmail>> {
        let exa_key = std::env::var("EXA_API_KEY").ok();
        let tavily_key = std::env::var("TAVILY_API_KEY").ok();

        if exa_key.is_none() && tavily_key.is_none() {
            anyhow::bail!("No web search keys available");
        }

        // Usar ShadowCrawlAPI directamente
        let api = crate::infra::shadowcrawl::ShadowCrawlAPI::new(
            "http://127.0.0.1:5000",
            exa_key,
            tavily_key,
            None,
        );

        let search_query = format!("{} email OR mail OR contact", query);
        let results = api.search(&search_query).await?;

        let mut emails = Vec::new();
        for r in &results {
            let text = format!("{} {}", r.title, r.snippet);
            for found in self.extract_emails(&text) {
                let validity = self.validate_email(&found);
                emails.push(FoundEmail {
                    email: found,
                    source_url: r.url.clone(),
                    context: r.snippet.chars().take(100).collect(),
                    source: "web-search".to_string(),
                    validity,
                });
            }
        }

        Ok(emails)
    }

    /// Extrae direcciones de email de un texto usando regex
    pub fn extract_emails(&self, text: &str) -> Vec<String> {
        // Regex para emails: username@domain.tld
        let re = regex::Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
            .expect("Email regex válido");

        let mut emails: Vec<String> = re
            .find_iter(text)
            .map(|m| m.as_str().to_lowercase())
            .collect();

        // Eliminar duplicados
        emails.sort();
        emails.dedup();

        emails
    }

    /// Valida formato y tipo de email
    pub fn validate_email(&self, email: &str) -> EmailValidity {
        // Verificar formato básico
        let email_re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Email validation regex válido");

        if !email_re.is_match(email) {
            return EmailValidity::InvalidFormat;
        }

        let domain = email.split('@').nth(1).unwrap_or("");

        // Verificar si es cuenta de rol
        let local_part = email.split('@').next().unwrap_or("");
        let role_prefixes = [
            "admin",
            "info",
            "support",
            "sales",
            "contact",
            "webmaster",
            "postmaster",
            "hostmaster",
            "noreply",
            "no-reply",
            "mailer-daemon",
        ];

        if role_prefixes.contains(&local_part.to_lowercase().as_str()) {
            return EmailValidity::RoleAccount;
        }

        // Verificar si es dominio desechable
        if self.disposable_domains.contains(domain) {
            return EmailValidity::LikelyDisposable;
        }

        EmailValidity::ValidFormat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_emails_simple() {
        let hunter = EmailHunter::new();
        let emails = hunter.extract_emails("Contact: user@example.com and admin@test.org");
        assert_eq!(emails.len(), 2);
        assert!(emails.contains(&"user@example.com".to_string()));
        assert!(emails.contains(&"admin@test.org".to_string()));
    }

    #[test]
    fn test_extract_emails_dedup() {
        let hunter = EmailHunter::new();
        let emails = hunter.extract_emails("user@example.com and user@example.com again");
        assert_eq!(emails.len(), 1);
    }

    #[test]
    fn test_validate_email() {
        let hunter = EmailHunter::new();
        assert_eq!(
            hunter.validate_email("user@example.com"),
            EmailValidity::ValidFormat
        );
        assert_eq!(
            hunter.validate_email("notanemail"),
            EmailValidity::InvalidFormat
        );
        assert_eq!(
            hunter.validate_email("admin@example.com"),
            EmailValidity::RoleAccount
        );
    }

    #[test]
    fn test_email_hunter_creation() {
        let hunter = EmailHunter::new();
        assert!(!hunter.disposable_domains.is_empty());
    }
}
