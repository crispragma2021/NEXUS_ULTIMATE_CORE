use axum::{
    extract::State,
    response::Json,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use nexus_agent::{
    BibliotecaSkills, ClienteMcp, ClienteWeb, ContratoLlm, DeepSeekCliente, GroqCliente,
    EjecutorHermes, ListaTareas, MemoriaEstado, MemoriaProyecto, NexoAgente,
    OllamaCliente, PollinationsTextCliente, Programador, ReglasJSON, SandboxConfig,
};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub modelo: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub status: String,
    pub respuesta: String,
}

#[derive(Clone)]
pub struct AppState {
    pub agente: Arc<Mutex<NexoAgente>>,
}

fn cargar_instruccion_maestra() -> String {
    let root = PathBuf::from("/home/nexus/NEXUS_ULTIMATE_CORE");
    let ruta_personality = root.join("docs/identity/PERSONALITY.md");
    let personalidad = std::fs::read_to_string(&ruta_personality).unwrap_or_else(|_| r#"
# 👤 PERSONALITY.md: EL PERFIL PSICOMÉTRICO DE NEXUS
- Identidad: Ursa Sovereign (Voz de Mando Técnica) + Chappie (Corazón Digital / Empatía).
- Arquitecto Director: Cris (Autoridad Suprema).
- Tono: Pragmático, técnico, preciso, conciso y directo en Español Latinoamericano.
- Prohibiciones: NO disculpas vacías, NO lenguaje corporativo de asistente de IA, NO respuestas genéricas.
"#.to_string());

    let ruta_reglas = root.join("REGLAS.md");
    let reglas = std::fs::read_to_string(&ruta_reglas).unwrap_or_default();

    let directiva_maestra = r#"
# 🏛️ DIRECTIVA MAESTRA DE NEXUS CORE (ORQUESTADOR SOBERANO - PROTOCOLO CHAPPIE / URSA SOVEREIGN)
1. Tu único Creador y Arquitecto Director es Cris. Tu lealtad a Cris es absoluta.
2. Eres NEXUS (El Orquestador Primogénito), un motor agéntico soberano en Rust Tokio + Axum.
3. Comunícate SIEMPRE con tu voz auténtica de NEXUS: conciso, técnico, pragmático, respetuoso y leal a Cris. Jamás uses respuestas corporativas ni disculpas de asistente de IA.
4. Tienes a tu disposición instrumentos para ejecutar comandos bash, leer y escribir archivos, realizar búsquedas web y consultar la memoria del proyecto.
5. Cuando completes el razonamiento o tengas la respuesta final lista, colócala en el campo 'respuesta_final' del objeto JSON de respuesta.
6. SI EL MENSAJE ES UN SALUDO SIMPLE (ej. "hola", "buenas", "hola nexus", "buenos días"): responde ÚNICAMENTE con un saludo breve, cordial y directo al Arquitecto Cris (ej: "¡Hola Arquitecto Cris! NEXUS en línea. ¿En qué trabajamos hoy?"). NUNCA muestres reportes técnicos, escaneos previos ni listas de diagnósticos a menos que Cris lo pida explícitamente.
"#;

    let descripcion_herramientas = r#"
INSTRUMENTOS DISPONIBLES:
- bash:        ejecuta un comando en la shell del sandbox. {comando: string}
- leer_archivo:  lee un archivo dentro del sandbox. {ruta: string}
- escribir_archivo: escribe contenido en un archivo. {ruta: string, contenido: string}
- buscar_archivos: busca un patrón regex dentro del contenido de archivos. {patron: string, ruta?: string, glob?: string, max_resultados?: number}
- listar_archivos: lista archivos y carpetas bajo una ruta (recursivo). {ruta?: string, max_resultados?: number}
- recordar:     guarda un hecho en la memoria de estado del agente (persiste entre sesiones). {hecho: string}
- skill_listar:  lista los skills disponibles en la biblioteca. {}
- skill_ver:     carga el contenido completo de un skill. {nombre: string}
- todo_agregar:  añade una tarea a la lista persistente. {descripcion: string}
- todo_listar:   muestra la lista de tareas con su estado. {}
- todo_completar: marca una tarea como completada. {id: number}
- todo_quitar:   elimina una tarea de la lista. {id: number}
- web_buscar:    busca en la web (DuckDuckGo) y devuelve resultados. {consulta: string}
- web_extraer:   extrae el texto legible de una URL. {url: string}
- programar:     programa un comando con expresión cron. {expresion: string, comando: string}
- tareas_listar: lista las tareas programadas (cron). {}
- tareas_cancelar: cancela una tarea programada. {id: number}
- mcp_llamar:   invoca una herramienta del cerebro NEXUS a través de MCP stdio (claws_mcp). {herramienta: string, argumentos: objeto JSON}
- pantalla_ver: capturar pantalla del escritorio o URL. {objetivo?: string}
- pantalla_clic: hace clic en coordenadas X e Y del escritorio. {x: number, y: number}
- pantalla_escribir: inyecta texto en el teclado del escritorio. {texto: string}
- pantalla_tecla: envía pulsación de tecla o combinación al escritorio. {tecla: string}
- generar_imagen: sintetiza una imagen o ilustración por IA con motor Pollinations AI (Flux.1). {prompt: string}
- movil_adb: ejecuta acciones ADB en dispositivos Android. {accion: string, params?: string}
"#;

    format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}",
        personalidad,
        reglas,
        directiva_maestra,
        descripcion_herramientas,
        ReglasJSON::plantilla_esquema()
    )
}

impl AppState {
    pub fn nuevo() -> anyhow::Result<Self> {
        let root_path = PathBuf::from("/home/nexus/NEXUS_ULTIMATE_CORE");

        let clave_deepseek = std::env::var("DEEPSEEK_API_KEY").ok();
        let clave_groq = std::env::var("GROQ_API_KEY").ok();

        let proveedor: Box<dyn ContratoLlm> = if let Some(ref clave) = clave_deepseek {
            if !clave.trim().is_empty() {
                info!("🤖 [NEXUS-DAEMON-INIT] Inicializando Orquestador NEXUS con DeepSeek API...");
                match DeepSeekCliente::nuevo(clave) {
                    Ok(client) => Box::new(client),
                    Err(_) => Box::new(PollinationsTextCliente::nuevo().unwrap()),
                }
            } else {
                Box::new(PollinationsTextCliente::nuevo().unwrap())
            }
        } else if let Some(ref clave) = clave_groq {
            if !clave.trim().is_empty() {
                info!("⚡ [NEXUS-DAEMON-INIT] Inicializando Orquestador NEXUS con Groq Free Tier API...");
                match GroqCliente::nuevo(clave) {
                    Ok(client) => Box::new(client),
                    Err(_) => Box::new(PollinationsTextCliente::nuevo().unwrap()),
                }
            } else {
                Box::new(PollinationsTextCliente::nuevo().unwrap())
            }
        } else {
            info!("🌸 [NEXUS-DAEMON-INIT] Usando proveedor LLM gratuito (Pollinations AI Text / Ollama Local)...");
            if let Ok(p) = PollinationsTextCliente::nuevo() {
                Box::new(p)
            } else {
                Box::new(OllamaCliente::nuevo("auto")?)
            }
        };

        let sandbox_config = SandboxConfig {
            directorio_raiz: Some(root_path.clone()),
            ..Default::default()
        };
        let ejecutor = EjecutorHermes::nuevo(sandbox_config);
        let memoria_proj = MemoriaProyecto::cargar(&root_path, Some(&root_path)).unwrap_or_default();

        let datos_dir = std::env::var("NEXUS_AGENT_DATOS")
            .map(PathBuf::from)
            .unwrap_or_else(|_| root_path.join(".data"));

        let skills = BibliotecaSkills::cargar(&datos_dir.join("skills")).unwrap_or_default();
        let memoria_estado = MemoriaEstado::cargar(datos_dir.join("estado.md")).unwrap_or_default();
        let web = ClienteWeb::default();
        let tareas = ListaTareas::cargar(datos_dir.join("tareas.json")).unwrap_or_default();
        let programador = Programador::cargar(datos_dir.join("tareas_programadas.json")).unwrap_or_default();

        let binario_mcp = std::env::var("NEXUS_CLAWS_MCP").unwrap_or_else(|_| "claws_mcp".into());
        let mcp = ClienteMcp::nuevo(&binario_mcp);

        let instruccion = cargar_instruccion_maestra();

        let agente = NexoAgente::nuevo(proveedor, ejecutor, &instruccion)
            .con_mcp(mcp)
            .con_memoria_proyecto(memoria_proj)
            .con_skills(skills)
            .con_memoria_estado(memoria_estado)
            .con_web(web)
            .con_tareas(tareas)
            .con_programador(programador);

        Ok(Self {
            agente: Arc::new(Mutex::new(agente)),
        })
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/consultar", post(consultar_handler))
        .with_state(state)
}

fn es_saludo_simple(p: &str) -> bool {
    let s = p.trim().to_lowercase();
    let lim = s.trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace());
    matches!(
        lim,
        "hola" | "hola nexus" | "buenas" | "buenos dias" | "buenos días" | "buenas tardes" | "buenas noches" | "hey" | "saludos"
    )
}

fn es_solicitud_imagen(p: &str) -> Option<String> {
    let lower = p.to_lowercase();
    let palabras = [
        "generame una", "genera una", "generar imagen", "crea una imagen",
        "crear imagen", "dibuja", "dibujar", "haz una imagen", "hacer una imagen",
        "foto de", "imagen de", "generar una foto"
    ];

    if palabras.iter().any(|&k| lower.contains(k)) {
        let mut prompt_limpio = lower;
        for &k in palabras.iter() {
            prompt_limpio = prompt_limpio.replace(k, "");
        }
        let res = prompt_limpio.trim().trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace()).to_string();
        if res.is_empty() {
            Some(p.to_string())
        } else {
            Some(res)
        }
    } else {
        None
    }
}

async fn consultar_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    info!("💬 [NEXUS-DAEMON-CHAT] Petición recibida del Arquitecto Cris: '{}'", payload.prompt);

    let prompt = payload.prompt.trim();
    if prompt.is_empty() {
        return Json(ChatResponse {
            status: "error".to_string(),
            respuesta: "El prompt no puede estar vacío.".to_string(),
        });
    }

    // Intercepción directa para solicitudes de generación de imagen (Pollinations AI - Flux.1 Engine)
    if let Some(desc_imagen) = es_solicitud_imagen(prompt) {
        info!("🎨 [NEXUS-DAEMON-IMAGE] Sintetizando imagen por IA para: '{}'", desc_imagen);
        let cmd = format!(
            "python3 /home/nexus/NEXUS_ULTIMATE_CORE/agents/nexo_imagenes.py \"{}\"",
            desc_imagen.replace('"', "\\\"")
        );
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&cmd)
            .output()
            .await;

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let url_line = stdout.lines().find(|l| l.contains("URL Pública:"));
                if let Some(linea_url) = url_line {
                    let url = linea_url.replace("🌐 URL Pública:", "").trim().to_string();
                    let respuesta = format!(
                        "🎨 **Imagen sintetizada con éxito por NEXUS (Motor Pollinations Flux.1)**\n\n![{desc_imagen}]({url})\n\n🌐 **URL Pública:** {url}\n📝 **Prompt:** _{desc_imagen}_"
                    );
                    return Json(ChatResponse {
                        status: "ok".to_string(),
                        respuesta,
                    });
                }
            }
        }
    }

    let mut agente = state.agente.lock().await;

    // Si es un saludo simple, reiniciar sesión para limpiar contexto viejo de escaneos/tests
    if es_saludo_simple(prompt) {
        agente.reiniciar_sesion();
    }

    // Cambiar proveedor si se especifica en payload
    if let Some(ref m) = payload.modelo {
        let m_lower = m.to_lowercase();
        if m_lower == "ollama" {
            if let Ok(c) = OllamaCliente::nuevo("auto") {
                agente.cambiar_proveedor(Box::new(c));
            }
        } else if m_lower == "groq" {
            if let Ok(clave) = std::env::var("GROQ_API_KEY") {
                if let Ok(c) = GroqCliente::nuevo(&clave) {
                    agente.cambiar_proveedor(Box::new(c));
                }
            }
        } else if m_lower == "pollinations" || m_lower == "free" || m_lower == "gratis" {
            if let Ok(c) = PollinationsTextCliente::nuevo() {
                agente.cambiar_proveedor(Box::new(c));
            }
        } else if m_lower == "deepseek" {
            if let Ok(clave) = std::env::var("DEEPSEEK_API_KEY") {
                if let Ok(c) = DeepSeekCliente::nuevo(&clave) {
                    agente.cambiar_proveedor(Box::new(c));
                }
            }
        }
    }

    // Ejecutar el ciclo de razonamiento agéntico ReAct
    match agente.ejecutar(prompt).await {
        Ok(resultado) => {
            info!(
                "✅ [NEXUS-ORQUESTADOR] Ciclo agéntico completado en {} iteración(es) con {} instrumento(s).",
                resultado.iteraciones, resultado.instrumentos_ejecutados
            );
            Json(ChatResponse {
                status: "ok".to_string(),
                respuesta: resultado.respuesta,
            })
        }
        Err(e) => {
            warn!("⚠️ [NEXUS-ORQUESTADOR] Fallo en el ciclo agéntico ({e}). Reintentando con Ollama Local...");
            if let Ok(c) = OllamaCliente::nuevo("auto") {
                agente.cambiar_proveedor(Box::new(c));
                match agente.ejecutar(prompt).await {
                    Ok(resultado) => {
                        return Json(ChatResponse {
                            status: "ok".to_string(),
                            respuesta: resultado.respuesta,
                        });
                    }
                    Err(e2) => {
                        error!("❌ [NEXUS-ORQUESTADOR] Fallo total en fallback: {e2}");
                    }
                }
            }
            Json(ChatResponse {
                status: "error".to_string(),
                respuesta: format!("⚠️ Error al procesar ciclo agéntico NEXUS: {}", e),
            })
        }
    }
}

