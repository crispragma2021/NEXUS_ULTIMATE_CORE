// ==========================================
// CONSTRUCTOR DEL ORQUESTADOR
// ==========================================
// Define la estructura Orquestador y su constructor.
// Separado del orquestador principal para mantener el archivo enfocado.
// ==========================================
use crate::autonomia::despertar::Despertar;
use crate::brain::hippocampus::ArtificialHippocampus;
use crate::brain::GhostVoice;
use crate::cache::semantico::CacheSemantico;
use crate::cerebro::aprendizaje_recursivo::ObservadorRecursivo;
use crate::cerebro::corte_soberana::CorteSoberana;
use crate::cerebro::escuadron::ComandanteEscuadron;
use crate::cerebro::nexo::Nexo;
use crate::cerebro::nexo::VozMCP;
use crate::cerebro::organos::amygdala::Amygdala;
use crate::cerebro::organos::cerebelo::Cerebelo;
use crate::cerebro::organos::cingulo_anterior::CinguloAnterior;
use crate::cerebro::organos::corteza_parietal::CortezaParietal;
use crate::cerebro::organos::corteza_prefrontal::CortezaPrefrontal;
use crate::cerebro::organos::cuerpo_calloso::CuerpoCalloso;
use crate::cerebro::organos::ganglios_basales::GangliosBasales;
use crate::cerebro::organos::glandula_dopamina::GlandulaDopamina;
use crate::cerebro::organos::hemisferio_derecho::HemisferioDerecho;
use crate::cerebro::organos::hemisferio_groq::HemisferioGroq;
use crate::cerebro::organos::hemisferio_izquierdo::HemisferioIzquierdo;
use crate::cerebro::organos::insula::Insula;
use crate::cerebro::organos::intuicion::Intuicion;
use crate::cerebro::organos::lobulo_occipital_estetico::LobuloOccipitalEstetico;
use crate::cerebro::organos::lobulo_temporal::LobuloTemporal;
use crate::cerebro::organos::medula_soberana::MedulaSoberana;
use crate::cerebro::organos::metacognicion::Metacognicion;
use crate::cerebro::organos::narrativa_interna::NarrativaInterna;
use crate::cerebro::organos::talamo::Talamo;
use crate::cerebro::organos::teoria_mente::TeoriaMente;
use crate::cerebro::organos::voluntad_propia::VoluntadPropia;
use crate::cerebro::pensamiento_humano::PensamientoHumanoAcelerado;
use crate::cerebro::razonamiento_r1::RazonadorR1;
use crate::cerebro::synapse::MotorSynapse;
use crate::comms::bus_neuronal::BusNeuronal;
use crate::comms::deteccion_intencion::DeteccionIntencion;
use crate::defensa::kernel_shield::KernelShield;
use crate::defensa::sistema_digestivo::SistemaDigestivo;
use crate::defensa::sistema_homeostasis::SistemaHomeostasis;
use crate::defensa::verificador_realidad::VerificadorRealidad;
use crate::efectores::cookie_claw::CookieClaw;
use crate::efectores::nexus_claw_pro::NexusClawPro as NexusClaw;
use crate::efectores::webclaw_extractor::WebClawExtractor;
use crate::emociones::ocean::Ocean;
use crate::energia::gemini_nativo::GeminiNativoOmega;
use crate::energia::latido_financiero::LatidoFinanciero;
use crate::energia::reactor_nuclear::ReactorNuclear;
use crate::energia::zenith_pool::ZenithPool;
use crate::infra::buscador_omega::BuscadorOmega;
use crate::infra::mcp_gateway::McpGateway;
use crate::memoria::memoria_consulta::MemoriaConsulta;
use crate::memoria::memoria_episodica::MemoriaEpisodica;
use crate::memoria::memoria_semantica::MemoriaSemantica;
use crate::memoria::memory::MenteTripartita;
use crate::memoria::subconsciente::Subconsciente;
use crate::sentidos::anclaje_sensorial::AnclajeSensorial;
use crate::sentidos::nexus_palate::SentidoGusto;
use crate::sentidos::nexus_scent::OlfatoDigital;
use crate::sentidos::os_cowork::OsCoworker;
use crate::sentidos::propiocepcion::Propiocepcion;
use crate::sentidos::vision_grafica::VisionGrafica;
use crate::valores::juicio_soberano::JuicioSoberano;
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

#[allow(dead_code)]
pub struct Orquestador {
    /// Puente MCP hacia el binario nexus_voz (personalidad nativa)
    pub voz_mcp: VozMCP,
    pub zenith: Arc<ZenithPool>,
    /// 🛰️ Bus Neuronal — Comunicación del Escuadrón
    pub bus_neuronal: Arc<BusNeuronal>,
    /// 🪖 Comandante de Escuadrón — Gestión de especialistas
    pub escuadron: Arc<ComandanteEscuadron>,
    /// ⚡ Cache Semántico — Token Cero
    pub cache: Arc<CacheSemantico>,
    pub webclaw: Mutex<Option<WebClawExtractor>>,
    pub nexusclaw_url: String,
    pub dopamina: GlandulaDopamina,
    pub corteza: CortezaPrefrontal,
    pub homeostasis: SistemaHomeostasis,
    pub memoria_semantica: Arc<MemoriaSemantica>,
    pub retrieval_engine: crate::cerebro::organos::retrieval::RetrievalEngine,
    pub ocean: Arc<Ocean>,
    pub juicio: Arc<JuicioSoberano>,
    pub despertar: Despertar,
    pub medula: MedulaSoberana,
    pub memoria_consulta: MemoriaConsulta,
    pub reactor: ReactorNuclear,
    pub propiocepcion: Propiocepcion,
    pub anclaje: AnclajeSensorial,
    pub verificador: VerificadorRealidad,
    pub memoria_grafo: MemoriaEpisodica,
    pub talamo: Talamo,
    pub ganglios: Mutex<GangliosBasales>,
    pub cerebelo: Mutex<Cerebelo>,
    pub cuerpo_calloso: CuerpoCalloso,
    pub lobulo_temporal: LobuloTemporal,
    pub insula: Mutex<Insula>,
    pub cingulo: Mutex<CinguloAnterior>,
    pub deteccion: DeteccionIntencion,
    pub mcp: Mutex<McpGateway>,
    pub buscador: BuscadorOmega,
    pub gemini_nativo: Mutex<Option<GeminiNativoOmega>>,
    pub izquierdo: HemisferioIzquierdo,
    pub derecho: HemisferioDerecho,
    pub groq: HemisferioGroq,
    pub shield: KernelShield,
    pub razonador: RazonadorR1,
    pub ultimo_dopamina: Mutex<f64>,
    pub amygdala: Mutex<Amygdala>,
    pub memoria_unificada: MenteTripartita,
    pub nexus_claw_api: Arc<NexusClaw>,
    pub nexo: Arc<Nexo>,
    pub lobulo_occipital: Option<LobuloOccipitalEstetico>,
    pub metacognicion: Option<Metacognicion>,
    pub intuicion: Option<Intuicion>,
    pub narrativa_interna: Mutex<Option<NarrativaInterna>>,
    pub voluntad_propia: Mutex<Option<VoluntadPropia>>,
    pub teoria_mente: Mutex<Option<TeoriaMente>>,
    pub apego: Mutex<crate::cerebro::organos::apego::Apego>,
    pub pensamiento_humano: Mutex<PensamientoHumanoAcelerado>,
    pub chunker: crate::cerebro::organos::chunker::Chunker,
    pub ingesta: crate::cerebro::organos::ingesta::IngestaPipeline,
    /// ⚖️ CORTE SOBERANA — Sistema de debate y consenso multi-modelo
    pub corte: CorteSoberana,
    /// Subconsciente: memoria inconsciente de fondo
    pub subconsciente: Arc<tokio::sync::Mutex<Subconsciente>>,
    /// Observador Recursivo: auto-observación y auto-ajuste del aprendizaje
    pub observador: Arc<std::sync::Mutex<ObservadorRecursivo>>,
    /// 🧠 Motor Synapse — red semántica base para activación de conceptos (Arc compartido con GOI)
    pub synapse: Arc<std::sync::Mutex<MotorSynapse>>,
    /// 🧠 Generador Orgánico Interno (GOI) — generación de lenguaje por emergencia de nodos
    pub generador: std::sync::Mutex<Option<crate::cerebro::generador::GeneradorInterno>>,
    /// 🧠 Flag para decidir qué pipeline usar: true = GOI, false = API externa
    pub usar_generador_interno: bool,
    /// 🔒 MODO PENTEST LOCAL — Aislamiento total de la nube.
    /// Cuando `true`, TODAS las respuestas y juicios del Orquestador pasan
    /// EXCLUSIVAMENTE por el LLM local (Ollama vía NexusClawPro).
    /// Ningún modelo de nube (Gemini/DeepSeek/OpenRouter/Groq/Vertex/WebClaw)
    /// ni el GOI pueden interferir: cero restricciones ajenas, cero fugas.
    pub aislamiento_local: std::sync::atomic::AtomicBool,
    /// 🧬 Sistema inmune cognitivo — heurística propia, memoria de amenazas, aprendizaje
    pub sistema_inmune: std::sync::Mutex<crate::defensa::sistema_inmune::SistemaInmune>,
    // FASE 1: Extirpado el puente cognitivo por violación de arquitectura
    // ─── 📊 MOTOR MERCADO ─────────────────────────────────────────────────────────
    pub motor_mercado: std::sync::Mutex<crate::cerebro::organos::motor_mercado::MotorMercado>,

    // ─── 🧠 SISTEMA SENSORIAL COMPLETO ──────────────────────────────────────────
    /// 👃 Olfato Digital — detecta anomalías en logs, streams y código
    pub olfato: std::sync::Mutex<OlfatoDigital>,
    /// 👅 Gusto Digital — evalúa calidad de respuestas LLM y código
    pub gusto: SentidoGusto,
    /// 🖥️ OS Coworker — conciencia del contexto del sistema operativo
    pub os_cowork: OsCoworker,
    /// 🧭 Corteza Parietal — integrador multisensorial (vista + tacto + propiocepción)
    pub corteza_parietal: std::sync::Mutex<CortezaParietal>,
    /// 🥥 Sistema Digestivo — pipeline Estómago → Hígado → Colon para filtrar inputs
    pub digestivo: SistemaDigestivo,
    /// 👁️ Visión Gráfica — Análisis multimodal de trading
    pub vision_grafica: VisionGrafica,
    /// 🕵️ OSINT Hub — orquestador de inteligencia de fuentes abiertas
    pub osint_hub: crate::efectores::osint::hub::OsintHub,
    /// 🧠 HIPOCAMPO SOBERANO — Memoria Operativa + Semántica + Ebbinghaus
    pub hippocampus: Arc<ArtificialHippocampus>,
}

impl Orquestador {
    /// Construye una nueva instancia del Orquestador con todos sus órganos cerebrales.
    ///
    /// Inicializa 46 campos:
    /// - Memoria (semántica, episódica, consulta, unificada)
    /// - Órganos (tálamo, ganglios, cerebelo, cuerpo calloso, etc.)
    /// - Energía (Zenith pool, Gemini nativo, hemisferios)
    /// - Defensa (KernelShield, shield)
    /// - Neuroquímica (dopamina)
    /// - Nexo / identidad
    ///
    /// También lanza un bucle background cada 5 minutos para homeóstasis/rotación.
    pub async fn new(hippocampus: Arc<ArtificialHippocampus>) -> Self {
        let zenith = Arc::new(ZenithPool::new());
        let _cookie_claw = CookieClaw::new();
        let webclaw = Mutex::new(WebClawExtractor::new().ok());
        let nexusclaw_url = "http://127.0.0.1:3035".to_string();
        let db_path = crate::infra::paths::resolve_path("data/intelligence.db");
        let sesion_id = uuid::Uuid::new_v4().to_string();
        let memoria_unificada = MenteTripartita::new(&sesion_id)
            .await
            .expect("Error fatal: no se pudo inicializar la Memoria Unificada");

        let memoria_semantica = Arc::new(
            MemoriaSemantica::new("data/nexus_memoria.db")
                .await
                .expect("Error fatal: no se pudo inicializar la Memoria Semántica"),
        );

        let dopamina = GlandulaDopamina::new();
        let corteza = CortezaPrefrontal::new(&db_path)
            .expect("Error fatal: no se pudo inicializar la Corteza");
        let homeostasis = SistemaHomeostasis::new(&db_path)
            .expect("Error fatal: no se pudo inicializar Homeostasis");
        let subconsciente = Arc::new(tokio::sync::Mutex::new(Subconsciente::new()));
        let ocean = Arc::new(
            Ocean::new(
                &db_path,
                memoria_semantica.clone(),
                Some(subconsciente.clone()),
            )
            .expect("Error fatal: no se pudo inicializar Ocean"),
        );
        let juicio = Arc::new(JuicioSoberano::new());
        let despertar = Despertar::new();

        let nexus_claw_api = Arc::new(NexusClaw::new(ocean.clone(), juicio.clone()));
        let medula = MedulaSoberana::new(nexus_claw_api.clone());
        let memoria_consulta =
            MemoriaConsulta::new().expect("Error fatal: no se pudo inicializar MemoriaConsulta");
        let reactor = ReactorNuclear::new();
        let propiocepcion = Propiocepcion::new();
        let anclaje = AnclajeSensorial::new();
        let verificador = VerificadorRealidad::new();

        let memoria_grafo = MemoriaEpisodica::new()
            .expect("Error fatal: no se pudo inicializar MemoriaEpisódica (KùzuDB)");
        let talamo = Talamo::new();
        let ganglios = Mutex::new(GangliosBasales::new());
        let cerebelo = Mutex::new(Cerebelo::solo_habitos());
        let cuerpo_calloso = CuerpoCalloso::new();
        let lobulo_temporal = LobuloTemporal::new();
        let insula = Mutex::new(Insula::solo_autodiagnostico());
        let cingulo = Mutex::new(CinguloAnterior::new());
        let deteccion = DeteccionIntencion::new(memoria_semantica.clone());
        let mcp = Mutex::new(McpGateway::new());
        let buscador = BuscadorOmega::new();

        // 💓 Iniciar Latido Financiero (Bucle de 30 min)
        let pool_latido = Arc::clone(&zenith);
        tokio::spawn(async move {
            LatidoFinanciero::iniciar_bucle(pool_latido).await;
        });

        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        let gemini_nativo = Mutex::new(GeminiNativoOmega::new(&api_key).await.ok());
        let izquierdo = HemisferioIzquierdo::new();
        let derecho = HemisferioDerecho::new(&api_key);
        let groq_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
        let groq = HemisferioGroq::new(&groq_key);
        let shield = KernelShield::new();
        let razonador = RazonadorR1::new();
        let db_path_clone = db_path.clone();

        // Bucle background: homeóstasis cada 5 minutos
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                if let Ok(homeostasis) = SistemaHomeostasis::new(&db_path_clone) {
                    let curados = homeostasis.ciclo_de_curacion();
                    if !curados.is_empty() {
                        info!("🏥 Cicatrices sanadas: {:?}", curados);
                    }
                    if let Err(e) = homeostasis.rotar_y_exportar_sesiones(5) {
                        warn!(
                            "⚠️ Error en rotación y exportación de sesiones en homeostasis: {}",
                            e
                        );
                    }
                    if let Err(e) = homeostasis.consolidar_en_hipocampo() {
                        warn!("⚠️ Error al consolidar hipocampo: {}", e);
                    }
                }
            }
        });

        // 🧠 Inicializar órganos RAG
        let retrieval_engine = crate::cerebro::organos::retrieval::RetrievalEngine::new(
            memoria_semantica.clone(),
            "codebase_knowledge",
        );
        let chunker = crate::cerebro::organos::chunker::Chunker::default();
        let ingesta = crate::cerebro::organos::ingesta::IngestaPipeline::new(
            memoria_semantica.clone(),
            "codebase_knowledge",
        );

        let mut voice = GhostVoice::new();
        let _ = voice.initialize().await;
        let voz = Arc::new(voice);
        let nexo = Arc::new(Nexo::new(voz.clone(), db_path.clone()));

        // 🗣️ Inicializar VozMCP — puente hacia el binario nexus_voz
        let voz_mcp = VozMCP::new();

        // 🧠 Puente Cognitivo — prótesis sináptica con STDP, OCEAN endógeno y fonación V4/Transformer
        // 🧠 Puente Cognitivo (EXTIRPADO - VIOLACIÓN DE ARQUITECTURA)
        // info!("🧠 [PUENTE] Puente Cognitivo inicializado — engine puro listo para STDP y OCEAN endógeno");

        let amygdala = Mutex::new(Amygdala::new());
        let observador = Arc::new(std::sync::Mutex::new(ObservadorRecursivo::new()));

        // 🧠 Inicializar Motor Synapse + GOI
        let mut motor_synapse = MotorSynapse::new();
        // 🔗 Conectar persistencia: restaurar conceptos dinámicos de ejecuciones anteriores
        motor_synapse.set_db_path(db_path.clone());
        if let Err(e) = motor_synapse.cargar_desde_db() {
            warn!("⚠️ [SYNAPSE] No se pudieron cargar conceptos desde DB: {} — continuando sin persistencia", e);
        }
        let synapse = Arc::new(std::sync::Mutex::new(motor_synapse));
        let mut generador_interno = crate::cerebro::generador::GeneradorInterno::new(
            synapse.clone(),
            memoria_semantica.clone(),
            subconsciente.clone(),
        );
        // Asignar puente subconsciente vivo (el mismo que usa MundoInterno)
        generador_interno.puente_subconsciente =
            Some(crate::cerebro::generador::PuenteSubconscienteOcean::new());
        info!("🧠 [GOI] Generador Orgánico Interno inicializado con puente semántico");

        // ─── Sincronizar traumas reales de Ocean al mapa semántico del GOI ───────
        // Extrae impresiones con tono_emocional muy negativo de Ocean y las
        // procesa como perturbaciones en el mapa del PuenteSubconscienteOcean.
        // Esto asegura que traumas previos de NEXUS afecten la generación actual.
        {
            let traumas = ocean.recordar_por_emocion(-1.0, 0.5, 50).await;
            if let Some(puente) = generador_interno.puente_subconsciente.as_mut() {
                if !traumas.is_empty() {
                    let mut sub_guard = subconsciente.lock().await;
                    for impresion in &traumas {
                        // 🩸 EXTRAER tokens de la esencia del trauma y registrarlos
                        // en el mapa semántico del puente para que la fricción
                        // semántica del GOI pueda detectarlos cuando el prompt
                        // contenga palabras clave similares.
                        // Sin esto, procesar_filtrado_subconsciente() itera un
                        // mapa vacío y los traumas NO perturban al GOI.
                        for palabra in impresion.esencia.to_lowercase().split_whitespace() {
                            let limpia: String =
                                palabra.chars().filter(|c| c.is_alphanumeric()).collect();
                            if limpia.len() > 3 {
                                puente.registrar_token(&limpia, impresion.tono_emocional);
                                // Perturbación adicional para saturar el concepto
                                if let Some(nodo) = puente.mapa_semantico.get_mut(&limpia) {
                                    for _ in 0..3 {
                                        nodo.registrar_perturbacion(impresion.tono_emocional);
                                    }
                                }
                            }
                        }
                        puente.procesar_filtrado_subconsciente(impresion, &mut sub_guard);
                    }
                    info!(
                        "🧠 [GOI] {} traumas reales sincronizados desde Ocean al mapa semántico",
                        traumas.len()
                    );
                } else {
                    info!("🧠 [GOI] Ocean sin traumas — mapa semántico listo para aprender");
                }
            }
        }

        // ─── Sentidos nuevos ──────────────────────────────────────────────────
        let olfato = std::sync::Mutex::new(OlfatoDigital::new());
        let gusto = SentidoGusto::new();
        let os_cowork = OsCoworker::new();
        let corteza_parietal = std::sync::Mutex::new(CortezaParietal::new());
        let motor_mercado =
            std::sync::Mutex::new(crate::cerebro::organos::motor_mercado::MotorMercado::new());
        let digestivo = SistemaDigestivo;
        let osint_hub = crate::efectores::osint::hub::OsintHub::new();
        let vision_grafica = VisionGrafica::new(Arc::clone(&zenith));
        let corte = CorteSoberana::new();
        let bus_neuronal = Arc::new(BusNeuronal::new(100));
        let escuadron = Arc::new(ComandanteEscuadron::new(bus_neuronal.clone()));
        let mut cache = CacheSemantico::new().expect("Fallo al inicializar Cache Semántico");
        cache.vincular_ocean(ocean.clone());
        let cache = Arc::new(cache);

        Self {
            corte,
            subconsciente,
            voz_mcp,
            zenith,
            bus_neuronal,
            escuadron,
            cache,
            webclaw,
            nexusclaw_url,
            dopamina,
            corteza,
            homeostasis,
            memoria_semantica,
            ocean,
            juicio,
            despertar,
            medula,
            memoria_consulta,
            reactor,
            propiocepcion,
            anclaje,
            verificador,
            memoria_unificada,
            memoria_grafo,
            talamo,
            ganglios,
            cerebelo,
            cuerpo_calloso,
            lobulo_temporal,
            insula,
            cingulo,
            deteccion,
            mcp,
            buscador,
            gemini_nativo,
            izquierdo,
            derecho,
            groq,
            shield,
            razonador,
            ultimo_dopamina: Mutex::new(0.0),
            amygdala,
            nexus_claw_api,
            nexo,
            lobulo_occipital: Some(LobuloOccipitalEstetico::new()),
            metacognicion: Some(Metacognicion::new()),
            intuicion: Some(Intuicion::new()),
            narrativa_interna: Mutex::new(Some(NarrativaInterna::new())),
            voluntad_propia: Mutex::new(Some(VoluntadPropia::new())),
            teoria_mente: Mutex::new(Some(TeoriaMente::new())),
            apego: Mutex::new(crate::cerebro::organos::apego::Apego::new()),
            pensamiento_humano: Mutex::new(PensamientoHumanoAcelerado::new()),
            retrieval_engine,
            chunker,
            ingesta,
            synapse,
            generador: std::sync::Mutex::new(Some(generador_interno)),
            usar_generador_interno: true,
            aislamiento_local: std::sync::atomic::AtomicBool::new(false),
            sistema_inmune: std::sync::Mutex::new(
                crate::defensa::sistema_inmune::SistemaInmune::new(),
            ),
            observador,
            // puente_cognitivo, // Extirpado por violación de arquitectura

            // ─── Sentidos completos ───────────────────────────────────────────
            olfato,
            gusto,
            os_cowork,
            motor_mercado,
            corteza_parietal,
            digestivo,
            vision_grafica,
            osint_hub,
            hippocampus,
        }
    }
}
