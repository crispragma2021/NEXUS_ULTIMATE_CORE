// ============================================================================
// 🧠 CUERPO CALLOSO GENERADOR — Puente Synapse ↔ MemoriaSemántica
// ============================================================================
// Propósito: Traduce constelaciones de conceptos activos en fragmentos
//   recuperables de MemoriaSemántica (LanceDB).
//
// Capa 2 del GOI: después de que la Corteza Asociativa (Synapse) activa
//   conceptos, esta capa recupera los fragmentos de memoria correspondientes.
// ============================================================================

use crate::cerebro::generador::{MAX_FRAGMENTOS_POR_CONSULTA, UMBRAL_ACTIVACION};
use crate::cerebro::organos::chunker::Chunker;
use crate::cerebro::synapse::MotorSynapse;
use crate::memoria::memoria_semantica::MemoriaSemantica;
use std::sync::Arc;

// ─── ESTRUCTURAS ────────────────────────────────────────────────────────────

/// Fragmento de memoria recuperado, listo para ser evaluado por el selector.
#[derive(Debug, Clone)]
pub struct FragmentoCandidato {
    /// Texto completo del fragmento recuperado.
    pub texto: String,
    /// Nivel de activación del concepto que originó este fragmento.
    pub activacion_origen: f32,
    /// Tono emocional asociado al fragmento (-1.0 a 1.0).
    pub tono_emocional: f32,
    /// Nombre del concepto/sistema fuente.
    pub fuente: String,
    /// ID del fragmento en MemoriaSemántica (LanceDB).
    pub id_fragmento: u64,
}

/// Puente entre la activación de conceptos y la recuperación de memoria.
pub struct CuerpoCallosoGenerador {
    /// Referencia al Synapse (Capa 1).
    pub synapse: Arc<std::sync::Mutex<MotorSynapse>>,
    /// Referencia a la MemoriaSemántica (LanceDB).
    pub semantica: Arc<MemoriaSemantica>,
    /// Chunker para tokenizar fragmentos si es necesario.
    pub chunker: Chunker,
}

impl CuerpoCallosoGenerador {
    /// Crea una nueva instancia del Cuerpo Calloso Generador.
    pub fn new(
        synapse: Arc<std::sync::Mutex<MotorSynapse>>,
        semantica: Arc<MemoriaSemantica>,
    ) -> Self {
        Self {
            synapse,
            semantica,
            chunker: Chunker::default(),
        }
    }

    /// Traduce una constelación de conceptos activos en fragmentos recuperables.
    ///
    /// Para cada concepto activo con activación > umbral, genera un embedding
    /// y busca en LanceDB fragmentos semánticamente similares. Si no hay
    /// resultados en LanceDB, cae en fallback sintético (Broca).
    ///
    /// FASE 2 — Cableado LanceDB real:
    ///   1. Generar embedding del concepto_id (NexusEmbedder soberano)
    ///   2. buscar_similares_con_texto(vector, MAX_FRAGMENTOS_POR_CONSULTA, "ocean_vectors")
    ///   3. Recuperar textos completos desde la columna `esencia` de LanceDB
    ///   4. Mapear a FragmentoCandidato con tono emocional inferido
    ///
    /// Retorna fragmentos ordenados por activación descendente.
    pub async fn recuperar_fragmentos(
        &self,
        constelacion: &[(String, f32)],
    ) -> Vec<FragmentoCandidato> {
        let mut fragmentos = Vec::new();

        for (concepto_id, activacion) in constelacion {
            if *activacion < UMBRAL_ACTIVACION {
                continue;
            }

            // ─── Búsqueda real en LanceDB ───────────────────────────────
            // Generar embedding del concepto activo

            // 🧬 FASE 2: Cableado LanceDB real
            // `self.semantica.generar_embedding()` usa nexus_embedder::NexusEmbedder
            // (SHA-256 angular ⊕ pesado nodal MotorSynapse, 768-dim, L2-normalizado).
            //
            // Si LanceDB devuelve fragmentos, se convierten a FragmentoCandidato.
            // Si falla (tabla vacía, error de embedding, etc.), se cae en fallback.
            let mut lancedb_exitoso = false;

            // Consultar solo si hay una URI real (no memory:// que solo vive en tests)
            let vector = self.semantica.generar_embedding(concepto_id).await;
            if let Ok(embedding) = vector {
                match self
                    .semantica
                    .buscar_similares_con_texto(
                        &embedding,
                        MAX_FRAGMENTOS_POR_CONSULTA,
                        "ocean_vectors",
                    )
                    .await
                {
                    Ok(resultados) if !resultados.is_empty() => {
                        for (id_frag, texto, distancia) in &resultados {
                            // Convertir distancia L2 a score de similitud (0..1)
                            // LanceDB usa distancia L2: menor = más similar
                            let score = 1.0 / (1.0 + distancia);

                            // Inferir tono emocional tentativo desde el texto
                            // (heurística simple: palabras negativas → tono negativo)
                            let tono = inferir_tono_desde_texto(texto);

                            fragmentos.push(FragmentoCandidato {
                                texto: texto.clone(),
                                activacion_origen: *activacion * score,
                                tono_emocional: tono,
                                fuente: concepto_id.clone(),
                                id_fragmento: *id_frag as u64,
                            });
                        }
                        lancedb_exitoso = true;
                    }
                    _ => { /* Fallback: intentar con otra tabla o Broca */ }
                }
            }

            // ─── Fallback a Broca sintética ──────────────────────────────
            // Si LanceDB no tiene datos (primera ejecución, tabla vacía, etc.),
            // generamos texto sintético desde el concepto activo usando la Broca.
            // Esto garantiza que el GOI siempre tenga fragmentos que procesar.
            if !lancedb_exitoso {
                let texto_sintetico = self.sintetizar_desde_concepto(concepto_id, *activacion);
                fragmentos.push(FragmentoCandidato {
                    texto: texto_sintetico,
                    activacion_origen: *activacion,
                    tono_emocional: 0.0,
                    fuente: concepto_id.clone(),
                    id_fragmento: 0,
                });
            }
        }

        // Ordenar por activación descendente
        fragmentos.sort_by(|a, b| {
            b.activacion_origen
                .partial_cmp(&a.activacion_origen)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limitar al máximo
        fragmentos.truncate(MAX_FRAGMENTOS_POR_CONSULTA);

        fragmentos
    }

    /// Sintetiza un fragmento de texto desde un concepto activo.
    /// Función temporal mientras no haya búsqueda real en MemoriaSemántica.
    fn sintetizar_desde_concepto(&self, concepto_id: &str, activacion: f32) -> String {
        if activacion < UMBRAL_ACTIVACION {
            return String::new();
        }

        // Generar texto usando la Broca interna
        let conceptos = vec![(concepto_id.to_string(), activacion)];
        self.synapse
            .lock()
            .map(|syn| syn.broca.sintetizar(&conceptos))
            .unwrap_or_else(|_| String::new())
    }

    /// Recupera un fragmento completo de LanceDB por su ID numérico.
    #[allow(dead_code)]
    async fn recuperar_por_id(&self, id: u64) -> Option<String> {
        // Consultar LanceDB usando buscar_similares_con_texto con un embedding
        // del texto vacío para recuperar por ID (aproximación).
        // TODO: Si LanceDB soporta búsqueda por ID nativa, reemplazar.
        if let Ok(vector) = self.semantica.generar_embedding("").await {
            if let Ok(resultados) = self
                .semantica
                .buscar_similares_con_texto(&vector, 10, "ocean_vectors")
                .await
            {
                for (id_frag, texto, _dist) in &resultados {
                    if *id_frag as u64 == id {
                        return Some(texto.clone());
                    }
                }
            }
        }
        None
    }
}

// ─── Helper: inferir tono emocional desde texto ───────────────────────────
//
// Heurística simple de bolsa de palabras: cuenta palabras con carga
// emocional conocida y retorna un promedio ponderado en rango -1.0 a 1.0.
// Esto permite que el CuerpoCalloso asigne un tono tentativo a fragmentos
// de LanceDB sin depender de un clasificador externo.

fn inferir_tono_desde_texto(texto: &str) -> f32 {
    let lower = texto.to_lowercase();
    let palabras: Vec<&str> = lower.split_whitespace().collect();
    if palabras.is_empty() {
        return 0.0;
    }

    let mut carga: f32 = 0.0;
    let mut contadas: usize = 0;

    for palabra in &palabras {
        // Palabras de valencia negativa (trauma, dificultad, error)
        if matches!(
            *palabra,
            "error"
                | "fallo"
                | "falló"
                | "grave"
                | "dolor"
                | "perdí"
                | "perdi"
                | "perdido"
                | "difícil"
                | "dificil"
                | "crítico"
                | "critico"
                | "emergencia"
                | "colapso"
                | "triste"
                | "mal"
                | "peor"
                | "nunca"
                | "problema"
        ) {
            carga -= 0.4;
            contadas += 1;
        }
        // Palabras de valencia positiva (éxito, logro, alegría)
        else if matches!(
            *palabra,
            "bien"
                | "logro"
                | "éxito"
                | "exito"
                | "alegría"
                | "alegria"
                | "feliz"
                | "genial"
                | "gracias"
                | "brillante"
                | "orgullo"
                | "hermoso"
                | "paz"
                | "amor"
                | "esperanza"
        ) {
            carga += 0.3;
            contadas += 1;
        }
    }

    if contadas == 0 {
        0.0
    } else {
        (carga / contadas as f32).clamp(-1.0, 1.0)
    }
}
