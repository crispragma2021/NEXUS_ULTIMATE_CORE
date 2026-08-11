// ============================================================================
// NEXUS-AGENT · main.rs — CLI de interacción
// ============================================================================
// Puntos de entrada:
//   - --proveedor deepseek|openai|ollama   (defecto: deepseek)
//   - --modelo <nombre>                    (defecto según proveedor)
//   - --comando "texto"                    ejecuta un único ciclo y termina
//   - (sin --comando)                      sesión interactiva por stdin
// Memoria de proyecto:
//   - NEXUS_AGENTE_GLOBAL: ruta a un AGENTE.md personal del Arquitecto (global)
//   - AGENTE.md:           se busca en cwd y carpetas superiores hasta la raíz
//   - NEXUS_AGENT_RAIZ:    raíz del sandbox; limita la subida de AGENTE.md
// ============================================================================

use anyhow::{Context, Result};
use nexus_agent::{
    BibliotecaSkills, ClienteMcp, ClienteWeb, ContratoLlm, DeepSeekCliente, Delegador,
    EjecutorHermes, ListaTareas, MemoriaEstado, MemoriaProyecto, ModeloCliente,
    ModeloClienteGenerico, NexoAgente, OllamaCliente, Programador, ReglasJSON, SandboxConfig,
    Transcripcion,
};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

const INSTRUCCION_MAESTRA: &str = r#"Eres NEXUS-Agent, el motor agéntico soberano del Arquitecto.
Objetivo: resolver la petición del usuario paso a paso usando tus instrumentos cuando sea necesario.

REGLAS:
1. Razona de forma concisa y honesta.
2. Si necesitas ejecutar algo en el sistema, usa el instrumento adecuado.
3. Cuando tengas la respuesta final, entrégala de forma clara y directa.
4. Para consultar el cerebro NEXUS (memoria, agentes, conocimiento, visión,
   diagnóstico, pensar, tribunal...), usa el instrumento mcp_llamar. Su primer
   argumento es 'herramienta' (nombre de la herramienta MCP) y el segundo es
   'argumentos' (objeto JSON). Las herramientas del cerebro se listan con
   mcp_llamar(herramienta="listar_herramientas", argumentos={}).
5. Si existe un skill de la biblioteca que aplique a la tarea (míralo en el
   catálogo de SKILLS DISPONIBLES), cárgalo con skill_ver antes de actuar y
   sigue sus pasos.
6. Si el Arquitecto te pide recordar algo (preferencia, decisión, aprendizaje),
   guárdalo con el instrumento recordar: quedará disponible en futuras sesiones.

"#;

/// Catálogo de instrumentos visibles para el modelo en la instrucción maestra.
const DESCRIPCION_HERRAMIENTAS: &str = r#"INSTRUMENTOS DISPONIBLES:
- bash:        ejecuta un comando en la shell del sandbox. {comando: string}
- leer_archivo:  lee un archivo dentro del sandbox. {ruta: string}
- escribir_archivo: escribe contenido en un archivo. {ruta: string, contenido: string}
- buscar_archivos: busca un patrón regex dentro del contenido de archivos.
               {patron: string, ruta?: string, glob?: string, max_resultados?: number}
- listar_archivos: lista archivos y carpetas bajo una ruta (recursivo).
               {ruta?: string, max_resultados?: number}
- recordar:     guarda un hecho en la memoria de estado del agente (persiste
               entre sesiones). {hecho: string}
- skill_listar:  lista los skills disponibles en la biblioteca. {}
- skill_ver:     carga el contenido completo de un skill. {nombre: string}
- todo_agregar:  añade una tarea a la lista persistente. {descripcion: string}
- todo_listar:   muestra la lista de tareas con su estado. {}
- todo_completar: marca una tarea como completada. {id: number}
- todo_quitar:   elimina una tarea de la lista. {id: number}
- web_buscar:    busca en la web (DuckDuckGo) y devuelve resultados. {consulta: string}
- web_extraer:   extrae el texto legible de una URL. {url: string}
- programar:     programa un comando con expresión cron. {expresion: string, comando: string}
- tareas_listar: lista las tareas programadas (cron). {}
- tareas_cancelar: cancela una tarea programada. {id: number}
- delegar:       lanza subagentes en paralelo con contexto aislado.
               {tareas: [{objetivo: string, contexto: string}], max_paralelas?: number, timeout_seg?: number}
- mcp_llamar:   invoca una herramienta del cerebro NEXUS a través de MCP stdio
               (claws_mcp). {herramienta: string, argumentos: objeto JSON}
               Ejemplos: consultar_memoria, buscar_conocimiento, listar_agentes,
               ejecutar_workflow, nexus_pensar, sentinel_diagnostic, nexus_tribunal.
               Usa argumentos={} cuando la herramienta no requiera parámetros.
"#;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let proveedor = arg_valor(&args, "--proveedor").unwrap_or_else(|| "deepseek".to_string());
    let modelo = arg_valor(&args, "--modelo");
    let comando = arg_valor(&args, "--comando");
    let modo_daemon = args.iter().any(|a| a == "--daemon");
    let subagente = arg_valor(&args, "--subagente");
    let contexto = arg_valor(&args, "--contexto").unwrap_or_default();

    // Directorio de datos (sesiones, skills, estado, tareas, cron).
    let datos_dir = std::env::var("NEXUS_AGENT_DATOS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".local/share/nexus-agent")
        });

    // Modo daemon: scheduler cron en primer plano (no necesita LLM).
    if modo_daemon {
        return ejecutar_daemon(&datos_dir).await;
    }

    // 1. Construir el proveedor según la selección
    let proveedor_dinamico: Box<dyn ContratoLlm> = match proveedor.as_str() {
        "deepseek" => {
            let clave = std::env::var("DEEPSEEK_API_KEY")
                .context("Falta la variable DEEPSEEK_API_KEY")?;
            match modelo.clone() {
                Some(m) => Box::new(DeepSeekCliente::con_modelo(&clave, &m)?),
                None => Box::new(DeepSeekCliente::nuevo(&clave)?),
            }
        }
        "openai" => {
            let config = ModeloCliente {
                proveedor: "openai".into(),
                modelo: modelo.clone().unwrap_or_else(|| "gpt-4o-mini".into()),
                url_base: std::env::var("OPENAI_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
                clave_api: Some(
                    std::env::var("OPENAI_API_KEY")
                        .context("Falta la variable OPENAI_API_KEY")?,
                ),
                extras: Default::default(),
            };
            Box::new(ModeloClienteGenerico::nuevo(&config)?)
        }
        "ollama" => {
            let m = modelo.clone().unwrap_or_else(|| "llama3".into());
            Box::new(OllamaCliente::nuevo(&m)?)
        }
        otro => {
            anyhow::bail!(
                "Proveedor desconocido: '{otro}'. Usa: deepseek | openai | ollama"
            )
        }
    };

    // 2. Sandbox del ejecutor: restringido a NEXUS_ULTIMATE_CORE por defecto
    let raiz = std::env::var("NEXUS_AGENT_RAIZ").ok().map(std::path::PathBuf::from);
    let config_sandbox = SandboxConfig {
        directorio_raiz: raiz.clone(),
        ..Default::default()
    };
    let ejecutor = EjecutorHermes::nuevo(config_sandbox);

    // 2.5 Memoria de proyecto: AGENTE.md jerárquico (global → proyecto → carpeta).
    //      Se fusiona al frente de la instrucción maestra para que el agente
    //      conozca las reglas del proyecto. Si no hay archivos o falla la
    //      lectura, el agente arranca igual sin memoria.
    let cwd = std::env::current_dir().context("No se pudo obtener el directorio actual")?;
    let memoria = match MemoriaProyecto::cargar(&cwd, raiz.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("⚠️ Aviso: no se pudo cargar la memoria de proyecto: {e}");
            MemoriaProyecto::default()
        }
    };
    if memoria.tiene_memoria() {
        println!(
            "🧠 Memoria de proyecto cargada ({} pieza{})",
            memoria.piezas().len(),
            if memoria.piezas().len() == 1 { "" } else { "s" }
        );
    }

    // 2.6 Capacidades absorbidas de Hermes: skills, sesión persistente y
    //      memoria de estado. Rutas configurables con variables de entorno;
    //      por defecto viven en ~/.local/share/nexus-agent/ (convención XDG).

    // Skills: biblioteca de procedimientos (SKILL.md con frontmatter).
    let dir_skills = std::env::var("NEXUS_AGENT_SKILLS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| datos_dir.join("skills"));
    let skills = match BibliotecaSkills::cargar(&dir_skills) {
        Ok(b) => {
            if b.cantidad() > 0 {
                println!(
                    "📚 {} skill(s) cargados desde {}",
                    b.cantidad(),
                    dir_skills.display()
                );
            }
            b
        }
        Err(e) => {
            eprintln!("⚠️ Aviso: no se pudo cargar la biblioteca de skills: {e}");
            BibliotecaSkills::default()
        }
    };

    // Sesión: transcripción JSONL de ESTA sesión (append-only).
    let dir_sesiones = std::env::var("NEXUS_AGENT_SESIONES")
        .map(PathBuf::from)
        .unwrap_or_else(|_| datos_dir.join("sesiones"));
    let ts_ahora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let ruta_sesion = dir_sesiones.join(format!("sesion-{ts_ahora}.jsonl"));
    let sesion = Transcripcion::nueva(ruta_sesion.clone()).ok();
    if sesion.is_some() {
        println!("🗒️  Sesión transcrita a {}", ruta_sesion.display());
    }

    // Memoria de estado: hechos duraderos del agente entre sesiones.
    let ruta_estado = std::env::var("NEXUS_AGENT_ESTADO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| datos_dir.join("estado.md"));
    let memoria_estado = MemoriaEstado::cargar(ruta_estado.clone()).unwrap_or_default();
    if memoria_estado.tiene_memoria() {
        println!(
            "🧠 Memoria de estado: {} hecho(s) desde {}",
            memoria_estado.entradas().len(),
            ruta_estado.display()
        );
    }

    // 2.7 Fase 2: web, tareas persistente y programador cron.
    let web = ClienteWeb::default();
    let tareas = ListaTareas::cargar(datos_dir.join("tareas.json")).unwrap_or_default();
    if !tareas.tareas().is_empty() {
        println!("📋 {} tarea(s) en la lista persistente", tareas.tareas().len());
    }
    let programador = Programador::cargar(datos_dir.join("tareas_programadas.json"))
        .unwrap_or_default();
    if !programador.tareas().is_empty() {
        println!("⏰ {} tarea(s) programada(s) (cron)", programador.tareas().len());
    }

    // 3. Cliente MCP hacia el cerebro NEXUS (claws_mcp).
    //    El binario se puede sobreescribir con NEXUS_CLAWS_MCP.
    let binario_mcp = std::env::var("NEXUS_CLAWS_MCP").unwrap_or_else(|_| "claws_mcp".into());
    let cliente_mcp = ClienteMcp::nuevo(&binario_mcp);

    // 4. Instrucción maestra = sistema + catálogo + esquema JSON obligatorio
    let instruccion = format!(
        "{}\n{}\n{}",
        INSTRUCCION_MAESTRA,
        DESCRIPCION_HERRAMIENTAS,
        ReglasJSON::plantilla_esquema()
    );

    let mut agente = NexoAgente::nuevo(proveedor_dinamico, ejecutor, &instruccion)
        .con_mcp(cliente_mcp)
        .con_memoria_proyecto(memoria)
        .con_skills(skills)
        .con_memoria_estado(memoria_estado)
        .con_web(web)
        .con_tareas(tareas)
        .con_programador(programador);
    if let Some(sesion) = sesion {
        agente = agente.con_sesion(sesion);
    }
    // El delegador NO se conecta en subagentes: limita la profundidad a 1
    // (un subagente no puede delegar a su vez).
    if subagente.is_none() {
        agente = agente.con_delegador(Delegador::nuevo(&proveedor, modelo.clone())?);
    }

    // 3.5 Reanudación opcional: --reanudar <ruta.jsonl> carga las últimas
    //      entradas de una sesión previa como contexto de arranque.
    if let Some(ruta_prev) = arg_valor(&args, "--reanudar") {
        match Transcripcion::reanudar(std::path::Path::new(&ruta_prev), 40) {
            Ok(mensajes) if !mensajes.is_empty() => {
                agente.reanudar_con(mensajes);
                println!("♻️  Sesión reanudada desde {}", ruta_prev);
            }
            Ok(_) => eprintln!("⚠️ Aviso: '{}' no tiene entradas reanudables", ruta_prev),
            Err(e) => eprintln!("⚠️ Aviso: no se pudo reanudar '{}': {e}", ruta_prev),
        }
    }

    // 3.6 Modo subagente (usado por `delegar`): ejecuta el objetivo con el
    //      contexto aislado inyectado tras la instrucción maestra e imprime
    //      SOLO la respuesta final (la lee el delegador del padre).
    if let Some(objetivo) = subagente {
        if !contexto.is_empty() {
            agente.reanudar_con(vec![nexus_agent::MensajeHistoria::sistema(format!(
                "CONTEXTO AISLADO DE LA TAREA DELEGADA:\n{contexto}"
            ))]);
        }
        let resultado = agente
            .ejecutar(&objetivo)
            .await
            .context("El subagente falló")?;
        println!("{}", resultado.respuesta);
        return Ok(());
    }

    // 4. Ejecutar comando único o sesión interactiva
    match comando {
        Some(cmd) => {
            let resultado = agente
                .ejecutar(&cmd)
                .await
                .context("El ciclo del agente falló")?;
            println!("{}", resultado.respuesta);
        }
        None => {
            println!("NEXUS-Agent interactivo. Escribe 'salir' para terminar.");
            let stdin = std::io::stdin();
            loop {
                print!("> ");
                std::io::stdout().flush().ok();
                let mut linea = String::new();
                let n = stdin
                    .lock()
                    .read_line(&mut linea)
                    .context("No se pudo leer stdin")?;
                if n == 0 {
                    break;
                }
                let linea = linea.trim().to_string();
                if linea.eq_ignore_ascii_case("salir")
                    || linea.eq_ignore_ascii_case("exit")
                    || linea.eq_ignore_ascii_case("quit")
                {
                    break;
                }
                if linea.is_empty() {
                    continue;
                }
                match agente.ejecutar(&linea).await {
                    Ok(r) => println!("{}", r.respuesta),
                    Err(e) => eprintln!("⚠️ Error del agente: {e}"),
                }
            }
        }
    }

    Ok(())
}

/// Daemon del scheduler: revisa las tareas cron cada 30 segundos y ejecuta
/// las que tocan. Corre SIN LLM (no necesita proveedor ni API keys), así que
/// puede vivir como servicio independiente del agente.
async fn ejecutar_daemon(datos_dir: &std::path::Path) -> Result<()> {
    let ruta = datos_dir.join("tareas_programadas.json");
    let mut programador = Programador::cargar(ruta.clone()).unwrap_or_default();
    println!(
        "⏰ Daemon NEXUS-Agent activo — {} tarea(s) programada(s) en {}",
        programador.tareas().len(),
        ruta.display()
    );
    loop {
        match programador.ejecutar_debidas().await {
            Ok(resultados) => {
                for r in &resultados {
                    println!("{}", r);
                }
            }
            Err(e) => eprintln!("⚠️ Error del programador: {e}"),
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

/// Lee el valor de un argumento `--clave valor` o `--clave=valor`.
fn arg_valor(args: &[String], clave: &str) -> Option<String> {
    let prefijo = format!("{clave}=");
    for a in args {
        if let Some(v) = a.strip_prefix(&prefijo) {
            return Some(v.to_string());
        }
    }
    let idx = args.iter().position(|a| a == clave)?;
    args.get(idx + 1).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_valor_con_igual() {
        let args = vec!["nexus-agent".into(), "--proveedor=ollama".into()];
        assert_eq!(arg_valor(&args, "--proveedor").as_deref(), Some("ollama"));
    }

    #[test]
    fn arg_valor_con_espacio() {
        let args = vec!["nexus-agent".into(), "--modelo".into(), "llama3".into()];
        assert_eq!(arg_valor(&args, "--modelo").as_deref(), Some("llama3"));
    }

    #[test]
    fn arg_valor_ausente_devuelve_none() {
        let args = vec!["nexus-agent".into()];
        assert_eq!(arg_valor(&args, "--comando"), None);
    }
}
