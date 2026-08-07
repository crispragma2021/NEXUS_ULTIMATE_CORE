use nix::sched::{sched_setaffinity, CpuSet};
use nix::unistd::Pid;
use std::fs;
use tracing::info;

/// [DNA ALDER-LAKE] Analiza la topología de la CPU Intel i7-12700F.
pub struct TopologiaCPU {
    pub p_cores: Vec<usize>, // Hilos de alto rendimiento (0-15)
    pub e_cores: Vec<usize>, // Hilos de eficiencia (16-19)
}

impl TopologiaCPU {
    pub fn detectar() -> Self {
        // Intentamos detectar vía sysfs (Fuente de verdad en Linux)
        let p_cores = fs::read_to_string("/sys/devices/cpu_core/cpus")
            .ok()
            .map(|s| Self::parse_cpu_list(&s))
            .unwrap_or_else(|| (0..16).collect()); // Fallback i7-12700F

        let e_cores = fs::read_to_string("/sys/devices/cpu_atom/cpus")
            .ok()
            .map(|s| Self::parse_cpu_list(&s))
            .unwrap_or_else(|| (16..20).collect()); // Fallback i7-12700F

        TopologiaCPU { p_cores, e_cores }
    }

    fn parse_cpu_list(lista: &str) -> Vec<usize> {
        let mut cores = Vec::new();
        for parte in lista.trim().split(',') {
            if parte.contains('-') {
                let rango: Vec<&str> = parte.split('-').collect();
                if rango.len() == 2 {
                    if let (Ok(inicio), Ok(fin)) =
                        (rango[0].parse::<usize>(), rango[1].parse::<usize>())
                    {
                        cores.extend(inicio..=fin);
                    }
                }
            } else if let Ok(num) = parte.parse::<usize>() {
                cores.push(num);
            }
        }
        cores
    }
}

pub struct AfinidadSoberana {
    topologia: TopologiaCPU,
}

impl Default for AfinidadSoberana {
    fn default() -> Self {
        Self::new()
    }
}

impl AfinidadSoberana {
    pub fn new() -> Self {
        Self {
            topologia: TopologiaCPU::detectar(),
        }
    }

    /// Vincula el hilo actual a los P-Cores para máximo rendimiento.
    pub fn exigir_p_cores(&self) {
        let mut cpuset = CpuSet::new();
        for &id in &self.topologia.p_cores {
            let _ = cpuset.set(id);
        }
        if sched_setaffinity(Pid::from_raw(0), &cpuset).is_ok() {
            info!("🧠 [AFINIDAD] Soberanía en P-Cores activada.");
        }
    }

    /// Vincula el hilo actual a los E-Cores para tareas de fondo.
    pub fn relegar_a_e_cores(&self) {
        let mut cpuset = CpuSet::new();
        for &id in &self.topologia.e_cores {
            let _ = cpuset.set(id);
        }
        if sched_setaffinity(Pid::from_raw(0), &cpuset).is_ok() {
            info!("🍃 [AFINIDAD] Tarea relegada a núcleos de eficiencia.");
        }
    }
}
