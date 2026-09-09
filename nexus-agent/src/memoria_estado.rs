// ============================================================================
// NEXUS-AGENT · memoria_estado.rs — Memoria de estado entre sesiones
// ============================================================================
// Patrón absorbido de Hermes: hechos duraderos que sobreviven entre sesiones
// y se inyectan al agente en cada arranque. A diferencia de la memoria de
// proyecto (AGENTE.md, instrucciones del repositorio), esta es la memoria del
// PROPIO agente: aprendizajes, preferencias del Arquitecto y decisiones que
// no deben repetirse ni olvidarse.
//
// Diseño propio, coherente con la filosofía NEXUS (solo lo bueno, lo que
// sirve): archivo de texto plano (un hecho por línea, formato `- hecho`),
// poda FIFO por límite de entradas y de caracteres, y escritura append-only
// con reescritura atómica al podar (nunca se corrompe el archivo a medias).
// ============================================================================

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Memoria de estado del agente, cargada en memoria y persistida en disco.
#[derive(Debug, Clone)]
pub struct MemoriaEstado {
    ruta: PathBuf,
    entradas: Vec<String>,
    max_entradas: usize,
    max_chars: usize,
}

impl Default for MemoriaEstado {
    fn default() -> Self {
        Self {
            ruta: PathBuf::new(),
            entradas: Vec::new(),
            max_entradas: 100,
            max_chars: 8_000,
        }
    }
}

impl MemoriaEstado {
    /// Carga la memoria desde `ruta`. Si el archivo no existe, arranca vacía
    /// (la memoria se crea con el primer `recordar`). Un fallo de lectura no
    /// aborta el arranque: se avisa y se continúa con memoria vacía pero con
    /// la ruta conservada (el siguiente `recordar` reintentará la escritura).
    pub fn cargar(ruta: PathBuf) -> Result<Self> {
        let mut memoria = Self { ruta, ..Default::default() };
        if memoria.ruta.is_file() {
            match std::fs::read_to_string(&memoria.ruta) {
                Ok(contenido) => {
                    memoria.entradas = contenido
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect();
                    memoria.podar();
                }
                Err(e) => eprintln!(
                    "⚠️ Aviso: no se pudo leer '{}': {e} (se continúa con memoria vacía)",
                    memoria.ruta.display()
                ),
            }
        }
        Ok(memoria)
    }

    /// Guarda un hecho nuevo (deduplicado contra los existentes) y persiste.
    pub fn recordar(&mut self, hecho: &str) -> Result<()> {
        let hecho = hecho.trim().to_string();
        if hecho.is_empty() {
            return Ok(());
        }
        // Deduplicación: un hecho ya recordado no se repite.
        if self.entradas.iter().any(|e| e == &hecho) {
            return Ok(());
        }
        self.entradas.push(hecho);
        self.podar();
        self.persistir()
    }

    /// Bloque marcado para inyectar en la instrucción maestra.
    pub fn fusionar(&self) -> String {
        if self.entradas.is_empty() {
            return String::new();
        }
        let mut out = String::from("# 🧠 Memoria de estado del agente\n");
        for e in &self.entradas {
            out.push_str(&format!("- {e}\n"));
        }
        out
    }

    /// ¿Hay hechos recordados?
    pub fn tiene_memoria(&self) -> bool {
        !self.entradas.is_empty()
    }

    /// Entradas en orden cronológico (para inspección y tests).
    pub fn entradas(&self) -> &[String] {
        &self.entradas
    }

    /// Poda FIFO: recorta por número de entradas y por presupuesto de chars.
    fn podar(&mut self) {
        // 1. Por cantidad
        if self.entradas.len() > self.max_entradas {
            let exceso = self.entradas.len() - self.max_entradas;
            self.entradas.drain(0..exceso);
        }
        // 2. Por presupuesto de caracteres (lo más nuevo sobrevive)
        let mut total = 0usize;
        let mut desde = self.entradas.len();
        for (i, e) in self.entradas.iter().enumerate().rev() {
            total += e.len() + 2;
            if total > self.max_chars {
                desde = i + 1;
                break;
            }
        }
        if desde < self.entradas.len() {
            self.entradas.drain(0..desde);
        }
    }

    /// Escribe el archivo completo (crea directorios padre).
    fn persistir(&self) -> Result<()> {
        if let Some(padre) = self.ruta.parent() {
            std::fs::create_dir_all(padre)
                .with_context(|| format!("No se pudo crear '{}'", padre.display()))?;
        }
        // Reescritura atómica: escribir a un temporal y renombrar.
        let tmp = self.ruta.with_extension("tmp");
        let mut contenido = String::new();
        for e in &self.entradas {
            contenido.push_str(e);
            contenido.push('\n');
        }
        std::fs::write(&tmp, contenido)
            .with_context(|| format!("No se pudo escribir '{}'", tmp.display()))?;
        std::fs::rename(&tmp, &self.ruta)
            .with_context(|| format!("No se pudo reemplazar '{}'", self.ruta.display()))?;
        Ok(())
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
            "nexus_estado_{}_{n}.md",
            std::process::id()
        ))
    }

    #[test]
    fn arranca_vacia_y_recuerda() {
        let ruta = ruta_temporal();
        let mut m = MemoriaEstado::cargar(ruta.clone()).unwrap();
        assert!(!m.tiene_memoria());

        m.recordar("El Arquitecto prefiere respuestas en español").unwrap();
        m.recordar("El Arquitecto prefiere respuestas en español").unwrap(); // dedup
        assert_eq!(m.entradas().len(), 1);
        assert!(m.fusionar().contains("español"));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn persiste_entre_cargas() {
        let ruta = ruta_temporal();
        {
            let mut m = MemoriaEstado::cargar(ruta.clone()).unwrap();
            m.recordar("hecho uno").unwrap();
            m.recordar("hecho dos").unwrap();
        }
        let m = MemoriaEstado::cargar(ruta.clone()).unwrap();
        assert_eq!(m.entradas().len(), 2);
        assert!(m.fusionar().contains("hecho dos"));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn poda_fifo_por_cantidad() {
        let ruta = ruta_temporal();
        let mut m = MemoriaEstado::cargar(ruta.clone()).unwrap();
        m.max_entradas = 3;
        for i in 0..5 {
            m.recordar(&format!("hecho {i}")).unwrap();
        }
        assert_eq!(m.entradas().len(), 3);
        // Lo más viejo se fue: quedan 2, 3, 4
        assert_eq!(m.entradas()[0], "hecho 2");
        assert_eq!(m.entradas()[2], "hecho 4");
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn poda_por_presupuesto_de_chars() {
        let ruta = ruta_temporal();
        let mut m = MemoriaEstado::cargar(ruta.clone()).unwrap();
        m.max_chars = 30;
        m.recordar("aaaa aaaa aaaa aaaa aaaa").unwrap(); // 24 chars
        m.recordar("bbbb").unwrap();
        let fusion = m.fusionar();
        assert!(fusion.contains("bbbb"));
        assert!(!fusion.contains("aaaa"));
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn ignora_archivo_vacio_o_inexistente() {
        let m = MemoriaEstado::cargar(ruta_temporal()).unwrap();
        assert!(!m.tiene_memoria());
        assert!(m.fusionar().is_empty());
    }
}
