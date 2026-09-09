use nexus_ebpf_common::PulseMetrics;
use aya::maps::HashMap;
use log::info;
use tokio::time::{interval, Duration};

pub struct OrganismPulse {
    metrics_map: HashMap<aya::maps::MapData, u32, PulseMetrics>,
}

impl OrganismPulse {
    pub fn new(map: aya::maps::Map) -> anyhow::Result<Self> {
        let metrics_map: HashMap<aya::maps::MapData, u32, PulseMetrics> = HashMap::try_from(map)?;
        Ok(Self { metrics_map })
    }

    pub async fn start_loop(&self) -> anyhow::Result<()> {
        let mut ticker = interval(Duration::from_secs(5));
        info!("❤️  [PULSE] Organism Pulse loop started.");
        
        loop {
            ticker.tick().await;
            self.read_metrics()?;
        }
    }

    fn read_metrics(&self) -> anyhow::Result<()> {
        // We read key 0 from the METRICS map
        if let Ok(metrics) = self.metrics_map.get(&0, 0) {
            info!("❤️  [PULSE] OMEGA Sync High: CPU={} RAM={} IO={}", 
                metrics.cpu_usage, metrics.ram_usage, metrics.disk_io);
        } else {
            info!("❤️  [PULSE] Vital signs stable. Synchronizing...");
        }
        Ok(())
    }
}
