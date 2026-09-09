// ============================================================================
// NEXUS-AGENT · sesion.rs — Transcripción persistente (JSONL reanudable)
// ============================================================================
// Patrón absorbido de Hermes: la conversación se guarda en disco y se puede
// reanudar en una sesión futura. Cada mensaje del bucle (usuario, asistente,
// instrumento) se anexa a un archivo JSONL — una entrada por línea — de modo
// que NEXUS-Agent recuerde qué hizo y qué decidió sin depender del proveedor.
//
// Diseño propio: escritura append-only (no se reescribe el archivo nunca),
// reanudación con ventana (solo las últimas N entradas vuelven al contexto,
// para no inflar el historial), y tolerancia a entradas corruptas (una línea
// rota no aborta la reanudación; se salta con aviso).
// ============================================================================

use crate::contrato::{MensajeHistoria, RolMensaje};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Una entrada de la transcripción (una línea del JSONL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntradaSesion {
    /// Marca de tiempo epoch (millis) — sin dependencias de cronología.
    pub ts: u64,
    /// Rol del mensaje: "sistema" | "usuario" | "asistente" | "instrumento".
    pub rol: String,
    pub contenido: String,
}

impl EntradaSesion {
    pub fn nueva(rol: &str, contenido: &str) -> Self {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self { ts, rol: rol.to_string(), contenido: contenido.to_string() }
    }
}

/// Transcripción append-only de una sesión.
#[derive(Debug, Clone)]
pub struct Transcripcion {
    ruta: PathBuf,
}

impl Transcripcion {
    /// Crea la transcripción en `ruta` (crea los directorios padre).
    pub fn nueva(ruta: PathBuf) -> Result<Self> {
        if let Some(padre) = ruta.parent() {
            std::fs::create_dir_all(padre)
                .with_context(|| format!("No se pudo crear '{}'", padre.display()))?;
        }
        Ok(Self { ruta })
    }

    /// Ruta del archivo de transcripción (para inspección y tests).
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }

    /// Anexa una entrada al final del JSONL. Los errores de escritura se
    /// devuelven para que el llamador decida (el bucle no debe romperse por
    /// un fallo de transcripción: se registra y se sigue).
    pub fn registrar(&self, rol: &str, contenido: &str) -> Result<()> {
        let linea = serde_json::to_string(&EntradaSesion::nueva(rol, contenido))?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ruta)
            .with_context(|| format!("No se pudo abrir '{}'", self.ruta.display()))?;
        std::io::Write::write_all(&mut f, linea.as_bytes())?;
        std::io::Write::write_all(&mut f, b"\n")?;
        Ok(())
    }

    /// Carga las últimas `max_entradas` entradas como historial reanudable.
    ///
    /// Las entradas "sistema" de la transcripción (instrucción maestra,
    /// resúmenes de compactación) NO se reinyectan: la instrucción maestra
    /// viva ya está en [0] del agente y los resúmenes antiguos solo
    /// ensuciarían el contexto. Las líneas corruptas se saltan con aviso.
    pub fn reanudar(ruta: &Path, max_entradas: usize) -> Result<Vec<MensajeHistoria>> {
        let contenido = std::fs::read_to_string(ruta)
            .with_context(|| format!("No se pudo leer '{}'", ruta.display()))?;
        let mut entradas: Vec<EntradaSesion> = Vec::new();
        for (i, linea) in contenido.lines().enumerate() {
            match serde_json::from_str::<EntradaSesion>(linea) {
                Ok(e) => entradas.push(e),
                Err(e) => eprintln!("⚠️ Sesión: línea {} corrupta, omitida: {e}", i + 1),
            }
        }
        let inicio = entradas.len().saturating_sub(max_entradas);
        Ok(entradas[inicio..]
            .iter()
            .filter_map(|e| match e.rol.as_str() {
                "usuario" => Some(MensajeHistoria::usuario(&e.contenido)),
                "asistente" => Some(MensajeHistoria {
                    rol: RolMensaje::Asistente,
                    contenido: e.contenido.clone(),
                }),
                "instrumento" => Some(MensajeHistoria {
                    rol: RolMensaje::Instrumento,
                    contenido: e.contenido.clone(),
                }),
                // "sistema" se omite (ver doc de la función)
                _ => None,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    fn ruta_temporal() -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "nexus_sesion_{}_{n}.jsonl",
            std::process::id()
        ))
    }

    #[test]
    fn registra_y_reanuda() {
        let ruta = ruta_temporal();
        let t = Transcripcion::nueva(ruta.clone()).unwrap();
        t.registrar("usuario", "hola").unwrap();
        t.registrar("asistente", "hola, Arquitecto").unwrap();
        t.registrar("instrumento", "ok: 42").unwrap();
        t.registrar("sistema", "instrucción maestra").unwrap();

        let historial = Transcripcion::reanudar(&ruta, 100).unwrap();
        // El sistema no se reinyecta
        assert_eq!(historial.len(), 3);
        assert_eq!(historial[0].rol, RolMensaje::Usuario);
        assert_eq!(historial[1].rol, RolMensaje::Asistente);
        assert_eq!(historial[2].rol, RolMensaje::Instrumento);
        assert!(historial.iter().all(|m| !m.contenido.contains("instrucción maestra")));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn ventana_limita_la_reanudacion() {
        let ruta = ruta_temporal();
        let t = Transcripcion::nueva(ruta.clone()).unwrap();
        for i in 0..10 {
            t.registrar("usuario", &format!("mensaje {i}")).unwrap();
        }
        let historial = Transcripcion::reanudar(&ruta, 3).unwrap();
        assert_eq!(historial.len(), 3);
        assert!(historial[0].contenido.contains("mensaje 7"));
        assert!(historial[2].contenido.contains("mensaje 9"));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn tolera_lineas_corruptas() {
        let ruta = ruta_temporal();
        let t = Transcripcion::nueva(ruta.clone()).unwrap();
        t.registrar("usuario", "bueno").unwrap();
        // Línea corrupta inyectada a mano
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&ruta)
            .unwrap();
        std::io::Write::write_all(&mut f, b"{json roto\n").unwrap();
        t.registrar("usuario", "después").unwrap();

        let historial = Transcripcion::reanudar(&ruta, 100).unwrap();
        assert_eq!(historial.len(), 2);
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn no_existe_archivo_devuelve_error() {
        assert!(Transcripcion::reanudar(&ruta_temporal(), 10).is_err());
    }
}
