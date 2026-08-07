#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeOmega {
    SysScan = 1,
    MemPrune = 2,
    SysFlush = 3,
    NetLock = 4,
    CoreLock = 5,
    SensorySnap = 6,
    NetClear = 7,
    NetThrottle = 8,
}

pub struct PulsoBinario {
    pub opcode: OpcodeOmega,
    pub payload_numerico: u64,
}

pub struct ProcesadorOmega;

impl ProcesadorOmega {
    pub fn procesar_paquete(paquete: &[u8; 16]) -> Result<PulsoBinario, &'static str> {
        // Verificar Checksum XOR (Byte 15)
        let mut checksum = 0u8;
        for i in 0..15 {
            checksum ^= paquete[i];
        }
        if checksum != paquete[15] {
            return Err("Checksum inválido en el pulso binario");
        }

        // Determinar opcode
        let opcode = match paquete[0] {
            1 => OpcodeOmega::SysScan,
            2 => OpcodeOmega::MemPrune,
            3 => OpcodeOmega::SysFlush,
            4 => OpcodeOmega::NetLock,
            5 => OpcodeOmega::CoreLock,
            6 => OpcodeOmega::SensorySnap,
            7 => OpcodeOmega::NetClear,
            8 => OpcodeOmega::NetThrottle,
            _ => return Err("Opcode no reconocido"),
        };

        // Leer payload (Bytes 1..9, little endian)
        let mut payload_bytes = [0u8; 8];
        payload_bytes.copy_from_slice(&paquete[1..9]);
        let payload_numerico = u64::from_le_bytes(payload_bytes);

        Ok(PulsoBinario {
            opcode,
            payload_numerico,
        })
    }
}
