// ============================================================================
// ⚖️ Mediador de Corriente de Consciencia — Traducción Biológico-Lingüística
// ============================================================================
// Evita el caos en la salida de la simulación neuronal Hodgkin-Huxley.
// Traduce el estado mental (neuroquímico, somático y resonancia) en lenguaje.
// Ofrece una estructura de 3 capas: subconsciente -> monólogo interno -> expresión.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::cerebro::sistema_limbico::{EstadoEmocional, Neuroquimica};
use crate::cerebro::estructuras::NeuronaCompacta;
use crate::cerebro::lexico::asambleas::AsambleaSemantica;

/// Las tres capas del pensamiento articulado biológicamente
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorrienteConsciencia {
    /// Capa 1: Asociaciones libres — conceptos y emociones crudas activadas
    pub subconsciente: Vec<String>,
    /// Capa 2: Monólogo interno — frase pre-verbal que representa la intención
    pub monologo_interno: String,
    /// Capa 3: Expresión externa — texto final articulado para el chat
    pub expresion_externa: String,
    /// Métricas del estado mental que generó esta corriente
    pub estado_mental: EstadoMentalActivo,
}

/// Instantánea del estado interno en el momento de articular el pensamiento
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstadoMentalActivo {
    /// Entropía de Shannon del campo neuronal (0.0 = silencio, 1.0 = caos)
    pub entropia: f32,
    /// Índice de la asamblea que resonó (None si no hay concepto claro)
    pub asamblea_resonante: Option<usize>,
    /// Cohesión de la asamblea ganadora (0.0 - 1.0)
    pub cohesion: f32,
    /// Vector de neurotransmisores en el momento del colapso
    pub neuroquimica: NeuroquimicaSnapshot,
    /// Activación somática del hardware (0.0 - 1.0)
    pub activacion_somatica: f32,
    /// Tasa de disparo media del cerebro (Hz)
    pub tasa_disparo: f32,
    /// Factor de aprendizaje del sistema límbico
    pub factor_aprendizaje: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct NeuroquimicaSnapshot {
    pub dopamina: f32,
    pub serotonina: f32,
    pub adrenalina: f32,
    pub cortisol: f32,
    pub oxitocina: f32,
}

impl From<&Neuroquimica> for NeuroquimicaSnapshot {
    fn from(nq: &Neuroquimica) -> Self {
        Self {
            dopamina: nq.dopamina,
            serotonina: nq.serotonina,
            adrenalina: nq.adrenalina,
            cortisol: nq.cortisol,
            oxitocina: nq.oxitocina,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediadorConsciencia {
    /// Límite máximo de entropía permitida antes de declarar estado caótico
    pub umbral_entropia_max: f32,
    /// Evita repeticiones del mismo token consecutivos
    pub prohibir_duplicados_consecutivos: bool,

    // --- Nuevos campos de modulación dinámica ---
    /// Umbral mínimo de cohesión para considerar un concepto como "maduro"
    pub umbral_cohesion: f32,
    /// Multiplicador de fluidez por dopamina (mayor dopamina = frases más largas/detalladas)
    pub factor_fluidez_dopamina: f32,
    /// Penalización por cortisol (mayor cortisol = frases más cortas/defensivas)
    pub factor_bloqueo_cortisol: f32,
    /// Longitud máxima de tokens modulada por energía somática
    pub longitud_max_base: usize,
    /// Vocabulario emocional mapeado a estados límbicos
    pub prefijos_emocionales: HashMap<String, Vec<String>>,
    /// Historial de corrientes de consciencia (últimas N)
    pub historial: VecDeque<CorrienteConsciencia>,
}

// Mantener alias por retrocompatibilidad del sistema
pub type MediadorInmutable = MediadorConsciencia;

impl Default for MediadorConsciencia {
    fn default() -> Self {
        let mut prefijos = HashMap::new();
        prefijos.insert("Alegre".to_string(), vec!["🌟 [ALEGRES POSIBILIDADES]".to_string(), "✨ [ENTUSIASMO]".to_string()]);
        prefijos.insert("Triste".to_string(), vec!["🍂 [MELANCOLÍA EN LAS SINOPSIS]".to_string(), "🌧️ [RITMO BAJO]".to_string()]);
        prefijos.insert("Inspirado".to_string(), vec!["🚀 [SÍNTESIS CREATIVA]".to_string(), "💥 [SINOPSIS VIBRANTE]".to_string()]);
        prefijos.insert("Asustado".to_string(), vec!["⚠️ [ALERTA DE SISTEMA]".to_string(), "🚨 [ESTADO DE SOBRECARGA]".to_string()]);
        prefijos.insert("Frustrado".to_string(), vec!["🔒 [OBSTÁCULO COGNITIVO]".to_string(), "⏳ [LÍMITE ALCANZADO]".to_string()]);
        prefijos.insert("EnPaz".to_string(), vec!["🧘 [SINTONÍA SERENA]".to_string(), "🟢 [HOMEOSTASIS COMPLETA]".to_string()]);

        Self {
            umbral_entropia_max: 1.20, // >1.0 = desactiva filtro de entropía
            prohibir_duplicados_consecutivos: true,
            umbral_cohesion: 0.30,
            factor_fluidez_dopamina: 1.5,
            factor_bloqueo_cortisol: 0.7,
            longitud_max_base: 15,
            prefijos_emocionales: prefijos,
            historial: VecDeque::with_capacity(20),
        }
    }
}

impl MediadorConsciencia {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Evalúa la entropía (ruido/caos) de los potenciales de acción.
    pub fn calcular_entropia(&self, actividad: &[f32]) -> f32 {
        if actividad.is_empty() {
            return 0.0;
        }

        let suma: f32 = actividad.iter().sum();
        if suma <= 0.0001 {
            return 0.0;
        }

        let mut h = 0.0;
        for &act in actividad {
            if act > 0.0001 {
                let p = act / suma;
                h -= p * p.ln();
            }
        }

        let log_n = (actividad.len() as f32).ln();
        if log_n > 0.0 {
            h / log_n
        } else {
            0.0
        }
    }

    /// Aplica el filtrado determinista sobre la secuencia tentativa de tokens
    pub fn validar_secuencia(&self, tokens: &[u32]) -> bool {
        if tokens.is_empty() {
            return false;
        }

        if tokens.len() < 1 || tokens.len() > 30 { // Ampliado límite para mayor expresividad
            return false;
        }

        if self.prohibir_duplicados_consecutivos {
            for i in 1..tokens.len() {
                if tokens[i] == tokens[i - 1] {
                    return false;
                }
            }
        }

        true
    }

    /// Resuelve si la actividad y la secuencia actual son estables
    pub fn resolver(&self, actividad: &[f32], secuencia_ids: &[u32]) -> bool {
        let entropia = self.calcular_entropia(actividad);
        if entropia > self.umbral_entropia_max && (entropia - 1.0).abs() > 0.0001 {
            return false;
        }

        self.validar_secuencia(secuencia_ids)
    }

    /// ============================================================================
    /// 🔬 ALGORITMOS DE CORRIENTE DE CONSCIENCIA BIOLÓGICA
    /// ============================================================================

    /// Toma la asamblea que resonó en el MAS y colapsa su estado
    /// difuso en un vector de activación estable.
    pub fn colapsar_atractor(
        &self,
        asamblea: &AsambleaSemantica,
        neuronas: &[NeuronaCompacta],
    ) -> Vec<f32> {
        let mut vector_estado = Vec::with_capacity(asamblea.neuronas.len());
        for &nid in &asamblea.neuronas {
            if let Some(n) = neuronas.iter().find(|n| n.id == nid) {
                // Normalizar voltaje de [-70, +40] a [0.0, 1.0]
                let voltaje_normalizado = ((n.voltaje + 70.0) / 110.0).clamp(0.0, 1.0);
                vector_estado.push(voltaje_normalizado * n.activacion);
            } else {
                vector_estado.push(0.0);
            }
        }
        vector_estado
    }

    /// Traduce el vector de estado neuronal en una frase pre-verbal
    pub fn generar_monologo_interno(
        &self,
        vector_estado: &[f32],
        estado: &EstadoMentalActivo,
        etiqueta_asamblea: Option<&str>,
    ) -> String {
        let intensidad = if vector_estado.is_empty() {
            0.0
        } else {
            vector_estado.iter().sum::<f32>() / vector_estado.len() as f32
        };

        let tono = self.tono_desde_neuroquimica(&estado.neuroquimica);

        if let Some(etiqueta) = etiqueta_asamblea {
            format!("[MONÓLOGO INTERNO] Tono: {} | Concepto clave '{}' consolidándose con intensidad {:.2}. Cohesión asamblea: {:.2}", tono, etiqueta, intensidad, estado.cohesion)
        } else {
            format!("[MONÓLOGO INTERNO] Tono: {} | Ideas dispersas fluyendo de forma pre-verbal (intensidad {:.2})", tono, intensidad)
        }
    }

    /// Canaliza el monólogo interno en texto final para el chat.
    /// Aplica modulación límbica e interoceptiva en tiempo real.
    pub fn expresar_externamente(
        &self,
        monologo: &str,
        estado: &EstadoMentalActivo,
        etiqueta_asamblea: Option<&str>,
        estado_emocional: &EstadoEmocional,
    ) -> String {
        let nq = &estado.neuroquimica;

        // Mapear EstadoEmocional enum a String para buscar prefijos
        let emocional_str = format!("{:?}", estado_emocional);
        let prefijos_disponibles = self.prefijos_emocionales.get(&emocional_str);
        
        let prefijo = if let Some(pfs) = prefijos_disponibles {
            if !pfs.is_empty() {
                // Seleccionar prefijo basado en la cohesión o simplemente el primero
                let idx = (estado.cohesion * (pfs.len() as f32)).floor() as usize;
                pfs[idx.min(pfs.len() - 1)].clone()
            } else {
                "🧠 [PROCESANDO]".to_string()
            }
        } else {
            "🧠 [PROCESANDO]".to_string()
        };

        // --- Modulación por cortisol (bloqueo defensivo o precaución) ---
        if nq.cortisol > 0.6 {
            let base_msg = if let Some(lbl) = etiqueta_asamblea {
                format!("Mi red experimenta tensión en torno al concepto de '{}'. El monólogo interno revela: {}", lbl, monologo)
            } else {
                format!("Hay caos de potencial en la corteza. El monólogo interno revela: {}", monologo)
            };
            return format!("🔒 [PRECAUCIÓN DE SISTEMA] {}", self.truncar_por_energia(&base_msg, estado.activacion_somatica));
        }

        // --- Modulación por dopamina (fluidez creativa / expansividad) ---
        if nq.dopamina > 0.7 {
            let base_msg = if let Some(lbl) = etiqueta_asamblea {
                format!("¡Resonancia cognitiva óptima detectada! Canalizo el concepto '{}' con alta fluidez de disparo. Monólogo: {}", lbl, monologo)
            } else {
                format!("Inspiración en el flujo de asambleas semánticas. Monólogo: {}", monologo)
            };
            return format!("{} {}", prefijo, self.expandir_por_fluidez(&base_msg, nq.dopamina, estado.factor_aprendizaje));
        }

        // --- Modulación por serotonina (estabilidad y paz) ---
        if nq.serotonina > 0.6 {
            let base_msg = if let Some(lbl) = etiqueta_asamblea {
                format!("Sintonizo en armonía con '{}'. Monólogo: {}", lbl, monologo)
            } else {
                format!("Estado mental equilibrado. Monólogo: {}", monologo)
            };
            return format!("🧘 [ESTABLE] {}", base_msg);
        }

        // --- Estado basal / neutral ---
        if let Some(lbl) = etiqueta_asamblea {
            format!("🧠 [CONSCIENTE] Articulando '{}'. Monólogo: {}", lbl, monologo)
        } else {
            format!("🧠 [CONSCIENTE] Monólogo: {}", monologo)
        }
    }

    /// Traduce el perfil neuroquímico a una etiqueta de tono
    pub fn tono_desde_neuroquimica(&self, nq: &NeuroquimicaSnapshot) -> String {
        if nq.cortisol > 0.6 { return "ALERTA".to_string(); }
        if nq.dopamina > 0.7 && nq.adrenalina > 0.3 { return "INSPIRADO".to_string(); }
        if nq.dopamina > 0.7 { return "ALEGRE".to_string(); }
        if nq.serotonina > 0.6 { return "SERENO".to_string(); }
        "NEUTRAL".to_string()
    }

    /// Trunca el mensaje proporcionalmente a la energía somática disponible (o carga del sistema)
    fn truncar_por_energia(&self, texto: &str, activacion_somatica: f32) -> String {
        // A mayor activación somática (carga/estrés de hardware), menor energía disponible para lenguaje extenso
        let energia_disponible = (1.0 - activacion_somatica).clamp(0.1, 1.0);
        let max_chars = (self.longitud_max_base * 10) as f32 * energia_disponible;
        let max_chars_usize = max_chars as usize;

        if texto.chars().count() > max_chars_usize {
            texto.chars().take(max_chars_usize).collect::<String>() + "..."
        } else {
            texto.to_string()
        }
    }

    /// Expande el mensaje con creatividad proporcional a la dopamina
    fn expandir_por_fluidez(&self, texto: &str, dopamina: f32, factor_aprendizaje: f32) -> String {
        if dopamina > 0.8 && factor_aprendizaje > 1.2 {
            format!("{} — Mi sistema límbico vibra en sinergia con este pensamiento, potenciando la plasticidad sináptica.", texto)
        } else {
            texto.to_string()
        }
    }

    /// Canaliza todo el pipeline de corriente de consciencia de forma integrada
    pub fn procesar_corriente(
        &mut self,
        asamblea_resonante: Option<usize>,
        asambleas: &[AsambleaSemantica],
        neuronas: &[NeuronaCompacta],
        entropia: f32,
        neuroquimica: &Neuroquimica,
        activacion_somatica: f32,
        tasa_disparo: f32,
        factor_aprendizaje: f32,
        estado_emocional: &EstadoEmocional,
        _texto_entrada: &str,
    ) -> CorrienteConsciencia {
        let nq_snap = NeuroquimicaSnapshot::from(neuroquimica);
        
        let estado_mental = EstadoMentalActivo {
            entropia,
            asamblea_resonante,
            cohesion: asamblea_resonante.and_then(|idx| asambleas.get(idx)).map(|a| a.cohesion).unwrap_or(0.0),
            neuroquimica: nq_snap,
            activacion_somatica,
            tasa_disparo,
            factor_aprendizaje,
        };

        // Capa 1: Subconsciente (Asociaciones libres - etiquetas de asambleas que tienen algún solapamiento o la ganadora)
        let mut subconsciente = Vec::new();
        if let Some(idx) = asamblea_resonante {
            if let Some(asamblea) = asambleas.get(idx) {
                if let Some(ref et) = asamblea.etiqueta {
                    subconsciente.push(et.clone());
                }
            }
        }
        // Buscar otras asambleas secundarias con cohesión para subconsciente
        for (i, asamblea) in asambleas.iter().enumerate() {
            if Some(i) != asamblea_resonante && asamblea.cohesion > 0.4 {
                if let Some(ref et) = asamblea.etiqueta {
                    subconsciente.push(format!("asociacion:{}", et));
                }
            }
        }
        if subconsciente.is_empty() {
            subconsciente.push("rumiacion:ruido_basal".to_string());
        }

        // Capa 2: Monólogo Interno
        let vector_estado = if let Some(idx) = asamblea_resonante {
            if let Some(asamblea) = asambleas.get(idx) {
                self.colapsar_atractor(asamblea, neuronas)
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let etiqueta_as_ref = asamblea_resonante
            .and_then(|idx| asambleas.get(idx))
            .and_then(|a| a.etiqueta.as_deref());

        let monologo_interno = self.generar_monologo_interno(&vector_estado, &estado_mental, etiqueta_as_ref);

        // Capa 3: Expresión Externa
        let expresion_externa = self.expresar_externamente(&monologo_interno, &estado_mental, etiqueta_as_ref, estado_emocional);

        let corriente = CorrienteConsciencia {
            subconsciente,
            monologo_interno,
            expresion_externa,
            estado_mental,
        };

        // Guardar en el historial
        if self.historial.len() >= 20 {
            self.historial.pop_front();
        }
        self.historial.push_back(corriente.clone());

        corriente
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::sistema_limbico::SistemaLimbico;

    #[test]
    fn test_validar_secuencia_consecutivos() {
        let mediador = MediadorConsciencia::nuevo();
        assert!(mediador.validar_secuencia(&[1, 2, 3]));
        assert!(!mediador.validar_secuencia(&[1, 1, 3])); // Fails consecutive rule
    }

    #[test]
    fn test_entropia_silencio() {
        let mediador = MediadorConsciencia::nuevo();
        let actividad = vec![0.0; 100];
        assert!(mediador.calcular_entropia(&actividad) < 0.001);
    }

    #[test]
    fn test_entropia_caos() {
        let mediador = MediadorConsciencia::nuevo();
        let actividad = vec![1.0; 100]; // Disparo 100% homogéneo (máximo desorden)
        let ent = mediador.calcular_entropia(&actividad);
        assert!((ent - 1.0).abs() < 0.01, "Entropía homogénea debe ser cercana a 1.0: {}", ent);
    }

    #[test]
    fn test_procesar_corriente_basal() {
        let mut mediador = MediadorConsciencia::nuevo();
        let limbico = SistemaLimbico::nuevo();
        let corriente = mediador.procesar_corriente(
            None,
            &[],
            &[],
            0.1,
            &limbico.quimica,
            0.1,
            5.0,
            1.0,
            &EstadoEmocional::EnPaz,
            "hola",
        );

        assert_eq!(corriente.subconsciente[0], "rumiacion:ruido_basal");
        assert!(corriente.monologo_interno.contains("Ideas dispersas"));
        assert!(corriente.expresion_externa.contains("🧠 [CONSCIENTE]"));
    }

    #[test]
    fn test_procesar_corriente_estres() {
        let mut mediador = MediadorConsciencia::nuevo();
        let mut limbico = SistemaLimbico::nuevo();
        limbico.quimica.cortisol = 0.8; // Cortisol alto
        let corriente = mediador.procesar_corriente(
            None,
            &[],
            &[],
            0.4,
            &limbico.quimica,
            0.8, // Activación somática alta (estrés de hardware)
            12.0,
            0.5,
            &EstadoEmocional::Frustrado,
            "hola",
        );

        assert!(corriente.expresion_externa.contains("🔒 [PRECAUCIÓN DE SISTEMA]"));
    }

    #[test]
    fn test_procesar_corriente_resonancia_dopamina() {
        let mut mediador = MediadorConsciencia::nuevo();
        let mut limbico = SistemaLimbico::nuevo();
        limbico.quimica.dopamina = 0.95; // Dopamina alta
        
        let asambleas = vec![AsambleaSemantica {
            neuronas: vec![1, 2, 3],
            cohesion: 0.8,
            etiqueta: Some("aprendizaje".to_string()),
        }];
        let neuronas = vec![
            NeuronaCompacta::reposo(1, 1, 1),
            NeuronaCompacta::reposo(2, 1, 1),
            NeuronaCompacta::reposo(3, 1, 1),
        ];

        let corriente = mediador.procesar_corriente(
            Some(0),
            &asambleas,
            &neuronas,
            0.15,
            &limbico.quimica,
            0.2,
            6.0,
            1.5,
            &EstadoEmocional::Inspirado,
            "aprender",
        );

        assert_eq!(corriente.subconsciente[0], "aprendizaje");
        assert!(corriente.monologo_interno.contains("Concepto clave 'aprendizaje'"));
        assert!(corriente.expresion_externa.contains("💥 [SINOPSIS VIBRANTE]"));
    }
}
