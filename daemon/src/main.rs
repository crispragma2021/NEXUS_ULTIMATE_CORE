use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Command;
use serde::{Deserialize, Serialize};

const SOCKET_PATH: &str = "/data/data/com.termux/files/home/.nexus_host.sock";

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
    let wrapper_path = "/data/data/com.termux/files/home/.local/bin/rish-exec";
    let output = Command::new(wrapper_path)
        .env("RISH_APPLICATION_ID", "com.termux")
        .args(&["-c", cmd_str])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if out.status.success() {
                Ok(if stdout.is_empty() { "OK".to_string() } else { stdout.trim().to_string() })
            } else {
                Err(if stderr.is_empty() { format!("Error: {}", out.status) } else { stderr.trim().to_string() })
            }
        }
        Err(e) => Err(format!("Fallo al ejecutar rish-exec: {}", e)),
    }
}

fn handle_client(mut stream: UnixStream) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    for line in reader.lines() {
        if let Ok(line_str) = line {
            if line_str.trim().is_empty() { continue; }

            let response = match serde_json::from_str::<ActionRequest>(&line_str) {
                Ok(ActionRequest::Tap { x, y }) => {
                    let cmd = format!("input tap {} {}", x, y);
                    let res = run_with_shizuku(&cmd);
                    ActionResponse { ok: res.is_ok(), result: res.unwrap_or_else(|e| e) }
                }
                Ok(ActionRequest::Type { text }) => {
                    let cmd = format!("input text '{}'", text.replace("'", "'\\''"));
                    let res = run_with_shizuku(&cmd);
                    ActionResponse { ok: res.is_ok(), result: res.unwrap_or_else(|e| e) }
                }
                Ok(ActionRequest::Screenshot) => {
                    let path = format!("/sdcard/DCIM/Screenshots/nexus_{}.png", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
                    let cmd = format!("screencap -p {}", path);
                    let res = run_with_shizuku(&cmd);
                    ActionResponse {
                        ok: res.is_ok(),
                        result: if res.is_ok() { path } else { res.unwrap_err() },
                    }
                }
                Ok(ActionRequest::GetScreenSize) => {
                    let res = run_with_shizuku("wm size");
                    ActionResponse { ok: res.is_ok(), result: res.unwrap_or_else(|e| e) }
                }
                Ok(ActionRequest::Exec { command }) => {
                    let res = run_with_shizuku(&command);
                    ActionResponse { ok: res.is_ok(), result: res.unwrap_or_else(|e| e) }
                }
                Err(e) => ActionResponse { ok: false, result: format!("JSON error: {}", e) },
            };

            if let Ok(json_res) = serde_json::to_string(&response) {
                let _ = writeln!(stream, "{}", json_res);
                let _ = stream.flush();
            }
        }
    }
}

fn main() {
    let _ = fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("No se pudo crear socket UNIX");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                std::thread::spawn(move || handle_client(s));
            }
            Err(_) => break,
        }
    }
}
