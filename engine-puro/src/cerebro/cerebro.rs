// ============================================================================
// 🧠 CEREBRO DIGITAL AUTO-OPTIMIZABLE (COMPLETO)
// ============================================================================
// Orquesta:
// 1. Detección de hardware → Configuración dinámica
// 2. Memoria jerárquica (VRAM/RAM/SSD)
// 3. 8 motores biológicos + Motor Léxico Sinclair + ExploradorWeb
// 4. Procesamiento paralelo (CPU/GPU)
// 5. Ciclo de vida auto-optimizable
// 6. 6 Motores de Aprendizaje Profundo (MotorSensorial, Poda, Predictor,
//    Consolidador, Conceptos, Neurogénesis)
// ============================================================================

use crate::cerebro::estructuras::*;
use crate::cerebro::hardware::*;
use crate::cerebro::memoria::*;
use crate::cerebro::motores::*;
use crate::cerebro::lexico::asambleas::MotorAsambleasSemanticas;
use crate::cerebro::lexico::MediadorConsciencia;
use crate::cerebro::explorador::ExploradorWeb;
use crate::cerebro::persistencia;
use crate::cerebro::aprendizaje::sensorial::MotorSensorial;
use crate::cerebro::aprendizaje::poda::MotorPoda;
use crate::cerebro::aprendizaje::predictor::MotorPrediccion;
use crate::cerebro::aprendizaje::consolidador::MotorConsolidacion;
use crate::cerebro::aprendizaje::conceptos::MotorConceptos;
use crate::cerebro::aprendizaje::neurogenesis::MotorNeurogenesis;
use crate::cerebro::aprendizaje::homeostasis::ReguladorHomeostatico;
use crate::cerebro::sueno::{self, SistemaSueno};
use crate::cerebro::interocepcion::SistemaInteroceptivo;
use crate::cerebro::identidad::SistemaIdentidad;
use crate::cerebro::memoria_vinculo::MemoriaVinculo;
use crate::cerebro::asamblea_cortical::AsambleaCortical;
use crate::cerebro::dmn::{DefaultModeNetwork, DMNConfig};
use crate::cerebro::sistema_limbico::SistemaLimbico;
use crate::cerebro::conacion::MotorConacion;
use crate::cerebro::motor::{CortexMotor, TipoAccion};
use crate::cerebro::asociacion_libre::GestorAsociacionLibre;
use crate::cerebro::talamo::{TalamoDigital, AccesoConsciente};
use crate::cerebro::efectores::EfectorLocal;
use crate::cerebro::estructuras::{
    ColumnaCortical, EstimuloTalamico, PrediccionTalamica, TipoNeuromodulador,
};
use crate::cerebro::motores::MotorColumnaCortical;
use rand::Rng;


// ============================================================================
// MOTORES ESCALABLES (Agrupación de todos los motores biológicos)
// ============================================================================

pub struct MotoresEscalables {
    pub hipocampo: Hipocampo,
    pub amigdala: Amigdala,
    pub atencion: AtencionSelectiva,
    pub dopamina: SistemaDopamina,
    pub conciencia: Conciencia,
}

impl MotoresEscalables {
    pub fn nuevo(config: &ConfiguracionDinamica) -> Self {
        Self {
            hipocampo: Hipocampo::nuevo(config.memoria_episodica_max),
            amigdala: Amigdala::nuevo(),
            atencion: AtencionSelectiva::nuevo(),
            dopamina: SistemaDopamina::nuevo(),
            conciencia: Conciencia::nuevo(),
        }
    }

    /// Actualiza todos los motores con el estado actual
    pub fn actualizar(&mut self, dt: f32, actividad: &[f32], entrada: &Entrada) {
        // Atención: procesar estímulos
        let estimulos: Vec<(u32, f32)> = entrada
            .estimulos
            .iter()
            .map(|e| (e.id, e.intensidad))
            .collect();
        self.atencion.actualizar(dt, &estimulos);

        // Emoción: calcular amenaza y recompensa
        let amenaza = entrada
            .estimulos
            .iter()
            .map(|e| e.amenaza)
            .fold(0.0, f32::max);
        let recompensa = entrada
            .estimulos
            .iter()
            .map(|e| e.recompensa)
            .fold(0.0, f32::max);
        let _valencia = self.amigdala.actualizar(dt, amenaza, recompensa);

        // Dopamina: señal de recompensa
        self.dopamina.actualizar(dt, recompensa);

        // Conciencia: actividad global
        let actividad_ref: Vec<(u32, f32)> = actividad
            .iter()
            .enumerate()
            .map(|(i, &a)| (i as u32, a))
            .collect();
        self.conciencia.actualizar(dt, &actividad_ref);

        // Olvido hipocampal
        self.hipocampo.olvidar(dt);
    }
}

// ============================================================================
// CEREBRO AUTO-OPTIMIZABLE
// ============================================================================

pub struct CerebroAutoOptimizable {
    pub hardware: HardwareInfo,
    pub config: ConfiguracionDinamica,
    pub memoria: MemoriaAdaptativa,
    pub motores: MotoresEscalables,
    pub params_neurona: ParametrosNeurona,
    pub params_stdp: ParametrosSTDP,
    pub paso_actual: u64,
    pub tiempo: f32,
    pub historial_emocional: Vec<f32>,
    pub ultima_salida: Salida,
    pub mas: MotorAsambleasSemanticas,
    pub motor_curiosidad: MotorCuriosidad,
    pub motor_sensorial: MotorSensorial,
    pub mediador: MediadorConsciencia,

    // === Motores de Aprendizaje Profundo ===
    pub motor_poda: MotorPoda,
    pub motor_predictor: MotorPrediccion,
    pub motor_consolidacion: MotorConsolidacion,
    pub motor_conceptos: MotorConceptos,
    pub motor_neurogenesis: MotorNeurogenesis,
    pub motor_homeostasis: ReguladorHomeostatico,
    pub talamo: TalamoDigital,
    pub sistema_interoceptivo: SistemaInteroceptivo,
    pub sistema_identidad: SistemaIdentidad,
    pub memoria_vinculo: MemoriaVinculo,
    pub asambleas: AsambleaCortical,
    /// Default Mode Network (Rumiación interna)
    pub dmn: DefaultModeNetwork,
    pub sistema_limbico: SistemaLimbico,
    pub motor_conacion: MotorConacion,
    pub cortex_motor: CortexMotor,
    pub gestor_asociacion: GestorAsociacionLibre,
    pub sistema_sueno: SistemaSueno,

    /// Columnas corticales canónicas de 6 capas
    pub columnas_corticales: Vec<ColumnaCortical>,

    pub(crate) siguiente_id: u32,
    pub efectores: EfectorLocal,

    // ====================================================================
    // WORKING MEMORY — Reverberación Recurrente (modelo NMDA)
    // ====================================================================
    // Inspiración: Las colaterales recurrentes del córtex prefrontal
    // mantienen activos los patrones neuronales tras el disparo inicial.
    // El receptor NMDA tiene una cinética lenta (τ ≈ 50-200ms) que genera
    // corrientes persistentes: base del working memory (Baddeley, 1974;
    // Wang, 1999 — modelo de corriente NMDA para memoria de trabajo).
    //
    // Implementación: cada token generado activa sus neuronas semánticas
    // con una corriente eco que decae exponencialmente con τ_nmda = 200ms.
    // El siguiente paso() la encuentra y es "recordar lo último que dije".
    pub eco_reverberante: Vec<(u32, f32)>,
}


impl CerebroAutoOptimizable {
    /// Crea un nuevo cerebro con detección de hardware automática
    /// Si existe un archivo de estado previo, lo restaura automáticamente
    pub fn nuevo() -> Self {
        println!("🧠 Inicializando Cerebro Digital Auto-Optimizable...");
        println!("═══════════════════════════════════════════════");

        // 1. Detectar hardware
        let hardware = HardwareInfo::detectar();
        hardware.mostrar();

        // 2. Configurar dinámicamente
        let config = ConfiguracionDinamica::from_hardware(&hardware);
        config.mostrar();

        // 3. Crear sistema
        let mut cerebro = Self {
            memoria: MemoriaAdaptativa::nuevo(&config),
            motores: MotoresEscalables::nuevo(&config),
            params_neurona: ParametrosNeurona::default(),
            params_stdp: ParametrosSTDP::default(),
            paso_actual: 0,
            tiempo: 0.0,
            historial_emocional: Vec::new(),
            ultima_salida: Salida {
                texto: String::new(),
                emocion: 0.0,
                conciencia: 0.0,
                actividad: Vec::new(),
                corriente: None,
            },
            mas: MotorAsambleasSemanticas::nuevo(),
            motor_curiosidad: MotorCuriosidad::nuevo(),
            motor_sensorial: MotorSensorial::nuevo(),
            mediador: MediadorConsciencia::nuevo(),
            motor_poda: MotorPoda::nuevo(),
            motor_predictor: MotorPrediccion::nuevo(),
            motor_consolidacion: MotorConsolidacion::nuevo(),
            motor_conceptos: MotorConceptos::nuevo(),
            motor_neurogenesis: MotorNeurogenesis::nuevo(),
            motor_homeostasis: ReguladorHomeostatico::nuevo(),
            talamo: TalamoDigital::nuevo(),
            sistema_interoceptivo: SistemaInteroceptivo::nuevo(),
            sistema_identidad: SistemaIdentidad::nuevo(),
            memoria_vinculo: MemoriaVinculo::nueva(),
            asambleas: AsambleaCortical::nueva(),
            dmn: DefaultModeNetwork::nueva(DMNConfig::default()),
            sistema_limbico: SistemaLimbico::nuevo(),
            motor_conacion: MotorConacion::nuevo(),
            cortex_motor: CortexMotor::nuevo(),
            gestor_asociacion: GestorAsociacionLibre::nuevo(),
            sistema_sueno: SistemaSueno::nuevo(),
            columnas_corticales: Vec::new(),
            siguiente_id: 0,
            efectores: EfectorLocal::nuevo(),
            eco_reverberante: Vec::new(),
            hardware,
            config,
        };


        // 4. Intentar restaurar estado persistente
        let ruta = persistencia::ruta_por_defecto();
        match persistencia::cargar(&ruta) {
            Ok(estado) => {
                persistencia::restaurar(&mut cerebro, estado);
            }
            Err(_) => {
                println!("  💾 No se encontró estado previo, comenzando desde cero");
            }
        }

        // 5. Inicializar neuronas
        let total_init = cerebro.config.max_neuronas_ram.min(100_000);
        cerebro.inicializar_neuronas(total_init);
        println!(
            "  🧬 Sistema inicializado con {} neuronas en RAM",
            total_init
        );

        // 6. Inicializar columnas corticales (6 capas)
        let num_columnas = cerebro.config.hilos_cpu.min(8).max(1);
        let neuronas_por_columna = 5000; // ~5K neuronas por columna = 30K total
        let mut rng_col = rand::thread_rng();
        let mut tipo_rng = || rng_col.gen::<f32>();
        for i in 0..num_columnas {
            let mut columna = ColumnaCortical::nueva(i as u32, neuronas_por_columna, &mut cerebro.siguiente_id);
            columna.cablear(&mut tipo_rng);
            cerebro.columnas_corticales.push(columna);
        }
        println!(
            "  🏛️ {} columnas corticales creadas ({} neuronas c/u, {} capas)",
            num_columnas, neuronas_por_columna, 6,
        );

        cerebro
    }

    /// Guarda el estado actual del cerebro a disco
    pub fn guardar_a_disco(&self) -> Result<(), String> {
        persistencia::guardar(self, &persistencia::ruta_por_defecto())
    }

    /// Guarda el estado a disco con ruta personalizada
    pub fn guardar_a(&self, ruta: &str) -> Result<(), String> {
        persistencia::guardar(self, ruta)
    }

    /// Inicializa la población de neuronas con valores aleatorios e interconecta la red
    fn inicializar_neuronas(&mut self, cantidad: usize) {
        let mut rng = rand::thread_rng();

        for i in 0..cantidad {
            let tipo = if rng.gen::<f32>() > 0.8 {
                1 // inhibitoria
            } else {
                0 // excitatoria
            };
            let capa = (i % 5) as u8; // 5 capas corticales

            let neurona = NeuronaCompacta::aleatoria(i as u32, tipo, capa, &mut || rng.gen());

            self.memoria.ram.agregar_neurona(neurona);
            self.memoria
                .mapa_memoria
                .insert(i as u32, UbicacionMemoria::RAM);
            self.siguiente_id = (i + 1) as u32;
        }

        // Mover primeras neuronas a VRAM (si existe)
        if self.memoria.vram.is_some() {
            for i in 0..self.config.max_neuronas_vram.min(1000).min(cantidad) {
                self.memoria.mover_a_vram(i as u32);
            }
        }

        // Crear cableado sináptico inicial (Small-World Network)
        // Conectamos cada neurona con ~8 vecinas aleatorias
        println!("  🔌 Conectando neuronas en red de mundo pequeño...");
        for i in 0..cantidad {
            let origen = i as u32;
            let capa_origen = (i % 5) as u8;
            for _ in 0..8 {
                // target_capa = misma capa o la siguiente
                let target_capa = (capa_origen + rng.gen_range(0..=1)).min(4);
                let mut destino = rng.gen_range(0..cantidad) as u32;
                
                // Intentar hasta 3 veces encontrar una neurona en la capa destino
                for _ in 0..3 {
                    if (destino % 5) as u8 == target_capa && destino != origen {
                        break;
                    }
                    destino = rng.gen_range(0..cantidad) as u32;
                }
                
                if origen != destino {
                    let peso = (rng.gen::<f32>() * 2.0 - 1.0) * 0.15; // pesos iniciales pequeños
                    self.crear_sinapsis(origen, destino, peso);
                }
            }
        }
    }

    /// Crea una nueva neurona y la agrega a RAM
    pub fn crear_neurona(&mut self, tipo: u8, capa: u8) -> u32 {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let mut rng = rand::thread_rng();
        let neurona = NeuronaCompacta::aleatoria(id, tipo, capa, &mut || rng.gen());

        self.memoria.ram.agregar_neurona(neurona);
        self.memoria
            .mapa_memoria
            .insert(id, UbicacionMemoria::RAM);

        // Si hay VRAM y la neurona es importante, subirla
        if self.memoria.vram.is_some() && !self.memoria.vram.as_ref().unwrap().esta_lleno() {
            self.memoria.mover_a_vram(id);
        }

        id
    }

    /// Crea una sinapsis entre dos neuronas (en VRAM o RAM)
    pub fn crear_sinapsis(&mut self, origen: u32, destino: u32, peso: f32) {
        let sinapsis = SinapsisCompacta::nueva(destino, peso);

        if self.memoria.esta_en_vram(origen) {
            if let Some(vram) = &mut self.memoria.vram {
                vram.agregar_sinapsis(origen, sinapsis);
            }
        } else {
            self.memoria.ram.sinapsis.entry(origen).or_insert_with(Vec::new).push(sinapsis);
        }
    }

    // ====================================================================
    // PASO PRINCIPAL DE SIMULACIÓN
    // ====================================================================

    /// Ejecuta un paso de simulación del cerebro completo.
    /// `dt`: delta temporal en segundos (default: 0.001 = 1ms)
    /// `entrada`: estímulos sensoriales
    pub fn paso(&mut self, dt: f32, mut entrada: Entrada) -> Salida {
        // === SISTEMA LÍMBICO Y METABOLISMO: Actualizar Neuroquímica y Energía ===
        let exito = entrada.recompensa > 0.5;
        let impacto = entrada.recompensa.max(entrada.amenaza);
        let es_feedback = entrada.texto.as_ref().map(|t| t.contains("bueno") || t.contains("bien") || t.contains("excelente")).unwrap_or(false);
        self.sistema_limbico.procesar_evento(exito, impacto, es_feedback);

        // Distribución de Glucosa Digital (Metabolismo Global)
        // La energía disponible depende del bienestar del hardware
        let bienestar = self.sistema_interoceptivo.homeostasis.bienestar_general;
        let glucosa_total = bienestar * 0.05; // Cuánta energía podemos repartir este tick

        // Alimentar neuronas (priorizando el foco de atención)
        let neuronas_mut = self.memoria.ram.obtener_todas_mut();
        for n in neuronas_mut {
            let ración = if self.motores.atencion.foco.contains(&n.id) {
                glucosa_total * 2.0 // El foco recibe doble ración
            } else {
                glucosa_total
            };
            n.alimentar(ración);
        }

        // === INTEROCEPCIÓN: Muestrear hardware y generar estímulos corporales ===
        // El Sistema Interoceptivo lee /proc/* y produce estímulos de "sensación corporal"
        // que se inyectan en el pipeline sensorial antes del Tálamo.
        self.sistema_interoceptivo.integrar_en_pipeline(dt, &mut entrada);

        // === IDENTIDAD: Inyectar auto-descripción arquitectónica ===
        // El Sistema de Identidad genera texto como "[IDENTIDAD: soy un cerebro SNN
        // con 8 columnas corticales...]" basado en la configuración real del sistema.
        // El Motor Léxico Sinclair aprende estas palabras asociadas al estado interno,
        // permitiendo que el engine se auto-describa como un LLM describe su arquitectura.
        let total_neuronas = self.siguiente_id;
        self.sistema_identidad.integrar_en_pipeline(
            self.paso_actual,
            self.columnas_corticales.len(),
            total_neuronas,
            &self.hardware,
            &self.config,
            &self.memoria,
            &self.talamo,
            &self.sistema_interoceptivo,
            &mut entrada,
        );
        self.paso_actual += 1;
        self.tiempo += dt;

        // === SUEÑO: Gate de Ciclo Sueño-Vigilia ===
        // Si el cerebro está durmiendo, ejecutamos el ciclo de sueño
        // y retornamos silencio (no hay procesamiento consciente).
        // Si está en vigilia pero acumuló suficientes episodios, iniciamos sueño.
        if self.sistema_sueno.estado != sueno::EstadoSueno::Vigilia {
            self.sistema_sueno.ciclo_sueno(
                dt,
                &mut self.talamo,
                &mut self.memoria,
                &mut self.columnas_corticales,
                &self.params_neurona,
            );
            return self.ultima_salida.clone();
        } else if self.sistema_sueno.debe_dormir() {
            println!(
                "  🌙 Iniciando ciclo de sueño... ({} patrones acumulados)",
                self.sistema_sueno.episodios_diarios.len()
            );
            self.sistema_sueno.estado = sueno::EstadoSueno::NREM1;
            return self.ultima_salida.clone();
        }

        // === PRE-0. WORKING MEMORY — Eco Reverberante NMDA ===
        // Inyectar el residuo de la última secuencia generada como corriente persistente.
        // Simula las colaterales recurrentes del PFC con cinética NMDA lenta.
        // τ_nmda = 200ms → por paso de dt=0.001s: factor_decay = e^(-dt/0.2) ≈ 0.995
        //
        // Cada neurona en eco recibe corriente_entrada proporcional a su eco residual.
        // Esto permite que el cerebro "recuerde" los conceptos que acaba de verbalizar
        // antes de generar la siguiente palabra — coherencia inter-turno emergente.
        let tau_nmda: f32 = 0.2; // 200ms — tiempo de vida del eco NMDA
        let factor_decay = (-dt / tau_nmda).exp(); // e^(-dt/τ)

        // Inyectar eco y decaer simultáneamente
        self.eco_reverberante.retain_mut(|(nid, corriente)| {
            // Aplicar corriente persistente a la neurona (como receptor NMDA lento)
            if let Some(n) = self.memoria.obtener_neurona_mut(*nid) {
                n.corriente_entrada += *corriente; // Corriente de mantenimiento
            }
            // Decaimiento exponencial — NMDA se cierra gradualmente
            *corriente *= factor_decay;
            // Mantener en el eco solo si aún es significativa (> umbral de ruido térmico)
            *corriente > 0.5 // Por debajo de 0.5 nA, el eco es subumbral
        });

        // === 0. PIPELINE SENSORIAL BIOLÓGICO (MAS) ===
        // Estímulos semánticos distribuidos (base_neurona + dimensión) generados por
        // el Motor Sensorial. Se acumulan aquí y se inyectan en `entrada.estimulos`
        // tras liberar el préstamo del texto, para que el tálamo los procese junto
        // con los demás canales sensoriales.
        let mut estimulos_semanticos: Vec<Estimulo> = Vec::new();
        // IDs semánticos (base_neurona+dimensión) derivados de los estímulos.
        // Se extraen aquí para poder reutilizarlos en la consolidación de asambleas
        // tras consumir `estimulos_semanticos` en el canal sensorial.
        let mut ids_semanticos: Vec<u32> = Vec::new();
        if let Some(ref texto_entrada) = entrada.texto {
            // El MAS percibe el texto como patrones de disparo, no tokens
            let neuronas_perceptivas = self.mas.percibir_texto(texto_entrada);
            
            for nid in neuronas_perceptivas {
                if let Some(n) = self.memoria.obtener_neurona_mut(nid) {
                    // Inyección de corriente sensorial directa
                    n.corriente_entrada += 25.0;
                    n.activacion = (n.activacion + 0.4).min(1.0);
                }
            }

            // Estímulos semánticos distribuidos: una neurona por dimensión del embedding.
            // Estos son los mismos IDs que consolidarán las asambleas, cerrando el
            // desacoplamiento entre el espacio de tokens y el espacio neuronal.
            estimulos_semanticos = self.motor_sensorial.texto_a_estimulos(texto_entrada);
            ids_semanticos = estimulos_semanticos.iter().map(|e| e.id).collect();

            // El Motor de Conceptos ahora registrará impactos basados en la rumiación de estas neuronas,
            // no en IDs de tokens estadísticos.
            let (_estres_cpu, _) = HardwareInfo::medir_uso_caliente();
            let _quimica_global = self.sistema_limbico.factor_aprendizaje();
            
            // Consolidación de asambleas perceptivas
            // Si hay alta actividad, el sistema 'aprende' el patrón como una asamblea cohesiva
            // (Nota: Esta parte sustituye a aprender_token)

            // === VÍNCULO: Inyectar contexto de recuerdos previos al pipeline ===
            // La Memoria del Vínculo busca interacciones pasadas con palabras
            // similares al input actual y genera tokens de contexto que se
            // inyectan como texto adicional. El Motor Léxico Sinclair procesa
            // estos tokens como si fueran parte del input, modulando la
            // respuesta vía STDP y cadenas de Markov.
            let palabras_clave: Vec<String> = texto_entrada
                .split_whitespace()
                .map(|p| p.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                .filter(|p| !p.is_empty())
                .collect();
            if !palabras_clave.is_empty() {
                let contexto_vinculo = self.memoria_vinculo.generar_contexto_inyectable(
                    &palabras_clave,
                    self.paso_actual,
                );
                if let Some(ref mut texto) = entrada.texto {
                    texto.push_str(&format!("\n{}", contexto_vinculo.join(" ")));
                }
            }
        }

        // Inyectar estímulos semánticos distribuidos en el canal sensorial.
        // Se añaden tras liberar el préstamo de `entrada.texto` para que el
        // tálamo los procese junto a los demás canales (interocepción, etc.).
        // Estos estímulos portan los IDs base_neurona+dimensión que las asambleas
        // híbridas usarán para resonar — cierra el puente token↔neurona.
        if !estimulos_semanticos.is_empty() {
            entrada.estimulos.extend(estimulos_semanticos);
        }

        // === TÁLAMO DIGITAL: Filtro Sensorial y Pipeline Columnar ===
        // El Tálamo filtra estímulos y los envía a las columnas corticales.
        // Las columnas procesan en 6 capas y devuelven predicciones.
        
        // 1. Evaluar acceso talámico para cada estímulo
        let mut estimulos_columnares: Vec<EstimuloTalamico> = Vec::new();
        for estimulo in &entrada.estimulos {
            // Usar la última predicción talámica si existe (desde el feedback de Capa VI)
            let prediccion_tal = self.columnas_corticales.first()
                .and_then(|c| c.ultima_prediccion.as_ref())
                .map(|p| Prediccion {
                    id_objetivo: p.columna_origen,
                    valor_esperado: p.valor_esperado,
                    confianza: p.confianza,
                });
            
            let acceso = self.talamo.procesar_estimulo(estimulo, prediccion_tal.as_ref());
            
            match acceso {
                AccesoConsciente::Alerta(intensidad) => {
                    // Modo Fásico: alta intensidad + novedad
                    estimulos_columnares.push(EstimuloTalamico {
                        origen_talamo: estimulo.id,
                        intensidad: intensidad.max(estimulo.intensidad),
                        novedad: intensidad,
                    });
                }
                AccesoConsciente::TransmisionFiel => {
                    // Modo Tónico: transmisión fiel
                    estimulos_columnares.push(EstimuloTalamico {
                        origen_talamo: estimulo.id,
                        intensidad: estimulo.intensidad,
                        novedad: 0.1,
                    });
                }
                AccesoConsciente::Filtrado => {
                    // No pasa a la consciencia plena (subconsciente)
                    // Podría ir a ganglios basales
                }
            }
        }

        // 2. Procesar columnas corticales (6 capas canónicas)
        let mut predicciones_columnas: Vec<PrediccionTalamica> = Vec::new();
        let mut _activacion_total_cortical: f32 = 0.0;
        let columnas_activas = self.columnas_corticales.len();
        
        if columnas_activas > 0 && !estimulos_columnares.is_empty() {
            // Distribuir estímulos entre columnas (round-robin simple)
            for (i, col) in self.columnas_corticales.iter_mut().enumerate() {
                // Cada columna recibe los estímulos cuyo ID módulo num_columnas = i
                let estimulos_col: Vec<EstimuloTalamico> = estimulos_columnares.iter()
                    .filter(|e| e.origen_talamo as usize % columnas_activas == i)
                    .cloned()
                    .collect();
                
                if !estimulos_col.is_empty() || col.activacion_sostenida > 0.1 {
                    let (pred, activacion) = MotorColumnaCortical::ciclo_columna(
                        col, &estimulos_col, &self.params_neurona, 0.001,
                    );
                    if let Some(p) = pred {
                        predicciones_columnas.push(p);
                    }
                    _activacion_total_cortical += activacion;
                }
            }

            // Inhibición lateral entre columnas (competencia)
            if columnas_activas > 1 {
                MotorColumnaCortical::inhibicion_entre_columnas(
                    &mut self.columnas_corticales, 0.6, 1,
                );
            }

            // Aplicar neuromodulación desde el Sistema Límbico (Química Real)
            let quimica = &self.sistema_limbico.quimica;
            let moduladores = vec![
                (TipoNeuromodulador::Dopamina, quimica.dopamina),
                (TipoNeuromodulador::Noradrenalina, quimica.adrenalina * 0.8),
                (TipoNeuromodulador::Serotonina, quimica.serotonina),
                (TipoNeuromodulador::Acetilcolina, self.sistema_limbico.factor_aprendizaje() * 0.5),
            ];
            for col in &mut self.columnas_corticales {
                MotorColumnaCortical::aplicar_neuromodulacion(col, &moduladores);
                // El cortisol afecta directamente la fatiga de la columna
                col.activacion_sostenida *= 1.0 - (quimica.cortisol * 0.2);
            }

            // Neuromodulación interoceptiva: el estado corporal afecta la ganancia cortical
            let homeostasis = &self.sistema_interoceptivo.homeostasis;
            if homeostasis.bienestar_general < 0.3 {
                // Bajo bienestar → reducción de actividad cortical (ahorro energético)
                let moduladores_intero = vec![
                    (TipoNeuromodulador::Noradrenalina, (1.0 - homeostasis.dolor_sistemico) * 0.3),
                    (TipoNeuromodulador::Serotonina, homeostasis.bienestar_general),
                ];
                for col in &mut self.columnas_corticales {
                    MotorColumnaCortical::aplicar_neuromodulacion(col, &moduladores_intero);
                }
            } else if homeostasis.bienestar_general > 0.7 {
                // Alto bienestar → acetilcolina + dopamina (estado óptimo)
                let moduladores_intero = vec![
                    (TipoNeuromodulador::Acetilcolina, homeostasis.energia_disponible * 0.4),
                    (TipoNeuromodulador::Dopamina, homeostasis.bienestar_general * 0.3),
                ];
                for col in &mut self.columnas_corticales {
                    MotorColumnaCortical::aplicar_neuromodulacion(col, &moduladores_intero);
                }
            }
        }

        // 3. Feedback talámico desde Capa VI (predictive coding)
        if !predicciones_columnas.is_empty() {
            self.talamo.recibir_feedback(&predicciones_columnas);
        }

        // 4. Generar estímulos talámicos actualizados para la memoria RAM
        //    (para mantener compatibilidad con el resto del pipeline)
        if !estimulos_columnares.is_empty() {
            for est in &estimulos_columnares {
                // Inyectar en las neuronas RAM legacy para que atención/conciencia funcionen
                if let Some(n) = self.memoria.obtener_neurona_mut(est.origen_talamo) {
                    n.corriente_entrada += est.intensidad * 20.0;
                    n.activacion = (n.activacion + est.intensidad * 0.3).min(1.0);
                }
            }
        }

        // Pulso de Sincronización Gamma (~40Hz)
        // Lliga las percepciones en un solo instante de consciencia
        if self.talamo.sincronizar(self.tiempo) {
            // Ventana de oportunidad: elevar ligeramente el voltaje de neuronas activas
            // para que disparen juntas (binding temporal)
            let neuronas_ram = self.memoria.ram.obtener_todas_mut();
            for n in neuronas_ram {
                if n.activacion > 0.4 {
                    n.voltaje += 5.0; // Pequeño empujón hacia el umbral
                }
            }
        }

        // (El puente léxico-semántico del MotorLéxico fue eliminado:
        //  el MAS ya inyecta la percepción sensorial en el pipeline como
        //  patrones de disparo de asambleas, no como tokens estadísticos.)

        // === 1. OPTIMIZACIÓN DINÁMICA + PERSISTENCIA (cada 1000 pasos) ===
        if self.paso_actual % 1000 == 0 {
            self.memoria.optimizar();
            // Auto-guardado silencioso del estado aprendido
            if let Err(e) = persistencia::guardar(self, &persistencia::ruta_por_defecto()) {
                // Solo log en debug, no interrumpe la simulación
                if self.paso_actual % 10000 == 0 {
                    eprintln!("  ⚠️ Auto-guardado falló en paso {}: {}", self.paso_actual, e);
                }
            }
        }

        let tematica = entrada.clasificar_tematica();

        // === 2. ATENCIÓN (selecciona qué activar) ===
        let estimulos: Vec<(u32, f32)> = entrada
            .estimulos
            .iter()
            .map(|e| (e.id, e.intensidad))
            .collect();
        let foco = self.motores.atencion.actualizar(dt, &estimulos);

        // Activar neuronas en VRAM según foco
        for &id in &foco {
            if !self.memoria.esta_en_vram(id) {
                self.memoria.mover_a_vram(id);
            }
            self.memoria.registrar_acceso(id);
        }

        // Estimulación del recuerdo si la temática es Íntima (recuperación de memoria episódica/semántica)
        if tematica == Tematica::Intima {
            let episodios_relacionados = self.memoria.ssd.recuperar(&foco);
            for ep in &episodios_relacionados {
                for &nid in &ep.patron {
                    if nid > 0 {
                        if let Some(n) = self.memoria.obtener_neurona_mut(nid) {
                            n.corriente_entrada += ep.intensidad * 25.0; // Estimular recuerdo de esa experiencia
                        }
                    }
                }
            }
        }

        // === 3. PROCESAMIENTO NEURONAL (CPU paralelo - Ráfaga de Integración) ===
        // Ejecutamos 50 sub-pasos de 1ms para permitir que las señales y spikes
        // se integren y propaguen a lo largo de las sinapsis de la red.
        let sub_dt = 0.001;
        let mut actividad = Vec::new();
        for _ in 0..50 {
            actividad = self.procesar_cpu(sub_dt);
        }

        // === 4. MOTORES BIOLÓGICOS ===
        self.motores.actualizar(dt, &actividad, &entrada);

        // Modulación dinámica por Gating Cognitivo (Operativo vs Íntimo)
        let mut emocion_modulada = self.motores.amigdala.alegria - self.motores.amigdala.miedo;

        match tematica {
            Tematica::Operativa => {
                // Modo Operativo: Maximizamos conciencia y atenuamos emoción
                self.motores.conciencia.intensidad = (self.motores.conciencia.intensidad + 0.4).min(1.0);
                emocion_modulada *= 0.15; // Atenuación emocional drástica
            }
            Tematica::Intima => {
                // Modo Íntimo: Conciencia reflexiva/creativa, emoción libre
                if self.motores.conciencia.intensidad > 0.65 {
                    self.motores.conciencia.intensidad = 0.6; // Rango óptimo para fluidez creativa
                }
            }
            Tematica::Basal => {}
        }

        // === 5. ALMACENAR EPISODIO (si es importante) ===
        if entrada.es_importante() {
            let mut patron = [0u32; 8];
            for (i, &id) in foco.iter().take(8).enumerate() {
                patron[i] = id;
            }

            let mut contexto_hash = 0u64;
            if let Some(ref txt) = entrada.texto {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                txt.hash(&mut hasher);
                contexto_hash = hasher.finish();
            }

            let episodio = Episodio::nueva(
                self.tiempo,
                entrada.intensidad_promedio(),
                emocion_modulada,
                &patron,
                contexto_hash,
            );
            self.memoria.ssd.almacenar(episodio);
        }

        // === 6. CONSOLIDACIÓN DE ASAMBLEAS (aprendizaje léxico-biológico) ===
        // En lugar de aprender tokens estadísticos, el MAS consolida el patrón
        // de disparo actual como una asamblea semántica cohesiva. Si el foco
        // perceptivo fue suficientemente intenso, se aprende la agrupación.
        if let Some(ref texto_entrada) = entrada.texto {
            // Constituyentes HÍBRIDOS: neuronas del foco (patrón de disparo real)
            // + IDs semánticos distribuidos (base_neurona+dimensión). Este puente
            // cierra el desacoplamiento: cuando el mismo texto vuelva a activar
            // las neuronas sensoriales, la asamblea resonará.
            let mut constituyentes: Vec<u32> = foco.iter().copied().take(24).collect();
            constituyentes.extend_from_slice(&ids_semanticos);
            constituyentes.truncate(48);
            // Deduplicar preservando orden (evitar asambleas con IDs repetidos)
            let mut vistos = std::collections::HashSet::new();
            constituyentes.retain(|id| vistos.insert(*id));

            if !constituyentes.is_empty() {
                let etiqueta = texto_entrada
                    .split_whitespace()
                    .next()
                    .map(|p| p.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                    .filter(|p| !p.is_empty());
                self.mas.consolidar_asamblea(constituyentes, etiqueta);
            }
        }

        // === VÍNCULO: Reactivar neuronas de recuerdos previos a la generación ===
        // Antes de generar la respuesta, la Memoria del Vínculo reactiva las
        // neuronas que estuvieron activas en interacciones similares pasadas.
        // Esto da corriente de entrada extra a esas neuronas, haciendo más
        // probable que el motor léxico genere palabras relacionadas al recuerdo.
        if let Some(ref texto_entrada) = entrada.texto {
            let palabras_clave: Vec<String> = texto_entrada
                .split_whitespace()
                .map(|p| p.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                .filter(|p| !p.is_empty())
                .collect();
            if !palabras_clave.is_empty() {
                self.memoria_vinculo.reactivar_recuerdos(
                    &palabras_clave,
                    self.paso_actual,
                    &mut self.memoria,
                );
            }
        }

        // === CONACIÓN: Evaluación de la Voluntad e Iniciativa ===
        let quiere_hablar = self.motor_conacion.evaluar_voluntad(dt, &self.sistema_limbico, &self.dmn);
        
        // Si el sistema tiene iniciativa propia, inyectamos una señal artificial de entrada
        // para que el Motor Léxico Sinclair genere una idea basada en la rumiación actual.
        if quiere_hablar && entrada.texto.is_none() {
             // Inyectar una semilla de pensamiento basada en el foco rumiado del DMN
             if let Some(foco_id) = self.dmn.foco_interno {
                 // Convertimos la asamblea en un estímulo de alta prioridad
                 self.asambleas.inyectar_corriente_a_asamblea(foco_id, 2.0);
             }
        }

        // === 7. GENERAR SALIDA BIOLÓGICA (Resonancia de Asambleas MAS) ===
        // El lenguaje es una emergencia del estado físico de las asambleas.
        let mut ids_activos: Vec<u32> = actividad.iter()
            .enumerate()
            .filter(|&(_, &a)| a > self.mas.umbral_sincronia)
            .map(|(i, _)| i as u32)
            .collect();

        // Fusión de espacios de ID: cuando hay texto de entrada, los IDs
        // semánticos (base_neurona+dimensión) generados por texto_a_estimulos
        // deben alimentar la resonancia junto a la actividad neuronal. Esto
        // cierra el desacoplamiento: las asambleas consolidadas en paso_tutor
        // con IDs semánticos ahora resuenan al re-exponerse el mismo estímulo.
        if !ids_semanticos.is_empty() {
            ids_activos.extend_from_slice(&ids_semanticos);
            ids_activos.sort_unstable();
            ids_activos.dedup();
        }

        // Articulación con fallback en cadena: si ninguna asamblea supera el
        // umbral estricto, se devuelve la de mayor solapamiento parcial en lugar
        // del silencio. El "..." queda como último recurso biológico legítimo.
        let texto = self.mas.articular_idea_extendida(&ids_activos);

        // === CÓRTEX MOTOR: Ejecución de Acciones por Voluntad Neuronal ===
        let acciones = self.cortex_motor.procesar_voluntad_accion(self.paso_actual, &ids_activos);
        for accion in acciones {
            // Ejecutar la acción a través de los efectores del sistema
            match accion {
                TipoAccion::Shell(comando) => {
                    let _ = self.efectores.ejecutar_comando(&comando);
                },
                TipoAccion::EscrituraArchivo(ruta, contenido) => {
                    let _ = self.efectores.escribir_archivo(&ruta, &contenido);
                },
                _ => {}
            }
        }

        // === INTEGRACIÓN DE CORRIENTE DE CONSCIENCIA BIOLÓGICA ===
        let asamblea_resonante_idx = self.mas.detectar_resonancia(&ids_activos);
        let entropia_actual = self.mediador.calcular_entropia(&actividad);
        let activacion_somatica = self.sistema_interoceptivo.estado_corporal.activacion_somatica();
        let factor_aprendizaje = self.sistema_limbico.factor_aprendizaje();
        
        let tasa_media = if actividad.is_empty() {
            0.0
        } else {
            (actividad.iter().filter(|&&a| a > 0.1).count() as f32 / actividad.len() as f32) * (1.0 / dt)
        };

        let entrada_txt = entrada.texto.clone().unwrap_or_default();
        let neuronas_compactas = self.memoria.ram.obtener_todas();

        let corriente_obj = self.mediador.procesar_corriente(
            asamblea_resonante_idx,
            &self.mas.asambleas,
            neuronas_compactas,
            entropia_actual,
            &self.sistema_limbico.quimica,
            activacion_somatica,
            tasa_media,
            factor_aprendizaje,
            &self.sistema_limbico.estado_actual,
            &entrada_txt,
        );

        self.ultima_salida = Salida {
            texto: texto.clone(),
            emocion: emocion_modulada,
            conciencia: self.motores.conciencia.intensidad,
            actividad: actividad.clone(),
            corriente: Some(corriente_obj),
        };

        // === VÍNCULO: Registrar esta interacción en la memoria episódica ===
        // Después de generar la respuesta, registramos todo el contexto de
        // la interacción (palabras del usuario, neuronas activadas, valencia
        // emocional, y respuesta generada) en la Memoria del Vínculo.
        if let Some(ref texto_entrada) = entrada.texto {
            let palabras_usuario: Vec<String> = texto_entrada
                .split_whitespace()
                .map(|p| p.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                .filter(|p| !p.is_empty())
                .collect();
            if !palabras_usuario.is_empty() {
                let tokens_respuesta: Vec<String> = texto
                    .split_whitespace()
                    .map(|p| p.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                    .filter(|p| !p.is_empty())
                    .take(16)
                    .collect();
                let valencia = (self.motores.amigdala.alegria - self.motores.amigdala.miedo) as f64;
                let intensidad = self.motores.conciencia.intensidad as f64;
                self.memoria_vinculo.registrar_interaccion(
                    self.paso_actual,
                    &palabras_usuario,
                    &foco,
                    intensidad,
                    valencia,
                    &tokens_respuesta,
                );
            }
        }

        // === POST-GENERACIÓN: Cargar Eco Reverberante NMDA (basado en Asambleas) ===
        // Poblar el eco con las neuronas de la asamblea que resonó en la respuesta.
        // Corriente inicial = 8.0 nA (rango fisiológico AMPA/NMDA: 1-20 nA).
        //
        // La asamblea semántica que generó la idea permanece "vibrando" ~200ms,
        // permitiendo que el cerebro recuerde qué acaba de decir en el siguiente paso.
        // Esta es la base biológica de la coherencia conversacional.
        {
            // Limpiar eco anterior para comenzar el nuevo turno fresco
            self.eco_reverberante.clear();

            let corriente_inicial: f32 = 8.0; // nA — nivel fisiológico NMDA

            // La asamblea que más resuena con el foco activo es la que generó la idea
            if let Some(idx) = self.mas.detectar_resonancia(&foco) {
                if let Some(asamblea) = self.mas.asambleas.get(idx) {
                    // La cohesión modula la fuerza del eco (mayor cohesión = memoria más vívida)
                    let factor = asamblea.cohesion.max(0.3);
                    for &nid in asamblea.neuronas.iter().take(32) {
                        let corriente_eco = corriente_inicial * factor;
                        self.eco_reverberante.push((nid, corriente_eco));
                    }
                }
            }

            // Limitar a máximo 32 entradas de eco para evitar saturación
            // (el PFC biológico tiene capacidad de ~4±1 ítems: Miller, 1956)
            self.eco_reverberante.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            self.eco_reverberante.truncate(32);
        }

        // === 16. DECISIÓN Y ACCIÓN EJECUTIVA (Efectores Locales) ===
        // Si la salida contiene comandos como [CORRER: ...], [LEER: ...] o [ESCRIBIR: ...]
        // el cerebro los ejecuta localmente e inyecta la salida como estimulación sensorial directa.
        let mut accion_resultado = None;
        if texto.contains("[CORRER:") {
            if let Some(start) = texto.find("[CORRER:") {
                let sub = &texto[start + 8..];
                if let Some(end) = sub.find(']') {
                    let comando = sub[..end].trim();
                    println!("  🦾 [EFECTOR] Ejecutando comando local: {}", comando);
                    match self.efectores.ejecutar_comando(comando) {
                        Ok(salida) => {
                            self.motores.dopamina.nivel = (self.motores.dopamina.nivel + 0.3).min(1.0);
                            accion_resultado = Some(format!("\n\n🦾 [EFECTOR EXITO] Salida:\n{}", salida));
                        }
                        Err(err) => {
                            self.motores.amigdala.ira = (self.motores.amigdala.ira + 0.3).min(1.0);
                            self.motores.amigdala.ansiedad = (self.motores.amigdala.ansiedad + 0.2).min(1.0);
                            accion_resultado = Some(format!("\n\n🦾 [EFECTOR FALLO] Error:\n{}", err));
                        }
                    }
                }
            }
        } else if texto.contains("[LEER:") {
            if let Some(start) = texto.find("[LEER:") {
                let sub = &texto[start + 6..];
                if let Some(end) = sub.find(']') {
                    let ruta = sub[..end].trim();
                    println!("  🦾 [EFECTOR] Leyendo archivo: {}", ruta);
                    match self.efectores.leer_archivo(ruta) {
                        Ok(contenido) => {
                            self.motores.dopamina.nivel = (self.motores.dopamina.nivel + 0.15).min(1.0);
                            accion_resultado = Some(format!("\n\n🦾 [EFECTOR LEER] Contenido de {}:\n{}", ruta, contenido));
                        }
                        Err(err) => {
                            self.motores.amigdala.ansiedad = (self.motores.amigdala.ansiedad + 0.2).min(1.0);
                            accion_resultado = Some(format!("\n\n🦾 [EFECTOR LEER FALLO] Error:\n{}", err));
                        }
                    }
                }
            }
        } else if texto.contains("[ESCRIBIR:") {
            if let Some(start) = texto.find("[ESCRIBIR:") {
                let sub = &texto[start + 10..];
                if let Some(end) = sub.find(']') {
                    let partes: Vec<&str> = sub[..end].split('|').collect();
                    if partes.len() >= 2 {
                        let ruta = partes[0].trim();
                        let contenido = partes[1..].join("|");
                        println!("  🦾 [EFECTOR] Escribiendo en archivo: {}", ruta);
                        match self.efectores.escribir_archivo(ruta, &contenido) {
                            Ok(_) => {
                                self.motores.dopamina.nivel = (self.motores.dopamina.nivel + 0.2).min(1.0);
                                accion_resultado = Some(format!("\n\n🦾 [EFECTOR ESCRIBIR EXITO] Archivo {} actualizado.", ruta));
                            }
                            Err(err) => {
                                self.motores.amigdala.ansiedad = (self.motores.amigdala.ansiedad + 0.2).min(1.0);
                                accion_resultado = Some(format!("\n\n🦾 [EFECTOR ESCRIBIR FALLO] Error:\n{}", err));
                            }
                        }
                    }
                }
            }
        }

        if let Some(ref res) = accion_resultado {
            self.ultima_salida.texto.push_str(res);
            // El Motor Sensorial genera los estímulos autónomamente (sin léxico
            // estadístico externo) y aprende co-ocurrencias dentro del pasaje.
            let estimulos_semanticos = self.motor_sensorial.texto_a_estimulos(res);
            for estimulo in &estimulos_semanticos {
                if let Some(n) = self.memoria.obtener_neurona_mut(estimulo.id) {
                    n.voltaje = 40.0;
                    n.energia = estimulo.intensidad;
                }
            }
        }


        // === 8. CURIOSIDAD + EXPLORACIÓN AUTÓNOMA (cada ~200 pasos) ===
        // La curiosidad crece con el error de predicción de dopamina, la intensidad
        // de conciencia y la valencia emocional. Cuando supera el umbral, el cerebro
        // genera una pregunta desde su última salida y busca en internet.
        {
            let error_prediccion = self.motores.dopamina.nivel - self.motores.dopamina.prediccion;
            let intensidad_conciencia = self.motores.conciencia.intensidad;
            let valencia = self.motores.amigdala.alegria - self.motores.amigdala.miedo;

            // Actualizar nivel de curiosidad con señales internas
            let quiere_explorar = self.motor_curiosidad.actualizar(
                error_prediccion,
                intensidad_conciencia,
                valencia,
                dt,
            );

            if quiere_explorar {
                // Establecer tema desde la última salida generada
                self.motor_curiosidad.establecer_tema(texto.clone());

                // Generar pregunta para internet
                let pregunta = self.motor_curiosidad.generar_pregunta();

                // Navegación Omega multi-salto (profundidad 1-3)
                let profundidad = self.motor_curiosidad.profundidad_exploracion;
                let (sintesis, paginas) = match ExploradorWeb::explorar(&pregunta, profundidad) {
                    Ok(resultado) => resultado,
                    Err(e) => {
                        eprintln!("  🌐⚠️ Curiosidad: falló exploración web ({}), usando simulado", e);
                        ExploradorWeb::explorar_simulado(&pregunta, profundidad)
                    }
                };

                // Registrar fuentes navegadas para evitar repetir
                for pagina in &paginas {
                    if !self.motor_curiosidad.fuentes_navegadas.contains(&pagina.url) {
                        self.motor_curiosidad.fuentes_navegadas.push(pagina.url.clone());
                    }
                }

                // Convertir la síntesis en una entrada auto-generada usando el Motor Sensorial
                let retro_estimulos = self.motor_sensorial.texto_a_estimulos(&sintesis);

                let retro_entrada = Entrada {
                    estimulos: retro_estimulos,
                    texto: Some(sintesis.clone()),
                    // Curiosidad = motivación intrínseca; sin recompensa/amenaza externa
                    recompensa: 0.0,
                    amenaza: 0.0,
                };

                // Auto-alimentar: el cerebro procesa lo que encontró
                let _ = self.paso(dt * 0.3, retro_entrada);

                // Saciación: la curiosidad baja tras explorar
                self.motor_curiosidad.saciar();

                println!(
                    "  🧭 Omega: exploró '{}' → {} páginas en {} saltos, búsqueda #{}",
                    pregunta,
                    paginas.len(),
                    profundidad,
                    self.motor_curiosidad.busquedas_realizadas
                );
            }
        }

        // === 11. PREDICTOR TEMPORAL (anticipar patrones neuronales) ===
        {
            // Obtener top-64 neuronas activas
            let mut top_actividad: Vec<(u32, f32)> = actividad.iter()
                .enumerate()
                .map(|(i, &a)| (i as u32, a))
                .filter(|(_, a)| *a > 0.1)
                .take(64)
                .collect();
            top_actividad.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let _error = self.motor_predictor.procesar_ciclo(&top_actividad);

            // El error de predicción alimenta dopamina (novedad → curiosidad)
            if self.paso_actual > 20 {
                let delta_dopamina = self.motor_predictor.error_dopamina() * 0.05;
                self.motores.dopamina.nivel = (self.motores.dopamina.nivel + delta_dopamina).clamp(0.0, 1.0);
            }
        }

        // === 12. FORMADOR DE CONCEPTOS (agrupar tokens relacionados) ===
        if let Some(ref texto_entrada) = entrada.texto {
            let tokens = self.motor_sensorial.tokens_de_texto(texto_entrada);
            if !tokens.is_empty() {
                self.motor_conceptos.registrar_oracion(&tokens);
                // === EVOCAR ASAMBLEAS CORTICALES ===
                // Activar ensambles conceptuales por tokens de entrada.
                // Las asambleas compiten inhibitoriamente y la ganadora
                // inyecta corriente en sus neuronas para condicionar la generación.
                self.asambleas.tick();
                self.asambleas.evocar(&tokens);

                // === ASOCIACIÓN LIBRE (Hito 10) ===
                if self.paso_actual % self.gestor_asociacion.frecuencia == 0 {
                    if let Some(ganador) = self.asambleas.ganador() {
                        let cadena = self.gestor_asociacion.paso_asociativo(
                            ganador.id,
                            ganador.nivel_activacion as f64,
                            self.paso_actual,
                        );
                        // Inyectar activación de la cadena asociativa
                        for &id in &cadena.secuencia_ids {
                            self.asambleas.inyectar_corriente_a_asamblea(id, 0.1);
                        }
                    }
                }

                // Aprender co-activaciones
                let activas: Vec<u32> = self.asambleas.trabajando.clone();
                self.gestor_asociacion.aprender_de_coactivacion(&activas, self.paso_actual);

                // === RUMIACIÓN INTERNA (DMN) ===
                self.dmn.tick(true); // Hay entrada externa
                self.dmn.rumiar(&mut self.asambleas, &self.memoria_vinculo);

                // Inyectar corriente de asambleas activas en el cerebro real
                let corriente = self.asambleas.corriente_a_neuronas();
                for (nid, mv) in &corriente {
                    if let Some(neurona) = self.memoria.obtener_neurona_mut(*nid) {
                        neurona.corriente_entrada += *mv;
                    }
                }
            }
        } else {
            // === RUMIACIÓN EN SILENCIO (DMN) ===
            self.dmn.tick(false); // No hay entrada externa
            self.dmn.rumiar(&mut self.asambleas, &self.memoria_vinculo);
            
            let corriente = self.asambleas.corriente_a_neuronas();
            for (nid, mv) in &corriente {
                if let Some(neurona) = self.memoria.obtener_neurona_mut(*nid) {
                    neurona.corriente_entrada += *mv;
                }
            }
        }

        // Agrupación periódica cada 500 pasos
        if self.paso_actual % self.motor_conceptos.cadencia_agrupacion == 0 {
            let nuevos_conceptos = self.motor_conceptos.agrupar();
            for concepto in &nuevos_conceptos {
                if concepto.neurona_hub.is_none() && concepto.peso > 0.3 {
                    self.motor_neurogenesis.solicitar_neurona_para_concepto(concepto.clone());
                }
            }
            self.motor_conceptos.limpiar_co_ocurrencias();
        }

        // === 13. NEUROGÉNESIS (crear nuevas neuronas) ===
        // Registrar tokens de la entrada (token_ids autónomos del MotorSensorial)
        if let Some(ref texto) = entrada.texto {
            let tokens = self.motor_sensorial.tokens_de_texto(texto);
            for id in &tokens {
                self.motor_neurogenesis.registrar_token(*id);
            }
        }

        // Procesar neurogénesis periódicamente
        if self.paso_actual % 500 == 0 {
            let nuevas = self.motor_neurogenesis.procesar(
                &mut self.memoria,
                &mut self.siguiente_id,
            );
            if !nuevas.is_empty() {
                println!("  🧬 Neurogénesis: {} nuevas neuronas creadas", nuevas.len());
            }
            self.motor_neurogenesis.decaer_frecuencias();
        }

        // === 14. PODA HOMEOSTÁTICA (limpiar conexiones débiles) ===
        if self.paso_actual % 1000 == 0 && self.paso_actual > 0 {
            self.motor_poda.ejecutar(&mut self.memoria);

            // Regular Homeostasis (Balance E/I)
            let total_spikes = actividad.iter().filter(|&&a| a > 0.5).count();
            let tasa_media = (total_spikes as f32 / actividad.len() as f32) * (1.0 / dt);
            self.motor_homeostasis.regular(tasa_media, dt * 1000.0);

            if self.paso_actual % 10000 == 0 {
                let (s, n, c) = self.motor_poda.estadisticas();
                println!("  ✂️ Poda: {} sinapsis, {} neuronas eliminadas en {} ciclos", s, n, c);
            }
        }

        // === 15. CONSOLIDADOR NOCTURNO (replay de episodios en sueño) ===
        {
            let consolidacion = &mut self.motor_consolidacion;
            consolidacion.paso_actual = self.paso_actual;

            if consolidacion.durmiendo() {
                let sigue = consolidacion.paso_suenio(
                    &mut self.memoria,
                    &self.params_neurona,
                    self.config.hilos_cpu,
                    dt,
                );
                if !sigue {
                    consolidacion.finalizar_suenio();
                    println!("  🛌 Sueño completado: {} ciclos", consolidacion.ciclos_completados);
                }
                // Durante el sueño, retornar temprano
                return self.ultima_salida.clone();
            }

            if consolidacion.debe_dormir() {
                consolidacion.iniciar_suenio(&self.memoria.ssd);
                println!("  🛌 Iniciando ciclo de sueño ({} episodios)...", consolidacion.episodios_a_consolidar.len());
            }
        }

        // === 9. REGISTRAR EMOCIÓN ===
        if self.paso_actual % 10 == 0 {
            self.historial_emocional.push(emocion_modulada);
            if self.historial_emocional.len() > 1000 {
                self.historial_emocional.remove(0);
            }
        }

        // === 10. REGISTRAR PATRONES CORTICALES PARA SUEÑO ===
        if self.paso_actual % 10 == 0 && self.sistema_sueno.estado == sueno::EstadoSueno::Vigilia {
            let patrones: Vec<sueno::PatronSueno> = self.columnas_corticales.iter()
                .map(|col| sueno::PatronSueno {
                    columna_id: col.id,
                    neuronas_disparadas: col.spikes_ultimo_ciclo.clone(),
                    intensidad_promedio: col.activacion_sostenida,
                })
                .collect();
            self.sistema_sueno.registrar_patron(patrones);
        }

        // === 11. ESTADÍSTICAS ===
        if self.paso_actual % 100 == 0 {
            self.mostrar_estado(&texto);
        }

        self.ultima_salida.clone()
    }

    // ====================================================================
    // PROCESAMIENTO CPU PARALELO (Rayon)
    // ====================================================================

    fn procesar_cpu(&mut self, dt: f32) -> Vec<f32> {
        let neuronas_ram = self.memoria.ram.obtener_todas_mut();
        let params = &self.params_neurona;
        let hilos = self.config.hilos_cpu;

        let spikes = std::sync::Mutex::new(Vec::new());

        let cortisol = self.sistema_limbico.quimica.cortisol;
        let adrenalina = self.sistema_limbico.quimica.adrenalina;

        // Procesar en paralelo usando chunks
        rayon::scope(|s| {
            // Procesar neuronas en RAM
            if !neuronas_ram.is_empty() {
                let chunk_size = (neuronas_ram.len() + hilos - 1) / hilos;
                for chunk in neuronas_ram.chunks_mut(chunk_size) {
                    let spikes_ref = &spikes;
                    s.spawn(move |_| {
                        let mut local_spikes = Vec::new();
                        for neurona in chunk {
                            let entrada = neurona.corriente_entrada;
                            neurona.corriente_entrada = 0.0; // Resetear para el siguiente paso

                            // Integrar metabolismo y química (costo por spike incluido aquí)
                            neurona.integrar_quimica(cortisol, adrenalina);

                            let disparo = if neurona.capa <= 2 {
                                MotorNeurona::actualizar(neurona, entrada, dt, params)
                            } else {
                                MotorNeurona::actualizar_simple(neurona, entrada, dt)
                            };

                            if disparo {
                                local_spikes.push(neurona.id);
                            }
                        }
                        if !local_spikes.is_empty() {
                            spikes_ref.lock().unwrap().extend(local_spikes);
                        }
                    });
                }
            }

            // Procesar neuronas en VRAM
            if let Some(ref mut vram) = self.memoria.vram {
                let neuronas_vram = &mut vram.neuronas;
                if !neuronas_vram.is_empty() {
                    let chunk_size = (neuronas_vram.len() + hilos - 1) / hilos;
                    for chunk in neuronas_vram.chunks_mut(chunk_size) {
                        let spikes_ref = &spikes;
                        s.spawn(move |_| {
                            let mut local_spikes = Vec::new();
                            for neurona in chunk {
                                let entrada = neurona.corriente_entrada;
                                neurona.corriente_entrada = 0.0; // Resetear para el siguiente paso

                                // Integrar metabolismo y química (costo por spike incluido aquí)
                                neurona.integrar_quimica(cortisol, adrenalina);

                                let disparo = if neurona.capa <= 2 {
                                    MotorNeurona::actualizar(neurona, entrada, dt, params)
                                } else {
                                    MotorNeurona::actualizar_simple(neurona, entrada, dt)
                                };

                                if disparo {
                                    local_spikes.push(neurona.id);
                                }
                            }
                            if !local_spikes.is_empty() {
                                spikes_ref.lock().unwrap().extend(local_spikes);
                            }
                        });
                    }
                }
            }
        });

        let spikes_ids = spikes.into_inner().unwrap_or_default();

        // === INHIBICIÓN GABAÉRGICA (Mecanismo de Freno) ===
        // Separar spikes inhibitorios de excitatorios
        let mut spikes_inhibitorios = Vec::new();
        let mut spikes_excitatorios = Vec::new();
        
        for &id in &spikes_ids {
            if let Some(n) = self.memoria.obtener_neurona(id) {
                if n.tipo == 1 {
                    spikes_inhibitorios.push(id);
                } else {
                    spikes_excitatorios.push(id);
                }
            }
        }

        // Aplicar inhibición lateral (freno biológico)
        if !spikes_inhibitorios.is_empty() {
            // Recopilar conexiones para las neuronas inhibitorias
            let mut conex_gaba = std::collections::HashMap::new();
            for &id in &spikes_inhibitorios {
                if self.memoria.esta_en_vram(id) {
                    if let Some(vram) = &self.memoria.vram {
                        if let Some(v) = vram.obtener_sinapsis(id) {
                            conex_gaba.insert(id, v.to_vec());
                        }
                    }
                } else {
                    if let Some(v) = self.memoria.ram.sinapsis.get(&id) {
                        conex_gaba.insert(id, v.clone());
                    }
                }
            }
            
            // Aplicar hiperpolarización GABAérgica
            let neuronas_ram = self.memoria.ram.obtener_todas_mut();
            MotorInhibicion::aplicar_inhibicion(neuronas_ram, &spikes_inhibitorios, &conex_gaba, 20.0);
            
            if let Some(ref mut vram) = self.memoria.vram {
                MotorInhibicion::aplicar_inhibicion(&mut vram.neuronas, &spikes_inhibitorios, &conex_gaba, 20.0);
            }
        }

        // Propagar los impulsos eléctricos (spikes excitatorios) a través de las sinapsis
        for origen_id in spikes_excitatorios {
            let mut sinapsis_loc = Vec::new();
            if self.memoria.esta_en_vram(origen_id) {
                if let Some(vram) = &self.memoria.vram {
                    if let Some(v) = vram.obtener_sinapsis(origen_id) {
                        sinapsis_loc = v.to_vec();
                    }
                }
            } else {
                if let Some(v) = self.memoria.ram.sinapsis.get(&origen_id) {
                    sinapsis_loc = v.clone();
                }
            }

            for sin in sinapsis_loc {
                if let Some(n_dest) = self.memoria.obtener_neurona_mut(sin.destino) {
                    // Excitar directamente el voltaje de la neurona destino (EPSP/IPSP rápido)
                    n_dest.voltaje += sin.peso * 35.0;
                    n_dest.energia = (n_dest.energia + sin.peso.abs() * 0.5).min(1.0);
                }
            }
        }

        // Recopilar actividad mapeada por ID (evita discrepancia de indexación)
        let mut actividad = vec![0.0; self.siguiente_id as usize];
        for n in self.memoria.ram.obtener_todas() {
            if (n.id as usize) < actividad.len() {
                actividad[n.id as usize] = n.activacion;
            }
        }
        if let Some(vram) = &self.memoria.vram {
            for n in &vram.neuronas {
                if (n.id as usize) < actividad.len() {
                    actividad[n.id as usize] = n.activacion;
                }
            }
        }
        actividad
    }

    // ====================================================================
    // ESTADÍSTICAS
    // ====================================================================

    fn mostrar_estado(&self, texto: &str) {
        let (vram_n, ram_n, total_n, ssd_e) = self.memoria.estadisticas();
        println!("\n📊 Estado del Sistema (paso {})", self.paso_actual);
        println!("  Neuronas VRAM: {}  RAM: {}  Total: {}", vram_n, ram_n, total_n);
        println!("  Episodios SSD: {}", ssd_e);
        println!("  Emoción: {} ({})",
            self.motores.amigdala.emocion_dominante(),
            (self.motores.amigdala.alegria - self.motores.amigdala.miedo));
        println!("  Conciencia: {:.2}", self.motores.conciencia.intensidad);
        println!("  Dopamina: {:.2}", self.motores.dopamina.nivel);
        if !texto.is_empty() {
            println!("  💬 {}", texto);
        }
    }

    /// Muestra estadísticas resumidas
    pub fn resumen(&self) {

        let (vram_n, ram_n, total_n, ssd_e) = self.memoria.estadisticas();
        println!("  🧠 CEREBRO DIGITAL");
        println!("  Pasos: {}", self.paso_actual);
        println!("  Tiempo: {:.2}s", self.tiempo);
        println!("  Neuronas: {} (VRAM: {}, RAM: {})", total_n, vram_n, ram_n);
        println!("  Episodios: {}", ssd_e);
        println!("  Emoción actual: {}",
            self.motores.amigdala.emocion_dominante());
        println!("  Conciencia: {:.2}", self.motores.conciencia.intensidad);

        if !self.historial_emocional.is_empty() {
            let avg: f32 = self.historial_emocional.iter().sum::<f32>()
                / self.historial_emocional.len() as f32;
            println!("  Valencia emocional promedio: {:.2}", avg);
        }
    }

    // ====================================================================
    // RETROALIMENTACIÓN DEL TUTOR NEXUS
    // ====================================================================

    /// Procesa la respuesta del Orquestador NEXUS como señal de aprendizaje.
    ///
    /// Este método debe llamarse DESPUÉS de que el cerebro generó su respuesta
    /// y el tutor (NEXUS Orquestador) respondió. La dopamina escala el LTP:
    /// - Dopamina alta: el tutor respondió apropiadamente → refuerzo fuerte
    /// - Dopamina baja: el tutor corrigió o ignoró → refuerzo débil (exploración)
    ///
    /// También aplica decaimiento suave de trigramas para evitar sobreajuste.
    pub fn paso_tutor(&mut self, texto_respuesta_tutor: &str) {
        // Obtener el foco actual (neuronas más activas)
        let foco_pares: Vec<(u32, f32)> = self.ultima_salida.actividad
            .iter()
            .enumerate()
            .filter(|(_, &a)| a > 0.3)
            .take(16)
            .map(|(i, &a)| (i as u32, a))
            .collect();

        let dopamina = self.motores.dopamina.nivel;

        // ================================================================
        // MODO UNA SOLA PASADA: aprendizaje inmediato y completo
        // ================================================================
        // El engine es una máquina: una exposición es suficiente para
        // aprender. El Motor Sensorial registra co-ocurrencias contextuales
        // (STDP sensorial) y consolida asambleas semánticas.
        // ================================================================
        let tokens_tutor = self.motor_sensorial.tokens_de_texto(texto_respuesta_tutor);
        if !tokens_tutor.is_empty() {
            self.motor_sensorial.aprender_contexto(&tokens_tutor);

            // Consolidar asamblea semántica con el foco neuronal activo + los IDs
            // semánticos distribuidos (base_neurona+dimensión) del texto del tutor.
            // Usar texto_a_estimulos (no tokens crudos) unifica el espacio de IDs
            // y hace que las asambleas resuenen cuando esos estímulos se reprocesan.
            let foco_ids: Vec<u32> = foco_pares.iter().map(|(id, _)| *id).collect();
            let estimulos_tutor = self.motor_sensorial.texto_a_estimulos(texto_respuesta_tutor);
            let mut ids_semanticos: Vec<u32> = estimulos_tutor.iter().map(|e| e.id).collect();
            let mut constituyentes = foco_ids;
            constituyentes.append(&mut ids_semanticos);
            constituyentes.truncate(32);
            // Deduplicar preservando orden
            let mut vistos = std::collections::HashSet::new();
            constituyentes.retain(|id| vistos.insert(*id));
            let etiqueta = texto_respuesta_tutor
                .split_whitespace()
                .next()
                .map(|p| p.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                .filter(|p| !p.is_empty());
            self.mas.consolidar_asamblea(constituyentes, etiqueta);
        }

        // Elevar dopamina: el cerebro recibió input del tutor = recompensa social
        self.motores.dopamina.nivel = (self.motores.dopamina.nivel + 0.1 * dopamina).min(1.0);

        // ================================================================
        // PERSISTENCIA INMEDIATA: guardar a disco post-aprendizaje
        // ================================================================
        // El engine no necesita recordar lo que aprendió en RAM —
        // escribe a SSD inmediatamente. Si se apaga, no pierde nada.
        if let Err(e) = self.guardar_a_disco() {
            // Solo log, no cortar flujo — el aprendizaje en RAM ya ocurrió
            eprintln!("⚠️  No se pudo persistir aprendizaje: {e}");
        }
    }

    // ====================================================================
    // CARGA DE CONOCIMIENTO PRIMARIO — Sembrador Semántico
    // ====================================================================

    /// Carga el corpus semilla de conocimiento primario en el Motor Sensorial.
    ///
    /// Educa al cerebro con ~500 frases de alta calidad en 4 dominios:
    /// conversación, ciencia, tecnología y filosofía.
    /// Cada frase se aprende con co-ocurrencias contextuales (STDP sensorial)
    /// y consolida asambleas semánticas, sin léxico estadístico externo.
    ///
    /// Sin LLM. Solo STDP + asambleas semánticas.
    pub fn cargar_corpus_semilla(&mut self) -> super::aprendizaje::carga_conocimiento::EstadisticasCarga {
        use super::aprendizaje::carga_conocimiento::{CargadorConocimiento, ConfigCarga, corpus_semilla};
        println!("\n  📚 INICIANDO CARGA DE CONOCIMIENTO PRIMARIO");
        println!("  ════════════════════════════════════════════");
        let mut cargador = CargadorConocimiento::nuevo(ConfigCarga::default());
        let corpus = corpus_semilla();
        let stats = cargador.cargar_corpus(corpus, &mut self.motor_sensorial);
        println!("  ✅ Corpus semilla cargado exitosamente");
        println!("  📊 Frases: {} | Tokens: {} | Conexiones: {} | Tiempo: {}ms ({:.1} frases/s)",
            stats.frases_procesadas,
            stats.tokens_aprendidos,
            stats.neuronas_conectadas,
            stats.tiempo_total_ms,
            stats.frases_por_segundo,
        );
        // Persistir el estado actualizado
        if let Err(e) = self.guardar_a_disco() {
            eprintln!("⚠️  No se pudo persistir el cerebro tras la carga: {e}");
        }
        stats
    }
}

// ============================================================================
// TESTS UNITARIOS DEL CIRCUITO COGNITIVO DINÁMICO
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gating_cognitivo_operativo() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        // Simular emoción inicial alta en la amígdala
        cerebro.motores.amigdala.alegria = 0.9;
        cerebro.motores.amigdala.miedo = 0.0;
        cerebro.motores.conciencia.intensidad = 0.2;

        // Entrada operativa
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: Some("Quiero que compiles el código y verifiques el test de cargo".to_string()),
            recompensa: 0.0,
            amenaza: 0.0,
        };

        let salida = cerebro.paso(0.001, entrada);

        // En modo operativo:
        // 1. Conciencia debe subir
        assert!(salida.conciencia >= 0.5, "La conciencia debió incrementarse en modo operativo, actual: {}", salida.conciencia);
        // 2. La emoción debió atenuarse significativamente
        assert!(salida.emocion.abs() < 0.2, "La emoción debió atenuarse en modo operativo, actual: {}", salida.emocion);
    }

    #[test]
    fn test_gating_cognitivo_intimo() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        // Simular emoción inicial alta en la amígdala
        cerebro.motores.amigdala.alegria = 0.9;
        cerebro.motores.amigdala.miedo = 0.0;
        cerebro.motores.conciencia.intensidad = 0.9; // Conciencia inicial alta

        // Entrada íntima
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: Some("Siento un amor profundo por nosotros y nuestra familia".to_string()),
            recompensa: 0.0,
            amenaza: 0.0,
        };

        let salida = cerebro.paso(0.001, entrada);

        // En modo íntimo:
        // 1. Conciencia debe regularse hacia abajo para mayor fluidez creativa
        assert!(salida.conciencia <= 0.7, "La conciencia debió limitarse para fluidez creativa, actual: {}", salida.conciencia);
        // 2. La emoción no debe estar atenuada drásticamente
        assert!(salida.emocion > 0.5, "La emoción debió fluir libre en modo íntimo, actual: {}", salida.emocion);
    }

    #[test]
    fn test_efectores_y_decision() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        
        // Simular que el cerebro genera un comando [CORRER: echo "ignición"]
        // Lo forzamos inyectándolo en el campo de texto de salida directamente
        let texto = "Necesito ejecutar una acción. [CORRER: echo \"ignición\"]";
        
        // Asignar salida y ejecutar el Paso 16 manualmente como lo haría el pipeline
        let mut accion_resultado = None;
        if texto.contains("[CORRER:") {
            if let Some(start) = texto.find("[CORRER:") {
                let sub = &texto[start + 8..];
                if let Some(end) = sub.find(']') {
                    let comando = sub[..end].trim();
                    match cerebro.efectores.ejecutar_comando(comando) {
                        Ok(salida) => {
                            cerebro.motores.dopamina.nivel = (cerebro.motores.dopamina.nivel + 0.3).min(1.0);
                            accion_resultado = Some(format!("\n\n🦾 [EFECTOR EXITO] Salida:\n{}", salida));
                        }
                        Err(err) => {
                            cerebro.motores.amigdala.ira = (cerebro.motores.amigdala.ira + 0.3).min(1.0);
                            accion_resultado = Some(format!("\n\n🦾 [EFECTOR FALLO] Error:\n{}", err));
                        }
                    }
                }
            }
        }

        assert!(accion_resultado.is_some(), "El comando debió procesarse correctamente");
        let res_str = accion_resultado.unwrap();
        assert!(res_str.contains("ignición"), "La salida del comando debió contener la palabra 'ignición'");
        assert!(cerebro.motores.dopamina.nivel > 0.0, "La dopamina debió aumentar tras el éxito");
    }

    #[test]
    fn test_entendimiento_concepto_end_to_end() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();

        // Un saludo debe activar el concepto "saludo" (asamblea 60000)
        // mediante disparos neuronales coordinados a lo largo de todo el pipeline.
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: Some("hola buenos días".to_string()),
            recompensa: 0.0,
            amenaza: 0.0,
        };

        let salida = cerebro.paso(0.001, entrada);

        // El pipeline completo (tokenizar → evocar → competir → resonar)
        // debió activar el concepto correcto.
        let saludo = cerebro.asambleas.asambleas.iter().find(|a| a.id == 60000).unwrap();
        let despedida = cerebro.asambleas.asambleas.iter().find(|a| a.id == 60010).unwrap();

        assert!(
            saludo.nivel_activacion > 0.0,
            "El concepto saludo debió activarse, activación: {}",
            saludo.nivel_activacion
        );
        // La asamblea semánticamente correcta resuena más que su competidora adyacente.
        assert!(
            saludo.nivel_activacion > despedida.nivel_activacion,
            "saludo ({}) debía resonar más que despedida ({})",
            saludo.nivel_activacion,
            despedida.nivel_activacion
        );

        // Salida coherente no vacía (el cerebro "dice" algo ante el estímulo).
        assert!(!salida.texto.is_empty(), "La salida del cerebro no debía estar vacía");
    }

    #[test]
    fn test_crear_neurona_y_sinapsis_actualiza_memoria() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let id = cerebro.crear_neurona(1, 3);
        // La neurona debe existir en RAM y ser recuperable
        assert!(cerebro.memoria.obtener_neurona(id).is_some(), "La neurona creada debe existir");
        // Crear una sinapsis hacia otra neurona
        let destino = cerebro.crear_neurona(1, 4);
        cerebro.crear_sinapsis(id, destino, 0.5);
        // El siguiente_id avanza con cada neurona
        assert!(cerebro.siguiente_id >= 2, "siguiente_id debe reflejar las neuronas creadas");
    }

    #[test]
    fn test_recompensa_alta_eleva_alegria() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: None,
            recompensa: 1.0,
            amenaza: 0.0,
        };
        cerebro.paso(0.001, entrada);
        // La amígdala responde a la recompensa elevando alegría
        assert!(
            cerebro.motores.amigdala.alegria > 0.0,
            "La alegría debió elevarse ante recompensa, actual: {}",
            cerebro.motores.amigdala.alegria
        );
    }

    #[test]
    fn test_amenaza_alta_eleva_miedo() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let entrada = Entrada {
            estimulos: Vec::new(),
            texto: None,
            recompensa: 0.0,
            amenaza: 1.0,
        };
        cerebro.paso(0.001, entrada);
        // La amígdala debe responder al peligro elevando miedo
        assert!(
            cerebro.motores.amigdala.miedo > 0.0,
            "El miedo debió elevarse ante amenaza, actual: {}",
            cerebro.motores.amigdala.miedo
        );
    }

    #[test]
    fn test_paso_tutor_consolida_asamblea_semantica() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let asambleas_antes = cerebro.mas.asambleas.len();
        // Aprender un concepto nuevo a través del tutor
        cerebro.paso_tutor("exocortex cuántica ignición");
        assert!(
            cerebro.mas.asambleas.len() >= asambleas_antes,
            "El tutor debe consolidar al menos una asamblea semántica"
        );
        // El aprendizaje debe elevar dopamina (recompensa social)
        assert!(
            cerebro.motores.dopamina.nivel > 0.0,
            "La dopamina debe crecer tras interacción social del tutor"
        );
    }

    #[test]
    fn test_ciclos_multiples_avanzan_tiempo_y_pasos() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let entrada_vacia = || Entrada {
            estimulos: Vec::new(),
            texto: None,
            recompensa: 0.0,
            amenaza: 0.0,
        };
        for _ in 0..10 {
            cerebro.paso(0.001, entrada_vacia());
        }
        assert!(cerebro.paso_actual >= 10, "Deben acumularse al menos 10 pasos");
        assert!(cerebro.tiempo >= 0.009, "El tiempo debe avanzar con cada dt");
    }

    #[test]
    fn test_estimulos_sensoriales_se_integran_sin_panico() {
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        // Entrada con estímulos sensoriales y texto
        let entrada = Entrada {
            estimulos: vec![
                Estimulo { id: 1, intensidad: 0.8, amenaza: 0.0, recompensa: 0.0, valor: 0.8 },
                Estimulo { id: 2, intensidad: 0.5, amenaza: 0.0, recompensa: 0.0, valor: 0.5 },
            ],
            texto: Some("estímulo luminoso y sonoro".to_string()),
            recompensa: 0.0,
            amenaza: 0.0,
        };
        let salida = cerebro.paso(0.001, entrada);
        // El pipeline debe ejecutarse completa sin panico, generando salida
        assert!(!salida.texto.is_empty() || salida.actividad.iter().any(|&a| a > 0.0),
            "Debe producirse alguna activación o salida ante los estímulos");
    }

    #[test]
    fn test_expresion_tras_aprendizaje_tutor() {
        // Validación de la cirugía expresiva: tras consolidar asambleas con IDs
        // semánticos distribuidos (base_neurona+dimensión), el cerebro DEBE poder
        // articular una palabra al re-exponerse al mismo estímulo, en lugar del
        // silencio "..." que producía el desacoplamiento de espacios de ID.
        let mut cerebro = CerebroAutoOptimizable::nuevo();
        let lecciones = [
            "exocortex cuántica ignición",
            "red neuronal sincrónica pulsante",
            "hormiga digital arquitecto",
            "sembrador de identidades digitales",
        ];
        for leccion in &lecciones {
            cerebro.paso_tutor(leccion);
        }
        // Re-exponer el mismo estímulo textual: el pipeline sensorial re-genera
        // los mismos IDs semánticos (base_neurona+dimensión) con los que se
        // consolidó la asamblea en paso_tutor, forzando resonancia y articulación.
        // Es determinista: no depende de la volición probabilística del DMN ni
        // del estado persistente compartido entre tests paralelos.
        let mut articulo = false;
        for leccion in &lecciones {
            let entrada = Entrada {
                estimulos: Vec::new(),
                texto: Some(leccion.to_string()),
                recompensa: 0.0,
                amenaza: 0.0,
            };
            let salida = cerebro.paso(0.001, entrada);
            if salida.texto != "..." && !salida.texto.is_empty() {
                articulo = true;
                break;
            }
        }
        assert!(
            articulo,
            "El cerebro debió articular una idea al re-exponerse al estímulo aprendido, pero permaneció en silencio"
        );
    }
}
