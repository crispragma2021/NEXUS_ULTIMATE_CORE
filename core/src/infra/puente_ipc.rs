// ==========================================
// PUENTE IPC NATIVO — UNIX SOCKETS (LATENCIA CERO)
// ==========================================
// Expone un socket local para transferir telemetría, posiciones y PnL
// en microsegundos sin TCP loopback pesado.
// Linux: Unix socket nativo en /tmp. Windows: TCP loopback en 127.0.0.1.
// ==========================================

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub struct PuenteIpc {
    socket_path: String,
    port: u16,
    tx: broadcast::Sender<String>,
}

impl PuenteIpc {
    pub fn new(path: &str) -> Self {
        let (tx, _) = broadcast::channel(256);
        // En Windows el path estilo /tmp/... se traduce a loopback TCP.
        let port = path
            .rsplit('/')
            .next()
            .and_then(|s| s.split('.').next())
            .and_then(|s| {
                s.bytes()
                    .fold(Some(0u16), |acc, b| acc.and_then(|v| v.checked_mul(31)?.checked_add(b as u16)))
            })
            .unwrap_or(0)
            % 10000
            + 20000;
        Self {
            socket_path: path.to_string(),
            port,
            tx,
        }
    }

    pub fn get_sender(&self) -> broadcast::Sender<String> {
        self.tx.clone()
    }

    pub async fn iniciar(&self) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            use tokio::net::UnixListener;
            // Limpiar socket previo si existe
            if std::path::Path::new(&self.socket_path).exists() {
                let _ = std::fs::remove_file(&self.socket_path);
            }
            let listener = UnixListener::bind(&self.socket_path)?;
            info!("🔌 [IPC] Unix socket activo: {}", self.socket_path);
            let tx = self.tx.clone();
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((mut stream, _)) => {
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let mut buf = vec![0u8; 8192];
                                loop {
                                    match stream.read(&mut buf).await {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            if let Ok(texto) =
                                                String::from_utf8(buf[..n].to_vec())
                                            {
                                                let _ = tx.send(texto);
                                            }
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("🔌 [IPC] Error aceptando conexión: {e}");
                        }
                    }
                }
            });
        }
        #[cfg(windows)]
        {
            use tokio::net::TcpListener;
            let addr = format!("127.0.0.1:{}", self.port);
            let listener = TcpListener::bind(&addr).await?;
            info!("🔌 [IPC] Puente loopback TCP activo: {addr}");
            let tx = self.tx.clone();
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((mut stream, _)) => {
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let mut buf = vec![0u8; 8192];
                                loop {
                                    match stream.read(&mut buf).await {
                                        Ok(0) | Err(_) => break,
                                        Ok(n) => {
                                            if let Ok(texto) =
                                                String::from_utf8(buf[..n].to_vec())
                                            {
                                                let _ = tx.send(texto);
                                            }
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("🔌 [IPC] Error aceptando conexión: {e}");
                        }
                    }
                }
            });
        }
        Ok(())
    }

    #[allow(unused)]
    fn _keep(_: Arc<()>, _: &str) {
        let _ = warn!("");
    }
}
