// ==========================================
// CEREBELO OMEGA - Coordinación Motora + Centro de Hábitos
// ==========================================
// Fusión anatómica:
//   - Cerebellum (de brain/cerebellum.rs): Coordinación motora, flux_loop, MotorCommand
//   - Cerebelo (original migrado): Sistema de hábitos y reflejos condicionados
// ==========================================

use crate::comms::actions::KernelAction;
use crate::security_protocol::ActionGateway;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

// ─── SISTEMA DE HÁBITOS (Cerebelo original migrado) ─────────────────────

#[derive(Debug, Clone)]
pub struct Habito {
    pub patron_comando: String,
    pub accion_asociada: String,
    pub frecuencia_uso: u32,
    pub ultima_ejecucion: std::time::Instant,
}

// ─── COORDINACIÓN MOTORA (Cerebellum de brain/cerebellum.rs) ────────────

#[derive(Debug, Clone)]
pub struct MotorCommand {
    pub name: String,
    pub payload: String,
    pub priority: u8,
}

/// 🧠 Cerebelo Unificado: Coordinación Motora + Sistema de Hábitos
///
/// El cerebelo de NEXUS orquesta la ejecución fluida de acciones motoras
/// (flux_loop) mientras automatiza tareas frecuentes en hábitos.
#[derive(Clone)]
pub struct Cerebelo {
    // ─── Motor (de brain/cerebellum.rs) ────────────────────────────────
    pub thalamus: Arc<crate::cerebro::organos::talamo::Talamo>,
    /// Cola de trayectorias (acciones motoras) pendientes.
    action_queue: Arc<RwLock<VecDeque<MotorCommand>>>,

    // ─── Hábitos (original migrado) ────────────────────────────────────
    pub habitos: HashMap<String, Habito>,
}

impl Cerebelo {
    /// Crea un Cerebelo completo con coordinación motora (requiere thalamus).
    pub fn new(thalamus: Arc<crate::cerebro::organos::talamo::Talamo>) -> Self {
        info!("🧠 [CEREBELO] Iniciando. Coordinación motora + sistema de hábitos activo.");
        Self {
            thalamus,
            action_queue: Arc::new(RwLock::new(VecDeque::new())),
            habitos: HashMap::new(),
        }
    }

    /// Constructor mínimo sin motor (solo hábitos). Compatibilidad con constructor.rs existente.
    pub fn solo_habitos() -> Self {
        info!("🧠 [CEREBELO] Iniciando en modo hábitos (sin coordinación motora).");
        Self {
            thalamus: Arc::new(crate::cerebro::organos::talamo::Talamo::new()),
            action_queue: Arc::new(RwLock::new(VecDeque::new())),
            habitos: HashMap::new(),
        }
    }

    // ─── MÉTODOS DE COORDINACIÓN MOTORA (desde brain/cerebellum.rs) ────

    /// Inyectar una acción en el flujo motor.
    pub async fn enqueue(&self, command: MotorCommand) {
        let mut queue = self.action_queue.write().await;
        queue.push_back(command);
    }

    /// El Latido del Cerebelo: Procesa acciones con fluidez biológica.
    pub async fn flux_loop(&self, gateway: Arc<ActionGateway>) {
        loop {
            let mut queue = self.action_queue.write().await;

            if let Some(cmd) = queue.pop_front() {
                let state = self.thalamus.get_state();

                let wait_ms = match state {
                    crate::cerebro::organos::talamo::EstadoConsciencia::Focus => 100,
                    crate::cerebro::organos::talamo::EstadoConsciencia::Activo => 300,
                    _ => 800,
                };

                drop(queue);

                println!("🧠 [CEREBELO] Coordinando flujo para: {}", cmd.name);

                let action = KernelAction {
                    action: cmd.name,
                    payload: serde_json::from_str(&cmd.payload).unwrap_or(serde_json::json!({})),
                    risk: cmd.priority,
                };

                let _ = gateway.execute_secure(&action, None).await;

                sleep(Duration::from_millis(wait_ms)).await;
            } else {
                drop(queue);
                sleep(Duration::from_millis(500)).await;
            }
        }
    }

    // ─── MÉTODOS DE HÁBITOS (original migrado) ─────────────────────────

    /// Registra o actualiza un hábito.
    pub fn aprender_habito(&mut self, patron_comando: &str, accion_asociada: &str) {
        let now = std::time::Instant::now();
        self.habitos
            .entry(patron_comando.to_string())
            .and_modify(|h| {
                h.frecuencia_uso += 1;
                h.ultima_ejecucion = now;
                debug!(
                    "🧠 [CEREBELO] Hábito '{}' actualizado. Frecuencia: {}",
                    patron_comando, h.frecuencia_uso
                );
            })
            .or_insert_with(|| {
                info!("🧠 [CEREBELO] Nuevo hábito aprendido: '{}'", patron_comando);
                Habito {
                    patron_comando: patron_comando.to_string(),
                    accion_asociada: accion_asociada.to_string(),
                    frecuencia_uso: 1,
                    ultima_ejecucion: now,
                }
            });
    }

    /// Intenta ejecutar un hábito si el comando coincide con un patrón.
    pub fn intentar_ejecutar_habito(&mut self, comando: &str) -> Option<String> {
        let lower_comando = comando.to_lowercase();
        for (patron, habito) in self.habitos.iter_mut() {
            if lower_comando.contains(&patron.to_lowercase()) {
                info!(
                    "🧠 [CEREBELO] Hábito '{}' detectado para el comando: '{}'",
                    patron, comando
                );
                habito.frecuencia_uso += 1;
                habito.ultima_ejecucion = std::time::Instant::now();
                return Some(habito.accion_asociada.clone());
            }
        }
        None
    }
}

impl Default for Cerebelo {
    fn default() -> Self {
        // ⚠️ Usar solo si no necesitas el motor. Para uso completo, usar Cerebelo::new(thalamus).
        panic!("Cerebelo::default() no está disponible. Usa Cerebelo::new(Arc<Talamo>) para inicialización completa.");
    }
}

// ─── ALIASES DE COMPATIBILIDAD ──────────────────────────────────────────

/// Alias inglés para compatibilidad con código legacy en brain/
pub use self::Cerebelo as Cerebellum;
