// 🌱 NEXUS OMEGA — Perfiles de Navegador por Identidad
// ============================================================
// Crea directorios de perfil Chromium/Chrome aislados para
// cada identidad, evitando contaminación cruzada de cookies,
// caché, localStorage, etc.

use crate::identities::types::SyntheticIdentity;
use anyhow::Result;
use std::path::PathBuf;

/// Gestiona perfiles de navegador aislados por identidad
pub struct BrowserProfileManager {
    base_dir: PathBuf,
}

impl BrowserProfileManager {
    /// Crea el gestor con directorio base para los perfiles
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Crea el directorio de perfil para una identidad
    pub fn create_profile(&self, identity: &SyntheticIdentity) -> Result<PathBuf> {
        let profile_dir = self.profile_path(identity);
        std::fs::create_dir_all(&profile_dir)?;
        Ok(profile_dir)
    }

    /// Obtiene la ruta del perfil (sin crearlo)
    pub fn profile_path(&self, identity: &SyntheticIdentity) -> PathBuf {
        let id_short = &identity.id.to_string()[..8];
        let safe_name: String = identity
            .profile
            .full_name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        self.base_dir.join(format!("{}_{}", id_short, safe_name))
    }

    /// Lista todos los perfiles existentes
    pub fn list_profiles(&self) -> Result<Vec<PathBuf>> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }
        let mut profiles = Vec::new();
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                profiles.push(entry.path());
            }
        }
        Ok(profiles)
    }

    /// Elimina un perfil de navegador (limpieza de identidad destruida)
    pub fn delete_profile(&self, identity: &SyntheticIdentity) -> Result<()> {
        let path = self.profile_path(identity);
        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }
        Ok(())
    }
}
