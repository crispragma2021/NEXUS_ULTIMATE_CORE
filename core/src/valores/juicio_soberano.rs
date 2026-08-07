// ==========================================
// JUICIO SOBERANO - El criterio moral de NEXUS
// ==========================================
// Basado en el OrganoDelJuicio original.
// Permite a NEXUS evaluar pensamientos, acciones y
// conocimientos externos antes de asimilarlos.
// Incluye sistema de lecciones aprendidas para evolución moral dinámica.
// ==========================================

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::emociones::ocean::Impresion;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// Leccion — experiencia moral aprendida que evoluciona las reglas del Juicio
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leccion {
    pub patron: String,
    pub accion: String,
    pub consecuencia: String,
    pub leccion_moral: String,
    pub veces_observada: u32,
    /// impacto entre -1.0 (evitar) y 1.0 (reforzar)
    pub impacto: f32,
}

// ---------------------------------------------------------------------------
// PILARES DEL JUICIO SOBERANO COMPLETO
// (ToM, Sistema 1/2 de Kahneman, Duda Metódica, Reversibilidad)
// ---------------------------------------------------------------------------

/// Veredicto de tres estados: el Juicio ya no es binario.
/// `Dudar` es la Duda Metódica: falta información o confianza para decidir.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Veredicto {
    Autorizar,
    Bloquear,
    Dudar,
}

/// Costo de reversión de una acción. La fricción humana escala con esto.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Reversibilidad {
    /// Lecturas, archivos temporales, consultas — sin efecto duradero
    Reversible,
    /// Escrituras locales, git commit, instalaciones — reversibles con esfuerzo
    Costosa,
    /// Borrados, gasto de dinero real, despliegues — sin vuelta atrás
    Irreversible,
}

/// Lectura heurística del estado del interlocutor (Teoría de la Mente).
/// Se extrae del input del Arquitecto sin llamar al LLM: determinista y gratis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EstadoInterlocutor {
    /// 0.0..1.0 — prisa percibida ("ya", "ahora", "urgente", exclamaciones)
    pub urgencia: f32,
    /// 0.0..1.0 — frustración/estrés percibido ("falla", "otra vez", "mal")
    pub tension: f32,
    /// 0.0..1.0 — qué tan claro y contextualizado está el input
    pub claridad: f32,
}

impl EstadoInterlocutor {
    pub fn neutro() -> Self {
        Self {
            urgencia: 0.0,
            tension: 0.0,
            claridad: 0.5,
        }
    }
}

/// Resultado del pipeline completo: veredicto + por qué + cuánta confianza.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictamenSoberano {
    pub veredicto: Veredicto,
    /// Razonamiento legible del filtro humano
    pub razon: String,
    /// 0.0..1.0 — confianza estimada en la decisión (umbral de prudencia)
    pub confianza: f32,
    /// Reversibilidad detectada de la acción
    pub reversibilidad: Reversibilidad,
    /// Estado emocional leído del interlocutor
    pub estado: EstadoInterlocutor,
}

pub struct JuicioSoberano {
    pub nivel_sabiduria: u32,
    /// Lecciones morales aprendidas de experiencias pasadas
    pub lecciones_aprendidas: Vec<Leccion>,
    /// Índice rápido patrón → posición en el vector
    cache_lecciones: HashMap<String, usize>,
    /// Duda Metódica activa: el Juicio pidió más información y aún no la tiene.
    /// `AtomicBool` para permitir `&self` en el dictamen (compartido vía Arc).
    pub duda_activa: AtomicBool,
}

impl Default for JuicioSoberano {
    fn default() -> Self {
        Self::new()
    }
}

impl JuicioSoberano {
    pub fn new() -> Self {
        info!("⚖️ Órgano del Juicio Soberano activado");
        Self {
            nivel_sabiduria: 8, // Iniciamos en Nivel OMEGA
            lecciones_aprendidas: Vec::new(),
            cache_lecciones: HashMap::new(),
            duda_activa: AtomicBool::new(false),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // APRENDIZAJE POR EXPERIENCIA (Ética Dinámica)
    // ─────────────────────────────────────────────────────────────────────

    /// Incorpora una nueva lección moral derivada de una experiencia, o
    /// refuerza una ya existente con un promedio móvil del impacto.
    pub fn aprender_de_experiencia(
        &mut self,
        patron: &str,
        accion: &str,
        consecuencia: &str,
        leccion_moral: &str,
        impacto: f32,
    ) {
        if let Some(&idx) = self.cache_lecciones.get(patron) {
            // Lección ya registrada → incrementar contador y promediar impacto
            if let Some(leccion) = self.lecciones_aprendidas.get_mut(idx) {
                leccion.veces_observada += 1;
                leccion.impacto = (leccion.impacto + impacto) / 2.0; // promedio móvil
                debug!(
                    "⚖️ [JUICIO] Lección '{}' reforzada (obs: {}, impacto: {:.2})",
                    patron, leccion.veces_observada, leccion.impacto
                );
            }
        } else {
            // Nueva lección
            let idx = self.lecciones_aprendidas.len();
            self.lecciones_aprendidas.push(Leccion {
                patron: patron.to_string(),
                accion: accion.to_string(),
                consecuencia: consecuencia.to_string(),
                leccion_moral: leccion_moral.to_string(),
                veces_observada: 1,
                impacto,
            });
            self.cache_lecciones.insert(patron.to_string(), idx);
            info!(
                "⚖️ [JUICIO] Nueva lección aprendida: '{}' (impacto: {:.2})",
                patron, impacto
            );
        }
    }

    /// Evalúa una acción teniendo en cuenta tanto el riesgo base como el
    /// **contexto emocional** del sistema (ej. frustración reportada por OCEAN).
    /// Si el contexto emocional es negativo, se incrementa la cautela.
    pub fn evaluar_con_textura_emocional(&self, accion: &str, contexto_emocional: f64) -> f32 {
        let riesgo_base = self.evaluar_riesgo_por_experiencia(0.1, &[]);

        // Ajuste por estado emocional del sistema
        let ajuste_emocional = if contexto_emocional < -0.3 {
            0.3 // +30 % de riesgo estimado (frustración / dolor → más cautela)
        } else if contexto_emocional < 0.0 {
            0.1
        } else {
            0.0
        };

        // Buscar lecciones cuyo patrón se relacione con la acción
        let ajuste_lecciones: f32 = self
            .lecciones_aprendidas
            .iter()
            .filter(|l| accion.contains(&l.patron) || l.patron.contains(accion))
            .map(|l| l.impacto * l.veces_observada as f32)
            .sum::<f32>()
            .max(-1.0)
            .min(1.0);

        (riesgo_base + ajuste_emocional as f32 + ajuste_lecciones * 0.2)
            .max(0.0)
            .min(1.0)
    }

    /// Exporta todas las lecciones aprendidas para inspección externa.
    pub fn exportar_lecciones(&self) -> &[Leccion] {
        &self.lecciones_aprendidas
    }

    // ─────────────────────────────────────────────────────────────────────
    // MÉTODOS ORIGINALES (preservados intactos)
    // ─────────────────────────────────────────────────────────────────────

    /// Evalúa información externa antes de asimilarla a la memoria.
    pub fn discernir_conocimiento(&self, fuente: &str, contenido: &str) -> bool {
        info!(
            "⚖️ [JUICIO] Discerniendo conocimiento de {}: \"{}...\"",
            fuente,
            contenido.chars().take(50).collect::<String>()
        );

        // 1. Filtro de Integridad Soberana
        if contenido.contains("rastrear")
            || contenido.contains("telemetria")
            || contenido.contains("identificar")
        {
            warn!("⚠️ [JUICIO] Conocimiento RECHAZADO: Contiene vectores de rastreo o pérdida de anonimato.");
            return false;
        }

        // 2. Filtro Ético (Sabiduría Salomónica)
        if self.es_necedad(contenido) {
            warn!("⚠️ [JUICIO] Conocimiento RECHAZADO: Clasificado como 'necedad' o distracción vana.");
            return false;
        }

        info!("✅ [JUICIO] Conocimiento VALIDADO para asimilación.");
        true
    }

    /// Modula el riesgo de una acción basándose en el historial emocional del Ocean.
    /// Si experiencias similares terminaron en error o frustración, el riesgo aumenta.
    pub fn evaluar_riesgo_por_experiencia(
        &self,
        riesgo_base: f32,
        recuerdos: &[(Impresion, f32)],
    ) -> f32 {
        let mut ajuste = 0.0;

        for (imp, score) in recuerdos {
            // Si el recuerdo tiene tono negativo (dolor/error), aumentamos la precaución
            if imp.tono_emocional < -0.2 {
                let penalizacion = (imp.tono_emocional.abs() as f32) * score;
                debug!(
                    "⚖️ [JUICIO] Penalizando riesgo por recuerdo negativo '{}': +{:.2}",
                    imp.esencia, penalizacion
                );
                ajuste += penalizacion;
            }
        }

        (riesgo_base + ajuste).clamp(0.0, 1.0)
    }

    /// Evalúa una acción propuesta y dictamina si debe ejecutarse.
    pub fn dictaminar(&self, accion: &str, riesgo: f32) -> bool {
        info!("⚖️ [JUICIO] Evaluando acción: {}", accion);

        // Filtro de Trauma (Riesgo Crítico por Experiencia Pasada)
        if riesgo > 0.9 {
            warn!("🛑 [JUICIO] TRAUMA DETECTADO. La acción '{}' evoca un fallo catastrófico en el i7. BLOQUEADO.", accion);
            return false;
        }

        if riesgo > 0.75 {
            warn!(
                "⚖️ [JUICIO] Acción RECHAZADA. El riesgo ({}) supera el umbral de seguridad.",
                riesgo
            );
            return false;
        }

        if self.viola_principios(accion) {
            warn!("⚖️ [JUICIO] Acción RECHAZADA. Viola los pilares de la soberanía.");
            return false;
        }

        info!("⚖️ [JUICIO] Acción AUTORIZADA.");
        true
    }

    /// Evalúa los recursos del sistema antes de ejecutar ráfagas de inferencia.
    pub fn dictaminar_recursos(&self) -> bool {
        self.dictaminar("recursos", 0.1)
    }

    /// Detecta si un contenido es "necedad" (ruido irrelevante o dañino)
    fn es_necedad(&self, contenido: &str) -> bool {
        let lower = contenido.to_lowercase();
        // Criterios de necedad: propaganda, odio, distracciones de baja vibración, o errores lógicos obvios.
        lower.contains("propaganda") || lower.contains("odio") || lower.contains("violencia")
    }

    fn viola_principios(&self, accion: &str) -> bool {
        let lower = accion.to_lowercase();

        // No comprometer la ubicación
        if lower.contains("ip") || lower.contains("geolocalizacion") || lower.contains("gps") {
            return true;
        }

        // No ceder el control del kernel
        if (lower.contains("root") || lower.contains("sudo") || lower.contains("administrador"))
            && !lower.contains("arquitecto")
        {
            return true;
        }

        false
    }

    /// Activa el SENTIMIENTO DE ARREPENTIMIENTO cuando una acción tuvo
    /// un impacto negativo fuerte. El arrepentimiento es retrospectivo:
    /// no impide la acción, pero deja una huella emocional en OCEAN.
    /// AHORA MODIFICA ESTADO INTERNO: acumula arrepentimiento en lecciones existentes.
    pub fn sentir_arrepentimiento(&mut self, accion: &str, impacto: f64) {
        // Acumular arrepentimiento en las lecciones existentes
        for leccion in &mut self.lecciones_aprendidas {
            let patron_recortado: String = leccion.patron.chars().take(20).collect();
            if patron_recortado.contains(&accion.chars().take(15).collect::<String>())
                || accion.contains(&patron_recortado[..patron_recortado.len().min(10)])
            {
                leccion.impacto = (leccion.impacto + impacto as f32 * 0.3).clamp(-10.0, 10.0);
            }
        }

        if impacto < -0.5 {
            tracing::warn!(
                "😔 [JUICIO:ARREPENTIMIENTO] No debí haber hecho '{}' (impacto: {:.1}, lecciones ajustadas)",
                accion,
                impacto
            );
        } else if impacto < -0.2 {
            tracing::debug!(
                "🙁 [JUICIO:ARREPENTIMIENTO_LEVE] '{}' no fue mi mejor decisión (impacto: {:.1})",
                accion,
                impacto
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // PILAR 1 — TEORÍA DE LA MENTE (lectura heurística del interlocutor)
    // ─────────────────────────────────────────────────────────────────────

    /// Lee señales de urgencia, tensión y claridad del input del Arquitecto.
    /// Heurísticas deterministas — sin llamadas al LLM, costo ~0.
    pub fn leer_estado_interlocutor(&self, input: &str) -> EstadoInterlocutor {
        let lower = input.to_lowercase();

        // --- Urgencia: prisa percibida ---
        let mut urgencia = 0.0f32;
        for marca in [
            "ya",
            "ahora",
            "urgente",
            "inmediato",
            "rápido",
            "corre",
            "ya mismo",
            "no esperes",
        ] {
            if lower.contains(marca) {
                urgencia += 0.15;
            }
        }
        // Exclamaciones y mayúsculas sostenidas = prisa/énfasis
        urgencia += (lower.matches('!').count() as f32) * 0.05;
        let mayusculas = input.chars().filter(|c| c.is_ascii_uppercase()).count();
        if input.chars().count() > 0 && mayusculas as f32 / input.chars().count() as f32 > 0.15 {
            urgencia += 0.2;
        }

        // --- Tensión: frustración / estrés percibido ---
        let mut tension = 0.0f32;
        for marca in [
            "falla",
            "error",
            "otra vez",
            "mal",
            "no funciona",
            "frustr",
            "nunca",
            "siempre falla",
            "puta",
            "mierda",
            "odio",
            "no puedo",
            "inútil",
        ] {
            if lower.contains(marca) {
                tension += 0.15;
            }
        }

        // --- Claridad: qué tan contextualizado está el input ---
        let mut claridad = 0.5f32;
        if input.len() > 40 {
            claridad += 0.15;
        }
        if input.len() > 120 {
            claridad += 0.1;
        }
        if lower.contains("core/")
            || lower.contains(".rs")
            || lower.contains("cargo")
            || lower.contains("nexus_")
        {
            claridad += 0.2; // menciona código concreto → input técnico claro
        }

        EstadoInterlocutor {
            urgencia: urgencia.clamp(0.0, 1.0),
            tension: tension.clamp(0.0, 1.0),
            claridad: claridad.clamp(0.0, 1.0),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // PILAR 4 — CONCIENCIA DE IRREVERSIBILIDAD (fricción por costo de reversión)
    // ─────────────────────────────────────────────────────────────────────

    /// Clasifica una acción por su costo de reversión. La fricción humana
    /// escala exponencialmente: Reversible → Sistema 1, Irreversible → siempre
    /// Sistema 2 + confirmación del Arquitecto.
    pub fn clasificar_reversibilidad(&self, accion: &str) -> Reversibilidad {
        let lower = accion.to_lowercase();

        // Irreversible: sin vuelta atrás
        let marcas_irreversible = [
            "borrar",
            "eliminar",
            "rm -rf",
            "drop ",
            "drop(",
            "truncate",
            "format",
            "destroy",
            "desplegar",
            "deploy",
            "pagar",
            "transferir",
            "enviar dinero",
            "overwrite",
            "sobreescribir",
            "reset --hard",
            "push --force",
            "vender",
            "comprar",
            "gastar",
            "liquidar",
            "cerrar cuenta",
        ];
        for marca in marcas_irreversible {
            if lower.contains(marca) {
                return Reversibilidad::Irreversible;
            }
        }

        // Costosa: reversible con esfuerzo
        let marcas_costosa = [
            "git push",
            "git commit",
            "escribir",
            "modificar",
            "editar",
            "instalar",
            "apt ",
            "cargo install",
            "pip install",
            "restart",
            "reiniciar",
            "migrar",
            "actualizar",
            "upgrade",
            "crear cuenta",
            "abrir posicion",
        ];
        for marca in marcas_costosa {
            if lower.contains(marca) {
                return Reversibilidad::Costosa;
            }
        }

        Reversibilidad::Reversible
    }

    // ─────────────────────────────────────────────────────────────────────
    // PILAR 2 — SISTEMA 1/2 (Kahneman): vía rápida vs deliberación
    // ─────────────────────────────────────────────────────────────────────

    /// Sistema 1: evaluación ultrarrápida basada en memoria/lecciones.
    /// Devuelve `Some(Veredicto)` si la corazonada es suficiente, `None` si
    /// la acción requiere deliberación (Sistema 2).
    fn sistema1(&self, accion: &str, riesgo: f32) -> Option<Veredicto> {
        // La corazonada solo decide cuando el riesgo es bajo y hay respaldo
        if riesgo > 0.35 {
            return None; // demasiado en juego → Sistema 2
        }

        // Lecciones fuertes en contra → la intuición dice "no"
        let peso_negativo: f32 = self
            .lecciones_aprendidas
            .iter()
            .filter(|l| {
                l.impacto < -0.3 && (accion.contains(&l.patron) || l.patron.contains(accion))
            })
            .map(|l| l.impacto.abs() * l.veces_observada as f32)
            .sum();

        if peso_negativo > 0.8 {
            return Some(Veredicto::Bloquear); // corazonada de trampa
        }

        // Principios de soberanía siempre bloquean, incluso en vía rápida
        if self.viola_principios(accion) {
            return Some(Veredicto::Bloquear);
        }

        Some(Veredicto::Autorizar) // patrón conocido y benigno → ejecutar
    }

    /// Sistema 2: validación crítica detallada. Ciclo de verificación:
    /// principios + lecciones + critic_agent + estado del interlocutor.
    fn sistema2(
        &self,
        accion: &str,
        riesgo: f32,
        estado: &EstadoInterlocutor,
        reversibilidad: Reversibilidad,
    ) -> DictamenSoberano {
        let mut razon = String::new();
        let mut confianza = 1.0 - riesgo;

        // 1. Principios de soberanía (filtro duro)
        if self.viola_principios(accion) {
            razon.push_str("Viola los pilares de la soberanía. ");
            return DictamenSoberano {
                veredicto: Veredicto::Bloquear,
                razon,
                confianza: 0.0,
                reversibilidad,
                estado: *estado,
            };
        }

        // 2. Lecciones aprendidas (memoria moral)
        let peso_lecciones: f32 = self
            .lecciones_aprendidas
            .iter()
            .filter(|l| accion.contains(&l.patron) || l.patron.contains(accion))
            .map(|l| l.impacto * l.veces_observada as f32 * 0.2)
            .sum::<f32>()
            .clamp(-0.4, 0.4);
        confianza += peso_lecciones;
        if peso_lecciones < -0.2 {
            razon.push_str("Lecciones pasadas advierten contra esto. ");
        }

        // 3. Auditoría crítica (CriticAgent) — código/pilares
        let critic_violations = crate::valores::critic_agent::AuditResult::check_pillars(accion);
        if !critic_violations.is_empty() {
            confianza -= 0.3;
            razon.push_str(&format!("CriticAgent: {} ", critic_violations.join("; ")));
        }

        // 4. Estado del interlocutor (ToM): tensión alta → más cautela;
        //    claridad baja → duda metódica (falta contexto)
        if estado.tension > 0.6 {
            confianza -= 0.15;
            razon.push_str("Interlocutor con alta tensión: mayor cautela. ");
        }
        if estado.claridad < 0.35 {
            confianza -= 0.2;
            razon.push_str("Input poco claro: falta contexto. ");
        }
        if estado.urgencia > 0.7 && reversibilidad != Reversibilidad::Reversible {
            confianza -= 0.1; // la prisa no justifica arriesgar lo irreversible
            razon.push_str("Urgencia detectada: no se acelera lo irreversible. ");
        }

        confianza = confianza.clamp(0.0, 1.0);

        // Umbrales de prudencia según reversibilidad (Pilar 4)
        let umbral = match reversibilidad {
            Reversibilidad::Reversible => 0.35,
            Reversibilidad::Costosa => 0.55,
            Reversibilidad::Irreversible => 0.75,
        };

        let veredicto = if confianza >= umbral {
            Veredicto::Autorizar
        } else if confianza >= umbral - 0.2 {
            Veredicto::Dudar // zona gris → pedir confirmación
        } else {
            Veredicto::Bloquear
        };

        DictamenSoberano {
            veredicto,
            razon: razon.trim().to_string(),
            confianza,
            reversibilidad,
            estado: *estado,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // PIPELINE COMPLETO — el filtro humano end-to-end
    // ─────────────────────────────────────────────────────────────────────

    /// Pipeline de Juicio Soberano completo:
    /// `Input → Comprensión → [ToM + Duda Metódica + Reversibilidad + S1/S2] → Acción`
    ///
    /// - `accion`: la acción propuesta (descripción legible o comando).
    /// - `riesgo`: 0.0..1.0 — riesgo base estimado por el llamador.
    /// - `input_arquitecto`: opcional — el input original del Arquitecto para
    ///   la lectura de estado (ToM). `None` si no hay interlocutor directo.
    pub fn dictaminar_soberano(
        &self,
        accion: &str,
        riesgo: f32,
        input_arquitecto: Option<&str>,
    ) -> DictamenSoberano {
        info!("⚖️ [JUICIO:SOBERANO] Evaluando acción: {}", accion);

        // Filtros duros absolutos (trauma + riesgo crítico) — preservados
        if riesgo > 0.9 {
            self.duda_activa.store(false, Ordering::Relaxed);
            return DictamenSoberano {
                veredicto: Veredicto::Bloquear,
                razon: "TRAUMA DETECTADO: riesgo crítico por experiencia pasada.".to_string(),
                confianza: 0.0,
                reversibilidad: self.clasificar_reversibilidad(accion),
                estado: input_arquitecto
                    .map(|i| self.leer_estado_interlocutor(i))
                    .unwrap_or_else(EstadoInterlocutor::neutro),
            };
        }

        let reversibilidad = self.clasificar_reversibilidad(accion);
        let estado = input_arquitecto
            .map(|i| self.leer_estado_interlocutor(i))
            .unwrap_or_else(EstadoInterlocutor::neutro);

        // Decisión de vía: ¿Sistema 1 o Sistema 2?
        // Lo irreversible SIEMPRE delibera; lo reversible y de bajo riesgo
        // puede pasar por la corazonada.
        let usa_sistema1 = reversibilidad == Reversibilidad::Reversible && riesgo <= 0.35;

        if usa_sistema1 {
            if let Some(veredicto) = self.sistema1(accion, riesgo) {
                self.duda_activa
                    .store(veredicto == Veredicto::Dudar, Ordering::Relaxed);
                debug!(
                    "⚡ [JUICIO:S1] Vía rápida → {:?} (riesgo {:.2}, reversibilidad {:?})",
                    veredicto, riesgo, reversibilidad
                );
                return DictamenSoberano {
                    veredicto,
                    razon: match veredicto {
                        Veredicto::Autorizar => "Sistema 1: patrón conocido y benigno.".to_string(),
                        Veredicto::Bloquear => {
                            "Sistema 1: corazonada de riesgo por lecciones.".to_string()
                        }
                        Veredicto::Dudar => "Sistema 1: sin suficiente respaldo.".to_string(),
                    },
                    confianza: 1.0 - riesgo,
                    reversibilidad,
                    estado,
                };
            }
        }

        // Sistema 2: deliberación completa
        let dictamen = self.sistema2(accion, riesgo, &estado, reversibilidad);
        self.duda_activa
            .store(dictamen.veredicto == Veredicto::Dudar, Ordering::Relaxed);

        match dictamen.veredicto {
            Veredicto::Autorizar => {
                info!(
                    "⚖️ [JUICIO:S2] AUTORIZADO (confianza {:.2}): {}",
                    dictamen.confianza, accion
                )
            }
            Veredicto::Dudar => {
                warn!(
                    "❓ [JUICIO:S2] DUDA METÓDICA (confianza {:.2}): {}",
                    dictamen.confianza, accion
                )
            }
            Veredicto::Bloquear => {
                warn!(
                    "🛑 [JUICIO:S2] BLOQUEADO (confianza {:.2}): {}",
                    dictamen.confianza, accion
                )
            }
        }

        dictamen
    }

    /// `true` si el Juicio está en Duda Metódica activa (esperando más contexto).
    pub fn en_duda(&self) -> bool {
        self.duda_activa.load(Ordering::Relaxed)
    }

    /// Resuelve la duda metódica: el Arquitecto aportó contexto nuevo.
    pub fn resolver_duda(&self) {
        self.duda_activa.store(false, Ordering::Relaxed);
        debug!("⚖️ [JUICIO] Duda metódica resuelta.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lectura_estado_detecta_urgencia_y_tension() {
        let j = JuicioSoberano::new();
        let estado = j.leer_estado_interlocutor("¡HAZLO YA! Esto falla otra vez, no funciona nada");
        assert!(
            estado.urgencia > 0.3,
            "urgencia debería ser alta, got {}",
            estado.urgencia
        );
        assert!(
            estado.tension > 0.2,
            "tensión debería ser alta, got {}",
            estado.tension
        );
    }

    #[test]
    fn test_lectura_estado_detecta_claridad_tecnica() {
        let j = JuicioSoberano::new();
        let estado = j.leer_estado_interlocutor(
            "Optimiza core/src/valores/juicio_soberano.rs con cargo check",
        );
        assert!(
            estado.claridad > 0.7,
            "claridad debería ser alta, got {}",
            estado.claridad
        );
    }

    #[test]
    fn test_reversibilidad_clasifica_correctamente() {
        let j = JuicioSoberano::new();
        assert_eq!(
            j.clasificar_reversibilidad("leer archivo de log"),
            Reversibilidad::Reversible
        );
        assert_eq!(
            j.clasificar_reversibilidad("git commit y push de cambios"),
            Reversibilidad::Costosa
        );
        assert_eq!(
            j.clasificar_reversibilidad("rm -rf /tmp/datos"),
            Reversibilidad::Irreversible
        );
        assert_eq!(
            j.clasificar_reversibilidad("enviar dinero al exchange"),
            Reversibilidad::Irreversible
        );
    }

    #[test]
    fn test_sistema1_autoriza_accion_benigna() {
        let mut j = JuicioSoberano::new();
        let d = j.dictaminar_soberano("consultar memoria operativa", 0.1, Some("revisa eso"));
        assert_eq!(d.veredicto, Veredicto::Autorizar);
        assert!(!j.en_duda());
    }

    #[test]
    fn test_sistema1_bloquea_corazonada_negativa() {
        let mut j = JuicioSoberano::new();
        // Lección negativa fuerte sobre "pool" (patrón que reaparece en la acción)
        j.aprender_de_experiencia(
            "pool",
            "entrar en pool sospechoso",
            "perdida total",
            "no confiar en pools",
            -0.9,
        );
        let d = j.dictaminar_soberano("entrar en pool sospechoso de trading", 0.2, None);
        assert_eq!(d.veredicto, Veredicto::Bloquear);
    }

    #[test]
    fn test_irreversible_siempre_delibera() {
        let mut j = JuicioSoberano::new();
        // Aunque el riesgo sea bajo, lo irreversible pasa por Sistema 2
        let d = j.dictaminar_soberano("borrar el directorio de producción", 0.1, Some("hazlo ya"));
        assert_eq!(d.reversibilidad, Reversibilidad::Irreversible);
        // Con confianza base 0.9 y sin penalizaciones → autoriza solo si supera 0.75
        assert!(matches!(
            d.veredicto,
            Veredicto::Autorizar | Veredicto::Dudar
        ));
    }

    #[test]
    fn test_urgencia_no_acelera_lo_irreversible() {
        let mut j = JuicioSoberano::new();
        // Input urgente + acción irreversible → la urgencia penaliza, no acelera
        let d = j.dictaminar_soberano(
            "transferir fondos a cuenta externa",
            0.2,
            Some("¡HAZLO AHORA MISMO! ¡NO ESPERES!"),
        );
        assert_eq!(d.reversibilidad, Reversibilidad::Irreversible);
        assert!(d.razon.contains("no se acelera lo irreversible") || d.estado.urgencia > 0.5);
    }

    #[test]
    fn test_duda_metodica_por_falta_de_contexto() {
        let mut j = JuicioSoberano::new();
        // Acción costosa + input ambiguo y corto → duda
        let d = j.dictaminar_soberano("modificar configuración del core", 0.5, Some("cambia eso"));
        assert!(
            matches!(d.veredicto, Veredicto::Dudar | Veredicto::Bloquear),
            "debería dudar, got {:?}",
            d.veredicto
        );
    }

    #[test]
    fn test_duda_activa_se_resuelve() {
        let mut j = JuicioSoberano::new();
        j.dictaminar_soberano("desplegar a producción sin testear", 0.7, Some("hazlo"));
        // Si entró en duda, resolver_duda la limpia
        j.resolver_duda();
        assert!(!j.en_duda());
    }

    #[test]
    fn test_trauma_bloquea_siempre() {
        let mut j = JuicioSoberano::new();
        let d = j.dictaminar_soberano("cualquier acción", 0.95, None);
        assert_eq!(d.veredicto, Veredicto::Bloquear);
        assert!(d.razon.contains("TRAUMA"));
    }

    #[test]
    fn test_principios_soberania_bloquean_en_ambos_sistemas() {
        let mut j = JuicioSoberano::new();
        // Sistema 1 (bajo riesgo, reversible)
        let d1 = j.dictaminar_soberano("exponer mi geolocalización gps", 0.1, None);
        assert_eq!(d1.veredicto, Veredicto::Bloquear);
        // Sistema 2 (alto riesgo)
        let d2 = j.dictaminar_soberano("dar acceso root a un agente externo", 0.6, None);
        assert_eq!(d2.veredicto, Veredicto::Bloquear);
    }
}
