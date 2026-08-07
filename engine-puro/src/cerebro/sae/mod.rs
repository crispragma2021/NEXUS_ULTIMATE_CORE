// ============================================================================
// 🧠 SPIKING ATTENTION ENGINE (SAE) — Atención Biológica para engine-puro
// ============================================================================
// Arquitectura documentada en plans/engine-puro-spiking-attention.md
//
// Captura la esencia matemática de la autoatención de los Transformers
// (Q·K^T → softmax → V) pero implementada sobre neuronas spiking con
// plasticidad biológica:
//   - StateEncoder: estado interno → trenes de spikes (tálamo)
//   - SpikingAttention: convolución temporal Q·K + neuromodulación + competencia
//   - Decoder: spikes → tokens (Área de Broca)
//   - Plasticity: STDP + homeostasis + structural plasticity
//   - Consolidation: ciclo vigilia-sueño
// ============================================================================

pub mod state_encoder;
pub mod spiking_attention;
pub mod decoder;
pub mod plasticity;
pub mod consolidation;
pub mod generator;
pub mod nucleo_numerico;
pub mod capa_biologica;
pub mod juez_e3;

// Re-export de la API principal.
pub use state_encoder::StateEncoder;
pub use spiking_attention::{SpikingAttention, AttentionConfig};
pub use decoder::{Decoder, DecoderConfig, TokenChoice};
pub use plasticity::{PlasticityEngine, Synapse};
pub use consolidation::{ConsolidationEngine, SleepPhase};
pub use generator::{
    Generator, GeneratorConfig, GeneratedSequence, GenerationStep, IntentionEncoder,
};
pub use nucleo_numerico::{
    BioTransformerCore, EntrenadorBio, NucleoConfig, Vocabulario, dispositivo,
    vocabulario_desde_tokens,
};
pub use capa_biologica::{
    CapaBiologica, CapaBiologicaConfig, ParametrosModulados, PasoIgg, SecuenciaIgg,
};
pub use juez_e3::{
    DictamenE3, EvaluacionSecuencia, FaseE3, dictaminar, evaluar_secuencia, reportar,
};

use crate::cerebro::sistema_limbico::Neuroquimica;
use crate::cerebro::sae::decoder::TokenUnit;

/// Configuración completa del SAE.
#[derive(Debug, Clone)]
pub struct SaeConfig {
    pub encoder: state_encoder::StateEncoder,
    pub attention: AttentionConfig,
    pub decoder: DecoderConfig,
}

impl Default for SaeConfig {
    fn default() -> Self {
        Self {
            encoder: state_encoder::StateEncoder::default(),
            attention: AttentionConfig::default(),
            decoder: DecoderConfig::default(),
        }
    }
}

/// Resultado de un ciclo de inferencia del SAE.
#[derive(Debug, Clone)]
pub struct SaeOutput {
    /// Token expresado (si el sistema decidió hablar).
    pub token: Option<TokenChoice>,
    /// Matriz de atención del último bloque.
    pub attention: Vec<Vec<f32>>,
    /// Output de activación (para plasticidad).
    pub activation: Vec<f32>,
}

/// Spiking Attention Engine orquestado: encoder → atención → decoder → IGG.
///
/// Toma el estado interno (vector + neuroquímica) y produce expresión, o
/// silencio si la tensión no supera el umbral del decodificador. Para
/// generar secuencias (oraciones), usa el Generador Guiado por Intención
/// (IGG) que reduce el vector de intención hasta la homeostasis.
pub struct SpikingAttentionEngine {
    pub config: SaeConfig,
    pub encoder: StateEncoder,
    pub attention: SpikingAttention,
    pub decoder: Decoder,
    pub generator: Generator,
}

impl SpikingAttentionEngine {
    pub fn new(config: SaeConfig, vocabulario: Vec<decoder::TokenUnit>) -> Self {
        let encoder = config.encoder.clone();
        let attention = SpikingAttention::new(config.attention.clone());
        let decoder = Decoder::new(config.decoder.clone(), vocabulario.clone());
        let generator = Generator::new(GeneratorConfig::default());
        Self {
            config,
            encoder,
            attention,
            decoder,
            generator,
        }
    }

    /// Genera una SECUENCIA completa (oración) guiada por intención.
    ///
    /// A diferencia de `forward` (un token), esto reduce el vector de
    /// intención M token a token hasta alcanzar la homeostasis (‖R‖ < ε).
    /// La neuroquímica modula α (dopamina), ε (cortisol) y el ruido
    /// (noradrenalina) durante la generación.
    pub fn generar_secuencia(
        &self,
        estado_vector: &[f32],
        memoria: &[decoder::TokenUnit],
        memoria_act: &[f32],
        neuroquimica: &Neuroquimica,
    ) -> GeneratedSequence {
        self.generator
            .generate(estado_vector, memoria, memoria_act, neuroquimica)
    }

    /// Ejecuta un ciclo completo: estado interno → spikes → atención → token.
    pub fn forward(
        &self,
        estado_vector: &[f32],
        values: &[f32],
        neuroquimica: &Neuroquimica,
    ) -> SaeOutput {
        // 1. Codificar estado a trenes de spikes (una dimensión → un tren Query).
        let estado = state_encoder::EstadoEntrada {
            vector: estado_vector.to_vec(),
            etiquetas: vec![],
        };
        let q_trains = self.encoder.encode(&estado);

        // 2. Las Keys son los mismos trenes (auto-atención biológica).
        let k_trains = q_trains.clone();

        // 3. Atención spiking.
        let att_out = self.attention.forward(&q_trains, &k_trains, values, neuroquimica);

        // 4. Decodificar la activación de salida en un token.
        let token = self.decoder.decode(&att_out.output, neuroquimica);

        SaeOutput {
            token,
            attention: att_out.attention_matrix,
            activation: att_out.output,
        }
    }
}
