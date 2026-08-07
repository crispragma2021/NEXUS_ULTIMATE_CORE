// ============================================================================
// core/src/infra/ghost_vm.rs — CONTROLADOR DE MICROVM FIRECRACKER NATIVO
// ============================================================================

use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info, warn};

pub struct GhostVmController {
    socket_path: PathBuf,
    binary_path: PathBuf,
    kernel_path: PathBuf,
    rootfs_path: PathBuf,
}

impl Default for GhostVmController {
    fn default() -> Self {
        Self::new()
    }
}

impl GhostVmController {
    pub fn new() -> Self {
        let base_path = PathBuf::from("/home/soberano/NEXUS_ULTIMATE_CORE/firecracker_env");
        Self {
            socket_path: PathBuf::from("/tmp/firecracker.socket"),
            binary_path: base_path.join("firecracker"),
            kernel_path: base_path.join("vmlinux.bin"),
            rootfs_path: base_path.join("hello-rootfs.ext4"),
        }
    }

    /// Comprobar si la MicroVM o Firecracker está activo
    pub async fn is_running(&self) -> bool {
        self.socket_path.exists()
    }

    /// Detener la MicroVM de forma inmediata y limpiar recursos
    pub async fn stop(&self) -> anyhow::Result<()> {
        info!("🛑 [GHOST-VM] Deteniendo MicroVM de forma amnésica...");

        // Matar proceso firecracker
        let _ = Command::new("sudo")
            .arg("pkill")
            .arg("-9")
            .arg("firecracker")
            .status()
            .await;

        // Limpiar socket e interfaces de red
        if self.socket_path.exists() {
            let _ = tokio::fs::remove_file(&self.socket_path).await;
        }

        let _ = Command::new("sudo")
            .arg("ip")
            .arg("link")
            .arg("del")
            .arg("tap0")
            .status()
            .await;

        // Eliminar copia temporal del rootfs en RAM
        let _ = tokio::fs::remove_file("/tmp/ghost-rootfs.ext4").await;

        info!("✨ [GHOST-VM] Entorno virtualizado destruido. RAM liberada.");
        Ok(())
    }

    /// Iniciar y configurar la MicroVM desde el núcleo
    pub async fn start(&self) -> anyhow::Result<()> {
        if self.is_running().await {
            warn!("⚠️ [GHOST-VM] La MicroVM ya está activa.");
            return Ok(());
        }

        info!("🔱 [GHOST-VM] Levantando orquestación de MicroVM desde el núcleo...");

        // 1. Crear la red TAP virtual
        let tap_status = Command::new("sudo")
            .args(["ip", "tuntap", "add", "dev", "tap0", "mode", "tap"])
            .status()
            .await?;
        if !tap_status.success() {
            error!("❌ [GHOST-VM] Error al crear interfaz tap0.");
        }

        Command::new("sudo")
            .args(["ip", "addr", "add", "172.16.0.1/24", "dev", "tap0"])
            .status()
            .await?;

        Command::new("sudo")
            .args(["ip", "link", "set", "tap0", "up"])
            .status()
            .await?;

        // 2. Duplicar RootFS a /tmp (RAM) para asegurar amnesia
        let tmp_rootfs = "/tmp/ghost-rootfs.ext4";
        tokio::fs::copy(&self.rootfs_path, tmp_rootfs).await?;

        // 3. Iniciar el binario de Firecracker
        let mut child = Command::new(&self.binary_path)
            .arg("--api-sock")
            .arg(&self.socket_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Esperar a que el socket se cree
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if !self.socket_path.exists() {
            return Err(anyhow::anyhow!("Socket de Firecracker no responde"));
        }

        // 4. Configurar Kernel a través del socket UNIX con curl
        let kernel_payload = serde_json::json!({
            "kernel_image_path": self.kernel_path.to_str().unwrap(),
            "boot_args": "console=ttyS0 reboot=k panic=1 pci=off ip=172.16.0.2::172.16.0.1:255.255.255.0:hs:eth0:off"
        });
        self.call_api("boot-source", &kernel_payload).await?;

        // 5. Configurar Drive
        let drive_payload = serde_json::json!({
            "drive_id": "rootfs",
            "path_on_host": tmp_rootfs,
            "is_root_device": true,
            "is_read_only": false
        });
        self.call_api("drives/rootfs", &drive_payload).await?;

        // 6. Configurar Interfaz de red
        let net_payload = serde_json::json!({
            "iface_id": "eth0",
            "host_dev_name": "tap0"
        });
        self.call_api("network-interfaces/eth0", &net_payload)
            .await?;

        // 7. Arrancar instancia
        let start_payload = serde_json::json!({
            "action_type": "InstanceStart"
        });
        self.call_api("actions", &start_payload).await?;

        info!("🚀 [GHOST-VM] MicroVM de Firecracker inicializada y en ejecución.");

        // Spawn centinela en background para cosechar el proceso si muere
        tokio::spawn(async move {
            let _ = child.wait().await;
            info!("💀 [GHOST-VM] El proceso de la MicroVM ha terminado.");
        });

        Ok(())
    }

    /// Ejecuta un comando dentro de la MicroVM y retorna su salida
    pub async fn execute_command(&self, cmd: &str) -> anyhow::Result<String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        info!("📡 [GHOST-VM] Enviando comando a la MicroVM: '{}'...", cmd);

        // Intentar conectar al puerto 8080 del guest
        let mut stream = TcpStream::connect("172.16.0.2:8080").await?;

        // Enviar el comando seguido de exit para cerrar la shell y devolver EOF
        let payload = format!("{}\nexit\n", cmd);
        stream.write_all(payload.as_bytes()).await?;
        stream.flush().await?;

        // Leer la respuesta completa
        let mut output = String::new();
        stream.read_to_string(&mut output).await?;

        Ok(output)
    }

    /// Auxiliar para llamar a la API REST de Firecracker a través del socket UNIX
    async fn call_api(&self, endpoint: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        let url = format!("http://localhost/{}", endpoint);
        let payload_str = serde_json::to_string(payload)?;

        let output = Command::new("curl")
            .arg("--unix-socket")
            .arg(&self.socket_path)
            .arg("-X")
            .arg("PUT")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(&payload_str)
            .arg(&url)
            .output()
            .await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Error API Firecracker ({}): {}",
                endpoint,
                err
            ));
        }

        Ok(())
    }
}
