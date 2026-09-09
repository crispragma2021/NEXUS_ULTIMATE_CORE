use serde::{Deserialize, Serialize};
use std::fs;
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareProfile {
    pub cpu_brand: String,
    pub logical_cores: usize,
    pub physical_cores: Option<usize>,
    pub total_memory_gb: f64,
    pub has_gpu: bool,
    pub os_name: String,
    pub timestamp: String,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let logical_cores = sys.cpus().len();
        let physical_cores = sys.physical_core_count();
        let total_memory_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

        // Detección simple de GPU (buscando drivers comunes)
        let has_gpu =
            fs::metadata("/dev/nvidia0").is_ok() || fs::metadata("/dev/dri/renderD128").is_ok();

        let os_name = System::name().unwrap_or_else(|| "Linux".to_string());

        Self {
            cpu_brand,
            logical_cores,
            physical_cores,
            total_memory_gb,
            has_gpu,
            os_name,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn save_temp(&self) -> anyhow::Result<()> {
        let path = "/tmp/nexus_hardware_profile.json";
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_temp() -> anyhow::Result<Self> {
        let path = "/tmp/nexus_hardware_profile.json";
        let json = fs::read_to_string(path)?;
        let profile = serde_json::from_str(&json)?;
        Ok(profile)
    }
}
