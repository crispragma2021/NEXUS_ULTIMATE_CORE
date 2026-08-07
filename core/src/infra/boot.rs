// ============================================================================
// ⚡ OMEGA-19: Boot Secuencial del Sistema Nervioso Central
// Arranque completo en 7 fases con health checks y modo degradado
// ============================================================================

use std::sync::Arc;
use std::time::Instant;
use sysinfo::System;
use tokio::sync::{Mutex as TokioMutex, RwLock, RwLock as TokioRwLock};
use tracing::{error, info, warn};

use crate::autodiagnostico::probes::{
    probe_api::ProbeApi, probe_bypass_telemetry::ProbeBypassTelemetry,
    probe_filesystem::ProbeFilesystem, probe_frontend::ProbeFrontend, probe_memory::ProbeMemory,
    probe_process::ProbeProcess,
};
use crate::autodiagnostico::sentinel_core::{HealthStatus, SentinelCore};
use crate::infra::puente_ipc::PuenteIpc;

use crate::arsenal::ArsenalSoberano;
use crate::brain::healer::Healer;
use crate::brain::hippocampus::ArtificialHippocampus;
use crate::brain::hypothalamus::Hypothalamus;
use crate::brain::nerve_system::NerveSystem;
use crate::brain::thalamus::Thalamus;
use crate::brain::vision::OmnipresentVision;
use crate::brain::{BrainStack, NeuralManager};
use crate::cerebro::corteza_asociativa::CortezaAsociativa;
use crate::cerebro::motor_sueno::MotorSueno;
use crate::cerebro::nexo::nexo_persona::NexoPersonaModule;
use crate::cerebro::organos::intuicion::Intuicion;
use crate::cerebro::orquestador::Orquestador;
use crate::emociones::limbico::SistemaLimbico;
use crate::emociones::sentimiento::SentimientoSoberano;
use crate::infra::mundo_interno::MundoInterno;
use crate::memoria::persistence::DatabaseManager;
use crate::security_protocol::{ActionGateway, SecurityProtocol};
use crate::valores::afinidad_soberana::AfinidadSoberana;
use crate::valores::juicio_soberano::JuicioSoberano;

// ============================================================================
// 🏷️ Tipos de Estado para cada fase
// ============================================================================

/// Estado individual de una fase del boot
#[derive(Debug, Clone, PartialEq)]
pub enum BootPhaseStatus {
    Pending,
    Running,
    Success,
    Failed(String),
}

impl BootPhaseStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, BootPhaseStatus::Success)
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            BootPhaseStatus::Pending => "⏳",
            BootPhaseStatus::Running => "🔄",
            BootPhaseStatus::Success => "✅",
            BootPhaseStatus::Failed(_) => "❌",
        }
    }
}

/// Reporte completo de las 7 fases del boot
#[derive(Debug, Clone)]
pub struct BootReport {
    pub hardware: BootPhaseStatus,
    pub persistencia: BootPhaseStatus,
    pub cortex: BootPhaseStatus,
    pub organos: BootPhaseStatus,
    pub mundo_interno: BootPhaseStatus,
    pub snc: BootPhaseStatus,
    pub health_check: BootPhaseStatus,
    pub duracion_ms: u64,
    pub modo_degradado: bool,
}

impl BootReport {
    /// Todas las fases completadas exitosamente
    pub fn is_full_success(&self) -> bool {
        self.hardware.is_ok()
            && self.persistencia.is_ok()
            && self.cortex.is_ok()
            && self.organos.is_ok()
            && self.mundo_interno.is_ok()
            && self.snc.is_ok()
            && self.health_check.is_ok()
    }

    /// El sistema puede operar aunque haya fallos no-críticos.
    /// Córtex y Órganos son críticos; el resto puede degradarse.
    pub fn is_operational(&self) -> bool {
        self.cortex.is_ok() && self.organos.is_ok()
    }

    /// Genera resumen textual multilínea del boot
    pub fn summary(&self) -> String {
        let fases: Vec<(&str, &BootPhaseStatus)> = vec![
            ("🔌 HARDWARE", &self.hardware),
            ("💾 PERSISTENCIA", &self.persistencia),
            ("🧠 CÓRTEX", &self.cortex),
            ("🧬 ÓRGANOS", &self.organos),
            ("🌌 MUNDO INTERNO", &self.mundo_interno),
            ("🦾 SNC", &self.snc),
            ("✅ HEALTH CHECK", &self.health_check),
        ];

        let mut lines: Vec<String> = fases
            .into_iter()
            .map(|(nombre, status)| {
                let emoji = status.emoji();
                let detalle = match status {
                    BootPhaseStatus::Failed(msg) => format!(" — {}", msg),
                    _ => String::new(),
                };
                format!("  {} {}{}", emoji, nombre, detalle)
            })
            .collect();

        let estado_gral = if self.is_full_success() {
            "✅ BOOT COMPLETO — Sistema 100% operativo"
        } else if self.is_operational() {
            "⚠️ BOOT DEGRADADO — Sistema funcional con fases no-críticas caídas"
        } else {
            "❌ BOOT FALLIDO — Sistema no operativo"
        };

        lines.push(String::new());
        lines.push(format!("⏱️ Duración: {} ms", self.duracion_ms));
        lines.push(format!("📊 Estado: {}", estado_gral));

        lines.join("\n")
    }
}

// ============================================================================
// 🔥 BootContext — Estructuras inicializadas durante el boot
// ============================================================================

/// Contiene todas las instancias creadas durante el boot
pub struct BootContext {
    pub sistema: Arc<RwLock<System>>,
    pub afinidad: AfinidadSoberana,
    pub db_manager: Option<Arc<DatabaseManager>>,
    pub brain_stack: Option<BrainStack>,
    pub thalamus: Option<Arc<Thalamus>>,
    pub gateway: Option<Arc<ActionGateway>>,
    pub neural: Option<Arc<NeuralManager>>,
    pub orquestador: Option<Arc<Orquestador>>,
    pub hippocampus: Option<Arc<ArtificialHippocampus>>,
    pub snc: Option<Arc<NerveSystem>>,
    pub _handle_mundo: Option<tokio::task::JoinHandle<()>>,
}

// ============================================================================
// 🔥 BootSequencer — Orquestador de arranque en 7 fases
// ============================================================================

/// Ejecuta el boot secuencial completo de NEXUS en 7 fases.
pub struct BootSequencer;

impl BootSequencer {
    /// Punto de entrada único: ejecuta las 7 fases en orden.
    /// Retorna un `BootReport` con el estado de cada fase y un `BootContext`
    /// con las estructuras inicializadas.
    pub async fn run() -> (BootReport, BootContext) {
        info!("🚀 [BOOT] ===== INICIANDO BOOT SECUENCIAL OMEGA-19 =====");
        let inicio = Instant::now();

        let mut ctx = BootContext {
            sistema: Arc::new(RwLock::new(System::new_all())),
            afinidad: AfinidadSoberana::new(),
            db_manager: None,
            brain_stack: None,
            thalamus: None,
            gateway: None,
            neural: None,
            orquestador: None,
            hippocampus: None,
            snc: None,
            _handle_mundo: None,
        };

        // FASE 1: 🔌 HARDWARE
        let hardware = Self::phase_hardware(&ctx).await;

        // FASE 2: 💾 PERSISTENCIA
        let (persistencia, db_manager) = Self::phase_persistencia().await;
        ctx.db_manager = db_manager;

        // FASE 3: 🧠 CÓRTEX — BrainStack unificado
        let (cortex_status, brain_stack, _, _, _) = Self::phase_cortex().await;
        // Extraer componentes del BrainStack para compatibilidad con fases posteriores
        if let Some(ref bs) = brain_stack {
            ctx.thalamus = Some(bs.thalamus.clone());
            ctx.gateway = Some(bs.gateway.clone());
            ctx.neural = Some(bs.neural_manager.clone());
        }
        ctx.brain_stack = brain_stack;

        // ─── CREAR HIPPOCAMPUS (antes de Fase 4 para que phase_organos lo necesite) ───
        let hippocampus = Arc::new(ArtificialHippocampus::new(
            ctx.db_manager.clone(),
            None,
            "/home/soberano/NEXUS_ULTIMATE_CORE/data/memory/vector_memories",
        ));
        ctx.hippocampus = Some(hippocampus.clone());
        info!("🗂️ [BOOT] ArtificialHippocampus: memoria consciente creada");

        // FASE 4: 🧬 ÓRGANOS
        let (organos, orquestador) = Self::phase_organos(hippocampus.clone()).await;
        ctx.orquestador = orquestador;

        // FASE 5: 🌌 MUNDO INTERNO
        let (mundo_interno, handle_mundo) = Self::phase_mundo_interno(&ctx).await;
        ctx._handle_mundo = handle_mundo;

        // FASE 6: 🦾 SISTEMA NERVIOSO
        let (snc, snc_instance) = Self::phase_snc(&ctx).await;
        ctx.snc = snc_instance;

        // FASE 7: ✅ HEALTH CHECK
        let health_check = Self::phase_health_check(&ctx).await;

        let duracion_ms = inicio.elapsed().as_millis() as u64;
        let modo_degradado = matches!(&health_check, BootPhaseStatus::Failed(_));

        let report = BootReport {
            hardware,
            persistencia,
            cortex: cortex_status,
            organos,
            mundo_interno,
            snc,
            health_check,
            duracion_ms,
            modo_degradado,
        };

        // Resumen final
        if report.is_full_success() {
            info!("🚀 [BOOT] Boot completo exitoso en {} ms", duracion_ms);
        } else if report.is_operational() {
            warn!(
                "⚠️ [BOOT] Boot degradado en {} ms — revisar fases fallidas",
                duracion_ms
            );
        } else {
            error!(
                "❌ [BOOT] Boot fallido en {} ms — sistema no operativo",
                duracion_ms
            );
        }

        info!("📋 [BOOT] Resumen:\n{}", report.summary());

        (report, ctx)
    }

    // ─── FASE 1: 🔌 HARDWARE ───────────────────────────────────────────────

    async fn phase_hardware(ctx: &BootContext) -> BootPhaseStatus {
        info!("🔌 [BOOT] Fase 1/7: Inicializando HARDWARE...");
        let inicio = Instant::now();

        // 1.1 Afinidad Soberana — bind a P-cores
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ctx.afinidad.exigir_p_cores();
        })) {
            Ok(()) => info!("✅ [BOOT] AfinidadSoberana: proceso asignado a P-cores"),
            Err(_) => warn!(
                "⚠️ [BOOT] AfinidadSoberana: no se pudo asignar CPU — continuando sin afinidad"
            ),
        }

        // 1.2 CPU info
        {
            let sys = ctx.sistema.read().await;
            info!(
                "🔌 [BOOT] CPUs lógicas: {} | RAM total: {} MiB",
                sys.cpus().len(),
                sys.total_memory() / 1024 / 1024,
            );
        }

        let elapsed = inicio.elapsed().as_millis() as u64;
        info!("✅ [BOOT] Fase 1 (HARDWARE) completada en {} ms", elapsed);
        BootPhaseStatus::Success
    }

    // ─── FASE 2: 💾 PERSISTENCIA ───────────────────────────────────────────

    async fn phase_persistencia() -> (BootPhaseStatus, Option<Arc<DatabaseManager>>) {
        info!("💾 [BOOT] Fase 2/7: Inicializando PERSISTENCIA...");
        let inicio = Instant::now();

        // 2.1 Verificar/crear rutas críticas
        let rutas_criticas = [
            "/home/soberano/NEXUS_ULTIMATE_CORE/data",
            "/home/soberano/NEXUS_ULTIMATE_CORE/data/memory",
        ];
        for ruta in &rutas_criticas {
            let p = std::path::Path::new(ruta);
            if !p.exists() {
                warn!("⚠️ [BOOT] Ruta no existe, creando: {}", ruta);
                if let Err(e) = std::fs::create_dir_all(p) {
                    warn!("⚠️ [BOOT] No se pudo crear ruta {}: {}", ruta, e);
                }
            }
        }

        // 2.2 DatabaseManager (SQLite persistente)
        match DatabaseManager::new("sqlite:nexus_intelligence.db").await {
            Ok(db) => {
                info!("✅ [BOOT] DatabaseManager: SQLite conectado");
                let elapsed = inicio.elapsed().as_millis() as u64;
                info!(
                    "✅ [BOOT] Fase 2 (PERSISTENCIA) completada en {} ms",
                    elapsed
                );
                (BootPhaseStatus::Success, Some(Arc::new(db)))
            }
            Err(e) => {
                warn!("⚠️ [BOOT] DatabaseManager: {} — continuando sin DB", e);
                let _elapsed = inicio.elapsed().as_millis() as u64;
                (BootPhaseStatus::Failed(format!("DB: {}", e)), None)
            }
        }
    }

    // ─── FASE 3: 🧠 CÓRTEX ─────────────────────────────────────────────────

    async fn phase_cortex() -> (
        BootPhaseStatus,
        Option<BrainStack>,
        Option<Arc<Thalamus>>,
        Option<Arc<ActionGateway>>,
        Option<Arc<NeuralManager>>,
    ) {
        info!("🧠 [BOOT] Fase 3/7: Inicializando CÓRTEX COGNITIVO (BrainStack)...");
        let inicio = Instant::now();

        // 3.1 initialize_brain_async — córtex cognitivo completo, retorna BrainStack
        match crate::brain::initialize_brain_async().await {
            Ok(brain_stack) => {
                info!("✅ [BOOT] initialize_brain_async: BrainStack listo");
                let elapsed = inicio.elapsed().as_millis() as u64;
                info!("✅ [BOOT] Fase 3 (CÓRTEX) completada en {} ms", elapsed);
                (
                    BootPhaseStatus::Success,
                    Some(brain_stack),
                    None, // thalamus via brain_stack
                    None, // gateway via brain_stack
                    None, // neural via brain_stack
                )
            }
            Err(e) => {
                warn!("⚠️ [BOOT] initialize_brain_async: error — {}", e);
                let _elapsed = inicio.elapsed().as_millis() as u64;
                (
                    BootPhaseStatus::Failed(format!("Córtex: {}", e)),
                    None,
                    None,
                    None,
                    None,
                )
            }
        }
    }

    // ─── FASE 4: 🧬 ÓRGANOS ────────────────────────────────────────────────

    async fn phase_organos(
        hippocampus: Arc<ArtificialHippocampus>,
    ) -> (BootPhaseStatus, Option<Arc<Orquestador>>) {
        info!("🧬 [BOOT] Fase 4/7: Construyendo ÓRGANOS CEREBRALES (46)...");
        let inicio = Instant::now();

        // Orquestador::new() ahora requiere hippocampus para el circuito de conciencia.
        let orquestador = Orquestador::new(hippocampus).await;
        info!("✅ [BOOT] Orquestador: 46 órganos cerebrales + hipocampo integrados");
        let elapsed = inicio.elapsed().as_millis() as u64;
        info!("✅ [BOOT] Fase 4 (ÓRGANOS) completada en {} ms", elapsed);
        (BootPhaseStatus::Success, Some(Arc::new(orquestador)))
    }

    // ─── FASE 5: 🌌 MUNDO INTERNO ──────────────────────────────────────────

    async fn phase_mundo_interno(
        ctx: &BootContext,
    ) -> (BootPhaseStatus, Option<tokio::task::JoinHandle<()>>) {
        info!("🌌 [BOOT] Fase 5/7: Iniciando MUNDO INTERNO...");
        let inicio = Instant::now();

        // MundoInterno::iniciar_bucle requiere órganos específicos.
        // El Orquestador no expone limbico/motor_sueno/corteza_asociativa
        // como campos públicos con el tipo exacto, así que los creamos aquí.
        //
        // MundoInterno::iniciar_bucle() espera:
        //   limbico: Arc<TokioMutex<SistemaLimbico>>
        //   intuicion: Arc<Mutex<Intuicion>>
        //   motor_sueno: Arc<Mutex<MotorSueno>>
        //   corteza_asociativa: Arc<Mutex<CortezaAsociativa>>
        //   juicio: Arc<Mutex<JuicioSoberano>>
        //
        // SistemaLimbico::new() espera:
        //   ocean: Arc<Ocean>
        //   juicio: Arc<Mutex<JuicioSoberano>>
        //   sentimiento: Arc<Mutex<SentimientoSoberano>>
        //   nexo_persona: Arc<RwLock<NexoPersonaModule>>

        let ocean_arc = match &ctx.orquestador {
            Some(orq) => orq.ocean.clone(),
            None => {
                warn!("⚠️ [BOOT] MundoInterno: Orquestador no disponible — saltando fase");
                let _elapsed = inicio.elapsed().as_millis() as u64;
                return (
                    BootPhaseStatus::Failed("Orquestador no disponible".into()),
                    None,
                );
            }
        };

        // Construir los 5 órganos que MundoInterno necesita
        let juicio_limbico = Arc::new(TokioMutex::new(JuicioSoberano::new()));
        let sentimiento = Arc::new(TokioMutex::new(SentimientoSoberano::new()));
        let db_path_buf =
            std::path::PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE/data/intelligence.db");
        let nexo_persona = Arc::new(TokioRwLock::new(NexoPersonaModule::new(db_path_buf)));

        let limbico_arc = Arc::new(TokioMutex::new(SistemaLimbico::new(
            ocean_arc,
            juicio_limbico,
            sentimiento,
            nexo_persona,
        )));

        let intuicion_arc = Arc::new(std::sync::Mutex::new(Intuicion::new()));

        let motor_sueno_arc = Arc::new(std::sync::Mutex::new(
            MotorSueno::new(23, 7), // dormir 23:00, despertar 7:00
        ));

        let corteza_asociativa_arc = Arc::new(std::sync::Mutex::new(CortezaAsociativa::new()));

        let juicio_arc = Arc::new(std::sync::Mutex::new(JuicioSoberano::new()));

        let subconsciente = ctx
            .orquestador
            .as_ref()
            .map(|orq| orq.subconsciente.clone());
        let observador = ctx.orquestador.as_ref().map(|orq| orq.observador.clone());
        let hippocampus = ctx.hippocampus.clone();
        let handle = MundoInterno::iniciar_bucle(
            limbico_arc,
            intuicion_arc,
            motor_sueno_arc,
            corteza_asociativa_arc,
            juicio_arc,
            subconsciente,
            observador,
            5,           // intervalo de 5 segundos entre ticks
            hippocampus, // hipocampo para consolidación de sueño
            ctx.orquestador.as_ref().unwrap().bus_neuronal.clone(),
        );

        info!("✅ [BOOT] MundoInterno: bucle de pensamiento autónomo iniciado");
        let elapsed = inicio.elapsed().as_millis() as u64;
        info!(
            "✅ [BOOT] Fase 5 (MUNDO INTERNO) completada en {} ms",
            elapsed
        );
        (BootPhaseStatus::Success, Some(handle))
    }

    // ─── FASE 6: 🦾 SISTEMA NERVIOSO (SNC Periférico) ──────────────────────

    async fn phase_snc(ctx: &BootContext) -> (BootPhaseStatus, Option<Arc<NerveSystem>>) {
        info!("🦾 [BOOT] Fase 6/7: Inicializando SISTEMA NERVIOSO PERIFÉRICO...");
        let inicio = Instant::now();

        // Extraer Thalamus (del contexto o crear uno nuevo)
        let thalamus = ctx
            .thalamus
            .clone()
            .unwrap_or_else(|| Arc::new(Thalamus::new()));

        let sys = ctx.sistema.clone();
        let (reflex_tx, _reflex_rx) = tokio::sync::mpsc::channel(1024);

        // DatabaseManager
        let db_manager = match ctx.db_manager.clone() {
            Some(db) => db,
            None => match DatabaseManager::new("sqlite:nexus_intelligence.db").await {
                Ok(db) => {
                    let db = Arc::new(db);
                    warn!("⚠️ [BOOT] DB emergencia creada para NerveSystem");
                    db
                }
                Err(e) => {
                    let _elapsed = inicio.elapsed().as_millis() as u64;
                    return (
                        BootPhaseStatus::Failed(format!("DB NerveSystem: {}", e)),
                        None,
                    );
                }
            },
        };

        let hippocampus = ctx.hippocampus.clone().unwrap_or_else(|| {
            warn!("⚠️ [BOOT] Hippocampus no encontrado en contexto — creando uno nuevo para NerveSystem");
            Arc::new(ArtificialHippocampus::new(
                ctx.db_manager.clone(),
                None,
                "/home/soberano/NEXUS_ULTIMATE_CORE/data/memory/vector_memories",
            ))
        });

        // Gateway
        let gateway = match ctx.gateway.clone() {
            Some(g) => g,
            None => {
                // Crear gateway de emergencia
                match SecurityProtocol::new([0u8; 32]) {
                    Ok(protocol) => Arc::new(ActionGateway::new(Arc::new(protocol))),
                    Err(e) => {
                        let _elapsed = inicio.elapsed().as_millis() as u64;
                        return (
                            BootPhaseStatus::Failed(format!("Gateway emergencia: {}", e)),
                            None,
                        );
                    }
                }
            }
        };

        let neural = ctx
            .neural
            .clone()
            .unwrap_or_else(|| Arc::new(NeuralManager::new()));
        let arsenal = Arc::new(ArsenalSoberano::new());
        let healer = Arc::new(Healer::new(db_manager));
        let hypothalamus = Arc::new(Hypothalamus::new(reflex_tx.clone(), thalamus.clone()));
        let vision = Arc::new(OmnipresentVision::new(Some(reflex_tx), Some(neural)));

        let snc = Arc::new(NerveSystem::new(
            thalamus,
            sys,
            hippocampus,
            gateway,
            arsenal,
            healer,
            hypothalamus,
            vision,
        ));

        info!("✅ [BOOT] NerveSystem: sistema nervioso periférico listo");
        let elapsed = inicio.elapsed().as_millis() as u64;
        info!("✅ [BOOT] Fase 6 (SNC) completada en {} ms", elapsed);
        (BootPhaseStatus::Success, Some(snc))
    }

    // ─── FASE 7: ✅ HEALTH CHECK ───────────────────────────────────────────

    async fn phase_health_check(ctx: &BootContext) -> BootPhaseStatus {
        info!("✅ [BOOT] Fase 7/7: Ejecutando HEALTH CHECK post-arranque...");
        let inicio = Instant::now();

        let mut checks = Vec::new();

        // 7.1 — System info
        {
            let sys = ctx.sistema.read().await;
            let ram_mib = sys.total_memory() / 1024 / 1024;
            if ram_mib > 0 {
                info!("✅ [HEALTH] System: {} MiB RAM total", ram_mib);
                checks.push(("System", true));
            } else {
                warn!("⚠️ [HEALTH] System: no se pudo leer memoria");
                checks.push(("System", false));
            }
        }

        // 7.2 — DatabaseManager
        let db_ok = ctx.db_manager.is_some();
        if db_ok {
            info!("✅ [HEALTH] DatabaseManager: presente");
        } else {
            warn!("⚠️ [HEALTH] DatabaseManager: ausente");
        }
        checks.push(("DatabaseManager", db_ok));

        // 7.3 — Thalamus
        let thalamus_ok = ctx.thalamus.is_some();
        if thalamus_ok {
            info!("✅ [HEALTH] Thalamus: gateway de consciencia activo");
        } else {
            warn!("⚠️ [HEALTH] Thalamus: ausente");
        }
        checks.push(("Thalamus", thalamus_ok));

        // 7.4 — ActionGateway
        let gateway_ok = ctx.gateway.is_some();
        if gateway_ok {
            info!("✅ [HEALTH] ActionGateway: seguridad activa");
        } else {
            warn!("⚠️ [HEALTH] ActionGateway: ausente");
        }
        checks.push(("ActionGateway", gateway_ok));

        // 7.5 — Orquestador (CRÍTICO)
        let orq_ok = ctx.orquestador.is_some();
        if orq_ok {
            info!("✅ [HEALTH] Orquestador: 46 órganos cerebrales operativos");
        } else {
            error!("❌ [HEALTH] Orquestador: AUSENTE — sistema en modo degradado severo");
        }
        checks.push(("Orquestador", orq_ok));

        // 7.6 — NerveSystem
        let snc_ok = ctx.snc.is_some();
        if snc_ok {
            info!("✅ [HEALTH] NerveSystem: SNC periférico operativo");
        } else {
            warn!("⚠️ [HEALTH] NerveSystem: ausente");
        }
        checks.push(("NerveSystem", snc_ok));

        // 7.7 — Hippocampus (Memoria Consciente)
        let hip_ok = ctx.hippocampus.is_some();
        if hip_ok {
            info!("✅ [HEALTH] Hippocampus: memoria consciente operativa");
        } else {
            warn!("⚠️ [HEALTH] Hippocampus: ausente");
        }
        checks.push(("Hippocampus", hip_ok));

        // 7.8 — MundoInterno
        let mundo_ok = ctx._handle_mundo.is_some();
        if mundo_ok {
            info!("✅ [HEALTH] MundoInterno: bucle de pensamiento autónomo activo");
        } else {
            warn!("⚠️ [HEALTH] MundoInterno: ausente");
        }
        checks.push(("MundoInterno", mundo_ok));

        // 7.6 — SentinelCore (NUEVO)
        let mut sentinel = SentinelCore::new();
        sentinel.registrar_probe(Box::new(ProbeApi::new()));
        sentinel.registrar_probe(Box::new(ProbeFrontend::new()));
        sentinel.registrar_probe(Box::new(ProbeProcess::new()));
        sentinel.registrar_probe(Box::new(ProbeFilesystem::new()));
        sentinel.registrar_probe(Box::new(ProbeMemory::new()));
        sentinel.registrar_probe(Box::new(ProbeBypassTelemetry::new()));
        let report = sentinel.run_full_diagnostic().await;
        info!(
            "🩺 [HEALTH] SentinelCore score: {:.2} — {}",
            report.score_global, report.resumen
        );
        let total = checks.len();
        let passed = checks.iter().filter(|(_, ok)| *ok).count();
        let elapsed = inicio.elapsed().as_millis() as u64;

        if passed == total && report.estado == HealthStatus::Healthy {
            info!(
                "✅ [BOOT] Fase 7 (HEALTH CHECK) — {}/{} checks OK en {} ms",
                passed, total, elapsed
            );

            // Inicializar e iniciar el Puente IPC nativo en segundo plano
            let puente = PuenteIpc::new("/tmp/nexus_trader.sock");
            if let Err(e) = puente.iniciar().await {
                error!("❌ [BOOT] Fallo al iniciar Puente IPC Socket Unix: {}", e);
            }

            BootPhaseStatus::Success
        } else {
            let failed = total - passed;
            warn!(
                "⚠️ [BOOT] Fase 7 (HEALTH CHECK) — {}/{} checks fallidos en {} ms",
                failed, total, elapsed
            );
            BootPhaseStatus::Failed(format!("{}/{} checks fallidos", failed, total))
        }
    }
}
