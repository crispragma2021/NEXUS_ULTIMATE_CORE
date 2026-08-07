// core/src/bin/nexus_native_bridge.rs
// 🔱 NEXUS OMEGA - Native Messaging Bridge (Rust -> Extension)

use serde_json::Value;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    loop {
        // 1. Leer longitud del mensaje (4 bytes, little endian)
        let mut len_bytes = [0u8; 4];
        if io::stdin().read_exact(&mut len_bytes).is_err() {
            break; // Stream cerrado
        }
        let len = u32::from_ne_bytes(len_bytes) as usize;

        // 2. Leer el cuerpo del mensaje
        let mut buffer = vec![0u8; len];
        io::stdin().read_exact(&mut buffer)?;

        let msg: Value = serde_json::from_slice(&buffer).expect("Error parseando JSON");

        // 3. Procesar órdenes de NEXUS (Echo para validación inicial)
        let response = serde_json::json!({
            "status": "RECEIVED",
            "echo": msg
        });

        // 4. Enviar respuesta al navegador (Chrome requiere 4 bytes de longitud + JSON)
        let out_msg = response.to_string();
        let out_len = out_msg.len() as u32;
        io::stdout().write_all(&out_len.to_ne_bytes())?;
        io::stdout().write_all(out_msg.as_bytes())?;
        io::stdout().flush()?;
    }
    Ok(())
}
