// ==========================================
// 🦎 CEREBRO REPTILIANO — Prioridades e inferencias instintivas
// ==========================================
// ==========================================

/// Prioridad de una inferencia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum InferencePriority {
    Baja,
    #[default]
    Normal,
    Alta,
    Critica,
    /// Alias en inglés para compatibilidad con omega_stress.rs.
    High,
}

/// Solicitud de inferencia instintiva.
#[derive(Debug, Clone, Default)]
pub struct InferenceRequest {
    /// Prompt principal de la solicitud.
    pub prompt: String,
    /// Prompt de sistema opcional.
    pub system_prompt: Option<String>,
    /// Imagen en base64 opcional.
    pub image_b64: Option<String>,
    /// Modelo a usar (opcional — usa el activo por defecto).
    pub model: Option<String>,
    /// Límite de tokens de salida.
    pub max_tokens: Option<u32>,
    /// Temperatura de muestreo.
    pub temperature: Option<f32>,
    /// Prioridad de la inferencia.
    pub priority: InferencePriority,
}
