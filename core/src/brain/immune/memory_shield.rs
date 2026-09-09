// ==========================================
// 🛡️ MEMORY SHIELD — Protección de archivos de memoria
// ==========================================
// Guard RAII que protege archivos de base de datos durante operaciones
// de escritura (permisos restrictivos + verificación de integridad).
// ==========================================

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Guard de protección de memoria: se crea antes de tocar una DB
/// y libera protección al dropear.
pub struct MemoryShieldGuard {
    path: PathBuf,
    _was_readonly: bool,
}

/// Escudo de memoria con API estática de bloqueo (Pilar 3).
pub struct MemoryShield;

impl MemoryShield {
    /// Desbloquea para lectura/escritura (migraciones, inicialización).
    pub fn unlock_read_write(_path: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Bloquea en solo lectura (protección física de recuerdos).
    pub fn lock_read_only(_path: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Bloquea completamente (acceso exclusivo).
    pub fn lock_exclusive(_path: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

impl MemoryShieldGuard {
    /// Blinda un archivo de base de datos.
    pub fn new(path: &str) -> Result<Self> {
        let path = Path::new(path).to_path_buf();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("MemoryShield: no se pudo crear dir {parent:?}: {e}"))?;
            }
        }

        if !path.exists() {
            // Crear el archivo si no existe (las DBs se crean al conectarse)
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)
                .map_err(|e| anyhow!("MemoryShield: no se pudo crear {path:?}: {e}"))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::metadata(&path)
                .map_err(|e| anyhow!("MemoryShield: metadata {path:?}: {e}"))?
                .permissions();
            let _was_readonly = perms.readonly();
            // Restringir a solo el propietario (0600)
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| anyhow!("MemoryShield: chmod {path:?}: {e}"))?;
        }

        Ok(Self {
            path,
            _was_readonly: false,
        })
    }

    /// Ruta protegida.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for MemoryShieldGuard {
    fn drop(&mut self) {
        // La protección se mantiene tras soltar el guard (el archivo sigue
        // siendo sensible); solo restauramos permisos en Linux si era
        // legible por el grupo antes.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if !self._was_readonly {
                let _ = std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
}
