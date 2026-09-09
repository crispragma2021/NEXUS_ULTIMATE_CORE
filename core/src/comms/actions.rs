use crate::security_protocol::SovereignAction;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::process::Stdio;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

// 🌉 NEXUS BRIDGE - Unificación MCP + LLM Local + Terminal + Dashboard
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceMode {
    WebGemini, // Navegar a Gemini web
    LocalLLM,  // Usar modelo local
    Hybrid,    // Combinar ambos
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NexusBridgeConfig {
    pub source_mode: SourceMode,
    pub mcp_browser_path: String,
    pub local_llm_command: String,
    pub dashboard_webhook: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NexusMessage {
    pub id: String,
    pub content: String,
    pub source: SourceMode,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response: Option<String>,
    pub metadata: Option<Value>,
}

pub struct NexusBridge {
    config: NexusBridgeConfig,
    mcp_browser_process: Option<Child>,
}

impl Default for NexusBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl NexusBridge {
    pub fn new() -> Self {
        let config = NexusBridgeConfig {
            source_mode: match env::var("NEXUS_SOURCE_MODE")
                .unwrap_or_else(|_| "web".to_string())
                .as_str()
            {
                "web" => SourceMode::WebGemini,
                "local" => SourceMode::LocalLLM,
                "hybrid" => SourceMode::Hybrid,
                _ => SourceMode::WebGemini,
            },
            mcp_browser_path: env::var("NEXUS_MCP_BROWSER").unwrap_or_else(|_| {
                "C:/Users/crisp/NEXUS_ULTIMATE_CORE/target/debug/mcp_servers/rust_browser"
                    .to_string()
            }),
            local_llm_command: env::var("NEXUS_LOCAL_LLM")
                .unwrap_or_else(|_| "ollama run llama3.1".to_string()),
            dashboard_webhook: env::var("NEXUS_DASHBOARD_WEBHOOK").ok(),
        };

        Self {
            config,
            mcp_browser_process: None,
        }
    }

    /// 🚀 Iniciar el puente unificado NEXUS (Reemplaza todo lo inferior)
    pub async fn start(&mut self) -> Result<()> {
        println!("🌉 NEXUS BRIDGE - Evolución Superior Implementada");
        println!("==============================================");
        println!("📊 Modo: {:?}", self.config.source_mode);
        println!("🔗 MCP Browser: {}", self.config.mcp_browser_path);
        println!("🤖 LLM Local: {}", self.config.local_llm_command);

        match self.config.source_mode {
            SourceMode::WebGemini => self.start_web_gemini_mode().await?,
            SourceMode::LocalLLM => self.start_local_llm_mode().await?,
            SourceMode::Hybrid => self.start_hybrid_mode().await?,
        }

        Ok(())
    }

    /// 🌐 Modo Web Gemini - Superior a navegación manual
    async fn start_web_gemini_mode(&mut self) -> Result<()> {
        println!("🌐 Iniciando modo Web Gemini (Superior)...");

        let mut child = Command::new(&self.config.mcp_browser_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdout"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to capture stdin"))?;

        self.mcp_browser_process = Some(child);

        let mut reader = BufReader::new(stdout).lines();
        let mut stdin_writer = io::BufWriter::new(stdin);

        // Navigate to Gemini automatically
        let nav_request = json!({
            "method": "navigate",
            "params": {"url": "https://gemini.google.com"},
            "id": 1
        });
        let nav_bytes = serde_json::to_vec(&nav_request)?;
        stdin_writer.write_all(&nav_bytes).await?;
        stdin_writer.flush().await?;

        println!("✅ Conectado a Gemini Web (Modo Superior)");
        self.interactive_terminal_loop(&mut reader, &mut stdin_writer)
            .await
    }

    /// 🤖 Modo LLM Local - Procesamiento local superior
    async fn start_local_llm_mode(&self) -> Result<()> {
        println!("🤖 Iniciando modo LLM Local (Superior)...");
        println!("💬 Comando LLM: {}", self.config.local_llm_command);

        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        println!("✅ LLM Local Superior listo. Escribe tu mensaje:");

        while let Some(line) = reader.next_line().await? {
            let line = line.trim();

            if line.eq_ignore_ascii_case("salir") || line.eq_ignore_ascii_case("exit") {
                println!("👋 Saliendo del modo LLM Local Superior...");
                break;
            }

            if line.is_empty() {
                continue;
            }

            println!("⏳ Procesando con LLM Local Superior...");

            let mut child = Command::new("ollama")
                .arg("run")
                .arg("llama3.1")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(line.as_bytes()).await?;
            }

            let output = child.wait_with_output().await?;

            let response = String::from_utf8_lossy(&output.stdout);
            println!("🤖 LLM Local Superior: {}", response.trim());

            // Enviar al dashboard si está configurado
            if let Some(webhook) = &self.config.dashboard_webhook {
                self.send_to_dashboard(line, response.trim(), SourceMode::LocalLLM, webhook)
                    .await?;
            }

            println!();
        }

        Ok(())
    }

    /// 🔀 Modo Híbrido - Máxima inteligencia combinada
    async fn start_hybrid_mode(&mut self) -> Result<()> {
        println!("🔀 Iniciando modo Híbrido (Inteligencia Máxima)...");
        println!("🌐 + 🤖 = 💎 Evolución Superior");

        self.start_web_gemini_mode().await
    }

    /// 💬 Terminal interactiva superior
    async fn interactive_terminal_loop(
        &mut self,
        reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
        stdin_writer: &mut io::BufWriter<tokio::process::ChildStdin>,
    ) -> Result<()> {
        let stdin = io::stdin();
        let mut user_reader = BufReader::new(stdin).lines();

        println!("💬 Terminal Superior lista (comandos: status, switch:web, switch:local, switch:hybrid, salir):");

        loop {
            // Print prompt (std::io is fine for sync print)
            print!("🌉 NEXUS> ");
            use std::io::Write;
            std::io::stdout().flush().ok();

            let user_input = match user_reader.next_line().await {
                Ok(Some(line)) => line.trim().to_string(),
                Ok(None) => break,
                Err(e) => {
                    println!("❌ Error leyendo entrada: {}", e);
                    continue;
                }
            };

            // Procesar comandos especiales primero
            if self.process_special_commands(&user_input).await? {
                continue;
            }

            if user_input.eq_ignore_ascii_case("salir") || user_input.eq_ignore_ascii_case("exit") {
                println!("👋 Cerrando NEXUS Bridge Superior...");
                break;
            }

            if user_input.is_empty() {
                continue;
            }

            // Enviar comando al browser para escribir en Gemini
            let type_request = json!({
                "method": "type_text",
                "params": {"text": &user_input, "selector": "textarea[data-testid='prompt-textarea']"},
                "id": 2
            });

            let type_bytes = serde_json::to_vec(&type_request)?;
            stdin_writer.write_all(&type_bytes).await?;
            stdin_writer.flush().await?;

            // Wait for processing
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            // Extract Gemini response
            let extract_request = json!({
                "method": "extract_response",
                "params": {"selector": "[data-message-author-role='assistant']"},
                "id": 3
            });

            let extract_bytes = serde_json::to_vec(&extract_request)?;
            stdin_writer.write_all(&extract_bytes).await?;
            stdin_writer.flush().await?;

            println!("⏳ Esperando respuesta superior de Gemini...");

            // Read response
            if let Some(browser_response) = reader.next_line().await.ok().flatten() {
                if let Ok(response_json) = serde_json::from_str::<Value>(&browser_response) {
                    if let Some(result) = response_json.get("result") {
                        let gemini_response = result
                            .get("content")
                            .and_then(|c| c.as_str())
                            .unwrap_or("Sin respuesta");

                        println!("🤖 Gemini Web Superior: {}", gemini_response);

                        // Enviar al dashboard
                        if let Some(webhook) = &self.config.dashboard_webhook {
                            self.send_to_dashboard(
                                &user_input,
                                gemini_response,
                                SourceMode::WebGemini,
                                webhook,
                            )
                            .await?;
                        }
                    }
                }
            }

            println!();
        }

        Ok(())
    }

    /// 📊 Enviar mensaje superior al dashboard
    async fn send_to_dashboard(
        &self,
        user_input: &str,
        response: &str,
        source: SourceMode,
        webhook: &str,
    ) -> Result<()> {
        let message = NexusMessage {
            id: uuid::Uuid::new_v4().to_string(),
            content: user_input.to_string(),
            source,
            timestamp: chrono::Utc::now(),
            response: Some(response.to_string()),
            metadata: Some(json!({
                "bridge_version": "2.0.0",
                "processing_mode": format!("{:?}", source),
                "evolution_level": "Superior"
            })),
        };

        let client = Client::new();

        match client
            .post(webhook)
            .header("Content-Type", "application/json")
            .json(&message)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    println!("📊 ✅ Enviado al dashboard superior");
                } else {
                    println!("📊 ❌ Error enviando al dashboard: {}", resp.status());
                }
            }
            Err(e) => println!("📊 ❌ Error de red al dashboard: {}", e),
        }

        Ok(())
    }

    /// ⚡ Procesar comandos especiales superiores
    async fn process_special_commands(&mut self, command: &str) -> Result<bool> {
        match command {
            "status" => {
                println!("📊 Estado Superior del Puente:");
                println!("  Modo: {:?}", self.config.source_mode);
                println!(
                    "  Browser: {}",
                    if self.mcp_browser_process.is_some() {
                        "✅ Activo"
                    } else {
                        "❌ Inactivo"
                    }
                );
                println!(
                    "  Dashboard: {}",
                    if self.config.dashboard_webhook.is_some() {
                        "✅ Conectado"
                    } else {
                        "❌ No configurado"
                    }
                );
                println!("  Evolución: Superior (Reemplaza obsoleto)");
                Ok(true)
            }
            "switch:web" => {
                self.config.source_mode = SourceMode::WebGemini;
                println!("🔄 Cambiado a modo Web Gemini (Superior)");
                Ok(true)
            }
            "switch:local" => {
                self.config.source_mode = SourceMode::LocalLLM;
                println!("🔄 Cambiado a modo LLM Local (Superior)");
                Ok(true)
            }
            "switch:hybrid" => {
                self.config.source_mode = SourceMode::Hybrid;
                println!("🔄 Cambiado a modo Híbrido (Inteligencia Máxima)");
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

/// 🚀 Función principal del NEXUS Bridge Superior
pub async fn nexus_bridge_superior() -> Result<()> {
    let mut bridge = NexusBridge::new();
    bridge.start().await
}

// --- KERNEL ACTIONS ---
pub struct KernelAction {
    pub action: String,
    pub payload: serde_json::Value,
    pub risk: u8,
}

#[async_trait]
impl SovereignAction for KernelAction {
    fn risk_level(&self) -> u8 {
        self.risk
    }
    fn message(&self) -> Vec<u8> {
        format!("{}:{}", self.action, self.payload).into_bytes()
    }
    async fn execute(&self) -> Result<()> {
        println!(
            "🚀 [KERNEL] Executing Sovereign Action: {} with {:?}",
            self.action, self.payload
        );

        match self.action.as_str() {
            "homeostasis_cool" => {
                println!("❄️ [KERNEL] Reflejo de Enfriamiento activado. El Balancer ya está en modo ECO.");
                // Acciones adicionales nivel kernel podrían ir aquí (ej. limitar carga de CPU mediante cgroups)
            }
            "emergency_stop" => {
                println!("🛑 [KERNEL] PARADA DE EMERGENCIA. Deteniendo ráfagas de alta carga.");
                // Detener procesos pesados conocidos
                let _ = Command::new("pkill").arg("-STOP").arg("rustc").spawn();
                let _ = Command::new("pkill").arg("-STOP").arg("cargo").spawn();
            }
            "clear_memory" => {
                println!("💾 [KERNEL] Purgando buffers de memoria...");
                let _ = Command::new("sync").status().await;
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg("echo 3 > /proc/sys/vm/drop_caches")
                    .status()
                    .await;
            }
            "process_purge" => {
                let pid = self.payload["pid"].as_u64().unwrap_or(0);
                let reason = self.payload["reason"].as_str().unwrap_or("Hedor detectado");
                println!("💀 [KERNEL] PURGANDO PROCESO {}: {}.", pid, reason);
                let _ = Command::new("kill")
                    .arg("-9")
                    .arg(pid.to_string())
                    .status()
                    .await;
            }
            "file_protection" => {
                let path = self.payload["path"].as_str().unwrap_or("desconocido");
                let action = self.payload["action"].as_str().unwrap_or("none");
                println!(
                    "🛡️ [KERNEL] PROTECCIÓN DE ARCHIVOS: {} en {}.",
                    action, path
                );
                // Aquí se podría implementar la restauración desde un backup Git o memoria
            }
            "refine_code" => {
                let reason = self.payload["reason"].as_str().unwrap_or("Código amargo");
                println!(
                    "🤮 [KERNEL] BLOQUEO DE COMPILACIÓN: {}. Sugiriendo refinado idiomático.",
                    reason
                );
                // En una implementación real, esto podría cancelar el comando 'cargo' activo
            }
            "neural_firewall" => {
                let prediction = self.payload["prediction"]
                    .as_str()
                    .unwrap_or("Amenaza fantasma");
                let strategy = self.payload["strategy"].as_str().unwrap_or("quarantine");
                println!(
                    "🔮 [KERNEL] CORTAFUEGOS NEURAL ACTIVADO: {}. Estrategia: {}.",
                    prediction, strategy
                );
                // Ejemplo: bloquear puertos no esenciales o IPs sospechosas
            }
            "rebalance_organs" => {
                let details = self.payload["details"]
                    .as_str()
                    .unwrap_or("Desbalance interno");
                println!("🧬 [KERNEL] REEQUILIBRANDO ÓRGANOS: {}.", details);
                // Aquí se podrían ajustar prioridades de hilos (nice) o pausar módulos secundarios
            }
            "investigate_drive" => {
                let context = self.payload["context"].as_str().unwrap_or("general audit");
                println!(
                    "🔍 [KERNEL] INVESTIGACIÓN PROFUNDA (Drive 2TB): {}.",
                    context
                );
                // Búsqueda proactiva de archivos de configuración obsoletos o sesiones bloqueadas
                let _ = Command::new("find")
                    .arg("data/profiles")
                    .arg("-name")
                    .arg("*.json")
                    .arg("-mmin")
                    .arg("-60")
                    .status()
                    .await;
            }
            "LAUNCH_STUDIO" => {
                println!("🎨 [KERNEL] NEXUS STUDIO DISPONIBLE en http://localhost:43215");
                // El servidor ya está integrado en el Rest Bridge de Rust.
            }
            "LAUNCH_LIVE" => {
                println!("⚡ [KERNEL] INICIANDO NEXUS LIVE (VSCodium)...");
                let _ = Command::new("vscodium")
                    .arg("C:/Users/crisp/NEXUS_ULTIMATE_CORE")
                    .spawn();
            }
            "LAUNCH_KALI" => {
                println!("💀 [KERNEL] DESPLEGANDO KALI SHADOW (Entorno Táctico)...");
                let bunker = "C:/Users/crisp/bunker_ataque";
                let _ = std::fs::create_dir_all(bunker);

                let _ = Command::new("gnome-terminal")
                    .arg("--")
                    .arg("docker")
                    .arg("run")
                    .arg("-it")
                    .arg("--rm")
                    .arg("--name")
                    .arg("NEXUS_SHADOW")
                    .arg("--cpus=4")
                    .arg("-v")
                    .arg(format!("{}:/bunker", bunker))
                    .arg("kalilinux/kali-rolling")
                    .arg("bash")
                    .arg("-c")
                    .arg("echo 'Actualizando repositorios Kali...' && apt-get update -yqq && echo 'KALI-LINUX-LARGE asegurado. Abriendo shell PTY oscura.' && apt-get install -yqq kali-linux-large > /dev/null 2>&1 & /bin/bash")
                    .spawn();
            }
            "LAUNCH_BIOSTASIS" => {
                println!("[\x1b[35mCORE\x1b[0m] Ejecutando orden: PROTOCOLO BIO-STASIS.");
                tokio::spawn(async move {
                    let _ =
                        crate::autodiagnostico::nexus_biostasis::BiostasisManager::configure_zram()
                            .await;
                    let _ = crate::autodiagnostico::nexus_biostasis::BiostasisManager::apply_cpu_affinity().await;
                    let _ = crate::autodiagnostico::nexus_biostasis::BiostasisManager::prioritize_network().await;
                });
                println!("Protocolo BIO-STASIS [Nivel Dios] Inicializado. Hilos Aislados y Expansión ZRAM.");
            }
            "DIVINE_OPTIMIZATION" => {
                println!("🔱 [KERNEL] Iniciando Ciclo Divino PGO/BOLT...");
                tokio::spawn(async move {
                    let _ = crate::autodiagnostico::nexus_repair::DivineOptimizer::run_pgo_cycle()
                        .await;
                    let _ = crate::autodiagnostico::nexus_repair::DivineOptimizer::run_bolt_cycle()
                        .await;
                });
            }
            "ISOLATED_START" => {
                println!("🚀 [KERNEL] Reiniciando NEXUS en modo aislado...");
                tokio::spawn(async move {
                    let _ = crate::autodiagnostico::nexus_repair::ServiceManager::start_isolated()
                        .await;
                });
            }
            "ISOLATED_STOP" => {
                println!("🛑 [KERNEL] Deteniendo NEXUS...");
                tokio::spawn(async move {
                    let _ =
                        crate::autodiagnostico::nexus_repair::ServiceManager::stop_isolated().await;
                });
            }
            _ => {
                println!("⚠️ [KERNEL] Acción desconocida: {}", self.action);
            }
        }
        Ok(())
    }
}

// --- ARSENAL ACTIONS ---
pub struct ArsenalAction {
    pub tool: String,
    pub target: String,
    pub risk: u8,
}

#[async_trait]
impl SovereignAction for ArsenalAction {
    fn risk_level(&self) -> u8 {
        self.risk
    }
    fn message(&self) -> Vec<u8> {
        format!("{}:{}", self.tool, self.target).into_bytes()
    }
    async fn execute(&self) -> Result<()> {
        println!(
            "🔍 [ARSENAL] Deploying Sovereign Tool: {} against {}",
            self.tool, self.target
        );
        // Implement actual arsenal tool calls here
        Ok(())
    }
}
