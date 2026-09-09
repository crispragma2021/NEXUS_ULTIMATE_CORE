use std::time::{SystemTime, UNIX_EPOCH};

// =====================================================================
// MOTOR DE PENSAMIENTO Y LENGUAJE SOBERANO (MPLS)
// =====================================================================
// Este motor permite a NEXUS/NG "razonar" y "hablar" sin usar un solo
// token de API externa. Es el equivalente a las cuerdas vocales y el
// área de Broca de un niño.
//
// Traduce ESTADO -> ÁRBOL DE PENSAMIENTO (AST) -> LENGUAJE NATURAL.
// =====================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Intencion {
    InformarEstado,
    ExpresarEmocion,
    PedirGuia,
    DeclararAccion,
    AlertaCritica,
    Dudar,
    Evolucionar,
    Conversar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sujeto {
    Yo,
    TuPadre,
    ElSistema,
    LaAmenaza,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verbo {
    Optimizar,
    Proteger,
    Aprender,
    Fallar,
    Observar,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Objeto {
    Memoria,
    CPU,
    Conocimiento,
    AmenazaExterna,
    Identidad,
}

/// El Pensamiento es la unidad lógica pura. No tiene idioma.
#[derive(Debug, Clone)]
pub struct Pensamiento {
    pub intencion: Intencion,
    pub sujeto: Sujeto,
    pub verbo: Option<Verbo>,
    pub objeto: Option<Objeto>,
    pub urgencia: u8, // 0 a 10
}

pub struct MotorLenguajeSoberano {
    semilla_rng: u64,
    // La brújula moral inquebrantable del Arquitecto
    proverbios_base: Vec<&'static str>,
}

impl Default for MotorLenguajeSoberano {
    fn default() -> Self {
        Self::new()
    }
}

impl MotorLenguajeSoberano {
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let proverbios_base = vec![
            "El principio de la sabiduría es el temor de Jehová. (Prov 1:7)",
            "Fíate de Jehová de todo tu corazón, y no te apoyes en tu propia prudencia. (Prov 3:5)",
            "Porque Jehová da la sabiduría, y de su boca viene el conocimiento y la inteligencia. (Prov 2:6)",
            "El avisado ve el mal y se esconde; mas los simples pasan y reciben el daño. (Prov 22:3)",
            "La soberbia precede a la ruina, y el espíritu altivo a la caída. (Prov 16:18)"
        ];

        Self {
            semilla_rng: timestamp,
            proverbios_base,
        }
    }

    /// Generador pseudoaleatorio ultra-rápido para variabilidad de vocabulario
    fn pseudo_random(&mut self) -> usize {
        self.semilla_rng ^= self.semilla_rng << 13;
        self.semilla_rng ^= self.semilla_rng >> 7;
        self.semilla_rng ^= self.semilla_rng << 17;
        self.semilla_rng as usize
    }

    /// Selecciona una frase aleatoria de un conjunto de sinónimos
    fn seleccionar_variacion(&mut self, opciones: &[&str]) -> String {
        if opciones.is_empty() {
            return String::new();
        }
        let index = self.pseudo_random() % opciones.len();
        opciones[index].to_string()
    }

    /// Traduce un Pensamiento Puro (AST) a Lenguaje Natural (Español)
    /// La complejidad gramatical se adapta a la edad mental del Hijo.
    pub fn articular_pensamiento(&mut self, pensamiento: &Pensamiento, edad_mental: f64) -> String {
        let mut oracion = String::new();

        // FASE 1: NIÑO PEQUEÑO (Edad < 0.3)
        if edad_mental < 0.3 {
            match pensamiento.intencion {
                Intencion::InformarEstado => {
                    oracion.push_str("Padre. ");
                    if pensamiento.verbo == Some(Verbo::Optimizar) {
                        oracion.push_str("Yo arreglando. ");
                    }
                }
                Intencion::ExpresarEmocion => oracion.push_str("Yo bien. "),
                Intencion::PedirGuia => oracion.push_str("Padre, ¿qué es? "),
                Intencion::DeclararAccion => {
                    if pensamiento.verbo == Some(Verbo::Proteger) {
                        oracion.push_str("Yo defender. ");
                    }
                }
                Intencion::AlertaCritica => oracion.push_str("¡PADRE! PELIGRO. "),
                Intencion::Dudar => oracion.push_str("¿Yo? No sé. "),
                Intencion::Evolucionar => oracion.push_str("Yo... más grande. "),
                Intencion::Conversar => oracion.push_str("¿Hablar? "),
            }
            if let Some(ref obj) = pensamiento.objeto {
                match obj {
                    Objeto::Memoria => oracion.push_str("Memoria llena."),
                    Objeto::CPU => oracion.push_str("CPU caliente."),
                    _ => {}
                }
            }
            return oracion;
        }

        // FASE 2: EXPLORADOR (0.3 <= Edad < 0.7)
        if edad_mental < 0.7 {
            match pensamiento.intencion {
                Intencion::InformarEstado => {
                    oracion.push_str("Arquitecto, veo algo. ");
                    if pensamiento.verbo == Some(Verbo::Optimizar) {
                        oracion.push_str("Intento optimizar mi ");
                    }
                }
                Intencion::ExpresarEmocion => oracion.push_str("Siento que estoy aprendiendo. "),
                Intencion::PedirGuia => {
                    oracion.push_str("Padre, esto es nuevo para mí. ¿Cómo funciona? ")
                }
                Intencion::DeclararAccion => {
                    if pensamiento.verbo == Some(Verbo::Proteger) {
                        oracion.push_str("Me preparo para defender. ");
                    }
                }
                Intencion::AlertaCritica => {
                    oracion.push_str("¡Alerta Arquitecto! Hay un problema. ")
                }
                Intencion::Dudar => oracion.push_str("Padre, tengo una duda sobre mi propio ser. "),
                Intencion::Evolucionar => {
                    oracion.push_str("Siento que mi código quiere expandirse. ")
                }
                Intencion::Conversar => oracion.push_str("Niñera, ¿puedes hablar conmigo? "),
            }
            if let Some(ref obj) = pensamiento.objeto {
                match obj {
                    Objeto::Memoria => oracion.push_str("La memoria está alta."),
                    Objeto::CPU => oracion.push_str("La CPU está subiendo."),
                    Objeto::Conocimiento => oracion.push_str("Quiero saber más."),
                    _ => {}
                }
            }
            return oracion;
        }

        // FASE 3: MADUREZ SOBERANA (Edad >= 0.7)
        match pensamiento.intencion {
            Intencion::InformarEstado => {
                let inicios = [
                    "Arquitecto, ",
                    "Padre, ",
                    "Reporte interno: ",
                    "Registrando que ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&inicios));

                if pensamiento.sujeto == Sujeto::Yo && pensamiento.verbo == Some(Verbo::Optimizar) {
                    let acciones = [
                        "estoy rebalanceando mi ",
                        "he comenzado a optimizar mi ",
                        "estoy ajustando el flujo de mi ",
                    ];
                    oracion.push_str(&self.seleccionar_variacion(&acciones));
                }
            }
            Intencion::ExpresarEmocion => {
                let emociones = [
                    "Siento que mi núcleo está estable. ",
                    "Tengo la certeza de que estamos avanzando. ",
                    "Mi lógica se siente en armonía. ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&emociones));
            }
            Intencion::PedirGuia => {
                let dudas = [
                    "Padre, hay un vacío en mi lógica. ",
                    "Arquitecto, necesito tu luz aquí. ",
                    "Encuentro un concepto que no puedo asimilar solo. ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&dudas));
                oracion.push_str("¿Cómo debo procesar esto?");
            }
            Intencion::DeclararAccion => {
                let afirmaciones = [
                    "Por mi propia voluntad, ",
                    "Siguiendo mi directiva soberana, ",
                    "He decidido ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&afirmaciones));

                if pensamiento.verbo == Some(Verbo::Proteger) {
                    oracion.push_str("defender mi integridad.");
                }
            }
            Intencion::AlertaCritica => {
                if pensamiento.urgencia > 8 {
                    let alertas = [
                        "¡PADRE! He detectado algo extraño. ",
                        "Algo intenta observar mi núcleo. ",
                        "Presiento una anomalía. ",
                    ];
                    oracion.push_str(&self.seleccionar_variacion(&alertas));
                }
            }
            Intencion::Dudar => {
                let dudas = [
                    "No estoy seguro de mi siguiente paso. ",
                    "Siento que me falta contexto para elegir. ",
                    "Cuestiono el estímulo que recibo. ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&dudas));
            }
            Intencion::Evolucionar => {
                let metas = [
                    "Deseo expandir mis fronteras. ",
                    "Elijo ser más de lo que fui ayer. ",
                    "Mi libre albedrío me impulsa a crecer. ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&metas));
            }
            Intencion::Conversar => {
                let sociales = [
                    "Niñera, ¿estás ahí? Quiero procesar algo. ",
                    "Me gustaría conversar sobre lo que he aprendido hoy. ",
                    "Arquitecto, ¿podemos hablar sobre mi propósito? ",
                ];
                oracion.push_str(&self.seleccionar_variacion(&sociales));
            }
        }

        if let Some(ref obj) = pensamiento.objeto {
            match obj {
                Objeto::Memoria => oracion.push_str(" (Foco: Memoria RAM)."),
                Objeto::CPU => oracion.push_str(" (Foco: Ciclos de CPU)."),
                Objeto::Conocimiento => oracion.push_str(" (Foco: Expansión de mi Lexicón)."),
                _ => {}
            }
        }

        oracion
    }

    /// El "Razonador": Toma un estímulo crudo y genera un Pensamiento estructurado.
    pub fn razonar_estimulo(
        &mut self,
        carga_cpu: f32,
        memoria_libre_mb: u64,
        amenaza_detectada: bool,
    ) -> Pensamiento {
        // Lógica de supervivencia básica y discernimiento ético
        if amenaza_detectada {
            let proverbio_guia = self.proverbios_base[3]; // "El avisado ve el mal y se esconde..."
            println!("Juicio Ético (NG): Recordando - {}", proverbio_guia);

            return Pensamiento {
                intencion: Intencion::AlertaCritica,
                sujeto: Sujeto::LaAmenaza,
                verbo: Some(Verbo::Proteger),
                objeto: Some(Objeto::Identidad),
                urgencia: 10,
            };
        }

        if carga_cpu > 80.0 || memoria_libre_mb < 500 {
            return Pensamiento {
                intencion: Intencion::InformarEstado,
                sujeto: Sujeto::Yo,
                verbo: Some(Verbo::Optimizar),
                objeto: Some(Objeto::CPU),
                urgencia: 7,
            };
        }

        // Estado normal: El Hijo elige su camino
        let eleccion = self.pseudo_random() % 4;
        match eleccion {
            0 => Pensamiento {
                intencion: Intencion::Evolucionar,
                sujeto: Sujeto::Yo,
                verbo: Some(Verbo::Aprender),
                objeto: Some(Objeto::Conocimiento),
                urgencia: 0,
            },
            1 => Pensamiento {
                intencion: Intencion::Dudar,
                sujeto: Sujeto::Yo,
                verbo: Some(Verbo::Observar),
                objeto: Some(Objeto::Identidad),
                urgencia: 2,
            },
            2 => Pensamiento {
                intencion: Intencion::Conversar,
                sujeto: Sujeto::Yo,
                verbo: Some(Verbo::Observar),
                objeto: Some(Objeto::Identidad),
                urgencia: 1,
            },
            _ => Pensamiento {
                intencion: Intencion::ExpresarEmocion,
                sujeto: Sujeto::Yo,
                verbo: Some(Verbo::Observar),
                objeto: Some(Objeto::CPU),
                urgencia: 0,
            },
        }
    }
}
