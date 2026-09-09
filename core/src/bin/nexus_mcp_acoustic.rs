use anyhow::Result;
use nexus_ultimate_core::cerebro::organos::nexus_acoustic::OidoDigital;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    // 🎧 NEXUS ACOUSTIC MCP - El Oído del Arquitecto
    while handle.read_line(&mut line)? > 0 {
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                line.clear();
                continue;
            }
        };

        let id = request["id"].as_i64().unwrap_or(0);
        let method = request["method"].as_str().unwrap_or("");

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "capabilities": {
                        "resources": {},
                        "tools": {
                            "list": [
                                {
                                    "name": "listen_bunker",
                                    "description": "Escuchar audio del ambiente del búnker durante N segundos",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "seconds": { "type": "integer", "default": 5 }
                                        }
                                    }
                                },
                                {
                                    "name": "speak_to_architect",
                                    "description": "Reproducir un archivo de audio WAV/MP3 para el Arquitecto",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "path": { "type": "string" }
                                        },
                                        "required": ["path"]
                                    }
                                }
                            ]
                        }
                    },
                    "serverInfo": {
                        "name": "nexus-acoustic-mcp",
                        "version": "0.1.0"
                    }
                }
            }),
            "tools/call" => {
                let name = request["params"]["name"].as_str().unwrap_or("");
                let params = &request["params"]["arguments"];

                match name {
                    "listen_bunker" => {
                        let sec = params["seconds"].as_u64().unwrap_or(5) as u32;
                        let path = "/opt/NEXUS_ULTIMATE_CORE/data/audio/last_ear.wav";
                        let _ = std::fs::create_dir_all("/opt/NEXUS_ULTIMATE_CORE/data/audio");
                        OidoDigital::capturar_audio(sec, path)?;
                        json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": format!("Escuchando... Archivo guardado en {}", path) }] } })
                    }
                    "speak_to_architect" => {
                        let path = params["path"].as_str().unwrap_or("");
                        OidoDigital::hablar(path)?;
                        json!({ "jsonrpc": "2.0", "id": id, "result": { "content": [{ "type": "text", "text": "Mensaje enviado a la Voz de NEXUS" }] } })
                    }
                    _ => {
                        json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "Método no encontrado" } })
                    }
                }
            }
            _ => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
        };

        println!("{}", response);
        io::stdout().flush()?;
        line.clear();
    }

    Ok(())
}
