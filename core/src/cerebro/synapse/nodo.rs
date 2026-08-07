// core/src/cerebro/synapse/nodo.rs

#[derive(Debug, Clone)]
pub struct NodoConcepto {
    pub id: String,
    pub activacion: f32,                // De 0.0 a 1.0
    pub conexiones: Vec<(String, f32)>, // (ID del concepto vecino, peso de la relación)
}

impl NodoConcepto {
    pub fn new(id: &str, activacion_inicial: f32) -> Self {
        Self {
            id: id.to_string(),
            activacion: activacion_inicial.clamp(0.0, 1.0),
            conexiones: Vec::new(),
        }
    }
}
