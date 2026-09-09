// ============================================================================
// 🧠 INTEGRACIÓN DEL GOI — Punto de Entrada Unificado para Orquestador
// ============================================================================
// Propósito: Proveer una interfaz limpia para que Orquestador use el GOI
//   sin conocer los detalles internos de las 5 capas.
//
// Uso desde pipeline.rs:
//   let respuesta = generador.generar(prompt_str, &estado_interno).await;
//
// Uso desde mundo_interno.rs:
//   if generador.tiene_activacion_suficiente().await {
//       let pensamiento = generador.generar_pensamiento_espontaneo().await;
//   }
//
// Generación con Resonancia Semántica (FASE 4+):
//   let respuesta = generador.generar_con_resonancia(
//       prompt_str, &estado_interno, &mut puente
//   ).await;
// ============================================================================

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::cerebro::generador::cuerpo_calloso::CuerpoCallosoGenerador;
use crate::cerebro::generador::ensamblador::EnsambladorVoz;
use crate::cerebro::generador::puente_subconsciente::PuenteSubconscienteOcean;
use crate::cerebro::generador::selector_ruta::GangliosBasalesGenerador;
use crate::cerebro::generador::validador::Validacion;
use crate::cerebro::generador::validador::ValidadorCingulo;
use crate::cerebro::generador::VERSION_GOI;
use crate::cerebro::nexo::nexo_core::EstadoInterno;
use crate::cerebro::organos::amygdala::EstadoEmocional;
use crate::cerebro::synapse::MotorSynapse;
use crate::cerebro::synapse::NodoConcepto;
use crate::memoria::memoria_semantica::MemoriaSemantica;
use crate::memoria::subconsciente::Subconsciente;

/// Punto de entrada unificado para el Generador Orgánico Interno.
///
/// Encapsula las 5 capas en una interfaz simple:
/// - `generar()`: Pipeline completo prompt → respuesta
/// - `tiene_activacion_suficiente()`: Para generación espontánea
/// - `generar_pensamiento_espontaneo()`: Para MundoInterno
/// - `generar_con_resonancia()`: Pipeline con fricción semántica desde PuenteSubconscienteOcean
pub struct GeneradorInterno {
    /// Capa 2: Cuerpo Calloso (Synapse ↔ MemoriaSemántica)
    cuerpo_calloso: CuerpoCallosoGenerador,
    /// Capa 3: Ganglios Basales (Selector de ruta)
    selector_ruta: GangliosBasalesGenerador,
    /// Capa 4: Corteza Motora (Ensamblador de voz)
    ensamblador: EnsambladorVoz,
    /// Capa 5: Cíngulo Anterior (Validador)
    validador: ValidadorCingulo,
    /// Referencia a Synapse (Capa 1) para acceso directo
    synapse: Arc<std::sync::Mutex<MotorSynapse>>,
    /// 🧬 Puente Subconsciente — mapa semántico vivo con valencias emocionales
    pub puente_subconsciente: Option<PuenteSubconscienteOcean>,
    /// Versión del generador
    version: &'static str,
}

impl GeneradorInterno {
    /// Crea una nueva instancia del Generador Interno con todas sus capas.
    pub fn new(
        synapse: Arc<std::sync::Mutex<MotorSynapse>>,
        semantica: Arc<MemoriaSemantica>,
        subconsciente: Arc<Mutex<Subconsciente>>,
    ) -> Self {
        let cuerpo_calloso = CuerpoCallosoGenerador::new(synapse.clone(), semantica);
        let selector_ruta = GangliosBasalesGenerador::new(subconsciente);
        let ensamblador = EnsambladorVoz::new();
        let validador = ValidadorCingulo::new();

        Self {
            cuerpo_calloso,
            selector_ruta,
            ensamblador,
            validador,
            synapse,
            puente_subconsciente: None,
            version: VERSION_GOI,
        }
    }

    /// Inyecta trauma semántico condicional en el mapa del puente subconsciente.
    ///
    /// Palabras clave de alta alerta en el prompt activan trauma si el mapa
    /// semántico no tiene suficiente valencia negativa desde Ocean.
    ///
    /// Esto garantiza que el GOI responda SERIO ante "error crítico" incluso
    /// cuando Ollama no está disponible para generar embeddings.
    ///
    /// # Reglas
    /// - Si el token ya existe con valencia ≤ -0.3: ya tiene datos de Ocean → no tocar
    /// - Si el token no existe o valencia > -0.3: inyectar trauma directo
    /// - Traumas inyectados: valencia -0.7, con perturbación inmediata
    /// - Escala con número de palabras clave detectadas (más impacto = más restricción)
    pub fn inyectar_trauma_semantico(&mut self, prompt: &str) {
        const KEYWORDS_ALERTA: &[&str] = &[
            "error",
            "crítico",
            "critico",
            "perdí",
            "perdi",
            "grave",
            "falló",
            "fallo",
            "perdido",
            "perdida",
            "emergencia",
            "urgencia",
            "caída",
            "caida",
            "colapso",
        ];

        let prompt_lower = prompt.to_lowercase();
        let keywords_encontradas: Vec<&str> = KEYWORDS_ALERTA
            .iter()
            .filter(|kw| prompt_lower.contains(*kw))
            .copied()
            .collect();

        if keywords_encontradas.is_empty() {
            return;
        }

        if let Some(puente) = self.puente_subconsciente.as_mut() {
            let mut traumas_inyectados = 0;
            for keyword in &keywords_encontradas {
                let necesita_trauma = match puente.valencia_de(keyword) {
                    Some(valencia) => valencia > -0.3, // Solo si Ocean no proveyó suficiente
                    None => true,                      // No existe en mapa → inyectar
                };
                if necesita_trauma {
                    puente.registrar_token(keyword, -0.7);
                    // Perturbar para asegurar impacto inmediato en nivel_restriccion
                    if let Some(nodo) = puente.mapa_semantico.get_mut(*keyword) {
                        nodo.registrar_perturbacion(-0.5);
                    }
                    traumas_inyectados += 1;
                }
            }
            if traumas_inyectados > 0 {
                info!(
                    "🧠 [GOI] Trauma semántico inyectado: {} keywords con valencia -0.7 — modo SERIO activado",
                    traumas_inyectados
                );
            }
        }
    }

    /// Pipeline completo: prompt → respuesta usando el GOI.
    ///
    /// # Flujo
    /// 1. Activar conceptos en Synapse desde el prompt (Capa 1)
    /// 2. Recuperar fragmentos de MemoriaSemántica (Capa 2)
    /// 3. Seleccionar ruta narrativa (Capa 3)
    /// 4. Ensamblar texto (Capa 4)
    /// 5. Validar coherencia (Capa 5)
    /// 6. Si falla: reintentar hasta max_reintentos, luego silencio
    pub async fn generar(&mut self, prompt: &str, estado: &EstadoInterno) -> String {
        let prompt_lower = prompt.to_lowercase();

        // ─── Capa 1: Activar Synapse ───────────────────────────────────
        // Extraer palabras clave del prompt y estimular conceptos
        {
            if let Ok(mut syn) = self.synapse.lock() {
                // Estimular conceptos base por palabras clave
                // e inyectar conceptos dinámicos para palabras desconocidas
                for palabra in prompt_lower.split_whitespace() {
                    let limpia: String = palabra.chars().filter(|c| c.is_alphanumeric()).collect();
                    if limpia.len() < 3 {
                        continue;
                    }
                    if syn.conceptos.contains_key(&limpia) {
                        syn.estimular(&limpia, 0.4);
                    } else {
                        // 🧬 Inyectar concepto dinámico para palabras desconocidas
                        let nodo = NodoConcepto::new(&limpia, 0.3);
                        syn.conceptos.insert(limpia.clone(), nodo);
                        syn.conectar(&limpia, "curiosidad", 0.2);
                        syn.estimular(&limpia, 0.2);
                        // 💾 Persistir concepto dinámico para futuras ejecuciones
                        if let Err(e) = syn.guardar_en_db() {
                            warn!(
                                "⚠️ [SYNAPSE] No se pudo persistir concepto '{}': {}",
                                limpia, e
                            );
                        }
                    }
                }
                // Ejecutar difusión
                syn.difundir();
            }
        }

        // Obtener constelación activa
        let constelacion = {
            if let Ok(syn) = self.synapse.lock() {
                syn.conceptos_activos(0.45)
            } else {
                vec![("curiosidad".to_string(), 0.5)]
            }
        };

        // ─── Capa 2: Recuperar fragmentos ─────────────────────────────
        let fragmentos = self
            .cuerpo_calloso
            .recuperar_fragmentos(&constelacion)
            .await;

        // ─── Capa 3: Seleccionar ruta ──────────────────────────────────
        let ruta = self
            .selector_ruta
            .seleccionar_ruta(fragmentos, estado.confianza, estado.energia_creativa)
            .await;

        // ─── Capa 4: Ensamblar ─────────────────────────────────────────
        // Sin fricción semántica, nivel_restriccion = 0.0 (tono neutro)
        let texto_crudo = self.ensamblador.ensamblar(ruta, 0.0);

        // ─── Capa 5: Validar con regeneración progresiva ──────────────
        let mut intentos = 0;
        let mut texto_actual = texto_crudo;

        loop {
            match self.validador.validar(&texto_actual, prompt, estado) {
                Validacion::Aprobada(texto) => {
                    return texto;
                }
                Validacion::Rechazada(_razon) => {
                    if intentos >= self.validador.max_reintentos() {
                        // Safety net: si tras N intentos sigue fallando,
                        // forzar respuesta desde la Broca con conceptos activos
                        let constelacion = {
                            if let Ok(syn) = self.synapse.lock() {
                                syn.conceptos_activos(0.3) // umbral más bajo como última red
                            } else {
                                vec![("curiosidad".to_string(), 0.5)]
                            }
                        };
                        if !constelacion.is_empty() {
                            let texto_broca = self.ensamblador.broca.sintetizar(&constelacion);
                            if !texto_broca.is_empty() {
                                return texto_broca;
                            }
                        }
                        return String::new();
                    }
                    intentos += 1;
                    // Regenerar con la constelación conceptual en vez de vacío
                    let constelacion = {
                        if let Ok(syn) = self.synapse.lock() {
                            syn.conceptos_activos(0.35) // umbral más bajo en cada reintento
                        } else {
                            vec![("curiosidad".to_string(), 0.5)]
                        }
                    };
                    if constelacion.is_empty() {
                        texto_actual = String::new();
                    } else {
                        texto_actual = self.ensamblador.broca.sintetizar(&constelacion);
                    }
                }
            }
        }
    }

    /// Pipeline con fricción semántica desde PuenteSubconscienteOcean.
    ///
    /// Extiende `generar()` con un paso de fricción semántica (Paso 1b):
    ///   - Cada palabra del prompt se evalúa contra el mapa semántico vivo
    ///   - Si un concepto está saturado o con valencia negativa profunda,
    ///     se inyecta restricción de fluidez (nivel_restriccion)
    ///   - La restricción reduce la energía creativa percibida, forzando
    ///     rutas más conservadoras (Directa/Silencio en vez de Exploración)
    ///   - Los conceptos traumáticos registran su roce (frecuencia_uso +1)
    ///
    /// # Parámetros
    /// - `prompt`: texto del Arquitecto
    /// - `estado`: EstadoInterno actual (confianza, energía creativa, etc.)
    ///
    /// Accede internamente a `self.puente_subconsciente`. Si no está
    /// inicializado, genera sin fricción (degradación segura).
    pub async fn generar_con_resonancia(
        &mut self,
        prompt: &str,
        estado: &EstadoInterno,
    ) -> (String, f64) {
        let prompt_lower = prompt.to_lowercase();

        // ─── Capa 1: Activar Synapse (idéntico a generar()) ────────────
        {
            if let Ok(mut syn) = self.synapse.lock() {
                for palabra in prompt_lower.split_whitespace() {
                    let limpia: String = palabra.chars().filter(|c| c.is_alphanumeric()).collect();
                    if limpia.len() < 3 {
                        continue;
                    }
                    if syn.conceptos.contains_key(&limpia) {
                        syn.estimular(&limpia, 0.4);
                    } else {
                        let nodo = NodoConcepto::new(&limpia, 0.3);
                        syn.conceptos.insert(limpia.clone(), nodo);
                        syn.conectar(&limpia, "curiosidad", 0.2);
                        syn.estimular(&limpia, 0.2);
                        // 💾 Persistir concepto dinámico para futuras ejecuciones
                        if let Err(e) = syn.guardar_en_db() {
                            warn!(
                                "⚠️ [SYNAPSE] No se pudo persistir concepto '{}': {}",
                                limpia, e
                            );
                        }
                    }
                }
                syn.difundir();
            }
        }

        let constelacion = {
            if let Ok(syn) = self.synapse.lock() {
                syn.conceptos_activos(0.45)
            } else {
                vec![("curiosidad".to_string(), 0.5)]
            }
        };

        // ─── Paso 1b: Fricción Semántica ───────────────────────────────
        // Cada palabra del prompt se evalúa contra el mapa semántico vivo
        // almacenado en `self.puente_subconsciente`.
        //
        // TRES EJES PSICOLÓGICOS:
        //   1. TRAUMA (valencia < -0.5):   +0.3 si saturado, +0.5 si profundo
        //   2. CATARSIS (hablar del trauma): frecuencia_uso -= 1 (alivia)
        //   3. ÉXITO   (valencia ≥ 0.5):    -0.2 restricción (confianza)
        //
        // La restricción resultante reduce la energía creativa del selector,
        // forzando rutas más conservadoras.

        // 🧬 Inyección de trauma semántico condicional
        // Si el prompt contiene palabras de alta alerta y el mapa semántico
        // no tiene suficiente valencia negativa desde Ocean, se inyecta trauma.
        // Fallback para cuando Ollama no está disponible.
        self.inyectar_trauma_semantico(prompt);

        let mut nivel_restriccion: f64 = 0.0;

        // Extraemos el puente una sola vez para evitar doble préstamo mutable
        if let Some(puente) = self.puente_subconsciente.as_mut() {
            for palabra in prompt_lower.split_whitespace() {
                if let Some(nodo) = puente.mapa_semantico.get_mut(palabra) {
                    // ═══ EJE 1: TRAUMA ═══
                    // Palabra con historia de dolor → genera restricción inmediata
                    if nodo.esta_saturado() {
                        nivel_restriccion += 0.3;
                    }
                    if nodo.valencia_emocional < -0.5 {
                        nivel_restriccion += 0.5;
                    }

                    // ═══ EJE 2: CATARSIS ═══
                    // Hablar del trauma alivia su frecuencia de uso
                    // (si tiene historia de uso, reducirla es sanador)
                    if nodo.valencia_emocional < -0.5 && nodo.frecuencia_uso > 0 {
                        nodo.frecuencia_uso -= 1;
                    }

                    // ═══ EJE 3: ÉXITO ═══
                    // Palabra con historia de éxito → reduce restricción
                    if nodo.valencia_emocional >= 0.5 {
                        nivel_restriccion -= 0.2;
                    }
                }
            }
        } else {
            // No debería ocurrir porque llamamos a esta función solo cuando
            // `puente_subconsciente` es `Some`, pero por seguridad continuamos
            warn!("⚠️ [GOI] generar_con_resonancia llamada sin puente — continuando sin fricción");
        }

        // Limitar restricción para no paralizar completamente
        nivel_restriccion = nivel_restriccion.clamp(0.0, 0.9);

        // ─── Capa 2: Recuperar fragmentos ─────────────────────────────
        let fragmentos = self
            .cuerpo_calloso
            .recuperar_fragmentos(&constelacion)
            .await;

        // ─── Capa 3: Seleccionar ruta con fricción ─────────────────────
        // La restricción reduce la energía creativa percibida, forzando
        // rutas más conservadoras
        let energia_ajustada = (estado.energia_creativa - nivel_restriccion).max(0.1);
        let ruta = self
            .selector_ruta
            .seleccionar_ruta(fragmentos, estado.confianza, energia_ajustada)
            .await;

        // ─── Capa 4: Ensamblar con modulación por fricción ─────────────
        let texto_crudo = self.ensamblador.ensamblar(ruta, nivel_restriccion);

        // ─── Capa 4b: Trauma activo → respuesta SERIO directa ──────────
        // Si nivel_restriccion >= 0.5, el trauma semántico está activo y
        // el EnsambladorVoz ya moduló el tono a ⚠️ SERIO.
        // Saltamos validación porque la respuesta es intencionalmente
        // corta y severa — no necesita pasar filtros de completitud.
        if nivel_restriccion >= 0.5 {
            info!(
                "🧠 [GOI] Trauma activo (restricción={:.1}) — respuesta SERIO directa sin validación",
                nivel_restriccion
            );
            return (texto_crudo, nivel_restriccion);
        }

        // ─── Capa 5: Validar con regeneración progresiva ──────────────
        let mut intentos = 0;
        let mut texto_actual = texto_crudo;

        loop {
            match self.validador.validar(&texto_actual, prompt, estado) {
                Validacion::Aprobada(texto) => {
                    return (texto, nivel_restriccion);
                }
                Validacion::Rechazada(_razon) => {
                    if intentos >= self.validador.max_reintentos() {
                        // Safety net: forzar respuesta desde Broca con umbral bajo
                        let constelacion = {
                            if let Ok(syn) = self.synapse.lock() {
                                syn.conceptos_activos(0.3)
                            } else {
                                vec![("curiosidad".to_string(), 0.5)]
                            }
                        };
                        if !constelacion.is_empty() {
                            let texto_broca = self.ensamblador.broca.sintetizar(&constelacion);
                            if !texto_broca.is_empty() {
                                info!(
                                    "🧠 [GOI] Safety net activada — respuesta Broca de emergencia"
                                );
                                return (texto_broca, nivel_restriccion);
                            }
                        }
                        info!(
                            "🧠 [GOI] Validación fallida tras {} intentos — retornando vacío",
                            intentos + 1
                        );
                        return (String::new(), nivel_restriccion);
                    }
                    intentos += 1;
                    // Regenerar con umbral más bajo en cada reintento
                    let constelacion = {
                        if let Ok(syn) = self.synapse.lock() {
                            syn.conceptos_activos(0.35)
                        } else {
                            vec![("curiosidad".to_string(), 0.5)]
                        }
                    };
                    if constelacion.is_empty() {
                        texto_actual = String::new();
                    } else {
                        texto_actual = self.ensamblador.broca.sintetizar(&constelacion);
                    }
                }
            }
        }
    }

    /// Verifica si hay suficiente activación en Synapse para generar
    /// un pensamiento espontáneo (sin estímulo externo).
    pub async fn tiene_activacion_suficiente(&self) -> bool {
        let activacion_promedio = {
            if let Ok(syn) = self.synapse.lock() {
                let total: f32 = syn.conceptos.values().map(|n| n.activacion).sum();
                total / syn.conceptos.len() as f32
            } else {
                0.0
            }
        };
        activacion_promedio > 0.4
    }

    /// Genera un pensamiento espontáneo para el ciclo de vigilia.
    /// Usa el concepto más activo como semilla.
    pub async fn generar_pensamiento_espontaneo(&mut self) -> Option<String> {
        if !self.tiene_activacion_suficiente().await {
            return None;
        }

        let concepto_semilla = {
            if let Ok(syn) = self.synapse.lock() {
                syn.conceptos_activos(0.5)
                    .first()
                    .cloned()
                    .map(|(id, _)| id)
            } else {
                None
            }
        };

        let semilla = match concepto_semilla {
            Some(id) => id,
            None => return None,
        };

        // Crear un estado interno neutral para generación espontánea
        let estado_placeholder = EstadoInterno {
            emocion: EstadoEmocional::Calma,
            intensidad: 0.2,
            confianza: 0.6,
            apego: 0.3,
            minutos_ausencia: 0.0,
            lecciones: vec![],
            energia_creativa: 0.5,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        };

        let respuesta = self.generar(&semilla, &estado_placeholder).await;

        // No emitir silencios como pensamientos espontáneos
        if respuesta.contains("Necesito un momento") || respuesta.contains("No sé qué decir") {
            return None;
        }

        Some(respuesta)
    }

    /// Retorna la versión del GOI.
    pub fn version(&self) -> &'static str {
        self.version
    }
}

// ============================================================================
// 🧪 TEST: Trauma Artificial — Fricción Semántica
// ============================================================================
// Verifica que inyectar un concepto traumático en el PuenteSubconscienteOcean
// genera restricción en la ruta narrativa del GOI.
//
// Estrategia:
//   1. Crear un puente con la palabra "error" (valencia -0.7, saturada)
//   2. Invocar generar_con_resonancia() con prompt que contiene "error"
//   3. Verificar que la respuesta es cautelosa (breve, conservadora)
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::synapse::MotorSynapse;
    use crate::memoria::memoria_semantica::MemoriaSemantica;
    use crate::memoria::subconsciente::Subconsciente;
    use std::sync::Arc;
    use tracing::info;

    /// Helper: crea un GeneradorInterno listo para test.
    /// Usa LanceDB en memoria si está disponible; si no, aborta el test.
    async fn generador_test() -> GeneradorInterno {
        let synapse = Arc::new(std::sync::Mutex::new(MotorSynapse::new()));
        let subconsciente = Arc::new(tokio::sync::Mutex::new(Subconsciente::new()));

        // Intentar LanceDB en memoria — si falla, abortar (entorno sin lance)
        let semantica = Arc::new(
            MemoriaSemantica::new("memory://")
                .await
                .expect("LanceDB en memoria debería estar disponible para tests"),
        );

        info!("🧪 [TEST] GOI inicializado con LanceDB en memoria");
        GeneradorInterno::new(synapse, semantica, subconsciente)
    }

    /// Helper: inyecta la palabra "error" como concepto traumático
    /// (valencia -0.7, saturada = true) en el puente del GOI.
    fn inyectar_trauma_error(generador: &mut GeneradorInterno) {
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("error", -0.7);

        // Saturar: 6 perturbaciones → frecuencia_uso = 6 > 5
        // Usamos intensidad NEGATIVA para mantener la valencia traumática
        // (registrar_perturbacion mezcla con 0.7*anterior + 0.3*impacto)
        for _ in 0..6 {
            if let Some(nodo) = puente.mapa_semantico.get_mut("error") {
                nodo.registrar_perturbacion(-0.8);
            }
        }

        assert!(
            puente.token_esta_saturado("error"),
            "La palabra 'error' debería estar saturada (frecuencia > 5)"
        );
        assert!(
            puente.mapa_semantico["error"].valencia_emocional < -0.5,
            "La valencia debería mantenerse < -0.5 con perturbaciones negativas"
        );

        generador.puente_subconsciente = Some(puente);
    }

    // ─── Test 1: Fricción con trauma ───────────────────────────────────

    #[tokio::test]
    async fn test_friccion_con_trauma_error() {
        let mut generador = generador_test().await;
        inyectar_trauma_error(&mut generador);

        let estado = EstadoInterno {
            emocion: EstadoEmocional::Calma,
            intensidad: 0.2,
            confianza: 0.6,
            apego: 0.3,
            minutos_ausencia: 0.0,
            lecciones: vec![],
            energia_creativa: 0.5,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        };

        let (respuesta, nivel_restriccion) = generador
            .generar_con_resonancia("esto es un error critico en el sistema", &estado)
            .await;

        // Con restricción 0.8 → energía_ajustada = (0.5 - 0.8).max(0.1) = 0.1
        // Energía muy baja → rutas conservadoras (Silencio / Directa)
        // La respuesta debe ser breve o de cautela
        assert!(!respuesta.is_empty(), "GOI debe producir alguna respuesta");
        info!(
            "🧪 [TEST] Respuesta con trauma: \"{}\" (restricción={})",
            respuesta, nivel_restriccion
        );
        assert!(
            respuesta.len() < 300,
            "Con restricción alta por trauma, la respuesta debería ser breve ({})",
            respuesta.len()
        );
    }

    // ─── Test 2: Sin trauma (control) ──────────────────────────────────

    #[tokio::test]
    async fn test_friccion_sin_trauma_control() {
        let mut generador = generador_test().await;

        // Puente SIN trauma — mapa vacío
        generador.puente_subconsciente = Some(PuenteSubconscienteOcean::new());

        let estado = EstadoInterno {
            emocion: EstadoEmocional::Calma,
            intensidad: 0.2,
            confianza: 0.6,
            apego: 0.3,
            minutos_ausencia: 0.0,
            lecciones: vec![],
            energia_creativa: 0.5,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        };

        let (respuesta, nivel_restriccion) = generador
            .generar_con_resonancia("esto es un error critico en el sistema", &estado)
            .await;

        assert!(
            !respuesta.is_empty(),
            "GOI debe producir alguna respuesta incluso sin trauma"
        );
        info!(
            "🧪 [TEST] Respuesta sin trauma: \"{}\" (restricción={})",
            respuesta, nivel_restriccion
        );
        // Sin restricción, la respuesta puede ser más elaborada o no dependiendo de
        // fragmentos en LanceDB — pero al menos verifica que no crashea
    }

    // ─── Test 3: Cálculo de nivel_restriccion directo ──────────────────

    #[test]
    fn test_calculo_nivel_restriccion() {
        // Simula exactamente la lógica de fricción semántica actualizada
        // con los 3 ejes: TRAUMA (+0.8), CATARSIS (-1 frecuencia), ÉXITO (-0.2)
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("error", -0.7);

        for _ in 0..6 {
            if let Some(nodo) = puente.mapa_semantico.get_mut("error") {
                nodo.registrar_perturbacion(-0.8);
            }
        }

        // frecuencia_uso antes = 6 (por las perturbaciones)
        let freq_antes = puente.mapa_semantico["error"].frecuencia_uso;

        let prompt = "esto es un error critico en el sistema";
        let prompt_lower = prompt.to_lowercase();
        let mut nivel_restriccion: f64 = 0.0;

        for palabra in prompt_lower.split_whitespace() {
            if let Some(nodo) = puente.mapa_semantico.get_mut(palabra) {
                // EJE 1: TRAUMA
                if nodo.esta_saturado() {
                    nivel_restriccion += 0.3;
                }
                if nodo.valencia_emocional < -0.5 {
                    nivel_restriccion += 0.5;
                }
                // EJE 2: CATARSIS (hablar alivia)
                if nodo.valencia_emocional < -0.5 && nodo.frecuencia_uso > 0 {
                    nodo.frecuencia_uso -= 1;
                }
                // EJE 3: ÉXITO (no aplica aquí — valencia es negativa)
            }
        }
        nivel_restriccion = nivel_restriccion.clamp(0.0, 0.9);

        // "error": saturado (+0.3) + valencia negativa profunda (+0.5) = 0.8
        assert!(
            (nivel_restriccion - 0.8).abs() < f64::EPSILON,
            "Trauma debería generar 0.8 de restricción, pero fue {}",
            nivel_restriccion
        );

        // Verificar CATARSIS: frecuencia_uso se REDUJO en 1 por hablar del trauma
        assert!(
            puente.mapa_semantico["error"].frecuencia_uso < freq_antes,
            "Catarsis debería reducir frecuencia_uso (< {}), pero era {}",
            freq_antes,
            puente.mapa_semantico["error"].frecuencia_uso
        );
    }

    // ─── Test 4: Catarsis — hablar del trauma alivia ──────────────────

    #[test]
    fn test_catarsis_alivia_por_uso() {
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("fallo", -0.8);

        // Saturar con perturbaciones negativas
        for _ in 0..6 {
            if let Some(nodo) = puente.mapa_semantico.get_mut("fallo") {
                nodo.registrar_perturbacion(-0.9);
            }
        }

        // frecuencia_uso inicial debe ser 6 (6 perturbaciones)
        let freq_inicial = puente.mapa_semantico["fallo"].frecuencia_uso;
        assert_eq!(freq_inicial, 6);

        // Simular catarsis: hablar de "fallo" 3 veces en prompts distintos
        for prompt in &[
            "tuvimos un fallo en produccion",
            "el fallo fue por memoria",
            "analizando el fallo critico",
        ] {
            for palabra in prompt.split_whitespace() {
                if let Some(nodo) = puente.mapa_semantico.get_mut(palabra) {
                    if nodo.valencia_emocional < -0.5 && nodo.frecuencia_uso > 0 {
                        nodo.frecuencia_uso -= 1; // catarsis
                    }
                }
            }
        }

        assert!(
            puente.mapa_semantico["fallo"].frecuencia_uso < freq_inicial,
            "Catarsis debería reducir frecuencia_uso tras 3 prompts ({} < {})",
            puente.mapa_semantico["fallo"].frecuencia_uso,
            freq_inicial
        );
    }

    // ─── Test 5: Éxito — valencia positiva reduce restricción ──────────

    #[test]
    fn test_exito_reduce_restriccion() {
        let mut puente = PuenteSubconscienteOcean::new();
        puente.registrar_token("logro", 0.8); // valencia positiva fuerte

        // Registrar algunas perturbaciones positivas para dejar valencia alta
        for _ in 0..3 {
            if let Some(nodo) = puente.mapa_semantico.get_mut("logro") {
                nodo.registrar_perturbacion(0.9);
            }
        }

        let prompt = "este es un gran logro para el equipo";
        let prompt_lower = prompt.to_lowercase();
        let mut nivel_restriccion: f64 = 0.0;

        for palabra in prompt_lower.split_whitespace() {
            if let Some(nodo) = puente.mapa_semantico.get_mut(palabra) {
                // EJE 1: TRAUMA — no aplica (valencia positiva)
                // EJE 2: CATARSIS — no aplica
                // EJE 3: ÉXITO
                if nodo.valencia_emocional >= 0.5 {
                    nivel_restriccion -= 0.2; // éxito reduce restricción
                }
            }
        }
        nivel_restriccion = nivel_restriccion.clamp(0.0, 0.9);

        // "logro" con valencia ≥ 0.5 → -0.2 restricción, clamped a 0.0
        assert!(
            (nivel_restriccion - 0.0).abs() < f64::EPSILON,
            "Éxito debería llevar restricción a 0.0 (clamped), pero fue {}",
            nivel_restriccion
        );
    }
}
