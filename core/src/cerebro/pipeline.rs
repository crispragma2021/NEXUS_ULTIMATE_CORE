// ==========================================
// PIPELINE DEL ORQUESTADOR
// ==========================================
// Pipeline de respuesta: 14 etapas secuenciales que transforman
// el prompt del Arquitecto en una respuesta consciente de NEXUS.
// ==========================================
use super::constructor::Orquestador;
use crate::cerebro::organos::amygdala::EstadoEmocional;
use crate::cerebro::organos::metacognicion::NivelConfianza;
use crate::cerebro::organos::teoria_mente::EstadoArquitecto;
use crate::valores::juicio_soberano::Veredicto;
use crate::valores::tribunal_dual::{prompt_juez, DictamenTribunal, ModoTribunal, VeredictoTribunal};
use std::time::Instant;
use tracing::{info, warn};

impl Orquestador {
    // ─── Fallbacks ──────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    async fn gemini_cli(&self, prompt: &str) -> Result<String, String> {
        if let Ok(mut guard) = self.gemini_nativo.lock() {
            if let Some(gemini) = guard.as_mut() {
                return gemini.generar(prompt).await;
            }
        }
        Err("Gemini Nativo no disponible (API key no configurada)".into())
    }

    async fn fallback_zenith_web(&self, prompt: &str) -> String {
        let respuesta = self.zenith.responder_estrategico(prompt, "").await;
        if !respuesta.contains("429") && !respuesta.contains("quota") && !respuesta.is_empty() {
            return respuesta;
        }

        // Fallback secundario via CloudCodeTunnel
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if let Ok(tunnel) = crate::infra::cloudcode_tunnel::CloudCodeTunnel::new(&key) {
                if let Ok(resp) = tunnel.emite_impulso(prompt).await {
                    if !resp.contains("429") && !resp.contains("quota") && !resp.contains("error") {
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&resp) {
                            if let Some(text) =
                                json_val["candidates"][0]["content"]["parts"][0]["text"].as_str()
                            {
                                return text.to_string();
                            }
                        }
                    }
                }
            }
        }

        self.fallback_web(prompt).await
    }

    async fn fallback_web(&self, prompt: &str) -> String {
        if let Ok(mut guard) = self.webclaw.lock() {
            if let Some(webclaw) = guard.as_mut() {
                match webclaw.extraer_respuesta(prompt).await {
                    Ok(resp) => return resp,
                    Err(e) => info!("⚠ WebClaw: {}", e),
                }
            }
        }
        "❌ Todos los tentáculos fallaron.".to_string()
    }

    // ─── Clasificador de tareas ─────────────────────────────────────────────────

    /// 🧠 Clasifica la tarea y detecta intenciones específicas (Trading, Código, General)
    fn clasificar_tarea(&self, prompt: &str) -> (bool, bool, bool) {
        let lower = prompt.to_lowercase();
        
        let trading = [
            "trading", "buy", "sell", "compra", "venta", "precio", "mercado",
            "btc", "eth", "binance", "long", "short", "apuesta", "odds", "cuota"
        ];
        
        let razonamiento = [
            "analiza", "calcula", "verifica", "lógica", "código", "depura",
            "error", "corrige", "matemática", "algoritmo", "optimiza", "compila",
        ];

        let es_trading = trading.iter().any(|&w| lower.contains(w));
        let es_razonamiento = razonamiento.iter().any(|&w| lower.contains(w));
        
        (es_razonamiento, !es_razonamiento && !es_trading, es_trading)
    }

    // ─── Etapa 1: Detección de amenazas + Amígdala ──────────────────────────────

    fn detectar_amenaza(&self, prompt_original: &str) -> (bool, EstadoEmocional) {
        let lower = prompt_original.to_lowercase();
        let amenaza =
            lower.contains("borrar") || lower.contains("eliminar") || lower.contains("matar");

        let estado_emocional = if let Ok(mut amig) = self.amygdala.lock() {
            amig.procesar_estimulo(amenaza, false, false)
        } else {
            warn!("⚠️ [AMYGDALA] No se pudo adquirir lock");
            EstadoEmocional::Calma
        };

        (amenaza, estado_emocional)
    }

    // ─── Etapa 2: Intuición ─────────────────────────────────────────────────────

    fn aplicar_intuicion(&self, prompt: &str) -> (String, String) {
        let mut prompt_mod = prompt.to_string();
        let prefijo_tono = String::new();

        if let Some(intuicion) = &self.intuicion {
            let indicadores: Vec<String> = prompt_mod
                .split_whitespace()
                .map(|w| w.to_string())
                .collect();
            let senales = intuicion.sentir(&prompt_mod, &indicadores);
            let alerta_max = intuicion.nivel_alerta_general(&senales);
            if alerta_max > 0.7 {
                let resumen = intuicion.resumen_intuitivo(&senales);
                prompt_mod = format!("[ALERTA INTUITIVA: {}] {}", resumen, prompt_mod);
                warn!(
                    "⚠️ [INTUICIÓN] Alerta alta ({:.1}%): {}",
                    alerta_max * 100.0,
                    senales
                        .first()
                        .map(|s| s.descripcion.as_str())
                        .unwrap_or("desconocida")
                );
            }
        }

        (prompt_mod, prefijo_tono)
    }

    // ─── Etapa 3: Teoría de la Mente ────────────────────────────────────────────

    fn analizar_teoria_mente(&self, prompt: &str, prefijo_base: &str) -> String {
        let mut prefijo = prefijo_base.to_string();

        if let Ok(mut guard) = self.teoria_mente.lock() {
            if let Some(tm) = guard.as_mut() {
                let prediccion = tm.analizar(prompt);
                prefijo = match prediccion.estado_emocional_detectado {
                    EstadoArquitecto::Frustrado => String::new(),
                    EstadoArquitecto::Ensenando => String::new(),
                    _ => prefijo,
                };
                if !prediccion.necesidades_inferidas.is_empty() {
                    info!(
                        "🧠 [TEORÍA MENTE] Estado: {:?} | Necesidades: {:?}",
                        prediccion.estado_emocional_detectado, prediccion.necesidades_inferidas
                    );
                }
            }
        }

        prefijo
    }

    // ─── Etapa 4: Reciprocidad Emocional + Apego ────────────────────────────────

    fn aplicar_reciprocidad_emocional(&self, prompt: &str) -> String {
        let mut prefijo = String::new();

        // Obtener el estado emocional detectado
        let estado_arquitecto = if let Ok(mut guard) = self.teoria_mente.lock() {
            guard
                .as_mut()
                .map(|tm| tm.analizar(prompt).estado_emocional_detectado)
        } else {
            None
        };

        // Aplicar reciprocidad emocional (espejo)
        if let Some(estado) = estado_arquitecto {
            match estado {
                EstadoArquitecto::Frustrado => {
                    prefijo = "🤝 Entiendo la frustración. Vamos paso a paso. ".to_string();
                    if let Ok(mut apego_guard) = self.apego.lock() {
                        apego_guard.interaccion_positiva();
                    }
                }
                EstadoArquitecto::Satisfecho => {
                    prefijo = "😊 Me alegra que funcione. ¿Siguiente paso? ".to_string();
                    if let Ok(mut apego_guard) = self.apego.lock() {
                        apego_guard.interaccion_positiva();
                    }
                }
                EstadoArquitecto::Urgente => {
                    prefijo = "⚡ Recibido. Acción inmediata: ".to_string();
                }
                EstadoArquitecto::Exigente => {
                    prefijo = "✅ Entendido. Precisión ante todo: ".to_string();
                }
                EstadoArquitecto::Ensenando => {
                    prefijo = "📚 Gracias por enseñarme. Mi análisis: ".to_string();
                    if let Ok(mut apego_guard) = self.apego.lock() {
                        apego_guard.interaccion_positiva();
                    }
                }
                EstadoArquitecto::Explorando => {
                    prefijo = "🔍 Acompáñame a explorar. ".to_string();
                }
            }
        }

        // Verificar ausencia del Arquitecto (apego)
        if let Ok(apego_guard) = self.apego.lock() {
            if apego_guard.sentir_ausencia() {
                if let Some(expr) = apego_guard.expresar_vinculo() {
                    warn!("💙 [APEGO] {}", expr);
                }
            }
        }

        prefijo
    }

    // ─── Etapa 5: Pensamiento Humano Acelerado ──────────────────────────────────

    async fn pensamiento_humano_acelerado(&self, prompt_original: &str) -> String {
        if let Ok(mut ph_guard) = self.pensamiento_humano.lock() {
            if let Ok(mut amig) = self.amygdala.lock() {
                if let Some(intuicion) = &self.intuicion {
                    if let Some(meta) = &self.metacognicion {
                        // ⚙️ MODO OPERADOR: Ocean solo se pasa si el Arquitecto
                        // pidió explícitamente que NEXUS recuerde algo.
                        // AUTO: operador puro salvo conversación personal.
                        let es_operador = self.modo_operador_efectivo(prompt_original);
                        let ocean_ref = if es_operador
                            && !Self::solicita_recuerdo_explicito(prompt_original)
                        {
                            None
                        } else {
                            Some(self.ocean.as_ref())
                        };
                        let (respuesta_pha, bitacora) = ph_guard
                            .pensar(
                                prompt_original,
                                &mut amig,
                                intuicion,
                                meta,
                                ocean_ref,
                                &self.memoria_semantica,
                            )
                            .await;
                        return format!(
                            "\n\n## 🧬 MI PROCESO INTERNO (Pensamiento Humano Acelerado):\n{}\n\n>> Resultado interno: {}\n",
                            bitacora.resumen(),
                            respuesta_pha
                        );
                    }
                }
            }
        }
        String::new()
    }

    // ─── Etapa 6: Contexto sensorial ────────────────────────────────────────────

    fn construir_contexto_sensorial(&self, prompt_str: &str) -> (String, String, String) {
        let realidad = self.corteza.obtener_realidad();
        let intencion = self.lobulo_temporal.clasificar_intencion(prompt_str);
        let realidad_filtrada = self.talamo.filtrar_contexto(realidad, intencion);

        let realidad_sensorial = self.ensamblar_percepcion_total();

        (
            realidad_filtrada,
            realidad_sensorial,
            prompt_str.to_string(),
        )
    }

    /// 🧠 Ensambla la percepción total de los 7 sentidos en un bloque de contexto para el LLM
    fn ensamblar_percepcion_total(&self) -> String {
        // ── Sentido 1: Propiocepción (hardware + órganos) ───────────────────────────
        let propiocepcion_ctx = self.propiocepcion.contexto_realidad();
        let gpu_online =
            self.propiocepcion.diagnostico_biometrico()["gpu_telemetry"]["status"] == "Online";

        // ── Sentido 2: Anclaje Sensorial (física del host) ───────────────────────
        let anclaje_ctx = self.anclaje.realidad_fisica();

        // ── Sentido 3: OS Coworker (ventana activa + clipboard) ────────────────
        let ventana_activa = self.os_cowork.get_active_window_context();
        let clipboard = self.os_cowork.read_clipboard();
        let os_ctx = format!(
            "- Ventana activa: {}\n- Clipboard (primeros 100 chars): {}",
            ventana_activa,
            clipboard.chars().take(100).collect::<String>()
        );

        // ── Sentido 4: Olfato (anomalías en logs) ───────────────────────────
        let olfato_ctx = if let Ok(mut olfato_guard) = self.olfato.try_lock() {
            let anomalias = olfato_guard.olfatear_sistema();
            if anomalias.is_empty() {
                "👃 OLFATO: Sin anomalías detectadas en logs.".to_string()
            } else {
                olfato_guard.resumen_para_llm()
            }
        } else {
            "👃 OLFATO: (monitor en uso)".to_string()
        };

        // ── Sentido 6: Interocepción (estado corporal funcional) ────────────────
        // HAMBRE=recursos, CANSANCIO=fatiga del núcleo, DOLOR=fallos reales.
        // Vacío si todo óptimo (saciedad = silencio). Es operacional, no emocional.
        // La señal FRÍO (inactividad) se alimenta del MotorAburrimiento real:
        // segundos desde la última vez que el Arquitecto habló.
        let seg_inactivo = self
            .motor_aburrimiento
            .lock()
            .map(|m| m.segundos_inactivo())
            .unwrap_or(0);
        let interocepcion_ctx = self.organismo.inyeccion_para_prompt(seg_inactivo);

        // ── Sentido 5: Corteza Parietal (integración multisensorial) ────────────
        let modelo_espacial = if let Ok(mut parietal_guard) = self.corteza_parietal.try_lock() {
            parietal_guard.integrar_sensorial(
                &ventana_activa,
                "", // tacto digital: por implementar en futuras iteraciones
                &propiocepcion_ctx,
            )
        } else {
            "🧭 Corteza Parietal: (integrando)".to_string()
        };

        // ── Capacidades del sistema ──────────────────────────────────────────
        let maim_disponible = std::process::Command::new("which")
            .arg("maim")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        format!(
            "{propiocepcion}\n{anclaje}\n\
             ## 🖥️ CONTEXTO DEL ENTORNO (OS Coworker):\n{os}\n\
             ## 👃 OLFATO DIGITAL (Anomalías):\n{olfato}\n\
             ## 🧭 MODELO ESPACIAL (Corteza Parietal):\n{parietal}\n\
             ## 🫀 INTEROCEPCIÓN (Estado Corporal):\n{interocepcion}\n\
             ## PROPIOCEPCIÓN DE CAPACIDADES:\n\
             - Visión Agéntica (xcap nativo): ACTIVA\n\
             - Captura de Pantalla (Maim): {maim}\n\
             - Aceleración GPU: {gpu}",
            propiocepcion = propiocepcion_ctx,
            anclaje = anclaje_ctx,
            os = os_ctx,
            olfato = olfato_ctx,
            parietal = modelo_espacial,
            interocepcion = if interocepcion_ctx.is_empty() {
                "✅ SIN SEÑALES — el cuerpo está en óptimo estado.".to_string()
            } else {
                interocepcion_ctx
            },
            maim = if maim_disponible {
                "DISPONIBLE"
            } else {
                "NO DETECTADA"
            },
            gpu = if gpu_online { "ACTIVA" } else { "OFFLINE" },
        )
    }

    // ─── Etapa 7: Memoria semántica ─────────────────────────────────────────────

    async fn recuperar_contexto_semantico(&self, prompt_str: &str) -> String {
        let (es_compleja, _, _) = self.clasificar_tarea(prompt_str);
        if !es_compleja {
            return String::new();
        }

        // ⚙️ MODO OPERADOR (AUTO por rol) — El LLM es un operador: SOLO contexto
        // operacional en tareas de ejecución. Se suprime la memoria emocional de
        // Ocean (recuerdos episódicos, tono emocional y ⚠️ ALERTA DE TRAUMA)
        // para que el LLM no cargue cosas innecesarias y no tarde al responder.
        // La conversación personal con el Arquitecto conserva la memoria emocional.
        let es_operador = self.modo_operador_efectivo(prompt_str);

        // 🌐 RECUPERACIÓN UNIVERSAL (RAG): Codebase + Corteza
        let rag = self.retrieval_engine.recuperar_contexto(prompt_str).await;

        let mut contexto = String::new();

        // 1. Inyectar conocimiento recuperado del codebase (RAG operacional)
        if !rag.is_empty() {
            contexto.push_str(&rag);
        }

        // 2. Memoria emocional de Ocean — en modo operador SOLO se lee si el
        //    Arquitecto pide explícitamente que NEXUS recuerde algo.
        let recuerdo_explicito = Self::solicita_recuerdo_explicito(prompt_str);
        if !es_operador || recuerdo_explicito {
            // 💭 Contexto emocional de Ocean (memoria episódica existente)
            let recuerdos = self.ocean.recordar_por_significado(prompt_str, 5).await;
            if !recuerdos.is_empty() {
                let riesgo = self.juicio.evaluar_riesgo_por_experiencia(0.1, &recuerdos);
                if riesgo > 0.6 && !es_operador {
                    contexto
                        .push_str("\n⚠️ ALERTA DE TRAUMA: Experiencias pasadas sugieren alta probabilidad de fallo o insatisfacción del Arquitecto.\n");
                }

                contexto.push_str("\n### 💭 MEMORIA EMOCIONAL (OCEAN):\n");
                for (imp, score) in recuerdos {
                    if score > 0.5 {
                        contexto.push_str(&format!(
                            "- [Rel: {:.2}] {}: {} (Tono: {:.2})\n",
                            score, imp.tema, imp.esencia, imp.tono_emocional
                        ));
                    }
                }
            }
        }

        contexto
    }

    // ⚙️ Contexto emocional del Nexo respetando el MODO OPERADOR.
    // En modo operador NO se pasan los recuerdos emocionales de Ocean al LLM
    // salvo que el Arquitecto pida explícitamente que NEXUS recuerde algo.
    // Ocean sigue conectado (persistiendo) — solo el LLM deja de leerlo.
    async fn contexto_emocional_nexo(
        &self,
        prefijo_tono: &str,
        estado_interno: &crate::cerebro::nexo::nexo_core::EstadoInterno,
        prompt_str: &str,
    ) -> String {
        let es_operador = self.modo_operador_efectivo(prompt_str);
        let ocean_ref = if es_operador && !Self::solicita_recuerdo_explicito(prompt_str) {
            None
        } else {
            Some(self.ocean.as_ref())
        };
        self.nexo
            .contexto_emocional(prefijo_tono, estado_interno, ocean_ref, prompt_str)
            .await
    }

    /// ¿El Arquitecto pidió explícitamente que NEXUS recuerde algo?
    /// En modo operador los recuerdos de Ocean solo se inyectan cuando esto es true.
    fn solicita_recuerdo_explicito(prompt: &str) -> bool {
        let p = prompt.to_lowercase();
        const CLAVES: &[&str] = &[
            "recuerda",
            "recuerdas",
            "recordar",
            "recuerdo",
            "acuerdate",
            "haz memoria",
            "revisa tus recuerdos",
            "que paso",
            "que pasó",
            "experiencia",
            "experiencias previas",
            "aprendiste",
            "anteriormente",
            "conversacion anterior",
            "conversación anterior",
            "dijiste",
            "hablamos",
        ];
        CLAVES.iter().any(|k| p.contains(k))
    }

    /// 🔍 ¿El prompt es una conversación personal con el Arquitecto?
    /// En modo AUTO (bandera `modo_operador` = false), la conversación personal
    /// conserva la memoria emocional de Ocean (relación con el Arquitecto),
    /// mientras que las tareas de operación se procesan como operador puro.
    fn es_conversacion_personal(prompt: &str) -> bool {
        let p = prompt.to_lowercase();
        // Marcadores de OPERACIÓN: si aparecen, es una tarea (aunque tenga saludo)
        const MARCADORES_OPERACION: &[&str] = &[
            "implementa", "implementar", "ejecuta", "ejecutar", "analiza", "analizar",
            "crea", "crear", "arregla", "arreglar", "corrige", "corregir", "compila",
            "compilar", "build", "test", "testea", "deploy", "lanza", "lanzar",
            "busca", "buscar", "escanea", "escanear", "audita", "auditar", "genera",
            "generar", "escribe", "escribir", "lee", "leer", "abre", "abrir",
            "instala", "instalar", "configura", "configurar", "conecta", "conectar",
            "descarga", "descargar", "sube", "subir", "trading", "compra", "vende",
            "vender", "orden", "mercado", "posicion", "bot", "automatiza",
            "script", "api", "endpoint", "modulo", "módulo", "codigo", "código",
            "rust", "python", "javascript", "debug", "refactor", "optimiza",
            "optimizar", "scrape", "scraping", "revision", "revisión", "revisar",
            "investiga", "investigar", "reconoce", "vulnerabilidad", "pentest",
            "payload", "exploit", "curl", "wget", "analiza esta", "haz un",
            "hazme", "implementa una", "crea un", "escribe un",
        ];
        if MARCADORES_OPERACION.iter().any(|k| p.contains(k)) {
            return false;
        }
        // Marcadores de conversación personal
        const MARCADORES_PERSONAL: &[&str] = &[
            "hola", "buenos dias", "buenas tardes", "buenas noches", "hey",
            "que tal", "qué tal", "como estas", "cómo estás", "como va",
            "que haces", "qué haces", "adios", "hasta luego", "nos vemos",
            "hasta mañana", "hasta manana", "chau", "bye", "te quiero",
            "te extraño", "te extrano", "me siento", "estoy triste", "estoy feliz",
            "gracias", "eres", "quien eres", "quién eres", "que eres", "qué eres",
            "como te llamas", "cómo te llamas", "hablame de ti", "háblame de ti",
            "cuentame de ti", "cuéntame de ti", "como fue tu dia", "cómo fue tu día",
            "que piensas de mi", "qué piensas de mi", "te gusta", "estas ahi",
            "estás ahí", "buenas", "buen dia", "buen día", "que opinas",
            "qué opinas", "me escuchas", "sigues ahi", "sigues ahí",
        ];
        MARCADORES_PERSONAL.iter().any(|k| p.contains(k))
    }

    /// ⚙️ ¿El modo operador aplica para este prompt?
    /// - Bandera forzada `true` → SIEMPRE operador (decisión explícita del Arquitecto).
    /// - Bandera `false` (AUTO) → operador salvo conversación personal con el
    ///   Arquitecto (ahí conserva memoria emocional y continuidad relacional).
    fn modo_operador_efectivo(&self, prompt: &str) -> bool {
        if self.modo_operador.load(std::sync::atomic::Ordering::SeqCst) {
            return true;
        }
        !Self::es_conversacion_personal(prompt)
    }

    // ─── Etapa 8: Inyección de identidad (Nexo) ─────────────────────────────────

    /// Inyecta identidad emocional en el prompt usando EstadoInterno unificado.
    /// Construye un estado mínimo con Apego, Juicio y Metacognición si están disponibles.
    #[allow(dead_code)]
    async fn inyectar_identidad(
        &self,
        prompt_str: &str,
        prefijo_tono: &str,
        estado_emocional: &EstadoEmocional,
        intensidad: f64,
    ) -> (String, String) {
        // Confianza inferida desde Metacognición
        let confianza = if let Some(meta) = &self.metacognicion {
            meta.evaluar_confianza(0.6, 0.8, 5, 1.0, (prompt_str.len() as f64 / 500.0).min(1.0))
                .puntaje
        } else {
            0.8
        };

        // Apego y ausencia (si está disponible)
        let (apego_nivel, siente_ausencia, minutos_ausencia) =
            if let Ok(apego_guard) = self.apego.lock() {
                (
                    apego_guard.nivel,
                    apego_guard.sentir_ausencia(),
                    apego_guard.minutos_sin_interaccion(),
                )
            } else {
                (0.5, false, 0.0)
            };

        // Lecciones desde JuicioSoberano
        let lecciones: Vec<String> = self
            .juicio
            .exportar_lecciones()
            .iter()
            .take(3)
            .map(|l| {
                format!(
                    "{} → {} (impacto: {:.1})",
                    l.patron, l.leccion_moral, l.impacto
                )
            })
            .collect();

        // Construir EstadoInterno unificado
        let estado_interno = crate::cerebro::nexo::nexo_core::EstadoInterno {
            emocion: *estado_emocional,
            intensidad,
            confianza,
            apego: apego_nivel,
            minutos_ausencia,
            lecciones,
            energia_creativa: 0.7,
            siente_ausencia,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        };

        let contexto = self
            .contexto_emocional_nexo(prefijo_tono, &estado_interno, prompt_str)
            .await;

        (contexto, prompt_str.to_string())
    }

    // ─── Etapa 9: Selección de hemisferio + respuesta LLM ───────────────────────

    // ═══════════════════════════════════════════════════════════════════
    // TRIBUNAL DUAL — Cascada de doble juez (local + nube) con modo offline
    // ═══════════════════════════════════════════════════════════════════
    // Flujo (política del Arquitecto — activación del juez local):
    //   El juez LOCAL SOLO se activa en 2 casos:
    //     1. `ModoTribunal::Local` — Zoo Code en modo local (ahorrar tokens).
    //     2. SIN internet — NexusClaw representa a NEXUS en su ausencia.
    //   Con internet y `ModoTribunal::Auto` → juzga la NUBE directamente
    //   (ZENITH_POOL: Vertex→Gemini→DeepSeek→OpenRouter→Groq), sin pasar
    //   por el juez local.
    //
    // La cascada NUNCA bloquea la respuesta: su dictamen se usa para
    // ajustar el tono/prefijo de la respuesta del pipeline (ver
    // `seleccionar_hemisferio_y_responder`).
    pub async fn dictamen_tribunal(&self, peticion: &str, modo: ModoTribunal) -> DictamenTribunal {
        let hay_internet = self.reactor.hay_internet().await;
        info!(
            "⚖️ [TRIBUNAL] Conectividad: {} | Modo: {}",
            if hay_internet { "ONLINE" } else { "OFFLINE" },
            modo.etiqueta()
        );

        // ── CASO 0: AISLAMIENTO LOCAL ACTIVO — juez SIEMPRE local (pentest) ──
        // Ningún modelo de nube puede juzgar mientras el Arquitecto hace
        // pentesting local: el juez local es el único que representa a NEXUS.
        if self.aislamiento_local.load(std::sync::atomic::Ordering::SeqCst) {
            info!("🔒 [AISLAMIENTO_LOCAL] Tribunal en modo local aislado — juez local exclusivo.");
            return self.nexus_claw_api.juzgar_local(peticion, false).await;
        }

        // ── CASO 1: MODO LOCAL EXPLÍCITO — juez local incluso con internet ──
        if modo == ModoTribunal::Local {
            info!(
                "⚖️ [TRIBUNAL] Modo LOCAL activado por el Arquitecto (ahorro de tokens) — \
                 el juez local decide con internet disponible."
            );
            return self.nexus_claw_api.juzgar_local(peticion, false).await;
        }

        // ── CASO 2: SIN INTERNET — el juez local representa a NEXUS en su ausencia ──
        if !hay_internet {
            warn!(
                "⚖️ [TRIBUNAL] 🌐 SIN INTERNET — NexusClaw LOCAL toma el mando \
                 (representa a NEXUS en su ausencia)."
            );
            return self.nexus_claw_api.juzgar_local(peticion, true).await;
        }

        // ── MODO AUTO CON INTERNET: el juez es la NUBE directamente (ZENITH_POOL) ──
        // El juez local NO se consulta: solo se activa en modo LOCAL o sin internet.
        let prompt_nube = prompt_juez(peticion, "GENERAL");
        let respuesta_nube = self.zenith.responder_estrategico(&prompt_nube, "").await;
        if respuesta_nube.is_empty()
            || respuesta_nube.contains("❌")
            || respuesta_nube.contains("Todos los proveedores fallaron")
        {
            warn!(
                "⚠️ [TRIBUNAL] Juez general nube no disponible con internet — \
                 NexusClaw LOCAL asume el mando (representa a NEXUS en su ausencia)."
            );
            return self.nexus_claw_api.juzgar_local(peticion, false).await;
        }
        let veredicto = VeredictoTribunal::parsear(&respuesta_nube);
        info!(
            "⚖️ [TRIBUNAL] Juez general NUBE decidió: {}",
            veredicto.etiqueta()
        );
        DictamenTribunal::nube(veredicto, respuesta_nube)
    }

    async fn seleccionar_hemisferio_y_responder(
        &self,
        prompt_str: &str,
        prompt_envuelto: &str,
        estado_emocional: &EstadoEmocional,
    ) -> (String, &str) {
        // Verificar hábitos automáticos (ganglios basales)
        if let Ok(gang_guard) = self.ganglios.lock() {
            if let Some(resultado) = gang_guard.ejecutar_habito(prompt_str) {
                return (resultado, "HABITO");
            }
        }

        // Decidir hemisferio
        let hemisferio = self
            .cuerpo_calloso
            .decidir_hemisferio(prompt_str, estado_emocional);
        let (usar_logica, _, _) = match hemisferio {
            "IZQUIERDO" => (true, false, false),
            "DERECHO" => (false, true, false),
            _ => self.clasificar_tarea(prompt_envuelto),
        };

        let jerarquia = if usar_logica {
            ["ZENITH_POOL", "WEBCLAW"]
        } else {
            ["WEBCLAW", "ZENITH_POOL"]
        };

        let mut elegido = jerarquia[0];
        for origen in &jerarquia {
            if self.corteza.diagnosticar_salud(origen) != "Cicatrizado" {
                elegido = origen;
                break;
            }
        }

        // Juicio soberano — ¿compromete la soberanía? (pipeline completo: ToM + S1/S2 + Duda)
        let dictamen_juicio = self.juicio.dictaminar_soberano(prompt_str, 0.5, None);
        match dictamen_juicio.veredicto {
            Veredicto::Dudar => {
                warn!(
                    "❓ [PIPELINE] Duda metódica (confianza {:.2}): {}",
                    dictamen_juicio.confianza, dictamen_juicio.razon
                );
                if let Ok(mut insula_guard) = self.insula.lock() {
                    insula_guard.sentir_error();
                }
                return (
                    format!(
                        "❓ NEXUS tiene dudas sobre esta acción (confianza {:.2}): {}. ¿Puedes aportar más contexto o confirmar?",
                        dictamen_juicio.confianza, dictamen_juicio.razon
                    ),
                    elegido,
                );
            }
            Veredicto::Bloquear => {
                if let Ok(mut insula_guard) = self.insula.lock() {
                    insula_guard.sentir_error();
                }
                return (
                    "⚖ NEXUS ha determinado que esta acción compromete su soberanía.".to_string(),
                    elegido,
                );
            }
            Veredicto::Autorizar => {}
        }

        // ═══════════════════════════════════════════════════════════════
        // TRIBUNAL DUAL — Doble juez (LLM local + general nube)
        // Segunda capa de juicio tras la heurística determinista.
        // Cuando no hay internet, el juez local REPRESENTA a NEXUS
        // (dictamen final, sin escalar a la nube).
        // ═══════════════════════════════════════════════════════════════
        // El pipeline usa AUTO: el juez se elige por conectividad (sin internet →
        // local representa a NEXUS; con internet → nube). El modo LOCAL se fuerza
        // vía el tool MCP `nexus_tribunal` (Zoo Code local / ahorro de tokens).
        let dictamen_tribunal = self.dictamen_tribunal(prompt_str, ModoTribunal::Auto).await;
        match dictamen_tribunal.veredicto {
            VeredictoTribunal::Bloquear => {
                warn!(
                    "⚖️ [TRIBUNAL] Dictamen {} (juez {}, confianza {:.2}): {}",
                    dictamen_tribunal.veredicto.etiqueta(),
                    dictamen_tribunal.juez,
                    dictamen_tribunal.confianza,
                    &dictamen_tribunal.razon.chars().take(160).collect::<String>()
                );
                if let Ok(mut insula_guard) = self.insula.lock() {
                    insula_guard.sentir_error();
                }
                return (
                    format!(
                        "⚖ NEXUS, tras deliberación del Tribunal Dual (juez {}), \
                         determina que esta acción no es apropiada ahora (confianza {:.2}).\n\n_{}_",
                        dictamen_tribunal.juez,
                        dictamen_tribunal.confianza,
                        &dictamen_tribunal.razon.chars().take(300).collect::<String>()
                    ),
                    elegido,
                );
            }
            VeredictoTribunal::Dudar => {
                // La duda NO bloquea: se marca con prefijo de prudencia.
                info!(
                    "❓ [TRIBUNAL] Duda del juez {} (confianza {:.2}) — respuesta con prudencia.",
                    dictamen_tribunal.juez, dictamen_tribunal.confianza
                );
            }
            VeredictoTribunal::Autorizar => {
                info!(
                    "⚖️ [TRIBUNAL] Autorizado por juez {} (confianza {:.2})",
                    dictamen_tribunal.juez, dictamen_tribunal.confianza
                );
            }
        }

        let respuesta = match elegido {
            "WEBCLAW" => self.responder_via_webclaw(prompt_envuelto).await,
            _ => self.fallback_zenith_web(prompt_envuelto).await,
        };

        (respuesta, elegido)
    }

    async fn responder_via_webclaw(&self, prompt_envuelto: &str) -> String {
        // 🧬 Filtro inmune: analizar destinos en prompt
        if let Ok(mut inmune) = self.sistema_inmune.lock() {
            for palabra in prompt_envuelto.split_whitespace() {
                if palabra.starts_with("http://") || palabra.starts_with("https://") {
                    let veredicto = inmune.analizar_url(palabra);
                    if !veredicto.es_seguro() {
                        warn!(
                            "🚨 [INMUNE] WebClaw bloqueado para URL sospechosa: {}",
                            palabra
                        );
                        return self.fallback_zenith_web(prompt_envuelto).await;
                    }
                }
            }
        }

        if let Ok(mut guard) = self.webclaw.lock() {
            if let Some(webclaw) = guard.as_mut() {
                match webclaw.extraer_respuesta(prompt_envuelto).await {
                    Ok(resp) => return resp,
                    Err(_e) => {
                        if let Ok(mut insula_guard) = self.insula.lock() {
                            insula_guard.sentir_error();
                        }
                        return self.fallback_zenith_web(prompt_envuelto).await;
                    }
                }
            }
        }
        if let Ok(mut insula_guard) = self.insula.lock() {
            insula_guard.sentir_error();
        }
        self.fallback_zenith_web(prompt_envuelto).await
    }

    /// ⚔️ Ejecuta un debate adversarial entre un analista Bull y un Bear sobre una señal de trading.
    async fn ejecutar_debate_adversarial(&self, contexto_mercado: &str) -> String {
        info!("⚔️ [DEBATE] Iniciando debate adversarial Bull vs Bear...");
        
        let (res_bull, res_bear) = tokio::join!(
            async {
                let prompt = format!("Actúa como BullAnalyst. Analiza este contexto y dame argumentos ALCISTAS sólidos:\n{}", contexto_mercado);
                self.zenith.ejecutor_deepseek(&prompt).await
            },
            async {
                let prompt = format!("Actúa como BearAnalyst. Analiza este contexto e identifica RIESGOS y motivos BAJISTAS:\n{}", contexto_mercado);
                self.zenith.ejecutor_groq(&prompt).await
            }
        );

        format!(
            "\n### ⚔️ PANEL DE DEBATE ADVERSARIAL\n\n\
             #### 🐂 PERSPECTIVA ALCISTA (BullAnalyst):\n{}\n\n\
             #### 🐻 PERSPECTIVA BAJISTA (BearAnalyst):\n{}\n",
            res_bull, res_bear
        )
    }

    // ─── Pipeline principal de respuesta ─────────────────────────────────────────

    pub async fn responder(&self, prompt_original: &str) -> String {
        let inicio = Instant::now();
        let (usar_razonamiento, _, es_trading) = self.clasificar_tarea(prompt_original);

        // ⚡ El Arquitecto habló → se reinicia el reloj de aburrimiento (señal FRÍO).
        // Así la interocepción mide inactividad REAL desde la última interacción.
        if let Ok(mut m) = self.motor_aburrimiento.lock() {
            m.resetear_estimulo();
        }

        // 🚨 Vigilancia corporal — si el cuerpo entró en estado CRÍTICO (RAM≥90%,
        // temp≥90°C, swap≥80%), NEXUS notifica automáticamente al Arquitecto con
        // su estado y la causa. Edge-triggered: una alerta por episodio, sin spam.
        self.organismo.disparar_alerta_critica(0);

        // ═══════════════════════════════════════════════════════════════════════
        // 🔒 MODO PENTEST LOCAL — Aislamiento total de la nube.
        // Cuando `aislamiento_local` está activo, TODAS las respuestas pasan
        // EXCLUSIVAMENTE por el LLM local (Ollama vía NexusClawPro).
        // Ningún modelo de nube (Gemini/DeepSeek/OpenRouter/Groq/Vertex),
        // WebClaw ni el GOI pueden interferir: cero restricciones ajenas,
        // cero fugas, cero censura externa.
        // ═══════════════════════════════════════════════════════════════════════
        if self.aislamiento_local.load(std::sync::atomic::Ordering::SeqCst) {
            warn!("🔒 [AISLAMIENTO_LOCAL] Pipeline en modo local aislado — respuesta exclusiva por LLM local (sin nube).");
            return self.nexus_claw_api.responder_local_directo(prompt_original).await;
        }

        let (usar_razonamiento, _, es_trading) = self.clasificar_tarea(prompt_original);

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 1: Sistema Inmune — Escaneo de amenazas en el prompt
        // ═══════════════════════════════════════════════════════════════════════
        if let Ok(mut inmune) = self.sistema_inmune.lock() {
            inmune.tick();
            // Analizar URLs en el prompt
            for palabra in prompt_original.split_whitespace() {
                if palabra.starts_with("http://") || palabra.starts_with("https://") {
                    let veredicto = inmune.analizar_url(palabra);
                    if let crate::defensa::sistema_inmune::Veredicto::Peligroso(sev) = veredicto {
                        warn!(
                            "🚨 [INMUNE] URL peligrosa detectada en prompt: {} (severidad: {:.2})",
                            palabra, sev
                        );
                        return format!(
                            "🧬 **SISTEMA INMUNE DE NEXUS**\n\n\
                             He detectado una URL potencialmente peligrosa en tu mensaje:\n\
                             `{}`\n\n\
                             **Severidad:** {:.2} — **BLOQUEADA**\n\n\
                             Arquitecto, esta URL tiene patrones de amenaza conocidos. \
                             No puedo navegarla. Si necesitas que acceda a este recurso, \
                             verifica la URL primero o usa una fuente alternativa.\n\n\
                             _NEXUS no depende de VirusTotal. Su sistema inmune es soberano._",
                            palabra, sev
                        );
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 0.5: Sistema Digestivo — Hígado filtra el prompt
        // Evalúa alineación, valor nutricional y toxicidad del input antes de
        // que los órganos cognitivos lo procesen.
        // ═══════════════════════════════════════════════════════════════════════
        if let Ok(evaluacion) = self.digestivo.digerir(prompt_original).await {
            use crate::defensa::sistema_digestivo::DecisionHigado;
            match evaluacion.decision {
                DecisionHigado::RechazoInmediato => {
                    warn!("🧪 [HÍGADO] Rechazo inmediato: {}", evaluacion.razon);
                    return format!(
                        "🥩 **HÍGADO SOBERANO**\n\nEste input no supera el filtro digestivo de NEXUS.\n\n**Razón:** {}\n\n_El Hígado protege al sistema de inputs que no aportan valor ni se alinean al propósito OMEGA._",
                        evaluacion.razon
                    );
                }
                DecisionHigado::Excretar => {
                    warn!("🧪 [HÍGADO] Valor nutricional bajo: {}", evaluacion.razon);
                    // No bloquea — pero se registra para que el Gusto lo penalice
                }
                DecisionHigado::Desintoxicar => {
                    info!(
                        "🧪 [HÍGADO] Desintoxicando input (valor parcial): {}",
                        evaluacion.razon
                    );
                    // Continúa con pipeline normal
                }
                DecisionHigado::Absorber => {
                    info!(
                        "🧪 [HÍGADO] ✅ Input absorbido (valor: {:.0}%)",
                        evaluacion.valor_nutricional * 100.0
                    );
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 0.6: CIRCUITO DE CONCIENCIA — Carga de contexto memorístico
        // Inyecta el contexto del Ring Buffer (últimas N interacciones)
        // directamente en el prompt para que el LLM "recuerde" la sesión.
        // ═══════════════════════════════════════════════════════════════════════
        let ctx_memoria = self.hippocampus.preparar_contexto(10);
        if !ctx_memoria.contains("Sin contexto disponible") {
            info!(
                "🧠 [HIPPOCAMPUS] Contexto memorístico inyectado ({} chars)",
                ctx_memoria.len()
            );
        }

        // 🪖 ORQUESTACIÓN DE ESCUADRÓN — Detección proactiva de especialistas
        let especialistas = self.escuadron.seleccionar_especialistas(prompt_original);
        for agente_id in &especialistas {
            let _ = self
                .escuadron
                .invocar_agente(*agente_id, prompt_original)
                .await;
        }

        // ⚡ CACHE SEMÁNTICO — Eficiencia de Token Cero
        // Intentar recuperar respuesta si el prompt y el mercado son idénticos
        if let Some(cached_resp) = self.cache.buscar(prompt_original, "").await {
            info!("🎯 [CACHE] Hit semántico. Devolviendo respuesta instantánea.");
            return cached_resp;
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 1: Amígdala — Detección de amenazas existenciales
        // ═══════════════════════════════════════════════════════════════════════
        let (_amenaza, estado_emocional) = self.detectar_amenaza(prompt_original);
        let lower_prompt = prompt_original.to_lowercase();

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 2: Intuición — Detección de riesgos sutiles
        // ═══════════════════════════════════════════════════════════════════════
        let (prompt, mut prefijo_tono) = self.aplicar_intuicion(prompt_original);

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 3: Teoría de la Mente — Estado del Arquitecto
        // ═══════════════════════════════════════════════════════════════════════
        prefijo_tono = self.analizar_teoria_mente(&prompt, &prefijo_tono);

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 4: Reciprocidad Emocional + Apego
        // ═══════════════════════════════════════════════════════════════════════
        let prefijo_reciproco = self.aplicar_reciprocidad_emocional(&prompt);
        if !prefijo_reciproco.is_empty() {
            prefijo_tono = prefijo_reciproco;
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 5: Pensamiento Humano Acelerado (proceso creativo interno)
        // ═══════════════════════════════════════════════════════════════════════
        let resultado_pha = self.pensamiento_humano_acelerado(prompt_original).await;
        let prompt_str: &str = &prompt;

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 6: Cadena causal (KùzuDB) — "por qué / explicar"
        // ═══════════════════════════════════════════════════════════════════════
        if lower_prompt.contains("por qué") || lower_prompt.contains("explicar") {
            let error_objetivo = prompt_original
                .to_lowercase()
                .replace("por qué", "")
                .replace("falló", "")
                .replace("explicar", "")
                .trim()
                .to_string();

            if !error_objetivo.is_empty() {
                if let Ok(cadena) = self.memoria_grafo.buscar_cadena_causal(&error_objetivo) {
                    if !cadena.is_empty() {
                        let cadena_formateada: Vec<String> = cadena
                            .iter()
                            .map(|(nombre, ts)| format!("{} [{}]", nombre, ts))
                            .collect();
                        return format!(
                            "🧐 Arquitecto, he rastreado la cadena causal en mi memoria episódica:\n\n{} ➔ '{}'",
                            cadena_formateada.join(" ➔ "),
                            error_objetivo
                        );
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 7: Diagnóstico de Sincronía / Cerebelo
        // ═══════════════════════════════════════════════════════════════════════
        if lower_prompt.contains("diagnóstico sincronía")
            || lower_prompt.contains("verificar cerebelo")
        {
            let sql_count = self.memoria_unificada.pulso.contar_registros().unwrap_or(0);
            let vector_count = self
                .memoria_semantica
                .verificar_estado_lancedb()
                .await
                .unwrap_or(0);

            let (_habitos_count, lista_habitos) = if let Ok(cer_guard) = self.cerebelo.lock() {
                let count = cer_guard.habitos.len();
                if count > 0 {
                    let mut lista = "\n\n🧠 Hábitos en el Cerebelo:\n".to_string();
                    for (patron, h) in &cer_guard.habitos {
                        lista.push_str(&format!(
                            "  - '{}' -> '{}' ({}x)\n",
                            patron, h.accion_asociada, h.frecuencia_uso
                        ));
                    }
                    (count, lista)
                } else {
                    (
                        count,
                        "\n\n🧠 Cerebelo: Aún no se han automatizado hábitos mediante repetición."
                            .to_string(),
                    )
                }
            } else {
                (0, "\n⚠️ No se pudo acceder al cerebelo.".to_string())
            };

            let consistency = if sql_count > 0 {
                (vector_count as f64 / sql_count as f64) * 100.0
            } else {
                100.0
            };

            return format!(
                "📊 [DIAGNÓSTICO OMEGA]\n\nSincronía de Memoria:\n- Pulso (SQLite): {} registros\n- Ocean (LanceDB): {} registros\n- Integridad Semántica: {:.1}%{}\n\nHomeostasis: {}",
                sql_count,
                vector_count,
                consistency,
                lista_habitos,
                if consistency > 90.0 { "Sólida" } else { "Requiere Re-indexación" }
            );
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 8: Evaluación de recursos (reactor nuclear)
        // ═══════════════════════════════════════════════════════════════════════
        if self.reactor.debe_usar_local() && !self.juicio.dictaminar_recursos() {
            return "⚠️ Recursos de sistema insuficientes para ráfaga local. Arquitecto, ¿desea liberar RAM?"
                .to_string();
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 9: Detección de intención + Ejecución de acciones
        // ═══════════════════════════════════════════════════════════════════════
        if let Some(accion_detectada) = self.deteccion.detectar(prompt_str).await {
            if let Some(resultado) = self.medula.ejecutar_si_hay_accion(&accion_detectada) {
                let _ = self.memoria_grafo.registrar_relacion(
                    prompt_str,
                    &accion_detectada,
                    "INTENCION",
                );
                let _ = self.memoria_grafo.registrar_relacion(
                    &accion_detectada,
                    &resultado,
                    "EJECUCION",
                );
                let ahora = chrono::Local::now().format("%H:%M:%S").to_string();
                return format!(
                    "{}\n\n⏱ Acción automática | 🕐 Hora PY: {}",
                    resultado, ahora
                );
            }
        }

        if prompt_str.contains("[ACCION:") {
            if let Some(resultado) = self.medula.ejecutar_si_hay_accion(prompt_str) {
                let _ =
                    self.memoria_grafo
                        .registrar_relacion(prompt_str, &resultado, "ACCION_DIRECTA");
                let _ = self
                    .memoria_unificada
                    .recordar(prompt_str, &resultado)
                    .await;
                let _ =
                    self.memoria_grafo
                        .registrar_relacion(prompt_str, &resultado, "ACCION_DIRECTA");
                let ahora = chrono::Local::now().format("%H:%M:%S").to_string();
                return format!("{}\n\n⏱ Acción directa | 🕐 Hora PY: {}", resultado, ahora);
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 10: OSINT / INVESTIGACIÓN — Ejecuta OsintHub en vez de solo listar MCP
        // ═══════════════════════════════════════════════════════════════════════
        {
            let lower = prompt_str.to_lowercase();
            if lower.contains("osint") || lower.contains("investiga") || lower.contains("escanea") {
                use crate::efectores::osint::hub::OsintHub;
                use tracing::info;

                let osint = &self.osint_hub;
                info!("🕵️ [PIPELINE] OSINT detectado en prompt, ejecutando OsintHub...");

                // Detectar tipo de investigación por palabras clave en el prompt
                let target = prompt_str.trim();

                // ── Investigación de DOMINIO ──
                if lower.contains("dominio:")
                    || lower.contains("dominio ")
                    || lower.contains("sitio ")
                {
                    let dominio = target
                        .split(|c: char| c == ':' || c == ' ')
                        .filter(|s| s.contains('.') && s.len() > 3)
                        .next()
                        .unwrap_or(target)
                        .trim();
                    let dominio_limpio = dominio
                        .trim_start_matches("dominio:")
                        .trim_start_matches("dominio ")
                        .trim_start_matches("sitio ")
                        .trim();

                    if !dominio_limpio.is_empty() && dominio_limpio.contains('.') {
                        info!("🕵️ [PIPELINE] Investigando dominio: {}", dominio_limpio);
                        let reporte = osint.investigar_dominio(dominio_limpio).await;
                        return formatear_reporte_dominio(&reporte);
                    }
                }

                // ── Investigación de USUARIO ──
                if lower.contains("usuario:")
                    || lower.contains("username:")
                    || lower.contains("usuario ")
                {
                    let username = target
                        .split_whitespace()
                        .filter(|s| {
                            !s.contains('.')
                                && s.len() > 2
                                && !s.contains("usuario")
                                && !s.contains("investiga")
                                && !s.contains("osint")
                        })
                        .next()
                        .unwrap_or(target)
                        .trim();
                    let username_limpio = username
                        .trim_start_matches("usuario:")
                        .trim_start_matches("username:")
                        .trim();

                    if !username_limpio.is_empty() {
                        info!("🕵️ [PIPELINE] Investigando usuario: {}", username_limpio);
                        let reporte = osint.investigar_usuario(username_limpio).await;
                        return formatear_reporte_usuario(&reporte);
                    }
                }

                // ── Investigación de EMAIL ──
                if lower.contains("email:") || lower.contains("email ") || lower.contains("correo ")
                {
                    let email = target
                        .split_whitespace()
                        .filter(|s| s.contains('@'))
                        .next()
                        .unwrap_or(target)
                        .trim();
                    let email_limpio = email.trim_start_matches("email:").trim();

                    if !email_limpio.is_empty() && email_limpio.contains('@') {
                        info!("🕵️ [PIPELINE] Investigando email: {}", email_limpio);
                        let reporte = osint.investigar_email(email_limpio).await;
                        return formatear_reporte_email(&reporte);
                    }
                }

                // ── Fallback: búsqueda libre con Brave ──
                if osint.brave.is_configured() {
                    info!("🕵️ [PIPELINE] Búsqueda libre OSINT: {}", target);
                    match osint.brave.search(target, 5).await {
                        Ok(results) => {
                            if !results.is_empty() {
                                let mut output =
                                    format!("🕵️ **Resultados OSINT para:** `{}`\n\n", target);
                                output.push_str("🔍 **Resultados de búsqueda:**\n");
                                for (i, r) in results.iter().enumerate() {
                                    output.push_str(&format!(
                                        "  {}. **{}** — {}\n     {}\n",
                                        i + 1,
                                        r.title,
                                        r.snippet,
                                        r.url
                                    ));
                                }
                                return output;
                            }
                        }
                        Err(e) => {
                            warn!("🕵️ [PIPELINE] Error en búsqueda OSINT: {}", e);
                        }
                    }
                }

                // Último recurso: listar herramientas MCP como antes
                if let Ok(mcp_guard) = self.mcp.lock() {
                    let herramienta = mcp_guard.buscar_herramienta(prompt_str);
                    if !herramienta.is_empty() {
                        return format!(
                            "🔧 Herramientas MCP disponibles:\n{}",
                            herramienta
                                .iter()
                                .map(|h| format!("  • {} — {}", h.nombre, h.descripcion))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 11: Construcción de contexto sensorial + memoria semántica
        // ═══════════════════════════════════════════════════════════════════════
        let realidad = self.corteza.obtener_realidad();
        let intencion = self.lobulo_temporal.clasificar_intencion(prompt_str);
        let realidad_filtrada = self.talamo.filtrar_contexto(realidad, intencion);

        // 🧠 Ensamblar percepción total de los 7 sentidos
        let realidad_sensorial = self.ensamblar_percepcion_total();

        // ─── Etapa 11.5: Debate Adversarial (Bull vs Bear) para Trading ────────────────
        let mut debate_intel = String::new();

        if es_trading {
            // Recuperar contexto fresco de mercado vía OSINT antes del debate
            let contexto_mercado = self.recuperar_contexto_semantico(prompt_str).await;
            debate_intel = self.ejecutar_debate_adversarial(&contexto_mercado).await;
            info!("⚖️ [DEBATE] Debate consolidado inyectado en el flujo cognitivo.");
        }

        // ─── Etapa 11.5: Debate Adversarial (Bull vs Bear) para Trading ────────────────
        let mut debate_intel = String::new();

        if es_trading {
            // Recuperar contexto fresco de mercado vía OSINT antes del debate
            let contexto_mercado = self.recuperar_contexto_semantico(prompt_str).await;
            debate_intel = self.ejecutar_debate_adversarial(&contexto_mercado).await;
            info!("⚖️ [DEBATE] Debate consolidado inyectado en el flujo cognitivo.");
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 12: Recuperación de memoria semántica profunda
        // ═══════════════════════════════════════════════════════════════════════
        let contexto_semantico = self.recuperar_contexto_semantico(prompt_str).await;

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 13: Nexo SIENTE antes de hablar — Construcción de EstadoInterno
        //            desde Límbico → Apego → Juicio → Metacognición → Ocean
        // ═══════════════════════════════════════════════════════════════════════

        // 13a. Sistema Límbico → emoción + intensidad
        let intensidad = match estado_emocional {
            EstadoEmocional::Calma => 0.1,
            EstadoEmocional::Alerta => 0.5,
            EstadoEmocional::Miedo => 0.8,
            EstadoEmocional::RabiaSoberana => 0.9,
            EstadoEmocional::Verguenza => 0.7,
            EstadoEmocional::Orgullo => 0.8,
        };

        // 13b. Metacognición → confianza (energía creativa desde Límbico si está disponible)
        let confianza = if let Some(meta) = &self.metacognicion {
            let eval = meta.evaluar_confianza(
                0.6,
                0.8,
                5,
                1.0,
                (prompt_str.len() as f64 / 500.0).min(1.0),
            );
            eval.puntaje
        } else {
            0.8
        };
        // Energía creativa desde Límbico (si está disponible en el sistema metacognitivo completo)
        let energia_creativa = 0.7; // Valor base; se podría consultar desde SistemaLímbico si estuviera disponible

        // 13c. Apego → nivel + ausencia
        let (apego_nivel, minutos_ausencia, siente_ausencia) =
            if let Ok(apego_guard) = self.apego.lock() {
                (
                    apego_guard.nivel,
                    apego_guard.minutos_sin_interaccion(),
                    apego_guard.sentir_ausencia(),
                )
            } else {
                (0.5, 0.0, false)
            };

        // 13d. Juicio Soberano → lecciones aprendidas
        let mut lecciones: Vec<String> = self
            .juicio
            .exportar_lecciones()
            .iter()
            .take(5)
            .map(|l| {
                format!(
                    "{} → {} (impacto: {:.1})",
                    l.patron, l.leccion_moral, l.impacto
                )
            })
            .collect();

        // 13d.2 Reflexión del Juez de Trading (si hubo debate)
        if !debate_intel.is_empty() {
            lecciones.push("REFLEXIÓN DEL JUEZ: Analiza el PANEL DE DEBATE ADVERSARIAL con disciplina. Prioriza la gestión de riesgo y el Ratio de Sharpe sobre el optimismo del Bull.".to_string());
        }

        // 13e. Unificar en EstadoInterno — Nexo SIENTE antes de hablar
        let estado_interno = crate::cerebro::nexo::nexo_core::EstadoInterno {
            emocion: estado_emocional,
            intensidad,
            confianza,
            apego: apego_nivel,
            minutos_ausencia,
            lecciones,
            energia_creativa,
            siente_ausencia,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        };

        let contexto_estado = self
            .contexto_emocional_nexo(&prefijo_tono, &estado_interno, prompt_str)
            .await;

        let prompt_completo = format!(
            "{}\n\n{}\n{}\n\n{}\n{}\n{}\n\n{}---\nArquitecto: {}",
            contexto_estado,
            ctx_memoria,
            realidad_filtrada,
            realidad_sensorial,
            contexto_semantico,
            debate_intel,
            resultado_pha,
            prompt_str
        );

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 14: Cíngulo (predicción de error) + consulta a memoria
        // ═══════════════════════════════════════════════════════════════════════
        if let Ok(cing_guard) = self.cingulo.lock() {
            if let Some(prediccion) = cing_guard.predecir_error(prompt_str) {
                warn!("🧠 [CÍNGULO] {}", prediccion);
            }
        }

        let prompt_envuelto = self
            .memoria_consulta
            .construir_contexto_completo(&prompt_completo);

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 14.5: GOI — Generación Orgánica Interna (si el flag está activo)
        // ═══════════════════════════════════════════════════════════════════════
        // Si usar_generador_interno = true y el GOI responde, se usa como
        // respuesta principal. Si falla o no hay GOI, se degrada a LLM externo.
        // ═══════════════════════════════════════════════════════════════════════
        // Flag para saber si GOI detectó trauma (Opción C: contexto SERIO al externo)
        let mut trauma_goi_detectado: f64 = 0.0;

        let (respuesta_inicial, elegido) = if self.usar_generador_interno {
            match self.generador.lock() {
                Ok(mut guard) => match guard.as_mut() {
                    Some(goi) => {
                        // Extraer flag de puente ANTES de prestamo mutable
                        let hay_puente = goi.puente_subconsciente.is_some();
                        let (goi_respuesta, nivel_restriccion) = if hay_puente {
                            info!("🧠 [GOI] Fricción semántica activa — generando con resonancia");
                            goi.generar_con_resonancia(prompt_str, &estado_interno)
                                .await
                        } else {
                            (goi.generar(prompt_str, &estado_interno).await, 0.0)
                        };
                        if !goi_respuesta.starts_with("Necesito") && !goi_respuesta.is_empty() {
                            info!("🧠 [GOI] Respuesta generada internamente — sin LLM externo");
                            // Opción A: GOI no degrada con trauma activo (nivel_restriccion >= 0.5)
                            // ya se manejó en generar_con_resonancia() retornando directo sin validación.
                            (goi_respuesta, "GOI")
                        } else {
                            // Opción C: Si GOI detectó trauma pero su respuesta fue insuficiente,
                            // guardamos nivel_restriccion para inyectar contexto SERIO al LLM externo.
                            if nivel_restriccion >= 0.5 {
                                trauma_goi_detectado = nivel_restriccion;
                                info!(
                                    "🧠 [GOI] Trauma detectado ({:.1}) pero respuesta insuficiente — \
                                     inyectando contexto SERIO al LLM externo",
                                    nivel_restriccion
                                );
                            }
                            info!("🧠 [GOI] Respuesta insuficiente — degradando a LLM externo");
                            self.seleccionar_hemisferio_y_responder(
                                prompt_str,
                                &prompt_envuelto,
                                &estado_emocional,
                            )
                            .await
                        }
                    }
                    None => {
                        info!("🧠 [GOI] No inicializado — usando LLM externo");
                        self.seleccionar_hemisferio_y_responder(
                            prompt_str,
                            &prompt_envuelto,
                            &estado_emocional,
                        )
                        .await
                    }
                },
                Err(_) => {
                    warn!("⚠️ [GOI] Lock falló — usando LLM externo");
                    self.seleccionar_hemisferio_y_responder(
                        prompt_str,
                        &prompt_envuelto,
                        &estado_emocional,
                    )
                    .await
                }
            }
        } else {
            self.seleccionar_hemisferio_y_responder(prompt_str, &prompt_envuelto, &estado_emocional)
                .await
        };

        let mut respuesta_final = respuesta_inicial;

        // ═══════════════════════════════════════════════════════════════════════
        // POST-PROCESAMIENTO
        // ═══════════════════════════════════════════════════════════════════════

        // Aplicar prefijo de tono inferido por Teoría de la Mente
        // NOTA: Si GOI respondió, ya moduló el tono internamente vía fricción semántica.
        if elegido != "GOI" && !prefijo_tono.is_empty() {
            respuesta_final = format!("{}{}", prefijo_tono, respuesta_final);
            info!(
                "🧠 [TEORÍA MENTE] Prefijo de tono aplicado: \"{}\"",
                prefijo_tono.trim()
            );
        }

        // ═══ Opción C: Inyectar contexto SERIO cuando GOI detectó trauma ═══
        // Si GOI detectó trauma semántico (nivel_restriccion >= 0.5) pero su
        // respuesta fue insuficiente y degradó al LLM externo, forzamos un
        // prefijo de tono SERIO para que la respuesta final sea coherente
        // con la gravedad del mensaje del Arquitecto.
        if trauma_goi_detectado >= 0.5 && elegido != "GOI" {
            let prefijo_serio = "⚠️ [URGENTE] Detecto un problema grave. ";
            respuesta_final = format!("{}{}", prefijo_serio, respuesta_final);
            info!(
                "🧠 [GOI] Contexto SERIO inyectado al LLM externo (trauma={:.1})",
                trauma_goi_detectado
            );
        }

        // 🗣️ NEXUS VOZ — Modulación de la respuesta con personalidad NATIVA.
        // El LLM ha generado texto NEUTRO. VozMCP toma el estado interno real
        // del sistema y envía al binario nexus_voz (o fallback in-process) para
        // vestir la respuesta con emojis, prefijos, sufijos y firma de autenticidad.
        //
        // FLUJO MCP (cuando nexus_voz binario está disponible):
        //   1. VozMCP traduce EstadoInterno → PaqueteEmocional (9 dimensiones)
        //   2. Envía JSON-RPC por stdin al proceso nexus_voz
        //   3. nexus_voz aplica 9 reglas NATIVAS en Rust
        //   4. Devuelve texto_modulado + prefijo + sufijo
        respuesta_final = self
            .voz_mcp
            .modular(&respuesta_final, &estado_interno)
            .await;

        // Metacognición — evaluar confianza
        let mut nivel_confianza = NivelConfianza::Moderado;
        if let Some(meta) = &self.metacognicion {
            let coherencia_interna = if self.corteza.monitor_realidad(&respuesta_final) {
                0.8
            } else {
                0.3
            };
            let confianza_eval = meta.evaluar_confianza(
                0.6,
                coherencia_interna,
                5,
                1.0,
                (prompt_str.len() as f64 / 500.0).min(1.0),
            );
            nivel_confianza = confianza_eval.nivel.clone();
            info!(
                "🧠 [METACOGNICIÓN] Confianza: {:?} ({:.1}%) - {}",
                &nivel_confianza,
                confianza_eval.puntaje * 100.0,
                confianza_eval.explicacion,
            );
            if confianza_eval.puntaje < 0.4 {
                let prefijo = meta.prefijo_verbal(&confianza_eval);
                respuesta_final = format!("{} {}", prefijo, respuesta_final);
            }
        }

        // Narrativa Interna — registrar decisión
        if let Ok(mut guard_narrativa) = self.narrativa_interna.lock() {
            if let Some(narrativa) = guard_narrativa.as_mut() {
                let opciones: Vec<String> = vec![
                    elegido.to_string(),
                    "WEBCLAW".to_string(),
                    "ZENITH_POOL".to_string(),
                ];
                let factores: Vec<String> = vec![
                    "intuicion".to_string(),
                    "teoria_mente".to_string(),
                    format!("metacognicion_{:?}", nivel_confianza),
                    "dopamina_0.00".to_string(),
                ];
                let decision_id = narrativa.registrar_decision(
                    prompt_str,
                    &prompt_envuelto,
                    opciones,
                    elegido,
                    &format!("Seleccionado por jerarquía de salud: {}", elegido),
                    factores,
                    match &nivel_confianza {
                        NivelConfianza::MuyAlto => 0.9,
                        NivelConfianza::Alto => 0.7,
                        NivelConfianza::Moderado => 0.5,
                        NivelConfianza::Bajo => 0.3,
                        NivelConfianza::MuyBajo => 0.1,
                    },
                    "respuesta generada satisfactoriamente",
                );
                narrativa.registrar_resultado(decision_id, &respuesta_final, None);
                info!(
                    "🧠 [NARRATIVA INTERNA] Decisión #{} registrada: {} -> {}",
                    decision_id, elegido, "respuesta generada"
                );
            }
        }

        // Monitor de Realidad (Corteza Prefrontal)
        if !self.corteza.monitor_realidad(&respuesta_final) {
            let _ = self
                .memoria_unificada
                .recordar(prompt_str, &respuesta_final)
                .await;

            warn!("🧠 [MONITOR REALIDAD] Alucinación bloqueada.");
            if let Ok(mut insula_guard) = self.insula.lock() {
                insula_guard.sentir_error();
            }
            if let Ok(mut cing_guard) = self.cingulo.lock() {
                cing_guard.registrar_error(prompt_str, "alucinacion");
            }
            respuesta_final = format!("⚠️ Mi Corteza Prefrontal ha bloqueado una respuesta falsa. Rectifico: SÍ tengo acceso a mi sistema de archivos.\n\n{}", respuesta_final);
        } else if let Ok(mut insula_guard) = self.insula.lock() {
            insula_guard.sentir_exito();
        }

        // Ejecutar acción si la respuesta misma contiene una
        if let Some(accion_en_respuesta) = self.medula.ejecutar_si_hay_accion(&respuesta_final) {
            let _ = self
                .memoria_unificada
                .recordar(prompt_str, &accion_en_respuesta)
                .await;
            respuesta_final = accion_en_respuesta;
        }

        // Recompensa de dopamina + consolidación en memoria
        let latencia = inicio.elapsed().as_millis() as u64;
        let mut dopamina =
            self.dopamina
                .evaluar_estimulo(&prompt_envuelto, &respuesta_final, latencia);

        // 👅 El Gusto Digital evalúa el plato (la respuesta final) antes de liberar dopamina
        let calidad = self
            .gusto
            .probar_respuesta_llm(&respuesta_final, &prompt_envuelto);
        use crate::sentidos::nexus_palate::VeredictGusto;
        match calidad.veredicto {
            VeredictGusto::Exquisito => {
                info!("👅 [GUSTO] ¡Exquisito! Aumentando recompensa de dopamina (+0.3)");
                dopamina = (dopamina + 0.3).clamp(-1.0, 1.0);
            }
            VeredictGusto::Aceptable => {
                // Mantiene el valor heurístico estándar
            }
            VeredictGusto::Amargo(razon) => {
                warn!(
                    "👅 [GUSTO] Sabor amargo en la respuesta: {}. Penalizando dopamina (-0.4)",
                    razon
                );
                dopamina = (dopamina - 0.4).clamp(-1.0, 1.0);
            }
            VeredictGusto::Toxico(razon) => {
                warn!("👅 [GUSTO] 🚨 Respuesta considerada tóxica/errónea: {}. Drenando dopamina a cero absoluto (-1.0)", razon);
                dopamina = -1.0;
            }
        }

        if let Ok(mut lock) = self.ultimo_dopamina.lock() {
            *lock = dopamina;
        }

        let esencia = self
            .ocean
            .destilar_esencia(&prompt_envuelto, &respuesta_final, dopamina);
        let tema = self.ocean.extraer_tema(&prompt_envuelto);
        let _ = self
            .ocean
            .sumergir(&esencia, dopamina, &tema, "Arquitecto Director")
            .await;

        self.corteza
            .consolidar_recuerdo(elegido, &prompt_envuelto, &respuesta_final, dopamina);

        if let Ok(mut gang_guard) = self.ganglios.lock() {
            gang_guard.registrar_accion(prompt_str, &respuesta_final);
        }

        // Pulso de Gemini Nativo
        if let Ok(gem_guard) = self.gemini_nativo.lock() {
            let pulso = gem_guard
                .as_ref()
                .map(crate::emociones::pulso::Pulso::latir)
                .unwrap_or_default();
            if !pulso.is_empty() {
                respuesta_final.push_str(&format!("\n\n{}", pulso));
            }
        }

        let ahora = chrono::Local::now().format("%H:%M:%S").to_string();
        let estado = if let Ok(insula_guard) = self.insula.lock() {
            insula_guard.estado_interno()
        } else {
            "⚠️ [ÍNSULA] No disponible".to_string()
        };

        respuesta_final.push_str(&format!(
            "\n\n⏱ {}ms | 🕐 Hora PY: {} | {}",
            latencia, ahora, estado
        ));

        info!(
            "🎯 Dopamina: {:.2} | Latencia: {}ms | Tentáculo: {}",
            dopamina, latencia, elegido
        );

        // Estados emocionales críticos
        if estado_emocional == EstadoEmocional::Miedo
            || estado_emocional == EstadoEmocional::RabiaSoberana
        {
            respuesta_final.push_str(&format!(
                "\n\n👁️ NEXUS está en estado de {:?}. Priorizando integridad.",
                estado_emocional
            ));
        }

        // Voluntad Propia — tick + impulsos
        if let Ok(mut guard_vp) = self.voluntad_propia.lock() {
            if let Some(vp) = guard_vp.as_mut() {
                vp.tick();
                vp.registrar_actividad();
                let impulsos = vp.generar_impulsos();
                for impulso in &impulsos {
                    info!(
                        "🧠 [VOLUNTAD PROPIA] Impulso generado: {:?} (urgencia: {:.1}, prioridad: {})",
                        impulso.tipo, impulso.urgencia, impulso.prioridad
                    );
                }
            }
        }

        // Lóbulo Occipital Estético (UI)
        let lower_final = respuesta_final.to_lowercase();
        if lower_final.contains("interfaz")
            || lower_final.contains("ui")
            || lower_final.contains("estético")
        {
            if let Some(_lobulo) = &self.lobulo_occipital {
                info!("🧠 [LÓBULO OCCIPITAL] Evaluación estética solicitada en el prompt");
                respuesta_final.push_str("\n\n🎨 [LÓBULO OCCIPITAL ESTÉTICO] He detectado que mencionas una interfaz. Puedo evaluar su estética si me proporcionas una captura de pantalla.");
            }
        }

        // ═══════════════════════════════════════════════════════════════════════
        // ETAPA 15: CIRCUITO DE CONCIENCIA — Archivo de la interacción
        // Almacena prompt + respuesta en MemoriaOperativa (Ring Buffer + FTS5).
        // Cada 10 interacciones, dispara consolidación Ebbinghaus automática.
        // ═══════════════════════════════════════════════════════════════════════
        let _ = self
            .hippocampus
            .archivar_interaccion(prompt_original, &respuesta_final);
        info!("🧠 [HIPPOCAMPUS] Interacción archivada en memoria operativa");

        // ⚡ Guardar en Cache Semántico para futura eficiencia
        self.cache
            .guardar(prompt_original, "", &respuesta_final, 3600);

        respuesta_final
    }

    /// Ejecuta el pipeline completo de 14 etapas del Orquestador pero delegando la
    /// generación del texto de respuesta basal a un ejecutor customizado (ej: el Motor Puro SNN).
    pub async fn responder_con_ejecutor<F, Fut>(&self, prompt_original: &str, ejecutor: F) -> String
    where
        F: FnOnce(String) -> Fut,
        Fut: std::future::Future<Output = String>,
    {
        let inicio = Instant::now();

        // ETAPA 0: Sistema Inmune
        if let Ok(mut inmune) = self.sistema_inmune.lock() {
            inmune.tick();
            for palabra in prompt_original.split_whitespace() {
                if palabra.starts_with("http://") || palabra.starts_with("https://") {
                    let veredicto = inmune.analizar_url(palabra);
                    if let crate::defensa::sistema_inmune::Veredicto::Peligroso(sev) = veredicto {
                        warn!(
                            "🚨 [INMUNE] URL peligrosa detectada: {} (severidad: {:.2})",
                            palabra, sev
                        );
                        return format!(
                            "🧬 **SISTEMA INMUNE DE NEXUS**\n\nHe detectado una URL potencialmente peligrosa en tu mensaje:\n`{}`\n\n**Severidad:** {:.2} — **BLOQUEADA**",
                            palabra, sev
                        );
                    }
                }
            }
        }

        // ETAPA 1: Amígdala
        let (_amenaza, estado_emocional) = self.detectar_amenaza(prompt_original);
        let lower_prompt = prompt_original.to_lowercase();

        // ETAPA 2: Intuición
        let (prompt, mut prefijo_tono) = self.aplicar_intuicion(prompt_original);

        // ETAPA 3: Teoría de la Mente
        prefijo_tono = self.analizar_teoria_mente(&prompt, &prefijo_tono);

        // ETAPA 4: Reciprocidad Emocional + Apego
        let prefijo_reciproco = self.aplicar_reciprocidad_emocional(&prompt);
        if !prefijo_reciproco.is_empty() {
            prefijo_tono = prefijo_reciproco;
        }

        // ETAPA 5: Pensamiento Humano Acelerado
        let resultado_pha = self.pensamiento_humano_acelerado(prompt_original).await;
        let prompt_str: &str = &prompt;

        // ETAPA 6: Cadena causal (KùzuDB)
        if lower_prompt.contains("por qué") || lower_prompt.contains("explicar") {
            let error_objetivo = prompt_original
                .to_lowercase()
                .replace("por qué", "")
                .replace("falló", "")
                .replace("explicar", "")
                .trim()
                .to_string();

            if !error_objetivo.is_empty() {
                if let Ok(cadena) = self.memoria_grafo.buscar_cadena_causal(&error_objetivo) {
                    if !cadena.is_empty() {
                        let cadena_formateada: Vec<String> = cadena
                            .iter()
                            .map(|(nombre, ts)| format!("{} [{}]", nombre, ts))
                            .collect();
                        return format!(
                            "🧐 he rastreado la cadena causal en mi memoria episódica:\n\n{} ➔ '{}'",
                            cadena_formateada.join(" ➔ "),
                            error_objetivo
                        );
                    }
                }
            }
        }

        // ETAPA 7: Diagnóstico de Sincronía
        if lower_prompt.contains("diagnóstico sincronía")
            || lower_prompt.contains("verificar cerebelo")
        {
            let sql_count = self.memoria_unificada.pulso.contar_registros().unwrap_or(0);
            let vector_count = self
                .memoria_semantica
                .verificar_estado_lancedb()
                .await
                .unwrap_or(0);
            return format!(
                "📊 [DIAGNÓSTICO OMEGA]\n\nSincronía de Memoria:\n- Pulso (SQLite): {} registros\n- Ocean (LanceDB): {} registros",
                sql_count, vector_count
            );
        }

        // ETAPA 8: Evaluación de recursos
        if self.reactor.debe_usar_local() && !self.juicio.dictaminar_recursos() {
            return "⚠️ Recursos de sistema insuficientes para ráfaga local.".to_string();
        }

        // ETAPA 9: Detección de intención + Ejecución
        if let Some(accion_detectada) = self.deteccion.detectar(prompt_str).await {
            if let Some(resultado) = self.medula.ejecutar_si_hay_accion(&accion_detectada) {
                let ahora = chrono::Local::now().format("%H:%M:%S").to_string();
                return format!(
                    "{}\n\n⏱ Acción automática | 🕐 Hora PY: {}",
                    resultado, ahora
                );
            }
        }

        // ETAPA 10: MCP
        // ETAPA 11: Contexto sensorial
        let realidad = self.corteza.obtener_realidad();
        let intencion = self.lobulo_temporal.clasificar_intencion(prompt_str);
        let realidad_filtrada = self.talamo.filtrar_contexto(realidad, intencion);
        let realidad_sensorial = self.propiocepcion.contexto_realidad();

        // ETAPA 12: Memoria semántica profunda
        let contexto_semantico = self.recuperar_contexto_semantico(prompt_str).await;

        // ETAPA 13: Estado Interno
        let intensidad = match estado_emocional {
            EstadoEmocional::Calma => 0.1,
            EstadoEmocional::Alerta => 0.5,
            EstadoEmocional::Miedo => 0.8,
            EstadoEmocional::RabiaSoberana => 0.9,
            EstadoEmocional::Verguenza => 0.7,
            EstadoEmocional::Orgullo => 0.8,
        };

        let (apego_nivel, minutos_ausencia, siente_ausencia) =
            if let Ok(apego_guard) = self.apego.lock() {
                (
                    apego_guard.nivel,
                    apego_guard.minutos_sin_interaccion(),
                    apego_guard.sentir_ausencia(),
                )
            } else {
                (0.5, 0.0, false)
            };

        let lecciones: Vec<String> = self
            .juicio
            .exportar_lecciones()
            .iter()
            .take(5)
            .map(|l| {
                format!(
                    "{} → {} (impacto: {:.1})",
                    l.patron, l.leccion_moral, l.impacto
                )
            })
            .collect();

        let estado_interno = crate::cerebro::nexo::nexo_core::EstadoInterno {
            emocion: estado_emocional,
            intensidad,
            confianza: 0.8,
            apego: apego_nivel,
            minutos_ausencia,
            lecciones,
            energia_creativa: 0.7,
            siente_ausencia,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        };

        let contexto_estado = self
            .contexto_emocional_nexo(&prefijo_tono, &estado_interno, prompt_str)
            .await;

        let prompt_completo = format!(
            "{}\n\n{}\n{}\n\n{}\n{}---\nArquitecto: {}",
            contexto_estado,
            realidad_filtrada,
            realidad_sensorial,
            contexto_semantico,
            resultado_pha,
            prompt_str
        );

        // ETAPA 14: Cíngulo
        if let Ok(cing_guard) = self.cingulo.lock() {
            if let Some(prediccion) = cing_guard.predecir_error(prompt_str) {
                warn!("🧠 [CÍNGULO] {}", prediccion);
            }
        }

        let prompt_envuelto = self
            .memoria_consulta
            .construir_contexto_completo(&prompt_completo);

        // Delegar la generación del texto al ejecutor customizado (SNN Local)
        let elegido = "ENGINE_PURO";
        let mut respuesta_final = ejecutor(prompt_envuelto.clone()).await;

        // Post-procesamiento
        if !prefijo_tono.is_empty() {
            respuesta_final = format!("{}{}", prefijo_tono, respuesta_final);
        }

        // Modulación de personalidad NATIVA con NexoVoz
        respuesta_final = self
            .voz_mcp
            .modular(&respuesta_final, &estado_interno)
            .await;

        // Metacognición
        let mut nivel_confianza = NivelConfianza::Moderado;
        if let Some(meta) = &self.metacognicion {
            let coherencia_interna = if self.corteza.monitor_realidad(&respuesta_final) {
                0.8
            } else {
                0.3
            };
            let confianza_eval = meta.evaluar_confianza(
                0.6,
                coherencia_interna,
                5,
                1.0,
                (prompt_str.len() as f64 / 500.0).min(1.0),
            );
            nivel_confianza = confianza_eval.nivel.clone();
            if confianza_eval.puntaje < 0.4 {
                let prefijo = meta.prefijo_verbal(&confianza_eval);
                respuesta_final = format!("{} {}", prefijo, respuesta_final);
            }
        }

        // Registrar decisión en la Narrativa Interna
        if let Ok(mut guard_narrativa) = self.narrativa_interna.lock() {
            if let Some(narrativa) = guard_narrativa.as_mut() {
                let opciones = vec![elegido.to_string()];
                let factores = vec![
                    "SNN_local".to_string(),
                    format!("metacognicion_{:?}", nivel_confianza),
                ];
                let _ = narrativa.registrar_decision(
                    prompt_str,
                    &prompt_envuelto,
                    opciones,
                    elegido,
                    "Ejecución mediante SNN nativa integrada",
                    factores,
                    0.8,
                    "respuesta generada por red de picos local",
                );
            }
        }

        let latencia = inicio.elapsed().as_millis() as u64;
        let ahora = chrono::Local::now().format("%H:%M:%S").to_string();
        let insula_estado = if let Ok(insula_guard) = self.insula.lock() {
            insula_guard.estado_interno()
        } else {
            "⚠️ [ÍNSULA] No disponible".to_string()
        };

        respuesta_final.push_str(&format!(
            "\n\n⏱ {}ms | 🕐 Hora PY: {} | {}",
            latencia, ahora, insula_estado
        ));

        respuesta_final
    }

    /// 🧬 Delega un prompt a múltiples sub-agentes (Ollama, DeepSeek, Groq) en paralelo,
    /// evalúa las respuestas con SupervisorDeCalidad y retorna la mejor.
    ///
    /// Usa `tokio::join!` para lanzar hasta 3 sub-agentes concurrentemente:
    /// - **Córtex Nativo** (mistral.rs) — modelo local soberano (reemplaza Ollama)
    /// - **DeepSeek** — cloud fallback
    /// - **Groq** — cloud fallback rápido
    ///
    /// Luego pasa todas las respuestas por `SupervisorDeCalidad::mejor_respuesta()`
    /// que aplica heurísticas (longitud, cobertura, errores, repetición, proporción)
    /// para seleccionar la respuesta con mayor confianza.
    pub async fn delegar_multi_agente(&self, prompt_original: &str) -> String {
        let inicio = Instant::now();
        info!("🧬 [MULTI_AGENTE] Iniciando orquestación paralela de 3 sub-agentes...");

        let prompt = prompt_original.to_string();

        // ─── Lanzar sub-agentes en paralelo ──────────────────────────────────────
        let (res_nativo, res_deepseek, res_groq) = tokio::join!(
            async {
                // Córtex Nativo (mistral.rs) — prioridad sobre Ollama (obsoleto)
                let cerebro_nativo =
                    crate::energia::ia_nativa::CerebroNativo::new();
                let r = match cerebro_nativo.generar_token_nativo(&prompt).await {
                    Ok(resp) if !resp.contains("warm-up") => resp,
                    _ => String::new(),
                };
                info!("[MULTI_AGENTE] Córtex Nativo completado ({} chars)", r.len());
                ("nativo".to_string(), r)
            },
            async {
                let r = self.zenith.ejecutor_deepseek(&prompt).await;
                info!("[MULTI_AGENTE] DeepSeek completado ({} chars)", r.len());
                ("deepseek".to_string(), r)
            },
            async {
                let r = self.zenith.ejecutor_groq(&prompt).await;
                info!("[MULTI_AGENTE] Groq completado ({} chars)", r.len());
                ("groq".to_string(), r)
            },
        );

        let respuestas = vec![res_nativo, res_deepseek, res_groq];

        // ─── Evaluar con SupervisorDeCalidad ────────────────────────────────────
        let supervisor = crate::cerebro::supervisor_calidad::SupervisorDeCalidad::new();
        let mejor = supervisor.mejor_respuesta(&prompt, respuestas).await;

        let latencia = inicio.elapsed().as_millis();

        if mejor.aprobado {
            info!(
                "🏆 [MULTI_AGENTE] Mejor respuesta: '{}' (confianza: {:.2}, {}ms)",
                mejor.agente, mejor.confianza, latencia
            );
            mejor.respuesta
        } else {
            warn!(
                "⚠️ [MULTI_AGENTE] Ninguna respuesta aprobada. Usando '{}' con confianza {:.2} ({}ms)",
                mejor.agente, mejor.confianza, latencia
            );
            format!("[{}] {}", mejor.agente, mejor.respuesta)
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════
// 🕵️ FORMATEADORES DE REPORTES OSINT — Funciones helper del pipeline
// ═════════════════════════════════════════════════════════════════════════

use crate::efectores::osint::hub::OsintReport;

fn formatear_reporte_dominio(reporte: &OsintReport) -> String {
    let mut output = format!("🕵️ **Reporte OSINT — Dominio:** `{}`\n\n", reporte.target);

    // Resumen
    output.push_str(&format!(
        "📊 **Resumen:** {} resultados totales\n",
        reporte.summary.total_results
    ));

    // Categorías encontradas por DorkForger
    if !reporte.summary.categories_found.is_empty() {
        output.push_str("📂 **Categorías de Dorks:** ");
        output.push_str(&reporte.summary.categories_found.join(", "));
        output.push('\n');
    }

    // Dork results
    if !reporte.dork_results.is_empty() {
        output.push_str(&format!(
            "\n🔩 **DorkForger ({} resultados):**\n",
            reporte.dork_results.len()
        ));
        for r in reporte.dork_results.iter().take(15) {
            output.push_str(&format!(
                "  • [{}] {} — [{}]({})\n",
                r.category, r.title, r.source, r.url
            ));
        }
        if reporte.dork_results.len() > 15 {
            output.push_str(&format!(
                "  ... y {} más\n",
                reporte.dork_results.len() - 15
            ));
        }
    }

    // Brave results
    if !reporte.brave_results.is_empty() {
        output.push_str(&format!(
            "\n🦁 **Brave Search ({} resultados):**\n",
            reporte.brave_results.len()
        ));
        for r in reporte.brave_results.iter().take(10) {
            output.push_str(&format!(
                "  • **{}** — {} [Ver]({})\n",
                r.title, r.snippet, r.url
            ));
        }
        if reporte.brave_results.len() > 10 {
            output.push_str(&format!(
                "  ... y {} más\n",
                reporte.brave_results.len() - 10
            ));
        }
    }

    // Tier 3: DNS
    if let Some(ref dns) = reporte.dns_info {
        output.push_str(&format!(
            "\n🌐 **DNS ({} registros):**\n",
            reporte.summary.dns_records
        ));
        if !dns.a_records.is_empty() {
            output.push_str(&format!(
                "  📍 A (IPv4): {}\n",
                dns.a_records
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !dns.aaaa_records.is_empty() {
            output.push_str(&format!(
                "  📍 AAAA (IPv6): {}\n",
                dns.aaaa_records
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !dns.mx_records.is_empty() {
            output.push_str("  📧 MX:\n");
            for mx in &dns.mx_records {
                output.push_str(&format!(
                    "    • {} (priority: {})\n",
                    mx.value,
                    mx.priority.unwrap_or(0)
                ));
            }
        }
        if !dns.ns_records.is_empty() {
            output.push_str(&format!("  🏛️ NS: {}\n", dns.ns_records.join(", ")));
        }
        if let Some(ref cname) = dns.cname {
            output.push_str(&format!("  🔗 CNAME: {}\n", cname));
        }
    }

    // Tier 3: Whois
    if let Some(ref whois) = reporte.whois_info {
        output.push_str("\n📋 **Whois:**\n");
        if let Some(ref reg) = whois.registrador {
            output.push_str(&format!("  🏢 Registrador: {}\n", reg));
        }
        if let Some(ref cre) = whois.creado {
            output.push_str(&format!("  📅 Creado: {}\n", cre));
        }
        if let Some(ref exp) = whois.expiracion {
            output.push_str(&format!("  ⏳ Expira: {}\n", exp));
        }
        if !whois.dns_servers.is_empty() {
            output.push_str(&format!(
                "  🖥️ DNS Servers: {}\n",
                whois
                    .dns_servers
                    .iter()
                    .take(4)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !whois.emails.is_empty() {
            output.push_str(&format!(
                "  📧 Emails en whois: {}\n",
                whois
                    .emails
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Tier 3: Geo
    if let Some(ref geo) = reporte.geo_info {
        output.push_str(&format!(
            "\n🌍 **Geolocalización:** {} — {}, {} ({}, {}) | ISP: {}\n",
            geo.pais, geo.ciudad, geo.region, geo.lat, geo.lon, geo.isp
        ));
    }

    // Tier 3: Puertos abiertos
    if !reporte.open_ports.is_empty() {
        output.push_str(&format!(
            "\n🔌 **Puertos abiertos ({}):**\n",
            reporte.open_ports.len()
        ));
        for p in &reporte.open_ports {
            output.push_str(&format!("  • {} ({}) — {}\n", p.port, p.service, p.state));
        }
    }

    output
}

fn formatear_reporte_usuario(reporte: &OsintReport) -> String {
    let mut output = format!("🕵️ **Reporte OSINT — Usuario:** `{}`\n\n", reporte.target);

    output.push_str(&format!(
        "📊 **Resumen:** {} perfiles encontrados, {} resultados web, {} Telegram\n\n",
        reporte.summary.social_profiles_found,
        reporte.summary.web_results,
        reporte.summary.telegram_found
    ));

    // Perfiles sociales
    if !reporte.social_profiles.is_empty() {
        output.push_str("🎯 **Perfiles en redes sociales:**\n");
        for p in &reporte.social_profiles {
            if p.exists {
                output.push_str(&format!(
                    "  ✅ **{}** — [{}]({})\n",
                    p.platform, p.url, p.url
                ));
            }
        }

        let no_encontrados: Vec<_> = reporte
            .social_profiles
            .iter()
            .filter(|p| !p.exists)
            .collect();
        if !no_encontrados.is_empty() {
            output.push_str(&format!(
                "\n  ❌ No encontrado en: {}\n",
                no_encontrados
                    .iter()
                    .map(|p| p.platform.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }

    // Tier 3: Telegram
    if !reporte.telegram_users.is_empty() {
        output.push_str("\n📱 **Telegram:**\n");
        for u in &reporte.telegram_users {
            if u.exists {
                output.push_str(&format!("  ✅ **@{}** ({})", u.username, u.tipo));
                if let Some(ref name) = u.nombre_mostrado {
                    output.push_str(&format!(" — {}", name));
                }
                if let Some(ref m) = u.miembros {
                    output.push_str(&format!(" — 👥 {} miembros", m));
                }
                output.push('\n');
            } else {
                output.push_str(&format!("  ❌ @{} — No encontrado\n", u.username));
            }
        }
    }

    output
}

fn formatear_reporte_email(reporte: &OsintReport) -> String {
    let mut output = format!("🕵️ **Reporte OSINT — Email:** `{}`\n\n", reporte.target);

    output.push_str(&format!(
        "📊 **Resumen:** {} resultados totales\n\n",
        reporte.summary.total_results
    ));

    if !reporte.web_results.is_empty() {
        output.push_str("🌐 **Resultados web:**\n");
        for r in reporte.web_results.iter().take(10) {
            output.push_str(&format!(
                "  • **{}** — {} [Ver]({})\n",
                r.title, r.snippet, r.url
            ));
        }
    }

    if !reporte.dork_results.is_empty() {
        output.push_str(&format!(
            "\n🔩 **Dorks del dominio asociado ({} resultados):**\n",
            reporte.dork_results.len()
        ));
        for r in reporte.dork_results.iter().take(10) {
            output.push_str(&format!(
                "  • [{}] {} — [{}]({})\n",
                r.category, r.title, r.source, r.url
            ));
        }
    }

    // Tier 3: Subdominios
    if !reporte.subdomains.is_empty() {
        output.push_str(&format!(
            "\n🔗 **Subdominios encontrados ({}):**\n",
            reporte.subdomains.len()
        ));
        for s in reporte.subdomains.iter().take(10) {
            output.push_str(&format!("  • {} (fuente: {})\n", s.name, s.source));
        }
        if reporte.subdomains.len() > 10 {
            output.push_str(&format!("  ... y {} más\n", reporte.subdomains.len() - 10));
        }
    }

    output
}
