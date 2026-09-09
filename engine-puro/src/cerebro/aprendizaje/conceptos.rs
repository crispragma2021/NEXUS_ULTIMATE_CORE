use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::cerebro::estructuras::ImpactoConceptual;

// ============================================================================
// MOTOR 2: FORMADOR DE CONCEPTOS — Agrupa tokens relacionados por co-ocurrencia
// ============================================================================
// Detecta qué tokens aparecen juntos frecuentemente y los agrupa en
// proto-conceptos. Usa una matriz de co-ocurrencia: (token_a, token_b) → conteo.
// Cada N pasos (500), escanea la matriz y forma clusters.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProtoConcepto {
    /// IDs de tokens que forman este concepto
    pub miembros: Vec<u32>,
    /// ID de la neurona hub que representa este concepto
    pub neurona_hub: Option<u32>,
    /// Peso del concepto (0.0-1.0): qué tan fuerte es la asociación
    pub peso: f32,
}

pub struct MotorConceptos {
    /// Matriz de co-ocurrencia: (token_a, token_b) → conteo
    /// Solo almacena token_a < token_b para evitar duplicados
    pub(crate) co_ocurrencias: HashMap<(u32, u32), u32>,

    /// Proto-conceptos formados
    pub conceptos: Vec<ProtoConcepto>,

    /// Umbral de co-ocurrencia para formar un concepto
    pub umbral_coocurrencia: u32,  // 10

    /// Tamaño de la ventana de contexto en tokens
    pub ventana_contexto: usize,  // 5

    /// Paso actual para escaneo periódico
    pub paso_actual: u64,

    /// Cada cuántos pasos se ejecuta el clustering
    pub cadencia_agrupacion: u64,  // 500

    /// Contador de conceptos formados
    pub conceptos_formados: u64,

    /// Mapeo semántico por impacto en caliente (One-Shot Learning)
    pub tabla_impactos: HashMap<u32, ImpactoConceptual>,
}

impl MotorConceptos {
    pub fn nuevo() -> Self {
        Self {
            co_ocurrencias: HashMap::new(),
            conceptos: Vec::new(),
            umbral_coocurrencia: 10,
            ventana_contexto: 5,
            paso_actual: 0,
            cadencia_agrupacion: 500,
            conceptos_formados: 0,
            tabla_impactos: HashMap::new(),
        }
    }

    /// Registra el impacto de un concepto/palabra en base a la química y estrés de hardware
    pub fn registrar_impacto(&mut self, token_id: u32, quimica: f32, estres: f32, anterior: Option<u32>) {
        let nuevo_impacto = ImpactoConceptual {
            id_binario: token_id,
            quimica_simulada: quimica,
            estres_hardware: estres,
            anclaje_contextual: anterior,
        };

        self.tabla_impactos.entry(token_id)
            .and_modify(|imp| {
                // Sobrescribir solo si el nuevo estrés o impacto emocional es mayor (One-Shot Learning adaptativo)
                if estres > imp.estres_hardware || quimica.abs() > imp.quimica_simulada.abs() {
                    *imp = nuevo_impacto.clone();
                }
            })
            .or_insert(nuevo_impacto);
    }

    /// Registra co-ocurrencias entre tokens en una oración
    pub fn registrar_oracion(&mut self, tokens: &[u32]) {
        for i in 0..tokens.len() {
            let ventana_fin = (i + self.ventana_contexto).min(tokens.len());
            for j in (i + 1)..ventana_fin {
                let a = tokens[i].min(tokens[j]);
                let b = tokens[i].max(tokens[j]);
                *self.co_ocurrencias.entry((a, b)).or_insert(0) += 1;
            }
        }
    }

    /// Ejecuta clustering de co-ocurrencias para formar/actualizar conceptos
    /// Retorna los conceptos nuevos o modificados
    pub fn agrupar(&mut self) -> Vec<ProtoConcepto> {
        let mut nuevos_conceptos: Vec<ProtoConcepto> = Vec::new();
        let mut procesados: std::collections::HashSet<u32> = std::collections::HashSet::new();

        // Encontrar pares con co-ocurrencia sobre el umbral
        let mut pares_fuertes: Vec<(u32, u32, u32)> = self.co_ocurrencias.iter()
            .filter(|&(_, &count)| count >= self.umbral_coocurrencia)
            .map(|(&(a, b), &count)| (a, b, count))
            .collect();

        // Ordenar por fuerza de co-ocurrencia descendente
        pares_fuertes.sort_by(|a, b| b.2.cmp(&a.2));

        for &(token_a, token_b, count) in &pares_fuertes {
            // Buscar si alguno ya pertenece a un proto-concepto
            let idx_a = self.concepto_idx_de(token_a);
            let idx_b = self.concepto_idx_de(token_b);

            match (idx_a, idx_b) {
                (Some(ia), Some(ib)) if ia == ib => {
                    // Ya están en el mismo concepto, actualizar peso
                    self.conceptos[ia].peso = (self.conceptos[ia].peso + 0.1).min(1.0);
                }
                (Some(ia), Some(ib)) => {
                    // Están en distintos conceptos → fusionar
                    let miembros_b = self.conceptos[ib].miembros.clone();
                    let peso_b = self.conceptos[ib].peso;

                    // Mover miembros de B a A
                    for &m in &miembros_b {
                        if !self.conceptos[ia].miembros.contains(&m) {
                            self.conceptos[ia].miembros.push(m);
                        }
                    }
                    self.conceptos[ia].peso = (self.conceptos[ia].peso + peso_b) / 2.0;

                    // Eliminar concepto B (swap_remove para no reordenar todo)
                    self.conceptos.remove(ib);
                }
                (Some(ia), None) => {
                    // A está en concepto, agregar B
                    if !self.conceptos[ia].miembros.contains(&token_b) {
                        self.conceptos[ia].miembros.push(token_b);
                        self.conceptos[ia].peso = (self.conceptos[ia].peso + (count as f32 / 100.0)).min(1.0);
                    }
                }
                (None, Some(ib)) => {
                    // B está en concepto, agregar A
                    if !self.conceptos[ib].miembros.contains(&token_a) {
                        self.conceptos[ib].miembros.push(token_a);
                        self.conceptos[ib].peso = (self.conceptos[ib].peso + (count as f32 / 100.0)).min(1.0);
                    }
                }
                (None, None) => {
                    // Buscar si algún token ya está en nuevos_conceptos (pendientes de agregar)
                    let mut expandido = false;
                    if let Some(c) = nuevos_conceptos.iter_mut().find(|c| c.miembros.contains(&token_a)) {
                        if !c.miembros.contains(&token_b) {
                            c.miembros.push(token_b);
                            c.peso = (c.peso + (count as f32 / 100.0)).min(1.0);
                        }
                        expandido = true;
                    }
                    if !expandido {
                        if let Some(c) = nuevos_conceptos.iter_mut().find(|c| c.miembros.contains(&token_b)) {
                            if !c.miembros.contains(&token_a) {
                                c.miembros.push(token_a);
                                c.peso = (c.peso + (count as f32 / 100.0)).min(1.0);
                            }
                            expandido = true;
                        }
                    }
                    if !expandido && !procesados.contains(&token_a) && !procesados.contains(&token_b) {
                        let peso = (count as f32 / 100.0).min(1.0);
                        let concepto = ProtoConcepto {
                            miembros: vec![token_a, token_b],
                            neurona_hub: None,
                            peso,
                        };
                        nuevos_conceptos.push(concepto);
                        procesados.insert(token_a);
                        procesados.insert(token_b);
                    }
                }
            }
        }

        // Agregar los nuevos conceptos a la lista global
        for concepto in &nuevos_conceptos {
            self.conceptos.push(concepto.clone());
            self.conceptos_formados += 1;
        }

        nuevos_conceptos
    }

    /// Busca el índice del proto-concepto que contiene un token
    fn concepto_idx_de(&self, token_id: u32) -> Option<usize> {
        self.conceptos.iter().position(|c| c.miembros.contains(&token_id))
    }

    /// Busca el proto-concepto que contiene un token
    pub fn concepto_de(&self, token_id: u32) -> Option<&ProtoConcepto> {
        self.conceptos.iter().find(|c| c.miembros.contains(&token_id))
    }

    /// Obtiene todos los miembros de un concepto dado un token
    pub fn miembros_relacionados(&self, token_id: u32) -> Vec<u32> {
        self.conceptos.iter()
            .find(|c| c.miembros.contains(&token_id))
            .map(|c| c.miembros.clone())
            .unwrap_or_default()
    }

    /// Estadísticas
    pub fn total_conceptos(&self) -> usize {
        self.conceptos.len()
    }

    /// Limpia co-ocurrencias viejas para evitar crecimiento infinito
    pub fn limpiar_co_ocurrencias(&mut self) {
        // Mantener solo pares con conteo >= 2
        self.co_ocurrencias.retain(|_, &mut count| count >= 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coocurrencia_simple() {
        let mut m = MotorConceptos::nuevo();
        m.registrar_oracion(&[1, 2, 3]);
        // tokens 1-2, 1-3, 2-3 deben tener co-ocurrencia 1
        assert_eq!(*m.co_ocurrencias.get(&(1, 2)).unwrap_or(&0), 1);
        assert_eq!(*m.co_ocurrencias.get(&(1, 3)).unwrap_or(&0), 1);
        assert_eq!(*m.co_ocurrencias.get(&(2, 3)).unwrap_or(&0), 1);
    }

    #[test]
    fn test_coocurrencia_fuera_ventana() {
        let mut m = MotorConceptos::nuevo();
        m.ventana_contexto = 2;
        m.registrar_oracion(&[1, 2, 3, 4, 5]);
        // 1 y 5 están fuera de ventana (distancia 4 > 2)
        assert_eq!(*m.co_ocurrencias.get(&(1, 5)).unwrap_or(&0), 0);
        // 1 y 2 están en ventana
        assert_eq!(*m.co_ocurrencias.get(&(1, 2)).unwrap_or(&0), 1);
    }

    #[test]
    fn test_agrupar_sobre_umbral() {
        let mut m = MotorConceptos::nuevo();
        m.umbral_coocurrencia = 5;
        // Repetir la misma oración 6 veces
        for _ in 0..6 {
            m.registrar_oracion(&[10, 20, 30]);
        }
        let nuevos = m.agrupar();
        assert!(nuevos.len() >= 1, "Debe crear al menos un concepto, creó {}", nuevos.len());
    }

    #[test]
    fn test_agrupar_bajo_umbral() {
        let mut m = MotorConceptos::nuevo();
        m.umbral_coocurrencia = 10;
        // Solo 3 co-ocurrencias, bajo el umbral
        for _ in 0..3 {
            m.registrar_oracion(&[10, 20]);
        }
        let nuevos = m.agrupar();
        assert_eq!(nuevos.len(), 0, "No debe crear conceptos bajo el umbral");
    }

    #[test]
    fn test_fusion_conceptos() {
        let mut m = MotorConceptos::nuevo();
        m.umbral_coocurrencia = 5;
        // Crear concepto A: {10, 20}
        for _ in 0..6 {
            m.registrar_oracion(&[10, 20]);
        }
        m.agrupar();

        // Crear concepto B: {20, 30} — 20 está en A, debería fusionarse
        for _ in 0..6 {
            m.registrar_oracion(&[20, 30]);
        }
        let _nuevos = m.agrupar();

        // Después de fusión, debe haber un solo concepto con {10, 20, 30}
        assert_eq!(m.conceptos.len(), 1, "Debe haber 1 concepto después de fusión");
        let concepto = &m.conceptos[0];
        assert!(concepto.miembros.contains(&10), "Debe contener 10");
        assert!(concepto.miembros.contains(&20), "Debe contener 20");
        assert!(concepto.miembros.contains(&30), "Debe contener 30");
    }

    #[test]
    fn test_concepto_de() {
        let mut m = MotorConceptos::nuevo();
        m.umbral_coocurrencia = 5;
        for _ in 0..6 {
            m.registrar_oracion(&[42, 43]);
        }
        m.agrupar();
        let c = m.concepto_de(42);
        assert!(c.is_some(), "Debe encontrar concepto para token 42");
        assert!(c.unwrap().miembros.contains(&43), "Concepto debe contener token 43");
    }

    #[test]
    fn test_miembros_relacionados() {
        let mut m = MotorConceptos::nuevo();
        m.umbral_coocurrencia = 5;
        for _ in 0..6 {
            m.registrar_oracion(&[100, 200, 300]);
        }
        m.agrupar();
        let relacionados = m.miembros_relacionados(100);
        assert!(relacionados.contains(&200), "Debe relacionar 100 con 200");
        assert!(relacionados.contains(&300), "Debe relacionar 100 con 300");
    }

    #[test]
    fn test_total_conceptos() {
        let mut m = MotorConceptos::nuevo();
        assert_eq!(m.total_conceptos(), 0);
        m.umbral_coocurrencia = 5;
        for _ in 0..6 {
            m.registrar_oracion(&[1, 2]);
        }
        m.agrupar();
        assert_eq!(m.total_conceptos(), 1);
    }

    #[test]
    fn test_one_shot_learning_impacto() {
        let mut m = MotorConceptos::nuevo();
        
        // Registrar impacto inicial neutro
        m.registrar_impacto(42, 0.1, 0.1, Some(10));
        let imp1 = m.tabla_impactos.get(&42).unwrap().clone();
        assert_eq!(imp1.quimica_simulada, 0.1);
        
        // Registrar un evento extremo (One-Shot update)
        m.registrar_impacto(42, -0.8, 0.95, Some(20));
        let imp2 = m.tabla_impactos.get(&42).unwrap().clone();
        assert_eq!(imp2.quimica_simulada, -0.8);
        assert_eq!(imp2.estres_hardware, 0.95);
        assert_eq!(imp2.anclaje_contextual, Some(20));
        
        // Un evento posterior más débil no debe sobreescribir la memoria extrema del One-Shot
        m.registrar_impacto(42, 0.2, 0.15, Some(30));
        let imp3 = m.tabla_impactos.get(&42).unwrap().clone();
        assert_eq!(imp3.estres_hardware, 0.95, "Memoria extrema debe persistir");
    }
}
