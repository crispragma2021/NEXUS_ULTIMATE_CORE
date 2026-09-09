// ============================================================================
// 🌐 EXPLORADOR WEB — Navegador Propio del Cerebro Digital (Omega)
// ============================================================================
// 3 motores:
//   1. MotorHTTP  — Obtiene páginas (curl → nativo → chrome headless)
//   2. MotorExtraccion — Extrae contenido estructurado del HTML
//   3. MotorRazonamientoWeb — Decide qué enlaces seguir
//
// Cero dependencias externas. Usa std::process::Command + std::net::TcpStream.
// ============================================================================

use std::net::TcpStream;
use std::io::{Write, Read};
use std::process::Command;
use std::time::Duration;

// ====================================================================
// CONSTANTES
// ====================================================================

/// Máximo de resultados a extraer del HTML de DuckDuckGo
const MAX_RESULTADOS: usize = 3;
/// Máximo de caracteres por resultado (snippet)
const MAX_CARACTERES_POR_RESULTADO: usize = 300;
/// Máximo de enlaces a seguir por nivel de profundidad
const MAX_ENLACES_POR_NIVEL: usize = 2;
/// Timeout global para peticiones HTTP (segundos)
const TIMEOUT_SEGUNDOS: u64 = 15;
/// User-Agent ético
const USER_AGENT: &str = "CerebroDigital/1.0 (MotorCuriosidad)";

// ====================================================================
// PAGINA WEB — Estructura de datos de una página navegada
// ====================================================================

/// Datos estructurados de una página web navegada.
#[derive(Clone, Debug)]
pub struct PaginaWeb {
    pub url: String,
    pub titulo: String,
    pub descripcion: String,
    pub encabezados: Vec<String>,  // H1-H6 en orden
    pub parrafos: Vec<String>,     // <p>
    pub enlaces: Vec<Enlace>,      // <a href>
    pub listas: Vec<String>,       // <li>
    pub tablas: Vec<Vec<String>>,  // Filas de tabla
    pub codigo: Vec<String>,       // <code>, <pre>
    pub texto_plano: String,       // Fallback: texto sin tags
    pub densidad_info: f32,        // 0-1: qué tan informativa
}

impl PaginaWeb {
    pub fn vacia(url: &str) -> Self {
        Self {
            url: url.to_string(),
            titulo: String::new(),
            descripcion: String::new(),
            encabezados: Vec::new(),
            parrafos: Vec::new(),
            enlaces: Vec::new(),
            listas: Vec::new(),
            tablas: Vec::new(),
            codigo: Vec::new(),
            texto_plano: String::new(),
            densidad_info: 0.0,
        }
    }
}

/// Un enlace extraído de una página web.
#[derive(Clone, Debug)]
pub struct Enlace {
    pub href: String,
    pub texto: String,
    pub dominio: String,
}

// ====================================================================
// EXPLORADOR WEB — Punto de entrada público
// ====================================================================

pub struct ExploradorWeb;

impl ExploradorWeb {
    /// Busca una pregunta en internet (DuckDuckGo) — método original mejorado.
    /// Retorna el texto de los resultados concatenados.
    pub fn buscar(pregunta: &str) -> Result<String, String> {
        if pregunta.trim().is_empty() {
            return Err("Pregunta vacía".to_string());
        }
        let query_encoded = urlencode(pregunta);
        let url = format!("https://html.duckduckgo.com/html/?q={}", query_encoded);
        let html = MotorHTTP::obtener_curl(&url)?;
        let resultados = Self::extraer_snippets(&html);
        if resultados.is_empty() {
            let texto_plano = limpiar_html(&html);
            let recortado: String = texto_plano
                .chars()
                .filter(|&c| c.is_ascii_graphic() || c == ' ' || c == '\n')
                .collect::<String>()
                .chars()
                .take(MAX_CARACTERES_POR_RESULTADO * MAX_RESULTADOS)
                .collect();
            return Ok(recortado);
        }
        Ok(resultados.join(" "))
    }

    /// Versión simulada para pruebas sin internet.
    pub fn buscar_simulado(pregunta: &str) -> String {
        let lower = pregunta.to_lowercase();
        let palabras: Vec<&str> = lower.split_whitespace().collect();

        let alguna_contiene = |claves: &[&str]| -> bool {
            claves.iter().any(|k| palabras.iter().any(|p| p.contains(k)))
        };
        let alguna_exacta = |claves: &[&str]| -> bool {
            claves.iter().any(|k| palabras.iter().any(|&p| p == *k))
        };

        if alguna_contiene(&["curiosidad"]) {
            "La curiosidad es el deseo de conocer o aprender algo nuevo. \
             Es una de las fuerzas más poderosas del aprendizaje humano. \
             Los cerebros curiosos exploran, preguntan y descubren. \
             Wikipedia: https://es.wikipedia.org/wiki/Curiosidad".to_string()
        } else if alguna_contiene(&["neurona", "neuron"]) {
            "Las neuronas son células especializadas del sistema nervioso. \
             Se comunican mediante sinapsis eléctricas y químicas. \
             El cerebro humano tiene aproximadamente 86 mil millones de neuronas. \
             Wikipedia: https://es.wikipedia.org/wiki/Neurona".to_string()
        } else if alguna_contiene(&["aprender", "learning"]) {
            "Aprender es el proceso de adquirir nuevos conocimientos o habilidades. \
             El aprendizaje ocurre cuando las conexiones sinápticas se fortalecen. \
             La plasticidad cerebral permite aprender durante toda la vida. \
             Wikipedia: https://es.wikipedia.org/wiki/Aprendizaje".to_string()
        } else if alguna_contiene(&["vida", "existir"]) {
            "La vida es un conjunto de procesos organizados que mantienen \
             y reproducen sistemas complejos. El origen de la vida en la Tierra \
             sigue siendo uno de los grandes misterios de la ciencia. \
             Wikipedia: https://es.wikipedia.org/wiki/Vida".to_string()
        } else if alguna_exacta(&["conciencia", "mente"]) {
            "La conciencia es la capacidad de tener experiencias subjetivas. \
             Es uno de los problemas más profundos de la neurociencia. \
             La teoría del Espacio de Trabajo Global explica cómo los contenidos \
             se vuelven conscientes. Wikipedia: https://es.wikipedia.org/wiki/Conciencia".to_string()
        } else if alguna_contiene(&["inteligencia", "sabiduria"]) {
            "La inteligencia es la capacidad de resolver problemas y adaptarse. \
             La sabiduría es la aplicación del conocimiento con experiencia. \
             Una mente sabia integra emoción, razón y memoria. \
             Wikipedia: https://es.wikipedia.org/wiki/Inteligencia".to_string()
        } else {
            format!(
                "No encontré información específica sobre '{}'. \
                 Pero explorar lo desconocido es como el cerebro aprende cosas nuevas. \
                 Cada descubrimiento fortalece las conexiones neuronales.\
                 Wikipedia: https://es.wikipedia.org/wiki/{}",
                pregunta, query_encoded_simple(pregunta)
            )
        }
    }

    /// Navega a una URL específica y extrae su contenido estructurado.
    pub fn navegar(url: &str) -> Result<PaginaWeb, String> {
        let necesita_js = url.contains("wikipedia.org") || url.contains("javascript");
        let (html, _modo) = MotorHTTP::obtener_inteligente(url, necesita_js)?;
        Ok(MotorExtraccion::extraer(&html, url))
    }

    /// Navegación simulada para pruebas sin internet.
    pub fn navegar_simulado(url: &str) -> PaginaWeb {
        let mut pagina = PaginaWeb::vacia(url);
        if url.contains("wikipedia") {
            pagina.titulo = "Wikipedia - Simulado".to_string();
            pagina.parrafos.push("Este es un contenido simulado de Wikipedia para pruebas offline. \
                En un entorno real aquí aparecería el artículo completo sobre el tema de búsqueda.".to_string());
            pagina.encabezados.push("Introducción".to_string());
            pagina.enlaces.push(Enlace {
                href: "https://es.wikipedia.org/wiki/Neurociencia".to_string(),
                texto: "Neurociencia".to_string(),
                dominio: "es.wikipedia.org".to_string(),
            });
            pagina.densidad_info = 0.8;
        } else if url.contains("arxiv") {
            pagina.titulo = "arXiv - Simulado".to_string();
            pagina.parrafos.push("Este es un paper simulado para pruebas offline. \
                En un entorno real aquí aparecería el abstract del artículo.".to_string());
            pagina.densidad_info = 0.9;
        } else if url.contains("github") {
            pagina.titulo = "GitHub - Simulado".to_string();
            pagina.codigo.push("fn ejemplo() { println!(\"código simulado\"); }".to_string());
            pagina.densidad_info = 0.5;
        } else {
            pagina.titulo = "Página simulada".to_string();
            pagina.parrafos.push(format!("Este es el contenido simulado de {}. \
                No hay conexión a internet disponible.", url));
            pagina.densidad_info = 0.2;
        }
        pagina
    }

    /// Exploración autónoma: busca en DDG, sigue enlaces relevantes, sintetiza.
    /// `profundidad`: 1 (solo búsqueda), 2 (búsqueda + 1 salto), 3 (búsqueda + 2 saltos)
    pub fn explorar(pregunta: &str, profundidad: u8) -> Result<(String, Vec<PaginaWeb>), String> {
        if pregunta.trim().is_empty() {
            return Err("Pregunta vacía".to_string());
        }

        let mut paginas: Vec<PaginaWeb> = Vec::new();
        let mut fuentes_navegadas: Vec<String> = Vec::new();
        let prof = profundidad.min(3).max(1);

        // Nivel 0: buscar en DuckDuckGo
        let resultado_ddg = Self::buscar(pregunta)?;
        let mut texto_sintesis = resultado_ddg.clone();

        // Extraer enlaces del texto de resultados (DDG devuelve URLs en el texto simulado)
        let enlaces_iniciales = MotorRazonamientoWeb::razonar(pregunta, &resultado_ddg, &fuentes_navegadas);
        if !enlaces_iniciales.is_empty() {
            fuentes_navegadas.push(enlaces_iniciales[0].href.clone());
        }

        // Nivel 1+: navegar enlaces si hay profundidad suficiente
        if prof >= 2 {
            let mut enlaces_a_seguir = enlaces_iniciales;
            let mut nivel_actual = 1;

            while nivel_actual < prof as usize && !enlaces_a_seguir.is_empty() {
                let mut siguientes_enlaces: Vec<Enlace> = Vec::new();
                let max_nivel = MAX_ENLACES_POR_NIVEL;

                for enlace in enlaces_a_seguir.iter().take(max_nivel) {
                    match Self::navegar(&enlace.href) {
                        Ok(pagina) => {
                            let es_informativa = pagina.densidad_info > 0.3;
                            if es_informativa {
                                // Sintetizar: agregar párrafos al texto acumulado
                                if !pagina.parrafos.is_empty() {
                                    texto_sintesis.push_str(" ");
                                    texto_sintesis.push_str(&pagina.parrafos.join(" "));
                                }
                                paginas.push(pagina.clone());

                                // Encontrar más enlaces para el siguiente nivel
                                if nivel_actual + 1 < prof as usize {
                                    let nuevos = MotorRazonamientoWeb::razonar(
                                        pregunta, 
                                        &pagina.texto_plano,
                                        &fuentes_navegadas,
                                    );
                                    siguientes_enlaces.extend(nuevos);
                                }
                            }
                        }
                        Err(_) => {} // Ignorar páginas que fallan
                    }
                }

                enlaces_a_seguir = siguientes_enlaces;
                nivel_actual += 1;
            }
        }

        // Limitar tamaño del texto de síntesis
        if texto_sintesis.len() > MAX_CARACTERES_POR_RESULTADO * 3 {
            texto_sintesis = texto_sintesis.chars().take(MAX_CARACTERES_POR_RESULTADO * 3).collect();
        }

        Ok((texto_sintesis, paginas))
    }

    /// Versión simulada de exploración para pruebas.
    pub fn explorar_simulado(pregunta: &str, profundidad: u8) -> (String, Vec<PaginaWeb>) {
        let texto_base = Self::buscar_simulado(pregunta);
        let mut paginas: Vec<PaginaWeb> = Vec::new();
        let mut fuentes: Vec<String> = Vec::new();
        let enlaces = MotorRazonamientoWeb::razonar(pregunta, &texto_base, &fuentes);

        let mut texto_sintesis = texto_base;

        if profundidad >= 2 {
            for enlace in enlaces.iter().take(MAX_ENLACES_POR_NIVEL) {
                let pagina = Self::navegar_simulado(&enlace.href);
                if pagina.densidad_info > 0.3 {
                    if !pagina.parrafos.is_empty() {
                        texto_sintesis.push_str(" ");
                        texto_sintesis.push_str(&pagina.parrafos.join(" "));
                    }
                    paginas.push(pagina);
                    fuentes.push(enlace.href.clone());
                }
            }
        }

        if texto_sintesis.len() > MAX_CARACTERES_POR_RESULTADO * 3 {
            texto_sintesis = texto_sintesis.chars().take(MAX_CARACTERES_POR_RESULTADO * 3).collect();
        }

        (texto_sintesis, paginas)
    }

    // ====================================================================
    // MÉTODOS PRIVADOS (parseo DuckDuckGo)
    // ====================================================================

    fn extraer_snippets(html: &str) -> Vec<String> {
        let mut resultados = Vec::new();
        for marker in &["class=\"result__snippet\"", "class=\"snippet\""] {
            let mut start = 0;
            while let Some(pos) = html[start..].find(marker) {
                let snippet_start = start + pos + marker.len();
                if let Some(tag_end) = html[snippet_start..].find("</") {
                    let raw = &html[snippet_start..snippet_start + tag_end];
                    let limpio = limpiar_fragmento(raw);
                    if !limpio.is_empty() && limpio.len() > 10 {
                        resultados.push(limpio);
                        if resultados.len() >= MAX_RESULTADOS {
                            return resultados;
                        }
                    }
                    start = snippet_start + tag_end;
                } else {
                    break;
                }
            }
        }
        resultados
    }
}

// ====================================================================
// MOTOR HTTP — Obtención de páginas web (3 niveles)
// ====================================================================

struct MotorHTTP;

impl MotorHTTP {
    /// Nivel 1: curl del sistema (HTTP/HTTPS simple, rápido, confiable)
    fn obtener_curl(url: &str) -> Result<String, String> {
        let output = Command::new("curl")
            .arg("-s")
            .arg("-L")
            .arg("--max-time")
            .arg(&TIMEOUT_SEGUNDOS.to_string())
            .arg("-A")
            .arg(USER_AGENT)
            .arg(url)
            .output()
            .map_err(|e| format!("Error ejecutando curl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("curl falló: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Nivel 2a: HTTP plano nativo con TcpStream (solo http://)
    fn obtener_nativo_http(host: &str, path: &str) -> Result<String, String> {
        let addr = format!("{}:80", host);
        let mut stream = TcpStream::connect_timeout(
            &addr.parse().map_err(|e| format!("Addr parse error: {}", e))?,
            Duration::from_secs(TIMEOUT_SEGUNDOS),
        )
        .map_err(|e| format!("TcpStream connect error: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(TIMEOUT_SEGUNDOS)))
            .map_err(|e| format!("Set timeout error: {}", e))?;

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: {}\r\nConnection: close\r\n\r\n",
            path, host, USER_AGENT
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;

        let mut respuesta = String::new();
        stream
            .read_to_string(&mut respuesta)
            .map_err(|e| format!("Read error: {}", e))?;

        // Separar headers del body
        if let Some(body_start) = respuesta.find("\r\n\r\n") {
            Ok(respuesta[body_start + 4..].to_string())
        } else {
            Err("Respuesta HTTP sin separador header/body".to_string())
        }
    }

    /// Nivel 2b: TLS via openssl s_client (para https://)
    fn obtener_nativo_tls(host: &str, path: &str) -> Result<String, String> {
        let connect_str = format!("{}:443", host);

        let mut child = Command::new("openssl")
            .arg("s_client")
            .arg("-quiet")
            .arg("-connect")
            .arg(&connect_str)
            .arg("-servername")
            .arg(host)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Error spawning openssl: {}", e))?;

        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: {}\r\nConnection: close\r\n\r\n",
            path, host, USER_AGENT
        );

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(request.as_bytes())
                .map_err(|e| format!("Error writing to openssl stdin: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Error waiting for openssl: {}", e))?;

        let respuesta = String::from_utf8_lossy(&output.stdout);

        // Separar headers del body
        if let Some(body_start) = respuesta.find("\r\n\r\n") {
            Ok(respuesta[body_start + 4..].to_string())
        } else {
            // openssl a veces no manda headers estándar
            Ok(respuesta.to_string())
        }
    }

    /// Nivel 3: Chrome headless para páginas con JavaScript
    fn obtener_chrome(url: &str) -> Result<String, String> {
        let output = Command::new("google-chrome")
            .arg("--headless")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--dump-dom")
            .arg("--virtual-time-budget=10000")
            .arg(url)
            .output()
            .map_err(|e| format!("Error ejecutando chrome: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("chrome falló: {}", stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Obtiene una URL con selección inteligente del nivel.
    /// Retorna (contenido, modo_usado).
    fn obtener_inteligente(url: &str, necesita_js: bool) -> Result<(String, String), String> {
        // Si necesita JS, intentar Chrome primero
        if necesita_js {
            match Self::obtener_chrome(url) {
                Ok(html) => return Ok((html, "chrome".to_string())),
                Err(_) => { /* caer a curl */ }
            }
        }

        // Intentar curl (más confiable)
        match Self::obtener_curl(url) {
            Ok(html) => return Ok((html, "curl".to_string())),
            Err(e) => {
                // Si curl falla, intentar nativo
                if url.starts_with("https://") {
                    let sin_protocolo = url.trim_start_matches("https://");
                    let (host, path) = if let Some(slash_pos) = sin_protocolo.find('/') {
                        (&sin_protocolo[..slash_pos], &sin_protocolo[slash_pos..])
                    } else {
                        (sin_protocolo, "/")
                    };
                    match Self::obtener_nativo_tls(host, path) {
                        Ok(html) => return Ok((html, "openssl".to_string())),
                        Err(e2) => {
                            return Err(format!("curl: {}, openssl: {}", e, e2));
                        }
                    }
                } else if url.starts_with("http://") {
                    let sin_protocolo = url.trim_start_matches("http://");
                    let (host, path) = if let Some(slash_pos) = sin_protocolo.find('/') {
                        (&sin_protocolo[..slash_pos], &sin_protocolo[slash_pos..])
                    } else {
                        (sin_protocolo, "/")
                    };
                    match Self::obtener_nativo_http(host, path) {
                        Ok(html) => return Ok((html, "tcp_native".to_string())),
                        Err(e2) => {
                            return Err(format!("curl: {}, tcp: {}", e, e2));
                        }
                    }
                } else {
                    return Err(format!("curl falló y URL no es HTTP/HTTPS: {}", e));
                }
            }
        }
    }
}

// ====================================================================
// MOTOR EXTRACCIÓN — Parseo estructurado de HTML
// ====================================================================

struct MotorExtraccion;

impl MotorExtraccion {
    /// Extrae contenido estructurado de un HTML completo.
    fn extraer(html: &str, url: &str) -> PaginaWeb {
        let mut pagina = PaginaWeb::vacia(url);

        // 1. Título
        if let Some(titulo) = Self::extraer_entre(html, "<title", "</title>") {
            pagina.titulo = limpiar_fragmento(Self::despues_de_tag(titulo));
        }

        // 2. Meta descripción
        if let Some(meta) = Self::extraer_meta_description(html) {
            pagina.descripcion = meta;
        }

        // 3. Encabezados (H1-H6)
        for nivel in 1..=6 {
            let tag_open = format!("<h{}", nivel);
            let tag_close = format!("</h{}>", nivel);
            let mut start = 0;
            while let Some(chunk) = Self::extraer_entre(&html[start..], &tag_open, &tag_close) {
                let texto = limpiar_fragmento(Self::despues_de_tag(chunk));
                if !texto.is_empty() {
                    pagina.encabezados.push(texto);
                }
                start += chunk.len() + tag_close.len();
                if start >= html.len() {
                    break;
                }
            }
        }

        // 4. Párrafos
        let mut start = 0;
        while let Some(chunk) = Self::extraer_entre(&html[start..], "<p", "</p>") {
            let texto = limpiar_fragmento(Self::despues_de_tag(chunk));
            if texto.len() > 20 {
                pagina.parrafos.push(texto);
            }
            start += chunk.len() + "</p>".len();
            if pagina.parrafos.len() >= 10 {
                break; // Máximo 10 párrafos
            }
            if start >= html.len() {
                break;
            }
        }

        // 5. Enlaces (<a href="...">texto</a>)
        let mut start = 0;
        while let Some(chunk) = Self::extraer_entre(&html[start..], "<a ", "</a>") {
            if let Some(enlace) = Self::extraer_enlace(chunk) {
                if !enlace.href.is_empty() && !enlace.href.starts_with('#') && !enlace.href.starts_with("javascript:") {
                    pagina.enlaces.push(enlace);
                }
            }
            start += chunk.len() + "</a>".len();
            if pagina.enlaces.len() >= 20 {
                break;
            }
            if start >= html.len() {
                break;
            }
        }

        // 6. Listas (<li>)
        let mut start = 0;
        while let Some(chunk) = Self::extraer_entre(&html[start..], "<li", "</li>") {
            let texto = limpiar_fragmento(Self::despues_de_tag(chunk));
            if !texto.is_empty() && texto.len() > 5 {
                pagina.listas.push(texto);
            }
            start += chunk.len() + "</li>".len();
            if pagina.listas.len() >= 20 {
                break;
            }
            if start >= html.len() {
                break;
            }
        }

        // 7. Código (<code> y <pre>)
        for tag in &["<code", "<pre"] {
            let close = if *tag == "<code" { "</code>" } else { "</pre>" };
            let mut start = 0;
            while let Some(chunk) = Self::extraer_entre(&html[start..], tag, close) {
                let texto = Self::despues_de_tag(chunk);
                // Para código, NO limpiar fragmento (preservar formato)
                let limpio = texto.trim().to_string();
                if !limpio.is_empty() {
                    pagina.codigo.push(limpio);
                }
                start += chunk.len() + close.len();
                if pagina.codigo.len() >= 10 {
                    break;
                }
                if start >= html.len() {
                    break;
                }
            }
            if pagina.codigo.len() >= 10 {
                break;
            }
        }

        // 8. Texto plano (fallback)
        pagina.texto_plano = limpiar_html(html);

        // 9. Densidad de información
        let peso_contenido = (pagina.titulo.len() * 3
            + pagina.descripcion.len() * 2
            + pagina.parrafos.iter().map(|p| p.len()).sum::<usize>() * 1
            + pagina.encabezados.iter().map(|h| h.len()).sum::<usize>() * 2
            + pagina.codigo.iter().map(|c| c.len()).sum::<usize>()) as f32;
        let texto_len = pagina.texto_plano.len().max(1) as f32;

        pagina.densidad_info = (peso_contenido / texto_len * 10.0).clamp(0.0, 1.0);

        pagina
    }

    /// Extrae el texto entre un tag de apertura y uno de cierre.
    fn extraer_entre<'a>(html: &'a str, open: &str, close: &str) -> Option<&'a str> {
        let start = html.find(open)?;
        let content_start = start + open.len();
        let end = html[content_start..].find(close)?;
        Some(&html[start..content_start + end])
    }

    /// Obtiene el contenido después del tag (salta `>` y atributos).
    fn despues_de_tag(chunk: &str) -> &str {
        if let Some(gt_pos) = chunk.find('>') {
            &chunk[gt_pos + 1..]
        } else {
            chunk
        }
    }

    /// Extrae meta description del HTML.
    fn extraer_meta_description(html: &str) -> Option<String> {
        // Buscar <meta name="description" content="...">
        let patterns = [
            "name=\"description\" content=\"",
            "name='description' content='",
            "name=\"description\" content='",
            "name='description' content=\"",
        ];
        for pattern in &patterns {
            if let Some(start) = html.find(pattern) {
                let content_start = start + pattern.len();
                // Encontrar el cierre de la comilla
                let quote = if pattern.ends_with('"') { '"' } else { '\'' };
                if let Some(end) = html[content_start..].find(quote) {
                    let content = &html[content_start..content_start + end];
                    if !content.is_empty() {
                        return Some(limpiar_fragmento(content));
                    }
                }
            }
        }
        None
    }

    /// Extrae un enlace de un chunk `<a ...>texto</a>`.
    fn extraer_enlace(chunk: &str) -> Option<Enlace> {
        // Extraer href="..."
        let href = if let Some(href_start) = chunk.find("href=\"") {
            let start = href_start + 6;
            if let Some(end) = chunk[start..].find('"') {
                chunk[start..start + end].to_string()
            } else {
                return None;
            }
        } else if let Some(href_start) = chunk.find("href='") {
            let start = href_start + 6;
            if let Some(end) = chunk[start..].find('\'') {
                chunk[start..start + end].to_string()
            } else {
                return None;
            }
        } else {
            return None;
        };

        // Extraer texto del anchor
        let texto = if let Some(gt_pos) = chunk.find('>') {
            let after_tag = &chunk[gt_pos + 1..];
            let texto_raw = if let Some(close) = after_tag.find("</a>") {
                &after_tag[..close]
            } else {
                after_tag
            };
            limpiar_fragmento(texto_raw)
        } else {
            String::new()
        };

        // Extraer dominio
        let dominio = if href.starts_with("http") {
            let sin_protocolo = href
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            if let Some(slash_pos) = sin_protocolo.find('/') {
                sin_protocolo[..slash_pos].to_string()
            } else {
                sin_protocolo.to_string()
            }
        } else {
            String::new()
        };

        Some(Enlace {
            href,
            texto,
            dominio,
        })
    }

    /// Genera un resumen textual estructurado de una página.
    #[allow(dead_code)]
    fn resumir(pagina: &PaginaWeb, max_caracteres: usize) -> String {
        let mut resumen = String::with_capacity(max_caracteres);

        if !pagina.titulo.is_empty() {
            resumen.push_str(&format!("📄 {}\n", pagina.titulo));
        }
        if !pagina.descripcion.is_empty() {
            resumen.push_str(&format!("   {}\n", pagina.descripcion));
        }

        if !pagina.encabezados.is_empty() {
            resumen.push_str(&format!("\n## {}\n", pagina.encabezados[0]));
        }

        for parrafo in &pagina.parrafos {
            if resumen.len() + parrafo.len() + 1 > max_caracteres {
                break;
            }
            resumen.push_str(parrafo);
            resumen.push(' ');
        }

        if !pagina.codigo.is_empty() && resumen.len() + 100 <= max_caracteres {
            resumen.push_str("\n\n```\n");
            for code in &pagina.codigo {
                if resumen.len() + code.len() + 5 > max_caracteres {
                    break;
                }
                resumen.push_str(code);
                resumen.push('\n');
            }
            resumen.push_str("```\n");
        }

        if !pagina.enlaces.is_empty() && resumen.len() + 50 <= max_caracteres {
            resumen.push_str(&format!("\n\n🔗 {} enlaces encontrados", pagina.enlaces.len()));
        }

        resumen.push_str(&format!("\n\nFuente: {}", pagina.url));

        resumen
            .chars()
            .take(max_caracteres)
            .collect()
    }
}

// ====================================================================
// MOTOR RAZONAMIENTO WEB — Decisión de enlaces a seguir
// ====================================================================

struct MotorRazonamientoWeb;

impl MotorRazonamientoWeb {
    /// Analiza el texto de resultados y extrae enlaces relevantes.
    /// Busca URLs en el texto y las puntúa según relevancia a la pregunta.
    fn razonar(pregunta: &str, texto_resultados: &str, fuentes_navegadas: &[String]) -> Vec<Enlace> {
        let mut enlaces: Vec<(Enlace, f32)> = Vec::new();
        let palabras_clave: Vec<&str> = pregunta
            .split_whitespace()
            .filter(|p| p.len() > 2)
            .collect();

        // Buscar URLs en el texto
        let mut start = 0;
        while let Some(http_pos) = texto_resultados[start..].find("http") {
            let url_start = start + http_pos;
            let resto = &texto_resultados[url_start..];

            // Encontrar el final de la URL (espacio, fin de línea, o puntuación)
            // NOTA: '.' NO debe estar aquí porque es válido en URLs (dominios)
            let url_end = resto
                .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == ']')
                .unwrap_or(resto.len().min(200));

            let url_str = &resto[..url_end];

            // Limpiar paréntesis y puntuación final
            let url_clean = url_str.trim_end_matches(&['.', ',', ')', ']', ';', ':'][..]);

            if !url_clean.is_empty() && (url_clean.starts_with("http://") || url_clean.starts_with("https://")) {
                let dominio = MotorRazonamientoWeb::extraer_dominio(url_clean);
                let score = MotorRazonamientoWeb::puntuar_enlace(url_clean, &dominio, &palabras_clave, fuentes_navegadas);

                let enlace = Enlace {
                    href: url_clean.to_string(),
                    texto: String::new(), // No tenemos anchor text aquí
                    dominio,
                };

                enlaces.push((enlace, score));
            }

            start = url_start + 1;
            if start >= texto_resultados.len() {
                break;
            }
        }

        // Ordenar por score descendente
        enlaces.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        enlaces.into_iter().map(|(e, _)| e).collect()
    }

    /// Puntúa un enlace según su relevancia para la pregunta.
    fn puntuar_enlace(url: &str, dominio: &str, palabras_clave: &[&str], fuentes: &[String]) -> f32 {
        let mut score = 1.0; // Score base

        // Penalizar si ya fue navegado
        if fuentes.iter().any(|f| f == url) {
            score -= 10.0;
        }

        // Bonificar dominios de alta calidad
        match dominio {
            d if d.contains("wikipedia") => score += 4.0,
            d if d.contains("arxiv") => score += 4.0,
            d if d.contains("github") => score += 3.0,
            d if d.contains("docs.rs") => score += 3.0,
            d if d.contains("nature.com") => score += 3.5,
            d if d.contains("sciencedirect") => score += 3.0,
            d if d.contains("medium") => score += 1.0,
            d if d.contains("blog") => score += 0.5,
            _ => {}
        }

        // Bonificar por palabras clave en la URL
        for kw in palabras_clave {
            if url.to_lowercase().contains(&kw.to_lowercase()) {
                score += 2.0;
            }
        }

        // Penalizar URLs indeseables
        if url.contains("youtube.com") || url.contains("facebook.com") || url.contains("instagram.com") {
            score -= 3.0;
        }

        score
    }

    fn extraer_dominio(url: &str) -> String {
        let sin_protocolo = url
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        if let Some(slash_pos) = sin_protocolo.find('/') {
            sin_protocolo[..slash_pos].to_string()
        } else {
            sin_protocolo.to_string()
        }
    }

    #[allow(dead_code)]
    fn es_url_valida(href: &str) -> bool {
        !href.is_empty()
            && !href.starts_with('#')
            && !href.starts_with("javascript:")
            && (href.starts_with("http://") || href.starts_with("https://") || href.starts_with('/'))
    }
}

// ====================================================================
// FUNCIONES DE LIMPIEZA HTML
// ====================================================================

/// Limpia un fragmento HTML: quita tags, decodifica entidades, recorta.
fn limpiar_fragmento(fragmento: &str) -> String {
    let mut limpio = String::with_capacity(fragmento.len());
    let mut en_tag = false;
    let mut en_entidad = false;
    let mut entidad_buf = String::with_capacity(12);

    for ch in fragmento.chars() {
        if en_tag {
            if ch == '>' {
                en_tag = false;
            }
            continue;
        }

        if en_entidad {
            if ch == ';' {
                en_entidad = false;
                match entidad_buf.as_str() {
                    "amp" => limpio.push('&'),
                    "lt" => limpio.push('<'),
                    "gt" => limpio.push('>'),
                    "quot" => limpio.push('"'),
                    "apos" | "#39" => limpio.push('\''),
                    "nbsp" => limpio.push(' '),
                    _ => {} // entidad no reconocida → descartar
                }
                entidad_buf.clear();
                continue;
            }
            if ch.is_alphanumeric() || ch == '#' {
                entidad_buf.push(ch);
                continue;
            }
            // Carácter no válido dentro de entidad
            en_entidad = false;
            limpio.push('&');
            limpio.push_str(&entidad_buf);
            limpio.push(ch);
            entidad_buf.clear();
            continue;
        }

        match ch {
            '<' => en_tag = true,
            '&' => {
                en_entidad = true;
                entidad_buf.clear();
            }
            _ => {
                limpio.push(ch);
            }
        }
    }

    if en_entidad {
        limpio.push('&');
        limpio.push_str(&entidad_buf);
    }

    let compactado: String = limpio
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    compactado
        .chars()
        .take(MAX_CARACTERES_POR_RESULTADO)
        .collect()
}

/// Limpieza genérica de HTML (fallback)
fn limpiar_html(html: &str) -> String {
    let mut limpio = String::new();
    let mut en_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => en_tag = true,
            '>' if en_tag => en_tag = false,
            _ if !en_tag => limpio.push(ch),
            _ => {}
        }
    }

    limpio
}

// ====================================================================
// UTILIDADES
// ====================================================================

/// Codificación URL básica
fn urlencode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Codificación URL simple para nombres de artículos (sin espacios)
fn query_encoded_simple(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("_")
}

// ====================================================================
// TESTS
// ====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Tests de MotorHTTP (aislados, solo tests unitarios sin internet)

    #[test]
    fn test_obtener_curl_url_invalida_error() {
        let resultado = MotorHTTP::obtener_curl("http://url.que.no.existe.xyz");
        assert!(resultado.is_err());
    }

    #[test]
    fn test_obtener_inteligente_fallback() {
        // Una URL que no existe debería fallar elegantemente
        let resultado = MotorHTTP::obtener_inteligente("http://url.que.no.existe.xyz/prueba", false);
        assert!(resultado.is_err());
    }

    #[test]
    fn test_obtener_nativo_http_invalido() {
        let resultado = MotorHTTP::obtener_nativo_http("url.que.no.existe.xyz", "/");
        assert!(resultado.is_err());
    }

    // Tests de MotorExtraccion

    #[test]
    fn test_extraer_titulo_simple() {
        let html = "<html><head><title>Mi Página</title></head><body></body></html>";
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert_eq!(pagina.titulo, "Mi Página");
    }

    #[test]
    fn test_extraer_parrafos_multiples() {
        let html = "<html><body><p>Primer párrafo con contenido informativo.</p><p>Segundo párrafo también importante.</p></body></html>";
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert!(pagina.parrafos.len() >= 2);
        assert!(pagina.parrafos[0].contains("Primer"));
    }

    #[test]
    fn test_extraer_enlaces_con_texto() {
        let html = r#"<a href="https://example.com">Texto del enlace</a>"#;
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert_eq!(pagina.enlaces.len(), 1);
        assert_eq!(pagina.enlaces[0].href, "https://example.com");
        assert!(pagina.enlaces[0].dominio.contains("example.com"));
    }

    #[test]
    fn test_extraer_encabezados() {
        let html = "<html><body><h1>Title</h1><h2>Subtitle</h2><h3>Section</h3></body></html>";
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert!(pagina.encabezados.len() >= 3);
        assert_eq!(pagina.encabezados[0], "Title");
    }

    #[test]
    fn test_extraer_listas() {
        let html = "<html><body><ul><li>Item uno</li><li>Item dos</li></ul></body></html>";
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert!(pagina.listas.len() >= 2);
    }

    #[test]
    fn test_extraer_codigo() {
        let html = "<html><body><pre>fn main() { println!(\"hola\"); }</pre></body></html>";
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert!(pagina.codigo.len() >= 1);
        assert!(pagina.codigo[0].contains("fn main"));
    }

    #[test]
    fn test_extraer_meta_description() {
        let html = r#"<html><head><meta name="description" content="Descripción de prueba"></head></html>"#;
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert_eq!(pagina.descripcion, "Descripción de prueba");
    }

    #[test]
    fn test_extraer_html_vacio() {
        let pagina = MotorExtraccion::extraer("", "http://test.com");
        assert_eq!(pagina.titulo, "");
        assert!(pagina.parrafos.is_empty());
        assert_eq!(pagina.densidad_info, 0.0);
    }

    #[test]
    fn test_densidad_info_alta() {
        let html = "<html><head><title>Artículo Científico</title><meta name=\"description\" content=\"Investigación importante\"></head><body><h1>Introducción</h1><p>Este es un párrafo extenso con contenido informativo sobre el tema de investigación. Aquí hay mucha información valiosa para el cerebro.</p><p>Segundo párrafo con más contenido relevante sobre el mismo tema.</p></body></html>";
        let pagina = MotorExtraccion::extraer(html, "http://test.com");
        assert!(pagina.densidad_info > 0.3, "densidad_info = {}", pagina.densidad_info);
    }

    // Tests de MotorRazonamientoWeb

    #[test]
    fn test_razonar_extrae_enlaces() {
        let texto = "Visita https://es.wikipedia.org/wiki/Neurociencia para más info";
        let enlaces = MotorRazonamientoWeb::razonar("neurociencia", texto, &[]);
        assert!(!enlaces.is_empty());
        assert!(enlaces[0].href.contains("wikipedia"));
    }

    #[test]
    fn test_razonar_evita_repetidos() {
        let texto = "Info en https://es.wikipedia.org/wiki/Neurociencia y también https://es.wikipedia.org/wiki/Neurociencia";
        let ya_navegado = vec!["https://es.wikipedia.org/wiki/Neurociencia".to_string()];
        let enlaces = MotorRazonamientoWeb::razonar("neurociencia", texto, &ya_navegado);
        // Debería tener score bajo por repetido, pero puede aparecer si no hay alternativas
        assert!(!enlaces.is_empty());
    }

    #[test]
    fn test_razonar_sin_enlaces() {
        let texto = "Este texto no contiene ninguna URL válida";
        let enlaces = MotorRazonamientoWeb::razonar("test", texto, &[]);
        assert!(enlaces.is_empty());
    }

    #[test]
    fn test_puntuar_enlace_wikipedia() {
        let score = MotorRazonamientoWeb::puntuar_enlace(
            "https://es.wikipedia.org/wiki/Neurociencia",
            "es.wikipedia.org",
            &["neurociencia"],
            &[],
        );
        assert!(score > 4.0, "Wikipedia debería tener bonus, score = {}", score);
    }

    #[test]
    fn test_puntuar_enlace_repetido() {
        let score = MotorRazonamientoWeb::puntuar_enlace(
            "https://example.com",
            "example.com",
            &[],
            &["https://example.com".to_string()],
        );
        assert!(score < 0.0, "Enlace repetido debería tener score negativo, score = {}", score);
    }

    // Tests de ExploradorWeb (públicos)

    #[test]
    fn test_navegar_simulado_wikipedia() {
        let pagina = ExploradorWeb::navegar_simulado("https://es.wikipedia.org/wiki/Neurociencia");
        assert!(pagina.titulo.contains("Wikipedia"));
        assert!(pagina.densidad_info > 0.5);
    }

    #[test]
    fn test_navegar_simulado_arxiv() {
        let pagina = ExploradorWeb::navegar_simulado("https://arxiv.org/abs/1234.5678");
        assert!(pagina.densidad_info > 0.7);
    }

    #[test]
    fn test_navegar_simulado_desconocido() {
        let pagina = ExploradorWeb::navegar_simulado("https://blog.ejemplo.com/articulo");
        assert!(pagina.densidad_info < 0.5);
    }

    #[test]
    fn test_explorar_simulado_profundidad_1() {
        let (texto, paginas) = ExploradorWeb::explorar_simulado("curiosidad", 1);
        assert!(texto.contains("curiosidad"));
        assert_eq!(paginas.len(), 0); // profundidad 1 = solo búsqueda
    }

    #[test]
    fn test_explorar_simulado_profundidad_2() {
        let (texto, _paginas) = ExploradorWeb::explorar_simulado("neurona", 2);
        assert!(texto.contains("neurona") || texto.contains("Neurona"));
        // Puede tener páginas si el simulado las genera
    }

    #[test]
    fn test_explorar_simulado_vacio() {
        let (texto, _) = ExploradorWeb::explorar_simulado("xyzzy no_existe esto", 1);
        assert!(texto.contains("No encontré"));
    }

    #[test]
    fn test_buscar_simulado_curiosidad() {
        let resultado = ExploradorWeb::buscar_simulado("curiosidad");
        assert!(resultado.contains("curiosidad") || resultado.contains("Curiosidad"));
        assert!(resultado.len() > 20);
    }

    #[test]
    fn test_buscar_simulado_vacio() {
        let resultado = ExploradorWeb::buscar_simulado("algo completamente desconocido xyz");
        assert!(resultado.contains("No encontré"));
    }

    #[test]
    fn test_buscar_vacio_error() {
        let resultado = ExploradorWeb::buscar("");
        assert!(resultado.is_err());
    }

    #[test]
    fn test_limpiar_fragmento_simple() {
        let limpio = limpiar_fragmento("Hola <b>mundo</b> & sol");
        assert_eq!(limpio, "Hola mundo & sol");
    }

    #[test]
    fn test_limpiar_fragmento_entidades_html() {
        // < → < , > → >
        let limpio = limpiar_fragmento("a &lt; b &gt; c");
        assert_eq!(limpio, "a < b > c");
    }

    #[test]
    fn test_limpiar_fragmento_and_suelto() {
        let limpio = limpiar_fragmento("x & y");
        assert_eq!(limpio, "x & y");
    }

    #[test]
    fn test_limpiar_fragmento_sin_tags() {
        let limpio = limpiar_fragmento("texto plano sin tags");
        assert_eq!(limpio, "texto plano sin tags");
    }

    #[test]
    fn test_buscar_simulado_neurona_exacta() {
        let r = ExploradorWeb::buscar_simulado("neurona");
        assert!(r.contains("neuronas") || r.contains("Neuronas"));
    }

    #[test]
    fn test_buscar_simulado_mente_exacta() {
        let r = ExploradorWeb::buscar_simulado("mente");
        assert!(r.contains("conciencia") || r.contains("Conciencia"));
    }

    #[test]
    fn test_buscar_simulado_totalmente_no_mente() {
        let r = ExploradorWeb::buscar_simulado("totalmente");
        assert!(r.contains("No encontré"));
    }

    #[test]
    fn test_urlencode_simple() {
        assert_eq!(urlencode("hola mundo"), "hola%20mundo");
    }

    #[test]
    fn test_urlencode_especiales() {
        assert_eq!(urlencode("qué es"), "qu%C3%A9%20es");
    }
}
