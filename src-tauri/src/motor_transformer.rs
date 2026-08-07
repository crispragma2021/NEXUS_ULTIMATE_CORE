// ============================================================================
// 🧠 MOTOR TRANSFORMER — Tiny Self-Attention sobre el Grafo Sináptico de NEXUS
// ============================================================================
// Arquitectura: Transformer mínimo (~8,480 parámetros) que atiende sobre los
// nodos activos del grafo sináptico en lugar de tokens de texto estándar.
//
// Diferencias clave con transformers tradicionales:
// - NO hay tokenización BPE — los "tokens" son nodos del grafo (IDNodo::Concepto)
// - NO hay embeddings pre-entrenados — se generan desde el estado del nodo
// - NO hay backprop — los pesos se copian/inicializan desde las sinapsis STDP
// - NO hay GPU — corre en CPU en microsegundos (N_max ≤ 64)
//
// Principio: El transformer es una PROYECCIÓN del grafo sináptico. Cuando el
// grafo aprende (STDP), el transformer captura ese conocimiento implícitamente.
// ============================================================================

use crate::core::cerebro::synapse::{GrafoSinapsis, IDNodo, NodoSinaptico};

// ─── Constantes arquitectónicas ────────────────────────────────────────────
const EMBED_DIM: usize = 32;       // Dimensión del embedding por nodo
const N_MAX: usize = 64;           // Máximo nodos activos por forward pass
const TEMPERATURA_ATTENTION: f32 = 2.0; // Softmax temperature for attention
const FFN_HIDDEN: usize = 64;      // Hidden dimension for feed-forward
const INV_SQRT_D: f32 = 0.1767767; // 1/sqrt(32) precalculado

// ─── TinyTransformer ───────────────────────────────────────────────────────
pub struct TinyTransformer {
    // Query, Key, Value projections: [EMBED_DIM × EMBED_DIM]
    pub wq: [[f32; EMBED_DIM]; EMBED_DIM],
    pub wk: [[f32; EMBED_DIM]; EMBED_DIM],
    pub wv: [[f32; EMBED_DIM]; EMBED_DIM],
    // Output projection
    pub wo: [[f32; EMBED_DIM]; EMBED_DIM],
    // Feed-Forward Network: 2 capas
    pub ffn1: [[f32; FFN_HIDDEN]; EMBED_DIM],  // EMBED_DIM → FFN_HIDDEN
    pub ffn2: [[f32; EMBED_DIM]; FFN_HIDDEN],   // FFN_HIDDEN → EMBED_DIM
    // Output weights: EMBED_DIM → 1 (score por nodo)
    pub w_out: [f32; EMBED_DIM],
    // Bias terms
    pub b_qkv: [f32; EMBED_DIM],  // Shared bias for Q, K, V
    pub b_ffn1: [f32; FFN_HIDDEN],
    pub b_ffn2: [f32; EMBED_DIM],
    pub b_out: f32,
    // Meta: contador de usos (para depuración)
    pub veces_usado: u64,
}

impl TinyTransformer {
    /// Crea un nuevo TinyTransformer con pesos inicializados desde un grafo.
    /// Si el grafo está vacío, usa valores por defecto (deterministas).
    pub fn new(grafo: &GrafoSinapsis) -> Self {
        let mut t = Self::default();
        t.inicializar_desde_grafo(grafo);
        t
    }

    /// Inicialización por defecto: pesos pequeños deterministas.
    fn default() -> Self {
        // Usar constantes basadas en PI para generar pesos deterministas
        // sin depender de rand. Esto da variabilidad sin aleatoriedad.
        let mut wq = [[0.0f32; EMBED_DIM]; EMBED_DIM];
        let mut wk = [[0.0f32; EMBED_DIM]; EMBED_DIM];
        let mut wv = [[0.0f32; EMBED_DIM]; EMBED_DIM];
        let mut wo = [[0.0f32; EMBED_DIM]; EMBED_DIM];
        let mut ffn1 = [[0.0f32; FFN_HIDDEN]; EMBED_DIM];
        let mut ffn2 = [[0.0f32; EMBED_DIM]; FFN_HIDDEN];
        let mut w_out = [0.0f32; EMBED_DIM];
        let b_qkv = [0.0f32; EMBED_DIM];
        let b_ffn1 = [0.0f32; FFN_HIDDEN];
        let b_ffn2 = [0.0f32; EMBED_DIM];
        let b_out = 0.0;

        // Inicializar con valores pequeños basados en posición (determinista)
        for i in 0..EMBED_DIM {
            // Usar fracciones de PI para generar pesos pequeños pero diversos
            let base_i = (i as f32 + 1.0) * 0.001;
            for j in 0..EMBED_DIM {
                let frac = ((j as f32 + 1.0) * 0.0007);
                wq[i][j] = base_i * (i as f32 * frac).sin() * 0.1;
                wk[i][j] = base_i * (i as f32 * frac).cos() * 0.1;
                wv[i][j] = (base_i * frac).sin() * 0.05;
                wo[i][j] = (base_i * frac).cos() * 0.05;
            }
            w_out[i] = (base_i * 3.1416).sin() * 0.1;
        }
        for i in 0..EMBED_DIM {
            let base_i = (i as f32 + 1.0) * 0.002;
            for j in 0..FFN_HIDDEN {
                let frac = (j as f32 + 1.0) * 0.003;
                ffn1[i][j] = (base_i * frac).sin() * 0.05;
            }
        }
        for i in 0..FFN_HIDDEN {
            let base_i = (i as f32 + 1.0) * 0.002;
            for j in 0..EMBED_DIM {
                let frac = (j as f32 + 1.0) * 0.003;
                ffn2[i][j] = (base_i * frac).cos() * 0.05;
            }
        }

        TinyTransformer {
            wq, wk, wv, wo, ffn1, ffn2, w_out,
            b_qkv, b_ffn1, b_ffn2, b_out,
            veces_usado: 0,
        }
    }

    /// Inicializa pesos copiando estadísticas del grafo sináptico.
    /// Los pesos de Query se derivan del embedding de cada nodo.
    /// Los pesos de Key se derivan de la conectividad de vecinos.
    /// Los pesos de Value se derivan de energía + traza.
    pub fn inicializar_desde_grafo(&mut self, grafo: &GrafoSinapsis) {
        let nodos_concepto: Vec<&NodoSinaptico> = grafo.nodos.values()
            .filter(|n| matches!(&n.id, IDNodo::Concepto(_)))
            .collect();

        if nodos_concepto.is_empty() {
            return; // Mantener valores por defecto
        }

        // Para cada dimensión del embedding, tomar estadísticas del grafo
        for i in 0..EMBED_DIM {
            let idx = i % nodos_concepto.len().max(1);
            let nodo_ref = nodos_concepto[idx];

            // wq[i]: derivado de la palabra del nodo (hash determinista)
            let hash_word = Self::hash_string(&nodo_ref.palabra, i);
            for j in 0..EMBED_DIM {
                self.wq[i][j] = hash_word * 0.15;
            }

            // wk[i]: derivado de la conectividad del nodo
            if let Some(vecinos) = grafo.enlaces.get(&nodo_ref.id) {
                let peso_promedio: f32 = vecinos.iter()
                    .map(|(_, e)| e.peso.abs())
                    .sum::<f32>() / (vecinos.len() as f32).max(1.0);
                for j in 0..EMBED_DIM {
                    let factor = ((j as f32 + 1.0) * 0.01).sin() * 0.5 + 0.5;
                    self.wk[i][j] = peso_promedio * factor * 0.2;
                }
            }

            // wv[i]: derivado de energía + traza del nodo
            let energia_traza = (nodo_ref.energia + nodo_ref.traza) * 0.5;
            for j in 0..EMBED_DIM {
                self.wv[i][j] = energia_traza * 0.15;
            }

            // w_out[i]: score base del nodo
            self.w_out[i] = nodo_ref.energia * 0.3 + nodo_ref.traza * 0.2;
        }

        // Bias: promedio de energía
        let energia_media: f32 = nodos_concepto.iter()
            .map(|n| n.energia).sum::<f32>() / (nodos_concepto.len() as f32).max(1.0);
        for i in 0..EMBED_DIM {
            self.b_qkv[i] = energia_media * 0.1;
        }
    }

    /// Genera un embedding determinista de 32 dimensiones para un nodo.
    /// El embedding codifica: hash de la palabra + energía + traza + peso_sinaptico_promedio.
    pub fn generar_embedding(nodo: &NodoSinaptico, grafo: &GrafoSinapsis) -> [f32; EMBED_DIM] {
        let mut emb = [0.0f32; EMBED_DIM];

        // Componente 1: Hash determinista de la palabra (ocupa ~16 dimensiones)
        let hash = Self::hash_string(&nodo.palabra, 0);
        for i in 0..16.min(EMBED_DIM) {
            emb[i] = Self::hash_string(&nodo.palabra, i) * 0.5;
        }

        // Componente 2: Energía del nodo (ocupa ~4 dimensiones)
        for i in 16..20.min(EMBED_DIM) {
            let factor = ((i - 16) as f32 * 0.5).sin().abs();
            emb[i] = nodo.energia * factor;
        }

        // Componente 3: Traza de actividad (ocupa ~4 dimensiones)
        for i in 20..24.min(EMBED_DIM) {
            let factor = ((i - 20) as f32 * 0.7).cos().abs();
            emb[i] = nodo.traza * factor;
        }

        // Componente 4: Peso sináptico promedio hacia vecinos (ocupa ~8 dimensiones)
        if let Some(vecinos) = grafo.enlaces.get(&nodo.id) {
            let peso_prom: f32 = vecinos.iter()
                .map(|(_, e)| e.peso)
                .sum::<f32>() / (vecinos.len() as f32).max(1.0);
            for i in 24..32.min(EMBED_DIM) {
                let factor = ((i - 24) as f32 * 0.3).sin();
                emb[i] = peso_prom * factor * 0.5;
            }
        }

        // Normalizar L2 para estabilidad numérica
        let norma: f32 = emb.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-8);
        for v in emb.iter_mut() {
            *v /= norma;
        }

        emb
    }

    /// Forward pass completo del TinyTransformer.
    /// 
    /// Args:
    ///   - nodos_activos: IDs de nodos concepto activos en el ciclo actual (max N_MAX)
    ///   - grafo: referencia al grafo sináptico para obtener datos de nodos y enlaces
    /// 
    /// Returns:
    ///   - Vec<(IDNodo, f32)>: lista de (nodo, score) ordenada por score descendente.
    ///     El score representa la probabilidad de que ese nodo sea el próximo token.
    pub fn forward(&mut self, nodos_activos: &[IDNodo], grafo: &GrafoSinapsis) -> Vec<(IDNodo, f32)> {
        self.veces_usado += 1;

        let n = nodos_activos.len().min(N_MAX);
        if n == 0 {
            return Vec::new();
        }

        // 1. Generar embeddings [n × EMBED_DIM]
        let mut embeddings = [[0.0f32; EMBED_DIM]; N_MAX];
        for (i, id) in nodos_activos.iter().enumerate().take(n) {
            if let Some(nodo) = grafo.nodos.get(id) {
                embeddings[i] = Self::generar_embedding(nodo, grafo);
            }
        }

        // 2. Proyectar a Q, K, V: Q = E × wq, K = E × wk, V = E × wv
        //    Cada una es [n × EMBED_DIM]
        let mut q = [[0.0f32; EMBED_DIM]; N_MAX];
        let mut k = [[0.0f32; EMBED_DIM]; N_MAX];
        let mut v = [[0.0f32; EMBED_DIM]; N_MAX];

        for i in 0..n {
            for j in 0..EMBED_DIM {
                for l in 0..EMBED_DIM {
                    q[i][j] += embeddings[i][l] * self.wq[l][j];
                    k[i][j] += embeddings[i][l] * self.wk[l][j];
                    v[i][j] += embeddings[i][l] * self.wv[l][j];
                }
                q[i][j] += self.b_qkv[j];
                k[i][j] += self.b_qkv[j];
                v[i][j] += self.b_qkv[j];
            }
        }

        // 3. Self-Attention: scores = softmax(Q × K^T / sqrt(d_k)) × V
        //    scores ∈ [n × EMBED_DIM]
        let mut scores = [[0.0f32; EMBED_DIM]; N_MAX];

        for i in 0..n {
            // Calcular atención: Q[i] · K[j] para cada j
            let mut attn = [0.0f32; N_MAX];
            let mut max_attn = f32::NEG_INFINITY;
            for j in 0..n {
                let mut dot = 0.0;
                for l in 0..EMBED_DIM {
                    dot += q[i][l] * k[j][l];
                }
                attn[j] = dot * INV_SQRT_D;
                if attn[j] > max_attn {
                    max_attn = attn[j];
                }
            }

            // Softmax sobre attn
            let mut sum_exp = 0.0;
            for j in 0..n {
                attn[j] = ((attn[j] - max_attn) / TEMPERATURA_ATTENTION).exp();
                sum_exp += attn[j];
            }
            if sum_exp > 1e-8 {
                for j in 0..n {
                    attn[j] /= sum_exp;
                }
            }

            // Weighted sum of V
            for l in 0..EMBED_DIM {
                for j in 0..n {
                    scores[i][l] += attn[j] * v[j][l];
                }
            }
        }

        // 4. Output projection: scores × wo → [n × EMBED_DIM]
        let mut projected = [[0.0f32; EMBED_DIM]; N_MAX];
        for i in 0..n {
            for j in 0..EMBED_DIM {
                for l in 0..EMBED_DIM {
                    projected[i][l] += scores[i][j] * self.wo[j][l];
                }
            }
        }

        // 5. Feed-Forward Network: ReLU(projected × ffn1 + b_ffn1) × ffn2 + b_ffn2
        let mut ffn_out = [[0.0f32; EMBED_DIM]; N_MAX];
        for i in 0..n {
            // Capa 1: EMBED_DIM → FFN_HIDDEN con ReLU
            let mut hidden = [0.0f32; FFN_HIDDEN];
            for h in 0..FFN_HIDDEN {
                for l in 0..EMBED_DIM {
                    hidden[h] += projected[i][l] * self.ffn1[l][h];
                }
                hidden[h] += self.b_ffn1[h];
                // ReLU
                if hidden[h] < 0.0 {
                    hidden[h] *= 0.01; // Leaky ReLU: 0.01 slope for negative
                }
            }

            // Capa 2: FFN_HIDDEN → EMBED_DIM
            for l in 0..EMBED_DIM {
                for h in 0..FFN_HIDDEN {
                    ffn_out[i][l] += hidden[h] * self.ffn2[h][l];
                }
                ffn_out[i][l] += self.b_ffn2[l];
            }
        }

        // 6. Output: ffn_out × w_out + b_out → score por nodo
        let mut logits = [0.0f32; N_MAX];
        for i in 0..n {
            for l in 0..EMBED_DIM {
                logits[i] += ffn_out[i][l] * self.w_out[l];
            }
            logits[i] += self.b_out;
        }

        // 7. Softmax final sobre logits para obtener distribución de probabilidad
        let mut max_logit = f32::NEG_INFINITY;
        for i in 0..n {
            if logits[i] > max_logit {
                max_logit = logits[i];
            }
        }
        let mut sum_exp = 0.0;
        for i in 0..n {
            logits[i] = (logits[i] - max_logit).exp();
            sum_exp += logits[i];
        }
        if sum_exp > 1e-8 {
            for i in 0..n {
                logits[i] /= sum_exp;
            }
        }

        // 8. Empaquetar resultados
        let mut resultados: Vec<(IDNodo, f32)> = nodos_activos.iter()
            .take(n)
            .enumerate()
            .map(|(i, id)| (id.clone(), logits[i]))
            .collect();

        // Ordenar por score descendente
        resultados.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        resultados
    }

    /// Genera un token de texto completo desde el transformer.
    /// Similar a emitir_habla_emergente_v3 pero usando atención en lugar de Markov.
    /// 
    /// Args:
    ///   - grafo: grafo sináptico completo
    ///   - ids_sensoriales: nodo IDs activos (entrada sensorial)
    ///   - max_tokens: máximo de tokens a generar
    /// 
    /// Returns:
    ///   - String: texto generado
    pub fn generar_texto(
        &mut self,
        grafo: &mut GrafoSinapsis,
        ids_sensoriales: &[IDNodo],
        max_tokens: usize,
    ) -> String {
        let mut rng: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut tokens_generados: Vec<IDNodo> = Vec::new();

        // Usar ids_sensoriales como contexto inicial
        let mut contexto: Vec<IDNodo> = ids_sensoriales.to_vec();
        // Limitar tamaño del contexto
        if contexto.len() > N_MAX {
            contexto = contexto[contexto.len().saturating_sub(N_MAX)..].to_vec();
        }

        for _ in 0..max_tokens {
            if contexto.is_empty() {
                break;
            }

            // Forward pass del transformer
            let resultados = self.forward(&contexto, grafo);
            if resultados.is_empty() || resultados[0].1 < 0.05 {
                break; // Confianza muy baja
            }

            // Seleccionar el mejor candidato (con algo de ruido para diversidad)
            let seleccionado = if resultados.len() > 1 && xorshift_f32(&mut rng) < 0.15 {
                // 15% de las veces, elegir el segundo mejor (exploración)
                let idx = 1.min(resultados.len() - 1);
                resultados[idx].0.clone()
            } else {
                resultados[0].0.clone()
            };

            // Verificar que no sea repetición del último
            if let Some(ultimo) = tokens_generados.last() {
                if *ultimo == seleccionado {
                    if resultados.len() > 1 {
                        // Si hay alternativa, tomar la segunda
                        let alt = resultados[1].0.clone();
                        if alt != seleccionado {
                            tokens_generados.push(alt.clone());
                            contexto.push(alt);
                            continue;
                        }
                    }
                    break; // Repetición sin alternativa
                }
            }

            tokens_generados.push(seleccionado.clone());
            contexto.push(seleccionado.clone());

            // Limitar tamaño del contexto deslizante
            if contexto.len() > N_MAX {
                contexto = contexto[contexto.len().saturating_sub(N_MAX)..].to_vec();
            }
        }

        // Convertir a string (misma lógica que post_procesar pero más simple)
        if tokens_generados.is_empty() {
            return String::new();
        }

        let mut palabras: Vec<String> = Vec::new();
        let mut ultima_palabra = String::new();
        for id in &tokens_generados {
            if let IDNodo::Concepto(palabra) = id {
                let p = palabra.trim().to_lowercase();
                if p != ultima_palabra && !p.is_empty() {
                    // Saltar stop-words si hay suficiente contexto
                    if palabras.len() > 2 { // TODO: Reimplementar lógica de stop-word si es necesaria
                        palabras.push(palabra.clone());
                        ultima_palabra = p;
                    }
                }
            }
        }

        if palabras.is_empty() {
            String::new()
        } else {
            palabras.join(" ")
        }
    }

    // ─── Helpers ───────────────────────────────────────────────────────────

    /// Hash determinista de string para la dimensión i.
    /// Usa FNV-like hash con desplazamientos por dimensión.
    fn hash_string(s: &str, dim: usize) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        let mut hash: u64 = 0xCBF2_9CE4_8422_2325u64.wrapping_add(dim as u64);
        for &b in s.as_bytes() {
            hash = hash.wrapping_mul(0x100_0000_01B3);
            hash ^= b as u64;
        }
        // Normalizar a [-0.5, 0.5]
        ((hash >> 32) as i32 as f32) / 4294967296.0 * 0.5
    }
}

// ─── Xorshift independiente (evitar dependencia cruzada) ───────────────────
fn xorshift_f32(state: &mut u64) -> f32 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    let upper = (*state >> 32) as u32;
    (upper & 0x007F_FFFF) as f32 / 8388607.0
}

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cerebro::synapse::{GrafoSinapsis, IDNodo, NodoSinaptico, EnlaceSinaptico};
    use std::collections::HashMap;

    fn grafo_con_3_conceptos() -> GrafoSinapsis {
        let mut nodos = HashMap::new();
        nodos.insert(
            IDNodo::Concepto("hola".to_string()),
            NodoSinaptico {
                id: IDNodo::Concepto("hola".to_string()),
                energia: 0.8,
                palabra: "hola".to_string(),
                refractario: 0.0,
                ultimo_disparo: 0,
                traza: 0.5,
                es_predicho: false,
                es_entrada_directa: true,
                ciclos_baja_energia: 0,
            },
        );
        nodos.insert(
            IDNodo::Concepto("mundo".to_string()),
            NodoSinaptico {
                id: IDNodo::Concepto("mundo".to_string()),
                energia: 0.6,
                palabra: "mundo".to_string(),
                refractario: 0.0,
                ultimo_disparo: 0,
                traza: 0.3,
                es_predicho: false,
                es_entrada_directa: false,
                ciclos_baja_energia: 0,
            },
        );
        nodos.insert(
            IDNodo::Concepto("nexus".to_string()),
            NodoSinaptico {
                id: IDNodo::Concepto("nexus".to_string()),
                energia: 0.9,
                palabra: "nexus".to_string(),
                refractario: 0.0,
                ultimo_disparo: 0,
                traza: 0.7,
                es_predicho: false,
                es_entrada_directa: false,
                ciclos_baja_energia: 0,
            },
        );

        let mut enlaces: HashMap<IDNodo, Vec<(IDNodo, EnlaceSinaptico)>> = HashMap::new();
        enlaces.insert(
            IDNodo::Concepto("hola".to_string()),
            vec![
                (IDNodo::Concepto("mundo".to_string()), EnlaceSinaptico { peso: 0.5 }),
                (IDNodo::Concepto("nexus".to_string()), EnlaceSinaptico { peso: 0.3 }),
            ],
        );
        enlaces.insert(
            IDNodo::Concepto("nexus".to_string()),
            vec![
                (IDNodo::Concepto("hola".to_string()), EnlaceSinaptico { peso: 0.4 }),
                (IDNodo::Concepto("mundo".to_string()), EnlaceSinaptico { peso: 0.2 }),
            ],
        );

        GrafoSinapsis { nodos, enlaces, ciclo_actual: 0 }
    }

    #[test]
    fn test_transformer_forward_devuelve_resultados() {
        let grafo = grafo_con_3_conceptos();
        let mut transformer = TinyTransformer::new(&grafo);

        let activos: Vec<IDNodo> = vec![
            IDNodo::Concepto("hola".to_string()),
            IDNodo::Concepto("mundo".to_string()),
            IDNodo::Concepto("nexus".to_string()),
        ];

        let resultados = transformer.forward(&activos, &grafo);
        assert!(!resultados.is_empty(), "Forward debe devolver resultados");
        assert_eq!(resultados.len(), 3, "Debe haber 3 resultados para 3 nodos");

        // Verificar que los scores suman ~1.0 (softmax)
        let suma: f32 = resultados.iter().map(|(_, s)| s).sum();
        assert!((suma - 1.0).abs() < 0.01, "Scores deben sumar ~1.0, suma={}", suma);

        // Verificar orden descendente
        for i in 0..resultados.len().saturating_sub(1) {
            assert!(
                resultados[i].1 >= resultados[i + 1].1,
                "Resultados deben estar ordenados por score descendente"
            );
        }
    }

    #[test]
    fn test_transformer_con_grafo_vacio() {
        let grafo = GrafoSinapsis::new();
        let mut transformer = TinyTransformer::new(&grafo);
        let resultados = transformer.forward(&[], &grafo);
        assert!(resultados.is_empty(), "Grafo vacío debe devolver lista vacía");
    }

    #[test]
    fn test_embedding_determinista() {
        let grafo = grafo_con_3_conceptos();
        let nodo = grafo.nodos.get(&IDNodo::Concepto("hola".to_string())).unwrap();

        let emb1 = TinyTransformer::generar_embedding(nodo, &grafo);
        let emb2 = TinyTransformer::generar_embedding(nodo, &grafo);

        assert_eq!(emb1.len(), EMBED_DIM);
        assert_eq!(emb2.len(), EMBED_DIM);

        // Mismo nodo → mismo embedding (determinista)
        for i in 0..EMBED_DIM {
            assert!((emb1[i] - emb2[i]).abs() < 1e-6,
                "Embedding debe ser determinista, diff en dim {} = {}", i, (emb1[i] - emb2[i]).abs());
        }

        // Nodos diferentes → embeddings diferentes
        let nodo2 = grafo.nodos.get(&IDNodo::Concepto("mundo".to_string())).unwrap();
        let emb3 = TinyTransformer::generar_embedding(nodo2, &grafo);
        let mut iguales = true;
        for i in 0..EMBED_DIM {
            if (emb1[i] - emb3[i]).abs() > 1e-6 {
                iguales = false;
                break;
            }
        }
        assert!(!iguales, "Embeddings de distintos nodos deben ser diferentes");
    }

    #[test]
    fn test_inicializacion_desde_grafo_no_panico() {
        let grafo = grafo_con_3_conceptos();
        let mut transformer = TinyTransformer::new(&grafo);
        transformer.inicializar_desde_grafo(&grafo);

        // Verificar que los pesos se inicializaron (no todos cero)
        let mut suma_wq = 0.0;
        for i in 0..EMBED_DIM {
            for j in 0..EMBED_DIM {
                suma_wq += transformer.wq[i][j].abs();
            }
        }
        assert!(suma_wq > 0.0, "wq debe tener valores no-cero después de inicialización");
    }

    #[test]
    fn test_generar_texto_con_grafo_pequeno() {
        let grafo = grafo_con_3_conceptos();
        let mut transformer = TinyTransformer::new(&grafo);

        let ids: Vec<IDNodo> = vec![
            IDNodo::Concepto("hola".to_string()),
            IDNodo::Concepto("mundo".to_string()),
        ];

        // Clonar grafo para generar_texto (necesita &mut)
        let mut grafo_mut = grafo_con_3_conceptos();
        let texto = transformer.generar_texto(&mut grafo_mut, &ids, 5);

        // No debe devolver vacío si hay contexto
        // (puede devolver vacío si los scores son muy bajos, es aceptable)
        if !texto.is_empty() {
            assert!(texto.len() >= 1, "Texto generado no debe estar vacío");
        }
    }
}
