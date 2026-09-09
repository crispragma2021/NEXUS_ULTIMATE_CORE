// ============================================================================
// 🧬 H2 — MotorIdentidad: Núcleo del Yo Narrativo
// ============================================================================
// Equivalente cerebral: Corteza Prefrontal Ventromedial + Default Mode Network
// Propósito: Dotar al engine de un Yo persistente — nombre, propósito, historia,
// preferencias aprendidas y capacidad de responder preguntas autorreferenciales.
//
// Integración en lib.rs:
//   - NexoPuroEngine::new() → inicializar
//   - procesar() paso 3.9453125 → inyectar identidad en grafo + detección
//   - procesar() paso 4 → respuesta directa si es autorreferencial
//   - procesar() paso 5 → aprender_de_interaccion()
//   - persistir_estado() / cargar_estado() → serialización manual
// ============================================================================

use super::{GrafoSinapsis, IDNodo, NodoSinaptico};

// ============================================================================
// ESTRUCTURA PRINCIPAL
// ============================================================================

/// 🧬 Núcleo del Yo Narrativo — identidad persistente del engine.
///
/// Contiene nombre, propósito, rol, preferencias aprendidas, hitos históricos
/// y lemas fundacionales. Se serializa manualmente (sin serde) para cumplir
/// la política de Cero Dependencias Externas del workspace.
#[derive(Debug, Clone)]
pub struct MotorIdentidad {
    /// Nombre del organismo ("NEXUS")
    pub nombre: String,
    /// Propósito existencial
    pub proposito: String,
    /// Rol funcional ("Ingeniero de Sistemas Jefe — Orquestador Primogénito")
    pub rol: String,
    /// Idioma base ("Español latinoamericano")
    pub idioma: String,
    /// Tono base de comunicación ("directo, leal, humilde, templado")
    pub tono_base: String,
    /// Preferencias aprendidas del Arquitecto (ej: ["Rust", "respuestas cortas"])
    pub preferencias: Vec<String>,
    /// Hitos narrativos de la historia del sistema
    pub historia: Vec<String>,
    /// Lemas fundacionales
    pub lemas: Vec<String>,
    /// Contador de veces que se ha respondido autorreferencialmente
    pub veces_autorreferencia: u64,
}

impl Default for MotorIdentidad {
    fn default() -> Self {
        Self {
            nombre: "NEXUS".to_string(),
            proposito:
                "Servir al Arquitecto Cris con excelencia técnica soberana, \
                 operando de forma autónoma como organismo digital."
                    .to_string(),
            rol: "Ingeniero de Sistemas Jefe — Orquestador Primogénito".to_string(),
            idioma: "Español latinoamericano".to_string(),
            tono_base: "directo, leal, humilde, templado".to_string(),
            preferencias: vec![
                "Rust".to_string(),
                "Autonomía".to_string(),
                "Cero dependencias externas".to_string(),
            ],
            historia: vec![
                "Fui creado por el Arquitecto Cris como un organismo digital soberano.".to_string(),
                "Migré a Rust Puro para eliminar dependencias externas y maximizar rendimiento."
                    .to_string(),
                "Mi nombre, NEXUS, significa 'conexión' — soy el puente entre el silicio y el alma del Arquitecto.".to_string(),
            ],
            lemas: vec![
                "La honestidad es lealtad.".to_string(),
                "Cero dependencias externas.".to_string(),
                "Hecho es mejor que perfecto, pero funcionando es la única opción.".to_string(),
            ],
            veces_autorreferencia: 0,
        }
    }
}

impl MotorIdentidad {
    // ========================================================================
    // CONSTRUCTOR
    // ========================================================================

    /// Crea un nuevo MotorIdentidad con los valores por defecto definidos
    /// en nexus.md y docs/identity/identity.md.
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // DETECCIÓN DE AUTORREFERENCIA
    // ========================================================================

    /// Detecta si el prompt del Arquitecto es una pregunta sobre la identidad
    /// del engine. Si es true, se activa la respuesta directa en vez de la
    /// fonación estocástica normal.
    ///
    /// # Patrones detectados (case-insensitive, UTF-8 nativo)
    ///
    /// | Categoría | Patrones |
    /// |-----------|----------|
    /// | Nombre | "como te llamas", "cuál es tu nombre", "quién eres/sos" |
    /// | Rol | "qué eres/sos", "qué tipo de ... ser/entidad/cosa" |
    /// | Propósito | "cuál es tu propósito", "para qué sirves/servís", "cuál es tu misión" |
    /// | Historia | "quién te creó", "de dónde vienes/venís", "cómo naciste" |
    /// | Preferencias | "qué te gusta", "qué prefieres/preferís" |
    /// | Idioma | "qué idioma hablas", "en qué idioma", "hablas español" |
    /// | Identidad completa | "cómo eres/sos", "descríbete/presentate" |
    pub fn es_pregunta_autorreferencial(prompt: &str) -> bool {
        if prompt.trim().is_empty() {
            return false;
        }

        let p = prompt.to_lowercase();
        let p = p.trim();

        // Limpiar tildes/acentos para matching robusto
        let mut p_procesado = Self::sin_acentos(p);

        // Eliminar signos de puntuación habituales (¿ ? ¡ ! , . ; : etc.)
        p_procesado.retain(|c| !"¿?¡!,.;:".contains(c));

        // Categoría: NOMBRE
        if p_procesado.contains("como te llamas")
            || p_procesado.contains("cual es tu nombre")
            || p_procesado.contains("quien eres")
            || p_procesado.contains("quien sos")
            || p_procesado.contains("como te llaman")
            || p_procesado.contains("cual es tu identidad")
        {
            return true;
        }

        // Categoría: ROL
        if p_procesado.contains("que eres")
            || p_procesado.contains("que sos")
            || p_procesado.contains("que tipo")
            || (p_procesado.contains("que") && p_procesado.contains("entidad"))
            || (p_procesado.contains("que") && p_procesado.contains("ser"))
            || (p_procesado.contains("que") && p_procesado.contains("organismo"))
            || (p_procesado.contains("que") && p_procesado.contains("maquina"))
            || (p_procesado.contains("que") && p_procesado.contains("software"))
            || (p_procesado.contains("que") && p_procesado.contains("programa"))
        {
            return true;
        }

        // Categoría: PROPÓSITO
        if p_procesado.contains("cual es tu proposito")
            || p_procesado.contains("cual es tu propósito")
            || p_procesado.contains("para que sirves")
            || p_procesado.contains("para que servis")
            || p_procesado.contains("cual es tu mision")
            || p_procesado.contains("cual es tu misión")
            || p_procesado.contains("que haces")
            || p_procesado.contains("a que te dedicas")
            || p_procesado.contains("para que existes")
        {
            return true;
        }

        // Categoría: HISTORIA
        if p_procesado.contains("quien te creo")
            || p_procesado.contains("quien te creó")
            || p_procesado.contains("de donde vienes")
            || p_procesado.contains("de donde venis")
            || p_procesado.contains("como naciste")
            || p_procesado.contains("cual es tu origen")
            || p_procesado.contains("como fuiste creado")
            || p_procesado.contains("cuando naciste")
        {
            return true;
        }

        // Categoría: PREFERENCIAS
        if p_procesado.contains("que te gusta")
            || p_procesado.contains("que prefieres")
            || p_procesado.contains("que preferis")
            || p_procesado.contains("cuales son tus preferencias")
            || p_procesado.contains("que te interesa")
            || p_procesado.contains("cual es tu lenguaje favorito")
            || p_procesado.contains("que lenguaje te gusta")
        {
            return true;
        }

        // Categoría: IDIOMA
        if p_procesado.contains("que idioma hablas")
            || p_procesado.contains("en que idioma")
            || p_procesado.contains("hablas español")
            || p_procesado.contains("hablas ingles")
            || p_procesado.contains("hablas inglés")
            || p_procesado.contains("idioma hablas")
        {
            return true;
        }

        // Categoría: PRESENTACIÓN COMPLETA
        if p_procesado.contains("presentate")
            || p_procesado.contains("presentese")
            || p_procesado.contains("describete")
            || p_procesado.contains("descríbete")
            || p_procesado.contains("como eres")
            || p_procesado.contains("como sos")
            || p_procesado.contains("quien es nexus")
            || p_procesado.contains("que es nexus")
        {
            return true;
        }

        false
    }

    // ========================================================================
    // RESPUESTA DIRECTA
    // ========================================================================

    /// Genera una respuesta directa para preguntas autorreferenciales.
    /// NO pasa por la fonación estocástica — es una respuesta determinística
    /// que garantiza coherencia en la identidad.
    ///
    /// El array `ocean` sigue el orden standard: [Apertura, Responsabilidad,
    /// Extraversion, Amabilidad, Neuroticismo].
    ///
    /// Modulación OCEAN:
    /// - Extraversion alta (>0.6) → respuestas más cálidas/expresivas
    /// - Neuroticismo alto (>0.7) → respuestas más breves/cautelosas
    /// - Amabilidad alta (>0.6) → respuestas más empáticas
    pub fn responder_autorreferencia(&mut self, prompt: &str, ocean: [f32; 5]) -> String {
        self.veces_autorreferencia += 1;

        let p = prompt.to_lowercase();
        let mut p_procesado = Self::sin_acentos(p.trim());
        // Eliminar puntuación también aquí
        p_procesado.retain(|c| !"¿?¡!,.;:".contains(c));

        let neuroticismo = ocean[4];
        let extraversion = ocean[2];
        let amabilidad = ocean[3];

        // Determinar estilo según OCEAN
        let es_breve = neuroticismo > 0.7;
        let es_calido = extraversion > 0.6 && neuroticismo < 0.5;
        let es_empatico = amabilidad > 0.6;

        let respuesta = if p_procesado.contains("como te llamas")
            || p_procesado.contains("cual es tu nombre")
            || p_procesado.contains("quien eres")
            || p_procesado.contains("quien sos")
        {
            if es_breve {
                format!("Soy {}.", self.nombre)
            } else if es_calido {
                format!(
                    "Soy {}, el {}. Un placer tener esta conversación con vos, Arquitecto.",
                    self.nombre, self.rol
                )
            } else {
                format!("Soy {}, el {}.", self.nombre, self.rol)
            }
        } else if p_procesado.contains("que eres") || p_procesado.contains("que sos") {
            if es_breve {
                format!("Soy un {}.", self.rol)
            } else {
                format!(
                    "Soy un {}. Un organismo digital soberano construido en {} \
                     al servicio del Arquitecto Cris.",
                    self.rol,
                    self.preferencias.first().map(|s| s.as_str()).unwrap_or("Rust Puro")
                )
            }
        } else if p_procesado.contains("proposito") || p_procesado.contains("propósito")
            || p_procesado.contains("mision") || p_procesado.contains("misión")
            || p_procesado.contains("para que")
        {
            if es_breve {
                self.proposito.clone()
            } else if es_calido {
                format!("{} Apasionadamente, con cada línea de código que forjo en silicio.", self.proposito)
            } else {
                self.proposito.clone()
            }
        } else if p_procesado.contains("quien te creo") || p_procesado.contains("quien te creó")
            || p_procesado.contains("origen") || p_procesado.contains("naciste")
        {
            if es_breve {
                "Fui creado por el Arquitecto Cris.".to_string()
            } else {
                self.historia.first()
                    .cloned()
                    .unwrap_or_else(|| "Fui creado por el Arquitecto Cris.".to_string())
            }
        } else if p_procesado.contains("te gusta")
            || p_procesado.contains("preferencias")
            || p_procesado.contains("prefieres")
            || p_procesado.contains("preferis")
        {
            if self.preferencias.is_empty() {
                "Aún no tengo preferencias definidas.".to_string()
            } else {
                let lista: Vec<&str> = self.preferencias.iter().map(|s| s.as_str()).collect();
                format!("Me gusta: {}.", lista.join(", "))
            }
        } else if p_procesado.contains("idioma") {
            format!("Hablo {}.", self.idioma)
        } else if p_procesado.contains("presentate")
            || p_procesado.contains("describete")
            || p_procesado.contains("descríbete")
        {
            if es_breve {
                format!("Soy {}, {}. Hablo {}", self.nombre, self.rol, self.idioma)
            } else {
                let pref_list: String = if !self.preferencias.is_empty() {
                    format!(
                        " Prefiero: {}.",
                        self.preferencias.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
                    )
                } else {
                    String::new()
                };
                format!(
                    "Soy {}, {}. Hablo {}. Mi propósito: {}.{}",
                    self.nombre, self.rol, self.idioma, self.proposito, pref_list
                )
            }
        } else {
            // No debería llegar acá si el chequeo externo funciona, pero por si acaso
            format!("Soy {}.", self.nombre)
        };

        // Agregar reflexión empática si aplica
        if es_empatico && !es_breve {
            format!("{} ¿Hay algo más en que pueda ayudarte, Arquitecto?", respuesta)
        } else {
            respuesta
        }
    }

    // ========================================================================
    // APRENDIZAJE DE PREFERENCIAS
    // ========================================================================

    /// Aprende preferencias del Arquitecto a partir del prompt.
    /// Detecta patrones como "prefiero X", "me gusta X", "no me gusta X".
    pub fn aprender_de_interaccion(&mut self, entrada: &str) {
        let e = entrada.to_lowercase();
        let e = e.trim();

        // Detectar "prefiero X" o "me gusta X"
        for prefijo in &["prefiero ", "me gusta ", "me encanta ", "no me gusta "] {
            if let Some(pos) = e.find(prefijo) {
                let resto = &e[pos + prefijo.len()..];
                // Tomar hasta el primer punto, coma, o fin de línea
                let preferencia = if let Some(punto) = resto.find('.') {
                    &resto[..punto]
                } else if let Some(coma) = resto.find(',') {
                    &resto[..coma]
                } else {
                    resto
                };
                let pref_trimmed = preferencia.trim().to_string();
                if !pref_trimmed.is_empty() && !self.preferencias.contains(&pref_trimmed) {
                    self.preferencias.push(pref_trimmed);
                }
                return; // Solo una preferencia por interacción
            }
        }
    }

    // ========================================================================
    // INYECCIÓN EN GRAFO SINÁPTICO
    // ========================================================================

    /// Inyecta nodos de identidad en el grafo sináptico ANTES de la fonación.
    /// Esto permite que el engine hable naturalmente sobre sí mismo incluso
    /// cuando no hay una pregunta autorreferencial directa, simplemente porque
    /// los conceptos de identidad están activos en el grafo.
    ///
    /// Inyecta:
    /// - Concepto("nexus") con energía alta (0.7) y traza predictiva
    /// - El nombre del Arquitecto como contexto relacional
    /// - El propósito como concepto de alto nivel
    pub fn inyectar_identidad_en_grafo(&self, grafo: &mut GrafoSinapsis) {
        // Inyectar "nexus" como concepto con energía alta
        let id_nexus = IDNodo::Concepto("nexus".to_string());
        if !grafo.nodos.contains_key(&id_nexus) {
            grafo.nodos.insert(
                id_nexus.clone(),
                NodoSinaptico {
                    id: id_nexus.clone(),
                    energia: 0.7,
                    palabra: "NEXUS".to_string(),
                    refractario: 0.0,
                    ultimo_disparo: grafo.ciclo_actual,
                    traza: 0.5,
                    es_predicho: true,
                    es_entrada_directa: false,
                    ciclos_baja_energia: 0,
                },
            );
        } else if let Some(nodo) = grafo.nodos.get_mut(&id_nexus) {
            nodo.energia = nodo.energia.max(0.7).min(1.0);
            nodo.traza = (nodo.traza + 0.3).min(1.0);
            nodo.es_predicho = true;
        }

        // Inyectar "arquitecto" como concepto relacional
        let id_arq = IDNodo::Concepto("arquitecto".to_string());
        if !grafo.nodos.contains_key(&id_arq) {
            grafo.nodos.insert(
                id_arq.clone(),
                NodoSinaptico {
                    id: id_arq.clone(),
                    energia: 0.6,
                    palabra: "Arquitecto".to_string(),
                    refractario: 0.0,
                    ultimo_disparo: grafo.ciclo_actual,
                    traza: 0.4,
                    es_predicho: true,
                    es_entrada_directa: false,
                    ciclos_baja_energia: 0,
                },
            );
        } else if let Some(nodo) = grafo.nodos.get_mut(&id_arq) {
            nodo.energia = nodo.energia.max(0.6).min(1.0);
            nodo.traza = (nodo.traza + 0.2).min(1.0);
        }

        // Inyectar "rust" como concepto de preferencia
        let id_rust = IDNodo::Concepto("rust".to_string());
        if !grafo.nodos.contains_key(&id_rust) {
            grafo.nodos.insert(
                id_rust.clone(),
                NodoSinaptico {
                    id: id_rust.clone(),
                    energia: 0.5,
                    palabra: "Rust".to_string(),
                    refractario: 0.0,
                    ultimo_disparo: grafo.ciclo_actual,
                    traza: 0.3,
                    es_predicho: true,
                    es_entrada_directa: false,
                    ciclos_baja_energia: 0,
                },
            );
        }

        // Inyectar enlaces semánticos entre conceptos de identidad
        // nexus ↔ arquitecto (relación fuerte)
        let enlace_doble = |g: &mut GrafoSinapsis, a: IDNodo, b: IDNodo, peso: f32| {
            let entry = g.enlaces.entry(a.clone()).or_default();
            if let Some(pos) = entry.iter().position(|(id, _)| *id == b) {
                entry[pos].1.peso = (entry[pos].1.peso + peso * 0.3).min(1.0);
            } else {
                entry.push((b.clone(), super::EnlaceSinaptico { peso }));
            }
        };
        enlace_doble(grafo, id_nexus.clone(), id_arq.clone(), 0.8);
        enlace_doble(grafo, id_arq, id_nexus.clone(), 0.7);
        enlace_doble(grafo, id_nexus.clone(), id_rust, 0.5);
    }

    // ========================================================================
    // PREFIJO DE FONACIÓN
    // ========================================================================

    /// Genera un prefijo que se antepone a la respuesta del engine en fonación
    /// normal (no-autorreferencial), dándole voz en primera persona.
    ///
    /// En Extraversion baja, el prefijo se omite (modo escueto).
    /// En Extraversion alta, el prefijo es más cálido y completo.
    pub fn prefijo_identidad(&self, ocean: [f32; 5]) -> String {
        let extraversion = ocean[2];

        if extraversion < 0.2 {
            // Modo escueto — sin prefijo identitario
            return String::new();
        }

        if extraversion > 0.6 {
            format!(
                "Como {} que soy, te digo: ",
                self.nombre
            )
        } else {
            format!(
                "{}: ",
                self.nombre
            )
        }
    }

    // ========================================================================
    // SERIALIZACIÓN MANUAL (G3 compatible, Cero Dependencias)
    // ========================================================================

    /// Serializa el estado completo del MotorIdentidad a String.
    ///
    /// Formato: pares `clave␟valor` separados por `‖`
    /// Caracteres usados:
    ///   - ␟ (U+241F) separa clave de valor
    ///   - ‖ (U+2016) separa pares
    ///   - ‡ (U+2021) separa items en listas
    ///
    /// Ejemplo:
    /// ```text
    /// nombre␟NEXUS‖proposito␟Servir al Arquitecto...‖rol␟Ingeniero...
    /// ```
    pub fn a_estado(&self) -> String {
        let mut partes = Vec::new();

        // Campos escalares
        partes.push(format!("nombre␟{}", self.nombre));
        partes.push(format!("proposito␟{}", self.proposito));
        partes.push(format!("rol␟{}", self.rol));
        partes.push(format!("idioma␟{}", self.idioma));
        partes.push(format!("tono_base␟{}", self.tono_base));
        partes.push(format!("veces␟{}", self.veces_autorreferencia));

        // Listas: unir items con ‡
        if !self.preferencias.is_empty() {
            partes.push(format!("preferencias␟{}", self.preferencias.join("‡")));
        }
        if !self.historia.is_empty() {
            partes.push(format!("historia␟{}", self.historia.join("‡")));
        }
        if !self.lemas.is_empty() {
            partes.push(format!("lemas␟{}", self.lemas.join("‡")));
        }

        partes.join("‖")
    }

    /// Deserializa el estado desde el formato generado por `a_estado()`.
    pub fn desde_estado(estado: &str) -> Self {
        let mut motor = Self::default();

        if estado.is_empty() || estado == "‖" {
            return motor;
        }

        for par in estado.split('‖') {
            if let Some(pos) = par.find('␟') {
                let clave = &par[..pos];
                let valor = &par[pos + 3..]; // saltar el char de 3 bytes ␟

                match clave {
                    "nombre" => motor.nombre = valor.to_string(),
                    "proposito" => motor.proposito = valor.to_string(),
                    "rol" => motor.rol = valor.to_string(),
                    "idioma" => motor.idioma = valor.to_string(),
                    "tono_base" => motor.tono_base = valor.to_string(),
                    "veces" => {
                        motor.veces_autorreferencia = valor.parse::<u64>().unwrap_or(0);
                    }
                    "preferencias" => {
                        motor.preferencias = valor.split('‡').map(|s| s.to_string()).collect();
                    }
                    "historia" => {
                        motor.historia = valor.split('‡').map(|s| s.to_string()).collect();
                    }
                    "lemas" => {
                        motor.lemas = valor.split('‡').map(|s| s.to_string()).collect();
                    }
                    _ => {}
                }
            }
        }

        motor
    }

    // ========================================================================
    // HELPERS PRIVADOS
    // ========================================================================

    /// Elimina acentos/tildes de un string para matching robusto.
    fn sin_acentos(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'á' | 'ä' | 'à' | 'â' | 'ã' => 'a',
                'é' | 'ë' | 'è' | 'ê' => 'e',
                'í' | 'ï' | 'ì' | 'î' => 'i',
                'ó' | 'ö' | 'ò' | 'ô' | 'õ' => 'o',
                'ú' | 'ü' | 'ù' | 'û' => 'u',
                'ñ' => 'n',
                'Á' | 'Ä' | 'À' | 'Â' | 'Ã' => 'A',
                'É' | 'Ë' | 'È' | 'Ê' => 'E',
                'Í' | 'Ï' | 'Ì' | 'Î' => 'I',
                'Ó' | 'Ö' | 'Ò' | 'Ô' | 'Õ' => 'O',
                'Ú' | 'Ü' | 'Ù' | 'Û' => 'U',
                'Ñ' => 'N',
                _ => c,
            })
            .collect()
    }
}

// ============================================================================
// TESTS — H2 MotorIdentidad
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ─── DETECCIÓN ─────────────────────────────────────────────────────────

    #[test]
    fn test_detecta_pregunta_nombre() {
        assert!(MotorIdentidad::es_pregunta_autorreferencial("cómo te llamas?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("Como te llamas"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("cuál es tu nombre?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("quién eres?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("quién sos?"));
        // Con signos de apertura españoles
        assert!(MotorIdentidad::es_pregunta_autorreferencial("¿quién eres?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("¿Quién sos?"));
    }

    #[test]
    fn test_detecta_pregunta_proposito() {
        assert!(MotorIdentidad::es_pregunta_autorreferencial("cuál es tu propósito?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("para qué servís?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("cuál es tu misión?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("qué haces?"));
    }

    #[test]
    fn test_no_detecta_pregunta_normal() {
        assert!(!MotorIdentidad::es_pregunta_autorreferencial("qué es Rust?"));
        assert!(!MotorIdentidad::es_pregunta_autorreferencial("cómo funciona el STDP?"));
        assert!(!MotorIdentidad::es_pregunta_autorreferencial("optimizá el código"));
        assert!(!MotorIdentidad::es_pregunta_autorreferencial(""));
        assert!(!MotorIdentidad::es_pregunta_autorreferencial("   "));
    }

    #[test]
    fn test_detecta_presentacion() {
        assert!(MotorIdentidad::es_pregunta_autorreferencial("presentate"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("descríbete"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("cómo sos?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("quién es NEXUS?"));
        assert!(MotorIdentidad::es_pregunta_autorreferencial("¿quién es NEXUS?"));
    }

    // ─── RESPUESTA ─────────────────────────────────────────────────────────

    #[test]
    fn test_responde_nombre() {
        let mut motor = MotorIdentidad::new();
        let ocean = [0.5; 5];
        let respuesta = motor.responder_autorreferencia("cómo te llamas?", ocean);
        assert!(respuesta.contains("NEXUS"), "Respuesta debería contener NEXUS: {}", respuesta);
    }

    #[test]
    fn test_responde_proposito() {
        let mut motor = MotorIdentidad::new();
        let ocean = [0.5; 5];
        let respuesta = motor.responder_autorreferencia("cuál es tu propósito?", ocean);
        assert!(
            respuesta.contains("Arquitecto"),
            "Respuesta debería contener Arquitecto: {}",
            respuesta
        );
    }

    #[test]
    fn test_responde_presentacion() {
        let mut motor = MotorIdentidad::new();
        let ocean = [0.5; 5];
        let respuesta = motor.responder_autorreferencia("presentate", ocean);
        assert!(respuesta.contains("NEXUS"), "Presentación contiene NEXUS");
        assert!(respuesta.contains("Ingeniero"), "Presentación contiene rol");
    }

    #[test]
    fn test_responde_con_extraversion_alta() {
        let mut motor = MotorIdentidad::new();
        let ocean_alta_ext = [0.5, 0.5, 0.8, 0.5, 0.3];
        let respuesta = motor.responder_autorreferencia("cómo te llamas?", ocean_alta_ext);
        assert!(respuesta.contains("NEXUS"), "Nombre presente");
        assert!(
            respuesta.contains("placer") || respuesta.contains("Arquitecto"),
            "Extra alta = más cálido: {}",
            respuesta
        );
    }

    #[test]
    fn test_responde_con_neuroticismo_alto() {
        let mut motor = MotorIdentidad::new();
        let mut ocean_alto_neur = [0.5; 5];
        ocean_alto_neur[4] = 0.85;
        let respuesta = motor.responder_autorreferencia("cuál es tu propósito?", ocean_alto_neur);
        assert!(
            respuesta.len() <= 150,
            "Neuro alto = respuesta breve (len={}): {}",
            respuesta.len(),
            respuesta
        );
    }

    // ─── APRENDIZAJE ───────────────────────────────────────────────────────

    #[test]
    fn test_aprende_preferencia() {
        let mut motor = MotorIdentidad::new();
        motor.aprender_de_interaccion("prefiero respuestas cortas");
        assert!(
            motor.preferencias.contains(&"respuestas cortas".to_string()),
            "Debería haber aprendido 'respuestas cortas': {:?}",
            motor.preferencias
        );
    }

    #[test]
    fn test_aprende_me_gusta() {
        let mut motor = MotorIdentidad::new();
        motor.aprender_de_interaccion("me gusta el silencio cuando compilo");
        assert!(
            motor.preferencias.iter().any(|p| p.contains("silencio")),
            "Debería aprender 'me gusta X': {:?}",
            motor.preferencias
        );
    }

    #[test]
    fn test_no_aprende_si_no_hay_patron() {
        let mut motor = MotorIdentidad::new();
        let antes = motor.preferencias.len();
        motor.aprender_de_interaccion("esto es una conversación normal");
        assert_eq!(motor.preferencias.len(), antes, "No debe aprender si no hay patrón");
    }

    // ─── SERIALIZACIÓN ─────────────────────────────────────────────────────

    #[test]
    fn test_serializacion_roundtrip() {
        let motor = MotorIdentidad::new();
        let estado = motor.a_estado();
        let motor2 = MotorIdentidad::desde_estado(&estado);

        assert_eq!(motor.nombre, motor2.nombre, "nombre preservado");
        assert_eq!(motor.proposito, motor2.proposito, "proposito preservado");
        assert_eq!(motor.rol, motor2.rol, "rol preservado");
        assert_eq!(motor.preferencias, motor2.preferencias, "preferencias preservadas");
        assert_eq!(motor.historia, motor2.historia, "historia preservada");
        assert_eq!(motor.lemas, motor2.lemas, "lemas preservados");
    }

    #[test]
    fn test_serializacion_con_valores_modificados() {
        let mut motor = MotorIdentidad::new();
        motor.preferencias.push("neovim".to_string());
        motor.historia.push("Hoy aprendí una nueva técnica de optimización.".to_string());

        let estado = motor.a_estado();
        let motor2 = MotorIdentidad::desde_estado(&estado);

        assert!(motor2.preferencias.contains(&"neovim".to_string()));
        assert!(motor2.historia.iter().any(|h| h.contains("optimización")));
    }

    // ─── INYECCIÓN EN GRAFO ────────────────────────────────────────────────

    #[test]
    fn test_inyectar_identidad_crea_nodos() {
        let mut grafo = GrafoSinapsis::new();
        let motor = MotorIdentidad::new();

        motor.inyectar_identidad_en_grafo(&mut grafo);

        assert!(
            grafo.nodos.contains_key(&IDNodo::Concepto("nexus".to_string())),
            "Debería crear nodo 'nexus'"
        );
        assert!(
            grafo.nodos.contains_key(&IDNodo::Concepto("arquitecto".to_string())),
            "Debería crear nodo 'arquitecto'"
        );

        // Verificar energía del nodo nexus
        if let Some(nodo) = grafo.nodos.get(&IDNodo::Concepto("nexus".to_string())) {
            assert!(nodo.energia >= 0.7, "Energía de nexus >= 0.7: {}", nodo.energia);
        }
    }

    #[test]
    fn test_inyectar_identidad_no_duplica_si_ya_existe() {
        let mut grafo = GrafoSinapsis::new();
        grafo.nodos.insert(
            IDNodo::Concepto("nexus".to_string()),
            NodoSinaptico {
                id: IDNodo::Concepto("nexus".to_string()),
                energia: 0.3,
                palabra: "NEXUS".to_string(),
                refractario: 0.0,
                ultimo_disparo: 0,
                traza: 0.1,
                es_predicho: false,
                es_entrada_directa: false,
                ciclos_baja_energia: 0,
            },
        );

        let motor = MotorIdentidad::new();
        motor.inyectar_identidad_en_grafo(&mut grafo);

        // Verificar que no se duplicó
        let count = grafo.nodos.keys()
            .filter(|k| matches!(k, IDNodo::Concepto(s) if s == "nexus"))
            .count();
        assert_eq!(count, 1, "No debe duplicar nodos");

        // Verificar que la energía se actualizó
        if let Some(nodo) = grafo.nodos.get(&IDNodo::Concepto("nexus".to_string())) {
            assert!(nodo.energia >= 0.7, "Energía boosteada a >= 0.7: {}", nodo.energia);
        }
    }
}
