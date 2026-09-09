use std::sync::atomic::{AtomicUsize, Ordering};

/// Cuentas de Google existentes con sus API keys de Gemini
const CUENTAS_EXISTENTES: &[(&str, usize)] = &[
    ("dogperro404@gmail.com", 10),        // 10 keys
    ("lucianiaquino53@gmail.com", 10),    // 10 keys
    ("crispragmatico2021@gmail.com", 10), // 10 keys
    ("divinemercy6321@gmail.com", 1),     // 1 key
];

pub struct GeminiKeyRotator {
    cuentas: Vec<CuentaGemini>,
    index: AtomicUsize,
}

pub struct CuentaGemini {
    pub email: String,
    pub api_keys: Vec<String>,
}

impl GeminiKeyRotator {
    /// Crea un rotador a partir de cuentas y sus listas de API keys
    pub fn new(cuentas: Vec<Vec<String>>) -> Self {
        let cuentas: Vec<CuentaGemini> = cuentas
            .into_iter()
            .map(|c| {
                let email = c.first().cloned().unwrap_or_default();
                let api_keys = c.into_iter().skip(1).collect();
                CuentaGemini { email, api_keys }
            })
            .collect();

        Self {
            cuentas,
            index: AtomicUsize::new(0),
        }
    }

    /// Crea un rotador con las cuentas por defecto
    /// Usa variables de entorno: GEMINI_KEY_0..N, GEMINI_EMAIL_0..N
    pub fn desde_env() -> Self {
        let mut cuentas = Vec::new();

        for (email, num_keys) in CUENTAS_EXISTENTES {
            let mut api_keys = Vec::new();
            for i in 0..*num_keys {
                let var_name = format!(
                    "GEMINI_KEY_{}_{}",
                    email.split('@').next().unwrap_or("nexus"),
                    i
                );
                if let Ok(key) = std::env::var(&var_name) {
                    api_keys.push(key);
                }
            }
            // Si no hay env vars, usar keys por defecto (solo para desarrollo)
            if api_keys.is_empty() {
                for i in 0..*num_keys {
                    let default_key = format!("AIzaSy{}_placeholder_key_{}", &email[..3], i);
                    api_keys.push(default_key);
                }
            }
            cuentas.push(CuentaGemini {
                email: email.to_string(),
                api_keys,
            });
        }

        Self {
            cuentas,
            index: AtomicUsize::new(0),
        }
    }

    /// Obtiene la siguiente API key en rotación round-robin
    pub fn siguiente_key(&self) -> Option<(String, String)> {
        if self.cuentas.is_empty() {
            return None;
        }

        let idx = self.index.fetch_add(1, Ordering::SeqCst);
        let cuenta_idx = idx % self.cuentas.len();
        let cuenta = &self.cuentas[cuenta_idx];

        if cuenta.api_keys.is_empty() {
            return None;
        }

        let key_idx = (idx / self.cuentas.len()) % cuenta.api_keys.len();
        let api_key = cuenta.api_keys[key_idx].clone();

        Some((cuenta.email.clone(), api_key))
    }

    /// Devuelve el estado actual de todas las cuentas
    pub fn estado(&self) -> serde_json::Value {
        let cuentas: Vec<serde_json::Value> = self
            .cuentas
            .iter()
            .map(|c| {
                serde_json::json!({
                    "email": c.email,
                    "keys_disponibles": c.api_keys.len()
                })
            })
            .collect();

        serde_json::json!({
            "cuentas": cuentas,
            "total_cuentas": self.cuentas.len(),
            "total_keys": self.cuentas.iter().map(|c| c.api_keys.len()).sum::<usize>()
        })
    }
}
