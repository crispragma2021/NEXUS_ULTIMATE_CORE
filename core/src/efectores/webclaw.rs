use crate::efectores::webclaw_extractor::WebClawExtractor;
use anyhow::Result;

/// 🌐 WEBCLAW: Web Pool con Sigilo Blindado (Legacy Wrapper)
/// Redirige las llamadas al extractor unificado WebClawExtractor para evitar redundancias de código.
#[allow(dead_code)]
pub struct WebClaw {
    extractor: WebClawExtractor,
}

impl WebClaw {
    pub fn new() -> Result<Self> {
        let extractor = WebClawExtractor::new()?;
        Ok(Self { extractor })
    }

    /// Extrae respuesta de Gemini Web delegando al WebClawExtractor
    pub async fn extraer_respuesta(&mut self, prompt: &str) -> Result<String> {
        self.extractor.extraer_respuesta(prompt).await
    }
}
