// 🌱 NEXUS OMEGA — Almacén de Identidades Cifrado (SQLite + AES-GCM)
// ============================================================

use crate::identities::types::{IdentityStatus, SyntheticIdentity};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

/// Almacenamiento persistente de identidades con cifrado AES-256-GCM
pub struct IdentityStore {
    conn: Mutex<Connection>,
    cipher: Aes256Gcm,
}

impl IdentityStore {
    /// Abre (o crea) la base de datos SQLite e inicializa el cifrado
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Crear tabla si no existe
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS identities (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pool',
                profile_data TEXT NOT NULL,
                fingerprint_data TEXT NOT NULL,
                operation_id TEXT,
                last_used TEXT,
                notes TEXT NOT NULL DEFAULT ''
            );
            CREATE TABLE IF NOT EXISTS email_accounts (
                identity_id TEXT NOT NULL,
                address TEXT NOT NULL,
                password_encrypted TEXT NOT NULL,
                provider TEXT NOT NULL,
                verified INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (identity_id) REFERENCES identities(id)
            );
            CREATE TABLE IF NOT EXISTS phone_accounts (
                identity_id TEXT NOT NULL,
                number TEXT NOT NULL,
                encrypted_data TEXT NOT NULL,
                provider TEXT NOT NULL,
                verified INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (identity_id) REFERENCES identities(id)
            );
            CREATE TABLE IF NOT EXISTS social_accounts (
                identity_id TEXT NOT NULL,
                platform TEXT NOT NULL,
                username TEXT NOT NULL,
                profile_url TEXT NOT NULL,
                verified INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (identity_id) REFERENCES identities(id)
            );",
        )?;

        // Derivar clave de cifrado desde un seed fijo + salt
        let key = derive_encryption_key();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow!("Failed to create AES cipher: {:?}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
            cipher,
        })
    }

    /// Guarda una identidad en la base de datos (cifrando contraseñas)
    pub fn save_identity(&self, identity: &SyntheticIdentity) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex error: {}", e))?;

        let profile_json = serde_json::to_string(&identity.profile)?;
        let fingerprint_json = serde_json::to_string(&identity.fingerprint)?;

        conn.execute(
            "INSERT OR REPLACE INTO identities (id, created_at, status, profile_data, fingerprint_data, operation_id, last_used, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                identity.id.to_string(),
                identity.created_at.to_rfc3339(),
                identity.status.to_string(),
                &profile_json,
                &fingerprint_json,
                identity.operation_id,
                identity.last_used.map(|d| d.to_rfc3339()),
                identity.notes,
            ],
        )?;

        // Guardar correos (contraseñas cifradas)
        for email in &identity.emails {
            let encrypted_pw = self.encrypt(&email.password)?;
            conn.execute(
                "INSERT OR REPLACE INTO email_accounts (identity_id, address, password_encrypted, provider, verified)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    identity.id.to_string(),
                    &email.address,
                    &encrypted_pw,
                    email.provider.to_string(),
                    email.verified as i32,
                ],
            )?;
        }

        // Guardar teléfonos
        for phone in &identity.phones {
            let encrypted_data = self.encrypt(&serde_json::to_string(phone)?)?;
            conn.execute(
                "INSERT OR REPLACE INTO phone_accounts (identity_id, number, encrypted_data, provider, verified)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    identity.id.to_string(),
                    &phone.number,
                    &encrypted_data,
                    phone.provider.to_string(),
                    phone.verified as i32,
                ],
            )?;
        }

        // Guardar redes sociales
        for acct in &identity.accounts {
            conn.execute(
                "INSERT OR REPLACE INTO social_accounts (identity_id, platform, username, profile_url, verified)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    identity.id.to_string(),
                    &acct.platform,
                    &acct.username,
                    &acct.profile_url,
                    acct.verified as i32,
                ],
            )?;
        }

        Ok(())
    }

    /// Lista todas las identidades activas en pool
    pub fn list_identities(
        &self,
        status_filter: Option<IdentityStatus>,
    ) -> Result<Vec<SyntheticIdentity>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex error: {}", e))?;

        let sql = match status_filter {
            Some(ref s) => format!(
                "SELECT id, created_at, status, profile_data, fingerprint_data, operation_id, last_used, notes FROM identities WHERE status = '{}'",
                s
            ),
            None => "SELECT id, created_at, status, profile_data, fingerprint_data, operation_id, last_used, notes FROM identities".to_string(),
        };

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(SyntheticIdentity::load_from_row(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))
        })?;

        let mut identities = Vec::new();
        for row in rows {
            identities.push(row?);
        }

        Ok(identities)
    }

    /// Actualiza el estado de una identidad
    pub fn update_status(&self, id: &str, status: &IdentityStatus) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex error: {}", e))?;
        conn.execute(
            "UPDATE identities SET status = ?1, last_used = ?2 WHERE id = ?3",
            params![status.to_string(), chrono::Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// Elimina permanentemente una identidad y todos sus datos asociados
    pub fn delete_identity(&self, id: &str) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex error: {}", e))?;
        conn.execute(
            "DELETE FROM email_accounts WHERE identity_id = ?1",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM phone_accounts WHERE identity_id = ?1",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM social_accounts WHERE identity_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM identities WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Cuenta total de identidades (por estado opcional)
    pub fn count(&self, status: Option<&IdentityStatus>) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow!("Mutex error: {}", e))?;
        let (sql, status_str) = match status {
            Some(s) => (
                "SELECT COUNT(*) FROM identities WHERE status = ?1",
                Some(s.to_string()),
            ),
            None => ("SELECT COUNT(*) FROM identities", None),
        };
        let mut stmt = conn.prepare(sql)?;
        let count: usize = match status_str {
            Some(ref s) => stmt.query_row(params![s], |row| row.get(0))?,
            None => stmt.query_row([], |row| row.get(0))?,
        };
        Ok(count)
    }

    // ── Cifrado/Descifrado ──────────────────────────────────────

    fn encrypt(&self, plaintext: &str) -> Result<String> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {:?}", e))?;
        // Formato: nonce (12 bytes) + ciphertext
        let mut combined = nonce_bytes.to_vec();
        combined.extend_from_slice(&ciphertext);
        Ok(BASE64.encode(&combined))
    }

    #[allow(dead_code)]
    fn decrypt(&self, encrypted: &str) -> Result<String> {
        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| anyhow!("Base64 decode error: {}", e))?;
        if combined.len() < 12 {
            return Err(anyhow!("Invalid encrypted data"));
        }
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {:?}", e))?;
        Ok(String::from_utf8(plaintext)?)
    }
}

/// Deriva clave AES-256 desde un seed fijo (+ machine-specific salt)
fn derive_encryption_key() -> [u8; 32] {
    use sha2::{Digest, Sha256};

    // Seed base: combinación de hostname + seed fijo
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "nexus-omega".to_string());
    let seed = format!("nexus::identity::store::{}::omega::2025", hostname);

    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}
