// ============================================================================
// 🧠 RESONANCIA SEMÁNTICA — NodoConceptoExpandido con carga emocional mutable
// ============================================================================
// Propósito: Evolucionar el NodoConcepto estático de Synapse hacia una
//   unidad con frecuencia de uso, valencia emocional y perturbación
//   subconsciente. Esto permite que el GOI evite repeticiones, tenga
//   memoria de trauma y module su salida según el estado interno.
//
// Dependencias: uuid, chrono (ambas ya en Cargo.toml)
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Nodo conceptual expandido con propiedades emocionales y dinámicas.
///
/// A diferencia de NodoConcepto (Synapse), esta estructura:
/// - Tiene un identificador único (UUID) persistente
/// - Rastrea frecuencia de uso para evitar bucles repetitivos
/// - Almacena valencia emocional intrínseca (-1.0 a 1.0)
/// - Mantiene un grafo de asociaciones con otros nodos
/// - Registra el último impacto subconsciente recibido
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodoConceptoExpandido {
    pub id: Uuid,
    pub token_clave: String,
    /// Frecuencia de uso reciente (evita bucles repetitivos en el aparato fonador)
    pub frecuencia_uso: u32,
    /// Tono emocional intrínseco del concepto (-1.0 a 1.0)
    pub valencia_emocional: f64,
    /// Última vez que el subconsciente alteró este nodo
    pub ultimo_impacto: DateTime<Utc>,
    /// Nodos asociados (grafo de activación semántica latente).
    /// Key: UUID del nodo relacionado, Value: fuerza del enlace (0.0 a 1.0)
    pub asociaciones: HashMap<Uuid, f64>,
}

impl NodoConceptoExpandido {
    /// Crea un nuevo nodo con valencia inicial.
    ///
    /// `token`: la palabra clave del concepto.
    /// `valencia`: tono emocional intrínseco (-1.0 = negativo, 0.0 = neutro, 1.0 = positivo).
    pub fn new(token: &str, valencia: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            token_clave: token.to_string(),
            frecuencia_uso: 0,
            valencia_emocional: valencia.clamp(-1.0, 1.0),
            ultimo_impacto: Utc::now(),
            asociaciones: HashMap::new(),
        }
    }

    /// El subconsciente altera la valencia del concepto basado en traumas o éxitos.
    ///
    /// `intensidad_impacto`: perturbación recibida (-1.0 a 1.0).
    ///   Negativo = trauma, Positivo = éxito.
    ///
    /// Aplica atenuación homeostática: el impacto se mezcla con la valencia actual
    /// en lugar de reemplazarla, evitando cambios bruscos.
    pub fn registrar_perturbacion(&mut self, intensidad_impacto: f64) {
        let mezcla = 0.3; // Factor de homeóstasis — sólo 30% del impacto se aplica
        self.valencia_emocional = (self.valencia_emocional * (1.0 - mezcla)
            + intensidad_impacto * mezcla)
            .clamp(-1.0, 1.0);
        self.ultimo_impacto = Utc::now();
        self.frecuencia_uso += 1;
    }

    /// Evalúa si el concepto está saturado por repetición excesiva.
    /// Cuando está saturado, el EnsambladorVoz debe evitar usarlo.
    pub fn esta_saturado(&self) -> bool {
        self.frecuencia_uso > 5
    }

    /// Conecta este nodo con otro en el grafo semántico.
    ///
    /// `otro_id`: UUID del nodo destino.
    /// `fuerza`: peso del enlace (0.0 a 1.0).
    pub fn asociar(&mut self, otro_id: Uuid, fuerza: f64) {
        self.asociaciones.insert(otro_id, fuerza.clamp(0.0, 1.0));
    }

    /// Enfría la frecuencia de uso en 1 unidad (homeostasis temporal).
    /// Se llama en cada tic de fondo. Nunca baja de 0.
    pub fn enfriar(&mut self) {
        if self.frecuencia_uso > 0 {
            self.frecuencia_uso -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodo_new_valencia_inicial() {
        let nodo = NodoConceptoExpandido::new("trauma", -0.8);
        assert_eq!(nodo.token_clave, "trauma");
        assert!((nodo.valencia_emocional - (-0.8)).abs() < 0.01);
        assert_eq!(nodo.frecuencia_uso, 0);
        assert!(!nodo.esta_saturado());
    }

    #[test]
    fn test_perturbacion_homeostatica() {
        let mut nodo = NodoConceptoExpandido::new("alegria", 0.5);
        nodo.registrar_perturbacion(-0.8); // Impacto negativo fuerte
                                           // Debería moverse hacia negativo pero no bruscamente
        assert!(nodo.valencia_emocional < 0.5);
        assert!(nodo.valencia_emocional > -0.8);
        assert_eq!(nodo.frecuencia_uso, 1);
    }

    #[test]
    fn test_saturacion_por_repeticion() {
        let mut nodo = NodoConceptoExpandido::new("repetir", 0.0);
        for _ in 0..6 {
            nodo.registrar_perturbacion(0.1);
        }
        assert!(nodo.esta_saturado());
    }

    #[test]
    fn test_enfriamiento_reduce_frecuencia() {
        let mut nodo = NodoConceptoExpandido::new("vital", 0.3);
        nodo.registrar_perturbacion(0.5);
        nodo.registrar_perturbacion(0.5);
        assert_eq!(nodo.frecuencia_uso, 2);
        nodo.enfriar();
        assert_eq!(nodo.frecuencia_uso, 1);
        nodo.enfriar();
        assert_eq!(nodo.frecuencia_uso, 0);
        nodo.enfriar(); // No baja de 0
        assert_eq!(nodo.frecuencia_uso, 0);
    }

    #[test]
    fn test_asociar_conecta_nodos() {
        let mut nodo = NodoConceptoExpandido::new("padre", 0.9);
        let hijo_id = Uuid::new_v4();
        nodo.asociar(hijo_id, 0.8);
        assert!(nodo.asociaciones.contains_key(&hijo_id));
        assert!((nodo.asociaciones[&hijo_id] - 0.8).abs() < 0.01);
    }
}
