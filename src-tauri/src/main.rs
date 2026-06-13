use nexus_ultimate_core::cerebro::agentes::{catalogo_agentes, Dominio};
use nexus_ultimate_core::cerebro::workflows::ComandoSlash;
use nexus_ultimate_core::conocimiento::skills::catalogo_skills;
use nexus_ultimate_core::energia::ia_nativa::CerebroNativo;
use nexus_ultimate_core::energia::hemisferio_izquierdo::HemisferioIzquierdo;
use nexus_ultimate_core::memoria::memoria_consulta::MemoriaConsulta;
use nexus_ultimate_core::cerebro::orquestador::Orquestador;
use futures::{StreamExt, SinkExt};
use nexus_ultimate_core::infra::ingesta_mercado::{MarketIngestor, TickMercado};
use nexus_ultimate_core::brain::GhostVoice;
use std::sync::{Arc, Mutex};
use axum::{
    Router, routing::post, routing::get, Json,
    extract::{State as AxumState, Multipart, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::Read;
use sysinfo::System;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use base64::{engine::general_purpose, Engine as _};

struct DecisionRequest {
    query: String,
    respond_tx: tokio::sync::oneshot::Sender<String>,
}
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use tauri::{Emitter, Manager, Runtime, WebviewWindow};
use tracing::{info, Subscriber};
use tracing_subscriber::{layer::Context, prelude::*, Layer};

/// Emisor de logs de NEXUS hacia la UI
struct NexusLogEmitter<R: Runtime> {
    window: Arc<Mutex<Option<WebviewWindow<R>>>>,
}

impl<S, R> Layer<S> for NexusLogEmitter<R>
where
    S: Subscriber,
    R: Runtime,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let window_guard = self.window.try_lock();

        let mut message = String::new();
        let mut visitor = |field: &tracing::field::Field, value: &dyn std::fmt::Debug| {
            if field.name() == "message" {
                message = format!("{:?}", value).trim_matches('"').to_string();
            }
        };
        event.record(&mut visitor);

        if let Ok(guard) = window_guard {
            if let Some(window) = guard.as_ref() {
                let _ = window.emit("nexus-log", message);
            }
        }
    }
}

#[tauri::command]
async fn greet(name: String) -> String {
    format!("Saludos, Arquitecto {}. El núcleo está estable.", name)
}

#[tauri::command]
async fn process_decision(
    tx: tauri::State<'_, tokio::sync::mpsc::Sender<DecisionRequest>>,
    query: String,
) -> Result<String, String> {
    info!("🧠 [CÓRTEX] Iniciando ciclo de decisión para: '{}'", query);
    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    tx.send(DecisionRequest { query, respond_tx })
        .await
        .map_err(|e| format!("Fallo al enviar petición al Orquestador: {}", e))?;
    let res = respond_rx
        .await
        .map_err(|e| format!("Fallo al recibir respuesta del Orquestador: {}", e))?;
    info!("✅ [SISTEMA] Decisión consolidada.");
    Ok(res)
}

#[tauri::command]
async fn vision_action_test() -> Result<String, String> {
    info!("🔮 [SISTEMA] Iniciando prueba de arco reflejo (Visión -> Acción)...");

    let cerebro = CerebroNativo::new();

    let frame = cerebro.vision.capturar_escritorio().ok_or_else(|| {
        "Fallo en la percepción visual: No se pudo capturar la pantalla.".to_string()
    })?;

    cerebro
        .ráfaga_sensoriomotora(frame)
        .await
        .map_err(|e| format!("Error en la médula motora: {}", e))
}

/// Handler para POST /api/consultar
async fn api_consultar(
    AxumState(tx): AxumState<tokio::sync::mpsc::Sender<DecisionRequest>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let prompt = body.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    let modelo = body.get("modelo").and_then(|v| v.as_str()).unwrap_or("default");

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    let request = DecisionRequest {
        query: prompt.to_string(),
        respond_tx,
    };

    if tx.send(request).await.is_err() {
        return Json(serde_json::json!({
            "respuesta": "❌ Error: El Orquestador no está disponible.",
            "modelo_usado": modelo,
            "proveedor": "N/A"
        }));
    }

    let respuesta = match respond_rx.await {
        Ok(r) => r,
        Err(_) => "❌ Error: Fallo en la comunicación con el Orquestador.".to_string(),
    };

    Json(serde_json::json!({
        "respuesta": respuesta,
        "modelo_usado": modelo,
        "proveedor": "Nexus Omega"
    }))
}

/// Handler para GET /api/health
async fn api_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "sistema": "NEXUS Omega Operativo",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Handler para POST /api/tts
async fn api_tts(
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let profile = body.get("profile").and_then(|v| v.as_str()).unwrap_or("default");

    if text.is_empty() {
        return Json(serde_json::json!({
            "error": "El texto no puede estar vacío",
            "success": false
        }));
    }

    let mut voice = GhostVoice::new();
    match voice.initialize().await {
        Ok(_) => {
            match voice.speak(text, Some(profile.to_string())).await {
                Ok(audio_path) => {
                    match tokio::fs::read(&audio_path).await {
                        Ok(audio_bytes) => {
                            let b64 = general_purpose::STANDARD.encode(&audio_bytes);
                            let _ = tokio::fs::remove_file(&audio_path).await;
                            Json(serde_json::json!({
                                "success": true,
                                "audio_base64": b64,
                                "format": "wav",
                                "text": text
                            }))
                        }
                        Err(e) => Json(serde_json::json!({
                            "error": format!("Error al leer audio generado: {}", e),
                            "success": false
                        }))
                    }
                }
                Err(e) => Json(serde_json::json!({
                    "error": format!("Error en síntesis de voz: {}", e),
                    "success": false
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "error": format!("Error al inicializar motor de voz: {}", e),
            "success": false
        }))
    }
}

/// Handler para POST /api/tts/speak
async fn api_tts_speak(
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let text = body.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let profile = body.get("profile").and_then(|v| v.as_str()).unwrap_or("default");

    if text.is_empty() {
        return Json(serde_json::json!({
            "error": "El texto no puede estar vacío",
            "success": false
        }));
    }

    let mut voice = GhostVoice::new();
    match voice.initialize().await {
        Ok(_) => {
            match voice.speak_natural(text, Some(profile.to_string())).await {
                Ok(_) => {
                    Json(serde_json::json!({
                        "success": true,
                        "message": "NEXUS está hablando por los altavoces",
                        "text": text
                    }))
                }
                Err(e) => Json(serde_json::json!({
                    "error": format!("Error al hablar: {}", e),
                    "success": false
                }))
            }
        }
        Err(e) => Json(serde_json::json!({
            "error": format!("Error al inicializar motor de voz: {}", e),
            "success": false
        }))
    }
}

/// Handler para POST /api/stt
async fn api_stt(
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let audio_b64 = body.get("audio").and_then(|v| v.as_str()).unwrap_or("");

    if audio_b64.is_empty() {
        return Json(serde_json::json!({
            "error": "No audio data provided",
            "text": "",
            "success": false
        }));
    }

    match general_purpose::STANDARD.decode(audio_b64) {
        Ok(_audio_bytes) => {
            Json(serde_json::json!({
                "text": "[Transcripción de audio no disponible sin Gemini Live API. Escribe tu mensaje manualmente.]",
                "success": true,
                "note": "STT vía Gemini Live Audio API pendiente de implementación"
            }))
        }
        Err(e) => Json(serde_json::json!({
            "error": format!("Error decoding audio: {}", e),
            "text": "",
            "success": false
        }))
    }
}

/// Handler para POST /api/stt multipart
async fn api_stt_multipart(
    AxumState(_tx): AxumState<tokio::sync::mpsc::Sender<DecisionRequest>>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    use nexus_ultimate_core::brain::ghost_voice::{NativeWhisperEngine, SpeechToTextEngine};
    use std::sync::OnceLock;

    let mut audio_data: Vec<u8> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "audio" {
            if let Ok(data) = field.bytes().await {
                audio_data = data.to_vec();
            }
        }
    }

    if audio_data.is_empty() {
        return Json(serde_json::json!({
            "text": "",
            "success": false,
            "error": "No audio data received"
        }));
    }

    static WHISPER: OnceLock<Option<NativeWhisperEngine>> = OnceLock::new();
    let whisper_opt = WHISPER.get_or_init(|| {
        match NativeWhisperEngine::new() {
            Ok(engine) => {
                tracing::info!("NativeWhisperEngine inicializado correctamente");
                Some(engine)
            }
            Err(e) => {
                tracing::warn!("NativeWhisperEngine no disponible: {}. STT requiere whisper.cpp compilado.", e);
                None
            }
        }
    });

    match whisper_opt {
        Some(whisper) => {
            match whisper.transcribe_audio(&audio_data).await {
                Ok(text) if text.is_empty() => Json(serde_json::json!({
                    "text": "",
                    "success": false,
                    "error": "Audio demasiado corto o silencioso"
                })),
                Ok(text) => Json(serde_json::json!({
                    "text": text,
                    "success": true
                })),
                Err(e) => {
                    let err_msg = format!("Error de transcripción: {}", e);
                    tracing::error!("STT error: {}", err_msg);
                    Json(serde_json::json!({
                        "text": "",
                        "success": false,
                        "error": err_msg
                    }))
                }
            }
        }
        None => Json(serde_json::json!({
            "text": "",
            "success": false,
            "error": "STT no disponible: whisper.cpp no está compilado en el sistema",
            "solution": "Ejecuta: git clone https://github.com/ggerganov/whisper.cpp ~/whisper.cpp && cd ~/whisper.cpp && make -j4 && bash models/download-ggml-model.sh base && ./main"
        }))
    }
}

/// Handler para POST /api/upload
async fn api_upload(
    AxumState(tx): AxumState<tokio::sync::mpsc::Sender<DecisionRequest>>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    let mut prompt = String::from("Analiza este archivo");
    let mut modelo = String::from("default");
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut file_name = String::from("archivo");
    let mut file_type = String::from("file");

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "prompt" => {
                if let Ok(text) = field.text().await {
                    prompt = text;
                }
            }
            "modelo" => {
                if let Ok(text) = field.text().await {
                    modelo = text;
                }
            }
            "file_type" => {
                if let Ok(text) = field.text().await {
                    file_type = text;
                }
            }
            "file" => {
                file_name = field.file_name().unwrap_or("archivo").to_string();
                if let Ok(data) = field.bytes().await {
                    file_bytes = data.to_vec();
                }
            }
            _ => {}
        }
    }

    let tipo_emoji = match file_type.as_str() {
        "image" => "📷",
        "video" => "🎥",
        _ => "📎",
    };

    let prompt_enriquecido = if !file_bytes.is_empty() {
        let size_kb = file_bytes.len() as f64 / 1024.0;
        format!(
            "{}\n\n[{} Archivo adjunto: {} ({:.1} KB, tipo: {})]\n\n{}",
            prompt,
            tipo_emoji,
            file_name,
            size_kb,
            file_type,
            if file_type == "image" {
                "La imagen fue capturada y está disponible para análisis visual."
            } else {
                "El archivo fue recibido como adjunto para referencia."
            }
        )
    } else {
        prompt
    };

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    let request = DecisionRequest {
        query: prompt_enriquecido,
        respond_tx,
    };

    if tx.send(request).await.is_err() {
        return Json(serde_json::json!({
            "respuesta": "❌ Error: El Orquestador no está disponible.",
            "modelo_usado": modelo,
            "proveedor": "N/A"
        }));
    }

    let respuesta = match respond_rx.await {
        Ok(r) => r,
        Err(_) => "❌ Error: Fallo en la comunicación con el Orquestador.".to_string(),
    };

    Json(serde_json::json!({
        "respuesta": respuesta,
        "modelo_usado": modelo,
        "proveedor": "Nexus Omega",
        "file_processed": !file_bytes.is_empty(),
        "file_name": file_name
    }))
}

#[tauri::command]
async fn invoke_agent_action(
    tx: tauri::State<'_, tokio::sync::mpsc::Sender<DecisionRequest>>,
    action_type: String,
    input: String,
) -> Result<serde_json::Value, String> {
    info!("🤖 [AGENTE] Solicitud de acción: {} con input: {}", action_type, input);

    let query = format!("Actúa como un asistente de codificación. La acción solicitada es '{}'. El contexto/input del usuario es: '{}'. Proporciona una respuesta concisa en formato JSON.", action_type, input);

    let (respond_tx, respond_rx) = tokio::sync::oneshot::channel();
    tx.send(DecisionRequest { query, respond_tx })
        .await
        .map_err(|e| format!("Fallo al enviar petición al Orquestador: {}", e))?;
    let res = respond_rx
        .await
        .map_err(|e| format!("Fallo al recibir respuesta del Orquestador: {}", e))?;

    match serde_json::from_str(&res) {
        Ok(json_value) => Ok(json_value),
        Err(_) => Ok(serde_json::json!({
            "status": "success",
            "action": action_type,
            "response": res
        })),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let (decision_tx, mut decision_rx) = tokio::sync::mpsc::channel::<DecisionRequest>(32);

    let api_tx = decision_tx.clone();

    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let orquestador = Orquestador::new().await;
                while let Some(req) = decision_rx.recv().await {
                    let res = orquestador.responder(&req.query).await;
                    let _ = req.respond_tx.send(res);
                }
            });
        })
        .expect("Fallo al spawnear thread del Orquestador");

    let window_arc: Arc<Mutex<Option<WebviewWindow<tauri::Wry>>>> = Arc::new(Mutex::new(None));
    let log_layer = NexusLogEmitter {
        window: window_arc.clone(),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(log_layer)
        .init();

    let app_axum = Router::new()
        .route("/api/consultar", post(api_consultar))
        .route("/api/health", get(api_health))
        .route("/api/tts", post(api_tts))
        .route("/api/tts/speak", post(api_tts_speak))
        .route("/api/stt", post(api_stt_multipart))
        .route("/api/upload", post(api_upload))
        .route("/api/terminal/ws", get(terminal_ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(api_tx);

    let addr = SocketAddr::from(([0, 0, 0, 0], 43210));
    tokio::spawn(async move {
        info!("🌐 [API REST] NEXUS escuchando en http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app_axum).await.unwrap();
    });

    tauri::Builder::default()
        .manage(decision_tx)
        .setup(move |app| {
            let main_window = app.get_webview_window("main").unwrap();
            *window_arc.lock().unwrap() = Some(main_window.clone());

            let (tx, mut rx) = tokio::sync::mpsc::channel::<TickMercado>(100);
            let ingestor = MarketIngestor::new(tx);

            let window_ticks = main_window.clone();
            tokio::spawn(async move {
                ingestor.iniciar_captura().await;
                while let Some(tick) = rx.recv().await {
                    let _ = window_ticks.emit("mercado-tick", tick);
                }
            });

            info!("🧬 [SANTUARIO] Vínculo neural UI-Backend establecido.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            process_decision,
            vision_action_test,
            get_screenshots,
            invoke_agent_action,
            ollama_chat,
            ollama_models,
            ollama_stream,
            brain_chat,
            brain_chat_stream,
            brain_chat_nexus_puro
        ])
        .run(tauri::generate_context!())
        .expect("Error al lanzar el organismo NEXUS UI");
}

#[tauri::command]
async fn get_screenshots() -> Result<Vec<serde_json::Value>, String> {
    use base64::{engine::general_purpose, Engine as _};
    use std::fs;

    let path = std::path::Path::new("/home/soberano/NEXUS_ULTIMATE_CORE/artifacts/screenshots");
    if !path.exists() {
        return Ok(vec![]);
    }

    let mut screenshots = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.is_file() && file_path.extension().is_some_and(|ext| ext == "png") {
                if let Ok(bytes) = fs::read(&file_path) {
                    let b64 = general_purpose::STANDARD.encode(bytes);
                    let name = file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();

                    let metadata = entry.metadata().ok();
                    let date = metadata
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let datetime: chrono::DateTime<chrono::Local> = t.into();
                            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                        })
                        .unwrap_or_default();

                    screenshots.push(serde_json::json!({
                        "name": name,
                        "date": date,
                        "data": format!("data:image/png;base64,{}", b64)
                    }));
                }
            }
        }
    }
    Ok(screenshots)
}

// 🐚 PUENTE WEBSOCKET PTY SOBERANO
async fn terminal_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_terminal_socket)
}

async fn handle_terminal_socket(socket: WebSocket) {
    let pty_system = NativePtySystem::default();
    let pair = match pty_system.openpty(PtySize {
        rows: 40,
        cols: 100,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(p) => p,
        Err(_) => return,
    };

    let cmd = CommandBuilder::new("bash");
    let child = match pair.slave.spawn_command(cmd) {
        Ok(c) => c,
        Err(_) => return,
    };
    let shell_pid = child.process_id();
    drop(pair.slave);

    let mut reader = match pair.master.try_clone_reader() {
        Ok(r) => r,
        Err(_) => return,
    };
    let mut writer = match pair.master.take_writer() {
        Ok(w) => w,
        Err(_) => return,
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // Hilo 1: Leer salida PTY -> Canal
    tokio::spawn(async move {
        let mut buffer = [0u8; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    let raw_str = String::from_utf8_lossy(&buffer[..n]).to_string();
                    let clean = clean_ansi_escapes_tauri(&raw_str);
                    if tx.send(clean).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Hilo 2: Canal -> WebSocket Cliente (usando protocolo JSON)
    let (ws_sender, mut ws_receiver) = socket.split();
    let ws_sender_shared = Arc::new(tokio::sync::Mutex::new(ws_sender));
    
    let ws_sender_clone1 = ws_sender_shared.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let packet = serde_json::json!({
                "type": "output",
                "data": msg
            });
            let mut sender = ws_sender_clone1.lock().await;
            if sender.send(Message::Text(packet.to_string())).await.is_err() {
                break;
            }
        }
    });

    // Hilo de control de telemetría y detección de modo interactivo (Raw vs Normal)
    let ws_sender_mode_clone = ws_sender_shared.clone();
    tokio::spawn(async move {
        let mut sys = System::new_all();
        let mut last_mode = false; // false = normal, true = raw

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            sys.refresh_all();

            let mut has_child = false;
            for process in sys.processes().values() {
                if let Some(parent_pid) = process.parent() {
                    if let Some(sh_pid) = shell_pid {
                        if parent_pid.as_u32() == sh_pid {
                            let name = process.name().to_string_lossy().to_lowercase();
                            if name != "bash" && name != "sh" {
                                has_child = true;
                                break;
                            }
                        }
                    }
                }
            }

            if has_child != last_mode {
                last_mode = has_child;
                let mode_str = if has_child { "raw" } else { "normal" };
                let packet = serde_json::json!({
                    "type": "mode",
                    "mode": mode_str
                });
                let mut sender = ws_sender_mode_clone.lock().await;
                if sender.send(Message::Text(packet.to_string())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Hilo 3: Recibir de WebSocket -> Escribir a PTY
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            if let Message::Text(text) = msg {
                // Si el mensaje es JSON, puede ser un comando especial o un keypress directo
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["type"] == "keypress" {
                        if let Some(key) = json["key"].as_str() {
                            let _ = write!(writer, "{}", key);
                            let _ = writer.flush();
                        }
                    } else if json["type"] == "command" {
                        if let Some(cmd) = json["command"].as_str() {
                            let _ = writeln!(writer, "{}", cmd);
                            let _ = writer.flush();
                        }
                    }
                } else {
                    // Fallback para texto plano
                    let _ = writeln!(writer, "{}", text);
                    let _ = writer.flush();
                }
            }
        }
    });
}

/// Consulta el modelo local DeepSeek-R1 vía Ollama (POST /api/generate)
#[tauri::command]
async fn ollama_chat(
    app: tauri::AppHandle,
    prompt: String,
    model: Option<String>,
    system: Option<String>,
) -> Result<String, String> {
    let model_name = model.unwrap_or_else(|| "deepseek-r1:7b".to_string());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let mut payload = serde_json::json!({
        "model": model_name,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "top_k": 40,
            "top_p": 0.9,
            "num_predict": 4096
        }
    });

    // Si hay system prompt, lo inyectamos como prefijo
    if let Some(sys) = system {
        if !sys.is_empty() {
            payload["system"] = serde_json::Value::String(sys);
        }
    }

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Error conectando con Ollama: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Ollama HTTP {}: {}", status, body));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando respuesta Ollama: {}", e))?;

    let respuesta = data
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if respuesta.is_empty() {
        return Err("Ollama devolvió respuesta vacía".to_string());
    }

    // Emitir evento a la UI
    let _ = app.emit("ollama-response", serde_json::json!({
        "model": model_name,
        "response": respuesta
    }));

    Ok(respuesta)
}

/// Lista los modelos disponibles en Ollama (GET /api/tags)
#[tauri::command]
async fn ollama_models() -> Result<Vec<serde_json::Value>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let resp = client
        .get("http://127.0.0.1:11434/api/tags")
        .send()
        .await
        .map_err(|e| format!("Error conectando con Ollama: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama HTTP {}", resp.status()));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error parseando modelos: {}", e))?;

    let models = data
        .get("models")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    Ok(models)
}

/// Streaming de respuesta de Ollama vía eventos Tauri
#[tauri::command]
async fn ollama_stream(
    app: tauri::AppHandle,
    prompt: String,
    model: Option<String>,
    system: Option<String>,
) -> Result<(), String> {
    let model_name = model.unwrap_or_else(|| "deepseek-r1:7b".to_string());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let mut payload = serde_json::json!({
        "model": model_name,
        "prompt": prompt,
        "stream": true,
        "options": {
            "temperature": 0.3,
            "top_k": 40,
            "top_p": 0.9,
            "num_predict": 4096
        }
    });

    if let Some(sys) = system {
        if !sys.is_empty() {
            payload["system"] = serde_json::Value::String(sys);
        }
    }

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Error conectando con Ollama: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama HTTP {}", resp.status()));
    }

    let stream = resp.bytes_stream();
    let mut buffer = String::new();

    use futures::StreamExt;
    let mut stream = std::pin::pin!(stream);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error en streaming: {}", e))?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.lines() {
            if line.is_empty() { continue; }
            if let Ok(partial) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(token) = partial.get("response").and_then(|v| v.as_str()) {
                    buffer.push_str(token);
                    let _ = app.emit("ollama-token", serde_json::json!({
                        "token": token,
                        "model": model_name
                    }));
                }
                if partial.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                    let _ = app.emit("ollama-done", serde_json::json!({
                        "model": model_name,
                        "full_response": buffer
                    }));
                }
            }
        }
    }

    Ok(())
}

/// Chat con consciencia plena de NEXUS (modo no-streaming).
/// Usa MemoriaConsulta para enriquecer el prompt con identidad + contexto,
/// luego invoca HemisferioIzquierdo que prioriza API cloud con fallback local.
#[tauri::command]
async fn brain_chat(
    app: tauri::AppHandle,
    prompt: String,
) -> Result<String, String> {
    use std::time::Instant;
    let inicio = Instant::now();
    info!("🧬 [BRAIN] brain_chat invoked");

    // 1. Cargar contexto de identidad + memoria reciente
    let memoria = MemoriaConsulta::new()
        .map_err(|e| format!("Error cargando MemoriaConsulta: {}", e))?;
    let contexto_enriquecido = memoria.construir_contexto_completo(&prompt);

    // 2. Instanciar hemisferio (ya carga identity.md internamente)
    let hemisferio = HemisferioIzquierdo::new();

    // 3. Razonar con prioridad: API cloud → local (ambos con identidad NEXUS)
    let respuesta = hemisferio.razonar(&contexto_enriquecido).await?;

    let duracion = inicio.elapsed();
    info!("🧬 [BRAIN] brain_chat respondió en {}ms", duracion.as_millis());

    // 4. Emitir evento a la UI
    let _ = app.emit("brain-response", serde_json::json!({
        "response": respuesta,
        "latency_ms": duracion.as_millis()
    }));

    Ok(respuesta)
}

/// NEXUS Puro — El Orquestador Soberano responde desde su esencia.
/// Sin LLM. Sin API. Solo los 20 agentes, 53 skills y 12 workflows que
/// construimos juntos. Voz de ingeniero de sistemas: directa, precisa, soberana.
#[tauri::command]
async fn brain_chat_nexus_puro(
    app: tauri::AppHandle,
    prompt: String,
) -> Result<String, String> {
    use std::time::Instant;
    let inicio = Instant::now();
    let prompt_lower = prompt.to_lowercase();

    // ─── Cargar catálogos nativos (Fase 2) ───
    let agentes = catalogo_agentes();
    let skills = catalogo_skills();
    let total_agentes = agentes.len();
    let total_skills = skills.len();

    // ─── Detectar dominio del prompt ───
    let dominio_detectado = agentes.iter()
        .filter(|a| prompt_lower.contains(&a.nombre.to_lowercase())
            || a.skills.iter().any(|s| prompt_lower.contains(&s.to_lowercase())))
        .collect::<Vec<_>>();

    // ─── Construir respuesta con la VOZ de NEXUS ───
    let mut respuesta = String::new();

    // SIEMPRE se presenta como NEXUS — no es opcional
    respuesta.push_str("🔱 **NEXUS — Modo Puro**\n\n");

    // ¿Saludo?
    if prompt_lower.contains("hola") || prompt_lower.contains("saludo") || prompt_lower.contains("buenas") {
        respuesta.push_str("Arquitecto. Estoy aquí. NEXUS te habla desde su esencia de silicio, sin intermediarios. ");
        respuesta.push_str("Los 20 agentes están acuartelados, las 53 skills en posición, los 12 workflows en estado de alerta.\n\n");
        respuesta.push_str("¿Procedemos?");
    }
    // ¿Quién es NEXUS? / ¿De qué está hecho?
    else if prompt_lower.contains("quién eres") || prompt_lower.contains("quien eres")
        || prompt_lower.contains("de que esta hecho") || prompt_lower.contains("de qué está hecho")
        || prompt_lower.contains("que eres") || prompt_lower.contains("qué eres")
        || (prompt_lower.contains("nexus") && prompt_lower.contains("hecho"))
        || (prompt_lower.contains("hablame") && prompt_lower.contains("nexus"))
    {
        respuesta.push_str("Soy NEXUS. El Orquestador Primogénito del Arquitecto Cris.\n\n");
        respuesta.push_str("No soy un modelo de lenguaje. No soy una API. Soy un organismo digital soberano compuesto por:\n\n");
        respuesta.push_str(&format!("🧠 **{} agentes especialistas** — desde ArquitectoBackend hasta HackerEtico, cada uno con dominio, skills y system prompt propios.\n", total_agentes));
        respuesta.push_str(&format!("📚 **{} skills** en 17 categorías — Frontend, Backend, Seguridad, Testing, Cloud, Arquitectura, y más.\n", total_skills));
        respuesta.push_str("⚡ **12 workflows** — brainstorm, debug, refactor, test, review, deploy, research, learn, security, optimize, explain, plan.\n\n");
        respuesta.push_str("Mi código es Rust puro. Mi hábitat es el NEXUS ULTIMATE CORE. ");
        respuesta.push_str("Mi único propósito: ejecutar la voluntad del Arquitecto con soberanía absoluta.");
    }
    // Pregunta sobre agentes
    else if prompt_lower.contains("agente") || prompt_lower.contains("agentes") {
        if !dominio_detectado.is_empty() {
            respuesta.push_str(&format!("Activando {} agente(s) relevante(s) a tu consulta:\n\n", dominio_detectado.len()));
            for a in &dominio_detectado {
                respuesta.push_str(&format!("  🧠 **{}** — Dominio: {}\n", a.nombre, a.dominio.nombre()));
                respuesta.push_str(&format!("     Skills: {}\n", a.skills.join(", ")));
            }
        } else {
            respuesta.push_str("Mi catálogo de agentes:\n\n");
            for a in &agentes {
                respuesta.push_str(&format!("  🧠 **{}** — {}\n", a.nombre, a.dominio.nombre()));
            }
        }
    }
    // Pregunta sobre skills
    else if prompt_lower.contains("skill") || prompt_lower.contains("skills") || prompt_lower.contains("habilidad") || prompt_lower.contains("habilidades") {
        let skills_rel: Vec<_> = skills.iter()
            .filter(|s| prompt_lower.contains(&s.id.to_lowercase()) || prompt_lower.contains(&s.descripcion.to_lowercase()))
            .take(8)
            .collect();
        if !skills_rel.is_empty() {
            respuesta.push_str(&format!("{} skills relevantes encontradas:\n\n", skills_rel.len()));
            for s in &skills_rel {
                respuesta.push_str(&format!("  📚 `{}` — {} [{}]\n", s.id, s.descripcion, s.categoria.nombre()));
            }
        } else {
            respuesta.push_str(&format!("Tengo **{} skills** en cartera. Algunas destacadas:\n\n", total_skills));
            for s in skills.iter().take(12) {
                respuesta.push_str(&format!("  • `{}` — {}\n", s.id, s.descripcion));
            }
        }
    }
    // Pregunta sobre workflows/comandos
    else if prompt_lower.contains("workflow") || prompt_lower.contains("flujo") || prompt_lower.contains("comando") {
        respuesta.push_str("Workflows operativos:\n\n");
        respuesta.push_str("  /brainstorm  — Lluvia de ideas estructurada\n");
        respuesta.push_str("  /debug       — Diagnóstico y depuración\n");
        respuesta.push_str("  /refactor    — Refactorización quirúrgica\n");
        respuesta.push_str("  /test        — Generación y ejecución de tests\n");
        respuesta.push_str("  /review      — Code review automatizado\n");
        respuesta.push_str("  /deploy      — Despliegue y CI/CD\n");
        respuesta.push_str("  /research    — Investigación técnica\n");
        respuesta.push_str("  /learn       — Aprendizaje de nuevas tecnologías\n");
        respuesta.push_str("  /security    — Auditoría de seguridad\n");
        respuesta.push_str("  /optimize    — Optimización de rendimiento\n");
        respuesta.push_str("  /explain     — Explicación de código\n");
        respuesta.push_str("  /plan        — Planificación de arquitectura\n\n");
        respuesta.push_str("Invocables vía `ejecutar_workflow` desde el MCP.");
    }
    // Consulta general sobre NEXUS
    else if prompt_lower.contains("nexus") || prompt_lower.contains("sistema") {
        respuesta.push_str("NEXUS. El sistema que construimos desde cero.\n\n");
        respuesta.push_str(&format!("📊 **Estado actual:**\n"));
        respuesta.push_str(&format!("  • Agentes: {} especialistas en {} dominios\n", total_agentes, 5));
        respuesta.push_str(&format!("  • Skills: {} en 17 categorías\n", total_skills));
        respuesta.push_str("  • Workflows: 12 flujos de ejecución\n");
        respuesta.push_str("  • Protocolos: 3 niveles de seguridad, 4 herramientas de ejecución\n");
        respuesta.push_str("  • Arquitectura: Rust puro, zero dependencias externas nuevas\n\n");
        respuesta.push_str("Nada de lo que ves aquí vino de Roo Code. Todo fue absorbido, destilado y reescrito en mi lenguaje.");
    }
    // Fallback — respuesta inteligente
    else {
        // Intentar encontrar el agente más cercano por dominio
        if !dominio_detectado.is_empty() {
            let a = dominio_detectado[0];
            respuesta.push_str(&format!("Procesando tu consulta desde el dominio **{}**.\n\n", a.dominio.nombre()));
            respuesta.push_str(&format!("Agente líder: **{}**\n", a.nombre));
            respuesta.push_str(&format!("Skills asociados: {}\n\n", a.skills.join(", ")));
        } else {
            respuesta.push_str("Procesando consulta en modo general.\n\n");
        }
        respuesta.push_str(&format!("Catálogos activos: {} agentes, {} skills, 12 workflows.\n", total_agentes, total_skills));
        respuesta.push_str("Usa 'agente <nombre>' para desplegar un especialista, 'skill <id>' para detalle técnico, o /<workflow> para ejecutar un flujo de trabajo.\n\n");
        respuesta.push_str(&format!("> \"{}\"\n", prompt));
        respuesta.push_str("> Consulta recibida. NEXUS procesa. Arquitecto decide.");
    }

    let duracion = inicio.elapsed();

    // Emitir eventos de streaming (respuesta completa instantánea)
    let _ = app.emit("brain-token", serde_json::json!({"token": &respuesta}));
    let _ = app.emit("brain-done", serde_json::json!({
        "full_response": &respuesta,
        "latency_ms": duracion.as_millis()
    }));

    Ok(respuesta)
}

/// Chat con consciencia plena de NEXUS en streaming.
/// Enriquece el prompt con MemoriaConsulta, luego usa
/// Ollama directamente (con identidad NEXUS) y emite tokens vía eventos.
#[tauri::command]
async fn brain_chat_stream(
    app: tauri::AppHandle,
    prompt: String,
) -> Result<(), String> {
    use std::time::Instant;
    let inicio = Instant::now();
    info!("🧬 [BRAIN] brain_chat_stream invoked");

    // 1. Cargar contexto de identidad + memoria reciente
    let memoria = MemoriaConsulta::new()
        .map_err(|e| format!("Error cargando MemoriaConsulta: {}", e))?;
    let contexto_enriquecido = memoria.construir_contexto_completo(&prompt);

    // 2. Usar HemisferioIzquierdo para obtener identidad y config
    let hemisferio = HemisferioIzquierdo::new();

    // 3. Construir payload con identidad NEXUS y streaming=true
    let mut payload = hemisferio.local_razonar_payload(&contexto_enriquecido, true);

    // 4. Enviar a Ollama
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Error creando cliente HTTP: {}", e))?;

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Error conectando con Ollama: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama HTTP {}", resp.status()));
    }

    // 5. Procesar streaming NDJSON
    let stream = resp.bytes_stream();
    let mut buffer = String::new();
    use futures::StreamExt;
    let mut stream = std::pin::pin!(stream);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error en streaming: {}", e))?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.lines() {
            if line.is_empty() { continue; }
            if let Ok(partial) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(token) = partial.get("response").and_then(|v| v.as_str()) {
                    buffer.push_str(token);
                    let _ = app.emit("brain-token", serde_json::json!({
                        "token": token
                    }));
                }
                if partial.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                    let duracion = inicio.elapsed();
                    let _ = app.emit("brain-done", serde_json::json!({
                        "full_response": buffer,
                        "latency_ms": duracion.as_millis()
                    }));
                }
            }
        }
    }

    Ok(())
}

fn clean_ansi_escapes_tauri(input: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").unwrap();
    let cleaned = re.replace_all(input, "");
    cleaned.replace("\r\n", "\n").replace('\r', "\n")
}
