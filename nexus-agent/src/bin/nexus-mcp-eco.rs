// ============================================================================
// NEXUS-AGENT · nexus-mcp-eco.rs — Servidor MCP stdio de prueba (eco)
// ============================================================================
// Este binario NO es el cerebro: es un servidor MCP mínimo que responde a
// `tools/list` y `tools/call` para verificar el adaptador cliente en tests de
// integración sin depender de `claws_mcp` (que exige la base de memoria y el
// Orquestador completo).
//
// Comportamiento:
//   - tools/call "eco"          → result { content:[{text:"eco-ok:<nombre>:<args>"}] }
//   - tools/call "fabricar_error" → result con isError:true
//   - tools/call "colgar"       → duerme 10 s (el cliente debe matarlo por timeout)
//   - Cualquier otro método     → error JSON-RPC -32601
//
// Lee una petición por línea desde stdin y responde por stdout. Al cerrarse
// stdin (EOF) el proceso termina — exactamente el contrato que usa el cliente.
// ============================================================================

use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() {
    let stdin = std::io::stdin();
    let mut linea = String::new();

    loop {
        linea.clear();
        let n = stdin.lock().read_line(&mut linea).unwrap_or(0);
        if n == 0 {
            // EOF: el cliente cerró stdin. Terminar.
            break;
        }

        let Ok(request) = serde_json::from_str::<Value>(&linea) else {
            continue;
        };
        let id = request.get("id").cloned();
        let metodo = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");

        let respuesta: Value = match metodo {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "serverInfo": { "name": "nexus-mcp-eco", "version": "0.1.0" }
                }
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "eco",
                            "description": "Herramienta eco de prueba para integración",
                            "inputSchema": { "type": "object", "properties": {} }
                        }
                    ]
                }
            }),
            "tools/call" => {
                let params = request.get("params").cloned().unwrap_or(Value::Null);
                let nombre = params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let argumentos = params.get("arguments").cloned().unwrap_or(Value::Null);

                match nombre {
                    "fabricar_error" => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": "error fabricado por el eco" }],
                            "isError": true
                        }
                    }),
                    "colgar" => {
                        // Duerme más que el timeout del cliente (1 s) para que
                        // el test verifique que el cliente mata el subproceso.
                        std::thread::sleep(std::time::Duration::from_secs(10));
                        json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": {
                                "content": [{ "type": "text", "text": "nunca debería llegar" }],
                                "isError": false
                            }
                        })
                    }
                    _ => json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{
                                "type": "text",
                                "text": format!("eco-ok:{nombre}:{argumentos}")
                            }],
                            "isError": false
                        }
                    }),
                }
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("método no soportado: {metodo}")
                }
            }),
        };

        let mut salida = serde_json::to_string(&respuesta).unwrap();
        salida.push('\n');
        let mut stdout = std::io::stdout();
        let _ = stdout.write_all(salida.as_bytes());
        let _ = stdout.flush();
    }
}
