// ──────────────────────────────────────────────
// 🔩 DORK FORGER — Motor de generación y ejecución de Google Dorks
// Reemplaza a DorkEngine con:
//   - 30+ dorks organizados por categoría
//   - Backend: Brave Search API (no Google directo, evita CAPTCHA)
//   - Fallback a WebSearchClient (Exa/Tavily)
//   - Categorización estructurada de resultados
// ──────────────────────────────────────────────

use crate::efectores::osint::brave_search::{BraveSearchClient, BraveSearchResult};
use crate::efectores::osint::web_search::{WebSearchClient, WebSearchResult};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Categorías de dorks para organización
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DorkCategory {
    Files,           // Archivos por extensión (php, asp, sql, pdf, xls, etc.)
    AdminPanels,     // Paneles de administración y login
    DataExposure,    // Exposición de datos sensibles
    Vulnerabilities, // Vulnerabilidades conocidas
    IPCameras,       // Cámaras IP y dispositivos IoT
    Configs,         // Archivos de configuración expuestos
    Backups,         // Backups y versiones antiguas
    Directories,     // Directory listing
    Emails,          // Direcciones de email expuestas
    Logs,            // Archivos de log
}

impl std::fmt::Display for DorkCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DorkCategory::Files => write!(f, "Archivos por extensión"),
            DorkCategory::AdminPanels => write!(f, "Paneles Admin"),
            DorkCategory::DataExposure => write!(f, "Exposición de Datos"),
            DorkCategory::Vulnerabilities => write!(f, "Vulnerabilidades"),
            DorkCategory::IPCameras => write!(f, "Cámaras IP"),
            DorkCategory::Configs => write!(f, "Configuraciones"),
            DorkCategory::Backups => write!(f, "Backups"),
            DorkCategory::Directories => write!(f, "Directory Listing"),
            DorkCategory::Emails => write!(f, "Correos Electrónicos"),
            DorkCategory::Logs => write!(f, "Archivos de Log"),
        }
    }
}

/// Resultado individual de un dork
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DorkResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub dork_used: String,
    pub category: DorkCategory,
    pub source: String,
}

impl From<BraveSearchResult> for DorkResult {
    fn from(r: BraveSearchResult) -> Self {
        Self {
            url: r.url,
            title: r.title,
            snippet: r.snippet,
            dork_used: String::new(),
            category: DorkCategory::Files,
            source: r.source,
        }
    }
}

impl From<WebSearchResult> for DorkResult {
    fn from(r: WebSearchResult) -> Self {
        Self {
            url: r.url,
            title: r.title,
            snippet: r.snippet,
            dork_used: String::new(),
            category: DorkCategory::Files,
            source: r.source,
        }
    }
}

/// Definición de un dork: query + categoría
struct DorkDef {
    query: &'static str,
    category: DorkCategory,
}

/// 🔩 Motor de Dorks con backend Brave Search
pub struct DorkForger {
    brave: BraveSearchClient,
    web: Option<WebSearchClient>,
    dorks: Vec<DorkDef>,
}

impl Default for DorkForger {
    fn default() -> Self {
        Self::new()
    }
}

impl DorkForger {
    pub fn new() -> Self {
        let brave = BraveSearchClient::new();
        let web = std::env::var("EXA_API_KEY")
            .or_else(|_| std::env::var("TAVILY_API_KEY"))
            .ok()
            .map(|_| WebSearchClient::new());

        // 30+ dorks organizados por categoría
        let dorks = vec![
            // ── Archivos por Extensión ──
            DorkDef {
                query: "ext:php OR ext:asp OR ext:aspx",
                category: DorkCategory::Files,
            },
            DorkDef {
                query: "ext:sql OR ext:db OR ext:bak",
                category: DorkCategory::Files,
            },
            DorkDef {
                query: "ext:pdf OR ext:doc OR ext:docx OR ext:xls",
                category: DorkCategory::Files,
            },
            DorkDef {
                query: "ext:xml OR ext:json OR ext:config",
                category: DorkCategory::Files,
            },
            DorkDef {
                query: "ext:log OR ext:txt OR ext:dat",
                category: DorkCategory::Files,
            },
            DorkDef {
                query: "ext:env OR ext:yml OR ext:yaml",
                category: DorkCategory::Configs,
            },
            DorkDef {
                query: "ext:conf OR ext:cfg OR ext:ini",
                category: DorkCategory::Configs,
            },
            DorkDef {
                query: "ext:pem OR ext:key OR ext:cert",
                category: DorkCategory::DataExposure,
            },
            // ── Paneles Administrativos ──
            DorkDef {
                query: "inurl:admin OR inurl:login OR inurl:panel",
                category: DorkCategory::AdminPanels,
            },
            DorkDef {
                query: "inurl:cpanel OR inurl:plesk OR inurl:webadmin",
                category: DorkCategory::AdminPanels,
            },
            DorkDef {
                query: "inurl:phpmyadmin OR inurl:pma",
                category: DorkCategory::AdminPanels,
            },
            DorkDef {
                query: "intitle:\"Control Panel\" OR intitle:\"Administration\"",
                category: DorkCategory::AdminPanels,
            },
            // ── Directorios y Listados ──
            DorkDef {
                query: "intitle:\"index of\"",
                category: DorkCategory::Directories,
            },
            DorkDef {
                query: "intitle:\"Index of /\" OR intitle:\"Directory Listing\"",
                category: DorkCategory::Directories,
            },
            DorkDef {
                query: "intitle:\"Parent Directory\"",
                category: DorkCategory::Directories,
            },
            // ── Exposición de Datos ──
            DorkDef {
                query: "\"confidential\" OR \"internal use only\"",
                category: DorkCategory::DataExposure,
            },
            DorkDef {
                query: "\"password\" OR \"passwd\" OR \"secret\"",
                category: DorkCategory::DataExposure,
            },
            DorkDef {
                query: "inurl:wp-config OR inurl:config.php",
                category: DorkCategory::DataExposure,
            },
            DorkDef {
                query: "\"api_key\" OR \"api-secret\" OR \"apikey\"",
                category: DorkCategory::DataExposure,
            },
            // ── Vulnerabilidades ──
            DorkDef {
                query: "inurl:wp-content/uploads OR inurl:wp-includes",
                category: DorkCategory::Vulnerabilities,
            },
            DorkDef {
                query: "intitle:\"SQLiteManager\" OR intitle:\"phpMyAdmin\"",
                category: DorkCategory::Vulnerabilities,
            },
            DorkDef {
                query: "inurl:debug OR inurl:test OR inurl:dev",
                category: DorkCategory::Vulnerabilities,
            },
            // ── Cámaras IP y IoT ──
            DorkDef {
                query: "intitle:\"Live View\" OR intitle:\"IP Camera\"",
                category: DorkCategory::IPCameras,
            },
            DorkDef {
                query: "inurl:\"view.shtml\" OR inurl:\"CgiStart\"",
                category: DorkCategory::IPCameras,
            },
            DorkDef {
                query: "intitle:\"webcam\" OR intitle:\"Network Camera\"",
                category: DorkCategory::IPCameras,
            },
            // ── Backups ──
            DorkDef {
                query: "inurl:backup OR inurl:backup.rar OR inurl:backup.zip",
                category: DorkCategory::Backups,
            },
            DorkDef {
                query: "intitle:\"backup\" ext:sql OR ext:bak",
                category: DorkCategory::Backups,
            },
            DorkDef {
                query: "ext:old OR ext:swp OR ext:save",
                category: DorkCategory::Backups,
            },
            // ── Logs ──
            DorkDef {
                query: "intitle:\"error.log\" OR intitle:\"access.log\"",
                category: DorkCategory::Logs,
            },
            DorkDef {
                query: "ext:log \"error\" OR \"warning\" OR \"fatal\"",
                category: DorkCategory::Logs,
            },
            // ── Correos Electrónicos ──
            DorkDef {
                query: "\"@\" \"email\" OR \"mail\" OR \"contact\"",
                category: DorkCategory::Emails,
            },
        ];

        Self { brave, web, dorks }
    }

    /// Escanea un dominio con todas las categorías de dorks
    /// Retorna todos los resultados encontrados
    pub async fn scan_domain(&self, domain: &str) -> anyhow::Result<Vec<DorkResult>> {
        info!(
            "🔩 [DORK-FORGER] Iniciando reconocimiento OSINT sobre: {}",
            domain
        );

        let mut all_results = Vec::new();
        let total = self.dorks.len();

        for (i, dork) in self.dorks.iter().enumerate() {
            let query = format!("site:{} {}", domain, dork.query);
            debug!(
                "🔩 [DORK-FORGER] ({}/{}) Ejecutando dork: {}",
                i + 1,
                total,
                query
            );

            match self.brave.search_dork(&query).await {
                Ok(results) => {
                    for r in results {
                        let mut dr: DorkResult = r.into();
                        dr.dork_used = dork.query.to_string();
                        dr.category = dork.category.clone();
                        all_results.push(dr);
                    }
                }
                Err(e) => {
                    // Si Brave falla, intentar con WebSearch (Exa/Tavily)
                    if let Some(ref web) = self.web {
                        debug!("🔩 [DORK-FORGER] Brave falló, intentando WebSearch fallback...");
                        match web.search(&query).await {
                            Ok(results) => {
                                for r in results {
                                    let mut dr: DorkResult = r.into();
                                    dr.dork_used = dork.query.to_string();
                                    dr.category = dork.category.clone();
                                    all_results.push(dr);
                                }
                            }
                            Err(e2) => {
                                warn!("🔩 [DORK-FORGER] Dork '{}' falló en ambos backends: Brave={}, WebSearch={}", dork.query, e, e2);
                            }
                        }
                    } else {
                        warn!(
                            "🔩 [DORK-FORGER] Dork '{}' falló: {} (sin WebSearch fallback)",
                            dork.query, e
                        );
                    }
                }
            }
        }

        info!(
            "✅ [DORK-FORGER] Escaneo completado. {} dorks ejecutados, {} resultados.",
            total,
            all_results.len()
        );
        Ok(all_results)
    }

    /// Escanea un dominio solo en categorías específicas
    pub async fn scan_categories(
        &self,
        domain: &str,
        categories: &[DorkCategory],
    ) -> anyhow::Result<Vec<DorkResult>> {
        info!(
            "🔩 [DORK-FORGER] Escaneo selectivo por categorías en: {}",
            domain
        );

        let mut all_results = Vec::new();

        for dork in &self.dorks {
            if !categories.contains(&dork.category) {
                continue;
            }

            let query = format!("site:{} {}", domain, dork.query);
            match self.brave.search_dork(&query).await {
                Ok(results) => {
                    for r in results {
                        let mut dr: DorkResult = r.into();
                        dr.dork_used = dork.query.to_string();
                        dr.category = dork.category.clone();
                        all_results.push(dr);
                    }
                }
                Err(e) => {
                    warn!("🔩 [DORK-FORGER] Dork '{}' falló: {}", dork.query, e);
                }
            }
        }

        Ok(all_results)
    }

    /// Lista las categorías disponibles
    pub fn categories(&self) -> Vec<DorkCategory> {
        use DorkCategory::*;
        vec![
            Files,
            AdminPanels,
            DataExposure,
            Vulnerabilities,
            IPCameras,
            Configs,
            Backups,
            Directories,
            Emails,
            Logs,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dork_forger_creation() {
        let forger = DorkForger::new();
        assert!(
            forger.dorks.len() >= 30,
            "Esperaba >= 30 dorks, tengo {}",
            forger.dorks.len()
        );
    }

    #[test]
    fn test_dork_forger_categories() {
        let forger = DorkForger::new();
        let cats = forger.categories();
        assert!(!cats.is_empty());
        assert!(cats.contains(&DorkCategory::Files));
        assert!(cats.contains(&DorkCategory::AdminPanels));
    }
}
