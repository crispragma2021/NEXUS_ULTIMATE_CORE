use reqwest::Client;
use std::env;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

pub struct GeminiNativoOmega {
    client: Client,
    pub api_accounts: Vec<Vec<String>>,
    pub cuenta_actual: usize,
    pub llave_en_cuenta: usize,
    pub llaves_agotadas: bool,
    ultimo_uso: Instant,
}

impl GeminiNativoOmega {
    pub async fn new(_unused_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new();
        let mut api_accounts = Vec::new();

        // Cargamos las cuentas según el nuevo formato OMEGA
        for i in 1..=10 {
            if let Ok(keys_str) = env::var(format!("GEMINI_ACCOUNT_{}_KEYS", i)) {
                let keys: Vec<String> = keys_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !keys.is_empty() {
                    api_accounts.push(keys);
                }
            }
        }

        // Compatibilidad si no se ha migrado el .env completamente
        if api_accounts.is_empty() {
            let mut flat_keys = Vec::new();
            for i in 1..=50 {
                if let Ok(key) = env::var(format!("GEMINI_KEY_{}", i)) {
                    flat_keys.push(key);
                }
            }
            if !flat_keys.is_empty() {
                api_accounts.push(flat_keys);
            }
        }

        info!(
            "🔥 [GEMINI OMEGA] Inicializado con {} Células de Energía",
            api_accounts.len()
        );

        Ok(Self {
            client,
            api_accounts,
            cuenta_actual: 0,
            llave_en_cuenta: 0,
            llaves_agotadas: false,
            ultimo_uso: Instant::now(),
        })
    }

    fn rotar_inteligente(&mut self) -> String {
        if self.api_accounts.is_empty() {
            return String::new();
        }

        let total_cuentas = self.api_accounts.len();
        let c_idx = self.cuenta_actual % total_cuentas;

        // Si volvemos a la primera cuenta, avanzamos la llave
        if self.cuenta_actual > 0 && self.cuenta_actual.is_multiple_of(total_cuentas) {
            self.llave_en_cuenta += 1;
        }

        let k_idx = self.llave_en_cuenta % self.api_accounts[c_idx].len();
        self.cuenta_actual += 1;

        self.api_accounts[c_idx][k_idx].clone()
    }

    pub async fn generar(&mut self, prompt: &str) -> Result<String, String> {
        let mut reintentos = 0;
        let max_reintentos = self.api_accounts.iter().map(|a| a.len()).sum::<usize>();

        while reintentos < max_reintentos {
            let api_key = self.rotar_inteligente();
            if api_key.is_empty() {
                break;
            }

            // Control de Rate Limit (mínimo 1s entre ráfagas)
            let espera = Duration::from_millis(1000);
            if self.ultimo_uso.elapsed() < espera {
                sleep(espera - self.ultimo_uso.elapsed()).await;
            }

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash-preview:generateContent?key={}", 
                api_key
            );

            let payload = serde_json::json!({
                "contents": [{"parts": [{"text": prompt}]}],
                "generationConfig": {"temperature": 0.8, "maxOutputTokens": 4096}
            });

            match self.client.post(&url).json(&payload).send().await {
                Ok(response) => {
                    self.ultimo_uso = Instant::now();
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            if let Some(text) =
                                data["candidates"][0]["content"]["parts"][0]["text"].as_str()
                            {
                                return Ok(text.to_string());
                            }
                        }
                    } else if response.status().as_u16() == 429 {
                        warn!(
                            "⚡ [GEMINI OMEGA] 429 en Cuenta {}. Rotando...",
                            (self.cuenta_actual - 1) % self.api_accounts.len() + 1
                        );
                        reintentos += 1;
                        continue;
                    } else {
                        return Err(format!("Error HTTP: {}", response.status()));
                    }
                }
                Err(e) => return Err(format!("Fallo de conexión: {}", e)),
            }
        }

        self.llaves_agotadas = true;
        Err("Todas las Células de Energía están agotadas o restringidas.".to_string())
    }
}
