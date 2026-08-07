use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::error;

const VAULT_PATH: &str = "/home/soberano/NEXUS_ULTIMATE_CORE/.vault/identidades.enc";

/// Vault cifrado con AES-256-GCM para credenciales
pub struct NexusVault {
    cipher: Aes256Gcm,
    cache: Mutex<HashMap<String, String>>,
}

impl NexusVault {
    pub fn new(key: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        Self {
            cipher,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Deriva una clave AES-256 a partir de una contraseña maestra usando SHA-256
    pub fn derivar_clave(master_password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(master_password.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    /// Guarda una credencial cifrada en el vault
    pub fn guardar_credencial(&self, email: &str, password: &str) -> anyhow::Result<()> {
        // Cifrar
        let nonce_bytes = Self::generar_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = format!("{}:{}", email, password);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("AES encrypt: {}", e))?;

        let entry = format!(
            "{}:{}",
            BASE64.encode(nonce_bytes),
            BASE64.encode(ciphertext)
        );

        // Actualizar archivo
        let mut entries = self.cargar_vault()?;
        entries.push(entry);
        self.guardar_vault(&entries)?;

        // Actualizar caché
        let mut cache = self.cache.lock().unwrap();
        cache.insert(email.to_string(), password.to_string());

        Ok(())
    }

    /// Recupera una contraseña del vault por email
    pub fn recuperar_credencial(&self, email: &str) -> anyhow::Result<Option<String>> {
        // Primero buscar en caché
        {
            let cache = self.cache.lock().unwrap();
            if let Some(pwd) = cache.get(email) {
                return Ok(Some(pwd.clone()));
            }
        }

        // Buscar en archivo
        let entries = self.cargar_vault()?;
        for entry in entries {
            let parts: Vec<&str> = entry.split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let nonce_bytes = BASE64
                .decode(parts[0])
                .map_err(|e| anyhow::anyhow!("Base64 decode: {}", e))?;
            let ciphertext = BASE64
                .decode(parts[1])
                .map_err(|e| anyhow::anyhow!("Base64 decode: {}", e))?;

            let nonce = Nonce::from_slice(&nonce_bytes);
            match self.cipher.decrypt(nonce, ciphertext.as_ref()) {
                Ok(plaintext) => {
                    let decrypted = String::from_utf8_lossy(&plaintext);
                    if let Some(stored_email) = decrypted.split(':').next() {
                        if stored_email == email {
                            let pwd = decrypted.split(':').nth(1).unwrap_or("").to_string();
                            // Actualizar caché
                            let mut cache = self.cache.lock().unwrap();
                            cache.insert(email.to_string(), pwd.clone());
                            return Ok(Some(pwd));
                        }
                    }
                }
                Err(e) => {
                    error!("[VAULT] Decrypt error: {}", e);
                }
            }
        }

        Ok(None)
    }

    fn generar_nonce() -> [u8; 12] {
        use rand::RngCore;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    fn cargar_vault(&self) -> anyhow::Result<Vec<String>> {
        let path = std::path::Path::new(VAULT_PATH);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(content
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect())
    }

    fn guardar_vault(&self, entries: &[String]) -> anyhow::Result<()> {
        let path = std::path::Path::new(VAULT_PATH);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = entries.join("\n");
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl std::fmt::Debug for NexusVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NexusVault").finish()
    }
}
