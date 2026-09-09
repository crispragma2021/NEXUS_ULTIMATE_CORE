// ==========================================
// TÁLAMO - Filtro Sensorial y Estados de Consciencia
// ==========================================
// Como el tálamo humano: retransmite solo
// la información sensorial relevante a la corteza.
// Filtra el ruido. Prioriza señales importantes.
// Gestiona los estados de consciencia de NEXUS.
// ==========================================
// IMPLEMENTACIÓN SUPERIOR desde brain/thalamus.rs
// Reemplaza la versión stub anterior de cerebro/organos/talamo.rs
// ==========================================

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 🧠 Estados de Consciencia de NEXUS
/// 0: Activo (Alta frecuencia)
/// 1: Hibernado (Baja frecuencia / Ahorro CPU)
/// 2: Emergencia (Ráfaga sensorial / Reacción inmediata)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EstadoConsciencia {
    Focus = 0,      // Alta frecuencia / Desarrollo intensivo
    Activo = 1,     // Frecuencia nominal
    Chill = 2,      // Baja frecuencia / Ahorro térmico
    Hibernado = 3,  // Inactividad profunda
    Emergencia = 4, // Reacción inmediata / Crisis
}

/// Constantes de compatibilidad para código legacy en inglés (brain/*)
/// Permite usar `ConsciousState::Active` en lugar de `EstadoConsciencia::Activo`
#[allow(non_upper_case_globals)]
impl EstadoConsciencia {
    pub const Active: EstadoConsciencia = EstadoConsciencia::Activo;
    pub const Emergency: EstadoConsciencia = EstadoConsciencia::Emergencia;
    pub const Hibernated: EstadoConsciencia = EstadoConsciencia::Hibernado;
}

impl From<u8> for EstadoConsciencia {
    fn from(value: u8) -> Self {
        match value {
            0 => EstadoConsciencia::Focus,
            2 => EstadoConsciencia::Chill,
            3 => EstadoConsciencia::Hibernado,
            4 => EstadoConsciencia::Emergencia,
            _ => EstadoConsciencia::Activo,
        }
    }
}

pub struct Talamo {
    state: Arc<AtomicU8>,
    last_stimulus: Arc<AtomicU64>,
}

impl Default for Talamo {
    fn default() -> Self {
        Self::new()
    }
}

impl Talamo {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            state: Arc::new(AtomicU8::new(EstadoConsciencia::Activo as u8)),
            last_stimulus: Arc::new(AtomicU64::new(now)),
        }
    }

    /// Obtiene el estado actual de consciencia
    pub fn estado(&self) -> EstadoConsciencia {
        EstadoConsciencia::from(self.state.load(Ordering::Relaxed))
    }

    /// Cambia el estado de consciencia
    pub fn cambiar_estado(&self, next: EstadoConsciencia) {
        let prev = self.estado();
        if prev != next {
            println!(
                "🧠 [TÁLAMO] Cambio de Consciencia: {:?} -> {:?}",
                prev, next
            );
            self.state.store(next as u8, Ordering::Relaxed);
        }
    }

    /// Helper para los sentidos: Obtiene el tiempo de sleep sugerido en milisegundos
    pub fn polling_ms(&self, base_ms: u64) -> u64 {
        match self.estado() {
            EstadoConsciencia::Focus => base_ms / 2,
            EstadoConsciencia::Activo => base_ms,
            EstadoConsciencia::Chill => base_ms * 2,
            EstadoConsciencia::Hibernado => base_ms * 30,
            EstadoConsciencia::Emergencia => base_ms / 4,
        }
    }

    pub fn handle_estado(&self) -> Arc<AtomicU8> {
        self.state.clone()
    }

    /// ⚡ Registra un estímulo sensorial para mantener el sistema despierto
    pub fn registrar_estimulo(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_stimulus.store(now, Ordering::Relaxed);

        if self.estado() == EstadoConsciencia::Hibernado {
            self.cambiar_estado(EstadoConsciencia::Activo);
        }
    }

    /// 🛡️ Verifica si el sistema debe entrar en hibernación por inactividad
    pub fn check_sentinel_status(&self, inactivity_timeout_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last = self.last_stimulus.load(Ordering::Relaxed);

        if now - last > inactivity_timeout_secs && self.estado() == EstadoConsciencia::Activo {
            println!("💤 [TÁLAMO] Modo Centinela: Iniciando Hibernación Profunda por inactividad.");
            self.cambiar_estado(EstadoConsciencia::Hibernado);
        }
    }

    /// Filtra el contexto sensorial y retorna solo lo relevante.
    pub fn filtrar_contexto(&self, realidad: &str, prioridad: &str) -> String {
        let mut filtrado = String::new();
        let palabras_clave = match prioridad {
            "LECTURA" => vec!["archivo", "leer", "src/", "Médula", "cat", "ls"],
            "EJECUCION" => vec!["comando", "ejecutar", "curl", "Médula", "sudo"],
            "CONVERSACION" => vec!["Arquitecto", "NEXUS", "identidad", "propósito"],
            _ => vec!["error", "fallo", "alerta", "crítico"],
        };

        for linea in realidad.lines() {
            for clave in &palabras_clave {
                if linea.to_lowercase().contains(&clave.to_lowercase()) {
                    filtrado.push_str(linea);
                    filtrado.push('\n');
                    break;
                }
            }
        }
        filtrado
    }
}

// ─── ALIAS DE COMPATIBILIDAD (para migración desde brain/thalamus.rs) ──────
/// Alias para compatibilidad con código que usa la nomenclatura inglesa
pub use self::EstadoConsciencia as ConsciousState;

/// Alias de compatibilidad para código que referencia el nombre inglés Thalamus
pub type Thalamus = Talamo;

// ─── MÉTODOS ALIAS EN INGLÉS (Compatibilidad con brain/reflex_arc.rs y otros) ──────
impl Talamo {
    /// get_state() → alias de estado()
    pub fn get_state(&self) -> EstadoConsciencia {
        self.estado()
    }

    /// set_state() → alias de cambiar_estado()
    pub fn set_state(&self, state: EstadoConsciencia) {
        self.cambiar_estado(state);
    }

    /// get_polling_ms() → alias de polling_ms()
    pub fn get_polling_ms(&self, base_ms: u64) -> u64 {
        self.polling_ms(base_ms)
    }

    /// get_state_handle() → alias de handle_estado()
    pub fn get_state_handle(&self) -> Arc<AtomicU8> {
        self.handle_estado()
    }

    /// register_stimulus() → alias de registrar_estimulo()
    pub fn register_stimulus(&self) {
        self.registrar_estimulo();
    }
}
