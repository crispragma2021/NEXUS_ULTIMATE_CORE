use anyhow::{anyhow, Result};
use reqwest::Client;
use std::time::Duration;
use tokio::time;
use tracing::{info, warn};

pub struct SmsActivateClient {
    api_key: String,
    client: Client,
}

impl SmsActivateClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Solicita un número para Google (go) en Paraguay (py - 68) o internacional
    pub async fn pedir_numero(&self, service: &str, country: &str) -> Result<(String, String)> {
        let url = format!(
            "https://api.sms-activate.org/stubs/handler_api.php?api_key={}&action=getNumber&service={}&country={}",
            self.api_key, service, country
        );

        let resp = self.client.get(&url).send().await?.text().await?;

        if resp.contains("ACCESS_NUMBER") {
            let parts: Vec<&str> = resp.split(':').collect();
            if parts.len() >= 3 {
                let id = parts[1].to_string();
                let number = parts[2].to_string();
                info!("📞 [SMS] Número obtenido: {} (ID: {})", number, id);
                return Ok((id, number));
            }
        }

        Err(anyhow!("Error obteniendo número: {}", resp))
    }

    /// Espera el código de verificación (máximo 5 minutos)
    pub async fn esperar_codigo(&self, id: &str) -> Result<String> {
        info!("⏳ [SMS] Esperando código para ID {}...", id);

        for _ in 0..60 {
            // 60 intentos * 5 segundos = 5 minutos
            let url = format!(
                "https://api.sms-activate.org/stubs/handler_api.php?api_key={}&action=getStatus&id={}",
                self.api_key, id
            );

            let resp = self.client.get(&url).send().await?.text().await?;

            if resp.contains("STATUS_OK") {
                let parts: Vec<&str> = resp.split(':').collect();
                if parts.len() >= 2 {
                    let code = parts[1].to_string();
                    info!("✅ [SMS] Código recibido: {}", code);
                    return Ok(code);
                }
            } else if resp.contains("STATUS_WAIT_CODE") {
                // Seguir esperando
            } else {
                warn!("⚠️ [SMS] Estado inesperado: {}", resp);
            }

            time::sleep(Duration::from_secs(5)).await;
        }

        Err(anyhow!("Timeout esperando código SMS"))
    }

    /// Confirma que el número fue usado con éxito
    pub async fn confirmar_exito(&self, id: &str) -> Result<()> {
        let url = format!(
            "https://api.sms-activate.org/stubs/handler_api.php?api_key={}&action=setStatus&status=6&id={}",
            self.api_key, id
        );
        let _ = self.client.get(&url).send().await?;
        Ok(())
    }
}
