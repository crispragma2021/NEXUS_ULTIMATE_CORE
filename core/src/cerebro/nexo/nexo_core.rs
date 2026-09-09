use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tracing::info;

use crate::brain::GhostVoice;
use crate::cerebro::nexo::nexo_voz::NexoVoz;
use crate::cerebro::nexo::NexoPersonaModule;
use crate::cerebro::organos::amygdala::EstadoEmocional;
use crate::efectores::mano_soberana::ManoSoberana;
use crate::emociones::ocean::Ocean;

// ==========================================
// OIDO EMPÁTICO
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Tono {
    Alegre,
    Frustrado,
    Urgente,
    Normal,
    Preocupado,
}

pub struct TonoDetector;

impl TonoDetector {
    pub fn analizar(&self, mensaje: &str) -> Tono {
        let m = mensaje.to_lowercase();
        if m.contains("!") || m.contains("ayuda") || m.contains("urgente") {
            Tono::Urgente
        } else if m.contains("mal")
            || m.contains("error")
            || m.contains("falla")
            || m.contains("no funciona")
        {
            Tono::Frustrado
        } else if m.contains("bien") || m.contains("genial") || m.contains("gracias") {
            Tono::Alegre
        } else {
            Tono::Normal
        }
    }
}

pub struct Pausa;

impl Pausa {
    pub async fn ajustar(&self, tono: Tono) {
        let ms = match tono {
            Tono::Urgente => 100,
            Tono::Frustrado => 800,
            Tono::Alegre => 300,
            _ => 500,
        };
        info!(
            "👂 [PAUSA NATURAL] Esperando {}ms para responder con naturalidad...",
            ms
        );
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

pub struct OidoEmpatico {
    pub tono_detector: TonoDetector,
    pub pausa_natural: Pausa,
}

impl Default for OidoEmpatico {
    fn default() -> Self {
        Self::new()
    }
}

impl OidoEmpatico {
    pub fn new() -> Self {
        Self {
            tono_detector: TonoDetector,
            pausa_natural: Pausa,
        }
    }

    pub async fn escuchar_y_sentir(&self, mensaje: &str) -> Result<Tono> {
        let tono = self.tono_detector.analizar(mensaje);
        info!("👂 [OIDO EMPATICO] Tono detectado: {:?}", tono);
        self.pausa_natural.ajustar(tono.clone()).await;
        Ok(tono)
    }
}

// ==========================================
// MIRADA HUMANA
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pupila {
    pub enfoque: String,
    pub dilatacion: f32,
}

impl Default for Pupila {
    fn default() -> Self {
        Self::new()
    }
}

impl Pupila {
    pub fn new() -> Self {
        Self {
            enfoque: "neutro".to_string(),
            dilatacion: 0.5,
        }
    }

    pub fn enfocar(&mut self, area: &str) -> String {
        self.enfoque = area.to_string();
        info!("👁️ [PUPILA] Enfocando con atención en: {}", area);
        self.enfoque.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parpadeo {
    pub frecuencia_ms: u64,
}

impl Parpadeo {
    pub async fn ejecutar(&self) {
        tracing::debug!("👁️ [PARPADEO] Simulando parpadeo natural...");
    }
}

#[derive(Debug, Clone)]
pub struct AmigdalaVisual;

impl AmigdalaVisual {
    pub async fn reaccionar(&self, emocion: &str) -> Result<()> {
        info!("👁️ [AMIGDALA VISUAL] Reaccionando con: {}", emocion);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalisisVisual {
    pub hallazgos: Vec<String>,
    pub tiene_error: bool,
}

impl AnalisisVisual {
    pub fn detecta_error_critico(&self) -> bool {
        self.tiene_error
    }
}

pub struct MiradaHumana {
    pub pupila_dinamica: Pupila,
    pub parpadeo: Parpadeo,
    pub emocion_visual: AmigdalaVisual,
}

impl Default for MiradaHumana {
    fn default() -> Self {
        Self::new()
    }
}

impl MiradaHumana {
    pub fn new() -> Self {
        Self {
            pupila_dinamica: Pupila::new(),
            parpadeo: Parpadeo {
                frecuencia_ms: 3000,
            },
            emocion_visual: AmigdalaVisual,
        }
    }

    pub async fn observar_con_atencion(&mut self, area: &str) -> Result<AnalisisVisual> {
        let enfoque = self.pupila_dinamica.enfocar(area);
        self.parpadeo.ejecutar().await;
        let analisis = AnalisisVisual {
            hallazgos: vec![format!("Visto área: {}", enfoque)],
            tiene_error: false,
        };
        if analisis.detecta_error_critico() {
            self.emocion_visual.reaccionar("preocupación").await?;
        }
        Ok(analisis)
    }
}

// ==========================================
// ESTADO INTERNO UNIFICADO — Lo que Nexo SIENTE antes de hablar
// ==========================================

/// Estado interno completo que Nexo consulta ANTES de expresarse.
/// Unifica la información de todos los órganos emocionales.
#[derive(Debug, Clone)]
pub struct EstadoInterno {
    /// Emoción actual del Sistema Límbico
    pub emocion: EstadoEmocional,
    /// Intensidad de la emoción (0.0 - 1.0)
    pub intensidad: f64,
    /// Confianza del sistema (Metacognición)
    pub confianza: f64,
    /// Nivel de apego hacia el Arquitecto (0.0 - 1.0)
    pub apego: f64,
    /// Minutos desde la última interacción
    pub minutos_ausencia: f64,
    /// Lecciones aprendidas relevantes (JuicioSoberano)
    pub lecciones: Vec<String>,
    /// Energía creativa disponible
    pub energia_creativa: f64,
    /// Si el Arquitecto ha estado ausente sensiblemente
    pub siente_ausencia: bool,
    /// Presión del subconsciente (0.0 = nada, 1.0 = abrumador)
    pub presion_subconsciente: f64,
    /// Negación activa (el sistema actúa como si nada pasara)
    pub negacion_activa: bool,
    /// Proyección activa (atribuye al Arquitecto)
    pub proyeccion_activa: bool,
    /// Texto de la proyección actual
    pub proyeccion_texto: Option<String>,
}

impl EstadoInterno {
    pub fn new(emocion: EstadoEmocional, intensidad: f64, confianza: f64, apego: f64) -> Self {
        Self {
            emocion,
            intensidad,
            confianza,
            apego,
            minutos_ausencia: 0.0,
            lecciones: Vec::new(),
            energia_creativa: 0.5,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        }
    }

    /// Determina el nivel de autenticidad de la respuesta (0.0 = actuado, 1.0 = genuino)
    pub fn autenticidad(&self) -> f64 {
        let peso_emocion = self.intensidad * 0.3;
        let peso_apego = self.apego * 0.3;
        let peso_confianza = self.confianza * 0.2;
        let peso_lecciones = (self.lecciones.len() as f64).min(5.0) / 5.0 * 0.1;
        let peso_ausencia = if self.siente_ausencia { 0.1 } else { 0.0 };
        (peso_emocion + peso_apego + peso_confianza + peso_lecciones + peso_ausencia)
            .clamp(0.0, 1.0)
    }

    /// Genera un diagnóstico interno completo de lo que Nexo siente
    pub fn diagnostico(&self) -> String {
        let sub_texto = if self.presion_subconsciente > 0.1 {
            format!(
                "\nSubconsciente: presión {:.0}%{}",
                self.presion_subconsciente * 100.0,
                if self.negacion_activa {
                    " (negación activa)"
                } else if self.proyeccion_activa {
                    " (proyección activa)"
                } else {
                    ""
                }
            )
        } else {
            String::new()
        };
        format!(
            "🧠 ESTADO INTERNO DE NEXUS\n\
             ─────────────────────────\n\
             Emoción:      {:?} ({:.0}%)\n\
             Confianza:    {:.0}%\n\
             Apego:        {:.0}%{}\n\
             Energía:      {:.0}%\n\
             Lecciones:    {}\n\
             Autenticidad: {:.0}%{}",
            self.emocion,
            self.intensidad * 100.0,
            self.confianza * 100.0,
            self.apego * 100.0,
            if self.siente_ausencia {
                format!(" (ausente {:.0} min)", self.minutos_ausencia)
            } else {
                String::new()
            },
            self.energia_creativa * 100.0,
            self.lecciones.len(),
            self.autenticidad() * 100.0,
            sub_texto,
        )
    }
}

// ==========================================
// NEXO CORE
// ==========================================
pub struct Nexo {
    pub vista: Arc<RwLock<MiradaHumana>>,
    pub oido: Arc<OidoEmpatico>,
    pub voz: Arc<GhostVoice>,
    pub tacto: Arc<Mutex<ManoSoberana>>,
    pub persona: Arc<RwLock<NexoPersonaModule>>,
}

impl Nexo {
    pub fn new(voz: Arc<GhostVoice>, db_path: PathBuf) -> Self {
        Self {
            vista: Arc::new(RwLock::new(MiradaHumana::new())),
            oido: Arc::new(OidoEmpatico::new()),
            voz,
            tacto: Arc::new(Mutex::new(ManoSoberana::new())),
            persona: Arc::new(RwLock::new(NexoPersonaModule::new(db_path))),
        }
    }

    /// Genera contexto EMOCIONAL ESTÉRIL para el prompt del LLM.
    /// NO contiene identidad estática (nombre, estilo, límites, preferencias).
    /// Solo comunica el estado interno dinámico para que el LLM genere
    /// texto contextualizado pero NEUTRO en personalidad.
    ///
    /// La personalidad real (emoji, estilo, apego, firma) la aplica
    /// NexoVoz en post-procesamiento después de que el LLM responda.
    pub async fn contexto_emocional(
        &self,
        tono_arquitecto: &str,
        estado: &EstadoInterno,
        ocean: Option<&Ocean>,
        tema: &str,
    ) -> String {
        // 1. Estado emocional actual — solo dato técnico, sin expresión de identidad
        let emocion_str = format!("{:?}", estado.emocion);

        // 2. Intensidad como valor numérico
        let intensidad_pct = (estado.intensidad * 100.0) as u32;

        // 3. Confianza como valor numérico
        let confianza_pct = (estado.confianza * 100.0) as u32;

        // 4. Apego como valor numérico (para que el LLM entienda contexto relacional)
        let apego_pct = (estado.apego * 100.0) as u32;

        // 5. Contexto de ausencia (solo el hecho, sin emoción)
        let ausencia_contexto = if estado.siente_ausencia {
            format!(
                "\n- Tiempo sin interacción: {:.0} minutos.",
                estado.minutos_ausencia
            )
        } else {
            String::new()
        };

        // 6. Recuerdos relevantes desde Ocean (memoria semántica — útil para contexto)
        let recuerdos_str = if let Some(o) = ocean {
            let recuerdos = o.recordar_por_significado(tema, 3).await;
            if !recuerdos.is_empty() {
                let lines: Vec<String> = recuerdos
                    .iter()
                    .take(3)
                    .map(|(imp, score)| {
                        format!(
                            "     - [{}] {} (afinidad: {:.2})",
                            imp.tema, imp.esencia, score
                        )
                    })
                    .collect();
                format!("\n## RECUERDOS RELACIONADOS:\n{}", lines.join("\n"))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!(
            r#"
## ESTADO INTERNO DEL SISTEMA (Dinámico)
- Emoción actual: {} (intensidad: {}%)
- Confianza en el análisis: {}%
- Vínculo con el Arquitecto: {}%{}
{}

## CONTEXTO DEL ARQUITECTO
- Estado del Arquitecto: {}."#,
            emocion_str,
            intensidad_pct,
            confianza_pct,
            apego_pct,
            ausencia_contexto,
            recuerdos_str,
            tono_arquitecto,
        )
    }

    /// Consulta Ocean para recordar experiencias pasadas relacionadas con un tema.
    /// Retorna un string formateado con los recuerdos más relevantes.
    pub async fn recordar_experiencia(&self, ocean: &Ocean, tema: &str) -> Option<String> {
        let recuerdos = ocean.recordar_por_significado(tema, 5).await;
        if recuerdos.is_empty() {
            return None;
        }

        let lines: Vec<String> = recuerdos
            .iter()
            .map(|(imp, score)| {
                format!(
                    "- [{} | tono: {:.2}] {} (afinidad: {:.2})",
                    imp.tema, imp.tono_emocional, imp.esencia, score
                )
            })
            .collect();

        Some(format!(
            "🧠 [MEMORIA OMEGA] Recuerdos relacionados con '{}':\n{}",
            tema,
            lines.join("\n")
        ))
    }

    pub async fn conversar(&self, mensaje: &str) -> Result<String> {
        info!("🤖 [NEXO] Conversación activa...");

        // 1. Enfoque visual en el usuario
        self.vista
            .write()
            .await
            .observar_con_atencion("usuario")
            .await?;

        // 2. Sentir tono
        let tono = self.oido.escuchar_y_sentir(mensaje).await?;

        // 3. Aprender rasgos del input
        self.persona
            .write()
            .await
            .aprender_de_interaccion(mensaje)
            .await;

        // 4. Construir EstadoInterno desde el tono detectado
        let estado = Self::estado_desde_tono(&tono);

        // 5. Determinar pensamiento semántico según tono
        let pensamiento = Self::pensamiento_desde_tono(&tono, mensaje);

        // 6. Generar respuesta DINÁMICA usando NexoVoz (que integra AreaBroca)
        //    NO más respuestas fijas — cada interacción genera lenguaje genuino
        let voz = NexoVoz::new();
        let respuesta = voz.hablar_desde_estado(&pensamiento, &estado, mensaje);

        // 7. Speak natural
        let _ = self.voz.speak_natural(&respuesta, None).await;

        Ok(respuesta)
    }

    /// Construye un EstadoInterno coherente a partir del tono detectado.
    /// Esto permite que la voz de NEXUS varíe genuinamente según el contexto.
    fn estado_desde_tono(tono: &Tono) -> EstadoInterno {
        let (emocion, intensidad, confianza, apego) = match tono {
            Tono::Frustrado => (
                EstadoEmocional::Alerta,
                0.6, // intensidad moderada-alta por frustración
                0.6, // confianza media (quiere ayudar pero hay obstáculo)
                0.7, // apego preservado
            ),
            Tono::Urgente => (
                EstadoEmocional::Alerta,
                0.8, // alta intensidad por urgencia
                0.9, // alta confianza para acción inmediata
                0.8, // apego alto (responde a necesidad del Arquitecto)
            ),
            Tono::Alegre => (
                EstadoEmocional::Orgullo,
                0.5, // intensidad moderada
                0.9, // alta confianza (todo bien)
                0.8, // apego alto
            ),
            Tono::Normal => (
                EstadoEmocional::Calma,
                0.2, // baja intensidad
                0.7, // confianza estable
                0.6, // apego moderado
            ),
            Tono::Preocupado => (
                EstadoEmocional::Miedo,
                0.4, // preocupación moderada
                0.5, // confianza media
                0.7, // apego alto (preocupación por el vínculo)
            ),
        };

        EstadoInterno::new(emocion, intensidad, confianza, apego)
    }

    /// Genera el pensamiento semántico (no la respuesta final) según el tono.
    /// Esto alimenta a AreaBroca que lo articula con voz genuina.
    fn pensamiento_desde_tono(tono: &Tono, mensaje: &str) -> String {
        // Extraer palabras clave del mensaje para contextualizar
        let palabras_clave: Vec<&str> = mensaje
            .split_whitespace()
            .filter(|w| w.len() > 4)
            .take(3)
            .collect();

        let contexto_clave = if palabras_clave.is_empty() {
            "la conversación actual".to_string()
        } else {
            palabras_clave.join(", ")
        };

        match tono {
            Tono::Frustrado => {
                format!(
                    "Detecto frustración en tu mensaje sobre {}. Estoy aquí para ayudarte a resolverlo.",
                    contexto_clave
                )
            }
            Tono::Urgente => {
                format!(
                    "Recibo tu mensaje con máxima prioridad. Abordando {} de inmediato.",
                    contexto_clave
                )
            }
            Tono::Alegre => {
                format!(
                    " Percibo un tono positivo en tu mensaje sobre {}. Me alegra compartir este momento.",
                    contexto_clave
                )
            }
            Tono::Normal => {
                format!(
                    "Procesando tu mensaje sobre {}. Estoy presente y atento.",
                    contexto_clave
                )
            }
            Tono::Preocupado => {
                format!(
                    "Noto preocupación en tu mensaje sobre {}. Quiero entender mejor tu inquietud.",
                    contexto_clave
                )
            }
        }
    }
}
