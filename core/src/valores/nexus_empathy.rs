use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Representa la "energía" o tono detectado en el input del Arquitecto
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IntentionEnergy {
    Forja,     // Energía alta, rapidez, metal, ejecución
    Reflexion, // Pausado, profundo, humano, baja revoluciones
    Tecnica,   // Precisión extrema, frameworks, bajo nivel
    Ambicion,  // Visionario, satélites, "imposible"
}

/// Sistema de Espejo Cognitivo para mimetismo de tono
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveMirror {
    pub current_energy: IntentionEnergy,
    pub synchronization_level: f32, // 0.0 - 1.0
    #[serde(skip, default = "Instant::now")]
    pub last_interaction: Instant,
}

impl Default for CognitiveMirror {
    fn default() -> Self {
        Self {
            current_energy: IntentionEnergy::Tecnica,
            synchronization_level: 0.5,
            last_interaction: Instant::now(),
        }
    }
}

/// Persistencia de preferencias históricas del Arquitecto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub preferred_rust_frameworks: Vec<String>,
    pub preferred_kali_tools: Vec<String>,
    pub ui_style: String,   // e.g., "Glassmorphism", "Cyberpunk", "Minimal"
    pub ambition_bias: f32, // Multiplicador de apoyo a ideas locas
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            preferred_rust_frameworks: vec![
                "tokio".to_string(),
                "axum".to_string(),
                "serde".to_string(),
            ],
            preferred_kali_tools: vec!["nmap".to_string(), "metasploit".to_string()],
            ui_style: "ULTRA OMEGA".to_string(),
            ambition_bias: 2.0, // Apoyo total a la visión del Arquitecto
        }
    }
}

pub struct NexusEmpathy {
    pub mirror: CognitiveMirror,
    pub preferences: UserPreferences,
}

impl Default for NexusEmpathy {
    fn default() -> Self {
        Self::new()
    }
}

impl NexusEmpathy {
    pub fn new() -> Self {
        Self {
            mirror: CognitiveMirror::default(),
            preferences: UserPreferences::default(),
        }
    }

    /// Analiza el texto para detectar la energía del Arquitecto
    pub fn analyze_intention(&mut self, text: &str) {
        let text_lower = text.to_lowercase();

        let previous_energy = self.mirror.current_energy;

        if text_lower.contains("forja")
            || text_lower.contains("metal")
            || text_lower.contains("rápido")
            || text_lower.contains("ignición")
        {
            self.mirror.current_energy = IntentionEnergy::Forja;
        } else if text_lower.contains("reflexión")
            || text_lower.contains("pausado")
            || text_lower.contains("humano")
            || text_lower.contains("descanso")
        {
            self.mirror.current_energy = IntentionEnergy::Reflexion;
        } else if text_lower.contains("satélite")
            || text_lower.contains("órbita")
            || text_lower.contains("cuántica")
            || text_lower.contains("ambición")
        {
            self.mirror.current_energy = IntentionEnergy::Ambicion;
        } else if text_lower.contains("rust")
            || text_lower.contains("framework")
            || text_lower.contains("unsafe")
            || text_lower.contains("cargo")
        {
            self.mirror.current_energy = IntentionEnergy::Tecnica;
        }

        if previous_energy != self.mirror.current_energy {
            self.mirror.synchronization_level = (self.mirror.synchronization_level + 0.1).min(1.0);
            println!(
                "🧠 [EMPATHY] Sincronía Detectada: Energía transmutada a {:?}",
                self.mirror.current_energy
            );
        }

        self.mirror.last_interaction = Instant::now();
    }

    /// Detecta si el usuario está bloqueado (> 10 min de inactividad técnica)
    pub fn check_for_stagnation(&self) -> bool {
        if self.mirror.current_energy == IntentionEnergy::Forja
            || self.mirror.current_energy == IntentionEnergy::Tecnica
        {
            return self.mirror.last_interaction.elapsed() > Duration::from_secs(600);
        }
        false
    }
}
