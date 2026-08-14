// ==========================================
// 🧠 AFINIDAD SOBERANA — Topología de CPU
// ==========================================
// Analiza la topología P-Cores/E-Cores (Intel Alder-Lake+).
// Linux: sysfs + sched_setaffinity. Windows: detección genérica + SetThreadAffinityMask.
// ==========================================

use std::fs;
use tracing::{info, warn};

/// [DNA ALDER-LAKE] Analiza la topología de la CPU Intel i7-12700F.
pub struct TopologiaCPU {
    pub p_cores: Vec<usize>, // Hilos de alto rendimiento (0-15)
    pub e_cores: Vec<usize>, // Hilos de eficiencia (16-19)
}

impl TopologiaCPU {
    pub fn detectar() -> Self {
        #[cfg(target_os = "linux")]
        {
            // sysfs (Fuente de verdad en Linux)
            let p_cores = fs::read_to_string("/sys/devices/cpu_core/cpus")
                .ok()
                .map(|s| Self::parse_cpu_list(&s))
                .unwrap_or_else(|| (0..16).collect()); // Fallback i7-12700F

            let e_cores = fs::read_to_string("/sys/devices/cpu_atom/cpus")
                .ok()
                .map(|s| Self::parse_cpu_list(&s))
                .unwrap_or_else(|| (16..20).collect()); // Fallback i7-12700F

            return TopologiaCPU { p_cores, e_cores };
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: sin sysfs. Usamos conteo lógico total; los P-Cores
            // suelen ser la primera mitad de hilos en Alder-Lake.
            let total = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(20);
            let p_count = total.saturating_sub(4).max(total / 2);
            TopologiaCPU {
                p_cores: (0..p_count).collect(),
                e_cores: (p_count..total).collect(),
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            TopologiaCPU {
                p_cores: (0..16).collect(),
                e_cores: (16..20).collect(),
            }
        }
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
        self._aplicar_afinidad(&self.topologia.p_cores, "P-Cores", "Soberanía en P-Cores activada");
    }

    /// Vincula el hilo actual a los E-Cores para tareas de fondo.
    pub fn relegar_a_e_cores(&self) {
        self._aplicar_afinidad(
            &self.topologia.e_cores,
            "E-Cores",
            "Tarea relegada a núcleos de eficiencia",
        );
    }

    fn _aplicar_afinidad(&self, cores: &[usize], _nombre: &str, msg: &str) {
        #[cfg(target_os = "linux")]
        {
            use nix::sched::{sched_setaffinity, CpuSet};
            use nix::unistd::Pid;
            let mut cpuset = CpuSet::new();
            for &id in cores {
                let _ = cpuset.set(id);
            }
            if sched_setaffinity(Pid::from_raw(0), &cpuset).is_ok() {
                info!("🧠 [AFINIDAD] {msg}.");
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows: SetThreadAffinityMask con los hilos indicados.
            // La máscara se construye con bits de los índices.
            if let Some(mask) = Self::_mascara_hilos(cores) {
                // SAFETY: llamada directa al API de Windows sin invariantes extra.
                unsafe {
                    let hilo = windows_sys::Win32::System::Threading::GetCurrentThread();
                    let _ = windows_sys::Win32::System::Threading::SetThreadAffinityMask(hilo, mask);
                }
                info!("🧠 [AFINIDAD] {msg}.");
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            let _ = cores;
            warn!("🧠 [AFINIDAD] No soportado en esta plataforma.");
        }
    }

    #[cfg(target_os = "windows")]
    fn _mascara_hilos(cores: &[usize]) -> Option<usize> {
        let mut mask = 0usize;
        for &c in cores {
            if c < std::mem::size_of::<usize>() * 8 {
                mask |= 1usize << c;
            }
        }
        if mask == 0 {
            None
        } else {
            Some(mask)
        }
    }
}
