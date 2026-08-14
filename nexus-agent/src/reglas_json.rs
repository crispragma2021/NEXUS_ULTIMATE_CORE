// ============================================================================
// NEXUS-AGENT · reglas_json.rs — Guardarraíl JSON estricto
// ============================================================================
// El bucle agéntico exige que el modelo devuelva pasos estructurados en JSON.
// Este módulo define:
//   - El esquema de un paso (razonamiento + instrumento opcional + respuesta final opcional)
//   - El parseo robusto: tolera markdown, bloques ```json```, texto circundante
//   - La corrección: si el modelo devuelve JSON inválido, se reinyecta el error
//     para que el modelo se autocorrija en el siguiente turno.
// ============================================================================

use serde::{Deserialize, Serialize};

/// Un instrumento que el agente desea invocar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentoLlamado {
    pub nombre: String,
    pub argumentos: serde_json::Value,
}

impl InstrumentoLlamado {
    /// Extrae un argumento string, con fallback a vacío.
    /// Acepta tanto valores string como objetos/arrays (los serializa a JSON).
    pub fn argumento(&self, clave: &str) -> Option<String> {
        self.argumentos.get(clave).and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else {
                // Objeto/array/número → serializar a JSON string
                serde_json::to_string(v).ok()
            }
        })
    }
}

/// Un paso del bucle agéntico. La respuesta final anula al instrumento.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasoEstructurado {
    /// Razonamiento del agente (visible en trazas).
    #[serde(default)]
    pub razonamiento: String,
    /// Instrumento a invocar (si el agente aún no termina).
    #[serde(default)]
    pub instrumento: Option<InstrumentoLlamado>,
    /// Respuesta final al usuario (presente solo cuando el agente termina).
    #[serde(default)]
    pub respuesta_final: Option<String>,
}

impl PasoEstructurado {
    pub fn es_final(&self) -> bool {
        self.respuesta_final.is_some()
    }
}

/// Guardarraíl: conoce el esquema, lo parsea con tolerancia y lo corrige.
#[derive(Debug, Clone, Copy)]
pub struct ReglasJSON;

impl ReglasJSON {
    /// Plantilla de instrucción que se incrusta en la instrucción maestra.
    pub fn plantilla_esquema() -> &'static str {
        r#"FORMATO DE RESPUESTA (OBLIGATORIO):
Debes responder SIEMPRE con un único objeto JSON válido, sin texto fuera de él.
Esquema exacto:
{
  "razonamiento": "tu razonamiento breve",
  "instrumento": { "nombre": "nombre_del_instrumento", "argumentos": { ... } },
  "respuesta_final": null
}
- Si ya tienes la respuesta para el usuario, pon "respuesta_final" y deja "instrumento": null.
- Si necesitas usar una herramienta, pon "instrumento" y deja "respuesta_final": null.
- "razonamiento" es obligatorio y debe ser conciso.
Instrumentos disponibles: bash, leer_archivo, escribir_archivo."#
    }

    /// Convierte el historial del paso en un mensaje de corrección para el modelo.
    pub fn mensaje_correccion(error: &str) -> String {
        format!(
            "Tu respuesta anterior no era un JSON válido según el esquema. \
             Error del parser: {error}. Vuelve a responder SOLO con el objeto JSON exacto."
        )
    }

    /// Intenta parsear un paso estructurado desde texto arbitrario del modelo.
    ///
    /// Estrategia de tolerancia:
    /// 1. Intentar parsear el texto completo como JSON.
    /// 2. Si falla, buscar el primer bloque ```json ... ```.
    /// 3. Si falla, buscar el primer '{' y último '}' del texto.
    pub fn parsear(texto: &str) -> Result<PasoEstructurado, String> {
        // 1) Texto completo
        if let Ok(paso) = serde_json::from_str::<PasoEstructurado>(texto.trim()) {
            return Ok(paso);
        }

        // 2) Bloque de código json
        let inicio = texto.find("```json").or_else(|| texto.find("```JSON"));
        if let Some(idx) = inicio {
            let resto = &texto[idx + "```json".len()..];
            if let Some(fin) = resto.find("```") {
                let candidato = resto[..fin].trim();
                if let Ok(paso) = serde_json::from_str::<PasoEstructurado>(candidato) {
                    return Ok(paso);
                }
            }
        }

        // 3) Primer '{' ... último '}'
        if let (Some(a), Some(b)) = (texto.find('{'), texto.rfind('}')) {
            if b > a {
                let candidato = &texto[a..=b];
                if let Ok(paso) = serde_json::from_str::<PasoEstructurado>(candidato) {
                    return Ok(paso);
                }
            }
        }

        Err(format!(
            "No se encontró un objeto JSON válido del esquema. Texto recibido (primeros 300 chars): {}",
            &texto.chars().take(300).collect::<String>()
        ))
    }

    /// Valida que el paso sea estructuralmente coherente.
    pub fn validar(paso: &PasoEstructurado) -> Result<(), String> {
        if paso.razonamiento.trim().is_empty() {
            return Err("El campo 'razonamiento' está vacío".into());
        }
        match (&paso.instrumento, &paso.respuesta_final) {
            (Some(_), Some(_)) => {
                Err("No puede haber 'instrumento' y 'respuesta_final' simultáneamente".into())
            }
            (None, None) => {
                Err("Debe haber 'instrumento' o 'respuesta_final', no ambos vacíos".into())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsea_json_plano() {
        let texto = r#"{"razonamiento":"pienso","instrumento":{"nombre":"bash","argumentos":{"comando":"ls"}},"respuesta_final":null}"#;
        let paso = ReglasJSON::parsear(texto).unwrap();
        assert_eq!(paso.razonamiento, "pienso");
        assert!(!paso.es_final());
        let inst = paso.instrumento.unwrap();
        assert_eq!(inst.nombre, "bash");
        assert_eq!(inst.argumento("comando").as_deref(), Some("ls"));
    }

    #[test]
    fn parsea_respuesta_final() {
        let texto = r#"{"razonamiento":"hecho","instrumento":null,"respuesta_final":"hola"}"#;
        let paso = ReglasJSON::parsear(texto).unwrap();
        assert!(paso.es_final());
        assert_eq!(paso.respuesta_final.as_deref(), Some("hola"));
    }

    #[test]
    fn tolera_bloque_json_markdown() {
        let texto = "Aquí va mi razonamiento:\n```json\n{\"razonamiento\":\"x\",\"instrumento\":null,\"respuesta_final\":\"ok\"}\n```\nFin.";
        let paso = ReglasJSON::parsear(texto).unwrap();
        assert_eq!(paso.respuesta_final.as_deref(), Some("ok"));
    }

    #[test]
    fn tolera_texto_circundante() {
        let texto = "Pensé: {\"razonamiento\":\"z\",\"instrumento\":null,\"respuesta_final\":\"listo\"} y eso es todo";
        let paso = ReglasJSON::parsear(texto).unwrap();
        assert_eq!(paso.respuesta_final.as_deref(), Some("listo"));
    }

    #[test]
    fn falla_con_texto_no_json() {
        assert!(ReglasJSON::parsear("esto no es json").is_err());
    }

    #[test]
    fn validacion_rechaza_ambos_campos() {
        let paso = PasoEstructurado {
            razonamiento: "r".into(),
            instrumento: Some(InstrumentoLlamado {
                nombre: "bash".into(),
                argumentos: serde_json::json!({}),
            }),
            respuesta_final: Some("f".into()),
        };
        assert!(ReglasJSON::validar(&paso).is_err());
    }

    #[test]
    fn validacion_rechaza_vacios() {
        let paso = PasoEstructurado {
            razonamiento: "r".into(),
            instrumento: None,
            respuesta_final: None,
        };
        assert!(ReglasJSON::validar(&paso).is_err());
    }

    #[test]
    fn validacion_acepta_instrumento_solo() {
        let paso = PasoEstructurado {
            razonamiento: "r".into(),
            instrumento: Some(InstrumentoLlamado {
                nombre: "bash".into(),
                argumentos: serde_json::json!({"comando": "ls"}),
            }),
            respuesta_final: None,
        };
        assert!(ReglasJSON::validar(&paso).is_ok());
    }
}
