// ============================================================================
// NEXUS-AGENT · ejecutor.rs — Ejecutor Hermes (sandbox de herramientas)
// ============================================================================
// El agente solo actúa sobre el mundo a través de este ejecutor. Todas las
// herramientas (bash, leer archivo, escribir archivo) pasan por un sandbox:
//   - Cota de salida por herramienta (evita respuestas kilométricas)
//   - Cota de tiempo de ejecución de comandos
//   - Restricción opcional de directorio raíz (impide escapar del sandbox)
//   - Lista negra de comandos peligrosos
// ============================================================================

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Configuración del sandbox del ejecutor.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Directorio raíz permitido (None = sin restricción de ruta).
    pub directorio_raiz: Option<PathBuf>,
    /// Límite de salida (bytes) por herramienta.
    pub limite_salida_bytes: usize,
    /// Tiempo máximo (segundos) de un comando bash.
    pub timeout_comando_seg: u64,
    /// Comandos bloqueados por seguridad (nombre del binario).
    pub comandos_bloqueados: Vec<String>,
    /// Directorio de trabajo inicial para comandos bash.
    pub directorio_trabajo: Option<PathBuf>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            directorio_raiz: None,
            limite_salida_bytes: 64 * 1024,
            timeout_comando_seg: 30,
            comandos_bloqueados: vec![
                "rm".into(),
                "shutdown".into(),
                "reboot".into(),
                "mkfs".into(),
                "dd".into(),
            ],
            directorio_trabajo: None,
        }
    }
}

/// Resultado de una herramienta ejecutada por el agente.
#[derive(Debug, Clone)]
pub struct ResultadoHerramienta {
    pub exitoso: bool,
    pub salida: String,
}

impl ResultadoHerramienta {
    pub fn exito(salida: impl Into<String>) -> Self {
        Self { exitoso: true, salida: salida.into() }
    }
    pub fn fallo(salida: impl Into<String>) -> Self {
        Self { exitoso: false, salida: salida.into() }
    }
}

/// El ejecutor que materializa las acciones del agente sobre el sistema.
#[derive(Debug, Clone)]
pub struct EjecutorHermes {
    pub config: SandboxConfig,
}

impl EjecutorHermes {
    pub fn nuevo(config: SandboxConfig) -> Self {
        Self { config }
    }

    /// Ejecuta un comando bash con las cotas del sandbox.
    pub async fn ejecutar_bash(&self, comando: &str) -> Result<ResultadoHerramienta> {
        // Validar lista negra
        if let Some(nombre) = Self::primer_comando(comando) {
            if self.config.comandos_bloqueados.iter().any(|b| b == &nombre) {
                return Ok(ResultadoHerramienta::fallo(format!(
                    "Comando '{}' está en la lista negra del sandbox",
                    nombre
                )));
            }
        }

        let dir = self
            .config
            .directorio_trabajo
            .clone()
            .or_else(|| self.config.directorio_raiz.clone());

        let mut hijo = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(comando)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(dir.unwrap_or_else(|| PathBuf::from(".")))
            .kill_on_drop(true)
            .spawn()
            .context("No se pudo lanzar bash")?;

        // Tomar las tuberías antes de esperar (para poder matar en timeout)
        let mut stdout = hijo
            .stdout
            .take()
            .context("No se pudo tomar la salida estándar")?;
        let mut stderr = hijo
            .stderr
            .take()
            .context("No se pudo tomar la salida de error")?;

        // Esperar con timeout; wait() no consume el Child, permite kill() después.
        let espera = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout_comando_seg),
            async {
                let mut out_buf = Vec::new();
                let mut err_buf = Vec::new();
                let (r_out, r_err) = tokio::join!(
                    stdout.read_to_end(&mut out_buf),
                    stderr.read_to_end(&mut err_buf),
                );
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

        let salida = match espera {
            Ok((Ok(estado), out_buf, err_buf)) => {
                let mut buf = String::new();
                buf.push_str(&String::from_utf8_lossy(&out_buf));
                if !err_buf.is_empty() {
                    buf.push_str(&String::from_utf8_lossy(&err_buf));
                }
                let exitoso = estado.success();
                let truncada = Self::truncar(&buf, self.config.limite_salida_bytes);
                ResultadoHerramienta { exitoso, salida: truncada }
            }
            Ok((Err(e), _, _)) => return Err(e),
            Err(_) => {
                let _ = hijo.kill().await;
                let _ = hijo.wait().await;
                ResultadoHerramienta::fallo(format!(
                    "Comando agotó el timeout de {}s",
                    self.config.timeout_comando_seg
                ))
            }
        };

        Ok(salida)
    }

    /// Lee un archivo (solo dentro del directorio raíz si está definido).
    pub async fn leer_archivo(&self, ruta: &str) -> Result<ResultadoHerramienta> {
        let ruta = self.sancionar_ruta(Path::new(ruta))?;
        match tokio::fs::read_to_string(&ruta).await {
            Ok(contenido) => {
                let truncada = Self::truncar(&contenido, self.config.limite_salida_bytes);
                Ok(ResultadoHerramienta::exito(truncada))
            }
            Err(e) => Ok(ResultadoHerramienta::fallo(format!(
                "No se pudo leer '{}': {e}",
                ruta.display()
            ))),
        }
    }

    /// Escribe contenido en un archivo (crea directorios padre).
    pub async fn escribir_archivo(&self, ruta: &str, contenido: &str) -> Result<ResultadoHerramienta> {
        let ruta = self.sancionar_ruta(Path::new(ruta))?;
        if let Some(padre) = ruta.parent() {
            if !padre.as_os_str().is_empty() {
                tokio::fs::create_dir_all(padre)
                    .await
                    .with_context(|| format!("No se pudo crear '{}'", padre.display()))?;
            }
        }
        let mut f = tokio::fs::File::create(&ruta)
            .await
            .with_context(|| format!("No se pudo crear '{}'", ruta.display()))?;
        f.write_all(contenido.as_bytes())
            .await
            .with_context(|| format!("No se pudo escribir en '{}'", ruta.display()))?;
        Ok(ResultadoHerramienta::exito(format!(
            "Archivo '{}' escrito ({} bytes)",
            ruta.display(),
            contenido.len()
        )))
    }

    /// Lista archivos y carpetas bajo `ruta` (recursivo), con tamaño.
    ///
    /// Patrón absorbido de Hermes (`search_files` modo ficheros): el agente
    /// necesita ver qué hay en el sistema antes de decidir qué tocar. La
    /// salida se recorta al límite de resultados para no inundar el contexto.
    pub async fn listar_archivos(
        &self,
        ruta: &str,
        max_resultados: usize,
    ) -> Result<ResultadoHerramienta> {
        let raiz = self.sancionar_ruta(Path::new(ruta))?;
        if !raiz.is_dir() {
            return Ok(ResultadoHerramienta::fallo(format!(
                "'{}' no es un directorio",
                raiz.display()
            )));
        }
        let mut salida = String::new();
        let mut directorios = 0usize;
        let mut archivos = 0usize;

        for (i, entrada) in walkdir::WalkDir::new(&raiz)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .enumerate()
        {
            if i >= max_resultados {
                salida.push_str(&format!(
                    "\n...[se muestran {max_resultados} de más entradas; usa una ruta más específica]...\n"
                ));
                break;
            }
            let ruta_rel = entrada.path().strip_prefix(&raiz).unwrap_or(entrada.path());
            let prefijo = if entrada.file_type().is_dir() {
                directorios += 1;
                "[dir] "
            } else {
                archivos += 1;
                ""
            };
            let tam = entrada
                .metadata()
                .map(|m| m.len())
                .unwrap_or(0);
            salida.push_str(&format!("{prefijo}{} ({} B)\n", ruta_rel.display(), tam));
        }

        salida.push_str(&format!(
            "— {} archivos, {} carpetas en '{}'",
            archivos,
            directorios,
            raiz.display()
        ));
        Ok(ResultadoHerramienta::exito(salida))
    }

    /// Busca un patrón regex dentro del contenido de archivos (recursivo).
    ///
    /// Patrón absorbido de Hermes (`search_files` modo contenido): devuelve
    /// `ruta:línea: texto`. Respeta un filtro de glob simple (p. ej. "*.rs")
    /// sobre el nombre del archivo, salta binarios grandes y acota el número
    /// de resultados y de archivos leídos para no colgar el agente.
    pub async fn buscar_archivos(
        &self,
        patron: &str,
        ruta: &str,
        glob: Option<&str>,
        max_resultados: usize,
    ) -> Result<ResultadoHerramienta> {
        let regex = match regex::Regex::new(patron) {
            Ok(r) => r,
            Err(e) => {
                return Ok(ResultadoHerramienta::fallo(format!(
                    "Patrón regex inválido '{patron}': {e}"
                )))
            }
        };
        let raiz = self.sancionar_ruta(Path::new(ruta))?;
        if !raiz.is_dir() {
            return Ok(ResultadoHerramienta::fallo(format!(
                "'{}' no es un directorio",
                raiz.display()
            )));
        }

        const MAX_BYTES_ARCHIVO: u64 = 2 * 1024 * 1024; // 2 MiB: binarios fuera
        let mut salida = String::new();
        let mut coincidencias = 0usize;
        let mut archivos_leidos = 0usize;

        for entrada in walkdir::WalkDir::new(&raiz)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if coincidencias >= max_resultados {
                salida.push_str(&format!(
                    "\n...[se muestran {max_resultados} coincidencias; afina el patrón o la ruta]...\n"
                ));
                break;
            }
            if !entrada.file_type().is_file() {
                continue;
            }
            let ruta_archivo = entrada.path();
            // Filtro por glob sobre el nombre del archivo
            if let Some(g) = glob {
                if !Self::glob_coincide(g, ruta_archivo.file_name().and_then(|n| n.to_str()).unwrap_or("")) {
                    continue;
                }
            }
            // Saltar binarios grandes (probable contenido no textual)
            let tam = entrada.metadata().map(|m| m.len()).unwrap_or(0);
            if tam > MAX_BYTES_ARCHIVO {
                continue;
            }
            archivos_leidos += 1;
            let contenido = match std::fs::read(ruta_archivo) {
                Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                Err(_) => continue,
            };
            for (i, linea) in contenido.lines().enumerate() {
                if regex.is_match(linea) {
                    let rel = ruta_archivo
                        .strip_prefix(&raiz)
                        .unwrap_or(ruta_archivo);
                    salida.push_str(&format!(
                        "{}:{}: {}\n",
                        rel.display(),
                        i + 1,
                        Self::recortar_linea(linea, 160)
                    ));
                    coincidencias += 1;
                    if coincidencias >= max_resultados {
                        break;
                    }
                }
            }
        }

        if coincidencias == 0 {
            Ok(ResultadoHerramienta::exito(format!(
                "Sin coincidencias para /{patron}/ en '{}' ({} archivos revisados)",
                raiz.display(),
                archivos_leidos
            )))
        } else {
            salida.push_str(&format!(
                "— {coincidencias} coincidencia(s) en {archivos_leidos} archivo(s) bajo '{}'",
                raiz.display()
            ));
            Ok(ResultadoHerramienta::exito(salida))
        }
    }

    /// Recorta una línea a `max` caracteres (con marca de corte).
    fn recortar_linea(linea: &str, max: usize) -> String {
        if linea.chars().count() <= max {
            linea.to_string()
        } else {
            let t: String = linea.chars().take(max).collect();
            format!("{t}…")
        }
    }

    /// Coincidencia de glob simple: soporta `*` (cualquier secuencia) y `?`
    /// (un carácter) sobre el nombre completo. Implementación propia sin
    /// dependencias, suficiente para filtros tipo "*.rs".
    fn glob_coincide(patron: &str, nombre: &str) -> bool {
        let partes: Vec<&str> = patron.split('*').collect();
        // Caso sin comodines: igualdad exacta
        if partes.len() == 1 {
            let patron = partes[0];
            if let Some(sin_interrogacion) = patron.strip_prefix('?') {
                // "?" al inicio: un carácter cualquiera + el resto
                return nombre.len() > sin_interrogacion.len()
                    && nombre.ends_with(sin_interrogacion);
            }
            return patron == nombre || patron == "*";
        }
        // Caso general: secuencia de prefijos/sufijos separados por '*'
        let mut resto = nombre;
        for (i, parte) in partes.iter().enumerate() {
            if parte.is_empty() {
                continue;
            }
            if i == 0 {
                if !resto.starts_with(parte) {
                    return false;
                }
                resto = &resto[parte.len()..];
            } else if i == partes.len() - 1 {
                return resto.ends_with(parte);
            } else {
                match resto.find(parte) {
                    Some(pos) => resto = &resto[pos + parte.len()..],
                    None => return false,
                }
            }
        }
        true
    }

    // --- Internos del sandbox ---

    /// Asegura que una ruta quede dentro del directorio raíz permitido.
    fn sancionar_ruta(&self, ruta: &Path) -> Result<PathBuf> {
        let ruta = if ruta.is_relative() {
            let base = self
                .config
                .directorio_raiz
                .clone()
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| PathBuf::from("."));
            base.join(ruta)
        } else {
            ruta.to_path_buf()
        };

        if let Some(raiz) = &self.config.directorio_raiz {
            let raiz_canon = raiz
                .canonicalize()
                .unwrap_or_else(|_| raiz.clone());
            let ruta_canon = ruta
                .canonicalize()
                .unwrap_or_else(|_| ruta.clone());
            if !ruta_canon.starts_with(&raiz_canon) {
                return Err(anyhow!(
                    "Ruta '{}' queda fuera del directorio raíz '{}'",
                    ruta.display(),
                    raiz.display()
                ));
            }
        }
        Ok(ruta)
    }

    /// Extrae el primer token del comando (el binario a ejecutar).
    fn primer_comando(comando: &str) -> Option<String> {
        comando
            .split_whitespace()
            .next()
            .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
    }

    /// Trunca la salida al límite de bytes.
    fn truncar(s: &str, limite: usize) -> String {
        if s.len() <= limite {
            s.to_string()
        } else {
            let mut t: String = s.chars().take(limite).collect();
            t.push_str("\n...[salida truncada]...");
            t
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrae_primer_comando() {
        assert_eq!(
            EjecutorHermes::primer_comando("ls -la /tmp").as_deref(),
            Some("ls")
        );
        assert_eq!(EjecutorHermes::primer_comando("  echo hola  ").as_deref(), Some("echo"));
        assert_eq!(EjecutorHermes::primer_comando("").as_deref(), None);
    }

    #[test]
    fn truncar_respeta_limite() {
        let texto = "a".repeat(1000);
        let truncada = EjecutorHermes::truncar(&texto, 100);
        assert!(truncada.len() <= 100 + 40);
        assert!(truncada.contains("truncada"));
    }

    #[test]
    fn truncar_no_toca_cortos() {
        let truncada = EjecutorHermes::truncar("hola", 100);
        assert_eq!(truncada, "hola");
    }

    #[tokio::test]
    async fn lista_negra_bloquea_comando() {
        let ejecutor = EjecutorHermes::nuevo(SandboxConfig::default());
        let res = ejecutor.ejecutar_bash("rm -rf /").await.unwrap();
        assert!(!res.exitoso);
        assert!(res.salida.contains("lista negra"));
    }

    #[tokio::test]
    async fn ejecuta_comando_simple() {
        let ejecutor = EjecutorHermes::nuevo(SandboxConfig::default());
        let res = ejecutor.ejecutar_bash("echo hola-mundo").await.unwrap();
        assert!(res.exitoso);
        assert!(res.salida.contains("hola-mundo"));
    }

    #[tokio::test]
    async fn escribe_y_lee_archivo() {
        let dir = std::env::temp_dir().join(format!("nexus-agent-test-{}", std::process::id()));
        let config = SandboxConfig {
            directorio_raiz: Some(dir.clone()),
            ..Default::default()
        };
        let ejecutor = EjecutorHermes::nuevo(config);
        let ruta = "notas/prueba.txt";
        let escrito = ejecutor.escribir_archivo(ruta, "contenido de prueba").await.unwrap();
        assert!(escrito.exitoso);
        let leido = ejecutor.leer_archivo(ruta).await.unwrap();
        assert!(leido.exitoso);
        assert!(leido.salida.contains("contenido de prueba"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn rechaza_ruta_fuera_de_raiz() {
        let dir = std::env::temp_dir().join(format!("nexus-agent-raiz-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let config = SandboxConfig {
            directorio_raiz: Some(dir.clone()),
            ..Default::default()
        };
        let ejecutor = EjecutorHermes::nuevo(config);
        let res = ejecutor.leer_archivo("/etc/hostname").await;
        assert!(res.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
