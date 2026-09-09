// ==========================================
// HIPOTÁLAMO - Regulación homeostática
// ==========================================
// Mantiene el equilibrio interno: temperatura, hambre de cómputo, sed de datos.
// Conecta con el sistema de defensa (homeostasis) y energía (tokens).
// ==========================================
// IMPLEMENTACIÓN SUPERIOR desde brain/hypothalamus.rs
// Reemplaza la versión stub anterior.
// Incluye control térmico real via RyzenAdj + ritmo circadiano.
// ==========================================

use crate::brain::reflex_arc::ReflexSignal;
use crate::cerebro::organos::talamo::Talamo;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Señales de homeostasis que el Hipotálamo emite al sistema
pub enum HomeostasisSignal {
    Normal,
    AlertaCalor,
    HambreTokens,
    SedDatos,
    Emergencia,
}

/// Hipotálamo: El Termostato y Reloj Maestro de NEXUS
/// Encargado de la homeostasis térmica y rítmica.
pub struct Hipotálamo {
    ryzenadj_path: String,
    reflex_tx: mpsc::Sender<ReflexSignal>,
    thalamus: Arc<Talamo>,
}

impl Hipotálamo {
    pub fn new(reflex_tx: mpsc::Sender<ReflexSignal>, thalamus: Arc<Talamo>) -> Self {
        Self {
            ryzenadj_path: "/opt/NEXUS_ULTIMATE_CORE/arsenal_hardware/RyzenAdj/ryzenadj"
                .to_string(),
            reflex_tx,
            thalamus,
        }
    }

    /// Evalúa la fiebre sistémica y orquesta acciones termogénicas.
    pub async fn regular_temperatura(&self, temp: f32) -> anyhow::Result<()> {
        if temp > 90.0 {
            println!(
                "🚨 [HIPOTÁLAMO] FIEBRE CRÍTICA: {}°C. Activando ENFRIAMIENTO DE EMERGENCIA.",
                temp
            );
            self.enfriamiento_emergencia().await?;
            let _ = self
                .reflex_tx
                .send(ReflexSignal::HeatSpike(temp as i32))
                .await;
        } else if temp > 85.0 {
            println!(
                "⚠️ [HIPOTÁLAMO] Fiebre Moderada: {}°C. Reduciendo TDP a nivel seguro.",
                temp
            );
            self.modo_frio().await?;
        } else if temp < 70.0 {
            self.modo_balanceado().await?;
        }
        Ok(())
    }

    async fn enfriamiento_emergencia(&self) -> anyhow::Result<()> {
        Command::new("sudo")
            .arg(&self.ryzenadj_path)
            .args([
                "--stapm-limit=10000",
                "--fast-limit=12000",
                "--slow-limit=10000",
                "--tctl-temp=70",
            ])
            .output()?;
        Ok(())
    }

    async fn modo_frio(&self) -> anyhow::Result<()> {
        Command::new("sudo")
            .arg(&self.ryzenadj_path)
            .args([
                "--stapm-limit=15000",
                "--fast-limit=18000",
                "--slow-limit=15000",
                "--tctl-temp=75",
            ])
            .output()?;
        Ok(())
    }

    async fn modo_balanceado(&self) -> anyhow::Result<()> {
        Command::new("sudo")
            .arg(&self.ryzenadj_path)
            .args([
                "--stapm-limit=25000",
                "--fast-limit=30000",
                "--slow-limit=25000",
                "--tctl-temp=85",
            ])
            .output()?;
        Ok(())
    }

    /// 🧬 SINTONIZACIÓN CIRCADIANA: El cerebro arranca al recibir "Luz"
    pub async fn start_circadian_loop(
        self: Arc<Self>,
        mut eye_rx: mpsc::Receiver<crate::brain::vision::HypothalamusSignal>,
    ) {
        println!("🧠 [HIPOTÁLAMO] Reloj Maestro en línea. Esperando sincronización fotofísica...");

        while let Some(signal) = eye_rx.recv().await {
            match signal {
                crate::brain::vision::HypothalamusSignal::BlueLightLevel(level) => {
                    if level > 0.7 {
                        if self.thalamus.estado()
                            == crate::cerebro::organos::talamo::EstadoConsciencia::Chill
                        {
                            println!("🌅 [HIPOTÁLAMO] Luz azul detectada ({:.2}). Despertando metabolismo (FOCUS).", level);
                            self.thalamus.cambiar_estado(
                                crate::cerebro::organos::talamo::EstadoConsciencia::Focus,
                            );
                            let _ = self.modo_balanceado().await;
                        }
                    } else if level < 0.2
                        && self.thalamus.estado()
                            == crate::cerebro::organos::talamo::EstadoConsciencia::Activo
                    {
                        println!("🌙 [HIPOTÁLAMO] Oscuridad detectada ({:.2}). Induciendo reparación (CHILL).", level);
                        self.thalamus.cambiar_estado(
                            crate::cerebro::organos::talamo::EstadoConsciencia::Chill,
                        );
                        let _ = self.modo_frio().await;
                    }
                }
                crate::brain::vision::HypothalamusSignal::ThreatAlert(threat) => {
                    println!(
                        "🚨 [HIPOTÁLAMO] Pico de Cortisol Digital: Amenaza detectada (Nivel {}).",
                        threat
                    );
                }
            }
        }
    }
}

// ─── ALIAS DE COMPATIBILIDAD ──────────────────────────────────────────────
pub use self::Hipotálamo as Hypothalamus;
