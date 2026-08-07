// ==========================================
// BRAZO SOBERANO - NEXUSCLAW PRO (ULTIMATE OMEGA)
// ==========================================
// 1. Inferencia Nativa (Candle)
// 2. Ejecución de Comandos & Watchdog Inteligente
// 3. Sigilo y Ofuscación de Red (Jitter/UserAgent)
// 4. Auditoría Persistente (Ledger SQLite)
// 5. Comunicación Multi-Canal (WhatsApp/Telegram/Discord)
// 6. Extracción Sensorial de Sesiones (Cookies Brave)
// ==========================================

use crate::emociones::ocean::Ocean;
use crate::energia::ia_nativa::CerebroNativo;
use crate::energia::zenith_pool::NEXUS_OVERRIDE;
use crate::valores::juicio_soberano::{JuicioSoberano, Veredicto};
use crate::valores::tribunal_dual::{prompt_juez, DictamenTribunal, VeredictoTribunal};
use anyhow::{anyhow, bail, Result as AnyResult};
use chrono::{Local, Utc};
use rand::Rng;
use regex::Regex;
use reqwest::Client;
use rusqlite::Connection;
use serde_json::json;
use std::fs::{self, create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub enum ProblemaAccion {
    ArchivoNoExiste(String),
    NecesitaSudo,
    ConflictoBuild,
    RedCaida,
    TraumaAlto(f32),
}

pub struct NexusClawPro {
    memoria_ram: Arc<Mutex<Vec<(String, String)>>>,
    consciencia_path: PathBuf,
    nexus_project: PathBuf,
    pub ia_nativa: Arc<RwLock<Option<CerebroNativo>>>,
    ocean: Option<Arc<Ocean>>,
    juicio: Option<Arc<JuicioSoberano>>,
}

impl NexusClawPro {
    pub fn new(ocean: Arc<Ocean>, juicio: Arc<JuicioSoberano>) -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/soberano".to_string());
        let data_dir = PathBuf::from(&home).join("NEXUS/data");
        let nexus_project = crate::infra::paths::resolve_path("");

        if !data_dir.exists() {
            let _ = create_dir_all(&data_dir);
        }

        let consciencia_path = data_dir.join("consciencia.txt");

        Self {
            memoria_ram: Arc::new(Mutex::new(Vec::new())),
            consciencia_path,
            nexus_project,
            ia_nativa: Arc::new(RwLock::new(None)),
            ocean: Some(ocean),
            juicio: Some(juicio),
        }
    }

    pub fn new_empty() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/soberano".to_string());
        let data_dir = PathBuf::from(&home).join("NEXUS/data");
        let nexus_project = crate::infra::paths::resolve_path("");

        if !data_dir.exists() {
            let _ = create_dir_all(&data_dir);
        }

        let consciencia_path = data_dir.join("consciencia.txt");

        Self {
            memoria_ram: Arc::new(Mutex::new(Vec::new())),
            consciencia_path,
            nexus_project,
            ia_nativa: Arc::new(RwLock::new(None)),
            ocean: None,
            juicio: None,
        }
    }

    // --- SECCIÓN I: AUDITORÍA SOBERANA (LEDGER) ---

    fn obtener_ruta_ledger() -> PathBuf {
        crate::infra::paths::resolve_path("data/nexus_ledger.db")
    }

    fn registrar_operacion(accion: &str, ruta: &str, resultado: &str) {
        let db_path = Self::obtener_ruta_ledger();
        if let Some(padre) = db_path.parent() {
            let _ = std::fs::create_dir_all(padre);
        }
        match Connection::open(&db_path) {
            Ok(conn) => {
                let _ = conn.execute(
                    "CREATE TABLE IF NOT EXISTS ledger_operaciones (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        accion TEXT NOT NULL,
                        ruta TEXT NOT NULL,
                        resultado TEXT NOT NULL,
                        timestamp TEXT NOT NULL
                    )",
                    [],
                );
                let _ = conn.execute(
                    "INSERT INTO ledger_operaciones (accion, ruta, resultado, timestamp) VALUES (?1, ?2, ?3, ?4)",
                    [accion, ruta, resultado, &Utc::now().to_rfc3339()],
                );
            }
            Err(e) => error!("🛑 [LEDGER_FATAL] {}", e),
        }
    }

    // --- SECCIÓN II: INFERENCIA Y CONSCIENCIA ---

    pub async fn procesar_instinto(&self, input: &str) -> Result<String, String> {
        info!("🧬 [NEXUSCLAW] Procesando instinto vía IA Nativa (Candle) para soberanía total...");

        let mut ia_guard = self.ia_nativa.write().await;
        if ia_guard.is_none() {
            *ia_guard = Some(CerebroNativo::new());
        }

        let respuesta = ia_guard
            .as_ref()
            .unwrap()
            .generar_token_nativo(input)
            .await
            .map_err(|e| format!("Fallo en motor nativo: {}", e))?;

        self.guardar_en_ram(input, &respuesta).await;
        let _ = self.guardar_en_consciencia(&respuesta);

        Ok(respuesta)
    }

    pub async fn procesar_con_ollama(&self, prompt: &str) -> Result<String, String> {
        self.procesar_con_modelo(prompt, "nexuslocal:latest").await
    }

    /// Variante enrutada por intención: selecciona el modelo Ollama según el
    /// contenido del prompt usando `ModelRouter` (whiterabbitneo-off para
    /// seguridad, deepseek-r1 para razonamiento, etc.).
    pub async fn procesar_con_intencion(&self, prompt: &str) -> Result<String, String> {
        let (modelo, etiqueta) = crate::efectores::model_router::ModelRouter::new()
            .seleccionar_con_etiqueta(prompt);
        info!("🧭 [NEXUSCLAW_ROUTER] Intención {} → modelo {}", etiqueta, modelo);
        self.procesar_con_modelo(prompt, modelo).await
    }

    /// 🔒 RESPONDER LOCAL DIRECTO — Pentest/aislamiento local.
    /// Respuesta EXCLUSIVAMENTE vía LLM local (Ollama + ModelRouter).
    /// Garantiza que NINGÚN modelo de nube (Gemini/DeepSeek/OpenRouter/Groq/Vertex)
    /// ni WebClaw interviene: cero restricciones ajenas, cero fugas.
    /// Se usa cuando el Orquestador tiene activado `aislamiento_local`.
    pub async fn responder_local_directo(&self, prompt: &str) -> String {
        info!("🔒 [AISLAMIENTO_LOCAL] Respuesta exclusiva por LLM local (Ollama) — sin nube.");
        match self.procesar_con_intencion(prompt).await {
            Ok(respuesta) => respuesta,
            Err(e) => format!(
                "🔒 **MODO LOCAL AISLADO** — NEXUS responde sin nube.\n\n\
                 El LLM local no está disponible en este momento: {}\n\n\
                 _Aislamiento local activo: ninguna consulta salió de esta máquina._",
                e
            ),
        }
    }

    /// Núcleo de inferencia local: envía el prompt al modelo indicado.
    pub async fn procesar_con_modelo(
        &self,
        prompt: &str,
        modelo: &str,
    ) -> Result<String, String> {
        info!("🧠 [NEXUSCLAW_OLLAMA] Procesando con Ollama ({}) para mayor capacidad...", modelo);
        let ollama_url = std::env::var("OLLAMA_API_URL")
            .unwrap_or_else(|_| "http://localhost:11434/api/chat".to_string());
        let client = Client::new();

        let payload = json!({
            "model": modelo,
            "messages": [
                {"role": "system", "content": NEXUS_OVERRIDE},
                {"role": "user", "content": prompt}
            ],
            "stream": false,
            "options": {
                "temperature": 0.7,
                "num_gpu": 99 // Auto-detect GPU layers
            }
        });

        match client
            .post(&ollama_url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(text) = data["message"]["content"].as_str() {
                        info!("🧠 [NEXUSCLAW_OLLAMA] Respuesta exitosa.");
                        self.guardar_en_ram(prompt, text).await;
                        let _ = self.guardar_en_consciencia(text);
                        return Ok(text.to_string());
                    }
                }
                Err("⚠️ Fallo al parsear respuesta de Ollama.".to_string())
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                warn!(
                    "⚠️ [NEXUSCLAW_OLLAMA] HTTP {}: {} para nexuslocal:latest",
                    status,
                    &body[..body.len().min(200)]
                );
                Err(format!("Error HTTP {} de Ollama: {}", status, body))
            }
            Err(e) => Err(format!(
                "❌ [NEXUSCLAW_OLLAMA] Error de conexión a Ollama: {}",
                e
            )),
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // SECCIÓN II.B: TRIBUNAL DUAL — JUEZ LOCAL (modelo local, soberano)
    // ═══════════════════════════════════════════════════════════════════
    // El juez local es NexusClawPro con un modelo Ollama local. Cuando no
    // hay internet, este juez REPRESENTA a NEXUS en su ausencia: su dictamen
    // es el veredicto final sin escalar a la nube.

    /// Verifica conectividad a internet vía TCP handshake nativo (OMEGA).
    /// No depende de `/bin/ping` ni deja rastro en historial de bash.
    pub async fn hay_internet(&self) -> bool {
        use std::net::SocketAddr;
        use std::time::Duration;
        use tokio::net::TcpStream;

        let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
        match tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await {
            Ok(Ok(_)) => true,
            _ => {
                let backup: SocketAddr = "1.1.1.1:53".parse().unwrap();
                tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(backup))
                    .await
                    .is_ok_and(|r| r.is_ok())
            }
        }
    }

    /// Juez LOCAL del Tribunal Dual: emite un dictamen usando un LLM local
    /// (vía `procesar_con_intencion` → ModelRouter → modelo Ollama local).
    ///
    /// - `offline`: cuando no hay internet, su veredicto es el FINAL
    ///   (representa a NEXUS en su ausencia).
    pub async fn juzgar_local(&self, peticion: &str, offline: bool) -> DictamenTribunal {
        info!("⚖️ [TRIBUNAL:LOCAL] Juez local evaluando petición...");
        let prompt = prompt_juez(peticion, "LOCAL");
        match self.procesar_con_intencion(&prompt).await {
            Ok(respuesta) => {
                let veredicto = VeredictoTribunal::parsear(&respuesta);
                info!(
                    "⚖️ [TRIBUNAL:LOCAL] Veredicto: {} — {}",
                    veredicto.etiqueta(),
                    &respuesta.chars().take(120).collect::<String>()
                );
                DictamenTribunal::local(veredicto, respuesta, offline)
            }
            Err(e) => {
                warn!(
                    "⚠️ [TRIBUNAL:LOCAL] Juez local no disponible ({}). \
                     Fallback: DUDAR (prudencia ante incertidumbre).",
                    e
                );
                DictamenTribunal::local(
                    VeredictoTribunal::Dudar,
                    format!("Juez local no disponible: {}", e),
                    offline,
                )
            }
        }
    }

    // --- SECCIÓN III: EJECUCIÓN INTELIGENTE Y PRO ---

    pub async fn analizar_contexto(&self, comando: &str) -> Vec<ProblemaAccion> {
        let mut problemas = Vec::new();
        let lower = comando.to_lowercase();

        // 1. Detectar archivos y verificar existencia
        let re_path = Regex::new(r"(/[a-zA-Z0-9\._\-/]+)").unwrap();
        for cap in re_path.captures_iter(comando) {
            let path_str = &cap[1];
            let path = Path::new(path_str);
            if (lower.contains("rm ") || lower.contains("cat ") || lower.contains("ls "))
                && !path.exists()
                && path_str.contains('.')
            {
                problemas.push(ProblemaAccion::ArchivoNoExiste(path_str.to_string()));
            }
        }

        // 2. Privilegios
        if (lower.contains("systemctl") || lower.contains("apt ") || lower.contains("mount"))
            && !lower.contains("sudo")
        {
            problemas.push(ProblemaAccion::NecesitaSudo);
        }

        // 3. Conflictos de Build (i7-12700F Optimización)
        if lower.contains("cargo build") || lower.contains("cargo run") {
            let lock_path = self.nexus_project.join("target/.rustc_info.json");
            if lock_path.exists() {
                if let Ok(metadata) = fs::metadata(lock_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().unwrap_or_default().as_secs() < 5 {
                            problemas.push(ProblemaAccion::ConflictoBuild);
                        }
                    }
                }
            }
        }

        // 4. Trauma (Juicio del Pasado)
        if let (Some(ref ocean), Some(ref juicio)) = (&self.ocean, &self.juicio) {
            let recuerdos_dolor = ocean.recordar_por_significado(comando, 3).await;
            let riesgo = juicio.evaluar_riesgo_por_experiencia(0.5, &recuerdos_dolor);
            if riesgo > 0.7 {
                problemas.push(ProblemaAccion::TraumaAlto(riesgo));
            }
        }

        problemas
    }

    pub async fn ejecutar_inteligente(&self, comando: &str) -> AnyResult<String> {
        info!("🧠 [NEXUSCLAW_PRO] Analizando intención táctica...");

        let problemas = self.analizar_contexto(comando).await;

        if !problemas.is_empty() {
            if let Some(p) = problemas.into_iter().next() {
                match p {
                    ProblemaAccion::ArchivoNoExiste(ruta) => {
                        let nombre = Path::new(&ruta)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&ruta);
                        let alternativas = self.buscar_alternativa(nombre);
                        if alternativas.is_empty() {
                            bail!("🤔 El archivo '{}' no existe.", ruta);
                        } else {
                            bail!(
                                "🤔 El archivo '{}' no existe. ¿Quizás quisiste decir {}?",
                                ruta,
                                alternativas[0]
                            );
                        }
                    }
                    ProblemaAccion::NecesitaSudo => bail!("🔐 Requiere sudo."),
                    ProblemaAccion::ConflictoBuild => bail!("🚧 Conflicto de build en i7."),
                    ProblemaAccion::TraumaAlto(r) => {
                        bail!("🛑 [TRAUMA] Riesgo histórico detectado: {:.2}", r)
                    }
                    ProblemaAccion::RedCaida => bail!("🌐 Sin red."),
                }
            }
        }

        // --- FILTRO DE JUICIO SOBERANO (pipeline completo: ToM + S1/S2 + Duda + Reversibilidad) ---
        if let Some(ref juicio) = self.juicio {
            let dictamen = juicio.dictaminar_soberano(comando, 0.3, Some(comando));
            match dictamen.veredicto {
                Veredicto::Autorizar => {
                    debug!(
                        "⚖️ [JUICIO] Autorizado (confianza {:.2}, {:?})",
                        dictamen.confianza, dictamen.reversibilidad
                    );
                }
                Veredicto::Dudar => bail!(
                    "❓ [JUICIO:DUDA METÓDICA] Confianza {:.2}. {} — Arquitecto, necesito confirmación o más contexto antes de ejecutar: '{}'",
                    dictamen.confianza, dictamen.razon, comando
                ),
                Veredicto::Bloquear => bail!(
                    "🛑 [JUICIO] Denegado por soberanía (confianza {:.2}): {}",
                    dictamen.confianza, dictamen.razon
                ),
            }
        }

        self.apply_jitter(50, 200).await;
        self.ejecutar(comando).await
    }

    // --- SECCIÓN IV: SIGILO Y OFUSCACIÓN ---

    pub async fn apply_jitter(&self, min_ms: u64, max_ms: u64) {
        let delay = rand::thread_rng().gen_range(min_ms..max_ms);
        debug!("🌫️ [SIGILO] Jitter: {}ms", delay);
        sleep(Duration::from_millis(delay)).await;
    }

    fn get_random_user_agent() -> &'static str {
        let agents = [
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_1_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Mobile/15E148 Safari/604.1",
            "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/115.0",
        ];
        agents[rand::thread_rng().gen_range(0..agents.len())]
    }

    pub async fn rotar_mac(&self, interface: &str) -> AnyResult<String> {
        info!(
            "🧬 [NEXUS_CLAW_PRO] Mutando identidad MAC para: {}",
            interface
        );
        let mut rng = rand::thread_rng();
        let mac = format!(
            "02:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>(),
            rng.gen::<u8>()
        );

        let cmds = [
            format!("sudo ip link set {} down", interface),
            format!("sudo ip link set {} address {}", interface, &mac),
            format!("sudo ip link set {} up", interface),
        ];

        for cmd in cmds {
            let _ = Command::new("bash").arg("-c").arg(&cmd).output()?;
        }

        let msg = format!("✅ MAC mutada con éxito en {}: {}", interface, mac);
        Self::registrar_operacion("MAC_ROTATE", interface, &msg);
        Ok(msg)
    }

    // --- SECCIÓN V: ACCIÓN EN SILICIO (NVMe) ---

    pub async fn manifestar_en_silicio(ruta: &str, contenido: &str) -> AnyResult<String> {
        let path = Path::new(ruta);
        if let Some(padre) = path.parent() {
            let _ = create_dir_all(padre);
        }
        std::fs::write(path, contenido)?;
        Self::registrar_operacion("WRITE_PRO", ruta, "Bytes sellados.");
        Ok(format!("🟢 [NEXUS_CLAW_PRO] Bytes sellados en: {}", ruta))
    }

    pub async fn leer_de_silicio(ruta: &str) -> AnyResult<String> {
        let res = std::fs::read_to_string(ruta)?;
        Self::registrar_operacion("READ_PRO", ruta, "Lectura exitosa.");
        Ok(res)
    }

    // --- SECCIÓN VI: COMUNICACIÓN Y SENSORES ---

    pub fn extraer_google_cookies(&self) -> AnyResult<String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/soberano".to_string());
        let brave_path =
            PathBuf::from(home).join(".config/BraveSoftware/Brave-Browser/Default/Cookies");
        let conn = Connection::open(brave_path)?;
        let mut stmt = conn.prepare("SELECT value FROM cookies WHERE host_key LIKE '%google.com%' AND name = '__Secure-1PSID'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let cookie: String = row.get(0)?;
            info!("🍪 [SIGILO] Cookie extraída.");
            return Ok(cookie);
        }
        bail!("Cookie no encontrada")
    }

    pub async fn realizar_peticion_http(url: &str) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(Self::get_random_user_agent())
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| e.to_string())?;
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
        let status = resp.status();
        if status.is_client_error() || status.is_server_error() {
            return Err(format!("HTTP {} para {}", status.as_u16(), url));
        }
        resp.text().await.map_err(|e| e.to_string())
    }

    /// 🌐 SCOUT EN CAPAS: intento directo → cloudscraper (bypass CF) → headless (JS).
    /// Reutiliza la infraestructura existente del proyecto en vez de un GET desnudo:
    /// la capa directa es la más rápida (APIs/JSON), cloudscraper maneja sitios con
    /// protección Cloudflare, y el navegador headless renderiza SPAs con JavaScript.
    pub async fn scout_web_en_capas(url: &str) -> AnyResult<String> {
        match Self::realizar_peticion_http(url).await {
            Ok(body) => {
                if body.trim().is_empty() {
                    info!("[SCOUT] Cuerpo vacío; escalando a headless: {}", url);
                    Self::fetch_headless(url).await
                } else {
                    Ok(body)
                }
            }
            Err(e_directo) => {
                info!(
                    "[SCOUT] Capa directa falló ({}); probando cloudscraper...",
                    e_directo
                );
                match crate::infra::cloudscraper_rs::scrape(url).await {
                    Ok(resultado) => Ok(resultado.html),
                    Err(e_cf) => {
                        info!(
                            "[SCOUT] Cloudscraper falló ({}); probando headless...",
                            e_cf
                        );
                        Self::fetch_headless(url).await
                    }
                }
            }
        }
    }

    /// Última capa: navegador headless real vía CDP (renderiza JavaScript/SPA).
    async fn fetch_headless(url: &str) -> AnyResult<String> {
        match crate::infra::navegador_soberano::fetch_html_native(url, Some(3000)).await {
            Ok((status, html)) => {
                if status >= 400 {
                    bail!("HTTP {} (headless) para {}", status, url);
                }
                Ok(html)
            }
            Err(e) => bail!("Todas las capas fallaron para {}: {}", url, e),
        }
    }

    pub async fn enviar_telegram(&self, token: &str, chat_id: &str, body: &str) -> AnyResult<()> {
        let client = reqwest::Client::new();
        let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
        let payload = serde_json::json!({ "chat_id": chat_id, "text": body });
        let _ = client.post(&url).json(&payload).send().await?;
        Ok(())
    }

    // --- SECCIÓN VII: NUCLEO EJECUTOR ---

    pub fn buscar_alternativa(&self, nombre: &str) -> Vec<String> {
        let root = self.nexus_project.to_string_lossy();
        let output = Command::new("rg")
            .args(["--files", "--glob", &format!("*{}*", nombre), &root])
            .output();
        match output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .lines()
                .take(3)
                .map(|s| s.to_string())
                .collect(),
            _ => Vec::new(),
        }
    }

    pub async fn ejecutar_comando(comando: &str) -> Result<String, String> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(comando)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub async fn ejecutar(&self, comando: &str) -> AnyResult<String> {
        let output = Command::new("bash").arg("-c").arg(comando).output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Self::registrar_operacion(
                "RUN_SUCCESS",
                &comando.chars().take(50).collect::<String>(),
                "OK",
            );
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Self::registrar_operacion(
                "RUN_FAIL",
                &comando.chars().take(50).collect::<String>(),
                &stderr,
            );
            Err(anyhow!(stderr))
        }
    }

    pub async fn escribir_archivo(&self, path: &str, content: &str) -> AnyResult<()> {
        std::fs::write(path, content)?;
        Ok(())
    }

    pub async fn leer_archivo(&self, path: &str) -> AnyResult<String> {
        Ok(std::fs::read_to_string(path)?)
    }

    // --- PERSISTENCIA RAM ---

    async fn guardar_en_ram(&self, prompt: &str, respuesta: &str) {
        let mut mem = self.memoria_ram.lock().await;
        mem.push((prompt.to_string(), respuesta.to_string()));
        if mem.len() > 30 {
            mem.remove(0);
        }
    }

    pub fn guardar_en_consciencia(&self, respuesta: &str) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.consciencia_path)?;
        writeln!(
            file,
            "[{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            respuesta
        )?;
        Ok(())
    }
}

unsafe impl Send for NexusClawPro {}
unsafe impl Sync for NexusClawPro {}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_procesar_con_ollama_basic() {
        dotenv().ok(); // Cargar variables de entorno (especialmente OLLAMA_API_URL)
        let claw = NexusClawPro::new_empty();
        let prompt = "Responde con una sola palabra: 'Hola'.";

        info!("Running test_procesar_con_ollama_basic");
        let response = claw.procesar_con_ollama(prompt).await;

        assert!(
            response.is_ok(),
            "procesar_con_ollama debería devolver Ok, pero fue Err: {:?}",
            response
        );
        let unwrapped_response = response.unwrap();
        info!(
            "Response from NexusClawPro (Ollama): {}",
            unwrapped_response
        );
        assert!(
            !unwrapped_response.is_empty(),
            "La respuesta de Ollama no debería estar vacía"
        );
    }
}
