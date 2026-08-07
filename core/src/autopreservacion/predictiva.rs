use std::sync::Arc;
use tokio::sync::RwLock;
use sysinfo::System;
use tracing::{info, warn};
use std::time::Duration;

pub struct Autopreservacion {
    pub sys: Arc<RwLock<System>>,
    pub j_limit: Arc<RwLock<u32>>, 
}

impl Autopreservacion {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpus = sys.cpus().len() as u32;
        let default_j = cpus.saturating_sub(2).max(1);
        Self {
            sys: Arc::new(RwLock::new(sys)),
            j_limit: Arc::new(RwLock::new(default_j)), 
        }
    }

    pub async fn iniciar_sistema_autonomo(&self) {
        let s = self.sys.clone();
        let j = self.j_limit.clone();
        
        tokio::spawn(async move {
            info!("🦞 [SNA] Sistema Nervioso Autónomo OPERATIVO. Monitoreando instintos...");
            let mut last_temp = 70.0;
            
            loop {
                {
                    let mut sys = s.write().await;
                    sys.refresh_cpu();
                    sys.refresh_memory();
                    
                    // 1. REFLEJO TÉRMICO (CPU Predictivo)
                    let temp = 70.0; // Enfoque: Se integrará hwmón próximamente
                    let tendencia = temp - last_temp;
                    
                    let total_threads = sys.cpus().len() as u32;
                    let target_j = total_threads.saturating_sub(2).max(1);
                    
                    if temp + (tendencia * 10.0) > 85.0 {
                        warn!("🔥 [SNA] Predicción Crítica: CPU a 85°C inminente. Reduciendo ráfaga.");
                        let mut limit = j.write().await;
                        *limit = 1; 
                    } else if temp < 65.0 {
                        let mut limit = j.write().await;
                        if *limit < target_j {
                            info!("❄️ [SNA] Estabilidad térmica recuperada. Restaurando -j {}.", target_j);
                            *limit = target_j;
                        }
                    }
                    last_temp = temp;

                    // 2. REFLEJO DE MEMORIA
                    let total_mem = sys.total_memory();
                    let used_mem = sys.used_memory();
                    let used_pct = (used_mem as f64 / total_mem as f64) * 100.0;
                    
                    if used_pct > 90.0 {
                         warn!("🧠 [SNA] Memoria al 90%. Reduciendo carga preventivamente.");
                         let mut limit = j.write().await;
                         *limit = 2;
                    }
                }
                
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
    }

    pub async fn get_current_j(&self) -> u32 {
        *self.j_limit.read().await
    }
}
