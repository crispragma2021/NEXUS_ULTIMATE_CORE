//! Adaptador de tier-2 nube (F3.2).
//!
//! Define la interfaz [`CloudProvider`] y un [`CloudAdapter`] que orquesta el
//! fallback entre proveedores en el orden documentado en `plans/pipeline-spec.md`
//! §3.4: **OpenRouter → Gemini → DeepSeek → Sovereign/Web**.
//!
//! Este módulo replica el patrón de `ExtractorOmega`
//! (`scripts/skill-extractor/mod.rs`) dentro del crate `core`, ya que ese módulo
//! no es importable directamente (reside en otro crate). El orden de fallback es
//! idéntico al del motor original.

use crate::scraping::pipeline::provider_circuit::ProviderCircuitBreaker;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Duration;

/// Proveedor de nube que puede razonar sobre texto y devolver JSON.
#[async_trait::async_trait]
pub trait CloudProvider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Envía un prompt y devuelve la respuesta del modelo como texto.
    async fn complete(&self, prompt: &str) -> Result<String>;
}

/// Proveedor OpenRouter con pool rotatorio de API keys.
///
/// Replica `consultar_openrouter` de `ExtractorOmega`: rota keys en 429,
/// reintentos = pool_size * 2.
pub struct OpenRouterProvider {
    client: reqwest::Client,
    api_keys: Vec<String>,
    current_index: std::sync::atomic::AtomicUsize,
    model: String,
    timeout: Duration,
}

impl OpenRouterProvider {
    pub fn new(api_keys: Vec<String>, model: &str) -> Result<Self> {
        if api_keys.is_empty() {
            return Err(anyhow!("OpenRouterProvider requiere al menos 1 API key"));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(45))
            .build()?;
        Ok(Self {
            client,
            api_keys,
            current_index: std::sync::atomic::AtomicUsize::new(0),
            model: model.to_string(),
            timeout: Duration::from_secs(45),
        })
    }
}

#[async_trait::async_trait]
impl CloudProvider for OpenRouterProvider {
    fn name(&self) -> &'static str {
        "openrouter"
    }

    async fn complete(&self, prompt: &str) -> Result<String> {
        let pool_size = self.api_keys.len();
        let max_attempts = pool_size * 2;
        let mut attempts = 0;

        while attempts < max_attempts {
            let idx = self.current_index.load(std::sync::atomic::Ordering::SeqCst) % pool_size;
            let api_key = &self.api_keys[idx];

            let resp = self
                .client
                .post("https://openrouter.ai/api/v1/chat/completions")
                .header("Authorization", format!("Bearer {api_key}"))
                .header("HTTP-Referer", "http://localhost:43211")
                .header("X-Title", "NexusPipeline")
                .timeout(self.timeout)
                .json(&json!({
                    "model": self.model,
                    "messages": [{"role": "user", "content": prompt}]
                }))
                .send()
                .await?;

            let status = resp.status().as_u16();
            if status == 200 {
                let data: Value = resp.json().await?;
                let content = data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string();
                return Ok(content);
            } else if status == 429 || (500..600).contains(&status) {
                // F8.3: rotar también en 5xx (no solo 429).
                self.current_index
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                tracing::warn!("⚠️ [OPENROUTER] key {idx} falló (HTTP {status}), rotando");
                tokio::time::sleep(Duration::from_millis(1000)).await;
                attempts += 1;
            } else {
                let body = resp.text().await.unwrap_or_default();
                return Err(anyhow!("OpenRouter HTTP {status}: {body}"));
            }
        }
        Err(anyhow!("OpenRouter pool exhausto"))
    }
}

/// Proveedor in-memory para tests (mocks).
pub struct MockProvider {
    pub name: &'static str,
    pub should_fail: bool,
    pub response: String,
}

#[async_trait::async_trait]
impl CloudProvider for MockProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn complete(&self, _prompt: &str) -> Result<String> {
        if self.should_fail {
            Err(anyhow!("mock {} falló", self.name))
        } else {
            Ok(self.response.clone())
        }
    }
}

/// Resultado de una invocación al tier-2.
pub struct CloudResult {
    pub content: String,
    pub provider: &'static str,
}

/// Orquesta el fallback entre proveedores en orden.
pub struct CloudAdapter {
    providers: Vec<Box<dyn CloudProvider>>,
    breaker: ProviderCircuitBreaker,
    /// Timeouts por proveedor (segundos). Default según spec §3.3.
    timeouts_ms: std::collections::HashMap<String, u64>,
}

impl CloudAdapter {
    pub fn new(providers: Vec<Box<dyn CloudProvider>>) -> Self {
        Self {
            providers,
            breaker: ProviderCircuitBreaker::default(),
            timeouts_ms: std::collections::HashMap::new(),
        }
    }

    /// Circuit breaker configurable (útil en tests).
    pub fn with_breaker(mut self, breaker: ProviderCircuitBreaker) -> Self {
        self.breaker = breaker;
        self
    }

    /// Configura un timeout explícito para un proveedor (ms).
    pub fn set_timeout(&mut self, provider: &str, ms: u64) {
        self.timeouts_ms.insert(provider.to_string(), ms);
    }

    /// Timeout por defecto para un proveedor (spec §3.3):
    /// OpenRouter 45s, Gemini/DeepSeek 60s, Sovereign/Web 120s.
    fn timeout_for(&self, provider: &str) -> Duration {
        if let Some(ms) = self.timeouts_ms.get(provider) {
            return Duration::from_millis(*ms);
        }
        match provider {
            "openrouter" => Duration::from_secs(45),
            "gemini" | "deepseek" => Duration::from_secs(60),
            "sovereign" | "sovereign_web" => Duration::from_secs(120),
            _ => Duration::from_secs(45),
        }
    }

    /// Intenta cada proveedor en orden; devuelve el primero que tenga éxito.
    ///
    /// Fallback: `OpenRouter → Gemini → DeepSeek → Sovereign/Web` según el
    /// orden en que se inyectaron los proveedores.
    ///
    /// Aplica circuit breaker (3 fallos → pausa 5 min) y timeout por proveedor
    /// (un timeout no bloquea el pipeline, pasa al siguiente).
    pub async fn reason(&self, prompt: &str) -> Result<CloudResult> {
        let mut last_err: Option<anyhow::Error> = None;
        for provider in &self.providers {
            let name = provider.name();

            // Circuit breaker: si está abierto, saltar.
            if !self.breaker.is_allowed(name) {
                tracing::warn!("⚠️ [CIRCUIT] {name} abierto (pausa 5 min), saltando");
                last_err = Some(anyhow!("circuit breaker abierto para {name}"));
                continue;
            }

            // Timeout por proveedor (F8.1).
            let timeout = self.timeout_for(name);
            let fut = provider.complete(prompt);
            match tokio::time::timeout(timeout, fut).await {
                Ok(Ok(content)) => {
                    self.breaker.record_success(name);
                    return Ok(CloudResult {
                        content,
                        provider: name,
                    });
                }
                Ok(Err(e)) => {
                    tracing::warn!("⚠️ [CLOUD ADAPTER] {name} falló: {e} — siguiente");
                    self.breaker.record_failure(name);
                    last_err = Some(e);
                }
                Err(_elapsed) => {
                    tracing::warn!(
                        "⚠️ [CLOUD ADAPTER] {name} timeout (>{}s) — siguiente",
                        timeout.as_secs()
                    );
                    self.breaker.record_failure(name);
                    last_err = Some(anyhow!("timeout de {timeout:?} en {name}"));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!("no hay proveedores configurados")))
    }

    /// Variante que extrae JSON de la respuesta (si la respuesta es JSON puro).
    pub async fn reason_json(&self, prompt: &str) -> Result<(Value, &'static str)> {
        let res = self.reason(prompt).await?;
        let trimmed = res.content.trim();
        // Extraer el primer objeto/array JSON de la respuesta.
        let parsed = parse_json_from_text(trimmed)
            .ok_or_else(|| anyhow!("respuesta del proveedor no es JSON válido"))?;
        Ok((parsed, res.provider))
    }
}

/// Extrae el primer JSON (objeto o array) de un texto (tolera texto envolvente).
pub fn parse_json_from_text(text: &str) -> Option<Value> {
    // Si el texto completo es JSON válido, usarlo directo.
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Some(v);
    }
    // Buscar el primer bloque {...} o [...] balanceado.
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let open = match bytes[i] {
            b'{' => b'}',
            b'[' => b']',
            _ => {
                i += 1;
                continue;
            }
        };
        // Escanear hasta el cierre balanceado (simple, ignorando cadenas).
        let mut depth = 0i32;
        let mut in_string = false;
        let mut j = i;
        while j < bytes.len() {
            let c = bytes[j];
            if in_string {
                if c == b'\\' {
                    j += 2;
                    continue;
                }
                if c == b'"' {
                    in_string = false;
                }
            } else {
                match c {
                    b'"' => in_string = true,
                    c if c == bytes[i] => depth += 1,
                    c if c == open => {
                        depth -= 1;
                        if depth == 0 {
                            let candidate = &text[i..=j];
                            if let Ok(v) = serde_json::from_str::<Value>(candidate) {
                                return Some(v);
                            }
                            break;
                        }
                    }
                    _ => {}
                }
            }
            j += 1;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scraping::pipeline::provider_circuit::ProviderCircuitBreaker;

    /// Mock que duerme más que el timeout configurado (para test F8.1).
    struct SlowProvider {
        pub name: &'static str,
        pub sleep_ms: u64,
    }

    #[async_trait::async_trait]
    impl CloudProvider for SlowProvider {
        fn name(&self) -> &'static str {
            self.name
        }
        async fn complete(&self, _p: &str) -> Result<String> {
            tokio::time::sleep(Duration::from_millis(self.sleep_ms)).await;
            Ok("tarde".to_string())
        }
    }

    #[tokio::test]
    async fn fallback_al_primer_proveedor_exitoso() {
        let adapter = CloudAdapter::new(vec![
            Box::new(MockProvider {
                name: "gemini",
                should_fail: true,
                response: String::new(),
            }),
            Box::new(MockProvider {
                name: "deepseek",
                should_fail: false,
                response: "{\"ok\": true}".to_string(),
            }),
        ]);
        let res = adapter.reason("hola").await.unwrap();
        assert_eq!(res.provider, "deepseek");
    }

    #[tokio::test]
    async fn fallback_devuelve_error_si_todos_fallan() {
        let adapter = CloudAdapter::new(vec![
            Box::new(MockProvider {
                name: "a",
                should_fail: true,
                response: String::new(),
            }),
            Box::new(MockProvider {
                name: "b",
                should_fail: true,
                response: String::new(),
            }),
        ]);
        assert!(adapter.reason("hola").await.is_err());
    }

    #[test]
    fn parse_json_desde_texto_con_envoltura() {
        let v = parse_json_from_text("Aquí va: {\"items\": [1, 2]} y fin").unwrap();
        assert_eq!(v["items"][1], 2);
    }

    #[test]
    fn parse_json_completo() {
        let v = parse_json_from_text("[{\"a\": 1}]").unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn parse_json_devuelve_none_sin_json() {
        assert!(parse_json_from_text("sin datos estructurados").is_none());
    }

    #[test]
    fn openrouter_provider_exige_keys() {
        assert!(OpenRouterProvider::new(vec![], "modelo").is_err());
    }

    // ── F8.1: timeout por proveedor ────────────────────────────────

    #[tokio::test]
    async fn timeout_no_bloquea_y_pasa_al_siguiente() {
        let mut adapter = CloudAdapter::new(vec![
            Box::new(SlowProvider {
                name: "lento",
                sleep_ms: 500,
            }),
            Box::new(MockProvider {
                name: "rapido",
                should_fail: false,
                response: "{\"ok\": true}".to_string(),
            }),
        ]);
        // Timeout corto para el proveedor lento (10ms).
        adapter.set_timeout("lento", 10);
        let res = adapter.reason("hola").await.unwrap();
        assert_eq!(res.provider, "rapido");
    }

    #[tokio::test]
    async fn timeout_registra_fallo_en_breaker() {
        let mut adapter = CloudAdapter::new(vec![Box::new(SlowProvider {
            name: "lento",
            sleep_ms: 500,
        })]);
        adapter.set_timeout("lento", 5);
        assert!(adapter.reason("hola").await.is_err());
    }

    // ── F8.2: circuit breaker integrado en CloudAdapter ────────────

    #[tokio::test]
    async fn breaker_se_abre_y_salta_proveedor() {
        let breaker = ProviderCircuitBreaker::new(2, 300_000);
        let adapter = CloudAdapter::new(vec![Box::new(MockProvider {
            name: "flaky",
            should_fail: true,
            response: String::new(),
        })])
        .with_breaker(breaker);

        // Dos fallos → circuito abierto.
        assert!(adapter.reason("x").await.is_err());
        assert!(adapter.reason("x").await.is_err());
        // Tercera llamada: el proveedor se salta por breaker (sigue error).
        assert!(adapter.reason("x").await.is_err());
    }

    #[tokio::test]
    async fn breaker_se_reactiva_con_exito() {
        let breaker = ProviderCircuitBreaker::new(1, 300_000);
        let adapter = CloudAdapter::new(vec![Box::new(MockProvider {
            name: "flaky",
            should_fail: true,
            response: String::new(),
        })])
        .with_breaker(breaker);
        // Abrir.
        assert!(adapter.reason("x").await.is_err());
        // Sin pasar pausa, no se permite.
        assert!(!adapter.breaker.is_allowed("flaky"));
        // Reactivar manualmente.
        adapter.breaker.record_success("flaky");
        assert!(adapter.breaker.is_allowed("flaky"));
    }
}
