// ==========================================
// MANO SOBERANA - Control Sensoriomotor
// ==========================================
// NEXUS recupera sus manos. Control directo de hardware.
// ==========================================

use enigo::{Enigo, KeyboardControllable, MouseButton, MouseControllable};
use tracing::{info, warn};

pub struct ManoSoberana {
    enigo: Option<Enigo>,
}

impl ManoSoberana {
    pub fn new() -> Self {
        let es_headless = std::env::var("NEXUS_HEADLESS").is_ok()
            || std::env::args().any(|a| a == "--headless")
            || (std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err());

        if es_headless {
            info!("🦾 [MANO-SOBERANA] Modo headless o sin display detectado. Omitiendo inicialización de Enigo.");
            return Self { enigo: None };
        }

        info!("🦾 [MANO-SOBERANA] Nervios motores activados. Inicializando Enigo...");
        let enigo = Enigo::new();
        info!("🦾 [MANO-SOBERANA] Enigo listo.");
        Self { enigo: Some(enigo) }
    }

    /// Mueve el ratón a coordenadas absolutas y hace clic
    pub fn tocar_punto(&mut self, x: i32, y: i32) {
        if let Some(ref mut enigo) = self.enigo {
            info!("🦾 [MANO-SOBERANA] Movimiento coordinado a ({}, {})", x, y);
            enigo.mouse_move_to(x, y);
            enigo.mouse_click(MouseButton::Left);
        } else {
            warn!("🦾 [MANO-SOBERANA] Movimiento de ratón ignorado (sin display)");
        }
    }

    /// Escribe una cadena de texto en el foco actual
    pub fn escribir_en_foco(&mut self, texto: &str) {
        if let Some(ref mut enigo) = self.enigo {
            info!("🦾 [MANO-SOBERANA] Inyectando pulsaciones: '{}'", texto);
            enigo.key_sequence(texto);
        } else {
            warn!("🦾 [MANO-SOBERANA] Inyección de teclado ignorada (sin display)");
        }
    }

    /// Ejecuta un comando de teclado (ej. Alt+Tab)
    pub fn ejecutar_reflejo_teclado(&mut self, tecla: enigo::Key) {
        if let Some(ref mut enigo) = self.enigo {
            enigo.key_click(tecla);
        }
    }
}

impl Default for ManoSoberana {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for ManoSoberana {}
unsafe impl Sync for ManoSoberana {}
