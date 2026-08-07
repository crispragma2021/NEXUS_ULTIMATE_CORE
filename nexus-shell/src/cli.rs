// ==========================================
// 🐚 NEXUS Shell — Interfaz de Línea de Comandos
// ==========================================
// Parser manual de argumentos (sin clap, cero dependencias extra)

use crate::config::ShellMode;

#[derive(Debug, PartialEq)]
pub enum Command {
    /// `nexus daemon start` — Inicia el servidor en background
    DaemonStart,
    /// `nexus daemon stop` — Detiene el servidor
    DaemonStop,
    /// `nexus daemon status` — Estado del daemon
    DaemonStatus,
    /// `nexus daemon foreground` — Inicia el servidor en primer plano
    DaemonForeground,
    /// `nexus eval <prompt>` — Evalúa un prompt con el CEREBRO
    Eval(String),
    /// `nexus status` — Salud del sistema
    Status,
    /// `nexus pensar <modo> <prompt>` — Variante específica de pensar
    Pensar { modo: String, prompt: String },
    /// `nexus v0 generate <prompt> [--session-id <uuid>]` — Genera UI con el pipeline multi-agente v0
    V0Generate { prompt: String, session_id: Option<String> },
    /// `nexus help` — Ayuda
    Help,
}

impl Command {
    /// Parsear argumentos de línea de comandos
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.is_empty() {
            return Ok(Command::Help);
        }

        let cmd = &args[0].to_lowercase();

        match cmd.as_str() {
            "daemon" | "d" => {
                if args.len() < 2 {
                    return Err("Uso: nexus daemon {start|stop|status|foreground}".into());
                }
                match args[1].to_lowercase().as_str() {
                    "start" => Ok(Command::DaemonStart),
                    "stop" => Ok(Command::DaemonStop),
                    "status" => Ok(Command::DaemonStatus),
                    "foreground" | "fg" => Ok(Command::DaemonForeground),
                    other => Err(format!("Subcomando desconocido: {other}. Usa: start, stop, status, foreground")),
                }
            }
            "eval" | "e" => {
                let prompt = args[1..].join(" ");
                if prompt.is_empty() {
                    return Err("Uso: nexus eval <prompt>".into());
                }
                Ok(Command::Eval(prompt))
            }
            "pensar" | "p" => {
                if args.len() < 3 {
                    return Err("Uso: nexus pensar <modo> <prompt>. Modos: auto, razonar, crear, debug".into());
                }
                let modo = args[1].to_lowercase();
                let prompt = args[2..].join(" ");
                Ok(Command::Pensar { modo, prompt })
            }
            "v0" => {
                // Formato: nexus v0 generate <prompt> [--session-id <uuid>]
                if args.len() < 3 || args[1].to_lowercase() != "generate" {
                    return Err("Uso: nexus v0 generate <prompt> [--session-id <uuid>]".into());
                }
                let mut session_id: Option<String> = None;
                let mut prompt_parts: Vec<String> = Vec::new();
                let mut i = 2;
                while i < args.len() {
                    if args[i] == "--session-id" || args[i] == "-s" {
                        if i + 1 >= args.len() {
                            return Err("Falta valor para --session-id".into());
                        }
                        session_id = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        prompt_parts.push(args[i].clone());
                        i += 1;
                    }
                }
                let prompt = prompt_parts.join(" ");
                if prompt.is_empty() {
                    return Err("Uso: nexus v0 generate <prompt> [--session-id <uuid>]".into());
                }
                Ok(Command::V0Generate { prompt, session_id })
            }
            "status" | "s" => Ok(Command::Status),
            "help" | "h" | "--help" | "-h" => Ok(Command::Help),
            _ => Err(format!("Comando desconocido: {cmd}. Usa: nexus help")),
        }
    }

    /// Determinar el modo shell según el comando
    pub fn shell_mode(&self) -> ShellMode {
        match self {
            Command::DaemonStart | Command::DaemonForeground => ShellMode::Daemon,
            Command::Eval(_) | Command::Pensar { .. } | Command::V0Generate { .. }
            | Command::Status | Command::Help => ShellMode::Cli,
            Command::DaemonStop | Command::DaemonStatus => ShellMode::Cli,
        }
    }
}

/// Imprimir ayuda en terminal
pub fn print_help() {
    println!(r#"
🐚 NEXUS Shell v{} — El cuerpo soberano del Orquestador

USO:
    nexus <comando> [subcomando] [argumentos...]

COMANDOS PRINCIPALES:
    daemon start       Inicia el servidor NEXUS en background
    daemon stop        Detiene el servidor
    daemon status      Estado del daemon
    daemon foreground  Inicia en primer plano (muestra logs en terminal)
    eval <prompt>      Evalúa un prompt con el CEREBRO y muestra respuesta
    pensar <m> <p>     Evalúa con modo específico (auto|razonar|crear|debug)
    v0 generate <p>    Genera UI con el pipeline multi-agente (--session-id opcional)
    status             Muestra salud del sistema y órganos activos
    help               Muestra esta ayuda

EJEMPLOS:
    nexus daemon start
    nexus eval "¿Qué sabes de este dominio?"
    nexus pensar razonar "Analiza esta vulnerabilidad"
    nexus v0 generate "crea un dashboard de ventas con tabla y grafico" --session-id abc-123
    nexus status
    nexus help

DOCS: https://nexus.sovereign
"#, env!("CARGO_PKG_VERSION"));
}

/// Imprimir header de NEXUS
pub fn print_header() {
    println!(r#" 🧠 NEXUS Shell v{} — Sistema Nervioso Central"#, env!("CARGO_PKG_VERSION"));
    println!(" ─────────────────────────────────────────────");
}
