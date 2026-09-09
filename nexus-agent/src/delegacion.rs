// ============================================================================
// NEXUS-AGENT · delegacion.rs — Subagentes en paralelo con contexto aislado
// ============================================================================
// Patrón absorbido de Hermes (delegación): lanzar N tareas independientes,
// cada una con su propio contexto, ejecutarlas en paralelo y recoger un
// resumen de cada una. Implementación soberana:
//
//   - Cada subagente es un PROCESO del mismo binario (`nexus-agent
//     --subagente "objetivo" --contexto "..."`), con su propio historial,
//     su propio bucle ReAct y el mismo proveedor/sandbox heredados.
//   - Aislamiento real: un subagente que se cuelga no bloquea a los demás
//     (timeout por tarea) ni corrompe el historial del padre.
//   - Límite de paralelismo (semáforo) y de profundidad: el modo subagente
//     NO expone la herramienta `delegar`, así que no hay recursión infinita
//     (profundidad máxima 1, como el patrón original).
// ============================================================================

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::sync::Semaphore;

/// Configuración del delegador.
#[derive(Debug, Clone)]
pub struct Delegador {
    /// Ruta al binario (por defecto, el ejecutable actual).
    pub binario: PathBuf,
    /// Máximo de subagentes en paralelo.
    pub max_paralelas: usize,
    /// Timeout (segundos) por subagente.
    pub timeout_seg: u64,
    /// Proveedor y modelo a pasar a cada subagente (heredados del padre).
    pub proveedor: String,
    pub modelo: Option<String>,
}

impl Delegador {
    /// Crea un delegador apuntando al binario actual.
    pub fn nuevo(proveedor: &str, modelo: Option<String>) -> Result<Self> {
        let binario = std::env::current_exe()
            .context("No se pudo resolver la ruta del binario actual")?;
        Ok(Self {
            binario,
            max_paralelas: 3,
            timeout_seg: 180,
            proveedor: proveedor.to_string(),
            modelo,
        })
    }

    /// Lanza `tareas` en paralelo (con semáforo) y devuelve un informe
    /// consolidado: una entrada por subagente con su objetivo y su salida.
    pub async fn delegar(&self, tareas: &[TareaDelegada]) -> Result<String> {
        if tareas.is_empty() {
            return Ok("No se delegó ninguna tarea (lista vacía).".to_string());
        }
        if tareas.len() > 8 {
            return Err(anyhow!(
                "Demasiadas tareas para delegar ({}). Máximo 8.",
                tareas.len()
            ));
        }
        let semaforo = Arc::new(Semaphore::new(self.max_paralelas));
        let mut manejos = Vec::with_capacity(tareas.len());
        for (i, tarea) in tareas.iter().enumerate() {
            let permiso = semaforo.clone().acquire_owned().await?;
            let tarea = tarea.clone();
            let self_ = self.clone();
            manejos.push(tokio::spawn(async move {
                let _permiso = permiso;
                let salida = self_.ejecutar_subagente(&tarea).await;
                (i, salida)
            }));
        }

        let mut resultados: Vec<(usize, Result<String>)> = Vec::with_capacity(tareas.len());
        for m in manejos {
            match m.await {
                Ok(par) => resultados.push(par),
                Err(e) => resultados.push((0, Err(anyhow!("Subagente abortado: {e}")))),
            }
        }
        resultados.sort_by_key(|(i, _)| *i);

        let mut informe = String::from("🧩 INFORME DE SUBAGENTES:\n");
        for (i, (_, res)) in resultados.iter().enumerate() {
            informe.push_str(&format!("--- Subagente {} (objetivo: {}) ---\n", i + 1, tareas[i].objetivo));
            match res {
                Ok(texto) => {
                    let texto: String = texto.chars().take(4 * 1024).collect();
                    informe.push_str(&texto);
                    informe.push('\n');
                }
                Err(e) => informe.push_str(&format!("⚠️ ERROR: {e}\n")),
            }
        }
        Ok(informe)
    }

    /// Ejecuta UN subagente como proceso aislado con timeout.
    ///
    /// Toma stdout/stderr ANTES de esperar para poder matar el proceso si
    /// agota el timeout (wait() no consume el Child; permite kill()).
    async fn ejecutar_subagente(&self, tarea: &TareaDelegada) -> Result<String> {
        let mut cmd = tokio::process::Command::new(&self.binario);
        cmd.arg("--subagente")
            .arg(&tarea.objetivo)
            .arg("--contexto")
            .arg(&tarea.contexto)
            .arg("--proveedor")
            .arg(&self.proveedor)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(modelo) = &self.modelo {
            cmd.arg("--modelo").arg(modelo);
        }

        let mut hijo = cmd.spawn().context("No se pudo lanzar el subagente")?;
        let mut stdout = hijo
            .stdout
            .take()
            .context("No se pudo tomar stdout del subagente")?;
        let mut stderr = hijo
            .stderr
            .take()
            .context("No se pudo tomar stderr del subagente")?;

        let espera = tokio::time::timeout(
            Duration::from_secs(self.timeout_seg),
            async {
                let mut out_buf = Vec::new();
                let mut err_buf = Vec::new();
                let (r_out, r_err) =
                    tokio::join!(stdout.read_to_end(&mut out_buf), stderr.read_to_end(&mut err_buf));
                if let Err(e) = r_out {
                    return (Err(anyhow!("Error leyendo stdout: {e}")), Vec::new(), Vec::new());
                }
                if let Err(e) = r_err {
                    return (Err(anyhow!("Error leyendo stderr: {e}")), out_buf, Vec::new());
                }
                let estado = hijo.wait().await;
                (estado.map_err(|e| anyhow!("Error esperando proceso: {e}")), out_buf, err_buf)
            },
        )
        .await;

        match espera {
            Ok((Ok(estado), out, err)) => {
                let mut texto = String::from_utf8_lossy(&out).to_string();
                if !err.is_empty() {
                    texto.push_str(&format!("\n[stderr] {}", String::from_utf8_lossy(&err)));
                }
                if estado.success() {
                    Ok(texto)
                } else {
                    Err(anyhow!(
                        "el subagente terminó con código {:?}: {}",
                        estado.code(),
                        texto.chars().take(500).collect::<String>()
                    ))
                }
            }
            Ok((Err(e), _, _)) => Err(anyhow!("fallo al esperar al subagente: {e}")),
            Err(_) => {
                let _ = hijo.kill().await;
                let _ = hijo.wait().await;
                Err(anyhow!(
                    "el subagente agotó el timeout de {}s",
                    self.timeout_seg
                ))
            }
        }
    }
}

/// Una tarea delegada: objetivo autónomo + contexto aislado.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TareaDelegada {
    pub objetivo: String,
    pub contexto: String,
}

impl TareaDelegada {
    pub fn nueva(objetivo: &str, contexto: &str) -> Self {
        Self { objetivo: objetivo.to_string(), contexto: contexto.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construye_tarea() {
        let t = TareaDelegada::nueva("resumir", "contexto de prueba");
        assert_eq!(t.objetivo, "resumir");
        assert_eq!(t.contexto, "contexto de prueba");
    }

    #[test]
    fn delegador_apunta_al_binario_actual() {
        let d = Delegador::nuevo("ollama", None).unwrap();
        assert!(d.binario.exists());
        assert_eq!(d.max_paralelas, 3);
    }

    #[test]
    fn rechaza_demasiadas_tareas() {
        let d = Delegador::nuevo("ollama", None).unwrap();
        let tareas: Vec<TareaDelegada> = (0..9)
            .map(|i| TareaDelegada::nueva(&format!("t{i}"), ""))
            .collect();
        // delegar es async; probamos el límite vía el método público en runtime
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(d.delegar(&tareas));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Máximo 8"));
    }

    #[test]
    fn lista_vacia_devuelve_mensaje() {
        let d = Delegador::nuevo("ollama", None).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let res = rt.block_on(d.delegar(&[])).unwrap();
        assert!(res.contains("No se delegó ninguna tarea"));
    }
}
