/// Phase 42: SyscallMonitor — Userspace handler para eventos de syscalls.
///
/// Recibe eventos del ring buffer eBPF y los procesa en tiempo real.
/// Alimenta directamente al ImmuneSystem de NEXUS.
use super::events::SyscallEvent;
use log::{info, warn};

/// Lista de syscalls de alto riesgo a monitorear.
/// Basada en MITRE ATT&CK para Linux.
const HIGH_RISK_SYSCALLS: &[u32] = &[
    59,  // execve  — ejecución de proceso
    322, // execveat — ejecución alternativa
    105, // setuid  — escalada de privilegios
    117, // setresuid
    2,   // open    — apertura de archivos sensibles
    257, // openat
    56,  // clone   — creación de procesos/threads
    57,  // fork
    39,  // getpid (recon)
    102, // getuid
];

pub struct SyscallMonitor {
    alert_count: u64,
}

impl SyscallMonitor {
    pub fn new() -> Self {
        info!("🔬 [SYSCALL] Monitor inicializado. {} syscalls de alto riesgo en watchlist.", HIGH_RISK_SYSCALLS.len());
        Self { alert_count: 0 }
    }

    /// Procesa un evento de syscall recibido del ring buffer eBPF.
    pub fn process_event(&mut self, event: &SyscallEvent) {
        let process_name = std::str::from_utf8(&event.comm)
            .unwrap_or("?")
            .trim_end_matches('\0');

        if HIGH_RISK_SYSCALLS.contains(&event.syscall_nr) {
            self.alert_count += 1;

            let risk = self.classify_risk(event.syscall_nr);

            if risk >= 8 {
                warn!("🚨 [SYSCALL] ALERTA CRÍTICA: syscall={} PID={} UID={} PROCESO={}",
                    event.syscall_nr, event.pid, event.uid, process_name);
                // TODO: Feed al ImmuneSystem para respuesta inmediata
            } else {
                info!("⚠️  [SYSCALL] Syscall monitorizada: syscall={} PID={} PROCESO={}",
                    event.syscall_nr, event.pid, process_name);
            }
        }
    }

    /// Clasifica el riesgo de una syscall (0-10).
    fn classify_risk(&self, syscall_nr: u32) -> u8 {
        match syscall_nr {
            59 | 322       => 9,  // execve/execveat — muy alta
            105 | 117      => 10, // setuid — crítico (escalada de privilegios)
            56 | 57        => 7,  // clone/fork — media-alta
            2 | 257        => 5,  // open/openat — media
            _              => 3,  // otras monitoreadas
        }
    }

    pub fn alert_count(&self) -> u64 { self.alert_count }
}
