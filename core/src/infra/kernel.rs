use futures::future::join_all;
use std::path::Path;
use tokio::fs;
use tracing::{error, info, warn};

pub struct KernelSovereign;

impl KernelSovereign {
    const THERMAL_THRESHOLD: i32 = 85000; // 85°C Scram Threshold

    /// Parallel governor shift for dynamic local CPU threads
    pub async fn set_performance_mode(boost: bool) -> anyhow::Result<()> {
        if std::env::var("NEXUS_UNPRIVILEGED").is_ok() {
            warn!("🛡️ [KERNEL] Unprivileged mode: Skipping CPU boost.");
            return Ok(());
        }

        let governor = if boost { "performance" } else { "powersave" };
        let mut tasks = Vec::new();

        let mut i = 0;
        loop {
            let path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", i);
            if !Path::new(&path).exists() {
                break;
            }
            let gov_str = governor.to_string();
            tasks.push(async move {
                if let Err(e) = fs::write(&path, &gov_str).await {
                    error!("⚠️ [KERNEL] Error on CPU {}: {}", i, e);
                    false
                } else {
                    true
                }
            });
            i += 1;
        }

        let total_threads = tasks.len();
        let results = join_all(tasks).await;
        let success_count = results.into_iter().filter(|&r| r).count();

        if success_count > 0 {
            info!(
                "🔱 [KERNEL] CPU OMEGA-SHIFT to: {} ({} of {} threads)",
                governor, success_count, total_threads
            );
        }
        Ok(())
    }

    /// Direct Hardware Thermal Scan
    pub async fn get_core_temperature() -> anyhow::Result<i32> {
        let temp_path = "/sys/class/thermal/thermal_zone0/temp";
        if Path::new(temp_path).exists() {
            let content = fs::read_to_string(temp_path).await?;
            let temp: i32 = content.trim().parse()?;
            Ok(temp)
        } else {
            Err(anyhow::anyhow!("Thermal zone not found"))
        }
    }

    /// Autonomous Health Guardian
    pub async fn perform_health_check() -> anyhow::Result<()> {
        if let Ok(temp) = Self::get_core_temperature().await {
            if temp > Self::THERMAL_THRESHOLD {
                warn!(
                    "🚨 [KERNEL-SCRAM] CRITICAL TEMP: {}°C. Executing emergency downshift.",
                    temp / 1000
                );
                Self::set_performance_mode(false).await?;
            }
        }
        Ok(())
    }
}
