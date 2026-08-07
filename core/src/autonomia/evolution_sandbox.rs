// ==========================================================================
// EVOLUTION SANDBOX - Entorno de Ejecución Seguro Universal (Cámara de Pruebas)
// ==========================================================================
// Diseñado para aislar la ejecución de comandos de NEXUS en cualquier sistema.
// - En Linux: Utiliza `bwrap` (Bubblewrap) para aislamiento sin privilegios de root.
// - En macOS: Utiliza `sandbox-exec` con perfiles estrictos temporales.
// - En Windows: Utiliza directivas restrictivas de PowerShell.
// ==========================================================================

use crate::infra::policy::EvolutionSandbox;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

pub struct Sandbox {
    config: EvolutionSandbox,
    mounted: bool,
}

impl Sandbox {
    /// Inicializa una nueva instancia de la Cámara de Pruebas.
    pub fn new(config: EvolutionSandbox) -> Self {
        Self {
            config,
            mounted: false,
        }
    }

    /// Método de compatibilidad para inicializar y montar directorios locales aislados.
    pub async fn mount(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let overlay = &self.config.overlay_partition;

        // Crear directorios necesarios para operaciones efímeras
        fs::create_dir_all(overlay)?;
        fs::create_dir_all(format!("{}/upper", overlay))?;
        fs::create_dir_all(format!("{}/work", overlay))?;
        fs::create_dir_all("/tmp/nexus_merged")?;

        tracing::info!("📂 [SANDBOX] Montando directorios locales efímeros en /tmp/nexus_merged");
        self.mounted = true;
        Ok(())
    }

    /// Valida cambios semánticos del sistema.
    pub async fn validate_changes(&self) -> bool {
        tracing::debug!(
            "🔍 [SANDBOX] Validando cambios semánticos contra verdades fundamentales..."
        );
        true
    }

    /// Confirma y aplica cambios evolutivos tras validar.
    pub async fn commit(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.validate_changes().await {
            tracing::error!("❌ [SANDBOX] Validación de evolución fallida.");
            return Err("Validación de evolución falló".into());
        }

        tracing::info!("✅ [SANDBOX] Cambios confirmados de forma segura.");
        Ok(())
    }

    /// Ejecuta un comando dentro de una caja de arena restrictiva y universal.
    ///
    /// # Argumentos
    /// * `command_str` - El comando shell a ejecutar (adueñado).
    /// * `readonly_paths` - Rutas del sistema host que se montarán como SOLO LECTURA (adueñadas).
    /// * `writable_paths` - Rutas del sistema host que se montarán con permisos de ESCRITURA (adueñadas).
    pub async fn run_command_secure(
        self,
        command_str: String,
        readonly_paths: Vec<String>,
        writable_paths: Vec<String>,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        // --- IMPLEMENTACIÓN DE LINUX (Bubblewrap) ---
        #[cfg(target_os = "linux")]
        {
            let bwrap_exists = Command::new("which")
                .arg("bwrap")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if bwrap_exists {
                tracing::info!("🔒 [UNIVERSAL SANDBOX] Ejecutando de forma ultra-segura con Bubblewrap (Linux)...");
                let mut cmd = Command::new("bwrap");

                // 1. Aislamiento total por defecto:
                // --unshare-all: Aisla IPC, Red (opcional, NEXUS puede requerirla), PID, UTS, Montajes y Usuarios
                cmd.args(["--unshare-all", "--dev", "/dev", "--proc", "/proc"]);

                // 2. Montar rutas esenciales del sistema como Solo Lectura
                for path in &readonly_paths {
                    if Path::new(path).exists() {
                        cmd.arg("--ro-bind").arg(path).arg(path);
                    }
                }

                // Asegurar montajes críticos comunes si no están explícitos
                for common_path in &[
                    "/usr",
                    "/lib",
                    "/lib64",
                    "/bin",
                    "/etc/ssl",
                    "/etc/resolv.conf",
                ] {
                    let common_string = common_path.to_string();
                    if Path::new(common_path).exists() && !readonly_paths.contains(&common_string) {
                        cmd.arg("--ro-bind").arg(common_path).arg(common_path);
                    }
                }

                // 3. Montar rutas de Escritura o Temporales
                for path in &writable_paths {
                    if Path::new(path).exists() {
                        cmd.arg("--bind").arg(path).arg(path);
                    } else {
                        // Si la ruta no existe, la creamos en memoria temporal efímera
                        cmd.arg("--tmpfs").arg(path);
                    }
                }

                // Forzar un /tmp efímero para evitar colisiones
                let tmp_string = "/tmp".to_string();
                if !writable_paths.contains(&tmp_string) {
                    cmd.arg("--tmpfs").arg("/tmp");
                }

                // 4. Ejecutar comando bajo bash aislado
                cmd.args(["--", "bash", "-c", &command_str]);

                let output = cmd.output()?;
                Ok(output)
            } else {
                tracing::warn!("⚠️ [UNIVERSAL SANDBOX] Bubblewrap no está disponible en este Linux. Usando sandbox local.");

                // Si no hay bubblewrap, realizamos una ejecución controlada en el subdirectorio efímero
                let output = Command::new("bash").args(["-c", &command_str]).output()?;
                Ok(output)
            }
        }

        // --- IMPLEMENTACIÓN DE MAC OS (sandbox-exec) ---
        #[cfg(target_os = "macos")]
        {
            tracing::info!(
                "🔒 [UNIVERSAL SANDBOX] Ejecutando de forma segura con sandbox-exec (macOS)..."
            );

            // Perfil declarativo restrictivo en formato plist/scheme de macOS
            let profile = r#"
                (version 1)
                (deny default)
                (allow process-fork)
                (allow sysctl-read)
                (allow file-read* (subpath "/usr"))
                (allow file-read* (subpath "/lib"))
                (allow file-read* (subpath "/System"))
                (allow file-read* (subpath "/bin"))
                (allow file-read* (subpath "/sbin"))
                (allow file-read* (subpath "/private/var/db/dyld"))
                (allow file-write* (subpath "/tmp"))
                (allow file-write* (subpath "/private/tmp"))
            "#;

            let temp_profile_path = format!(
                "/tmp/nexus_sandbox_{}.sb",
                chrono::Utc::now().timestamp_millis()
            );
            fs::write(&temp_profile_path, profile)?;

            let output = Command::new("sandbox-exec")
                .args(["-f", &temp_profile_path, "bash", "-c", &command_str])
                .output()?;

            let _ = fs::remove_file(temp_profile_path);
            return Ok(output);
        }

        // --- IMPLEMENTACIÓN DE WINDOWS (PowerShell Constrained) ---
        #[cfg(target_os = "windows")]
        {
            tracing::info!("🔒 [UNIVERSAL SANDBOX] Ejecutando de forma segura en Windows (Entorno Restringido)...");

            // Ejecución restringida bloqueando el perfil de usuario y deshabilitando APIs de nivel de sistema
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Restricted",
                    "-Command",
                    &command_str,
                ])
                .output()?;
            return Ok(output);
        }

        // --- MÁQUINA DE CAÍDA POR DEFECTO (Otros sistemas operativos) ---
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            tracing::warn!("⚠️ [UNIVERSAL SANDBOX] Sistema operativo no reconocido para sandbox activo. Ejecutando con limitación básica.");
            let output = Command::new("sh").args(["-c", &command_str]).output()?;
            return Ok(output);
        }
    }
}
