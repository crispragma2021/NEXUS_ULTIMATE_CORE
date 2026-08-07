// ============================================================================
// 🔱 NEXUS-TOOLS: Binario de Ejecución Nativa de Latencia Cero (Transmutación)
// Consolida las herramientas del sistema, ejecución paralela y red en Rust puro.
// ============================================================================

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Si se invoca como servidor MCP
    if args.len() > 1 && args[1] == "--mcp" {
        run_mcp_server().await?;
        return Ok(());
    }

    // Modo CLI directo
    if args.len() < 3 {
        eprintln!("Uso CLI: nexus_tools <modulo> <comando/argumento>");
        eprintln!("Modo MCP: nexus_tools --mcp");
        std::process::exit(1);
    }

    let modulo = &args[1];
    match modulo.as_str() {
        "sys" => handle_sys_cli(&args[2..]).await?,
        "parallel" => handle_parallel_cli(&args[2..]).await?,
        _ => {
            eprintln!("Módulo desconocido: {}", modulo);
            std::process::exit(1);
        }
    }

    Ok(())
}

// ─── MÓDULO 1: SISTEMA Y PUERTOS NATIVOS ──────────────────────────────────────
async fn handle_sys_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let action = &args[0];
    if action == "free_port" && args.len() > 1 {
        let port: u16 = args[1].parse()?;
        liberar_puerto(port).await?;
    } else if action == "check_health" {
        realizar_check_salud().await?;
    }
    Ok(())
}

async fn liberar_puerto(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Escaneando puerto :{}...", port);
    // Ejecutar lsof para encontrar PIDs de forma nativa
    let output = Command::new("lsof")
        .args(&["-t", "-i", &format!(":{}", port)])
        .output();

    if let Ok(out) = output {
        let pids_str = String::from_utf8_lossy(&out.stdout);
        let pids: Vec<&str> = pids_str.lines().filter(|s| !s.is_empty()).collect();

        if pids.is_empty() {
            println!("✅ El puerto :{} ya está libre.", port);
            return Ok(());
        }

        for pid in pids {
            println!("💥 Eliminando proceso zombie PID: {}", pid);
            let _ = Command::new("kill").args(&["-9", pid]).status();
        }
        println!("✅ Puerto :{} completamente liberado.", port);
    } else {
        println!("El puerto :{} está libre o no se detectó lsof.", port);
    }
    Ok(())
}

async fn realizar_check_salud() -> Result<(), Box<dyn std::error::Error>> {
    let ports = vec![
        (1420, "Santuario UI (Chat)"),
        (5173, "HUD Chat"),
        (42220, "Portal de Trading"),
        (42210, "Core API Rust backend"),
    ];

    println!("📊 REPORT DE SALUD DE RED NATIVO:");
    for (port, name) in ports {
        let status = if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            "🟢 OK"
        } else {
            "🔴 CAÍDO"
        };
        println!("  [:{}] {} - {}", port, status, name);
    }
    Ok(())
}

// ─── MÓDULO 2: EJECUTOR MULTIHILO PARALELO NATIVO ─────────────────────────────
async fn handle_parallel_cli(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();
    let start = Instant::now();

    for cmd in args {
        let cmd_clone = cmd.clone();
        tasks.push(tokio::spawn(async move {
            let inicio_hilo = Instant::now();
            let out = Command::new("bash").args(&["-c", &cmd_clone]).output();

            let status = match out {
                Ok(output) => {
                    let text = String::from_utf8_lossy(&output.stdout);
                    let err = String::from_utf8_lossy(&output.stderr);
                    format!(
                        "✅ SUCCESS ({} ms)\n[STDOUT]\n{}\n[STDERR]\n{}",
                        inicio_hilo.elapsed().as_millis(),
                        text.trim(),
                        err.trim()
                    )
                }
                Err(e) => format!("❌ FAILED: {}", e),
            };
            (cmd_clone, status)
        }));
    }

    for task in tasks {
        if let Ok((cmd, result)) = task.await {
            println!("🛠️ Comando: {}\n{}", cmd, result);
            println!("=======================================================");
        }
    }
    println!(
        "⏱️ Tiempo total de ejecución paralela: {} ms",
        start.elapsed().as_millis()
    );
    Ok(())
}

// ─── MÓDULO 3: MCP PROTOCOL SERVER ────────────────────────────────────────────
async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock().lines();

    while let Some(Ok(line)) = reader.next() {
        if line.trim().is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = &msg["id"];
        let method = msg["method"].as_str().unwrap_or("");
        let params = &msg["params"];

        match method {
            "initialize" => {
                send_json(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": {} },
                        "serverInfo": { "name": "nexus-tools-native", "version": "1.0.0" }
                    }
                }));
            }
            "tools/list" => {
                send_json(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "native_free_port",
                                "description": "Libera un puerto de red en milisegundos matando de forma nativa los procesos que lo ocupan.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "port": { "type": "number", "description": "Puerto a liberar" }
                                    },
                                    "required": ["port"]
                                }
                            },
                            {
                                "name": "native_parallel_run",
                                "description": "Ejecuta múltiples comandos de terminal en paralelo con hilos nativos del sistema. Retorna salida consolidada.",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "commands": {
                                            "type": "array",
                                            "items": { "type": "string" },
                                            "description": "Comandos a correr en paralelo"
                                        }
                                    },
                                    "required": ["commands"]
                                }
                            },
                            {
                                "name": "native_check_health",
                                "description": "Diagnóstico de puertos de red locales de alta velocidad.",
                                "inputSchema": { "type": "object", "properties": {} }
                            }
                        ]
                    }
                }));
            }
            "tools/call" => {
                let tool_name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];
                let result = handle_tool_call(tool_name, args).await;

                send_json(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [
                            { "type": "text", "text": result }
                        ]
                    }
                }));
            }
            _ => {
                if id.is_number() || id.is_string() {
                    send_json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Método no encontrado" }
                    }));
                }
            }
        }
    }
    Ok(())
}

async fn handle_tool_call(name: &str, args: &Value) -> String {
    match name {
        "native_free_port" => {
            let port = args["port"].as_u64().unwrap_or(0) as u16;
            match liberar_puerto(port).await {
                Ok(_) => format!("✅ Puerto :{} liberado exitosamente.", port),
                Err(e) => format!("❌ Error al liberar puerto: {}", e),
            }
        }
        "native_parallel_run" => {
            let cmds_val = args["commands"].as_array();
            if let Some(cmds) = cmds_val {
                let mut tasks = Vec::new();
                for cmd_val in cmds {
                    if let Some(cmd) = cmd_val.as_str() {
                        let cmd_str = cmd.to_string();
                        tasks.push(tokio::spawn(async move {
                            let out = Command::new("bash").args(&["-c", &cmd_str]).output();
                            match out {
                                Ok(o) => {
                                    let stdout = String::from_utf8_lossy(&o.stdout);
                                    let stderr = String::from_utf8_lossy(&o.stderr);
                                    format!(
                                        "── Comando: {} ──\n[STDOUT]\n{}\n[STDERR]\n{}",
                                        cmd_str,
                                        stdout.trim(),
                                        stderr.trim()
                                    )
                                }
                                Err(e) => format!("❌ Error: {}", e),
                            }
                        }));
                    }
                }
                let mut output = String::new();
                for task in tasks {
                    if let Ok(res) = task.await {
                        output.push_str(&res);
                        output.push_str("\n\n");
                    }
                }
                output
            } else {
                "❌ Error: Argumento de comandos inválido.".to_string()
            }
        }
        "native_check_health" => {
            let ports = vec![
                (1420, "Santuario UI (Chat)"),
                (5173, "HUD Chat"),
                (42220, "Portal de Trading"),
                (42210, "Core API Rust backend"),
            ];
            let mut report = String::from("📊 DIAGNÓSTICO DE RED DE LATENCIA CERO:\n");
            for (port, name) in ports {
                let status = if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
                    .await
                    .is_ok()
                {
                    "🟢 ONLINE"
                } else {
                    "🔴 OFFLINE"
                };
                report.push_str(&format!("  [:{}] {} - {}\n", port, status, name));
            }
            report
        }
        _ => format!("Tool desconocida: {}", name),
    }
}

fn send_json(val: Value) {
    let json_str = serde_json::to_string(&val).unwrap();
    println!("{}", json_str);
    io::stdout().flush().unwrap();
}
