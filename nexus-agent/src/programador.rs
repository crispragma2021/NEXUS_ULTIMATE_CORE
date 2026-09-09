// ============================================================================
// NEXUS-AGENT · programador.rs — Scheduler cron (tareas programadas)
// ============================================================================
// Patrón absorbido de Hermes: ejecución programada de tareas con expresiones
// cron. El agente programa comandos (`programar`), los lista y los cancela;
// el modo `--daemon` del binario revisa periódicamente las tareas y ejecuta
// las que tocan (expresión cron evaluada contra el reloj local).
//
// Persistencia: JSON (una tarea por entrada), escritura atómica, IDs
// incrementales. La ejecución de una tarea es un comando bash con timeout;
// el resultado se registra en un log anexo (sin tocar el sandbox del agente).
// ============================================================================

use anyhow::{anyhow, Context, Result};
use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Una tarea programada con expresión cron.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TareaProgramada {
    pub id: u64,
    /// Expresión cron estándar de 5 campos (min hora día-mes mes día-sem).
    pub expresion: String,
    /// Comando bash a ejecutar cuando toque.
    pub comando: String,
    /// Epoch millis de creación.
    pub creado: u64,
}

/// Programador de tareas cron cargado en memoria y persistido en disco.
#[derive(Debug, Clone, Default)]
pub struct Programador {
    ruta: PathBuf,
    tareas: Vec<TareaProgramada>,
    /// Segundos de timeout por comando (por defecto 60).
    pub timeout_comando_seg: u64,
}

impl Programador {
    /// Carga las tareas desde `ruta`. Archivo inexistente → vacío; corrupto
    /// → aviso y arranque vacío (no aborta el daemon).
    pub fn cargar(ruta: PathBuf) -> Result<Self> {
        let mut p = Self { ruta, tareas: Vec::new(), timeout_comando_seg: 60 };
        if p.ruta.is_file() {
            match std::fs::read_to_string(&p.ruta) {
                Ok(contenido) => match serde_json::from_str::<Vec<TareaProgramada>>(&contenido) {
                    Ok(tareas) => p.tareas = tareas,
                    Err(e) => eprintln!(
                        "⚠️ Aviso: '{}' corrupto ({e}); se arranca sin tareas",
                        p.ruta.display()
                    ),
                },
                Err(e) => eprintln!("⚠️ Aviso: no se pudo leer '{}': {e}", p.ruta.display()),
            }
        }
        Ok(p)
    }

    /// Programa un comando con una expresión cron. Valida la expresión antes
    /// de persistir. Error si la expresión es inválida.
    ///
    /// Se acepta cron estándar de 5 campos (min hora día-mes mes día-sem) o
    /// el formato extendido de 6/7 campos con segundos de la crate `cron`;
    /// las expresiones de 5 campos se normalizan anteponiendo "0 " (segundo 0).
    pub fn programar(&mut self, expresion: &str, comando: &str) -> Result<TareaProgramada> {
        let expresion = normalizar_cron(expresion);
        let comando = comando.trim().to_string();
        if comando.is_empty() {
            anyhow::bail!("El comando no puede estar vacío");
        }
        // Validar la expresión cron ANTES de guardar
        expresion
            .parse::<cron::Schedule>()
            .map_err(|e| anyhow!("Expresión cron inválida '{expresion}': {e}"))?;
        let id = self.tareas.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let tarea = TareaProgramada {
            id,
            expresion,
            comando,
            creado: crate::tareas::epoch_millis(),
        };
        self.tareas.push(tarea.clone());
        self.persistir()?;
        Ok(tarea)
    }

    /// Vista legible de las tareas programadas.
    pub fn listar(&self) -> String {
        if self.tareas.is_empty() {
            return "No hay tareas programadas.".to_string();
        }
        let mut out = String::from("⏰ TAREAS PROGRAMADAS (cron):\n");
        for t in &self.tareas {
            out.push_str(&format!(
                "[{}] cron='{}' → {}\n",
                t.id, t.expresion, t.comando
            ));
        }
        out
    }

    /// Cancela una tarea por id. Error si no existe.
    pub fn cancelar(&mut self, id: u64) -> Result<()> {
        let antes = self.tareas.len();
        self.tareas.retain(|t| t.id != id);
        if self.tareas.len() == antes {
            anyhow::bail!("No existe la tarea programada #{id}");
        }
        self.persistir()
    }

    /// Ejecuta las tareas cuyo cron disparó en el minuto actual.
    ///
    /// Devuelve los resultados (comando + salida truncada) para que el daemon
    /// los registre. Un comando fallido NO aborta el resto: se registra el
    /// error y se continúa.
    pub async fn ejecutar_debidas(&mut self) -> Result<Vec<String>> {
        let ahora = Local::now();
        let minuto_actual = ahora.minute();
        let mut resultados = Vec::new();
        let debidas: Vec<TareaProgramada> = self
            .tareas
            .iter()
            .filter(|t| cron_dispara_en_minuto(&t.expresion, &ahora))
            .cloned()
            .collect();

        for tarea in debidas {
            let salida = tokio::time::timeout(
                Duration::from_secs(self.timeout_comando_seg),
                ejecutar_comando(&tarea.comando),
            )
            .await;

            match salida {
                Ok(Ok(texto)) => resultados.push(format!(
                    "[cron #{} '{}'] OK: {}",
                    tarea.id, tarea.expresion, texto
                )),
                Ok(Err(e)) => resultados.push(format!(
                    "[cron #{} '{}'] ERROR: {e}",
                    tarea.id, tarea.expresion
                )),
                Err(_) => resultados.push(format!(
                    "[cron #{} '{}'] TIMEOUT ({}s)",
                    tarea.id, tarea.expresion, self.timeout_comando_seg
                )),
            }
        }
        let _ = minuto_actual; // el filtro ya usa el reloj; se mantiene para claridad
        Ok(resultados)
    }

    /// Tareas actuales (para inspección y tests).
    pub fn tareas(&self) -> &[TareaProgramada] {
        &self.tareas
    }

    fn persistir(&self) -> Result<()> {
        if let Some(padre) = self.ruta.parent() {
            std::fs::create_dir_all(padre)
                .with_context(|| format!("No se pudo crear '{}'", padre.display()))?;
        }
        let tmp = self.ruta.with_extension("tmp");
        let datos = serde_json::to_string_pretty(&self.tareas)?;
        std::fs::write(&tmp, datos)
            .with_context(|| format!("No se pudo escribir '{}'", tmp.display()))?;
        std::fs::rename(&tmp, &self.ruta)
            .with_context(|| format!("No se pudo reemplazar '{}'", self.ruta.display()))?;
        Ok(())
    }
}

/// Normaliza una expresión cron: las de 5 campos (estándar, sin segundos)
/// se convierten a 6 anteponiendo "0 ". Las de 6/7 campos pasan tal cual.
fn normalizar_cron(expresion: &str) -> String {
    let exp = expresion.trim();
    let campos = exp.split_whitespace().count();
    if campos == 5 {
        format!("0 {exp}")
    } else {
        exp.to_string()
    }
}

/// ¿La expresión cron dispara en el minuto actual?
///
/// El cron estándar tiene resolución de minuto: se comprueba si la siguiente
/// ocurrencia desde `inicio_de_minuto - 1s` cae dentro del minuto actual.
fn cron_dispara_en_minuto(expresion: &str, ahora: &chrono::DateTime<Local>) -> bool {
    let schedule = match normalizar_cron(expresion).parse::<cron::Schedule>() {
        Ok(s) => s,
        Err(_) => return false,
    };
    // Inicio del minuto actual (hora local con offset correcto)
    let inicio = ahora
        .with_second(0)
        .and_then(|d| d.with_nanosecond(0))
        .unwrap_or(*ahora);
    let un_minuto = chrono::Duration::minutes(1);
    let candidato = schedule
        .after(&(inicio - chrono::Duration::seconds(1)))
        .next();
    match candidato {
        Some(t) => t >= inicio && t < inicio + un_minuto,
        None => false,
    }
}

/// Ejecuta un comando bash y devuelve la salida (truncada a 2 KiB).
async fn ejecutar_comando(comando: &str) -> Result<String> {
    let hijo = tokio::process::Command::new("bash")
        .arg("-c")
        .arg(comando)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .context("No se pudo lanzar bash")?;
    let salida = hijo
        .wait_with_output()
        .await
        .context("No se pudo esperar al comando")?;
    let mut texto = String::from_utf8_lossy(&salida.stdout).to_string();
    if !salida.stderr.is_empty() {
        texto.push_str(&String::from_utf8_lossy(&salida.stderr));
    }
    let texto: String = texto.chars().take(2 * 1024).collect();
    if salida.status.success() {
        Ok(texto)
    } else {
        Err(anyhow!("exit {:?}: {}", salida.status.code(), texto))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    fn ruta_temporal() -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nexus_cron_{}_{n}.json", std::process::id()))
    }

    #[test]
    fn programa_valida_y_lista() {
        let ruta = ruta_temporal();
        let mut p = Programador::cargar(ruta.clone()).unwrap();
        let t = p.programar("0 9 * * *", "echo buenos dias").unwrap();
        assert_eq!(t.id, 1);
        assert!(p.listar().contains("0 9 * * *"));
        assert!(p.listar().contains("echo buenos dias"));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn rechaza_expresion_cron_invalida() {
        let ruta = ruta_temporal();
        let mut p = Programador::cargar(ruta.clone()).unwrap();
        assert!(p.programar("no es cron", "echo x").is_err());
        assert!(p.tareas().is_empty());
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn persiste_entre_cargas() {
        let ruta = ruta_temporal();
        {
            let mut p = Programador::cargar(ruta.clone()).unwrap();
            p.programar("*/5 * * * *", "echo tick").unwrap();
        }
        let p = Programador::cargar(ruta.clone()).unwrap();
        assert_eq!(p.tareas().len(), 1);
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn cancela_por_id() {
        let ruta = ruta_temporal();
        let mut p = Programador::cargar(ruta.clone()).unwrap();
        p.programar("0 0 * * *", "echo a").unwrap();
        p.programar("0 0 * * *", "echo b").unwrap();
        p.cancelar(1).unwrap();
        assert_eq!(p.tareas().len(), 1);
        assert!(p.cancelar(99).is_err());
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn cron_dispara_cada_minuto() {
        // "*/1 * * * *" dispara todos los minutos
        let ahora = Local::now();
        assert!(cron_dispara_en_minuto("*/1 * * * *", &ahora));
        // "0 9 * * *" NO dispara a menos que sea las 9:00
        if ahora.hour() != 9 || ahora.minute() != 0 {
            assert!(!cron_dispara_en_minuto("0 9 * * *", &ahora));
        }
    }
}
