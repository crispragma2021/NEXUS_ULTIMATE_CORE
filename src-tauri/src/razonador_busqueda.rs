// ============================================================================
// 🧬 ÓRGANO DE BÚSQUEDA INTELIGENTE (Tool-Augmented Reasoning)
// ============================================================================
// Permite al modelo local 8B acceder a información externa mediante búsqueda
// web inteligente: clasifica necesidad → genera query optimizada → ejecuta
// herramientas (DuckDuckGo, HTTP scraping) → extrae y razona sobre resultados.
//
// Filosofía DeepSeek: el modelo no busca crudo. Razona QUÉ buscar, CÓMO
// formularlo, FILTRA resultados, y SINTETIZA conocimiento antes de responder.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::time::Instant;
use reqwest::Client;
use serde_json::json;

// ─── Tipos Públicos ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NecesidadBusqueda {
    NoNecesita,
    FactualSimple,
    FactualMultiple,
    DocumentacionTecnica,
    DatosActuales,
    CodigoEjemplo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HerramientaBusqueda {
    DuckDuckGo,
    ScrapingDirecto,
    BrowserNativo,
    MultipleFuentes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryGenerada {
    pub query: String,
    pub fuente: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoBusqueda {
    pub fuente: String,
    pub titulo: String,
    pub contenido: String,
    pub relevancia: f32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoConsolidado {
    pub resultados: Vec<ResultadoBusqueda>,
    pub query_usada: String,
    pub herramienta_usada: HerramientaBusqueda,
    pub latencia_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespuestaBusqueda {
    pub respuesta: String,
    pub confianza: f32,
    pub fuentes_usadas: Vec<String>,
    pub info_insuficiente: bool,
    pub latencia_total_ms: u64,
    pub busqueda_realizada: bool,
}

// ─── Clasificador de Necesidad (Rust puro, 0 LLM) ──────────────────────────

/// Analiza el prompt con heurísticas en Rust para determinar si necesita
/// búsqueda externa. Sin llamadas al modelo — puro algoritmo.
pub fn clasificar_necesidad(prompt: &str) -> NecesidadBusqueda {
    let prompt_lower = prompt.to_lowercase();
    let palabras: Vec<&str> = prompt_lower.split_whitespace().collect();

    // Señales de búsqueda factual
    let factual_clues = [
        "qué es", "quién es", "cuándo fue", "dónde está", "cómo se llama",
        "capital de", "significa", "definición", "historia de",
        "qué significa", "cuál es", "quien es",
    ];

    // Señales de datos actuales / tiempo real
    let actualidad_clues = [
        "último", "última", "noticias", "precio", "cotización",
        "clima", "temperatura", "pronóstico", "hoy", "ahora",
        "latest", "current", "news", "price", "weather",
    ];

    // Señales de documentación técnica
    let docs_clues = [
        "documentación", "docs", "api", "reference", "tutorial",
        "cómo usar", "how to", "install", "setup", "config",
    ];

    // Señales de necesidad de código
    let codigo_clues = [
        "ejemplo", "example", "código", "code snippet", "implementa",
        "patrón", "pattern", "boilerplate",
    ];

    let contiene_factual = factual_clues.iter().any(|c| prompt_lower.contains(c));
    let contiene_actualidad = actualidad_clues.iter().any(|c| prompt_lower.contains(c));
    let contiene_docs = docs_clues.iter().any(|c| prompt_lower.contains(c));
    let contiene_codigo = codigo_clues.iter().any(|c| prompt_lower.contains(c));
    let termina_en_pregunta = prompt_lower.trim().ends_with("?");
    let es_corto = palabras.len() < 30;

    // Si el prompt YA contiene información suficiente, no buscar
    let contiene_respuesta_interna = prompt_lower.contains("según")
        || prompt_lower.contains("como sabemos")
        || prompt_lower.contains("basado en");

    if contiene_respuesta_interna {
        return NecesidadBusqueda::NoNecesita;
    }

    // Contar señales activas
    let mut senales = 0u32;
    if contiene_factual { senales += 1; }
    if contiene_actualidad { senales += 1; }
    if contiene_docs { senales += 1; }
    if contiene_codigo { senales += 1; }
    if es_corto && termina_en_pregunta { senales += 1; }

    if senales == 0 {
        return NecesidadBusqueda::NoNecesita;
    }

    // Clasificar tipo de necesidad
    if contiene_actualidad {
        NecesidadBusqueda::DatosActuales
    } else if contiene_docs {
        NecesidadBusqueda::DocumentacionTecnica
    } else if contiene_codigo {
        NecesidadBusqueda::CodigoEjemplo
    } else if senales >= 2 {
        NecesidadBusqueda::FactualMultiple
    } else {
        NecesidadBusqueda::FactualSimple
    }
}

// ─── Generador de Query Inteligente (1 llamada Ollama) ─────────────────────

/// Genera la consulta de búsqueda optimizada usando el modelo local.
pub async fn generar_query(
    prompt: &str,
    _necesidad: &NecesidadBusqueda,
    ollama_api_base: &str,
    ollama_model: &str,
    client: &Client,
) -> Result<QueryGenerada, String> {
    let system_prompt = r#"
Eres un generador de consultas de búsqueda web. Tu única tarea es
generar la MEJOR consulta de búsqueda para encontrar la información
que el usuario necesita.

Reglas:
- Máximo 8 palabras
- Idioma: español o inglés según el prompt
- Incluye el término técnico exacto
- Si pide código, añade "example" o "tutorial"
- Si pide documentación, añade "docs" o "documentation"
- Si pide actualizaciones, añade el año actual

Devuelve SOLO un JSON: {"query": "tu consulta aquí", "fuente": "duckduckgo|docs|multiple"}
"#;

    let messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": format!(
            "Genera la consulta de búsqueda óptima para: {}",
            prompt
        )}),
    ];

    let request_body = json!({
        "model": ollama_model,
        "messages": messages,
        "options": {
            "temperature": 0.2,
            "top_p": 0.9,
        },
        "format": "json",
    });

    let res = client.post(format!("{}/api/chat", ollama_api_base))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Error al generar query: {}", e))?;

    let body: serde_json::Value = res.json::<serde_json::Value>().await
        .map_err(|e| format!("Error al parsear respuesta de query: {}", e))?;

    let content = body["message"]["content"].as_str()
        .ok_or_else(|| "Respuesta vacía del generador de query".to_string())?;

    // Intentar parsear como JSON
    let query_gen: QueryGenerada = serde_json::from_str(content)
        .map_err(|e| format!("Error al parsear query generada: {}. Raw: {}", e, content))?;

    Ok(query_gen)
}

// ─── Ejecutor de Herramientas (Rust puro, 0 LLM) ──────────────────────────

pub mod herramientas {
    use super::{ResultadoBusqueda, QueryGenerada, ResultadoConsolidado, HerramientaBusqueda, NecesidadBusqueda};
    use std::time::Instant;

    /// Busca en DuckDuckGo usando scraping HTML (sin API key).
    pub async fn buscar_duckduckgo(
        query: &str,
        max_resultados: usize,
        client: &reqwest::Client,
    ) -> Vec<ResultadoBusqueda> {
        let encoded = urlencoding(query);
        let url = format!("https://html.duckduckgo.com/html/?q={}", encoded);

        let res = match client.get(&url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return vec![],
        };

        let html = match res.text().await {
            Ok(t) => t,
            Err(_) => return vec![],
        };

        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut resultados = Vec::new();

        // DuckDuckGo HTML structure: results are in <div class="result__body"> blocks
        for block in html.split(r#"class="result__body""#).skip(1) {
            if resultados.len() >= max_resultados {
                break;
            }

            let url_val = block.split(r#"href=""#)
                .nth(1)
                .and_then(|s| s.split('"').next())
                .unwrap_or("")
                .to_string();

            let titulo = block.split(r#"class="result__a""#)
                .nth(1)
                .and_then(|s| s.split('>').nth(1))
                .and_then(|s| s.split("</a>").next())
                .map(strip_html_tags)
                .unwrap_or_default();

            let snippet = block.split(r#"class="result__snippet""#)
                .nth(1)
                .and_then(|s| s.split('>').nth(1))
                .and_then(|s| s.split("</a>").next())
                .map(strip_html_tags)
                .unwrap_or_default();

            if !url_val.is_empty() && !titulo.is_empty() {
                let relevancia = if snippet.to_lowercase().contains(&titulo.to_lowercase()) {
                    0.9
                } else if !snippet.is_empty() {
                    0.7
                } else {
                    0.5
                };

                resultados.push(ResultadoBusqueda {
                    fuente: url_val,
                    titulo,
                    contenido: truncar_texto(&snippet, 2000),
                    relevancia,
                    timestamp: timestamp.clone(),
                });
            }
        }

        resultados
    }

    /// Scrapea una URL directa y extrae texto relevante.
    pub async fn scrapear_url(
        url: &str,
        client: &reqwest::Client,
    ) -> Option<ResultadoBusqueda> {
        let res = match client.get(url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return None,
        };

        if !res.status().is_success() {
            return None;
        }

        let html = match res.text().await {
            Ok(t) => t,
            Err(_) => return None,
        };

        // Extraer título
        let titulo = html.split("<title>")
            .nth(1)
            .and_then(|s| s.split("</title>").next())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Extraer texto plano
        let contenido = extract_text_simple(&html);

        let timestamp = chrono::Utc::now().to_rfc3339();

        Some(ResultadoBusqueda {
            fuente: url.to_string(),
            titulo,
            contenido: truncar_texto(&contenido, 4000),
            relevancia: 0.8,
            timestamp,
        })
    }

    /// Orquesta la búsqueda según la herramienta seleccionada.
    pub async fn ejecutar_busqueda(
        query: &QueryGenerada,
        _necesidad: &NecesidadBusqueda,
        client: &reqwest::Client,
    ) -> ResultadoConsolidado {
        let inicio = Instant::now();

        let (resultados, herramienta) = match query.fuente.as_str() {
            "docs" => {
                // DuckDuckGo primero, luego scrapear URLs top
                let ddg_results = buscar_duckduckgo(&query.query, 3, client).await;
                let mut consolidado = ddg_results;
                let top_urls: Vec<String> = consolidado.iter()
                    .take(2)
                    .map(|r| r.fuente.clone())
                    .collect();
                for url in &top_urls {
                    if let Some(scrapeado) = scrapear_url(url, client).await {
                        consolidado.push(scrapeado);
                    }
                }
                (consolidado, HerramientaBusqueda::MultipleFuentes)
            },
            "multiple" => {
                let ddg_results = buscar_duckduckgo(&query.query, 5, client).await;
                let mut consolidado = ddg_results;
                let top_urls: Vec<String> = consolidado.iter()
                    .take(2)
                    .map(|r| r.fuente.clone())
                    .collect();
                for url in &top_urls {
                    if let Some(scrapeado) = scrapear_url(url, client).await {
                        consolidado.push(scrapeado);
                    }
                }
                (consolidado, HerramientaBusqueda::MultipleFuentes)
            },
            _ => {
                // DuckDuckGo por defecto
                let ddg_results = buscar_duckduckgo(&query.query, 5, client).await;
                (ddg_results, HerramientaBusqueda::DuckDuckGo)
            }
        };

        ResultadoConsolidado {
            resultados,
            query_usada: query.query.clone(),
            herramienta_usada: herramienta,
            latencia_ms: inicio.elapsed().as_millis() as u64,
        }
    }

    // ─── Helpers de scraping ────────────────────────────────────────────

    fn urlencoding(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                ' ' => '+'.to_string(),
                c if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' => c.to_string(),
                c => format!("%{:02X}", c as u8),
            })
            .collect()
    }

    fn strip_html_tags(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut in_tag = false;
        for c in input.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }
        let trimmed: String = result.chars()
            .filter(|c| !c.is_control())
            .collect();
        trimmed.trim().to_string()
    }

    fn extract_text_simple(html: &str) -> String {
        // Remover scripts
        let sin_scripts = html
            .split("<script")
            .enumerate()
            .filter_map(|(i, s)| {
                if i > 0 {
                    s.split("</script>").nth(1)
                } else {
                    Some(s)
                }
            })
            .collect::<Vec<&str>>()
            .join(" ");

        // Remover styles
        let sin_styles = sin_scripts
            .split("<style")
            .enumerate()
            .filter_map(|(i, s)| {
                if i > 0 {
                    s.split("</style>").nth(1)
                } else {
                    Some(s)
                }
            })
            .collect::<Vec<&str>>()
            .join(" ");

        strip_html_tags(&sin_styles)
    }

    fn truncar_texto(texto: &str, max_chars: usize) -> String {
        if texto.len() <= max_chars {
            texto.to_string()
        } else {
            let mut truncated = texto[..max_chars].to_string();
            truncated.push_str("...");
            truncated
        }
    }
}

// ─── Extractor + Razonador (1 llamada Ollama con contexto aumentado) ─────

/// Toma los resultados de búsqueda y produce una respuesta razonada.
pub async fn razonar_con_busqueda(
    prompt: &str,
    resultados: &ResultadoConsolidado,
    ollama_api_base: &str,
    ollama_model: &str,
    client: &Client,
) -> RespuestaBusqueda {
    let inicio = Instant::now();

    // Si no hay resultados, devolver respuesta directa honesta
    if resultados.resultados.is_empty() {
        return RespuestaBusqueda {
            respuesta: "No se encontraron resultados de búsqueda para tu consulta. Intenta reformular la pregunta con términos más específicos.".to_string(),
            confianza: 0.0,
            fuentes_usadas: vec![],
            info_insuficiente: true,
            latencia_total_ms: inicio.elapsed().as_millis() as u64,
            busqueda_realizada: true,
        };
    }

    let system_prompt = r#"
Eres un analista de información. Tu tarea es:

1. REVISAR los fragmentos de búsqueda proporcionados
2. EXTRAER la información relevante para la pregunta original
3. SINTETIZAR una respuesta coherente y CITADA
4. Si los fragmentos no contienen la respuesta, INDÍCALO explícitamente
   (no inventes información faltante)

Reglas:
- Si confianza < 0.5, marca info_insuficiente = true
- Prefiere fuentes más recientes cuando hay conflicto
- No copies texto literalmente — parafrasea y sintetiza

Devuelve JSON: {"respuesta": "...", "confianza": 0.0-1.0, "info_insuficiente": false}
"#;

    // Construir contexto con las fuentes encontradas
    let mut fuentes_str = String::from("Fuentes encontradas:\n");
    for (i, r) in resultados.resultados.iter().enumerate() {
        fuentes_str.push_str(&format!(
            "[{}] {} → {}\n",
            i + 1,
            r.fuente,
            truncar_texto(&r.contenido, 1500)
        ));
    }

    let user_message = format!(
        "Pregunta original: {}\n\n{}",
        prompt,
        fuentes_str
    );

    let messages = vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": user_message}),
    ];

    let request_body = json!({
        "model": ollama_model,
        "messages": messages,
        "options": {
            "temperature": 0.3,
            "top_p": 0.9,
        },
        "format": "json",
    });

    let res = match client.post(format!("{}/api/chat", ollama_api_base))
        .json(&request_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return RespuestaBusqueda {
                respuesta: format!("Error al contactar a Ollama para analizar resultados: {}", e),
                confianza: 0.0,
                fuentes_usadas: resultados.resultados.iter().map(|r| r.fuente.clone()).collect(),
                info_insuficiente: true,
                latencia_total_ms: inicio.elapsed().as_millis() as u64,
                busqueda_realizada: true,
            };
        }
    };

    let body: serde_json::Value = match res.json::<serde_json::Value>().await {
        Ok(b) => b,
        Err(e) => {
            return RespuestaBusqueda {
                respuesta: format!("Error al parsear respuesta del análisis: {}", e),
                confianza: 0.0,
                fuentes_usadas: resultados.resultados.iter().map(|r| r.fuente.clone()).collect(),
                info_insuficiente: true,
                latencia_total_ms: inicio.elapsed().as_millis() as u64,
                busqueda_realizada: true,
            };
        }
    };

    let content = body["message"]["content"].as_str().unwrap_or("");

    // Obtener fuentes usadas
    let fuentes_usadas: Vec<String> = resultados.resultados.iter()
        .map(|r| r.fuente.clone())
        .collect();

    // Intentar parsear como JSON estructurado
    if let Ok(rb) = serde_json::from_str::<RespuestaBusquedaPartial>(content) {
        RespuestaBusqueda {
            respuesta: rb.respuesta,
            confianza: rb.confianza,
            fuentes_usadas,
            info_insuficiente: rb.info_insuficiente,
            latencia_total_ms: inicio.elapsed().as_millis() as u64,
            busqueda_realizada: true,
        }
    } else {
        // Fallback: usar texto plano como respuesta
        RespuestaBusqueda {
            respuesta: content.to_string(),
            confianza: 0.6,
            fuentes_usadas,
            info_insuficiente: false,
            latencia_total_ms: inicio.elapsed().as_millis() as u64,
            busqueda_realizada: true,
        }
    }
}

// Helper interno para parsear respuesta parcial del modelo
#[derive(Debug, Deserialize)]
struct RespuestaBusquedaPartial {
    respuesta: String,
    confianza: f32,
    #[serde(default)]
    info_insuficiente: bool,
}

fn truncar_texto(texto: &str, max_chars: usize) -> String {
    if texto.len() <= max_chars {
        texto.to_string()
    } else {
        let mut truncated = texto[..max_chars].to_string();
        truncated.push_str("...");
        truncated
    }
}

// ─── Orquestador Principal ─────────────────────────────────────────────────

/// Punto de entrada único para búsqueda inteligente.
/// 1. Clasifica necesidad (0 LLM)
/// 2. Si no necesita búsqueda → retorna None (delega a CoT)
/// 3. Si necesita → genera query → ejecuta herramientas → razona
pub async fn procesar_con_busqueda(
    prompt: &str,
    ollama_api_base: &str,
    ollama_model: &str,
) -> Option<RespuestaBusqueda> {
    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(_) => return None,
    };

    // Paso 1: Clasificar necesidad (Rust puro, 0 LLM)
    let necesidad = clasificar_necesidad(prompt);

    if necesidad == NecesidadBusqueda::NoNecesita {
        return None; // Delegar al CoT normal
    }

    // Paso 2: Generar query inteligente (1 Ollama)
    let query_gen = match generar_query(prompt, &necesidad, ollama_api_base, ollama_model, &client).await {
        Ok(q) => q,
        Err(_e) => {
            // Fallback: usar el prompt como query directa
            QueryGenerada {
                query: prompt.to_string(),
                fuente: "duckduckgo".to_string(),
            }
        }
    };

    // Paso 3: Ejecutar herramientas de búsqueda (Rust puro, 0 LLM)
    let resultados = herramientas::ejecutar_busqueda(&query_gen, &necesidad, &client).await;

    // Paso 4: Razonar sobre los resultados (1 Ollama)
    let respuesta = razonar_con_busqueda(prompt, &resultados, ollama_api_base, ollama_model, &client).await;

    Some(respuesta)
}
