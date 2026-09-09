use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::info;

// ============================================================================
// ROTADOR DE API KEYS GEMINI — Integración con 4 cuentas Google
// ============================================================================
// Hereda y mejora legacy/nexus-orquestador/src/sinapsis_gemini.rs
// Las 4 cuentas existentes:
//   CÉLULA 1: dogperro404@gmail.com (10/10 keys)
//   CÉLULA 2: lucianiaquino53@gmail.com (10/10 keys)
//   CÉLULA 3: crispragmatico2021@gmail.com (10/10 keys)
//   CÉLULA 4: divinemercy6321@gmail.com (1/10 key)
// ============================================================================

const CUENTAS_EXISTENTES: &[(&str, usize)] = &[
    ("dogperro404@gmail.com", 10),
    ("lucianiaquino53@gmail.com", 10),
    ("crispragmatico2021@gmail.com", 10),
    ("divinemercy6321@gmail.com", 1),
];

pub struct GeminiKeyRotator {
    /// Cuentas organizadas: Vec<cuenta: Vec<api_keys>>
    cuentas: Vec<CuentaGemini>,
    cuenta_actual: AtomicUsize,
    key_actual: AtomicUsize,
    total_keys: usize,
}

#[derive(Debug, Clone)]
pub struct CuentaGemini {
    pub email: String,
    pub keys: Vec<String>,
}

impl GeminiKeyRotator {
    pub fn new(cuentas: Vec<Vec<String>>) -> Self {
        let total: usize = cuentas.iter().map(|c| c.len()).sum();
        let cuentas_estructura: Vec<CuentaGemini> = cuentas
            .into_iter()
            .enumerate()
            .map(|(i, keys)| {
                let email = CUENTAS_EXISTENTES
                    .get(i)
                    .map(|(e, _)| e.to_string())
                    .unwrap_or_else(|| format!("cuenta{}@gmail.com", i + 1));
                CuentaGemini { email, keys }
            })
            .collect();

        info!(
            "🔄 [GEMINI] Rotator inicializado: {} cuentas, {} keys totales",
            cuentas_estructura.len(),
            total
        );

        Self {
            cuentas: cuentas_estructura,
            cuenta_actual: AtomicUsize::new(0),
            key_actual: AtomicUsize::new(0),
            total_keys: total,
        }
    }

    /// Crea un rotador con las cuentas por defecto desde variables de entorno
    pub fn desde_env() -> Self {
        // En producción, las keys vienen de .env
        // Esta es una estructura placeholder
        let cuentas = vec![
            vec![std::env::var("GEMINI_API_KEY_1").unwrap_or_default()],
            vec![std::env::var("GEMINI_API_KEY_2").unwrap_or_default()],
        ];
        Self::new(cuentas)
    }

    /// Obtiene la siguiente API key en rotación round-robin
    pub fn siguiente_key(&self) -> Option<(String, String)> {
        if self.total_keys == 0 {
            return None;
        }

        let c_idx = self.cuenta_actual.load(Ordering::SeqCst) % self.cuentas.len();
        let cuenta = &self.cuentas[c_idx];

        if cuenta.keys.is_empty() {
            self.cuenta_actual.fetch_add(1, Ordering::SeqCst);
            return self.siguiente_key();
        }

        let k_idx = self.key_actual.load(Ordering::SeqCst) % cuenta.keys.len();
        let key = cuenta.keys[k_idx].clone();

        // Avanzar contadores
        self.key_actual.fetch_add(1, Ordering::SeqCst);
        if k_idx + 1 >= cuenta.keys.len() {
            self.cuenta_actual.fetch_add(1, Ordering::SeqCst);
        }

        Some((cuenta.email.clone(), key))
    }

    /// Reporta el estado actual del pool de keys
    pub fn estado(&self) -> serde_json::Value {
        let cuentas: Vec<serde_json::Value> = self
            .cuentas
            .iter()
            .map(|c| {
                serde_json::json!({
                    "email": c.email,
                    "keys": c.keys.len(),
                    "activa": c.keys.iter().any(|k| !k.is_empty())
                })
            })
            .collect();

        serde_json::json!({
            "total_cuentas": self.cuentas.len(),
            "total_keys": self.total_keys,
            "cuentas": cuentas
        })
    }

    pub fn cantidad_cuentas(&self) -> usize {
        self.cuentas.len()
    }

    pub fn cantidad_keys(&self) -> usize {
        self.total_keys
    }
}
