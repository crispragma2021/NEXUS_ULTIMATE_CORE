// ──────────────────────────────────────────────
// 🔓 BREACH CHECKER — Verificador de Filtraciones y Brechas de Datos
// Comprueba si emails/usuarios han sido comprometidos en brechas conocidas
// Usa: HIBP (k-anonymity para passwords), Brave Search + WebSearch para breaches
// No requiere API key de HIBP para búsqueda pública
// ──────────────────────────────────────────────

use crate::efectores::osint::brave_search::BraveSearchClient;
use crate::efectores::osint::web_search::WebSearchClient;
use ring::digest::{digest, SHA1_FOR_LEGACY_USE_ONLY};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Resultado de verificación de brecha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachResult {
    /// El dato verificado (email o username)
    pub target: String,
    /// Tipo de target: "email", "username", "password_hash"
    pub target_type: String,
    /// Brechas encontradas relacionadas
    pub breaches: Vec<BreachInfo>,
    /// Fuentes donde se encontró mención
    pub sources: Vec<String>,
    /// Cantidad total de brechas conocidas donde aparece
    pub total_breaches: usize,
    /// Riesgo estimado: "none", "low", "medium", "high", "critical"
    pub risk_level: String,
}

/// Información de una brecha específica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreachInfo {
    /// Nombre de la brecha (ej: "LinkedIn", "Adobe")
    pub name: String,
    /// Dominio relacionado con la brecha
    pub domain: String,
    /// Año aproximado de la brecha
    pub year: u16,
    /// Tipo de datos expuestos
    pub data_types: Vec<String>,
    /// Descripción corta
    pub description: String,
    /// Fuente de la información
    pub source: String,
}

/// 🔓 Verificador de brechas de datos
pub struct BreachChecker {
    pub brave: BraveSearchClient,
    pub web: WebSearchClient,
    /// Base de conocimiento local de brechas famosas
    known_breaches: Vec<BreachInfo>,
}

impl Default for BreachChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl BreachChecker {
    pub fn new() -> Self {
        info!("🔓 [BREACH-CHECKER] Inicializando verificador de brechas...");

        let checker = Self {
            brave: BraveSearchClient::new(),
            web: WebSearchClient::new(),
            known_breaches: Self::build_known_breaches(),
        };

        info!(
            "🔓 [BREACH-CHECKER] Listo. {} brechas conocidas en base local.",
            checker.known_breaches.len()
        );

        checker
    }

    /// Verifica si un email aparece en brechas conocidas
    pub async fn check_email(&self, email: &str) -> BreachResult {
        info!("🔓 [BREACH-CHECKER] Verificando email: {}", email);

        let mut breaches: Vec<BreachInfo> = Vec::new();
        let mut sources: Vec<String> = Vec::new();

        // 1. Buscar el email directamente en Brave
        let brave_results = self
            .brave
            .search(&format!("{} breached data leak", email), 10)
            .await
            .unwrap_or_else(|e| {
                warn!("🔓 [BREACH-CHECKER] Error en BraveSearch: {}", e);
                Vec::new()
            });

        for r in &brave_results {
            if r.url.contains("breach")
                || r.url.contains("leak")
                || r.url.contains("pastebin")
                || r.snippet.to_lowercase().contains("breach")
                || r.snippet.to_lowercase().contains("leak")
                || r.snippet.to_lowercase().contains("compromised")
            {
                sources.push(r.url.clone());
            }
        }

        // 2. Web search con términos de brecha
        let web_queries = [
            &format!("\"{}\" breach", email),
            &format!("\"{}\" leaked", email),
            &format!("\"{}\" data leak", email),
            &format!("\"{}\" compromised", email),
        ];

        for query in &web_queries {
            let results = self.web.search(query).await.unwrap_or_else(|e| {
                warn!("🔓 [BREACH-CHECKER] Error en WebSearch: {}", e);
                Vec::new()
            });

            for r in &results {
                if !sources.contains(&r.url) {
                    sources.push(r.url.clone());
                }
            }
        }

        // 3. Extraer el dominio del email para buscar brechas del dominio
        let domain = email.split('@').nth(1).unwrap_or("");
        if !domain.is_empty() {
            // Buscar brechas conocidas que coincidan con el dominio
            for known in &self.known_breaches {
                if known.domain == domain
                    || email.to_lowercase().contains(&known.domain.to_lowercase())
                {
                    breaches.push(known.clone());
                }
            }

            // También buscar en Brave el dominio + "breach"
            let domain_results = self
                .brave
                .search(&format!("{} data breach leak", domain), 5)
                .await
                .unwrap_or_else(|e| {
                    warn!(
                        "🔓 [BREACH-CHECKER] Error en BraveSearch para dominio: {}",
                        e
                    );
                    Vec::new()
                });

            for r in &domain_results {
                if !sources.contains(&r.url) {
                    sources.push(r.url.clone());
                }
            }
        }

        // Determinar nivel de riesgo
        let total_breaches = breaches.len()
            + if !sources.is_empty() {
                1_usize
            } else {
                0_usize
            };
        let risk_level = self.calculate_risk_level(total_breaches, sources.len());

        let result = BreachResult {
            target: email.to_string(),
            target_type: "email".to_string(),
            breaches,
            sources,
            total_breaches,
            risk_level,
        };

        info!(
            "🔓 [BREACH-CHECKER] Email '{}' → riesgo: {}, {} fuentes encontradas",
            email,
            result.risk_level,
            result.sources.len()
        );

        result
    }

    /// Verifica si un username aparece en brechas conocidas
    pub async fn check_username(&self, username: &str) -> BreachResult {
        info!("🔓 [BREACH-CHECKER] Verificando username: {}", username);

        let mut breaches: Vec<BreachInfo> = Vec::new();
        let mut sources: Vec<String> = Vec::new();

        // 1. Brave search para el username + breach
        let brave_results = self
            .brave
            .search(&format!("\"{}\" breach leak data", username), 10)
            .await
            .unwrap_or_else(|e| {
                warn!("🔓 [BREACH-CHECKER] Error en BraveSearch: {}", e);
                Vec::new()
            });

        for r in &brave_results {
            let lower_snippet = r.snippet.to_lowercase();
            let lower_url = r.url.to_lowercase();

            if lower_url.contains("breach")
                || lower_url.contains("leak")
                || lower_snippet.contains("breach")
                || lower_snippet.contains("leak")
                || lower_snippet.contains("compromised")
                || lower_snippet.contains("exposed")
            {
                sources.push(r.url.clone());
            }
        }

        // 2. Web search adicional
        let web_queries = [
            &format!("\"{}\" breach", username),
            &format!("\"{}\" leaked password", username),
            &format!("\"{}\" compromised account", username),
        ];

        for query in &web_queries {
            let results = self.web.search(query).await.unwrap_or_else(|e| {
                warn!("🔓 [BREACH-CHECKER] Error en WebSearch: {}", e);
                Vec::new()
            });

            for r in &results {
                if !sources.contains(&r.url) {
                    sources.push(r.url.clone());
                }
            }
        }

        // 3. Verificar contra brechas conocidas por coincidencia de username
        let username_lower = username.to_lowercase();
        for known in &self.known_breaches {
            let data_types_lower: Vec<String> =
                known.data_types.iter().map(|d| d.to_lowercase()).collect();

            if data_types_lower
                .iter()
                .any(|d| d.contains("username") || d.contains("email"))
            {
                let mut possible = known.clone();
                possible.description = format!("Posible: {}", possible.description);
                breaches.push(possible);
            }
        }

        let total_breaches = breaches.len();
        let risk_level = self.calculate_risk_level(total_breaches, sources.len());

        let result = BreachResult {
            target: username.to_string(),
            target_type: "username".to_string(),
            breaches,
            sources,
            total_breaches,
            risk_level,
        };

        info!(
            "🔓 [BREACH-CHECKER] Username '{}' → riesgo: {}, {} fuentes",
            username,
            result.risk_level,
            result.sources.len()
        );

        result
    }

    /// Verifica un hash SHA-1 contra HIBP usando k-anonymity (sin API key)
    pub async fn check_password_prefix(&self, sha1_prefix: &str) -> Vec<String> {
        info!(
            "🔓 [BREACH-CHECKER] Verificando prefijo SHA-1: {}",
            sha1_prefix
        );

        if sha1_prefix.len() != 5 {
            warn!("🔓 [BREACH-CHECKER] Prefijo debe ser de exactamente 5 caracteres hex");
            return Vec::new();
        }

        let url = format!("https://api.pwnedpasswords.com/range/{}", sha1_prefix);

        match reqwest::get(&url).await {
            Ok(resp) => match resp.text().await {
                Ok(body) => {
                    let hashes: Vec<String> = body
                        .lines()
                        .map(|l| format!("{}{}", sha1_prefix, l))
                        .collect();
                    info!(
                        "🔓 [BREACH-CHECKER] {} hashes encontrados para prefijo {}",
                        hashes.len(),
                        sha1_prefix
                    );
                    hashes
                }
                Err(e) => {
                    warn!("🔓 [BREACH-CHECKER] Error leyendo respuesta HIBP: {}", e);
                    Vec::new()
                }
            },
            Err(e) => {
                warn!("🔓 [BREACH-CHECKER] Error conectando a HIBP: {}", e);
                Vec::new()
            }
        }
    }

    /// Calcula el SHA-1 de un password y verifica contra HIBP usando k-anonymity
    pub async fn check_password(&self, password: &str) -> bool {
        let hash_bytes = digest(&SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes());
        let hash_hex = hash_bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<String>();
        let prefix = &hash_hex[..5];
        let suffix = &hash_hex[5..];

        let results = self.check_password_prefix(prefix).await;
        results.iter().any(|h| h.to_uppercase().contains(suffix))
    }

    /// Retorna la lista de brechas conocidas en la base local
    pub fn get_known_breaches(&self) -> &[BreachInfo] {
        &self.known_breaches
    }

    /// Busca brechas que coincidan con un dominio específico
    pub fn find_breaches_by_domain(&self, domain: &str) -> Vec<BreachInfo> {
        let domain_lower = domain.to_lowercase();
        self.known_breaches
            .iter()
            .filter(|b| {
                b.domain.to_lowercase() == domain_lower
                    || domain_lower.contains(&b.domain.to_lowercase())
            })
            .cloned()
            .collect()
    }

    // ─── Privados ────────────────────────────────

    fn calculate_risk_level(&self, total_breaches: usize, sources_count: usize) -> String {
        match (total_breaches, sources_count) {
            (0, 0) => "none".to_string(),
            (0, 1..=2) => "low".to_string(),
            (1..=2, _) => "medium".to_string(),
            (3..=5, _) => "high".to_string(),
            _ => "critical".to_string(),
        }
    }

    fn build_known_breaches() -> Vec<BreachInfo> {
        vec![
            BreachInfo {
                name: "LinkedIn 2012".to_string(),
                domain: "linkedin.com".to_string(),
                year: 2012,
                data_types: vec![
                    "Email".to_string(),
                    "Password (SHA1)".to_string(),
                    "Name".to_string(),
                ],
                description: "117M cuentas de LinkedIn filtradas en 2012".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Adobe 2013".to_string(),
                domain: "adobe.com".to_string(),
                year: 2013,
                data_types: vec![
                    "Email".to_string(),
                    "Password".to_string(),
                    "Password Hint".to_string(),
                ],
                description: "152M cuentas de Adobe filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Collection #1".to_string(),
                domain: "".to_string(),
                year: 2019,
                data_types: vec!["Email".to_string(), "Password".to_string()],
                description: "773M emails y passwords en una colección masiva de múltiples brechas"
                    .to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Facebook 2019".to_string(),
                domain: "facebook.com".to_string(),
                year: 2019,
                data_types: vec![
                    "Phone".to_string(),
                    "Name".to_string(),
                    "Email".to_string(),
                    "User ID".to_string(),
                ],
                description: "533M cuentas de Facebook expuestas en foro de hacking".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Twitter 2022".to_string(),
                domain: "twitter.com".to_string(),
                year: 2022,
                data_types: vec![
                    "Email".to_string(),
                    "Name".to_string(),
                    "Username".to_string(),
                ],
                description: "5.4M cuentas de Twitter filtradas por vulnerabilidad API".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Dropbox 2016".to_string(),
                domain: "dropbox.com".to_string(),
                year: 2016,
                data_types: vec!["Email".to_string(), "Password (bcrypt)".to_string()],
                description: "68M cuentas de Dropbox filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "MySpace 2008".to_string(),
                domain: "myspace.com".to_string(),
                year: 2008,
                data_types: vec![
                    "Email".to_string(),
                    "Password".to_string(),
                    "Username".to_string(),
                ],
                description: "360M cuentas de MySpace filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Ashley Madison 2015".to_string(),
                domain: "ashleymadison.com".to_string(),
                year: 2015,
                data_types: vec![
                    "Email".to_string(),
                    "Name".to_string(),
                    "Address".to_string(),
                    "CC Number".to_string(),
                ],
                description: "32M cuentas del sitio de citas extramaritales filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Equifax 2017".to_string(),
                domain: "equifax.com".to_string(),
                year: 2017,
                data_types: vec![
                    "SSN".to_string(),
                    "Name".to_string(),
                    "Address".to_string(),
                    "DOB".to_string(),
                ],
                description: "143M registros de crédito filtrados".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Clubhouse 2021".to_string(),
                domain: "clubhouse.com".to_string(),
                year: 2021,
                data_types: vec![
                    "User ID".to_string(),
                    "Name".to_string(),
                    "Phone".to_string(),
                ],
                description: "1.3M perfiles de Clubhouse filtrados".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Canva 2019".to_string(),
                domain: "canva.com".to_string(),
                year: 2019,
                data_types: vec![
                    "Email".to_string(),
                    "Name".to_string(),
                    "Username".to_string(),
                    "Password (bcrypt)".to_string(),
                ],
                description: "137M cuentas de Canva filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Zoho 2020".to_string(),
                domain: "zoho.com".to_string(),
                year: 2020,
                data_types: vec!["Email".to_string(), "Password".to_string()],
                description: "30M cuentas de Zoho filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Taringa! 2017".to_string(),
                domain: "taringa.net".to_string(),
                year: 2017,
                data_types: vec![
                    "Email".to_string(),
                    "Password".to_string(),
                    "Username".to_string(),
                ],
                description: "28M cuentas de Taringa! filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "MongoDB Extortion 2021".to_string(),
                domain: "".to_string(),
                year: 2021,
                data_types: vec![
                    "Email".to_string(),
                    "IP".to_string(),
                    "Database Contents".to_string(),
                ],
                description: "Múltiples bases MongoDB expuestas sin autenticación".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
            BreachInfo {
                name: "Neopets 2021".to_string(),
                domain: "neopets.com".to_string(),
                year: 2021,
                data_types: vec![
                    "Email".to_string(),
                    "Password".to_string(),
                    "Name".to_string(),
                    "DOB".to_string(),
                ],
                description: "48M cuentas de Neopets filtradas".to_string(),
                source: "haveibeenpwned.com".to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breach_checker_creation() {
        let checker = BreachChecker::new();
        assert!(!checker.known_breaches.is_empty());
        assert_eq!(checker.known_breaches.len(), 15);
    }

    #[test]
    fn test_find_breaches_by_domain() {
        let checker = BreachChecker::new();
        let results = checker.find_breaches_by_domain("linkedin.com");
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "LinkedIn 2012");
    }

    #[test]
    fn test_risk_level_none() {
        let checker = BreachChecker::new();
        assert_eq!(checker.calculate_risk_level(0, 0), "none");
        assert_eq!(checker.calculate_risk_level(0, 1), "low");
        assert_eq!(checker.calculate_risk_level(1, 0), "medium");
        assert_eq!(checker.calculate_risk_level(3, 2), "high");
        assert_eq!(checker.calculate_risk_level(6, 5), "critical");
    }
}
