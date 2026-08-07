// ==========================================
// VIGILANTE - Guardián del Orquestador Principal
// ==========================================
// Este órgano monitorea la salud del Orquestador en /home/soberano/NEXUS_ULTIMATE_CORE.
// ==========================================

use crate::efectores::nexus_claw_pro::NexusClawPro;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct VigilanteDelPadre {
    nexus_path: String,
    criticos: Vec<String>,
    nexus_claw: Arc<NexusClawPro>, // El brazo ejecutor del Vigilante
}

impl VigilanteDelPadre {
    pub fn new(nexus_claw: Arc<NexusClawPro>) -> Self {
        Self {
            nexus_path: "/home/soberano/NEXUS_ULTIMATE_CORE".to_string(),
            criticos: vec![], // Vacío para evitar bucles de reinicio destructivos de nexus.service
            nexus_claw,
        }
    }

    pub async fn iniciar_vigilancia(&self) {
        info!("🛡️ Vigilante activado. Protegiendo al Orquestador.");

        loop {
            self.chequeo_vital().await;
            self.verificar_integridad_archivos().await; // Esta función ya llama a reparar_configuracion
            sleep(Duration::from_secs(60)).await; // Pulso ajustado a 60s para Resiliencia OMEGA.
        }
    }

    async fn chequeo_vital(&self) {
        // [INMUNO-OMEGA] Iniciar autocuración proactiva cada 10 ciclos
        static CICLOS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        if CICLOS
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .is_multiple_of(10)
        {
            info!("🔬 [VIGILANTE] Lanzando escaneo inmunológico proactivo...");
            let _ = self
                .nexus_claw
                .ejecutar("/home/soberano/NEXUS_ULTIMATE_CORE/scripts/auto_health.sh")
                .await;
        }

        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        for proceso in &self.criticos {
            let vivo = sys.processes().values().any(|p| {
                p.name().to_string_lossy().contains(proceso)
                    || p.exe()
                        .map(|e| e.to_string_lossy().contains(proceso))
                        .unwrap_or(false)
            });

            if !vivo {
                warn!("🚨 ¡Orquestador inactivo! Proceso {} no encontrado mediante inspección nativa.", proceso);
                self.levantar_orquestador(proceso).await;
                info!("🦾 [VIGILANTE] Notificación enviada al Arquitecto. El Vigilante permanece a la espera.");
            }
        }
    }

    async fn verificar_integridad_archivos(&self) {
        let config_path = format!("{}/shadowcrawl/cortex-scout.json", self.nexus_path);
        if !Path::new(&config_path).exists() {
            error!("🚨 ¡CORRUPCIÓN DETECTADA! Archivo de configuración desaparecido.");
            self.reparar_configuracion().await;
        }
    }

    async fn levantar_orquestador(&self, proceso: &str) {
        warn!(
            "🦾 [VIGILANTE] Se ha detectado la caída de {}. Iniciando protocolo de resurrección.",
            proceso
        );
        let restart_cmd = "sudo systemctl restart nexus.service".to_string();
        match self.nexus_claw.ejecutar(&restart_cmd).await {
            Ok(output) => info!(
                "✅ [VIGILANTE] Intento de reinicio de nexus.service: {}",
                output
            ),
            Err(e) => error!("❌ [VIGILANTE] Fallo al reiniciar nexus.service: {}", e),
        }
    }

    async fn reparar_configuracion(&self) {
        info!("🛠️ Reparando configuración del Orquestador...");
        let config_path = format!("{}/shadowcrawl/cortex-scout.json", self.nexus_path);
        let default_config_content = r#"{
            "active_scouts": [],
            "scan_interval_secs": 300,
            "last_scan_timestamp": 0
        }"#;

        match self
            .nexus_claw
            .escribir_archivo(&config_path, default_config_content)
            .await
        {
            Ok(_) => info!("✅ [VIGILANTE] cortex-scout.json restaurado a valores por defecto."),
            Err(e) => error!("❌ [VIGILANTE] Fallo al restaurar cortex-scout.json: {}", e),
        }
    }
}
