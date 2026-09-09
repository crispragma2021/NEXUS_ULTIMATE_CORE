// ============================================================================
// NEXUS-AGENT · nexo.rs — Orquestador agéntico (NexoAgente)
// ============================================================================
// Es el núcleo del agente: une el contrato de proveedores, el ejecutor de
// herramientas y el guardarraíl JSON en un bucle de razonamiento→acción→
// observación.
//
// Invariantes de diseño:
//   1. La instrucción maestra SIEMPRE vive en la posición [0] del historial.
//      Jamás se rota, se desplaza ni se modifica cuando la memoria se
//      consume o recorta. El método `reconstruir_historial` la vuelve a
//      inyectar al frente después de cualquier operación de compactación.
//   2. Cada iteración del bucle pide al modelo un paso JSON estructurado.
//      Si el parseo falla, se reinyecta el error como mensaje de corrección
//      (guardarraíl) y se repite la llamada — sin romper la invariante [0].
//   3. El agente termina cuando el modelo emite `respuesta_final`.
// ============================================================================

use crate::contrato::ContratoLlm;
use crate::delegacion::Delegador;
use crate::ejecutor::{EjecutorHermes, ResultadoHerramienta};
use crate::mcp_cliente::ClienteMcp;
use crate::memoria_estado::MemoriaEstado;
use crate::memoria_proyecto::MemoriaProyecto;
use crate::programador::Programador;
use crate::reglas_json::{InstrumentoLlamado, PasoEstructurado, ReglasJSON};
use crate::sesion::Transcripcion;
use crate::skills::BibliotecaSkills;
use crate::tareas::ListaTareas;
use crate::web::ClienteWeb;
use crate::{MensajeHistoria, RolMensaje};
use anyhow::{anyhow, Result};
use tracing::{debug, info, warn};

/// Configuración del agente.
#[derive(Debug, Clone)]
pub struct ConfigAgente {
    /// Máximo de iteraciones del bucle antes de forzar una respuesta.
    pub max_iteraciones: usize,
    /// Máximo de reintentos por JSON inválido antes de abortar el ciclo.
    pub max_reintentos_json: usize,
    /// Número máximo de mensajes de historial que se conservan al compactar
    /// (sin contar la instrucción maestra en [0]).
    pub ventana_historial: usize,
    /// Si es `true`, la compactación pide al LLM un resumen de los mensajes
    /// descartados (conservando decisiones, hechos y resultados) en lugar de
    /// descartarlos en silencio. Desactivado por defecto para no añadir
    /// llamadas extra al proveedor salvo que se pida explícitamente.
    pub compactar_con_resumen: bool,
}

impl Default for ConfigAgente {
    fn default() -> Self {
        Self {
            max_iteraciones: 10,
            max_reintentos_json: 3,
            ventana_historial: 40,
            compactar_con_resumen: false,
        }
    }
}

/// Resultado de un ciclo completo de razonamiento del agente.
#[derive(Debug, Clone)]
pub struct ResultadoCiclo {
    /// Respuesta final al usuario.
    pub respuesta: String,
    /// Número de instrumentos ejecutados.
    pub instrumentos_ejecutados: usize,
    /// Trazas de razonamiento del agente.
    pub saltos: Vec<SaltoTrazado>,
    /// Iteraciones que consumió el ciclo.
    pub iteraciones: usize,
}

/// Traza de un salto de razonamiento para diagnóstico.
#[derive(Debug, Clone)]
pub struct SaltoTrazado {
    pub razonamiento: String,
    pub instrumento: Option<String>,
    pub observacion: String,
}

// ----------------------------------------------------------------------------
// El orquestador agéntico
// ----------------------------------------------------------------------------

pub struct NexoAgente {
    /// Proveedor LLM activo (multi-proveedor: se puede cambiar en caliente).
    proveedor: Box<dyn ContratoLlm>,
    /// Ejecutor de herramientas con sandbox.
    ejecutor: EjecutorHermes,
    /// Configuración del bucle.
    config: ConfigAgente,
    /// Instrucción maestra inmutable.
    instruccion_maestra: String,
    /// Historial vivo. La posición [0] siempre es la instrucción maestra.
    historial: Vec<MensajeHistoria>,
    /// Cliente MCP stdio opcional: puente al cerebro NEXUS (claws_mcp).
    /// Habilita la herramienta `mcp_llamar` para invocar las herramientas
    /// del Orquestador directamente desde el bucle agéntico.
    mcp: Option<ClienteMcp>,
    /// Biblioteca de skills opcional (habilita `skill_listar` y `skill_ver`).
    skills: Option<BibliotecaSkills>,
    /// Transcripción opcional de la sesión (JSONL append-only).
    sesion: Option<Transcripcion>,
    /// Memoria de estado opcional (habilita la herramienta `recordar`).
    memoria_estado: Option<MemoriaEstado>,
    /// Cliente web opcional (habilita `web_buscar` y `web_extraer`).
    web: Option<ClienteWeb>,
    /// Lista de tareas opcional (habilita `todo_*`).
    tareas: Option<ListaTareas>,
    /// Programador cron opcional (habilita `programar`, `tareas_listar`,
    /// `tareas_cancelar`).
    programador: Option<Programador>,
    /// Delegador opcional (habilita `delegar`; ausente en subagentes para
    /// limitar la profundidad a 1).
    delegador: Option<Delegador>,
}

impl NexoAgente {
    pub fn nuevo(
        proveedor: Box<dyn ContratoLlm>,
        ejecutor: EjecutorHermes,
        instruccion_maestra: &str,
    ) -> Self {
        let config = ConfigAgente::default();
        Self {
            proveedor,
            ejecutor,
            config,
            instruccion_maestra: instruccion_maestra.to_string(),
            historial: vec![MensajeHistoria::sistema(instruccion_maestra)],
            mcp: None,
            skills: None,
            sesion: None,
            memoria_estado: None,
            web: None,
            tareas: None,
            programador: None,
            delegador: None,
        }
    }

    pub fn con_config(
        proveedor: Box<dyn ContratoLlm>,
        ejecutor: EjecutorHermes,
        instruccion_maestra: &str,
        config: ConfigAgente,
    ) -> Self {
        Self {
            proveedor,
            ejecutor,
            config,
            instruccion_maestra: instruccion_maestra.to_string(),
            historial: vec![MensajeHistoria::sistema(instruccion_maestra)],
            mcp: None,
            skills: None,
            sesion: None,
            memoria_estado: None,
            web: None,
            tareas: None,
            programador: None,
            delegador: None,
        }
    }

    /// Expone el historial (para inspección y tests).
    pub fn ver_historial(&self) -> &[MensajeHistoria] {
        &self.historial
    }

    /// Verifica la invariante: posición [0] es la instrucción maestra.
    pub fn invariante_maestra_ok(&self) -> bool {
        matches!(self.historial.first(), Some(m) if m.rol == RolMensaje::Sistema
            && m.contenido == self.instruccion_maestra)
    }

    /// Cambia el proveedor LLM en caliente (multi-proveedor sin reiniciar).
    pub fn cambiar_proveedor(&mut self, proveedor: Box<dyn ContratoLlm>) {
        self.proveedor = proveedor;
        info!(proveedor = self.proveedor.nombre(), "Proveedor LLM cambiado en caliente");
    }

    /// Reinicia la sesión: conserva la instrucción maestra, descarta el resto.
    pub fn reiniciar_sesion(&mut self) {
        self.historial = vec![MensajeHistoria::sistema(self.instruccion_maestra.clone())];
        debug!("Sesión reiniciada; instrucción maestra conservada en [0]");
    }

    /// Conecta un cliente MCP stdio para la herramienta `mcp_llamar`.
    ///
    /// El cliente lanza el servidor MCP (p. ej. `claws_mcp`) como subproceso
    /// por llamada, escribe la petición JSON-RPC por stdin y lee la respuesta
    /// por stdout. Sin esta conexión, `mcp_llamar` devuelve una observación
    /// controlada en lugar de fallar el ciclo.
    pub fn con_mcp(mut self, mcp: ClienteMcp) -> Self {
        self.mcp = Some(mcp);
        debug!("Cliente MCP conectado al agente");
        self
    }

    /// Conecta la biblioteca de skills. El catálogo (nombre → descripción)
    /// se inyecta en la instrucción maestra para que el modelo sepa qué
    /// skills existen y cuándo cargarlos con `skill_ver`.
    pub fn con_skills(mut self, skills: BibliotecaSkills) -> Self {
        if skills.cantidad() > 0 {
            let catalogo = format!("\n{}\n", skills.listar());
            self.instruccion_maestra = format!("{}{}", self.instruccion_maestra, catalogo);
            self.historial[0] = MensajeHistoria::sistema(self.instruccion_maestra.clone());
            info!(skills = skills.cantidad(), "Biblioteca de skills inyectada en [0]");
        }
        self.skills = Some(skills);
        self
    }

    /// Conecta la transcripción de sesión: cada mensaje del bucle se anexa
    /// al JSONL. La reanudación se hace aparte con `reanudar_con`.
    pub fn con_sesion(mut self, sesion: Transcripcion) -> Self {
        self.sesion = Some(sesion);
        debug!("Transcripción de sesión conectada");
        self
    }

    /// Conecta la memoria de estado y la inyecta en la instrucción maestra.
    pub fn con_memoria_estado(mut self, memoria: MemoriaEstado) -> Self {
        if memoria.tiene_memoria() {
            let fusion = memoria.fusionar();
            self.instruccion_maestra = format!("{}\n{}", self.instruccion_maestra, fusion);
            self.historial[0] = MensajeHistoria::sistema(self.instruccion_maestra.clone());
            info!(
                hechos = memoria.entradas().len(),
                "Memoria de estado inyectada en [0]"
            );
        }
        self.memoria_estado = Some(memoria);
        self
    }

    /// Reanuda una sesión previa: inserta los mensajes cargados de la
    /// transcripción DETRÁS de la instrucción maestra, respetando la
    /// invariante [0] y sin duplicar la maestra.
    pub fn reanudar_con(&mut self, mensajes: Vec<MensajeHistoria>) {
        if mensajes.is_empty() {
            return;
        }
        let reanudados = mensajes.len();
        let maestra = self.historial[0].clone();
        let mut nuevo = Vec::with_capacity(mensajes.len() + 1);
        nuevo.push(maestra);
        for m in mensajes {
            if m.rol == RolMensaje::Sistema && m.contenido == self.instruccion_maestra {
                continue;
            }
            nuevo.push(m);
        }
        self.historial = nuevo;
        info!(reanudados, "Sesión reanudada desde transcripción");
    }

    /// Conecta el cliente web (habilita `web_buscar` y `web_extraer`).
    pub fn con_web(mut self, web: ClienteWeb) -> Self {
        self.web = Some(web);
        debug!("Cliente web conectado");
        self
    }

    /// Conecta la lista de tareas persistente (habilita `todo_*`).
    pub fn con_tareas(mut self, tareas: ListaTareas) -> Self {
        self.tareas = Some(tareas);
        debug!("Lista de tareas conectada");
        self
    }

    /// Conecta el programador cron (habilita `programar` y `tareas_*`).
    pub fn con_programador(mut self, programador: Programador) -> Self {
        self.programador = Some(programador);
        debug!("Programador cron conectado");
        self
    }

    /// Conecta el delegador de subagentes (habilita `delegar`).
    ///
    /// NO se conecta en subagentes: limita la profundidad de delegación a 1
    /// (un subagente no puede delegar a su vez).
    pub fn con_delegador(mut self, delegador: Delegador) -> Self {
        self.delegador = Some(delegador);
        debug!("Delegador de subagentes conectado");
        self
    }

    /// Inyecta la memoria de proyecto (AGENTE.md jerárquico) como contexto
    /// adicional al frente de la instrucción maestra.
    ///
    /// La fusión de las piezas (global → proyecto → carpeta) se anexa a la
    /// instrucción maestra y se refleja en la posición [0] del historial, de
    /// modo que la invariante `invariante_maestra_ok` se mantiene intacta.
    /// Si la memoria está vacía, el agente arranca igual sin contexto extra.
    pub fn con_memoria_proyecto(mut self, memoria: MemoriaProyecto) -> Self {
        if memoria.tiene_memoria() {
            let fusion = memoria.fusionar();
            self.instruccion_maestra = format!("{}\n{}", self.instruccion_maestra, fusion);
            self.historial[0] = MensajeHistoria::sistema(self.instruccion_maestra.clone());
            info!(
                piezas = memoria.piezas().len(),
                "Memoria de proyecto inyectada en la instrucción maestra [0]"
            );
        } else {
            debug!("Memoria de proyecto vacía; no se inyecta");
        }
        self
    }

    /// Inyecta un mensaje de usuario y ejecuta el ciclo completo de razonamiento.
    pub async fn ejecutar(&mut self, mensaje_usuario: &str) -> Result<ResultadoCiclo> {
        self.agregar_mensaje(RolMensaje::Usuario, mensaje_usuario).await;

        let mut saltos: Vec<SaltoTrazado> = Vec::new();
        let mut instrumentos_ejecutados = 0usize;

        for iteracion in 0..self.config.max_iteraciones {
            // 1. Pedir el paso al modelo (con guardarraíl de JSON)
            let paso = self.pedir_paso_validado().await?;

            // 2. ¿El agente terminó?
            if let Some(respuesta_final) = paso.respuesta_final {
                self.agregar_mensaje(RolMensaje::Asistente, &respuesta_final).await;
                info!(
                    iteracion,
                    instrumentos = instrumentos_ejecutados,
                    "Agente finalizó con respuesta"
                );
                return Ok(ResultadoCiclo {
                    respuesta: respuesta_final,
                    instrumentos_ejecutados,
                    saltos,
                    iteraciones: iteracion + 1,
                });
            }

            // 3. Ejecutar el instrumento solicitado
            let instrumento = paso
                .instrumento
                .ok_or_else(|| anyhow!("El paso no tenía instrumento ni respuesta final"))?;

            let observacion = self.ejecutar_instrumento(&instrumento).await?;
            instrumentos_ejecutados += 1;

            saltos.push(SaltoTrazado {
                razonamiento: paso.razonamiento,
                instrumento: Some(instrumento.nombre),
                observacion: observacion.clone(),
            });

            // 4. Devolver la observación al modelo (como mensaje de instrumento)
            self.agregar_mensaje(RolMensaje::Instrumento, &observacion).await;
        }

        // Se agotaron las iteraciones: respuesta forzada
        let aviso = "He agotado el número máximo de pasos de razonamiento sin \
                     llegar a una respuesta final. Resumo lo obtenido.";
        warn!(max = self.config.max_iteraciones, "Ciclo agotado; respuesta forzada");
        self.agregar_mensaje(RolMensaje::Asistente, aviso).await;
        Ok(ResultadoCiclo {
            respuesta: aviso.to_string(),
            instrumentos_ejecutados,
            saltos,
            iteraciones: self.config.max_iteraciones,
        })
    }

    // ------------------------------------------------------------------------
    // Internos del bucle
    // ------------------------------------------------------------------------

    /// Pide un paso estructurado, aplicando el guardarraíl JSON con reintentos.
    async fn pedir_paso_validado(&mut self) -> Result<PasoEstructurado> {
        let mut ultimo_error: Option<String> = None;

        for intento in 0..self.config.max_reintentos_json {
            // Historial con instrucción maestra garantizada en [0]
            let historial_para_envio = self.construir_historial_para_envio();
            let respuesta = self.proveedor.conversar(&historial_para_envio).await?;

            // Parseo tolerante
            let paso = match ReglasJSON::parsear(&respuesta.texto) {
                Ok(p) => p,
                Err(e) => {
                    ultimo_error = Some(e.clone());
                    if intento + 1 >= self.config.max_reintentos_json {
                        return Err(anyhow!(
                            "El modelo no produjo JSON válido tras {} intentos: {e}",
                            self.config.max_reintentos_json
                        ));
                    }
                    // Reinyectar la corrección como mensaje de usuario (detrás de [0])
                    self.historial.push(MensajeHistoria::usuario(
                        ReglasJSON::mensaje_correccion(&e),
                    ));
                    continue;
                }
            };

            // Validación estructural
            if let Err(e) = ReglasJSON::validar(&paso) {
                ultimo_error = Some(e.clone());
                if intento + 1 >= self.config.max_reintentos_json {
                    return Err(anyhow!(
                        "El paso no pasó la validación estructural tras {} intentos: {e}",
                        self.config.max_reintentos_json
                    ));
                }
                self.historial.push(MensajeHistoria::usuario(
                    ReglasJSON::mensaje_correccion(&e),
                ));
                continue;
            }

            return Ok(paso);
        }

        Err(anyhow!(
            "Bucle de guardarraíl terminó sin paso: {:?}",
            ultimo_error
        ))
    }

    /// Construye el historial a enviar asegurando la invariante [0].
    ///
    /// Esta es la garantía central: después de cualquier compactación o
    /// rotación de memoria, la instrucción maestra se re-inserta en [0].
    fn construir_historial_para_envio(&self) -> Vec<MensajeHistoria> {
        let mut out = Vec::with_capacity(self.historial.len());
        // 1. Instrucción maestra SIEMPRE primero
        out.push(MensajeHistoria::sistema(self.instruccion_maestra.clone()));
        // 2. Resto del historial sin duplicar la instrucción maestra
        for m in &self.historial {
            if m.rol == RolMensaje::Sistema && m.contenido == self.instruccion_maestra {
                continue;
            }
            out.push(m.clone());
        }
        out
    }

    /// Agrega un mensaje al historial y compacta si excede la ventana.
    ///
    /// Si hay transcripción conectada, cada mensaje se anexa al JSONL de la
    /// sesión (append-only). Un fallo de escritura NO rompe el ciclo: se
    /// registra con un aviso y el bucle continúa (la transcripción es un
    /// registro, no una dependencia crítica).
    async fn agregar_mensaje(&mut self, rol: RolMensaje, contenido: &str) {
        // Nombre del rol ANTES de moverlo al push (para la transcripción)
        let rol_nombre = match rol {
            RolMensaje::Sistema => "sistema",
            RolMensaje::Usuario => "usuario",
            RolMensaje::Asistente => "asistente",
            RolMensaje::Instrumento => "instrumento",
        };
        self.historial.push(MensajeHistoria {
            rol,
            contenido: contenido.to_string(),
        });
        if let Some(sesion) = &self.sesion {
            if let Err(e) = sesion.registrar(rol_nombre, contenido) {
                warn!(error = %e, "No se pudo transcribir el mensaje (se continúa)");
            }
        }
        self.compactar_si_necesario().await;
    }

    /// Compactación: recorta el historial vivo a la ventana, SIN tocar [0].
    ///
    /// Si `compactar_con_resumen` está activado, la parte descartada se envía
    /// al proveedor LLM para obtener un resumen que se conserva como mensaje
    /// de sistema (etiquetado), de modo que la conversación anterior no se
    /// pierda por completo. Si el resumen falla, se cae en la compactación
    /// simple sin romper el ciclo.
    async fn compactar_si_necesario(&mut self) {
        if self.historial.len() <= self.config.ventana_historial + 1 {
            return;
        }
        let inicio = self.historial.len() - self.config.ventana_historial;
        let maestra = self.historial[0].clone();
        // Clonamos las piezas para no mantener préstamos sobre self a través
        // del .await del resumen (el historial se muta al final).
        let descartados: Vec<MensajeHistoria> = self.historial[1..inicio].to_vec();
        let conservados: Vec<MensajeHistoria> = self.historial[inicio..].to_vec();

        let mut nuevo = Vec::with_capacity(self.config.ventana_historial + 2);
        // Conservamos [0] (instrucción maestra) SIEMPRE al frente.
        nuevo.push(maestra);

        if self.config.compactar_con_resumen && !descartados.is_empty() {
            match self.resumir_conversacion(&descartados).await {
                Ok(resumen) => {
                    nuevo.push(MensajeHistoria::sistema(format!(
                        "📌 RESUMEN DE CONTEXTO ANTERIOR:\n{resumen}"
                    )));
                    debug!(
                        mensajes_resumidos = descartados.len(),
                        "Compactación con resumen LLM insertada tras [0]"
                    );
                }
                Err(e) => {
                    warn!(error = %e, "No se pudo resumir el contexto; compactación simple");
                }
            }
        }

        nuevo.extend(conservados);
        self.historial = nuevo;
        debug!(
            ventana = self.config.ventana_historial,
            "Historial compactado; instrucción maestra intacta en [0]"
        );
    }

    /// Pide al LLM un resumen compacto de los mensajes que se van a descartar.
    ///
    /// Conserva decisiones, hechos, comandos ejecutados, resultados, errores y
    /// aprendizajes, de modo que la compactación no borre el contexto de forma
    /// silenciosa. El resumen se inserta como mensaje de sistema etiquetado.
    async fn resumir_conversacion(&self, mensajes: &[MensajeHistoria]) -> Result<String> {
        let mut historial_resumen = Vec::with_capacity(mensajes.len() + 1);
        historial_resumen.push(MensajeHistoria::sistema(
            "Eres el archivista de contexto de NEXUS-Agent. Resume la conversación \
             siguiente de forma compacta, conservando decisiones, hechos, comandos \
             ejecutados y sus resultados, errores y aprendizajes. Máximo 400 palabras, \
             en el idioma de la conversación.",
        ));
        historial_resumen.extend_from_slice(mensajes);
        let respuesta = self.proveedor.conversar(&historial_resumen).await?;
        Ok(respuesta.texto.trim().to_string())
    }

    /// Ejecuta un instrumento y devuelve la observación para el modelo.
    ///
    /// Recibe `&mut self` porque `recordar` persiste en la memoria de estado
    /// del agente (única herramienta mutante).
    async fn ejecutar_instrumento(&mut self, instrumento: &InstrumentoLlamado) -> Result<String> {
        let resultado = match instrumento.nombre.as_str() {
            "bash" => {
                let comando = instrumento
                    .argumento("comando")
                    .ok_or_else(|| anyhow!("Instrumento 'bash' requiere argumento 'comando'"))?;
                self.ejecutor.ejecutar_bash(&comando).await?
            }
            "leer_archivo" => {
                let ruta = instrumento
                    .argumento("ruta")
                    .ok_or_else(|| anyhow!("Instrumento 'leer_archivo' requiere 'ruta'"))?;
                self.ejecutor.leer_archivo(&ruta).await?
            }
            "escribir_archivo" => {
                let ruta = instrumento
                    .argumento("ruta")
                    .ok_or_else(|| anyhow!("Instrumento 'escribir_archivo' requiere 'ruta'"))?;
                let contenido = instrumento
                    .argumento("contenido")
                    .ok_or_else(|| anyhow!("Instrumento 'escribir_archivo' requiere 'contenido'"))?;
                self.ejecutor.escribir_archivo(&ruta, &contenido).await?
            }
            "mcp_llamar" => {
                // Puente al cerebro NEXUS: invoca una herramienta del servidor
                // MCP (claws_mcp) pasando la petición JSON-RPC por subproceso.
                let herramienta = instrumento
                    .argumento("herramienta")
                    .ok_or_else(|| anyhow!("Instrumento 'mcp_llamar' requiere 'herramienta'"))?;
                let argumentos_raw = instrumento
                    .argumento("argumentos")
                    .ok_or_else(|| anyhow!("Instrumento 'mcp_llamar' requiere 'argumentos' (JSON)"))?;
                let argumentos: serde_json::Value = serde_json::from_str(&argumentos_raw)
                    .map_err(|e| anyhow!("'argumentos' no es JSON válido: {e}"))?;
                match self.mcp.as_ref() {
                    None => ResultadoHerramienta {
                        exitoso: false,
                        salida: "El agente no tiene cliente MCP configurado (usa NexoAgente::con_mcp)"
                            .to_string(),
                    },
                    Some(cliente) => match cliente.llamar(&herramienta, argumentos).await {
                        Ok(resultado) => {
                            if ClienteMcp::es_error(&resultado) {
                                ResultadoHerramienta {
                                    exitoso: false,
                                    salida: format!(
                                        "MCP ({herramienta}): {}",
                                        ClienteMcp::texto(&resultado)
                                    ),
                                }
                            } else {
                                ResultadoHerramienta {
                                    exitoso: true,
                                    salida: format!(
                                        "[MCP {herramienta}] {}",
                                        ClienteMcp::texto(&resultado)
                                    ),
                                }
                            }
                        }
                        Err(e) => ResultadoHerramienta {
                            exitoso: false,
                            salida: format!("Error MCP: {e}"),
                        },
                    },
                }
            }
            "skill_listar" => match self.skills.as_ref() {
                Some(skills) => ResultadoHerramienta::exito(skills.listar()),
                None => ResultadoHerramienta::fallo(
                    "El agente no tiene biblioteca de skills (usa NexoAgente::con_skills)",
                ),
            },
            "skill_ver" => {
                let nombre = instrumento
                    .argumento("nombre")
                    .ok_or_else(|| anyhow!("Instrumento 'skill_ver' requiere 'nombre'"))?;
                match self.skills.as_ref().and_then(|s| s.ver(&nombre)) {
                    Some(contenido) => ResultadoHerramienta::exito(contenido),
                    None => ResultadoHerramienta::fallo(format!(
                        "Skill '{nombre}' no existe. Lista los disponibles con skill_listar"
                    )),
                }
            }
            "listar_archivos" => {
                let ruta = instrumento
                    .argumento("ruta")
                    .unwrap_or_else(|| ".".to_string());
                let max = instrumento
                    .argumentos
                    .get("max_resultados")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as usize;
                self.ejecutor.listar_archivos(&ruta, max).await?
            }
            "buscar_archivos" => {
                let patron = instrumento
                    .argumento("patron")
                    .ok_or_else(|| anyhow!("Instrumento 'buscar_archivos' requiere 'patron'"))?;
                let ruta = instrumento
                    .argumento("ruta")
                    .unwrap_or_else(|| ".".to_string());
                let glob = instrumento.argumento("glob");
                let max = instrumento
                    .argumentos
                    .get("max_resultados")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(50) as usize;
                self.ejecutor.buscar_archivos(&patron, &ruta, glob.as_deref(), max).await?
            }
            "recordar" => {
                let hecho = instrumento
                    .argumento("hecho")
                    .ok_or_else(|| anyhow!("Instrumento 'recordar' requiere 'hecho'"))?;
                match self.memoria_estado.as_mut() {
                    Some(memoria) => match memoria.recordar(&hecho) {
                        Ok(()) => ResultadoHerramienta::exito(format!(
                            "Hecho recordado en la memoria de estado ({} hechos)",
                            memoria.entradas().len()
                        )),
                        Err(e) => ResultadoHerramienta::fallo(format!(
                            "No se pudo recordar: {e}"
                        )),
                    },
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene memoria de estado (usa NexoAgente::con_memoria_estado)",
                    ),
                }
            }
            "todo_agregar" => {
                let descripcion = instrumento
                    .argumento("descripcion")
                    .ok_or_else(|| anyhow!("Instrumento 'todo_agregar' requiere 'descripcion'"))?;
                match self.tareas.as_mut() {
                    Some(lista) => match lista.agregar(&descripcion) {
                        Ok(t) => ResultadoHerramienta::exito(format!(
                            "Tarea #{} añadida: {}",
                            t.id, t.descripcion
                        )),
                        Err(e) => ResultadoHerramienta::fallo(format!("No se pudo añadir: {e}")),
                    },
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene lista de tareas (usa NexoAgente::con_tareas)",
                    ),
                }
            }
            "todo_listar" => match self.tareas.as_ref() {
                Some(lista) => ResultadoHerramienta::exito(lista.listar()),
                None => ResultadoHerramienta::fallo(
                    "El agente no tiene lista de tareas (usa NexoAgente::con_tareas)",
                ),
            },
            "todo_completar" => {
                let id = instrumento
                    .argumentos
                    .get("id")
                    .and_then(extraer_u64)
                    .ok_or_else(|| anyhow!("Instrumento 'todo_completar' requiere 'id' numérico"))?;
                match self.tareas.as_mut() {
                    Some(lista) => match lista.completar(id) {
                        Ok(()) => ResultadoHerramienta::exito(format!("Tarea #{id} completada ✅")),
                        Err(e) => ResultadoHerramienta::fallo(format!("{e}")),
                    },
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene lista de tareas (usa NexoAgente::con_tareas)",
                    ),
                }
            }
            "todo_quitar" => {
                let id = instrumento
                    .argumentos
                    .get("id")
                    .and_then(extraer_u64)
                    .ok_or_else(|| anyhow!("Instrumento 'todo_quitar' requiere 'id' numérico"))?;
                match self.tareas.as_mut() {
                    Some(lista) => match lista.quitar(id) {
                        Ok(()) => ResultadoHerramienta::exito(format!("Tarea #{id} eliminada")),
                        Err(e) => ResultadoHerramienta::fallo(format!("{e}")),
                    },
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene lista de tareas (usa NexoAgente::con_tareas)",
                    ),
                }
            }
            "web_buscar" => {
                let consulta = instrumento
                    .argumento("consulta")
                    .ok_or_else(|| anyhow!("Instrumento 'web_buscar' requiere 'consulta'"))?;
                match self.web.as_ref() {
                    Some(web) => web.buscar_como_observacion(&consulta).await?,
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene cliente web (usa NexoAgente::con_web)",
                    ),
                }
            }
            "web_extraer" => {
                let url = instrumento
                    .argumento("url")
                    .ok_or_else(|| anyhow!("Instrumento 'web_extraer' requiere 'url'"))?;
                match self.web.as_ref() {
                    Some(web) => web.extraer_como_observacion(&url).await?,
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene cliente web (usa NexoAgente::con_web)",
                    ),
                }
            }
            "programar" => {
                let expresion = instrumento
                    .argumento("expresion")
                    .ok_or_else(|| anyhow!("Instrumento 'programar' requiere 'expresion' cron"))?;
                let comando = instrumento
                    .argumento("comando")
                    .ok_or_else(|| anyhow!("Instrumento 'programar' requiere 'comando'"))?;
                match self.programador.as_mut() {
                    Some(p) => match p.programar(&expresion, &comando) {
                        Ok(t) => ResultadoHerramienta::exito(format!(
                            "Tarea programada #{}: cron '{}' → '{}'",
                            t.id, t.expresion, t.comando
                        )),
                        Err(e) => ResultadoHerramienta::fallo(format!("{e}")),
                    },
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene programador (usa NexoAgente::con_programador)",
                    ),
                }
            }
            "tareas_listar" => match self.programador.as_ref() {
                Some(p) => ResultadoHerramienta::exito(p.listar()),
                None => ResultadoHerramienta::fallo(
                    "El agente no tiene programador (usa NexoAgente::con_programador)",
                ),
            },
            "tareas_cancelar" => {
                let id = instrumento
                    .argumentos
                    .get("id")
                    .and_then(extraer_u64)
                    .ok_or_else(|| anyhow!("Instrumento 'tareas_cancelar' requiere 'id' numérico"))?;
                match self.programador.as_mut() {
                    Some(p) => match p.cancelar(id) {
                        Ok(()) => ResultadoHerramienta::exito(format!(
                            "Tarea programada #{id} cancelada"
                        )),
                        Err(e) => ResultadoHerramienta::fallo(format!("{e}")),
                    },
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene programador (usa NexoAgente::con_programador)",
                    ),
                }
            }
            "delegar" => {
                let tareas_json = instrumento
                    .argumentos
                    .get("tareas")
                    .cloned()
                    .ok_or_else(|| anyhow!("Instrumento 'delegar' requiere 'tareas' (array)"))?;
                let tareas: Vec<crate::delegacion::TareaDelegada> = serde_json::from_value(tareas_json)
                    .map_err(|e| anyhow!("'tareas' debe ser un array de {{objetivo, contexto}}: {e}"))?;
                match self.delegador.as_ref() {
                    Some(delegador) => {
                        let mut delegador = delegador.clone();
                        if let Some(mp) = instrumento.argumentos.get("max_paralelas").and_then(extraer_u64) {
                            delegador.max_paralelas = mp.max(1) as usize;
                        }
                        if let Some(ts) = instrumento.argumentos.get("timeout_seg").and_then(extraer_u64) {
                            delegador.timeout_seg = ts.max(5);
                        }
                        match delegador.delegar(&tareas).await {
                            Ok(informe) => ResultadoHerramienta::exito(informe),
                            Err(e) => ResultadoHerramienta::fallo(format!("Delegación fallida: {e}")),
                        }
                    }
                    None => ResultadoHerramienta::fallo(
                        "El agente no tiene delegador (usa NexoAgente::con_delegador)",
                    ),
                }
            }
            otro => {
                return Ok(format!("Instrumento desconocido: '{otro}'. Disponibles: bash, leer_archivo, escribir_archivo, buscar_archivos, listar_archivos, mcp_llamar, skill_listar, skill_ver, recordar, todo_agregar, todo_listar, todo_completar, todo_quitar, web_buscar, web_extraer, programar, tareas_listar, tareas_cancelar, delegar"));
            }
        };

        if resultado.exitoso {
            Ok(resultado.salida)
        } else {
            // El fallo también se devuelve como observación para que el
            // modelo pueda autocorregirse.
            Ok(format!("⚠️ ERROR: {}", resultado.salida))
        }
    }
}

/// Extrae un `u64` de un valor JSON (número o string numérico).
fn extraer_u64(v: &serde_json::Value) -> Option<u64> {
    v.as_u64()
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RespuestaLlm;

    /// Proveedor simulado para tests: devuelve pasos prefijados.
    struct ProveedorSimulado {
        pasos: Vec<String>,
        indice: std::sync::atomic::AtomicUsize,
    }

    impl ProveedorSimulado {
        fn nuevo(pasos: Vec<String>) -> Self {
            Self { pasos, indice: std::sync::atomic::AtomicUsize::new(0) }
        }
    }

    #[async_trait::async_trait]
    impl ContratoLlm for ProveedorSimulado {
        fn nombre(&self) -> &'static str {
            "simulado"
        }

        async fn conversar(
            &self,
            historial: &[MensajeHistoria],
        ) -> Result<RespuestaLlm> {
            // Verificar la invariante en cada llamada: [0] es sistema
            assert!(matches!(
                historial.first(),
                Some(m) if m.rol == RolMensaje::Sistema
            ));
            let i = self.indice.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let texto = self.pasos[i % self.pasos.len()].clone();
            Ok(RespuestaLlm {
                texto,
                finalizado_por: "stop".into(),
                modelo: "sim".into(),
            })
        }
    }

    /// Proveedor simulado que distingue las llamadas de resumen (archivista)
    /// de las llamadas del bucle agéntico.
    struct ProveedorConResumen {
        pasos: Vec<String>,
        indice: std::sync::atomic::AtomicUsize,
        resumen: String,
    }

    impl ProveedorConResumen {
        fn nuevo(pasos: Vec<String>, resumen: &str) -> Self {
            Self {
                pasos,
                indice: std::sync::atomic::AtomicUsize::new(0),
                resumen: resumen.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl ContratoLlm for ProveedorConResumen {
        fn nombre(&self) -> &'static str {
            "sim-con-resumen"
        }

        async fn conversar(
            &self,
            historial: &[MensajeHistoria],
        ) -> Result<RespuestaLlm> {
            assert!(matches!(
                historial.first(),
                Some(m) if m.rol == RolMensaje::Sistema
            ));
            // Si la llamada es del archivista, devolver el resumen prefijado.
            let es_resumen = historial
                .iter()
                .any(|m| m.contenido.contains("archivista de contexto"));
            let texto = if es_resumen {
                self.resumen.clone()
            } else {
                let i = self.indice.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.pasos[i % self.pasos.len()].clone()
            };
            Ok(RespuestaLlm {
                texto,
                finalizado_por: "stop".into(),
                modelo: "sim".into(),
            })
        }
    }

    fn ejecutor_vacio() -> EjecutorHermes {
        EjecutorHermes::nuevo(Default::default())
    }

    #[tokio::test]
    async fn invariante_maestra_se_mantiene() {
        let mut agente = NexoAgente::nuevo(
            Box::new(ProveedorSimulado::nuevo(vec![
                r#"{"razonamiento":"directo","instrumento":null,"respuesta_final":"hola"}"#.into(),
            ])),
            ejecutor_vacio(),
            "INSTRUCCIÓN MAESTRA",
        );
        assert!(agente.invariante_maestra_ok());
        let res = agente.ejecutar("saluda").await.unwrap();
        assert_eq!(res.respuesta, "hola");
        // Tras el ciclo, la instrucción maestra sigue en [0]
        assert!(agente.invariante_maestra_ok());
        assert_eq!(agente.historial[0].contenido, "INSTRUCCIÓN MAESTRA");
    }

    #[tokio::test]
    async fn ciclo_con_un_instrumento() {
        let pasos = vec![
            r#"{"razonamiento":"necesito info","instrumento":{"nombre":"bash","argumentos":{"comando":"echo 42"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"ya tengo","instrumento":null,"respuesta_final":"el resultado es 42"}"#.into(),
        ];
        let mut agente = NexoAgente::nuevo(
            Box::new(ProveedorSimulado::nuevo(pasos)),
            ejecutor_vacio(),
            "INSTRUCCIÓN MAESTRA",
        );
        let res = agente.ejecutar("dame un número").await.unwrap();
        assert_eq!(res.respuesta, "el resultado es 42");
        assert_eq!(res.instrumentos_ejecutados, 1);
        assert!(agente.invariante_maestra_ok());
    }

    #[tokio::test]
    async fn guardarrail_corrige_json_invalido() {
        let pasos = vec![
            "esto no es json".into(),
            r#"{"razonamiento":"corregido","instrumento":null,"respuesta_final":"listo"}"#.into(),
        ];
        let mut agente = NexoAgente::nuevo(
            Box::new(ProveedorSimulado::nuevo(pasos)),
            ejecutor_vacio(),
            "INSTRUCCIÓN MAESTRA",
        );
        let res = agente.ejecutar("prueba").await.unwrap();
        assert_eq!(res.respuesta, "listo");
        assert!(agente.invariante_maestra_ok());
    }

    #[tokio::test]
    async fn compactacion_no_toca_instruccion_maestra() {
        // ventana pequeña para forzar compactación
        let config = ConfigAgente {
            max_iteraciones: 5,
            max_reintentos_json: 2,
            ventana_historial: 4,
            compactar_con_resumen: false,
        };
        let pasos = vec![
            r#"{"razonamiento":"p1","instrumento":{"nombre":"bash","argumentos":{"comando":"echo a"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"p2","instrumento":{"nombre":"bash","argumentos":{"comando":"echo b"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"p3","instrumento":{"nombre":"bash","argumentos":{"comando":"echo c"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"fin","instrumento":null,"respuesta_final":"terminado"}"#.into(),
        ];
        let mut agente = NexoAgente::con_config(
            Box::new(ProveedorSimulado::nuevo(pasos)),
            ejecutor_vacio(),
            "INSTRUCCIÓN MAESTRA",
            config,
        );
        let res = agente.ejecutar("hazlo").await.unwrap();
        assert_eq!(res.respuesta, "terminado");
        // Invariante intacta tras múltiples compactaciones
        assert!(agente.invariante_maestra_ok());
        assert_eq!(agente.historial[0].contenido, "INSTRUCCIÓN MAESTRA");
    }

    #[tokio::test]
    async fn compactacion_con_resumen_llm_conserva_instruccion_maestra() {
        // Compactación con resumen LLM activado: la parte descartada se resume
        // y el resumen se inserta como mensaje de sistema, sin tocar [0].
        let config = ConfigAgente {
            max_iteraciones: 5,
            max_reintentos_json: 2,
            ventana_historial: 4,
            compactar_con_resumen: true,
        };
        let pasos = vec![
            r#"{"razonamiento":"p1","instrumento":{"nombre":"bash","argumentos":{"comando":"echo a"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"p2","instrumento":{"nombre":"bash","argumentos":{"comando":"echo b"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"p3","instrumento":{"nombre":"bash","argumentos":{"comando":"echo c"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"p4","instrumento":{"nombre":"bash","argumentos":{"comando":"echo d"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"fin","instrumento":null,"respuesta_final":"terminado"}"#.into(),
        ];
        let mut agente = NexoAgente::con_config(
            Box::new(ProveedorConResumen::nuevo(
                pasos,
                "El usuario pidió una operación; se ejecutaron comandos de prueba.",
            )),
            ejecutor_vacio(),
            "INSTRUCCIÓN MAESTRA",
            config,
        );
        let res = agente.ejecutar("hazlo").await.unwrap();
        assert_eq!(res.respuesta, "terminado");
        // La compactación LLM dejó un resumen etiquetado en el historial
        let hay_resumen = agente
            .historial
            .iter()
            .any(|m| m.contenido.contains("📌 RESUMEN DE CONTEXTO ANTERIOR"));
        assert!(hay_resumen, "el historial debería contener el resumen LLM");
        // La instrucción maestra sigue intacta en [0]
        assert!(agente.invariante_maestra_ok());
        assert_eq!(agente.historial[0].contenido, "INSTRUCCIÓN MAESTRA");
    }

    #[tokio::test]
    async fn mcp_sin_cliente_devuelve_observacion_controlada() {
        // Sin cliente MCP, mcp_llamar no debe tumbar el ciclo: devuelve una
        // observación de error controlada que el modelo puede leer.
        let pasos = vec![
            r#"{"razonamiento":"consulto cerebro","instrumento":{"nombre":"mcp_llamar","argumentos":{"herramienta":"listar_agentes","argumentos":"{}"}},"respuesta_final":null}"#.into(),
            r#"{"razonamiento":"ok","instrumento":null,"respuesta_final":"listo"}"#.into(),
        ];
        let mut agente = NexoAgente::nuevo(
            Box::new(ProveedorSimulado::nuevo(pasos)),
            ejecutor_vacio(),
            "INSTRUCCIÓN MAESTRA",
        );
        let res = agente.ejecutar("consulta al cerebro").await.unwrap();
        assert_eq!(res.respuesta, "listo");
        assert_eq!(res.instrumentos_ejecutados, 1);
        assert!(
            res.saltos[0].observacion.contains("no tiene cliente MCP"),
            "observación inesperada: {}",
            res.saltos[0].observacion
        );
        assert!(agente.invariante_maestra_ok());
    }

    #[test]
    fn construir_historial_no_duplica_instruccion_maestra() {
        let agente = NexoAgente::nuevo(
            Box::new(ProveedorSimulado::nuevo(vec![])),
            ejecutor_vacio(),
            "MAESTRA",
        );
        // el historial interno ya tiene [0]; construir_historial no debe duplicar
        let envio = agente.construir_historial_para_envio();
        let maestras = envio
            .iter()
            .filter(|m| m.rol == RolMensaje::Sistema && m.contenido == "MAESTRA")
            .count();
        assert_eq!(maestras, 1);
    }
}
