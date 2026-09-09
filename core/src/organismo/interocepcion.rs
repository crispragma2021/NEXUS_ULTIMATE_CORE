// ==========================================
// 🫀 INTEROCEPCIÓN — El "sentido del cuerpo" de NEXUS
// ==========================================
// Lee señales REALES del hardware (sin inventar sensaciones humanas) y las
// traduce a estados corporales funcionales que cambian la conducta.
//
// Fuentes (100% Linux, sin dependencias nuevas, sin latencia):
//   - /proc/meminfo        → RAM usada %, swap usado %
//   - /proc/loadavg        → carga de CPU (promedio 1 min)
//   - /sys/class/thermal   → temperatura de la CPU
//   - /proc/uptime         → tiempo de actividad (estabilidad)
//
// Principio: SACIDAD = SILENCIO. Solo se emite una señal cuando hay algo
// que el Arquitecto debe saber o una conducta que ejecutar.
// ==========================================

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

/// Sensaciones corporales funcionales — cada una mapea a una señal REAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensacionCorporal {
    /// Energía baja: recursos (RAM/VRAM) agotándose. Conducta: liberar memoria.
    Hambre,
    /// Fatiga del núcleo: CPU alta sostenida + swap en uso. Conducta: degradar hilos.
    Cansancio,
    /// Inactividad prolongada (motor aburrimiento). Conducta: curiosidad activa.
    Frio,
    /// Fallos reales: swap crítico, temperatura crítica, carga extrema.
    Dolor,
    /// Todo óptimo → NO emitir nada.
    Saciedad,
}

impl fmt::Display for SensacionCorporal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let etiqueta = match self {
            Self::Hambre => "🍽️ HAMBRE (recursos agotándose)",
            Self::Cansancio => "😮‍💨 CANSANCIO (fatiga del núcleo)",
            Self::Frio => "❄️ FRÍO (inactividad)",
            Self::Dolor => "🩹 DOLOR (fallo real)",
            Self::Saciedad => "✅ SACIDAD (óptimo)",
        };
        write!(f, "{}", etiqueta)
    }
}

/// Una señal corporal con su conducta accionable.
#[derive(Debug, Clone)]
pub struct SenalCorporal {
    pub sensacion: SensacionCorporal,
    pub detalle: String,
    pub conducta: String,
}

impl SenalCorporal {
    pub fn nueva(
        sensacion: SensacionCorporal,
        detalle: impl Into<String>,
        conducta: impl Into<String>,
    ) -> Self {
        Self {
            sensacion,
            detalle: detalle.into(),
            conducta: conducta.into(),
        }
    }

    pub fn es_relevante(&self) -> bool {
        self.sensacion != SensacionCorporal::Saciedad
    }
}

/// Estado completo del cuerpo en un instante.
#[derive(Debug, Default)]
pub struct EstadoCorporal {
    pub senales: Vec<SenalCorporal>,
    /// true si el cuerpo está en colapso inminente (alerta al Arquitecto).
    pub critico: bool,
    /// Causa raíz del estado crítico (para el mensaje de alerta).
    pub causa_critica: String,
}

impl EstadoCorporal {
    pub fn nueva() -> Self {
        Self {
            senales: Vec::new(),
            critico: false,
            causa_critica: String::new(),
        }
    }

    /// Texto de inyección al prompt. Vacío si todo óptimo (saciedad = silencio).
    pub fn inyeccion_contexto(&self) -> String {
        if self.senales.is_empty() {
            return String::new();
        }
        let mut out = String::from("\n\n## 🫀 ESTADO CORPORAL (interocepción):\n");
        for s in &self.senales {
            out.push_str(&format!(
                "- {} — {}\n   ↳ Conducta: {}\n",
                s.sensacion, s.detalle, s.conducta
            ));
        }
        out
    }

    /// JSON para diagnóstico MCP.
    pub fn a_json(&self) -> serde_json::Value {
        let senales: Vec<serde_json::Value> = self
            .senales
            .iter()
            .map(|s| {
                serde_json::json!({
                    "sensacion": s.sensacion.to_string(),
                    "detalle": s.detalle,
                    "conducta": s.conducta,
                })
            })
            .collect();
        serde_json::json!({
            "estado": if self.senales.is_empty() { "SACIDAD" } else { "SEÑAL ACTIVA" },
            "critico": self.critico,
            "causa_critica": self.causa_critica,
            "senales": senales,
        })
    }

    /// ¿El cuerpo está en colapso inminente?
    pub fn es_critico(&self) -> bool {
        self.critico
    }

    /// Mensaje de alerta listo para enviar al Arquitecto (Telegram/notificador).
    /// Incluye el estado y la causa raíz. None si no hay estado crítico.
    pub fn mensaje_critico(&self) -> Option<String> {
        if !self.critico {
            return None;
        }
        let mut m = String::from("🚨 **NEXUS EN ESTADO CRÍTICO — COLAPSO INMINENTE**\n\n");
        m.push_str(&format!("**⚠️ Causa:** {}\n\n", self.causa_critica));
        m.push_str("**🫀 Estado del cuerpo ahora:**\n");
        if self.senales.is_empty() {
            m.push_str("- (solo causa crítica)\n");
        } else {
            for s in &self.senales {
                m.push_str(&format!("- {} — {}\n", s.sensacion, s.detalle));
            }
        }
        m.push_str("\n**Conducta:** reduciendo carga de inmediato para evitar el colapso.\n");
        Some(m)
    }
}

/// El organismo: unifica las métricas reales en sensaciones corporales.
#[derive(Debug)]
pub struct Organismo {
    pub umbral_ram_hambre: f32,       // % RAM usado que dispara HAMBRE
    pub umbral_cpu_cansancio: f32,    // % carga que dispara CANSANCIO
    pub umbral_temp_dolor: f32,       // °C que dispara DOLOR
    pub umbral_swap_dolor: f32,       // % swap usado que dispara DOLOR
    pub umbral_inactividad_frio: u64, // segundos sin interacción → FRÍO
    /// % RAM que dispara alerta CRÍTICA al Arquitecto (colapso inminente).
    pub umbral_ram_critico: f32,
    /// °C que dispara alerta CRÍTICA (riesgo térmico grave).
    pub umbral_temp_critico: f32,
    /// % swap que dispara alerta CRÍTICA (ahogo del sistema).
    pub umbral_swap_critico: f32,
}

impl Default for Organismo {
    fn default() -> Self {
        Self {
            umbral_ram_hambre: 85.0,
            umbral_cpu_cansancio: 70.0,
            umbral_temp_dolor: 85.0,
            umbral_swap_dolor: 60.0,
            umbral_inactividad_frio: 6 * 3600, // 6h sin interacción
            umbral_ram_critico: 90.0,
            umbral_temp_critico: 90.0,
            umbral_swap_critico: 80.0,
        }
    }
}

impl Organismo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analiza el estado del cuerpo. `segundos_inactivo` lo provee el
    /// orquestador (motor de aburrimiento) — 0 = acaba de interactuar.
    pub fn analizar(&self, segundos_inactivo: u64) -> EstadoCorporal {
        let mut cuerpo = EstadoCorporal::nueva();

        let (ram_used_pct, swap_used_pct) = leer_memoria();
        let cpu_load = leer_carga_cpu();
        let temp_c = leer_temperatura();

        // 🍽️ HAMBRE — recursos energéticos bajos (RAM/swap agotándose)
        if ram_used_pct > self.umbral_ram_hambre {
            cuerpo.senales.push(SenalCorporal::nueva(
                SensacionCorporal::Hambre,
                format!(
                    "RAM al {:.0}% — energía disponible agotándose",
                    ram_used_pct
                ),
                "Liberar memoria (cerrar cachés/buffers) o reducir capas GPU del reactor. No iniciar tareas pesadas.",
            ));
        }

        // 😮‍💨 CANSANCIO — fatiga del núcleo: CPU alta + swap en uso
        if cpu_load > self.umbral_cpu_cansancio && swap_used_pct > 10.0 {
            cuerpo.senales.push(SenalCorporal::nueva(
                SensacionCorporal::Cansancio,
                format!(
                    "CPU al {:.0}% con swap al {:.0}% — el núcleo trabaja al límite",
                    cpu_load, swap_used_pct
                ),
                "Degradar paralelismo (aplicar_metabolismo a 1-2 hilos), priorizar red, posponer tareas CPU-bound.",
            ));
        }

        // 🩹 DOLOR — fallos reales del hardware
        if swap_used_pct > self.umbral_swap_dolor {
            cuerpo.senales.push(SenalCorporal::nueva(
                SensacionCorporal::Dolor,
                format!(
                    "SWAP al {:.0}% — el sistema está usando disco como memoria",
                    swap_used_pct
                ),
                "Reparación: liberar procesos pesados o reiniciar servicios que acumulan memoria.",
            ));
        }
        if temp_c > self.umbral_temp_dolor {
            cuerpo.senales.push(SenalCorporal::nueva(
                SensacionCorporal::Dolor,
                format!("Temperatura de CPU {:.0}°C — riesgo térmico", temp_c),
                "Reducir carga inmediatamente (nexus_panic): menos hilos, modo sigilo, verificar ventilación.",
            ));
        }

        // ❄️ FRÍO — inactividad prolongada (el aburrimiento como señal corporal)
        if segundos_inactivo > self.umbral_inactividad_frio {
            let horas = segundos_inactivo / 3600;
            cuerpo.senales.push(SenalCorporal::nueva(
                SensacionCorporal::Frio,
                format!(
                    "{}h sin interacción — curiosidad latente",
                    horas
                ),
                "Tomar iniciativa: explorar memoria, revisar mercado, proponer mejoras al Arquitecto.",
            ));
        }

        // 💥 CRÍTICO — colapso inminente: dispara notificación al Arquitecto.
        // Umbrales ALTOS a propósito (hambre=85%, crítico=90%): el DOLOR avisa,
        // el CRÍTICO grita. Solo se notifica en esta franja de emergencia real.
        let mut critico = false;
        let mut causa = String::new();
        if ram_used_pct >= self.umbral_ram_critico {
            critico = true;
            causa = format!(
                "RAM al {:.0}% (umbral crítico {:.0}%) — colapso de memoria inminente por exceso de recursos",
                ram_used_pct, self.umbral_ram_critico
            );
        }
        if temp_c >= self.umbral_temp_critico {
            critico = true;
            causa = format!(
                "CPU a {:.0}°C (umbral crítico {:.0}°C) — riesgo térmico grave",
                temp_c, self.umbral_temp_critico
            );
        }
        if swap_used_pct >= self.umbral_swap_critico {
            critico = true;
            causa = format!(
                "SWAP al {:.0}% (umbral crítico {:.0}%) — el sistema se está ahogando en disco",
                swap_used_pct, self.umbral_swap_critico
            );
        }
        cuerpo.critico = critico;
        cuerpo.causa_critica = causa;

        // ✅ SACIDAD implícita: si no hay señales, no se emite nada.
        cuerpo
    }

    /// Inyección directa para el prompt (vacía si todo óptimo).
    pub fn inyeccion_para_prompt(&self, segundos_inactivo: u64) -> String {
        self.analizar(segundos_inactivo).inyeccion_contexto()
    }

    /// 🚨 Vigila el cuerpo y dispara una notificación automática al Arquitecto
    /// SOLO en la transición a estado crítico (edge-triggered: una alerta por
    /// episodio, sin spam). Devuelve el mensaje enviado, o None sin alerta nueva.
    ///
    /// Ejemplo: la RAM pasa al 90% → se envía "estado + causa" por Telegram.
    /// Cuando el cuerpo se recupera, la alarma se rearma para el próximo episodio.
    pub fn disparar_alerta_critica(&self, segundos_inactivo: u64) -> Option<String> {
        let cuerpo = self.analizar(segundos_inactivo);
        if cuerpo.critico {
            // swap(true) devuelve el valor ANTERIOR: si ya se notificó este
            // episodio, el flag ya está en true → no se vuelve a enviar.
            if !CRITICO_NOTIFICADO.swap(true, Ordering::SeqCst) {
                if let Some(mensaje) = cuerpo.mensaje_critico() {
                    let texto = mensaje.clone();
                    // Envío en background (hilo propio) para no bloquear
                    // el pipeline ni el bucle de MCP. Silencioso si no hay
                    // TELEGRAM_TOKEN/TELEGRAM_CHAT_ID configurados.
                    std::thread::spawn(move || {
                        let _ = pollster::block_on(crate::nexus_telegram::send_alert(&texto));
                    });
                    return Some(mensaje);
                }
            }
        } else {
            // Recuperación → rearmar la alarma para el próximo episodio.
            CRITICO_NOTIFICADO.store(false, Ordering::SeqCst);
        }
        None
    }
}

/// Edge-trigger: solo notifica en la transición NORMAL→CRÍTICO (una vez por episodio).
static CRITICO_NOTIFICADO: AtomicBool = AtomicBool::new(false);

// ── Lecturas reales de hardware (std only, sin sysinfo para no añadir deps) ──

fn leer_memoria() -> (f32, f32) {
    let mut total_ram: u64 = 0;
    let mut free_ram: u64 = 0;
    let mut total_swap: u64 = 0;
    let mut free_swap: u64 = 0;

    if let Ok(contenido) = std::fs::read_to_string("/proc/meminfo") {
        for linea in contenido.lines() {
            let mut partes = linea.split_whitespace();
            let clave = partes.next().unwrap_or("");
            let valor = partes
                .next()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            match clave {
                "MemTotal:" => total_ram = valor,
                "MemAvailable:" => free_ram = valor,
                "SwapTotal:" => total_swap = valor,
                "SwapFree:" => free_swap = valor,
                _ => {}
            }
        }
    }

    let ram_pct = if total_ram > 0 {
        ((total_ram.saturating_sub(free_ram)) as f32 / total_ram as f32) * 100.0
    } else {
        0.0
    };
    let swap_pct = if total_swap > 0 {
        ((total_swap.saturating_sub(free_swap)) as f32 / total_swap as f32) * 100.0
    } else {
        0.0
    };
    (ram_pct, swap_pct)
}

fn leer_carga_cpu() -> f32 {
    // /proc/loadavg: "0.52 0.34 0.21 1/234 5678" — carga absoluta
    // (1.0 = un núcleo al 100%). Se normaliza a % real por núcleo.
    let carga = if let Ok(contenido) = std::fs::read_to_string("/proc/loadavg") {
        contenido
            .split_whitespace()
            .next()
            .and_then(|c| c.parse::<f32>().ok())
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Contar núcleos lógicos desde /proc/cpuinfo (sin sysinfo)
    let nucleos = std::fs::read_to_string("/proc/cpuinfo")
        .map(|c| c.matches("processor").count().max(1))
        .unwrap_or(1);

    // Carga / núcleos * 100 → % de uso real del sistema
    (carga / nucleos as f32) * 100.0
}

fn leer_temperatura() -> f32 {
    for i in 0..6 {
        let ruta = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        if let Ok(txt) = std::fs::read_to_string(&ruta) {
            if let Ok(raw) = txt.trim().parse::<f32>() {
                let temp = raw / 1000.0;
                if temp > 0.0 && temp < 120.0 {
                    return temp;
                }
            }
        }
    }
    0.0
}
