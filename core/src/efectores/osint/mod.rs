// ──────────────────────────────────────────────
// 🕵️ OSINT: Módulo de Inteligencia de Fuentes Abiertas
// Orquestador: OsintHub
// Tier 1: BraveSearch, WebSearch, SocialHunter, DorkForger
// Tier 2: SubdomainEnum, CertTransparency, EmailHunter, BreachChecker
// Tier 3: DNSResolver, GeoWhois, PortScanner, TelegramScraper, TorSearch
// Legacy: DorkEngine, UsernameScanner, ShadowCrawlClient
// ──────────────────────────────────────────────

// ── Nuevos módulos Tier 1 ──
pub mod brave_search;
pub mod dork_forger;
pub mod hub;
pub mod social_hunter;
pub mod web_search;

// ── Nuevos módulos Tier 2 ──
pub mod breach_checker;
pub mod cert_transparency;
pub mod email_hunter;
pub mod subdomain_enum;

// ── Nuevos módulos Tier 3 ──
pub mod dns_resolver;
pub mod geo_whois;
pub mod port_scanner;
pub mod telegram_scraper;
pub mod tor_search;

// ── Módulos legacy (mantenidos para retrocompatibilidad) ──
pub mod dork_engine;
pub mod shadow_client;
pub mod username_enum;

// ── Re-exports nuevos Tier 1 ──
pub use brave_search::{BraveSearchClient, BraveSearchResult};
pub use dork_forger::{DorkCategory, DorkForger, DorkResult};
pub use hub::{OsintHub, OsintReport, OsintSummary, OsintTarget};
pub use social_hunter::{SocialHunter, SocialProfile};
pub use web_search::{WebSearchClient, WebSearchResult};

// ── Re-exports nuevos Tier 2 ──
pub use breach_checker::{BreachChecker, BreachInfo, BreachResult};
pub use cert_transparency::{CertInfo, CertTransparency};
pub use email_hunter::{EmailHunter, EmailValidity, FoundEmail};
pub use subdomain_enum::{Subdomain, SubdomainEnumerator};

// ── Re-exports nuevos Tier 3 ──
pub use dns_resolver::{DnsRecord, DnsResolver, DnsResult};
pub use geo_whois::{GeoResult, GeoWhois, WhoisResult};
pub use port_scanner::{OpenPort, PortScanner};
pub use telegram_scraper::{TelegramScraper, TelegramUser};
pub use tor_search::{TorSearch, TorSearchResult, TorStatus};

// ── Re-exports legacy ──
pub use dork_engine::DorkEngine;
pub use shadow_client::{ShadowCrawlClient, ShadowSearchResult};
pub use username_enum::UsernameScanner;
