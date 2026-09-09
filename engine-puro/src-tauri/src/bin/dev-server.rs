// ============================================================================
// 🦀 Dev-Server — Servidor HTTP estático en Rust puro (std-only)
// Sirve archivos del directorio dist/ en el puerto 43211
// ============================================================================

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;

const PUERTO: u16 = 43211;
const DIRECTORIO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/dist");

fn main() {
    let direccion = format!("127.0.0.1:{}", PUERTO);
    eprintln!("🧬 Dev-Server: sirviendo {} → http://{}", DIRECTORIO, direccion);

    let listener = TcpListener::bind(&direccion).expect("No se pudo bindear el puerto");
    listener.set_nonblocking(true).ok();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => { thread::spawn(|| { let _ = manejar(stream); }); }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => { eprintln!("⚠️ Error: {}", e); break; }
        }
    }
}

fn manejar(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    let mut reader = BufReader::new(&mut stream);
    reader.read_line(&mut buf)?;

    let ruta = buf.split_whitespace().nth(1).unwrap_or("/").trim_start_matches('/');
    let ruta = if ruta.is_empty() || ruta == "/" { "index.html".to_string() } else { ruta.split('?').next().unwrap_or(ruta).to_string() };
    let path = Path::new(DIRECTORIO).join(&ruta);

    // Leer resto de headers
    loop { let mut h = String::new(); reader.read_line(&mut h)?; if h.trim().is_empty() { break; } }

    let (status, ct, body) = if path.exists() && path.is_file() {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let ct = match ext { "html" => "text/html; charset=utf-8", "js" => "text/javascript; charset=utf-8", "css" => "text/css; charset=utf-8", "json"=>"application/json","png"=>"image/png","ico"=>"image/x-icon","svg"=>"image/svg+xml", _ => "text/plain" };
        ("200 OK", ct, fs::read(&path)?)
    } else {
        let index = Path::new(DIRECTORIO).join("index.html");
        if index.exists() { ("200 OK", "text/html; charset=utf-8", fs::read(&index)?) }
        else { ("404 Not Found", "text/plain", b"404".to_vec()) }
    };

    write!(stream, "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n", status, ct, body.len())?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}
