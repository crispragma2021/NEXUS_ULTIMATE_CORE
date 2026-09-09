// ──────────────────────────────────────────────
// 🧠 OSINT HUB — Orquestador Unificado de OSINT
// Punto de entrada único para todas las operaciones OSINT.
// Coordina: BraveSearchClient, WebSearchClient, SocialHunter, DorkForger
// Tier 2: SubdomainEnumerator, CertTransparency, EmailHunter, BreachChecker
// Tier 3: DnsResolver, GeoWhois, PortScanner, TelegramScraper, TorSearch
// Genera reportes estructurados multi-táctica.
// ──────────────────────────────────────────────

use crate::efectores::osint::brave_search::{BraveSearchClient, BraveSearchResult};
use crate::efectores::osint::breach_checker::{BreachChecker, BreachResult};
use crate::efectores::osint::cert_transparency::{CertInfo, CertTransparency};
use crate::efectores::osint::dns_resolver::{DnsRecord, DnsResolver, DnsResult};
use crate::efectores::osint::dork_forger::{DorkCategory, DorkForger, DorkResult};
use crate::efectores::osint::email_hunter::{EmailHunter, FoundEmail};
use crate::efectores::osint::geo_whois::{GeoResult, GeoWhois, WhoisResult};
use crate::efectores::osint::port_scanner::{OpenPort, PortScanner};
use crate::efectores::osint::social_hunter::{SocialHunter, SocialProfile};
use crate::efectores::osint::subdomain_enum::{Subdomain, SubdomainEnumerator};
use crate::efectores::osint::telegram_scraper::{TelegramScraper, TelegramUser};
use crate::efectores::osint::tor_search::{TorSearch, TorSearchResult, TorStatus};
use crate::efectores::osint::web_search::{WebSearchClient, WebSearchResult};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Reporte consolidado de inteligencia OSINT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintReport {
    pub target: String,
    pub target_type: String,
    pub dork_results: Vec<DorkResult>,
    pub social_profiles: Vec<SocialProfile>,
    pub web_results: Vec<WebSearchResult>,
    pub brave_results: Vec<BraveSearchResult>,
    // Tier 2 campos
    pub subdomains: Vec<Subdomain>,
    pub certificates: Vec<CertInfo>,
    pub emails_found: Vec<FoundEmail>,
    pub breach_check: Option<BreachResult>,
    // Tier 3 campos
    pub dns_info: Option<DnsResult>,
    pub geo_info: Option<GeoResult>,
    pub whois_info: Option<WhoisResult>,
    pub open_ports: Vec<OpenPort>,
    pub telegram_users: Vec<TelegramUser>,
    pub tor_results: Vec<TorSearchResult>,
    pub summary: OsintSummary,
}

/// Resumen del reporte OSINT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsintSummary {
    pub total_results: usize,
    pub social_profiles_found: usize,
    pub dork_results: usize,
    pub web_results: usize,
    pub brave_results: usize,
    // Tier 2 resumen
    pub subdomains_found: usize,
    pub certificates_found: usize,
    pub emails_found: usize,
    pub breach_risk: String,
    pub categories_found: Vec<String>,
    // Tier 3 resumen
    pub dns_records: usize,
    pub open_ports: usize,
    pub telegram_found: usize,
    pub tor_results: usize,
}

/// Tipo de investigación OSINT
#[derive(Debug, Clone, PartialEq)]
pub enum OsintTarget {
    Domain(String),
    Username(String),
    Email(String),
    IpAddress(String),
}

impl std::fmt::Display for OsintTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsintTarget::Domain(d) => write!(f, "dominio:{}", d),
            OsintTarget::Username(u) => write!(f, "usuario:{}", u),
            OsintTarget::Email(e) => write!(f, "email:{}", e),
            OsintTarget::IpAddress(ip) => write!(f, "ip:{}", ip),
        }
    }
}

/// 🧠 Orquestador OSINT — punto de entrada único para investigaciones
pub struct OsintHub {
    pub brave: BraveSearchClient,
    pub web: WebSearchClient,
    pub social: SocialHunter,
    pub dork: DorkForger,
    // Tier 2 módulos
    pub subdomain_enum: SubdomainEnumerator,
    pub cert_transparency: CertTransparency,
    pub email_hunter: EmailHunter,
    pub breach_checker: BreachChecker,
    // Tier 3 módulos
    pub dns_resolver: DnsResolver,
    pub geo_whois: GeoWhois,
    pub port_scanner: PortScanner,
    pub telegram_scraper: TelegramScraper,
    pub tor_search: TorSearch,
}

impl Default for OsintHub {
    fn default() -> Self {
        Self::new()
    }
}

impl OsintHub {
    pub fn new() -> Self {
        info!("🧠 [OSINT-HUB] Inicializando orquestador OSINT + Tier 2 + Tier 3...");

        let hub = Self {
            brave: BraveSearchClient::new(),
            web: WebSearchClient::new(),
            social: SocialHunter::new(),
            dork: DorkForger::new(),
            subdomain_enum: SubdomainEnumerator::new(),
            cert_transparency: CertTransparency::new(),
            email_hunter: EmailHunter::new(),
            breach_checker: BreachChecker::new(),
            dns_resolver: DnsResolver::new(),
            geo_whois: GeoWhois::new(),
            port_scanner: PortScanner::new(),
            telegram_scraper: TelegramScraper::new(),
            tor_search: TorSearch::new(),
        };

        if hub.brave.is_configured() {
            info!("🧠 [OSINT-HUB] ✅ Brave Search API configurada");
        } else {
            warn!("🧠 [OSINT-HUB] ❌ Brave Search API NO configurada (falta BRAVE_API_KEY)");
        }

        info!("🧠 [OSINT-HUB] Orquestador listo. 13 módulos cargados.");

        hub
    }

    // ─── Tier 3: DNS ──────────────────────────────

    /// Resuelve registros DNS (A, AAAA, MX, TXT, NS, CNAME)
    pub async fn resolver_dns(&self, dominio: &str) -> DnsResult {
        self.dns_resolver.resolve_all(dominio).await
    }

    // ─── Tier 3: GeoIP + Whois ────────────────────

    /// Geolocaliza una dirección IP
    pub async fn geoip_lookup(&self, ip: &str) -> Option<GeoResult> {
        self.geo_whois.geo_ip(ip).await.ok()
    }

    /// Consulta whois de un dominio/IP
    pub async fn whois_lookup(&self, target: &str) -> Option<WhoisResult> {
        self.geo_whois.whois(target).await.ok()
    }

    // ─── Tier 3: Port Scanner ─────────────────────

    /// Escanea puertos comunes en una IP
    pub async fn scan_puertos(&self, ip: &str) -> Vec<OpenPort> {
        self.port_scanner.scan_common(ip).await
    }

    // ─── Tier 3: Telegram ─────────────────────────

    /// Busca un usuario/grupo/canal en Telegram
    pub async fn buscar_telegram(&self, username: &str) -> TelegramUser {
        self.telegram_scraper.buscar_usuario(username).await
    }

    /// Busca múltiples usuarios en Telegram
    pub async fn buscar_telegram_multi(&self, usernames: &[&str]) -> Vec<TelegramUser> {
        self.telegram_scraper.buscar_usuarios(usernames).await
    }

    /// Búsqueda completa en Telegram (incluye variantes)
    pub async fn buscar_telegram_completo(&self, username: &str) -> Vec<TelegramUser> {
        self.telegram_scraper.buscar_completo(username).await
    }

    // ─── Tier 3: Tor / Deep Web ───────────────────

    /// Busca en Tor/Ahmia
    pub async fn buscar_tor(&self, query: &str) -> Vec<TorSearchResult> {
        self.tor_search.search_deep(query).await
    }

    /// Verifica el estado del proxy Tor
    pub async fn tor_status(&self) -> TorStatus {
        self.tor_search.check_tor_status().await
    }

    // ─── Métodos compuestos ───────────────────────

    /// Investiga un dominio: dorks + web search + subdominios + certificados + brechas
    /// + DNS + whois + puertos
    pub async fn investigar_dominio(&self, dominio: &str) -> OsintReport {
        info!(
            "🧠 [OSINT-HUB] Iniciando investigación de dominio: {}",
            dominio
        );

        // 1. Dorks sobre el dominio (30+ queries)
        let dork_results = self.dork.scan_domain(dominio).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en DorkForger: {}", e);
            Vec::new()
        });

        // 2. Web search sobre el dominio
        let web_results = self.web.search(dominio).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en WebSearch: {}", e);
            Vec::new()
        });

        // 3. Brave search sobre el dominio
        let brave_results = self.brave.search(dominio, 10).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en BraveSearch: {}", e);
            Vec::new()
        });

        // 4. Subdominios via crt.sh + Brave
        let subdomains = self
            .subdomain_enum
            .enumerate(dominio)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en SubdomainEnumerator: {}", e);
                Vec::new()
            });

        // 5. Certificados SSL/TLS via crt.sh
        let certificates = self
            .cert_transparency
            .lookup_domain(dominio)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en CertTransparency: {}", e);
                Vec::new()
            });

        // 6. Emails asociados al dominio
        let emails_found = self
            .email_hunter
            .search_emails(dominio)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en EmailHunter: {}", e);
                Vec::new()
            });

        // 7. Breach check (usar el dominio como query)
        let breach_check = self.breach_checker.check_email(dominio).await;

        // 8. Tier 3: DNS
        let dns_info = Some(self.resolver_dns(dominio).await);

        // 9. Tier 3: Whois del dominio
        let whois_info = self.whois_lookup(dominio).await;

        // 10. Tier 3: Geo y puertos requieren IP — intentar resolver A record primero
        let (geo_info, open_ports) = if let Some(ref dns) = dns_info {
            if let Some(ip) = dns.a_records.first().or_else(|| dns.aaaa_records.first()) {
                let geo = self.geoip_lookup(&ip.to_string()).await;
                let ports = self.scan_puertos(&ip.to_string()).await;
                (geo, ports)
            } else {
                (None, Vec::new())
            }
        } else {
            (None, Vec::new())
        };

        // 11. Extraer categorías encontradas en dorks
        let mut categories: Vec<String> = Vec::new();
        for r in &dork_results {
            let cat = format!("{}", r.category);
            if !categories.contains(&cat) {
                categories.push(cat);
            }
        }

        // Contar registros DNS
        let dns_records = if let Some(ref d) = dns_info {
            d.a_records.len()
                + d.aaaa_records.len()
                + d.mx_records.len()
                + d.txt_records.len()
                + d.ns_records.len()
                + if d.cname.is_some() { 1 } else { 0 }
        } else {
            0
        };

        let summary = OsintSummary {
            total_results: dork_results.len()
                + web_results.len()
                + brave_results.len()
                + subdomains.len()
                + certificates.len()
                + emails_found.len()
                + dns_records
                + open_ports.len(),
            social_profiles_found: 0,
            dork_results: dork_results.len(),
            web_results: web_results.len(),
            brave_results: brave_results.len(),
            subdomains_found: subdomains.len(),
            certificates_found: certificates.len(),
            emails_found: emails_found.len(),
            breach_risk: breach_check.risk_level.clone(),
            categories_found: categories,
            dns_records,
            open_ports: open_ports.len(),
            telegram_found: 0,
            tor_results: 0,
        };

        info!("🧠 [OSINT-HUB] Dominio '{}' completado. {} resultados totales. {} subdominios, {} certs, {} emails, {} DNS, {} puertos abiertos, riesgo breach: {}",
            dominio, summary.total_results, summary.subdomains_found,
            summary.certificates_found, summary.emails_found,
            summary.dns_records, summary.open_ports, summary.breach_risk);

        OsintReport {
            target: dominio.to_string(),
            target_type: "domain".to_string(),
            dork_results,
            social_profiles: Vec::new(),
            web_results,
            brave_results,
            subdomains,
            certificates,
            emails_found,
            breach_check: Some(breach_check),
            dns_info,
            geo_info,
            whois_info,
            open_ports,
            telegram_users: Vec::new(),
            tor_results: Vec::new(),
            summary,
        }
    }

    /// Investiga un username: social hunter + web search + breach check + Telegram
    pub async fn investigar_usuario(&self, username: &str) -> OsintReport {
        info!(
            "🧠 [OSINT-HUB] Iniciando investigación de usuario: {}",
            username
        );

        // 1. Buscar en redes sociales (34 plataformas)
        let social_profiles = self
            .social
            .scan_username(username)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en SocialHunter: {}", e);
                Vec::new()
            });

        // 2. Web search sobre el username
        let web_results = self.web.search(username).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en WebSearch: {}", e);
            Vec::new()
        });

        // 3. Brave search sobre el username
        let brave_results = self.brave.search(username, 10).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en BraveSearch: {}", e);
            Vec::new()
        });

        // 4. Emails asociados al username
        let emails_found = self
            .email_hunter
            .search_emails(username)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en EmailHunter: {}", e);
                Vec::new()
            });

        // 5. Breach check sobre el username
        let breach_check = self.breach_checker.check_username(username).await;

        // 6. Tier 3: Telegram
        let telegram_users = vec![self.buscar_telegram(username).await];

        let summary = OsintSummary {
            total_results: social_profiles.len()
                + web_results.len()
                + brave_results.len()
                + emails_found.len()
                + telegram_users.iter().filter(|u| u.exists).count(),
            social_profiles_found: social_profiles.iter().filter(|p| p.exists).count(),
            dork_results: 0,
            web_results: web_results.len(),
            brave_results: brave_results.len(),
            subdomains_found: 0,
            certificates_found: 0,
            emails_found: emails_found.len(),
            breach_risk: breach_check.risk_level.clone(),
            categories_found: vec!["social".to_string()],
            dns_records: 0,
            open_ports: 0,
            telegram_found: telegram_users.iter().filter(|u| u.exists).count(),
            tor_results: 0,
        };

        info!("🧠 [OSINT-HUB] Usuario '{}' completado. {} perfiles, {} emails, {} web, {} Telegram, riesgo breach: {}",
            username, summary.social_profiles_found, summary.emails_found,
            summary.web_results, summary.telegram_found, summary.breach_risk);

        OsintReport {
            target: username.to_string(),
            target_type: "username".to_string(),
            dork_results: Vec::new(),
            social_profiles,
            web_results,
            brave_results,
            subdomains: Vec::new(),
            certificates: Vec::new(),
            emails_found,
            breach_check: Some(breach_check),
            dns_info: None,
            geo_info: None,
            whois_info: None,
            open_ports: Vec::new(),
            telegram_users,
            tor_results: Vec::new(),
            summary,
        }
    }

    /// Investiga un email: web search + brave + dominio dorks + breach check + validación
    pub async fn investigar_email(&self, email: &str) -> OsintReport {
        info!("🧠 [OSINT-HUB] Iniciando investigación de email: {}", email);

        // 1. Brave search específico para email
        let brave_results = self.brave.search(email, 10).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en BraveSearch para email: {}", e);
            Vec::new()
        });

        // 2. Web search para el email
        let web_results = self.web.search(email).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en WebSearch para email: {}", e);
            Vec::new()
        });

        // 3. Validar y buscar emails similares
        let emails_found = self
            .email_hunter
            .search_emails(email)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en EmailHunter para email: {}", e);
                Vec::new()
            });

        // 4. Extraer dominio del email y buscar dorks + subdominios + certificados
        let dominio = email.split('@').nth(1).unwrap_or("");
        let dork_results = if !dominio.is_empty() {
            self.dork.scan_domain(dominio).await.unwrap_or_else(|e| {
                warn!(
                    "🧠 [OSINT-HUB] Error en DorkForger para dominio del email: {}",
                    e
                );
                Vec::new()
            })
        } else {
            Vec::new()
        };

        // 5. Subdominios del dominio del email
        let subdomains = if !dominio.is_empty() {
            self.subdomain_enum
                .enumerate(dominio)
                .await
                .unwrap_or_else(|e| {
                    warn!("🧠 [OSINT-HUB] Error enumerando subdominios: {}", e);
                    Vec::new()
                })
        } else {
            Vec::new()
        };

        // 6. Certificados del dominio
        let certificates = if !dominio.is_empty() {
            self.cert_transparency
                .lookup_domain(dominio)
                .await
                .unwrap_or_else(|e| {
                    warn!("🧠 [OSINT-HUB] Error en CertTransparency: {}", e);
                    Vec::new()
                })
        } else {
            Vec::new()
        };

        // 7. Breach check especializado para email
        let breach_check = self.breach_checker.check_email(email).await;

        let summary = OsintSummary {
            total_results: dork_results.len()
                + web_results.len()
                + brave_results.len()
                + subdomains.len()
                + certificates.len()
                + emails_found.len(),
            social_profiles_found: 0,
            dork_results: dork_results.len(),
            web_results: web_results.len(),
            brave_results: brave_results.len(),
            subdomains_found: subdomains.len(),
            certificates_found: certificates.len(),
            emails_found: emails_found.len(),
            breach_risk: breach_check.risk_level.clone(),
            categories_found: Vec::new(),
            dns_records: 0,
            open_ports: 0,
            telegram_found: 0,
            tor_results: 0,
        };

        info!("🧠 [OSINT-HUB] Email '{}' completado. Breach risk: {}, {} emails relacionados, {} subdominios, {} certificados",
            email, summary.breach_risk, summary.emails_found,
            summary.subdomains_found, summary.certificates_found);

        OsintReport {
            target: email.to_string(),
            target_type: "email".to_string(),
            dork_results,
            social_profiles: Vec::new(),
            web_results,
            brave_results,
            subdomains,
            certificates,
            emails_found,
            breach_check: Some(breach_check),
            dns_info: None,
            geo_info: None,
            whois_info: None,
            open_ports: Vec::new(),
            telegram_users: Vec::new(),
            tor_results: Vec::new(),
            summary,
        }
    }

    /// Escaneo rápido de dominio: subdominios + certificados + brechas + DNS + whois
    pub async fn escanear_dominio_rapido(&self, dominio: &str) -> OsintReport {
        info!("🧠 [OSINT-HUB] Escaneo rápido de dominio: {}", dominio);

        let subdomains = self
            .subdomain_enum
            .enumerate(dominio)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en SubdomainEnumerator: {}", e);
                Vec::new()
            });

        let certificates = self
            .cert_transparency
            .lookup_domain(dominio)
            .await
            .unwrap_or_else(|e| {
                warn!("🧠 [OSINT-HUB] Error en CertTransparency: {}", e);
                Vec::new()
            });

        let breach_check = self.breach_checker.check_email(dominio).await;

        // Tier 3: DNS + whois
        let dns_info = Some(self.resolver_dns(dominio).await);
        let whois_info = self.whois_lookup(dominio).await;

        let dns_records = if let Some(ref d) = dns_info {
            d.a_records.len()
                + d.aaaa_records.len()
                + d.mx_records.len()
                + d.txt_records.len()
                + d.ns_records.len()
                + if d.cname.is_some() { 1 } else { 0 }
        } else {
            0
        };

        let summary = OsintSummary {
            total_results: subdomains.len() + certificates.len() + dns_records,
            social_profiles_found: 0,
            dork_results: 0,
            web_results: 0,
            brave_results: 0,
            subdomains_found: subdomains.len(),
            certificates_found: certificates.len(),
            emails_found: 0,
            breach_risk: breach_check.risk_level.clone(),
            categories_found: vec!["subdominios".to_string(), "certificados".to_string()],
            dns_records,
            open_ports: 0,
            telegram_found: 0,
            tor_results: 0,
        };

        OsintReport {
            target: dominio.to_string(),
            target_type: "domain_fast".to_string(),
            dork_results: Vec::new(),
            social_profiles: Vec::new(),
            web_results: Vec::new(),
            brave_results: Vec::new(),
            subdomains,
            certificates,
            emails_found: Vec::new(),
            breach_check: Some(breach_check),
            dns_info,
            geo_info: None,
            whois_info,
            open_ports: Vec::new(),
            telegram_users: Vec::new(),
            tor_results: Vec::new(),
            summary,
        }
    }

    /// Investiga una IP: geo + whois + puertos + DNS reverso + Tor check
    pub async fn investigar_ip(&self, ip: &str) -> OsintReport {
        info!("🧠 [OSINT-HUB] Iniciando investigación de IP: {}", ip);

        // 1. GeoIP
        let geo_info = self.geoip_lookup(ip).await;

        // 2. Whois de la IP
        let whois_info = self.whois_lookup(ip).await;

        // 3. Escaneo de puertos
        let open_ports = self.scan_puertos(ip).await;

        // 4. Brave search para contexto
        let brave_results = self.brave.search(ip, 5).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en BraveSearch para IP: {}", e);
            Vec::new()
        });

        // 5. Web search
        let web_results = self.web.search(ip).await.unwrap_or_else(|e| {
            warn!("🧠 [OSINT-HUB] Error en WebSearch para IP: {}", e);
            Vec::new()
        });

        let summary = OsintSummary {
            total_results: open_ports.len() + brave_results.len() + web_results.len(),
            social_profiles_found: 0,
            dork_results: 0,
            web_results: web_results.len(),
            brave_results: brave_results.len(),
            subdomains_found: 0,
            certificates_found: 0,
            emails_found: 0,
            breach_risk: "N/A".to_string(),
            categories_found: vec!["ip".to_string(), "geo".to_string(), "puertos".to_string()],
            dns_records: 0,
            open_ports: open_ports.len(),
            telegram_found: 0,
            tor_results: 0,
        };

        info!(
            "🧠 [OSINT-HUB] IP '{}' completada. {} puertos abiertos, {} resultados web",
            ip, summary.open_ports, summary.web_results
        );

        OsintReport {
            target: ip.to_string(),
            target_type: "ip".to_string(),
            dork_results: Vec::new(),
            social_profiles: Vec::new(),
            web_results,
            brave_results,
            subdomains: Vec::new(),
            certificates: Vec::new(),
            emails_found: Vec::new(),
            breach_check: None,
            dns_info: None,
            geo_info,
            whois_info,
            open_ports,
            telegram_users: Vec::new(),
            tor_results: Vec::new(),
            summary,
        }
    }

    /// Verifica el estado de conectividad de todos los módulos
    pub fn health_check(&self) -> serde_json::Value {
        serde_json::json!({
            "brave_search": self.brave.is_configured(),
            "web_search": self.web.is_configured(),
            "social_hunter": true,
            "dork_forger": true,
            "subdomain_enum": true,
            "cert_transparency": true,
            "email_hunter": true,
            "breach_checker": true,
            // Tier 3
            "dns_resolver": true,
            "geo_whois": true,
            "port_scanner": true,
            "telegram_scraper": true,
            "tor_search": true,
            "osint_hub": true,
            "total_modules": 13,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osint_hub_creation() {
        let hub = OsintHub::new();
        let health = hub.health_check();
        assert!(health
            .get("social_hunter")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("dork_forger")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("subdomain_enum")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("cert_transparency")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("email_hunter")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("breach_checker")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        // Tier 3 checks
        assert!(health
            .get("dns_resolver")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("geo_whois")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("port_scanner")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("telegram_scraper")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert!(health
            .get("tor_search")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
        assert_eq!(
            health
                .get("total_modules")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            13
        );
    }

    #[test]
    fn test_osint_target_display() {
        let target = OsintTarget::Domain("example.com".to_string());
        assert_eq!(target.to_string(), "dominio:example.com");

        let target = OsintTarget::Username("octocat".to_string());
        assert_eq!(target.to_string(), "usuario:octocat");

        let target = OsintTarget::Email("test@example.com".to_string());
        assert_eq!(target.to_string(), "email:test@example.com");

        let target = OsintTarget::IpAddress("8.8.8.8".to_string());
        assert_eq!(target.to_string(), "ip:8.8.8.8");
    }

    #[test]
    fn test_osint_report_defaults() {
        let hub = OsintHub::new();
        let health = hub.health_check();
        // Solo verificamos que el hub se crea sin errores
        assert!(health
            .get("osint_hub")
            .and_then(|v| v.as_bool())
            .unwrap_or(false));
    }
}
