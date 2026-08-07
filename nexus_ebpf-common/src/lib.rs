#![no_std]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SyscallEvent {
    pub pid: u32,
    pub uid: u32,
    pub syscall_nr: u32,
    pub comm: [u8; 16],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PulseMetrics {
    pub cpu_usage: u32, // Simplified for now
    pub ram_usage: u32,
    pub disk_io: u32,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for SyscallEvent {}
#[cfg(feature = "user")]
unsafe impl aya::Pod for PulseMetrics {}
