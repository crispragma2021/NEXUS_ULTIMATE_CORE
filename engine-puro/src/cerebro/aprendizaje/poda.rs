use crate::cerebro::memoria::MemoriaAdaptativa;
use crate::cerebro::estructuras::SinapsisCompacta;
use std::collections::HashSet;

// ============================================================================
// MOTOR 4: PODA HOMEOSTÁTICA — Elimina conexiones débiles y neuronas inactivas
// ============================================================================

pub struct MotorPoda {
    pub umbral_peso_min: f32,
    pub max_sinapsis_por_neurona: usize,
    pub umbral_frecuencia_min: f32,
    pub ventana_inactividad: u64,
    pub edad_minima: u64,
    pub max_eliminar_por_ciclo: usize,
    pub sinapsis_eliminadas: u64,
    pub neuronas_eliminadas: u64,
    pub ciclos_poda: u64,
    pub paso_actual: u64,
}

impl MotorPoda {
    pub fn nuevo() -> Self {
        Self {
            umbral_peso_min: 0.01,
            max_sinapsis_por_neurona: 256,
            umbral_frecuencia_min: 0.01,
            ventana_inactividad: 10000,
            edad_minima: 1000,
            max_eliminar_por_ciclo: 100,
            sinapsis_eliminadas: 0,
            neuronas_eliminadas: 0,
            ciclos_poda: 0,
            paso_actual: 0,
        }
    }

    /// Ejecuta un ciclo completo de poda sobre la memoria
    pub fn ejecutar(&mut self, memoria: &mut MemoriaAdaptativa) {
        self.paso_actual += 1;
        self.podar_sinapsis(memoria);
        self.podar_neuronas(memoria);
        self.ciclos_poda += 1;
    }

    fn podar_sinapsis(&mut self, memoria: &mut MemoriaAdaptativa) {
        let mut eliminadas = 0u64;

        // Poda en RAM
        let neuronas_ram: Vec<u32> = memoria.ram.neuronas.iter().map(|n| n.id).collect();
        let ids_ram: HashSet<u32> = neuronas_ram.iter().cloned().collect();

        for &nid in &neuronas_ram {
            if let Some(sinapsis) = memoria.ram.sinapsis.get(&nid) {
                let vivas: Vec<SinapsisCompacta> = sinapsis.iter()
                    .filter(|s| s.peso.abs() >= self.umbral_peso_min && ids_ram.contains(&s.destino))
                    .copied()
                    .collect();

                let finales = if vivas.len() > self.max_sinapsis_por_neurona {
                    let mut sorted = vivas.clone();
                    sorted.sort_by(|a, b| b.peso.abs().partial_cmp(&a.peso.abs()).unwrap_or(std::cmp::Ordering::Equal));
                    sorted.truncate(self.max_sinapsis_por_neurona);
                    eliminadas += (vivas.len() - sorted.len()) as u64;
                    sorted
                } else {
                    eliminadas += (sinapsis.len() - vivas.len()) as u64;
                    vivas
                };
                memoria.ram.sinapsis.insert(nid, finales);
            }
        }

        // Poda en VRAM
        if let Some(vram) = &mut memoria.vram {
            let neuronas_vram: Vec<u32> = vram.neuronas.iter().map(|n| n.id).collect();
            for &nid in &neuronas_vram {
                if let Some(sinapsis) = vram.sinapsis.get(&nid) {
                    let vivas: Vec<SinapsisCompacta> = sinapsis.iter()
                        .filter(|s| s.peso.abs() >= self.umbral_peso_min)
                        .copied()
                        .collect();

                    let finales = if vivas.len() > self.max_sinapsis_por_neurona {
                        let mut sorted = vivas.clone();
                        sorted.sort_by(|a, b| b.peso.abs().partial_cmp(&a.peso.abs()).unwrap_or(std::cmp::Ordering::Equal));
                        sorted.truncate(self.max_sinapsis_por_neurona);
                        eliminadas += (vivas.len() - sorted.len()) as u64;
                        sorted
                    } else {
                        eliminadas += (sinapsis.len() - vivas.len()) as u64;
                        vivas
                    };
                    vram.sinapsis.insert(nid, finales);
                }
            }
        }

        self.sinapsis_eliminadas += eliminadas;
    }

    fn podar_neuronas(&mut self, memoria: &mut MemoriaAdaptativa) {
        let mut a_eliminar = Vec::new();

        for neurona in &memoria.ram.neuronas {
            if (neurona.edad as u64) < self.edad_minima { continue; }
            if neurona.frecuencia < self.umbral_frecuencia_min
                && neurona.activacion < 0.01
                && neurona.edad as u64 > self.ventana_inactividad
            {
                a_eliminar.push(neurona.id);
            }
        }

        let max_eliminar = self.max_eliminar_por_ciclo.min(a_eliminar.len());
        for &id in a_eliminar.iter().take(max_eliminar) {
            for (_origen, sinapsis) in &mut memoria.ram.sinapsis {
                sinapsis.retain(|s| s.destino != id);
            }
            if let Some(vram) = &mut memoria.vram {
                for (_origen, sinapsis) in &mut vram.sinapsis {
                    sinapsis.retain(|s| s.destino != id);
                }
            }
            memoria.ram.eliminar_neurona(id);
            self.neuronas_eliminadas += 1;
        }
    }

    pub fn reorganizar(&mut self, memoria: &mut MemoriaAdaptativa) {
        if memoria.vram.is_none() { return; }

        let mut candidatas: Vec<(u32, f32)> = memoria.ram.neuronas.iter()
            .filter(|n| !memoria.esta_en_vram(n.id))
            .map(|n| (n.id, n.activacion))
            .collect();
        candidatas.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for &(id, _) in candidatas.iter().take(100) {
            memoria.mover_a_vram(id);
        }
    }

    pub fn estadisticas(&self) -> (u64, u64, u64) {
        (self.sinapsis_eliminadas, self.neuronas_eliminadas, self.ciclos_poda)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::cerebro::CerebroAutoOptimizable;
    use crate::cerebro::estructuras::NeuronaCompacta;
    use crate::cerebro::estructuras::SinapsisCompacta;

    #[test]
    fn test_podar_sinapsis_debiles() {
        let mut poda = MotorPoda::nuevo();
        let mut c = CerebroAutoOptimizable::nuevo();
        // Crear neuronas de destino reales
        let dest_fuerte = c.crear_neurona(0, 0);
        let dest_debil = c.crear_neurona(0, 0);
        // Usar neurona ID 5000 como fuente (permanece en RAM, IDs 0-999 van a VRAM)
        let fuente_ram = 5000u32;
        c.memoria.ram.sinapsis.entry(fuente_ram).or_default().push(
            SinapsisCompacta::nueva(dest_debil, 0.005)
        );
        c.memoria.ram.sinapsis.entry(fuente_ram).or_default().push(
            SinapsisCompacta::nueva(dest_fuerte, 0.5)
        );
        // También para neurona ID 5001 (también en RAM)
        let fuente_ram2 = 5001u32;
        c.memoria.ram.sinapsis.entry(fuente_ram2).or_default().push(
            SinapsisCompacta::nueva(dest_debil, 0.005)
        );
        c.memoria.ram.sinapsis.entry(fuente_ram2).or_default().push(
            SinapsisCompacta::nueva(dest_fuerte, 0.5)
        );
        let total_antes: usize = c.memoria.ram.sinapsis.values().map(|v| v.len()).sum();
        poda.podar_sinapsis(&mut c.memoria);
        let total_despues: usize = c.memoria.ram.sinapsis.values().map(|v| v.len()).sum();
        assert!(total_despues < total_antes, "Deben eliminarse sinapsis débiles, antes={} después={}", total_antes, total_despues);
    }

    #[test]
    fn test_limite_sinapsis_por_neurona() {
        let mut poda = MotorPoda::nuevo();
        poda.max_sinapsis_por_neurona = 2;
        let mut c = CerebroAutoOptimizable::nuevo();
        // Crear 5 neuronas destino reales (IDs > 99999, no estarán en RAM)
        let mut dests = Vec::new();
        for _ in 0..5 { dests.push(c.crear_neurona(0, 0)); }
        // Usar neurona fuente en RAM (ID 5000)
        let fuente_ram = 5000u32;
        c.memoria.ram.sinapsis.insert(fuente_ram, dests.iter().map(|&d| SinapsisCompacta::nueva(d, 0.5)).collect());
        poda.podar_sinapsis(&mut c.memoria);
        let restantes = c.memoria.ram.sinapsis.get(&fuente_ram).map(|v| v.len()).unwrap_or(0);
        assert!(restantes <= 2, "Máximo 2 sinapsis, quedan {}", restantes);
    }

    #[test]
    fn test_podar_neurona_inactiva() {
        let mut poda = MotorPoda::nuevo();
        poda.umbral_frecuencia_min = 0.01;
        poda.edad_minima = 0;
        poda.ventana_inactividad = 0;
        let mut c = CerebroAutoOptimizable::nuevo();
        // Crear neurona y forzarla a permanecer en RAM
        let id = c.siguiente_id;
        c.siguiente_id += 1;
        let neurona = NeuronaCompacta::reposo(id, 0, 0);
        c.memoria.ram.agregar_neurona(neurona);
        c.memoria.mapa_memoria.insert(id, crate::cerebro::memoria::UbicacionMemoria::RAM);
        // Forzar como inactiva
        if let Some(n) = c.memoria.ram.obtener_neurona_mut(id) {
            n.frecuencia = 0.0;
            n.activacion = 0.0;
            n.edad = 10001;
        }
        let total_antes = c.memoria.ram.total_neuronas();
        poda.ejecutar(&mut c.memoria);
        let total_despues = c.memoria.ram.total_neuronas();
        assert!(total_despues < total_antes, "Neurona inactiva debe eliminarse, antes={} después={}", total_antes, total_despues);
    }

    #[test]
    fn test_proteger_neurona_joven() {
        let mut poda = MotorPoda::nuevo();
        poda.edad_minima = 1000;
        let mut c = CerebroAutoOptimizable::nuevo();
        // Crear neurona y forzarla a permanecer en RAM
        let id = c.siguiente_id;
        c.siguiente_id += 1;
        let neurona = NeuronaCompacta::reposo(id, 0, 0);
        c.memoria.ram.agregar_neurona(neurona);
        c.memoria.mapa_memoria.insert(id, crate::cerebro::memoria::UbicacionMemoria::RAM);
        if let Some(n) = c.memoria.ram.obtener_neurona_mut(id) {
            n.frecuencia = 0.0;
            n.activacion = 0.0;
            n.edad = 100;
        }
        let total_antes = c.memoria.ram.total_neuronas();
        poda.ejecutar(&mut c.memoria);
        let total_despues = c.memoria.ram.total_neuronas();
        assert_eq!(total_despues, total_antes, "Neurona joven no debe eliminarse");
    }

    #[test]
    fn test_reorganizar_vram() {
        let mut poda = MotorPoda::nuevo();
        let mut c = CerebroAutoOptimizable::nuevo();
        poda.reorganizar(&mut c.memoria);
    }

    #[test]
    fn test_max_eliminar_por_ciclo() {
        let mut poda = MotorPoda::nuevo();
        poda.max_eliminar_por_ciclo = 2;
        poda.umbral_frecuencia_min = 0.01;
        poda.edad_minima = 0;
        poda.ventana_inactividad = 0;
        let mut c = CerebroAutoOptimizable::nuevo();
        for _ in 0..5 {
            // Forzar neuronas a permanecer en RAM
            let id = c.siguiente_id;
            c.siguiente_id += 1;
            let neurona = NeuronaCompacta::reposo(id, 0, 0);
            c.memoria.ram.agregar_neurona(neurona);
            c.memoria.mapa_memoria.insert(id, crate::cerebro::memoria::UbicacionMemoria::RAM);
            if let Some(n) = c.memoria.ram.obtener_neurona_mut(id) {
                n.frecuencia = 0.0;
                n.activacion = 0.0;
                n.edad = 10001;
            }
        }
        let antes = poda.neuronas_eliminadas;
        poda.ejecutar(&mut c.memoria);
        let diff = poda.neuronas_eliminadas - antes;
        assert!(diff <= 2, "Máximo 2 eliminaciones, fueron {}", diff);
    }

    #[test]
    fn test_podar_sinapsis_huerfanas() {
        let mut poda = MotorPoda::nuevo();
        let mut c = CerebroAutoOptimizable::nuevo();
        // Crear neurona fuente
        let id_fuente = c.siguiente_id;
        c.siguiente_id += 1;
        let neurona = NeuronaCompacta::reposo(id_fuente, 0, 0);
        c.memoria.ram.agregar_neurona(neurona);
        c.memoria.mapa_memoria.insert(id_fuente, crate::cerebro::memoria::UbicacionMemoria::RAM);
        // Sinapsis a neuronas que NO existen (IDs gigantescos fuera de rango)
        c.memoria.ram.sinapsis.entry(id_fuente).or_default().push(SinapsisCompacta::nueva(9_999_999, 0.5));
        c.memoria.ram.sinapsis.entry(id_fuente).or_default().push(SinapsisCompacta::nueva(9_999_998, 0.3));
        let antes = c.memoria.ram.sinapsis.get(&id_fuente).map(|v| v.len()).unwrap_or(0);
        poda.podar_sinapsis(&mut c.memoria);
        let despues = c.memoria.ram.sinapsis.get(&id_fuente).map(|v| v.len()).unwrap_or(0);
        assert!(despues < antes, "Sinapsis huérfanas deben eliminarse");
        assert_eq!(despues, 0, "Sinapsis sin destino válido eliminadas, quedaron {}", despues);
    }
}
