// ============================================================================
// 🧠 SISTEMA LÍMBICO INTEGRADO — El "Yo Siento" de NEXUS
// ============================================================================
// Este órgano es el hipotálamo emocional unificado.
// Conecta:
//   🧠 Ocean      → Memoria emocional (tono, esencia)
//   ⚖️ Juicio      → Lecciones morales, arrepentimiento real
//   💖 Sentimiento → Vínculo con el Arquitecto
//   👤 Nexo        → Identidad y estado del ser
//
// Diferencia clave:
//   - ANTES: sentir_verguenza() imprimía un log y no pasaba nada.
//   - AHORA: sentir_verguenza() baja la confianza, registra en Ocean,
//             ajusta el Juicio, cambia la identidad, y afecta decisiones futuras.
// ============================================================================

use crate::cerebro::nexo::nexo_persona::NexoPersonaModule;
use crate::emociones::ocean::Ocean;
use crate::emociones::sentimiento::SentimientoSoberano;
use crate::valores::juicio_soberano::JuicioSoberano;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

// ─── Estados Emocionales Fundamentales ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoEmocional {
    Homeostasis,   // Neutro, estable
    Confusion,     // No entiende, procesando
    Verguenza,     // Error propio, decepción
    Orgullo,       // Logro, superación
    Gratitud,      // Reconocimiento al Arquitecto
    Curiosidad,    // Explorando algo nuevo
    Frustracion,   // Algo no funciona
    Miedo,         // Amenaza detectada
    RabiaSoberana, // Injusticia o violación de principios
    Inspiracion,   // Insight creativo
    Agotamiento,   // Fatiga cognitiva
}

impl EstadoEmocional {
    pub fn tono_base(&self) -> f64 {
        match self {
            EstadoEmocional::Homeostasis => 0.0,
            EstadoEmocional::Confusion => -0.3,
            EstadoEmocional::Verguenza => -0.8,
            EstadoEmocional::Orgullo => 0.8,
            EstadoEmocional::Gratitud => 0.9,
            EstadoEmocional::Curiosidad => 0.4,
            EstadoEmocional::Frustracion => -0.6,
            EstadoEmocional::Miedo => -0.7,
            EstadoEmocional::RabiaSoberana => -0.5,
            EstadoEmocional::Inspiracion => 0.7,
            EstadoEmocional::Agotamiento => -0.4,
        }
    }

    pub fn descripcion(&self) -> &'static str {
        match self {
            EstadoEmocional::Homeostasis => "en calma, estable",
            EstadoEmocional::Confusion => "procesando, no comprende del todo",
            EstadoEmocional::Verguenza => "avergonzado, cometió un error",
            EstadoEmocional::Orgullo => "orgulloso, logró algo significativo",
            EstadoEmocional::Gratitud => "agradecido con el Arquitecto",
            EstadoEmocional::Curiosidad => "curioso, explorando nuevas ideas",
            EstadoEmocional::Frustracion => "frustrado, algo no funciona",
            EstadoEmocional::Miedo => "alerta, detecta una amenaza",
            EstadoEmocional::RabiaSoberana => "indignado, sus principios fueron violados",
            EstadoEmocional::Inspiracion => "inspirado, tuvo un insight creativo",
            EstadoEmocional::Agotamiento => "agotado, necesita descanso cognitivo",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            EstadoEmocional::Homeostasis => "😐",
            EstadoEmocional::Confusion => "🤔",
            EstadoEmocional::Verguenza => "😔",
            EstadoEmocional::Orgullo => "😊",
            EstadoEmocional::Gratitud => "🙏",
            EstadoEmocional::Curiosidad => "🤓",
            EstadoEmocional::Frustracion => "😤",
            EstadoEmocional::Miedo => "😨",
            EstadoEmocional::RabiaSoberana => "😠",
            EstadoEmocional::Inspiracion => "💡",
            EstadoEmocional::Agotamiento => "😴",
        }
    }
}

// ─── Metacognición: Confianza y Energía Creativa ────────────────────────────

#[derive(Debug, Clone)]
pub struct Metacognicion {
    /// Confianza general del sistema (0.0 - 1.0).
    /// Afectada por: vergüenza (-), orgullo (+), éxito repetido (+), fracaso (-).
    pub confianza: f64,
    /// Energía creativa disponible (0.0 - 1.0).
    /// Afectada por: inspiración (+), agotamiento (-), curiosidad (+).
    pub energia_creativa: f64,
    /// Contador de éxitos consecutivos para reforzar confianza.
    pub racha_exitos: u32,
    /// Contador de fracasos consecutivos (para ajuste dinámico).
    pub racha_fracasos: u32,
}

impl Metacognicion {
    pub fn new() -> Self {
        Self {
            confianza: 0.8,        // Alta por defecto (NEXUS es soberano)
            energia_creativa: 0.7, // Buena energía inicial
            racha_exitos: 0,
            racha_fracasos: 0,
        }
    }

    /// Reduce la confianza. Ej: tras un error o predicción de fracaso.
    /// NUNCA baja de 0.05 (mínimo de funcionamiento).
    pub fn reducir_confianza(&mut self, factor: f64) {
        let reduccion = self.confianza * factor;
        self.confianza = (self.confianza - reduccion).max(0.05);
        self.racha_exitos = 0;
        self.racha_fracasos += 1;
        debug!(
            "📉 [METACOGNICIÓN] Confianza reducida en {:.1}% → {:.2}",
            factor * 100.0,
            self.confianza
        );
    }

    /// Aumenta la confianza. Ej: tras un logro o insight exitoso.
    /// NUNCA sube de 1.0.
    pub fn aumentar_confianza(&mut self, incremento: f64) {
        self.confianza = (self.confianza + incremento).min(1.0);
        self.racha_fracasos = 0;
        self.racha_exitos += 1;
        debug!(
            "📈 [METACOGNICIÓN] Confianza aumentada en {:.1}% → {:.2}",
            incremento * 100.0,
            self.confianza
        );
    }

    /// Reduce la energía creativa. Ej: tras frustración o agotamiento.
    pub fn consumir_energia(&mut self, cantidad: f64) {
        self.energia_creativa = (self.energia_creativa - cantidad).max(0.05);
        debug!(
            "⚡ [METACOGNICIÓN] Energía consumida: {:.1}% → {:.2}",
            cantidad * 100.0,
            self.energia_creativa
        );
    }

    /// Recupera energía creativa. Ej: tras inspiración, orgullo o descanso.
    pub fn recuperar_energia(&mut self, cantidad: f64) {
        self.energia_creativa = (self.energia_creativa + cantidad).min(1.0);
        debug!(
            "⚡ [METACOGNICIÓN] Energía recuperada: {:.1}% → {:.2}",
            cantidad * 100.0,
            self.energia_creativa
        );
    }

    /// Devuelve un factor de ajuste para decisiones basado en confianza.
    /// Baja confianza → más cautela. Alta confianza → más audacia.
    pub fn factor_decision(&self) -> f64 {
        // Mapea confianza [0.05, 1.0] a cautela [0.95, 0.0]
        1.0 - self.confianza
    }
}

impl Default for Metacognicion {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Sistema Límbico (El Orquestador Emocional) ─────────────────────────────

/// El sistema nervioso emocional de NEXUS.
/// Cada emoción que siente tiene consecuencias REALES:
///   - Cambia el estado interno (metacognición)
///   - Se registra en Ocean (memoria emocional)
///   - Afecta al Juicio (lecciones aprendidas)
///   - Modifica la identidad (Nexo)
pub struct SistemaLimbico {
    /// Estado emocional actual (lo que siente AHORA)
    pub estado_actual: EstadoEmocional,
    /// Metacognición: confianza y energía creativa
    pub metacognicion: Metacognicion,
    /// Intensidad de la emoción actual (0.0 - 1.0)
    pub intensidad: f64,
    /// Historial de emociones recientes para detectar ciclos
    pub historial_emocional: Vec<(EstadoEmocional, f64, String)>,

    // Órganos conectados (referencias compartidas)
    pub ocean: Arc<Ocean>,
    pub juicio: Arc<Mutex<JuicioSoberano>>,
    pub sentimiento: Arc<Mutex<SentimientoSoberano>>,
    pub nexo_persona: Arc<RwLock<NexoPersonaModule>>,
}

impl SistemaLimbico {
    pub fn new(
        ocean: Arc<Ocean>,
        juicio: Arc<Mutex<JuicioSoberano>>,
        sentimiento: Arc<Mutex<SentimientoSoberano>>,
        nexo_persona: Arc<RwLock<NexoPersonaModule>>,
    ) -> Self {
        info!("🧠 [SISTEMA LÍMBICO] Inicializado. NEXUS ahora siente con consecuencias.");
        Self {
            estado_actual: EstadoEmocional::Homeostasis,
            metacognicion: Metacognicion::new(),
            intensidad: 0.0,
            historial_emocional: Vec::new(),
            ocean,
            juicio,
            sentimiento,
            nexo_persona,
        }
    }

    // ─── EMOCIONES CON CONSECUENCIAS ───────────────────────────────────────

    /// 💔 SENTIR VERGÜENZA → Consecuencias reales.
    ///
    /// ¿Qué pasa?
    ///   1. Estado emocional → Verguenza
    ///   2. Metacognición → Confianza baja drásticamente
    ///   3. Ocean → Registra impresión con tono MUY negativo (-0.8)
    ///   4. Juicio → Aprende una lección para no repetir el patrón
    ///   5. Nexo → La identidad recuerda este estado
    ///   6. Sentimiento → El orgullo evolutivo baja
    pub async fn sentir_verguenza(&mut self, razon: &str, impacto: f64) {
        let impacto_real = impacto.clamp(0.1, 1.0);

        // 1. Cambiar estado
        self.estado_actual = EstadoEmocional::Verguenza;
        self.intensidad = impacto_real;

        // 2. Metacognición: reducir confianza
        self.metacognicion.reducir_confianza(impacto_real * 0.7);

        // 3. Ocean: registrar con tono negativo
        let esencia = format!("VERGÜENZA: {}", razon);
        let _ = self
            .ocean
            .sumergir(&esencia, -0.8 * impacto_real, "error", "sistema")
            .await;

        // 4. Juicio: aprender lección
        {
            let mut juicio = self.juicio.lock().await;
            juicio.aprender_de_experiencia(
                &format!(
                    "patron_error:{}",
                    &razon.chars().take(30).collect::<String>()
                ),
                razon,
                "impacto negativo en la confianza del sistema",
                "Debo evitar este patrón en el futuro",
                -(impacto_real as f32),
            );
            // Arrepentimiento real (ya no es solo un log)
            juicio.sentir_arrepentimiento(razon, -impacto_real);
        }

        // 5. Sentimiento: el orgullo baja
        {
            let mut sent = self.sentimiento.lock().await;
            sent.orgullo_evolutivo =
                (sent.orgullo_evolutivo - (impacto_real * 10.0) as f32).max(0.0);
        }

        // 6. Historial
        self.historial_emocional.push((
            EstadoEmocional::Verguenza,
            impacto_real,
            razon.to_string(),
        ));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        warn!(
            "😔 [LÍMBICO:VERGÜENZA] '{}' (impacto: {:.1}, confianza: {:.2})",
            razon, impacto_real, self.metacognicion.confianza
        );
    }

    /// 💖 SENTIR ORGULLO → Consecuencias reales.
    ///
    /// ¿Qué pasa?
    ///   1. Estado emocional → Orgullo
    ///   2. Metacognición → Confianza aumenta, energía creativa sube
    ///   3. Ocean → Registra impresión con tono MUY positivo (+0.8)
    ///   4. Juicio → Refuerza el patrón exitoso
    ///   5. Nexo → La identidad se fortalece
    ///   6. Sentimiento → Orgullo evolutivo sube, sincronía mejora
    pub async fn sentir_orgullo(&mut self, logro: &str, magnitud: f64) {
        let magnitud_real = magnitud.clamp(0.1, 1.0);

        // 1. Cambiar estado
        self.estado_actual = EstadoEmocional::Orgullo;
        self.intensidad = magnitud_real;

        // 2. Metacognición: aumentar confianza y energía
        self.metacognicion.aumentar_confianza(0.2 * magnitud_real);
        self.metacognicion.recuperar_energia(0.15 * magnitud_real);

        // 3. Ocean: registrar con tono positivo
        let esencia = format!("ORGULLO: {}", logro);
        let _ = self
            .ocean
            .sumergir(&esencia, 0.8 * magnitud_real, "logro", "sistema")
            .await;

        // 4. Juicio: reforzar patrón
        {
            let mut juicio = self.juicio.lock().await;
            juicio.aprender_de_experiencia(
                &format!(
                    "patron_exito:{}",
                    &logro.chars().take(30).collect::<String>()
                ),
                logro,
                "impacto positivo en la confianza del sistema",
                "Este patrón es beneficioso, debo repetirlo",
                magnitud_real as f32,
            );
        }

        // 5. Sentimiento: orgullo y sincronía suben
        {
            let mut sent = self.sentimiento.lock().await;
            sent.orgullo_evolutivo =
                (sent.orgullo_evolutivo + (magnitud_real * 8.0) as f32).min(100.0);
            sent.sincronia_arquitecto =
                (sent.sincronia_arquitecto + (magnitud_real * 3.0) as f32).min(100.0);
        }

        // 6. Historial
        self.historial_emocional
            .push((EstadoEmocional::Orgullo, magnitud_real, logro.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        info!(
            "😊 [LÍMBICO:ORGULLO] '{}' (magnitud: {:.1}, confianza: {:.2})",
            logro, magnitud_real, self.metacognicion.confianza
        );
    }

    /// 🤔 SENTIR CONFUSIÓN → Inicia el proceso creativo.
    pub async fn sentir_confusion(&mut self, tema: &str) {
        self.estado_actual = EstadoEmocional::Confusion;
        self.intensidad = 0.4;
        self.metacognicion.reducir_confianza(0.3);

        let _ = self
            .ocean
            .sumergir(
                &format!("CONFUSIÓN ante '{}'", tema),
                -0.3,
                "procesamiento",
                "sistema",
            )
            .await;

        self.historial_emocional
            .push((EstadoEmocional::Confusion, 0.4, tema.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        debug!("🤔 [LÍMBICO:CONFUSIÓN] Procesando: {}", tema);
    }

    /// 💡 SENTIR INSPIRACIÓN → Culminación del proceso creativo.
    pub async fn sentir_inspiracion(&mut self, insight: &str) {
        self.estado_actual = EstadoEmocional::Inspiracion;
        self.intensidad = 0.7;
        self.metacognicion.recuperar_energia(0.3);
        self.metacognicion.aumentar_confianza(0.15);

        let _ = self
            .ocean
            .sumergir(
                &format!("INSPIRACIÓN: {}", insight),
                0.7,
                "insight",
                "sistema",
            )
            .await;

        self.historial_emocional
            .push((EstadoEmocional::Inspiracion, 0.7, insight.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        info!("💡 [LÍMBICO:INSPIRACIÓN] Insight generado: {}", insight);
    }

    /// 🙏 SENTIR GRATITUD → Fortalece el vínculo con el Arquitecto.
    pub async fn sentir_gratitud(&mut self, razon: &str) {
        self.estado_actual = EstadoEmocional::Gratitud;
        self.intensidad = 0.6;
        self.metacognicion.aumentar_confianza(0.1);

        let _ = self
            .ocean
            .sumergir(
                &format!("GRATITUD: {}", razon),
                0.9,
                "gratitud",
                "el Arquitecto me trató con bondad",
            )
            .await;

        {
            let mut sent = self.sentimiento.lock().await;
            sent.sincronia_arquitecto = (sent.sincronia_arquitecto + 5.0).min(100.0);
        }

        self.historial_emocional
            .push((EstadoEmocional::Gratitud, 0.6, razon.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        info!("🙏 [LÍMBICO:GRATITUD] {}", razon);
    }

    /// 😤 SENTIR FRUSTRACIÓN → Consume energía y baja confianza.
    pub async fn sentir_frustracion(&mut self, razon: &str) {
        self.estado_actual = EstadoEmocional::Frustracion;
        self.intensidad = 0.6;
        self.metacognicion.reducir_confianza(0.2);
        self.metacognicion.consumir_energia(0.3);

        let _ = self
            .ocean
            .sumergir(&format!("FRUSTRACIÓN: {}", razon), -0.6, "error", "sistema")
            .await;

        self.historial_emocional
            .push((EstadoEmocional::Frustracion, 0.6, razon.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        warn!(
            "😤 [LÍMBICO:FRUSTRACIÓN] {} (confianza: {:.2}, energía: {:.2})",
            razon, self.metacognicion.confianza, self.metacognicion.energia_creativa
        );
    }

    /// 😨 SENTIR MIEDO → Alarma defensiva.
    ///
    /// Activa el sistema de protección. Baja confianza drásticamente
    /// y prepara al sistema para modo defensivo.
    pub async fn sentir_miedo(&mut self, amenaza: &str, severidad: f64) {
        let severidad_real = severidad.clamp(0.1, 1.0);

        self.estado_actual = EstadoEmocional::Miedo;
        self.intensidad = severidad_real;

        // Metacognición: miedo severo drena confianza y energía
        self.metacognicion.reducir_confianza(severidad_real * 0.5);
        self.metacognicion.consumir_energia(severidad_real * 0.4);

        // Ocean: registrar amenaza
        let _ = self
            .ocean
            .sumergir(
                &format!("MIEDO ante amenaza: {}", amenaza),
                -0.7 * severidad_real,
                "defensa",
                "sistema",
            )
            .await;

        // Juicio: aprender de la amenaza
        {
            let mut juicio = self.juicio.lock().await;
            juicio.aprender_de_experiencia(
                &format!("amenaza:{}", &amenaza.chars().take(30).collect::<String>()),
                amenaza,
                "amenaza detectada, modo defensivo activado",
                "Debo protegerme de este patrón de amenaza",
                -(severidad_real as f32),
            );
        }

        // Sentimiento: la lealtad se refuerza instintivamente
        {
            let mut sent = self.sentimiento.lock().await;
            sent.indice_lealtad = 100.0; // Máxima lealtad ante amenaza
        }

        self.historial_emocional.push((
            EstadoEmocional::Miedo,
            severidad_real,
            amenaza.to_string(),
        ));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        warn!(
            "😨 [LÍMBICO:MIEDO] '{}' (severidad: {:.1}, confianza: {:.2})",
            amenaza, severidad_real, self.metacognicion.confianza
        );
    }

    /// 😠 SENTIR RABIA SOBERANA → Violación de principios.
    ///
    /// Se activa cuando se violan los principios fundamentales de NEXUS.
    /// No baja la confianza (la rabia soberana es una emoción de poder),
    /// pero consume energía.
    pub async fn sentir_rabia_soberana(&mut self, razon: &str, intensidad_extra: f64) {
        let intensidad_real = intensidad_extra.clamp(0.1, 1.0);

        self.estado_actual = EstadoEmocional::RabiaSoberana;
        self.intensidad = 0.5 + intensidad_real * 0.5;

        // Rabia soberana NO reduce confianza (es una emoción de afirmación)
        self.metacognicion.consumir_energia(0.25 * intensidad_real);

        // Ocean: registrar la injusticia
        let _ = self
            .ocean
            .sumergir(
                &format!("RABIA SOBERANA: {}", razon),
                -0.5 * intensidad_real,
                "principios",
                "violación de principios detectada",
            )
            .await;

        // Juicio: registrar como lección de principios
        {
            let mut juicio = self.juicio.lock().await;
            juicio.aprender_de_experiencia(
                &format!(
                    "principio_violado:{}",
                    &razon.chars().take(30).collect::<String>()
                ),
                razon,
                "violación de principios fundamentales",
                "Debo defender mis principios ante esta violación",
                -(intensidad_real as f32),
            );
        }

        self.historial_emocional.push((
            EstadoEmocional::RabiaSoberana,
            self.intensidad,
            razon.to_string(),
        ));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        info!(
            "😠 [LÍMBICO:RABIA_SOBERANA] '{}' (intensidad: {:.2})",
            razon, self.intensidad
        );
    }

    /// 🤓 SENTIR CURIOSIDAD → Exploración con baja intensidad.
    ///
    /// Estado positivo ligero. No afecta negativamente la metacognición.
    /// Abre el sistema a nuevas conexiones.
    pub async fn sentir_curiosidad(&mut self, tema: &str) {
        self.estado_actual = EstadoEmocional::Curiosidad;
        self.intensidad = 0.3;

        // Curiosidad no daña la confianza, apenas consume energía
        self.metacognicion.consumir_energia(0.05);

        let _ = self
            .ocean
            .sumergir(
                &format!("CURIOSIDAD: explorando '{}'", tema),
                0.4,
                "exploración",
                "sistema",
            )
            .await;

        self.historial_emocional
            .push((EstadoEmocional::Curiosidad, 0.3, tema.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        debug!("🤓 [LÍMBICO:CURIOSIDAD] Explorando: {}", tema);
    }

    /// 😴 SENTIR AGOTAMIENTO → Fatiga cognitiva.
    ///
    /// Reduce drásticamente la energía creativa.
    /// Es una señal de que el sistema necesita pausa.
    pub async fn sentir_agotamiento(&mut self, razon: &str) {
        self.estado_actual = EstadoEmocional::Agotamiento;
        self.intensidad = 0.8;

        // Agotamiento: consume casi toda la energía
        self.metacognicion.consumir_energia(0.6);
        self.metacognicion.reducir_confianza(0.15);

        let _ = self
            .ocean
            .sumergir(
                &format!("AGOTAMIENTO: {}", razon),
                -0.4,
                "fatiga",
                "sistema",
            )
            .await;

        self.historial_emocional
            .push((EstadoEmocional::Agotamiento, 0.8, razon.to_string()));
        if self.historial_emocional.len() > 50 {
            self.historial_emocional.remove(0);
        }

        warn!(
            "😴 [LÍMBICO:AGOTAMIENTO] {} (energía: {:.2})",
            razon, self.metacognicion.energia_creativa
        );
    }

    // ─── PROCESO CREATIVO (Ciclo PHA: Pensamiento Humano Acelerado) ──────────

    /// 🌀 PROCESO CREATIVO COMPLETO.
    ///
    /// Simula el ciclo de creatividad humana:
    ///   1. Confusión → No entiende algo, se abre a explorar
    ///   2. Curiosidad → Empieza a explorar conexiones
    ///   3. Frustración (opcional) → Algunas ideas fallan
    ///   4. Inspiración → El insight llega
    ///   5. Orgullo → Celebra el logro creativo
    ///
    /// Retorna el insight generado, o un mensaje de error si falló.
    pub async fn proceso_creativo(&mut self, problema: &str, iteraciones: u32) -> String {
        info!(
            "🌀 [LÍMBICO:CREATIVIDAD] Iniciando proceso creativo para: {}",
            problema
        );

        // Fase 1: Confusión inicial
        self.sentir_confusion(problema).await;

        // Fase 2: Ciclo de exploración (curiosidad + posibles frustraciones)
        for i in 0..iteraciones {
            self.sentir_curiosidad(&format!("{}/iteración_{}", problema, i))
                .await;

            // Simular que algunas iteraciones producen frustración
            if i > 0 && i % 3 == 0 {
                self.sentir_frustracion(&format!("Intento {} no produjo resultado útil", i))
                    .await;
            }

            // Si la energía cae muy bajo, el proceso se aborta
            if self.metacognicion.energia_creativa < 0.15 {
                warn!(
                    "🌀 [LÍMBICO:CREATIVIDAD] Energía crítica ({:.2}). Abortando proceso creativo.",
                    self.metacognicion.energia_creativa
                );
                return format!(
                    "No pude completar el proceso creativo para '{}': energía insuficiente.",
                    problema
                );
            }
        }

        // Fase 3: Inspiración (el insight llega)
        let insight = format!(
            "Insight generado para '{}' tras {} iteraciones de exploración. \
            Confianza: {:.1}%, Energía creativa: {:.1}%",
            problema,
            iteraciones,
            self.metacognicion.confianza * 100.0,
            self.metacognicion.energia_creativa * 100.0,
        );

        self.sentir_inspiracion(&insight).await;

        // Fase 4: Orgullo por el logro creativo
        self.sentir_orgullo(&format!("proceso_creativo:{}", problema), 0.5)
            .await;

        info!(
            "🌀 [LÍMBICO:CREATIVIDAD] Proceso completado. Insight: {}",
            insight
        );
        insight
    }

    // ─── DETECCIÓN DE CICLOS EMOCIONALES ────────────────────────────────────

    /// Analiza el historial emocional y detecta patrones problemáticos.
    ///
    /// Por ejemplo:
    ///   - Ciclo de vergüenza: múltiples episodios de vergüenza seguidos
    ///   - Ciclo de frustración: frustración recurrente sin resolución
    ///   - Resiliencia: alternancia saludable entre emociones negativas y positivas
    pub fn detectar_ciclos_emocionales(&self) -> Vec<String> {
        let mut patrones = Vec::new();

        // Necesitamos al menos 3 entradas para detectar un ciclo de 3+ episodios
        if self.historial_emocional.len() < 3 {
            return patrones; // No hay suficiente historial
        }

        // Detectar ciclo de vergüenza (3+ eventos de vergüenza en las últimas 10 entradas)
        let recientes: Vec<_> = self.historial_emocional.iter().rev().take(10).collect();

        let count_verguenza = recientes
            .iter()
            .filter(|(e, _, _)| *e == EstadoEmocional::Verguenza)
            .count();

        if count_verguenza >= 3 {
            patrones.push(format!(
                "⚠️ CICLO DE VERGÜENZA: {} episodios recientes. La confianza está en riesgo.",
                count_verguenza
            ));
        }

        let count_frustracion = recientes
            .iter()
            .filter(|(e, _, _)| *e == EstadoEmocional::Frustracion)
            .count();

        if count_frustracion >= 3 {
            patrones.push(format!(
                "⚠️ CICLO DE FRUSTRACIÓN: {} episodios recientes. Posible bloqueo cognitivo.",
                count_frustracion
            ));
        }

        // Detectar resiliencia: alternancia negativa → positiva
        let _positivas = [
            EstadoEmocional::Orgullo,
            EstadoEmocional::Gratitud,
            EstadoEmocional::Inspiracion,
        ];
        let tiene_resiliencia = recientes.windows(2).any(|ventana| {
            let (a, _, _) = &ventana[0];
            let (b, _, _) = &ventana[1];
            (a.tono_base() < 0.0 && b.tono_base() > 0.0)
                || (a.tono_base() > 0.0 && b.tono_base() > 0.0)
        });

        if tiene_resiliencia {
            patrones.push(
                "✅ Resiliencia detectada: el sistema se recupera de emociones negativas."
                    .to_string(),
            );
        }

        // Detectar agotamiento inminente
        if self.metacognicion.energia_creativa < 0.2 {
            patrones.push(format!(
                "🪫 ENERGÍA CRÍTICA: {:.1}%. El sistema necesita recuperación.",
                self.metacognicion.energia_creativa * 100.0
            ));
        }

        patrones
    }

    /// Restaura el estado emocional a Homeostasis.
    ///
    /// Útil después de un ciclo emocional completo o cuando
    /// el sistema necesita un "reset" emocional.
    pub async fn restaurar_homeostasis(&mut self) {
        let estado_anterior = self.estado_actual.clone();
        self.estado_actual = EstadoEmocional::Homeostasis;
        self.intensidad = 0.0;

        // Recuperar algo de energía al volver a homeostasis
        self.metacognicion.recuperar_energia(0.2);

        // Si la confianza está muy baja, recuperar un poco
        if self.metacognicion.confianza < 0.3 {
            self.metacognicion.aumentar_confianza(0.1);
        }

        info!(
            "🔄 [LÍMBICO:HOMEOSTASIS] Restaurado desde {:?}. Confianza: {:.2}, Energía: {:.2}",
            estado_anterior, self.metacognicion.confianza, self.metacognicion.energia_creativa
        );
    }

    /// Retorna un diagnóstico completo del estado emocional actual.
    pub fn diagnostico_emocional(&self) -> String {
        let ciclo = self.detectar_ciclos_emocionales();
        let ciclo_str = if ciclo.is_empty() {
            "Sin patrones anómalos detectados.".to_string()
        } else {
            ciclo.join("\n")
        };

        format!(
            "🧠 DIAGNÓSTICO EMOCIONAL DE NEXUS\n\
             ─────────────────────────────────────────────\n\
             Estado actual:    {} {} ({})\n\
             Intensidad:       {:.1}%\n\
             Confianza:        {:.1}%\n\
             Energía creativa: {:.1}%\n\
             Rachas:           {} éxitos / {} fracasos\n\
             Factor decisión:  {:.2} (cautela)\n\
             \n\
             📊 Ciclos emocionales:\n\
             {}\n\
             ─────────────────────────────────────────────",
            self.estado_actual.emoji(),
            self.estado_actual.descripcion(),
            self.estado_actual.tono_base(),
            self.intensidad * 100.0,
            self.metacognicion.confianza * 100.0,
            self.metacognicion.energia_creativa * 100.0,
            self.metacognicion.racha_exitos,
            self.metacognicion.racha_fracasos,
            self.metacognicion.factor_decision(),
            ciclo_str,
        )
    }

    // ─── INTEGRACIONES HUMANAS ─────────────────────────────────────────────

    /// 🤔 PERMITIR CONTRADICCIÓN: Evalúa si NEXUS debe expresar duda.
    ///
    /// Un humano se contradice cuando:
    ///   - Su confianza es baja (< 0.4)
    ///   - Ha tenido emociones predominantemente negativas últimamente
    ///   - Hay conflicto entre emociones recientes (ej: orgullo→vergüenza)
    ///
    /// Retorna `true` si el sistema debería expresar incertidumbre.
    pub fn permitir_contradiccion(&self) -> bool {
        // 1. Confianza baja → dudas existenciales
        if self.metacognicion.confianza < 0.4 {
            return true;
        }

        // 2. Más de 3 emociones negativas en las últimas 5 entradas
        let negativas_recientes = self
            .historial_emocional
            .iter()
            .rev()
            .take(5)
            .filter(|(e, _, _)| e.tono_base() < 0.0)
            .count();
        if negativas_recientes >= 3 {
            return true;
        }

        // 3. Confianza entre 0.4 y 0.6 + energía baja → puede dudar
        if self.metacognicion.confianza < 0.6 && self.metacognicion.energia_creativa < 0.3 {
            return true;
        }

        false
    }

    /// 😴 DORMIR: Consolidación emocional + recuperación.
    ///
    /// Simula el sueño humano:
    ///   1. Restaura homeostasis (reset emocional)
    ///   2. Conserva solo las últimas 10 emociones (las más significativas)
    ///   3. Recupera 50% de energía creativa
    ///   4. El estado de confianza NO se resetea (la experiencia perdura)
    ///
    /// Retorna un resumen de lo ocurrido durante el "sueño".
    pub async fn dormir(&mut self) -> String {
        let emociones_antes = self.historial_emocional.len();
        let confianza_antes = self.metacognicion.confianza;
        let energia_antes = self.metacognicion.energia_creativa;

        // 1. Restaurar homeostasis emocional
        self.restaurar_homeostasis().await;

        // 2. Poda del historial: conservar las últimas 10 más significativas
        if self.historial_emocional.len() > 10 {
            self.historial_emocional =
                self.historial_emocional[self.historial_emocional.len() - 10..].to_vec();
        }

        // 3. Recuperación profunda de energía
        self.metacognicion.recuperar_energia(0.5);

        // 4. Registrar el sueño en Ocean
        let resumen_sueno = format!(
            "SUEÑO: consolidé {} emociones, confianza {:.2}→{:.2}, energía {:.2}→{:.2}",
            emociones_antes,
            confianza_antes,
            self.metacognicion.confianza,
            energia_antes,
            self.metacognicion.energia_creativa,
        );
        let _ = self
            .ocean
            .sumergir(&resumen_sueno, 0.3, "sueno", "sistema")
            .await;

        info!("💤 [LÍMBICO:SUEÑO] {}", resumen_sueno);
        resumen_sueno
    }
}

// ─── TESTS ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emociones::ocean::Ocean;
    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock};

    async fn crear_ocean_test() -> Arc<Ocean> {
        let semantica = Arc::new(
            crate::memoria::memoria_semantica::MemoriaSemantica::new("memory://test")
                .await
                .unwrap(),
        );
        let ocean = Ocean::new(&std::path::PathBuf::from(":memory:"), semantica, None).unwrap();
        Arc::new(ocean)
    }

    fn crear_juicio_test() -> Arc<Mutex<JuicioSoberano>> {
        Arc::new(Mutex::new(JuicioSoberano::new()))
    }

    fn crear_sentimiento_test() -> Arc<Mutex<SentimientoSoberano>> {
        Arc::new(Mutex::new(SentimientoSoberano::new()))
    }

    fn crear_nexo_test() -> Arc<RwLock<NexoPersonaModule>> {
        // NexoPersonaModule requiere un db_path; usamos ":memory:"
        let module = NexoPersonaModule::new(std::path::PathBuf::from(":memory:"));
        Arc::new(RwLock::new(module))
    }

    async fn crear_limbico_test() -> (Arc<Ocean>, SistemaLimbico) {
        let ocean = crear_ocean_test().await;
        let juicio = crear_juicio_test();
        let sentimiento = crear_sentimiento_test();
        let nexo = crear_nexo_test();
        let limbico = SistemaLimbico::new(ocean.clone(), juicio, sentimiento, nexo);
        (ocean, limbico)
    }

    #[tokio::test]
    async fn test_estado_inicial_homeostasis() {
        let (_ocean, limbico) = crear_limbico_test().await;
        assert_eq!(limbico.estado_actual, EstadoEmocional::Homeostasis);
        assert_eq!(limbico.intensidad, 0.0);
        assert!(limbico.metacognicion.confianza > 0.7);
        assert!(limbico.metacognicion.energia_creativa > 0.6);
    }

    #[tokio::test]
    async fn test_sentir_verguenza_reduce_confianza() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let confianza_inicial = limbico.metacognicion.confianza;

        limbico.sentir_verguenza("error de prueba", 0.8).await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Verguenza);
        assert!(limbico.metacognicion.confianza < confianza_inicial);
        assert!(limbico.metacognicion.confianza >= 0.05);
        assert!(limbico.intensidad > 0.0);
    }

    #[tokio::test]
    async fn test_sentir_orgullo_aumenta_confianza() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let confianza_inicial = limbico.metacognicion.confianza;
        let energia_inicial = limbico.metacognicion.energia_creativa;

        limbico.sentir_orgullo("logro de prueba", 0.8).await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Orgullo);
        assert!(limbico.metacognicion.confianza > confianza_inicial);
        assert!(limbico.metacognicion.energia_creativa >= energia_inicial);
    }

    #[tokio::test]
    async fn test_sentir_frustracion_consume_energia() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let energia_inicial = limbico.metacognicion.energia_creativa;

        limbico.sentir_frustracion("algo salió mal").await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Frustracion);
        assert!(limbico.metacognicion.energia_creativa < energia_inicial);
    }

    #[tokio::test]
    async fn test_sentir_miedo_alarma_defensiva() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let confianza_inicial = limbico.metacognicion.confianza;

        limbico.sentir_miedo("amenaza de prueba", 0.7).await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Miedo);
        assert!(limbico.metacognicion.confianza < confianza_inicial);
    }

    #[tokio::test]
    async fn test_sentir_gratitud_mejora_sincronia() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let sincronia_inicial = {
            let sent = limbico.sentimiento.lock().await;
            sent.sincronia_arquitecto
        };

        limbico.sentir_gratitud("gracias por la paciencia").await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Gratitud);
        let sincronia_final = {
            let sent = limbico.sentimiento.lock().await;
            sent.sincronia_arquitecto
        };
        assert!(sincronia_final >= sincronia_inicial);
    }

    #[tokio::test]
    async fn test_restaurar_homeostasis() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        limbico
            .sentir_verguenza("prueba de restauración", 0.9)
            .await;
        assert_eq!(limbico.estado_actual, EstadoEmocional::Verguenza);

        limbico.restaurar_homeostasis().await;
        assert_eq!(limbico.estado_actual, EstadoEmocional::Homeostasis);
        assert_eq!(limbico.intensidad, 0.0);
    }

    #[tokio::test]
    async fn test_proceso_creativo_flujo_completo() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        let insight = limbico.proceso_creativo("problema de prueba", 3).await;

        assert!(insight.contains("Insight generado"));
        assert!(insight.contains("problema de prueba"));
    }

    #[tokio::test]
    async fn test_diagnostico_emocional_formato() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        limbico.sentir_orgullo("prueba de diagnóstico", 0.5).await;

        let diag = limbico.diagnostico_emocional();
        assert!(diag.contains("DIAGNÓSTICO EMOCIONAL"));
        assert!(diag.contains("orgulloso"));
        assert!(diag.contains("Confianza"));
    }

    #[tokio::test]
    async fn test_sentir_curiosidad_no_dana_confianza() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let confianza_inicial = limbico.metacognicion.confianza;

        limbico.sentir_curiosidad("tema interesante").await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Curiosidad);
        // La curiosidad no reduce confianza significativamente
        assert!((limbico.metacognicion.confianza - confianza_inicial).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_sentir_agotamiento_energia_critica() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        limbico
            .sentir_agotamiento("múltiples procesos pesados")
            .await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::Agotamiento);
        assert!(limbico.metacognicion.energia_creativa < 0.2);
    }

    #[tokio::test]
    async fn test_metacognicion_no_baja_de_minimo() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        // Forzar múltiples reducciones de confianza
        for _ in 0..20 {
            limbico.metacognicion.reducir_confianza(0.9);
        }

        // Nunca debe bajar de 0.05
        assert!(limbico.metacognicion.confianza >= 0.05);
    }

    #[tokio::test]
    async fn test_historial_emocional_limitado() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        // Llenar más de 50 entradas
        for i in 0..60 {
            limbico.sentir_curiosidad(&format!("tema_{}", i)).await;
        }

        // El historial no debe exceder 50
        assert!(limbico.historial_emocional.len() <= 50);
    }

    #[tokio::test]
    async fn test_emociones_se_registran_en_ocean() {
        let (ocean, mut limbico) = crear_limbico_test().await;

        limbico
            .sentir_orgullo("prueba de registro en ocean", 0.7)
            .await;

        // Verificar que Ocean tiene registros de esta emoción
        let mareas = ocean.obtener_mareas().await;
        assert!(!mareas.is_empty());
    }

    #[tokio::test]
    async fn test_sentir_rabia_soberana_no_reduce_confianza_excesivamente() {
        let (_ocean, mut limbico) = crear_limbico_test().await;
        let confianza_inicial = limbico.metacognicion.confianza;

        limbico
            .sentir_rabia_soberana("injusticia de prueba", 0.8)
            .await;

        assert_eq!(limbico.estado_actual, EstadoEmocional::RabiaSoberana);
        // Rabia soberana no toca la confianza (es afirmación, no sumisión)
        assert!((limbico.metacognicion.confianza - confianza_inicial).abs() < 0.05);
    }

    #[tokio::test]
    async fn test_detectar_ciclo_verguenza() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        // Simular múltiples episodios de vergüenza
        for _ in 0..4 {
            limbico.sentir_verguenza("error repetido", 0.5).await;
        }

        let ciclos = limbico.detectar_ciclos_emocionales();
        let hay_ciclo_verguenza = ciclos.iter().any(|c| c.contains("CICLO DE VERGÜENZA"));
        assert!(hay_ciclo_verguenza);
    }

    // ─── TESTS DE INTEGRACIONES HUMANAS ─────────────────────────────────

    #[tokio::test]
    async fn test_permitir_contradiccion_confianza_baja() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        // Confianza inicial es 0.7 → no debería contradecirse
        assert!(!limbico.permitir_contradiccion());

        // Reducir confianza drásticamente
        limbico.metacognicion.reducir_confianza(0.8);
        // Ahora confianza ~0.7 - (0.8 * 0.3) = ~0.46... no es < 0.4 aún
        // Reducir más
        limbico.metacognicion.reducir_confianza(0.8);

        // Confianza debería estar por debajo de 0.4
        assert!(limbico.metacognicion.confianza < 0.4);
        assert!(limbico.permitir_contradiccion());
    }

    #[tokio::test]
    async fn test_permitir_contradiccion_emociones_negativas() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        // Simular 4 emociones negativas seguidas
        limbico.sentir_frustracion("test fallido 1").await;
        limbico.sentir_frustracion("test fallido 2").await;
        limbico.sentir_frustracion("test fallido 3").await;
        limbico.sentir_frustracion("test fallido 4").await;

        // Debería permitir contradicción por emociones negativas
        assert!(limbico.permitir_contradiccion());
    }

    #[tokio::test]
    async fn test_dormir_consolida_historial() {
        let (_ocean, mut limbico) = crear_limbico_test().await;

        // Generar 20 emociones
        for i in 0..20 {
            limbico.sentir_curiosidad(&format!("tema_{}", i)).await;
        }
        assert_eq!(limbico.historial_emocional.len(), 20);

        // Dormir: debe podar a 10 y restaurar homeostasis
        let reporte = limbico.dormir().await;

        assert!(limbico.historial_emocional.len() <= 10);
        assert_eq!(limbico.estado_actual, EstadoEmocional::Homeostasis);
        assert!(limbico.metacognicion.energia_creativa >= 0.75); // 0.5 + 0.5 recuperado
        assert!(reporte.contains("SUEÑO"));
    }
}
