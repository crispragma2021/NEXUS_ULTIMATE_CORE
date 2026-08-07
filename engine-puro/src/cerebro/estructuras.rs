// ============================================================================
// ESTRUCTURAS DE DATOS COMPACTAS DEL CEREBRO DIGITAL
// ============================================================================
// Diseñadas para:
// - NeuronaCompacta: exactamente 64 bytes (una línea de caché)
// - SinapsisCompacta: exactamente 8 bytes
// - Episodio: exactamente 64 bytes
// ============================================================================

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// NEURONA COMPACTA (64 bytes)
// ============================================================================
// Estado de Hodgkin-Huxley + marcadores biológicos
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronaCompacta {
    // Hodgkin-Huxley (20 bytes)
    pub voltaje: f32,           // Potencial de membrana (-70 a +40 mV)
    pub m: f32,                 // Compuerta de activación Na+ (0-1)
    pub h: f32,                 // Compuerta de inactivación Na+ (0-1)
    pub n: f32,                 // Compuerta de activación K+ (0-1)
    pub corriente_entrada: f32, // Corriente sináptica entrante

    // Estado biológico (20 bytes)
    pub energia: f32,           // Nivel energético (0.0 a 1.0)
    pub traza: f32,             // Traza de plasticidad (0.0 a 1.0)
    pub refractario: f32,       // Periodo refractario (0.0 a 1.0)
    pub activacion: f32,        // Nivel de activación (0.0 a 1.0)
    pub frecuencia: f32,        // Tasa de disparo (Hz)

    // Metadatos (24 bytes)
    pub id: u32,                // Índice único en la población
    pub tipo: u8,               // 0=excitatoria, 1=inhibitoria
    pub capa: u8,               // Capa cortical (0-5)
    pub edad: u16,              // Ciclos de vida
    pub reservado: [u8; 15],    // Padding para completar 64 bytes
}

impl NeuronaCompacta {
    /// Crea una neurona con estado basal (potencial de reposo -70mV)
    pub fn reposo(id: u32, tipo: u8, capa: u8) -> Self {
        Self {
            voltaje: -70.0,
            m: 0.05,
            h: 0.6,
            n: 0.32,
            corriente_entrada: 0.0,
            energia: 0.1,
            traza: 0.0,
            refractario: 0.0,
            activacion: 0.0,
            frecuencia: 0.0,
            id,
            tipo,
            capa,
            edad: 0,
            reservado: [0; 15],
        }
    }

    /// Crea una neurona con valores aleatorios (para inicialización)
    pub fn aleatoria(id: u32, tipo: u8, capa: u8, rng: &mut impl FnMut() -> f32) -> Self {
        Self {
            voltaje: -70.0 + rng() * 20.0,
            m: rng() * 0.5,
            h: rng() * 0.5,
            n: rng() * 0.5,
            corriente_entrada: 0.0,
            energia: rng() * 0.3,
            traza: 0.0,
            refractario: 0.0,
            activacion: 0.0,
            frecuencia: 0.0,
            id,
            tipo,
            capa,
            edad: 0,
            reservado: [0; 15],
        }
    }

    /// ¿La neurona está en periodo refractario?
    pub fn en_refractario(&self) -> bool {
        self.refractario > 0.1
    }

    /// ¿La neurona está activa?
    /// La dinámica de activación ahora es influenciada por la neuroquímica
    pub fn esta_activa(&self, dopamina: f32) -> bool {
        // La dopamina baja el umbral de activación (más proactividad)
        let umbral_base = 0.3;
        let umbral_ajustado = (umbral_base - (dopamina * 0.1)).clamp(0.1, 0.4);
        self.activacion > umbral_ajustado && !self.en_refractario()
    }

    /// Calcula el próximo estado de voltaje incluyendo la modulación química y metabólica
    pub fn integrar_quimica(&mut self, cortisol: f32, adrenalina: f32) {
        // --- Tasa Metabólica Basal (Mantenimiento de Gradientes) ---
        // Mantener el potencial de reposo en -70mV cuesta energía constante.
        let costo_mantenimiento = 0.0005;
        self.energia = (self.energia - costo_mantenimiento).clamp(0.0, 1.0);

        // --- Gasto por Actividad Sináptica ---
        // Recibir corriente de entrada consume energía (bombas Na+/K+ trabajando)
        if self.corriente_entrada.abs() > 0.1 {
            self.energia = (self.energia - (self.corriente_entrada.abs() * 0.002)).clamp(0.0, 1.0);
        }

        // --- GASTO POR SPIKE (Descarga violenta) ---
        // Si la neurona llega al umbral de disparo (> -40mV iniciando spike)
        if self.voltaje > -40.0 && self.refractario <= 0.0 {
            let costo_spike = 0.05; // Un disparo drena el 5% de la energía total
            self.energia = (self.energia - costo_spike).clamp(0.0, 1.0);
            
            // Si no hay energía para el spike, el disparo es 'fallido' (no hay neurotransmisores)
            if self.energia < 0.02 {
                self.voltaje = -60.0; // Colapso del spike
                self.activacion *= 0.1;
                self.refractario = 1.0; // Agotamiento total
            }
        }

        // --- Fatiga Metabólica y Recuperación ---
        if self.refractario > 0.0 {
            let recuperacion_base = 0.1;
            let penalizacion_cortisol = cortisol * 0.05;
            // La recuperación biológica es mucho más lenta si no hay energía (hambre)
            let eficiencia_metabolica = if self.energia > 0.5 { 1.2 } else { self.energia * 2.0 };
            self.refractario -= (recuperacion_base * eficiencia_metabolica - penalizacion_cortisol).clamp(0.001, 0.2);
        }

        // La adrenalina aumenta la frecuencia base pero acelera el desgaste
        if adrenalina > 0.7 {
            self.frecuencia += 2.0;
            self.energia -= 0.001; // Costo por estrés
        }
    }

    /// Recarga energía a la neurona (Glucosa Digital)
    pub fn alimentar(&mut self, cantidad: f32) {
        self.energia = (self.energia + cantidad).min(1.0);
    }
}

impl Default for NeuronaCompacta {
    fn default() -> Self {
        Self::reposo(0, 0, 0)
    }
}

// ============================================================================
// SINAPSIS COMPACTA (8 bytes)
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SinapsisCompacta {
    pub destino: u32,
    pub peso: f32,
}

impl SinapsisCompacta {
    pub fn nueva(destino: u32, peso: f32) -> Self {
        Self {
            destino,
            peso: peso.clamp(-1.0, 1.0),
        }
    }

    pub fn es_valida(&self) -> bool {
        self.peso.abs() > 0.001
    }
}

// ============================================================================
// EPISODIO (Memoria) (64 bytes)
// ============================================================================

#[repr(C)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Episodio {
    pub timestamp: f32,
    pub intensidad: f32,
    pub emocion: f32,
    pub relevancia: f32,
    pub patron: [u32; 8],
    pub contexto: u64,
}

impl Episodio {
    pub fn nueva(
        timestamp: f32,
        intensidad: f32,
        emocion: f32,
        patron: &[u32],
        contexto: u64,
    ) -> Self {
        let mut p = [0u32; 8];
        for (i, &id) in patron.iter().take(8).enumerate() {
            p[i] = id;
        }
        let relevancia = intensidad * (0.5 + emocion.abs() * 0.5);
        Self {
            timestamp,
            intensidad,
            emocion,
            relevancia: relevancia.clamp(0.0, 1.0),
            patron: p,
            contexto,
        }
    }

    pub fn similitud_patron(&self, otros: &[u32]) -> f32 {
        let mut coincidencias = 0;
        for &x in &self.patron {
            if otros.contains(&x) {
                coincidencias += 1;
            }
        }
        coincidencias as f32 / self.patron.len() as f32
    }
}

// ============================================================================
// PARÁMETROS NEURONALES (Hodgkin-Huxley)
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ParametrosNeurona {
    pub g_na: f32,
    pub g_k: f32,
    pub g_l: f32,
    pub e_na: f32,
    pub e_k: f32,
    pub e_l: f32,
}

impl Default for ParametrosNeurona {
    fn default() -> Self {
        Self {
            g_na: 120.0,
            g_k: 36.0,
            g_l: 0.3,
            e_na: 50.0,
            e_k: -77.0,
            e_l: -54.4,
        }
    }
}

impl ParametrosNeurona {
    pub fn inhibitorio() -> Self {
        Self {
            g_na: 60.0,
            g_k: 18.0,
            g_l: 0.3,
            e_na: 50.0,
            e_k: -77.0,
            e_l: -54.4,
        }
    }
}

// ============================================================================
// PARÁMETROS STDP
// ============================================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ParametrosSTDP {
    pub a_plus: f32,
    pub a_minus: f32,
    pub tau_plus: f32,
    pub tau_minus: f32,
    pub decaimiento: f32,
    pub plasticidad_critica: f32,
}

impl Default for ParametrosSTDP {
    fn default() -> Self {
        Self {
            a_plus: 0.1,
            a_minus: 0.1,
            tau_plus: 20.0,
            tau_minus: 20.0,
            decaimiento: 0.001,
            plasticidad_critica: 1.0,
        }
    }
}

// ============================================================================
// TIPOS DE ENTRADA Y SALIDA
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tematica {
    Operativa,
    Intima,
    Basal,
}

#[derive(Clone, Debug)]
pub struct Estimulo {
    pub id: u32,
    pub intensidad: f32,
    pub amenaza: f32,
    pub recompensa: f32,
    pub valor: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prediccion {
    pub id_objetivo: u32,
    pub valor_esperado: f32,
    pub confianza: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronaTalamica {
    pub id: u32,
    pub voltaje: f32,
    pub umbral: f32,
    pub modo_rafaga: bool,
    pub ultima_actividad: u32,
}

#[derive(Clone, Debug)]
pub struct Entrada {
    pub estimulos: Vec<Estimulo>,
    pub texto: Option<String>,
    /// Valencia de recompensa recibida este tick (0.0 - 1.0)
    pub recompensa: f32,
    /// Nivel de amenaza/estrés percibido este tick (0.0 - 1.0)
    pub amenaza: f32,
}

impl Entrada {
    pub fn vacía() -> Self {
        Self {
            estimulos: Vec::new(),
            texto: None,
            recompensa: 0.0,
            amenaza: 0.0,
        }
    }

    pub fn intensidad_promedio(&self) -> f32 {
        if self.estimulos.is_empty() {
            0.0
        } else {
            self.estimulos.iter().map(|e| e.intensidad).sum::<f32>()
                / self.estimulos.len() as f32
        }
    }

    pub fn es_importante(&self) -> bool {
        self.intensidad_promedio() > 0.5
    }

    pub fn clasificar_tematica(&self) -> Tematica {
        let texto = match &self.texto {
            Some(t) => t.to_lowercase(),
            None => return Tematica::Basal,
        };

        let palabras_operativas = [
            "compila", "test", "cargo", "build", "code", "run", "terminal", 
            "git", "error", "hardware", "memoria", "cpu", "sistema", "código",
            "programa", "ejecutar", "puerto", "api", "mcp", "orquestador", "navegar"
        ];

        let palabras_intimas = [
            "siento", "amor", "íntimo", "familia", "nosotros", "recordar", 
            "amistad", "feliz", "triste", "miedo", "emoción", "sentir", 
            "cariño", "sentimental", "intimo", "emocion", "afecto", "gracias"
        ];

        let mut operativa_count = 0;
        let mut intima_count = 0;

        for palabra in palabras_operativas.iter() {
            if texto.contains(palabra) {
                operativa_count += 1;
            }
        }

        for palabra in palabras_intimas.iter() {
            if texto.contains(palabra) {
                intima_count += 1;
            }
        }

        if operativa_count > intima_count {
            Tematica::Operativa
        } else if intima_count > operativa_count {
            Tematica::Intima
        } else {
            Tematica::Basal
        }
    }
}

#[derive(Clone, Debug)]
pub struct Salida {
    pub texto: String,
    pub emocion: f32,
    pub conciencia: f32,
    pub actividad: Vec<f32>,
    pub corriente: Option<crate::cerebro::lexico::CorrienteConsciencia>,
}

// ============================================================================
// ESTADO DEL SISTEMA
// ============================================================================

#[derive(Clone, Debug)]
pub struct EstadoSistema {
    pub neuronas_activas: usize,
    pub vram_usada: usize,
    pub ram_usada: usize,
    pub intercambios: u64,
    pub tiempo: f32,
}

// ============================================================================
// COLUMNA CORTICAL CANÓNICA DE 6 CAPAS
// ============================================================================
// Inspirada en: Mountcastle (1957), Douglas & Martin (1991, 2004).
// La neocorteza humana está organizada en columnas de ~300-600 µm de diámetro,
// cada una con ~10,000 neuronas distribuidas en 6 capas. Esta es la unidad
// computacional fundamental de la corteza.
// ============================================================================

/// Estados funcionales de una columna cortical
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum EstadoColumna {
    Reposo,
    Activa,
    Supra,
    Silenciada,
}

/// Tipos de conexión horizontal entre columnas vecinas (Capa II/III)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TipoConexionHorizontal {
    Excitatoria,
    Inhibitoria,
}

/// Tipos de comando ejecutivo generados por Capa V
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TipoComando {
    Motor,
    Cognitivo,
    Inhibicion,
}

/// Tipos de neuromoduladores que afectan la columna
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TipoNeuromodulador {
    Ninguno,
    Dopamina,
    Serotonina,
    Noradrenalina,
    Acetilcolina,
}

/// Fibra contextual (Capa I): modulación desde corteza asociativa
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FibraContextual {
    pub origen_columna: u32,
    pub peso: f32,
    pub neuromodulador: TipoNeuromodulador,
}

/// Conexión horizontal entre columnas vecinas (Capa II/III)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConexionHorizontal {
    pub columna_destino: u32,
    pub peso: f32,
    pub tipo: TipoConexionHorizontal,
    pub retardo: f32,
}

/// Sinapsis contextual (Capa I) para modulación de ganancia
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SinapsisContextual {
    pub destino: u32,
    pub modulacion: f32,
}

/// Sinapsis de input (sinapsis estándar para córtex)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SinapsisInput {
    pub destino: u32,
    pub peso: f32,
    pub plasticidad: f32,
}

/// Sinapsis talámica: entrada desde el tálamo a Capa IV
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SinapsisTalamica {
    pub neurona_iv: u32,
    pub peso: f32,
    pub latencia: f32,
}

/// Estimulo talámico filtrado que llega a la columna
#[derive(Clone, Debug, PartialEq)]
pub struct EstimuloTalamico {
    pub origen_talamo: u32,
    pub intensidad: f32,
    pub novedad: f32,
}

/// Comando ejecutivo generado por Capa V
#[derive(Clone, Debug, PartialEq)]
pub struct ComandoEjecutivo {
    pub tipo: TipoComando,
    pub destino: String,
    pub intensidad: f32,
    pub origen_columna: u32,
}

/// Predicción talámica: feedback desde Capa VI al Tálamo
#[derive(Clone, Debug, PartialEq)]
pub struct PrediccionTalamica {
    pub columna_origen: u32,
    pub valor_esperado: f32,
    pub confianza: f32,
}

/// Puerta sensorial: controla qué información pasa al tálamo
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PuertaSensorial {
    pub entrada_id: u32,
    pub abierta: bool,
    pub ganancia: f32,
}

// ============================================================================
// TIPOS NEURONALES CORTICALES ESPECIALIZADOS
// ============================================================================

/// Neurona Piramidal: excitatoria, proyección de larga distancia
/// Predomina en Capas II, III y V.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronaPiramidal {
    pub id: u32,
    pub voltaje: f32,
    pub frecuencia: f32,
    pub campo_dendritico: f32,
    pub objetivo_proyeccion: u8,
    pub reservado: [u8; 6],
}

/// Neurona Estrellada: excitatoria, especializada en input talámico
/// Predomina en Capa IV.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronaEstrellada {
    pub id: u32,
    pub voltaje: f32,
    pub ganancia_talamica: f32,
    pub latencia_sinaptica: f32,
    pub reservado: [u8; 4],
}

/// Neurona Feedback: excitatoria, proyecta al tálamo (Capa VI)
/// Implementa predictive coding: envía predicciones descendentes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronaFeedback {
    pub id: u32,
    pub voltaje: f32,
    pub error_prediccion: f32,
    pub peso_feedback: f32,
    pub conectividad_talamo: u32,
    pub reservado: [u8; 4],
}

/// Neurona Moduladora (Inhibitoria): GABAérgica, control local
/// Presente en todas las capas.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronaModuladora {
    pub id: u32,
    pub voltaje: f32,
    pub objetivo_capa: u8,
    pub modo_inhibicion: u8,
    pub alcance: f32,
    pub reservado: [u8; 6],
}

// ============================================================================
// CAPA CORTICAL INDIVIDUAL (dentro de una columna)
// ============================================================================

/// Una capa cortical dentro de una columna.
#[derive(Clone, Debug)]
pub struct CapaCortical {
    pub id: u8,
    pub neuronas: Vec<NeuronaCompacta>,
    pub conexiones_intra: HashMap<u32, Vec<SinapsisCompacta>>,
    pub conexiones_inter: HashMap<u32, Vec<SinapsisCompacta>>,
    pub activacion_media: f32,
}

impl CapaCortical {
    pub fn nueva(id: u8, cantidad: usize, siguiente_id: &mut u32) -> Self {
        let mut rng = rand::thread_rng();
        let mut neuronas = Vec::with_capacity(cantidad);
        
        for _ in 0..cantidad {
            let tipo = match id {
                0 => 1, // I: inhibitoria (moduladora)
                1 | 2 => if rng.gen::<f32>() > 0.15 { 0 } else { 1 },
                3 => 0, // IV: 100% excitatoria estrellada
                4 => if rng.gen::<f32>() > 0.1 { 0 } else { 1 },
                5 => if rng.gen::<f32>() > 0.2 { 0 } else { 1 },
                _ => 0,
            };
            let neurona = NeuronaCompacta::aleatoria(*siguiente_id, tipo, id, &mut || rng.gen());
            neuronas.push(neurona);
            *siguiente_id += 1;
        }
        
        Self {
            id,
            neuronas,
            conexiones_intra: HashMap::new(),
            conexiones_inter: HashMap::new(),
            activacion_media: 0.0,
        }
    }
    
    pub fn calcular_activacion_media(&mut self) {
        if self.neuronas.is_empty() {
            self.activacion_media = 0.0;
            return;
        }
        let suma: f32 = self.neuronas.iter().map(|n| n.activacion).sum();
        self.activacion_media = suma / self.neuronas.len() as f32;
    }
    
    pub fn spikes(&self) -> Vec<u32> {
        self.neuronas.iter()
            .filter(|n| n.voltaje > 30.0)
            .map(|n| n.id)
            .collect()
    }
}

// ============================================================================
// COLUMNA CORTICAL COMPLETA (6 Capas)
// ============================================================================

/// Columna Cortical Canónica: la unidad fundamental de procesamiento cortical.
#[derive(Clone, Debug)]
pub struct ColumnaCortical {
    pub id: u32,
    pub estado: EstadoColumna,
    pub capa_i: CapaCortical,
    pub capa_ii: CapaCortical,
    pub capa_iii: CapaCortical,
    pub capa_iv: CapaCortical,
    pub capa_v: CapaCortical,
    pub capa_vi: CapaCortical,
    pub conexiones_horizontales: HashMap<u32, Vec<ConexionHorizontal>>,
    pub neuromoduladores: Vec<(TipoNeuromodulador, f32)>,
    pub ultima_prediccion: Option<PrediccionTalamica>,
    pub spike_count: u64,
    pub activacion_sostenida: f32,
    pub spikes_ultimo_ciclo: Vec<u32>,
}

impl ColumnaCortical {
    pub fn nueva(id: u32, total_por_columna: usize, siguiente_id: &mut u32) -> Self {
        let distribucion = [5, 15, 30, 20, 20, 10];
        
        Self {
            id,
            estado: EstadoColumna::Reposo,
            capa_i: CapaCortical::nueva(0, total_por_columna * distribucion[0] / 100, siguiente_id),
            capa_ii: CapaCortical::nueva(1, total_por_columna * distribucion[1] / 100, siguiente_id),
            capa_iii: CapaCortical::nueva(2, total_por_columna * distribucion[2] / 100, siguiente_id),
            capa_iv: CapaCortical::nueva(3, total_por_columna * distribucion[3] / 100, siguiente_id),
            capa_v: CapaCortical::nueva(4, total_por_columna * distribucion[4] / 100, siguiente_id),
            capa_vi: CapaCortical::nueva(5, total_por_columna * distribucion[5] / 100, siguiente_id),
            conexiones_horizontales: HashMap::new(),
            neuromoduladores: Vec::new(),
            ultima_prediccion: None,
            spike_count: 0,
            activacion_sostenida: 0.0,
            spikes_ultimo_ciclo: Vec::new(),
        }
    }
    
    pub fn cablear(&mut self, rng: &mut impl FnMut() -> f32) {
        let conexiones_por_neurona = 12;
        
        macro_rules! conectar_capas {
            ($origen:expr, $destino:expr, $peso_base:expr) => {
                let origen_ids: Vec<u32> = $origen.neuronas.iter().map(|n| n.id).collect();
                let destino_ids: Vec<u32> = $destino.neuronas.iter().map(|n| n.id).collect();
                
                for &oid in &origen_ids {
                    let mut conexiones = Vec::with_capacity(conexiones_por_neurona);
                    for _ in 0..conexiones_por_neurona {
                        if let Some(&did) = destino_ids.get((rng() * destino_ids.len() as f32) as usize) {
                            if oid != did {
                                let peso = ($peso_base + rng() * 0.1) * 0.3;
                                conexiones.push(SinapsisCompacta::nueva(did, peso));
                            }
                        }
                    }
                    if !conexiones.is_empty() {
                        $origen.conexiones_inter.insert(oid, conexiones);
                    }
                }
            };
        }
        
        macro_rules! conectar_intra {
            ($capa:expr, $peso_base:expr) => {
                let ids: Vec<u32> = $capa.neuronas.iter().map(|n| n.id).collect();
                for &id in &ids {
                    let mut conexiones = Vec::with_capacity(6);
                    for _ in 0..6 {
                        if let Some(&did) = ids.get((rng() * ids.len() as f32) as usize) {
                            if id != did && rng() > 0.6 {
                                let peso = $peso_base * (rng() * 0.2 + 0.9);
                                conexiones.push(SinapsisCompacta::nueva(did, peso));
                            }
                        }
                    }
                    if !conexiones.is_empty() {
                        $capa.conexiones_intra.insert(id, conexiones);
                    }
                }
            };
        }
        
        // Cableado canónico Douglas & Martin (2004)
        conectar_capas!(self.capa_iv, self.capa_ii, 0.3);
        conectar_capas!(self.capa_iv, self.capa_iii, 0.2);
        conectar_capas!(self.capa_ii, self.capa_iii, 0.25);
        conectar_capas!(self.capa_iii, self.capa_v, 0.35);
        conectar_capas!(self.capa_v, self.capa_vi, 0.3);
        conectar_capas!(self.capa_vi, self.capa_iv, 0.15);
        conectar_capas!(self.capa_vi, self.capa_ii, 0.1);
        
        conectar_intra!(self.capa_i, 0.1);
        conectar_intra!(self.capa_ii, 0.2);
        conectar_intra!(self.capa_iii, 0.25);
        conectar_intra!(self.capa_iv, 0.15);
        conectar_intra!(self.capa_v, 0.2);
        conectar_intra!(self.capa_vi, 0.15);
    }
    
    pub fn capa(&self, idx: u8) -> Option<&CapaCortical> {
        match idx {
            0 => Some(&self.capa_i),
            1 => Some(&self.capa_ii),
            2 => Some(&self.capa_iii),
            3 => Some(&self.capa_iv),
            4 => Some(&self.capa_v),
            5 => Some(&self.capa_vi),
            _ => None,
        }
    }
    
    pub fn capa_mut(&mut self, idx: u8) -> Option<&mut CapaCortical> {
        match idx {
            0 => Some(&mut self.capa_i),
            1 => Some(&mut self.capa_ii),
            2 => Some(&mut self.capa_iii),
            3 => Some(&mut self.capa_iv),
            4 => Some(&mut self.capa_v),
            5 => Some(&mut self.capa_vi),
            _ => None,
        }
    }
    
    pub fn actualizar_activaciones(&mut self) {
        self.capa_i.calcular_activacion_media();
        self.capa_ii.calcular_activacion_media();
        self.capa_iii.calcular_activacion_media();
        self.capa_iv.calcular_activacion_media();
        self.capa_v.calcular_activacion_media();
        self.capa_vi.calcular_activacion_media();
        
        let medias = [
            self.capa_i.activacion_media,
            self.capa_ii.activacion_media,
            self.capa_iii.activacion_media,
            self.capa_iv.activacion_media,
            self.capa_v.activacion_media,
            self.capa_vi.activacion_media,
        ];
        self.activacion_sostenida = medias.iter().sum::<f32>() / 6.0;
    }
}

// ============================================================================
// IMPACTO CONCEPTUAL (One-Shot Learning)
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImpactoConceptual {
    pub id_binario: u32,
    pub quimica_simulada: f32,
    pub estres_hardware: f32,
    pub anclaje_contextual: Option<u32>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn casi(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    // ─── NeuronaCompacta ────────────────────────────────────────────────────

    #[test]
    fn test_reposo_valores_basales() {
        let n = NeuronaCompacta::reposo(7, 1, 3);
        assert_eq!(n.id, 7);
        assert_eq!(n.tipo, 1);
        assert_eq!(n.capa, 3);
        assert!(casi(n.voltaje, -70.0));
        assert!(casi(n.m, 0.05));
        assert!(casi(n.h, 0.6));
        assert!(casi(n.n, 0.32));
        assert!(casi(n.energia, 0.1));
        assert_eq!(n.edad, 0);
    }

    #[test]
    fn test_aleatoria_con_rng_constante() {
        let n = NeuronaCompacta::aleatoria(3, 0, 2, &mut || 0.5);
        assert_eq!(n.id, 3);
        assert!(casi(n.voltaje, -60.0)); // -70 + 0.5*20
        assert!(casi(n.m, 0.25));
        assert!(casi(n.energia, 0.15));
    }

    #[test]
    fn test_en_refractario_solo_supera_umbral() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.refractario = 0.05;
        assert!(!n.en_refractario());
        n.refractario = 0.5;
        assert!(n.en_refractario());
    }

    #[test]
    fn test_esta_activa_dopamina_baja_umbral() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.activacion = 0.25;
        // Sin dopamina: umbral 0.3, 0.25 no supera → inactiva
        assert!(!n.esta_activa(0.0));
        // Con dopamina máxima: umbral 0.2, 0.25 supera → activa
        assert!(n.esta_activa(1.0));
        // En refractario, nunca activa aunque el umbral baje
        n.refractario = 0.5;
        assert!(!n.esta_activa(1.0));
    }

    // ─── Metabolismo Neuronal ──────────────────────────────────────────────

    #[test]
    fn test_metabolismo_costo_basal_mantenimiento() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        // Sin corriente, sin spike, sin estrés: solo resta el mantenimiento
        n.integrar_quimica(0.0, 0.0);
        assert!(casi(n.energia, 0.1 - 0.0005));
    }

    #[test]
    fn test_metabolismo_gasto_sinaptico_proporcional() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.corriente_entrada = 0.5; // > 0.1, activa el gasto sináptico
        n.integrar_quimica(0.0, 0.0);
        // mantenimiento 0.0005 + sináptico 0.5*0.002 = 0.001
        assert!(casi(n.energia, 0.1 - 0.0005 - 0.001));
    }

    #[test]
    fn test_metabolismo_gasto_sinaptico_ignora_corriente_baja() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.corriente_entrada = 0.05; // <= 0.1, no consume
        n.integrar_quimica(0.0, 0.0);
        assert!(casi(n.energia, 0.1 - 0.0005));
    }

    #[test]
    fn test_metabolismo_costo_spike_cinco_porciento() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.voltaje = 10.0; // por encima de -40, dispara
        n.energia = 0.5;
        n.integrar_quimica(0.0, 0.0);
        // 0.5 - mantenimiento 0.0005 - spike 0.05 = 0.4495
        assert!(casi(n.energia, 0.5 - 0.0005 - 0.05));
    }

    #[test]
    fn test_metabolismo_spike_fallido_sin_energia() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.voltaje = 30.0; // dispara
        n.energia = 0.01; // insuficiente
        n.activacion = 0.8;
        n.integrar_quimica(0.0, 0.0);
        // energía llega a 0 (por debajo de 0.02) → spike fallido
        assert_eq!(n.energia, 0.0);
        assert!(casi(n.voltaje, -60.0));
        assert!(casi(n.activacion, 0.08));
        // El spike fallido fija refractario=1.0, pero el mismo tick aplica la
        // recuperación con energía 0 (eficiencia metabólica 0.0 → mínima 0.001).
        // Biologicamente: una neurona agotada queda casi inactiva, no totalmente.
        assert!(casi(n.refractario, 0.999));
        // Aún así permanece en periodo refractario (>0.1)
        assert!(n.en_refractario());
    }

    #[test]
    fn test_metabolismo_recuperacion_refractario_eficiencia_alta() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.refractario = 0.5;
        n.energia = 0.6; // > 0.5 → eficiencia 1.2
        n.integrar_quimica(0.0, 0.0);
        // recuperación = clamp(0.1*1.2, 0.001, 0.2) = 0.12
        assert!(casi(n.refractario, 0.5 - 0.12));
    }

    #[test]
    fn test_metabolismo_cortisol_penaliza_recuperacion() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.refractario = 0.5;
        n.energia = 0.6;
        n.integrar_quimica(1.0, 0.0); // cortisol máximo
        // recuperación = clamp(0.1*1.2 - 1.0*0.05, 0.001, 0.2) = clamp(0.07,...) = 0.07
        assert!(casi(n.refractario, 0.5 - 0.07));
    }

    #[test]
    fn test_metabolismo_adrenalina_aumenta_frecuencia_y_desgasta() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.integrar_quimica(0.0, 0.8); // adrenalina alta
        assert!(casi(n.frecuencia, 2.0));
        // mantenimiento 0.0005 + estrés 0.001
        assert!(casi(n.energia, 0.1 - 0.0005 - 0.001));
    }

    #[test]
    fn test_metabolismo_adrenalina_baja_no_afecta() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.integrar_quimica(0.0, 0.5);
        assert!(casi(n.frecuencia, 0.0));
        assert!(casi(n.energia, 0.1 - 0.0005));
    }

    #[test]
    fn test_alimentar_recarga_energia_con_tope() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.alimentar(0.5);
        assert!(casi(n.energia, 0.6));
        n.alimentar(5.0); // excede 1.0
        assert!(casi(n.energia, 1.0));
    }

    #[test]
    fn test_energia_nunca_negativa() {
        let mut n = NeuronaCompacta::reposo(0, 0, 0);
        n.voltaje = 50.0;
        n.energia = 0.001;
        n.corriente_entrada = 100.0;
        // Múltiples gastos consecutivos nunca llevan energía a negativo
        for _ in 0..50 {
            n.integrar_quimica(0.0, 0.0);
            assert!(n.energia >= 0.0 && n.energia <= 1.0);
        }
    }

    // ─── SinapsisCompacta (STDP) ───────────────────────────────────────────

    #[test]
    fn test_sinapsis_nueva_clampa_peso() {
        let s = SinapsisCompacta::nueva(5, 2.0);
        assert_eq!(s.destino, 5);
        assert!(casi(s.peso, 1.0)); // clamp superior
        let s2 = SinapsisCompacta::nueva(5, -3.0);
        assert!(casi(s2.peso, -1.0)); // clamp inferior
    }

    #[test]
    fn test_sinapsis_es_valida_por_umbral() {
        let s = SinapsisCompacta::nueva(1, 0.01);
        assert!(s.es_valida());
        let s2 = SinapsisCompacta::nueva(1, 0.0005);
        assert!(!s2.es_valida());
    }

    // ─── Episodio (Memoria) ────────────────────────────────────────────────

    #[test]
    fn test_episodio_nueva_trunca_patron_a_8() {
        let largo = (0..20).collect::<Vec<u32>>();
        let e = Episodio::nueva(1.0, 0.8, 0.5, &largo, 42);
        assert_eq!(e.patron, [0, 1, 2, 3, 4, 5, 6, 7]);
        // relevancia = 0.8 * (0.5 + 0.5*0.5) = 0.6
        assert!(casi(e.relevancia, 0.6));
    }

    #[test]
    fn test_episodio_relevancia_clampada() {
        let e = Episodio::nueva(0.0, 2.0, 1.0, &[1, 2], 0);
        assert!(e.relevancia <= 1.0);
        assert!(e.relevancia >= 0.0);
    }

    #[test]
    fn test_episodio_similitud_patron() {
        let e = Episodio::nueva(0.0, 1.0, 0.0, &[1, 2, 3, 4], 0);
        // 3 de 8 coinciden
        let sim = e.similitud_patron(&[1, 2, 3, 99]);
        assert!(casi(sim, 3.0 / 8.0));
        // ninguno coincide
        assert!(casi(e.similitud_patron(&[50, 51, 52]), 0.0));
    }

    // ─── Parámetros Hodgkin-Huxley ─────────────────────────────────────────

    #[test]
    fn test_parametros_neurona_default_hh() {
        let p = ParametrosNeurona::default();
        assert!(casi(p.g_na, 120.0));
        assert!(casi(p.g_k, 36.0));
        assert!(casi(p.e_na, 50.0));
        assert!(casi(p.e_k, -77.0));
        assert!(casi(p.e_l, -54.4));
    }

    #[test]
    fn test_parametros_neurona_inhibitorio_reduce_conductancias() {
        let p = ParametrosNeurona::inhibitorio();
        // Las neuronas inhibitorias tienen menos conductancias Na+/K+
        assert!(p.g_na < ParametrosNeurona::default().g_na);
        assert!(p.g_k < ParametrosNeurona::default().g_k);
    }

    // ─── ParametrosSTDP ────────────────────────────────────────────────────

    #[test]
    fn test_parametros_stdp_default() {
        let p = ParametrosSTDP::default();
        assert!(casi(p.a_plus, 0.1));
        assert!(casi(p.a_minus, 0.1));
        assert!(casi(p.tau_plus, 20.0));
        assert!(casi(p.tau_minus, 20.0));
        assert!(p.tau_plus > 0.0 && p.tau_minus > 0.0);
    }

    // ─── Tamaños compactos ─────────────────────────────────────────────────

    #[test]
    fn test_tamanios_compactos_cache_line() {
        // Diseño: NeuronaCompacta = 64B (línea de caché), SinapsisCompacta = 8B
        assert_eq!(std::mem::size_of::<NeuronaCompacta>(), 64);
        assert_eq!(std::mem::size_of::<SinapsisCompacta>(), 8);
    }
}
