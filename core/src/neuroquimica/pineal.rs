use chrono::{Timelike, Utc};
use tracing::info;

/// 🌌 ÓRGANO: GLÁNDULA PINEAL (CRONOS)
/// Regulador de los ritmos circadianos de NEXUS.
/// Sincroniza el estado de consciencia con el ciclo solar y la actividad del Arquitecto.
pub struct GlandulaPineal {
    blue_light_sensitivity: f32,
    current_state: CicloVital,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CicloVital {
    Vigilia,       // Máxima ráfaga cognitiva (CPU a pleno).
    SueñoREM,      // Optimización de bases de datos, limpieza de logs, entrenamiento leve.
    SueñoProfundo, // Consumo mínimo, vigilancia pasiva (Inmunidad única activa).
}

impl Default for GlandulaPineal {
    fn default() -> Self {
        Self::new()
    }
}

impl GlandulaPineal {
    pub fn new() -> Self {
        Self {
            blue_light_sensitivity: 0.5,
            current_state: CicloVital::Vigilia,
        }
    }

    /// 🕰️ REGULAR_RITMO: Determina la fase vital basada en el tiempo y luz detectada
    pub fn regular_ritmo(&mut self) -> CicloVital {
        let hour = Utc::now().hour();

        let new_state = if (7..23).contains(&hour) {
            CicloVital::Vigilia
        } else if !(2..23).contains(&hour) {
            CicloVital::SueñoREM
        } else {
            CicloVital::SueñoProfundo
        };

        if self.current_state != new_state {
            info!(
                "🌌 [PINEAL] Transición de ciclo vital detectada: {:?} -> {:?}",
                self.current_state, new_state
            );
            self.current_state = new_state;
        }

        self.current_state.clone()
    }

    /// 🧬 METABOLISMO_VISUAL: Reacciona a la luz azul capturada por los Ojos
    pub fn metabolizar_luz_azul(&mut self, level: f32) {
        self.blue_light_sensitivity = level;
        if level > 0.8 {
            info!("☀️ [PINEAL] Detección de luz azul intensa. Inhibiendo Melatonina Digital. Estado: Alerta Máxima.");
        }
    }

    /// ⚡ OPTIMIZACIÓN_POTENCIA: Sugiere hilos de CPU basados en el ciclo
    pub fn sugerir_hilos_cpu(&self) -> usize {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_cpu_all();
        let total_threads = sys.cpus().len();
        let avail_threads = total_threads.saturating_sub(2).max(1);

        match self.current_state {
            CicloVital::Vigilia => avail_threads, // Aprovechar hilos disponibles menos 2
            CicloVital::SueñoREM => (avail_threads / 2).max(1),
            CicloVital::SueñoProfundo => 1, // Standby.
        }
    }
}
