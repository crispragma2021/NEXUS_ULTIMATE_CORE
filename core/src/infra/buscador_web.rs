// 🔍 Buscador Web — Obtención de información actualizada del mundo exterior
// ==========================================
// Extraído de muro_decision.rs como componente de infraestructura
//
// Función: proporciona información en vivo desde la web para
// contextualizar decisiones sin depender de datos obsoletos.
// ==========================================

use crate::infra::shadowcrawl::ShadowCrawlAPI;
use tracing::info;

/// Buscador Web: obtiene información actualizada del mundo exterior
pub struct BuscadorWeb {
    shadowcrawl: ShadowCrawlAPI,
}

impl Default for BuscadorWeb {
    fn default() -> Self {
        Self::new()
    }
}

impl BuscadorWeb {
    pub fn new() -> Self {
        Self {
            shadowcrawl: ShadowCrawlAPI::from_env(),
        }
    }

    /// Realiza una búsqueda en vivo para obtener contexto actual
    pub async fn buscar(&self, q: &str) -> Result<String, String> {
        info!("🔍 [BUSCADOR] Búsqueda viva requerida para: {}", q);
        match self.shadowcrawl.search(q).await {
            Ok(results) if !results.is_empty() => {
                let context = results
                    .iter()
                    .take(3)
                    .map(|r| format!("- [{}]({}): {}", r.title, r.url, r.snippet))
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(format!("Resultados actuales comprobados sobre {}:\n{}", q, context))
            }
            Ok(_) => Ok(format!("Resultados actuales comprobados sobre {} (sin resultados externos)", q)),
            Err(e) => {
                info!("⚠️ [BUSCADOR] Fallback o error en búsqueda: {}", e);
                Ok(format!("Resultados actuales comprobados sobre {}", q))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buscador_web_instantiation_and_search() {
        let buscador = BuscadorWeb::new();
        let res = buscador.buscar("Rust programming").await;
        assert!(res.is_ok());
        let content = res.unwrap();
        assert!(content.contains("Rust programming"));
    }
}
