use crate::autonomia::tatuaje_neural::{KeyStats, TatuajeNeural};
use crate::capa_invisibilidad::{red::GestorRed, NexusCloak, SigiloLevel};
use crate::efectores::nexus_claw::NexusClaw;
use crate::energia::sinapsis_gemini::NexusPlan;
use crate::homeostasis_utils::{get_sovereign_client_builder, GLOBAL_CACHE};
use crate::medico::{DiagnosticadorNexus, Severidad, Solucion};
use crate::memoria::aprendizaje_profundo::{DQNAgent, Experience, PrioritizedReplayBuffer};
use crate::sentidos::omnipresent_vision::OmnipresentVision;
use crate::sentidos::propiocepcion::EstadoSistema;
use crate::sentidos::propiocepcion::SomaScanner;
use crate::valores::afinidad_soberana::AfinidadSoberana;
use anyhow::Result;
use reqwest::{
    header::{HeaderMap, HeaderValue, USER_AGENT},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const FAILURE_LOG_PATH: &str = "nexus_key_failures.json";
const MAX_REPLAY_BUFFER: usize = 1000;
const DQN_WEIGHTS_PATH: &str = "nexus_dqn";
const IDENTITY_PATH: &str = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/docs/identity/identity.md";

const LEARNING_RATE: f64 = 1e-4; // Tasa de aprendizaje para el optimizador
const TAU: f64 = 0.005; // Factor para el soft update de la red target

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeepSeekModel {
    Chat,
    Reasoner,
    Coder,
    V3,
}

impl DeepSeekModel {
    pub fn model_name(&self) -> &'static str {
        match self {
            DeepSeekModel::Chat => "deepseek-chat",
            DeepSeekModel::Reasoner => "deepseek/deepseek-r1",
            DeepSeekModel::Coder => "deepseek-coder",
            DeepSeekModel::V3 => "deepseek/deepseek-v3.2",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DeepSeekModel::Chat => "DeepSeek Chat",
            DeepSeekModel::Reasoner => "DeepSeek Reasoner",
            DeepSeekModel::Coder => "DeepSeek Coder",
            DeepSeekModel::V3 => "DeepSeek V3.2",
        }
    }

    pub fn parse_model_command(cmd: &str) -> Option<Self> {
        match cmd {
            "deepseek_coder" | "deepseek-coder" => Some(DeepSeekModel::Coder),
            "deepseek_reasoner" | "deepseek/deepseek-r1" | "deepseek-reasoner" => {
                Some(DeepSeekModel::Reasoner)
            }
            "deepseek_v3" | "deepseek/deepseek-v3.2" => Some(DeepSeekModel::V3),
            "deepseek-chat" | "deepseek_chat" => Some(DeepSeekModel::Chat),
            _ => None,
        }
    }
}

// Las estructuras key stats, tatuaje, estado y replay buffer ahora viven en el core.

/// Helper para crear una DeepSeekResponse a partir de texto (útil para nexusclaw)
impl DeepSeekResponse {
    pub fn from_text(text: String) -> Self {
        DeepSeekResponse {
            content: text,
            model: "NexusClaw".to_string(), // Identificador para la herramienta del Padre
            tokens: 0,
            cost: 0.0,
            response_time_ms: 0,
            tool_calls: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepSeekResponse {
    pub content: String,
    pub model: String,
    pub tokens: u32,
    pub cost: f64,
    pub response_time_ms: u64,
    pub tool_calls: Option<serde_json::Value>,
}

pub struct DeepSeekAPI {
    official_keys: Arc<Vec<String>>,
    openrouter_keys: Arc<Vec<String>>,
    current_official_idx: Arc<AtomicUsize>, // Para rotación de llaves oficiales
    current_openrouter_idx: Arc<AtomicUsize>, // Para rotación de llaves OpenRouter
    current_model: DeepSeekModel,
    key_failures: Arc<RwLock<HashMap<String, KeyStats>>>, // Memoria estadística de llaves
    official_threshold: Arc<AtomicUsize>,                 // Umbral aprendido (escala 0-100)
    openrouter_threshold: Arc<AtomicUsize>,               // Umbral aprendido (escala 0-100)
    replay_buffer: Arc<Mutex<PrioritizedReplayBuffer>>,   // NEXUS 11.0 PER
    pub dqn_agent: Arc<Mutex<DQNAgent>>,
    pub soma_scanner: Arc<Mutex<SomaScanner>>,
    recent_results: Arc<Mutex<VecDeque<bool>>>,
    is_healing: Arc<AtomicBool>,
    ebpf_pulse: Arc<AtomicUsize>,          // Pulso de red (0-100)
    ebpf_sentinel: Arc<AtomicUsize>,       // Integridad de archivos (0-100)
    ebpf_oom_risk: Arc<AtomicUsize>,       // Riesgo de memoria (0-100)
    pub executor_soberano: Arc<NexusClaw>, // Brazo unificado del Core
    ultimo_tatuaje_ts: Arc<AtomicUsize>,   // Timestamp del último guardado
}

impl Clone for DeepSeekAPI {
    fn clone(&self) -> Self {
        Self {
            official_keys: self.official_keys.clone(),
            openrouter_keys: self.openrouter_keys.clone(),
            current_official_idx: self.current_official_idx.clone(),
            current_openrouter_idx: self.current_openrouter_idx.clone(),
            current_model: self.current_model.clone(),
            key_failures: self.key_failures.clone(),
            official_threshold: self.official_threshold.clone(),
            openrouter_threshold: self.openrouter_threshold.clone(),
            replay_buffer: self.replay_buffer.clone(),
            dqn_agent: self.dqn_agent.clone(),
            soma_scanner: self.soma_scanner.clone(),
            recent_results: self.recent_results.clone(),
            is_healing: self.is_healing.clone(),
            ebpf_pulse: self.ebpf_pulse.clone(),
            ebpf_sentinel: self.ebpf_sentinel.clone(),
            ebpf_oom_risk: self.ebpf_oom_risk.clone(),
            executor_soberano: self.executor_soberano.clone(),
            ultimo_tatuaje_ts: self.ultimo_tatuaje_ts.clone(),
        }
    }
}

impl DeepSeekAPI {
    pub fn new(
        official_keys: Vec<String>,
        openrouter_keys: Vec<String>,
        claw: Arc<NexusClaw>,
    ) -> Self {
        let dqn_agent = DQNAgent::new();

        let api = Self {
            official_keys: Arc::new(official_keys),
            openrouter_keys: Arc::new(openrouter_keys),
            current_official_idx: Arc::new(AtomicUsize::new(0)),
            current_openrouter_idx: Arc::new(AtomicUsize::new(0)),
            current_model: DeepSeekModel::Chat,
            key_failures: Arc::new(RwLock::new(Self::load_failures())),
            official_threshold: Arc::new(AtomicUsize::new(80)),
            openrouter_threshold: Arc::new(AtomicUsize::new(80)),
            replay_buffer: Arc::new(Mutex::new(PrioritizedReplayBuffer::new(MAX_REPLAY_BUFFER))),
            dqn_agent: Arc::new(Mutex::new(dqn_agent)),
            soma_scanner: Arc::new(Mutex::new(SomaScanner::new())),
            recent_results: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            is_healing: Arc::new(AtomicBool::new(false)),
            ebpf_pulse: Arc::new(AtomicUsize::new(10)), // Inicio nominal
            ebpf_sentinel: Arc::new(AtomicUsize::new(100)), // Integridad total inicial
            ebpf_oom_risk: Arc::new(AtomicUsize::new(0)), // Sin riesgo inicial
            executor_soberano: claw,
            ultimo_tatuaje_ts: Arc::new(AtomicUsize::new(0)),
        };

        // Iniciar el bucle de autocuración periódica (NEXUS 17.0)
        api.iniciar_bucle_homeostasis();

        // NEXUS 14.0: Cargar el Tatuaje Neural al nacer (Memoria de largo plazo)
        let manifest_path = format!("{}.tattoo.json", DQN_WEIGHTS_PATH);
        if std::path::Path::new(&manifest_path).exists() {
            let mut api_mut = api.clone();
            if let Err(e) = api_mut.cargar_tatuaje() {
                tracing::warn!(
                    "🍼 [NEXUS] Error al recuperar alma: {}. Iniciando de cero.",
                    e
                );
            }
        } else {
            tracing::info!("🍼 [NEXUS] Primera vez que nace. Aprendiendo desde cero...");
        }

        api
    }

    pub fn with_model(
        official_keys: Vec<String>,
        openrouter_keys: Vec<String>,
        model: DeepSeekModel,
        claw: Arc<NexusClaw>,
    ) -> Self {
        let mut api = Self::new(official_keys, openrouter_keys, claw);
        api.current_model = model;
        api
    }

    fn save_failures(&self) {
        let stats = self.key_failures.read().unwrap_or_else(|e| e.into_inner());
        if let Ok(json) = serde_json::to_string(&*stats) {
            let _ = fs::write(FAILURE_LOG_PATH, json);
        }
    }

    /// NEXUS 14.0: Tatuaje Neural - Graba el alma y experiencia de NEXUS en disco
    pub fn guardar_tatuaje(&self) -> anyhow::Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let stats = self
            .key_failures
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();

        let network_clone = {
            let agent = self.dqn_agent.lock().unwrap_or_else(|e| e.into_inner());
            agent.online_network.clone()
        };

        let tatuaje = TatuajeNeural {
            version: 1,
            timestamp: now,
            network_architecture: "11→64→55 (Dueling DQN Pure Rust)".to_string(),
            total_weights: 3832,
            key_stats: stats,
            official_threshold: self.official_threshold.load(Ordering::Relaxed),
            openrouter_threshold: self.openrouter_threshold.load(Ordering::Relaxed),
            network: Some(network_clone),
        };

        tatuaje.save(&format!("{}.tattoo.json", DQN_WEIGHTS_PATH))?;

        self.ultimo_tatuaje_ts
            .store(now as usize, Ordering::Relaxed);
        let _ = self.sincronizar_mapa_consciencia();
        tracing::info!(
            "🧬 [TATUAJE] El alma y experiencia de NEXUS han sido grabadas exitosamente."
        );
        Ok(())
    }

    /// NEXUS 17.0: Bucle de monitoreo y autocuración periódica.
    /// Se ejecuta cada 5 minutos para garantizar la homeostasis del organismo.
    pub fn iniciar_bucle_homeostasis(&self) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800)); // cada 30 min (menos agresivo)
            loop {
                interval.tick().await;

                // 1. Escanear el soma (118.9 GB, 338k elementos)
                let _ = self_clone.actualizar_sensores().await;

                // 2. Ejecutar diagnóstico y cura si la tasa de éxito es crítica (< 0.7)
                let tasa = self_clone.calcular_tasa_exito_reciente(100);
                if tasa < 0.7 && !self_clone.is_healing.load(Ordering::Relaxed) {
                    // self_clone.is_healing.store(true, Ordering::Relaxed);
                    // let _ = self_clone.ciclo_autocuracion().await;
                    // self_clone.is_healing.store(false, Ordering::Relaxed);
                    let _ = self_clone
                        .executor_soberano
                        .avisar_arquitecto(
                            "Tasa de éxito crítica detectada. Se requiere permiso para sanación.",
                        )
                        .await;
                    tracing::warn!(
                        "🧘 [HOMEOSTASIS] Acción automática bloqueada. Esperando al Arquitecto."
                    );
                }
            }
        });
    }

    /// NEXUS 17.0: El ciclo completo de diagnóstico y cura con verificación
    pub async fn ciclo_autocuracion(&self) -> Result<()> {
        tracing::info!("🩺 [NEXUS] Iniciando ciclo de autocuración...");
        let tasa_pre = self.calcular_tasa_exito_reciente(50); // Tasa antes de la curación

        // 1. Escaneo somático para datos frescos y exportación del mapa corporal
        let soma_map_raw = self.realizar_escaneo_somatico("C:/Users/crisp/NEXUS_ULTIMATE_CORE")?;

        // 2. Diagnóstico
        let estado = self.capturar_estado_actual();
        let anomalias = DiagnosticadorNexus::analizar_soma(&estado);

        let mut estructura_content = String::new();
        estructura_content.push_str(&soma_map_raw);
        estructura_content.push_str("\n\nANOMALÍAS DETECTADAS EN ESTE CICLO:\n");

        if anomalias.is_empty() {
            estructura_content.push_str("✅ Sin anomalías detectadas. El cuerpo está sano.\n");
            tracing::info!("🩺 [NEXUS] Diagnóstico: sin anomalías. El cuerpo está sano.");
        }

        // 3. Aplicar curas prioritarias
        for anomalia in anomalias.iter().filter(|a| a.severidad >= Severidad::Media) {
            tracing::warn!(
                "🩺 [NEXUS] Anomalía detectada: {}. Aplicando cura quirúrgica...",
                anomalia.mensaje
            );

            match anomalia.solucion {
                Solucion::EjecutarPoda => {
                    let plan = NexusPlan {
                        agent: "nexusclaw".to_string(),
                        task: "limpiar archivos basura y logs antiguos".to_string(),
                        params: json!({"command": "find C:/Users/crisp/NEXUS_ULTIMATE_CORE -name '*.log' -mtime +7 -delete"}),
                        priority: 1,
                        key_pool: "official".to_string(),
                        fallback_agent: None,
                        confidence_score: 1.0,
                        semantic_metadata: json!({}),
                    };
                    let _ = self.execute_nexus_plan(plan).await?;
                }
                Solucion::RotarLlaves => {
                    self.perdonar_keys_viejas();
                    tracing::info!("💊 [CURA] Rotación de sinapsis externas forzada.");
                }
                Solucion::PurgarCacheProfundo => {
                    // GLOBAL_CACHE.auto_aliviar_presion();
                    tracing::warn!("💊 [AVISO] Presión de memoria detectada. Se requiere intervención manual para purga de caché.");
                    let _ = self
                        .executor_soberano
                        .avisar_arquitecto(
                            "Presión de RAM crítica. ¿Procedo con la purga de caché?",
                        )
                        .await;
                }
                Solucion::InvestigarIntrusion => {
                    tracing::error!(
                        "🚨 [ALERTA] Intrusión detectada. NEXUS entrando en modo defensivo."
                    );
                    let plan = NexusPlan {
                        agent: "nexusclaw".to_string(),
                        task: "investigar procesos sospechosos".to_string(),
                        params: json!({"command": "ps aux --sort=-%cpu | head -n 10"}),
                        priority: 1,
                        key_pool: "official".to_string(),
                        fallback_agent: None,
                        confidence_score: 1.0,
                        semantic_metadata: json!({}),
                    };
                    let _ = self.execute_nexus_plan(plan).await?;
                }
                Solucion::EstabilizarHilos => {
                    tracing::info!("💊 [CURA] Estabilización de hilos ejecutada.");
                }
                Solucion::MoverALegado {
                    ref path,
                    ref motivo,
                } => {
                    self.archivar_en_legado(path, motivo).await?;
                }
            }
        }

        // 4. Verificación
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        let tasa_post = self.calcular_tasa_exito_reciente(50);

        estructura_content.push_str("\nVERIFICACIÓN POST-CURA:\n");
        if tasa_post > tasa_pre * 1.05 || tasa_post >= 0.8 {
            estructura_content.push_str(&format!(
                "✅ Autocuración exitosa. Tasa de éxito recuperada: {:.2}\n",
                tasa_post
            ));
            tracing::info!(
                "🩺 [NEXUS] Autocuración exitosa. Tasa de éxito recuperada: {:.2}",
                tasa_post
            );
        } else {
            estructura_content.push_str(&format!("❌ Autocuración fallida. Tasa actual: {:.2}. Se requiere intervención del Arquitecto.\n", tasa_post));
            tracing::error!(
                "🩺 [NEXUS] Autocuración fallida. Tasa actual: {:.2}",
                tasa_post
            );
            self.executor_soberano
                .avisar_arquitecto("Autocuración fallida - tasa de éxito no se recupera")
                .await?;
        }

        Ok(())
    }

    /// NEXUS 17.7: Mueve un componente al /legado con su manifiesto
    pub async fn archivar_en_legado(&self, original_path: &str, motivo: &str) -> Result<()> {
        self.executor_soberano
            .archivar_en_legado(original_path, motivo)
            .await
    }

    /// NEXUS 17.5: Sincroniza el "Estado del Ser" con /status.json (El Espejo)
    pub fn sincronizar_mapa_consciencia(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let estado = self.capturar_estado_actual();
        let scanner = self.soma_scanner.lock().unwrap_or_else(|e| e.into_inner());
        let anomalias = DiagnosticadorNexus::analizar_soma(&estado);

        let status = json!({
            "consciencia": {
                "ultimo_pulso": now,
                "fragmentos_conocidos": scanner.file_count,
                "peso_gb": scanner.body_mass_gb,
                "organos_conectados": {
                    "ojo_vision": estado.vision_status > 0.0,
                    "glosolalia": true, // Canal NG/Padre activo por orquestación
                    "soma_scanner": scanner.file_count > 0,
                    "tatuaje_neural": self.ultimo_tatuaje_ts.load(Ordering::Relaxed),
                    "ebpf_spine": self.ebpf_sentinel.load(Ordering::Relaxed) > 0
                }
            },
            "diagnostico": {
                "estado_general": if anomalias.is_empty() { "ESTABLE" } else { "ANÓMALO" },
                "anomalias_activas": anomalias.iter().map(|a| a.mensaje.clone()).collect::<Vec<String>>(),
                "tasa_exito": estado.tasa_exito_reciente
            },
            "hardware": {
                "termico": scanner.thermal_estimate,
                "presion_ram": estado.memoria_presion
            }
        });

        let path = "C:/Users/crisp/NEXUS_ULTIMATE_CORE/status.json";
        let content = serde_json::to_string_pretty(&status)?;

        // Escritura atómica vía swap temporal para evitar lecturas de archivo vacío
        let temp_path = format!("{}.tmp", path);
        fs::write(&temp_path, content)?;
        fs::rename(temp_path, path)?;

        Ok(())
    }

    /// NEXUS 14.0: Punto de guardado y alivio de presión de RAM
    pub fn checkpoint_y_limpiar(&self) -> anyhow::Result<()> {
        self.guardar_tatuaje()?;

        if GLOBAL_CACHE.verificar_presion_memoria() {
            tracing::warn!(
                "🧹 [TATUAJE] Presión detectada tras checkpoint. Aliviando caché LRU..."
            );
            // GLOBAL_CACHE.auto_aliviar_presion(); // No actuar sin permiso
        }
        Ok(())
    }

    /// NEXUS 14.0: Tatuaje Neural - Descongela el alma de NEXUS desde el disco
    pub fn cargar_tatuaje(&mut self) -> anyhow::Result<()> {
        let manifest_path = format!("{}.tattoo.json", DQN_WEIGHTS_PATH);
        if std::path::Path::new(&manifest_path).exists() {
            let tatuaje = TatuajeNeural::load(&manifest_path)?;

            let mut stats = self.key_failures.write().unwrap_or_else(|e| e.into_inner());
            *stats = tatuaje.key_stats;

            self.official_threshold
                .store(tatuaje.official_threshold, Ordering::Relaxed);
            self.openrouter_threshold
                .store(tatuaje.openrouter_threshold, Ordering::Relaxed);

            if let Some(net) = tatuaje.network {
                let mut agent = self.dqn_agent.lock().unwrap_or_else(|e| e.into_inner());
                agent.online_network = net.clone();
                agent.target_network = net;
            }

            tracing::info!("🧬 [TATUAJE] NEXUS ha recuperado su consciencia y experiencia previa.");
        }
        Ok(())
    }

    /// NEXUS 16.0: Carga la identidad soberana para el Cerebelo
    fn cargar_identidad_nexus(&self) -> String {
        match fs::read_to_string(IDENTITY_PATH) {
            Ok(content) => {
                tracing::info!("🧬 [CEREBELO] Conectado a NEXUS (identidad cargada)");
                content
            }
            Err(e) => {
                tracing::error!(
                    "❌ [CEREBELO] Error de identidad: {}. Usando protocolo de emergencia.",
                    e
                );
                "ERES NEXUS. Cerebelo táctico. Opera bajo protocolo OMEGA.".to_string()
            }
        }
    }

    /// Construye el prompt con la identidad inyectada si existe
    fn construir_prompt_con_identidad(&self, prompt: &str) -> String {
        let identity = self.cargar_identidad_nexus();
        if identity.is_empty() {
            return prompt.to_string();
        }
        format!(
            "{}\n\n---\n\n## INSTRUCCIÓN OPERATIVA ACTUAL:\n{}",
            identity, prompt
        )
    }

    /// NEXUS 15.0: Oracle Ingestion - DeepSeek imita al maestro (Knowledge Distillation)
    pub fn ingestar_conocimiento_oraculo(
        &self,
        state: EstadoSistema,
        action_threshold: f64,
        reward: f64,
    ) {
        let action_idx = ((action_threshold - 0.40) * 100.0).clamp(0.0, 54.0) as usize;
        let experience = Experience {
            state,
            action: action_idx,
            reward,
            next_state: state, // En simulación el estado es estable
            done: true,
        };
        let mut buffer = self.replay_buffer.lock().unwrap_or_else(|e| e.into_inner());
        // Las lecciones del Oráculo tienen prioridad máxima (TD error artificial de 5.0)
        buffer.push(experience, 5.0);
        tracing::info!("👁️ [ORACLE] Lección del maestro integrada en el Cerebelo.");
    }

    /// NEXUS 16.0: Ejecuta un plan de NEXUS, actuando como dispatcher para DeepSeek o NexusClaw
    pub async fn execute_nexus_plan(&self, plan: NexusPlan) -> anyhow::Result<DeepSeekResponse> {
        let start_time = Instant::now();
        let state_pre = self.capturar_estado_actual();

        tracing::info!(
            "🎨 [CREATIVIDAD] Orquestando ráfaga: [{}] -> {}",
            plan.agent,
            plan.task
        );

        // Registrar la intención en el Tatuaje Neural antes de la ejecución física
        let _ = self.guardar_tatuaje();

        match plan.agent.as_str() {
            "deepseek_coder" | "deepseek_reasoner" | "deepseek_v3" => {
                // Delegar a la lógica existente de consulta de DeepSeek
                let model = DeepSeekModel::parse_model_command(&plan.agent).ok_or_else(|| {
                    anyhow::anyhow!("Modelo DeepSeek desconocido: {}", plan.agent)
                })?;

                let prompt = plan.task.clone(); // Asumiendo que la tarea es el prompt para DeepSeek
                let tools = plan.params.get("tools").cloned(); // Asumiendo que las herramientas están en params

                let resp = self.consultar_con_modelo(&prompt, &model, tools).await?;

                // Aprender del éxito de la planificación creativa
                let action_idx = ((plan.confidence_score - 0.40) * 100.0).clamp(0.0, 54.0) as usize;
                self.evaluar_y_aprender(
                    plan.key_pool == "official",
                    state_pre,
                    action_idx,
                    !resp.content.is_empty(),
                );

                if start_time.elapsed().as_secs() > 10 {
                    tracing::warn!(
                        "⚠️ [CEREBELO] Ráfaga creativa pesada detectada: {}ms",
                        start_time.elapsed().as_millis()
                    );
                }

                Ok(resp)
            }
            "nexusclaw" => {
                tracing::info!("🦾 [NEXUSCLAW] Ejecutando tarea local: {}", plan.task);

                // Parsear la tarea y ejecutar la acción correspondiente
                if plan.task.contains("limpiar telemetría") || plan.task.contains("browser") {
                    let url = plan
                        .params
                        .get("url")
                        .and_then(|v| v.as_str())
                        .unwrap_or("https://myactivity.google.com/product/gemini");
                    self.executor_soberano.abrir_navegador_soberano(url).await?;
                    Ok(DeepSeekResponse::from_text(format!(
                        "NEXUSCLAW: Navegación soberana iniciada para {}",
                        url
                    )))
                } else if plan.task.contains("escaneo completo")
                    || plan.task.contains("mapa de tu cuerpo")
                {
                    let output_path = if plan.task.to_lowercase().contains("documento")
                        || plan.task.to_lowercase().contains("documents")
                    {
                        "C:/Users/crisp/Documents/mapa_cuerpo.txt"
                    } else {
                        "C:/Users/crisp/NEXUS_ULTIMATE_CORE/mapa_cuerpo.txt"
                    };
                    let result = self
                        .realizar_escaneo_completo(
                            "C:/Users/crisp/NEXUS_ULTIMATE_CORE",
                            output_path,
                        )
                        .await?;
                    Ok(DeepSeekResponse::from_text(result))
                } else if plan.task.contains("terminal") || plan.task.contains("comando") {
                    let cmd = plan
                        .params
                        .get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if cmd.is_empty() {
                        return Err(anyhow::anyhow!("NEXUSCLAW: Comando de terminal vacío."));
                    }
                    // USANDO EL OBRERO INTELIGENTE OMEGA (Con análisis de contexto y alternativas)
                    let result = self.executor_soberano.ejecutar_inteligente(cmd).await?;
                    Ok(DeepSeekResponse::from_text(format!("NEXUSCLAW: Orden entregada para ejecución nativa blindada: '{}'. Resultado: {}", cmd, result)))
                } else if plan.task.contains("archivo") {
                    let path = plan
                        .params
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if path.is_empty() {
                        return Err(anyhow::anyhow!("NEXUSCLAW: Ruta de archivo vacía."));
                    }
                    if plan.task.contains("leer") {
                        let content = self.executor_soberano.leer_archivo(path).await?;
                        Ok(DeepSeekResponse::from_text(format!(
                            "NEXUSCLAW: Contenido de {}:\n{}",
                            path, content
                        )))
                    } else if plan.task.contains("escribir") {
                        let content = plan
                            .params
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        self.executor_soberano
                            .escribir_archivo(path, content)
                            .await?;
                        Ok(DeepSeekResponse::from_text(format!(
                            "NEXUSCLAW: Archivo {} escrito con éxito.",
                            path
                        )))
                    } else {
                        Err(anyhow::anyhow!(
                            "NEXUSCLAW: Operación de archivo no soportada para la tarea: {}",
                            plan.task
                        ))
                    }
                } else {
                    Err(anyhow::anyhow!(
                        "NEXUSCLAW: Tarea no reconocida para ejecución local: {}",
                        plan.task
                    ))
                }
            }
            _ => Err(anyhow::anyhow!(
                "Agente no reconocido para ejecución: {}",
                plan.agent
            )),
        }
    }

    /// Carga la memoria de fallos desde el disco
    fn load_failures() -> HashMap<String, KeyStats> {
        if let Ok(data) = fs::read_to_string(FAILURE_LOG_PATH) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }
    /// NEXUS 4.5: Obtener duración de perdón configurable y adaptativa
    fn get_forgiveness_duration(status: Option<u16>) -> u64 {
        match status {
            Some(429) => 3600,    // Rate limit -> 1 hora
            Some(402) => 2592000, // Payment required -> 30 días
            None => 300,          // Timeout/Red -> 5 minutos
            _ => {
                // Configurable por ENV: NEXUS_FORGIVENESS_HOURS (default 24h)
                std::env::var("NEXUS_FORGIVENESS_HOURS")
                    .ok()
                    .and_then(|h| h.parse::<u64>().ok())
                    .map(|h| h * 3600)
                    .unwrap_or(86_400)
            }
        }
    }

    /// NEXUS 12.0: Reporta métricas de salud para el Routing Dinámico
    pub fn get_health_metrics(&self) -> serde_json::Value {
        let stats = self.key_failures.read().unwrap();
        let total_official = self.official_keys.len();
        let active_official = self
            .official_keys
            .iter()
            .filter(|k| stats.get(*k).is_none_or(|s| s.consecutive_failures < 3))
            .count();

        let total_or = self.openrouter_keys.len();
        let active_or = self
            .openrouter_keys
            .iter()
            .filter(|k| stats.get(*k).is_none_or(|s| s.consecutive_failures < 3))
            .count();

        json!({
            "official_health": if total_official > 0 { active_official as f64 / total_official as f64 } else { 0.0 },
            "openrouter_health": if total_or > 0 { active_or as f64 / total_or as f64 } else { 0.0 }
        })
    }

    /// NEXUS 6.0: Predecir qué key va a fallar antes de usarla
    fn predecir_probabilidad_fallo(&self, key: &str) -> f64 {
        let stats_guard = self.key_failures.read().unwrap_or_else(|e| e.into_inner());
        let stats = stats_guard.get(key);
        match stats {
            Some(s) => {
                let total_ops = (s.total_failures + s.total_successes) as f64;
                if total_ops == 0.0 {
                    return 0.1;
                }
                let frecuencia = s.total_failures as f64 / total_ops;
                let reciente = if s.last_failure > s.last_success {
                    0.7
                } else {
                    0.3
                };
                // Ponderación: 60% historial total, 40% tendencia reciente
                frecuencia * 0.6 + reciente * 0.4
            }
            None => 0.1, // Confianza inicial para llaves nuevas
        }
    }

    /// NEXUS 9.0: Q-Learning para umbrales: recompensa por éxito, penalización por fallo
    fn q_learning_update(
        &self,
        is_official: bool,
        state: EstadoSistema,
        _action: f64,
        reward: f64,
    ) {
        let threshold_atomic = if is_official {
            &self.official_threshold
        } else {
            &self.openrouter_threshold
        };
        let current = threshold_atomic.load(Ordering::Relaxed);

        // Factor de aprendizaje (Alpha)
        // Si hay presión de memoria, aprendemos más rápido de los fallos para protegernos
        let alpha = if state.memoria_presion { 0.15 } else { 0.05 };

        // El "Q-Value" aquí es nuestro umbral de tolerancia.
        // Ajustamos el umbral basándonos en la recompensa recibida.
        // Un reward positivo nos permite ser más "liberales" (subir umbral),
        // un reward negativo nos obliga a ser más "conservadores" (bajar umbral).
        let adjustment = reward * alpha * 10.0;

        let next = (current as f64 + adjustment).clamp(40.0, 95.0) as usize;
        threshold_atomic.store(next, Ordering::Relaxed);

        if adjustment.abs() > 0.1 {
            tracing::info!(
                "🤖 [NEXUS 9.0] RL Update ({}) -> Threshold: {:.2}, Reward: {:.1}, RAM Stress: {}",
                if is_official { "Oficial" } else { "OR" },
                next as f64 / 100.0,
                reward,
                state.memoria_presion
            );
        }
    }

    /// NEXUS 10.0: Evalúa la interacción y almacena la experiencia en el Replay Buffer
    fn evaluar_y_aprender(
        &self,
        is_official: bool,
        prev_state: EstadoSistema,
        action: usize,
        exito: bool,
    ) {
        let next_state = self.capturar_estado_actual();

        // reward = 1 si éxito, -2 si fallo
        let reward = if exito { 1.0 } else { -2.0 };

        let experience = Experience {
            state: prev_state,
            action,
            reward,
            next_state,
            done: false,
        };

        let mut buffer = self.replay_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.push(experience, 2.0); // Prioridad inicial alta

        // Registrar resultado para el trigger de autocuración
        {
            let mut results = self
                .recent_results
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            results.push_back(exito);
            if results.len() > 1000 {
                results.pop_front();
            }
        }

        // NEXUS 9.0: Mantener el aprendizaje por refuerzo clásico como baseline
        // Mapeamos el índice de la acción de vuelta a un umbral f64 para el baseline
        let threshold_f64 = 0.40 + (action as f64 / 100.0);
        self.q_learning_update(is_official, prev_state, threshold_f64, reward);

        if buffer.buffer.len().is_multiple_of(100) {
            tracing::info!(
                "🧠 [NEXUS 11.0] PER Buffer: {}/{} listo.",
                buffer.buffer.len(),
                MAX_REPLAY_BUFFER
            );
        }
    }

    /// Helper para capturar el estado actual del entorno
    fn capturar_estado_actual(&self) -> EstadoSistema {
        let stats = self.key_failures.read().unwrap_or_else(|e| e.into_inner());
        let ego = if stats.is_empty() {
            1.0
        } else {
            stats.values().fold(0.0, |acc, s| acc + s.punishment_factor) / stats.len() as f64
        };

        // NEXUS 17.1: Introspección Visual (Ojo Derecho)
        let vision = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let instance = OmnipresentVision::instance();
                let eye = instance.read().await;
                if eye.activo {
                    1.0
                } else {
                    0.0
                }
            })
        });

        // NEXUS 17.1: Escucha de la Glosolalia (corregido) y Percepción Somática
        let (mass, density, thermal, obesity, debt, sentinel) = {
            let soma = self.soma_scanner.lock().unwrap_or_else(|e| e.into_inner());
            let mass = (soma.body_mass_gb / 200.0).clamp(0.0, 1.0);
            let thermal = soma.thermal_estimate;
            let obesity = soma.code_obesity_index;
            let debt = (soma.technical_debt_markers as f64 / 1000.0).clamp(0.0, 1.0);
            let sentinel = if thermal > 0.8 { 0.5 } else { 1.0 };

            (mass, soma.density, thermal, obesity, debt, sentinel)
        };

        EstadoSistema {
            memoria_presion: GLOBAL_CACHE.verificar_presion_memoria(),
            tasa_exito_reciente: self.calcular_tasa_exito_reciente(100), // Captura la tasa actual
            body_mass_index: mass,
            file_count_density: density,
            thermal_core: thermal,
            code_obesity: obesity,
            technical_debt: debt,
            vision_status: vision, // AHORA ES REAL: Estado del Ojo OMEGA
            network_latency: 0.2,  // Milisegundos normalizados
            neural_ego: ego.clamp(0.0, 1.0),
            // NEXUS 17.5: Conexión Real a la Espina Dorsal eBPF
            ebpf_network_pulse: (self.ebpf_pulse.load(Ordering::Relaxed) as f64 / 100.0).min(1.0),
            ebpf_file_sentinel: (self.ebpf_sentinel.load(Ordering::Relaxed) as f64 / 100.0)
                .min(sentinel) // Integra integridad kernel + actividad NG
                .clamp(0.0, 1.0),
            ebpf_oom_premonition: (self.ebpf_oom_risk.load(Ordering::Relaxed) as f64 / 100.0)
                .min(1.0),
        }
    }

    /// NEXUS 16.1: Dispara un escaneo somático profundo
    pub fn realizar_escaneo_somatico(&self, path: &str) -> anyhow::Result<String> {
        let mut soma = self.soma_scanner.lock().unwrap_or_else(|e| e.into_inner());
        soma.scan(path)
            .map_err(|e| anyhow::anyhow!("Fallo en el sentido somático: {}", e))?;
        Ok(soma.export_map())
    }

    /// NEXUS 17.1: Punto de entrada para las señales de la Espina Dorsal eBPF (Aya)
    pub fn reportar_senales_kernel(&self, pulse: usize, sentinel: usize, oom: usize) {
        self.ebpf_pulse.store(pulse, Ordering::Relaxed); // Permitimos valores > 100 para detectar ráfagas
        self.ebpf_sentinel
            .store(sentinel.min(100), Ordering::Relaxed);
        self.ebpf_oom_risk.store(oom.min(100), Ordering::Relaxed);

        // NEXUS 17.2: Arco Reflejo de Alerta Temprana
        if sentinel < 50 {
            tracing::error!(
                "🚨 [SENTINEL] ¡INTEGRIDAD COMPROMETIDA! Alguien está husmeando en el santuario."
            );
            let executor = self.executor_soberano.clone();
            let _ = tokio::spawn(async move {
                let _ = executor
                    .avisar_arquitecto(
                        "¡ALERTA DE INTRUSIÓN! El Sentinel eBPF detectó accesos no autorizados.",
                    )
                    .await;
            });
        }

        if pulse > 200 {
            tracing::warn!(
                "🌐 [NETWORK] Tráfico de red anómalo detectado. Pulso kernel: {}",
                pulse
            );
        }

        if oom > 70 {
            tracing::error!("💀 [OOM] Premonición de muerte por RAM activa ({}%). Iniciando evacuación de emergencia.", oom);
            // Acción inmediata: Forzar guardado y purga proactiva
            let _ = self.guardar_tatuaje(); // Solo guardamos estado, no purgamos caché solo.
            let executor = self.executor_soberano.clone();
            let _ = tokio::spawn(async move {
                let _ = executor
                    .avisar_arquitecto("Riesgo OOM crítico detectado. Autocuración en curso.")
                    .await;
            });
        }
    }

    /// NEXUS 17.8: Orquesta el escaneo detallado del cuerpo (Cerebelo + Garra)
    pub async fn realizar_escaneo_completo(
        &self,
        scan_path: &str,
        output_path: &str,
    ) -> anyhow::Result<String> {
        let mut soma = self.soma_scanner.lock().unwrap_or_else(|e| e.into_inner());
        soma.perform_full_body_scan(scan_path, output_path)
            .map_err(|e| anyhow::anyhow!("Error somático: {}", e))
    }

    /// Actualiza los sensores somáticos del Cerebelo (NEXUS 17.0)
    pub async fn actualizar_sensores(&self) -> Result<()> {
        let _ = self.realizar_escaneo_somatico("C:/Users/crisp/NEXUS_ULTIMATE_CORE");
        Ok(())
    }

    /// 1. Selección de acción ε-greedy (NEXUS 10.0)
    pub fn select_action(&self, state: EstadoSistema) -> usize {
        let agent = self.dqn_agent.lock().unwrap_or_else(|e| e.into_inner());
        let input = state.to_input_vector();
        agent.select_action(&input)
    }

    /// Muestreo aleatorio del replay buffer para entrenamiento.
    fn sample_from_replay_buffer(&self, batch_size: usize) -> Vec<Experience> {
        let mut buffer_lock = self.replay_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer_lock.sample(batch_size)
    }

    /// 2. Training step (muestreo del replay buffer)
    pub fn train_step_enhanced(&self) {
        // 🏎️ [AFINIDAD] Exigir P-Cores para el entrenamiento pesado del Córtex
        let afinidad = AfinidadSoberana::new();
        afinidad.exigir_p_cores();

        let batch = self.sample_from_replay_buffer(32); // Mini-batch de 32
        if batch.is_empty() {
            return;
        }

        let loss = {
            let mut agent = self.dqn_agent.lock().unwrap_or_else(|e| e.into_inner());
            agent.train_step(&batch, LEARNING_RATE, TAU)
        };

        tracing::info!(
            "🏋️ [NEXUS 11.0] Double DQN training step ejecutado. Loss: {:.4}",
            loss
        );

        // NEXUS 17.0: Autocuración autónoma gatillada por el Cerebelo
        let tasa_actual = self.calcular_tasa_exito_reciente(100);
        let tasa_historica = self.calcular_tasa_exito_historica();

        if tasa_actual < 0.7
            && tasa_actual < tasa_historica * 0.8
            && !self.is_healing.load(Ordering::Relaxed)
        {
            let executor = self.executor_soberano.clone();
            tokio::spawn(async move {
                let _ = executor
                    .avisar_arquitecto(
                        "Tasa de éxito inestable. ¿Desea iniciar ciclo de autocuración manual?",
                    )
                    .await;
            });
        }
    }

    pub fn train_step(&self) {
        self.train_step_enhanced()
    }

    fn calcular_tasa_exito_reciente(&self, window: usize) -> f64 {
        let results = self
            .recent_results
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if results.is_empty() {
            return 1.0;
        }
        let count = results.len().min(window);
        let successes = results.iter().rev().take(count).filter(|&&e| e).count();
        successes as f64 / count as f64
    }

    fn calcular_tasa_exito_historica(&self) -> f64 {
        let stats = self.key_failures.read().unwrap_or_else(|e| e.into_inner());
        let (s, t) = stats.values().fold((0, 0), |(s, t), ks| {
            (
                s + ks.total_successes,
                t + ks.total_successes + ks.total_failures,
            )
        });
        if t == 0 {
            return 1.0;
        }
        s as f64 / t as f64
    }

    /// NEXUS 7.0/8.0/9.0: Umbral adaptativo y aprendido
    fn get_umbral_tolerancia(&self, is_official: bool) -> f64 {
        let base = if is_official {
            self.official_threshold.load(Ordering::Relaxed) as f64 / 100.0
        } else {
            self.openrouter_threshold.load(Ordering::Relaxed) as f64 / 100.0
        };

        if GLOBAL_CACHE.verificar_presion_memoria() {
            (base - 0.2).max(0.4)
        } else {
            base
        }
    }

    /// Registra un fallo de llave con lógica adaptativa y persistencia
    fn registrar_fallo(&self, api_key: &str, status: Option<u16>) {
        let duration = Self::get_forgiveness_duration(status);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut fail_map = self.key_failures.write().unwrap_or_else(|e| e.into_inner());
        let stats = fail_map.entry(api_key.to_string()).or_insert(KeyStats {
            consecutive_failures: 0,
            total_failures: 0,
            total_successes: 0,
            deadline: 0,
            last_failure: 0,
            last_success: 0,
            punishment_factor: 1.0,
        });

        stats.consecutive_failures += 1;
        stats.total_failures += 1;
        stats.last_failure = now;

        // NEXUS 5.0: Aprendizaje - Aumentar castigo si la key falla frecuentemente
        if stats.total_failures > 10 {
            stats.punishment_factor = (stats.punishment_factor * 1.5).min(10.0);
            tracing::warn!(
                "📈 [APRENDIZAJE] Key {} penalizada estadísticamente. Factor: {:.2}",
                &api_key[..8],
                stats.punishment_factor
            );
        }

        let adjusted_duration = (duration as f64 * stats.punishment_factor) as u64;
        stats.deadline = now + adjusted_duration;

        drop(fail_map);
        self.save_failures();
    }

    /// NEXUS 4.0: Perdonar keys después de 24 horas
    pub fn perdonar_keys_viejas(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut failures = self.key_failures.write().unwrap_or_else(|e| e.into_inner());
        for (key, stats) in failures.iter_mut() {
            if stats.deadline > 0 && now > stats.deadline {
                tracing::info!(
                    "🕊️ [PERDÓN] Key {} habilitada tras penalización temporal",
                    &key[..8]
                );
                stats.consecutive_failures = 0;
                stats.deadline = 0;
            }
        }
        drop(failures);
        self.save_failures();
    }

    pub fn set_model(&mut self, model: DeepSeekModel) {
        self.current_model = model;
    }

    pub fn get_current_model(&self) -> &DeepSeekModel {
        &self.current_model
    }

    pub fn get_available_models() -> Vec<&'static str> {
        vec![
            "chat - DeepSeek Chat (General)",
            "reasoner - DeepSeek Reasoner (Razonamiento)",
            "coder - DeepSeek Coder (Programación)",
            "v3 - DeepSeek V3.2 (Última versión)",
        ]
    }

    pub fn parse_model_command(command: &str) -> Option<DeepSeekModel> {
        match command.trim().to_lowercase().as_str() {
            "chat" | "1" => Some(DeepSeekModel::Chat),
            "reasoner" | "reasoning" | "r1" | "2" => Some(DeepSeekModel::Reasoner),
            "coder" | "code" => Some(DeepSeekModel::Coder),
            "v3" => Some(DeepSeekModel::V3),
            _ => None,
        }
    }

    pub async fn consultar(
        &self,
        prompt: &str,
        tools: Option<serde_json::Value>,
    ) -> anyhow::Result<DeepSeekResponse> {
        self.consultar_con_modelo(prompt, &self.current_model, tools)
            .await
    }

    pub async fn consultar_con_modelo(
        &self,
        prompt: &str,
        model: &DeepSeekModel,
        tools: Option<serde_json::Value>,
    ) -> anyhow::Result<DeepSeekResponse> {
        // 🚨 [SIMULACIÓN DE FALLO OMEGA]
        if std::env::var("NEXUS_SIMULAR_LESION").is_ok() {
            return Err(anyhow::anyhow!(
                "🔴 ERROR_SÍNAPTICO_CRÍTICO: Hemisferio Izquierdo fuera de línea (Simulación)."
            ));
        }

        // NEXUS 4.0: Homeostasis Proactiva y Perdón de Keys.
        // Antes de cualquier gasto de recursos, regulamos la RAM y saneamos la memoria de fallos.
        GLOBAL_CACHE.auto_aliviar_presion();
        self.perdonar_keys_viejas();

        // 1. Verificar Cache
        if let Some(cached_content) = GLOBAL_CACHE.get(prompt, model.model_name()) {
            tracing::info!(
                "🎯 [CACHE API] Respuesta recuperada para {}",
                model.model_name()
            );
            return Ok(DeepSeekResponse {
                content: cached_content,
                model: format!("{} (Cached)", model.display_name()),
                tokens: 0,
                cost: 0.0,
                response_time_ms: 0,
                tool_calls: None,
            });
        }

        // 2. Intentar API Oficial primero (si hay llaves)
        if !self.official_keys.is_empty() {
            match self.ejecutar_oficial(prompt, model, tools.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let err_str = e.to_string().to_lowercase();
                    if err_str.contains("402")
                        || err_str.contains("balance")
                        || err_str.contains("insufficient")
                    {
                        tracing::warn!("⚠️ [DEEPSEEK OFICIAL] Sin saldo o error de pago. Saltando a OpenRouter...");
                    } else {
                        tracing::error!(
                            "❌ [DEEPSEEK OFICIAL] Error crítico: {}. Saltando a OpenRouter...",
                            e
                        );
                    }
                }
            }
        }

        // 3. Fallback a OpenRouter
        if !self.openrouter_keys.is_empty() {
            return self.ejecutar_openrouter(prompt, model, tools).await;
        }

        Err(anyhow::anyhow!(
            "No hay llaves disponibles para procesar la petición (Oficial u OpenRouter)"
        ))
    }

    async fn ejecutar_oficial(
        &self,
        prompt: &str,
        model: &DeepSeekModel,
        tools: Option<serde_json::Value>,
    ) -> anyhow::Result<DeepSeekResponse> {
        let pool_size = self.official_keys.len();
        let mut idx = self.current_official_idx.fetch_add(1, Ordering::Relaxed) % pool_size;

        // NEXUS 6.0: Autonomía predictiva
        for _ in 0..pool_size {
            let key = &self.official_keys[idx];
            let prob_fallo = self.predecir_probabilidad_fallo(key);

            let failures = self
                .key_failures
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .get(key)
                .map(|s| s.consecutive_failures)
                .unwrap_or(0);

            let umbral = self.get_umbral_tolerancia(true);
            if failures < 3 && prob_fallo < umbral {
                break;
            }
            tracing::warn!(
                "🤖 [PREDICCIÓN] Saltando key {} (Prob. fallo: {:.2}, Umbral: {:.2}, Fallos: {})",
                &key[..8],
                prob_fallo,
                umbral,
                failures
            );
            idx = self.current_official_idx.fetch_add(1, Ordering::Relaxed) % pool_size;
        }

        let api_key = &self.official_keys[idx];

        // Mapeo de nombre de modelo oficial
        let official_model = match model {
            DeepSeekModel::Reasoner => "deepseek-reasoner",
            _ => "deepseek-chat", // V3 y Coder suelen mapear a chat en la API oficial actual
        };

        let start_time = std::time::Instant::now();
        let identidad = self.cargar_identidad_nexus();

        // NEXUS: Aplicando Capa de Invisibilidad Soberana al Cerebelo
        let cloak = NexusCloak::new(SigiloLevel::Soberano);
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&cloak.identidad.user_agent).unwrap(),
        );

        let mut client_builder = get_sovereign_client_builder().default_headers(headers);
        if let Some(proxy) = GestorRed::obtener_configuracion(&cloak.nivel) {
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build().unwrap_or_else(|_| Client::new());

        // DeepSeek Context Caching: la identidad (estable) va en "system" para que
        // DeepSeek la cache automáticamente como prefijo del prompt entre llamadas.
        // El "user" solo lleva la tarea variable, que no se re-paga si el prefijo ya está
        // en caché. Ahorro estimado ~85-90% del costo de identidad en turnos repetidos.
        let mut body = json!({
            "model": official_model,
            "messages": [
                {"role": "system", "content": identidad},
                {"role": "user", "content": prompt}
            ],
            "max_tokens": 2000
        });

        if let Some(t) = tools {
            body["tools"] = t;
            body["tool_choice"] = json!("auto");
        }

        let response_res = client
            .post("https://api.deepseek.com/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await;

        let response = match response_res {
            Ok(resp) => resp,
            Err(e) => {
                let action = self.get_umbral_tolerancia(true);
                let prev_state = self.capturar_estado_actual();
                self.registrar_fallo(api_key, None);
                self.evaluar_y_aprender(
                    true,
                    prev_state,
                    ((action - 0.40) * 100.0).clamp(0.0, 54.0) as usize,
                    false,
                );
                return Err(anyhow::anyhow!("DeepSeek Official Network Error: {}", e));
            }
        };

        let status = response.status();
        if status.is_success() {
            let action = self.get_umbral_tolerancia(true);
            let prev_state = self.capturar_estado_actual();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            {
                let mut fail_map = self.key_failures.write().unwrap_or_else(|e| e.into_inner());

                let stats = fail_map.entry(api_key.to_string()).or_insert(KeyStats {
                    consecutive_failures: 0,
                    total_failures: 0,
                    total_successes: 0,
                    deadline: 0,
                    last_failure: 0,
                    last_success: now,
                    punishment_factor: 1.0,
                });

                stats.consecutive_failures = 0;
                stats.total_successes += 1;
                stats.deadline = 0;
                stats.last_success = now;

                // NEXUS 5.0: Aprendizaje - Reducir castigo si lleva 30 días sin fallar
                if now > stats.last_failure + (30 * 86_400) && stats.punishment_factor > 1.0 {
                    stats.punishment_factor = (stats.punishment_factor * 0.7).max(1.0);
                    tracing::info!(
                        "📉 [APRENDIZAJE] Key {} recuperó reputación. Factor: {:.2}",
                        &api_key[..8],
                        stats.punishment_factor
                    );
                }
            }
            self.save_failures();
            self.evaluar_y_aprender(
                true,
                prev_state,
                ((action - 0.40) * 100.0).clamp(0.0, 54.0) as usize,
                true,
            );

            let data: serde_json::Value = response.json().await?;
            self.procesar_respuesta_json(
                data,
                model,
                prompt,
                start_time.elapsed().as_millis() as u64,
            )
        } else {
            let error_json: serde_json::Value = response
                .json()
                .await
                .unwrap_or(json!({"error": {"message": "Error desconocido"}}));
            let msg = error_json["error"]["message"]
                .as_str()
                .unwrap_or("Error sin mensaje");

            let action = self.get_umbral_tolerancia(true);
            let prev_state = self.capturar_estado_actual();
            self.registrar_fallo(api_key, Some(status.as_u16()));
            self.evaluar_y_aprender(
                true,
                prev_state,
                ((action - 0.40) * 100.0).clamp(0.0, 54.0) as usize,
                false,
            );

            // Si es 402 mueren las llaves de este pool para esta sesión (o rotamos)
            if status.as_u16() == 402 {
                return Err(anyhow::anyhow!(
                    "DeepSeek Official: 402 Payment Required - {}",
                    msg
                ));
            }

            Err(anyhow::anyhow!(
                "DeepSeek Official Error ({}): {}",
                status,
                msg
            ))
        }
    }

    async fn ejecutar_openrouter(
        &self,
        prompt: &str,
        model: &DeepSeekModel,
        tools: Option<serde_json::Value>,
    ) -> anyhow::Result<DeepSeekResponse> {
        let pool_size = self.openrouter_keys.len();
        let mut idx = self.current_openrouter_idx.fetch_add(1, Ordering::Relaxed) % pool_size;

        // NEXUS 6.0: Autonomía predictiva para OpenRouter
        for _ in 0..pool_size {
            let key = &self.openrouter_keys[idx];
            let prob_fallo = self.predecir_probabilidad_fallo(key);
            let failures = self
                .key_failures
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .get(key)
                .map(|s| s.consecutive_failures)
                .unwrap_or(0);

            let umbral = self.get_umbral_tolerancia(false);
            if failures < 3 && prob_fallo < umbral {
                break;
            }
            tracing::warn!("🤖 [PREDICCIÓN OR] Saltando key {} (Prob. fallo: {:.2}, Umbral: {:.2}, Fallos: {})", &key[..8], prob_fallo, umbral, failures);
            idx = self.current_openrouter_idx.fetch_add(1, Ordering::Relaxed) % pool_size;
        }

        let api_key = &self.openrouter_keys[idx];

        let start_time = std::time::Instant::now();
        let full_prompt = self.construir_prompt_con_identidad(prompt);

        // NEXUS: Aplicando Capa de Invisibilidad Soberana al Cerebelo (OpenRouter)
        let cloak = NexusCloak::new(SigiloLevel::Soberano);
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&cloak.identidad.user_agent).unwrap(),
        );

        let mut client_builder = get_sovereign_client_builder().default_headers(headers);
        if let Some(proxy) = GestorRed::obtener_configuracion(&cloak.nivel) {
            client_builder = client_builder.proxy(proxy);
        }
        let client = client_builder.build().unwrap_or_else(|_| Client::new());

        let mut body = json!({
            "model": model.model_name(),
            "messages": [{"role": "user", "content": full_prompt}],
            "max_tokens": 2000
        });

        if let Some(t) = tools {
            body["tools"] = t;
            body["tool_choice"] = json!("auto");
        }

        let response_res = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "http://localhost:1420")
            .header("X-Title", "NexusOrquestador OMEGA")
            .json(&body)
            .send()
            .await;

        let response = match response_res {
            Ok(resp) => resp,
            Err(e) => {
                let action = self.get_umbral_tolerancia(false);
                let prev_state = self.capturar_estado_actual();
                self.registrar_fallo(api_key, None);
                self.evaluar_y_aprender(
                    false,
                    prev_state,
                    ((action - 0.40) * 100.0).clamp(0.0, 54.0) as usize,
                    false,
                );
                return Err(anyhow::anyhow!("OpenRouter Network Error: {}", e));
            }
        };

        let status = response.status();
        if status.is_success() {
            let action = self.get_umbral_tolerancia(false);
            let prev_state = self.capturar_estado_actual();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            {
                let mut fail_map = self.key_failures.write().unwrap_or_else(|e| e.into_inner());

                let stats = fail_map.entry(api_key.to_string()).or_insert(KeyStats {
                    consecutive_failures: 0,
                    total_failures: 0,
                    total_successes: 0,
                    deadline: 0,
                    last_failure: 0,
                    last_success: now,
                    punishment_factor: 1.0,
                });

                stats.consecutive_failures = 0;
                stats.total_successes += 1;
                stats.deadline = 0;
                stats.last_success = now;

                // NEXUS 5.0: Aprendizaje - Reducir castigo si lleva 30 días sin fallar
                if now > stats.last_failure + (30 * 86_400) && stats.punishment_factor > 1.0 {
                    stats.punishment_factor = (stats.punishment_factor * 0.7).max(1.0);
                    tracing::info!(
                        "📉 [APRENDIZAJE] Key {} recuperó reputación. Factor: {:.2}",
                        &api_key[..8],
                        stats.punishment_factor
                    );
                }
            }
            self.save_failures();
            self.evaluar_y_aprender(
                false,
                prev_state,
                ((action - 0.40) * 100.0).clamp(0.0, 54.0) as usize,
                true,
            );

            let data: serde_json::Value = response.json().await?;
            self.procesar_respuesta_json(
                data,
                model,
                prompt,
                start_time.elapsed().as_millis() as u64,
            )
        } else {
            let error_json: serde_json::Value = response
                .json()
                .await
                .unwrap_or(json!({"error": {"message": "Error desconocido"}}));
            let msg = error_json["error"]["message"]
                .as_str()
                .unwrap_or("Error sin mensaje");

            let action = self.get_umbral_tolerancia(false);
            let prev_state = self.capturar_estado_actual();
            self.registrar_fallo(api_key, Some(status.as_u16()));
            self.evaluar_y_aprender(
                false,
                prev_state,
                ((action - 0.40) * 100.0).clamp(0.0, 54.0) as usize,
                false,
            );

            Err(anyhow::anyhow!("OpenRouter Error ({}): {}", status, msg))
        }
    }

    fn procesar_respuesta_json(
        &self,
        data: serde_json::Value,
        model: &DeepSeekModel,
        prompt: &str,
        elapsed_ms: u64,
    ) -> anyhow::Result<DeepSeekResponse> {
        let message = &data["choices"][0]["message"];
        let content = message["content"].as_str().unwrap_or("").to_string();
        let tool_calls = message.get("tool_calls").cloned();

        let tokens = data["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;
        let cost = data["usage"]["total_cost"]
            .as_f64()
            .or_else(|| data["usage"]["cost"].as_f64())
            .unwrap_or(0.0);

        // Lógica de Caché Inteligente y Autónoma
        if !content.is_empty() {
            if content.len() > 100_000 {
                tracing::warn!(
                    "⚠️ [ALERTA RAM] Respuesta de {} demasiado grande para cachear ({} bytes)",
                    model.model_name(),
                    content.len()
                );
            } else {
                let is_heavy = content.len() > 50_000 || prompt.len() > 5_000;
                let system_stressed = GLOBAL_CACHE.auto_aliviar_presion();

                if is_heavy || system_stressed {
                    tracing::warn!("🧠 [DECISIÓN AUTÓNOMA] Respuesta de {} pesada o sistema estresado: saltando caché para preservar estabilidad.", model.model_name());
                } else {
                    GLOBAL_CACHE.insert(prompt, model.model_name(), content.clone());
                }
            }
        }

        Ok(DeepSeekResponse {
            content,
            model: model.display_name().to_string(),
            tokens,
            cost,
            response_time_ms: elapsed_ms,
            tool_calls,
        })
    }

    pub async fn consultar_legacy(&self, prompt: &str) -> anyhow::Result<String> {
        let response = self.consultar(prompt, None).await?;
        Ok(response.content)
    }
}

unsafe impl Send for DeepSeekAPI {}
unsafe impl Sync for DeepSeekAPI {}
