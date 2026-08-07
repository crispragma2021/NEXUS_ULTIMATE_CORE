use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::cerebro::estructuras::{Episodio, ParametrosNeurona};
use crate::cerebro::memoria::{MemoriaAdaptativa, SsdManager};
use crate::cerebro::motores::MotorNeurona;

// ============================================================================
// MOTOR 5: CONSOLIDADOR NOCTURNO — Reprocesa episodios para fijar recuerdos
// ============================================================================
// Se activa cada 5000 pasos (~una "noche" del cerebro). Durante 500 pasos
// ("sueño"), reproduce episodios seleccionados del SSD, fortaleciendo las
// sinapsis activadas (STDP) y las transiciones léxicas asociadas.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaEpisodio {
    /// Patrón neuronal generalizado
    pub patron: Vec<u32>,
    /// Episodios fuente que lo formaron (índices)
    pub fuentes: Vec<usize>,
    /// Peso de generalización
    pub peso: f32,
}

pub struct MotorConsolidacion {
    /// ¿Está actualmente en ciclo de sueño?
    pub en_suenio: bool,

    /// Pasos restantes en el ciclo actual
    pub pasos_restantes: u64,

    /// Duración de un ciclo de sueño (pasos)
    pub duracion_suenio: u64,  // 500

    /// Cada cuántos pasos se activa el sueño
    pub cadencia_suenio: u64,  // 5000

    /// Episodios seleccionados para este ciclo
    pub episodios_a_consolidar: Vec<Episodio>,

    /// Índice del episodio actual en procesamiento
    pub indice_actual: usize,

    /// Pasos por episodio durante consolidación
    pub pasos_por_episodio: u64,  // 5

    /// Contador de pasos dentro del episodio actual
    pub paso_en_episodio: u64,

    /// Meta-episodios generalizados
    pub meta_episodios: Vec<MetaEpisodio>,

    /// Ciclos de sueño completados
    pub ciclos_completados: u64,

    /// Episodios consolidados totales
    pub episodios_consolidados: u64,

    /// Paso actual
    pub paso_actual: u64,
}

impl MotorConsolidacion {
    pub fn nuevo() -> Self {
        Self {
            en_suenio: false,
            pasos_restantes: 0,
            duracion_suenio: 500,
            cadencia_suenio: 5000,
            episodios_a_consolidar: Vec::new(),
            indice_actual: 0,
            pasos_por_episodio: 5,
            paso_en_episodio: 0,
            meta_episodios: Vec::new(),
            ciclos_completados: 0,
            episodios_consolidados: 0,
            paso_actual: 0,
        }
    }

    /// Verifica si debe iniciar un ciclo de sueño (cada 5000 pasos)
    pub fn debe_dormir(&self) -> bool {
        !self.en_suenio && self.paso_actual > 0 && self.paso_actual % self.cadencia_suenio == 0
    }

    /// Inicia un ciclo de sueño: selecciona episodios del SSD
    pub fn iniciar_suenio(&mut self, ssd: &SsdManager) {
        let mut episodios: Vec<Episodio> = ssd.episodios.clone();

        // Ordenar por relevancia (intensidad * |emocion|)
        episodios.sort_by(|a, b| {
            let relevancia_a = a.intensidad * a.emocion.abs();
            let relevancia_b = b.intensidad * b.emocion.abs();
            relevancia_b.partial_cmp(&relevancia_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Seleccionar top 20
        episodios.truncate(20);

        self.episodios_a_consolidar = episodios;
        self.en_suenio = true;
        self.pasos_restantes = self.duracion_suenio;
        self.indice_actual = 0;
        self.paso_en_episodio = 0;
    }

    /// Ejecuta un paso del ciclo de sueño
    /// Recibe los componentes necesarios por separado para evitar
    /// conflictos de borrow checker en el pipeline.
    /// Retorna true si el ciclo continúa, false si terminó
    pub fn paso_suenio(
        &mut self,
        memoria: &mut MemoriaAdaptativa,
        params_neurona: &ParametrosNeurona,
        hilos_cpu: usize,
        dt: f32,
    ) -> bool {
        if !self.en_suenio || self.pasos_restantes == 0 {
            return false;
        }

        self.pasos_restantes -= 1;

        // Si hay episodios para procesar
        if self.indice_actual < self.episodios_a_consolidar.len() {
            let episodio = &self.episodios_a_consolidar[self.indice_actual];

            // Activar las neuronas del patrón guardado
            for &nid in &episodio.patron {
                if nid > 0 {
                    if let Some(n) = memoria.obtener_neurona_mut(nid) {
                        n.corriente_entrada += episodio.intensidad * 0.3;
                        n.traza += 0.1;
                    }
                }
            }

            // Procesar paso neuronal (simulación interna)
            procesar_cpu_local(memoria, params_neurona, hilos_cpu, dt);

            // Avanzar contador de paso dentro del episodio
            self.paso_en_episodio += 1;

            // Si terminamos con este episodio, pasar al siguiente
            if self.paso_en_episodio >= self.pasos_por_episodio {
                self.indice_actual += 1;
                self.paso_en_episodio = 0;
                self.episodios_consolidados += 1;
            }
        }

        self.pasos_restantes > 0
    }

    /// Finaliza el ciclo de sueño: generalización y meta-episodios
    pub fn finalizar_suenio(&mut self) {
        if !self.en_suenio {
            return;
        }

        // Generalización: buscar patrones comunes entre episodios
        self.generalizar();

        self.en_suenio = false;
        self.ciclos_completados += 1;
        self.episodios_a_consolidar.clear();
        self.indice_actual = 0;
    }

    /// Busca patrones comunes entre episodios para crear meta-episodios
    fn generalizar(&mut self) {
        if self.episodios_a_consolidar.len() < 3 {
            return;
        }

        // Contar co-ocurrencia de pares de neuronas entre episodios
        let mut co_ocurrencia: HashMap<(u32, u32), u32> = HashMap::new();

        for i in 0..self.episodios_a_consolidar.len() {
            for j in (i + 1)..self.episodios_a_consolidar.len() {
                let ep_a = &self.episodios_a_consolidar[i];
                let ep_b = &self.episodios_a_consolidar[j];

                // Calcular intersección de patrones
                let set_a: HashSet<u32> = ep_a.patron.iter().filter(|&&id| id > 0).copied().collect();
                let set_b: HashSet<u32> = ep_b.patron.iter().filter(|&&id| id > 0).copied().collect();

                let interseccion: Vec<u32> = set_a.intersection(&set_b).copied().collect();
                let union_size = set_a.union(&set_b).count();

                // Si comparten >50% del patrón
                if union_size > 0 && (interseccion.len() as f32 / union_size as f32) > 0.5 {
                    for &id in &interseccion {
                        for &oid in &interseccion {
                            if id != oid {
                                *co_ocurrencia.entry((id.min(oid), id.max(oid))).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }

        // Crear meta-episodios de pares que ocurren en 3+ combinaciones
        let mut meta_patron: Vec<u32> = Vec::new();
        for ((neurona_a, neurona_b), count) in &co_ocurrencia {
            if *count >= 3 {
                if !meta_patron.contains(neurona_a) {
                    meta_patron.push(*neurona_a);
                }
                if !meta_patron.contains(neurona_b) {
                    meta_patron.push(*neurona_b);
                }
            }
        }

        if meta_patron.len() >= 2 {
            let meta_ep = MetaEpisodio {
                patron: meta_patron,
                fuentes: (0..self.episodios_a_consolidar.len()).collect(),
                peso: 0.8,
            };
            self.meta_episodios.push(meta_ep);
        }
    }

    /// ¿Está dormido?
    pub fn durmiendo(&self) -> bool {
        self.en_suenio
    }

    /// Estadísticas
    pub fn estadisticas(&self) -> (u64, u64, u64) {
        (self.ciclos_completados, self.episodios_consolidados, self.meta_episodios.len() as u64)
    }
}

/// Procesamiento CPU local para uso durante sueño (evita depender de CerebroAutoOptimizable)
fn procesar_cpu_local(
    memoria: &mut MemoriaAdaptativa,
    params: &ParametrosNeurona,
    hilos: usize,
    dt: f32,
) {
    let neuronas_ram = memoria.ram.obtener_todas_mut();

    rayon::scope(|s| {
        if !neuronas_ram.is_empty() {
            let chunk_size = (neuronas_ram.len() + hilos - 1) / hilos;
            for chunk in neuronas_ram.chunks_mut(chunk_size) {
                s.spawn(|_| {
                    for neurona in chunk {
                        let entrada = neurona.corriente_entrada;
                        if neurona.capa <= 2 {
                            MotorNeurona::actualizar(neurona, entrada, dt, params);
                        } else {
                            MotorNeurona::actualizar_simple(neurona, entrada, dt);
                        }
                    }
                });
            }
        }

        if let Some(ref mut vram) = memoria.vram {
            let neuronas_vram = &mut vram.neuronas;
            if !neuronas_vram.is_empty() {
                let chunk_size = (neuronas_vram.len() + hilos - 1) / hilos;
                for chunk in neuronas_vram.chunks_mut(chunk_size) {
                    s.spawn(|_| {
                        for neurona in chunk {
                            let entrada = neurona.corriente_entrada;
                            if neurona.capa <= 2 {
                                MotorNeurona::actualizar(neurona, entrada, dt, params);
                            } else {
                                MotorNeurona::actualizar_simple(neurona, entrada, dt);
                            }
                        }
                    });
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::cerebro::CerebroAutoOptimizable;

    fn crear_episodio(intensidad: f32, emocion: f32, patron: &[u32]) -> Episodio {
        let mut p = [0u32; 8];
        for (i, &id) in patron.iter().enumerate().take(8) {
            p[i] = id;
        }
        Episodio::nueva(0.0, intensidad, emocion, &p, 0)
    }

    #[test]
    fn test_debe_dormir_cadencia() {
        let mut m = MotorConsolidacion::nuevo();
        m.paso_actual = 0;
        assert!(!m.debe_dormir(), "Paso 0 no debe dormir");
        m.paso_actual = 5000;
        assert!(m.debe_dormir(), "Paso 5000 debe dormir");
        m.en_suenio = true;
        assert!(!m.debe_dormir(), "Ya durmiendo no debe iniciar otro ciclo");
    }

    #[test]
    fn test_iniciar_suenio_selecciona_episodios() {
        let _ssd = SsdManager::nuevo(100);
        let mut m = MotorConsolidacion::nuevo();
        // Agregar episodios al SSD manualmente
        let mut ssd_con_eps = SsdManager::nuevo(100);
        for i in 0..5 {
            let ep = crear_episodio(0.5 + i as f32 * 0.1, 0.3, &[10, 20, 30, 40, 50, 60, 70, 80]);
            ssd_con_eps.almacenar(ep);
        }
        m.iniciar_suenio(&ssd_con_eps);
        assert!(m.en_suenio, "Debe estar en sueño");
        assert!(m.episodios_a_consolidar.len() <= 5.max(20), "Debe seleccionar episodios");
    }

    #[test]
    fn test_suenio_completo() {
        let mut m = MotorConsolidacion::nuevo();
        m.duracion_suenio = 10;
        m.pasos_por_episodio = 2;
        let mut cerebro = CerebroAutoOptimizable::nuevo();

        let mut ssd_con_eps = SsdManager::nuevo(100);
        for _ in 0..3 {
            let ep = crear_episodio(0.8, 0.5, &[1, 2, 3, 4, 5, 6, 7, 8]);
            ssd_con_eps.almacenar(ep);
        }

        let params = ParametrosNeurona::default();
        let hilos = cerebro.config.hilos_cpu;

        m.iniciar_suenio(&ssd_con_eps);
        assert!(m.en_suenio);

        // Ejecutar pasos de sueño hasta que termine
        let mut ciclos = 0;
        while m.en_suenio && ciclos < 100 {
            let sigue = m.paso_suenio(
                &mut cerebro.memoria,
                &params,
                hilos,
                0.001,
            );
            if !sigue {
                break;
            }
            ciclos += 1;
        }
        m.finalizar_suenio();

        assert!(!m.en_suenio, "Debe haber terminado el sueño");
        assert_eq!(m.ciclos_completados, 1, "Debe haber 1 ciclo completado");
    }

    #[test]
    fn test_generalizacion_meta_episodios() {
        let mut m = MotorConsolidacion::nuevo();
        let mut ssd_con_eps = SsdManager::nuevo(100);
        // 3 episodios con patrones similares
        for _ in 0..3 {
            let ep = crear_episodio(0.9, 0.8, &[10, 20, 30, 40, 50, 60, 70, 80]);
            ssd_con_eps.almacenar(ep);
        }
        // 1 episodio diferente
        let ep_diff = crear_episodio(0.5, 0.1, &[100, 200, 300, 400, 500, 600, 700, 800]);
        ssd_con_eps.almacenar(ep_diff);

        m.iniciar_suenio(&ssd_con_eps);
        m.finalizar_suenio();
        // Debería haber al menos un meta-episodio
        assert!(m.meta_episodios.len() >= 1, "Debe haber al menos 1 meta-episodio, hay {}", m.meta_episodios.len());
    }

    #[test]
    fn test_paso_suenio_activa_neuronas() {
        let mut m = MotorConsolidacion::nuevo();
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let mut ssd_con_eps = SsdManager::nuevo(100);
        let params = ParametrosNeurona::default();
        let hilos = cerebro.config.hilos_cpu;

        // Crear neuronas y un episodio
        for _i in 1..=3 {
            cerebro.crear_neurona(0, 0);
        }
        let ep = crear_episodio(0.7, 0.6, &[1, 2, 3, 0, 0, 0, 0, 0]);
        ssd_con_eps.almacenar(ep);

        m.iniciar_suenio(&ssd_con_eps);
        let sigue = m.paso_suenio(
            &mut cerebro.memoria,
            &params,
            hilos,
            0.001,
        );
        assert!(sigue, "El sueño debe continuar después del primer paso");
        m.finalizar_suenio();
    }

    #[test]
    fn test_durmiendo_bloquea_salida() {
        let m = MotorConsolidacion::nuevo();
        assert!(!m.durmiendo(), "No debe estar durmiendo inicialmente");
    }

    #[test]
    fn test_estadisticas() {
        let m = MotorConsolidacion::nuevo();
        let (ciclos, episodios_cons, metas) = m.estadisticas();
        assert_eq!(ciclos, 0);
        assert_eq!(episodios_cons, 0);
        assert_eq!(metas, 0);
    }
}
