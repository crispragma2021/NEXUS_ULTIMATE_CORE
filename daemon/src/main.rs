use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

fn get_socket_path() -> PathBuf {
    if let Ok(custom) = env::var("NEXUS_SOCKET_PATH") {
        return PathBuf::from(custom);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".nexus_host.sock")
}

#[derive(Deserialize, Debug)]
#[serde(tag = "action", content = "payload")]
enum ActionRequest {
    Tap { x: u32, y: u32 },
    Type { text: String },
    Screenshot,
    Exec { command: String },
    GetScreenSize,
}

#[derive(Serialize)]
struct ActionResponse {
    ok: bool,
    result: String,
}

fn run_with_shizuku(cmd_str: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("rish -c '{}'", cmd_str))
        .output()
        .map_err(|e| format!("Failed to exec: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn handle_client(mut stream: UnixStream) {
    let reader = BufReader::new(stream.try_clone().expect("Cannot clone stream"));
    for line in reader.lines() {
        if let Ok(req_str) = line {
            let res = match serde_json::from_str::<ActionRequest>(&req_str) {
                Ok(ActionRequest::Tap { x, y }) => {
                    let cmd = format!("input tap {} {}", x, y);
                    match run_with_shizuku(&cmd) {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::Type { text }) => {
                    let cmd = format!("input text '{}'", text.replace("'", "'\\''"));
                    match run_with_shizuku(&cmd) {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::Screenshot) => {
                    let cmd = "screencap -p /sdcard/Download/nexus_shot.png";
                    match run_with_shizuku(cmd) {
                        Ok(_) => ActionResponse { ok: true, result: "/sdcard/Download/nexus_shot.png".to_string() },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::Exec { command }) => {
                    match run_with_shizuku(&command) {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Ok(ActionRequest::GetScreenSize) => {
                    match run_with_shizuku("wm size") {
                        Ok(r) => ActionResponse { ok: true, result: r },
                        Err(e) => ActionResponse { ok: false, result: e },
                    }
                }
                Err(e) => ActionResponse {
                    ok: false,
                    result: format!("JSON parse error: {}", e),
                },
            };

            if let Ok(resp_json) = serde_json::to_string(&res) {
                let _ = writeln!(stream, "{}", resp_json);
            }
        }
    }
}

fn main() {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        let _ = fs::remove_file(&socket_path);
    }

    println!("Iniciando Nexus Host Daemon en {:?}", socket_path);
    let listener = UnixListener::bind(&socket_path).expect("No se pudo enlazar al socket Unix");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Error en conexión entrante: {}", e),
        }
    }
}
