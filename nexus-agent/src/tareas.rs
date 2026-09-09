// ============================================================================
// NEXUS-AGENT · tareas.rs — Lista de tareas persistente (patrón todo)
// ============================================================================
// Patrón absorbido de Hermes: lista de tareas con estado que el agente
// mantiene entre ciclos y entre sesiones. Persistencia en JSON (un archivo
// por directorio de datos), escritura atómica (tmp + rename), IDs
// incrementales. Herramientas: todo_agregar, todo_listar, todo_completar,
// todo_quitar.
// ============================================================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Una tarea de la lista.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tarea {
    pub id: u64,
    pub descripcion: String,
    /// "pendiente" | "completada"
    pub estado: String,
    /// Epoch millis de creación.
    pub creado: u64,
}

/// Lista de tareas cargada en memoria y persistida en disco.
#[derive(Debug, Clone, Default)]
pub struct ListaTareas {
    ruta: PathBuf,
    tareas: Vec<Tarea>,
}

impl ListaTareas {
    /// Carga la lista desde `ruta`. Si el archivo no existe, arranca vacía.
    /// Un archivo corrupto no aborta: se avisa y se arranca vacío.
    pub fn cargar(ruta: PathBuf) -> Result<Self> {
        let mut lista = Self { ruta, tareas: Vec::new() };
        if lista.ruta.is_file() {
            match std::fs::read_to_string(&lista.ruta) {
                Ok(contenido) => match serde_json::from_str::<Vec<Tarea>>(&contenido) {
                    Ok(tareas) => lista.tareas = tareas,
                    Err(e) => eprintln!(
                        "⚠️ Aviso: '{}' corrupto ({e}); se arranca con lista vacía",
                        lista.ruta.display()
                    ),
                },
                Err(e) => eprintln!(
                    "⚠️ Aviso: no se pudo leer '{}': {e}",
                    lista.ruta.display()
                ),
            }
        }
        Ok(lista)
    }

    /// Añade una tarea pendiente y persiste.
    pub fn agregar(&mut self, descripcion: &str) -> Result<Tarea> {
        let descripcion = descripcion.trim().to_string();
        if descripcion.is_empty() {
            anyhow::bail!("La descripción de la tarea no puede estar vacía");
        }
        let id = self.tareas.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        let creado = epoch_millis();
        let tarea = Tarea { id, descripcion, estado: "pendiente".into(), creado };
        self.tareas.push(tarea.clone());
        self.persistir()?;
        Ok(tarea)
    }

    /// Vista legible de la lista para el modelo (con estado y antigüedad).
    pub fn listar(&self) -> String {
        if self.tareas.is_empty() {
            return "No hay tareas en la lista.".to_string();
        }
        let mut out = String::from("📋 LISTA DE TAREAS:\n");
        for t in &self.tareas {
            let marca = if t.estado == "completada" { "✅" } else { "⬜" };
            out.push_str(&format!("{marca} [{}] {} (creada {})\n", t.id, t.descripcion, t.creado));
        }
        out.push_str(&format!(
            "— {} pendiente(s), {} completada(s)",
            self.tareas.iter().filter(|t| t.estado == "pendiente").count(),
            self.tareas.iter().filter(|t| t.estado == "completada").count()
        ));
        out
    }

    /// Marca una tarea como completada. Error si no existe.
    pub fn completar(&mut self, id: u64) -> Result<()> {
        let tarea = self
            .tareas
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| anyhow::anyhow!("No existe la tarea #{id}"))?;
        if tarea.estado != "completada" {
            tarea.estado = "completada".into();
            self.persistir()?;
        }
        Ok(())
    }

    /// Elimina una tarea. Error si no existe.
    pub fn quitar(&mut self, id: u64) -> Result<()> {
        let antes = self.tareas.len();
        self.tareas.retain(|t| t.id != id);
        if self.tareas.len() == antes {
            anyhow::bail!("No existe la tarea #{id}");
        }
        self.persistir()
    }

    /// Tareas actuales (para inspección y tests).
    pub fn tareas(&self) -> &[Tarea] {
        &self.tareas
    }

    /// Escritura atómica: JSON a un temporal y renombrado.
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

/// Epoch millis actual.
pub fn epoch_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    fn ruta_temporal() -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nexus_tareas_{}_{n}.json", std::process::id()))
    }

    #[test]
    fn agrega_lista_completa_y_quita() {
        let ruta = ruta_temporal();
        let mut lista = ListaTareas::cargar(ruta.clone()).unwrap();
        assert!(lista.listar().contains("No hay tareas"));

        let t1 = lista.agregar("implementar delegación").unwrap();
        let t2 = lista.agregar("revisar PR").unwrap();
        assert_eq!(t1.id, 1);
        assert_eq!(t2.id, 2);

        let vista = lista.listar();
        assert!(vista.contains("implementar delegación"));
        assert!(vista.contains("2 pendiente(s)"));

        lista.completar(1).unwrap();
        assert!(lista.listar().contains("✅"));

        lista.quitar(2).unwrap();
        assert_eq!(lista.tareas().len(), 1);

        assert!(lista.completar(99).is_err());
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn persiste_entre_cargas() {
        let ruta = ruta_temporal();
        {
            let mut lista = ListaTareas::cargar(ruta.clone()).unwrap();
            lista.agregar("tarea persistente").unwrap();
        }
        let lista = ListaTareas::cargar(ruta.clone()).unwrap();
        assert_eq!(lista.tareas().len(), 1);
        assert!(lista.listar().contains("tarea persistente"));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn ids_son_incrementales_tras_recargar() {
        let ruta = ruta_temporal();
        let mut lista = ListaTareas::cargar(ruta.clone()).unwrap();
        lista.agregar("primera").unwrap();
        let t = lista.agregar("segunda").unwrap();
        assert_eq!(t.id, 2);
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn rechaza_descripcion_vacia() {
        let ruta = ruta_temporal();
        let mut lista = ListaTareas::cargar(ruta.clone()).unwrap();
        assert!(lista.agregar("   ").is_err());
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn tolera_archivo_corrupto() {
        let ruta = ruta_temporal();
        std::fs::write(&ruta, "{no es json").unwrap();
        let lista = ListaTareas::cargar(ruta.clone()).unwrap();
        assert!(lista.tareas().is_empty());
        let _ = std::fs::remove_file(&ruta);
    }
}
