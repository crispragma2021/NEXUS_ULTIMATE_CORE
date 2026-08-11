// ============================================================================
// NEXUS-AGENT · mcp_cliente.rs — Adaptador MCP stdio hacia el cerebro NEXUS
// ============================================================================
// Conecta al agente con la fachada `claws_mcp` (nexus-claws-mcp v3.2.0):
// el servidor MCP stdio que expone el cerebro completo del Orquestador
// (nexus_pensar, consultar_memoria, listar_agentes, ejecutar_workflow,
// leer_archivo, escribir_archivo, órganos sensoriales, etc.).
//
// Por qué un subproceso por llamada (y no un proceso persistente):
//   - MCP stdio no exige reutilizar el proceso; cada cliente lanza su propia
//     instancia del servidor (igual que hace RooCode / Antigravity con el
//     mismo claws_mcp).
//   - Al cerrar stdin tras el request se produce EOF → el servidor termina solo.
//   - Aislamiento total: un servidor colgado no afecta al agente ni a otros
//     clientes (la frontera es el proceso, no un mutex compartido).
//
// Protocolo: JSON-RPC 2.0 sobre stdio (una petición por línea).
//   → stdin:  {"jsonrpc":"2.0","method":"tools/call","params":{...},"id":N}
//   ← stdout: {"jsonrpc":"2.0","id":N,"result":{...}}
//
// Tolerancia: si el servidor mezcla logs en stdout (líneas no JSON), se
// descartan; solo se acepta la respuesta cuyo "id" coincide con la llamada.
// ============================================================================

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::ChildStdout;

/// Configuración del cliente MCP.
#[derive(Debug, Clone)]
pub struct ConfigClienteMcp {
    /// Ruta o nombre del binario servidor (por defecto "claws_mcp").
    pub binario: String,
    /// Argumentos fijos pasados al servidor.
    pub args: Vec<String>,
    /// Timeout total por llamada (segundos).
    pub timeout_seg: u64,
}

impl Default for ConfigClienteMcp {
    fn default() -> Self {
        Self {
            binario: "claws_mcp".into(),
            args: Vec::new(),
            timeout_seg: 60,
        }
    }
}

/// Cliente MCP stdio: habla JSON-RPC 2.0 con el servidor NEXUS (claws_mcp).
///
/// Cada llamada lanza un subproceso aislado del servidor, escribe el request,
/// cierra stdin (EOF) y lee la respuesta con timeout. Si el servidor se
/// cuelga, el timeout lo mata; el agente jamás se bloquea.
#[derive(Debug, Clone)]
pub struct ClienteMcp {
    config: ConfigClienteMcp,
    /// Contador atómico compartido: cada llamada usa un id único, incluso si
    /// el cliente se clona entre hilos/tareas.
    proximo_id: Arc<AtomicU64>,
}

impl ClienteMcp {
    /// Cliente con el binario indicado (ruta o nombre en PATH) y valores por
    /// defecto: timeout de 60 s, sin argumentos extra.
    pub fn nuevo(binario: &str) -> Self {
        Self {
            config: ConfigClienteMcp {
                binario: binario.to_string(),
                ..Default::default()
            },
            proximo_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn con_config(config: ConfigClienteMcp) -> Self {
        Self {
            config,
            proximo_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Ajusta el timeout de espera de respuesta (segundos).
    pub fn con_timeout(mut self, segundos: u64) -> Self {
        self.config.timeout_seg = segundos;
        self
    }

    /// Lista las herramientas que expone el servidor (tools/list).
    pub async fn listar_herramientas(&self) -> Result<Value> {
        self.enviar("tools/list", json!({})).await
    }

    /// Invoca una herramienta del cerebro NEXUS (tools/call).
    ///
    /// `argumentos` es el objeto JSON con los parámetros de la herramienta.
    /// Devuelve el objeto `result` completo: `{ content, isError }`.
    pub async fn llamar(&self, herramienta: &str, argumentos: Value) -> Result<Value> {
        let params = json!({ "name": herramienta, "arguments": argumentos });
        let respuesta = self.enviar("tools/call", params).await?;
        respuesta
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow!("El servidor MCP no devolvió 'result' en la respuesta"))
    }

    /// Extrae el texto plano de un resultado MCP (content[].text).
    ///
    /// Si no hay `content`, devuelve la representación JSON del resultado.
    pub fn texto(resultado: &Value) -> String {
        let mut out = String::new();
        if let Some(content) = resultado.get("content").and_then(|c| c.as_array()) {
            for item in content {
                if let Some(texto) = item.get("text").and_then(|t| t.as_str()) {
                    out.push_str(texto);
                    out.push('\n');
                }
            }
        }
        let recortado = out.trim_end().to_string();
        if recortado.is_empty() {
            resultado.to_string()
        } else {
            recortado
        }
    }

    /// ¿El resultado marcó error (isError == true)?
    pub fn es_error(resultado: &Value) -> bool {
        resultado
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Envía un request JSON-RPC y espera la respuesta con el id buscado.
    async fn enviar(&self, metodo: &str, params: Value) -> Result<Value> {
        let id = self.proximo_id.fetch_add(1, Ordering::SeqCst);

        let mut hijo = tokio::process::Command::new(&self.config.binario)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| {
                format!(
                    "No se pudo lanzar el servidor MCP '{}'. \
                     ¿Está compilado y en el PATH? Usa NEXUS_CLAWS_MCP para una ruta explícita.",
                    self.config.binario
                )
            })?;

        let mut stdin = hijo
            .stdin
            .take()
            .context("No se pudo tomar stdin del servidor MCP")?;
        let stdout = hijo
            .stdout
            .take()
            .context("No se pudo tomar stdout del servidor MCP")?;
        let stderr = hijo
            .stderr
            .take()
            .context("No se pudo tomar stderr del servidor MCP")?;

        // 1. Escribir el request y cerrar stdin (EOF → el servidor termina solo)
        let request = json!({ "jsonrpc": "2.0", "method": metodo, "params": params, "id": id });
        let mut linea = serde_json::to_string(&request)?;
        linea.push('\n');
        stdin.write_all(linea.as_bytes()).await?;
        stdin.flush().await?;
        drop(stdin);

        // 2. Drenar stderr en una tarea (evita bloqueo si el servidor loguea)
        let tarea_stderr = tokio::spawn(async move {
            let mut lector = BufReader::new(stderr);
            let mut buf = String::new();
            while lector.read_line(&mut buf).await.unwrap_or(0) > 0 {
                tracing::debug!(stderr = buf.trim(), "salida del servidor MCP");
                buf.clear();
            }
        });

        // 3. Leer stdout línea a línea con timeout, buscando el id de la llamada
        let espera = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout_seg),
            Self::leer_respuesta(stdout, id),
        )
        .await;

        // 4. Terminar el proceso (ya respondió o expiró) y recolectar
        let _ = hijo.kill().await;
        let _ = hijo.wait().await;
        let _ = tarea_stderr.await;

        match espera {
            Ok(Ok(respuesta)) => Ok(respuesta),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!(
                "El servidor MCP no respondió en {}s (método '{metodo}')",
                self.config.timeout_seg
            )),
        }
    }

    /// Lee líneas de stdout hasta encontrar la respuesta con `id` buscado.
    ///
    /// Las líneas que no sean JSON (logs mezclados) se descartan; las que
    /// sean JSON con otro id (respuesta a otra llamada) se ignoran.
    async fn leer_respuesta(stdout: ChildStdout, id: u64) -> Result<Value> {
        let mut lector = BufReader::new(stdout);
        let mut linea = String::new();
        loop {
            linea.clear();
            let n = lector.read_line(&mut linea).await?;
            if n == 0 {
                return Err(anyhow!(
                    "El servidor MCP cerró stdout sin responder (id {id})"
                ));
            }
            let Ok(valor) = serde_json::from_str::<Value>(&linea) else {
                tracing::debug!(linea = linea.trim(), "línea no JSON del servidor MCP");
                continue;
            };
            if valor.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(error) = valor.get("error") {
                    let msg = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("error desconocido");
                    return Err(anyhow!("Error JSON-RPC del servidor MCP: {msg}"));
                }
                return Ok(valor);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texto_extrae_content_text() {
        let resultado = json!({
            "content": [
                { "type": "text", "text": "línea uno" },
                { "type": "text", "text": "línea dos" }
            ],
            "isError": false
        });
        assert_eq!(ClienteMcp::texto(&resultado), "línea uno\nlínea dos");
    }

    #[test]
    fn texto_devuelve_json_si_no_hay_content() {
        let resultado = json!({ "otro": "campo" });
        assert!(ClienteMcp::texto(&resultado).contains("campo"));
    }

    #[test]
    fn es_error_detecta_flag() {
        assert!(ClienteMcp::es_error(&json!({ "isError": true })));
        assert!(!ClienteMcp::es_error(&json!({ "isError": false })));
        assert!(!ClienteMcp::es_error(&json!({ "exitoso": true })));
    }
}
