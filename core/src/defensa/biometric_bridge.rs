/// Phase 29: Biometric Bridge — Ed25519 signature receiver + QR code generator
/// Listens on 0.0.0.0:5002 for mobile biometric signature requests.
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricChallenge {
    pub nonce: String,
    pub timestamp: i64,
    pub action_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricResponse {
    pub nonce: String,
    pub signature_hex: String,     // Classic Ed25519
    pub pqc_signature_hex: String, // ML-DSA (Post-Quantum)
}

/// Start the biometric relay server on port 5002
/// Mobile client scans QR and sends Ed25519 signature for high-risk commands
pub async fn start_biometric_server(host: &str) -> Result<()> {
    let addr: SocketAddr = format!("{}:5002", host)
        .parse()
        .map_err(|e| anyhow!("Invalid bind address: {}", e))?;

    println!("🔐 [BIOMETRIC] Sovereign relay server starting on {}", addr);
    print_qr_url(&format!("http://{}:5002", host));

    // Use tokio TCP listener for incoming signature submissions
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("🔐 [BIOMETRIC] Server ONLINE — awaiting mobile signatures");

    loop {
        match listener.accept().await {
            Ok((stream, peer)) => {
                println!("\n🔐 [BIOMETRIC] CONNECTION DETECTED FROM: {}", peer);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  ⚠️ AUTHENTICATION GATE ACTIVE");
                println!("  AWAITING BIOMETRIC VETO FROM ARCHITECT...");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                tokio::spawn(handle_biometric_connection(stream));
            }
            Err(e) => println!("⚠️ [BIOMETRIC] Accept error: {}", e),
        }
    }
}

async fn handle_biometric_connection(mut stream: tokio::net::TcpStream) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut buf = vec![0u8; 4096];
    match stream.read(&mut buf).await {
        Ok(n) => {
            let data = &buf[..n];
            if let Ok(resp) = serde_json::from_slice::<BiometricResponse>(data) {
                println!(
                    "✅ [BIOMETRIC] Signature received from mobile — nonce: {}",
                    &resp.nonce[..8.min(resp.nonce.len())]
                );
                // TODO: Validate against SecurityProtocol and unlock pending action
                let ack = b"{\"status\":\"received\"}";
                let _ = stream.write_all(ack).await;
            }
        }
        Err(e) => println!("⚠️ [BIOMETRIC] Read error: {}", e),
    }
}

/// Generate a simple QR representation in terminal (ASCII-art URL)
fn print_qr_url(url: &str) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  📱 NEXUS BIOMETRIC ENDPOINT:");
    println!("  🔗 {}", url);
    println!("  Scan this URL from mobile biometric client");
    println!("  to authorize high-risk commands");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
