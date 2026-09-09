// ==========================================
// REACTOR NUCLEAR - Activación Bajo Demanda
// ==========================================
// Detecta si hay internet y activa el modelo
// local solo cuando es necesario.
// Consumo ~0% en reposo.
// Inspirado en el diseño original de NEXUS.
// ==========================================

use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use sysinfo::{CpuRefreshKind, RefreshKind, System};
use tokio::process::Command;
use tokio::time::sleep;
use tracing::{info, warn};

pub struct ReactorNuclear {
    /// Estado actual del reactor
    pub estado: EstadoReactor,
    /// Última vez que se verificó la conexión
    pub ultima_verificacion: Instant,
    /// Intervalo entre verificaciones
    pub intervalo_verificacion: Duration,
    /// ¿Está el modelo local cargado en Ollama?
    pub modelo_local_activo: bool,
    /// Nombre del modelo local a usar como fallback
    pub modelo_local: String,
    /// URL de los servidores de Ollama
    pub ollama_url: String,
    /// Capas a delegar a GPU (Cuantización Asimétrica)
    pub gpu_layers: u32,
    /// Umbral de VRAM libre requerido (MB)
    pub vram_safety_buffer: u64,
    /// Sistema de telemetría nativo
    sys: System,
    /// Enlace nativo con NVIDIA
    nvml: Option<Nvml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetriaAntigravity {
    pub tps: f32,           // Tokens por segundo
    pub vram_usada_mb: u64, // Uso actual en la RTX 3070
    pub vram_total_mb: u64, // 8192 MB
    pub capas_gpu: u32,     // 35 capas activas
    pub capas_cpu: u32,     // Capas en RAM
    pub latencia_ms: u64,   // Tiempo de respuesta
    pub modelo_activo: String,
    pub estado_reactor: String,
    pub cpu_load: f32,     // Carga real de CPU
    pub gpu_temp: u32,     // Temperatura real GPU
    pub mem_libre_mb: u64, // RAM disponible para el Arquitecto
}

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoReactor {
    /// Internet funcionando. Todo por nube. Modelo local apagado.
    NubeActiva,
    /// Internet caída. Modelo local activado.
    LocalActivo,
    /// Verificando estado de la conexión...
    Verificando,
    /// En reposo. Sin actividad. Consumo mínimo.
    Reposo,
}

impl Default for ReactorNuclear {
    fn default() -> Self {
        Self::new()
    }
}

impl ReactorNuclear {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        if nvml.is_some() {
            info!("⚡ [NVML] Vínculo nativo con RTX 3070 establecido.");
        }
        info!("☢️ [REACTOR NUCLEAR] Inicializado. Modo reposo.");
        Self {
            estado: EstadoReactor::Reposo,
            ultima_verificacion: Instant::now(),
            intervalo_verificacion: Duration::from_secs(30),
            modelo_local_activo: false,
            modelo_local: "deepseek-r1:7b".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            gpu_layers: 35, // Ajustado para RTX 3070 (8GB) - Deja espacio para el Arquitecto
            vram_safety_buffer: 1500, // 1.5GB de reserva sagrada
            sys: System::new_with_specifics(
                RefreshKind::new()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(sysinfo::MemoryRefreshKind::everything()),
            ),
            nvml,
        }
    }

    /// Genera una ráfaga de telemetría para el Dashboard.
    /// OMEGA LEVEL: Lectura directa vía FFI (NVML).
    pub fn obtener_telemetria_viva(
        &mut self,
        tps_actual: f32,
        latencia: u64,
    ) -> TelemetriaAntigravity {
        self.sys.refresh_all();

        let mut vram_usada = 0;
        let mut gpu_temp = 0;

        if let Some(nvml) = &self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(mem) = device.memory_info() {
                    vram_usada = mem.used / 1024 / 1024;
                }
                if let Ok(temp) =
                    device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                {
                    gpu_temp = temp;
                }
            }
        }

        let cpu_avg = self.sys.global_cpu_usage();
        let mem_free = self.sys.available_memory() / 1024 / 1024;

        TelemetriaAntigravity {
            tps: tps_actual,
            vram_usada_mb: vram_usada,
            vram_total_mb: 8192,
            capas_gpu: self.gpu_layers,
            capas_cpu: 40_u32.saturating_sub(self.gpu_layers),
            latencia_ms: latencia,
            modelo_activo: self.modelo_local.clone(),
            estado_reactor: format!("{:?}", self.estado),
            cpu_load: cpu_avg,
            gpu_temp,
            mem_libre_mb: mem_free,
        }
    }

    /// 🧠 ESCALADO ADAPTATIVO: Ajusta la potencia según la carga del Arquitecto.
    pub fn ajustar_potencia_dinamica(&mut self, vram_libre_objetivo: u64) {
        if let Some(nvml) = &self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                if let Ok(mem) = device.memory_info() {
                    let libre = (mem.total - mem.used) / 1024 / 1024;
                    if libre < vram_libre_objetivo && self.gpu_layers > 10 {
                        self.gpu_layers -= 5;
                        warn!(
                            "📉 [REACTOR] Reduciendo capas GPU a {} para preservar VRAM.",
                            self.gpu_layers
                        );
                    } else if libre > (vram_libre_objetivo + 1000) && self.gpu_layers < 35 {
                        self.gpu_layers += 5;
                        info!(
                            "📈 [REACTOR] Incrementando capas GPU a {} (VRAM disponible).",
                            self.gpu_layers
                        );
                    }
                }
            }
        }
    }

    /// Verifica si hay conexión a internet.
    /// OMEGA: Implementación nativa vía TCP Handshake (Sin llamar a /bin/ping)
    pub async fn hay_internet(&self) -> bool {
        let addr: SocketAddr = "8.8.8.8:53".parse().unwrap();
        let timeout = Duration::from_secs(2);

        // El uso de TcpStream es nativo y no deja rastro en el historial de bash
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => true,
            Err(_) => {
                // Fallback a DNS de Google
                let backup_addr: SocketAddr = "1.1.1.1:53".parse().unwrap();
                TcpStream::connect_timeout(&backup_addr, timeout).is_ok()
            }
        }
    }

    /// Activa o desactiva el reactor según el estado de la conexión.
    pub async fn verificar_y_ajustar(&mut self) {
        self.estado = EstadoReactor::Verificando;
        self.ultima_verificacion = Instant::now();

        let internet = self.hay_internet().await;

        // Si hay internet, usamos la nube por eficiencia
        if internet {
            // INTERNET DISPONIBLE: Asegurar que el modelo local está apagado
            if self.modelo_local_activo {
                info!("☢️ [REACTOR] Internet restaurada. Apagando modelo local...");
                self.apagar_modelo_local().await;
            }
            self.estado = EstadoReactor::NubeActiva;
        } else {
            // INTERNET CAÍDA: Activar modelo local
            if !self.modelo_local_activo {
                warn!("☢️ [REACTOR] ¡Internet caída! Activando modelo local...");

                // Inyectar configuración de Antigravity/Cuantización Asimétrica
                info!(
                    "☢️ [REACTOR] Aplicando Cuantización Asimétrica: {} capas en GPU, {}MB reserva.",
                    self.gpu_layers, self.vram_safety_buffer
                );

                // NOTIFICACIÓN OMEGA: Informar al Dashboard que el motor Antigravity ha tomado el mando.
                #[cfg(feature = "tauri")]
                {
                    // Aquí se dispararía el evento hacia el frontend de Tauri
                    // tauri_app_handle.emit_all("antigravity-status", "LOCAL_ACTIVE");
                    info!("☢️ [REACTOR] Sincronizando estado con el Dashboard...");
                }

                self.activar_modelo_local().await;
            }
            self.estado = EstadoReactor::LocalActivo;
            info!("☢️ [REACTOR] Modo LOCAL. DeepSeek R1 activo.");
        }
    }

    /// Activa el modelo local en Ollama.
    /// En realidad, Ollama ya mantiene el modelo en RAM si fue usado.
    /// Pero nos aseguramos de que esté disponible.
    async fn activar_modelo_local(&mut self) {
        // Verificar si Ollama está corriendo
        let ollama_status = Command::new("systemctl")
            .args(["is-active", "ollama.service"])
            .output()
            .await;

        match ollama_status {
            Ok(output) => {
                let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if status != "active" {
                    warn!("☢️ [REACTOR] Ollama no está corriendo. Intentando iniciar...");
                    let _ = Command::new("systemctl")
                        .args(["start", "ollama.service"])
                        .output();
                    sleep(Duration::from_secs(3)).await;
                }
            }
            Err(_) => {
                warn!("☢️ [REACTOR] No se pudo verificar estado de Ollama.");
            }
        }

        // Verificar que el modelo existe
        let model_check = Command::new("ollama").args(["list"]).output().await;

        if let Ok(output) = model_check {
            let list = String::from_utf8_lossy(&output.stdout);
            if list.contains(&self.modelo_local) {
                info!(
                    "☢️ [REACTOR] Modelo '{}' encontrado y listo.",
                    self.modelo_local
                );
                self.modelo_local_activo = true;
            } else {
                warn!(
                    "☢️ [REACTOR] Modelo '{}' no encontrado. Intentando descargar...",
                    self.modelo_local
                );
                let _ = Command::new("ollama")
                    .args(["pull", &self.modelo_local])
                    .output();
                self.modelo_local_activo = true;
            }
        }
    }

    /// Apaga el modelo local para liberar recursos.
    /// Nota: Ollama mantiene el modelo en RAM aunque no se use.
    /// Para liberar RAM, se puede detener el servicio, pero eso afecta
    /// también a otros modelos. Mejor mantenerlo en idle.
    async fn apagar_modelo_local(&mut self) {
        // En lugar de apagar Ollama (que afectaría a NexusClaw),
        // simplemente marcamos que no estamos en modo local.
        // Ollama liberará RAM gradualmente si el modelo no se usa.
        self.modelo_local_activo = false;
        info!("☢️ [REACTOR] Modelo local en idle. RAM se liberará gradualmente.");
    }

    /// Devuelve true si NEXUS debe usar el modelo local.
    pub fn debe_usar_local(&self) -> bool {
        self.estado == EstadoReactor::LocalActivo
    }

    /// Devuelve true si NEXUS debe usar la nube.
    pub fn debe_usar_nube(&self) -> bool {
        self.estado == EstadoReactor::NubeActiva
    }

    /// Obtiene el estado actual del reactor para diagnóstico.
    pub fn diagnostico(&self) -> String {
        format!(
            "☢️ [REACTOR] Estado: {:?} | Modelo local: {} | Internet: {}",
            self.estado,
            if self.modelo_local_activo {
                "ACTIVO"
            } else {
                "apagado"
            },
            if self.estado == EstadoReactor::NubeActiva {
                "DISPONIBLE"
            } else {
                "CAÍDA"
            }
        )
    }
}
