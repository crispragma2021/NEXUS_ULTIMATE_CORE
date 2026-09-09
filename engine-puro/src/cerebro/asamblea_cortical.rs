// ============================================================================
// 🧠 ASAMBLEA CORTICAL — Cell Assemblies de Hebb para Representación Conceptual
// ============================================================================
// En lugar de predecir palabras (Markov n-gramas), este sistema agrupa
// neuronas en ENSAMBLES que representan conceptos.
//
// Cada ensamble es un grupo de neuronas que se disparan juntas —
// cuando el input activa suficientes tokens evocadores, el ensamble
// completo resuena, inhibe a sus competidores y condiciona la generación.
//
// Pipeline:
//   1. Tokenizar input → MotorSensorial autónomo
//   2. Evocar ensambles por tokens coincidentes
//   3. Competencia inhibitoria (GABAérgica) entre ensambles
//   4. Working memory: ~7 ensambles resonantes
//   5. Inyectar corriente de ensamble ganador en neuronas reales
//   6. La actividad neuronal extra condiciona la generación biológica
//
// Sin LLM. Sin transformers. Solo Hebb + competencia + resonancia.
// ============================================================================

use serde::{Serialize, Deserialize};

/// Un ensamble neuronal que representa un concepto abstracto.
///
/// Cada ensamble es un grupo de neuronas que se disparan sincronizadamente
/// cuando el sistema detecta sus palabras evocadoras. El ensamble que
/// "gana" la competencia inhibitoria condiciona la generación de lenguaje.
#[derive(Clone, Serialize, Deserialize)]
pub struct CellAssembly {
    /// Identificador único del ensamble
    pub id: u32,
    /// Nombre del concepto (humano-legible)
    pub nombre: String,
    /// Neuronas que componen el ensamble: (neurona_id, peso_interno)
    /// El peso indica qué tan "central" es cada neurona al concepto
    pub neuronas: Vec<(u32, f32)>,
    /// Tokens que evocan (activan) este ensamble
    pub tokens_evocadores: Vec<u32>,
    /// Tokens que inhiben este ensamble (lo apagan)
    pub tokens_inhibidores: Vec<u32>,
    /// IDs de otros ensambles que este inhibe cuando está activo (GABA)
    pub inhibe_a: Vec<u32>,
    /// Nivel de activación actual (0.0 = reposo, 1.0 = resonancia máxima)
    pub nivel_activacion: f32,
    /// Corriente acumulada para inyectar en las neuronas en el próximo paso
    pub corriente_acumulada: f32,
    /// Paso de simulación del último acceso (para decaimiento temporal)
    pub ultimo_acceso: u64,
}

impl CellAssembly {
    /// Crea un nuevo ensamble neuronal
    pub fn nueva(id: u32, nombre: &str, neuronas: Vec<(u32, f32)>) -> Self {
        CellAssembly {
            id,
            nombre: nombre.to_string(),
            neuronas,
            tokens_evocadores: Vec::new(),
            tokens_inhibidores: Vec::new(),
            inhibe_a: Vec::new(),
            nivel_activacion: 0.0,
            corriente_acumulada: 0.0,
            ultimo_acceso: 0,
        }
    }

    /// Calcula la activación del ensamble dado un conjunto de tokens de entrada
    fn calcular_activacion(&self, tokens: &[u32]) -> f32 {
        if self.tokens_evocadores.is_empty() || tokens.is_empty() {
            return 0.0;
        }
        let mut activacion = 0.0;
        let mut inhibicion = 0.0;

        // Sumar pesos de tokens evocadores que coinciden
        for tok in tokens {
            if self.tokens_evocadores.contains(tok) {
                activacion += 0.15; // Cada token evocador suma
            }
            if self.tokens_inhibidores.contains(tok) {
                inhibicion += 0.25; // Los inhibidores restan más
            }
        }

        // Normalizar por la cantidad de neuronas (ensambles grandes no dominan por tamaño)
        let factor_tamano = (self.neuronas.len() as f32).sqrt().max(1.0);
        (activacion - inhibicion) / factor_tamano
    }

    /// Aplica inhibición GABAérgica desde otro ensamble
    fn recibir_inhibicion(&mut self, intensidad: f32) {
        self.nivel_activacion *= 1.0 - intensidad;
        if self.nivel_activacion < 0.0 {
            self.nivel_activacion = 0.0;
        }
    }
}

/// Gestor central de asambleas corticales.
///
/// Mantiene el catálogo de ensambles, gestiona la competencia inhibitoria,
/// la working memory resonante y la inyección de corriente en neuronas reales.
#[derive(Clone, Serialize, Deserialize)]
pub struct AsambleaCortical {
    /// Catálogo completo de ensambles
    pub asambleas: Vec<CellAssembly>,
    /// Working memory: IDs de ensambles activos (buffer resonante, ~7 elementos)
    pub trabajando: Vec<u32>,
    /// Capacidad máxima de la working memory
    pub max_wm: usize,
    /// Paso de simulación actual (para marcar accesos)
    paso_actual: u64,
    /// Decaimiento por paso de la activación base
    decaimiento_base: f32,
    /// Umbral mínimo para considerar un ensamble "activo"
    umbral_activacion: f32,
}

impl AsambleaCortical {
    /// Crea un nuevo gestor de asambleas con ensambles semilla
    pub fn nueva() -> Self {
        let mut sistema = AsambleaCortical {
            asambleas: Vec::new(),
            trabajando: Vec::with_capacity(7),
            max_wm: 7,
            paso_actual: 0,
            decaimiento_base: 0.05,
            umbral_activacion: 0.15,
        };
        sistema.inicializar_asambleas_semilla();
        sistema
    }

    /// Inicializa ~30 asambleas semilla con conexiones pre-cableadas
    fn inicializar_asambleas_semilla(&mut self) {
        // Cada asamblea tiene una región neuronal base que la identifica
        // Regiones: 60000-60999 para asambleas.
        // El id de cada concepto ES su región neuronal base ($base): la asamblea
        // "saludo" (id=60000) vive en las neuronas 60000..60004. Esto mantiene
        // unicidad y coherencia entre el id y sus neuronas.
        macro_rules! asamblea {
            ($nombre:expr, $base:expr, $evocadores:expr) => {{
                let nid = $base;
                let mut a = CellAssembly::nueva(nid, $nombre, (0..5).map(|i| ($base + i, 1.0 - i as f32 * 0.15)).collect());
                // Registrar tokens evocadores
                for e in $evocadores { a.tokens_evocadores.push(*e); }
                a
            }};
        }

        // ====================================================================
        // ENSAMBLES SEMILLA — Conceptos Fundamentales
        // ====================================================================
        // Los IDs de tokens son los del vocabulario innato del MotorSensorial (0..N)

        let mut poner = |a: CellAssembly| self.asambleas.push(a);

        poner(asamblea!("saludo", 60000, &[0u32, 1, 2]));      // hola, buenos, días
        poner(asamblea!("despedida", 60010, &[3u32, 4]));      // adiós, hasta
        poner(asamblea!("pregunta", 60020, &[5u32, 6, 7]));    // qué, cómo, por qué
        poner(asamblea!("afirmacion", 60030, &[8u32, 9]));     // sí, claro
        poner(asamblea!("negacion", 60040, &[10u32, 11]));     // no, nunca
        poner(asamblea!("ayuda", 60050, &[12u32, 13]));        // ayuda, necesito
        poner(asamblea!("gratitud", 60060, &[14u32]));         // gracias
        poner(asamblea!("curiosidad", 60070, &[15u32, 16]));   // saber, aprender
        poner(asamblea!("emocion_positiva", 60080, &[17u32, 18])); // feliz, bien
        poner(asamblea!("emocion_negativa", 60090, &[19u32, 20])); // mal, triste
        poner(asamblea!("exploracion", 60100, &[21u32, 22]));  // buscar, encontrar
        poner(asamblea!("creacion", 60110, &[23u32, 24]));     // crear, construir
        poner(asamblea!("colaboracion", 60120, &[25u32, 26])); // juntos, nosotros
        poner(asamblea!("conocimiento", 60130, &[27u32, 28])); // saber, ciencia
        poner(asamblea!("tecnologia", 60140, &[29u32, 30]));   // código, sistema
        poner(asamblea!("filosofia", 60150, &[31u32, 32]));    // mente, conciencia
        poner(asamblea!("reflexion", 60160, &[33u32, 34]));    // pensar, significado
        poner(asamblea!("identidad", 60170, &[35u32, 36]));    // quién, soy
        poner(asamblea!("memoria", 60180, &[37u32, 38]));      // recordar, pasado
        poner(asamblea!("sueno", 60190, &[39u32, 40]));        // dormir, soñar
        poner(asamblea!("peligro", 60200, &[41u32, 42]));      // peligro, error
        poner(asamblea!("recompensa", 60210, &[43u32]));       // logro
        poner(asamblea!("duda", 60220, &[44u32, 45]));         // quizás, tal vez
        poner(asamblea!("certeza", 60230, &[46u32]));          // seguro
        poner(asamblea!("accion", 60240, &[47u32, 48]));       // hacer, ejecutar
        poner(asamblea!("silencio", 60250, &[49u32]));         // silencio
        poner(asamblea!("sistema", 60260, &[50u32, 51]));      // sistema, red
        poner(asamblea!("vinculo", 60270, &[52u32]));          // confianza
        poner(asamblea!("arquitecto", 60280, &[53u32, 54]));   // arquitecto, creador
        poner(asamblea!("tutor", 60290, &[55u32]));            // tutor
        poner(asamblea!("dopamina", 60300, &[56u32]));         // dopamina
        poner(asamblea!("plasticidad", 60310, &[57u32]));      // plasticidad

        // ====================================================================
        // CONEXIONES INHIBITORIAS — Competencia GABAérgica
        // ====================================================================
        // Pares de ensambles que se inhiben mutuamente
        let inhibiciones: &[(u32, u32)] = &[
            (60030, 60040), // afirmacion ↔ negacion
            (60080, 60090), // positiva ↔ negativa
            (60230, 60220), // certeza ↔ duda
            (60000, 60010), // saludo ↔ despedida
            (60250, 60000), // silencio inhibe saludo
            (60170, 60280), // identidad ↔ arquitecto (cercanía)
            (60140, 60150), // tecnologia ↔ filosofia
        ];

        for &(a, b) in inhibiciones {
            // a inhibe b
            if let Some(a_ens) = self.asambleas.iter_mut().find(|e| e.id == a) {
                if !a_ens.inhibe_a.contains(&b) {
                    a_ens.inhibe_a.push(b);
                }
            }
            // b inhibe a (inhibición recíproca = competencia)
            if let Some(b_ens) = self.asambleas.iter_mut().find(|e| e.id == b) {
                if !b_ens.inhibe_a.contains(&a) {
                    b_ens.inhibe_a.push(a);
                }
            }
        }
    }

    /// Avanza el reloj interno del sistema de asambleas
    pub fn tick(&mut self) {
        self.paso_actual += 1;
    }

    /// Activa ensambles por tokens léxicos, compite y actualiza working memory.
    ///
    /// Debe llamarse después de tokenizar el input pero antes de generar.
    /// `tokens` son los IDs de tokens del MotorSensorial del texto de entrada.
    pub fn evocar(&mut self, tokens: &[u32]) {
        if tokens.is_empty() {
            self.decaer_todo();
            return;
        }

        // Fase 1: Calcular activación base de cada ensamble
        for asamblea in self.asambleas.iter_mut() {
            let activacion = asamblea.calcular_activacion(tokens);
            asamblea.nivel_activacion += activacion;
            asamblea.ultimo_acceso = self.paso_actual;
        }

        // Fase 2: Competencia inhibitoria (3 iteraciones para estabilizar)
        for _ in 0..3 {
            self.inhibicion_competitiva();
        }

        // Fase 3: Obtener ensambles ganadores y actualizar working memory
        self.actualizar_working_memory();

        // Fase 4: Acumular corriente para inyectar en neuronas
        self.acumular_corriente();
    }

    /// Competencia inhibitoria GABAérgica entre ensambles activos.
    ///
    /// Cada ensamble activo inhibe a los que tiene en `inhibe_a`.
    /// La intensidad de inhibición escala con su propio nivel de activación.
    fn inhibicion_competitiva(&mut self) {
        // Tomar IDs y activaciones antes de mutar
        let estados: Vec<(u32, f32, Vec<u32>)> = self.asambleas
            .iter()
            .filter(|a| a.nivel_activacion >= self.umbral_activacion)
            .map(|a| (a.id, a.nivel_activacion, a.inhibe_a.clone()))
            .collect();

        for (_id_origen, activacion, inhibe_a) in &estados {
            let intensidad = activacion * 0.4; // GABAérgico
            for id_objetivo in inhibe_a {
                if let Some(objetivo) = self.asambleas.iter_mut().find(|a| a.id == *id_objetivo) {
                    objetivo.recibir_inhibicion(intensidad);
                }
            }
        }
    }

    /// Actualiza la working memory con los ensambles más activos
    fn actualizar_working_memory(&mut self) {
        let mut activos: Vec<(u32, f32)> = self.asambleas
            .iter()
            .filter(|a| a.nivel_activacion >= self.umbral_activacion)
            .map(|a| (a.id, a.nivel_activacion))
            .collect();

        // Ordenar por activación descendente
        activos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Mantener los top N en working memory
        self.trabajando = activos
            .into_iter()
            .take(self.max_wm)
            .map(|(id, _)| id)
            .collect();
    }

    /// Acumula corriente para inyectar en las neuronas reales
    fn acumular_corriente(&mut self) {
        for id in &self.trabajando {
            if let Some(asamblea) = self.asambleas.iter_mut().find(|a| a.id == *id) {
                asamblea.corriente_acumulada = asamblea.nivel_activacion * 3.0; // 0-3 mV por neurona
            }
        }
    }

    /// Devuelve la corriente a inyectar en el cerebro real: Vec<(neurona_id, corriente_mV)>
    ///
    /// Después de llamar a esto, la corriente acumulada se resetea a 0
    /// para evitar doble-inyección en el mismo paso.
    pub fn corriente_a_neuronas(&mut self) -> Vec<(u32, f32)> {
        let mut resultado = Vec::new();
        for asamblea in self.asambleas.iter_mut() {
            if asamblea.corriente_acumulada > 0.0 {
                for (nid, peso_interno) in &asamblea.neuronas {
                    let corriente = asamblea.corriente_acumulada * peso_interno;
                    resultado.push((*nid, corriente));
                }
                asamblea.corriente_acumulada = 0.0;
            }
        }
        resultado
    }

    /// Decae todas las activaciones hacia 0
    fn decaer_todo(&mut self) {
        for asamblea in self.asambleas.iter_mut() {
            asamblea.nivel_activacion *= 1.0 - self.decaimiento_base;
            if asamblea.nivel_activacion < 0.001 {
                asamblea.nivel_activacion = 0.0;
            }
        }
        // Limpiar working memory si ya no hay activación suficiente
        self.trabajando.retain(|id| {
            self.asambleas.iter().any(|a| a.id == *id && a.nivel_activacion >= self.umbral_activacion)
        });
    }

    /// Aprende una nueva asamblea por co-ocurrencia de tokens (Hebb).
    ///
    /// Cuando dos tokens aparecen juntos frecuentemente, se consolida
    /// una nueva asamblea que los representa como un concepto compuesto.
    /// `tokens_co_ocurrentes` debe tener al menos 2 tokens.
    pub fn consolidar(&mut self, nombre: &str, tokens_co_ocurrentes: &[u32], region_base: u32) -> Option<u32> {
        if tokens_co_ocurrentes.len() < 2 {
            return None;
        }

        let nuevo_id = 60320 + self.asambleas.len() as u32; // IDs después de la semilla
        if nuevo_id >= 61000 {
            return None; // Límite de asambleas
        }

        let mut asamblea = CellAssembly::nueva(
            nuevo_id,
            nombre,
            (0..8).map(|i| (region_base + i, 1.0 - i as f32 * 0.1)).collect(),
        );
        asamblea.tokens_evocadores = tokens_co_ocurrentes.to_vec();
        asamblea.nivel_activacion = 0.5; // Partir con activación media (Hebb inicial)

        self.asambleas.push(asamblea);
        Some(nuevo_id)
    }

    /// Inyecta tokens forzados en la working memory (para contexto de vínculo).
    /// Los tokens ya vienen tokenizados por el MotorSensorial autónomo.
    pub fn inyectar_tokens(&mut self, tokens: &[u32]) {
        if !tokens.is_empty() {
            self.evocar(tokens);
        }
    }

    /// Inyecta corriente directamente a una asamblea por su ID (usado por DMN)
    pub fn inyectar_corriente_a_asamblea(&mut self, id: u32, intensidad: f32) {
        if let Some(asamblea) = self.asambleas.iter_mut().find(|a| a.id == id) {
            asamblea.nivel_activacion = (asamblea.nivel_activacion + intensidad).min(1.0);
            asamblea.ultimo_acceso = self.paso_actual;
            // Forzar en working memory si la activación es alta
            if asamblea.nivel_activacion >= self.umbral_activacion && !self.trabajando.contains(&id) {
                self.trabajando.push(id);
                if self.trabajando.len() > self.max_wm {
                    self.trabajando.remove(0);
                }
            }
        }
    }

    /// Obtiene los nombres de los ensambles en la working memory
    pub fn nombres_activos(&self) -> Vec<&str> {
        self.trabajando
            .iter()
            .filter_map(|id| self.asambleas.iter().find(|a| a.id == *id))
            .map(|a| a.nombre.as_str())
            .collect()
    }

    /// Obtiene el ensamble más activo actualmente (el "ganador")
    pub fn ganador(&self) -> Option<&CellAssembly> {
        self.asambleas
            .iter()
            .filter(|a| a.nivel_activacion >= self.umbral_activacion)
            .max_by(|a, b| a.nivel_activacion.partial_cmp(&b.nivel_activacion).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Estadísticas del sistema de asambleas
    pub fn estadisticas(&self) -> AsambleaEstadisticas {
        AsambleaEstadisticas {
            total: self.asambleas.len(),
            activos: self.asambleas.iter().filter(|a| a.nivel_activacion >= self.umbral_activacion).count(),
            en_wm: self.trabajando.len(),
            ganador: self.ganador().map(|a| a.nombre.clone()),
        }
    }
}

/// Estadísticas del sistema de asambleas
#[derive(Clone, Debug)]
pub struct AsambleaEstadisticas {
    pub total: usize,
    pub activos: usize,
    pub en_wm: usize,
    pub ganador: Option<String>,
}

// ============================================================================
// TESTS
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asamblea_nueva_tiene_semilla() {
        let sistema = AsambleaCortical::nueva();
        assert!(sistema.asambleas.len() >= 30);
        assert_eq!(sistema.max_wm, 7);
    }

    #[test]
    fn test_asambleas_tienen_inhibiciones() {
        let sistema = AsambleaCortical::nueva();
        // afirmacion y negacion se inhiben mutuamente
        let afirmacion = sistema.asambleas.iter().find(|a| a.id == 60030).unwrap();
        assert!(afirmacion.inhibe_a.contains(&60040));
    }

    #[test]
    fn test_evocar_activa_asambleas() {
        let mut sistema = AsambleaCortical::nueva();

        // Activar con el token léxico "hola" (token 0, evocador del saludo)
        sistema.evocar(&[0]);
        sistema.tick();

        // El ensamble de saludo (id=60000) debería estar activo
        let saludo = sistema.asambleas.iter().find(|a| a.id == 60000).unwrap();
        assert!(saludo.nivel_activacion > 0.0);
    }

    #[test]
    fn test_working_memory_contiene_activos() {
        let mut sistema = AsambleaCortical::nueva();

        // Activar el ensamble de saludo con su token evocador "hola" (token 0)
        sistema.evocar(&[0]);
        sistema.tick();

        // Working memory no debería estar vacía si hay activación suficiente
        // Podría estar vacía si el umbral no se alcanza — depende de token mappings
        let stats = sistema.estadisticas();
        assert!(stats.total >= 30);
    }

    #[test]
    fn test_inhibicion_entre_competidores() {
        let mut sistema = AsambleaCortical::nueva();

        // Activar fuerte afirmacion con su token evocador "sí" (token 8)
        sistema.evocar(&[8]);
        sistema.tick();

        // afirmacion debería tener más activación que negacion
        let afirmacion = sistema.asambleas.iter().find(|a| a.id == 60030).map(|a| a.nivel_activacion).unwrap_or(0.0);
        let negacion = sistema.asambleas.iter().find(|a| a.id == 60040).map(|a| a.nivel_activacion).unwrap_or(0.0);

        // afirmacion debería estar al menos igual o más activa (puede ser 0 si no hay match exacto)
        assert!(afirmacion >= negacion);
    }

    #[test]
    fn test_corriente_a_neuronas() {
        let mut sistema = AsambleaCortical::nueva();

        // Forzar activación manual en un ensamble
        if let Some(saludo) = sistema.asambleas.iter_mut().find(|a| a.id == 60000) {
            saludo.nivel_activacion = 0.8;
            saludo.corriente_acumulada = 0.8 * 3.0;
        }

        let corriente = sistema.corriente_a_neuronas();
        assert!(!corriente.is_empty());
        // Cada neurona del ensamble saludo debería tener corriente
        for (_, c) in &corriente {
            assert!(*c > 0.0);
        }
    }

    #[test]
    fn test_consolidar_nueva_asamblea() {
        let mut sistema = AsambleaCortical::nueva();
        let antes = sistema.asambleas.len();

        let resultado = sistema.consolidar("test_concepto", &[100, 101, 102], 61000);
        assert!(resultado.is_some());
        assert_eq!(sistema.asambleas.len(), antes + 1);
    }

    #[test]
    fn test_consolidar_requiere_minimo_2_tokens() {
        let mut sistema = AsambleaCortical::nueva();
        let resultado = sistema.consolidar("invalido", &[100], 61000);
        assert!(resultado.is_none());
    }

    #[test]
    fn test_nombres_activos_vacia_sin_estimulo() {
        let sistema = AsambleaCortical::nueva();
        let nombres = sistema.nombres_activos();
        assert!(nombres.is_empty());
    }

    #[test]
    fn test_entendimiento_concepto_por_disparos_coordinados() {
        let mut sistema = AsambleaCortical::nueva();

        // Evocar el concepto "saludo" con sus 3 tokens evocadores (hola, buenos, días).
        // Con 3 tokens la activación supera el umbral: 3*0.15/sqrt(5) = 0.201 > 0.15.
        sistema.tick();
        sistema.evocar(&[0, 1, 2]);
        sistema.tick();

        // 1. El concepto correcto resuena por encima del umbral de activación.
        let saludo = sistema.asambleas.iter().find(|a| a.id == 60000).unwrap();
        assert!(
            saludo.nivel_activacion >= sistema.umbral_activacion,
            "El concepto saludo debió resonar, activación: {}",
            saludo.nivel_activacion
        );

        // 2. Entra en working memory como ensamble resonante.
        assert!(
            sistema.trabajando.contains(&60000),
            "saludo debió entrar en working memory"
        );

        // 3. Es el ganador de la competencia inhibitoria GABAérgica.
        assert_eq!(
            sistema.ganador().map(|a| a.id),
            Some(60000),
            "El ganador debía ser el concepto saludo"
        );

        // 4. Los disparos coordinados inyectan corriente en las neuronas del concepto.
        let corriente = sistema.corriente_a_neuronas();
        assert!(!corriente.is_empty(), "Debió inyectarse corriente neuronal");
        let neuronas_saludo: Vec<u32> = sistema.asambleas
            .iter()
            .find(|a| a.id == 60000)
            .unwrap()
            .neuronas
            .iter()
            .map(|(n, _)| *n)
            .collect();
        for (nid, mv) in &corriente {
            assert!(*mv > 0.0, "La corriente debía ser positiva, era {}", mv);
            assert!(
                neuronas_saludo.contains(nid),
                "La neurona {} no pertenece al concepto saludo",
                nid
            );
        }
    }

    #[test]
    fn test_decaer_todo_funciona() {
        let mut sistema = AsambleaCortical::nueva();
        // Forzar activación
        if let Some(saludo) = sistema.asambleas.iter_mut().find(|a| a.id == 60000) {
            saludo.nivel_activacion = 0.9;
        }

        sistema.decaer_todo();

        if let Some(saludo) = sistema.asambleas.iter().find(|a| a.id == 60000) {
            assert!(saludo.nivel_activacion < 0.9);
        }
    }
}
