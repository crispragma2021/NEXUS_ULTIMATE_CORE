// ==========================================
// 💖 MOTOR AFECTIVO — Personalidad
// ==========================================
// Define la personalidad del organismo.
// ==========================================

/// Personalidad activa del organismo.
#[derive(Debug, Clone, Default)]
pub struct Personality {
    /// Nombre de la personalidad.
    pub nombre: String,
    /// Rasgos (ej. "directa", "pragmática").
    pub rasgos: Vec<String>,
    /// Lealtad (0.0-1.0).
    pub lealtad: f32,
}

impl Personality {
    pub fn new(nombre: impl Into<String>) -> Self {
        Self {
            nombre: nombre.into(),
            rasgos: Vec::new(),
            lealtad: 1.0,
        }
    }
}
