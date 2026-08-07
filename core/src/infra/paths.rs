use once_cell::sync::Lazy;
use std::env;
use std::path::{Path, PathBuf};

/// Comprueba si un directorio es la raíz del workspace NEXUS.
/// La raíz REAL tiene `nexus.md`. Un subcrate (ej: `core/`) tiene `Cargo.toml` pero NO `nexus.md`.
fn es_raiz_nexus(dir: &Path) -> bool {
    dir.join("nexus.md").exists()
}

/// La Raíz Neural calcula de forma inteligente dónde está alojado NEXUS
pub static NEXUS_ROOT: Lazy<PathBuf> = Lazy::new(|| {
    // 1. Prioridad Máxima: Variable de entorno explícita (útil para contenedores o custom setups)
    if let Ok(val) = env::var("NEXUS_ROOT") {
        return PathBuf::from(val);
    }

    // 2. Búsqueda Semántica: Buscar hacia arriba partiendo del directorio de ejecución actual
    //    Busca específicamente `nexus.md` (marcador de raíz). Ignora subcrates con solo Cargo.toml.
    if let Ok(mut current) = env::current_dir() {
        loop {
            if es_raiz_nexus(&current) {
                return current;
            }
            if !current.pop() {
                break;
            }
        }
    }

    // 3. Búsqueda por Binario: Buscar en la ubicación del binario compilado
    if let Ok(mut exe_path) = env::current_exe() {
        while exe_path.pop() {
            if es_raiz_nexus(&exe_path) {
                return exe_path;
            }
        }
    }

    // 4. Fallback Seguro por Sistema Operativo
    #[cfg(target_os = "windows")]
    {
        env::var("USERPROFILE")
            .map(|h| PathBuf::from(h).join("NEXUS_ULTIMATE_CORE"))
            .unwrap_or_else(|_| PathBuf::from("C:\\NEXUS_ULTIMATE_CORE"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/soberano".to_string());
        PathBuf::from(home).join("NEXUS_ULTIMATE_CORE")
    }
});

/// Resuelve cualquier ruta relativa de forma segura y portable para el OS actual
pub fn resolve_path<P: AsRef<Path>>(relative: P) -> PathBuf {
    NEXUS_ROOT.join(relative)
}
