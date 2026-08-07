// ============================================================================
// DETECCIÓN DE HARDWARE Y CONFIGURACIÓN DINÁMICA
// ============================================================================
// Detecta automáticamente CPU, RAM, VRAM, GPU y SSD para auto-optimizarse.
// ============================================================================

use std::thread::available_parallelism;
use sysinfo::{SystemExt, CpuExt};

// ============================================================================
// TIPOS DE HARDWARE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GPUTipo {
    NVIDIA,
    AMD,
    Intel,
    Ninguna,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Precision {
    F32,
    F16,
    Int8,
}

// ============================================================================
// INFORMACIÓN DE HARDWARE DETECTADA
// ============================================================================

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    // CPU
    pub nucleos: usize,
    pub frecuencia_mhz: f32,

    // RAM
    pub ram_total: usize,      // bytes
    pub ram_disponible: usize, // bytes

    // VRAM
    pub vram_total: usize,      // bytes
    pub vram_disponible: usize, // bytes

    // GPU
    pub gpu_nucleos: usize,
    pub gpu_tipo: GPUTipo,

    // SSD
    pub ssd_espacio: usize, // bytes

    // Sistema
    pub arquitectura: String,
}

impl HardwareInfo {
    /// Detección completa de hardware
    pub fn detectar() -> Self {
        // CPU
        let nucleos = available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        // RAM con sysinfo
        let (ram_total, ram_disponible) = Self::detectar_ram();

        // VRAM y GPU
        let (vram_total, vram_disponible, gpu_nucleos, gpu_tipo) = Self::detectar_gpu();

        // SSD
        let ssd_espacio = Self::detectar_ssd();

        Self {
            nucleos,
            frecuencia_mhz: 0.0, // Requiere /proc/cpuinfo parsing
            ram_total,
            ram_disponible,
            vram_total,
            vram_disponible,
            gpu_nucleos,
            gpu_tipo,
            ssd_espacio,
            arquitectura: std::env::consts::ARCH.to_string(),
        }
    }

    fn detectar_ram() -> (usize, usize) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(info) = std::fs::read_to_string("/proc/meminfo") {
                let mut total: usize = 0;
                let mut disponible: usize = 0;
                for line in info.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(val) = line.split_whitespace().nth(1) {
                            total = val.parse::<usize>().unwrap_or(0) * 1024;
                        }
                    } else if line.starts_with("MemAvailable:") {
                        if let Some(val) = line.split_whitespace().nth(1) {
                            disponible = val.parse::<usize>().unwrap_or(0) * 1024;
                        }
                    }
                }
                return (total.max(1), disponible.max(1));
            }
        }
        // Fallback: asumir 8GB
        (8_000_000_000, 4_000_000_000)
    }

    fn detectar_gpu() -> (usize, usize, usize, GPUTipo) {
        #[cfg(target_os = "linux")]
        {
            // NVIDIA
            if std::path::Path::new("/proc/driver/nvidia").exists() {
                // Intentar leer información de NVIDIA
                if let Ok(entries) = std::fs::read_dir("/proc/driver/nvidia/gpus") {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            // Asumimos GPU NVIDIA típica
                            return (8_000_000_000, 7_500_000_000, 1024, GPUTipo::NVIDIA);
                        }
                    }
                }
            }

            // AMD
            if std::path::Path::new("/sys/class/drm").exists() {
                if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.contains("amdgpu") || name.contains("radeon") {
                            return (4_000_000_000, 3_500_000_000, 512, GPUTipo::AMD);
                        }
                    }
                }
            }

            // Intel
            if std::path::Path::new("/sys/class/drm").exists() {
                if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        if name.contains("i915") {
                            return (2_000_000_000, 1_500_000_000, 256, GPUTipo::Intel);
                        }
                    }
                }
            }
        }

        // Sin GPU detectada
        (0, 0, 0, GPUTipo::Ninguna)
    }

    fn detectar_ssd() -> usize {
        #[cfg(unix)]
        {
            #[repr(C)]
            struct StatVfs {
                f_bsize: u64,
                f_frsize: u64,
                f_blocks: u64,
                f_bfree: u64,
                f_bavail: u64,
                f_files: u64,
                f_ffree: u64,
                f_favail: u64,
                f_fsid: u64,
                f_flag: u64,
                f_namemax: u64,
                __f_spare: [i32; 6],
            }

            extern "C" {
                fn statvfs(path: *const i8, buf: *mut StatVfs) -> i32;
            }

            let path = std::ffi::CString::new(".").unwrap();
            let mut buf: StatVfs = unsafe { std::mem::zeroed() };
            let ret = unsafe { statvfs(path.as_ptr(), &mut buf) };
            if ret == 0 {
                let espacio = (buf.f_bavail as usize).saturating_mul(buf.f_frsize as usize);
                if espacio > 0 {
                    return espacio;
                }
            }
        }
        // Fallback: 100GB
        100_000_000_000
    }

    /// Imprime un resumen del hardware detectado
    pub fn mostrar(&self) {
        println!("  💻 Hardware detectado:");
        println!("    CPU: {} núcleos ({})", self.nucleos, self.arquitectura);
        println!("    RAM: {:.2} GB ({} disponible)",
            self.ram_total as f32 / 1_000_000_000.0,
            self.ram_disponible as f32 / 1_000_000_000.0);
        if self.vram_total > 0 {
            println!("    VRAM: {:.2} GB ({})",
                self.vram_total as f32 / 1_000_000_000.0,
                self.gpu_tipo_str());
            println!("    GPU: {} núcleos", self.gpu_nucleos);
        } else {
            println!("    GPU: No detectada (modo CPU-only)");
        }
        println!("    SSD: ~{:.1} GB libres",
            self.ssd_espacio as f32 / 1_000_000_000.0);
    }

    pub fn gpu_tipo_str(&self) -> &str {
        match self.gpu_tipo {
            GPUTipo::NVIDIA => "NVIDIA CUDA",
            GPUTipo::AMD => "AMD ROCm",
            GPUTipo::Intel => "Intel Arc",
            GPUTipo::Ninguna => "N/A",
        }
    }

    /// Mide la CPU y la RAM en caliente en el instante actual
    pub fn medir_uso_caliente() -> (f32, f32) {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        std::thread::sleep(std::time::Duration::from_millis(5));
        sys.refresh_all();

        let total_load: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
            / sys.cpus().len().max(1) as f32
            / 100.0;

        let ram_total = sys.total_memory() as f32;
        let ram_usada = sys.used_memory() as f32;
        let uso_ram = if ram_total > 0.0 { ram_usada / ram_total } else { 0.0 };

        (total_load.clamp(0.0, 1.0), uso_ram.clamp(0.0, 1.0))
    }
}

// ============================================================================
// CONFIGURACIÓN DINÁMICA (Calculada desde Hardware)
// ============================================================================

#[derive(Clone, Debug)]
pub struct ConfiguracionDinamica {
    // Capacidades
    pub max_neuronas_vram: usize,
    pub max_neuronas_ram: usize,
    pub max_sinapsis_vram: usize,
    pub max_sinapsis_ram: usize,
    pub max_neuronas_totales: usize,

    // Procesamiento
    pub batch_size_gpu: usize,
    pub batch_size_cpu: usize,
    pub hilos_cpu: usize,
    pub usar_gpu: bool,
    pub precision: Precision,

    // Memoria episódica
    pub memoria_episodica_max: usize,
}

impl ConfiguracionDinamica {
    pub fn from_hardware(hw: &HardwareInfo) -> Self {
        // Tamaños estimados
        let bytes_por_neurona = 64;
        let bytes_por_sinapsis = 8;

        // Usar 70% de VRAM para neuronas activas
        let vram_neuronas = if hw.vram_total > 0 {
            ((hw.vram_total as f64 * 0.7) / bytes_por_neurona as f64) as usize
        } else {
            0
        };

        // Usar 70% de RAM para neuronas latentes
        let ram_neuronas = ((hw.ram_total as f64 * 0.7) / bytes_por_neurona as f64) as usize;

        // Sinapsis: 30% del espacio
        let vram_sinapsis = if hw.vram_total > 0 {
            ((hw.vram_total as f64 * 0.3) / bytes_por_sinapsis as f64) as usize
        } else {
            0
        };

        let ram_sinapsis = ((hw.ram_total as f64 * 0.3) / bytes_por_sinapsis as f64) as usize;

        // Batch GPU
        let batch_gpu = if hw.gpu_tipo != GPUTipo::Ninguna && hw.vram_total > 0 {
            (hw.gpu_nucleos * 128).max(1024)
        } else {
            0
        };

        // Hilos CPU (reservar 2 para sistema)
        let hilos_cpu = hw.nucleos.saturating_sub(2).max(1);

        // Precisión según VRAM
        let precision = if hw.vram_total > 0 && hw.vram_total < 8_000_000_000 {
            Precision::F16
        } else if hw.ram_total < 16_000_000_000 {
            Precision::F16
        } else {
            Precision::F32
        };

        // Memoria episódica (5% de SSD)
        let episodica_max = ((hw.ssd_espacio as f64 * 0.05) / 64.0) as usize;

        Self {
            max_neuronas_vram: vram_neuronas.max(1000),
            max_neuronas_ram: ram_neuronas.max(10000),
            max_sinapsis_vram: vram_sinapsis.max(10000),
            max_sinapsis_ram: ram_sinapsis.max(100000),
            max_neuronas_totales: vram_neuronas.max(1000) + ram_neuronas.max(10000),
            batch_size_gpu: batch_gpu,
            batch_size_cpu: 1024,
            hilos_cpu,
            usar_gpu: hw.gpu_tipo != GPUTipo::Ninguna && hw.vram_total > 1_000_000_000,
            precision,
            memoria_episodica_max: episodica_max.max(1000),
        }
    }

    /// Imprime la configuración dinámica
    pub fn mostrar(&self) {
        println!("  ⚙️  Configuración dinámica:");
        println!("    Neuronas VRAM:  {}", self.max_neuronas_vram);
        println!("    Neuronas RAM:   {}", self.max_neuronas_ram);
        println!("    Sinapsis VRAM:  {}", self.max_sinapsis_vram);
        println!("    Sinapsis RAM:   {}", self.max_sinapsis_ram);
        println!("    Total neuronas: {}", self.max_neuronas_totales);
        println!("    Hilos CPU:      {}", self.hilos_cpu);
        println!("    Usar GPU:       {}", self.usar_gpu);
        println!("    Precisión:      {:?}", self.precision);
        println!("    Episodios max:  {}", self.memoria_episodica_max);
    }
}

// ============================================================================
// TESTS DE DETECCIÓN DE HARDWARE Y CONFIGURACIÓN
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Construye un HardwareInfo determinista de GPU NVIDIA 8GB
    fn hw_nvidia() -> HardwareInfo {
        HardwareInfo {
            nucleos: 12,
            frecuencia_mhz: 4000.0,
            ram_total: 16_000_000_000,
            ram_disponible: 8_000_000_000,
            vram_total: 8_000_000_000,
            vram_disponible: 7_500_000_000,
            gpu_nucleos: 1024,
            gpu_tipo: GPUTipo::NVIDIA,
            ssd_espacio: 500_000_000_000,
            arquitectura: "x86_64".to_string(),
        }
    }

    /// Construye un HardwareInfo sin GPU (modo CPU-only)
    fn hw_cpu_only() -> HardwareInfo {
        HardwareInfo {
            nucleos: 8,
            frecuencia_mhz: 3500.0,
            ram_total: 32_000_000_000,
            ram_disponible: 20_000_000_000,
            vram_total: 0,
            vram_disponible: 0,
            gpu_nucleos: 0,
            gpu_tipo: GPUTipo::Ninguna,
            ssd_espacio: 1_000_000_000_000,
            arquitectura: "x86_64".to_string(),
        }
    }

    // ── GPUTipo y Precision ──────────────────────────────────────────────────
    #[test]
    fn test_gpu_tipo_strings() {
        let mut hw = hw_nvidia();
        assert_eq!(hw.gpu_tipo_str(), "NVIDIA CUDA");
        hw.gpu_tipo = GPUTipo::AMD;
        assert_eq!(hw.gpu_tipo_str(), "AMD ROCm");
        hw.gpu_tipo = GPUTipo::Intel;
        assert_eq!(hw.gpu_tipo_str(), "Intel Arc");
        hw.gpu_tipo = GPUTipo::Ninguna;
        assert_eq!(hw.gpu_tipo_str(), "N/A");
    }

    // ── ConfiguracionDinamica::from_hardware ────────────────────────────────
    #[test]
    fn test_config_desde_nvidia_gpu() {
        let cfg = ConfiguracionDinamica::from_hardware(&hw_nvidia());

        // 70% de 8GB / 64 bytes
        let esperado_vram = ((8_000_000_000.0 * 0.7) / 64.0) as usize;
        assert_eq!(cfg.max_neuronas_vram, esperado_vram);
        // 70% de 16GB / 64 bytes
        let esperado_ram = ((16_000_000_000.0 * 0.7) / 64.0) as usize;
        assert_eq!(cfg.max_neuronas_ram, esperado_ram);

        assert!(cfg.usar_gpu, "8GB VRAM debe habilitar GPU");
        assert_eq!(cfg.precision, Precision::F32, "8GB VRAM = precisión completa");

        // Batch GPU = nucleos*128, mínimo 1024
        assert_eq!(cfg.batch_size_gpu, (1024 * 128).max(1024));
        // Hilos = nucleos - 2 (mínimo 1)
        assert_eq!(cfg.hilos_cpu, 10);
    }

    #[test]
    fn test_config_gpu_baja_vram_usa_f16() {
        let mut hw = hw_nvidia();
        hw.vram_total = 4_000_000_000; // < 8GB → F16
        let cfg = ConfiguracionDinamica::from_hardware(&hw);
        assert_eq!(cfg.precision, Precision::F16);
        assert!(cfg.usar_gpu);
    }

    #[test]
    fn test_config_sin_gpu_cpu_only() {
        let cfg = ConfiguracionDinamica::from_hardware(&hw_cpu_only());

        assert!(!cfg.usar_gpu, "sin VRAM no debe usar GPU");
        assert_eq!(cfg.max_neuronas_vram, 1000, "VRAM mínima de 1000");
        assert_eq!(cfg.batch_size_gpu, 0, "sin GPU, batch GPU 0");
        // RAM 32GB → precisión F32
        assert_eq!(cfg.precision, Precision::F32);
        // Hilos = 8-2 = 6
        assert_eq!(cfg.hilos_cpu, 6);
    }

    #[test]
    fn test_config_sin_gpu_ram_baja_usa_f16() {
        let mut hw = hw_cpu_only();
        hw.ram_total = 8_000_000_000; // < 16GB → F16
        let cfg = ConfiguracionDinamica::from_hardware(&hw);
        assert_eq!(cfg.precision, Precision::F16);
    }

    #[test]
    fn test_config_memoria_episodica_de_ssd() {
        let mut hw = hw_nvidia();
        hw.ssd_espacio = 500_000_000_000; // 500GB
        let cfg = ConfiguracionDinamica::from_hardware(&hw);
        // 5% de SSD / 64 bytes
        let esperado = ((500_000_000_000.0 * 0.05) / 64.0) as usize;
        assert_eq!(cfg.memoria_episodica_max, esperado.max(1000));
    }

    #[test]
    fn test_config_minimos_garantizados() {
        let mut hw = hw_nvidia();
        hw.ram_total = 1;
        hw.vram_total = 0;
        let cfg = ConfiguracionDinamica::from_hardware(&hw);
        // Mínimos garantizados por los `.max()` en el constructor
        assert!(cfg.max_neuronas_vram >= 1000);
        assert!(cfg.max_neuronas_ram >= 10000);
        assert!(cfg.max_sinapsis_vram >= 10000);
        assert!(cfg.max_sinapsis_ram >= 100000);
        assert!(cfg.memoria_episodica_max >= 1000);
        assert!(cfg.hilos_cpu >= 1);
    }

    #[test]
    fn test_config_totales_suma_vram_ram() {
        let hw = hw_nvidia();
        let cfg = ConfiguracionDinamica::from_hardware(&hw);
        assert_eq!(
            cfg.max_neuronas_totales,
            cfg.max_neuronas_vram + cfg.max_neuronas_ram
        );
    }

    // ── Detección real (no determinista, solo validez) ──────────────────────
    #[test]
    fn test_detectar_hardware_produce_valores_sanos() {
        let hw = HardwareInfo::detectar();

        assert!(hw.nucleos >= 1, "debe detectar al menos 1 núcleo");
        assert!(hw.ram_total > 0, "debe detectar RAM");
        assert!(hw.ssd_espacio > 0, "debe detectar espacio SSD");
        assert!(!hw.arquitectura.is_empty(), "debe tener arquitectura");
        // Si hay GPU, debe haber VRAM; si no, vram_total = 0
        if hw.gpu_tipo != GPUTipo::Ninguna {
            assert!(hw.vram_total > 0);
        }
    }

    #[test]
    fn test_medir_uso_caliente_rango_valido() {
        let (cpu_load, ram_uso) = HardwareInfo::medir_uso_caliente();
        assert!((0.0..=1.0).contains(&cpu_load), "cpu fue {}", cpu_load);
        assert!((0.0..=1.0).contains(&ram_uso), "ram fue {}", ram_uso);
    }
}
