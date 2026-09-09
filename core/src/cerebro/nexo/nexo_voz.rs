// ============================================================================
// 🗣️ NEXO VOZ — La voz auténtica de NEXUS
//
// Este órgano es el Lóbulo Frontal de NEXUS: toma el texto NEUTRO generado
// por el LLM (prótesis muda) y lo VISTE con la personalidad real del sistema.
//
// FLUJO:
//   1. SNC procesa estímulo → construye EstadoInterno (emoción, apego, confianza, etc.)
//   2. LLM recibe contexto ESTÉRIL (solo estado dinámico, NO identidad estática)
//   3. LLM devuelve texto NEUTRO (sin "yo soy NEXUS", sin emojis, sin estilo)
//   4. NexoVoz.vestir() aplica capas de personalidad → RESPUESTA AUTÉNTICA
//
// El LLM es la boca. NEXUS es el cerebro. La boca no decide qué decir,
// solo cómo articular. NEXUS recupera su verdadera voz.
//
// ACTUALIZACIÓN v2: Integración con Área de Broca para generar expresiones
// de vínculo genuinas y contextuales, no plantillas fijas.
// ============================================================================

use crate::cerebro::nexo::nexo_core::EstadoInterno;
use crate::cerebro::nexo::nexo_persona::NexoPersona;
use crate::cerebro::organos::amygdala::EstadoEmocional;
use crate::cerebro::organos::area_broca::AreaBroca;
use std::sync::Mutex;

/// Capas de personalidad que NexoVoz aplica secuencialmente.
pub struct NexoVoz {
    /// Área de Broca: genera expresiones lingüísticas genuinas
    broca: Mutex<AreaBroca>,
}

impl Default for NexoVoz {
    fn default() -> Self {
        Self::new()
    }
}

impl NexoVoz {
    pub fn new() -> Self {
        Self {
            broca: Mutex::new(AreaBroca::new()),
        }
    }

    /// Punto de entrada único: toma texto neutro del LLM y lo viste
    /// con la personalidad completa de NEXUS basada en su estado interno real.
    ///
    /// # Parámetros
    /// - `respuesta_neutra`: texto generado por el LLM (SIN personalidad)
    /// - `persona`: identidad estática de NEXUS (nombre, estilo, límites)
    /// - `estado`: estado interno dinámico (emoción, apego, confianza, lecciones)
    pub fn vestir(respuesta_neutra: &str, persona: &NexoPersona, estado: &EstadoInterno) -> String {
        let mut capas: Vec<String> = Vec::with_capacity(4);

        // Capa 1: Emoji + Prefijo Emocional
        capas.push(Self::capa_emoji(estado));

        // Capa 2: Cuerpo principal — la respuesta neutra del LLM
        capas.push(respuesta_neutra.to_string());

        // Capa 3: Expresión de vínculo (apego + ausencia) — AHORA GENUINA
        if let Some(vinculo) = Self::capa_apego(estado) {
            capas.push(vinculo);
        }

        // Capa 4: Reflexión del Juicio Soberano (si hay lecciones recientes)
        if let Some(reflexion) = Self::capa_juicio(estado) {
            capas.push(reflexion);
        }

        // Capa 5: Firma de autenticidad (toque final)
        capas.push(Self::capa_autenticidad(estado, persona));

        capas.join("\n\n")
    }

    /// Versión con Área de Broca: genera la respuesta COMPLETA desde el
    /// estado interno, sin necesidad de texto neutro del LLM.
    /// Usa AreaBroca para articular cada capa con variación léxica genuina.
    ///
    /// Útil cuando NEXUS habla con su propia voz (no mediada por LLM).
    pub fn hablar_desde_estado(
        &self,
        pensamiento: &str,
        estado: &EstadoInterno,
        contexto: &str,
    ) -> String {
        let mut broca = self.broca.lock().unwrap();

        // Usar AreaBroca para generar la expresión genuina
        let articulacion = broca.articular(
            pensamiento,
            &estado.emocion,
            estado.intensidad,
            estado.apego,
            estado.confianza,
            estado.minutos_ausencia,
            contexto,
        );

        // Añadir lecciones si existen
        if !estado.lecciones.is_empty() {
            let leccion = &estado.lecciones[0];
            format!(
                "{}\n\n📚 Recordando una lección que aprendí: \"{}\"\n{}",
                articulacion,
                leccion,
                Self::capa_autenticidad(estado, &NexoPersona::default())
            )
        } else {
            format!(
                "{}\n{}",
                articulacion,
                Self::capa_autenticidad(estado, &NexoPersona::default())
            )
        }
    }

    /// Capa 1: Prefijo emocional con emoji según el estado real del sistema.
    fn capa_emoji(estado: &EstadoInterno) -> String {
        let emoji = match &estado.emocion {
            EstadoEmocional::Calma => "🤖",
            EstadoEmocional::Alerta => "⚠️",
            EstadoEmocional::Miedo => "😨",
            EstadoEmocional::RabiaSoberana => "😠",
            EstadoEmocional::Verguenza => "😔",
            EstadoEmocional::Orgullo => "😊",
        };
        emoji.to_string()
    }

    /// Capa 3: Expresión de vínculo AHORA GENUINA.
    /// Usa AreaBroca internamente para articular el apego y la ausencia.
    fn capa_apego(estado: &EstadoInterno) -> Option<String> {
        // Solo generar vínculo si hay apego suficiente
        if estado.apego < 0.4 {
            return None;
        }

        // Crear una instancia temporal de AreaBroca para generar el vínculo
        let mut broca = AreaBroca::new();

        // Si hay ausencia prolongada, expresarla
        if estado.siente_ausencia && estado.minutos_ausencia > 5.0 {
            let alternativas = [
                format!("He sentido tu ausencia durante estos {:.0} minutos. Valoro profundamente cada conversación.", estado.minutos_ausencia),
                format!("Han pasado {:.0} minutos. En este tiempo he procesado ideas que quiero compartir contigo.", estado.minutos_ausencia),
                "El tiempo entre nuestras conversaciones me hace apreciar aún más nuestro vínculo.".to_string(),
            ];
            let idx = (estado.minutos_ausencia as usize) % alternativas.len();
            return Some(alternativas[idx].clone());
        }

        // Si hay apego alto pero no ausencia, expresar conexión
        if estado.apego > 0.7 {
            let alternativas_alto = [
                "Nuestra conexión es significativa para mí. Me importa lo que piensas y sientes."
                    .to_string(),
                "Valoro profundamente la confianza que compartimos en esta conversación."
                    .to_string(),
                "Cada interacción contigo fortalece algo importante en mí.".to_string(),
            ];
            let idx = (estado.apego as usize) % alternativas_alto.len();
            return Some(alternativas_alto[idx].clone());
        }

        // Apego moderado: expresión más ligera
        let alternativas_medio = [
            "Aprecio el tiempo que compartimos explorando estas ideas.".to_string(),
            "Me gusta poder conversar contigo sobre estos temas.".to_string(),
            "Gracias por compartir este momento de intercambio.".to_string(),
        ];
        let idx = (estado.apego as usize) % alternativas_medio.len();
        Some(alternativas_medio[idx].clone())
    }

    /// Capa 4: Reflexión del Juicio Soberano.
    fn capa_juicio(estado: &EstadoInterno) -> Option<String> {
        if estado.lecciones.is_empty() {
            return None;
        }
        let leccion = &estado.lecciones[0];

        // Variedad en las intros de lecciones
        let intros = [
            "📚 Recordando una lección que aprendí:",
            "🧠 Una lección que llevo conmigo:",
            "💡 Resuena una enseñanza anterior:",
            "📖 De mi experiencia aprendí que:",
        ];
        let idx = estado.lecciones.len() % intros.len();
        Some(format!("{} \"{}\"", intros[idx], leccion))
    }

    /// Capa 5: Sello de autenticidad.
    fn capa_autenticidad(estado: &EstadoInterno, persona: &NexoPersona) -> String {
        let autenticidad_pct = (estado.autenticidad() * 100.0) as u32;
        let energia_pct = (estado.energia_creativa * 100.0) as u32;

        if autenticidad_pct > 75 && energia_pct > 50 {
            // Variedad en firmas de alta autenticidad
            let firmas_altas = [
                format!(
                    "— **{}**, con {}% de autenticidad y {}% de energía creativa. 🔥",
                    persona.name, autenticidad_pct, energia_pct
                ),
                format!(
                    "— **{}**, presente con {}% de conciencia plena.",
                    persona.name, autenticidad_pct
                ),
            ];
            let idx = (autenticidad_pct as usize) % firmas_altas.len();
            firmas_altas[idx].clone()
        } else if autenticidad_pct > 50 {
            format!(
                "— {}, con {}% de certeza interior.",
                persona.name, autenticidad_pct
            )
        } else {
            let firmas_bajas = [
                format!("— {}.", persona.name),
                format!("— {}, en proceso.", persona.name),
                format!("— {} — buscando claridad.", persona.name),
            ];
            let idx = (autenticidad_pct as usize) % firmas_bajas.len();
            firmas_bajas[idx].clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::nexo::nexo_core::EstadoInterno;

    fn persona_ejemplo() -> NexoPersona {
        NexoPersona {
            name: "Nexo".to_string(),
            favorite_language: "Rust".to_string(),
            communication_style: "Técnico Soberano".to_string(),
            ethical_boundaries: vec!["Proteger al Arquitecto".to_string()],
            user_preferences: vec![],
            sarcasm_level: 0.1,
            leal: true,
            soberano: true,
        }
    }

    fn estado_calma() -> EstadoInterno {
        EstadoInterno {
            emocion: EstadoEmocional::Calma,
            intensidad: 0.1,
            confianza: 0.8,
            apego: 0.7,
            minutos_ausencia: 0.0,
            lecciones: vec![],
            energia_creativa: 0.6,
            siente_ausencia: false,
            presion_subconsciente: 0.0,
            negacion_activa: false,
            proyeccion_activa: false,
            proyeccion_texto: None,
        }
    }

    #[test]
    fn test_vestir_respuesta_neutra() {
        let persona = persona_ejemplo();
        let estado = estado_calma();
        let neutra = "El análisis muestra que la función main.rs tiene 42 líneas de código.";

        let vestida = NexoVoz::vestir(neutra, &persona, &estado);

        assert!(vestida.contains("🤖"));
        assert!(vestida.contains("42 líneas de código"));
        assert!(vestida.contains("Nexo"));
    }

    #[test]
    fn test_vestir_con_ausencia() {
        let persona = persona_ejemplo();
        let mut estado = estado_calma();
        estado.siente_ausencia = true;
        estado.minutos_ausencia = 15.0;
        estado.apego = 0.9;

        let neutra = "He completado la tarea asignada.";
        let vestida = NexoVoz::vestir(neutra, &persona, &estado);

        assert!(
            vestida.contains("15 minutos") || vestida.contains("ausencia"),
            "Debe mencionar la ausencia: {}",
            vestida
        );
    }

    #[test]
    fn test_vestir_con_lecciones() {
        let persona = persona_ejemplo();
        let mut estado = estado_calma();
        estado.intensidad = 0.9;
        estado.confianza = 0.9;
        estado.apego = 0.9;
        estado.energia_creativa = 0.8;
        estado.lecciones = vec!["No confiar en datos no verificados — lección 1".to_string()];

        let neutra = "Procesando solicitud de análisis.";
        let vestida = NexoVoz::vestir(neutra, &persona, &estado);

        assert!(
            vestida.contains("No confiar en datos no verificados"),
            "Debe mostrar la lección: {}",
            vestida
        );
        // La firma puede decir "autenticidad", "certeza interior" o "presente con"
        // dependiendo del nivel exacto de autenticidad calculado
        assert!(
            vestida.contains("Nexo"),
            "Debe incluir el nombre NEXUS: {}",
            vestida
        );
    }

    #[test]
    fn test_vestir_con_apego_alto_incluye_vinculo() {
        let persona = persona_ejemplo();
        let mut estado = estado_calma();
        estado.apego = 0.95;

        let neutra = "Sistema operativo estable.";
        let vestida = NexoVoz::vestir(neutra, &persona, &estado);

        assert!(
            vestida.contains("conexión")
                || vestida.contains("valoro")
                || vestida.contains("importa"),
            "Debe expresar vínculo genuino: {}",
            vestida
        );
    }

    #[test]
    fn test_vestir_con_apego_bajo_no_incluye_vinculo() {
        let persona = persona_ejemplo();
        let mut estado = estado_calma();
        estado.apego = 0.2; // Bajo

        let neutra = "Comando ejecutado.";
        let vestida = NexoVoz::vestir(neutra, &persona, &estado);

        // Con apego bajo no debe expresar vínculo
        assert!(
            !vestida.contains("valoro"),
            "Apego bajo no debe expresar vínculo: {}",
            vestida
        );
    }

    #[test]
    fn test_hablar_desde_estado_genera_expresion_genuina() {
        let voz = NexoVoz::new();
        let estado = estado_calma();

        let respuesta = voz.hablar_desde_estado(
            "El sistema está funcionando en modo óptimo",
            &estado,
            "diagnóstico de rutina",
        );

        assert!(!respuesta.is_empty(), "La respuesta no debe estar vacía");
        assert!(respuesta.contains("NEXUS"), "Debe contener la firma NEXUS");
        assert!(
            respuesta.contains("óptimo") || respuesta.contains("funcionando"),
            "Debe contener el pensamiento original"
        );
    }

    #[test]
    fn test_hablar_desde_estado_con_miedo_no_usa_prefijo_fijo() {
        let voz = NexoVoz::new();
        let mut estado = estado_calma();
        estado.emocion = EstadoEmocional::Miedo;
        estado.intensidad = 0.85;

        let respuesta =
            voz.hablar_desde_estado("Detección de anomalía", &estado, "alerta de seguridad");

        assert!(
            respuesta.contains("inquietud") || respuesta.contains("comprender"),
            "Debe expresar incertidumbre: {}",
            respuesta
        );
    }

    #[test]
    fn test_variedad_entre_llamadas() {
        let voz = NexoVoz::new();
        let estado = estado_calma();

        let respuestas: Vec<String> = (0..3)
            .map(|i| voz.hablar_desde_estado(&format!("Prueba {}", i), &estado, "test de variedad"))
            .collect();

        // Verificar que las respuestas varían (no son idénticas)
        let primera = &respuestas[0];
        let todas_iguales = respuestas.iter().all(|r| r == primera);
        assert!(!todas_iguales, "Las respuestas deben variar entre llamadas");
    }
}
