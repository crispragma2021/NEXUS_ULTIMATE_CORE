// ============================================================================
// NEXUS-AGENT · memoria_proyecto.rs — Memoria de proyecto (AGENTE.md jerárquico)
// ============================================================================
// Equivalente propio al CLAUDE.md de Claude Code: archivos de instrucciones y
// contexto que el agente carga al inicio para conocer el proyecto.
//
// Jerarquía (de lo más general a lo más específico — lo específico gana):
//   1. Global:   variable de entorno NEXUS_AGENTE_GLOBAL (ruta a un AGENTE.md
//                personal del Arquitecto, fuera del proyecto).
//   2. Proyecto: cada AGENTE.md encontrado al subir desde el cwd hasta la raíz
//                del sandbox (si se define) o la raíz del sistema.
//
// Se fusionan en orden; cada pieza se etiqueta con su ruta de origen para que
// el modelo sepa qué instrucciones vienen de dónde. Nomenclatura propia NEXUS.
// ============================================================================

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Memoria de proyecto cargada desde los AGENTE.md jerárquicos.
#[derive(Debug, Default)]
pub struct MemoriaProyecto {
    /// Piezas en orden de precedencia: (ruta de origen, contenido).
    piezas: Vec<(PathBuf, String)>,
}

impl MemoriaProyecto {
    /// Carga la memoria de proyecto para el directorio de trabajo `cwd`.
    ///
    /// `raiz_sandbox` limita hasta dónde se sube buscando AGENTE.md (si el
    /// agente corre en un sandbox, no debe leer instrucciones fuera de él).
    pub fn cargar(cwd: &Path, raiz_sandbox: Option<&Path>) -> Result<Self> {
        let mut piezas: Vec<(PathBuf, String)> = Vec::new();

        // 1. Pieza global (fuera del proyecto)
        if let Ok(global) = std::env::var("NEXUS_AGENTE_GLOBAL") {
            let ruta = PathBuf::from(global);
            if ruta.is_file() {
                let contenido = std::fs::read_to_string(&ruta).with_context(|| {
                    format!("No se pudo leer el AGENTE.md global {}", ruta.display())
                })?;
                piezas.push((ruta, contenido));
            }
        }

        // 2. Piezas jerárquicas: recoger directorios desde cwd hacia arriba
        let mut dirs: Vec<PathBuf> = Vec::new();
        let mut actual = cwd.to_path_buf();
        loop {
            dirs.push(actual.clone());
            if let Some(raiz) = raiz_sandbox {
                if actual == raiz {
                    break;
                }
            }
            match actual.parent() {
                Some(p) if p != actual => actual = p.to_path_buf(),
                _ => break,
            }
        }
        // De lo más general (arriba) a lo más específico (cwd)
        for dir in dirs.into_iter().rev() {
            let candidato = dir.join("AGENTE.md");
            if candidato.is_file() {
                let contenido = std::fs::read_to_string(&candidato)
                    .with_context(|| format!("No se pudo leer {}", candidato.display()))?;
                piezas.push((candidato, contenido));
            }
        }

        Ok(Self { piezas })
    }

    /// Combina todas las piezas en un bloque de contexto marcado por ruta.
    ///
    /// Devuelve cadena vacía si no hay memoria.
    pub fn fusionar(&self) -> String {
        if self.piezas.is_empty() {
            return String::new();
        }
        let mut out = String::from("# 🧠 Memoria de proyecto (AGENTE.md)\n");
        for (ruta, contenido) in &self.piezas {
            out.push_str(&format!("\n## Desde `{}`\n\n", ruta.display()));
            out.push_str(contenido.trim());
            out.push('\n');
        }
        out
    }

    /// ¿Hay al menos una pieza de memoria cargada?
    pub fn tiene_memoria(&self) -> bool {
        !self.piezas.is_empty()
    }

    /// Piezas cargadas, en orden de precedencia (para inspección y tests).
    pub fn piezas(&self) -> &[(PathBuf, String)] {
        &self.piezas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    /// Serializa los tests que tocan variables de entorno y el sistema de
    /// archivos: `cargar` lee `NEXUS_AGENTE_GLOBAL`, así que todos los tests
    /// deben ejecutarse en exclusión mutua para no leer la pieza global de
    /// otro test que corre en paralelo.
    static LOCK_ENTORNO: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Directorio temporal único por test (evita colisiones entre tests).
    fn dir_temporal() -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "nexus_agente_memoria_{}_{n}",
            std::process::id()
        ))
    }

    fn escribir(ruta: &Path, contenido: &str) {
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(ruta, contenido).unwrap();
    }

    #[test]
    fn carga_jerarquica_desde_carpetas() {
        let _guard = LOCK_ENTORNO.lock().unwrap();
        let raiz = dir_temporal();
        escribir(&raiz.join("AGENTE.md"), "instrucciones de la raíz");
        escribir(&raiz.join("sub/AGENTE.md"), "instrucciones de sub");
        // El cwd es una carpeta más profunda sin AGENTE.md propio
        let cwd = raiz.join("sub/proyecto");

        let memoria = MemoriaProyecto::cargar(&cwd, Some(&raiz)).unwrap();

        assert!(memoria.tiene_memoria());
        let piezas = memoria.piezas();
        assert_eq!(piezas.len(), 2, "se esperan raíz y sub");
        // Orden de precedencia: primero lo más general
        assert_eq!(piezas[0].1, "instrucciones de la raíz");
        assert_eq!(piezas[1].1, "instrucciones de sub");
        // El fusionado incluye ambas piezas con sus rutas
        let fusion = memoria.fusionar();
        assert!(fusion.contains("instrucciones de la raíz"));
        assert!(fusion.contains("instrucciones de sub"));
        assert!(fusion.contains("AGENTE.md"));
        let _ = std::fs::remove_dir_all(&raiz);
    }

    #[test]
    fn carga_vacia_sin_archivos() {
        let _guard = LOCK_ENTORNO.lock().unwrap();
        let dir = dir_temporal();
        let memoria = MemoriaProyecto::cargar(&dir, Some(&dir)).unwrap();
        assert!(!memoria.tiene_memoria());
        assert!(memoria.fusionar().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn global_por_variable_de_entorno() {
        // El acceso a variables de entorno se serializa para no colisionar
        // con otros tests en paralelo.
        let _guard = LOCK_ENTORNO.lock().unwrap();

        let dir = dir_temporal();
        let global = dir.join("AGENTE_GLOBAL.md");
        escribir(&global, "instrucciones personales del Arquitecto");

        std::env::set_var("NEXUS_AGENTE_GLOBAL", &global);
        let memoria = MemoriaProyecto::cargar(&dir, Some(&dir)).unwrap();
        std::env::remove_var("NEXUS_AGENTE_GLOBAL");

        assert!(memoria.tiene_memoria());
        assert!(memoria.fusionar().contains("instrucciones personales"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sandbox_limita_la_subida() {
        let _guard = LOCK_ENTORNO.lock().unwrap();
        let raiz = dir_temporal();
        // AGENTE.md FUERA del sandbox (más arriba que la raíz)
        escribir(&raiz.join("padre/AGENTE.md"), "fuera del sandbox");
        let cwd = raiz.join("padre/sandbox/proyecto");
        // El AGENTE.md dentro del sandbox
        escribir(&cwd.join("AGENTE.md"), "dentro del sandbox");

        // Con raíz del sandbox en padre/sandbox no debe cargar el de padre/
        let sandbox = raiz.join("padre/sandbox");
        let memoria = MemoriaProyecto::cargar(&cwd, Some(&sandbox)).unwrap();
        let piezas = memoria.piezas();
        assert_eq!(piezas.len(), 1, "solo debe cargar el AGENTE.md del sandbox");
        assert_eq!(piezas[0].1, "dentro del sandbox");
        let _ = std::fs::remove_dir_all(&raiz);
    }
}
