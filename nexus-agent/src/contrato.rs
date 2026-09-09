// ============================================================================
// NEXUS-AGENT · contrato.rs — Capa de abstracción de proveedores LLM
// ============================================================================
// El motor agéntico se comunica con una interfaz genérica (ContratoLlm).
// Cada proveedor concreto (DeepSeek, OpenAI-compatible, Ollama local)
// implementa ese contrato con su propio transporte HTTP.
//
// Principio de diseño: el agente NUNCA conoce el proveedor concreto;
// solo conoce el contrato. Cambiar de proveedor es una decisión de
// configuración, no de código.
// ============================================================================

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::collections::HashMap;

// ----------------------------------------------------------------------------
// Tipos de mensaje (wire protocol neutral al proveedor)
// ----------------------------------------------------------------------------

/// Rol de cada mensaje en el historial de la conversación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RolMensaje {
    Sistema,
    Usuario,
    Asistente,
    Instrumento,
}

impl RolMensaje {
    /// Etiqueta textual usada en el wire protocol de cada proveedor.
    pub fn etiqueta(&self) -> &'static str {
        match self {
            RolMensaje::Sistema => "system",
            RolMensaje::Usuario => "user",
            RolMensaje::Asistente => "assistant",
            RolMensaje::Instrumento => "tool",
        }
    }
}

/// Entrada del historial. La posición 0 está reservada a la instrucción maestra.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MensajeHistoria {
    pub rol: RolMensaje,
    pub contenido: String,
}

impl MensajeHistoria {
    pub fn sistema(contenido: impl Into<String>) -> Self {
        Self { rol: RolMensaje::Sistema, contenido: contenido.into() }
    }
    pub fn usuario(contenido: impl Into<String>) -> Self {
        Self { rol: RolMensaje::Usuario, contenido: contenido.into() }
    }
    pub fn asistente(contenido: impl Into<String>) -> Self {
        Self { rol: RolMensaje::Asistente, contenido: contenido.into() }
    }
    pub fn instrumento(contenido: impl Into<String>) -> Self {
        Self { rol: RolMensaje::Instrumento, contenido: contenido.into() }
    }
}

/// Respuesta cruda de un proveedor LLM.
#[derive(Debug, Clone)]
pub struct RespuestaLlm {
    pub texto: String,
    pub finalizado_por: String,
    pub modelo: String,
}

/// Variable de entorno para el proveedor (clave API, base URL, modelo por defecto).
#[derive(Debug, Clone)]
pub struct VariableEntorno {
    pub clave: String,
    pub valor: String,
}

/// Salto de razonamiento interno del agente (pensamiento).
#[derive(Debug, Clone)]
pub struct SaltoAgente {
    pub razonamiento: String,
    pub decisiones: Vec<String>,
}

// ----------------------------------------------------------------------------
// El contrato: la única superficie que el motor agéntico conoce
// ----------------------------------------------------------------------------

/// Contrato mínimo que todo proveedor LLM debe cumplir para alimentar al agente.
#[async_trait]
pub trait ContratoLlm: Send + Sync {
    /// Nombre del proveedor (para diagnóstico y trazas).
    fn nombre(&self) -> &'static str;

    /// Envía el historial completo (con la instrucción maestra ya en [0])
    /// y devuelve la respuesta cruda del modelo.
    async fn conversar(&self, historial: &[MensajeHistoria]) -> Result<RespuestaLlm>;

    /// Reinicia la sesión subyacente si el proveedor la mantiene.
    async fn reiniciar_sesion(&self) -> Result<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Modelo de configuración compartido
// ----------------------------------------------------------------------------

/// Identidad de un modelo: proveedor + modelo + URL base + clave.
#[derive(Debug, Clone)]
pub struct ModeloCliente {
    pub proveedor: String,
    pub modelo: String,
    pub url_base: String,
    pub clave_api: Option<String>,
    pub extras: HashMap<String, String>,
}

impl ModeloCliente {
    /// Crea un modelo desde variables de entorno (VARIABLE, URL, MODELO).
    pub fn desde_variables(env: &[VariableEntorno], proveedor: &str, url: &str, modelo: &str) -> Self {
        let mut mapa = HashMap::new();
        for v in env {
            mapa.insert(v.clave.clone(), v.valor.clone());
        }
        Self {
            proveedor: proveedor.to_string(),
            modelo: mapa
                .get(modelo)
                .cloned()
                .unwrap_or_else(|| modelo.to_string()),
            url_base: mapa.get(url).cloned().unwrap_or_default(),
            clave_api: Self::buscar_clave_api(&mapa),
            extras: HashMap::new(),
        }
    }

    /// Busca la clave API con tolerancia de prefijos:
    ///   - prioriza `API_KEY` genérica
    ///   - acepta `{PREFIJO}_API_KEY` (p. ej. DEEPSEEK_API_KEY, OPENAI_API_KEY)
    fn buscar_clave_api(mapa: &HashMap<String, String>) -> Option<String> {
        if let Some(v) = mapa.get("API_KEY") {
            return Some(v.clone());
        }
        for (clave, valor) in mapa {
            let mayus = clave.to_uppercase();
            if mayus.ends_with("_API_KEY") {
                return Some(valor.clone());
            }
        }
        None
    }

    pub fn con_extra(mut self, clave: &str, valor: &str) -> Self {
        self.extras.insert(clave.to_string(), valor.to_string());
        self
    }
}

// ----------------------------------------------------------------------------
// Transporte HTTP compartido (JSON estándar del wire protocol)
// ----------------------------------------------------------------------------

/// Estructura de respuesta JSON estándar de un endpoint estilo chat-completions.
#[derive(Debug, Deserialize)]
struct RespuestaChatCompletions {
    choices: Vec<OpcionChat>,
    #[serde(default)]
    model: String,
}

#[derive(Debug, Deserialize)]
struct OpcionChat {
    message: MensajeRespuesta,
    #[serde(default)]
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct MensajeRespuesta {
    #[serde(default)]
    content: String,
}

/// Cliente HTTP compartido para todos los proveedores de tipo OpenAI.
#[derive(Debug, Clone)]
struct TransporteChatCompletions {
    http: reqwest::Client,
    url: String,
    clave: Option<String>,
    modelo: String,
}

impl TransporteChatCompletions {
    fn nuevo(modelo: &ModeloCliente) -> Result<Self> {
        let url = if modelo.url_base.ends_with('/') {
            format!("{}chat/completions", modelo.url_base)
        } else {
            format!("{}/chat/completions", modelo.url_base)
        };
        Ok(Self {
            http: reqwest::Client::new(),
            url,
            clave: modelo.clave_api.clone(),
            modelo: modelo.modelo.clone(),
        })
    }

    async fn enviar(&self, historial: &[MensajeHistoria]) -> Result<RespuestaLlm> {
        let mut cuerpo = serde_json::json!({
            "model": self.modelo,
            "messages": historial
                .iter()
                .map(|m| serde_json::json!({
                    "role": m.rol.etiqueta(),
                    "content": m.contenido,
                }))
                .collect::<Vec<_>>(),
            "temperature": 0.7,
        });

        // Si el proveedor soporta respuestas en JSON estricto, lo pedimos.
        if let Some(ext) = self.extras_json() {
            cuerpo["response_format"] = ext;
        }

        let mut peticion = self.http.post(&self.url).json(&cuerpo);
        if let Some(clave) = &self.clave {
            peticion = peticion.bearer_auth(clave);
        }

        let respuesta = peticion
            .send()
            .await
            .context("No se pudo contactar al proveedor LLM")?;

        let estado_http = respuesta.status();
        if !estado_http.is_success() {
            let texto = respuesta.text().await.unwrap_or_default();
            return Err(anyhow!("Proveedor LLM respondió HTTP {}: {}", estado_http, texto));
        }

        let cuerpo_parseado: RespuestaChatCompletions = respuesta
            .json()
            .await
            .context("Respuesta del proveedor no es JSON válido")?;

        let opcion = cuerpo_parseado
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("El proveedor no devolvió ninguna opción"))?;

        Ok(RespuestaLlm {
            texto: opcion.message.content,
            finalizado_por: opcion.finish_reason,
            modelo: if cuerpo_parseado.model.is_empty() {
                self.modelo.clone()
            } else {
                cuerpo_parseado.model
            },
        })
    }

    /// Extras serializados para el cuerpo (solo si el proveedor lo soporta).
    fn extras_json(&self) -> Option<serde_json::Value> {
        None // por defecto sin response_format; los proveedores concretos lo activan
    }
}

// ----------------------------------------------------------------------------
// Proveedor 1: DeepSeek (endpoint https://api.deepseek.com)
// ----------------------------------------------------------------------------

pub const DEEPSEEK_URL: &str = "https://api.deepseek.com";
pub const DEEPSEEK_MODELO: &str = "deepseek-chat";

/// Cliente DeepSeek: transporte OpenAI-compatible hacia api.deepseek.com.
#[derive(Debug, Clone)]
pub struct DeepSeekCliente {
    transporte: TransporteChatCompletions,
}

impl DeepSeekCliente {
    pub fn nuevo(clave_api: &str) -> Result<Self> {
        let modelo = ModeloCliente {
            proveedor: "deepseek".into(),
            modelo: DEEPSEEK_MODELO.into(),
            url_base: DEEPSEEK_URL.into(),
            clave_api: Some(clave_api.to_string()),
            extras: HashMap::new(),
        };
        Ok(Self { transporte: TransporteChatCompletions::nuevo(&modelo)? })
    }

    pub fn con_modelo(clave_api: &str, modelo: &str) -> Result<Self> {
        let mut base = Self::nuevo(clave_api)?;
        base.transporte.modelo = modelo.to_string();
        Ok(base)
    }
}

#[async_trait]
impl ContratoLlm for DeepSeekCliente {
    fn nombre(&self) -> &'static str {
        "deepseek"
    }

    async fn conversar(&self, historial: &[MensajeHistoria]) -> Result<RespuestaLlm> {
        self.transporte.enviar(historial).await
    }
}

// ----------------------------------------------------------------------------
// Proveedor 2: OpenAI-compatible genérico (cualquier endpoint /chat/completions)
// ----------------------------------------------------------------------------

/// Cliente genérico para cualquier API compatible con OpenAI chat completions.
/// Permite cambiar de proveedor en caliente sin tocar el motor agéntico.
#[derive(Debug, Clone)]
pub struct ModeloClienteGenerico {
    transporte: TransporteChatCompletions,
}

impl ModeloClienteGenerico {
    pub fn nuevo(config: &ModeloCliente) -> Result<Self> {
        Ok(Self { transporte: TransporteChatCompletions::nuevo(config)? })
    }
}

#[async_trait]
impl ContratoLlm for ModeloClienteGenerico {
    fn nombre(&self) -> &'static str {
        "openai-compatible"
    }

    async fn conversar(&self, historial: &[MensajeHistoria]) -> Result<RespuestaLlm> {
        self.transporte.enviar(historial).await
    }
}

// ----------------------------------------------------------------------------
// Proveedor 3: Ollama local (http://localhost:11434)
// ----------------------------------------------------------------------------

pub const OLLAMA_URL: &str = "http://localhost:11434";
pub const OLLAMA_MODELO: &str = "llama3";

/// Cliente Ollama: inferencia 100% local, sin clave API.
#[derive(Debug, Clone)]
pub struct OllamaCliente {
    http: reqwest::Client,
    url: String,
    modelo: String,
}

impl OllamaCliente {
    pub fn nuevo(modelo: &str) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::new(),
            url: format!("{}/api/chat", OLLAMA_URL),
            modelo: modelo.to_string(),
        })
    }

    pub fn con_url(modelo: &str, url_base: &str) -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::new(),
            url: format!("{}/api/chat", url_base),
            modelo: modelo.to_string(),
        })
    }
}

#[async_trait]
impl ContratoLlm for OllamaCliente {
    fn nombre(&self) -> &'static str {
        "ollama-local"
    }

    async fn conversar(&self, historial: &[MensajeHistoria]) -> Result<RespuestaLlm> {
        // Ollama usa su propio esquema: messages con roles "system"/"user"/"assistant"
        let cuerpo = serde_json::json!({
            "model": self.modelo,
            "messages": historial
                .iter()
                .map(|m| {
                    let rol = match m.rol {
                        RolMensaje::Sistema => "system",
                        RolMensaje::Usuario => "user",
                        RolMensaje::Asistente => "assistant",
                        RolMensaje::Instrumento => "tool",
                    };
                    serde_json::json!({ "role": rol, "content": m.contenido })
                })
                .collect::<Vec<_>>(),
            "stream": false,
        });

        let respuesta = self
            .http
            .post(&self.url)
            .json(&cuerpo)
            .send()
            .await
            .context("No se pudo contactar a Ollama local")?;

        let estado_http = respuesta.status();
        if !estado_http.is_success() {
            let texto = respuesta.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama respondió HTTP {}: {}", estado_http, texto));
        }

        #[derive(Deserialize)]
        struct RespuestaOllama {
            message: MensajeRespuesta,
            #[serde(default)]
            model: String,
        }

        let parseada: RespuestaOllama = respuesta
            .json()
            .await
            .context("Respuesta de Ollama no es JSON válido")?;

        Ok(RespuestaLlm {
            texto: parseada.message.content,
            finalizado_por: "stop".into(),
            modelo: if parseada.model.is_empty() {
                self.modelo.clone()
            } else {
                parseada.model
            },
        })
    }
}

// ----------------------------------------------------------------------------
// Tests del contrato
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MensajeHistoria;

    #[test]
    fn transporte_construye_url_chat_completions() {
        let modelo = ModeloCliente {
            proveedor: "x".into(),
            modelo: "m".into(),
            url_base: "https://api.example.com".into(),
            clave_api: Some("k".into()),
            extras: HashMap::new(),
        };
        let t = TransporteChatCompletions::nuevo(&modelo).unwrap();
        assert_eq!(t.url, "https://api.example.com/chat/completions");
        assert_eq!(t.modelo, "m");
    }

    #[test]
    fn transporte_sin_doble_barra_en_url() {
        let modelo = ModeloCliente {
            proveedor: "x".into(),
            modelo: "m".into(),
            url_base: "https://api.example.com/".into(),
            clave_api: None,
            extras: HashMap::new(),
        };
        let t = TransporteChatCompletions::nuevo(&modelo).unwrap();
        assert_eq!(t.url, "https://api.example.com/chat/completions");
    }

    #[test]
    fn etiquetas_de_rol_son_correctas() {
        assert_eq!(RolMensaje::Sistema.etiqueta(), "system");
        assert_eq!(RolMensaje::Usuario.etiqueta(), "user");
        assert_eq!(RolMensaje::Asistente.etiqueta(), "assistant");
        assert_eq!(RolMensaje::Instrumento.etiqueta(), "tool");
    }

    #[test]
    fn mensaje_historia_constructores() {
        let s = MensajeHistoria::sistema("instrucción");
        let u = MensajeHistoria::usuario("hola");
        let a = MensajeHistoria::asistente("adiós");
        assert_eq!(s.rol, RolMensaje::Sistema);
        assert_eq!(u.rol, RolMensaje::Usuario);
        assert_eq!(a.rol, RolMensaje::Asistente);
        assert_eq!(s.contenido, "instrucción");
    }

    #[test]
    fn modelo_desde_variables_lee_clave() {
        let env = vec![
            VariableEntorno { clave: "DEEPSEEK_API_KEY".into(), valor: "abc".into() },
            VariableEntorno { clave: "DEEPSEEK_URL".into(), valor: "https://api.deepseek.com".into() },
            VariableEntorno { clave: "DEEPSEEK_MODEL".into(), valor: "deepseek-reasoner".into() },
        ];
        let modelo = ModeloCliente::desde_variables(
            &env,
            "deepseek",
            "DEEPSEEK_URL",
            "DEEPSEEK_MODEL",
        );
        assert_eq!(modelo.clave_api, Some("abc".to_string()));
        assert_eq!(modelo.modelo, "deepseek-reasoner");
        assert_eq!(modelo.url_base, "https://api.deepseek.com");
    }
}
