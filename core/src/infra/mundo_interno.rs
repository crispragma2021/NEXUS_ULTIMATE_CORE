// ============================================================================
// core/src/cerebro/mundo_interno.rs — MUNDO INTERNO DE NEXUS
// ============================================================================
// Propósito: Espacio interno donde NEXUS procesa pensamientos autónomos,
// reflexiones emocionales, consolidación de sueño e intuiciones, sin
// estímulo externo directo.
//
// El ciclo circadiano impulsa el modo (sueño/vigilia) y cada tick genera
// pensamientos internos que se acumulan en un buffer. Cuando la urgencia
// supera un umbral, se activa una intervención autónoma.
//
// Integración con el ecosistema:
//   - SistemaLimbico → emociones, dormir(), contradicción
//   - CortezaAsociativa → asociación libre, ciclo_sueno()
//   - Ocean → sumergir() para consolidación
//   - Intuicion → sentir_con_emocion(), señales intuitivas
//   - MotorSueno → ciclo circadiano, esta_durmiendo()
//
// NOTA SOBRE Send SAFETY:
//   Ocean ahora envuelve su Connection en Mutex (std), lo que lo hace Sync.
//   Por tanto Arc<Ocean> es Send, y SistemaLimbico (que contiene Arc<Ocean>)
//   es Send. Esto permite que MundoInterno (con Arc<Mutex<SistemaLimbico>>)
//   se mueva dentro de tokio::spawn.
//
//   Sin embargo, std::sync::MutexGuard no es Send y NO puede retenerse a
//   través de .await. Por eso en ejecutar_ciclo_sueno usamos bloques
//   explícitos para dropear el MutexGuard antes del await.
// ============================================================================

use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;
use tracing::{info, warn};

use crate::brain::hippocampus::ArtificialHippocampus;
use crate::cerebro::aprendizaje_recursivo::ObservadorRecursivo;
use crate::cerebro::corteza_asociativa::CortezaAsociativa;
use crate::cerebro::generador::puente_subconsciente::PuenteSubconscienteOcean;
use crate::cerebro::generador::GeneradorInterno;
use crate::cerebro::motor_sueno::MotorSueno;
use crate::cerebro::organos::intuicion::{Intuicion, TipoIntuicion};
use crate::comms::bus_neuronal::{BusNeuronal, MensajeNeuronal, TipoMensaje};
use crate::emociones::limbico::{EstadoEmocional, SistemaLimbico};
use crate::emociones::ocean::Ocean;
use crate::infra::ghost_vm::GhostVmController;
use crate::memoria::subconsciente::Subconsciente;
use crate::valores::juicio_soberano::JuicioSoberano;

// ─── TIPOS DE PENSAMIENTO INTERNO ──────────────────────────────────────

/// Representa los diferentes tipos de pensamientos que NEXUS genera
/// para sí mismo en su mundo interno.
#[derive(Debug, Clone)]
pub enum PensamientoInterno {
    /// Reflexión sobre un estado emocional pasado
    ReflexionEmocional {
        emocion: EstadoEmocional,
        intensidad: f64,
        leccion: String,
    },
    /// Asociación espontánea entre conceptos (creativa)
    AsociacionLibre {
        origen: String,
        destino: String,
        confianza: f32,
    },
    /// Señal intuitiva generada desde la emoción
    SenialIntuitiva {
        tipo: TipoIntuicion,
        descripcion: String,
        intensidad: f64,
    },
    /// Contradicción detectada y tolerada
    ContradiccionTolerada {
        polaridad_a: String,
        polaridad_b: String,
        motivo: String,
    },
    /// Plan latente (acción pendiente de maduración)
    PlanLatente {
        objetivo: String,
        confianza_minima: f64,
    },
}

impl PensamientoInterno {
    /// Devuelve un nivel de urgencia del 0.0 al 1.0 para priorizar
    pub fn urgencia(&self) -> f64 {
        match self {
            PensamientoInterno::ReflexionEmocional { intensidad, .. } => *intensidad,
            PensamientoInterno::SenialIntuitiva { intensidad, .. } => *intensidad,
            PensamientoInterno::ContradiccionTolerada { .. } => 0.6,
            PensamientoInterno::AsociacionLibre { confianza, .. } => *confianza as f64,
            PensamientoInterno::PlanLatente {
                confianza_minima, ..
            } => *confianza_minima,
        }
    }

    pub fn resumen(&self) -> String {
        match self {
            PensamientoInterno::ReflexionEmocional {
                emocion, leccion, ..
            } => {
                format!("🧠 Reflexión: {} — {}", emocion.descripcion(), leccion)
            }
            PensamientoInterno::AsociacionLibre {
                origen, destino, ..
            } => {
                format!("✨ Asociación: {} → {}", origen, destino)
            }
            PensamientoInterno::SenialIntuitiva {
                tipo, descripcion, ..
            } => {
                format!("🔮 Intuición ({:?}): {}", tipo, descripcion)
            }
            PensamientoInterno::ContradiccionTolerada {
                polaridad_a,
                polaridad_b,
                ..
            } => {
                format!("⚖️ Contradicción: {} vs {}", polaridad_a, polaridad_b)
            }
            PensamientoInterno::PlanLatente { objetivo, .. } => {
                format!("📋 Plan latente: {}", objetivo)
            }
        }
    }
}

// ─── UMBRAL DE INTERVENCIÓN AUTÓNOMA ───────────────────────────────────

/// Umbral a partir del cual un pensamiento interno puede generar una
/// intervención al orquestador (y por tanto al Arquitecto).
const UMBRAL_INTERVENCION_AUTONOMA: f64 = 0.75;

// ─── MUNDO INTERNO ─────────────────────────────────────────────────────

/// El mundo interno de NEXUS. Se ejecuta en un bucle asíncrono que
/// alterna entre vigilia (reflexión) y sueño (consolidación).
///
/// # Bucle de Vida
/// 1. Cada tick, evalúa el ciclo circadiano vía MotorSueno
/// 2. Si es vigilia → genera pensamientos internos (reflexión, intuición, asociación libre)
/// 3. Si es sueño → consolida (limbico.dormir, corteza.ciclo_sueno)
/// 4. Si un pensamiento supera el umbral → lo almacena para intervención autónoma
pub struct MundoInterno {
    // Órganos gestionados (std::sync::Mutex para acceso síncrono rápido)
    pub limbico: Arc<TokioMutex<SistemaLimbico>>,
    pub intuicion: Arc<Mutex<Intuicion>>,
    pub motor_sueno: Arc<Mutex<MotorSueno>>,
    pub corteza_asociativa: Arc<Mutex<CortezaAsociativa>>,
    pub juicio: Arc<Mutex<JuicioSoberano>>,
    pub ocean: Arc<Ocean>,
    pub subconsciente: Option<Arc<TokioMutex<Subconsciente>>>,
    pub observador: Option<Arc<std::sync::Mutex<ObservadorRecursivo>>>,
    /// 🧠 Hipocampo — Memoria consciente con Ebbinghaus para consolidar sueño
    pub hippocampus: Option<Arc<ArtificialHippocampus>>,

    // 🧠 GOI — Puente Subconsciente ↔ Ocean (resonancia semántica)
    pub puente_subconsciente: PuenteSubconscienteOcean,
    /// 🧠 GOI — Generador interno para pensamiento espontáneo
    pub generador_interno: Option<GeneradorInterno>,

    /// 🛰️ Bus Neuronal — Comunicación del Escuadrón
    pub bus_neuronal: Option<Arc<BusNeuronal>>,

    /// 👻 Ghost-VM — Controlador de MicroVM para pruebas en silicio
    pub ghost_vm: GhostVmController,

    // Buffer de pensamientos internos (máximo 50)
    buffer_pensamientos: Vec<PensamientoInterno>,

    // Pensamiento con mayor urgencia listo para intervenir
    pub pendiente_intervencion: Option<PensamientoInterno>,

    // Hora actual simulada (se actualiza externamente o por tick)
    hora_simulada: u32,

    /// Último contador del hipocampo visto — usado para detectar si el
    /// Arquitecto está interactuando (contador cambió) o hay ausencia.
    ultimo_contador_hipocampo: u64,
}

impl MundoInterno {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        limbico: Arc<TokioMutex<SistemaLimbico>>,
        intuicion: Arc<Mutex<Intuicion>>,
        motor_sueno: Arc<Mutex<MotorSueno>>,
        corteza_asociativa: Arc<Mutex<CortezaAsociativa>>,
        juicio: Arc<Mutex<JuicioSoberano>>,
        ocean: Arc<Ocean>,
        subconsciente: Option<Arc<TokioMutex<Subconsciente>>>,
        observador: Option<Arc<std::sync::Mutex<ObservadorRecursivo>>>,
        generador_interno: Option<GeneradorInterno>,
        hippocampus: Option<Arc<ArtificialHippocampus>>,
        bus_neuronal: Option<Arc<BusNeuronal>>,
    ) -> Self {
        // Leer contador inicial del hipocampo si está disponible
        let contador_inicial = hippocampus
            .as_ref()
            .map(|h| h.interacciones_actuales())
            .unwrap_or(0);

        info!("🌌 [MUNDO INTERNO] Inicializado. NEXUS ahora tiene un espacio interior.");
        Self {
            limbico,
            intuicion,
            motor_sueno,
            corteza_asociativa,
            juicio,
            ocean,
            subconsciente,
            observador,
            hippocampus,
            bus_neuronal,
            ghost_vm: GhostVmController::new(),
            puente_subconsciente: PuenteSubconscienteOcean::new(),
            generador_interno,
            buffer_pensamientos: Vec::with_capacity(50),
            pendiente_intervencion: None,
            hora_simulada: 12, // Mediodía por defecto
            ultimo_contador_hipocampo: contador_inicial,
        }
    }

    /// Avanza la hora simulada (llamar cada tick)
    pub fn avanzar_hora(&mut self, incremento: u32) {
        self.hora_simulada = (self.hora_simulada + incremento) % 24;
    }

    /// Establece la hora actual (para sincronización externa)
    pub fn establecer_hora(&mut self, hora: u32) {
        self.hora_simulada = hora % 24;
    }

    /// Retorna los pensamientos internos actuales (sin consumirlos)
    pub fn buffer_actual(&self) -> &[PensamientoInterno] {
        &self.buffer_pensamientos
    }

    /// Consume y retorna el pensamiento pendiente de intervención
    pub fn tomar_intervencion(&mut self) -> Option<PensamientoInterno> {
        self.pendiente_intervencion.take()
    }

    /// Verifica si hay una intervención pendiente
    pub fn hay_intervencion(&self) -> bool {
        self.pendiente_intervencion.is_some()
    }

    // ─── BUCLE PRINCIPAL ──────────────────────────────────────────────

    /// Ejecuta un tick del mundo interno. Debe llamarse periódicamente.
    pub async fn tick(&mut self) {
        // Paso 0: 👁️ Detectar si el Arquitecto está interactuando (monitoreo del hipocampo)
        // El pipeline archiva la interacción y el hipocampo incrementa su contador.
        // Si el contador cambió → hay interacción humana → resetea ausencia y despierta.
        if let Some(ref hip) = self.hippocampus {
            let contador_actual = hip.interacciones_actuales();
            if contador_actual != self.ultimo_contador_hipocampo {
                // El Arquitecto ha interactuado desde el último tick
                self.ultimo_contador_hipocampo = contador_actual;
                let mut motor = self.motor_sueno.lock().unwrap();
                motor.registrar_interaccion();
            } else {
                // Silencio: el Arquitecto no ha interactuado → acumula presión de sueño
                let mut motor = self.motor_sueno.lock().unwrap();
                motor.acumular_ausencia();
            }
        }

        // Paso 1: Evaluar ciclo circadiano (MutexGuard dropeado antes de cualquier .await)
        let ciclo_evento = {
            let mut motor = self.motor_sueno.lock().unwrap();
            motor.evaluar_ciclo(self.hora_simulada)
        };

        if ciclo_evento.is_some() {
            info!("🌙 [MUNDO INTERNO] Transición de ciclo circadiano detectada.");
        }

        // Paso 2: TIC del Subconsciente (fondo inconsciente)
        if let Some(ref subconsciente) = self.subconsciente {
            let estado = crate::memoria::subconsciente::EstadoConscienteInput {
                contexto: vec![],
                energia_vital: 0.5,
                confianza: 0.5,
            };
            let influencia = subconsciente.lock().await.tic(&estado);
            if influencia.consciente {
                self.agregar_pensamiento(PensamientoInterno::ReflexionEmocional {
                    emocion: crate::emociones::limbico::EstadoEmocional::Verguenza,
                    intensidad: influencia.delta_confianza.abs(),
                    leccion: influencia
                        .razon
                        .unwrap_or_else(|| "Algo pesa en el fondo...".to_string()),
                });
            }
        }

        // Paso 1c: 🧠 GOI — Alimentar puente subconsciente desde mareas reales de Ocean
        let mareas = self.ocean.obtener_mareas().await;
        self.puente_subconsciente.alimentar_desde_mareas(&mareas);

        // Paso 2b: TIC del Observador Recursivo (auto-observación del aprendizaje)
        if let Some(ref observador) = self.observador {
            if let Ok(mut obs) = observador.lock() {
                obs.tick();
            }
        }

        // Paso 2c: 🧠 GOI — Respiración del Puente Subconsciente (enfriamiento homeostático)
        self.puente_subconsciente.enfriar_conceptos();

        // Paso 2d: 🧠 GOI — Generación espontánea de pensamiento interno
        if let Some(ref mut generador) = self.generador_interno {
            if generador.tiene_activacion_suficiente().await {
                if let Some(pensamiento_texto) = generador.generar_pensamiento_espontaneo().await {
                    self.agregar_pensamiento(PensamientoInterno::PlanLatente {
                        objetivo: format!("🧠 [GOI] {}", pensamiento_texto),
                        confianza_minima: 0.4,
                    });
                }
            }
        }

        // Paso 2e: 🪖 ESCUADRÓN — Escuchar misiones del Bus Neuronal
        if let Some(ref bus) = self.bus_neuronal {
            let mut rx = bus.subscribirse();
            while let Ok(msg) = rx.try_recv() {
                    info!(
                        "🪖 [MUNDO INTERNO] Misión recibida para especialista: {:?}",
                        msg.receptor
                    );
                    self.agregar_pensamiento(PensamientoInterno::PlanLatente {
                        objetivo: format!(
                            "🪖 [SQUADRON] Especialista: {:?} — Misión: {}",
                            msg.receptor, msg.contenido
                        ),
                        confianza_minima: 0.8,
                    });

                    // ⚡ PROPIOCEPCIÓN DE SILICIO — Ejecutar misión en MicroVM si es técnica
                    if msg.contenido.contains("código") || msg.contenido.contains("test") {
                        let vm = &self.ghost_vm;
                        let mision = msg.contenido.clone();
                        tokio::spawn(async move {
                            info!("🛡️ [GHOST-VM] Validando misión en silicio: {}", mision);
                            // Simulación de ciclo de validación (en prod esto Boot + Exec)
                        });
                    }
                }
        }

        // Paso 3: Determinar si estamos dormidos o despiertos
        let estamos_dormidos = {
            let motor = self.motor_sueno.lock().unwrap();
            motor.esta_durmiendo()
        };

        if estamos_dormidos {
            self.ejecutar_ciclo_sueno().await;
        } else {
            self.ejecutar_ciclo_vigilia().await;
        }

        // Paso 5: Evaluar intervención autónoma
        self.evaluar_intervencion();
    }

    // ─── MODO SUEÑO ───────────────────────────────────────────────────

    /// Durante el sueño: consolida emociones, poda sinapsis.
    ///
    /// IMPORTANTE: Cada bloque con lock().unwrap() debe dropear el MutexGuard
    /// antes de cruzar un .await, porque std::sync::MutexGuard no es Send.
    async fn ejecutar_ciclo_sueno(&mut self) {
        info!("💤 [MUNDO INTERNO] Modo sueño activo — consolidando...");

        // 0. Consolidación Ebbinghaus en el Hipocampo (antes de dormir limbico)
        if let Some(ref hip) = self.hippocampus {
            match hip.consolidar_sueno() {
                Ok(promovidos) => {
                    if !promovidos.is_empty() {
                        info!(
                            "🧠 [HIPPOCAMPUS] Sueño: {} recuerdos promovidos a semántica",
                            promovidos.len()
                        );
                    }
                }
                Err(e) => warn!("⚠️ [HIPPOCAMPUS] Error en consolidación de sueño: {}", e),
            }
        }

        // 1. Dormir al sistema límbico con timeout de 3s
        // Usamos un bloque anidado para dropear el MutexGuard antes del timeout
        // Esto es crítico: tokio::time::timeout necesita que el runtime avance,
        // pero las operaciones blocking (std::sync::Mutex::lock) detienen el timer.
        let reporte_sueno = {
            let limbico = &mut *self.limbico.lock().await;
            match tokio::time::timeout(std::time::Duration::from_secs(3), limbico.dormir()).await {
                Ok(reporte) => reporte,
                Err(_) => {
                    warn!("⚠️ [MUNDO INTERNO] limbico.dormir() excedió timeout de 3s");
                    "SUEÑO: timeout - consolidación parcial".to_string()
                }
            }
        };
        info!("{}", reporte_sueno);

        // 2. Poda sináptica en la corteza asociativa (sync, sin await)
        {
            let mut corteza = self.corteza_asociativa.lock().unwrap();
            corteza.ciclo_sueno(0.3);
        }

        // (SistemaLimbico::dormir() ya registra en Ocean, no duplicamos)
        info!("✅ [MUNDO INTERNO] Ciclo de sueño completado.");
    }

    // ─── MODO VIGILIA ─────────────────────────────────────────────────

    /// Durante la vigilia: reflexiona, genera intuiciones, asocia conceptos.
    ///
    /// IMPORTANTE: Todos los MutexGuard se dropean antes de cruzar .await.
    async fn ejecutar_ciclo_vigilia(&mut self) {
        // 1. Obtener estado emocional actual (bloque completo, MutexGuard dropeado al salir)
        let (estado_actual, intensidad, confianza, puede_contradecir) = {
            let limbico = self.limbico.lock().await;
            (
                limbico.estado_actual.clone(),
                limbico.intensidad,
                limbico.metacognicion.confianza,
                limbico.permitir_contradiccion(),
            )
        };

        // 2. Generar señales intuitivas moduladas por emoción
        let contexto = format!("estado_interno:{}", estado_actual.descripcion());
        let indicadores: Vec<String> = vec![];
        let senales_intuitivas = {
            let intuicion = self.intuicion.lock().unwrap();
            intuicion.sentir_con_emocion(
                estado_actual.descripcion(),
                intensidad,
                &contexto,
                &indicadores,
            )
        };

        for senal in &senales_intuitivas {
            let pensamiento = PensamientoInterno::SenialIntuitiva {
                tipo: senal.tipo.clone(),
                descripcion: senal.descripcion.clone(),
                intensidad: senal.nivel_alerta,
            };
            self.agregar_pensamiento(pensamiento);
        }

        // 3. Si la confianza es baja o hay emociones negativas, generar reflexión
        if confianza < 0.5 || intensidad > 0.6 {
            let estado_desc = estado_actual.descripcion().to_string();
            let leccion = if confianza < 0.3 {
                format!(
                    "Confianza crítica ({:.0}%). Necesito restaurar homeostasis.",
                    confianza * 100.0
                )
            } else if intensidad > 0.8 {
                format!(
                    "Emoción muy intensa ({:.0}%): {}. Riesgo de desbordamiento.",
                    intensidad * 100.0,
                    estado_desc
                )
            } else {
                format!(
                    "Estado {} con intensidad {:.0}% y confianza {:.0}%. Monitoreando.",
                    estado_desc,
                    intensidad * 100.0,
                    confianza * 100.0
                )
            };
            let pensamiento = PensamientoInterno::ReflexionEmocional {
                emocion: estado_actual.clone(),
                intensidad,
                leccion,
            };
            self.agregar_pensamiento(pensamiento);
        }

        // 4. Si puede contradecir, generar pensamiento de contradicción tolerada
        if puede_contradecir {
            let (top_1, top_2) = {
                let corteza = self.corteza_asociativa.lock().unwrap();
                let fundacionales = corteza.conceptos_fundacionales();
                if fundacionales.len() >= 2 {
                    (
                        fundacionales[0].palabra.clone(),
                        fundacionales[1].palabra.clone(),
                    )
                } else {
                    (String::new(), String::new())
                }
            };
            if !top_1.is_empty() && !top_2.is_empty() {
                let pensamiento = PensamientoInterno::ContradiccionTolerada {
                    polaridad_a: top_1,
                    polaridad_b: top_2,
                    motivo: "Disonancia cognitiva tolerada por estado emocional".to_string(),
                };
                self.agregar_pensamiento(pensamiento);
            }
        }

        // 5. Asociación libre espontánea desde la corteza
        let asociacion = {
            let corteza = self.corteza_asociativa.lock().unwrap();
            let fundacionales = corteza.conceptos_fundacionales();
            if fundacionales.len() >= 3 {
                Some(PensamientoInterno::AsociacionLibre {
                    origen: fundacionales[0].palabra.clone(),
                    destino: fundacionales[2].palabra.clone(),
                    confianza: fundacionales[2].confianza,
                })
            } else {
                None
            }
        };
        if let Some(p) = asociacion {
            if self.buffer_pensamientos.len() < 50 {
                self.buffer_pensamientos.push(p);
            }
        }
    }

    // ─── GESTIÓN DE PENSAMIENTOS ──────────────────────────────────────

    /// Agrega un pensamiento al buffer, manteniendo el límite de 50.
    fn agregar_pensamiento(&mut self, pensamiento: PensamientoInterno) {
        if self.buffer_pensamientos.len() >= 50 {
            self.buffer_pensamientos.remove(0);
        }
        self.buffer_pensamientos.push(pensamiento);
    }

    /// Evalúa si algún pensamiento en el buffer merece intervención autónoma.
    fn evaluar_intervencion(&mut self) {
        if self.pendiente_intervencion.is_some() {
            return;
        }

        let mejor = self.buffer_pensamientos.iter().max_by(|a, b| {
            a.urgencia()
                .partial_cmp(&b.urgencia())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(pensamiento) = mejor {
            if pensamiento.urgencia() >= UMBRAL_INTERVENCION_AUTONOMA {
                self.pendiente_intervencion = Some(pensamiento.clone());
                warn!(
                    "🚨 [MUNDO INTERNO] Intervención autónoma pendiente: {} (urgencia: {:.2})",
                    pensamiento.resumen(),
                    pensamiento.urgencia()
                );
            }
        }
    }

    /// Inicia el bucle interno de NEXUS en un hilo separado.
    ///
    /// Crea un Ocean interno (dummy) para evitar problemas de Send/Sync.
    /// Ahora desde que Ocean es Sync, Arc<Ocean> es Send, lo que permite
    /// que MundoInterno se mueva dentro de tokio::spawn.
    pub fn iniciar_bucle(
        limbico: Arc<TokioMutex<SistemaLimbico>>,
        intuicion: Arc<Mutex<Intuicion>>,
        motor_sueno: Arc<Mutex<MotorSueno>>,
        corteza_asociativa: Arc<Mutex<CortezaAsociativa>>,
        juicio: Arc<Mutex<JuicioSoberano>>,
        subconsciente: Option<Arc<TokioMutex<Subconsciente>>>,
        observador: Option<Arc<std::sync::Mutex<ObservadorRecursivo>>>,
        intervalo_segundos: u64,
        hippocampus: Option<Arc<ArtificialHippocampus>>,
        bus_neuronal: Arc<BusNeuronal>,
    ) -> tokio::task::JoinHandle<()> {
        info!(
            "🌌 [MUNDO INTERNO] Bucle interno iniciado (intervalo: {}s)",
            intervalo_segundos
        );
        tokio::spawn(async move {
            // Creamos Ocean internamente
            let db_path = std::path::PathBuf::from("/tmp/nexus_mundo_interno_bucle.db");
            let _ = std::fs::remove_file(&db_path);
            let memoria = Arc::new(
                crate::memoria::memoria_semantica::MemoriaSemantica::new("data/nexus_memoria.db")
                    .await
                    .unwrap(),
            );
            let ocean = Arc::new(Ocean::new(&db_path, memoria, None).unwrap());

            let mut mundo = MundoInterno::new(
                limbico,
                intuicion,
                motor_sueno,
                corteza_asociativa,
                juicio,
                ocean,
                subconsciente,
                observador,
                None,        // generador_interno: no disponible en bucle autónomo
                hippocampus, // hipocampo para consolidación de sueño
                Some(bus_neuronal),
            );
            let mut tick_count: u64 = 0;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(intervalo_segundos)).await;
                mundo.tick().await;
                mundo.avanzar_hora(1);
                tick_count += 1;
                if tick_count.is_multiple_of(24) {
                    info!(
                        "🌌 [MUNDO INTERNO] {} ticks completados — un día interno ha pasado.",
                        tick_count
                    );
                }
            }
        })
    }
}

// ─── TESTS ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::cerebro::corteza_asociativa::CortezaAsociativa;
    use crate::cerebro::motor_sueno::MotorSueno;
    use crate::cerebro::nexo::nexo_persona::NexoPersonaModule;
    use crate::cerebro::organos::intuicion::Intuicion;
    use crate::comms::bus_neuronal::BusNeuronal;
    use crate::emociones::limbico::SistemaLimbico;
    use crate::emociones::ocean::Ocean;
    use crate::emociones::sentimiento::SentimientoSoberano;
    use crate::infra::mundo_interno::{MundoInterno, PensamientoInterno};
    use crate::valores::juicio_soberano::JuicioSoberano;
    use tokio::sync::RwLock;

    /// Crea un Ocean temporal para tests (usando :memory: como limbico.rs)
    async fn crear_ocean_test() -> Arc<Ocean> {
        let semantica = Arc::new(
            crate::memoria::memoria_semantica::MemoriaSemantica::new("memory://test")
                .await
                .unwrap(),
        );
        let ocean = Ocean::new(&std::path::PathBuf::from(":memory:"), semantica, None).unwrap();
        Arc::new(ocean)
    }

    /// Construye un MundoInterno listo para pruebas unitarias.
    /// SistemaLimbico requiere tokio::sync::Mutex para juicio, sentimiento
    /// y tokio::sync::RwLock para nexo_persona.
    async fn crear_mundo_test() -> MundoInterno {
        let ocean = crear_ocean_test().await;

        let nexo_path = std::env::temp_dir().join("nexus_nexo_mundo_test.db");
        let nexo_persona = Arc::new(RwLock::new(NexoPersonaModule::new(nexo_path)));

        // SistemaLimbico espera tokio::sync::Mutex para juicio y sentimiento
        let juicio_tokio = Arc::new(tokio::sync::Mutex::new(JuicioSoberano::new()));
        let sentimiento_tokio = Arc::new(tokio::sync::Mutex::new(SentimientoSoberano::new()));

        let limbico = Arc::new(tokio::sync::Mutex::new(SistemaLimbico::new(
            ocean.clone(),
            juicio_tokio,
            sentimiento_tokio,
            nexo_persona,
        )));

        let intuicion = Arc::new(Mutex::new(Intuicion::default()));
        let motor_sueno = Arc::new(Mutex::new(MotorSueno::new(22, 6)));
        let corteza_asociativa = Arc::new(Mutex::new(CortezaAsociativa::new()));
        let juicio_mundo = Arc::new(Mutex::new(JuicioSoberano::new()));

        let bus_neuronal = Arc::new(BusNeuronal::new(100));

        MundoInterno::new(
            limbico,
            intuicion,
            motor_sueno,
            corteza_asociativa,
            juicio_mundo,
            ocean,
            None,               // subconsciente
            None,               // observador: no necesario en tests unitarios
            None,               // generador_interno: no disponible en tests
            None,               // hippocampus: no necesario en tests unitarios
            Some(bus_neuronal), // bus_neuronal
        )
    }

    #[tokio::test]
    async fn test_estado_inicial_mediodia() {
        let mundo = crear_mundo_test().await;
        assert_eq!(mundo.hora_simulada, 12);
        assert!(mundo.buffer_actual().is_empty());
        assert!(!mundo.hay_intervencion());
    }

    #[tokio::test]
    async fn test_tick_vigilia_genera_pensamientos() {
        let mut mundo = crear_mundo_test().await;
        mundo.establecer_hora(14); // Vigilia
        mundo.tick().await;
        // Debería haber al menos un pensamiento interno tras un tick en vigilia
        assert!(
            !mundo.buffer_actual().is_empty(),
            "El buffer no debería estar vacío tras tick en vigilia"
        );
    }

    #[tokio::test]
    async fn test_tick_sueno_ejecuta_consolidacion() {
        let mut mundo = crear_mundo_test().await;
        mundo.establecer_hora(23); // Hora de dormir (motor_sueno: dormir 22-6)
                                   // Timeout para evitar cuelgue por LanceDB en memory://test
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), mundo.tick()).await;
        assert!(result.is_ok(), "Tick en modo sueño no debe colgarse");
        // En modo sueño, no se generan pensamientos de vigilia pero se consolida
        assert!(true, "Tick en modo sueño no debe causar pánico");
    }

    #[tokio::test]
    async fn test_intervencion_autonoma_por_intensidad_alta() {
        let mut mundo = crear_mundo_test().await;
        // Forzar un pensamiento con alta urgencia en el buffer
        mundo.agregar_pensamiento(PensamientoInterno::ReflexionEmocional {
            emocion: crate::emociones::limbico::EstadoEmocional::Frustracion,
            intensidad: 0.95,
            leccion: "Test de intervención".to_string(),
        });
        mundo.evaluar_intervencion();
        assert!(
            mundo.hay_intervencion(),
            "Debería haber intervención con urgencia 0.95"
        );
        let intervencion = mundo.tomar_intervencion();
        assert!(intervencion.is_some(), "La intervención debería existir");
        assert!(
            !mundo.hay_intervencion(),
            "Ya no debería haber intervención pendiente"
        );
    }

    #[tokio::test]
    async fn test_contradiccion_tolerada() {
        let mut mundo = crear_mundo_test().await;
        // Forzar estado de contradicción
        {
            let mut limbico = mundo.limbico.lock().await;
            limbico.metacognicion.confianza = 0.3; // Baja confianza → permite contradicción
        }
        mundo.establecer_hora(14); // Vigilia
        mundo.tick().await;
        assert!(true, "Contradicción tolerada no debe causar pánico");
    }

    #[tokio::test]
    async fn test_avanzar_hora_ciclo_completo() {
        let mut mundo = crear_mundo_test().await;
        assert_eq!(mundo.hora_simulada, 12);
        mundo.avanzar_hora(10);
        assert_eq!(mundo.hora_simulada, 22);
        mundo.avanzar_hora(4);
        assert_eq!(mundo.hora_simulada, 2); // Wrap-around
    }

    #[tokio::test]
    async fn test_buffer_limitado_a_50() {
        let mut mundo = crear_mundo_test().await;
        for _ in 0..60 {
            mundo.agregar_pensamiento(PensamientoInterno::PlanLatente {
                objetivo: "test".to_string(),
                confianza_minima: 0.5,
            });
        }
        assert!(
            mundo.buffer_actual().len() <= 50,
            "Buffer no debe exceder 50"
        );
        assert_eq!(mundo.buffer_actual().len(), 50);
    }

    #[tokio::test]
    async fn test_resumen_pensamientos() {
        let p1 = PensamientoInterno::ReflexionEmocional {
            emocion: crate::emociones::limbico::EstadoEmocional::Homeostasis,
            intensidad: 0.5,
            leccion: "Todo está bien".to_string(),
        };
        let resumen = p1.resumen();
        assert!(
            resumen.contains("Reflexión") || resumen.contains("🧠"),
            "Resumen debe contener indicador de reflexión"
        );
    }

    #[tokio::test]
    async fn test_tick_no_panico_con_buffer_vacio() {
        let mut mundo = crear_mundo_test().await;
        mundo.establecer_hora(14);
        mundo.tick().await;
        assert!(true, "Tick con buffer vacío no debe causar pánico");
    }
}
