use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Mutex;
use tracing::{info, warn};

/// SharedRingBuffer: Un canal de comunicación de ultra-baja latencia para pulsos binarios.
/// Implementación inicial con Mutex para simplificar, con objetivo de Lock-Free en el futuro.
pub struct SharedRingBuffer {
    buffer: Mutex<VecDeque<[u8; 16]>>,
    capacity: usize,
}

impl SharedRingBuffer {
    pub fn new(capacity: usize) -> Self {
        info!(
            "🌌 [OMEGA-WIRE] Inicializando Ring Buffer con capacidad: {}",
            capacity
        );
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Empuja un pulso binario al buffer. Si está lleno, el pulso más antiguo se descarta.
    pub fn push(&self, pulse: [u8; 16]) -> Result<()> {
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| anyhow::anyhow!("Fallo al bloquear Ring Buffer: {}", e))?;
        if buffer.len() == self.capacity {
            let _ = buffer.pop_front(); // Descartar el más antiguo para hacer espacio
            warn!("⚠️ [OMEGA-WIRE] Ring Buffer lleno. Descartando pulso antiguo.");
        }
        buffer.push_back(pulse);
        Ok(())
    }

    /// Extrae el pulso binario más antiguo del buffer.
    pub fn pop(&self) -> Result<Option<[u8; 16]>> {
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| anyhow::anyhow!("Fallo al bloquear Ring Buffer: {}", e))?;
        Ok(buffer.pop_front())
    }
}
