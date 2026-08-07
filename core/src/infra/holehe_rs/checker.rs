// 🔱 holehe_rs — Transmutación Rust Pura de holehe (OSINT email checker)
// Verifica si un email está registrado en múltiples servicios online mediante
// consultas HTTP a endpoints de "password reset" / "forgot password".
//
// Servicios: Adobe, Amazon, Apple, Bitbucket, Blizzard, Booking, Discord,
// Dropbox, eBay, Facebook, GitHub, GitLab, Instagram, LastPass, LinkedIn,
// Netflix, Pinterest, ProtonMail, Reddit, Signal, Snapchat, Spotify,
// Telegram, TikTok, Twitch, Twitter/X, WordPress, Yahoo.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

// ── Tipos ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResult {
    pub service: String,
    pub name: String,
    pub exists: bool,
    pub method: CheckMethod,
    pub status_code: u16,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckMethod {
    HttpStatus,
    BodyContent,
    Header,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoleheResult {
    pub email: String,
    pub results: Vec<ServiceResult>,
    pub total_checks: usize,
    pub registered: usize,
    pub not_registered: usize,
    pub errors: usize,
}

#[derive(Debug, Clone)]
pub struct HoleheConfig {
    pub timeout_secs: u64,
    pub delay_ms: u64,
    pub user_agent: String,
}

impl Default for HoleheConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 15,
            delay_ms: 200,
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36".into(),
        }
    }
}

// ── Helper HTTP ─────────────────────────────────────────────────────

fn http_client(config: &HoleheConfig) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::limited(3))
        .user_agent(&config.user_agent)
        .build()
        .map_err(|e| anyhow!("HTTP client: {}", e))
}

// ── Servicios ────────────────────────────────────────────────────────

/// Lista de servicios con su función de verificación
const SERVICE_LIST: &[ServiceDef] = &[
    ServiceDef {
        id: "adobe",
        name: "Adobe",
        endpoint: "https://auth.services.adobe.com/signup/v1/users/accounts",
        method: MethodDef::PostJson("email"),
    },
    ServiceDef {
        id: "amazon",
        name: "Amazon",
        endpoint: "https://www.amazon.com/ap/forgotpassword",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "apple",
        name: "Apple",
        endpoint: "https://id.apple.com/complete/verify_email",
        method: MethodDef::PostJson("email"),
    },
    ServiceDef {
        id: "bitbucket",
        name: "Bitbucket",
        endpoint: "https://bitbucket.org/account/reset_password/",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "blizzard",
        name: "Blizzard",
        endpoint: "https://account.blizzard.com/forgot-password?email={email}",
        method: MethodDef::GetCheckBody("send"),
    },
    ServiceDef {
        id: "booking",
        name: "Booking",
        endpoint: "https://account.booking.com/lostpassword",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "discord",
        name: "Discord",
        endpoint: "https://discord.com/api/v9/auth/forgot",
        method: MethodDef::PostJsonExists204,
    },
    ServiceDef {
        id: "dropbox",
        name: "Dropbox",
        endpoint: "https://www.dropbox.com/ajax/check_email_registered",
        method: MethodDef::PostFormRegistered,
    },
    ServiceDef {
        id: "ebay",
        name: "eBay",
        endpoint: "https://reg.ebay.com/reg/PartialReset?email={email}",
        method: MethodDef::GetCheckBody("reset"),
    },
    ServiceDef {
        id: "facebook",
        name: "Facebook",
        endpoint: "https://www.facebook.com/login/identify?email={email}",
        method: MethodDef::GetCheckBody("email"),
    },
    ServiceDef {
        id: "github",
        name: "GitHub",
        endpoint: "https://github.com/account_recovery",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "gitlab",
        name: "GitLab",
        endpoint: "https://gitlab.com/users/password/new",
        method: MethodDef::PostForm("user[email]"),
    },
    ServiceDef {
        id: "instagram",
        name: "Instagram",
        endpoint: "https://www.instagram.com/api/v1/web/accounts/account_recovery_send_ajax/",
        method: MethodDef::PostForm("email_or_username"),
    },
    ServiceDef {
        id: "lastpass",
        name: "LastPass",
        endpoint: "https://lastpass.com/enterprise.php",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "linkedin",
        name: "LinkedIn",
        endpoint: "https://www.linkedin.com/check/email",
        method: MethodDef::PostFormExists,
    },
    ServiceDef {
        id: "netflix",
        name: "Netflix",
        endpoint: "https://www.netflix.com/PasswordReset?email={email}",
        method: MethodDef::GetOk,
    },
    ServiceDef {
        id: "pinterest",
        name: "Pinterest",
        endpoint: "https://www.pinterest.com/reset_password/",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "protonmail",
        name: "ProtonMail",
        endpoint: "https://api.protonmail.ch/pks/lookup?op=index&search={email}",
        method: MethodDef::GetCheckBody("pub"),
    },
    ServiceDef {
        id: "reddit",
        name: "Reddit",
        endpoint: "https://www.reddit.com/api/check-email",
        method: MethodDef::PostFormRegisteredJSON,
    },
    ServiceDef {
        id: "signal",
        name: "Signal",
        endpoint: "https://signal.org/check_email?email={email}",
        method: MethodDef::GetCheckBody("registered"),
    },
    ServiceDef {
        id: "snapchat",
        name: "Snapchat",
        endpoint: "https://accounts.snapchat.com/accounts/password_reset_request",
        method: MethodDef::PostForm("email"),
    },
    ServiceDef {
        id: "spotify",
        name: "Spotify",
        endpoint: "https://www.spotify.com/password-reset/?email={email}",
        method: MethodDef::GetOk,
    },
    ServiceDef {
        id: "telegram",
        name: "Telegram",
        endpoint: "https://oauth.telegram.org/auth/check_email",
        method: MethodDef::PostJsonRegistered,
    },
    ServiceDef {
        id: "tiktok",
        name: "TikTok",
        endpoint: "https://www.tiktok.com/api/v1/user/check_email",
        method: MethodDef::PostJsonRegistered,
    },
    ServiceDef {
        id: "twitch",
        name: "Twitch",
        endpoint: "https://passport.twitch.tv/password_resets",
        method: MethodDef::PostJson("email"),
    },
    ServiceDef {
        id: "twitter",
        name: "Twitter/X",
        endpoint: "https://api.twitter.com/i/users/email_available.json",
        method: MethodDef::PostFormRegisteredJSON,
    },
    ServiceDef {
        id: "wordpress",
        name: "WordPress",
        endpoint: "https://public-api.wordpress.com/rest/v1.1/users/email/exists",
        method: MethodDef::PostJsonExists,
    },
    ServiceDef {
        id: "yahoo",
        name: "Yahoo",
        endpoint: "https://login.yahoo.com/account/module?module=account&.src=login&email={email}",
        method: MethodDef::GetOk,
    },
];

struct ServiceDef {
    id: &'static str,
    name: &'static str,
    endpoint: &'static str,
    method: MethodDef,
}

enum MethodDef {
    /// GET, espera HTTP 200 = existe
    GetOk,
    /// GET, busca keyword en body
    GetCheckBody(&'static str),
    /// POST form-encoded con campo = email
    PostForm(&'static str),
    /// POST form, body contiene "registered":true
    PostFormRegistered,
    /// POST form, body contiene "taken":true
    PostFormRegisteredJSON,
    /// POST JSON con campo = email
    PostJson(&'static str),
    /// POST JSON, body contiene "exists":true
    PostJsonExists,
    /// POST JSON, status 204 = existe
    PostJsonExists204,
    /// POST JSON, body contiene "registered"
    PostJsonRegistered,
    /// POST form, body contiene info de existencia
    PostFormExists,
}

fn replace_email(template: &str, email: &str) -> String {
    template.replace("{email}", &urlencoding(email))
}

fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            b' ' => result.push_str("%20"),
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

/// Ejecuta la verificación para un servicio específico
async fn check_service(email: &str, def: &ServiceDef, config: &HoleheConfig) -> ServiceResult {
    let client = match http_client(config) {
        Ok(c) => c,
        Err(e) => {
            return ServiceResult {
                service: def.id.to_string(),
                name: def.name.to_string(),
                exists: false,
                method: CheckMethod::Unknown,
                status_code: 0,
                message: format!("Client error: {}", e),
            };
        }
    };

    let url = replace_email(def.endpoint, email);

    let resp = match &def.method {
        MethodDef::GetOk | MethodDef::GetCheckBody(_) => client.get(&url).send().await,
        MethodDef::PostForm(field) => {
            let body = format!("{}={}", field, urlencoding(email));
            client
                .post(&url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body)
                .send()
                .await
        }
        MethodDef::PostFormExists
        | MethodDef::PostFormRegistered
        | MethodDef::PostFormRegisteredJSON => {
            let body = format!("email={}", urlencoding(email));
            client
                .post(&url)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(body)
                .send()
                .await
        }
        MethodDef::PostJson(field) => {
            let json = serde_json::json!({ *field: email });
            client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&json)
                .send()
                .await
        }
        MethodDef::PostJsonExists
        | MethodDef::PostJsonExists204
        | MethodDef::PostJsonRegistered => {
            let json = serde_json::json!({"email": email});
            client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&json)
                .send()
                .await
        }
    };

    match resp {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            let lower = body.to_lowercase();

            let exists = match &def.method {
                MethodDef::GetOk => status == 200,
                MethodDef::GetCheckBody(keyword) => {
                    status == 200
                        && (lower.contains(keyword)
                            || lower.contains("reset")
                            || lower.contains("email"))
                }
                MethodDef::PostForm(_) => status == 200,
                MethodDef::PostFormExists => status == 200 && !lower.contains("not_registered"),
                MethodDef::PostFormRegistered => {
                    status == 200 && lower.contains("\"registered\":true")
                }
                MethodDef::PostFormRegisteredJSON => {
                    status == 200 && lower.contains("\"taken\":true")
                }
                MethodDef::PostJson(_) => status == 200,
                MethodDef::PostJsonExists => status == 200 && lower.contains("\"exists\":true"),
                MethodDef::PostJsonExists204 => status == 204,
                MethodDef::PostJsonRegistered => status == 200 && lower.contains("registered"),
            };

            let msg = if exists {
                format!("HTTP {} — registered", status)
            } else {
                format!("HTTP {} — not found", status)
            };

            ServiceResult {
                service: def.id.to_string(),
                name: def.name.to_string(),
                exists,
                method: if status == 204 {
                    CheckMethod::HttpStatus
                } else {
                    CheckMethod::BodyContent
                },
                status_code: status,
                message: msg,
            }
        }
        Err(e) => ServiceResult {
            service: def.id.to_string(),
            name: def.name.to_string(),
            exists: false,
            method: CheckMethod::Unknown,
            status_code: 0,
            message: format!("Error: {}", e),
        },
    }
}

// ── Funciones públicas ──────────────────────────────────────────────

/// Verifica un email contra todos los servicios conocidos
pub async fn check_email(email: &str) -> Result<HoleheResult> {
    check_email_with_config(email, &HoleheConfig::default()).await
}

/// Verifica con configuración personalizada
pub async fn check_email_with_config(email: &str, config: &HoleheConfig) -> Result<HoleheResult> {
    if !email.contains('@') {
        return Err(anyhow!("Invalid email: {}", email));
    }

    let mut results = Vec::with_capacity(SERVICE_LIST.len());
    let mut registered = 0usize;
    let mut not_registered = 0usize;
    let mut errors = 0usize;

    info!(
        "[holehe_rs] Checking {} ({} services)",
        email,
        SERVICE_LIST.len()
    );

    for def in SERVICE_LIST {
        debug!("[holehe_rs] {} ...", def.name);
        let result = check_service(email, def, config).await;

        if result.exists {
            registered += 1;
        } else if result.status_code == 0 {
            errors += 1;
        } else {
            not_registered += 1;
        }
        results.push(result);

        if config.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(config.delay_ms)).await;
        }
    }

    info!(
        "[holehe_rs] ✓ {} — {} registered / {} not / {} errors",
        email, registered, not_registered, errors
    );

    Ok(HoleheResult {
        email: email.to_string(),
        results,
        total_checks: SERVICE_LIST.len(),
        registered,
        not_registered,
        errors,
    })
}

/// Filtra servicios donde el email está registrado
pub fn filter_registered(result: &HoleheResult) -> Vec<&ServiceResult> {
    result.results.iter().filter(|r| r.exists).collect()
}

/// Filtra servicios con error
pub fn filter_errors(result: &HoleheResult) -> Vec<&ServiceResult> {
    result
        .results
        .iter()
        .filter(|r| r.status_code == 0)
        .collect()
}

// ══════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlencoding_basic() {
        assert_eq!(urlencoding("test@test.com"), "test%40test.com");
    }

    #[test]
    fn test_urlencoding_spaces() {
        assert_eq!(urlencoding("a b"), "a%20b");
    }

    #[test]
    fn test_replace_email() {
        let result = replace_email("https://example.com?email={email}", "test@test.com");
        assert_eq!(result, "https://example.com?email=test%40test.com");
    }

    #[test]
    fn test_replace_email_no_placeholder() {
        let result = replace_email("https://example.com/endpoint", "test@test.com");
        assert_eq!(result, "https://example.com/endpoint");
    }

    #[test]
    fn test_filter_registered_empty() {
        let result = HoleheResult {
            email: "t@t.com".into(),
            results: vec![ServiceResult {
                service: "test".into(),
                name: "Test".into(),
                exists: false,
                method: CheckMethod::HttpStatus,
                status_code: 404,
                message: "NF".into(),
            }],
            total_checks: 1,
            registered: 0,
            not_registered: 1,
            errors: 0,
        };
        assert!(filter_registered(&result).is_empty());
    }

    #[test]
    fn test_filter_registered_found() {
        let result = HoleheResult {
            email: "t@t.com".into(),
            results: vec![ServiceResult {
                service: "gh".into(),
                name: "GitHub".into(),
                exists: true,
                method: CheckMethod::HttpStatus,
                status_code: 200,
                message: "OK".into(),
            }],
            total_checks: 1,
            registered: 1,
            not_registered: 0,
            errors: 0,
        };
        assert_eq!(filter_registered(&result).len(), 1);
    }

    #[test]
    fn test_filter_errors() {
        let result = HoleheResult {
            email: "t@t.com".into(),
            results: vec![ServiceResult {
                service: "err".into(),
                name: "Error".into(),
                exists: false,
                method: CheckMethod::Unknown,
                status_code: 0,
                message: "timeout".into(),
            }],
            total_checks: 1,
            registered: 0,
            not_registered: 0,
            errors: 1,
        };
        assert_eq!(filter_errors(&result).len(), 1);
    }

    #[test]
    fn test_holehe_config_default() {
        let cfg = HoleheConfig::default();
        assert_eq!(cfg.timeout_secs, 15);
        assert_eq!(cfg.delay_ms, 200);
    }

    #[test]
    fn test_check_method_partial_eq() {
        assert_eq!(CheckMethod::HttpStatus, CheckMethod::HttpStatus);
        assert_ne!(CheckMethod::HttpStatus, CheckMethod::BodyContent);
    }

    #[test]
    fn test_serde_roundtrip() {
        let hr = HoleheResult {
            email: "t@t.com".into(),
            results: vec![ServiceResult {
                service: "gh".into(),
                name: "GitHub".into(),
                exists: true,
                method: CheckMethod::HttpStatus,
                status_code: 200,
                message: "OK".into(),
            }],
            total_checks: 1,
            registered: 1,
            not_registered: 0,
            errors: 0,
        };
        let json = serde_json::to_string(&hr).unwrap();
        let back: HoleheResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.email, "t@t.com");
        assert_eq!(back.registered, 1);
    }

    #[test]
    fn test_service_list_count() {
        assert!(
            SERVICE_LIST.len() >= 20,
            "Expected >=20, got {}",
            SERVICE_LIST.len()
        );
    }

    #[test]
    fn test_service_list_unique_ids() {
        let mut ids = std::collections::HashSet::new();
        for s in SERVICE_LIST {
            assert!(ids.insert(s.id), "Duplicate: {}", s.id);
        }
    }

    #[test]
    fn test_service_list_all_have_endpoints() {
        for s in SERVICE_LIST {
            assert!(!s.endpoint.is_empty(), "Empty endpoint for {}", s.id);
        }
    }
}
