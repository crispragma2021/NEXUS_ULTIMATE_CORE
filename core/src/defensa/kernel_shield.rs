// ==========================================
// KERNEL SHIELD 6.9+ - Protección BPF REAL
// ==========================================

use aya::programs::Lsm;
use aya::{Btf, Ebpf, EbpfLoader};
use std::fs;
use std::process::Command;
use tracing::{error, info};

pub struct KernelShield {
    bpf: Option<Ebpf>,
    programs_loaded: bool,
}

impl Default for KernelShield {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelShield {
    pub fn new() -> Self {
        Self {
            bpf: None,
            programs_loaded: false,
        }
    }

    pub async fn activar_proteccion_memoria(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🛡️ Activando BPF Arenas para protección multi-tenant");

        // 1. Verificar kernel 6.9+
        let kernel = self.verificar_kernel().await?;
        let is_supported = || -> Option<bool> {
            let parts: Vec<&str> = kernel.split('.').collect();
            let major: u32 = parts.first()?.parse().ok()?;
            let minor: u32 = parts.get(1)?.parse().ok()?;
            Some(major > 6 || (major == 6 && minor >= 9))
        }()
        .unwrap_or(false);

        if !is_supported {
            error!("Kernel {} no soporta BPF Arenas (requiere 6.9+)", kernel);
            return Err("Kernel too old".into());
        }

        // 2. Cargar programa eBPF precompilado
        let bpf_bytes = fs::read("target/bpf/programs/nexus_shield.o")?;
        let mut bpf = EbpfLoader::new().load(&bpf_bytes)?;

        // 3. Obtener y adjuntar programa LSM
        let btf = Btf::from_sys_fs()?;
        let program: &mut Lsm = bpf
            .program_mut("nexus_mem_protect")
            .ok_or("Program 'nexus_mem_protect' not found")?
            .try_into()?;

        program.load("mm_alloc", &btf)?;
        program.attach()?;

        self.bpf = Some(bpf);
        self.programs_loaded = true;

        info!("✅ BPF Shield activado correctamente");
        Ok(())
    }

    async fn verificar_kernel(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("uname").arg("-r").output()?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
