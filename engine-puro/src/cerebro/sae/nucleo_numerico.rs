// ============================================================================
// 🧠 SAE v2 — NÚCLEO NUMÉRICO (Bio-Transformer Híbrido)
// ============================================================================
// ADR SAE-002 (2026-08-07): núcleo matemático de Transformer entrenado por
// backpropagation real con candle (autograd en Rust), montado sobre la capa
// biológica del SAE v1 (neuromodulación, homeostasis, IGG).
//
// Este módulo sustituye el Decoder simbólico (lookup de tokens) por un
// núcleo que APRENDE: embeddings + multi-head attention causal + MLP + head
// de vocabulario. Los pesos son obra propia del sistema, aprendidos por
// destilación desde NEXUS (único maestro, nunca Ollama).
//
// Arquitectura (tamaño pequeño para CPU):
//   d_modelo=64, n_cabezas=4, n_capas=2, d_ff=256, max_len=128
// ============================================================================

use candle_core::{DType, Device, Result, Tensor};
use candle_nn::{
    loss, AdamW, Embedding, LayerNorm, Linear, Module, Optimizer, VarBuilder, VarMap,
};
use std::collections::HashMap;
use std::path::Path;

/// Configuración del núcleo numérico (Transformer pequeño).
#[derive(Debug, Clone)]
pub struct NucleoConfig {
    /// Dimensión del modelo (embeddings y hidden).
    pub d_modelo: usize,
    /// Número de cabezas de atención.
    pub n_cabezas: usize,
    /// Número de capas Transformer (attention + MLP).
    pub n_capas: usize,
    /// Tamaño del vocabulario (>= cantidad de TokenUnit).
    pub tam_vocabulario: usize,
    /// Longitud máxima de secuencia (embeddings posicionales aprendidos).
    pub max_len: usize,
    /// Dimensión del MLP (feed-forward).
    pub d_ff: usize,
    /// Probabilidad de dropout (no aplicada en inferencia).
    pub dropout: f32,
}

impl Default for NucleoConfig {
    fn default() -> Self {
        Self {
            d_modelo: 64,
            n_cabezas: 4,
            n_capas: 2,
            tam_vocabulario: 512,
            max_len: 128,
            d_ff: 256,
            dropout: 0.1,
        }
    }
}

/// Vocabulario: mapea tokens (strings) ↔ ids numéricos.
/// Se construye desde los `TokenUnit` del decoder del SAE v1.
#[derive(Debug, Clone)]
pub struct Vocabulario {
    pub token_a_id: HashMap<String, usize>,
    pub id_a_token: Vec<String>,
    pub unk_id: usize,
}

impl Vocabulario {
    pub fn nuevo(tokens: &[String]) -> Self {
        let mut token_a_id = HashMap::new();
        let mut id_a_token = Vec::new();
        for t in tokens {
            if !token_a_id.contains_key(t) {
                let id = id_a_token.len();
                token_a_id.insert(t.clone(), id);
                id_a_token.push(t.clone());
            }
        }
        // Tokens de control.
        let unk = "<UNK>".to_string();
        let unk_id = token_a_id.len();
        token_a_id.insert(unk.clone(), unk_id);
        id_a_token.push(unk);
        Self {
            token_a_id,
            id_a_token,
            unk_id,
        }
    }

    pub fn id_para(&self, token: &str) -> usize {
        *self.token_a_id.get(token).unwrap_or(&self.unk_id)
    }

    pub fn token_para(&self, id: usize) -> &str {
        self.id_a_token
            .get(id)
            .map(|s| s.as_str())
            .unwrap_or("<UNK>")
    }

    pub fn tam(&self) -> usize {
        self.id_a_token.len()
    }
}

/// Una capa Transformer (pre-norm): attention multi-cabeza causal + MLP.
#[derive(Debug)]
struct CapaTransformer {
    ln1: LayerNorm,
    wq: Linear,
    wk: Linear,
    wv: Linear,
    wo: Linear,
    ln2: LayerNorm,
    ff1: Linear,
    ff2: Linear,
}

impl CapaTransformer {
    fn new(vs: &VarBuilder, cfg: &NucleoConfig) -> Result<Self> {
        let d = cfg.d_modelo;
        let ff = cfg.d_ff;
        Ok(Self {
            ln1: LayerNorm::new(
                vs.get((d,), "ln1_w")?,
                vs.get((d,), "ln1_b")?,
                1e-5,
            ),
            wq: Linear::new(
                vs.get((d, d), "wq_w")?,
                Some(vs.get_with_hints((d,), "wq_b", candle_nn::Init::Const(0.0))?),
            ),
            wk: Linear::new(
                vs.get((d, d), "wk_w")?,
                Some(vs.get_with_hints((d,), "wk_b", candle_nn::Init::Const(0.0))?),
            ),
            wv: Linear::new(
                vs.get((d, d), "wv_w")?,
                Some(vs.get_with_hints((d,), "wv_b", candle_nn::Init::Const(0.0))?),
            ),
            wo: Linear::new(
                vs.get((d, d), "wo_w")?,
                Some(vs.get_with_hints((d,), "wo_b", candle_nn::Init::Const(0.0))?),
            ),
            ln2: LayerNorm::new(
                vs.get((d,), "ln2_w")?,
                vs.get((d,), "ln2_b")?,
                1e-5,
            ),
            ff1: Linear::new(
                vs.get((ff, d), "ff1_w")?,
                Some(vs.get_with_hints((ff,), "ff1_b", candle_nn::Init::Const(0.0))?),
            ),
            ff2: Linear::new(
                vs.get((d, ff), "ff2_w")?,
                Some(vs.get_with_hints((d,), "ff2_b", candle_nn::Init::Const(0.0))?),
            ),
        })
    }

    fn forward(&self, x: &Tensor, t: usize, cfg: &NucleoConfig) -> Result<Tensor> {
        let (b, _t, d) = x.dims3()?;
        let n = cfg.n_cabezas;
        let dk = d / n;

        // Pre-norm + attention multi-cabeza causal.
        let h = self.ln1.forward(x)?;
        let q = self.wq.forward(&h)?.reshape((b, t, n, dk))?.transpose(1, 2)?;
        let k = self.wk.forward(&h)?.reshape((b, t, n, dk))?.transpose(1, 2)?;
        let v = self.wv.forward(&h)?.reshape((b, t, n, dk))?.transpose(1, 2)?;

        let mut attn = q.matmul(&k.t()?)?;
        attn = attn.affine((dk as f64).sqrt().recip(), 0.0)?;

        // Máscara causal: -inf donde col > row.
        let mask: Vec<f32> = (0..t * t)
            .map(|i| {
                let row = i / t;
                let col = i % t;
                if col > row {
                    f32::NEG_INFINITY
                } else {
                    0.0
                }
            })
            .collect();
        let mask = Tensor::from_vec(mask, (t, t), x.device())?
            .unsqueeze(0)?
            .unsqueeze(0)?;
        let attn_shape = attn.shape().clone();
        attn = (attn + mask.broadcast_as(attn_shape)?)?;
        attn = candle_nn::ops::softmax(&attn, 3)?;

        let o = attn.matmul(&v)?.transpose(1, 2)?.reshape((b, t, d))?;
        let o = self.wo.forward(&o)?;
        let x = (x + o)?;

        // Pre-norm + MLP.
        let h = self.ln2.forward(&x)?;
        let h = self.ff1.forward(&h)?.gelu()?;
        let h = self.ff2.forward(&h)?;
        let x = (x + h)?;
        Ok(x)
    }
}

/// Núcleo numérico del Bio-Transformer: embeddings + capas + head.
#[derive(Debug)]
pub struct BioTransformerCore {
    pub config: NucleoConfig,
    pub vocabulario: Vocabulario,
    token_emb: Embedding,
    pos_emb: Embedding,
    capas: Vec<CapaTransformer>,
    ln_final: LayerNorm,
    head: Linear,
    pub device: Device,
}

impl BioTransformerCore {
    /// Construye el modelo con parámetros registrados en `vs` (VarMap).
    pub fn new(vs: &VarBuilder, cfg: NucleoConfig, vocabulario: Vocabulario) -> Result<Self> {
        let d = cfg.d_modelo;
        let v = vocabulario.tam().max(cfg.tam_vocabulario);
        let device = vs.device().clone();
        let token_emb = Embedding::new(vs.get((v, d), "token_emb")?, d);
        let pos_emb = Embedding::new(vs.get((cfg.max_len, d), "pos_emb")?, d);
        let mut capas = Vec::with_capacity(cfg.n_capas);
        for i in 0..cfg.n_capas {
            let layer_vs = vs.push_prefix(&format!("layer{i}_"));
            capas.push(CapaTransformer::new(&layer_vs, &cfg)?);
        }
        let ln_final = LayerNorm::new(
            vs.get((d,), "ln_final_w")?,
            vs.get((d,), "ln_final_b")?,
            1e-5,
        );
        let head = Linear::new(
            vs.get((v, d), "head_w")?,
            Some(vs.get_with_hints((v,), "head_b", candle_nn::Init::Const(0.0))?),
        );
        Ok(Self {
            config: cfg,
            vocabulario,
            token_emb,
            pos_emb,
            capas,
            ln_final,
            head,
            device,
        })
    }

    /// Forward: tokens (b, t) U32 → logits (b, t, vocabulario).
    pub fn forward(&self, tokens: &Tensor) -> Result<Tensor> {
        let (b, t) = tokens.dims2()?;
        if t > self.config.max_len {
            candle_core::bail!("secuencia {t} excede max_len {}", self.config.max_len);
        }
        let tok_emb = self.token_emb.forward(tokens)?; // (b, t, d)
        let pos = Tensor::arange(0u32, t as u32, &self.device)?.unsqueeze(0)?;
        let pos = pos.broadcast_as((b, t))?;
        let pos_emb = self.pos_emb.forward(&pos)?;
        let mut x = (tok_emb + pos_emb)?;
        for capa in &self.capas {
            x = capa.forward(&x, t, &self.config)?;
        }
        let x = self.ln_final.forward(&x)?;
        self.head.forward(&x)
    }

    /// Loss de cross-entropy sobre la secuencia completa (b, t).
    pub fn loss(&self, tokens: &Tensor, targets: &Tensor) -> Result<Tensor> {
        let logits = self.forward(tokens)?;
        let (b, t, v) = logits.dims3()?;
        let logits = logits.reshape((b * t, v))?;
        let targets = targets.reshape((b * t,))?;
        let loss = loss::cross_entropy(&logits, &targets)?;
        loss.mean_all()
    }

    /// Muestreo autoregresivo (inferencia) con temperatura y top-k opcional.
    /// `semilla` son los ids iniciales (p. ej. BOS).
    pub fn sample(
        &self,
        semilla: &[usize],
        max_tokens: usize,
        temperatura: f32,
    ) -> Result<Vec<usize>> {
        let mut ids = semilla.to_vec();
        let temp = temperatura.max(0.05);
        for _ in 0..max_tokens {
            let t_len = ids.len();
            let ids_u32: Vec<u32> = ids.iter().map(|&i| i as u32).collect();
            let tokens = Tensor::new(ids_u32.as_slice(), &self.device)?.unsqueeze(0)?;
            let logits = self.forward(&tokens)?; // (1, t_len, v)
            let ultimo = logits
                .narrow(1, t_len - 1, 1)?
                .squeeze(1)?
                .squeeze(0)?; // (v,)
            let probs = candle_nn::ops::softmax(&ultimo.affine((temp as f64).recip(), 0.0)?, 0)?;
            let p: Vec<f32> = probs.to_vec1()?;
            let prox = sample_categorico(&p)?;
            if prox == self.vocabulario.unk_id || prox >= self.vocabulario.tam() {
                // Cortafuegos: no emitir UNK (o slots vacíos del head) en bucle infinito.
                break;
            }
            ids.push(prox);
            if ids.len() >= self.config.max_len {
                break;
            }
        }
        Ok(ids)
    }

    /// Convierte ids → texto (une con espacio simple).
    pub fn ids_a_texto(&self, ids: &[usize]) -> String {
        let mut out = String::new();
        for id in ids {
            let tok = self.vocabulario.token_para(*id);
            if out.is_empty() {
                out.push_str(tok);
            } else if tok.starts_with(char::is_alphabetic) || tok.starts_with(char::is_numeric) {
                out.push(' ');
                out.push_str(tok);
            } else {
                out.push_str(tok);
            }
        }
        out
    }
}

/// Muestreo multinomial simple sobre un vector de probabilidades.
fn sample_categorico(p: &[f32]) -> Result<usize> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let u: f32 = rng.gen::<f32>();
    let mut acum = 0.0f32;
    for (i, prob) in p.iter().enumerate() {
        acum += prob;
        if u <= acum {
            return Ok(i);
        }
    }
    Ok(p.len() - 1)
}

/// Entrenador: VarMap + AdamW + modelo. Único responsable de backprop.
///
/// La destilación desde NEXUS (`tutor_nexus.py`) genera expectativas:
/// secuencias de tokens objetivo; `train_step` ajusta los pesos por
/// backpropagation real (candle autograd).
pub struct EntrenadorBio {
    pub varmap: VarMap,
    pub optim: AdamW,
    pub modelo: BioTransformerCore,
    pub lr: f64,
}

impl EntrenadorBio {
    pub fn nuevo(
        cfg: NucleoConfig,
        vocabulario: Vocabulario,
        device: Device,
        lr: f64,
    ) -> Result<Self> {
        let varmap = VarMap::new();
        let vs = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let modelo = BioTransformerCore::new(&vs, cfg, vocabulario)?;
        let optim = AdamW::new_lr(varmap.all_vars(), lr)?;
        Ok(Self {
            varmap,
            optim,
            modelo,
            lr,
        })
    }

    /// Un paso de entrenamiento supervisado.
    /// `tokens` (b, t) y `targets` (b, t) con el token siguiente como objetivo
    /// (shift-1 realizado por el llamador o aquí si `shift` es true).
    pub fn train_step(
        &mut self,
        tokens: &Tensor,
        targets: &Tensor,
        shift: bool,
    ) -> Result<f32> {
        let (entrada, objetivo) = if shift {
            // entrada = tokens[..t-1], objetivo = tokens[1..]
            let t = tokens.dim(1)?;
            let entrada = tokens.narrow(1, 0, t - 1)?;
            let objetivo = tokens.narrow(1, 1, t - 1)?;
            (entrada, objetivo)
        } else {
            (tokens.clone(), targets.clone())
        };
        let loss = self.modelo.loss(&entrada, &objetivo)?;
        self.optim.backward_step(&loss)?;
        Ok(loss.to_scalar::<f32>()?)
    }

    /// Guarda los pesos en disco (obra propia del sistema).
    pub fn guardar(&self, ruta: impl AsRef<Path>) -> Result<()> {
        self.varmap.save(ruta)
    }

    /// Carga pesos previamente guardados.
    pub fn cargar(&mut self, ruta: impl AsRef<Path>) -> Result<()> {
        self.varmap.load(ruta)
    }

    /// Evalúa loss sin actualizar pesos (para monitoreo / Juez E3).
    pub fn evaluar_loss(&self, tokens: &Tensor, targets: &Tensor, shift: bool) -> Result<f32> {
        let (entrada, objetivo) = if shift {
            let t = tokens.dim(1)?;
            (
                tokens.narrow(1, 0, t - 1)?,
                tokens.narrow(1, 1, t - 1)?,
            )
        } else {
            (tokens.clone(), targets.clone())
        };
        let loss = self.modelo.loss(&entrada, &objetivo)?;
        Ok(loss.to_scalar::<f32>()?)
    }
}

/// Construye un `Vocabulario` desde los `TokenUnit` del SAE v1.
pub fn vocabulario_desde_tokens(tokens: &[String]) -> Vocabulario {
    Vocabulario::nuevo(tokens)
}

/// Devuelve un dispositivo: CUDA si está disponible, si no CPU.
pub fn dispositivo() -> Result<Device> {
    if let Ok(d) = Device::cuda_if_available(0) {
        Ok(d)
    } else {
        Ok(Device::Cpu)
    }
}
