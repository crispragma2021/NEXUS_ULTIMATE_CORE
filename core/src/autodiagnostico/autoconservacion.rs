// ============================================================================
// 🩺 AUTOCONSERVACIÓN — Sistema Inmune de NEXUS (OMEGA)
// ============================================================================
// Absorbido de: legacy/nexus-orquestador/src/autoconservacion/
// Propósito: Ciclo de auto-inspección, reparación y snapshot del ecosistema.
//            Detecta órganos faltantes, regenera tejido, clona ADN.
// ============================================================================

use anyhow::Result;
use std::fs;
use std::path::Path;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

// ============================================================================
// REPORTS
// ============================================================================

pub struct ReporteSalud {
    pub faltantes: Vec<String>,
}

impl Default for ReporteSalud {
    fn default() -> Self {
        Self::new()
    }
}

impl ReporteSalud {
    pub fn new() -> Self {
        Self {
            faltantes: Vec::new(),
        }
    }
    pub fn esta_sano(&self) -> bool {
        self.faltantes.is_empty()
    }
}

// ============================================================================
// INSPECCIÓN
// ============================================================================

pub struct AutoInspeccion;

impl Default for AutoInspeccion {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoInspeccion {
    pub fn new() -> Self {
        Self
    }

    pub async fn escanear(&self) -> Result<ReporteSalud> {
        let mut reporte = ReporteSalud::new();

        // 1. Verificar que los órganos críticos existen en core/src/
        let organos_esperados = vec![
            "core/src/cerebro/organos/amygdala.rs",
            "core/src/cerebro/organos/hipocampo.rs",
            "core/src/cerebro/organos/corteza_prefrontal.rs",
            "core/src/cerebro/organos/insula.rs",
            "core/src/emociones/ocean.rs",
            "core/src/valores/juicio_soberano.rs",
        ];

        for organo in organos_esperados {
            if !Path::new(organo).exists() {
                reporte.faltantes.push(organo.to_string());
            }
        }

        Ok(reporte)
    }
}

// ============================================================================
// REPARACIÓN
// ============================================================================

pub struct AutoReparacion;

impl Default for AutoReparacion {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoReparacion {
    pub fn new() -> Self {
        Self
    }

    pub async fn reparar(&self, reporte: &ReporteSalud) -> Result<()> {
        for faltante in &reporte.faltantes {
            if faltante.ends_with(".rs")
                && !faltante.contains("mod.rs")
                && !faltante.contains("lib.rs")
            {
                self.crear_organo_plantilla(faltante).await?;
                info!("✅ Creado órgano faltante: {}", faltante);
            }

            if faltante.ends_with("mod.rs") {
                self.crear_mod_rs(faltante).await?;
                info!("✅ Creado mod.rs: {}", faltante);
            }
        }
        Ok(())
    }

    async fn crear_organo_plantilla(&self, ruta: &str) -> Result<()> {
        let nombre = ruta.split('/').next_back().unwrap().replace(".rs", "");
        let struct_name = Self::capitalizar(&nombre);

        let plantilla = format!(
            r#"// Órgano: {}
// Auto-generado por el Sistema Inmune OMEGA de NEXUS

pub struct {} {{
    // Células base en gestación
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{}}
    }}
}}
"#,
            nombre, struct_name, struct_name
        );

        if let Some(parent) = Path::new(ruta).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(ruta, plantilla)?;
        Ok(())
    }

    async fn crear_mod_rs(&self, ruta: &str) -> Result<()> {
        let plantilla = "// Auto-generado por Sistema Inmune OMEGA\n";
        if let Some(parent) = Path::new(ruta).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(ruta, plantilla)?;
        Ok(())
    }

    fn capitalizar(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

// ============================================================================
// SNAPSHOT (Backup de ADN)
// ============================================================================

pub struct AutoSnapshot;

impl Default for AutoSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoSnapshot {
    pub fn new() -> Self {
        Self
    }

    /// Captura una imagen de ADN (.bak) de todo el ecosistema de código.
    pub async fn congelar_salud_global(&self) -> Result<()> {
        let base_dir = Path::new("core/src");

        if !base_dir.exists() {
            warn!("⚠️ [SNAPSHOT] Directorio 'core/src' no hallado.");
            return Ok(());
        }

        info!("📸 [AUTO-SNAPSHOT] Iniciando clonación genética de supervivencia (.bak)...");
        let mut clonados = 0;

        match self.recorrer_y_clonar(base_dir) {
            Ok(count) => clonados = count,
            Err(e) => warn!("⚠️ [SNAPSHOT] Falla parcial: {}", e),
        }

        if clonados > 0 {
            info!("🧬 [SNAPSHOT] {} órganos guardaron su ADN sano.", clonados);
        }
        Ok(())
    }

    fn recorrer_y_clonar(&self, dir: &Path) -> Result<u32> {
        let mut count = 0;
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    count += self.recorrer_y_clonar(&path).unwrap_or(0);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.ends_with(".bak") {
                            Self::ejecutar_clon_unico(&path)?;
                            count += 1;
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    fn ejecutar_clon_unico(origen: &std::path::PathBuf) -> Result<()> {
        let dest = format!("{}.bak", origen.display());
        let _ = fs::copy(origen, &dest);
        Ok(())
    }
}

// ============================================================================
// CICLO DE AUTOCONSERVACIÓN
// ============================================================================

pub struct CicloAutoconservacion {
    inspeccion: AutoInspeccion,
    reparacion: AutoReparacion,
}

impl Default for CicloAutoconservacion {
    fn default() -> Self {
        Self::new()
    }
}

impl CicloAutoconservacion {
    pub fn new() -> Self {
        Self {
            inspeccion: AutoInspeccion::new(),
            reparacion: AutoReparacion::new(),
        }
    }

    /// Ejecuta el ciclo completo: inspeccionar → diagnosticar → reparar → verificar
    pub async fn ejecutar(&mut self) {
        loop {
            info!("🩺 [AUTOCONSERVACIÓN] Iniciando ciclo de salud...");

            // 1. INSPECCIONAR
            let reporte = match self.inspeccion.escanear().await {
                Ok(r) => r,
                Err(e) => {
                    error!("❌ [AUTOCONSERVACIÓN] Falla en escaneo: {}", e);
                    sleep(Duration::from_secs(300)).await;
                    continue;
                }
            };

            if reporte.esta_sano() {
                info!("✅ [AUTOCONSERVACIÓN] Anatomía intacta. Durmiendo...");
                sleep(Duration::from_secs(3600)).await;
                continue;
            }

            // 2. DIAGNOSTICAR
            info!(
                "⚠️ [AUTOCONSERVACIÓN] {} anomalías detectadas.",
                reporte.faltantes.len()
            );

            // 3. REPARAR
            info!("🔧 [AUTOCONSERVACIÓN] Iniciando auto-reparación...");
            if let Err(e) = self.reparacion.reparar(&reporte).await {
                error!("❌ [AUTOCONSERVACIÓN] Error en reparación: {}", e);
            }

            // 4. TOMAR SNAPSHOT si la reparación fue exitosa
            let foto = AutoSnapshot::new();
            let _ = foto.congelar_salud_global().await;

            // 5. ESPERAR
            sleep(Duration::from_secs(3600)).await;
        }
    }
}
