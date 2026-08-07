// 📱 NEXUS OMEGA — Cliente SMS Activate para verificación telefónica
// ============================================================
// API REST de sms-activate.org — obtención de números virtuales
// y recepción de SMS para verificación de cuentas (Gmail, Telegram, WhatsApp, etc.)
//
// Endpoints:
//   GET /stubs/handler_api.php?api_key=KEY&action=getBalance
//   GET /stubs/handler_api.php?api_key=KEY&action=getNumbersStatus&country=CODE
//   GET /stubs/handler_api.php?api_key=KEY&action=getNumber&country=CODE&service=SERVICE
//   GET /stubs/handler_api.php?api_key=KEY&action=setStatus&status=N&id=ID
//   GET /stubs/handler_api.php?api_key=KEY&action=getStatus&id=ID

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const BASE_URL: &str = "https://api.sms-man.com/stubs/handler_api.php";

/// Servicios soportados por SMS Activate
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmsService {
    Google,    // go
    Telegram,  // tg
    WhatsApp,  // wa
    Facebook,  // fb
    Twitter,   // tw
    Instagram, // ig
    Outlook,   // outlook
    Custom(&'static str),
}

impl SmsService {
    pub fn code(&self) -> &str {
        match self {
            SmsService::Google => "go",
            SmsService::Telegram => "tg",
            SmsService::WhatsApp => "wa",
            SmsService::Facebook => "fb",
            SmsService::Twitter => "tw",
            SmsService::Instagram => "ig",
            SmsService::Outlook => "outlook",
            SmsService::Custom(c) => c,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "google" | "gmail" | "go" => Some(SmsService::Google),
            "telegram" | "tg" => Some(SmsService::Telegram),
            "whatsapp" | "wa" => Some(SmsService::WhatsApp),
            "facebook" | "fb" => Some(SmsService::Facebook),
            "twitter" | "tw" => Some(SmsService::Twitter),
            "instagram" | "ig" => Some(SmsService::Instagram),
            "outlook" => Some(SmsService::Outlook),
            _ => None,
        }
    }
}

/// Códigos de país para SMS Activate
#[derive(Debug, Clone, Copy)]
pub enum SmsCountry {
    Paraguay,
    Argentina,
    Brazil,
    Mexico,
    Colombia,
    Chile,
    Peru,
    Usa,
    Spain,
    Uruguay,
    Ecuador,
    Custom(u32),
}

impl SmsCountry {
    pub fn code(&self) -> u32 {
        match self {
            SmsCountry::Paraguay => 12,
            SmsCountry::Argentina => 7,
            SmsCountry::Brazil => 6,
            SmsCountry::Mexico => 14,
            SmsCountry::Colombia => 17,
            SmsCountry::Chile => 8,
            SmsCountry::Peru => 15,
            SmsCountry::Usa => 1,
            SmsCountry::Spain => 34,
            SmsCountry::Uruguay => 27,
            SmsCountry::Ecuador => 19,
            SmsCountry::Custom(c) => *c,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "paraguay" => Some(SmsCountry::Paraguay),
            "argentina" => Some(SmsCountry::Argentina),
            "brazil" | "brasil" => Some(SmsCountry::Brazil),
            "mexico" => Some(SmsCountry::Mexico),
            "colombia" => Some(SmsCountry::Colombia),
            "chile" => Some(SmsCountry::Chile),
            "peru" => Some(SmsCountry::Peru),
            "usa" | "eeuu" | "estados_unidos" => Some(SmsCountry::Usa),
            "spain" | "españa" => Some(SmsCountry::Spain),
            _ => None,
        }
    }
}

/// Estado de una activación SMS
#[derive(Debug, Clone, PartialEq)]
pub enum ActivationStatus {
    /// Esperando SMS (código 1)
    Pending,
    /// SMS recibido (código 6)
    SmsReceived(String),
    /// Cancelada
    Canceled,
    /// Finalizada
    Finished,
}

/// Resultado de una activación de número
#[derive(Debug, Clone)]
pub struct ActivationResult {
    pub activation_id: String,
    pub phone_number: String,
    pub country_code: u32,
    pub service: String,
    pub status: ActivationStatus,
    pub sms_code: Option<String>,
}

impl ActivationResult {
    pub fn phone_display(&self) -> String {
        format!("+{}", self.phone_number)
    }
}

/// Estados de SMS Activate
#[derive(Deserialize)]
struct ActivationResponse {
    #[allow(dead_code)]
    status: Option<String>,
}

// ── Cliente Principal ──────────────────────────────────────────

pub struct SmsActivateClient {
    client: Client,
    api_key: String,
}

impl SmsActivateClient {
    /// Crea el cliente. Busca SMS_ACTIVATE_API_KEY en entorno.
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("SMS_ACTIVATE_API_KEY").map_err(|_| {
            anyhow!("SMS_ACTIVATE_API_KEY no configurada. Obtén una en https://sms-activate.org")
        })?;

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self { client, api_key })
    }

    /// Constructor con API key explícita (para testing)
    pub fn with_key(api_key: String) -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        Ok(Self { client, api_key })
    }

    /// Consulta el saldo de la cuenta
    pub async fn get_balance(&self) -> Result<f64> {
        let resp = self.call_api(&[("action", "getBalance")]).await?;

        // Respuesta: "ACCESS_BALANCE:123.45"
        let body = resp.text().await?;
        if body.starts_with("ACCESS_BALANCE:") {
            let balance_str = body.trim_start_matches("ACCESS_BALANCE:");
            let balance: f64 = balance_str
                .trim()
                .parse()
                .map_err(|e| anyhow!("Error parsing balance '{}': {}", balance_str, e))?;
            Ok(balance)
        } else {
            Err(anyhow!("Error getting balance: {}", body))
        }
    }

    /// Obtiene números disponibles por servicio y país
    pub async fn get_numbers_status(
        &self,
        country: Option<SmsCountry>,
    ) -> Result<HashMap<String, u32>> {
        let mut params = vec![("action", "getNumbersStatus")];
        let country_code;
        if let Some(ref c) = country {
            country_code = c.code().to_string();
            params.push(("country", &country_code));
        }

        let resp = self.call_api(&params).await?;
        let body = resp.text().await?;

        // Parsear JSON: {"go_0":123, "tg_0":45, ...}
        let map: HashMap<String, serde_json::Value> = serde_json::from_str(&body).map_err(|e| {
            anyhow!(
                "Error parsing numbers status '{}': {}",
                &body[..body.len().min(200)],
                e
            )
        })?;

        let result = map
            .into_iter()
            .filter_map(|(k, v)| v.as_u64().map(|n| (k, n as u32)))
            .collect();

        Ok(result)
    }

    /// Solicita un número para un servicio específico
    pub async fn get_number(
        &self,
        service: SmsService,
        country: SmsCountry,
    ) -> Result<ActivationResult> {
        let country_code = country.code().to_string();
        let resp = self
            .call_api(&[
                ("action", "getNumber"),
                ("country", &country_code),
                ("service", service.code()),
                ("operator", "any"),
            ])
            .await?;

        let body = resp.text().await?;

        // Respuesta esperada: "ACCESS_NUMBER:123456789:4455667788"
        // formato: ACCESS_NUMBER:activationId:phoneNumber
        if body.starts_with("ACCESS_NUMBER:") {
            let parts: Vec<&str> = body.splitn(3, ':').collect();
            if parts.len() < 3 {
                return Err(anyhow!("Unexpected response format: {}", body));
            }
            let activation_id = parts[1].to_string();
            let phone_number = parts[2].trim().to_string();

            Ok(ActivationResult {
                activation_id,
                phone_number,
                country_code: country.code(),
                service: service.code().to_string(),
                status: ActivationStatus::Pending,
                sms_code: None,
            })
        } else if body.starts_with("NO_NUMBERS") {
            Err(anyhow!(
                "No hay números disponibles para {} en país {}",
                service.code(),
                country.code()
            ))
        } else if body.starts_with("NO_BALANCE") {
            Err(anyhow!("Saldo insuficiente en SMS Activate"))
        } else {
            Err(anyhow!("Error getting number: {}", body))
        }
    }

    /// Espera y obtiene el código SMS de una activación
    /// Polling cada 5 segundos hasta timeout (120s por defecto)
    pub async fn wait_for_sms(&self, activation_id: &str, timeout_secs: u64) -> Result<String> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                // Cancelar activación por timeout
                let _ = self.set_status(activation_id, 8).await; // 8 = cancel
                return Err(anyhow!(
                    "Timeout esperando SMS para activación {} ({}s)",
                    activation_id,
                    timeout_secs
                ));
            }

            let status = self.get_status(activation_id).await?;
            match status {
                ActivationStatus::SmsReceived(code) => {
                    // Marcar como finalizada (status 6 = complete)
                    let _ = self.set_status(activation_id, 6).await;
                    return Ok(code);
                }
                ActivationStatus::Pending => {
                    // Esperar 5 segundos antes de reintentar
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                ActivationStatus::Canceled | ActivationStatus::Finished => {
                    return Err(anyhow!(
                        "Activación {} terminó inesperadamente",
                        activation_id
                    ));
                }
            }
        }
    }

    /// Obtiene el estado actual de una activación
    pub async fn get_status(&self, activation_id: &str) -> Result<ActivationStatus> {
        let resp = self
            .call_api(&[("action", "getStatus"), ("id", activation_id)])
            .await?;

        let body = resp.text().await?;

        if body.starts_with("STATUS_WAIT_CODE") {
            Ok(ActivationStatus::Pending)
        } else if body.starts_with("STATUS_CANCEL") {
            Ok(ActivationStatus::Canceled)
        } else if let Some(code) = body.strip_prefix("STATUS_OK_") {
            // STATUS_OK_123456
            Ok(ActivationStatus::SmsReceived(code.trim().to_string()))
        } else {
            Err(anyhow!("Unknown activation status: {}", body))
        }
    }

    /// Cambia el estado de una activación
    /// status: 1=ready, 6=complete, 8=cancel
    pub async fn set_status(&self, activation_id: &str, status: u8) -> Result<()> {
        let status_str = status.to_string();
        let resp = self
            .call_api(&[
                ("action", "setStatus"),
                ("status", &status_str),
                ("id", activation_id),
            ])
            .await?;

        let body = resp.text().await?;
        if !body.starts_with("ACCESS") {
            return Err(anyhow!("Error setting status: {}", body));
        }
        Ok(())
    }

    /// Libera un número (cancela la activación)
    pub async fn release_number(&self, activation_id: &str) -> Result<()> {
        self.set_status(activation_id, 8).await
    }

    // ── Privado ──────────────────────────────────────────────────

    async fn call_api(&self, params: &[(&str, &str)]) -> Result<reqwest::Response> {
        let _url = reqwest::Url::parse_with_params(BASE_URL, params)
            .map_err(|e| anyhow!("Error building URL: {}", e))?;

        // Construir URL manualmente para incluir api_key en query
        let url_str = format!(
            "{}?api_key={}&{}",
            BASE_URL,
            self.api_key,
            params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding(v)))
                .collect::<Vec<_>>()
                .join("&")
        );

        let resp = self.client.get(&url_str).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("SMS Activate API error {}: {}", status, text));
        }

        Ok(resp)
    }
}

/// URL-encode simple (evita agregar dependencia solo para esto)
fn urlencoding(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
