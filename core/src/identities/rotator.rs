// 🌱 NEXUS OMEGA — Rotador de Identidades para Operaciones
// ============================================================
// Gestiona el ciclo de vida operativo: rotación, expiración,
// selección de identidad activa del pool.

use crate::identities::storage::IdentityStore;
use crate::identities::types::{IdentityStatus, SyntheticIdentity};
use anyhow::Result;
use rand::Rng;

/// Estrategia de rotación de identidades
pub struct IdentityRotator {
    store: IdentityStore,
}

impl IdentityRotator {
    pub fn new(store: IdentityStore) -> Self {
        Self { store }
    }

    /// Obtiene la siguiente identidad disponible del pool (aleatoria)
    pub fn next_available(&self) -> Result<Option<SyntheticIdentity>> {
        let pool = self.store.list_identities(Some(IdentityStatus::Pool))?;
        if pool.is_empty() {
            return Ok(None);
        }
        let idx = rand::thread_rng().gen_range(0..pool.len());
        Ok(Some(pool[idx].clone()))
    }

    /// Activa una identidad para una operación específica
    pub fn activate(&self, identity: &mut SyntheticIdentity, operation_id: &str) -> Result<()> {
        identity.mark_active(operation_id);
        self.store
            .update_status(&identity.id.to_string(), &IdentityStatus::Active)?;
        Ok(())
    }

    /// Pone una identidad en estado durmiente (disponible pero en pausa)
    pub fn dormir(&self, identity: &SyntheticIdentity) -> Result<()> {
        self.store
            .update_status(&identity.id.to_string(), &IdentityStatus::Dormant)?;
        Ok(())
    }

    /// Reactiva una identidad durmiente
    pub fn reactivar(&self, identity: &mut SyntheticIdentity) -> Result<()> {
        identity.status = IdentityStatus::Pool;
        self.store
            .update_status(&identity.id.to_string(), &IdentityStatus::Pool)?;
        Ok(())
    }

    /// Expira identidades que llevan demasiado tiempo inactivas
    pub fn expire_old(&self, max_days_inactive: i64) -> Result<usize> {
        let all = self.store.list_identities(None)?;
        let now = chrono::Utc::now();
        let mut expired_count = 0;

        for identity in &all {
            if identity.status == IdentityStatus::Destroyed {
                continue;
            }
            let last_used = identity.last_used.unwrap_or(identity.created_at);
            let days_since = (now - last_used).num_days();
            if days_since > max_days_inactive {
                self.store
                    .update_status(&identity.id.to_string(), &IdentityStatus::Expired)?;
                expired_count += 1;
            }
        }

        Ok(expired_count)
    }

    /// Reporta el estado actual del pool
    pub fn pool_report(&self) -> Result<String> {
        let total = self.store.count(None)?;
        let pool = self.store.count(Some(&IdentityStatus::Pool))?;
        let active = self.store.count(Some(&IdentityStatus::Active))?;
        let dormant = self.store.count(Some(&IdentityStatus::Dormant))?;
        let expired = self.store.count(Some(&IdentityStatus::Expired))?;
        let destroyed = self.store.count(Some(&IdentityStatus::Destroyed))?;

        Ok(format!(
            "📊 POOL DE IDENTIDADES
   Total:     {}
   Pool:      {}
   Activas:   {}
   Durmientes: {}
   Expiradas:  {}
   Destruidas: {}",
            total, pool, active, dormant, expired, destroyed
        ))
    }
}
