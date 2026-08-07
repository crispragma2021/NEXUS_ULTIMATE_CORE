// ============================================================================
// 🧬 NEXUS CAPTCHA Orchestrator — Decisión y Estrategia de Resolución
// ============================================================================
// Detecta CAPTCHA en página, decide estrategia (biométrica / API externa),
// gestiona rotación de IP/perfil y coordina la resolución completa.
// ============================================================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::api_client::{
    CaptchaApiClient, CaptchaProvider, CaptchaResult, CaptchaTaskParams, CaptchaType,
};

// ---------------------------------------------------------------------------
// Estrategias de resolución
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionStrategy {
    /// Solo evasión biométrica + rotación IP/Perfil (sin API externa)
    EvasionOnly,
    /// Usar API externa de resolución (Capsolver / 2Captcha)
    ApiExternal,
    /// Evasión primero, si falla → API externa
    EvasionThenApi,
    /// Solo rotación de identidad/IP
    RotateOnly,
    /// Desactivado
    None,
}

// ---------------------------------------------------------------------------
// Tipos de CAPTCHA detectables
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectedCaptcha {
    GoogleRecaptchaV2,
    GoogleRecaptchaV3,
    HCaptcha,
    CloudflareTurnstile,
    CloudflareChallenge,
    FunCaptcha,
    Generic,
    None,
}

impl DetectedCaptcha {
    /// Convierte a CaptchaType para la API externa.
    pub fn to_api_type(&self) -> Option<CaptchaType> {
        match self {
            Self::GoogleRecaptchaV2 => Some(CaptchaType::RecaptchaV2),
            Self::GoogleRecaptchaV3 => Some(CaptchaType::RecaptchaV3),
            Self::HCaptcha => Some(CaptchaType::HCaptcha),
            Self::CloudflareTurnstile => Some(CaptchaType::Turnstile),
            Self::FunCaptcha => Some(CaptchaType::FunCaptcha),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Configuración del Orchestrator
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaOrchestratorConfig {
    /// Estrategia de resolución por defecto
    pub default_strategy: ResolutionStrategy,
    /// Tiempo máximo de espera para evasión biométrica (ms)
    pub evasion_timeout_ms: u64,
    /// Número máximo de rotaciones de IP antes de fallback a API
    pub max_rotations_before_api: u32,
    /// Proveedor externo (Capsolver / 2Captcha)
    pub api_provider: Option<CaptchaProvider>,
    /// API key del proveedor externo
    pub api_key: Option<String>,
    /// Umbral de confianza para reCAPTCHA v3 (0.0 - 1.0)
    pub recaptcha_v3_threshold: f64,
}

impl Default for CaptchaOrchestratorConfig {
    fn default() -> Self {
        Self {
            default_strategy: ResolutionStrategy::EvasionThenApi,
            evasion_timeout_ms: 15_000,
            max_rotations_before_api: 3,
            api_provider: Some(CaptchaProvider::Capsolver),
            api_key: None,
            recaptcha_v3_threshold: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Resultado de la orquestación
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// Éxito de la operación
    pub success: bool,
    /// Estrategia utilizada
    pub strategy_used: ResolutionStrategy,
    /// CAPTCHA detectado
    pub captcha_detected: DetectedCaptcha,
    /// Token resuelto (si aplica)
    pub token: Option<String>,
    /// Número de rotaciones realizadas
    pub rotations_performed: u32,
    /// Tiempo total en ms
    pub total_time_ms: u64,
    /// Mensaje descriptivo
    pub message: String,
}

// ---------------------------------------------------------------------------
// Callbacks de integración con el navegador
// ---------------------------------------------------------------------------

/// Interfaz de comunicación con el navegador para detección y evasión.
/// Debe ser implementada por el integrador (nexus_browser_mcp, etc.)
#[async_trait::async_trait]
pub trait BrowserBridge: Send + Sync {
    /// Detecta qué tipo de CAPTCHA hay en la página actual.
    /// Retorna el tipo detectado y los parámetros (site_key, action, etc.)
    async fn detect_captcha(&self) -> Result<DetectedCaptchaResult>;

    /// Intenta evasión biométrica (movimiento humano, scroll, etc.)
    async fn evade_biometric(&self) -> Result<bool>;

    /// Inyecta un token de resolución en la página.
    async fn inject_token(&self, token: &str) -> Result<bool>;

    /// Rotación de IP (Tor) o perfil de identidad.
    async fn rotate_profile(&self) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct DetectedCaptchaResult {
    pub captcha: DetectedCaptcha,
    pub site_key: Option<String>,
    pub action: Option<String>,
    pub url: String,
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub struct CaptchaOrchestrator {
    config: CaptchaOrchestratorConfig,
    api_client: Option<CaptchaApiClient>,
    stats: OrchestratorStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchestratorStats {
    pub total_attempts: u64,
    pub evasion_success: u64,
    pub api_success: u64,
    pub api_failures: u64,
    pub rotations_performed: u64,
}

impl CaptchaOrchestrator {
    /// Crea un nuevo orquestador con la configuración dada.
    pub fn new(config: CaptchaOrchestratorConfig) -> Self {
        let api_client = match (&config.api_key, config.api_provider) {
            (Some(key), Some(provider)) => {
                Some(CaptchaApiClient::new(key.clone(), provider, None, None))
            }
            _ => None,
        };

        Self {
            config,
            api_client,
            stats: OrchestratorStats::default(),
        }
    }

    /// Punto de entrada principal: orquesta la resolución de CAPTCHA.
    ///
    /// 1. Detecta tipo de CAPTCHA
    /// 2. Decide estrategia según configuración
    /// 3. Ejecuta: evasión biométrica → rotación IP → API externa
    /// 4. Retorna resultado
    pub async fn resolve<B: BrowserBridge>(
        &mut self,
        bridge: &B,
        override_strategy: Option<ResolutionStrategy>,
    ) -> OrchestrationResult {
        let start = Instant::now();
        let strategy = override_strategy.unwrap_or(self.config.default_strategy);

        // 1. Detectar CAPTCHA
        let detection = match bridge.detect_captcha().await {
            Ok(d) => d,
            Err(e) => {
                return OrchestrationResult {
                    success: false,
                    strategy_used: strategy,
                    captcha_detected: DetectedCaptcha::None,
                    token: None,
                    rotations_performed: 0,
                    total_time_ms: start.elapsed().as_millis() as u64,
                    message: format!("Error detectando CAPTCHA: {}", e),
                };
            }
        };

        self.stats.total_attempts += 1;

        if detection.captcha == DetectedCaptcha::None {
            return OrchestrationResult {
                success: true,
                strategy_used: ResolutionStrategy::None,
                captcha_detected: DetectedCaptcha::None,
                token: None,
                rotations_performed: 0,
                total_time_ms: start.elapsed().as_millis() as u64,
                message: "No se detectó CAPTCHA en la página".into(),
            };
        }

        match strategy {
            ResolutionStrategy::None => {
                return OrchestrationResult {
                    success: false,
                    strategy_used: ResolutionStrategy::None,
                    captcha_detected: detection.captcha.clone(),
                    token: None,
                    rotations_performed: 0,
                    total_time_ms: start.elapsed().as_millis() as u64,
                    message: format!(
                        "Estrategia desactivada. CAPTCHA detectado: {:?}",
                        detection.captcha.clone()
                    ),
                };
            }

            ResolutionStrategy::RotateOnly => {
                // Solo rotar — útil para cuando el CAPTCHA es temporal
                let rotated = bridge.rotate_profile().await.unwrap_or(false);
                self.stats.rotations_performed += 1;

                return OrchestrationResult {
                    success: rotated,
                    strategy_used: ResolutionStrategy::RotateOnly,
                    captcha_detected: detection.captcha,
                    token: None,
                    rotations_performed: 1,
                    total_time_ms: start.elapsed().as_millis() as u64,
                    message: if rotated {
                        "Perfil rotado para evitar CAPTCHA".into()
                    } else {
                        "Fallo al rotar perfil".into()
                    },
                };
            }

            ResolutionStrategy::EvasionOnly => {
                let evaded = bridge.evade_biometric().await.unwrap_or(false);
                if evaded {
                    self.stats.evasion_success += 1;
                }

                return OrchestrationResult {
                    success: evaded,
                    strategy_used: ResolutionStrategy::EvasionOnly,
                    captcha_detected: detection.captcha,
                    token: None,
                    rotations_performed: 0,
                    total_time_ms: start.elapsed().as_millis() as u64,
                    message: if evaded {
                        "CAPTCHA evadido biométricamente".into()
                    } else {
                        "Evasión biométrica fallida".into()
                    },
                };
            }

            ResolutionStrategy::EvasionThenApi => {
                // FASE 1: Evasión biométrica
                let evaded = bridge.evade_biometric().await.unwrap_or(false);
                if evaded {
                    self.stats.evasion_success += 1;
                    return OrchestrationResult {
                        success: true,
                        strategy_used: ResolutionStrategy::EvasionThenApi,
                        captcha_detected: detection.captcha,
                        token: None,
                        rotations_performed: 0,
                        total_time_ms: start.elapsed().as_millis() as u64,
                        message: "CAPTCHA evadido biométricamente (Fase 1)".into(),
                    };
                }

                // FASE 2: Rotar IP/Perfil (múltiples intentos)
                let mut rotations = 0u32;
                for _ in 0..self.config.max_rotations_before_api {
                    let rotated = bridge.rotate_profile().await.unwrap_or(false);
                    rotations += 1;
                    self.stats.rotations_performed += 1;

                    if rotated {
                        // Re-intentar evasión biométrica después de rotar
                        let re_evaded = bridge.evade_biometric().await.unwrap_or(false);
                        if re_evaded {
                            self.stats.evasion_success += 1;
                            return OrchestrationResult {
                                success: true,
                                strategy_used: ResolutionStrategy::EvasionThenApi,
                                captcha_detected: detection.captcha,
                                token: None,
                                rotations_performed: rotations,
                                total_time_ms: start.elapsed().as_millis() as u64,
                                message: format!("CAPTCHA evadido tras rotación #{}", rotations),
                            };
                        }
                    }
                }

                // FASE 3: API externa si está configurada
                if let Some(api) = &self.api_client {
                    if let Some(api_type) = detection.captcha.to_api_type() {
                        match self.resolve_via_api(api, api_type, &detection).await {
                            Ok(result) => {
                                self.stats.api_success += 1;

                                // Inyectar token en la página
                                if let Some(ref token) = result.token {
                                    let _ = bridge.inject_token(token).await;
                                }

                                return OrchestrationResult {
                                    success: true,
                                    strategy_used: ResolutionStrategy::EvasionThenApi,
                                    captcha_detected: detection.captcha,
                                    token: result.token,
                                    rotations_performed: rotations,
                                    total_time_ms: start.elapsed().as_millis() as u64,
                                    message: format!(
                                        "CAPTCHA resuelto vía API externa ({}ms)",
                                        result.resolve_time_ms
                                    ),
                                };
                            }
                            Err(e) => {
                                self.stats.api_failures += 1;
                                return OrchestrationResult {
                                    success: false,
                                    strategy_used: ResolutionStrategy::EvasionThenApi,
                                    captcha_detected: detection.captcha,
                                    token: None,
                                    rotations_performed: rotations,
                                    total_time_ms: start.elapsed().as_millis() as u64,
                                    message: format!("Evasión fallida + API falló: {}", e),
                                };
                            }
                        }
                    }
                }

                // Sin API configurada — reportar fallo
                OrchestrationResult {
                    success: false,
                    strategy_used: ResolutionStrategy::EvasionThenApi,
                    captcha_detected: detection.captcha,
                    token: None,
                    rotations_performed: rotations,
                    total_time_ms: start.elapsed().as_millis() as u64,
                    message: "Evasión + rotaciones fallidas, sin API externa configurada".into(),
                }
            }

            ResolutionStrategy::ApiExternal => {
                if let Some(api) = &self.api_client {
                    if let Some(api_type) = detection.captcha.to_api_type() {
                        match self.resolve_via_api(api, api_type, &detection).await {
                            Ok(result) => {
                                self.stats.api_success += 1;

                                if let Some(ref token) = result.token {
                                    let _ = bridge.inject_token(token).await;
                                }

                                return OrchestrationResult {
                                    success: true,
                                    strategy_used: ResolutionStrategy::ApiExternal,
                                    captcha_detected: detection.captcha,
                                    token: result.token,
                                    rotations_performed: 0,
                                    total_time_ms: start.elapsed().as_millis() as u64,
                                    message: format!(
                                        "CAPTCHA resuelto vía API externa ({}ms)",
                                        result.resolve_time_ms
                                    ),
                                };
                            }
                            Err(e) => {
                                self.stats.api_failures += 1;
                                return OrchestrationResult {
                                    success: false,
                                    strategy_used: ResolutionStrategy::ApiExternal,
                                    captcha_detected: detection.captcha,
                                    token: None,
                                    rotations_performed: 0,
                                    total_time_ms: start.elapsed().as_millis() as u64,
                                    message: format!("API externa falló: {}", e),
                                };
                            }
                        }
                    }
                }

                OrchestrationResult {
                    success: false,
                    strategy_used: ResolutionStrategy::ApiExternal,
                    captcha_detected: detection.captcha,
                    token: None,
                    rotations_performed: 0,
                    total_time_ms: start.elapsed().as_millis() as u64,
                    message: "API externa no configurada o tipo no soportado".into(),
                }
            }
        }
    }

    /// Resuelve vía API externa.
    async fn resolve_via_api(
        &self,
        api: &CaptchaApiClient,
        captcha_type: CaptchaType,
        detection: &DetectedCaptchaResult,
    ) -> Result<CaptchaResult> {
        let params =
            CaptchaTaskParams::new(&detection.url, detection.site_key.as_deref().unwrap_or(""));

        let params = match captcha_type {
            CaptchaType::RecaptchaV3 => params.with_min_score(self.config.recaptcha_v3_threshold),
            _ => params,
        };

        api.solve(captcha_type, params).await
    }

    /// Estadísticas del orquestador.
    pub fn stats(&self) -> &OrchestratorStats {
        &self.stats
    }

    /// Resetea las estadísticas.
    pub fn reset_stats(&mut self) {
        self.stats = OrchestratorStats::default();
    }

    /// Configura o cambia la API key en runtime.
    pub fn set_api_key(&mut self, key: String, provider: CaptchaProvider) {
        self.config.api_key = Some(key.clone());
        self.config.api_provider = Some(provider);
        self.api_client = Some(CaptchaApiClient::new(key, provider, None, None));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock Bridge para testing
    struct MockBridge {
        captcha: DetectedCaptcha,
        evade_success: bool,
        rotate_success: bool,
    }

    #[async_trait::async_trait]
    impl BrowserBridge for MockBridge {
        async fn detect_captcha(&self) -> Result<DetectedCaptchaResult> {
            Ok(DetectedCaptchaResult {
                captcha: self.captcha.clone(),
                site_key: Some("6Lc_test_key".into()),
                action: None,
                url: "https://example.com".into(),
            })
        }

        async fn evade_biometric(&self) -> Result<bool> {
            Ok(self.evade_success)
        }

        async fn inject_token(&self, _token: &str) -> Result<bool> {
            Ok(true)
        }

        async fn rotate_profile(&self) -> Result<bool> {
            Ok(self.rotate_success)
        }
    }

    #[tokio::test]
    async fn test_no_captcha_detected() {
        let config = CaptchaOrchestratorConfig::default();
        let mut orch = CaptchaOrchestrator::new(config);

        let bridge = MockBridge {
            captcha: DetectedCaptcha::None,
            evade_success: false,
            rotate_success: false,
        };

        let result = orch.resolve::<MockBridge>(&bridge, None).await;
        assert!(result.success);
        assert_eq!(result.captcha_detected, DetectedCaptcha::None);
        assert_eq!(result.strategy_used, ResolutionStrategy::None);
    }

    #[tokio::test]
    async fn test_evasion_only_success() {
        let config = CaptchaOrchestratorConfig {
            default_strategy: ResolutionStrategy::EvasionOnly,
            ..Default::default()
        };
        let mut orch = CaptchaOrchestrator::new(config);

        let bridge = MockBridge {
            captcha: DetectedCaptcha::GoogleRecaptchaV2,
            evade_success: true,
            rotate_success: false,
        };

        let result = orch.resolve::<MockBridge>(&bridge, None).await;
        assert!(result.success);
        assert_eq!(result.strategy_used, ResolutionStrategy::EvasionOnly);
    }

    #[tokio::test]
    async fn test_evasion_fails_rotation_not_configured() {
        let config = CaptchaOrchestratorConfig {
            default_strategy: ResolutionStrategy::EvasionThenApi,
            api_key: None, // Sin API externa
            ..Default::default()
        };
        let mut orch = CaptchaOrchestrator::new(config);

        let bridge = MockBridge {
            captcha: DetectedCaptcha::HCaptcha,
            evade_success: false,
            rotate_success: true, // Rotación funciona
        };

        let result = orch.resolve::<MockBridge>(&bridge, None).await;
        // Debe fallar porque evasión falla y no hay API configurada
        assert!(!result.success);
        assert_eq!(result.rotations_performed, 3); // Intentó 3 rotaciones
    }
}
