// ============================================================================
// 🧠 ASOCIACIÓN LIBRE — Transiciones Hebbianas entre Asambleas
// ============================================================================
// Permite el pensamiento en cadena espontáneo.
// Principio: "Neurons that fire together, wire together" (Hebb, 1949).
// Extendido a asambleas: asambleas co-activas -> transiciones direccionales.
// ============================================================================

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// ========================================
// TRANSICIÓN HEBB (el "cable" entre ideas)
// ========================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransicionHebb {
    pub desde: u32,
    pub hacia: u32,
    pub peso: f64,
    pub coactivaciones: u32,
    pub ultima_coactivacion: u64,
    pub fatiga: f64,
    pub creada_en: u64,
}

impl TransicionHebb {
    pub fn nueva(desde: u32, hacia: u32, timestamp: u64) -> Self {
        TransicionHebb {
            desde,
            hacia,
            peso: 0.01,
            coactivaciones: 1,
            ultima_coactivacion: timestamp,
            fatiga: 0.0,
            creada_en: timestamp,
        }
    }

    pub fn reforzar(&mut self, timestamp: u64) {
        self.coactivaciones += 1;
        self.ultima_coactivacion = timestamp;
        self.peso = (0.01 + 0.15 * (self.coactivaciones as f64).ln()).min(1.0);
        self.fatiga = (self.fatiga - 0.2).max(0.0);
    }

    pub fn debilitar(&mut self, factor_decaimiento: f64) {
        self.peso = (self.peso - factor_decaimiento).max(0.0);
    }

    pub fn fatigar(&mut self, cantidad: f64) {
        self.fatiga = (self.fatiga + cantidad).min(1.0);
    }

    pub fn peso_efectivo(&self) -> f64 {
        self.peso * (1.0 - self.fatiga)
    }
}

// ========================================
// GRAFO DE ASOCIACIONES
// ========================================

#[derive(Clone, Serialize, Deserialize)]
pub struct GrafoAsociativo {
    pub transiciones: HashMap<(u32, u32), TransicionHebb>,
    pub salientes: HashMap<u32, Vec<(u32, f64)>>,
    pub umbral_propagacion: f64,
    pub decaimiento: f64,
    pub max_saltos_por_ciclo: usize,
    pub total_transiciones: usize,
    pub transiciones_podadas: usize,
}

impl GrafoAsociativo {
    pub fn nuevo() -> Self {
        GrafoAsociativo {
            transiciones: HashMap::new(),
            salientes: HashMap::new(),
            umbral_propagacion: 0.05,
            decaimiento: 0.001,
            max_saltos_por_ciclo: 5,
            total_transiciones: 0,
            transiciones_podadas: 0,
        }
    }

    pub fn registrar_coactivacion(&mut self, desde: u32, hacia: u32, timestamp: u64) {
        if desde == hacia { return; }
        let clave = (desde, hacia);
        if let Some(transicion) = self.transiciones.get_mut(&clave) {
            transicion.reforzar(timestamp);
        } else {
            let transicion = TransicionHebb::nueva(desde, hacia, timestamp);
            self.transiciones.insert(clave, transicion);
            self.total_transiciones += 1;
        }
        self.reconstruir_indice_saliente(desde);
    }

    pub fn propagar_desde(&mut self, desde_id: u32, corriente: f64, _timestamp: u64) -> Vec<(u32, f64)> {
        let mut activaciones = Vec::new();
        if let Some(salidas) = self.salientes.get(&desde_id) {
            for (hacia_id, _) in salidas.iter() {
                let clave = (desde_id, *hacia_id);
                if let Some(transicion) = self.transiciones.get_mut(&clave) {
                    let peso_efectivo = transicion.peso_efectivo();
                    if peso_efectivo > self.umbral_propagacion {
                        let corriente_propagada = corriente * peso_efectivo;
                        activaciones.push((*hacia_id, corriente_propagada));
                        transicion.fatigar(0.3);
                    }
                }
            }
        }
        activaciones.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        activaciones.truncate(self.max_saltos_por_ciclo);
        activaciones
    }

    pub fn cadena_asociativa(&mut self, desde_id: u32, corriente_inicial: f64, timestamp: u64, profundidad_max: usize) -> Vec<Vec<(u32, f64)>> {
        let mut cadena = Vec::new();
        let mut corriente = corriente_inicial;
        let mut actual = desde_id;
        for _ in 0..profundidad_max {
            let activaciones = self.propagar_desde(actual, corriente, timestamp);
            if activaciones.is_empty() { break; }
            cadena.push(activaciones.clone());
            if let Some((siguiente_id, siguiente_corriente)) = activaciones.first() {
                actual = *siguiente_id;
                corriente = *siguiente_corriente * 0.8;
            } else { break; }
        }
        cadena
    }

    pub fn podar_debiles(&mut self) -> usize {
        let a_eliminar: Vec<(u32, u32)> = self.transiciones.iter()
            .filter(|(_, t)| t.peso < 0.001)
            .map(|(k, _)| *k)
            .collect();
        let count = a_eliminar.len();
        for clave in a_eliminar {
            self.transiciones.remove(&clave);
            self.transiciones_podadas += 1;
        }
        if count > 0 { self.reconstruir_todos_indices(); }
        count
    }

    pub fn decaimiento_global(&mut self) {
        for transicion in self.transiciones.values_mut() {
            transicion.debilitar(self.decaimiento);
            transicion.fatiga = (transicion.fatiga - 0.05).max(0.0);
        }
    }

    fn reconstruir_indice_saliente(&mut self, desde_id: u32) {
        let mut salidas: Vec<(u32, f64)> = self.transiciones.iter()
            .filter(|((d, _), _)| *d == desde_id)
            .map(|((_, h), t)| (*h, t.peso))
            .collect();
        salidas.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        self.salientes.insert(desde_id, salidas);
    }

    fn reconstruir_todos_indices(&mut self) {
        self.salientes.clear();
        let claves: Vec<u32> = self.transiciones.keys().map(|(d, _)| *d).collect();
        for desde_id in claves { self.reconstruir_indice_saliente(desde_id); }
    }
}

// ========================================
// GESTOR DE ASOCIACIÓN LIBRE
// ========================================

#[derive(Clone, Serialize, Deserialize)]
pub struct GestorAsociacionLibre {
    pub grafo: GrafoAsociativo,
    pub historial_cadenas: Vec<CadenaAsociativa>,
    pub activo: bool,
    pub frecuencia: u64,
    pub profundidad_max: usize,
    pub total_cadenas: usize,
    pub total_propagaciones: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CadenaAsociativa {
    pub timestamp: u64,
    pub secuencia_ids: Vec<u32>,
    pub corriente_inicial: f64,
    pub profundidad: usize,
    pub tokens_generados: Vec<String>,
}

impl GestorAsociacionLibre {
    pub fn nuevo() -> Self {
        GestorAsociacionLibre {
            grafo: GrafoAsociativo::nuevo(),
            historial_cadenas: Vec::new(),
            activo: true,
            frecuencia: 5,
            profundidad_max: 5,
            total_cadenas: 0,
            total_propagaciones: 0,
        }
    }

    pub fn paso_asociativo(&mut self, asamblea_id: u32, corriente: f64, timestamp: u64) -> CadenaAsociativa {
        let cadena_raw = self.grafo.cadena_asociativa(asamblea_id, corriente, timestamp, self.profundidad_max);
        let mut secuencia_ids = vec![asamblea_id];
        for eslabon in &cadena_raw {
            if let Some((id, _)) = eslabon.first() { secuencia_ids.push(*id); }
        }
        let cadena = CadenaAsociativa {
            timestamp,
            secuencia_ids,
            corriente_inicial: corriente,
            profundidad: cadena_raw.len(),
            tokens_generados: Vec::new(),
        };
        self.historial_cadenas.push(cadena.clone());
        self.total_cadenas += 1;
        self.total_propagaciones += cadena_raw.len();
        if self.historial_cadenas.len() > 100 { self.historial_cadenas.remove(0); }
        cadena
    }

    pub fn aprender_de_coactivacion(&mut self, activas: &[u32], timestamp: u64) {
        for &i in activas {
            for &j in activas {
                if i != j { self.grafo.registrar_coactivacion(i, j, timestamp); }
            }
        }
    }

    pub fn mantenimiento(&mut self) {
        self.grafo.decaimiento_global();
        self.grafo.podar_debiles();
    }
}

impl Default for GestorAsociacionLibre {
    fn default() -> Self { GestorAsociacionLibre::nuevo() }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-4, "esperaba {a}, obtuve {b}");
    }

    #[test]
    fn test_transicion_nueva_valores_iniciales() {
        let t = TransicionHebb::nueva(1, 2, 100);
        assert_eq!(t.desde, 1);
        assert_eq!(t.hacia, 2);
        casi(t.peso, 0.01);
        assert_eq!(t.coactivaciones, 1);
        casi(t.fatiga, 0.0);
        assert_eq!(t.ultima_coactivacion, 100);
        assert_eq!(t.creada_en, 100);
    }

    #[test]
    fn test_reforzar_crece_peso_logaritmicamente() {
        let mut t = TransicionHebb::nueva(1, 2, 0);
        t.reforzar(1); // coactivaciones = 2
        t.reforzar(2); // coactivaciones = 3
        // peso = 0.01 + 0.15*ln(3)
        let esperado = 0.01 + 0.15 * (3.0f64).ln();
        casi(t.peso, esperado);
        assert_eq!(t.coactivaciones, 3);
        assert_eq!(t.ultima_coactivacion, 2);
    }

    #[test]
    fn test_reforzar_reduce_fatiga() {
        let mut t = TransicionHebb::nueva(1, 2, 0);
        t.fatigar(0.5);
        casi(t.fatiga, 0.5);
        t.reforzar(1);
        casi(t.fatiga, 0.3); // 0.5 - 0.2
    }

    #[test]
    fn test_peso_efectivo_considera_fatiga() {
        let mut t = TransicionHebb::nueva(1, 2, 0);
        t.fatigar(0.5);
        // peso=0.01, fatiga=0.5 -> 0.01 * (1-0.5) = 0.005
        casi(t.peso_efectivo(), 0.005);
    }

    #[test]
    fn test_debilitar_baja_peso_sin_negativo() {
        let mut t = TransicionHebb::nueva(1, 2, 0);
        t.debilitar(0.005);
        casi(t.peso, 0.005);
        t.debilitar(1.0);
        casi(t.peso, 0.0);
    }

    #[test]
    fn test_fatigar_clampa_a_uno() {
        let mut t = TransicionHebb::nueva(1, 2, 0);
        t.fatigar(2.0);
        casi(t.fatiga, 1.0);
    }

    #[test]
    fn test_grafo_nuevo_config_por_defecto() {
        let g = GrafoAsociativo::nuevo();
        assert_eq!(g.umbral_propagacion, 0.05);
        assert_eq!(g.decaimiento, 0.001);
        assert_eq!(g.max_saltos_por_ciclo, 5);
        assert_eq!(g.total_transiciones, 0);
        assert!(g.transiciones.is_empty());
    }

    #[test]
    fn test_registrar_coactivacion_ignora_mismo_nodo() {
        let mut g = GrafoAsociativo::nuevo();
        g.registrar_coactivacion(1, 1, 0);
        assert_eq!(g.total_transiciones, 0);
    }

    #[test]
    fn test_registrar_coactivacion_crea_y_refuerza() {
        let mut g = GrafoAsociativo::nuevo();
        g.registrar_coactivacion(1, 2, 0);
        g.registrar_coactivacion(1, 2, 1);
        assert_eq!(g.total_transiciones, 1);
        let t = g.transiciones.get(&(1, 2)).unwrap();
        assert_eq!(t.coactivaciones, 2);
        // índice saliente reconstruido
        let salidas = g.salientes.get(&1).unwrap();
        assert_eq!(salidas.len(), 1);
        assert_eq!(salidas[0].0, 2);
    }

    #[test]
    fn test_propagar_desde_sobre_umbral() {
        let mut g = GrafoAsociativo::nuevo();
        // 10 coactivaciones => peso = 0.01 + 0.15*ln(10) ≈ 0.355 > 0.05
        for i in 0..10 {
            g.registrar_coactivacion(1, 2, i);
        }
        // Capturar peso_efectivo ANTES de propagar (propagar aplica fatiga 0.3)
        let peso_pre = g.transiciones.get(&(1, 2)).unwrap().peso_efectivo();
        let act = g.propagar_desde(1, 1.0, 100);
        assert_eq!(act.len(), 1);
        let (id, corriente) = act[0];
        assert_eq!(id, 2);
        casi(corriente, peso_pre);
    }

    #[test]
    fn test_propagar_desde_bajo_umbral_vacio() {
        let mut g = GrafoAsociativo::nuevo();
        // Solo 1 coactivación => peso 0.01 < 0.05
        g.registrar_coactivacion(1, 2, 0);
        let act = g.propagar_desde(1, 1.0, 100);
        assert!(act.is_empty());
    }

    #[test]
    fn test_propagacion_fatiga_al_usar() {
        let mut g = GrafoAsociativo::nuevo();
        for i in 0..10 {
            g.registrar_coactivacion(1, 2, i);
        }
        let peso_inicial = g.transiciones.get(&(1, 2)).unwrap().peso_efectivo();
        g.propagar_desde(1, 1.0, 100);
        let peso_despues = g.transiciones.get(&(1, 2)).unwrap().peso_efectivo();
        assert!(peso_despues < peso_inicial, "fatiga debe reducir peso efectivo");
    }

    #[test]
    fn test_cadena_asociativa_encadena_nodos() {
        let mut g = GrafoAsociativo::nuevo();
        // 1 -> 2 -> 3
        for i in 0..10 {
            g.registrar_coactivacion(1, 2, i);
            g.registrar_coactivacion(2, 3, i);
        }
        let cadena = g.cadena_asociativa(1, 1.0, 100, 5);
        assert!(!cadena.is_empty(), "debe generar al menos un eslabón");
        assert_eq!(cadena[0][0].0, 2);
    }

    #[test]
    fn test_cadena_asociativa_sin_salidas_vacia() {
        let mut g = GrafoAsociativo::nuevo();
        let cadena = g.cadena_asociativa(99, 1.0, 100, 5);
        assert!(cadena.is_empty());
    }

    #[test]
    fn test_podar_debiles_elimina_bajo_peso() {
        let mut g = GrafoAsociativo::nuevo();
        // Peso alto (10 coactivaciones) => sobrevive
        for i in 0..10 {
            g.registrar_coactivacion(2, 3, i);
        }
        // Peso debilitado hasta bajo 0.001 => se poda
        g.registrar_coactivacion(1, 2, 0);
        if let Some(t) = g.transiciones.get_mut(&(1, 2)) {
            t.debilitar(0.02); // 0.01 - 0.02 -> clamp a 0.0 < 0.001
        }
        let podados = g.podar_debiles();
        assert_eq!(podados, 1);
        assert_eq!(g.transiciones_podadas, 1);
        assert!(g.transiciones.contains_key(&(2, 3)));
        assert!(!g.transiciones.contains_key(&(1, 2)));
    }

    #[test]
    fn test_decaimiento_global_debilita_y_recupera_fatiga() {
        let mut g = GrafoAsociativo::nuevo();
        g.registrar_coactivacion(1, 2, 0);
        let peso_inicial = g.transiciones.get(&(1, 2)).unwrap().peso;
        g.decaimiento_global();
        let t = g.transiciones.get(&(1, 2)).unwrap();
        casi(t.peso, peso_inicial - 0.001);
        casi(t.fatiga, 0.0); // no tenía fatiga, recupera sin bajar de 0
    }

    #[test]
    fn test_gestor_nuevo_valores() {
        let gestor = GestorAsociacionLibre::nuevo();
        assert!(gestor.activo);
        assert_eq!(gestor.frecuencia, 5);
        assert_eq!(gestor.profundidad_max, 5);
        assert_eq!(gestor.total_cadenas, 0);
        assert!(gestor.historial_cadenas.is_empty());
    }

    #[test]
    fn test_paso_asociativo_registra_cadena() {
        let mut gestor = GestorAsociacionLibre::nuevo();
        for i in 0..10 {
            gestor.grafo.registrar_coactivacion(1, 2, i);
        }
        let cadena = gestor.paso_asociativo(1, 1.0, 100);
        assert_eq!(cadena.timestamp, 100);
        assert!(cadena.secuencia_ids.len() >= 2);
        assert_eq!(cadena.secuencia_ids[0], 1);
        assert_eq!(gestor.total_cadenas, 1);
        assert_eq!(gestor.historial_cadenas.len(), 1);
    }

    #[test]
    fn test_aprender_de_coactivacion_todas_las_parejas() {
        let mut gestor = GestorAsociacionLibre::nuevo();
        gestor.aprender_de_coactivacion(&[1, 2, 3], 0);
        // Parejas direccionales: 3*2 = 6 transiciones
        assert_eq!(gestor.grafo.total_transiciones, 6);
        assert!(gestor.grafo.transiciones.contains_key(&(1, 2)));
        assert!(gestor.grafo.transiciones.contains_key(&(3, 1)));
        assert!(!gestor.grafo.transiciones.contains_key(&(1, 1)));
    }

    #[test]
    fn test_mantenimiento_ejecuta_decaimiento_y_poda() {
        let mut gestor = GestorAsociacionLibre::nuevo();
        // Transición fuerte
        for i in 0..10 {
            gestor.grafo.registrar_coactivacion(2, 3, i);
        }
        // Transición debilitada que se podará tras mantenimiento
        gestor.grafo.registrar_coactivacion(1, 2, 0);
        if let Some(t) = gestor.grafo.transiciones.get_mut(&(1, 2)) {
            t.debilitar(0.02); // 0.01 - 0.02 -> 0.0 < 0.001
        }
        gestor.mantenimiento();
        assert!(gestor.grafo.transiciones.contains_key(&(2, 3)));
        assert!(!gestor.grafo.transiciones.contains_key(&(1, 2)));
    }
}
