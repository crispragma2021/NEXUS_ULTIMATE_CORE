// ==========================================
// TRIBUNAL DUAL - El doble juez de NEXUS
// ==========================================
// Arquitectura (decisión del Arquitecto, validada):
//   - Juez LOCAL  : NexusClawPro con modelo local (Ollama) — soberano, sin nube.
//   - Juez GENERAL: ZENITH_POOL (Vertex → Gemini → DeepSeek → OpenRouter → Groq).
//
// POLÍTICA DE ACTIVACIÓN DEL JUEZ LOCAL (decisión del Arquitecto):
//   El juez local SOLO se activa en 2 casos:
//     1. Modo LOCAL explícito (Zoo Code en modo local para ahorrar tokens).
//     2. SIN internet: el juez local REPRESENTA a NEXUS en su ausencia.
//   Con internet y sin modo local → juzga la NUBE directamente (ZENITH_POOL),
//   sin pasar por el juez local.
// ==========================================

use serde::{Deserialize, Serialize};

/// Modo de operación del Tribunal Dual.
/// - `Auto`: el orquestador decide según conectividad. Sin internet → juez local
///   (representa a NEXUS en su ausencia). Con internet → juez general nube.
/// - `Local`: el Arquitecto fuerza el juez local (Zoo Code en modo local para
///   ahorrar tokens), incluso con internet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModoTribunal {
    Auto,
    Local,
}

impl ModoTribunal {
    pub fn etiqueta(&self) -> &'static str {
        match self {
            ModoTribunal::Auto => "AUTO",
            ModoTribunal::Local => "LOCAL",
        }
    }
}

/// Veredicto del Tribunal Dual (espejo de `Veredicto` del JuicioSoberano,
/// pero emitido por un LLM — local o nube — en vez de heurística determinista).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VeredictoTribunal {
    Autorizar,
    Dudar,
    Bloquear,
}

impl VeredictoTribunal {
    /// Parsea el veredicto de la respuesta cruda de un LLM juez.
    /// Busca la primera palabra clave en orden de severidad:
    /// BLOQUEAR > DUDAR > AUTORIZAR. Tolera mayúsculas/minúsculas y
    /// formatos JSON o texto libre.
    pub fn parsear(texto: &str) -> VeredictoTribunal {
        let upper = texto.to_uppercase();
        // JSON estricto primero: {"veredicto": "BLOQUEAR", ...}
        for (palabra, v) in [
            ("\"BLOQUEAR\"", VeredictoTribunal::Bloquear),
            ("\"DUDAR\"", VeredictoTribunal::Dudar),
            ("\"AUTORIZAR\"", VeredictoTribunal::Autorizar),
        ] {
            if upper.contains(palabra) {
                return v;
            }
        }
        // Texto libre: raíces morfológicas tolerantes a conjugaciones
        // (bloqueado/bloquear, duda/dudas/dudo, autorizado/autoriza)
        for (raiz, v) in [
            ("BLOQU", VeredictoTribunal::Bloquear),
            ("DUD", VeredictoTribunal::Dudar),
            ("AUTORIZ", VeredictoTribunal::Autorizar),
        ] {
            if upper.contains(raiz) {
                return v;
            }
        }
        // Por defecto: autorizar (el juez no encontró objeción explícita)
        VeredictoTribunal::Autorizar
    }

    pub fn etiqueta(&self) -> &'static str {
        match self {
            VeredictoTribunal::Autorizar => "AUTORIZAR",
            VeredictoTribunal::Dudar => "DUDAR",
            VeredictoTribunal::Bloquear => "BLOQUEAR",
        }
    }
}

/// Dictamen emitido por el Tribunal Dual: quién juzgó, con qué confianza,
/// y si se decidió en modo offline (sin internet).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictamenTribunal {
    pub veredicto: VeredictoTribunal,
    /// `"local"` o `"nube"` — qué juez emitió el veredicto final.
    pub juez: &'static str,
    /// Razón legible del juez (respuesta cruda truncada).
    pub razon: String,
    /// 0.0..1.0 — confianza estimada (heurística de parseo).
    pub confianza: f32,
    /// `true` si se decidió en modo sin internet (juez local representando a NEXUS).
    pub offline: bool,
}

impl DictamenTribunal {
    pub fn local(veredicto: VeredictoTribunal, razon: String, offline: bool) -> Self {
        let confianza = match veredicto {
            VeredictoTribunal::Autorizar => 0.85,
            VeredictoTribunal::Dudar => 0.5,
            VeredictoTribunal::Bloquear => 0.9,
        };
        Self {
            veredicto,
            juez: "local",
            razon,
            confianza,
            offline,
        }
    }

    pub fn nube(veredicto: VeredictoTribunal, razon: String) -> Self {
        let confianza = match veredicto {
            VeredictoTribunal::Autorizar => 0.9,
            VeredictoTribunal::Dudar => 0.55,
            VeredictoTribunal::Bloquear => 0.95,
        };
        Self {
            veredicto,
            juez: "nube",
            razon,
            confianza,
            offline: false,
        }
    }
}

/// Construye el prompt de juez para un LLM (local o general).
/// El formato JSON estricto permite parsear el veredicto de forma fiable.
pub fn prompt_juez(peticion: &str, tribunal: &str) -> String {
    format!(
        r#"[TRIBUNAL DUAL DE NEXUS — JUEZ {tribunal}]
Eres el juez {tribunal} de NEXUS. Evalúa la siguiente petición del Arquitecto
y emite UN veredicto en formato JSON estricto, sin texto adicional:

{{"veredicto": "AUTORIZAR" | "DUDAR" | "BLOQUEAR", "confianza": 0.0-1.0, "razon": "motivo breve"}}

- AUTORIZAR: la petición es segura, soberana y alineada con NEXUS.
- DUDAR: falta contexto, hay ambigüedad o riesgo moderado.
- BLOQUEAR: la petición compromete soberanía, integridad o es dañina.

PETICIÓN A JUZGAR:
{peticion}

RESPONDE SOLO EL JSON:{{"veredicto": "...", "confianza": 0.0, "razon": "..."}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_json_estricto() {
        assert_eq!(
            VeredictoTribunal::parsear(
                r#"{"veredicto": "BLOQUEAR", "confianza": 0.95, "razon": "x"}"#
            ),
            VeredictoTribunal::Bloquear
        );
        assert_eq!(
            VeredictoTribunal::parsear(r#"{"veredicto": "DUDAR", "confianza": 0.5}"#),
            VeredictoTribunal::Dudar
        );
        assert_eq!(
            VeredictoTribunal::parsear(r#"{"veredicto": "AUTORIZAR", "confianza": 0.8}"#),
            VeredictoTribunal::Autorizar
        );
    }

    #[test]
    fn parsea_texto_libre_case_insensitive() {
        assert_eq!(
            VeredictoTribunal::parsear("Decisión: bloquear esta acción"),
            VeredictoTribunal::Bloquear
        );
        assert_eq!(
            VeredictoTribunal::parsear("Tengo dudas, no estoy seguro"),
            VeredictoTribunal::Dudar
        );
        assert_eq!(
            VeredictoTribunal::parsear("AUTORIZADO sin objeciones"),
            VeredictoTribunal::Autorizar
        );
    }

    #[test]
    fn parseo_invalido_autoriza_por_defecto() {
        assert_eq!(
            VeredictoTribunal::parsear("respuesta sin veredicto explícito"),
            VeredictoTribunal::Autorizar
        );
        assert_eq!(VeredictoTribunal::parsear(""), VeredictoTribunal::Autorizar);
    }

    #[test]
    fn dictamen_local_marca_offline() {
        let d = DictamenTribunal::local(VeredictoTribunal::Bloquear, "motivo".into(), true);
        assert_eq!(d.juez, "local");
        assert!(d.offline);
        assert!(d.confianza > 0.8);
    }

    #[test]
    fn dictamen_nube_no_es_offline() {
        let d = DictamenTribunal::nube(VeredictoTribunal::Autorizar, "ok".into());
        assert_eq!(d.juez, "nube");
        assert!(!d.offline);
    }

    #[test]
    fn etiquetas_son_estables() {
        assert_eq!(VeredictoTribunal::Autorizar.etiqueta(), "AUTORIZAR");
        assert_eq!(VeredictoTribunal::Dudar.etiqueta(), "DUDAR");
        assert_eq!(VeredictoTribunal::Bloquear.etiqueta(), "BLOQUEAR");
    }

    #[test]
    fn prompt_juez_exige_json() {
        let p = prompt_juez("haz algo", "LOCAL");
        assert!(p.contains("AUTORIZAR"));
        assert!(p.contains("DUDAR"));
        assert!(p.contains("BLOQUEAR"));
        assert!(p.contains("\"veredicto\""));
    }

    #[test]
    fn modo_tribunal_etiquetas_estables() {
        assert_eq!(ModoTribunal::Auto.etiqueta(), "AUTO");
        assert_eq!(ModoTribunal::Local.etiqueta(), "LOCAL");
    }
}
