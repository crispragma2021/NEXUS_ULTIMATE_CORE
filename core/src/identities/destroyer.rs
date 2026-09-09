// 🌱 NEXUS OMEGA — Destructor Seguro de Identidades
// ============================================================
// Proceso de destrucción controlada: cierra cuentas, limpia
// perfiles de navegador, elimina rastros.

use crate::identities::browser_profile::BrowserProfileManager;
use crate::identities::storage::IdentityStore;
use crate::identities::types::{IdentityStatus, SyntheticIdentity};
use anyhow::Result;

/// Ejecuta la destrucción segura de una identidad
pub struct IdentityDestroyer {
    store: IdentityStore,
    browser_mgr: BrowserProfileManager,
}

impl IdentityDestroyer {
    pub fn new(store: IdentityStore, browser_mgr: BrowserProfileManager) -> Self {
        Self { store, browser_mgr }
    }

    /// Destruye una identidad: marca como destruida en DB,
    /// elimina perfil de navegador, limpia datos asociados
    pub fn destroy(&self, mut identity: SyntheticIdentity) -> Result<()> {
        println!("💀 Destruyendo identidad: {}", identity.profile.summary());

        // 1. Marcar como destruida en la base de datos
        self.store
            .update_status(&identity.id.to_string(), &IdentityStatus::Destroyed)?;

        // 2. Eliminar perfil de navegador
        if let Err(e) = self.browser_mgr.delete_profile(&identity) {
            eprintln!("⚠️  No se pudo eliminar perfil de navegador: {}", e);
        }

        // 3. Limpiar datos sensibles en memoria
        identity.emails.clear();
        identity.phones.clear();
        identity.accounts.clear();
        identity.notes.clear();

        println!("✅ Identidad destruida: {}", &identity.id.to_string()[..8]);
        Ok(())
    }

    /// Destruye todas las identidades expiradas
    pub fn destroy_expired(&self) -> Result<usize> {
        let expired = self.store.list_identities(Some(IdentityStatus::Expired))?;
        let count = expired.len();

        for identity in expired {
            self.destroy(identity)?;
        }

        println!("💀 {} identidades expiradas destruidas", count);
        Ok(count)
    }

    /// Eliminación física de la fila en DB (post-destrucción)
    pub fn purge(&self, identity: &SyntheticIdentity) -> Result<()> {
        self.store.delete_identity(&identity.id.to_string())?;
        println!(
            "🧹 Identidad purgada de la DB: {}",
            &identity.id.to_string()[..8]
        );
        Ok(())
    }
}
