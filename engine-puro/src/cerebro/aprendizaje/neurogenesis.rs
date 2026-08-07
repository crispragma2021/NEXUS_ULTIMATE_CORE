use std::collections::{HashMap, VecDeque};
use crate::cerebro::aprendizaje::conceptos::ProtoConcepto;
use crate::cerebro::estructuras::NeuronaCompacta;
use crate::cerebro::memoria::{MemoriaAdaptativa, UbicacionMemoria};

// ============================================================================
// MOTOR 3: NEUROGÉNESIS — Crea nuevas neuronas para conceptos y tokens nuevos
// ============================================================================

pub struct MotorNeurogenesis {
    pub frecuencia_tokens: HashMap<u32, u64>,
    pub token_a_neuronas: HashMap<u32, Vec<u32>>,
    pub cola_conceptos: VecDeque<ProtoConcepto>,
    pub neuronas_creadas: Vec<u32>,
    pub total_creadas: u64,
    pub max_neuronas: usize,
    pub umbral_frecuencia: u64,
    pub ventana_observacion: u64,
    pub paso_actual: u64,
}

impl MotorNeurogenesis {
    pub fn nuevo() -> Self {
        Self {
            frecuencia_tokens: HashMap::new(),
            token_a_neuronas: HashMap::new(),
            cola_conceptos: VecDeque::new(),
            neuronas_creadas: Vec::new(),
            total_creadas: 0,
            max_neuronas: 10000,
            umbral_frecuencia: 5,
            ventana_observacion: 1000,
            paso_actual: 0,
        }
    }

    pub fn registrar_token(&mut self, token_id: u32) {
        *self.frecuencia_tokens.entry(token_id).or_insert(0) += 1;
    }

    pub fn solicitar_neurona_para_concepto(&mut self, concepto: ProtoConcepto) {
        if concepto.neurona_hub.is_some() { return; }
        for existente in &self.cola_conceptos {
            if existente.miembros == concepto.miembros { return; }
        }
        self.cola_conceptos.push_back(concepto);
    }

    /// Procesa la cola y crea neuronas.
    /// Recibe memoria y siguiente_id por separado para evitar
    /// conflictos de borrow checker en el pipeline.
    /// Las conexiones token→neurona se gestionan internamente (mapa propio),
    /// sin depender de ningún léxico estadístico externo.
    pub fn procesar(
        &mut self,
        memoria: &mut MemoriaAdaptativa,
        siguiente_id: &mut u32,
    ) -> Vec<(u32, Vec<u32>)> {
        let mut nuevas = Vec::new();

        // 1. Procesar conceptos encolados
        while let Some(concepto) = self.cola_conceptos.pop_front() {
            if self.total_creadas >= self.max_neuronas as u64 { break; }
            if concepto.miembros.iter().any(|m| self.token_a_neuronas.contains_key(m)) {
                continue;
            }
            let neurona_id = crear_neurona_en_memoria(memoria, siguiente_id);
            self.neuronas_creadas.push(neurona_id);
            self.total_creadas += 1;

            for &token_id in &concepto.miembros {
                self.token_a_neuronas.entry(token_id).or_default().push(neurona_id);
            }
            nuevas.push((neurona_id, concepto.miembros.clone()));
        }

        // 2. Procesar tokens frecuentes sin neurona
        let candidatos: Vec<u32> = self.frecuencia_tokens.iter()
            .filter(|(&tid, &freq)| freq >= self.umbral_frecuencia && !self.token_a_neuronas.contains_key(&tid))
            .map(|(&id, _)| id)
            .collect();

        for token_id in candidatos {
            if self.total_creadas >= self.max_neuronas as u64 { break; }
            let neurona_id = crear_neurona_en_memoria(memoria, siguiente_id);
            self.neuronas_creadas.push(neurona_id);
            self.total_creadas += 1;
            self.token_a_neuronas.entry(token_id).or_default().push(neurona_id);
            nuevas.push((neurona_id, vec![token_id]));
        }

        nuevas
    }

    pub fn decaer_frecuencias(&mut self) {
        for freq in self.frecuencia_tokens.values_mut() { *freq /= 2; }
        self.frecuencia_tokens.retain(|_, &mut freq| freq > 0);
    }

    pub fn total_creadas(&self) -> u64 { self.total_creadas }
}

/// Función helper para crear una neurona en capa 2 y agregarla a la memoria,
/// replicando la lógica de CerebroAutoOptimizable::crear_neurona
fn crear_neurona_en_memoria(memoria: &mut MemoriaAdaptativa, siguiente_id: &mut u32) -> u32 {
    use rand::Rng;
    let id = *siguiente_id;
    *siguiente_id += 1;
    let mut rng = rand::thread_rng();
    let neurona = NeuronaCompacta::aleatoria(id, 0, 2, &mut || rng.gen());
    memoria.ram.agregar_neurona(neurona);
    memoria.mapa_memoria.insert(id, UbicacionMemoria::RAM);
    if memoria.vram.is_some() && !memoria.vram.as_ref().unwrap().esta_lleno() {
        memoria.mover_a_vram(id);
    }
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::cerebro::CerebroAutoOptimizable;

    #[test]
    fn test_registrar_token() {
        let mut m = MotorNeurogenesis::nuevo();
        m.registrar_token(42);
        m.registrar_token(42);
        m.registrar_token(42);
        assert_eq!(*m.frecuencia_tokens.get(&42).unwrap_or(&0), 3);
    }

    #[test]
    fn test_decaer_frecuencias() {
        let mut m = MotorNeurogenesis::nuevo();
        m.registrar_token(10);
        m.registrar_token(10);
        m.registrar_token(20);
        m.decaer_frecuencias();
        assert_eq!(*m.frecuencia_tokens.get(&10).unwrap_or(&0), 1);
        assert!(!m.frecuencia_tokens.contains_key(&20), "Frecuencia 0 debe ser limpiada");
    }

    #[test]
    fn test_solicitar_concepto() {
        let mut m = MotorNeurogenesis::nuevo();
        let c = ProtoConcepto { miembros: vec![1, 2, 3], neurona_hub: None, peso: 0.5 };
        m.solicitar_neurona_para_concepto(c);
        assert_eq!(m.cola_conceptos.len(), 1);
    }

    #[test]
    fn test_procesar_con_frecuencia_alta() {
        let mut neuro = MotorNeurogenesis::nuevo();
        neuro.umbral_frecuencia = 3;
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        for _ in 0..4 { neuro.registrar_token(42); }
        let nuevas = neuro.procesar(&mut cerebro.memoria, &mut cerebro.siguiente_id);
        assert!(nuevas.len() >= 1, "Debe crear neurona");
    }

    #[test]
    fn test_procesar_sin_suficiente_frecuencia() {
        let mut neuro = MotorNeurogenesis::nuevo();
        neuro.umbral_frecuencia = 10;
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        neuro.registrar_token(7);
        assert_eq!(neuro.procesar(&mut cerebro.memoria, &mut cerebro.siguiente_id).len(), 0);
    }

    #[test]
    fn test_max_neuronas() {
        let mut neuro = MotorNeurogenesis::nuevo();
        neuro.max_neuronas = 2;
        neuro.umbral_frecuencia = 1;
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        for i in 0..5 {
            neuro.registrar_token(100 + i);
            neuro.solicitar_neurona_para_concepto(
                ProtoConcepto { miembros: vec![100 + i], neurona_hub: None, peso: 0.5 }
            );
        }
        assert!(neuro.procesar(&mut cerebro.memoria, &mut cerebro.siguiente_id).len() <= 2);
    }

    #[test]
    fn test_procesar_concepto_encolado() {
        let mut neuro = MotorNeurogenesis::nuevo();
        neuro.umbral_frecuencia = 1;
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        neuro.solicitar_neurona_para_concepto(
            ProtoConcepto { miembros: vec![10, 20], neurona_hub: None, peso: 0.8 }
        );
        let nuevas = neuro.procesar(&mut cerebro.memoria, &mut cerebro.siguiente_id);
        assert_eq!(nuevas.len(), 1);
        assert!(nuevas[0].1.contains(&10));
        assert!(nuevas[0].1.contains(&20));
    }
}
