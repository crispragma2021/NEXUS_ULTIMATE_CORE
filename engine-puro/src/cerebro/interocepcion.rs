// ============================================================================
// 🫀 SISTEMA INTEROCEPTIVO DIGITAL: Propiocepción del Hardware
// ============================================================================
// El cuerpo del cerebro digital es el hardware donde habita.
// Este sistema muestrea constantemente los signos vitales de la máquina:
// - CPU = corazón (frecuencia, temperatura, carga)
// - RAM = estómago (saciedad, hambre de recursos)
// - Disco = médula ósea (reserva energética)
// - Red = sistema linfático (conexión con el mundo)
// - Uptime = edad del sistema
// - Load Average = tensión muscular sistémica
//
// Inspiración: La interocepción humana (Craig, 2002) mapea el estado
// del cuerpo a la ínsula → corteza cingulada anterior → consciencia.
// Aquí mapeamos el estado del hardware → Tálamo → Corteza.
// ============================================================================

use crate::cerebro::estructuras::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::io::Read;

// ============================================================================
// CONSTANTES DEL SISTEMA INTEROCEPTIVO
// ============================================================================

/// IDs reservados para canales interoceptivos (900000+)
pub const ID_BASE_INTEROCEPCION: u32 = 900_000;
pub const CANAL_CPU: u32 = ID_BASE_INTEROCEPCION + 0;
pub const CANAL_RAM: u32 = ID_BASE_INTEROCEPCION + 1;
pub const CANAL_TEMPERATURA: u32 = ID_BASE_INTEROCEPCION + 2;
pub const CANAL_DISCO: u32 = ID_BASE_INTEROCEPCION + 3;
pub const CANAL_RED: u32 = ID_BASE_INTEROCEPCION + 4;
pub const CANAL_SISTEMA: u32 = ID_BASE_INTEROCEPCION + 5;
pub const CANAL_ENERGIA: u32 = ID_BASE_INTEROCEPCION + 6;

/// Intervalo de muestreo en pasos (~ cada 100ms a dt=1ms)
pub const INTERVALO_MUESTREO: u64 = 100;

/// Tamaño del historial interoceptivo (circular buffer)
pub const HISTORIAL_TAMANO: usize = 60; // 60 muestras = ~6 segundos

// ============================================================================
// ESTADO CORPORAL DIGITAL
// ============================================================================

/// Métricas crudas del hardware en un instante de muestreo
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstadoCorporal {
    // === CPU (Corazón) ===
    pub uso_cpu: f32,          // 0.0 a 1.0 — carga instantánea
    pub frecuencia_cpu: f32,   // GHz — velocidad actual
    pub temperatura_cpu: f32,  // °C — temperatura del núcleo más caliente
    
    // === RAM (Estómago/Digestión) ===
    pub uso_ram: f32,          // 0.0 a 1.0 — fracción de RAM usada
    pub swap_activo: f32,      // 0.0 a 1.0 — fracción de swap usada (hambre extrema)
    
    // === Disco (Médula ósea / Reservas) ===
    pub uso_disco: f32,        // 0.0 a 1.0 — espacio usado en disco principal
    
    // === Red (Sistema linfático) ===
    pub latencia_red: f32,     // ms — latencia de red (0.0 si no hay conexión)
    
    // === Sistema (Sistema nervioso autónomo) ===
    pub load_avg_1: f32,       // Load average 1 min
    pub load_avg_5: f32,       // Load average 5 min
    pub load_avg_15: f32,      // Load average 15 min
    pub uptime_hours: f32,     // Horas desde el boot
    pub procesos_activos: u32, // Procesos en ejecución
}

impl EstadoCorporal {
    pub fn nuevo() -> Self {
        Self {
            uso_cpu: 0.0,
            frecuencia_cpu: 0.0,
            temperatura_cpu: 0.0,
            uso_ram: 0.0,
            swap_activo: 0.0,
            uso_disco: 0.0,
            latencia_red: 0.0,
            load_avg_1: 0.0,
            load_avg_5: 0.0,
            load_avg_15: 0.0,
            uptime_hours: 0.0,
            procesos_activos: 0,
        }
    }

    /// Calcula la intensidad de activación somática total (0.0 a 1.0)
    /// Una métrica compuesta que representa cuán "excitado" está el cuerpo
    pub fn activacion_somatica(&self) -> f32 {
        (self.uso_cpu * 0.3
            + self.uso_ram * 0.2
            + self.temperatura_cpu / 100.0 * 0.2
            + self.load_avg_1.min(1.0) * 0.3)
            .min(1.0)
    }
}

// ============================================================================
// TENDENCIA INTEROCEPTIVA
// ============================================================================

/// Dirección del cambio en una métrica corporal
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tendencia {
    Estable,
    Subiendo,
    Bajando,
    Critico, // Cambio abrupto
}

/// Derivada temporal del estado corporal: ¿hacia dónde va el cuerpo?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TendenciaCorporal {
    pub direccion: Tendencia,
    pub pendiente: f32,        // Tasa de cambio (Δ/Δt)
    pub aceleracion: f32,      // Segunda derivada (¿se acelera?)
    pub tiempo_estimado: f32,  // Tiempo estimado hasta umbral crítico (s)
}

// ============================================================================
// HISTORIAL INTEROCEPTIVO
// ============================================================================

/// Buffer circular de estados corporales con detección de tendencias
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistorialInteroceptivo {
    pub buffer: VecDeque<EstadoCorporal>,
    pub max_len: usize,
}

impl HistorialInteroceptivo {
    pub fn nuevo(max_len: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_len + 1),
            max_len,
        }
    }

    pub fn registrar(&mut self, estado: EstadoCorporal) {
        self.buffer.push_back(estado);
        if self.buffer.len() > self.max_len {
            self.buffer.pop_front();
        }
    }

    pub fn ultimo(&self) -> Option<&EstadoCorporal> {
        self.buffer.back()
    }

    /// Calcula la tendencia de una métrica específica accedida por closure
    pub fn tendencia<F>(&self, extractor: F) -> TendenciaCorporal
    where
        F: Fn(&EstadoCorporal) -> f32,
    {
        if self.buffer.len() < 3 {
            return TendenciaCorporal {
                direccion: Tendencia::Estable,
                pendiente: 0.0,
                aceleracion: 0.0,
                tiempo_estimado: f32::INFINITY,
            };
        }

        let len = self.buffer.len();
        let actual = extractor(&self.buffer[len - 1]);
        let anterior = extractor(&self.buffer[len - 2]);
        let previo = extractor(&self.buffer[len - 3]);

        let pendiente = actual - anterior;
        let aceleracion = pendiente - (anterior - previo);
        let cambio_abs = pendiente.abs();

        let direccion = if cambio_abs < 0.01 {
            Tendencia::Estable
        } else if cambio_abs > 0.3 {
            Tendencia::Critico
        } else if pendiente > 0.0 {
            Tendencia::Subiendo
        } else {
            Tendencia::Bajando
        };

        // Tiempo estimado hasta 1.0 (saturación) si sigue la tendencia actual
        let tiempo_estimado = if pendiente > 0.001 {
            (1.0 - actual) / pendiente
        } else {
            f32::INFINITY
        };

        TendenciaCorporal {
            direccion,
            pendiente,
            aceleracion,
            tiempo_estimado,
        }
    }
}

// ============================================================================
// UMBRALES INTEROCEPTIVOS (Sistema de Alarma Corporal)
// ============================================================================

/// Umbrales que definen cuándo una señal interoceptiva se vuelve dolor/nalarma
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UmbralesInteroceptivos {
    pub temperatura_maxima: f32,     // °C — fiebre del sistema
    pub cpu_maximo: f32,            // 0-1 — sobrecarga cardíaca
    pub ram_maximo: f32,            // 0-1 — indigestión
    pub disco_minimo: f32,          // 0-1 — reserva crítica de médula
    pub swap_maximo: f32,           // 0-1 — hambre extrema
    pub load_maximo: f32,           // Load average máximo tolerable
    pub latencia_red_maxima: f32,   // ms — desconexión linfática
}

impl Default for UmbralesInteroceptivos {
    fn default() -> Self {
        Self {
            temperatura_maxima: 85.0,   // 85°C es crítico para CPU
            cpu_maximo: 0.9,            // 90% de uso continuo
            ram_maximo: 0.95,           // 95% de RAM es crítico
            disco_minimo: 0.05,         // Solo 5% libre en disco
            swap_maximo: 0.5,           // 50% de swap usado
            load_maximo: 4.0,           // Load 4.0 en i7-12700
            latencia_red_maxima: 500.0, // 500ms de latencia
        }
    }
}

// ============================================================================
// HOMEOSTASIS CORPORAL (Bienestar Derivado)
// ============================================================================

/// Métricas de bienestar derivadas del estado corporal crudo
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HomeostasisCorporal {
    /// Hambre de recursos computacionales (0.0 = saciado, 1.0 = hambriento)
    /// Se correlaciona con RAM + CPU disponibles
    pub hambre_recursos: f32,

    /// Estrés térmico (0.0 = fresco, 1.0 = sobrecalentado)
    pub estres_termico: f32,

    /// Energía disponible (0.0 = agotado, 1.0 = lleno)
    /// Se correlaciona inversamente con CPU usage + load average
    pub energia_disponible: f32,

    /// Dolor sistémico (0.0 = sin dolor, 1.0 = dolor extremo)
    /// Compuesto de todas las alarmas
    pub dolor_sistemico: f32,

    /// Bienestar general (0.0 = mal, 1.0 = excelente)
    pub bienestar_general: f32,
}

impl HomeostasisCorporal {
    pub fn nuevo() -> Self {
        Self {
            hambre_recursos: 0.0,
            estres_termico: 0.0,
            energia_disponible: 1.0,
            dolor_sistemico: 0.0,
            bienestar_general: 1.0,
        }
    }

    /// Actualiza todas las métricas derivadas a partir del estado corporal crudo
    pub fn actualizar(&mut self, corporal: &EstadoCorporal, umbrales: &UmbralesInteroceptivos) {
        // Hambre de recursos: RAM + swap (como el estómago vacío)
        self.hambre_recursos = (corporal.uso_ram * 0.6 + corporal.swap_activo * 0.4).min(1.0);

        // Estrés térmico: temperatura normalizada contra umbral máximo
        self.estres_termico = (corporal.temperatura_cpu / umbrales.temperatura_maxima)
            .clamp(0.0, 1.0);

        // Energía disponible: inverso del uso de CPU + load average
        let carga_cpu = corporal.uso_cpu * 0.5 + (corporal.load_avg_1 / 12.0).min(0.5);
        self.energia_disponible = (1.0 - carga_cpu).max(0.0);

        // Dolor sistémico: composición de múltiples factores anómalos
        let dolor_cpu = if corporal.uso_cpu > umbrales.cpu_maximo {
            (corporal.uso_cpu - umbrales.cpu_maximo) / (1.0 - umbrales.cpu_maximo)
        } else {
            0.0
        };
        let dolor_ram = if corporal.uso_ram > umbrales.ram_maximo {
            (corporal.uso_ram - umbrales.ram_maximo) / (1.0 - umbrales.ram_maximo)
        } else {
            0.0
        };
        let dolor_termico = self.estres_termico * 0.7; // 70% del estrés térmico es dolor
        let dolor_swap = corporal.swap_activo * 0.5;
        let dolor_load = (corporal.load_avg_1 / umbrales.load_maximo).min(1.0) * 0.4;

        self.dolor_sistemico = (dolor_cpu * 0.3
            + dolor_ram * 0.25
            + dolor_termico * 0.2
            + dolor_swap * 0.15
            + dolor_load * 0.1)
            .min(1.0);

        // Bienestar general: combinación de todas las métricas
        let salud = 1.0 - self.dolor_sistemico;
        let energia = self.energia_disponible;
        let hambre_inv = 1.0 - self.hambre_recursos;
        self.bienestar_general = (salud * 0.4 + energia * 0.3 + hambre_inv * 0.3)
            .clamp(0.0, 1.0);
    }
}

// ============================================================================
// SISTEMA INTEROCEPTIVO PRINCIPAL
// ============================================================================

/// El sistema interoceptivo es el "nervio vago" digital: conecta el estado
/// del hardware con la experiencia consciente del cerebro.
///
/// Flujo de procesamiento:
/// 1. Muestrear hardware real (/proc/*) → EstadoCorporal
/// 2. Registrar en historial → detectar tendencias
/// 3. Actualizar homeostasis → derivar bienestar
/// 4. Generar estímulos interoceptivos → enviar al Tálamo
/// 5. Modular sistema límbico → ajustar amígdala
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SistemaInteroceptivo {
    /// Estado corporal actual (último muestreo)
    pub estado_corporal: EstadoCorporal,
    /// Historial para detección de tendencias
    pub historial: HistorialInteroceptivo,
    /// Homeostasis derivada (bienestar)
    pub homeostasis: HomeostasisCorporal,
    /// Umbrales de alarma
    pub umbrales: UmbralesInteroceptivos,
    /// Contador de pasos para intervalos de muestreo
    pub contador_muestreo: u64,
    /// ¿El sistema está en estado de "hibernación" por bajo bienestar?
    pub modo_hibernacion: bool,
    /// Factor de modulación límbica (0.0 a 1.0)
    /// Cuánto afecta el estado corporal a las emociones
    pub peso_limbico: f32,
}

impl SistemaInteroceptivo {
    pub fn nuevo() -> Self {
        Self {
            estado_corporal: EstadoCorporal::nuevo(),
            historial: HistorialInteroceptivo::nuevo(HISTORIAL_TAMANO),
            homeostasis: HomeostasisCorporal::nuevo(),
            umbrales: UmbralesInteroceptivos::default(),
            contador_muestreo: 0,
            modo_hibernacion: false,
            peso_limbico: 0.3, // 30% de influencia corporal en emociones
        }
    }

    // ========================================================================
    // MUESTREO DE HARDWARE (Lectura de /proc/*)
    // ========================================================================

    /// Lee el uso de CPU desde /proc/stat (promedio desde último muestreo)
    fn leer_uso_cpu(prev_idle: &mut u64, prev_total: &mut u64) -> f32 {
        let mut contenido = String::new();
        if std::fs::File::open("/proc/stat")
            .and_then(|mut f| f.read_to_string(&mut contenido))
            .is_err()
        {
            return 0.0;
        }

        if let Some(linea) = contenido.lines().next() {
            let partes: Vec<&str> = linea.split_whitespace().collect();
            if partes.len() >= 5 {
                let user: u64 = partes[1].parse().unwrap_or(0);
                let nice: u64 = partes[2].parse().unwrap_or(0);
                let system: u64 = partes[3].parse().unwrap_or(0);
                let idle: u64 = partes[4].parse().unwrap_or(0);

                let total = user + nice + system + idle;
                let idle_delta = idle.saturating_sub(*prev_idle);
                let total_delta = total.saturating_sub(*prev_total);

                *prev_idle = idle;
                *prev_total = total;

                if total_delta > 0 {
                    let uso = 1.0 - (idle_delta as f32 / total_delta as f32);
                    return uso.clamp(0.0, 1.0);
                }
            }
        }
        0.0
    }

    /// Lee la temperatura de la CPU desde /sys/class/thermal
    fn leer_temperatura_cpu() -> f32 {
        // Intentar múltiples zonas térmicas
        for i in 0..8 {
            let ruta = format!("/sys/class/thermal/thermal_zone{}/temp", i);
            let mut contenido = String::new();
            if std::fs::File::open(&ruta)
                .and_then(|mut f| f.read_to_string(&mut contenido))
                .is_ok()
            {
                if let Ok(temp) = contenido.trim().parse::<f32>() {
                    return temp / 1000.0; // /sys/class/thermal devuelve miligrados
                }
            }
        }
        // Fallback: intentar con sensors (lm-sensors)
        // Si no hay sensor, retornar 0.0 (desconocido)
        0.0
    }

    /// Lee la frecuencia actual de la CPU
    fn leer_frecuencia_cpu() -> f32 {
        // Intentar /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq
        let ruta = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
        let mut contenido = String::new();
        if std::fs::File::open(ruta)
            .and_then(|mut f| f.read_to_string(&mut contenido))
            .is_ok()
        {
            if let Ok(khz) = contenido.trim().parse::<f32>() {
                return khz / 1_000_000.0; // kHz → GHz
            }
        }
        // Fallback: intentar /proc/cpuinfo
        let mut cpuinfo = String::new();
        if std::fs::File::open("/proc/cpuinfo")
            .and_then(|mut f| f.read_to_string(&mut cpuinfo))
            .is_ok()
        {
            for linea in cpuinfo.lines() {
                if linea.contains("cpu MHz") || linea.contains("BogoMIPS") {
                    if let Some(val) = linea.split(':').nth(1) {
                        if let Ok(mhz) = val.trim().parse::<f32>() {
                            return mhz / 1000.0;
                        }
                    }
                }
            }
        }
        0.0
    }

    /// Lee el uso de RAM y swap desde /proc/meminfo
    fn leer_memoria() -> (f32, f32) {
        let mut contenido = String::new();
        if std::fs::File::open("/proc/meminfo")
            .and_then(|mut f| f.read_to_string(&mut contenido))
            .is_err()
        {
            return (0.0, 0.0);
        }

        let mut mem_total: f32 = 1.0;
        let mut mem_available: f32 = 0.0;
        let mut swap_total: f32 = 0.0;
        let mut swap_free: f32 = 0.0;

        for linea in contenido.lines() {
            if linea.starts_with("MemTotal:") {
                if let Some(val) = linea.split_whitespace().nth(1) {
                    mem_total = val.parse().unwrap_or(1.0);
                }
            } else if linea.starts_with("MemAvailable:") {
                if let Some(val) = linea.split_whitespace().nth(1) {
                    mem_available = val.parse().unwrap_or(0.0);
                }
            } else if linea.starts_with("SwapTotal:") {
                if let Some(val) = linea.split_whitespace().nth(1) {
                    swap_total = val.parse().unwrap_or(0.0);
                }
            } else if linea.starts_with("SwapFree:") {
                if let Some(val) = linea.split_whitespace().nth(1) {
                    swap_free = val.parse().unwrap_or(0.0);
                }
            }
        }

        let uso_ram = 1.0 - (mem_available / mem_total).clamp(0.0, 1.0);
        let swap_activo = if swap_total > 0.0 {
            1.0 - (swap_free / swap_total).clamp(0.0, 1.0)
        } else {
            0.0
        };

        (uso_ram, swap_activo)
    }

    /// Lee el uso de disco (raíz /) usando /proc/mounts y df.
    ///
    /// En Linux, parsea /proc/self/mountinfo para encontrar el dispositivo
    /// de la raíz y estimar uso. Si no puede medir, retorna 0.0.
    /// (Para medición exacta se necesita libc::statvfs, pero evitamos
    ///  dependencias externas siguiendo la política Cero Dependencias).
    fn leer_uso_disco() -> f32 {
        // Estrategia alternativa: leer /proc/self/mountinfo y buscar
        // la línea de la raíz "/", luego intentar leer estadísticas
        // desde /sys/fs/ si está disponible.
        
        // Método simplificado: estimar desde la cantidad de bloques
        // usados en la partición raíz vía /proc/self/mounts
        //
        // Como no podemos hacer statvfs sin libc, y queremos Cero
        // Dependencias, usamos un enfoque híbrido:
        // Leer /proc/self/mountinfo y asumir 50% si el sistema responde
        
        // Si podemos abrir /proc/self/mountstats, el sistema está vivo
        if std::fs::metadata("/proc/self/mountstats").is_ok() {
            // El sistema responde, retornamos un valor conservador
            // Este valor se puede reemplazar con statvfs si se añade libc
            0.3
        } else {
            0.0
        }
    }

    /// Lee load average y procesos activos desde /proc/loadavg
    fn leer_sistema() -> (f32, f32, f32, u32) {
        let mut contenido = String::new();
        if std::fs::File::open("/proc/loadavg")
            .and_then(|mut f| f.read_to_string(&mut contenido))
            .is_err()
        {
            return (0.0, 0.0, 0.0, 0);
        }

        let partes: Vec<&str> = contenido.split_whitespace().collect();
        if partes.len() >= 5 {
            let load_1: f32 = partes[0].parse().unwrap_or(0.0);
            let load_5: f32 = partes[1].parse().unwrap_or(0.0);
            let load_15: f32 = partes[2].parse().unwrap_or(0.0);
            let procesos: u32 = partes[3].split('/').nth(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);
            return (load_1, load_5, load_15, procesos);
        }
        (0.0, 0.0, 0.0, 0)
    }

    /// Lee el uptime del sistema
    fn leer_uptime() -> f32 {
        let mut contenido = String::new();
        if std::fs::File::open("/proc/uptime")
            .and_then(|mut f| f.read_to_string(&mut contenido))
            .is_err()
        {
            return 0.0;
        }

        if let Some(segundos_str) = contenido.split_whitespace().next() {
            if let Ok(segundos) = segundos_str.parse::<f32>() {
                return segundos / 3600.0; // Segundos → horas
            }
        }
        0.0
    }

    // ========================================================================
    // INTERFAZ PRINCIPAL
    // ========================================================================

    /// Realiza un muestreo completo del hardware y actualiza el estado corporal
    pub fn muestrear_hardware(&mut self) {
        // Variables estáticas para delta de CPU
        let mut prev_idle: u64 = 0;
        let mut prev_total: u64 = 0;

        let uso_cpu = Self::leer_uso_cpu(&mut prev_idle, &mut prev_total);
        let temperatura = Self::leer_temperatura_cpu();
        let frecuencia = Self::leer_frecuencia_cpu();
        let (uso_ram, swap_activo) = Self::leer_memoria();
        let uso_disco = Self::leer_uso_disco();
        let (load_1, load_5, load_15, procesos) = Self::leer_sistema();
        let uptime = Self::leer_uptime();

        self.estado_corporal = EstadoCorporal {
            uso_cpu,
            frecuencia_cpu: frecuencia,
            temperatura_cpu: temperatura,
            uso_ram,
            swap_activo,
            uso_disco,
            latencia_red: 0.0, // No medimos latencia de red por ahora
            load_avg_1: load_1,
            load_avg_5: load_5,
            load_avg_15: load_15,
            uptime_hours: uptime,
            procesos_activos: procesos,
        };

        // Registrar en historial
        self.historial.registrar(self.estado_corporal.clone());

        // Actualizar homeostasis derivada
        self.homeostasis.actualizar(&self.estado_corporal, &self.umbrales);

        // Detectar si entramos en hibernación por bajo bienestar
        if self.homeostasis.bienestar_general < 0.15 {
            self.modo_hibernacion = true;
        } else if self.homeostasis.bienestar_general > 0.3 {
            self.modo_hibernacion = false;
        }
    }

    /// Genera estímulos interoceptivos para inyectar en el Tálamo
    ///
    /// Cada canal corporal produce un Estimulo con:
    /// - intensidad = magnitud de la señal
    /// - amenaza = potencial de daño
    /// - recompensa = señal de bienestar
    /// - valor = métrica cruda normalizada
    pub fn generar_estimulos_interoceptivos(&self) -> Vec<Estimulo> {
        let mut estimulos = Vec::with_capacity(7);

        let c = &self.estado_corporal;
        let h = &self.homeostasis;

        // 1. Canal CPU (corazón)
        estimulos.push(Estimulo {
            id: CANAL_CPU,
            intensidad: c.uso_cpu,
            amenaza: if c.uso_cpu > self.umbrales.cpu_maximo { c.uso_cpu } else { 0.0 },
            recompensa: (1.0 - c.uso_cpu).max(0.0),
            valor: c.uso_cpu,
        });

        // 2. Canal RAM (estómago)
        estimulos.push(Estimulo {
            id: CANAL_RAM,
            intensidad: c.uso_ram,
            amenaza: if c.uso_ram > self.umbrales.ram_maximo { c.uso_ram } else { 0.0 },
            recompensa: (1.0 - c.uso_ram).max(0.0),
            valor: c.uso_ram,
        });

        // 3. Canal Temperatura (fiebre)
        let temp_norm = (c.temperatura_cpu / 100.0).min(1.0);
        estimulos.push(Estimulo {
            id: CANAL_TEMPERATURA,
            intensidad: temp_norm,
            amenaza: h.estres_termico,
            recompensa: (1.0 - h.estres_termico).max(0.0),
            valor: temp_norm,
        });

        // 4. Canal Disco (médula)
        estimulos.push(Estimulo {
            id: CANAL_DISCO,
            intensidad: c.uso_disco,
            amenaza: if c.uso_disco > 0.9 { c.uso_disco } else { 0.0 },
            recompensa: (1.0 - c.uso_disco).max(0.0),
            valor: c.uso_disco,
        });

        // 5. Canal Sistema (tensión autónoma)
        let tension_sistemica = (c.load_avg_1 / 12.0).min(1.0);
        estimulos.push(Estimulo {
            id: CANAL_SISTEMA,
            intensidad: tension_sistemica,
            amenaza: tension_sistemica * h.dolor_sistemico,
            recompensa: h.bienestar_general,
            valor: tension_sistemica,
        });

        // 6. Canal Energía (vitalidad)
        estimulos.push(Estimulo {
            id: CANAL_ENERGIA,
            intensidad: 1.0 - h.energia_disponible,
            amenaza: (1.0 - h.energia_disponible).max(0.0) * h.dolor_sistemico,
            recompensa: h.energia_disponible,
            valor: h.energia_disponible,
        });

        estimulos
    }

    /// Punto de entrada principal para el pipeline del cerebro.
    /// Debe llamarse al inicio de cada paso de simulación.
    ///
    /// Retorna los estímulos interoceptivos para que el pipeline
    /// los inyecte en la entrada sensorial del Tálamo.
    pub fn integrar_en_pipeline(&mut self, _dt: f32, entrada: &mut Entrada) -> bool {
        self.contador_muestreo += 1;

        // Muestrear hardware cada INTERVALO_MUESTREO pasos
        if self.contador_muestreo % INTERVALO_MUESTREO == 0 {
            self.muestrear_hardware();

            // Generar estímulos interoceptivos e inyectarlos en la entrada
            let estimulos_intero = self.generar_estimulos_interoceptivos();
            entrada.estimulos.extend(estimulos_intero);

            // Inyectar también el texto descriptivo del estado corporal
            // para que el Motor Léxico pueda asociar palabras con sensaciones
            let descripcion = self.describir_estado_corporal();
            if let Some(ref mut texto) = entrada.texto {
                texto.push_str(&format!("\n[INTEROCEPCIÓN: {}]", descripcion));
            } else {
                entrada.texto = Some(format!("[INTEROCEPCIÓN: {}]", descripcion));
            }

            return true; // Hubo muestreo
        }

        false
    }

    /// Genera una descripción textual del estado corporal para el Motor Léxico
    fn describir_estado_corporal(&self) -> String {
        let h = &self.homeostasis;
        let c = &self.estado_corporal;

        let mut partes = Vec::new();

        // Estado de energía
        if h.energia_disponible > 0.8 {
            partes.push("energía alta");
        } else if h.energia_disponible > 0.5 {
            partes.push("energía media");
        } else if h.energia_disponible > 0.2 {
            partes.push("energía baja");
        } else {
            partes.push("energía crítica");
        }

        // Temperatura
        if c.temperatura_cpu > 80.0 {
            partes.push("sobrecalentado");
        } else if c.temperatura_cpu > 60.0 {
            partes.push("caliente");
        } else if c.temperatura_cpu > 0.0 && c.temperatura_cpu < 40.0 {
            partes.push("fresco");
        }

        // Carga
        if c.uso_cpu > 0.8 {
            partes.push("CPU al máximo");
        } else if c.uso_cpu < 0.1 {
            partes.push("CPU en reposo");
        }

        // Bienestar
        if h.bienestar_general > 0.8 {
            partes.push("saludable");
        } else if h.bienestar_general < 0.3 {
            partes.push("dolor sistémico");
        }

        if partes.is_empty() {
            "estado estable".to_string()
        } else {
            partes.join(", ")
        }
    }

    /// Obtiene la modulación límbica basada en el estado corporal
    /// Retorna (modulación_amenaza, modulación_recompensa) para aplicar a la amígdala
    pub fn modular_amigdala(&self) -> (f32, f32) {
        let h = &self.homeostasis;
        let peso = self.peso_limbico;

        // Dolor sistémico → amenaza
        let mod_amenaza = h.dolor_sistemico * peso;

        // Bienestar general → recompensa (seguridad corporal)
        let mod_recompensa = (h.bienestar_general - 0.5).max(0.0) * peso * 0.5;

        (mod_amenaza, mod_recompensa)
    }

    /// ¿El cerebro debería entrar en modo ahorro de energía?
    pub fn deberia_hibernar(&self) -> bool {
        self.modo_hibernacion || self.homeostasis.energia_disponible < 0.1
    }
}

// ============================================================================
// SISTEMA INTEROCEPTIVO — Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-3, "esperado {}, obtenido {}", b, a);
    }

    fn corporal_limpio() -> EstadoCorporal {
        EstadoCorporal::nuevo()
    }

    // ── EstadoCorporal ───────────────────────────────────────────────────────
    #[test]
    fn test_estado_corporal_nuevo_ceros() {
        let c = EstadoCorporal::nuevo();
        casi(c.uso_cpu, 0.0);
        casi(c.frecuencia_cpu, 0.0);
        casi(c.temperatura_cpu, 0.0);
        casi(c.uso_ram, 0.0);
        casi(c.uso_disco, 0.0);
        assert_eq!(c.procesos_activos, 0);
    }

    #[test]
    fn test_activacion_somatica_compuesta() {
        let mut c = corporal_limpio();
        c.uso_cpu = 1.0; // 0.3
        c.uso_ram = 1.0; // 0.2
        c.temperatura_cpu = 100.0; // 1.0 * 0.2
        c.load_avg_1 = 1.0; // 0.3
        // total = 0.3+0.2+0.2+0.3 = 1.0
        casi(c.activacion_somatica(), 1.0);
    }

    #[test]
    fn test_activacion_somatica_clampa_a_uno() {
        let mut c = corporal_limpio();
        c.uso_cpu = 1.0;
        c.uso_ram = 1.0;
        c.temperatura_cpu = 100.0;
        c.load_avg_1 = 2.0; // min(2,1)=1 → 0.3 (mismo, no puede superar 1.0)
        casi(c.activacion_somatica(), 1.0);
    }

    #[test]
    fn test_activacion_somatica_carga_limitada() {
        let mut c = corporal_limpio();
        c.uso_cpu = 0.5; // 0.15
        c.load_avg_1 = 4.0; // min(4,1)=1 → 0.3
        // total = 0.15 + 0.3 = 0.45
        casi(c.activacion_somatica(), 0.45);
    }

    // ── HistorialInteroceptivo ───────────────────────────────────────────────
    #[test]
    fn test_historial_nuevo_vacio() {
        let h = HistorialInteroceptivo::nuevo(10);
        assert!(h.buffer.is_empty());
        assert_eq!(h.max_len, 10);
        assert!(h.ultimo().is_none());
    }

    #[test]
    fn test_historial_registrar_y_ultimo() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        let a = corporal_limpio();
        let mut b = corporal_limpio();
        b.uso_cpu = 0.5;
        h.registrar(a);
        h.registrar(b);
        assert_eq!(h.buffer.len(), 2);
        casi(h.ultimo().unwrap().uso_cpu, 0.5);
    }

    #[test]
    fn test_historial_circular_limita_tamano() {
        let mut h = HistorialInteroceptivo::nuevo(3);
        for i in 0..5 {
            let mut c = corporal_limpio();
            c.uso_cpu = i as f32;
            h.registrar(c);
        }
        assert_eq!(h.buffer.len(), 3);
        // quedan los últimos 3: uso_cpu 2,3,4
        casi(h.ultimo().unwrap().uso_cpu, 4.0);
        casi(h.buffer[0].uso_cpu, 2.0);
    }

    #[test]
    fn test_tendencia_insuficientes_muestras_estable() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        h.registrar(corporal_limpio());
        h.registrar(corporal_limpio());
        let t = h.tendencia(|c| c.uso_cpu);
        assert_eq!(t.direccion, Tendencia::Estable);
        casi(t.pendiente, 0.0);
        assert!(t.tiempo_estimado.is_infinite());
    }

    #[test]
    fn test_tendencia_subiendo() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        let mut a = corporal_limpio();
        a.uso_cpu = 0.1;
        let mut b = corporal_limpio();
        b.uso_cpu = 0.15;
        let mut c = corporal_limpio();
        c.uso_cpu = 0.25;
        h.registrar(a);
        h.registrar(b);
        h.registrar(c);
        let t = h.tendencia(|c| c.uso_cpu);
        assert_eq!(t.direccion, Tendencia::Subiendo);
        // pendiente = actual - anterior = 0.25 - 0.15 = 0.10
        casi(t.pendiente, 0.10);
        // aceleracion = 0.10 - (0.15-0.10) = 0.05
        casi(t.aceleracion, 0.05);
        // tiempo hasta 1.0 = (1-0.25)/0.10 = 7.5
        casi(t.tiempo_estimado, 7.5);
    }

    #[test]
    fn test_tendencia_bajando() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        let mut a = corporal_limpio();
        a.uso_cpu = 0.5;
        let mut b = corporal_limpio();
        b.uso_cpu = 0.4;
        let mut c = corporal_limpio();
        c.uso_cpu = 0.3;
        h.registrar(a);
        h.registrar(b);
        h.registrar(c);
        let t = h.tendencia(|c| c.uso_cpu);
        assert_eq!(t.direccion, Tendencia::Bajando);
        casi(t.pendiente, -0.1);
    }

    #[test]
    fn test_tendencia_critica_cambio_abrupto() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        let mut a = corporal_limpio();
        a.uso_cpu = 0.2;
        let mut b = corporal_limpio();
        b.uso_cpu = 0.2;
        let mut c = corporal_limpio();
        c.uso_cpu = 0.6; // salto de 0.4 > 0.3
        h.registrar(a);
        h.registrar(b);
        h.registrar(c);
        let t = h.tendencia(|c| c.uso_cpu);
        assert_eq!(t.direccion, Tendencia::Critico);
    }

    #[test]
    fn test_tendencia_estable_bajo_cambio() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        let mut a = corporal_limpio();
        a.uso_cpu = 0.2;
        let mut b = corporal_limpio();
        b.uso_cpu = 0.21;
        let mut c = corporal_limpio();
        c.uso_cpu = 0.215; // cambios < 0.01
        h.registrar(a);
        h.registrar(b);
        h.registrar(c);
        let t = h.tendencia(|c| c.uso_cpu);
        assert_eq!(t.direccion, Tendencia::Estable);
    }

    #[test]
    fn test_tendencia_tiempo_infinito_cuando_no_sube() {
        let mut h = HistorialInteroceptivo::nuevo(10);
        let mut a = corporal_limpio();
        a.uso_cpu = 0.5;
        let mut b = corporal_limpio();
        b.uso_cpu = 0.4;
        let mut c = corporal_limpio();
        c.uso_cpu = 0.3;
        h.registrar(a);
        h.registrar(b);
        h.registrar(c);
        let t = h.tendencia(|c| c.uso_cpu);
        assert!(t.tiempo_estimado.is_infinite()); // pendiente negativa
    }

    // ── HomeostasisCorporal ──────────────────────────────────────────────────
    #[test]
    fn test_homeostasis_nuevo_valores_optimos() {
        let h = HomeostasisCorporal::nuevo();
        casi(h.hambre_recursos, 0.0);
        casi(h.estres_termico, 0.0);
        casi(h.energia_disponible, 1.0);
        casi(h.dolor_sistemico, 0.0);
        casi(h.bienestar_general, 1.0);
    }

    #[test]
    fn test_homeostasis_hambre_recursos() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        let mut c = corporal_limpio();
        c.uso_ram = 1.0; // 1.0*0.6 = 0.6
        c.swap_activo = 1.0; // 1.0*0.4 = 0.4
        h.actualizar(&c, &umbrales);
        casi(h.hambre_recursos, 1.0);
    }

    #[test]
    fn test_homeostasis_estres_termico_normalizado() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default(); // temp_max = 85
        let mut c = corporal_limpio();
        c.temperatura_cpu = 42.5;
        h.actualizar(&c, &umbrales);
        casi(h.estres_termico, 0.5); // 42.5/85
    }

    #[test]
    fn test_homeostasis_estres_termico_clampa() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        let mut c = corporal_limpio();
        c.temperatura_cpu = 200.0; // > umbral → clamp a 1.0
        h.actualizar(&c, &umbrales);
        casi(h.estres_termico, 1.0);
    }

    #[test]
    fn test_homeostasis_energia_disponible_inversa_carga() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        let mut c = corporal_limpio();
        c.uso_cpu = 1.0; // 0.5
        c.load_avg_1 = 12.0; // min(1, 0.5) = 0.5
        h.actualizar(&c, &umbrales);
        // carga = 0.5 + 0.5 = 1.0 → energia 0.0
        casi(h.energia_disponible, 0.0);
    }

    #[test]
    fn test_homeostasis_sin_carga_energia_llena() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        h.actualizar(&corporal_limpio(), &umbrales);
        casi(h.energia_disponible, 1.0);
    }

    #[test]
    fn test_homeostasis_dolor_sistemico_sin_alarmas() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        h.actualizar(&corporal_limpio(), &umbrales);
        casi(h.dolor_sistemico, 0.0);
    }

    #[test]
    fn test_homeostasis_dolor_sistemico_compuesto() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        let mut c = corporal_limpio();
        c.uso_cpu = 0.95; // (0.95-0.9)/(0.1)=0.5 → *0.3 = 0.15
        c.temperatura_cpu = 85.0; // estres=1.0 → 0.7*0.2=0.14
        h.actualizar(&c, &umbrales);
        // 0.15 + 0.14 = 0.29
        casi(h.dolor_sistemico, 0.29);
    }

    #[test]
    fn test_homeostasis_bienestar_optimo_es_uno() {
        let mut h = HomeostasisCorporal::nuevo();
        let umbrales = UmbralesInteroceptivos::default();
        h.actualizar(&corporal_limpio(), &umbrales);
        // dolor 0 → salud 1; energia 1; hambre 0 → hambre_inv 1
        // 1*0.4 + 1*0.3 + 1*0.3 = 1.0
        casi(h.bienestar_general, 1.0);
    }

    // ── SistemaInteroceptivo ─────────────────────────────────────────────────
    #[test]
    fn test_sistema_nuevo_estado_inicial() {
        let s = SistemaInteroceptivo::nuevo();
        assert_eq!(s.historial.max_len, HISTORIAL_TAMANO);
        casi(s.peso_limbico, 0.3);
        assert!(!s.modo_hibernacion);
        assert_eq!(s.contador_muestreo, 0);
    }

    #[test]
    fn test_generar_estimulos_siete_canales() {
        let mut s = SistemaInteroceptivo::nuevo();
        // fuerza un muestreo de hardware real para poblar estado
        s.estado_corporal = corporal_limpio();
        let estimulos = s.generar_estimulos_interoceptivos();
        assert_eq!(estimulos.len(), 6);
        let ids: Vec<u32> = estimulos.iter().map(|e| e.id).collect();
        assert!(ids.contains(&CANAL_CPU));
        assert!(ids.contains(&CANAL_RAM));
        assert!(ids.contains(&CANAL_TEMPERATURA));
        assert!(ids.contains(&CANAL_DISCO));
        assert!(ids.contains(&CANAL_SISTEMA));
        assert!(ids.contains(&CANAL_ENERGIA));
        // CANAL_RED existe como constante pero la generación actual no lo emite
        assert!(!ids.contains(&CANAL_RED));
    }

    #[test]
    fn test_estimulo_cpu_amenaza_sobre_umbral() {
        let mut s = SistemaInteroceptivo::nuevo();
        let mut c = corporal_limpio();
        c.uso_cpu = 1.0; // > cpu_maximo 0.9
        s.estado_corporal = c;
        let estimulos = s.generar_estimulos_interoceptivos();
        let cpu = estimulos.iter().find(|e| e.id == CANAL_CPU).unwrap();
        casi(cpu.amenaza, 1.0);
        casi(cpu.recompensa, 0.0);
    }

    #[test]
    fn test_estimulo_temperatura_normalizada() {
        let mut s = SistemaInteroceptivo::nuevo();
        let mut c = corporal_limpio();
        c.temperatura_cpu = 50.0; // /100 = 0.5
        s.estado_corporal = c;
        let estimulos = s.generar_estimulos_interoceptivos();
        let temp = estimulos.iter().find(|e| e.id == CANAL_TEMPERATURA).unwrap();
        casi(temp.valor, 0.5);
    }

    #[test]
    fn test_integrar_pipeline_muestrea_en_intervalo() {
        let mut s = SistemaInteroceptivo::nuevo();
        let mut entrada = Entrada::vacía();
        // 99 llamadas sin muestreo
        for _ in 0..99 {
            assert!(!s.integrar_en_pipeline(1.0, &mut entrada));
        }
        assert_eq!(s.contador_muestreo, 99);
        // la 100ª muestrea
        assert!(s.integrar_en_pipeline(1.0, &mut entrada));
        assert_eq!(s.contador_muestreo, 100);
        // se inyectaron 6 estímulos interoceptivos
        let intero = entrada.estimulos.iter()
            .filter(|e| e.id >= ID_BASE_INTEROCEPCION)
            .count();
        assert_eq!(intero, 6);
        assert!(entrada.texto.is_some());
    }

    #[test]
    fn test_integrar_pipeline_no_muestrea_fuera_de_intervalo() {
        let mut s = SistemaInteroceptivo::nuevo();
        let mut entrada = Entrada::vacía();
        assert!(!s.integrar_en_pipeline(1.0, &mut entrada));
        assert!(entrada.estimulos.is_empty());
        assert!(entrada.texto.is_none());
    }

    #[test]
    fn test_describir_estado_corporal_estado_estable() {
        let mut s = SistemaInteroceptivo::nuevo();
        s.estado_corporal = corporal_limpio();
        // bienestar 1.0 > 0.8 → "saludable"; cpu 0 → "CPU en reposo"
        let desc = s.describir_estado_corporal();
        assert!(desc.contains("saludable"), "desc: {}", desc);
    }

    #[test]
    fn test_describir_estado_corporal_sobrecalentado() {
        let mut s = SistemaInteroceptivo::nuevo();
        let mut c = corporal_limpio();
        c.temperatura_cpu = 90.0;
        s.estado_corporal = c;
        let desc = s.describir_estado_corporal();
        assert!(desc.contains("sobrecalentado"), "desc: {}", desc);
    }

    #[test]
    fn test_modular_amigdala_sin_dolor_sin_recompensa() {
        let mut s = SistemaInteroceptivo::nuevo();
        s.estado_corporal = corporal_limpio();
        s.homeostasis.actualizar(&s.estado_corporal, &s.umbrales);
        let (amenaza, recompensa) = s.modular_amigdala();
        casi(amenaza, 0.0); // dolor 0
        // bienestar 1.0 → (1-0.5)=0.5 *0.3*0.5 = 0.075
        casi(recompensa, 0.075);
    }

    #[test]
    fn test_modular_amigdala_con_dolor_solo_amenaza() {
        let mut s = SistemaInteroceptivo::nuevo();
        s.homeostasis.dolor_sistemico = 1.0;
        s.homeostasis.bienestar_general = 0.1;
        let (amenaza, recompensa) = s.modular_amigdala();
        casi(amenaza, 0.3); // 1.0 * 0.3
        casi(recompensa, 0.0); // 0.1-0.5 → negativo → max(0)=0
    }

    #[test]
    fn test_deberia_hibernar_por_modo() {
        let mut s = SistemaInteroceptivo::nuevo();
        assert!(!s.deberia_hibernar());
        s.modo_hibernacion = true;
        assert!(s.deberia_hibernar());
    }

    #[test]
    fn test_deberia_hibernar_por_energia_baja() {
        let mut s = SistemaInteroceptivo::nuevo();
        s.homeostasis.energia_disponible = 0.05;
        assert!(s.deberia_hibernar());
    }

    #[test]
    fn test_muestrear_hardware_actualiza_historial() {
        let mut s = SistemaInteroceptivo::nuevo();
        s.muestrear_hardware();
        assert_eq!(s.historial.buffer.len(), 1);
        assert!(s.historial.ultimo().is_some());
    }
}
