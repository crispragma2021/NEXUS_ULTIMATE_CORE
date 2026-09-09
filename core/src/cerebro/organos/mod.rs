pub mod amygdala;
pub mod area_broca;
pub mod cerebelo;
pub mod cingulo_anterior;
pub mod clasificador_regimen;
pub mod corteza_motora;
pub mod corteza_parietal;
pub mod corteza_prefrontal;
pub mod cuerpo_calloso;
pub mod ganglios_basales;
pub mod hipocampo;
pub mod hipotalamo;
pub mod insula;
/// NOTA: `intuicion` debe permanecer pub — referenciado directamente desde
/// pensamiento_humano, constructor, infra/boot, infra/mundo_interno y prediccion.
/// Usar preferentemente `organos::intuition::*` para imports nuevos.
pub mod intuicion; // 🧠 Implementación real (433 líneas)
pub mod intuition; // 🔌 API pública unificada (re-exporta todo desde intuicion)
pub mod lobulo_occipital_estetico;
pub mod lobulo_temporal;
pub mod metacognicion;
pub mod motor_mercado;
pub mod narrativa_interna;
pub mod neocorteza;
pub mod proprioception;
pub mod talamo;
pub mod teoria_mente;
pub mod voluntad_propia;

// 🧠 ÓRGANOS RAG — Sistema de Recuperación Aumentada
pub mod chunker;
pub mod ingesta;
pub mod reranker;
pub mod retrieval;

// ─── Puentes de compatibilidad ──────────────────────────────────────────
// Los hemisferios energéticos fueron migrados a crate::energia
pub use crate::efectores::medula_soberana;
pub use crate::emociones::apego;
pub use crate::energia::hemisferio_derecho;
pub use crate::energia::hemisferio_groq;
pub use crate::energia::hemisferio_izquierdo;
pub use crate::neuroquimica::glandula_dopamina;
pub use crate::neuroquimica::pineal;
pub use crate::sentidos::nexus_acoustic;
