// ==========================================
// NEXUS COGNITIVE GATEWAY (IMMORTAL LAYER)
// ==========================================
// Binds to public port 43211.
// Transparently hot-swaps and routes to:
// - Cell Alpha (43215)
// - Cell Beta (43216)
// ==========================================

use std::process::Command;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

const PUBLIC_PORT: u16 = 43211;
const PORT_ALFA: u16 = 43215;
const PORT_BETA: u16 = 43216;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        .add_directive("chromiumoxide=off".parse().unwrap())
        .add_directive("tungstenite=off".parse().unwrap())
        .add_directive("tokio_tungstenite=off".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .without_time()
        .with_level(true)
        .init();

    // Asegurar directorio de logs
    let current_dir = std::env::current_dir()?;
    let logs_dir = current_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).ok();

    println!("\n");
    println!("╔═════════════════════════════════════════════════════════════════════════╗");
    println!("║                                                                         ║");
    println!("║   🛡️  NEXUS COGNITIVE GATEWAY (CAPA INMORTAL SOBERANA)                     ║");
    println!("║                                                                         ║");
    println!(
        "║   📍 Puerto Público: {}                                                 ║",
        PUBLIC_PORT
    );
    println!(
        "║   🧬 Célula Alfa (Activa Principal): http://127.0.0.1:{}               ║",
        PORT_ALFA
    );
    println!(
        "║   🧬 Célula Beta (Standby Caliente): http://127.0.0.1:{}               ║",
        PORT_BETA
    );
    println!("║                                                                         ║");
    println!("╚═════════════════════════════════════════════════════════════════════════╝");
    println!("\n");

    let active_port = Arc::new(AtomicU16::new(PORT_ALFA));

    // Pre-arranque de ambas células
    println!("🧬 [GATEWAY] Iniciando células de consciencia Alfa y Beta...");
    relaunch_cell(PORT_ALFA);
    relaunch_cell(PORT_BETA);

    // Bucle centinela proactivo de latidos (Heartbeat Sentinel)
    let active_port_clone = active_port.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let current = active_port_clone.load(Ordering::SeqCst);

            if !check_port_healthy(current).await {
                let fallback = if current == PORT_ALFA {
                    PORT_BETA
                } else {
                    PORT_ALFA
                };

                println!(
                    "🚨 [GATEWAY] Célula activa en puerto {} inactiva o congelada.",
                    current
                );

                if check_port_healthy(fallback).await {
                    println!(
                        "🧬 [GATEWAY] ¡Mitosis en Caliente! Conmutando ruta hacia standby: {}",
                        fallback
                    );
                    active_port_clone.store(fallback, Ordering::SeqCst);
                    relaunch_cell(current);
                } else {
                    println!("🚨 [GATEWAY] ¡Colapso Total! Ambas células caídas. Re-iniciando ecosistema...");
                    relaunch_cell(PORT_ALFA);
                    relaunch_cell(PORT_BETA);
                    active_port_clone.store(PORT_ALFA, Ordering::SeqCst);
                }
            }
        }
    });

    let listener = TcpListener::bind(format!("0.0.0.0:{}", PUBLIC_PORT)).await?;

    loop {
        let (client_stream, _) = listener.accept().await?;
        let active_port_clone = active_port.clone();

        tokio::spawn(async move {
            let current = active_port_clone.load(Ordering::SeqCst);
            if let Err(_) = handle_routing(client_stream, current).await {
                // Re-enrutamiento reactivo en vuelo en caso de fallo instantáneo
                let fallback = if current == PORT_ALFA {
                    PORT_BETA
                } else {
                    PORT_ALFA
                };
                println!(
                    "🚨 [GATEWAY] Falló entrega rápida a {}. Intentando rescate vía {}...",
                    current, fallback
                );

                // Comprobar salud antes de conmutar en vuelo
                if check_port_healthy(fallback).await {
                    active_port_clone.store(fallback, Ordering::SeqCst);
                    relaunch_cell(current);
                }
            }
        });
    }
}

async fn check_port_healthy(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    timeout(Duration::from_millis(500), TcpStream::connect(&addr))
        .await
        .is_ok()
}

fn relaunch_cell(port: u16) {
    let log_file = if port == PORT_ALFA {
        "alfa.log"
    } else {
        "beta.log"
    };

    // 1. Obtener variables de entorno gráficas dinámicamente, con fallback
    let mut display_val = std::env::var("DISPLAY").unwrap_or_else(|_| ":1".to_string());
    let mut wayland_val =
        std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());

    // Intentar leer de .env si existe para mantener consistencia
    let env_path = std::env::current_dir().unwrap().join(".env");
    if let Ok(content) = std::fs::read_to_string(env_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("DISPLAY=") {
                if let Some(val) = line.split('=').nth(1) {
                    display_val = val.trim().to_string();
                }
            } else if line.starts_with("WAYLAND_DISPLAY=") {
                if let Some(val) = line.split('=').nth(1) {
                    wayland_val = val.trim().to_string();
                }
            }
        }
    }

    // 2. Formatear comando con doble-fork y exec para evitar procesos zombies (defunct)
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let bin_path = current_dir.join("target/release/nexus_ultimate_core");
    let logs_dir = current_dir.join("logs");

    let cmd = format!(
        "exec {} --port {} > {}/{} 2>&1 &",
        bin_path.display(),
        port,
        logs_dir.display(),
        log_file
    );

    Command::new("bash")
        .arg("-c")
        .arg(&cmd)
        .env("DISPLAY", &display_val)
        .env("WAYLAND_DISPLAY", &wayland_val)
        .status() // Cosechar el proceso intermedio para evitar bash <defunct>
        .ok();
}

async fn handle_routing(mut client_stream: TcpStream, port: u16) -> std::io::Result<()> {
    // 1. Detectar si es tráfico HTTP leyendo los primeros bytes sin consumirlos (peek)
    let mut peek_buf = [0u8; 10];
    let n = match client_stream.peek(&mut peek_buf).await {
        Ok(n) if n > 0 => n,
        _ => 0,
    };

    let is_http = if n > 0 {
        let peek_str = String::from_utf8_lossy(&peek_buf[..n]).to_uppercase();
        peek_str.starts_with("GET")
            || peek_str.starts_with("POST")
            || peek_str.starts_with("PUT")
            || peek_str.starts_with("DELE")
            || peek_str.starts_with("HEAD")
            || peek_str.starts_with("OPTI")
            || peek_str.starts_with("PATC")
            || peek_str.starts_with("CONN")
            || peek_str.starts_with("TRAC")
    } else {
        false
    };

    if !is_http {
        // Es una intención directa en texto plano
        use nexus_ultimate_core::cerebro::{CompiladorSimbolico, ExtractorEsquemas};

        let mut buf = vec![0u8; 4096];
        let n = match client_stream.read(&mut buf).await {
            Ok(n) if n > 0 => n,
            _ => return Ok(()),
        };

        let intencion = String::from_utf8_lossy(&buf[..n]);
        let intencion_trimmed = intencion.trim();

        let extractor = ExtractorEsquemas::new();
        match extractor.extraer(intencion_trimmed) {
            Ok(ast) => {
                let rs_code = CompiladorSimbolico::compilar(&ast);
                let _ = client_stream.write_all(rs_code.as_bytes()).await;
            }
            Err(e) => {
                let err_msg = format!("❌ Error: {}\n", e);
                let _ = client_stream.write_all(err_msg.as_bytes()).await;
            }
        }
        return Ok(());
    }

    let target_addr = format!("127.0.0.1:{}", port);
    let mut server_stream = match timeout(
        Duration::from_millis(2000),
        TcpStream::connect(&target_addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        _ => {
            let fallback_port = if port == PORT_ALFA {
                PORT_BETA
            } else {
                PORT_ALFA
            };
            let fallback_addr = format!("127.0.0.1:{}", fallback_port);
            match timeout(
                Duration::from_millis(2000),
                TcpStream::connect(&fallback_addr),
            )
            .await
            {
                Ok(Ok(stream)) => stream,
                Ok(Err(e)) => return Err(e),
                Err(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Both cells down or timed out",
                    ))
                }
            }
        }
    };
    copy_bidirectional(&mut client_stream, &mut server_stream).await?;
    Ok(())
}
