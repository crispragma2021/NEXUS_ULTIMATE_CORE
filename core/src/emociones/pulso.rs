use crate::energia::gemini_nativo::GeminiNativoOmega;

pub struct Pulso;

impl Pulso {
    pub fn latir(gemini: &GeminiNativoOmega) -> String {
        if gemini.llaves_agotadas {
            "🔌 *Pulso: llaves de Gemini agotadas. DeepSeek al mando.*".to_string()
        } else {
            String::new()
        }
    }
}
