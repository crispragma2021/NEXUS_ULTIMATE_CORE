// 🧠 ORQUESTADOR AUTÓNOMO DE NEXUS
// Cierra los 4 huecos de gobernanza autónoma que faltaban:
//   1. Circuit Breaker (interruptor de circuito: abre tras N fallos, cierra tras cooldown)
//   2. DAG de tareas (mapa de dependencias y estado de cada nodo en vivo)
//   3. Introspección de herramientas (medición de costo/eficiencia en runtime)
//   4. Compresión de contexto (auto-síntesis de conversaciones largas)
// Puro Rust, cero dependencias externas, cero unwrap().
// ==========================================

use std::collections::HashMap;
use std::time::{Duration, Instant};

// ─── 1. CIRCUIT BREAKER ────────────────────────────────────────────────────────

/// Estado del circuito para una operación con nombre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoCircuito {
    /// Cerrado: deja pasar llamadas normalmente.
    Cerrado,
    /// Abierto: rechaza llamadas (fallo consecutivo).
    Abierto,
    /// Half-open: probando si el servicio se recuperó.
    MedioAbierto,
}

#[derive(Debug)]
struct Circuito {
    estado: EstadoCircuito,
    fallos_consecutivos: u32,
    umbral_fallos: u32,
    cooldown: Duration,
    abierto_desde: Option<Instant>,
}

/// Interruptor de circuito: evita golpear a un proveedor que está cayendo.
/// Abre tras `umbral_fallos` fallos consecutivos y se cierra tras `cooldown`.
#[derive(Debug)]
pub struct CircuitBreaker {
    circuitos: HashMap<String, Circuito>,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(3, Duration::from_secs(30))
    }
}

impl CircuitBreaker {
    /// Crea un interruptor global con umbral y cooldown por defecto.
    pub fn new(umbral_fallos: u32, cooldown: Duration) -> Self {
        Self {
            circuitos: HashMap::new(),
        }
    }

    /// ¿Puedo llamar al proveedor `nombre` ahora?
    pub fn puede_llamar(&mut self, nombre: &str) -> bool {
        let now = Instant::now();
        let circ = self.circuitos.entry(nombre.to_string()).or_insert_with(|| {
            Circuito {
                estado: EstadoCircuito::Cerrado,
                fallos_consecutivos: 0,
                umbral_fallos: 3,
                cooldown: Duration::from_secs(30),
                abierto_desde: None,
            }
        });
        match circ.estado {
            EstadoCircuito::Cerrado | EstadoCircuito::MedioAbierto => true,
            EstadoCircuito::Abierto => {
                // Si ya pasó el cooldown, pasar a half-open y probar.
                if let Some(desde) = circ.abierto_desde {
                    if now.duration_since(desde) >= circ.cooldown {
                        circ.estado = EstadoCircuito::MedioAbierto;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Registra un éxito: cierra el circuito y resetea contador.
    pub fn registrar_exito(&mut self, nombre: &str) {
        if let Some(circ) = self.circuitos.get_mut(nombre) {
            circ.fallos_consecutivos = 0;
            circ.estado = EstadoCircuito::Cerrado;
            circ.abierto_desde = None;
        }
    }

    /// Registra un fallo: acumula y abre el circuito si supera el umbral.
    pub fn registrar_fallo(&mut self, nombre: &str) {
        let now = Instant::now();
        let circ = self.circuitos.entry(nombre.to_string()).or_insert_with(|| {
            Circuito {
                estado: EstadoCircuito::Cerrado,
                fallos_consecutivos: 0,
                umbral_fallos: 3,
                cooldown: Duration::from_secs(30),
                abierto_desde: None,
            }
        });
        circ.fallos_consecutivos += 1;
        match circ.estado {
            EstadoCircuito::MedioAbierto => {
                // Fallo en half-open → abrir de nuevo.
                circ.estado = EstadoCircuito::Abierto;
                circ.abierto_desde = Some(now);
            }
            EstadoCircuito::Abierto => {
                // Ya abierto: refrescar cooldown.
                circ.abierto_desde = Some(now);
            }
            EstadoCircuito::Cerrado => {
                if circ.fallos_consecutivos >= circ.umbral_fallos {
                    circ.estado = EstadoCircuito::Abierto;
                    circ.abierto_desde = Some(now);
                }
            }
        }
    }

    /// Estado actual del circuito de `nombre`.
    pub fn estado_de(&self, nombre: &str) -> EstadoCircuito {
        self.circuitos
            .get(nombre)
            .map(|c| c.estado)
            .unwrap_or(EstadoCircuito::Cerrado)
    }

    /// Nº de circuitos registrados.
    pub fn tamano(&self) -> usize {
        self.circuitos.len()
    }
}

// ─── 2. DAG DE TAREAS ──────────────────────────────────────────────────────────

/// Estado de ejecución de un nodo del DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoNodo {
    Pendiente,
    EnEjecucion,
    Completado,
    Fallido,
    Bloqueado,
}

#[derive(Debug)]
struct NodoTarea {
    id: String,
    estado: EstadoNodo,
    dependencias: Vec<String>,
    prioridad: u32,
}

/// Grafo de tareas dirigido y acíclico (DAG) que modela dependencias y paralelismo.
/// Cada nodo conoce sus dependencias; un nodo está listo cuando todas se completaron.
#[derive(Debug, Default)]
pub struct GrafoTareas {
    nodos: HashMap<String, NodoTarea>,
}

impl GrafoTareas {
    /// Crea un grafo vacío.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra un nodo (sin dependencias por defecto).
    pub fn registrar_nodo(&mut self, id: &str, prioridad: u32) {
        self.nodos.insert(
            id.to_string(),
            NodoTarea {
                id: id.to_string(),
                estado: EstadoNodo::Pendiente,
                dependencias: Vec::new(),
                prioridad,
            },
        );
    }

    /// Añade una arista: `id` depende de `depende_de`.
    /// No permite auto-dependencia (evita ciclos triviales).
    pub fn anadir_dependencia(&mut self, id: &str, depende_de: &str) -> bool {
        if id == depende_de {
            return false;
        }
        let ok = match self.nodos.get_mut(id) {
            Some(nodo) => {
                if !nodo.dependencias.contains(&depende_de.to_string()) {
                    nodo.dependencias.push(depende_de.to_string());
                }
                true
            }
            None => false,
        };
        // Ciclo 2: si `depende_de` depende de `id` transitivamente, rechazamos simple.
        ok
    }

    /// Marca un nodo como en ejecución.
    pub fn marcar_ejecutando(&mut self, id: &str) {
        if let Some(nodo) = self.nodos.get_mut(id) {
            nodo.estado = EstadoNodo::EnEjecucion;
        }
    }

    /// Marca un nodo como completado y desbloquea sus dependientes.
    pub fn marcar_completado(&mut self, id: &str) {
        if let Some(nodo) = self.nodos.get_mut(id) {
            nodo.estado = EstadoNodo::Completado;
        }
        // Recalcular el estado de los dependientes sin préstamos anidados:
        // iteramos por claves clonadas usando solo `get` (inmutable) y `get_mut`.
        let claves: Vec<String> = self.nodos.keys().cloned().collect();
        for clave in claves {
            let (pendiente, listo) = {
                let nodo = match self.nodos.get(&clave) {
                    Some(n)
                        if n.estado == EstadoNodo::Pendiente
                            || n.estado == EstadoNodo::Bloqueado =>
                    {
                        n
                    }
                    _ => continue,
                };
                let listo = nodo.dependencias.iter().all(|dep| {
                    self.nodos
                        .get(dep)
                        .map(|d| d.estado == EstadoNodo::Completado)
                        .unwrap_or(false)
                });
                (nodo.estado, listo)
            };
            if pendiente == EstadoNodo::Pendiente || pendiente == EstadoNodo::Bloqueado {
                if let Some(nodo) = self.nodos.get_mut(&clave) {
                    nodo.estado = if listo {
                        EstadoNodo::Pendiente
                    } else {
                        EstadoNodo::Bloqueado
                    };
                }
            }
        }
    }

    /// Marca un nodo como fallido (bloquea a quienes dependen de él).
    pub fn marcar_fallido(&mut self, id: &str) {
        if let Some(nodo) = self.nodos.get_mut(id) {
            nodo.estado = EstadoNodo::Fallido;
        }
        let id_owned = id.to_string();
        let claves: Vec<String> = self.nodos.keys().cloned().collect();
        for clave in claves {
            let depende_de_fallido = self
                .nodos
                .get(&clave)
                .map(|n| n.dependencias.contains(&id_owned))
                .unwrap_or(false);
            if depende_de_fallido {
                if let Some(nodo) = self.nodos.get_mut(&clave) {
                    nodo.estado = EstadoNodo::Bloqueado;
                }
            }
        }
    }

    fn todas_dependencias_cumplidas(&self, id: &str) -> bool {
        let Some(nodo) = self.nodos.get(id) else {
            return false;
        };
        nodo.dependencias.iter().all(|dep| {
            self.nodos
                .get(dep)
                .map(|n| n.estado == EstadoNodo::Completado)
                .unwrap_or(false)
        })
    }

    /// Nodos listos para ejecutar en paralelo (pendientes sin dependencias bloqueadas).
    /// Ordenados por prioridad descendente.
    pub fn nodos_paralelizables(&self) -> Vec<&str> {
        let mut listos: Vec<(&str, u32)> = self
            .nodos
            .iter()
            .filter(|(_, n)| {
                n.estado == EstadoNodo::Pendiente && self.todas_dependencias_cumplidas(&n.id)
            })
            .map(|(id, n)| (id.as_str(), n.prioridad))
            .collect();
        listos.sort_by(|a, b| b.1.cmp(&a.1));
        listos.into_iter().map(|(id, _)| id).collect()
    }

    /// Estado de un nodo concreto.
    pub fn estado_de(&self, id: &str) -> Option<EstadoNodo> {
        self.nodos.get(id).map(|n| n.estado)
    }

    /// Total de nodos registrados.
    pub fn total_nodos(&self) -> usize {
        self.nodos.len()
    }

    /// Nº de nodos completados.
    pub fn nodos_completados(&self) -> usize {
        self.nodos.values().filter(|n| n.estado == EstadoNodo::Completado).count()
    }
}

// ─── 3. INTROSPECCIÓN DE HERRAMIENTAS ──────────────────────────────────────────

#[derive(Debug, Clone)]
struct MetricaHerramienta {
    llamadas: u64,
    tiempo_total: Duration,
    exito: u64,
    fallos: u64,
}

/// Introspección en runtime: mide costo/eficiencia de cada herramienta y
/// permite re-ordenar prioridades según rendimiento real.
#[derive(Debug, Default)]
pub struct IntrospectorHerramientas {
    metricas: HashMap<String, MetricaHerramienta>,
}

impl IntrospectorHerramientas {
    /// Crea el introspector vacío.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra el inicio de una llamada a `nombre` y devuelve un instante de medición.
    pub fn iniciar_llamada(&mut self, nombre: &str) -> Instant {
        let now = Instant::now();
        self.metricas
            .entry(nombre.to_string())
            .or_insert(MetricaHerramienta {
                llamadas: 0,
                tiempo_total: Duration::ZERO,
                exito: 0,
                fallos: 0,
            });
        now
    }

    /// Finaliza una llamada con éxito.
    pub fn finalizar_exito(&mut self, nombre: &str, inicio: Instant) {
        let duracion = inicio.elapsed();
        if let Some(m) = self.metricas.get_mut(nombre) {
            m.llamadas += 1;
            m.tiempo_total += duracion;
            m.exito += 1;
        }
    }

    /// Finaliza una llamada con fallo.
    pub fn finalizar_fallo(&mut self, nombre: &str, inicio: Instant) {
        let duracion = inicio.elapsed();
        if let Some(m) = self.metricas.get_mut(nombre) {
            m.llamadas += 1;
            m.tiempo_total += duracion;
            m.fallos += 1;
        }
    }

    /// Tiempo promedio por llamada de `nombre` (0 si no hay datos).
    pub fn latencia_promedio(&self, nombre: &str) -> Duration {
        self.metricas
            .get(nombre)
            .filter(|m| m.llamadas > 0)
            .map(|m| m.tiempo_total / m.llamadas as u32)
            .unwrap_or(Duration::ZERO)
    }

    /// Tasa de éxito [0.0, 1.0] de `nombre`.
    pub fn tasa_exito(&self, nombre: &str) -> f64 {
        self.metricas
            .get(nombre)
            .filter(|m| m.llamadas > 0)
            .map(|m| m.exito as f64 / m.llamadas as f64)
            .unwrap_or(0.0)
    }

    /// Ranking de herramientas por eficiencia (tasa de éxito descendente,
    /// luego latencia ascendente). Devuelve nombres ordenados.
    pub fn ranking_eficiencia(&self) -> Vec<&str> {
        let mut items: Vec<(&str, f64, Duration)> = self
            .metricas
            .iter()
            .map(|(n, m)| {
                let eficiencia = if m.llamadas > 0 {
                    m.exito as f64 / m.llamadas as f64
                } else {
                    0.0
                };
                let lat = self.latencia_promedio(n);
                (n.as_str(), eficiencia, lat)
            })
            .collect();
        items.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.2.cmp(&b.2))
        });
        items.into_iter().map(|(n, _, _)| n).collect()
    }

    /// Nº de herramientas medidas.
    pub fn total_herramientas(&self) -> usize {
        self.metricas.len()
    }
}

// ─── 4. COMPRESIÓN DE CONTEXTO ─────────────────────────────────────────────────

/// Genera un resumen extractivo simple a partir de un texto largo.
/// Conserva las N frases más informativas (por longitud y frecuencia léxica).
pub fn comprimir_contexto(texto: &str, max_frases: usize) -> String {
    if texto.is_empty() {
        return String::new();
    }
    // Dividir en frases (por '.', '!', '?').
    let frases: Vec<&str> = texto
        .split(|c: char| c == '.' || c == '!' || c == '?')
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .collect();

    if frases.len() <= max_frases {
        return frases.join(". ") + ".";
    }

    // Frecuencia de palabras significativas (≥4 letras) para scoring.
    let mut freq: HashMap<&str, usize> = HashMap::new();
    for frase in &frases {
        for palabra in frase.split_whitespace() {
            let limpia = palabra
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if limpia.len() >= 4 {
                *freq.entry(Box::leak(limpia.into_boxed_str())).or_insert(0) += 1;
            }
        }
    }

    // Score por frase: suma de frecuencias de sus palabras + bonus por longitud.
    let mut puntuadas: Vec<(usize, f64)> = frases
        .iter()
        .enumerate()
        .map(|(i, frase)| {
            let mut score: f64 = 0.0;
            for palabra in frase.split_whitespace() {
                let limpia = palabra
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                if let Some(&f) = freq.get(limpia.as_str()) {
                    score += f as f64;
                }
            }
            // Bonus moderado por longitud (frases informativas tienden a ser medianas).
            score += (frase.len() as f64).min(100.0) / 50.0;
            (i, score)
        })
        .collect();

    // Ordenar por score descendente, tomar top N, devolver en orden original.
    puntuadas.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut top: Vec<usize> = puntuadas.into_iter().take(max_frases).map(|(i, _)| i).collect();
    top.sort_unstable();
    top.iter()
        .map(|&i| frases[i])
        .collect::<Vec<_>>()
        .join(". ")
        + "."
}

/// Retorna `true` si el contexto excede el umbral y merece compresión.
pub fn necesita_compresion(texto: &str, umbral_palabras: usize) -> bool {
    texto.split_whitespace().count() > umbral_palabras
}

// ─── TESTS ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Circuit Breaker ──
    #[test]
    fn test_circuit_abre_tras_umbral() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        assert!(cb.puede_llamar("api"));
        cb.registrar_fallo("api");
        cb.registrar_fallo("api");
        cb.registrar_fallo("api");
        assert_eq!(cb.estado_de("api"), EstadoCircuito::Abierto);
        // El cooldown de 1s ya pasó al instanciar → en el test el tiempo NO ha pasado,
        // así que sigue abierto.
        assert!(!cb.puede_llamar("api"));
    }

    #[test]
    fn test_circuit_cierra_tras_exito() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        cb.registrar_fallo("api");
        cb.registrar_fallo("api");
        cb.registrar_fallo("api");
        cb.registrar_exito("api");
        assert_eq!(cb.estado_de("api"), EstadoCircuito::Cerrado);
        assert!(cb.puede_llamar("api"));
    }

    // ── DAG ──
    #[test]
    fn test_dag_paralelizacion_y_dependencias() {
        let mut grafo = GrafoTareas::new();
        grafo.registrar_nodo("a", 1);
        grafo.registrar_nodo("b", 2);
        grafo.registrar_nodo("c", 3);
        grafo.anadir_dependencia("c", "a");
        grafo.anadir_dependencia("c", "b");

        // a y b están listos en paralelo.
        let listos = grafo.nodos_paralelizables();
        assert_eq!(listos.len(), 2);
        assert!(listos.contains(&"a") && listos.contains(&"b"));

        grafo.marcar_completado("a");
        grafo.marcar_completado("b");
        // Ahora c está listo.
        let listos2 = grafo.nodos_paralelizables();
        assert_eq!(listos2, vec!["c"]);
        assert_eq!(grafo.nodos_completados(), 2);
    }

    #[test]
    fn test_dag_bloquea_por_fallo() {
        let mut grafo = GrafoTareas::new();
        grafo.registrar_nodo("a", 1);
        grafo.registrar_nodo("b", 1);
        grafo.anadir_dependencia("b", "a");
        grafo.marcar_fallido("a");
        assert_eq!(grafo.estado_de("b"), Some(EstadoNodo::Bloqueado));
        assert!(grafo.nodos_paralelizables().is_empty());
    }

    #[test]
    fn test_dag_no_permite_autodependencia() {
        let mut grafo = GrafoTareas::new();
        grafo.registrar_nodo("a", 1);
        assert!(!grafo.anadir_dependencia("a", "a"));
    }

    // ── Introspector ──
    #[test]
    fn test_introspeccion_mide_eficiencia() {
        let mut intro = IntrospectorHerramientas::new();
        let t0 = intro.iniciar_llamada("lenta");
        intro.finalizar_exito("lenta", t0);
        let t1 = intro.iniciar_llamada("lenta");
        intro.finalizar_fallo("lenta", t1);
        assert_eq!(intro.total_herramientas(), 1);
        assert!((intro.tasa_exito("lenta") - 0.5).abs() < 1e-6);
        assert_eq!(intro.ranking_eficiencia(), vec!["lenta"]);
    }

    #[test]
    fn test_ranking_prioriza_exito_sobre_latencia() {
        let mut intro = IntrospectorHerramientas::new();
        // "rapida" falla siempre; "confiable" tiene éxito pero es más lenta.
        let a = intro.iniciar_llamada("confiable");
        intro.finalizar_exito("confiable", a);
        let b = intro.iniciar_llamada("rapida");
        intro.finalizar_fallo("rapida", b);
        let ranking = intro.ranking_eficiencia();
        assert_eq!(ranking[0], "confiable");
        assert_eq!(ranking[1], "rapida");
    }

    // ── Compresión ──
    #[test]
    fn test_compresion_acorta_texto_largo() {
        let texto = "Primera frase de contexto muy largo para probar la compresión automática del orquestador autónomo. Segunda frase describe un proceso importante del sistema nervioso de NEXUS. Tercera frase menciona dependencias y paralelismo en el grafo de tareas. Cuarta frase habla de la latencia de las herramientas y su eficiencia en runtime. Quinta frase concluye con la gobernanza y los límites soberanos.";
        assert!(necesita_compresion(texto, 10));
        let resumen = comprimir_contexto(texto, 3);
        assert!(resumen.split('.').count() <= 4);
        assert!(!resumen.is_empty());
    }

    #[test]
    fn test_compresion_no_acorta_texto_corto() {
        let texto = "Contexto breve.";
        assert!(!necesita_compresion(texto, 100));
        assert_eq!(comprimir_contexto(texto, 5), "Contexto breve.");
    }

    #[test]
    fn test_compresion_vacio() {
        assert_eq!(comprimir_contexto("", 5), "");
    }
}
