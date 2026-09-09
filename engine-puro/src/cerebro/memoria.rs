// ============================================================================
// MEMORIA SELECTIVA JERÁRQUICA (VRAM / RAM / SSD)
// ============================================================================
// Sistema de memoria de 3 niveles con migración automática según frecuencia de uso.
// - VRAM: Neuronas activas (acceso rápido, capacidad limitada)
// - RAM: Neuronas latentes (acceso medio, capacidad media)
// - SSD: Memoria episódica (acceso lento, capacidad masiva)
// ============================================================================

use crate::cerebro::estructuras::*;
use crate::cerebro::hardware::ConfiguracionDinamica;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// UBICACIONES DE MEMORIA
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UbicacionMemoria {
    VRAM,
    RAM,
    SSD,
    Swap,
}

// ============================================================================
// MANAGER DE VRAM (Nivel 1 — Neuronas activas)
// ============================================================================

pub struct VramManager {
    pub neuronas: Vec<NeuronaCompacta>,
    pub sinapsis: HashMap<u32, Vec<SinapsisCompacta>>,
    pub capacidad_neuronas: usize,
    pub capacidad_sinapsis: usize,
}

impl VramManager {
    pub fn nuevo(cap_neuronas: usize, cap_sinapsis: usize) -> Self {
        let cap = cap_neuronas.max(1000);
        Self {
            neuronas: Vec::with_capacity(cap.min(100_000)),
            sinapsis: HashMap::with_capacity((cap_neuronas / 10).min(10_000)),
            capacidad_neuronas: cap,
            capacidad_sinapsis: cap_sinapsis,
        }
    }

    pub fn agregar_neurona(&mut self, neurona: NeuronaCompacta) -> bool {
        if self.neuronas.len() < self.capacidad_neuronas {
            let _pos = self.neuronas.len();
            self.neuronas.push(neurona);
            self.sinapsis.entry(neurona.id).or_insert_with(Vec::new);
            true
        } else {
            // Reemplazar la menos activa
            if let Some(pos) = self
                .neuronas
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.activacion
                        .partial_cmp(&b.1.activacion)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
            {
                let vieja = self.neuronas[pos];
                self.sinapsis.remove(&vieja.id);
                self.neuronas[pos] = neurona;
                self.sinapsis.entry(neurona.id).or_insert_with(Vec::new);
            }
            false
        }
    }

    pub fn eliminar_neurona(&mut self, id: u32) -> Option<NeuronaCompacta> {
        if let Some(pos) = self.neuronas.iter().position(|n| n.id == id) {
            let neurona = self.neuronas.remove(pos);
            self.sinapsis.remove(&id);
            Some(neurona)
        } else {
            None
        }
    }

    pub fn obtener_neurona(&self, id: u32) -> Option<&NeuronaCompacta> {
        self.neuronas.iter().find(|n| n.id == id)
    }

    pub fn obtener_neurona_mut(&mut self, id: u32) -> Option<&mut NeuronaCompacta> {
        self.neuronas.iter_mut().find(|n| n.id == id)
    }

    pub fn obtener_sinapsis(&self, id: u32) -> Option<&[SinapsisCompacta]> {
        self.sinapsis.get(&id).map(|v| v.as_slice())
    }

    pub fn agregar_sinapsis(&mut self, origen: u32, sinapsis: SinapsisCompacta) {
        self.sinapsis.entry(origen).or_insert_with(Vec::new).push(sinapsis);
    }

    pub fn total_neuronas(&self) -> usize {
        self.neuronas.len()
    }

    pub fn esta_lleno(&self) -> bool {
        self.neuronas.len() >= self.capacidad_neuronas
    }

    /// Encuentra la neurona candidata a ser desalojada (menos activa)
    pub fn candidato_desalojo(&self) -> Option<u32> {
        self.neuronas
            .iter()
            .min_by(|a, b| {
                a.activacion
                    .partial_cmp(&b.activacion)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|n| n.id)
    }
}

// ============================================================================
// MANAGER DE RAM (Nivel 2 — Neuronas latentes)
// ============================================================================

pub struct RamManager {
    pub neuronas: Vec<NeuronaCompacta>,
    pub sinapsis: HashMap<u32, Vec<SinapsisCompacta>>,
    pub capacidad_neuronas: usize,
    pub capacidad_sinapsis: usize,
}

impl RamManager {
    pub fn nuevo(cap_neuronas: usize, cap_sinapsis: usize) -> Self {
        let cap = cap_neuronas.max(10000);
        Self {
            neuronas: Vec::with_capacity(cap.min(100_000)),
            sinapsis: HashMap::with_capacity((cap / 10).min(10_000)),
            capacidad_neuronas: cap,
            capacidad_sinapsis: cap_sinapsis,
        }
    }

    pub fn agregar_neurona(&mut self, neurona: NeuronaCompacta) -> bool {
        if self.neuronas.len() < self.capacidad_neuronas {
            self.neuronas.push(neurona);
            true
        } else {
            // Sobrescribir la menos activa
            if let Some(pos) = self
                .neuronas
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.activacion
                        .partial_cmp(&b.1.activacion)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
            {
                let vieja = self.neuronas[pos];
                self.sinapsis.remove(&vieja.id);
                self.neuronas[pos] = neurona;
            }
            true
        }
    }

    pub fn eliminar_neurona(&mut self, id: u32) -> Option<NeuronaCompacta> {
        if let Some(pos) = self.neuronas.iter().position(|n| n.id == id) {
            let neurona = self.neuronas.remove(pos);
            self.sinapsis.remove(&id);
            Some(neurona)
        } else {
            None
        }
    }

    pub fn obtener_neurona(&self, id: u32) -> Option<&NeuronaCompacta> {
        self.neuronas.iter().find(|n| n.id == id)
    }

    pub fn obtener_neurona_mut(&mut self, id: u32) -> Option<&mut NeuronaCompacta> {
        self.neuronas.iter_mut().find(|n| n.id == id)
    }

    pub fn obtener_todas(&self) -> &[NeuronaCompacta] {
        &self.neuronas
    }

    pub fn obtener_todas_mut(&mut self) -> &mut [NeuronaCompacta] {
        &mut self.neuronas
    }

    pub fn total_neuronas(&self) -> usize {
        self.neuronas.len()
    }
}

// ============================================================================
// MANAGER DE SSD (Nivel 3 — Memoria Episódica)
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct SsdManager {
    pub episodios: Vec<Episodio>,
    pub capacidad_maxima: usize,
}

impl SsdManager {
    pub fn nuevo(capacidad: usize) -> Self {
        Self {
            episodios: Vec::with_capacity(capacidad.min(100_000)),
            capacidad_maxima: capacidad,
        }
    }

    pub fn almacenar(&mut self, episodio: Episodio) {
        self.episodios.push(episodio);
        if self.episodios.len() > self.capacidad_maxima {
            self.episodios.remove(0); // FIFO
        }
    }

    pub fn recuperar(&self, patron: &[u32]) -> Vec<Episodio> {
        let mut resultados: Vec<Episodio> = self
            .episodios
            .iter()
            .filter(|e| {
                e.similitud_patron(patron) > 0.5
            })
            .cloned()
            .collect();
        resultados.truncate(10);
        resultados
    }

    pub fn total_episodios(&self) -> usize {
        self.episodios.len()
    }
}

// ============================================================================
// SISTEMA DE MEMORIA ADAPTATIVA (Orquestador de los 3 niveles)
// ============================================================================

pub struct MemoriaAdaptativa {
    pub vram: Option<VramManager>,
    pub ram: RamManager,
    pub ssd: SsdManager,
    pub mapa_memoria: HashMap<u32, UbicacionMemoria>,
    pub acceso_frecuencia: HashMap<u32, u64>,
}

impl MemoriaAdaptativa {
    pub fn nuevo(config: &ConfiguracionDinamica) -> Self {
        let vram = if config.usar_gpu && config.max_neuronas_vram > 0 {
            Some(VramManager::nuevo(
                config.max_neuronas_vram,
                config.max_sinapsis_vram,
            ))
        } else {
            None
        };

        Self {
            vram,
            ram: RamManager::nuevo(config.max_neuronas_ram, config.max_sinapsis_ram),
            ssd: SsdManager::nuevo(config.memoria_episodica_max),
            mapa_memoria: HashMap::new(),
            acceso_frecuencia: HashMap::new(),
        }
    }

    /// Mueve una neurona de RAM a VRAM
    pub fn mover_a_vram(&mut self, id: u32) -> bool {
        if let Some(vram) = &mut self.vram {
            if let Some(neurona) = self.ram.eliminar_neurona(id) {
                if vram.agregar_neurona(neurona) {
                    self.mapa_memoria.insert(id, UbicacionMemoria::VRAM);
                    *self.acceso_frecuencia.entry(id).or_insert(0) += 1;
                    return true;
                } else {
                    // VRAM llena, desalojar
                    self.desalojar_de_vram();
                    return self.mover_a_vram(id);
                }
            }
        }
        false
    }

    /// Mueve una neurona de VRAM a RAM
    pub fn mover_a_ram(&mut self, id: u32) -> bool {
        if let Some(vram) = &mut self.vram {
            if let Some(neurona) = vram.eliminar_neurona(id) {
                if self.ram.agregar_neurona(neurona) {
                    self.mapa_memoria.insert(id, UbicacionMemoria::RAM);
                    return true;
                }
            }
        }
        false
    }

    /// Desaloja la neurona menos usada de VRAM a RAM
    pub fn desalojar_de_vram(&mut self) {
        if let Some(vram) = &mut self.vram {
            if let Some(id) = vram.candidato_desalojo() {
                self.mover_a_ram(id);
            }
        }
    }

    /// Obtiene una neurona desde cualquier nivel (VRAM primero, luego RAM)
    pub fn obtener_neurona(&self, id: u32) -> Option<&NeuronaCompacta> {
        if let Some(vram) = &self.vram {
            if let Some(n) = vram.obtener_neurona(id) {
                return Some(n);
            }
        }
        self.ram.obtener_neurona(id)
    }

    /// Obtiene una neurona mutable (solo RAM por simplicidad)
    pub fn obtener_neurona_mut(&mut self, id: u32) -> Option<&mut NeuronaCompacta> {
        // Verificar si está en VRAM y traerla
        if let Some(vram) = &self.vram {
            if vram.obtener_neurona(id).is_some() {
                // Mover a RAM para modificar
                self.mover_a_ram(id);
            }
        }
        self.ram.obtener_neurona_mut(id)
    }

    /// ¿Está la neurona en VRAM?
    pub fn esta_en_vram(&self, id: u32) -> bool {
        self.mapa_memoria
            .get(&id)
            .map_or(false, |&loc| loc == UbicacionMemoria::VRAM)
    }

    /// Optimiza la distribución: mueve las más usadas a VRAM
    pub fn optimizar(&mut self) {
        if self.vram.is_none() {
            return;
        }

        let mut accesos: Vec<(u32, u64)> = self
            .acceso_frecuencia
            .iter()
            .map(|(&id, &freq)| (id, freq))
            .collect();
        accesos.sort_by(|a, b| b.1.cmp(&a.1));

        for &(id, _) in accesos.iter().take(1000) {
            if !self.esta_en_vram(id) {
                self.mover_a_vram(id);
            }
        }
    }

    /// Registra un acceso para estadísticas
    pub fn registrar_acceso(&mut self, id: u32) {
        *self.acceso_frecuencia.entry(id).or_insert(0) += 1;
    }

    /// Estadísticas de memoria
    pub fn estadisticas(&self) -> (usize, usize, usize, usize) {
        let vram_n = self.vram.as_ref().map(|v| v.total_neuronas()).unwrap_or(0);
        let ram_n = self.ram.total_neuronas();
        let ssd_e = self.ssd.total_episodios();
        (vram_n, ram_n, vram_n + ram_n, ssd_e)
    }
}

// ============================================================================
// TESTS DE LA MEMORIA SELECTIVA JERÁRQUICA
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::hardware::Precision;

    fn casi(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    fn config_con_gpu() -> ConfiguracionDinamica {
        ConfiguracionDinamica {
            max_neuronas_vram: 1000,
            max_neuronas_ram: 10000,
            max_sinapsis_vram: 10000,
            max_sinapsis_ram: 100000,
            max_neuronas_totales: 11000,
            batch_size_gpu: 1024,
            batch_size_cpu: 1024,
            hilos_cpu: 8,
            usar_gpu: true,
            precision: Precision::F32,
            memoria_episodica_max: 1000,
        }
    }

    fn config_sin_gpu() -> ConfiguracionDinamica {
        let mut c = config_con_gpu();
        c.usar_gpu = false;
        c.max_neuronas_vram = 0;
        c
    }

    fn neurona(id: u32, activacion: f32) -> NeuronaCompacta {
        let mut n = NeuronaCompacta::reposo(id, 0, 0);
        n.activacion = activacion;
        n
    }

    // ── VRAM Manager ─────────────────────────────────────────────────────────
    #[test]
    fn test_vram_agrega_y_obtiene() {
        let mut v = VramManager::nuevo(3, 100);
        assert!(v.agregar_neurona(neurona(1, 0.5)));
        assert_eq!(v.total_neuronas(), 1);
        assert_eq!(v.obtener_neurona(1).unwrap().id, 1);
        assert!(!v.esta_lleno());
        assert_eq!(v.candidato_desalojo(), Some(1));
    }

    #[test]
    fn test_vram_reemplaza_menos_activa_al_llenar() {
        // `nuevo` fuerza mínimo 1000; fijamos capacidad directa (campo pub)
        let mut v = VramManager::nuevo(1000, 100);
        v.capacidad_neuronas = 2;
        v.agregar_neurona(neurona(1, 0.1)); // menos activa
        v.agregar_neurona(neurona(2, 0.9));

        // Lleno; nueva neurona 3 reemplaza la menos activa (id 1)
        assert!(!v.agregar_neurona(neurona(3, 0.5)));
        assert_eq!(v.total_neuronas(), 2);
        assert!(v.obtener_neurona(3).is_some());
        assert!(v.obtener_neurona(1).is_none());
        assert!(v.obtener_neurona(2).is_some());
    }

    #[test]
    fn test_vram_eliminar_remueve_sinapsis() {
        let mut v = VramManager::nuevo(3, 100);
        v.agregar_neurona(neurona(1, 0.5));
        v.agregar_sinapsis(1, SinapsisCompacta::nueva(2, 0.5));

        let eliminada = v.eliminar_neurona(1);
        assert!(eliminada.is_some());
        assert_eq!(eliminada.unwrap().id, 1);
        assert!(v.obtener_sinapsis(1).is_none());
        assert_eq!(v.total_neuronas(), 0);
    }

    #[test]
    fn test_vram_eliminar_inexistente() {
        let mut v = VramManager::nuevo(3, 100);
        assert!(v.eliminar_neurona(99).is_none());
    }

    #[test]
    fn test_vram_candidato_desalojo_minima_activa() {
        let mut v = VramManager::nuevo(3, 100);
        v.agregar_neurona(neurona(1, 0.8));
        v.agregar_neurona(neurona(2, 0.2));
        v.agregar_neurona(neurona(3, 0.5));
        assert_eq!(v.candidato_desalojo(), Some(2));
    }

    // ── RAM Manager ──────────────────────────────────────────────────────────
    #[test]
    fn test_ram_agrega_y_obtiene_todas() {
        let mut r = RamManager::nuevo(2, 100);
        r.agregar_neurona(neurona(1, 0.5));
        r.agregar_neurona(neurona(2, 0.6));
        assert_eq!(r.total_neuronas(), 2);
        assert_eq!(r.obtener_todas().len(), 2);
        assert_eq!(r.obtener_neurona(2).unwrap().id, 2);
    }

    #[test]
    fn test_ram_reemplaza_menos_activa_siempre_retorna_true() {
        // `nuevo` fuerza mínimo 10000; fijamos capacidad directa (campo pub)
        let mut r = RamManager::nuevo(10000, 100);
        r.capacidad_neuronas = 1;
        r.agregar_neurona(neurona(1, 0.3));
        // Lleno, pero agrega reemplazando
        assert!(r.agregar_neurona(neurona(2, 0.9)));
        assert_eq!(r.total_neuronas(), 1);
        assert!(r.obtener_neurona(2).is_some());
    }

    #[test]
    fn test_ram_eliminar_y_obtener_mut() {
        let mut r = RamManager::nuevo(3, 100);
        r.agregar_neurona(neurona(1, 0.5));
        r.obtener_neurona_mut(1).unwrap().energia = 0.9;
        assert!(casi(r.obtener_neurona(1).unwrap().energia, 0.9));

        let eliminada = r.eliminar_neurona(1);
        assert!(eliminada.is_some());
        assert!(r.obtener_neurona(1).is_none());
    }

    // ── SSD Manager ──────────────────────────────────────────────────────────
    fn episodio(patron: &[u32]) -> Episodio {
        Episodio::nueva(0.0, 1.0, 0.5, patron, 0)
    }

    const P8: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

    #[test]
    fn test_ssd_almacena_y_recupera() {
        let mut s = SsdManager::nuevo(10);
        s.almacenar(episodio(&P8));
        assert_eq!(s.total_episodios(), 1);
        // Similitud 8/8 = 1.0 > 0.5
        let rec = s.recuperar(&P8);
        assert_eq!(rec.len(), 1);
    }

    #[test]
    fn test_ssd_fifo_descarta_el_mas_antiguo() {
        let mut s = SsdManager::nuevo(2);
        s.almacenar(episodio(&P8));
        s.almacenar(episodio(&[11, 12, 13, 14, 15, 16, 17, 18]));
        s.almacenar(episodio(&[21, 22, 23, 24, 25, 26, 27, 28]));

        assert_eq!(s.total_episodios(), 2);
        // El primero (P8) fue descartado por FIFO
        assert!(s.recuperar(&P8).is_empty());
        assert!(s.recuperar(&[21, 22, 23, 24, 25, 26, 27, 28]).len() == 1);
    }

    #[test]
    fn test_ssd_recupera_solo_similitud_alta() {
        let mut s = SsdManager::nuevo(10);
        s.almacenar(episodio(&[1, 2, 3, 4]));
        assert!(s.recuperar(&[90, 91, 92, 93]).is_empty());
    }

    // ── Memoria Adaptativa ───────────────────────────────────────────────────
    #[test]
    fn test_memoria_nueva_con_gpu_tiene_vram() {
        let m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        assert!(m.vram.is_some());
        assert_eq!(m.vram.as_ref().unwrap().total_neuronas(), 0);
        assert_eq!(m.estadisticas().0, 0); // vram
    }

    #[test]
    fn test_memoria_nueva_sin_gpu_no_tiene_vram() {
        let m = MemoriaAdaptativa::nuevo(&config_sin_gpu());
        assert!(m.vram.is_none());
        assert_eq!(m.estadisticas().0, 0);
    }

    #[test]
    fn test_mover_a_vram_desde_ram() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        m.ram.agregar_neurona(neurona(1, 0.5));

        assert!(m.mover_a_vram(1));
        assert!(m.esta_en_vram(1));
        assert!(m.ram.obtener_neurona(1).is_none());
        assert_eq!(m.vram.as_ref().unwrap().total_neuronas(), 1);
        // Ahora se encuentra en VRAM
        assert!(m.obtener_neurona(1).is_some());
    }

    #[test]
    fn test_mover_a_ram_desde_vram() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        m.ram.agregar_neurona(neurona(1, 0.5));
        m.mover_a_vram(1);

        assert!(m.mover_a_ram(1));
        assert!(!m.esta_en_vram(1));
        assert!(m.ram.obtener_neurona(1).is_some());
        assert!(m.vram.as_ref().unwrap().obtener_neurona(1).is_none());
    }

    #[test]
    fn test_mover_a_vram_sin_gpu_no_op() {
        let mut m = MemoriaAdaptativa::nuevo(&config_sin_gpu());
        m.ram.agregar_neurona(neurona(1, 0.5));
        assert!(!m.mover_a_vram(1));
        assert!(m.ram.obtener_neurona(1).is_some());
    }

    #[test]
    fn test_obtener_neurona_mut_trae_de_vram_a_ram() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        m.ram.agregar_neurona(neurona(1, 0.5));
        m.mover_a_vram(1);

        m.obtener_neurona_mut(1).unwrap().energia = 0.77;

        // Al mutar, la neurona fue movida a RAM
        assert!(m.ram.obtener_neurona(1).unwrap().energia == 0.77);
        assert!(!m.esta_en_vram(1));
    }

    #[test]
    fn test_obtener_neurona_inexistente() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        assert!(m.obtener_neurona(999).is_none());
        assert!(m.obtener_neurona_mut(999).is_none());
    }

    #[test]
    fn test_registrar_acceso_y_optimizar() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        m.ram.agregar_neurona(neurona(1, 0.5));
        m.ram.agregar_neurona(neurona(2, 0.5));
        m.ram.agregar_neurona(neurona(3, 0.5));

        m.registrar_acceso(2);
        m.registrar_acceso(2);
        m.registrar_acceso(3);

        m.optimizar();

        // Los más accedidos (2 y 3) deben estar en VRAM
        assert!(m.esta_en_vram(2));
        assert!(m.esta_en_vram(3));
    }

    #[test]
    fn test_desalojar_de_vram_vacia_no_panica() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        m.desalojar_de_vram(); // No hay nada que desalojar
        assert_eq!(m.vram.as_ref().unwrap().total_neuronas(), 0);
    }

    #[test]
    fn test_estadisticas_completas() {
        let mut m = MemoriaAdaptativa::nuevo(&config_con_gpu());
        m.ram.agregar_neurona(neurona(1, 0.5));
        m.mover_a_vram(1);
        m.ssd.almacenar(episodio(&[1, 2]));

        let (vram_n, ram_n, total_n, ssd_e) = m.estadisticas();
        assert_eq!(vram_n, 1);
        assert_eq!(ram_n, 0);
        assert_eq!(total_n, 1);
        assert_eq!(ssd_e, 1);
    }
}
