//! core/src/cerebro/synapse/types.rs
//! Tipos de datos fundamentales para la sinapsis y el grafo conceptual de NEXUS.

use std::collections::HashMap;

/// Identificador de un nodo en el grafo sináptico.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IDNodo {
    Concepto(String),
    /// Contador de refuerzo identitario: sube con menciones al sistema.
    /// Efecto: fortalece sinapsis entre palabras en contexto de autoreconocimiento.
    RefuerzoIdentidad,
    /// Medidor de tensión: sube con señales de error/conflicto/amenaza.
    /// Efecto: umbral de habla más alto (escucha más), poda más agresiva.
    Tension,
    /// Tono afectivo global del sistema [-1.0..1.0].
    /// Modula dirección del aprendizaje: tono positivo refuerza, negativo debilita.
    TonoGlobal,
    /// Inhibición global: acumulador de señal inhibitoria GABAérgica.
    /// Cuando un concepto "gana" la competencia, inyecta energía aquí
    /// y esto reduce la energía de los demás conceptos (circuito WTA).
    /// Efecto: selecciona UNA respuesta coherente en vez de ruido múltiple.
    Inhibicion,
    /// Intención autónoma persistente: meta actual que guía el comportamiento.
    /// Equivale a la corteza prefrontal medial — sostiene una "meta" entre ciclos.
    /// Cuando no hay input externo, el sistema rumia alrededor de esta intención.
    Intencion(String),
    /// Curiosidad: energía interna que impulsa exploración de nodos con baja traza.
    /// Equivale al VTA/SNc — genera pulso dopaminérgico hacia la novedad.
    Curiosidad,
    /// Contador de ciclos sin input externo para activar rumia autónoma.
    /// Es un nodo "tick" que incrementa cada vez que procesar() se llama sin prompt.
    CicloInterno,
}

impl IDNodo {
    pub fn a_string(&self) -> String {
        match self {
            IDNodo::Concepto(s) => format!("concepto:{}", s),
            IDNodo::RefuerzoIdentidad => "refuerzo_identidad".to_string(),
            IDNodo::Tension => "tension".to_string(),
            IDNodo::TonoGlobal => "tono_global".to_string(),
            IDNodo::Inhibicion => "inhibicion".to_string(),
            IDNodo::Intencion(s) => format!("intencion:{}", s),
            IDNodo::Curiosidad => "curiosidad".to_string(),
            IDNodo::CicloInterno => "ciclo_interno".to_string(),
        }
    }

    pub fn desde_string(s: &str) -> Self {
        if s.starts_with("concepto:") {
            IDNodo::Concepto(s["concepto:".len()..].to_string())
        } else if s == "refuerzo_identidad" {
            IDNodo::RefuerzoIdentidad
        } else if s == "tension" {
            IDNodo::Tension
        } else if s == "inhibicion" {
            IDNodo::Inhibicion
        } else if s.starts_with("intencion:") {
            IDNodo::Intencion(s["intencion:".len()..].to_string())
        } else if s == "curiosidad" {
            IDNodo::Curiosidad
        } else if s == "ciclo_interno" {
            IDNodo::CicloInterno
        } else {
            IDNodo::TonoGlobal // Default o error
        }
    }
}

/// Enlace sináptico
#[derive(Debug, Clone)]
pub struct EnlaceSinaptico {
    pub peso: f32, // Rango [-1.0, 1.0]
}

/// Nodo sináptico con su nivel de activación (energía), refractariedad e historial de disparo (STDP)
#[derive(Debug, Clone)]
pub struct NodoSinaptico {
    pub id: IDNodo,
    pub energia: f32,
    pub palabra: String,
    pub refractario: f32,
    pub ultimo_disparo: u64, // Turno o paso de simulación del último disparo para STDP
    pub traza: f32,          // Traza de actividad reciente [0..1] con decaimiento exponencial (τ=3)
    pub es_predicho: bool,   // Si fue activado por predicción (no por entrada directa)
    pub es_entrada_directa: bool, // Si fue creado por entrada sensorial directa (no repetir en fonación)
}

pub struct GrafoSinapsis {
    pub nodos: HashMap<IDNodo, NodoSinaptico>,
    pub enlaces: HashMap<IDNodo, Vec<(IDNodo, EnlaceSinaptico)>>,
    pub ciclo_actual: u64, // Para el tracking de `ultimo_disparo`
}

impl GrafoSinapsis {
    pub fn new() -> Self {
        let mut gs = Self {
            nodos: HashMap::new(),
            enlaces: HashMap::new(),
            ciclo_actual: 0,
        };
        gs.inicializar_nodos_quimicos();
        gs
    }

    fn inicializar_nodos_quimicos(&mut self) {
        // Inicializar los 7 nodos bioquímicos + motivacionales del sistema
        let nodos_iniciales = vec![
            (IDNodo::RefuerzoIdentidad, 0.0, "refuerzo_identidad"),
            (IDNodo::Tension, 0.0, "tension"),
            (IDNodo::TonoGlobal, 0.5, "tono_global"),
            (IDNodo::Inhibicion, 0.0, "inhibicion"),
            (IDNodo::Intencion("explorar".to_string()), 0.3, "intencion"),
            (IDNodo::Curiosidad, 0.2, "curiosidad"),
            (IDNodo::CicloInterno, 0.0, "ciclo_interno"),
        ];
        for (id, energia, palabra) in nodos_iniciales {
            self.nodos.insert(
                id.clone(),
                NodoSinaptico {
                    id,
                    energia,
                    palabra: palabra.to_string(),
                    refractario: 0.0,
                    ultimo_disparo: 0,
                    traza: 0.0,
                    es_predicho: false,
                    es_entrada_directa: false,
                },
            );
        }
    }
}
