// ============================================================================
// 🔄 BUCLE REACTIVO — Ejecución práctica del Orquestador
// ============================================================================
// El Orquestador NO es teoría: es un bucle donde un LLM genera una intención,
// NEXUS la ejecuta como acción real (shell / archivo / decisión), observa el
// resultado y se lo devuelve al LLM para que reaccione. El ciclo se repite
// hasta lograr el objetivo, agotar iteraciones o recibir una señal de fin.
//
// Ciclo:
//   [LLM] ──intención──▶ [Ejecutor] ──acción real──▶ [Observador] ──resultado──▶
//                                                                              │
//   ◀───────────────────────── retroalimentación ─────────────────────────────┘
//
// Desacoplado de la fuente LLM (Ollama / vLLM / OpenRouter) vía el trait
// `InterfazLLM`. Integra el CircuitBreaker para no martillear un proveedor
// caído. Puro Rust, sin dependencias nuevas, cero unwrap() en runtime.
// ============================================================================

use crate::cerebro::orquestador_autonomo::CircuitBreaker;
use std::process::Command;
use std::time::Duration;

// ─── Interfaz del LLM ────────────────────────────────────────────────────────

/// Fuente de inteligencia externa. Implementar para Ollama, vLLM u OpenRouter.
/// `completar` recibe el historial de mensajes y devuelve el texto crudo.
pub trait InterfazLLM: Send + Sync {
    /// Nombre del proveedor (para métricas y logs).
    fn nombre(&self) -> &'static str;
    /// Envía el contexto/historial y devuelve la respuesta cruda del modelo.
    fn completar(&mut self, contexto: &str) -> Result<String, String>;
}

// ─── Intención que el LLM puede expresar ─────────────────────────────────────

/// Acción que NEXUS puede ejecutar en el mundo real.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Accion {
    /// Ejecuta un comando de shell y captura stdout/stderr + código de salida.
    Shell(String),
    /// Escribe contenido en una ruta (crea directorios si faltan).
    EscribirArchivo { ruta: String, contenido: String },
    /// Lee el contenido de un archivo.
    LeerArchivo(String),
    /// No ejecuta nada: señal para terminar el bucle con éxito.
    Terminar,
    /// Decisión simbólica de NEXUS (para métricas / lógica sin I/O).
    Decision(String),
}

/// Resultado de ejecutar una acción.
#[derive(Debug, Clone)]
pub struct ResultadoAccion {
    pub accion: Accion,
    pub salida: String,
    pub exito: bool,
    pub duracion_ms: u128,
}

/// Error de parseo de intención.
#[derive(Debug)]
pub struct IntencionError(pub String);

// ─── Parámetros del bucle ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConfigBucle {
    /// Iteraciones máximas del ciclo reactivo antes de abortar.
    pub max_iteraciones: u32,
    /// Umbral de fallos del CircuitBreaker antes de pausar el LLM.
    pub umbral_fallos: u32,
    /// Cooldown del CircuitBreaker tras abrirse.
    pub cooldown: Duration,
    /// Puerto/endpoint de la fuente LLM (solo informativo en logs).
    pub fuente_descripcion: String,
}

impl Default for ConfigBucle {
    fn default() -> Self {
        Self {
            max_iteraciones: 5,
            umbral_fallos: 3,
            cooldown: Duration::from_secs(30),
            fuente_descripcion: "local".to_string(),
        }
    }
}

// ─── El bucle reactivo ───────────────────────────────────────────────────────

pub struct BucleReactivo {
    llm: Box<dyn InterfazLLM>,
    circuito: CircuitBreaker,
    config: ConfigBucle,
    /// Historial del ciclo: contexto alimentado al LLM (intención + resultados).
    historial: Vec<String>,
}

impl BucleReactivo {
    pub fn nuevo(llm: Box<dyn InterfazLLM>, config: ConfigBucle) -> Self {
        Self {
            llm,
            circuito: CircuitBreaker::new(config.umbral_fallos, config.cooldown),
            config,
            historial: Vec::new(),
        }
    }

    /// Registra el objetivo inicial que el LLM debe perseguir.
    pub fn fijar_objetivo(&mut self, objetivo: &str) {
        self.historial.push(format!("OBJETIVO: {objetivo}"));
    }

    /// Devuelve el contexto completo que se envía al LLM en cada turno.
    fn contexto_actual(&self) -> String {
        self.historial.join("\n")
    }

    /// Añade un par intención/resultado al historial para que el LLM reaccione.
    fn registrar_observacion(&mut self, res: &ResultadoAccion) {
        let estado = if res.exito { "OK" } else { "FALLO" };
        self.historial.push(format!(
            "ACCION={:?} | {estado} | {}ms | SALIDA: {}",
            res.accion, res.duracion_ms, res.salida
        ));
        // Límite de historial para no inflar el contexto (compresión simple).
        if self.historial.len() > 40 {
            let resto = self.historial.split_off(self.historial.len() - 20);
            self.historial = resto;
        }
    }

    /// Ejecuta el ciclo reactivo completo contra un objetivo.
    /// Devuelve el resultado final o el primer error bloqueante.
    pub async fn ejecutar(&mut self, objetivo: &str) -> Result<String, String> {
        self.fijar_objetivo(objetivo);

        for iter in 0..self.config.max_iteraciones {
            // Circuit Breaker: no llamar al LLM si está abierto.
            if !self.circuito.puede_llamar(self.llm.nombre()) {
                return Err(format!(
                    "Circuito abierto para {:?}: proveedor LLM en cooldown",
                    self.llm.nombre()
                ));
            }

            // 1) Pedir intención al LLM. Obtener contexto primero para evitar
            // préstamos en conflicto con `&mut self.llm`.
            let contexto = self.contexto_actual();
            let respuesta = match self.llm.completar(&contexto) {
                Ok(r) => {
                    self.circuito.registrar_exito(self.llm.nombre());
                    r
                }
                Err(e) => {
                    self.circuito.registrar_fallo(self.llm.nombre());
                    return Err(format!("LLM {:?} falló: {e}", self.llm.nombre()));
                }
            };

            // 2) Parsear la intención en una acción ejecutable.
            let accion = match parsear_intencion(&respuesta) {
                Ok(a) => a,
                Err(e) => {
                    self.historial
                        .push(format!("INTENCION_INVALIDA: {}", e.0));
                    continue; // pedirle al LLM que se corrija
                }
            };

            // 3) Señal de término.
            if accion == Accion::Terminar {
                return Ok(self.historial.join("\n"));
            }

            // 4) Ejecutar la acción real y observar el resultado.
            let resultado = ejecutar_accion(accion.clone()).await;

            // 5) Retroalimentar: el LLM verá el resultado y reaccionará.
            self.registrar_observacion(&resultado);

            if iter + 1 >= self.config.max_iteraciones && !resultado.exito {
                return Err(format!(
                    "No se alcanzó el objetivo en {} iteraciones. Última salida: {}",
                    self.config.max_iteraciones, resultado.salida
                ));
            }
        }

        Ok(self.historial.join("\n"))
    }
}

// ─── Parseo de intención del LLM ─────────────────────────────────────────────

/// Interpreta la respuesta cruda del LLM y extrae una `Accion`.
/// Formato reconocido (línea por línea):
///   SHELL <comando>
///   ESCRIBIR <ruta> | <contenido>
///   LEER <ruta>
///   DECISION <texto>
///   FIN
fn parsear_intencion(respuesta: &str) -> Result<Accion, IntencionError> {
    for linea in respuesta.lines() {
        let t = linea.trim();
        if t.is_empty() {
            continue;
        }
        let (comando, resto) = match t.split_once(char::is_whitespace) {
            Some(p) => p,
            None => (t, ""),
        };

        match comando.to_uppercase().as_str() {
            "SHELL" if !resto.is_empty() => return Ok(Accion::Shell(resto.to_string())),
            "LEER" if !resto.is_empty() => return Ok(Accion::LeerArchivo(resto.to_string())),
            "ESCRIBIR" if !resto.is_empty() => {
                if let Some((ruta, contenido)) = resto.split_once('|') {
                    return Ok(Accion::EscribirArchivo {
                        ruta: ruta.trim().to_string(),
                        contenido: contenido.trim().to_string(),
                    });
                }
                return Ok(Accion::EscribirArchivo {
                    ruta: resto.to_string(),
                    contenido: String::new(),
                });
            }
            "DECISION" => return Ok(Accion::Decision(resto.to_string())),
            "FIN" | "TERMINAR" => return Ok(Accion::Terminar),
            _ => continue, // ignorar líneas de razonamiento / prosa
        }
    }
    Err(IntencionError(
        "No se encontró una intención ejecutable en la respuesta del LLM".into(),
    ))
}

// ─── Ejecutor de acciones ────────────────────────────────────────────────────

/// Ejecuta una acción real y captura su resultado.
pub async fn ejecutar_accion(accion: Accion) -> ResultadoAccion {
    let inicio = std::time::Instant::now();
    let (salida, exito) = match &accion {
        Accion::Shell(comando) => ejecutar_shell(comando),
        Accion::LeerArchivo(ruta) => leer_archivo(ruta),
        Accion::EscribirArchivo { ruta, contenido } => escribir_archivo(ruta, contenido),
        Accion::Decision(texto) => (texto.clone(), true),
        Accion::Terminar => ("Fin".to_string(), true),
    };
    let duracion_ms = inicio.elapsed().as_millis();
    ResultadoAccion {
        accion,
        salida,
        exito,
        duracion_ms,
    }
}

fn ejecutar_shell(comando: &str) -> (String, bool) {
    let salida = Command::new("sh").arg("-c").arg(comando).output();
    match salida {
        Ok(out) => {
            let mut texto = String::from_utf8_lossy(&out.stdout).to_string();
            if !out.stderr.is_empty() {
                if !texto.is_empty() {
                    texto.push('\n');
                }
                texto.push_str(&String::from_utf8_lossy(&out.stderr));
            }
            (texto, out.status.success())
        }
        Err(e) => (format!("Error ejecutando shell: {e}"), false),
    }
}

fn leer_archivo(ruta: &str) -> (String, bool) {
    match std::fs::read_to_string(ruta) {
        Ok(c) => (c, true),
        Err(e) => (format!("Error leyendo {ruta}: {e}"), false),
    }
}

fn escribir_archivo(ruta: &str, contenido: &str) -> (String, bool) {
    if let Some(parent) = std::path::Path::new(ruta).parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return (format!("Error creando directorios: {e}"), false);
            }
        }
    }
    match std::fs::write(ruta, contenido) {
        Ok(()) => (format!("Escrito {ruta} ({} bytes)", contenido.len()), true),
        Err(e) => (format!("Error escribiendo {ruta}: {e}"), false),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// InterfazLLM de prueba: devuelve intenciones en secuencia y reacciona
    /// al contenido del contexto (simula un LLM reactivo).
    struct LlmFake {
        pasos: Vec<&'static str>,
        paso: usize,
    }

    impl InterfazLLM for LlmFake {
        fn nombre(&self) -> &'static str {
            "fake"
        }
        fn completar(&mut self, _contexto: &str) -> Result<String, String> {
            let paso = self.paso;
            self.paso += 1;
            if paso < self.pasos.len() {
                Ok(self.pasos[paso].to_string())
            } else {
                Ok("FIN".to_string())
            }
        }
    }

    #[test]
    fn test_parsea_shell() {
        let a = parsear_intencion("razonando...\nSHELL echo hola\nmás texto").unwrap();
        assert_eq!(a, Accion::Shell("echo hola".to_string()));
    }

    #[test]
    fn test_parsea_escribir() {
        let a = parsear_intencion("ESCRIBIR /tmp/x.txt | contenido").unwrap();
        assert_eq!(
            a,
            Accion::EscribirArchivo {
                ruta: "/tmp/x.txt".to_string(),
                contenido: "contenido".to_string(),
            }
        );
    }

    #[test]
    fn test_parsea_fin() {
        assert_eq!(parsear_intencion("FIN").unwrap(), Accion::Terminar);
        assert_eq!(parsear_intencion("TERMINAR").unwrap(), Accion::Terminar);
    }

    #[test]
    fn test_parsea_rechaza_sin_accion() {
        assert!(parsear_intencion("solo texto sin comando").is_err());
    }

    #[tokio::test]
    async fn test_ejecuta_shell_real() {
        let res = ejecutar_accion(Accion::Shell("echo nexus".to_string())).await;
        assert!(res.exito);
        assert_eq!(res.salida.trim(), "nexus");
    }

    #[tokio::test]
    async fn test_escribir_y_leer_archivo() {
        let ruta = format!("/tmp/nexus_bucle_{}.txt", std::process::id());
        let esc = ejecutar_accion(Accion::EscribirArchivo {
            ruta: ruta.clone(),
            contenido: "datos".to_string(),
        })
        .await;
        assert!(esc.exito);

        let lee = ejecutar_accion(Accion::LeerArchivo(ruta)).await;
        assert!(lee.exito);
        assert_eq!(lee.salida.trim(), "datos");
    }

    #[test]
    fn test_bucle_completo_reacciona() {
        let mut bucle = BucleReactivo::nuevo(
            Box::new(LlmFake {
                pasos: vec!["SHELL echo paso1", "SHELL echo paso2"],
                paso: 0,
            }),
            ConfigBucle::default(),
        );
        // Usamos tokio runtime síncrono en el test.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let resultado = rt.block_on(bucle.ejecutar("probar reacción"));
        assert!(resultado.is_ok());
        let texto = resultado.unwrap();
        assert!(texto.contains("paso1"));
        assert!(texto.contains("paso2"));
    }

    #[test]
    fn test_circuito_protege_llm_caido() {
        struct LlmCaido;
        impl InterfazLLM for LlmCaido {
            fn nombre(&self) -> &'static str {
                "caido"
            }
            fn completar(&mut self, _c: &str) -> Result<String, String> {
                Err("timeout".into())
            }
        }
        let mut bucle = BucleReactivo::nuevo(
            Box::new(LlmCaido),
            ConfigBucle {
                umbral_fallos: 1,
                ..Default::default()
            },
        );
        let rt = tokio::runtime::Runtime::new().unwrap();
        let r1 = rt.block_on(bucle.ejecutar("x"));
        assert!(r1.is_err());
        // El segundo fallo debería quedar bloqueado por el circuito.
        let r2 = rt.block_on(bucle.ejecutar("x"));
        assert!(r2.is_err());
        assert!(r2.unwrap_err().contains("Circuito"));
    }
}
