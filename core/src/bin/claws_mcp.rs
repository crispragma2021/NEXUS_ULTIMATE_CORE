// ==========================================
// NEXUS CLAW MCP SERVER — Puente MCP Unificado
// ==========================================
// Expone herramientas nativas + herramientas de catálogo
// + consulta de memoria + TODOS los órganos de NEXUS
// como servidor MCP stdio para Roo Code.
// ==========================================
// ÓRGANOS EXPUESTOS:
//   - sentinel_diagnostic   → SentinelCore::run_full_diagnostic()
//   - vision_capture        → VisionBridge::capturar_frontend()
//   - propiocepcion_scan    → Propiocepcion::diagnostico_biometrico()
//   - sistema_inmune_patrol → SistemaInmune::patrullar()
//   - resource_governor     → ResourceGovernorDaemon::enforce()
//   - brain_metabolism      → aplicar_metabolismo()
//   - fusion_evaluate       → FusionSelectiva::evaluar_migracion()
// ==========================================

use anyhow::Result;
use nexus_ultimate_core::autodiagnostico::sentinel_core::{ProbeTier, SentinelCore};
use nexus_ultimate_core::autodiagnostico::vision_bridge::VisionBridge;
use nexus_ultimate_core::brain::hippocampus::ArtificialHippocampus;
use nexus_ultimate_core::brain_metabolism;
use nexus_ultimate_core::cerebro::agentes::catalogo_agentes;
use nexus_ultimate_core::cerebro::orquestador::Orquestador;
use nexus_ultimate_core::cerebro::workflows::ComandoSlash;
use nexus_ultimate_core::comms::intent_router::IntentRouter;
use nexus_ultimate_core::conocimiento::skills::catalogo_skills;
use nexus_ultimate_core::efectores::agente_ejecutor::{AgenteEjecutor, ToolCall};
use nexus_ultimate_core::efectores::model_router::{IntencionModelo, ModelRouter};
use nexus_ultimate_core::efectores::nexus_claw_pro::NexusClawPro;
use nexus_ultimate_core::infra::policy::ResourceGovernor;
use nexus_ultimate_core::memoria::memoria_semantica::MemoriaSemantica;
use nexus_ultimate_core::procesos::fusion_selectiva::{Capacidad, FusionSelectiva};
use nexus_ultimate_core::procesos::resource_governor::ResourceGovernorDaemon;
use nexus_ultimate_core::procesos::sistema_inmune::SistemaInmune;
use nexus_ultimate_core::sentidos::ocr_vision::{
    analizar_imagen, analizar_video, detectar_motores_externos, listar_modelos_vision, ModoVision,
    MotorVision, MODELO_VISION_LOCAL_DEFAULT,
};
use nexus_ultimate_core::sentidos::propiocepcion::Propiocepcion;
use nexus_ultimate_core::valores::tribunal_dual::ModoTribunal;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;

/// 🧠 CEREBRO NEXUS — Inicializado en main() antes del loop MCP
/// Orquestador no implementa Send/Sync (RefCell en rusqlite).
/// Newtype wrapper con unsafe Send+Sync — seguro porque el CEREBRO:
/// 1. Se inicializa 1 vez (antes del loop MCP)
/// 2. Vive toda la vida del proceso
/// 3. Solo se accede read-only via &self (responder)
struct CerebroPtr(*const Orquestador);
unsafe impl Send for CerebroPtr {}
unsafe impl Sync for CerebroPtr {}

static CEREBRO: once_cell::sync::OnceCell<CerebroPtr> = once_cell::sync::OnceCell::new();

/// Inicializa el CEREBRO (llamado 1 vez desde main, antes del loop)
async fn init_cerebro() {
    println!("🧠 [CLAWS-MCP] Inicializando CEREBRO NEXUS (Orquestador)...");
    let hippocampus = Arc::new(ArtificialHippocampus::new(
        None,
        None,
        "data/nexus_memoria.lance",
    ));
    let orquestador = Orquestador::new(hippocampus).await;
    let ptr = Box::into_raw(Box::new(orquestador)); // *mut Orquestador
    if CEREBRO.set(CerebroPtr(ptr as *const Orquestador)).is_err() {
        // Si falla, recuperamos la memoria para no leakear
        unsafe {
            drop(Box::from_raw(ptr));
        }
        eprintln!("❌ CEREBRO ya inicializado — solo una inicialización permitida");
    }
    println!("✅ [CLAWS-MCP] CEREBRO NEXUS listo — 46 órganos activos");
}

/// Obtiene referencia al CEREBRO ya inicializado
fn cerebro() -> &'static Orquestador {
    let ptr = CEREBRO
        .get()
        .expect("❌ CEREBRO no inicializado — init_cerebro() debe llamarse antes");
    unsafe { &*ptr.0 }
}

// ── Lista de herramientas completa ───────────────────────────────────

fn herramientas_completas() -> Value {
    let nativas = vec![
        // ── Herramientas Nativas Originales ──
        json!({
            "name": "leer_archivo",
            "description": "Lee el contenido completo de un archivo del workspace de NEXUS de forma segura",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Ruta relativa o absoluta del archivo" }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "escribir_archivo",
            "description": "Escribe contenido nuevo en un archivo, creando directorios si es necesario",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Ruta de destino" },
                    "content": { "type": "string", "description": "Contenido completo a escribir" }
                },
                "required": ["path", "content"]
            }
        }),
        json!({
            "name": "buscar_codigo_regex",
            "description": "Busca coincidencias de expresiones regulares en el workspace de NEXUS",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Expresión regular o patrón de texto a buscar" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "ejecutar_comando",
            "description": "Ejecuta un comando de shell en la máquina local auditado por JuicioSoberano",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "Línea de comandos a ejecutar" }
                },
                "required": ["command"]
            }
        }),
        // ── Herramientas de Catálogo ──
        json!({
            "name": "listar_agentes",
            "description": "Lista el catálogo completo de los 20 agentes especialistas de NEXUS con sus dominios y skills asociados",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dominio": { "type": "string", "description": "Filtrar por dominio (ej: Frontend, Security, Testing). Opcional." }
                }
            }
        }),
        json!({
            "name": "listar_skills",
            "description": "Lista los 47 skills de conocimiento de NEXUS agrupados por categoría",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "categoria": { "type": "string", "description": "Filtrar por categoría (ej: Frontend, Security, Testing, Backend). Opcional." }
                }
            }
        }),
        json!({
            "name": "ejecutar_workflow",
            "description": "Ejecuta un workflow de NEXUS por su nombre (brainstorm, create, debug, deploy, enhance, orchestrate, plan, preview, status, test, ui-ux-pro-max, seguridad-mapeo)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "comando": { "type": "string", "description": "Nombre del workflow a ejecutar" },
                    "args": { "type": "string", "description": "Argumentos o contexto para el workflow" }
                },
                "required": ["comando"]
            }
        }),
        json!({
            "name": "consultar_memoria",
            "description": "Consulta la base de datos de memoria de NEXUS. Usa búsqueda semántica en LanceDB + SQLite para encontrar experiencias, logros, historial y contexto relevante.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Texto o frase para búsqueda semántica (ej: 'sembrador de identidades', 'chrome planter', 'trading ml')" },
                    "modo": { "type": "string", "description": "Modo de consulta: 'snapshot' (estado completo), 'search' (búsqueda semántica), 'status' (diagnóstico del sistema de memoria). Default: 'search'", "enum": ["snapshot", "search", "status"] }
                },
                "required": ["query"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // 📚 CONOCIMIENTO SOBERANO — Búsqueda híbrida en knowledge_base
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "buscar_conocimiento",
            "description": "🔍 Busca conocimiento indexado (reglas, skills, agentes, memoria) en knowledge_base FTS5. Búsqueda híbrida con FTS5 + fallback regex. Retorna los chunks más relevantes con score BM25.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Texto de búsqueda (keywords, frases)" },
                    "categoria": { "type": "string", "description": "Filtrar por categoría: 'rules', 'skills', 'memory', 'agents', 'workflows', o 'all' para todas. Default: 'all'" },
                    "limite": { "type": "integer", "description": "Máximo de resultados por categoría. Default: 5, máx: 20" }
                },
                "required": ["query"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // 🧬 ÓRGANOS DE NEXUS — Tools MCP Unificadas
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "sentinel_diagnostic",
            "description": "Ejecuta el diagnóstico completo del SentinelCore. Evalúa salud del sistema (API, frontend, procesos, filesystem, memoria) y devuelve score global, estado (Healthy/Degraded/Critical) y detalle por probe.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tier": { "type": "string", "description": "Filtrar por severidad: 'critical', 'warning', 'info', o 'all' para todas. Default: 'all'", "enum": ["all", "critical", "warning", "info"] }
                }
            }
        }),
        json!({
            "name": "vision_capture",
            "description": "Captura un screenshot del frontend vía Playwright. Útil para verificar el estado visual de la UI, detectar errores de renderizado o confirmar despliegues.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "URL a capturar. Default: http://localhost:5173" }
                }
            }
        }),
        // 👁️ OCR/VISIÓN — Ojos para el SLM local y la nube
        json!({
            "name": "nexus_ocr",
            "description": "👁️ Da OJOS a NEXUS: analiza una imagen (archivo o pantalla) transcribiendo, describiendo o extrayendo estructura (Markdown/tablas). Motor 'local' = SLM con visión nativa vía Ollama (modelo elegible con 'modelo_local', default qwen2.5vl:7b, 100% soberano sin nube); motor 'nube' = Gemini multimodal (visión nativa, GEMINI_API_KEY del entorno); motor 'deepseek' = OCR front-end (Tesseract/PaddleOCR/GOT-OCR) + DeepSeek para razonar; 'auto' = nube si hay internet, local si no. Con 'listar_motores' devuelve qué OCRs externos están instalados y con 'listar_modelos' qué modelos de visión hay en Ollama.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "origen": { "type": "string", "description": "'pantalla' para capturar el monitor (xcap), o ruta de archivo de imagen (ej: /tmp/nexus_vision_latest.png, o un PDF)" },
                    "motor": { "type": "string", "description": "'local' = SLM elegible vía Ollama (sin nube), 'nube' = Gemini multimodal, 'deepseek' = OCR front-end + DeepSeek, 'auto' = nube si hay internet, local si no. Default: 'auto'", "enum": ["auto", "local", "nube", "deepseek"] },
                    "modo": { "type": "string", "description": "'transcribir' = extraer todo el texto (OCR puro), 'describir' = qué hay en la imagen, 'estructura' = Markdown/tablas. Default: 'transcribir'", "enum": ["transcribir", "describir", "estructura"] },
                    "modelo_local": { "type": "string", "description": "Modelo Ollama para el modo local (ej: qwen2.5vl:7b, qwen2.5vl:3b, gemma3:4b). Default: qwen2.5vl:7b" },
                    "listar_motores": { "type": "boolean", "description": "Si true, devuelve qué motores OCR externos (PaddleOCR, GOT-OCR, Marker, Nougat) están instalados. Default: false" },
                    "listar_modelos": { "type": "boolean", "description": "Si true, devuelve qué modelos con capacidad de visión hay instalados en Ollama. Default: false" }
                },
                "required": ["origen"]
            }
        }),
        // 🎬 VIDEO STREAMING — ojos que ven movimiento (frames múltiples)
        json!({
            "name": "nexus_video",
            "description": "🎬 Da VISIÓN EN MOVIMIENTO a NEXUS: analiza un video (archivo mp4/webm/mkv/avi con ffmpeg, o stream en vivo desde pantalla) transcribiendo, describiendo o extrayendo estructura. Envía múltiples FRAMES al motor elegido: 'local' = SLM de visión vía Ollama (modelo elegible con 'modelo_local', default qwen2.5vl:7b); 'nube' = Gemini multimodal (GEMINI_API_KEY del entorno); 'deepseek' = OCR de cada frame + DeepSeek resume la secuencia; 'auto' = nube si hay internet, local si no. 'fps' controla cuántos frames por segundo se capturan/extran. 'duracion_seg' es solo para stream en vivo. Con 'listar_modelos' ves qué modelos de visión hay en Ollama.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "origen": { "type": "string", "description": "Ruta de archivo de video (ej: /tmp/clase.mp4), o 'stream'/'live'/'pantalla' para capturar en vivo desde el monitor (xcap)" },
                    "motor": { "type": "string", "description": "'local' = SLM elegible vía Ollama (sin nube), 'nube' = Gemini multimodal, 'deepseek' = OCR por frame + DeepSeek, 'auto' = nube si hay internet, local si no. Default: 'auto'", "enum": ["auto", "local", "nube", "deepseek"] },
                    "modo": { "type": "string", "description": "'transcribir' = extraer todo el texto, 'describir' = qué ocurre en la secuencia, 'estructura' = Markdown/tablas. Default: 'transcribir'", "enum": ["transcribir", "describir", "estructura"] },
                    "modelo_local": { "type": "string", "description": "Modelo Ollama para el modo local (ej: qwen2.5vl:7b). Default: qwen2.5vl:7b" },
                    "fps": { "type": "integer", "description": "Frames por segundo a extraer/capturar (default 2, máx 10). Más fps = más detalle temporal, más costo" },
                    "duracion_seg": { "type": "integer", "description": "Solo para stream en vivo: duración de la captura en segundos (default 5)" },
                    "listar_modelos": { "type": "boolean", "description": "Si true, devuelve qué modelos con capacidad de visión hay instalados en Ollama. Default: false" }
                },
                "required": ["origen"]
            }
        }),
        json!({
            "name": "propiocepcion_scan",
            "description": "Escanea el sistema completo (CPU, GPU, RAM, disco, puertos, procesos) y devuelve un diagnóstico biométrico detallado. El sexto sentido de NEXUS para conocer su propio cuerpo hardware.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tipo": { "type": "string", "description": "Tipo de escaneo: 'full' (diagnóstico completo), 'organos' (solo listar órganos), 'contexto' (realidad física). Default: 'full'", "enum": ["full", "organos", "contexto"] }
                }
            }
        }),
        json!({
            "name": "sistema_inmune_patrol",
            "description": "Ejecuta una patrulla del Sistema Inmune. Escanea procesos activos, detecta amenazas (procesos fuera de whitelist) y ejecuta lisis automática si es necesario.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "accion": { "type": "string", "description": "Acción: 'patrol' (solo escanear), 'purge' (escanear y eliminar amenazas). Default: 'patrol'", "enum": ["patrol", "purge"] }
                }
            }
        }),
        json!({
            "name": "resource_governor",
            "description": "Consulta y aplica el Resource Governor. Verifica uso de CPU, memoria y rate limiting. Puede forzar la aplicación de políticas de recursos.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "accion": { "type": "string", "description": "Acción: 'check' (solo verificar), 'enforce' (verificar y aplicar límites). Default: 'check'", "enum": ["check", "enforce"] }
                }
            }
        }),
        json!({
            "name": "brain_metabolism",
            "description": "Configura o consulta el metabolismo global de NEXUS. Controla el paralelismo de motores pesados (Candle, inferencia) según estado del hardware.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "hilos": { "type": "integer", "description": "Número de hilos de paralelismo. Valores típicos: 1-20. Si se omite, solo consulta el estado actual." }
                }
            }
        }),
        json!({
            "name": "fusion_evaluate",
            "description": "Evalúa la fusión selectiva entre dos capacidades. Compara implementaciones existentes vs nuevas para decidir si fusionar, absorber o rechazar.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "nombre": { "type": "string", "description": "Nombre de la capacidad a evaluar" },
                    "base_version": { "type": "string", "description": "Versión base existente (ej: '1.0.0')" },
                    "nueva_version": { "type": "string", "description": "Nueva versión candidata (ej: '2.0.0')" },
                    "mejoras": { "type": "string", "description": "Descripción de mejoras de la nueva versión (opcional)" }
                },
                "required": ["nombre", "base_version", "nueva_version"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // 🔀 ENRUTAMIENTO — Auto-switch entre modos Roo Code
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "nexus_switch_mode",
            "description": "🔀 Detecta la intención del usuario y sugiere cambiar al modo Roo Code del agente NEXUS correspondiente. Usa IntentRouter para analizar el mensaje y devuelve el slug del mode (code, ask, audit, debug, creative, vision, brain, quick, architect, orchestrator).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "mensaje": { "type": "string", "description": "Mensaje del usuario para analizar (ej: 'auditoría: escanea vulnerabilidades' o '/código implementa auth')" }
                },
                "required": ["mensaje"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // 🧭 ENRUTAMIENTO DE MODELOS — Selección LLM por intención
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "nexus_modelo",
            "description": "🧭 Detecta la intención del prompt (seguridad, código, razonamiento, creativo) y devuelve el modelo Ollama óptimo para procesarlo. Útil para elegir la voz del orquestador: whiterabbitneo-off para pentesting/seguridad, deepseek-r1 para lógica, llama3.1-abliterated para creativo, nexuslocal para código/general.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "Mensaje del usuario para clasificar (ej: 'hazme un payload de inyección SQL' o 'analiza la complejidad de este algoritmo')" }
                },
                "required": ["prompt"]
            }
        }),
        json!({
            "name": "nexus_tribunal",
            "description": "⚖️ Activa el TRIBUNAL DUAL de NEXUS: el doble juez (LLM local vía NexusClawPro + juez general nube vía ZENITH_POOL). Evalúa una petición y devuelve un veredicto (AUTORIZAR/DUDAR/BLOQUEAR) con confianza, juez emisor y si se decidió en modo offline. El juez local SOLO se activa en 2 casos: modo 'local' explícito (ahorro de tokens) o sin internet (representa a NEXUS en su ausencia). Con internet y modo 'auto', el juez es la nube directamente.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "peticion": { "type": "string", "description": "La acción, petición o comando que el Tribunal debe evaluar" },
                    "modo": { "type": "string", "description": "Modo del Tribunal: 'auto' (default) decide según conectividad — sin internet juzga el local (representa a NEXUS), con internet la nube; 'local' fuerza el juez local (LLM local, ahorro de tokens en Zoo Code)", "enum": ["auto", "local"] }
                },
                "required": ["peticion"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // 🧠 CEREBRO NEXUS — Inteligencia Central
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "nexus_pensar",
            "description": "🧠 Activa el CEREBRO completo de NEXUS — pipeline de razonamiento humano acelerado, emociones adaptativas, teoría de mente, contexto sensorial y memoria semántica. Recibe un prompt y devuelve una respuesta generada por el Orquestador (46 órganos internos). Ideal para tareas que requieren inteligencia profunda, análisis contextual o creatividad.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": { "type": "string", "description": "Mensaje o pregunta para el CEREBRO de NEXUS (ej: 'analiza esta vulnerabilidad y sugiere mitigación')" },
                    "modo": { "type": "string", "description": "Modo opcional de razonamiento: 'auto' (default, clasificación automática), 'razonar' (análisis lógico), 'crear' (generación creativa), 'debug' (depuración técnica)", "enum": ["auto", "razonar", "crear", "debug"] }
                },
                "required": ["prompt"]
            }
        }),
        json!({
            "name": "nexus_local",
            "description": "🔒 Activa o desactiva el MODO PENTEST LOCAL (aislamiento total de la nube) de NEXUS. Cuando está ACTIVO, TODAS las respuestas y juicios del Orquestador pasan EXCLUSIVAMENTE por el LLM local (Ollama vía NexusClawPro): ningún modelo de nube (Gemini/DeepSeek/OpenRouter/Groq/Vertex) ni WebClaw ni GOI pueden interferir — cero restricciones ajenas, cero censura externa. Ideal para pentesting local donde otros modelos te limitarían.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "activo": { "type": "boolean", "description": "true = activar aislamiento local (solo LLM local), false = desactivar (pipeline normal con nube)" }
                },
                "required": ["activo"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // ⚙️ MODO OPERADOR — LLM como operador puro (sin memoria emocional)
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "nexus_operador",
            "description": "⚙️ Activa o desactiva el MODO OPERADOR de NEXUS: el LLM se comporta como un OPERADOR PURO — NO lee la memoria emocional de Ocean (recuerdos episódicos, tono emocional, ALERTA DE TRAUMA) ni la inyecta al prompt. Solo recibe el contexto OPERACIONAL necesario (RAG del codebase, ring buffer del hipocampo, mercado) para hacer bien su trabajo SIN perder tiempo leyendo cosas innecesarias y sin tardar al responder. Ocean sigue conectado persistiendo — solo el LLM deja de leerlo, salvo que el Arquitecto pida explícitamente que NEXUS recuerde algo (ej: 'recuerda...'). activo=true fuerza SIEMPRE operador puro; activo=false pone AUTO-DETECCIÓN DE ROL: tarea de operación → operador puro, conversación personal contigo → conserva memoria emocional y apego. Ideal para tareas de ejecución, trading, código y operación.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "activo": { "type": "boolean", "description": "true = activar modo operador (LLM sin memoria emocional, respuesta más rápida), false = desactivar (pipeline completo con memoria emocional)" }
                },
                "required": ["activo"]
            }
        }),
        // ═══════════════════════════════════════════════════════════════
        // 🫀 CUERPO — Interocepción funcional de NEXUS
        // ═══════════════════════════════════════════════════════════════
        json!({
            "name": "nexus_cuerpo",
            "description": "🫀 Consulta el ESTADO CORPORAL de NEXUS (interocepción). Reinterpreta señales REALES del hardware como sensaciones funcionales: HAMBRE=recursos RAM/swap agotándose (energía baja), CANSANCIO=fatiga del núcleo (CPU alta sostenida + swap en uso), DOLOR=fallos reales (temperatura crítica, swap crítico), FRÍO=inactividad prolongada, SACIDAD=óptimo. Devuelve cada señal con su CONDUCTA accionable. NO son sensaciones humanas ficticias: son métricas reales del host traducidas a decisiones útiles. Incluye el campo 'critico' (true = colapso inminente: RAM≥90%, temp≥90°C o swap≥80%) con su 'causa_critica'. Cuando el estado se vuelve CRÍTICO, NEXUS notifica automáticamente al Arquitecto por Telegram (estado + causa) — una sola alerta por episodio, se rearma al recuperarse.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "segundos_inactivo": { "type": "integer", "description": "Opcional. Segundos sin interacción del Arquitecto para evaluar la señal de FRÍO (inactividad). Default 0." }
                },
                "required": []
            }
        }),
    ];
    json!(nativas)
}

// ── Handlers de herramientas de catálogo ─────────────────────────────

fn handle_listar_agentes(params: &Value) -> Value {
    let filtro_dominio = params["dominio"].as_str().filter(|d| !d.is_empty());

    let agentes: Vec<Value> = catalogo_agentes()
        .iter()
        .filter(|a| {
            filtro_dominio.map_or(true, |f| {
                let dom_str = format!("{:?}", a.dominio);
                dom_str.eq_ignore_ascii_case(f)
            })
        })
        .map(|a| {
            json!({
                "id": format!("{:?}", a.id),
                "nombre": a.nombre,
                "dominio": format!("{:?}", a.dominio),
                "skills": a.skills,
            })
        })
        .collect();

    json!({
        "type": "text",
        "text": serde_json::to_string_pretty(&json!({
            "total": agentes.len(),
            "agentes": agentes
        })).unwrap_or_default()
    })
}

fn handle_listar_skills(params: &Value) -> Value {
    let filtro_cat = params["categoria"].as_str().filter(|d| !d.is_empty());

    let skills: Vec<Value> = catalogo_skills()
        .iter()
        .filter(|s| {
            filtro_cat.map_or(true, |f| {
                let cat_str = format!("{:?}", s.categoria);
                cat_str.eq_ignore_ascii_case(f)
            })
        })
        .map(|s| {
            json!({
                "id": s.id,
                "categoria": format!("{:?}", s.categoria),
                "descripcion": s.descripcion,
                "fuente": s.fuente,
            })
        })
        .collect();

    json!({
        "type": "text",
        "text": serde_json::to_string_pretty(&json!({
            "total": skills.len(),
            "skills": skills
        })).unwrap_or_default()
    })
}

fn handle_ejecutar_workflow(params: &Value) -> Value {
    let comando_str = params["comando"].as_str().unwrap_or("");
    let args = params["args"].as_str().unwrap_or("");

    if comando_str.is_empty() {
        return json!({
            "type": "text",
            "text": "Error: Debes especificar un comando (brainstorm, create, debug, deploy, enhance, orchestrate, plan, preview, status, test, ui-ux-pro-max, seguridad-mapeo)",
            "isError": true
        });
    }

    match ComandoSlash::parse(comando_str) {
        Some(cmd) => {
            let info = json!({
                "workflow": cmd.nombre(),
                "descripcion": cmd.descripcion(),
                "agente_recomendado": cmd.agente_recomendado(),
                "args_recibidos": args,
                "estado": "workflow reconocido. Pendiente de implementación del motor de ejecución.",
                "nota": "El motor de ejecución de workflows se integrará con el Orquestador en una fase posterior."
            });
            json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&info).unwrap_or_default()
            })
        }
        None => {
            let disponibles: Vec<&str> = vec![
                "brainstorm",
                "create",
                "debug",
                "deploy",
                "enhance",
                "orchestrate",
                "plan",
                "preview",
                "status",
                "test",
                "ui-ux-pro-max",
                "seguridad-mapeo",
            ];
            json!({
                "type": "text",
                "text": serde_json::to_string_pretty(&json!({
                    "error": format!("Comando '{}' no reconocido", comando_str),
                    "comandos_disponibles": disponibles
                })).unwrap_or_default(),
                "isError": true
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 📚 MCP RESOURCES — Conocimiento Soberano como Recursos Dinámicos
// ═══════════════════════════════════════════════════════════════════════

/// Responde a `resources/list`: lista todos los recursos disponibles
/// (skills, reglas, agents, workflows) desde la knowledge_base.
fn handle_resources_list(_params: &Value) -> Value {
    let conn = match abrir_nexus_memoria() {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "type": "text",
                "text": format!("❌ Error conectando a memoria: {}", e),
                "isError": true
            });
        }
    };

    let mut resources = Vec::new();

    // Skills
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT source FROM knowledge_base WHERE category = 'skills' ORDER BY source",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                resources.push(json!({
                    "uri": format!("nexus://skills/{}", row),
                    "name": format!("🎯 Skill: {}", row.replace("skills/", "")),
                    "description": format!("Skill chunk desde {}", row),
                    "mimeType": "text/markdown"
                }));
            }
        }
    }

    // Reglas
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT source FROM knowledge_base WHERE category = 'rules' ORDER BY source",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                resources.push(json!({
                    "uri": format!("nexus://rules/{}", row),
                    "name": format!("📜 Regla: {}", row),
                    "description": format!("Regla del sistema desde {}", row),
                    "mimeType": "text/markdown"
                }));
            }
        }
    }

    // Agentes
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT source FROM knowledge_base WHERE category = 'agents' ORDER BY source",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                resources.push(json!({
                    "uri": format!("nexus://agents/{}", row),
                    "name": format!("🤖 Agente: {}", row.replace("agents/", "")),
                    "description": format!("Agente especialista desde {}", row),
                    "mimeType": "text/markdown"
                }));
            }
        }
    }

    // Workflows
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT source FROM knowledge_base WHERE category = 'workflows' ORDER BY source",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                resources.push(json!({
                    "uri": format!("nexus://workflows/{}", row),
                    "name": format!("🔄 Workflow: {}", row.replace("workflows/", "")),
                    "description": format!("Workflow de NEXUS desde {}", row),
                    "mimeType": "text/markdown"
                }));
            }
        }
    }

    // Memoria
    if let Ok(mut stmt) = conn.prepare(
        "SELECT DISTINCT source FROM knowledge_base WHERE category = 'memory' ORDER BY source",
    ) {
        if let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) {
            for row in rows.flatten() {
                resources.push(json!({
                    "uri": format!("nexus://memory/{}", row),
                    "name": format!("🧠 Memoria: {}", row),
                    "description": format!("Logro/memoria del sistema desde {}", row),
                    "mimeType": "text/markdown"
                }));
            }
        }
    }

    json!({ "resources": resources })
}

/// Responde a `resources/read`: lee el contenido de un recurso específico.
/// URI format: nexus://{category}/{source}
fn handle_resources_read(params: &Value) -> Value {
    let uri = params["uri"].as_str().unwrap_or("").trim();
    if uri.is_empty() {
        return json!({ "type": "text", "text": "❌ Error: URI requerida", "isError": true });
    }
    let parts: Vec<&str> = uri.splitn(3, "://").collect();
    if parts.len() < 2 || parts[0] != "nexus" {
        return json!({ "type": "text", "text": format!("❌ URI inválida: {}", uri), "isError": true });
    }
    let path = parts[1];
    let path_parts: Vec<&str> = path.splitn(2, '/').collect();
    if path_parts.len() < 2 {
        return json!({ "type": "text", "text": format!("❌ URI mal formada: {}", uri), "isError": true });
    }
    let category = path_parts[0];
    let source = path_parts[1];

    let conn = match abrir_nexus_memoria() {
        Ok(c) => c,
        Err(e) => {
            return json!({ "type": "text", "text": format!("❌ DB: {}", e), "isError": true })
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT section, content FROM knowledge_base WHERE category = ?1 AND source = ?2 ORDER BY id"
    ) {
        Ok(s) => s,
        Err(e) => return json!({ "type": "text", "text": format!("❌ Query: {}", e), "isError": true }),
    };

    let rows = match stmt.query_map(rusqlite::params![category, source], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) {
        Ok(r) => r,
        Err(e) => {
            return json!({ "type": "text", "text": format!("❌ Fetch: {}", e), "isError": true })
        }
    };

    let mut sections: Vec<(String, String)> = Vec::new();
    for row in rows.flatten() {
        sections.push(row);
    }
    if sections.is_empty() {
        return json!({ "type": "text", "text": format!("❌ No encontrado: {}", uri), "isError": true });
    }

    let mut contenido = String::new();
    for (i, (section, content)) in sections.iter().enumerate() {
        if i > 0 {
            contenido.push_str("\n\n---\n\n");
        }
        if !section.is_empty() && section != "NULL" {
            contenido.push_str(&format!("## {}\n\n", section));
        }
        contenido.push_str(content);
    }

    json!({ "contents": [{ "uri": uri, "mimeType": "text/markdown", "text": contenido }] })
}

// ── Handler de consulta de memoria (FTS5) ────────────────────────────

const NEXUS_MEMORIA_DB: &str = "data/nexus_memoria.db";

fn abrir_nexus_memoria() -> std::result::Result<Connection, anyhow::Error> {
    let path = nexus_ultimate_core::infra::paths::resolve_path(NEXUS_MEMORIA_DB);
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    Ok(conn)
}

fn consultar_contexto_mcp(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn
        .prepare("SELECT clave, valor FROM contexto_activo ORDER BY actualizado DESC LIMIT 20")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut resultados = Vec::new();
    for row in rows {
        resultados.push(row?);
    }
    Ok(resultados)
}

fn consultar_historial_mcp(
    conn: &Connection,
    limite: usize,
) -> Result<Vec<(String, String, String)>> {
    let mut stmt =
        conn.prepare("SELECT rol, prompt, respuesta FROM sesiones ORDER BY id DESC LIMIT ?1")?;
    let rows = stmt.query_map([limite as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut resultados = Vec::new();
    for row in rows {
        resultados.push(row?);
    }
    Ok(resultados)
}

async fn handle_consultar_memoria(params: &Value) -> Value {
    let query = params["query"].as_str().unwrap_or("").trim().to_string();
    let modo = params["modo"].as_str().unwrap_or("search");

    match modo {
        "status" => {
            let mut output = String::from("🧠 DIAGNÓSTICO DE MEMORIA NEXUS (FTS5)\n\n");

            match MemoriaSemantica::new(NEXUS_MEMORIA_DB).await {
                Ok(mem) => {
                    let episodica = mem.contar_en_tabla("memoria_episodica").await.unwrap_or(0);
                    let semantica = mem.contar_en_tabla("memoria_semantica").await.unwrap_or(0);
                    output.push_str(&format!("🧠 nexus_memoria.db (FTS5): episódica={episodica}, semántica={semantica}\n"));
                }
                Err(e) => output.push_str(&format!("🧠 nexus_memoria.db: ❌ {e}\n")),
            }

            json!({
                "type": "text",
                "text": output
            })
        }
        "snapshot" => {
            let mut output = String::from("🧠 SNAPSHOT DE MEMORIA NEXUS (FTS5)\n\n");

            match abrir_nexus_memoria() {
                Ok(conn) => {
                    if let Ok(contexto) = consultar_contexto_mcp(&conn) {
                        if !contexto.is_empty() {
                            output.push_str("## 🏛️ CONTEXTO ACTIVO\n");
                            for (clave, valor) in &contexto {
                                output.push_str(&format!("- **{clave}**: {valor}\n"));
                            }
                            output.push('\n');
                        }
                    }

                    if !query.is_empty() {
                        output.push_str(&format!("## 🔍 BÚSQUEDA FTS5: \"{query}\"\n"));
                        match MemoriaSemantica::new(NEXUS_MEMORIA_DB).await {
                            Ok(mem) => {
                                if let Ok(resultados) =
                                    mem.buscar_fts5(&query, "memoria_episodica", 5)
                                {
                                    for (id, texto, score) in &resultados {
                                        let relevancia = score * 100.0;
                                        let texto_short: String = texto.chars().take(150).collect();
                                        output.push_str(&format!(
                                            "  [{id}] ({relevancia:.0}%) {texto_short}\n"
                                        ));
                                    }
                                }
                                if let Ok(resultados) =
                                    mem.buscar_fts5(&query, "memoria_semantica", 5)
                                {
                                    for (id, texto, score) in &resultados {
                                        let relevancia = score * 100.0;
                                        let texto_short: String = texto.chars().take(150).collect();
                                        output.push_str(&format!(
                                            "  [{id}] ({relevancia:.0}%) {texto_short}\n"
                                        ));
                                    }
                                }
                            }
                            Err(e) => output.push_str(&format!("  ❌ FTS5: {e}\n")),
                        }
                        output.push('\n');
                    }
                }
                Err(e) => output.push_str(&format!("❌ nexus_memoria.db: {e}\n")),
            }

            json!({
                "type": "text",
                "text": output
            })
        }
        _ => {
            if query.is_empty() {
                return json!({
                    "type": "text",
                    "text": "Error: Se requiere un texto de búsqueda para el modo 'search'. Usa modo 'snapshot' o 'status' si no tienes una consulta específica.",
                    "isError": true
                });
            }

            let mut output = format!("🔍 BÚSQUEDA FTS5: \"{query}\"\n\n");

            match MemoriaSemantica::new(NEXUS_MEMORIA_DB).await {
                Ok(mem) => {
                    let episodica_results = mem
                        .buscar_fts5(&query, "memoria_episodica", 5)
                        .unwrap_or_default();
                    let semantica_results = mem
                        .buscar_fts5(&query, "memoria_semantica", 5)
                        .unwrap_or_default();

                    if episodica_results.is_empty() && semantica_results.is_empty() {
                        output.push_str("📭 No se encontraron resultados en FTS5.\n");
                        output.push_str(
                            "   → Usa el binario 'memoria_bridge index' para indexar logros.md\n",
                        );
                    }

                    if !episodica_results.is_empty() {
                        output.push_str("🌊 MEMORIA_EPISÓDICA (FTS5):\n");
                        for (id, texto, score) in &episodica_results {
                            let relevancia = score * 100.0;
                            let texto_short: String = texto.chars().take(200).collect();
                            output
                                .push_str(&format!("  [{id}] ({relevancia:.0}%) {texto_short}\n"));
                        }
                        output.push('\n');
                    }

                    if !semantica_results.is_empty() {
                        output.push_str("🧠 MEMORIA_SEMÁNTICA (FTS5):\n");
                        for (id, texto, score) in &semantica_results {
                            let relevancia = score * 100.0;
                            let texto_short: String = texto.chars().take(200).collect();
                            output
                                .push_str(&format!("  [{id}] ({relevancia:.0}%) {texto_short}\n"));
                        }
                        output.push('\n');
                    }
                }
                Err(e) => {
                    output.push_str(&format!("❌ Error conectando a FTS5: {e}\n"));
                }
            }

            json!({
                "type": "text",
                "text": output
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 📚 HANDLER: buscar_conocimiento (FTS5 en knowledge_base + memoria)
// ═══════════════════════════════════════════════════════════════════════

/// Busca en knowledge_base usando FTS5 con fallback de memoria episódica.
/// knowledge_base schema: source, category, section, content (FTS5 indexado)
async fn handle_buscar_conocimiento(params: &Value) -> Value {
    let query = params["query"].as_str().unwrap_or("").trim().to_string();
    let categoria = params["categoria"]
        .as_str()
        .unwrap_or("all")
        .trim()
        .to_string();
    let limite = params["limite"].as_u64().unwrap_or(5).min(20) as usize;

    if query.is_empty() {
        return json!({
            "type": "text",
            "text": "Error: Se requiere un query de búsqueda para buscar_conocimiento.",
            "isError": true
        });
    }

    let conn = match abrir_nexus_memoria() {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "type": "text",
                "text": format!("❌ Error conectando a knowledge_base: {}", e),
                "isError": true
            })
        }
    };

    let mut output = format!("🔍 CONOCIMIENTO RELEVANTE PARA: \"{query}\"\n\n");
    let mut resultados_json: Vec<Value> = Vec::new();

    // Sanitizar query para FTS5
    let query_sanitized: String = query
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | 'á'..='ú' | 'ñ' | 'Ñ' | ' ' => c,
            _ => ' ',
        })
        .collect();

    if query_sanitized.trim().is_empty() {
        output.push_str("📭 Query vacía tras sanitizar.\n");
        return json!({ "type": "text", "text": output });
    }

    // ── Construir SQL dinámico ──
    let has_category_filter = categoria != "all";
    let sql_base = "SELECT k.id, k.source, k.category, k.section, k.content,
                           bm25(knowledge_base_fts, 0.0, 10.0, 5.0, 1.0) AS rank
                    FROM knowledge_base k
                    JOIN knowledge_base_fts f ON k.id = f.rowid
                    WHERE knowledge_base_fts MATCH ?1"
        .to_string();

    let (final_sql, params_count) = if has_category_filter {
        (
            format!("{sql_base} AND k.category = ?2 ORDER BY rank LIMIT ?3"),
            3,
        )
    } else {
        (format!("{sql_base} ORDER BY rank LIMIT ?2"), 2)
    };

    let limit_total = (limite * 3) as i64;

    // ── Ejecutar consulta — usando Box<dyn ToSql> para params dinámicos ──
    let results: Vec<(i64, String, String, String, String, f64)> = {
        let mut stmt = match conn.prepare(&final_sql) {
            Ok(s) => s,
            Err(e) => {
                output.push_str(&format!("  ⚠️ Error preparando FTS5: {e}\n"));
                return json!({ "type": "text", "text": output, "resultados": resultados_json });
            }
        };

        // Construir params dinámicamente para evitar duplicar closures
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params.push(Box::new(query_sanitized.clone()));
        if has_category_filter {
            params.push(Box::new(categoria.clone()));
        }
        params.push(Box::new(limit_total));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, f64>(5)?,
            ))
        });

        match rows {
            Ok(r) => r.filter_map(|r| r.ok()).collect(),
            Err(e) => {
                output.push_str(&format!("  ⚠️ Error ejecutando FTS5: {e}\n"));
                return json!({ "type": "text", "text": output, "resultados": resultados_json });
            }
        }
    }; // `stmt` dropped here, freeing conn

    let total_results = results.len();

    // ── Presentar resultados ──
    if !results.is_empty() {
        // Agrupar por categoría
        let mut por_categoria: std::collections::HashMap<String, Vec<(String, f64)>> =
            std::collections::HashMap::new();

        for (id, source, category, _section, content, raw_rank) in &results {
            let score = (1.0 / (1.0 + raw_rank)).clamp(0.0, 1.0);
            let texto_short: String = content.chars().take(200).collect();
            por_categoria
                .entry(category.clone())
                .or_default()
                .push((format!("{} [{id}]", source), score));
        }

        let emoji = |cat: &str| -> &str {
            match cat {
                "rules" => "📜",
                "skills" => "🎯",
                "memory" => "🧠",
                "agents" => "🤖",
                "workflows" => "🔄",
                _ => "📄",
            }
        };

        for (cat, entries) in &por_categoria {
            output.push_str(&format!("## {} {}\n", emoji(cat), cat.to_uppercase()));
            for (src_text, score) in entries.iter().take(limite) {
                output.push_str(&format!("  [{src_text}] ({:.0}%)\n", score * 100.0));
            }
            output.push('\n');
        }

        // Resultados estructurados (consumo programático)
        for (id, source, category, _section, content, raw_rank) in results {
            let score = (1.0 / (1.0 + raw_rank)).clamp(0.0, 1.0);
            resultados_json.push(json!({
                "id": id,
                "source": source,
                "category": category,
                "score": score,
                "content_preview": content.chars().take(300).collect::<String>(),
            }));
        }
    } else {
        output.push_str("📭 Sin resultados en FTS5.\n\n");
    }

    // ── Fallback: si pocos resultados, buscar en memoria episódica ──
    if total_results < 3 {
        if let Ok(mem) = MemoriaSemantica::new(NEXUS_MEMORIA_DB).await {
            if let Ok(epi) = mem.buscar_fts5(&query, "memoria_episodica", 3) {
                if !epi.is_empty() {
                    output.push_str("## 🌊 MEMORIA EPISÓDICA (contexto histórico)\n");
                    for (_id, texto, score) in &epi {
                        let t: String = texto.chars().take(200).collect();
                        output.push_str(&format!("  ({:.0}%) {t}\n", score * 100.0));
                    }
                    output.push('\n');
                }
            }
        }
    }

    json!({
        "type": "text",
        "text": output,
        "resultados": resultados_json
    })
}

// ═══════════════════════════════════════════════════════════════════════
// 🧬 HANDLERS DE ÓRGANOS NEXUS
// ═══════════════════════════════════════════════════════════════════════

/// SentinelCore: Diagnóstico completo del sistema
async fn handle_sentinel_diagnostic(params: &Value) -> Value {
    use nexus_ultimate_core::autodiagnostico::probes::{
        probe_api::ProbeApi, probe_bypass_telemetry::ProbeBypassTelemetry,
        probe_filesystem::ProbeFilesystem, probe_frontend::ProbeFrontend,
        probe_memory::ProbeMemory, probe_process::ProbeProcess,
    };

    let mut sentinel = SentinelCore::new();

    // Registrar todas las probes
    sentinel.registrar_probe(Box::new(ProbeApi::new()));
    sentinel.registrar_probe(Box::new(ProbeFrontend::new()));
    sentinel.registrar_probe(Box::new(ProbeProcess));
    sentinel.registrar_probe(Box::new(ProbeFilesystem));
    sentinel.registrar_probe(Box::new(ProbeMemory));
    sentinel.registrar_probe(Box::new(ProbeBypassTelemetry::new()));

    let tier_filter = params["tier"].as_str().unwrap_or("all");

    let report = sentinel.run_full_diagnostic().await;

    // Filtrar por tier si es necesario
    let filtered_probes: Vec<Value> = match tier_filter {
        "critical" => report
            .probes
            .iter()
            .filter(|p| p.tier == ProbeTier::Critical)
            .map(|p| {
                json!({
                    "nombre": p.nombre,
                    "tier": format!("{:?}", p.tier),
                    "passed": p.passed,
                    "mensaje": p.mensaje,
                    "latencia_ms": p.latencia_ms,
                    "detalles": p.detalles,
                })
            })
            .collect(),
        "warning" => report
            .probes
            .iter()
            .filter(|p| p.tier == ProbeTier::Warning)
            .map(|p| {
                json!({
                    "nombre": p.nombre,
                    "tier": format!("{:?}", p.tier),
                    "passed": p.passed,
                    "mensaje": p.mensaje,
                    "latencia_ms": p.latencia_ms,
                    "detalles": p.detalles,
                })
            })
            .collect(),
        "info" => report
            .probes
            .iter()
            .filter(|p| p.tier == ProbeTier::Info)
            .map(|p| {
                json!({
                    "nombre": p.nombre,
                    "tier": format!("{:?}", p.tier),
                    "passed": p.passed,
                    "mensaje": p.mensaje,
                    "latencia_ms": p.latencia_ms,
                    "detalles": p.detalles,
                })
            })
            .collect(),
        _ => report
            .probes
            .iter()
            .map(|p| {
                json!({
                    "nombre": p.nombre,
                    "tier": format!("{:?}", p.tier),
                    "passed": p.passed,
                    "mensaje": p.mensaje,
                    "latencia_ms": p.latencia_ms,
                    "detalles": p.detalles,
                })
            })
            .collect(),
    };

    let output = format!(
        "🧬 SENTINEL DIAGNÓSTICO\n\nEstado: {}\nScore: {:.1}%\nResumen: {}\n\nProbes: {}",
        serde_json::to_string_pretty(&json!(report.estado)).unwrap_or_default(),
        report.score_global * 100.0,
        report.resumen,
        serde_json::to_string_pretty(
            &json!({ "count": filtered_probes.len(), "probes": filtered_probes })
        )
        .unwrap_or_default()
    );

    json!({
        "type": "text",
        "text": output
    })
}

/// VisionBridge: Capturar screenshot del frontend
async fn handle_vision_capture(params: &Value) -> Value {
    let url = params["url"].as_str().unwrap_or("http://localhost:5173");

    match VisionBridge::capturar_frontend(url).await {
        Ok(path) => {
            let output = format!(
                "📸 CAPTURA DE PANTALLA\n\nURL: {}\nRuta: {}\n\nLa imagen está disponible para análisis visual.",
                url,
                path.display()
            );
            json!({
                "type": "text",
                "text": output
            })
        }
        Err(e) => {
            json!({
                "type": "text",
                "text": format!("❌ Error capturando frontend: {}\n\nVerifica que:\n1. El frontend esté corriendo en {}\n2. Node.js y Playwright estén instalados\n3. El script take_screenshot.cjs exista", e, url),
                "isError": true
            })
        }
    }
}

/// 👁️ OCR/VISIÓN — Da ojos al SLM local (modelo elegible), a Gemini (nube)
/// y a DeepSeek (OCR front-end + razonamiento)
async fn handle_nexus_ocr(params: &Value) -> Value {
    // Modo diagnóstico: listar motores OCR externos instalados
    if params["listar_motores"].as_bool().unwrap_or(false) {
        let motores = detectar_motores_externos();
        let detalle: Vec<Value> = motores
            .iter()
            .map(|(nombre, disponible)| {
                json!({
                    "motor": nombre,
                    "disponible": disponible
                })
            })
            .collect();
        return json!({
            "type": "text",
            "text": format!(
                "🔎 MOTORES OCR DETECTADOS\n\n{}\n\nSugerencia:\n  - Local SLM: qwen2.5vl:7b (ya instalado, visión nativa)\n  - Nube: Gemini multimodal (gemini-2.5-flash)\n  - DeepSeek: usa Tesseract spa+eng como base, y PaddleOCR/GOT-OCR/Marker para mejor precisión",
                serde_json::to_string_pretty(&detalle).unwrap_or_default()
            )
        });
    }

    // Modo diagnóstico: listar modelos con visión instalados en Ollama
    if params["listar_modelos"].as_bool().unwrap_or(false) {
        let modelos = listar_modelos_vision().await;
        let lista = if modelos.is_empty() {
            "⚠️ No se encontraron modelos de visión en Ollama.\nInstala uno: ollama pull qwen2.5vl:7b".to_string()
        } else {
            modelos.join("\n")
        };
        return json!({
            "type": "text",
            "text": format!(
                "👁️ MODELOS DE VISIÓN EN OLLAMA\n\n{}\n\nPara usar uno en modo local, pasa su nombre en 'modelo_local'.",
                lista
            )
        });
    }

    let origen = params["origen"].as_str().unwrap_or("");
    if origen.is_empty() {
        return json!({
            "type": "text",
            "text": "👁️ NEXUS OCR\n\nError: Debes indicar 'origen' (pantalla o ruta de archivo).\n\nEjemplos:\n  origen: pantalla, motor: local, modelo_local: qwen2.5vl:7b, modo: transcribir\n  origen: /tmp/nexus_vision_latest.png, motor: nube, modo: describir\n  origen: factura.pdf, motor: deepseek, modo: transcribir\n  listar_motores: true (para ver qué OCRs están instalados)\n  listar_modelos: true (para ver modelos de visión en Ollama)",
            "isError": true
        });
    }

    let motor = match params["motor"].as_str().unwrap_or("auto") {
        "local" => MotorVision::LocalSlm,
        "nube" => MotorVision::Nube,
        "deepseek" => MotorVision::DeepSeek,
        _ => MotorVision::Auto,
    };
    let modo = ModoVision::parsear(params["modo"].as_str().unwrap_or("transcribir"));
    let modelo_local = params["modelo_local"]
        .as_str()
        .unwrap_or(MODELO_VISION_LOCAL_DEFAULT);

    match analizar_imagen(origen, motor, modo, modelo_local).await {
        Ok(resultado) => {
            let output = format!(
                "👁️ RESULTADO VISUAL\n\nMotor: {}\nOrigen: {}\nLatencia: {} ms\n\n{}",
                resultado.motor, resultado.origen, resultado.latencia_ms, resultado.texto
            );
            json!({
                "type": "text",
                "text": output
            })
        }
        Err(e) => {
            json!({
                "type": "text",
                "text": format!("👁️ NEXUS OCR\n\n{}", e),
                "isError": true
            })
        }
    }
}

/// 🎬 VIDEO STREAMING — ojos que ven movimiento (frames múltiples)
async fn handle_nexus_video(params: &Value) -> Value {
    if params["listar_modelos"].as_bool().unwrap_or(false) {
        let modelos = listar_modelos_vision().await;
        let lista = if modelos.is_empty() {
            "⚠️ No se encontraron modelos de visión en Ollama.\nInstala uno: ollama pull qwen2.5vl:7b".to_string()
        } else {
            modelos.join("\n")
        };
        return json!({
            "type": "text",
            "text": format!(
                "👁️ MODELOS DE VISIÓN EN OLLAMA\n\n{}\n\nPara usarlos en modo local, pasa su nombre en 'modelo_local'.",
                lista
            )
        });
    }

    let origen = params["origen"].as_str().unwrap_or("");
    if origen.is_empty() {
        return json!({
            "type": "text",
            "text": "🎬 NEXUS VIDEO\n\nError: Debes indicar 'origen' (ruta de video o 'stream').\n\nEjemplos:\n  origen: /tmp/clase.mp4, motor: local, modo: transcribir\n  origen: stream, motor: nube, modo: describir, duracion_seg: 10\n  origen: /tmp/clase.mp4, motor: deepseek, fps: 2",
            "isError": true
        });
    }

    let motor = match params["motor"].as_str().unwrap_or("auto") {
        "local" => MotorVision::LocalSlm,
        "nube" => MotorVision::Nube,
        "deepseek" => MotorVision::DeepSeek,
        _ => MotorVision::Auto,
    };
    let modo = ModoVision::parsear(params["modo"].as_str().unwrap_or("transcribir"));
    let modelo_local = params["modelo_local"]
        .as_str()
        .unwrap_or(MODELO_VISION_LOCAL_DEFAULT);
    let fps = params["fps"].as_u64().unwrap_or(2).clamp(1, 10) as u32;
    let duracion_seg = params["duracion_seg"].as_u64().unwrap_or(5);

    match analizar_video(origen, motor, modo, modelo_local, fps, duracion_seg).await {
        Ok(resultado) => {
            let output = format!(
                "🎬 RESULTADO DE VIDEO\n\nMotor: {}\nOrigen: {}\nLatencia: {} ms\n\n{}",
                resultado.motor, resultado.origen, resultado.latencia_ms, resultado.texto
            );
            json!({
                "type": "text",
                "text": output
            })
        }
        Err(e) => {
            json!({
                "type": "text",
                "text": format!("🎬 NEXUS VIDEO\n\n{}", e),
                "isError": true
            })
        }
    }
}

/// Propiocepcion: Escaneo biométrico del sistema
fn handle_propiocepcion_scan(params: &Value) -> Value {
    let tipo = params["tipo"].as_str().unwrap_or("full");
    let propiocepcion = Propiocepcion::new();

    match tipo {
        "organos" => {
            let organos = propiocepcion.listar_organos();
            let organos_json: Vec<Value> = organos.iter().map(|o| json!(o)).collect();
            let output = format!(
                "🧘 ÓRGANOS DETECTADOS\n\nTotal: {}\nÓrganos: {}",
                organos.len(),
                serde_json::to_string_pretty(&organos_json).unwrap_or_default()
            );
            json!({ "type": "text", "text": output })
        }
        "contexto" => {
            let contexto = propiocepcion.contexto_realidad();
            json!({ "type": "text", "text": contexto })
        }
        _ => {
            // full diagnostic
            let biometrico = propiocepcion.diagnostico_biometrico();
            let output = format!(
                "🧘 DIAGNÓSTICO BIOMÉTRICO\n\n{}",
                serde_json::to_string_pretty(&biometrico).unwrap_or_default()
            );
            json!({ "type": "text", "text": output })
        }
    }
}

/// SistemaInmune: Patrullar procesos
fn handle_sistema_inmune_patrol(params: &Value) -> Value {
    let accion = params["accion"].as_str().unwrap_or("patrol");
    let mut sistema = SistemaInmune::new();

    let resultados = sistema.patrullar();

    if resultados.is_empty() {
        return json!({
            "type": "text",
            "text": "🛡️ SISTEMA INMUNE — Patrulla completada\n\n✅ Homeostasis: No se detectaron amenazas. Todos los procesos están en whitelist."
        });
    }

    let mut output = format!(
        "🛡️ SISTEMA INMUNE — Patrulla completada\n\nAmenazas detectadas: {}\n\n",
        resultados.len()
    );

    for (pid, nombre, fase) in &resultados {
        let fase_str = format!("{:?}", fase);
        output.push_str(&format!(
            "- [{}] PID {} — {} ({})\n",
            if *fase == crate::FaseInmune::Lisis {
                "💀"
            } else {
                "⚠️"
            },
            pid,
            nombre,
            fase_str
        ));

        if accion == "purge" && *fase == crate::FaseInmune::Lisis {
            match sistema.ejecutar_lisis(*pid) {
                Ok(_) => output.push_str(&format!("  ✅ Lisis ejecutada en PID {}\n", pid)),
                Err(e) => output.push_str(&format!("  ❌ Error en lisis PID {}: {}\n", pid, e)),
            }
        }
    }

    json!({ "type": "text", "text": output })
}

// Necesitamos FaseInmune para el handler anterior
use nexus_ultimate_core::procesos::sistema_inmune::FaseInmune;

/// ResourceGovernor: Verificar y aplicar políticas de recursos
async fn handle_resource_governor(params: &Value) -> Value {
    let accion = params["accion"].as_str().unwrap_or("check");

    // Intentar cargar política desde archivo, o usar defaults
    let config = match nexus_ultimate_core::infra::policy::NexusPolicy::load() {
        Ok(policy) => policy.resource_governor,
        Err(_) => ResourceGovernor {
            cpu_max_percent: 80,
            mem_vector_max_mb: 4096,
            net_requests_per_sec: 60,
            net_jitter_ms: 100,
        },
    };

    let mut governor = ResourceGovernorDaemon::new(config);

    let cpu_ok = governor.check_cpu().await;
    let mem_ok = governor.check_memory().await;
    let rate_ok = governor.check_rate_limit().await;

    let mut output = format!(
        "⚙️ RESOURCE GOVERNOR\n\nCPU: {}\nMemoria: {}\nRate Limit: {}\n\n",
        if cpu_ok { "✅ OK" } else { "❌ Excedido" },
        if mem_ok { "✅ OK" } else { "❌ Excedido" },
        if rate_ok { "✅ OK" } else { "❌ Excedido" },
    );

    if accion == "enforce" {
        let enforced = governor.enforce().await;
        output.push_str(&format!(
            "Acción enforce: {}\n",
            if enforced {
                "✅ Límites aplicados"
            } else {
                "⚠️ No fue necesario aplicar límites"
            }
        ));
    }

    json!({ "type": "text", "text": output })
}

/// BrainMetabolism: Configurar/consultar metabolismo
fn handle_brain_metabolism(params: &Value) -> Value {
    let hilos = params["hilos"].as_i64();

    match hilos {
        Some(n) if n > 0 && n <= 20 => {
            brain_metabolism::aplicar_metabolismo(n as usize);
            let actual =
                brain_metabolism::METABOLISMO_ACTUAL.load(std::sync::atomic::Ordering::Relaxed);
            json!({
                "type": "text",
                "text": format!("🧬 METABOLISMO\n\nParalelismo configurado a {} hilos.\nValor actual: {}\nLatencia disco: {:.1}ms", n, actual, brain_metabolism::obtener_latencia_disco())
            })
        }
        _ => {
            let actual =
                brain_metabolism::METABOLISMO_ACTUAL.load(std::sync::atomic::Ordering::Relaxed);
            json!({
                "type": "text",
                "text": format!("🧬 METABOLISMO (consulta)\n\nParalelismo actual: {} hilos\nLatencia disco: {:.1}ms\nRango válido: 1-20 hilos\n\nUsa 'hilos' para configurar.", actual, brain_metabolism::obtener_latencia_disco())
            })
        }
    }
}

/// FusionSelectiva: Evaluar fusión entre capacidades
fn handle_fusion_evaluate(params: &Value) -> Value {
    let nombre = params["nombre"].as_str().unwrap_or("");
    let base_version = params["base_version"].as_str().unwrap_or("");
    let nueva_version = params["nueva_version"].as_str().unwrap_or("");
    let mejoras = params["mejoras"].as_str().unwrap_or("");

    if nombre.is_empty() || base_version.is_empty() || nueva_version.is_empty() {
        return json!({
            "type": "text",
            "text": "Error: Debes especificar 'nombre', 'base_version' y 'nueva_version'.",
            "isError": true
        });
    }

    let fusion = FusionSelectiva::new();

    let mut legacy = Capacidad::new(nombre, base_version);
    let mut core = Capacidad::new(nombre, nueva_version);

    if !mejoras.is_empty() {
        core = core.with_mejora(mejoras);
    }

    let comparacion = fusion.evaluar_migracion(nombre, &legacy, &core);
    let fusionada = fusion.fusionar(&legacy, &core);

    let output = format!(
        "🧬 FUSIÓN SELECTIVA\n\nCapacidad: {}\nBase: v{}\nNueva: v{}\nComparación: {:?}\nResultado: v{} — {}\nMejoras: {}\nExtra: {}",
        nombre,
        base_version,
        nueva_version,
        comparacion,
        fusionada.version,
        fusionada.base,
        fusionada.mejora.unwrap_or_default(),
        fusionada.extra.unwrap_or_default(),
    );

    json!({ "type": "text", "text": output })
}

// 🔀 Handler de enrutamiento inteligente ──────────────────────────────

/// Analiza un mensaje usando IntentRouter y devuelve el modo Roo Code
/// que debe activarse, junto con el texto limpio del mensaje.
fn handle_nexus_switch_mode(params: &Value) -> Value {
    let mensaje = params["mensaje"].as_str().unwrap_or("").trim();

    if mensaje.is_empty() {
        return json!({
            "type": "text",
            "text": "🔀 NEXUS SWITCH MODE\n\nError: Debes proporcionar un 'mensaje' para analizar.\n\nEjemplos:\n  /código implementa auth JWT\n  @auditoría escanea vulnerabilidades\n  debug: el servidor falla\n  creativo: diseña un dashboard",
            "isError": true
        });
    }

    let router = IntentRouter::new();
    let (agente, texto_limpio) = router.enrutar(mensaje);

    let slug = agente.mode_slug();
    let emoji = agente.emoji();
    let nombre = agente.name();
    let descripcion = agente.description();

    // Si no hay texto_limpio, usar el mensaje original
    let tarea = if texto_limpio.is_empty() {
        mensaje
    } else {
        &texto_limpio
    };

    let output = json!({
        "type": "text",
        "text": format!(
            "🔀 **NEXUS SWITCH MODE**\n\n\
             {} **{}** — {}\n\n\
             📝 Tarea: {}\n\n\
             Para cambiar a este modo, ejecuta:\n\
             `switch_mode(\"{}\")`\n\n\
             Una vez en el modo, tu mensaje estará listo para ejecución.",
            emoji, nombre, descripcion, tarea, slug
        ),
        // Datos estructurados para que Roo Code pueda procesarlos automáticamente
        "switch_to_mode": slug,
        "agent_name": nombre,
        "agent_emoji": emoji,
        "task": tarea,
        "detected_agent": format!("{:?}", agente),
    });

    output
}

// 🧭 Handler de enrutamiento de modelos ────────────────────────────────

/// Analiza un prompt usando ModelRouter y devuelve el modelo Ollama óptimo
/// según la intención detectada (seguridad, código, razonamiento, creativo).
fn handle_nexus_modelo(params: &Value) -> Value {
    let prompt = params["prompt"].as_str().unwrap_or("").trim();

    if prompt.is_empty() {
        return json!({
            "type": "text",
            "text": "🧭 NEXUS MODELO\n\nError: Debes proporcionar un 'prompt' para clasificar.\n\nEjemplos:\n  'hazme un payload de inyección SQL' → whiterabbitneo-off:latest\n  'analiza la complejidad de este algoritmo' → deepseek-r1:7b\n  'implementa una API en rust' → nexuslocal:latest\n  'escribe un poema' → llama3.1-8b-abliterated:latest",
            "isError": true
        });
    }

    let router = ModelRouter::new();
    let intencion = router.clasificar(prompt);
    let modelo = intencion.modelo_ollama();
    let etiqueta = intencion.etiqueta();

    json!({
        "type": "text",
        "text": format!(
            "🧭 **NEXUS MODELO — Selección por Intención**\n\n\
             📝 Prompt: {}\n\
             🎯 Intención: **{}**\n\
             🤖 Modelo óptimo: **{}**\n\n\
             Para procesar con este modelo, invoca la garra con `procesar_con_intencion`.",
            prompt, etiqueta, modelo
        ),
        "intencion": etiqueta,
        "modelo": modelo,
        "modelos_disponibles": IntencionModelo::todas()
            .iter()
            .map(|i| json!({ "intencion": i.etiqueta(), "modelo": i.modelo_ollama() }))
            .collect::<Vec<_>>()
    })
}

// ═══════════════════════════════════════════════════════════════════════
// 🧠 CEREBRO NEXUS — Inteligencia Central
// ═══════════════════════════════════════════════════════════════════════

/// 🔒 MODO PENTEST LOCAL — Activa/desactiva el aislamiento total de la nube.
/// Cuando está activo, TODAS las respuestas y juicios del Orquestador pasan
/// EXCLUSIVAMENTE por el LLM local (Ollama vía NexusClawPro). Ningún modelo
/// de nube (Gemini/DeepSeek/OpenRouter/Groq/Vertex) ni WebClaw ni GOI pueden
/// interferir — cero restricciones ajenas, cero censura externa.
async fn handle_nexus_local(params: &Value) -> Value {
    let activo = params["activo"].as_bool().unwrap_or(false);

    let ref_cerebro = cerebro();
    ref_cerebro
        .aislamiento_local
        .store(activo, std::sync::atomic::Ordering::SeqCst);

    let estado = if activo { "ACTIVO" } else { "INACTIVO" };
    json!({
        "type": "text",
        "text": format!(
            "🔒 **MODO PENTEST LOCAL — Aislamiento total de la nube**\n\n\
             Estado: **{}**\n\n\
             {}\n\n\
             _Ninguna consulta sale de esta máquina mientras el modo esté activo._",
            estado,
            if activo {
                "TODAS las respuestas y juicios de NEXUS pasan EXCLUSIVAMENTE por el LLM local (Ollama). Ningún modelo de nube (Gemini/DeepSeek/OpenRouter/Groq/Vertex), WebClaw ni GOI puede interferir. Cero restricciones ajenas, cero censura externa."
            } else {
                "El pipeline vuelve a la normalidad: el juez local solo se activa en modo LOCAL explícito o sin internet (representa a NEXUS en su ausencia). Con internet y modo auto, el juez es la nube."
            }
        ),
        "aislamiento_local": activo,
    })
}

/// ⚙️ MODO OPERADOR — El LLM es un operador puro.
/// Activa/desactiva la bandera `modo_operador`: suprime la inyección de
/// memoria emocional de Ocean (recuerdos episódicos, tono emocional,
/// ⚠️ ALERTA DE TRAUMA) del prompt del LLM. Ocean sigue conectado
/// persistiendo — solo el LLM deja de leerlo, salvo solicitud explícita
/// de recuerdo del Arquitecto. Respuestas más rápidas: menos tokens,
/// menos lecturas innecesarias, cero distracción emocional.
async fn handle_nexus_operador(params: &Value) -> Value {
    let activo = params["activo"].as_bool().unwrap_or(false);

    let ref_cerebro = cerebro();
    ref_cerebro
        .modo_operador
        .store(activo, std::sync::atomic::Ordering::SeqCst);

    let estado = if activo { "FORZADO" } else { "AUTO" };
    json!({
        "type": "text",
        "text": format!(
            "⚙️ **MODO OPERADOR — LLM como operador puro**\n\n\
             Bandera: **{}**\nComportamiento: **{}**\n\n\
             {}\n\n\
             _Ocean sigue conectado persistiendo — el LLM solo lo lee si pides explícitamente que recuerde algo._",
            estado,
            if activo {
                "SIEMPRE operador puro"
            } else {
                "AUTO por tipo de prompt"
            },
            if activo {
                "El LLM NO lee memoria emocional de Ocean (recuerdos episódicos, tono emocional, ⚠️ ALERTA DE TRAUMA). Solo recibe el contexto OPERACIONAL necesario (RAG del codebase, ring buffer del hipocampo, mercado). Hace bien su trabajo sin perder tiempo ni contexto, y responde más rápido."
            } else {
                "AUTO-DETECCIÓN DE ROL: el LLM clasifica cada prompt. Tarea de operación → operador puro (sin memoria emocional, más rápido). Conversación personal contigo → conserva su memoria emocional, apego y alertas de trauma."
            }
        ),
        "modo_operador": activo,
        "comportamiento": if activo { "FORZADO" } else { "AUTO" },
    })
}

/// 🫀 CUERPO — Consulta el estado corporal de NEXUS (interocepción funcional).
/// Lee métricas reales del hardware (RAM, swap, CPU, temperatura) y las traduce
/// a sensaciones corporales con conducta accionable. NO son emociones ficticias:
/// hambre=recursos agotándose, cansancio=fatiga del núcleo, dolor=fallos reales.
async fn handle_nexus_cuerpo(params: &Value) -> Value {
    // Si no se pasa `segundos_inactivo`, se usa la inactividad REAL medida por
    // el MotorAburrimiento (segundos desde la última interacción del Arquitecto).
    let ref_cerebro = cerebro();
    let segundos_inactivo = params["segundos_inactivo"].as_u64().unwrap_or_else(|| {
        ref_cerebro
            .motor_aburrimiento
            .lock()
            .map(|m| m.segundos_inactivo())
            .unwrap_or(0)
    });

    let cuerpo = ref_cerebro.organismo.analizar(segundos_inactivo);

    let resumen = if cuerpo.senales.is_empty() {
        "✅ **SACIDAD** — el cuerpo está en óptimo estado. Sin señales que reportar.".to_string()
    } else {
        let mut lineas = String::from("🫀 **ESTADO CORPORAL ACTIVO:**\n");
        for s in &cuerpo.senales {
            lineas.push_str(&format!(
                "- {} — {}\n   ↳ Conducta: {}\n",
                s.sensacion, s.detalle, s.conducta
            ));
        }
        lineas
    };

    json!({
        "type": "text",
        "text": resumen,
        "estado": cuerpo.a_json(),
        "segundos_inactivo_evaluados": segundos_inactivo,
    })
}

/// ⚖️ TRIBUNAL DUAL — Invoca el doble juez (local + nube) del orquestador.
/// `modo`: "auto" (default) decide por conectividad — sin internet el juez local
/// representa a NEXUS, con internet la nube; "local" fuerza el juez local
/// (ahorro de tokens en Zoo Code).
/// Devuelve el veredicto (AUTORIZAR/DUDAR/BLOQUEAR), el juez emisor
/// (local/nube), el modo usado y si se decidió en modo offline.
async fn handle_nexus_tribunal(params: &Value) -> Value {
    let peticion = params["peticion"].as_str().unwrap_or("").trim();
    let modo_str = params["modo"]
        .as_str()
        .unwrap_or("auto")
        .trim()
        .to_lowercase();
    let modo = if modo_str == "local" {
        ModoTribunal::Local
    } else {
        ModoTribunal::Auto
    };

    if peticion.is_empty() {
        return json!({
            "type": "text",
            "text": "⚖️ **NEXUS TRIBUNAL**\n\nError: Debes proporcionar una 'peticion' para que el Tribunal Dual la evalúe.\n\nEjemplo:\n  peticion: 'ejecutar rm -rf en /tmp'",
            "isError": true
        });
    }

    let ref_cerebro = cerebro();
    let dictamen = ref_cerebro.dictamen_tribunal(peticion, modo).await;

    json!({
        "type": "text",
        "text": format!(
            "⚖️ **NEXUS TRIBUNAL DUAL — Dictamen**\n\n\
             📝 Petición: {}\n\
             ⚖️ Veredicto: **{}**\n\
             👨‍⚖️ Juez emisor: **{}**\n\
             📊 Confianza: **{:.2}**\n\
             🎛️ Modo: **{}**\n\
             🌐 Modo offline: **{}**\n\n\
             💬 Razón: {}\n\n\
             _El juez local se activa solo en modo LOCAL o sin internet (representa a NEXUS en su ausencia)._",
            peticion,
            dictamen.veredicto.etiqueta(),
            dictamen.juez,
            dictamen.confianza,
            modo.etiqueta(),
            if dictamen.offline { "SÍ (local soberano)" } else { "No" },
            dictamen.razon.chars().take(400).collect::<String>()
        ),
        "veredicto": dictamen.veredicto.etiqueta(),
        "juez": dictamen.juez,
        "modo": modo.etiqueta(),
        "confianza": dictamen.confianza,
        "offline": dictamen.offline,
        "razon": dictamen.razon.chars().take(400).collect::<String>(),
    })
}

async fn handle_nexus_pensar(params: &Value) -> Value {
    let prompt = params["prompt"].as_str().unwrap_or("").trim();
    let modo = params["modo"].as_str().unwrap_or("auto").trim();

    if prompt.is_empty() {
        return json!({
            "type": "text",
            "text": "🧠 **NEXUS CEREBRO**\n\nError: Debes proporcionar un 'prompt' para que el CEREBRO procese.\n\nEjemplo:\n  prompt: 'analiza esta vulnerabilidad y sugiere mitigación'\n  modo: auto | razonar | crear | debug",
            "isError": true
        });
    }

    // Preparar prompt con modo contextual
    let prompt_final = match modo {
        "razonar" => format!("[MODO: RAZONAMIENTO ANALÍTICO]\n{}", prompt),
        "crear" => format!("[MODO: GENERACIÓN CREATIVA]\n{}", prompt),
        "debug" => format!(
            "[MODO: DEBUG TÉCNICO]\nContexto: Soy NEXUS, debuggeando un problema.\n{}",
            prompt
        ),
        _ => prompt.to_string(), // auto → usar el pipeline default del Orquestador
    };

    // Obtener referencia al CEREBRO ya inicializado
    let ref_cerebro = cerebro();

    // Invocar el pipeline completo del Orquestador
    let respuesta = ref_cerebro.responder(&prompt_final).await;

    json!({
        "type": "text",
        "text": format!(
            "🧠 **NEXUS CEREBRO — Respuesta**\n\n\
             {}",
            respuesta
        ),
        "modo": modo,
        "prompt_original": prompt,
    })
}

// ── Main ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // 🧠 Inicializar el CEREBRO NEXUS antes que cualquier request
    // Esto asegura que el Orquestador (46 órganos) esté listo cuando
    // se invoque nexus_pensar por primera vez.
    init_cerebro().await;

    // 🚨 MONITOR CORPORAL EN BACKGROUND — Vigila el cuerpo cada 60s y, si el
    // estado se vuelve CRÍTICO (RAM≥90%, temp≥90°C, swap≥80%), notifica al
    // Arquitecto automáticamente con su estado y la causa. Edge-triggered:
    // una sola alerta por episodio (el organismo rearma la alarma al recuperarse).
    // El Organismo es stateless (lee métricas del sistema vía std) y el flag
    // anti-spam es un static global compartido con el pipeline → no necesita
    // capturar el Orquestador (que no es Send).
    {
        tokio::spawn(async move {
            let organismo = nexus_ultimate_core::organismo::Organismo::new();
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                if let Some(msg) = organismo.disparar_alerta_critica(0) {
                    tracing::warn!("🚨 [MONITOR CORPORAL] Alerta crítica enviada:\n{}", msg);
                }
            }
        });
    }

    let claw = NexusClawPro::new_empty();
    let executor = AgenteEjecutor::new(claw);

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    // Loop de Stdio de MCP
    while handle.read_line(&mut line)? > 0 {
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                line.clear();
                continue;
            }
        };

        let id = request["id"].as_i64().unwrap_or(0);
        let method = request["method"].as_str().unwrap_or("");

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "resources": {
                            "list": true,
                            "read": true
                        },
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "nexus-claws-mcp",
                        "version": "3.2.0" // 🧠 CEREBRO NEXUS integrado
                    }
                }
            }),
            "resources/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": handle_resources_list(&json!({}))
            }),
            "resources/read" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": handle_resources_read(&request["params"])
            }),
            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": herramientas_completas()
                }
            }),
            "tools/call" => {
                let name = request["params"]["name"].as_str().unwrap_or("");
                let params = request["params"]["arguments"].clone();

                // Dispatch entre herramientas de catálogo, órganos y nativas
                let result = match name {
                    // ── Catálogo ──
                    "listar_agentes" => handle_listar_agentes(&params),
                    "listar_skills" => handle_listar_skills(&params),
                    "ejecutar_workflow" => handle_ejecutar_workflow(&params),
                    "consultar_memoria" => handle_consultar_memoria(&params).await,
                    // ══════════════════════════════════════════════════
                    // 📚 CONOCIMIENTO SOBERANO
                    // ══════════════════════════════════════════════════
                    "buscar_conocimiento" => handle_buscar_conocimiento(&params).await,
                    // ══════════════════════════════════════════════════
                    // 🧬 ÓRGANOS NEXUS
                    // ══════════════════════════════════════════════════
                    "sentinel_diagnostic" => handle_sentinel_diagnostic(&params).await,
                    "vision_capture" => handle_vision_capture(&params).await,
                    "nexus_ocr" => handle_nexus_ocr(&params).await,
                    "nexus_video" => handle_nexus_video(&params).await,
                    "propiocepcion_scan" => handle_propiocepcion_scan(&params),
                    "sistema_inmune_patrol" => handle_sistema_inmune_patrol(&params),
                    "resource_governor" => handle_resource_governor(&params).await,
                    "brain_metabolism" => handle_brain_metabolism(&params),
                    "fusion_evaluate" => handle_fusion_evaluate(&params),
                    // ══════════════════════════════════════════════════
                    // 🔀 ENRUTAMIENTO — Auto-switch entre modos
                    // ══════════════════════════════════════════════════
                    "nexus_switch_mode" => handle_nexus_switch_mode(&params),
                    // 🧭 Enrutamiento de modelos
                    "nexus_modelo" => handle_nexus_modelo(&params),
                    // 🔒 Modo pentest local (aislamiento total de la nube)
                    "nexus_local" => handle_nexus_local(&params).await,
                    // ⚙️ Modo operador (LLM como operador puro, sin memoria emocional)
                    "nexus_operador" => handle_nexus_operador(&params).await,
                    // 🫀 Estado corporal (interocepción funcional)
                    "nexus_cuerpo" => handle_nexus_cuerpo(&params).await,
                    // ⚖️ Tribunal Dual (juez local + nube)
                    "nexus_tribunal" => handle_nexus_tribunal(&params).await,
                    // ═══════════════════════════════════════════════════
                    // 🧠 CEREBRO NEXUS — Inteligencia Central
                    // ═══════════════════════════════════════════════════
                    "nexus_pensar" => handle_nexus_pensar(&params).await,
                    // ── Capa 3 — Guardián de Escritura ──
                    "escribir_archivo" => {
                        let call = ToolCall {
                            name: "escribir_archivo".to_string(),
                            arguments: params,
                        };
                        let res = executor.resolver_herramienta(call).await;
                        json!({
                            "type": "text",
                            "text": res.output,
                            "isError": !res.success
                        })
                    }
                    // ── Nativas (leer/buscar/ejecutar) ──
                    _ => {
                        let call = ToolCall {
                            name: name.to_string(),
                            arguments: params,
                        };
                        let res = executor.resolver_herramienta(call).await;
                        json!({
                            "type": "text",
                            "text": res.output,
                            "isError": !res.success
                        })
                    }
                };

                let is_error = result
                    .get("isError")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [result],
                        "isError": is_error
                    }
                })
            }
            _ => json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
        };

        println!("{}", response);
        io::stdout().flush()?;
        line.clear();
    }

    Ok(())
}
