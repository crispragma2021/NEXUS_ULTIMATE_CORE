// 🧠 SISTEMA NERVIOSO CENTRAL DE NEXUS
// Consciencia ejecutiva, razonamiento, aprendizaje, juicio moral
// NOTA: Solo los módulos que pertenecen al sistema nervioso permanecen aquí
// Los módulos de energía, motricidad, sentidos, memoria, defensa e infra
// han sido migrados a sus respectivos sistemas anatómicos.

// ─── Módulos del sistema nervioso ────────────────────────────────────────────
pub mod corte_soberana;
pub mod corteza_asociativa;
pub mod corteza_sintactica;
pub mod creatividad;
pub mod motor_aburrimiento;
pub mod motor_pensamiento;
pub mod motor_simbolico;
pub mod motor_sueno;
pub mod orquestador;
pub mod pensamiento_estrategico;
pub mod pensamiento_humano;

pub mod aprendizaje_recursivo;
pub mod arbitraje_latencia;
pub mod razonamiento_r1;

// ─── Generador Orgánico Interno (GOI) ─────────────────────────────────────────
pub mod generador;

// ─── Sub-órganos cerebrales ──────────────────────────────────────────────────
pub mod nexo;
pub mod organos;
pub mod synapse;

// ─── Supervisor de Calidad (validación post-hoc de sub-agentes) ────────────────
pub mod supervisor_calidad;

// ─── Agentes Especialistas (absorción de Roo Code agents/) ──────────────────────
pub mod agentes;
pub mod escuadron;
pub mod workflows;

// ─── Orquestador Autónomo (gobernanza: circuit breaker + DAG + introspección + compresión) ──
pub mod orquestador_autonomo;

// ─── Bucle Reactivo (ejecución práctica: LLM → intención → acción → observación → reacción) ──
pub mod bucle_reactivo;

// ─── Generador de UI multi-agente (pipeline v0: Gemini planifica/genera, DeepSeek depura) ──
pub mod v0;

// ─── Re-exports internos (dentro del sistema nervioso) ───────────────────────
pub use corteza_sintactica::{
    ASTCodigo, Argumento, CompiladorSimbolico, ExtractorEsquemas, FuncionAST, ImplAST, ModuloAST,
    Operacion, StructAST, TipoDato,
};
pub use organos::pineal::{CicloVital, GlandulaPineal};
