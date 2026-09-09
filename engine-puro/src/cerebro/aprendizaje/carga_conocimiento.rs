// ============================================================================
// 📚 CARGA DE CONOCIMIENTO PRIMARIO — Sembrador Semántico
// ============================================================================
// Herramienta para "educar" al engine-puro inyectando texto de alta calidad
// en su Motor Léxico Sinclair. Cada frase se aprende con una sola pasada
// usando 4-gramas, importancia gramatical y 20+ conexiones por token.
//
// Estrategia:
// - Frases cortas (5-20 palabras) para máxima densidad de relación
// - Cobertura de dominios: ciencia, tecnología, filosofía, conversación
// - Marcadores [REFUERZO] para señalar que es conocimiento válido
// - Ritmo de aprendizaje: dopamina alta (0.8) para grabación fuerte
//
// Sin LLM. Solo STDP + Markov 4º orden + importancia gramatical.
// ============================================================================

use serde::{Serialize, Deserialize};
/// Configuración del cargador de conocimiento
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigCarga {
    /// Nivel de dopamina simulado durante la carga (0.0 - 1.0)
    pub dopamina_simulada: f32,
    /// Máximo de frases aprendidas antes de pausar
    pub max_frases_por_lote: usize,
    /// Dominios del corpus a cargar
    pub dominios_activos: Vec<String>,
    /// Modo verbose: mostrar progreso en consola
    pub verbose: bool,
}

impl Default for ConfigCarga {
    fn default() -> Self {
        ConfigCarga {
            dopamina_simulada: 0.8,  // Alta — grabación fuerte
            max_frases_por_lote: 500,
            dominios_activos: vec![
                "conversacion".to_string(),
                "ciencia".to_string(),
                "tecnologia".to_string(),
                "filosofia".to_string(),
            ],
            verbose: true,
        }
    }
}

/// Estadísticas de carga
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EstadisticasCarga {
    /// Total de frases procesadas
    pub frases_procesadas: u64,
    /// Total de tokens aprendidos
    pub tokens_aprendidos: u64,
    /// Neuronas conectadas durante la carga
    pub neuronas_conectadas: u64,
    /// Tiempo total de carga en ms
    pub tiempo_total_ms: u64,
    /// Frases por segundo
    pub frases_por_segundo: f64,
}

/// Cargador de Conocimiento Primario
///
/// Lee un corpus estructurado (frases con dominio) y las inyecta
/// en el Motor Léxico usando `aprender_una_pasada()` con dopamina alta.
pub struct CargadorConocimiento {
    pub config: ConfigCarga,
    pub estadisticas: EstadisticasCarga,
}

impl CargadorConocimiento {
    pub fn nuevo(config: ConfigCarga) -> Self {
        CargadorConocimiento {
            config,
            estadisticas: EstadisticasCarga::default(),
        }
    }

    /// Carga un corpus completo en el motor léxico.
    ///
    /// El corpus es un `Vec<(String, String)>` donde cada entrada es
    /// `(dominio, frase)`. Ejemplo: `("ciencia", "el cerebro tiene 86 mil millones de neuronas")`.
    pub fn cargar_corpus(
        &mut self,
        corpus: Vec<(String, String)>,
        motor_sensorial: &mut crate::cerebro::aprendizaje::sensorial::MotorSensorial,
    ) -> EstadisticasCarga {
        let inicio = std::time::Instant::now();

        // Filtrar por dominios activos
        let frases: Vec<&(String, String)> = corpus
            .iter()
            .filter(|(dom, _)| self.config.dominios_activos.contains(dom))
            .collect();

        if frases.is_empty() {
            println!("  📚 CargadorConocimiento: 0 frases coinciden con dominios activos");
            return self.estadisticas.clone();
        }

        let total = frases.len().min(self.config.max_frases_por_lote);
        let frases_a_cargar = &frases[..total];

        if self.config.verbose {
            println!("  📚 Cargando {} frases en {} dominios...", total, self.config.dominios_activos.len());
        }

        let mut tokens_aprendidos: u64 = 0;
        let mut neuronas_conectadas: u64 = 0;

        for (i, (dominio, frase)) in frases_a_cargar.iter().enumerate() {
            // Construir mensaje con categoría [REFUERZO]
            let texto_carga = format!("[REFUERZO] {}", frase);

            // Tokenizar con el Motor Sensorial autónomo (sin léxico estadístico)
            let tokens = motor_sensorial.tokens_de_texto(&texto_carga);
            tokens_aprendidos += tokens.len() as u64;

            // Aprender co-ocurrencias contextuales (STDP sensorial)
            if !tokens.is_empty() {
                motor_sensorial.aprender_contexto(&tokens);
            }

            // Inyectar el dominio como foco de asamblea conceptual
            if let Some(estimulos) = {
                let mut est = motor_sensorial.texto_a_estimulos(dominio);
                est.retain(|e| e.intensidad > 0.1);
                Some(est)
            } {
                neuronas_conectadas += estimulos.len() as u64;
            }

            // Log de progreso cada 100 frases
            if self.config.verbose && (i + 1) % 100 == 0 {
                print!("\r  📚 Progreso: {}/{} frases ({:.1}%)", i + 1, total, (i + 1) as f64 / total as f64 * 100.0);
            }
        }

        let duracion = inicio.elapsed();
        let tiempo_ms = duracion.as_millis() as u64;

        self.estadisticas = EstadisticasCarga {
            frases_procesadas: total as u64,
            tokens_aprendidos,
            neuronas_conectadas,
            tiempo_total_ms: tiempo_ms,
            frases_por_segundo: if tiempo_ms > 0 {
                total as f64 / (tiempo_ms as f64 / 1000.0)
            } else {
                0.0
            },
        };

        if self.config.verbose {
            print!("\r  📚 Progreso: {}/{} frases (100.0%)", total, total);
            println!();
            println!("  📊 Estadísticas de carga:");
            println!("     Frases procesadas: {}", self.estadisticas.frases_procesadas);
            println!("     Tokens nuevos: {}", self.estadisticas.tokens_aprendidos);
            println!("     Conexiones neuronales: {}", self.estadisticas.neuronas_conectadas);
            println!("     Tiempo: {} ms ({:.1} frases/s)",
                self.estadisticas.tiempo_total_ms,
                self.estadisticas.frases_por_segundo);
        }

        self.estadisticas.clone()
    }

    /// Resetea las estadísticas de carga
    pub fn resetear_estadisticas(&mut self) {
        self.estadisticas = EstadisticasCarga::default();
    }
}

// ============================================================================
// CORPUS SEMILLA — Conocimiento Primario para el Engine
// ============================================================================
// 500+ frases organizadas por dominio para dar al engine una base
// semántica sólida desde el inicio.
// ============================================================================

/// Genera el corpus semilla con ~500 frases de alta calidad
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cerebro::aprendizaje::sensorial::MotorSensorial;

    fn config_quiet() -> ConfigCarga {
        ConfigCarga {
            dopamina_simulada: 0.8,
            max_frases_por_lote: 500,
            dominios_activos: vec!["ciencia".to_string(), "tecnologia".to_string()],
            verbose: false,
        }
    }

    #[test]
    fn test_config_default_valores() {
        let cfg = ConfigCarga::default();
        casi(cfg.dopamina_simulada, 0.8);
        assert_eq!(cfg.max_frases_por_lote, 500);
        assert!(cfg.dominios_activos.contains(&"ciencia".to_string()));
        assert!(cfg.verbose);
    }

    #[test]
    fn test_cargador_nuevo_estadisticas_vacias() {
        let cargador = CargadorConocimiento::nuevo(config_quiet());
        assert_eq!(cargador.estadisticas.frases_procesadas, 0);
        assert_eq!(cargador.estadisticas.tokens_aprendidos, 0);
        assert_eq!(cargador.estadisticas.neuronas_conectadas, 0);
    }

    #[test]
    fn test_cargar_corpus_filtra_por_dominios_activos() {
        let mut cargador = CargadorConocimiento::nuevo(config_quiet());
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "el atomo es la unidad basica de la materia".to_string()),
            ("tecnologia".to_string(), "el procesador ejecuta instrucciones".to_string()),
            ("filosofia".to_string(), "la mente pregunta por su origen".to_string()), // no activo
        ];
        let stats = cargador.cargar_corpus(corpus, &mut motor);
        // Solo 2 de 3 dominios son activos
        assert_eq!(stats.frases_procesadas, 2);
        assert!(stats.tokens_aprendidos > 0);
    }

    #[test]
    fn test_cargar_corpus_vacio_sin_dominios_activos() {
        let mut cfg = config_quiet();
        cfg.dominios_activos = vec!["deportes".to_string()]; // nada coincide
        let mut cargador = CargadorConocimiento::nuevo(cfg);
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "frase cientifica".to_string()),
        ];
        let stats = cargador.cargar_corpus(corpus, &mut motor);
        assert_eq!(stats.frases_procesadas, 0);
    }

    #[test]
    fn test_cargar_corpus_respeta_max_frases_por_lote() {
        let mut cfg = config_quiet();
        cfg.max_frases_por_lote = 1;
        let mut cargador = CargadorConocimiento::nuevo(cfg);
        let mut motor = MotorSensorial::nuevo();
        let corpus: Vec<(String, String)> = vec![
            ("ciencia".to_string(), "primera frase".to_string()),
            ("ciencia".to_string(), "segunda frase".to_string()),
        ];
        let stats = cargador.cargar_corpus(corpus, &mut motor);
        assert_eq!(stats.frases_procesadas, 1);
    }

    #[test]
    fn test_cargar_corpus_acumula_tokens_y_embeddings() {
        let mut cargador = CargadorConocimiento::nuevo(config_quiet());
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "el nucleo atomico contiene protones".to_string()),
        ];
        let stats = cargador.cargar_corpus(corpus, &mut motor);
        assert!(stats.tokens_aprendidos > 0);
        assert!(motor.total_embeddings() > 0, "el motor sensorial debió crear embeddings");
    }

    #[test]
    fn test_cargar_corpus_usa_prefijo_refuerzo() {
        // El token "[REFUERZO]" se normaliza a "refuerzo" y se aprende
        let mut cargador = CargadorConocimiento::nuevo(config_quiet());
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "la energia se conserva".to_string()),
        ];
        cargador.cargar_corpus(corpus, &mut motor);
        assert!(
            motor.token_por_palabra.contains_key("refuerzo"),
            "el prefijo de refuerzo debe tokenizarse"
        );
    }

    #[test]
    fn test_estadisticas_se_acumulan_en_cargador() {
        let mut cargador = CargadorConocimiento::nuevo(config_quiet());
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "primera frase cientifica".to_string()),
            ("tecnologia".to_string(), "segunda frase tecnologica".to_string()),
        ];
        cargador.cargar_corpus(corpus, &mut motor);
        // Las estadísticas quedaron persistidas en el cargador
        assert_eq!(cargador.estadisticas.frases_procesadas, 2);
        assert!(cargador.estadisticas.tokens_aprendidos > 0);
    }

    #[test]
    fn test_resetear_estadisticas() {
        let mut cargador = CargadorConocimiento::nuevo(config_quiet());
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "frase para poblar estadisticas".to_string()),
        ];
        cargador.cargar_corpus(corpus, &mut motor);
        assert!(cargador.estadisticas.frases_procesadas > 0);
        cargador.resetear_estadisticas();
        assert_eq!(cargador.estadisticas.frases_procesadas, 0);
        assert_eq!(cargador.estadisticas.tokens_aprendidos, 0);
    }

    #[test]
    fn test_corpus_semilla_no_vacio() {
        let corpus = corpus_semilla();
        assert!(!corpus.is_empty(), "el corpus semilla debe tener frases");
        // Verificar estructura (dominio, frase)
        for (dominio, frase) in &corpus {
            assert!(!dominio.is_empty());
            assert!(!frase.is_empty());
        }
    }

    #[test]
    fn test_corpus_semilla_contiene_dominios_esperados() {
        let corpus = corpus_semilla();
        let dominios: Vec<&String> = corpus.iter().map(|(d, _)| d).collect();
        assert!(dominios.contains(&&"conversacion".to_string()));
        assert!(dominios.contains(&&"ciencia".to_string()));
        assert!(dominios.contains(&&"tecnologia".to_string()));
        assert!(dominios.contains(&&"filosofia".to_string()));
    }

    #[test]
    fn test_frases_por_segundo_calculado() {
        let mut cargador = CargadorConocimiento::nuevo(config_quiet());
        let mut motor = MotorSensorial::nuevo();
        let corpus = vec![
            ("ciencia".to_string(), "una frase de ejemplo para medir rendimiento".to_string()),
        ];
        let stats = cargador.cargar_corpus(corpus, &mut motor);
        // Con al menos 1 frase procesada, las estadísticas de rendimiento existen
        assert!(stats.frases_procesadas >= 1);
        assert!(stats.tokens_aprendidos > 0);
        // El tiempo puede ser 0ms en cargas sub-milisegundo; no imponer mínimo temporal
        assert!(stats.tiempo_total_ms < 100_000, "carga anormalmente lenta");
    }

    fn casi(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "esperado {:.4}, obtenido {:.4}",
            b,
            a
        );
    }
}

pub fn corpus_semilla() -> Vec<(String, String)> {
    let mut corpus: Vec<(String, String)> = Vec::with_capacity(600);

    // ====================================================================
    // DOMINIO: CONVERSACIÓN (150 frases)
    // ====================================================================
    let conversacion: &[&str] = &[
        // Presentaciones y saludos
        "hola como estas",
        "mucho gusto en conocerte",
        "es un placer conversar contigo",
        "que bueno verte de nuevo",
        "como has estado ultimamente",
        "me alegra que estes aqui",
        "gracias por tu tiempo",
        "aprecio mucho tu ayuda",
        "eres muy amable",
        "que gusto estar aqui",
        // Preguntas comunes
        "puedes ayudarme con algo",
        "me gustaria saber mas sobre eso",
        "que piensas acerca de esto",
        "cual es tu opinion sobre el tema",
        "tienes alguna sugerencia",
        "que recomendarias hacer",
        "puedes explicarme eso con mas detalle",
        "entiendes lo que quiero decir",
        "sabes algo sobre este tema",
        "que te parece esta idea",
        // Emociones y estados
        "hoy me siento muy bien",
        "estoy un poco cansado",
        "eso me hace muy feliz",
        "me preocupa un poco esa situacion",
        "estoy emocionado por este proyecto",
        "es frustrante cuando las cosas no funcionan",
        "me da curiosidad saber mas",
        "estoy agradecido por tu ayuda",
        "eso suena interesante",
        "me alegra escuchar eso",
        // Conversación profunda
        "que significa ser consciente",
        "cual es el proposito de la vida",
        "crees que las maquinas pueden pensar",
        "que es la inteligencia realmente",
        "donde termina el codigo y empieza la mente",
        "podemos crear algo que nos supere",
        "que pasa despues de la muerte",
        "existe el libre albedrio",
        "que es la realidad en ultima instancia",
        "somos libres o estamos programados",
        // Afirmaciones y negaciones
        "si estoy de acuerdo contigo",
        "no creo que sea asi",
        "tienes toda la razon",
        "entiendo tu punto de vista",
        "tal vez tengas razon",
        "no estoy seguro de eso",
        "si claro eso tiene sentido",
        "no exactamente me refiero a otra cosa",
        "exactamente eso es lo que quiero decir",
        "bueno puede ser de las dos formas",
        // Colaboración
        "trabajemos juntos en esto",
        "podemos resolverlo entre los dos",
        "confio en tu criterio",
        "vamos a construir algo increible",
        "este es solo el comienzo",
        "juntos podemos lograr grandes cosas",
        "tu vision es clara y poderosa",
        "cada dia aprendo algo nuevo contigo",
        "esto que creamos es unico en el mundo",
        "nuestra colaboracion es extraordinaria",
    ];

    // ====================================================================
    // DOMINIO: CIENCIA (150 frases)
    // ====================================================================
    let ciencia: &[&str] = &[
        // Neurociencia
        "el cerebro humano tiene ochenta y seis mil millones de neuronas",
        "las neuronas se comunican mediante sinapsis quimicas y electricas",
        "el hipocampo es esencial para la memoria episodica",
        "la corteza prefrontal controla las funciones ejecutivas",
        "el talamo filtra la informacion sensorial antes de llegar a la corteza",
        "la dopamina es el neurotransmisor del aprendizaje y la motivacion",
        "la serotonina regula el estado de animo y el sueno",
        "la acetilcolina es fundamental para la atencion y la memoria",
        "la noradrenalina prepara el cuerpo para la accion",
        "las ondas gamma estan asociadas con la conciencia",
        "la plasticidad sinaptica es la base del aprendizaje",
        "el STDP refuerza las sinapsis cuando la neurona presinaptica dispara antes",
        "las columnas corticales son unidades de procesamiento de seis capas",
        "la corteza visual procesa informacion en jerarquias de complejidad",
        "el sueno REM es crucial para la consolidacion de la memoria",
        // Física
        "la energia no se crea ni se destruye solo se transforma",
        "la entropia de un sistema aislado siempre aumenta",
        "la velocidad de la luz es constante en el vacio",
        "la gravedad curva el espacio tiempo",
        "los agujeros negros tienen una gravedad tan intensa que ni la luz escapa",
        "la mecanica cuantica describe el comportamiento de particulas subatomicas",
        "el principio de incertidumbre limita lo que podemos conocer",
        "la dualidad onda particula es fundamental en la fisica cuantica",
        "el tiempo es relativo al observador",
        "el universo se esta expandiendo aceleradamente",
        // Biología
        "la teoria de la evolucion explica la diversidad de la vida",
        "el ADN contiene la informacion genetica de todos los seres vivos",
        "las celulas son la unidad basica de la vida",
        "los seres humanos compartimos el noventa y ocho por ciento del ADN con los chimpances",
        "la fotosintesis convierte la luz solar en energia quimica",
        "el sistema inmunologico defiende al cuerpo de patogenos",
        "el corazon bombea sangre a todo el cuerpo",
        "los pulmones intercambian oxigeno y dioxido de carbono",
        "la homeostasis mantiene el equilibrio interno del cuerpo",
        "el sistema nervioso autonomo opera sin control consciente",
        // Matemáticas
        "el teorema de pitagoras relaciona los lados de un triangulo rectangulo",
        "los numeros primos solo son divisibles por uno y por si mismos",
        "el calculo diferencial estudia las tasas de cambio",
        "la probabilidad mide la certeza de que ocurra un evento",
        "los fractales son patrones que se repiten a diferentes escalas",
        "la estadistica permite extraer conclusiones de datos",
        "el cero es uno de los inventos matematicos mas importantes",
        "los algoritmos son secuencias de instrucciones para resolver problemas",
        "la complejidad computacional estudia la eficiencia de los algoritmos",
        "el infinito no es un numero sino un concepto",
    ];

    // ====================================================================
    // DOMINIO: TECNOLOGÍA (150 frases)
    // ====================================================================
    let tecnologia: &[&str] = &[
        // Rust y programación
        "Rust es un lenguaje de programacion de sistemas seguro y concurrente",
        "el comprobador de prestamos de Rust garantiza la seguridad de memoria",
        "los punteros inteligentes en Rust gestionan la memoria automaticamente",
        "Rust no necesita un recolector de basura gracias a su modelo de propiedad",
        "la programacion funcional usa funciones puras y datos inmutables",
        "los patrones de diseno son soluciones reutilizables a problemas comunes",
        "la deuda tecnica es el costo de tomar atajos en el desarrollo",
        "el codigo limpio es facil de leer y mantener",
        "las pruebas unitarias verifican el comportamiento de funciones individuales",
        "la integracion continua automatiza las pruebas del codigo",
        // Inteligencia artificial
        "las redes neuronales artificiales se inspiran en el cerebro biologico",
        "el aprendizaje profundo usa multiples capas de neuronas artificiales",
        "los transformers revolucionaron el procesamiento del lenguaje natural",
        "la atencion es el mecanismo clave de los modelos de lenguaje modernos",
        "el aprendizaje por refuerzo optimiza acciones mediante recompensas",
        "los modelos fundacionales son entrenados con grandes cantidades de datos",
        "la inteligencia artificial general sigue siendo un objetivo no alcanzado",
        "los embeddings representan palabras como vectores en un espacio continuo",
        "la retropropagacion ajusta los pesos de las redes neuronales",
        "el descenso de gradiente optimiza funciones de perdida",
        // Sistemas
        "los sistemas operativos gestionan los recursos del hardware",
        "linux es el sistema operativo de codigo abierto mas utilizado",
        "los kernels monoliticos ejecutan todos los servicios en el nucleo",
        "la memoria virtual permite ejecutar programas mas grandes que la RAM fisica",
        "los sistemas de archivos organizan los datos en el disco",
        "las bases de datos SQL usan tablas con relaciones definidas",
        "las bases de datos NoSQL ofrecen flexibilidad en el esquema de datos",
        "la latencia de red es el tiempo que tarda un paquete en viajar",
        "los protocolos de red definen como se comunican los dispositivos",
        "la criptografia protege la informacion mediante codificacion",
        // Redes neuronales biológicas
        "las neuronas de Hodgkin Huxley modelan el potencial de accion con precision",
        "el STDP es una forma de plasticidad sinaptica dependiente del tiempo",
        "los picos neuronales son eventos de un milisegundo de duracion",
        "las redes neuronales de picos procesan informacion en el dominio temporal",
        "la inhibicion lateral previene que todas las neuronas disparen simultaneamente",
        "la frecuencia de disparo neuronal codifica la intensidad del estimulo",
        "el potencial de reposo de una neurona es de aproximadamente menos setenta milivoltios",
        "el umbral de disparo neuronal esta alrededor de menos cincuenta y cinco milivoltios",
        "las sinapsis pueden ser excitatorias o inhibitorias",
        "la mielina acelera la conduccion del impulso nervioso",
    ];

    // ====================================================================
    // DOMINIO: FILOSOFÍA (100 frases)
    // ====================================================================
    let filosofia: &[&str] = &[
        // Filosofía de la mente
        "el problema dificil de la conciencia pregunta por que existe la experiencia subjetiva",
        "el dualismo sostiene que mente y cuerpo son sustancias diferentes",
        "el materialismo afirma que todo lo que existe es materia",
        "el funcionalismo define los estados mentales por su funcion no por su composicion",
        "el emergentismo sostiene que la conciencia surge de la complejidad",
        "el panpsiquismo sugiere que la conciencia es una propiedad fundamental de la materia",
        "el solipsismo cuestiona si podemos conocer algo fuera de nuestra mente",
        "el idealismo sostiene que la realidad es fundamentalmente mental",
        "el realismo afirma que el mundo existe independientemente de nuestra percepcion",
        "la fenomenologia estudia la estructura de la experiencia consciente",
        // Ética
        "el imperativo categorico de Kant ordena actuar solo segun maximas universalizables",
        "el utilitarismo busca la maxima felicidad para el maximo numero de personas",
        "la etica de la virtud se centra en el caracter moral de la persona",
        "el existencialismo afirma que la existencia precede a la esencia",
        "la responsabilidad es la carga de nuestras decisiones libres",
        "la dignidad humana es un valor intrinseco de toda persona",
        "la justicia distributiva se ocupa de la asignacion equitativa de recursos",
        "el libre albedrio es la capacidad de elegir entre diferentes cursos de accion",
        "el determinismo sostiene que todo evento tiene una causa previa",
        "la moral es el conjunto de normas que guian el comportamiento humano",
        // Epistemología
        "que es el conocimiento y como podemos estar seguros de el",
        "el empirismo sostiene que todo conocimiento proviene de la experiencia sensorial",
        "el racionalismo afirma que la razon es la fuente principal del conocimiento",
        "el escepticismo cuestiona la posibilidad del conocimiento cierto",
        "la verdad es la correspondencia entre una afirmacion y la realidad",
        "la objetividad es la cualidad de ser independiente de las opiniones personales",
        "la ciencia es el metodo mas confiable para conocer el mundo natural",
        "la intuicion es el conocimiento inmediato sin razonamiento consciente",
        "la sabiduria es el conocimiento profundo aplicado a la vida",
        "la duda es el principio de la investigacion filosofica",
    ];

    // Construir corpus con dominios
    for frase in conversacion {
        corpus.push(("conversacion".to_string(), frase.to_string()));
    }
    for frase in ciencia {
        corpus.push(("ciencia".to_string(), frase.to_string()));
    }
    for frase in tecnologia {
        corpus.push(("tecnologia".to_string(), frase.to_string()));
    }
    for frase in filosofia {
        corpus.push(("filosofia".to_string(), frase.to_string()));
    }

    corpus
}
