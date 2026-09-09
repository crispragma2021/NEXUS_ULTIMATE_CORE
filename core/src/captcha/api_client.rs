// ============================================================================
// 🧬 NEXUS CAPTCHA API Client — Resolución externa vía Capsolver / 2Captcha
// ============================================================================
// Cliente genérico para APIs de resolución de CAPTCHA.
// Soporta: Capsolver, 2Captcha (fácilmente extensible a Anti-Captcha, etc.)
// ============================================================================

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Tipos de CAPTCHA
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptchaType {
    /// reCAPTCHA v2 (checkbox / invisible)
    RecaptchaV2,
    /// reCAPTCHA v3 (score-based)
    RecaptchaV3,
    /// hCaptcha
    HCaptcha,
    /// Cloudflare Turnstile
    Turnstile,
    /// Image CAPTCHA (OCR-ready)
    ImageCaptcha,
    /// FunCaptcha
    FunCaptcha,
    /// GeeTest
    GeeTest,
    /// AWS WAF
    AwsWaf,
}

impl CaptchaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RecaptchaV2 => "ReCaptchaV2TaskProxyless",
            Self::RecaptchaV3 => "ReCaptchaV3TaskProxyless",
            Self::HCaptcha => "HCaptchaTaskProxyless",
            Self::Turnstile => "AntiTurnstileTaskProxyless",
            Self::ImageCaptcha => "ImageToTextTask",
            Self::FunCaptcha => "FunCaptchaTaskProxyless",
            Self::GeeTest => "GeeTestTaskProxyless",
            Self::AwsWaf => "AwsWafClassification",
        }
    }
}

// ---------------------------------------------------------------------------
// Payloads genéricos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CaptchaTask {
    #[serde(rename = "type")]
    pub task_type: String,
    pub websiteURL: String,
    pub websiteKey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pageAction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minScore: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isInvisible: Option<bool>,
    // Para ImageCaptcha
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub Case: Option<bool>,
    // Para GeeTest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTaskPayload {
    pub clientKey: String,
    pub task: CaptchaTask,
}

#[derive(Debug, Clone, Serialize)]
pub struct GetTaskResultPayload {
    pub clientKey: String,
    pub taskId: String,
}

// ---------------------------------------------------------------------------
// Respuestas de API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskResponse {
    pub error_id: i32,
    #[serde(default)]
    pub error_code: String,
    #[serde(default)]
    pub error_description: String,
    #[serde(default)]
    pub task_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskResultResponse {
    pub error_id: i32,
    #[serde(default)]
    pub error_code: String,
    #[serde(default)]
    pub error_description: String,
    pub status: String,
    #[serde(default)]
    pub solution: Option<JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBalanceResponse {
    pub error_id: i32,
    #[serde(default)]
    pub error_code: String,
    #[serde(default)]
    pub error_description: String,
    #[serde(default)]
    pub balance: f64,
}

// ---------------------------------------------------------------------------
// Resultado de resolución
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CaptchaResult {
    /// Token de resolución (g-recaptcha-response, hcaptcha token, etc.)
    pub token: Option<String>,
    /// Solución raw de la API
    pub raw_solution: Option<JsonValue>,
    /// Costo estimado en USD
    pub cost_estimate: Option<f64>,
    /// Tiempo total de resolución en ms
    pub resolve_time_ms: u64,
}

// ---------------------------------------------------------------------------
// Provider Enum
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CaptchaProvider {
    Capsolver,
    TwoCaptcha,
}

impl CaptchaProvider {
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::Capsolver => "https://api.capsolver.com",
            Self::TwoCaptcha => "https://2captcha.com",
        }
    }

    pub fn create_task_endpoint(&self) -> &'static str {
        match self {
            Self::Capsolver => "/createTask",
            Self::TwoCaptcha => "/in.php",
        }
    }

    pub fn get_task_result_endpoint(&self) -> &'static str {
        match self {
            Self::Capsolver => "/getTaskResult",
            Self::TwoCaptcha => "/res.php",
        }
    }

    pub fn balance_endpoint(&self) -> &'static str {
        match self {
            Self::Capsolver => "/getBalance",
            Self::TwoCaptcha => "/res.php?action=getbalance",
        }
    }
}

// ---------------------------------------------------------------------------
// Client principal
// ---------------------------------------------------------------------------

pub struct CaptchaApiClient {
    client: Client,
    api_key: String,
    provider: CaptchaProvider,
    poll_interval_ms: u64,
    max_poll_attempts: u32,
}

impl CaptchaApiClient {
    /// Crea un nuevo cliente CAPTCHA.
    ///
    /// # Arguments
    /// * `api_key` - API key del proveedor
    /// * `provider` - Proveedor (Capsolver | TwoCaptcha)
    /// * `poll_interval_ms` - Intervalo entre polls (default: 2000ms)
    /// * `max_poll_attempts` - Máximo de reintentos (default: 60 = 2min)
    pub fn new(
        api_key: String,
        provider: CaptchaProvider,
        poll_interval_ms: Option<u64>,
        max_poll_attempts: Option<u32>,
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Fallo al crear HTTP client para CAPTCHA API"),
            api_key,
            provider,
            poll_interval_ms: poll_interval_ms.unwrap_or(2000),
            max_poll_attempts: max_poll_attempts.unwrap_or(60),
        }
    }

    // -----------------------------------------------------------------------
    // Saldo
    // -----------------------------------------------------------------------

    /// Consulta el saldo disponible en la cuenta.
    pub async fn get_balance(&self) -> Result<f64> {
        let url = format!(
            "{}{}",
            self.provider.base_url(),
            self.provider.balance_endpoint()
        );

        let payload = serde_json::json!({
            "clientKey": self.api_key,
        });

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Error al consultar saldo CAPTCHA API")?;

        let balance_resp: GetBalanceResponse = resp
            .json()
            .await
            .context("Error al parsear respuesta de saldo")?;

        if balance_resp.error_id != 0 {
            anyhow::bail!(
                "Error de CAPTCHA API (saldo): [{}] {}",
                balance_resp.error_code,
                balance_resp.error_description
            );
        }

        Ok(balance_resp.balance)
    }

    // -----------------------------------------------------------------------
    // Crear tarea
    // -----------------------------------------------------------------------

    /// Crea una tarea de resolución de CAPTCHA.
    pub async fn create_task(
        &self,
        captcha_type: CaptchaType,
        task_params: CaptchaTaskParams,
    ) -> Result<String> {
        let url = format!(
            "{}{}",
            self.provider.base_url(),
            self.provider.create_task_endpoint()
        );

        let task = CaptchaTask {
            task_type: captcha_type.as_str().to_string(),
            websiteURL: task_params.website_url.clone(),
            websiteKey: task_params.site_key.clone(),
            pageAction: task_params.page_action,
            minScore: task_params.min_score,
            isInvisible: task_params.is_invisible,
            body: task_params.image_body,
            Case: task_params.case_sensitive,
            gt: task_params.gt,
            challenge: task_params.challenge,
        };

        match self.provider {
            CaptchaProvider::Capsolver => {
                let payload = CreateTaskPayload {
                    clientKey: self.api_key.clone(),
                    task,
                };

                let resp = self
                    .client
                    .post(&url)
                    .json(&payload)
                    .send()
                    .await
                    .context("Error al crear tarea CAPTCHA")?;

                let create_resp: CreateTaskResponse = resp
                    .json()
                    .await
                    .context("Error al parsear respuesta de creación")?;

                if create_resp.error_id != 0 {
                    anyhow::bail!(
                        "Error de Capsolver: [{}] {}",
                        create_resp.error_code,
                        create_resp.error_description
                    );
                }

                Ok(create_resp.task_id)
            }
            CaptchaProvider::TwoCaptcha => {
                // 2Captcha usa query params, no JSON body
                let method = "userrecaptcha".to_string();
                let json = "1".to_string();
                let params = [
                    ("key", &self.api_key),
                    ("method", &method),
                    ("googlekey", &task_params.site_key),
                    ("pageurl", &task_params.website_url),
                    ("json", &json),
                ];

                let resp = self
                    .client
                    .get(&url)
                    .query(&params)
                    .send()
                    .await
                    .context("Error al crear tarea en 2Captcha")?;

                let body: JsonValue = resp.json().await?;

                if let Some(status) = body.get("status").and_then(|s| s.as_i64()) {
                    if status != 1 {
                        let err = body
                            .get("error_text")
                            .and_then(|e| e.as_str())
                            .unwrap_or("error desconocido");
                        anyhow::bail!("Error de 2Captcha: {}", err);
                    }
                }

                let task_id = body
                    .get("request")
                    .and_then(|r| r.as_str())
                    .context("2Captcha no devolvió taskId")?
                    .to_string();

                Ok(task_id)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Obtener resultado (con polling)
    // -----------------------------------------------------------------------

    /// Hace polling hasta obtener el resultado de la tarea.
    /// Devuelve error si se excede `max_poll_attempts`.
    pub async fn get_task_result(&self, task_id: &str) -> Result<CaptchaResult> {
        let url = format!(
            "{}{}",
            self.provider.base_url(),
            self.provider.get_task_result_endpoint()
        );

        let start = std::time::Instant::now();

        match self.provider {
            CaptchaProvider::Capsolver => {
                let payload = GetTaskResultPayload {
                    clientKey: self.api_key.clone(),
                    taskId: task_id.to_string(),
                };

                for attempt in 0..self.max_poll_attempts {
                    let resp = self
                        .client
                        .post(&url)
                        .json(&payload)
                        .send()
                        .await
                        .context("Error al consultar resultado CAPTCHA")?;

                    let result: TaskResultResponse =
                        resp.json().await.context("Error al parsear resultado")?;

                    if result.error_id != 0 {
                        anyhow::bail!(
                            "Error de Capsolver (resultado): [{}] {}",
                            result.error_code,
                            result.error_description
                        );
                    }

                    if result.status == "ready" {
                        let solution = result.solution.unwrap_or_default();
                        let token = solution
                            .get("gRecaptchaResponse")
                            .or_else(|| solution.get("token"))
                            .or_else(|| solution.get("text"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        return Ok(CaptchaResult {
                            token,
                            raw_solution: Some(solution),
                            cost_estimate: None,
                            resolve_time_ms: start.elapsed().as_millis() as u64,
                        });
                    }

                    // Aún procesando — esperar
                    tokio::time::sleep(Duration::from_millis(self.poll_interval_ms)).await;

                    if attempt >= self.max_poll_attempts - 1 {
                        anyhow::bail!("Timeout esperando resultado CAPTCHA (task_id: {})", task_id);
                    }
                }

                anyhow::bail!("No se pudo resolver CAPTCHA (task_id: {})", task_id);
            }
            CaptchaProvider::TwoCaptcha => {
                let params = [
                    ("key", &self.api_key),
                    ("action", &"get".to_string()),
                    ("id", &task_id.to_string()),
                    ("json", &"1".to_string()),
                ];

                for attempt in 0..self.max_poll_attempts {
                    let resp = self
                        .client
                        .get(&url)
                        .query(&params)
                        .send()
                        .await
                        .context("Error al consultar resultado 2Captcha")?;

                    let body: JsonValue = resp.json().await?;

                    if let Some(status) = body.get("status").and_then(|s| s.as_i64()) {
                        if status == 1 {
                            let token = body
                                .get("request")
                                .and_then(|r| r.as_str())
                                .map(|s| s.to_string());

                            return Ok(CaptchaResult {
                                token,
                                raw_solution: Some(body),
                                cost_estimate: None,
                                resolve_time_ms: start.elapsed().as_millis() as u64,
                            });
                        }
                    }

                    // CAPCHA_NOT_READY
                    tokio::time::sleep(Duration::from_millis(self.poll_interval_ms)).await;

                    if attempt >= self.max_poll_attempts - 1 {
                        anyhow::bail!(
                            "Timeout esperando resultado 2Captcha (task_id: {})",
                            task_id
                        );
                    }
                }

                anyhow::bail!("No se pudo resolver CAPTCHA (task_id: {})", task_id);
            }
        }
    }

    /// Método completo: crear tarea + polling hasta obtener resultado.
    pub async fn solve(
        &self,
        captcha_type: CaptchaType,
        task_params: CaptchaTaskParams,
    ) -> Result<CaptchaResult> {
        let task_id = self.create_task(captcha_type, task_params).await?;
        self.get_task_result(&task_id).await
    }
}

// ---------------------------------------------------------------------------
// Parámetros de tarea (independientes del provider)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CaptchaTaskParams {
    pub website_url: String,
    pub site_key: String,
    pub page_action: Option<String>,
    pub min_score: Option<f64>,
    pub is_invisible: Option<bool>,
    // Para ImageCaptcha
    pub image_body: Option<String>,
    pub case_sensitive: Option<bool>,
    // Para GeeTest
    pub gt: Option<String>,
    pub challenge: Option<String>,
}

impl CaptchaTaskParams {
    /// Constructor rápido para reCAPTCHA/hCaptcha.
    pub fn new(website_url: impl Into<String>, site_key: impl Into<String>) -> Self {
        Self {
            website_url: website_url.into(),
            site_key: site_key.into(),
            page_action: None,
            min_score: None,
            is_invisible: None,
            image_body: None,
            case_sensitive: None,
            gt: None,
            challenge: None,
        }
    }

    /// Para reCAPTCHA v3, configura el score mínimo esperado.
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Para reCAPTCHA v2 invisible.
    pub fn invisible(mut self) -> Self {
        self.is_invisible = Some(true);
        self
    }

    /// Para Image CAPTCHA.
    pub fn image(mut self, base64_body: String, case_sensitive: bool) -> Self {
        self.image_body = Some(base64_body);
        self.case_sensitive = Some(case_sensitive);
        self
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_captcha_type_strings() {
        assert_eq!(
            CaptchaType::RecaptchaV2.as_str(),
            "ReCaptchaV2TaskProxyless"
        );
        assert_eq!(CaptchaType::HCaptcha.as_str(), "HCaptchaTaskProxyless");
        assert_eq!(
            CaptchaType::Turnstile.as_str(),
            "AntiTurnstileTaskProxyless"
        );
        assert_eq!(CaptchaType::ImageCaptcha.as_str(), "ImageToTextTask");
    }

    #[test]
    fn test_task_params_builder() {
        let params = CaptchaTaskParams::new("https://example.com", "6Lc...")
            .with_min_score(0.5)
            .invisible();

        assert_eq!(params.website_url, "https://example.com");
        assert_eq!(params.min_score, Some(0.5));
        assert_eq!(params.is_invisible, Some(true));
    }

    #[tokio::test]
    async fn test_balance_without_key() {
        let client =
            CaptchaApiClient::new("INVALID_KEY".into(), CaptchaProvider::Capsolver, None, None);
        let result = client.get_balance().await;
        // Debe fallar — clave inválida
        assert!(result.is_err());
    }
}
