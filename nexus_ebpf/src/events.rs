#![allow(dead_code)]
// Phase 42: Shared event types — ring buffer bridge between eBPF kernel programs and userspace.

/// Event emitted when a suspicious syscall is intercepted.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SyscallEvent {
    pub pid: u32,
    pub uid: u32,
    pub syscall_nr: u32,
    pub comm: [u8; 16],
}

/// Event emitted when a network packet is flagged.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: u8,
    pub action: NetworkAction,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum NetworkAction {
    Allow = 0,
    Block = 1,
    Alert = 2,
}
