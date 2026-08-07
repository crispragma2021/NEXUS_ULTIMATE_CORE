use crate::efectores::nexus_claw_pro::NexusClawPro;
use anyhow::Result;
use std::sync::Arc;

/// 🦅 NEXUS_CLAW: Lóbulo de Infiltración y Exploración Soberana (Legacy Wrapper)
/// Redirige las llamadas al motor unificado NexusClawPro para evitar redundancias.
pub struct NexusClaw {
    claw_pro: NexusClawPro,
    user_agent: String,
}

impl NexusClaw {
    pub fn new(hippo: Option<Arc<crate::brain::hippocampus::ArtificialHippocampus>>) -> Self {
        let _ = hippo; // Conservar firma legacy sin uso
        Self {
            claw_pro: NexusClawPro::new_empty(),
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) NexusClaw/OMEGA".to_string(),
        }
    }

    pub fn new_empty() -> Self {
        Self {
            claw_pro: NexusClawPro::new_empty(),
            user_agent: "NexusClaw/OMEGA".to_string(),
        }
    }

    /// 🌐 SCOUT_WEB: Redirigido a petición HTTP sigilosa
    pub async fn scout_web(&self, url: &str) -> Result<String> {
        NexusClawPro::realizar_peticion_http(url)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// 🦾 SYSTEM_SCAVENGE: Patrullar procesos (implementación limpia)
    pub fn system_scavenge(&self) -> Vec<String> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let mut hallazgos = Vec::new();
        for (pid, process) in sys.processes() {
            let name = process.name();
            if name.to_string_lossy().contains("docker")
                || name.to_string_lossy().contains("vbox")
                || name.to_string_lossy().contains("virt")
            {
                hallazgos.push(format!("NODO INFRA: {:?} (PID: {})", name, pid));
            }
        }
        hallazgos
    }

    /// 🥷 STEALTH_MODE: Cambiar postura de sigilo
    pub fn stealth_mode(&mut self, is_stealth: bool) {
        if is_stealth {
            self.user_agent =
                "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
                    .to_string();
        } else {
            self.user_agent = "NexusClaw/OMEGA".to_string();
        }
    }

    pub async fn avisar_arquitecto(&self, msg: &str) -> Result<()> {
        tracing::warn!("🔱 [ALERTA AL ARQUITECTO] {}", msg);
        Ok(())
    }

    pub async fn archivar_en_legado(&self, path: &str, motivo: &str) -> Result<()> {
        tracing::info!("📦 [LEGADO] Archivando {} debido a: {}", path, motivo);
        Ok(())
    }

    pub async fn abrir_navegador_soberano(&self, url: &str) -> Result<()> {
        tracing::info!("🌐 [NAVEGADOR SOBERANO] Abriendo URL: {}", url);
        Ok(())
    }

    pub async fn ejecutar_inteligente(&self, cmd: &str) -> Result<String> {
        self.claw_pro.ejecutar_inteligente(cmd).await
    }

    pub async fn leer_archivo(&self, path: &str) -> Result<String> {
        NexusClawPro::leer_de_silicio(path).await
    }

    pub async fn escribir_archivo(&self, path: &str, content: &str) -> Result<()> {
        NexusClawPro::manifestar_en_silicio(path, content).await?;
        Ok(())
    }
}
