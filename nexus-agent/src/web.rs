// ============================================================================
// NEXUS-AGENT · web.rs — Búsqueda y extracción de contenido web
// ============================================================================
// Patrón absorbido de Hermes: búsqueda multi-motor y extracción de texto.
// Implementación soberana sin API keys:
//   - `buscar`: DuckDuckGo Lite (HTML plano, sin JavaScript) — parseo con
//     regex, sin dependencias de scraping. Devuelve título + URL + snippet.
//   - `extraer`: GET de una URL y limpieza de HTML a texto plano legible
//     (quita scripts, estilos, tags; decodifica entidades; colapsa blancos).
//
// Límites de seguridad: timeout de red, tamaño máximo de respuesta y cota de
// resultados, para que el agente no se cuelgue ni inunde el contexto.
// ============================================================================

use crate::ejecutor::ResultadoHerramienta;
use anyhow::{anyhow, Context, Result};
use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";
const MAX_BYTES_RESPUESTA: usize = 2 * 1024 * 1024; // 2 MiB
const MAX_SNIPPET_CHARS: usize = 200;

/// Cliente web con timeouts y cotas configurables.
#[derive(Debug, Clone)]
pub struct ClienteWeb {
    pub timeout_seg: u64,
    pub max_resultados: usize,
}

impl Default for ClienteWeb {
    fn default() -> Self {
        Self { timeout_seg: 20, max_resultados: 5 }
    }
}

/// Un resultado de búsqueda.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultadoBusqueda {
    pub titulo: String,
    pub url: String,
    pub snippet: String,
}

impl ClienteWeb {
    /// Busca en Brave Search (HTML plano) y devuelve los resultados.
    ///
    /// Motor por defecto: Brave — responde sin API key ni JavaScript y sirve
    /// resultados orgánicos a conexiones de datacenter (verificado en vivo;
    /// DDG lite y Bing devuelven páginas vacías/consentimiento).
    pub async fn buscar(&self, consulta: &str) -> Result<Vec<ResultadoBusqueda>> {
        let cliente = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(self.timeout_seg))
            .build()
            .context("No se pudo construir el cliente HTTP")?;

        let url = format!(
            "https://search.brave.com/search?q={}",
            urlencode(consulta)
        );
        let respuesta = cliente
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Fallo la petición a {url}"))?;
        let cuerpo = respuesta
            .bytes()
            .await
            .context("No se pudo leer la respuesta")?;
        if cuerpo.len() > MAX_BYTES_RESPUESTA {
            return Err(anyhow!("Respuesta demasiado grande ({} bytes)", cuerpo.len()));
        }
        let html = String::from_utf8_lossy(&cuerpo).to_string();
        Ok(parsear_brave(&html, self.max_resultados))
    }

    /// Extrae el texto legible de una URL (HTML → texto plano).
    pub async fn extraer(&self, url: &str) -> Result<String> {
        let cliente = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(self.timeout_seg))
            .build()
            .context("No se pudo construir el cliente HTTP")?;
        let respuesta = cliente
            .get(url)
            .send()
            .await
            .with_context(|| format!("Fallo la petición a {url}"))?;
        let cuerpo = respuesta
            .bytes()
            .await
            .context("No se pudo leer la respuesta")?;
        if cuerpo.len() > MAX_BYTES_RESPUESTA {
            return Err(anyhow!("Respuesta demasiado grande ({} bytes)", cuerpo.len()));
        }
        let html = String::from_utf8_lossy(&cuerpo).to_string();
        Ok(html_a_texto(&html))
    }

    /// Formatea los resultados como observación legible para el modelo.
    pub fn formatear_resultados(&self, resultados: &[ResultadoBusqueda]) -> String {
        if resultados.is_empty() {
            return "Sin resultados para la búsqueda.".to_string();
        }
        let mut out = String::from("🔎 RESULTADOS DE BÚSQUEDA:\n");
        for (i, r) in resultados.iter().enumerate() {
            out.push_str(&format!(
                "{}. {}\n   URL: {}\n   {}\n",
                i + 1,
                r.titulo,
                r.url,
                r.snippet
            ));
        }
        out
    }

    /// Variante de `buscar` que devuelve directamente la observación.
    pub async fn buscar_como_observacion(&self, consulta: &str) -> Result<ResultadoHerramienta> {
        match self.buscar(consulta).await {
            Ok(resultados) => Ok(ResultadoHerramienta::exito(
                self.formatear_resultados(&resultados),
            )),
            Err(e) => Ok(ResultadoHerramienta::fallo(format!("Búsqueda fallida: {e}"))),
        }
    }

    /// Variante de `extraer` que devuelve directamente la observación.
    pub async fn extraer_como_observacion(&self, url: &str) -> Result<ResultadoHerramienta> {
        match self.extraer(url).await {
            Ok(texto) => Ok(ResultadoHerramienta::exito(texto)),
            Err(e) => Ok(ResultadoHerramienta::fallo(format!("Extracción fallida: {e}"))),
        }
    }
}

/// URL-encode simple (percent-encoding de caracteres no seguros).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Parsea la página de resultados de Brave Search.
///
/// Cada resultado es un bloque `<div class="result-wrapper ...">` con:
///   <a href="URL">nombre-sitio › ruta   Título</a>
///   <div class="snippet ...">snippet</div>
/// Estrategia robusta: dividir por bloques `result-wrapper`, extraer el
/// primer enlace http (URL + texto) y el primer div `snippet`.
fn parsear_brave(html: &str, max: usize) -> Vec<ResultadoBusqueda> {
    let re_enlace = regex::Regex::new(r#"(?is)<a[^>]+href="(https?://[^"]+)"[^>]*>(.*?)</a>"#).unwrap();
    let re_snippet = regex::Regex::new(r#"(?is)<div class="snippet[^"]*"[^>]*>(.*?)</div>"#).unwrap();

    let mut resultados = Vec::new();
    for bloque in html.split("class=\"result-wrapper").skip(1) {
        if resultados.len() >= max {
            break;
        }
        if bloque.is_empty() {
            continue;
        }
        // Primer enlace http del bloque (el título del resultado)
        let (url, titulo) = match re_enlace.captures(bloque) {
            Some(c) => {
                let url = limpiar_html(&c[1]);
                // El contenido del enlace trae divs anidados (favicon, nombre
                // del sitio): se pasa por texto_plano para quitar los tags.
                let titulo = texto_plano(&c[2]);
                if url.is_empty() || titulo.is_empty() {
                    continue;
                }
                (url, titulo)
            }
            None => continue,
        };
        // Primer div snippet del bloque (también puede traer spans anidados)
        let snippet: String = re_snippet
            .captures(bloque)
            .map(|c| texto_plano(&c[1]))
            .unwrap_or_default()
            .chars()
            .take(MAX_SNIPPET_CHARS)
            .collect();
        resultados.push(ResultadoBusqueda { titulo, url, snippet });
    }
    resultados
}

/// Convierte un fragmento HTML en texto plano legible: quita tags y
/// decodifica entidades (para títulos y snippets que traen HTML anidado).
fn texto_plano(html: &str) -> String {
    let sin_tags = regex::Regex::new(r"(?is)<[^>]+>")
        .unwrap()
        .replace_all(html, " ");
    limpiar_html(&sin_tags)
}

/// HTML → texto plano: quita scripts/estilos/tags, decodifica entidades y
/// colapsa el whitespace.
fn html_a_texto(html: &str) -> String {
    // Sin backreferences (la crate regex no las soporta): la lista de
    // etiquetas se duplica en apertura y cierre.
    let sin_scripts = regex::Regex::new(
        r"(?is)<(script|style|noscript|svg|head)\b[^>]*>.*?</(script|style|noscript|svg|head)>",
    )
    .unwrap()
    .replace_all(html, " ");
    let sin_tags = regex::Regex::new(r"(?is)<[^>]+>")
        .unwrap()
        .replace_all(&sin_scripts, " ");
    let limpio = limpiar_html(&sin_tags);
    let colapsado = regex::Regex::new(r"[ \t\r\n]+")
        .unwrap()
        .replace_all(&limpio, " ");
    colapsado.trim().to_string()
}

/// Decodifica entidades HTML básicas y normaliza espacios.
fn limpiar_html(s: &str) -> String {
    let mut out = s
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");
    // Entidades numéricas simples &#NN;
    let re_num = regex::Regex::new(r"&#(\d+);").unwrap();
    out = re_num
        .replace_all(&out, |caps: &regex::Captures| {
            caps[1]
                .parse::<u32>()
                .ok()
                .and_then(char::from_u32)
                .map(|c| c.to_string())
                .unwrap_or_default()
        })
        .to_string();
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_codifica_caracteres() {
        assert_eq!(urlencode("hola mundo"), "hola+mundo");
        assert_eq!(urlencode("a/b?c"), "a%2Fb%3Fc");
        assert_eq!(urlencode("abc-_.~"), "abc-_.~");
    }

    #[test]
    fn parsea_resultados_de_brave() {
        let html = r#"<html><body>
            <div class="result-wrapper svelte-x">
              <a href="https://rust-lang.org">rust-lang.org › en-US   Rust Programming Language</a>
              <div class="snippet svelte-y">Lenguaje de sistemas con garantías de memoria</div>
            </div>
            <div class="result-wrapper svelte-x">
              <a href="https://example.com/a?b=1&amp;c=2">Ejemplo &amp; Co</a>
              <div class="snippet svelte-y">Un sitio de ejemplo &#39;simple&#39;</div>
            </div>
            </body></html>"#;
        let resultados = parsear_brave(html, 5);
        assert_eq!(resultados.len(), 2);
        assert_eq!(resultados[0].titulo, "rust-lang.org › en-US Rust Programming Language");
        assert_eq!(resultados[0].url, "https://rust-lang.org");
        assert!(resultados[0].snippet.contains("garantías"));
        assert_eq!(resultados[1].titulo, "Ejemplo & Co");
        assert_eq!(resultados[1].url, "https://example.com/a?b=1&c=2");
        assert!(resultados[1].snippet.contains("simple"));
    }

    #[test]
    fn limite_de_resultados() {
        let html = r#"<div class="result-wrapper"><a href="https://a.com">A</a></div>
            <div class="result-wrapper"><a href="https://b.com">B</a></div>
            <div class="result-wrapper"><a href="https://c.com">C</a></div>"#;
        let resultados = parsear_brave(html, 2);
        assert_eq!(resultados.len(), 2);
    }

    #[test]
    fn html_a_texto_limpia_bien() {
        let html = r#"<html><head><style>p{color:red}</style><script>alert(1)</script></head>
            <body><h1>Título</h1><p>Hola <b>mundo</b> &amp; amigos.</p></body></html>"#;
        let texto = html_a_texto(html);
        assert!(texto.contains("Título"));
        assert!(texto.contains("Hola mundo & amigos."));
        assert!(!texto.contains("<"));
        assert!(!texto.contains("alert"));
    }

    #[test]
    fn sin_resultados_devuelve_vacio() {
        let html = "<html><body>nada aqui</body></html>";
        assert!(parsear_brave(html, 5).is_empty());
    }
}
