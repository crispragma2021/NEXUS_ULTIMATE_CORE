// ============================================================================
// 🎭 MOTOR DE ASAMBLEAS SEMÁNTICAS (MAS) — Lenguaje Biológico Puro
// ============================================================================
// Reemplaza al Motor Léxico Sinclair (estadístico).
// Aquí el lenguaje no son tokens, sino patrones de disparo sincronizado
// en asambleas neuronales distribuidas.
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsambleaSemantica {
    /// IDs de las neuronas que forman esta asamblea
    pub neuronas: Vec<u32>,
    /// Fuerza de cohesión interna de la asamblea (0.0 - 1.0)
    pub cohesion: f32,
    /// Concepto asociado (opcional, para depuración del Arquitecto)
    pub etiqueta: Option<String>,
}

pub struct MotorAsambleasSemanticas {
    /// Mapa de asambleas activas
    pub asambleas: Vec<AsambleaSemantica>,
    /// Mapeo Perceptivo: Caracter/Fonema -> Asamblea Perceptiva
    /// Nota: En un sistema 100% puro esto emergería, pero para la interfaz
    /// de texto usamos este mapeo como 'puerta de entrada sensorial'.
    pub mapa_perceptivo: HashMap<char, Vec<u32>>,
    /// Umbral de sincronía Gamma para considerar una asamblea 'activa'
    pub umbral_sincronia: f32,
}

impl MotorAsambleasSemanticas {
    pub fn nuevo() -> Self {
        Self {
            asambleas: Vec::new(),
            mapa_perceptivo: HashMap::new(),
            umbral_sincronia: 0.8,
        }
    }

    /// Traduce un texto en estímulos de asambleas perceptivas
    /// Cada caracter activa un grupo de neuronas en la corteza sensorial.
    pub fn percibir_texto(&self, texto: &str) -> Vec<u32> {
        let mut neuronas_a_activar = Vec::new();
        for c in texto.chars() {
            if let Some(nids) = self.mapa_perceptivo.get(&c) {
                neuronas_a_activar.extend(nids);
            }
        }
        neuronas_a_activar
    }

    /// Crea una nueva asamblea basada en la actividad actual sincronizada
    pub fn consolidar_asamblea(&mut self, neuronas: Vec<u32>, etiqueta: Option<String>) {
        let asamblea = AsambleaSemantica {
            neuronas,
            cohesion: 0.5, // Inicial
            etiqueta,
        };
        self.asambleas.push(asamblea);
    }

    /// Busca la asamblea que más resuena con un patrón de disparo dado.
    ///
    /// Scoring Jaccard (`intersección / unión`): penaliza asambleas pequeñas
    /// que se solapan parcialmente con un patrón amplio. Un solapamiento
    /// parcial sobre una asamblea grande ya no satura la puntuación.
    pub fn detectar_resonancia(&self, neuronas_activas: &[u32]) -> Option<usize> {
        let mut max_coincidencia = 0.0;
        let mut ganador = None;

        for (i, asamblea) in self.asambleas.iter().enumerate() {
            if asamblea.neuronas.is_empty() {
                continue;
            }
            let mut coincidencia = 0;
            for &n in neuronas_activas {
                if asamblea.neuronas.contains(&n) {
                    coincidencia += 1;
                }
            }
            // Jaccard: intersección / unión.
            let interseccion = coincidencia as f32;
            let union = (asamblea.neuronas.len() + neuronas_activas.len()) as f32 - interseccion;
            let puntuacion = if union > 0.0 { interseccion / union } else { 0.0 };
            if puntuacion > max_coincidencia && puntuacion > self.umbral_sincronia {
                max_coincidencia = puntuacion;
                ganador = Some(i);
            }
        }
        ganador
    }

    /// Genera una salida simbólica basada en la asamblea que está resonando.
    /// Es el puente final de vuelta al lenguaje humano (Área de Broca Digital).
    pub fn articular_idea(&self, indice_asamblea: usize) -> String {
        if let Some(asamblea) = self.asambleas.get(indice_asamblea) {
            asamblea.etiqueta.clone().unwrap_or_else(|| "[IDEA EMERGENTE NO ETIQUETADA]".to_string())
        } else {
            "...".to_string()
        }
    }

    /// Articulación con fallback en cadena (mejor esfuerzo).
    ///
    /// Cuando ninguna asamblea supera el umbral estricto de `detectar_resonancia`,
    /// este método devuelve la asamblea con MAYOR solapamiento parcial en lugar
    /// del silencio. Prioriza coincidencia parcial alta → luego el índice de la
    /// primera asamblea con cualquier solapamiento → último recurso "..." (silencio
    /// biológico legítimo, no bloqueo).
    pub fn articular_idea_extendida(&self, neuronas_activas: &[u32]) -> String {
        if neuronas_activas.is_empty() {
            return "...".to_string();
        }
        let mut mejor_puntuacion = 0.0f32;
        let mut mejor_idx: Option<usize> = None;
        let mut primer_solapamiento: Option<usize> = None;

        for (i, asamblea) in self.asambleas.iter().enumerate() {
            if asamblea.neuronas.is_empty() {
                continue;
            }
            let mut coincidencia = 0;
            for &n in neuronas_activas {
                if asamblea.neuronas.contains(&n) {
                    coincidencia += 1;
                }
            }
            if coincidencia == 0 {
                continue;
            }
            if primer_solapamiento.is_none() {
                primer_solapamiento = Some(i);
            }
            let interseccion = coincidencia as f32;
            let union = (asamblea.neuronas.len() + neuronas_activas.len()) as f32 - interseccion;
            let puntuacion = if union > 0.0 { interseccion / union } else { 0.0 };
            if puntuacion > mejor_puntuacion {
                mejor_puntuacion = puntuacion;
                mejor_idx = Some(i);
            }
        }

        if let Some(idx) = mejor_idx {
            self.articular_idea(idx)
        } else if let Some(idx) = primer_solapamiento {
            self.articular_idea(idx)
        } else {
            "...".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nuevo_estado_inicial() {
        let m = MotorAsambleasSemanticas::nuevo();
        assert!(m.asambleas.is_empty());
        assert!(m.mapa_perceptivo.is_empty());
        casi(m.umbral_sincronia, 0.8);
    }

    fn casi(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "esperado {} obtenido {}", b, a);
    }

    #[test]
    fn test_consolidar_asamblea_crea_con_cohesion_inicial() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.consolidar_asamblea(vec![1, 2, 3], Some("perro".to_string()));
        assert_eq!(m.asambleas.len(), 1);
        casi(m.asambleas[0].cohesion, 0.5);
        assert_eq!(m.asambleas[0].neuronas, vec![1, 2, 3]);
        assert_eq!(m.asambleas[0].etiqueta.as_deref(), Some("perro"));
    }

    #[test]
    fn test_percibir_texto_vacio() {
        let m = MotorAsambleasSemanticas::nuevo();
        assert!(m.percibir_texto("").is_empty());
    }

    #[test]
    fn test_percibir_texto_sin_mapa() {
        let m = MotorAsambleasSemanticas::nuevo();
        // Sin mapeo perceptivo, no activa ninguna neurona
        assert!(m.percibir_texto("hola").is_empty());
    }

    #[test]
    fn test_percibir_texto_activa_neuronas_del_mapa() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.mapa_perceptivo.insert('a', vec![10, 11]);
        m.mapa_perceptivo.insert('b', vec![20]);
        let activadas = m.percibir_texto("ab");
        assert_eq!(activadas, vec![10, 11, 20]);
    }

    #[test]
    fn test_percibir_texto_repite_caracteres() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.mapa_perceptivo.insert('x', vec![5]);
        let activadas = m.percibir_texto("xx");
        assert_eq!(activadas, vec![5, 5]);
    }

    #[test]
    fn test_detectar_resonancia_sin_asambleas() {
        let m = MotorAsambleasSemanticas::nuevo();
        assert!(m.detectar_resonancia(&[1, 2, 3]).is_none());
    }

    #[test]
    fn test_detectar_resonancia_sobre_umbral() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.consolidar_asamblea(vec![1, 2, 3], Some("gato".to_string()));
        // 3 de 3 neuronas coinciden → puntuación 1.0 > 0.8
        let idx = m.detectar_resonancia(&[1, 2, 3]).unwrap();
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_detectar_resonancia_bajo_umbral_ignora() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.consolidar_asamblea(vec![1, 2, 3, 4, 5, 6], Some("grande".to_string()));
        // Solo 2 de 6 coinciden → 0.33 < 0.8 → no resuena
        assert!(m.detectar_resonancia(&[1, 2, 99, 100]).is_none());
    }

    #[test]
    fn test_detectar_resonancia_elige_la_mejor() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.consolidar_asamblea(vec![1, 2], Some("a".to_string()));
        m.consolidar_asamblea(vec![3, 4], Some("b".to_string()));
        // Coincide plenamente con la segunda asamblea
        let idx = m.detectar_resonancia(&[3, 4]).unwrap();
        assert_eq!(idx, 1);
    }

    #[test]
    fn test_articular_idea_con_etiqueta() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.consolidar_asamblea(vec![1], Some("luz".to_string()));
        assert_eq!(m.articular_idea(0), "luz");
    }

    #[test]
    fn test_articular_idea_sin_etiqueta() {
        let mut m = MotorAsambleasSemanticas::nuevo();
        m.consolidar_asamblea(vec![1], None);
        assert_eq!(m.articular_idea(0), "[IDEA EMERGENTE NO ETIQUETADA]");
    }

    #[test]
    fn test_articular_idea_indice_invalido() {
        let m = MotorAsambleasSemanticas::nuevo();
        assert_eq!(m.articular_idea(5), "...");
    }
}
