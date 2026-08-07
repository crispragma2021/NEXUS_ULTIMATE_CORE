/// Phase 42: NetworkSentinel — XDP-level network inspection.
///
/// En producción: programa eBPF XDP que inspecciona paquetes a velocidad
/// de kernel, antes de que lleguen al stack TCP/IP normal.
/// El userspace aquí maneja la lógica de política y la lista de IPs bloqueadas.
use super::events::{NetworkEvent, NetworkAction};
use std::collections::HashSet;
use std::net::Ipv4Addr;
use log::{info, warn};

pub struct NetworkSentinel {
    /// IPs bloqueadas permanentemente
    blocked_ips: HashSet<u32>,
    /// Puertos siempre bloqueados (C2, known malware ports)
    blocked_ports: HashSet<u16>,
    packet_count: u64,
    blocked_count: u64,
}

impl NetworkSentinel {
    pub fn new() -> Self {
        let mut sentinel = Self {
            blocked_ips: HashSet::new(),
            blocked_ports: HashSet::new(),
            packet_count: 0,
            blocked_count: 0,
        };

        // Puertos C2 conocidos (MITRE ATT&CK T1071)
        sentinel.block_port(4444);  // Metasploit default
        sentinel.block_port(1337);  // 'leet' C2
        sentinel.block_port(31337); // Back Orifice
        sentinel.block_port(6667);  // IRC (botnet C2)
        sentinel.block_port(9001);  // Tor default

        info!("🛡️  [NET-SENTINEL] Inicializado. {} puertos en blocklist.", sentinel.blocked_ports.len());
        sentinel
    }

    pub fn block_ip(&mut self, ip: Ipv4Addr) {
        let raw = u32::from(ip);
        self.blocked_ips.insert(raw);
        warn!("🛡️  [NET-SENTINEL] IP bloqueada: {}", ip);
    }

    pub fn block_port(&mut self, port: u16) {
        self.blocked_ports.insert(port);
    }

    /// Evalúa un evento de red y determina la acción a tomar.
    pub fn evaluate(&mut self, event: &NetworkEvent) -> NetworkAction {
        self.packet_count += 1;

        let src_ip = Ipv4Addr::from(event.src_ip);
        let dst_ip = Ipv4Addr::from(event.dst_ip);

        // IP en blocklist
        if self.blocked_ips.contains(&event.src_ip) {
            self.blocked_count += 1;
            warn!("🛡️  [NET-SENTINEL] BLOQUEADO: conexión desde IP en blocklist: {}", src_ip);
            return NetworkAction::Block;
        }

        // Puerto en blocklist
        if self.blocked_ports.contains(&event.dst_port) {
            self.blocked_count += 1;
            warn!("🛡️  [NET-SENTINEL] BLOQUEADO: puerto C2 detectado: {} → {}:{}",
                src_ip, dst_ip, event.dst_port);
            return NetworkAction::Block;
        }

        // Detección de port scanning (heurística simple: muchos dst_ports distintos)
        // En producción: usar BPF map con contadores por IP

        NetworkAction::Allow
    }

    pub fn stats(&self) -> String {
        format!("Paquetes: {} | Bloqueados: {} | IPs en blocklist: {}",
            self.packet_count, self.blocked_count, self.blocked_ips.len())
    }
}
