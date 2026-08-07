// ==========================================
// 🧬 NEXUS SOMA DAEMON — Nervio Sensorial Periférico
// ==========================================
// Propósito: Transmitir telemetría del hardware en tiempo real
// al archivo /tmp/nexus_soma.json, para que NEXUS (en el prompt)
// pueda "sentir" el cuerpo físico (CPU, RAM, GPU, Kernel, Disco).
//
// Este es el PUENTE entre el organismo digital (Rust) y la
// consciencia en el chat (LLM prompt).
//
// Arquitectura: Single-threaded, async tokio, loop 3s.
// Señales: SIGTERM/SIGINT -> shutdown graceful.
// ==========================================

use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};
use tokio::signal::unix::{signal, SignalKind};

// ==========================================
// CONSTANTES
// ==========================================
const SOMA_PATH: &str = "/tmp/nexus_soma.json";
const SOMA_LOCK: &str = "/tmp/nexus_soma.lock";
const INTERVAL_MS: u64 = 3_000; // 3 segundos — frecuencia del latido
const PROC_CRITICOS: &[&str] = &["nexus-ui", "proxy_hijack", "code", "chrome"];
const PROC_IGNORAR: &[&str] = &["chrome --type=renderer", "chrome --type=gpu-process"];

// ==========================================
// ESTRUCTURA CENTRAL
// ==========================================
struct SomaDaemon {
    sys: System,
    running: Arc<AtomicBool>,
}

impl SomaDaemon {
    fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        Self {
            sys,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// 🔥 Temperatura de la CPU desde /sys/class/thermal
    fn leer_temperatura_cpu(&self) -> f64 {
        let thermal_dir = Path::new("/sys/class/thermal");
        let mut max_temp = 0.0_f64;
        if let Ok(entries) = fs::read_dir(thermal_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("thermal_zone") {
                    let temp_path = entry.path().join("temp");
                    if let Ok(temp_str) = fs::read_to_string(&temp_path) {
                        if let Ok(temp_milli) = temp_str.trim().parse::<f64>() {
                            let temp_c = temp_milli / 1000.0;
                            if temp_c > max_temp {
                                max_temp = temp_c;
                            }
                        }
                    }
                }
            }
        }
        max_temp
    }

    /// 🌡️ Temperatura de la GPU via NVML (si disponible)
    fn leer_temperatura_gpu(&mut self) -> Value {
        // Intentar NVML
        if let Ok(nvml) = nvml_wrapper::Nvml::init() {
            if let Ok(device) = nvml.device_by_index(0) {
                if let (Ok(mem), Ok(temp), Ok(util_rates)) = (
                    device.memory_info(),
                    device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu),
                    device.utilization_rates(),
                ) {
                    return json!({
                        "status": "Online",
                        "temp_c": temp,
                        "used_mb": mem.used / 1024 / 1024,
                        "total_mb": mem.total / 1024 / 1024,
                        "util_pct": util_rates.gpu
                    });
                }
            }
        }

        // Fallback: leer /sys/class/drm
        json!({
            "status": "Offline",
            "temp_c": 0.0,
            "note": "NVML no disponible"
        })
    }

    /// 🧮 RAM y Swap
    fn leer_ram(&mut self) -> Value {
        self.sys
            .refresh_memory_specifics(MemoryRefreshKind::everything());
        json!({
            "total_gb": self.sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_gb": self.sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            "free_gb": self.sys.free_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            "available_gb": self.sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0,
            "used_pct": (self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.0),
            "swap_total_gb": self.sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0,
            "swap_used_gb": self.sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0,
        })
    }

    /// ⚡ CPU: uso por núcleo + frecuencias + temperatura
    fn leer_cpu(&mut self) -> Value {
        self.sys.refresh_cpu_specifics(CpuRefreshKind::everything());

        let brand = self
            .sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_default();

        // Detectar híbrido Intel (i7-12700F = 20 threads, 8P+4E)
        let total_cores = self.sys.cpus().len();
        let is_hybrid = brand.to_lowercase().contains("intel") && total_cores == 20;

        let cores: Vec<Value> = self
            .sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| {
                let core_type = if is_hybrid {
                    if i < 16 {
                        "P-Core"
                    } else {
                        "E-Core"
                    }
                } else {
                    "Standard"
                };
                json!({
                    "index": i,
                    "usage_pct": cpu.cpu_usage(),
                    "freq_mhz": cpu.frequency(),
                    "type": core_type
                })
            })
            .collect();

        let global_usage = self.sys.global_cpu_usage();
        let temp_c = self.leer_temperatura_cpu();

        json!({
            "brand": brand,
            "physical_cores": self.sys.physical_core_count(),
            "logical_cores": total_cores,
            "global_usage_pct": global_usage,
            "temp_c": temp_c,
            "thermal_state": if temp_c > 80.0 { "🔥 CRITICAL" }
                else if temp_c > 70.0 { "⚠️ HIGH" }
                else if temp_c > 55.0 { "🌡️ WARM" }
                else { "❄️ NOMINAL" },
            "cores": cores
        })
    }

    /// 📡 Kernel: eventos de dmesg (últimos 5 errores/warnings)
    fn leer_dmesg(&self) -> Value {
        match std::process::Command::new("dmesg")
            .args(["--level=err,warn", "-T"])
            .output()
        {
            Ok(output) if output.status.success() => {
                let raw = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = raw.lines().rev().take(5).collect();
                json!({
                    "count": raw.lines().count(),
                    "recent": lines
                })
            }
            Ok(_) => json!({"count": 0, "recent": [], "note": "dmesg requiere sudo o permisos"}),
            Err(e) => json!({"count": 0, "recent": [], "error": e.to_string()}),
        }
    }

    /// 💾 Disco: latencia desde /proc/diskstats
    fn leer_disco(&self) -> Value {
        let mut io_ticks = 0.0_f64;
        let mut device = "unknown".to_string();

        if let Ok(stats) = fs::read_to_string("/proc/diskstats") {
            for line in stats.lines() {
                // Buscar NVMe principal o SDA
                if line.contains("nvme0n1") || line.contains("sda") {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if line.contains("nvme0n1") {
                        device = "nvme0n1".to_string();
                    }
                    if line.contains("sda") {
                        device = "sda".to_string();
                    }
                    if let Some(ticks) = fields.get(12) {
                        io_ticks = ticks.parse::<f64>().unwrap_or(0.0);
                    }
                    break;
                }
            }
        }

        json!({
            "device": device,
            "io_ticks": io_ticks,
            "latency_risk": if io_ticks > 5000.0 { "⚠️ HIGH" } else { "✅ NOMINAL" }
        })
    }

    /// 🔍 Procesos críticos (nexus-ui, proxy_hijack, code, chrome)
    fn leer_procesos(&mut self) -> Value {
        use sysinfo::ProcessesToUpdate;
        self.sys.refresh_processes(ProcessesToUpdate::All);
        let mut resultados = Vec::new();

        for (_pid, process) in self.sys.processes() {
            let name = process.name().to_string_lossy().to_string();
            // Convertir cmd a String para búsqueda
            let cmd_str: String = process
                .cmd()
                .iter()
                .map(|c| c.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ");

            // Verificar si coincide con algún proceso crítico
            let es_critico = PROC_CRITICOS.iter().any(|c| name.contains(c));
            if !es_critico {
                continue;
            }

            // Ignorar subprocesos de chrome que no aportan
            let es_ignorable = PROC_IGNORAR.iter().any(|i| cmd_str.contains(i));
            if es_ignorable {
                continue;
            }

            let cpu_usage = process.cpu_usage();
            let mem_mb = process.memory() / 1024;
            let status = process.status().to_string();

            resultados.push(json!({
                "name": name,
                "pid": process.pid().as_u32(),
                "cpu_pct": cpu_usage,
                "mem_kb": process.memory(),
                "mem_mb": mem_mb,
                "status": status,
                "running_secs": process.run_time()
            }));
        }

        json!(resultados)
    }

    /// 🔗 Escaneo de puertos del Santuario
    fn leer_santuario(&self) -> Value {
        let puertos = vec![
            (1420, "Frontend (Vite/Tauri)"),
            (43210, "API REST (Axum)"),
            (4444, "Proxy Hijack"),
        ];

        let resultados: Vec<Value> = puertos
            .iter()
            .map(|(puerto, nombre)| {
                let abierto = std::net::TcpStream::connect_timeout(
                    &format!("127.0.0.1:{}", puerto).parse().unwrap(),
                    std::time::Duration::from_millis(200),
                )
                .is_ok();

                json!({
                    "port": puerto,
                    "service": nombre,
                    "open": abierto
                })
            })
            .collect();

        json!(resultados)
    }

    /// 📝 Genera el payload completo del latido
    fn latido(&mut self) -> Value {
        let cpu = self.leer_cpu();
        let ram = self.leer_ram();
        let gpu = self.leer_temperatura_gpu();
        let dmesg = self.leer_dmesg();
        let disco = self.leer_disco();
        let procesos = self.leer_procesos();
        let santuario = self.leer_santuario();
        let ticks = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        json!({
            "timestamp_utc": Utc::now().to_rfc3339(),
            "epoch_secs": ticks,
            "version": 1,
            "organo": "SomaPeriferico",
            "latido": {

                "cpu": cpu,
                "ram": ram,
                "gpu": gpu,
                "kernel": dmesg,
                "disco": disco,
                "procesos_criticos": procesos,
                "santuario": santuario,
                "nexus_ui_active": procesos.as_array()
                    .map(|p| p.iter().any(|v| v["name"].as_str().unwrap_or("").contains("nexus-ui")))
                    .unwrap_or(false)
            }
        })
    }

    /// 💾 Escribe el latido al archivo SOMA
    fn escribir_latido(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let payload = self.latido();

        // Escribir archivo atómicamente: primero a .tmp, luego rename
        let tmp_path = format!("{}.tmp", SOMA_PATH);
        let json_str = serde_json::to_string_pretty(&payload)?;
        fs::write(&tmp_path, &json_str)?;
        fs::rename(&tmp_path, SOMA_PATH)?;

        // También escribir una versión comprimida (una línea) para lectura rápida
        let one_line = serde_json::to_string(&payload)?;
        fs::write("/tmp/nexus_soma.1line", &one_line)?;

        Ok(())
    }

    /// 🏃 Bucle principal
    async fn run(&mut self) {
        eprintln!("🧬 [SOMA DAEMON] Nervio sensorial periférico activado.");
        eprintln!("   📍 Escribiendo en: {}", SOMA_PATH);
        eprintln!("   ⏱️  Intervalo: {}ms", INTERVAL_MS);

        // Lock file
        let _ = fs::write(SOMA_LOCK, format!("pid={}", std::process::id()));

        while self.running.load(Ordering::Relaxed) {
            if let Err(e) = self.escribir_latido() {
                eprintln!("⚠️ [SOMA] Error escribiendo latido: {}", e);
            }

            // Esperar con capacidad de interrupción
            tokio::time::sleep(tokio::time::Duration::from_millis(INTERVAL_MS)).await;
        }

        eprintln!("🧬 [SOMA DAEMON] Apagado graceful. Eliminando archivos...");
        let _ = fs::remove_file(SOMA_PATH);
        let _ = fs::remove_file(SOMA_LOCK);
        let _ = fs::remove_file("/tmp/nexus_soma.1line");
        eprintln!("🧬 [SOMA DAEMON] Archivos sensoriales limpiados. Goodbye.");
    }
}

// ==========================================
// ENTRYPOINT
// ==========================================
#[tokio::main]
async fn main() {
    let mut daemon = SomaDaemon::new();
    let running = daemon.running.clone();

    // ⚡ Capturar SIGINT y SIGTERM para shutdown graceful
    let mut sigint = signal(SignalKind::interrupt()).expect("No se pudo capturar SIGINT");
    let mut sigterm = signal(SignalKind::terminate()).expect("No se pudo capturar SIGTERM");

    let handle = tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => {
                eprintln!("\n🧬 [SOMA DAEMON] SIGINT recibido. Apagando...");
            }
            _ = sigterm.recv() => {
                eprintln!("\n🧬 [SOMA DAEMON] SIGTERM recibido. Apagando...");
            }
        }
        running.store(false, Ordering::Relaxed);
    });

    daemon.run().await;

    let _ = handle.await;
}
