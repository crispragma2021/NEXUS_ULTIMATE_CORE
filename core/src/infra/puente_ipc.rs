// ==========================================
// PUENTE IPC NATIVO — UNIX SOCKETS (LATENCIA CERO)
// ==========================================
// Expone un socket Unix local en `/tmp/nexus_trader.sock` para
// transferir telemetría, posiciones y PnL en microsegundos sin TCP loopback.
// ==========================================

use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub struct PuenteIpc {
    socket_path: String,
    tx: broadcast::Sender<String>,
}

impl PuenteIpc {
    pub fn new(path: &str) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            socket_path: path.to_string(),
            tx,
        }
    }

    /// Retorna un clon del sender para enviar mensajes desde otras partes del core
    pub fn get_sender(&self) -> broadcast::Sender<String> {
        self.tx.clone()
    }

    /// Inicia el bucle de escucha del socket Unix
    pub async fn iniciar(&self) -> io::Result<()> {
        let path = Path::new(&self.socket_path);

        // Limpiar socket anterior si existe
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }

        let listener = UnixListener::bind(path)?;
        info!("📡 Puente IPC Unix Socket activo en {}", self.socket_path);

        let tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let rx = tx.subscribe();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, rx).await {
                                warn!("🔌 Cliente IPC desconectado: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("❌ Error aceptando conexión en socket Unix: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

async fn handle_client(
    mut stream: UnixStream,
    mut rx: broadcast::Receiver<String>,
) -> io::Result<()> {
    let mut buf = [0u8; 1];
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        let payload = format!("{}\n", text);
                        stream.write_all(payload.as_bytes()).await?;
                        stream.flush().await?;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        warn!("⚠️ Cliente IPC lento, mensajes descartados");
                    }
                    Err(_) => break,
                }
            }
            // Detectar desconexión del cliente leyendo 0 bytes
            res = stream.read(&mut buf) => {
                if let Ok(0) = res {
                    break;
                }
            }
        }
    }
    Ok(())
}

use std::io;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unix_socket_ipc() {
        let sock_path = "/tmp/test_nexus_ipc.sock";
        let puente = PuenteIpc::new(sock_path);
        assert!(puente.iniciar().await.is_ok());

        let tx = puente.get_sender();

        // Simular conexión de cliente
        let client_conn = UnixStream::connect(sock_path).await;
        assert!(client_conn.is_ok());
        let mut client = client_conn.unwrap();

        // Dar un momento para que el loop del servidor acepte la conexión y se suscriba
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Enviar mensaje a través del puente
        tx.send("TELEMETRY_OK".to_string()).unwrap();

        // Leer en el cliente
        let mut buf = vec![0u8; 13];
        let bytes_read = client.read_exact(&mut buf).await;
        assert!(bytes_read.is_ok());
        assert_eq!(String::from_utf8_lossy(&buf), "TELEMETRY_OK\n");

        // Limpiar
        let _ = std::fs::remove_file(sock_path);
    }
}
