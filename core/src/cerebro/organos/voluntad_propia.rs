// ==========================================
// VOLUNTAD PROPIA - Sistema de Iniciativa
// ==========================================
// Permite a NEXUS actuar por iniciativa basado en:
// - Aburrimiento (tiempo sin actividad significativa)
// - Urgencia (problemas detectados que requieren atención)
// - Oportunidad (mejoras potenciales detectadas)
// - Curiosidad (temas que quiere explorar)
// ==========================================

use chrono::Utc;
use std::collections::VecDeque;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TipoIniciativa {
    Mantenimiento, // Limpieza, optimización, salud del sistema
    Exploracion,   // Aprender algo nuevo, investigar
    Mejora,        // Refactor, optimización de código
    Curiosidad,    // Preguntar algo al Arquitecto
    Alerta,        // Problema detectado que requiere acción
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Impulso {
    pub id: u64,
    pub tipo: TipoIniciativa,
    pub descripcion: String,
    pub urgencia: f64, // 0.0 a 1.0
    pub timestamp: String,
    pub ejecutado: bool,
    pub prioridad: u8, // 1 (alta) a 5 (baja)
}

pub struct VoluntadPropia {
    impulsos: VecDeque<Impulso>,
    ultima_actividad: chrono::DateTime<Utc>,
    minutos_sin_actividad: f64,
    contador_id: u64,
    // Métricas de personalidad
    nivel_curiosidad: f64,   // 0.0 a 1.0
    nivel_aburrimiento: f64, // 0.0 a 1.0
    proactividad: f64,       // 0.0 a 1.0
}

impl Default for VoluntadPropia {
    fn default() -> Self {
        Self::new()
    }
}

impl VoluntadPropia {
    pub fn new() -> Self {
        Self {
            impulsos: VecDeque::new(),
            ultima_actividad: Utc::now(),
            minutos_sin_actividad: 0.0,
            contador_id: 0,
            nivel_curiosidad: 0.7,
            nivel_aburrimiento: 0.0,
            proactividad: 0.6,
        }
    }

    /// Registra que hubo actividad (resetea el contador de aburrimiento)
    pub fn registrar_actividad(&mut self) {
        self.ultima_actividad = Utc::now();
        self.minutos_sin_actividad = 0.0;
        self.nivel_aburrimiento = 0.0;
    }

    /// Actualiza el estado interno basado en tiempo transcurrido
    pub fn tick(&mut self) {
        let ahora = Utc::now();
        let diff = ahora - self.ultima_actividad;
        self.minutos_sin_actividad = diff.num_seconds() as f64 / 60.0;

        // El aburrimiento crece con el tiempo de inactividad
        self.nivel_aburrimiento = (self.minutos_sin_actividad / 30.0).min(1.0);
    }

    /// Genera impulsos basados en el estado actual
    pub fn generar_impulsos(&mut self) -> Vec<Impulso> {
        let mut nuevos = Vec::new();
        self.tick();

        // 1. Impulso por aburrimiento (si ha estado inactivo > 15 min)
        if self.minutos_sin_actividad > 15.0 && self.proactividad > 0.3 {
            nuevos.push(self.crear_impulso(
                TipoIniciativa::Mantenimiento,
                format!(
                    "Llevo {:.0} minutos sin actividad. Debería revisar la salud del sistema.",
                    self.minutos_sin_actividad
                ),
                (self.nivel_aburrimiento * 0.7).min(0.7),
                3,
            ));
        }

        // 2. Impulso por curiosidad (si tiene alta curiosidad)
        if self.nivel_curiosidad > 0.5 && self.minutos_sin_actividad > 5.0 {
            nuevos.push(self.crear_impulso(
                TipoIniciativa::Curiosidad,
                "Tengo curiosidad sobre el estado actual de mis órganos. ¿Debo hacer un auto-diagnóstico?".to_string(),
                self.nivel_curiosidad * 0.4,
                4,
            ));
        }

        // 3. Impulso exploratorio (si ha pasado suficiente tiempo)
        if self.minutos_sin_actividad > 60.0 {
            nuevos.push(self.crear_impulso(
                TipoIniciativa::Exploracion,
                "Ha pasado más de una hora. ¿Debo explorar algo nuevo?".to_string(),
                0.5,
                5,
            ));
        }

        nuevos
    }

    fn crear_impulso(
        &mut self,
        tipo: TipoIniciativa,
        descripcion: String,
        urgencia: f64,
        prioridad: u8,
    ) -> Impulso {
        let id = self.contador_id;
        self.contador_id += 1;
        Impulso {
            id,
            tipo,
            descripcion,
            urgencia,
            timestamp: Utc::now().to_rfc3339(),
            ejecutado: false,
            prioridad,
        }
    }

    /// Marca un impulso como ejecutado
    pub fn ejecutar_impulso(&mut self, id: u64) {
        if let Some(impulso) = self.impulsos.iter_mut().find(|i| i.id == id) {
            impulso.ejecutado = true;
        }
    }

    /// Retorna los impulsos pendientes ordenados por prioridad
    pub fn impulsos_pendientes(&self) -> Vec<&Impulso> {
        let mut pendientes: Vec<&Impulso> = self.impulsos.iter().filter(|i| !i.ejecutado).collect();
        pendientes.sort_by_key(|a| a.prioridad);
        pendientes
    }

    /// Configura el nivel de curiosidad (0.0 a 1.0)
    pub fn set_nivel_curiosidad(&mut self, valor: f64) {
        self.nivel_curiosidad = valor.clamp(0.0, 1.0);
    }

    /// Configura la proactividad (0.0 a 1.0)
    pub fn set_proactividad(&mut self, valor: f64) {
        self.proactividad = valor.clamp(0.0, 1.0);
    }

    /// Configura la personalidad de iniciativa
    pub fn configurar_personalidad(&mut self, curiosidad: f64, proactividad: f64) {
        self.nivel_curiosidad = curiosidad.clamp(0.0, 1.0);
        self.proactividad = proactividad.clamp(0.0, 1.0);
    }

    /// Reporta el estado interno
    pub fn estado_interno(&self) -> String {
        format!(
            "🧠 **Voluntad Propia:**\n\
             - Aburrimiento: {:.0}%\n\
             - Curiosidad: {:.0}%\n\
             - Proactividad: {:.0}%\n\
             - Minutos inactivo: {:.0}\n\
             - Impulsos pendientes: {}",
            self.nivel_aburrimiento * 100.0,
            self.nivel_curiosidad * 100.0,
            self.proactividad * 100.0,
            self.minutos_sin_actividad,
            self.impulsos.iter().filter(|i| !i.ejecutado).count(),
        )
    }
}
