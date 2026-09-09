// ============================================================================
// NEXUS-AGENT · skills.rs — Biblioteca de habilidades (SKILL.md con frontmatter)
// ============================================================================
// Patrón absorbido de Hermes: biblioteca de procedimientos reutilizables con
// metadatos (nombre + descripción) y carga bajo demanda. El agente ve el
// catálogo (nombre → descripción) en su instrucción maestra, elige el skill
// relevante según la tarea y carga su contenido completo con `skill_ver`.
//
// Formato de cada skill (un archivo SKILL.md por carpeta):
//   ---
//   name: mi-skill
//   description: Úsalo cuando la tarea sea sobre X. Hace Y.
//   ---
//   <cuerpo: pasos numerados, comandos exactos, advertencias>
//
// Nomenclatura y diseño propios NEXUS; el frontmatter se parsea a mano
// (sin dependencias YAML) para mantener el cráter de dependencias mínimo.
// ============================================================================

use anyhow::{Context, Result};
use std::path::Path;

/// Un skill cargado desde disco.
#[derive(Debug, Clone)]
pub struct Skill {
    pub nombre: String,
    pub descripcion: String,
    pub contenido: String,
}

/// Biblioteca de skills indexada en memoria.
#[derive(Debug, Default)]
pub struct BibliotecaSkills {
    skills: Vec<Skill>,
}

impl BibliotecaSkills {
    /// Escanea `dir` (recursivo) en busca de archivos `SKILL.md`.
    ///
    /// Un skill sin frontmatter válido se omite con un aviso (no se aborta la
    /// carga por un archivo mal formado). Si el directorio no existe, se
    /// devuelve una biblioteca vacía: el agente funciona sin skills.
    pub fn cargar(dir: &Path) -> Result<Self> {
        let mut skills = Vec::new();
        if !dir.is_dir() {
            return Ok(Self { skills });
        }

        let mut errores: Vec<String> = Vec::new();
        for entrada in walkdir::WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entrada.file_type().is_file() {
                continue;
            }
            let ruta = entrada.path();
            if ruta.file_name().and_then(|n| n.to_str()) != Some("SKILL.md") {
                continue;
            }
            match Self::leer_skill(ruta) {
                Ok(skill) => skills.push(skill),
                Err(e) => errores.push(format!("{}: {e}", ruta.display())),
            }
        }

        if !errores.is_empty() {
            eprintln!("⚠️ Skills omitidos por formato inválido:");
            for err in &errores {
                eprintln!("   - {err}");
            }
        }

        skills.sort_by(|a, b| a.nombre.cmp(&b.nombre));
        Ok(Self { skills })
    }

    /// Parsea un archivo SKILL.md: frontmatter `--- ... ---` + cuerpo.
    fn leer_skill(ruta: &Path) -> Result<Skill> {
        let crudo = std::fs::read_to_string(ruta)
            .with_context(|| format!("No se pudo leer {}", ruta.display()))?;
        let (frontmatter, cuerpo) = Self::separar_frontmatter(&crudo)?;
        let nombre = Self::campo(&frontmatter, "name")
            .ok_or_else(|| anyhow::anyhow!("falta el campo 'name'"))?;
        let descripcion = Self::campo(&frontmatter, "description")
            .ok_or_else(|| anyhow::anyhow!("falta el campo 'description'"))?;
        Ok(Skill {
            nombre,
            descripcion,
            contenido: cuerpo.trim().to_string(),
        })
    }

    /// Separa `---\n...\n---\n<cuerpo>` en (frontmatter, cuerpo).
    fn separar_frontmatter(crudo: &str) -> Result<(String, String)> {
        let resto = crudo.strip_prefix("---").ok_or_else(|| {
            anyhow::anyhow!("debe comenzar con la línea '---' (frontmatter)")
        })?;
        let fin = resto.find("\n---").ok_or_else(|| {
            anyhow::anyhow!("frontmatter sin cerrar (falta '---' de cierre)")
        })?;
        let frontmatter = resto[..fin].to_string();
        let cuerpo = resto[fin + 4..].to_string();
        Ok((frontmatter, cuerpo))
    }

    /// Extrae `clave: valor` de la primera línea del frontmatter que la tenga.
    fn campo(frontmatter: &str, clave: &str) -> Option<String> {
        frontmatter.lines().find_map(|l| {
            let (k, v) = l.split_once(':')?;
            if k.trim() == clave {
                Some(v.trim().to_string())
            } else {
                None
            }
        })
    }

    /// Catálogo compacto para la instrucción maestra (nombre → descripción).
    pub fn listar(&self) -> String {
        if self.skills.is_empty() {
            return "No hay skills instalados en la biblioteca.".to_string();
        }
        let mut out = String::from("SKILLS DISPONIBLES:\n");
        for s in &self.skills {
            out.push_str(&format!("- {}: {}\n", s.nombre, s.descripcion));
        }
        out
    }

    /// Contenido completo de un skill por nombre.
    pub fn ver(&self, nombre: &str) -> Option<String> {
        self.skills.iter().find(|s| s.nombre == nombre).map(|s| {
            format!(
                "# SKILL: {}\n{}\n---\n{}\n",
                s.nombre, s.descripcion, s.contenido
            )
        })
    }

    /// Skills cuyo nombre o descripción coincide con las palabras clave.
    /// Útil para sugerir al modelo qué skill cargar según la petición.
    pub fn coinciden(&self, consulta: &str) -> Vec<&Skill> {
        let terminos: Vec<String> = consulta
            .split_whitespace()
            .map(|t| t.to_lowercase())
            .filter(|t| t.len() > 2)
            .collect();
        if terminos.is_empty() {
            return Vec::new();
        }
        self.skills
            .iter()
            .filter(|s| {
                let objetivo = format!("{} {}", s.nombre, s.descripcion).to_lowercase();
                terminos.iter().any(|t| objetivo.contains(t.as_str()))
            })
            .collect()
    }

    /// Número de skills cargados (para el arranque y tests).
    pub fn cantidad(&self) -> usize {
        self.skills.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static CONTADOR: AtomicUsize = AtomicUsize::new(0);

    fn dir_temporal() -> PathBuf {
        let n = CONTADOR.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nexus_skills_{}_{n}", std::process::id()))
    }

    fn escribir(ruta: &Path, contenido: &str) {
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(ruta, contenido).unwrap();
    }

    #[test]
    fn parsea_frontmatter_y_cuerpo() {
        let crudo = "---\nname: rust-build\n description:   Úsalo para compilar Rust  \n---\nPaso 1: cargo build\nPaso 2: cargo test\n";
        let (fm, cuerpo) = BibliotecaSkills::separar_frontmatter(crudo).unwrap();
        assert_eq!(BibliotecaSkills::campo(&fm, "name").as_deref(), Some("rust-build"));
        assert_eq!(
            BibliotecaSkills::campo(&fm, "description").as_deref(),
            Some("Úsalo para compilar Rust")
        );
        assert!(cuerpo.contains("cargo build"));
    }

    #[test]
    fn rechaza_skill_sin_frontmatter() {
        let crudo = "sin marcadores ---\nname: x\n";
        assert!(BibliotecaSkills::separar_frontmatter(crudo).is_err());
    }

    #[test]
    fn carga_skills_desde_carpeta() {
        let dir = dir_temporal();
        escribir(
            &dir.join("build/SKILL.md"),
            "---\nname: rust-build\ndescription: Compilar proyectos Rust\n---\n1. cargo build\n2. cargo test\n",
        );
        escribir(
            &dir.join("git/SKILL.md"),
            "---\nname: git-safe\ndescription: Commits seguros sin archivos prohibidos\n---\nNunca hagas git add de .json\n",
        );
        // Archivo que NO es SKILL.md debe ignorarse
        escribir(&dir.join("build/notas.txt"), "basura");

        let biblio = BibliotecaSkills::cargar(&dir).unwrap();
        assert_eq!(biblio.cantidad(), 2);

        let catalogo = biblio.listar();
        assert!(catalogo.contains("rust-build"));
        assert!(catalogo.contains("git-safe"));

        let contenido = biblio.ver("rust-build").unwrap();
        assert!(contenido.contains("cargo build"));
        assert!(biblio.ver("no-existe").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn carga_vacia_si_no_existe_directorio() {
        let biblio = BibliotecaSkills::cargar(&dir_temporal()).unwrap();
        assert_eq!(biblio.cantidad(), 0);
        assert!(biblio.listar().contains("No hay skills"));
    }

    #[test]
    fn coincidencias_por_palabras_clave() {
        let dir = dir_temporal();
        escribir(
            &dir.join("web/SKILL.md"),
            "---\nname: web-search\ndescription: Búsqueda y extracción de contenido web\n---\nbody\n",
        );
        let biblio = BibliotecaSkills::cargar(&dir).unwrap();
        assert_eq!(biblio.coinciden("buscar en la web").len(), 1);
        assert_eq!(biblio.coinciden("compilar").len(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
