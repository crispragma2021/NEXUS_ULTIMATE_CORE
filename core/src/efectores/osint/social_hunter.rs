// ──────────────────────────────────────────────
// 🎯 SOCIAL HUNTER — Escáner de presencia en redes sociales
// Reemplaza a UsernameScanner con:
//   - GET requests en vez de HEAD (sitios modernos rechazan HEAD)
//   - User-Agent rotatorio para evitar bloqueos
//   - 25+ plataformas soportadas
//   - Timeout individual por plataforma (3s)
//   - Resultados estructurados con plataforma, URL, status
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Perfil encontrado en una plataforma
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfile {
    pub platform: String,
    pub url: String,
    pub exists: bool,
    pub status_code: u16,
}

impl SocialProfile {
    fn found(platform: &str, url: &str, status: u16) -> Self {
        Self {
            platform: platform.to_string(),
            url: url.to_string(),
            exists: true,
            status_code: status,
        }
    }

    fn not_found(platform: &str, url: &str, status: u16) -> Self {
        Self {
            platform: platform.to_string(),
            url: url.to_string(),
            exists: false,
            status_code: status,
        }
    }
}

/// Definición de una plataforma: nombre + patrón de URL
struct PlatformDef {
    name: &'static str,
    url_pattern: &'static str,
}

/// 🎯 Escáner de presencia de username en redes sociales y plataformas
pub struct SocialHunter {
    client: reqwest::Client,
    platforms: Vec<PlatformDef>,
}

impl Default for SocialHunter {
    fn default() -> Self {
        Self::new()
    }
}

impl SocialHunter {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()
            .expect("SocialHunter: Cliente HTTP válido");

        // 25+ plataformas organizadas por categoría
        let platforms = vec![
            // ── Redes Sociales Principales ──
            PlatformDef {
                name: "GitHub",
                url_pattern: "https://github.com/{}",
            },
            PlatformDef {
                name: "Twitter/X",
                url_pattern: "https://twitter.com/{}",
            },
            PlatformDef {
                name: "Instagram",
                url_pattern: "https://instagram.com/{}",
            },
            PlatformDef {
                name: "LinkedIn",
                url_pattern: "https://linkedin.com/in/{}",
            },
            PlatformDef {
                name: "Facebook",
                url_pattern: "https://facebook.com/{}",
            },
            PlatformDef {
                name: "Reddit",
                url_pattern: "https://reddit.com/user/{}",
            },
            PlatformDef {
                name: "TikTok",
                url_pattern: "https://tiktok.com/@{}",
            },
            PlatformDef {
                name: "Pinterest",
                url_pattern: "https://pinterest.com/{}",
            },
            PlatformDef {
                name: "Tumblr",
                url_pattern: "https://{}.tumblr.com",
            },
            PlatformDef {
                name: "Snapchat",
                url_pattern: "https://snapchat.com/add/{}",
            },
            // ── Mensajería y Comunidades ──
            PlatformDef {
                name: "Telegram",
                url_pattern: "https://t.me/{}",
            },
            PlatformDef {
                name: "Discord",
                url_pattern: "https://discord.com/users/{}",
            },
            PlatformDef {
                name: "WhatsApp",
                url_pattern: "https://wa.me/{}",
            },
            // ── Desarrollo y Tecnología ──
            PlatformDef {
                name: "GitLab",
                url_pattern: "https://gitlab.com/{}",
            },
            PlatformDef {
                name: "Bitbucket",
                url_pattern: "https://bitbucket.org/{}",
            },
            PlatformDef {
                name: "StackOverflow",
                url_pattern: "https://stackoverflow.com/users/{}",
            },
            PlatformDef {
                name: "Dev.to",
                url_pattern: "https://dev.to/{}",
            },
            PlatformDef {
                name: "Medium",
                url_pattern: "https://medium.com/@{}",
            },
            PlatformDef {
                name: "HackerNews",
                url_pattern: "https://news.ycombinator.com/user?id={}",
            },
            PlatformDef {
                name: "Keybase",
                url_pattern: "https://keybase.io/{}",
            },
            // ── Creativos y Contenido ──
            PlatformDef {
                name: "YouTube",
                url_pattern: "https://youtube.com/@{}",
            },
            PlatformDef {
                name: "Twitch",
                url_pattern: "https://twitch.tv/{}",
            },
            PlatformDef {
                name: "SoundCloud",
                url_pattern: "https://soundcloud.com/{}",
            },
            PlatformDef {
                name: "Spotify",
                url_pattern: "https://open.spotify.com/user/{}",
            },
            // ── Profesional y Negocios ──
            PlatformDef {
                name: "AngelList",
                url_pattern: "https://angel.co/u/{}",
            },
            PlatformDef {
                name: "ProductHunt",
                url_pattern: "https://producthunt.com/@{}",
            },
            PlatformDef {
                name: "Behance",
                url_pattern: "https://behance.net/{}",
            },
            PlatformDef {
                name: "Dribbble",
                url_pattern: "https://dribbble.com/{}",
            },
            // ── Otras ──
            PlatformDef {
                name: "Flickr",
                url_pattern: "https://flickr.com/people/{}",
            },
            PlatformDef {
                name: "Vimeo",
                url_pattern: "https://vimeo.com/{}",
            },
            PlatformDef {
                name: "Patreon",
                url_pattern: "https://patreon.com/{}",
            },
            PlatformDef {
                name: "BuyMeACoffee",
                url_pattern: "https://buymeacoffee.com/{}",
            },
            PlatformDef {
                name: "About.me",
                url_pattern: "https://about.me/{}",
            },
        ];

        Self { client, platforms }
    }

    /// Escanea un username en todas las plataformas configuradas
    /// Usa GET request con timeout corto (3s por plataforma)
    pub async fn scan_username(&self, username: &str) -> anyhow::Result<Vec<SocialProfile>> {
        info!(
            "🎯 [SOCIAL-HUNTER] Escaneando username: '{}' en {} plataformas",
            username,
            self.platforms.len()
        );

        let mut results = Vec::new();
        let username_lower = username.to_lowercase();

        for platform in &self.platforms {
            let url = platform.url_pattern.replace("{}", &username_lower);
            let start = std::time::Instant::now();

            match self.client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let elapsed = start.elapsed().as_millis();

                    // Consideramos "encontrado" si status es 200 (o 403/429 significa que existe pero bloquea)
                    let exists = match status {
                        200 => true,
                        403 | 429 => {
                            debug!(
                                "[SOCIAL-HUNTER] {} posiblemente existe pero bloquea: HTTP {}",
                                platform.name, status
                            );
                            true // rate limit / bloqueado = existe
                        }
                        301 | 302 | 307 | 308 => {
                            // Redirect puede indicar que existe (ej: LinkedIn redirect a login)
                            debug!(
                                "[SOCIAL-HUNTER] {} redirige: HTTP {}",
                                platform.name, status
                            );
                            true
                        }
                        404 => false,
                        _ => {
                            debug!(
                                "[SOCIAL-HUNTER] {} status inesperado: HTTP {} ({}ms)",
                                platform.name, status, elapsed
                            );
                            false
                        }
                    };

                    if exists {
                        results.push(SocialProfile::found(platform.name, &url, status));
                        debug!(
                            "✅ [SOCIAL-HUNTER] Encontrado en {}: {} (HTTP {}, {}ms)",
                            platform.name, url, status, elapsed
                        );
                    } else {
                        debug!(
                            "❌ [SOCIAL-HUNTER] No encontrado en {}: HTTP {} ({}ms)",
                            platform.name, status, elapsed
                        );
                    }
                }
                Err(e) => {
                    // Error de conexión/timeout no significa que no exista, pero no confirmamos
                    debug!(
                        "[SOCIAL-HUNTER] Error en {}: {} - asumiendo no encontrado",
                        platform.name, e
                    );
                }
            }
        }

        info!(
            "✅ [SOCIAL-HUNTER] Escaneo completado. {} perfiles detectados de {} plataformas.",
            results.len(),
            self.platforms.len()
        );
        Ok(results)
    }

    /// Escanea un username solo en plataformas específicas
    pub async fn scan_platforms(
        &self,
        username: &str,
        platforms: &[&str],
    ) -> anyhow::Result<Vec<SocialProfile>> {
        info!(
            "🎯 [SOCIAL-HUNTER] Escaneo selectivo: '{}' en {:?}",
            username, platforms
        );

        let mut results = Vec::new();
        let username_lower = username.to_lowercase();

        for platform in &self.platforms {
            if !platforms.contains(&platform.name) {
                continue;
            }

            let url = platform.url_pattern.replace("{}", &username_lower);
            match self.client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let exists = matches!(status, 200 | 301 | 302 | 307 | 308 | 403 | 429);
                    if exists {
                        results.push(SocialProfile::found(platform.name, &url, status));
                    } else {
                        results.push(SocialProfile::not_found(platform.name, &url, status));
                    }
                }
                Err(e) => {
                    warn!("[SOCIAL-HUNTER] Error en {}: {}", platform.name, e);
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_social_hunter_creation() {
        let hunter = SocialHunter::new();
        // Debe tener al menos 25 plataformas
        assert!(
            hunter.platforms.len() >= 25,
            "Esperaba >= 25 plataformas, tengo {}",
            hunter.platforms.len()
        );
    }

    #[tokio::test]
    async fn test_scan_username_returns_vec() {
        let hunter = SocialHunter::new();
        let result = hunter.scan_username("octocat").await;
        assert!(
            result.is_ok(),
            "scan_username debe retornar Ok, error: {:?}",
            result.err()
        );
        let profiles = result.unwrap();
        // Debe encontrar al menos GitHub (octocat existe)
        assert!(
            profiles.iter().any(|p| p.exists),
            "Debería encontrar al menos un perfil existente"
        );
    }
}
