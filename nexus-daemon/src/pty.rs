use axum::extract::ws::{Message, WebSocket};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info};

pub async fn handle_pty_socket(mut socket: WebSocket) {
    info!("Conexión de Terminal (PTY) establecida.");

    // Establecer el sistema de PTY
    let pty_system = NativePtySystem::default();
    
    // Crear el PTY
    let pair = match pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(e) => {
            error!("Fallo al abrir PTY: {}", e);
            return;
        }
    };

    // Comando a ejecutar (bash por defecto)
    let cmd = CommandBuilder::new("bash");
    
    // Spawn del proceso dentro del PTY
    let mut child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(e) => {
            error!("Fallo al ejecutar el shell: {}", e);
            return;
        }
    };

    // Clonar el master para leer y escribir
    let mut reader = pair.master.try_clone_reader().unwrap();
    let writer = pair.master.take_writer().unwrap();
    let writer = Arc::new(Mutex::new(writer));

    // Canal para enviar la salida del PTY de vuelta al WebSocket
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // Hilo para leer la salida del PTY (bloqueante)
    std::thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buf[..n]).to_string();
                    if tx.blocking_send(output).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Bucle principal de la conexión WebSocket
    loop {
        tokio::select! {
            // Recibir mensajes del PTY y enviarlos al WebSocket
            Some(output) = rx.recv() => {
                let msg = serde_json::json!({
                    "type": "output",
                    "data": output
                });
                if socket.send(Message::Text(msg.to_string())).await.is_err() {
                    break;
                }
            }
            // Recibir mensajes del WebSocket y escribirlos en el PTY
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                            if parsed["type"] == "keypress" || parsed["type"] == "input" {
                                if let Some(input) = parsed["key"].as_str().or(parsed["data"].as_str()) {
                                    let mut w = writer.lock().unwrap();
                                    let _ = w.write_all(input.as_bytes());
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    info!("Conexión de Terminal cerrada. Limpiando PTY...");
    // Terminar el proceso
    let _ = child.kill();
    let _ = child.wait();
}
