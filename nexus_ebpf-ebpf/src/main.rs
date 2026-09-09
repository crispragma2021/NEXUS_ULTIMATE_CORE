#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{kprobe, map, tracepoint},
    maps::HashMap,
    programs::{ProbeContext, TracePointContext},
    helpers::bpf_get_current_comm,
};
use aya_log_ebpf::info;

/// --- MAPAS DE TELEMETRÍA ---
/// Almacenan el estado que el Orquestador leerá cada 5 segundos.

#[map(name = "NEXUS_STATS")]
static mut STATS: HashMap<u32, u64> = HashMap::with_max_entries(10, 0);

const KEY_NETWORK_PULSE: u32 = 1;
const KEY_FILE_SENTINEL: u32 = 2;
const KEY_OOM_RISK: u32 = 3;

/// 1. VIGILANCIA DE ARCHIVOS (Sentinela)
/// Detecta si procesos ajenos intentan acceder al santuario de NEXUS.
#[kprobe]
pub fn trace_open(ctx: ProbeContext) -> u32 {
    match try_trace_open(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_trace_open(_ctx: ProbeContext) -> Result<u32, u32> {
    // En un sistema real, leeríamos el path del registro RDI/RSI.
    // Por ahora, detectamos la ejecución de la syscall como pulso de actividad.
    let mut val = unsafe { STATS.get(&KEY_FILE_SENTINEL).copied().unwrap_or(100) };
    
    // Si el proceso no es 'nexus', bajamos la integridad preventivamente
    let comm = bpf_get_current_comm().unwrap_or([0; 16]);
    if comm[0] != b'n' || comm[1] != b'e' {
        if val > 10 {
            val -= 1; // Erosión de integridad por acceso externo
        }
    }

    unsafe { STATS.insert(&KEY_FILE_SENTINEL, &val, 0).map_err(|_| 0u32)? };
    Ok(0)
}

/// 2. MONITOR DE RED (Pulso)
/// Captura ráfagas de paquetes para medir la actividad del sistema.
#[tracepoint(category = "net", name = "netif_receive_skb")]
pub fn monitor_network(ctx: TracePointContext) -> u32 {
    let _ = try_monitor_network(ctx);
    0
}

fn try_monitor_network(_ctx: TracePointContext) -> Result<u32, u32> {
    let mut count = unsafe { STATS.get(&KEY_NETWORK_PULSE).copied().unwrap_or(0) };
    count += 1;
    
    // Si el pulso es muy alto, el Orquestador lo interpretará como saturación
    unsafe { STATS.insert(&KEY_NETWORK_PULSE, &count, 0).map_err(|_| 0u32)? };
    Ok(0)
}

/// 3. PREMONICIÓN DE OOM (Memoria)
/// Se activa cuando el kernel entra en 'direct reclaim', señal de presión crítica.
#[tracepoint(category = "vmscan", name = "mm_vmscan_direct_reclaim_begin")]
pub fn oom_premonition(ctx: TracePointContext) -> u32 {
    let _ = try_oom_premonition(ctx);
    0
}

fn try_oom_premonition(_ctx: TracePointContext) -> Result<u32, u32> {
    let mut risk = unsafe { STATS.get(&KEY_OOM_RISK).copied().unwrap_or(0) };
    
    if risk < 100 {
        risk += 10; // Incremento rápido del riesgo de muerte por RAM
    }

    unsafe { STATS.insert(&KEY_OOM_RISK, &risk, 0).map_err(|_| 0u32)? };
    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}