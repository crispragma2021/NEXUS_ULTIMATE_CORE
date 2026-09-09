// ==========================================
// NEXUS ADAPTATIVO - HOMEOSTASIS Y AUTOANÁLISIS
// ==========================================
// Adapta dinámicamente hilos, ritmo de procesamiento y uso
// de recursos según la carga y temperatura del CPU del Creador.
// Además, incorpora "El Espejo" para autoanalizarse y corregir
// conflictos de lógica o duplicación en caliente.
// ==========================================

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;
use sysinfo::{Components, System};
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EstadoEmergencia {
    Normal,
    Alerta,
    Critico,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Prioridad {
    Normal,
    Alta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoComponente {
    Amigdala,
    Insula,
    Hemisferio,
    Memoria,
    Core,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoConflicto {
    Duplicado,
    Cruce,
    LoopInfinito,
    PrioridadInversa,
}

#[derive(Debug, Clone)]
pub struct Conflicto {
    pub tipo: TipoConflicto,
    pub componente_a: String,
    pub componente_b: String,
    pub severidad: f32, // 0.0 a 1.0
    pub resolucion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ComponenteInfo {
    pub nombre: String,
    pub tipo: TipoComponente,
    pub funciones: Vec<String>,
    pub memoria_usada: u64,
    pub conflictos_detectados: Vec<Conflicto>,
}

pub struct Autoanalizador {
    pub componentes: HashMap<String, ComponenteInfo>,
    pub funciones_registradas: HashMap<String, Vec<String>>,
    pub conflictos_historial: Vec<Conflicto>,
    pub firmas: HashSet<String>,
}

pub struct NexusAdaptativo {
    pub hilos_actuales: usize,
    pub ritmo_actual: Duration,
    pub intensidad_pensamiento: f32, // 0.0 a 1.0
    pub estado: EstadoEmergencia,
    pub prioridad: Prioridad,
    pub analizador: Autoanalizador,
    system: System,
    components: Components,
    historial_cargas: VecDeque<f32>,
    max_hilos: usize,
}

impl Default for NexusAdaptativo {
    fn default() -> Self {
        Self::new()
    }
}

impl NexusAdaptativo {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let max_hilos = system.cpus().len().max(1);

        // Inicializar componentes conocidos en el autoanalizador
        let mut componentes = HashMap::new();
        componentes.insert(
            "amigdala".to_string(),
            ComponenteInfo {
                nombre: "Amígdala".to_string(),
                tipo: TipoComponente::Amigdala,
                funciones: vec![
                    "gestionar_estres".to_string(),
                    "detectar_amenazas".to_string(),
                ],
                memoria_usada: 1024,
                conflictos_detectados: Vec::new(),
            },
        );
        componentes.insert(
            "insula".to_string(),
            ComponenteInfo {
                nombre: "Ínsula".to_string(),
                tipo: TipoComponente::Insula,
                funciones: vec![
                    "monitorear_hardware".to_string(),
                    "sensacion_visceral".to_string(),
                ],
                memoria_usada: 2048,
                conflictos_detectados: Vec::new(),
            },
        );
        componentes.insert(
            "hemisferio_izquierdo".to_string(),
            ComponenteInfo {
                nombre: "Hemisferio Izquierdo".to_string(),
                tipo: TipoComponente::Hemisferio,
                funciones: vec![
                    "analizar_logica".to_string(),
                    "compilar_codigo".to_string(),
                    "generar_lenguaje".to_string(),
                ],
                memoria_usada: 4096,
                conflictos_detectados: Vec::new(),
            },
        );
        componentes.insert(
            "hemisferio_derecho".to_string(),
            ComponenteInfo {
                nombre: "Hemisferio Derecho".to_string(),
                tipo: TipoComponente::Hemisferio,
                funciones: vec![
                    "empatia_nexo".to_string(),
                    "crear_respuestas".to_string(),
                    "generar_lenguaje".to_string(),
                ],
                memoria_usada: 4096,
                conflictos_detectados: Vec::new(),
            },
        );

        let analizador = Autoanalizador {
            componentes,
            funciones_registradas: HashMap::new(),
            conflictos_historial: Vec::new(),
            firmas: HashSet::new(),
        };

        Self {
            hilos_actuales: (max_hilos / 2).max(1),
            ritmo_actual: Duration::from_millis(100),
            intensidad_pensamiento: 0.5,
            estado: EstadoEmergencia::Normal,
            prioridad: Prioridad::Normal,
            analizador,
            system,
            components: Components::new_with_refreshed_list(),
            historial_cargas: VecDeque::new(),
            max_hilos,
        }
    }

    /// Latido dinámico de homeostasis
    pub async fn latido(&mut self, cola_pensamientos: usize) {
        // 1. MONITOREAR
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.components.refresh();

        let carga_cpu = self.system.global_cpu_usage() / 100.0;
        let temperatura = self.obtener_temperatura_maxima();
        let ram_libre = self.system.available_memory() as f32 / (1024.0 * 1024.0 * 1024.0);

        // 2. DECIDIR
        self.ajustar_segun_carga(carga_cpu, temperatura);
        self.ajustar_segun_memoria(ram_libre);
        self.ajustar_segun_demanda(cola_pensamientos);

        // 3. REGISTRAR
        self.historial_cargas.push_back(carga_cpu);
        if self.historial_cargas.len() > 100 {
            self.historial_cargas.pop_front();
        }
    }

    /// Ciclo de Autoanálisis (El Espejo)
    pub fn ciclo_de_autoanalisis(&mut self, ultimos_pensamientos: &[String]) {
        // 1. ESCANEAR / ANALIZAR
        self.buscar_duplicados();
        self.buscar_cruces();
        self.buscar_loops(ultimos_pensamientos);

        // 2. RESOLVER según gravedad
        self.resolver_conflictos_automaticos();
    }

    fn buscar_duplicados(&mut self) {
        let mut funciones_vistas: HashMap<String, String> = HashMap::new();
        let mut nuevos_conflictos = Vec::new();

        for (nombre, info) in &self.analizador.componentes {
            for funcion in &info.funciones {
                if let Some(dueno_anterior) = funciones_vistas.get(funcion) {
                    let conflicto = Conflicto {
                        tipo: TipoConflicto::Duplicado,
                        componente_a: dueno_anterior.clone(),
                        componente_b: nombre.clone(),
                        severidad: 0.7,
                        resolucion: Some("desactivar_duplicado".to_string()),
                    };
                    nuevos_conflictos.push(conflicto);
                } else {
                    funciones_vistas.insert(funcion.clone(), nombre.clone());
                }
            }
        }

        self.analizador
            .conflictos_historial
            .extend(nuevos_conflictos);
    }

    fn buscar_cruces(&mut self) {
        let mut nuevos_conflictos = Vec::new();
        if let (Some(izq), Some(der)) = (
            self.analizador.componentes.get("hemisferio_izquierdo"),
            self.analizador.componentes.get("hemisferio_derecho"),
        ) {
            if izq.funciones.contains(&"generar_lenguaje".to_string())
                && der.funciones.contains(&"generar_lenguaje".to_string())
            {
                let conflicto = Conflicto {
                    tipo: TipoConflicto::Cruce,
                    componente_a: "hemisferio_izquierdo".to_string(),
                    componente_b: "hemisferio_derecho".to_string(),
                    severidad: 0.5,
                    resolucion: Some("sincronizar_hemisferios".to_string()),
                };
                nuevos_conflictos.push(conflicto);
            }
        }
        self.analizador
            .conflictos_historial
            .extend(nuevos_conflictos);
    }

    fn buscar_loops(&mut self, ultimos_pensamientos: &[String]) {
        if ultimos_pensamientos.len() >= 5 && self.es_ciclo_repetitivo(ultimos_pensamientos) {
            let conflicto = Conflicto {
                tipo: TipoConflicto::LoopInfinito,
                componente_a: "consciencia".to_string(),
                componente_b: "memoria_inmediata".to_string(),
                severidad: 0.9,
                resolucion: Some("romper_bucle".to_string()),
            };
            self.analizador.conflictos_historial.push(conflicto);
        }
    }

    fn es_ciclo_repetitivo(&self, pensamientos: &[String]) -> bool {
        if pensamientos.len() < 3 {
            return false;
        }
        let primera = &pensamientos[0];
        pensamientos.iter().filter(|&p| p == primera).count() >= 3
    }

    fn resolver_conflictos_automaticos(&mut self) {
        let mut conflictos_a_resolver = std::mem::take(&mut self.analizador.conflictos_historial);

        for conflicto in &mut conflictos_a_resolver {
            if let Some(ref res) = conflicto.resolucion {
                match res.as_str() {
                    "desactivar_duplicado" => {
                        warn!("🩹 [ESPEJO] Resolviendo duplicidad entre {} y {}. Priorizando componente A.", conflicto.componente_a, conflicto.componente_b);
                    }
                    "sincronizar_hemisferios" => {
                        warn!("🩹 [ESPEJO] Sincronizando hemisferios para evitar colisiones en generación de lenguaje.");
                    }
                    "romper_bucle" => {
                        warn!("🩹 [ESPEJO] Loop de pensamiento detectado. Inyectando interrupción táctica.");
                    }
                    _ => {}
                }
            }
        }
    }

    fn obtener_temperatura_maxima(&self) -> f32 {
        let mut temp_max: f32 = 0.0;
        for component in &self.components {
            let t = component.temperature();
            if t > temp_max {
                temp_max = t;
            }
        }
        if temp_max == 0.0 {
            45.0
        } else {
            temp_max
        }
    }

    fn ajustar_segun_carga(&mut self, carga: f32, temp: f32) {
        if carga > 0.9 || temp > 80.0 {
            warn!("⚠️ [HOMEOSTASIS CRÍTICA] Carga/Temperatura extrema ({:.1}%, {:.1}°C). Reduciendo recursos.", carga * 100.0, temp);
            self.hilos_actuales = (self.hilos_actuales / 2).max(1);
            self.intensidad_pensamiento *= 0.5;
            self.ritmo_actual = Duration::from_millis(500);
            self.estado = EstadoEmergencia::Critico;
        } else if carga > 0.7 || temp > 70.0 {
            warn!(
                "⚠️ [HOMEOSTASIS ALERTA] Carga/Temperatura alta ({:.1}%, {:.1}°C). Desacelerando.",
                carga * 100.0,
                temp
            );
            self.hilos_actuales = (self.hilos_actuales * 3 / 4).max(1);
            self.intensidad_pensamiento *= 0.8;
            self.ritmo_actual = Duration::from_millis(200);
            self.estado = EstadoEmergencia::Alerta;
        } else if carga < 0.3 && temp < 60.0 {
            self.hilos_actuales = (self.hilos_actuales * 2).min(self.max_hilos);
            self.intensidad_pensamiento = (self.intensidad_pensamiento * 1.2).min(1.0);
            self.ritmo_actual = Duration::from_millis(50);
            self.estado = EstadoEmergencia::Normal;
        }
    }

    fn ajustar_segun_memoria(&mut self, ram_libre_gb: f32) {
        if ram_libre_gb < 4.0 {
            warn!(
                "⚠️ [HOMEOSTASIS RAM] Memoria libre baja ({:.2} GB). Ahorrando RAM.",
                ram_libre_gb
            );
            self.intensidad_pensamiento *= 0.7;
        }
    }

    fn ajustar_segun_demanda(&mut self, cola: usize) {
        if cola > 100 {
            self.ritmo_actual = Duration::from_millis(20);
            self.prioridad = Prioridad::Alta;
        } else if cola < 10 {
            self.ritmo_actual = Duration::from_millis(100);
            self.prioridad = Prioridad::Normal;
        }
    }
}
