// ==========================================
// MOTOR MERCADO — Órgano de Contexto Financiero
// ==========================================
// Órgano del ORQUESTADOR (API-dependent/online).
// Recibe ticks de MarketIngestor, los procesa en
// señales de trading, detecta tendencias y expone
// contexto financiero a la pipeline del Orquestador.
//
// ⚠️ SIN VINCULACIÓN CON engine-puro
// ==========================================

use crate::infra::ingesta_mercado::{MarketIngestor, TickMercado};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Ventana de ticks para análisis de tendencia
const VENTANA_TICKS: usize = 100;

/// Tendencia detectada por el motor
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TendenciaMercado {
    Alcista,
    Bajista,
    Lateral,
    Volatil,
}

impl std::fmt::Display for TendenciaMercado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TendenciaMercado::Alcista => write!(f, "📈 Alcista"),
            TendenciaMercado::Bajista => write!(f, "📉 Bajista"),
            TendenciaMercado::Lateral => write!(f, "➡️ Lateral"),
            TendenciaMercado::Volatil => write!(f, "⚡ Volátil"),
        }
    }
}

/// Señal de trading generada por el motor
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SenalMercado {
    pub simbolo: String,
    /// "COMPRA" | "VENTA" | "NEUTRAL" | "ALERTA"
    pub accion: String,
    pub precio_actual: f64,
    pub confianza: f64,
    pub fundamento: String,
    pub timestamp: u64,
    pub tendencia: TendenciaMercado,
}

/// Resumen del estado del mercado para inyectar en la pipeline
#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextoMercado {
    pub btc: ActivoContexto,
    /// Símbolos adicionales monitoreados (ej: NVDA, ETH)
    pub otros: Vec<ActivoContexto>,
    /// Últimas señales generadas
    pub senales_recientes: Vec<SenalMercado>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ActivoContexto {
    pub simbolo: String,
    pub precio_actual: f64,
    pub cambio_porcentual: f64,
    pub volumen_24h: f64,
    pub tendencia: TendenciaMercado,
    pub sentimiento: String,
}

impl ActivoContexto {
    fn vacio(simbolo: &str) -> Self {
        Self {
            simbolo: simbolo.to_string(),
            precio_actual: 0.0,
            cambio_porcentual: 0.0,
            volumen_24h: 0.0,
            tendencia: TendenciaMercado::Lateral,
            sentimiento: "desconocido".to_string(),
        }
    }
}

/// Ventana deslizante de ticks para un símbolo
struct VentanaTicks {
    ticks: VecDeque<TickMercado>,
    max: usize,
}

impl VentanaTicks {
    fn new(max: usize) -> Self {
        Self {
            ticks: VecDeque::with_capacity(max + 1),
            max,
        }
    }

    fn push(&mut self, tick: TickMercado) {
        if self.ticks.len() >= self.max {
            self.ticks.pop_front();
        }
        self.ticks.push_back(tick);
    }

    fn precio_actual(&self) -> f64 {
        self.ticks.back().map(|t| t.precio).unwrap_or(0.0)
    }

    fn precio_apertura_periodo(&self) -> f64 {
        self.ticks.front().map(|t| t.precio).unwrap_or(0.0)
    }

    fn cambio_porcentual(&self) -> f64 {
        let actual = self.precio_actual();
        let apertura = self.precio_apertura_periodo();
        if apertura > 0.0 {
            ((actual - apertura) / apertura) * 100.0
        } else {
            0.0
        }
    }

    fn volumen_total(&self) -> f64 {
        self.ticks.iter().map(|t| t.volumen).sum()
    }

    fn detectar_tendencia(&self) -> TendenciaMercado {
        let len = self.ticks.len();
        if len < 10 {
            return TendenciaMercado::Lateral;
        }

        // Calcular media móvil simple
        let precios: Vec<f64> = self.ticks.iter().map(|t| t.precio).collect();
        let mitad = len / 2;
        let mma_corta: f64 = precios[mitad..].iter().sum::<f64>() / (len - mitad) as f64;
        let mma_larga: f64 = precios[..mitad].iter().sum::<f64>() / mitad as f64;

        let diff = ((mma_corta - mma_larga) / mma_larga) * 100.0;

        // Volatilidad: desviación estándar de los últimos ticks
        let media = precios.iter().sum::<f64>() / len as f64;
        let varianza: f64 = precios.iter().map(|p| (p - media).powi(2)).sum::<f64>() / len as f64;
        let volatilidad = varianza.sqrt();
        let volatilidad_relativa = if media > 0.0 {
            volatilidad / media * 100.0
        } else {
            0.0
        };

        if volatilidad_relativa > 2.0 {
            return TendenciaMercado::Volatil;
        }

        if diff > 0.5 {
            TendenciaMercado::Alcista
        } else if diff < -0.5 {
            TendenciaMercado::Bajista
        } else {
            TendenciaMercado::Lateral
        }
    }

    fn generar_senal(&self) -> Option<SenalMercado> {
        let len = self.ticks.len();
        if len < 20 {
            return None;
        }

        let tendencia = self.detectar_tendencia();
        let precio = self.precio_actual();
        let cambio = self.cambio_porcentual();

        match tendencia {
            TendenciaMercado::Alcista if cambio > 1.5 => {
                let simbolo = self.ticks[0].simbolo.clone();
                let timestamp = self.ticks.back().map(|t| t.timestamp).unwrap_or(0);
                Some(SenalMercado {
                    simbolo,
                    accion: "COMPRA".to_string(),
                    precio_actual: precio,
                    confianza: (cambio / 5.0).min(1.0),
                    fundamento: format!(
                        "Tendencia alcista detectada con cambio de {:.2}% en ventana de {} ticks",
                        cambio, len
                    ),
                    timestamp,
                    tendencia,
                })
            }
            TendenciaMercado::Bajista if cambio < -1.5 => {
                let simbolo = self.ticks[0].simbolo.clone();
                let timestamp = self.ticks.back().map(|t| t.timestamp).unwrap_or(0);
                Some(SenalMercado {
                    simbolo,
                    accion: "VENTA".to_string(),
                    precio_actual: precio,
                    confianza: (-cambio / 5.0).min(1.0),
                    fundamento: format!(
                        "Tendencia bajista detectada con cambio de {:.2}% en ventana de {} ticks",
                        cambio, len
                    ),
                    timestamp,
                    tendencia,
                })
            }
            TendenciaMercado::Volatil => {
                let simbolo = self.ticks[0].simbolo.clone();
                let timestamp = self.ticks.back().map(|t| t.timestamp).unwrap_or(0);
                Some(SenalMercado {
                    simbolo,
                    accion: "ALERTA".to_string(),
                    precio_actual: precio,
                    confianza: 0.7,
                    fundamento: format!(
                        "Volatilidad alta detectada ({:.2}% de desviación relativa)",
                        cambio.abs()
                    ),
                    timestamp,
                    tendencia,
                })
            }
            _ => None,
        }
    }
}

/// 🧠 Motor Mercado — Órgano de inteligencia financiera del Orquestador
///
/// - Recibe ticks en tiempo real desde [`MarketIngestor`] vía `mpsc`
/// - Mantiene ventanas deslizantes por símbolo
/// - Detecta tendencias (alcista, bajista, lateral, volátil)
/// - Genera señales de trading contextuales
/// - Expone [`ContextoMercado`] para inyección en la pipeline del Orquestador
pub struct MotorMercado {
    /// Receptor de ticks provenientes de MarketIngestor
    rx: mpsc::Receiver<TickMercado>,
    /// Ingestor que captura datos del mercado
    pub ingestor: MarketIngestor,
    /// Ventanas deslizantes por símbolo
    ventanas: std::collections::HashMap<String, VentanaTicks>,
    /// Últimas señales generadas (máx 20)
    senales: VecDeque<SenalMercado>,
    /// Flag de inicialización
    activo: bool,
}

impl MotorMercado {
    /// Crea un nuevo MotorMercado con su propio canal y MarketIngestor
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(200);
        let ingestor = MarketIngestor::new(tx.clone());
        info!("🧠 [MOTOR MERCADO] Órgano de contexto financiero inicializado — esperando ticks del Orquestador");
        Self {
            rx,
            ingestor,
            ventanas: std::collections::HashMap::new(),
            senales: VecDeque::with_capacity(21),
            activo: false,
        }
    }

    /// Inicia la captura de datos del mercado
    pub async fn activar(&mut self) {
        if self.activo {
            warn!("⚠️ [MOTOR MERCADO] Ya está activo — ignorando doble activación");
            return;
        }
        self.ingestor.iniciar_captura().await;
        self.activo = true;
        info!("🧠 [MOTOR MERCADO] Captura de mercado activada");
    }

    /// Procesa ticks pendientes en el canal (llamar desde el event loop del Orquestador)
    pub async fn procesar_pendientes(&mut self) {
        while let Ok(tick) = self.rx.try_recv() {
            self.procesar_tick(tick);
        }
    }

    /// Procesa un tick individual: actualiza ventana y genera señal si corresponde
    fn procesar_tick(&mut self, tick: TickMercado) {
        let simbolo = tick.simbolo.clone();

        let ventana = self
            .ventanas
            .entry(simbolo.clone())
            .or_insert_with(|| VentanaTicks::new(VENTANA_TICKS));

        ventana.push(tick);

        // Generar señal si hay datos suficientes
        if let Some(senal) = ventana.generar_senal() {
            if self.senales.len() >= 20 {
                self.senales.pop_front();
            }
            info!(
                "🧠 [MOTOR MERCADO] Señal generada: {} {} @ ${:.2} (confianza: {:.1}%)",
                senal.accion,
                senal.simbolo,
                senal.precio_actual,
                senal.confianza * 100.0
            );
            self.senales.push_back(senal);
        }
    }

    /// Procesa ticks de forma asíncrona (spawn desde pipeline)
    pub async fn procesar_stream(&mut self) {
        while let Some(tick) = self.rx.recv().await {
            self.procesar_tick(tick);
        }
    }

    /// Obtiene el contexto actual del mercado para inyectar en la pipeline
    pub async fn obtener_contexto(&mut self) -> ContextoMercado {
        self.procesar_pendientes().await;

        let mut otros = Vec::new();

        // Procesar BTC primero (principal)
        let btc = if let Some(ventana) = self.ventanas.get("BTC") {
            ActivoContexto {
                simbolo: "BTC".to_string(),
                precio_actual: ventana.precio_actual(),
                cambio_porcentual: ventana.cambio_porcentual(),
                volumen_24h: ventana.volumen_total(),
                tendencia: ventana.detectar_tendencia(),
                sentimiento: match ventana.detectar_tendencia() {
                    TendenciaMercado::Alcista => "optimista".to_string(),
                    TendenciaMercado::Bajista => "pesimista".to_string(),
                    TendenciaMercado::Lateral => "neutral".to_string(),
                    TendenciaMercado::Volatil => "incierto".to_string(),
                },
            }
        } else {
            ActivoContexto::vacio("BTC")
        };

        // Procesar otros símbolos
        for (simbolo, ventana) in &self.ventanas {
            if simbolo != "BTC" {
                otros.push(ActivoContexto {
                    simbolo: simbolo.clone(),
                    precio_actual: ventana.precio_actual(),
                    cambio_porcentual: ventana.cambio_porcentual(),
                    volumen_24h: ventana.volumen_total(),
                    tendencia: ventana.detectar_tendencia(),
                    sentimiento: match ventana.detectar_tendencia() {
                        TendenciaMercado::Alcista => "optimista".to_string(),
                        TendenciaMercado::Bajista => "pesimista".to_string(),
                        TendenciaMercado::Lateral => "neutral".to_string(),
                        TendenciaMercado::Volatil => "incierto".to_string(),
                    },
                });
            }
        }

        let senales_recientes: Vec<SenalMercado> =
            self.senales.iter().rev().take(5).cloned().collect();

        ContextoMercado {
            btc,
            otros,
            senales_recientes,
        }
    }

    /// Genera texto de contexto financiero para inyectar en prompts del Orquestador
    pub async fn generar_contexto_texto(&mut self) -> String {
        let contexto = self.obtener_contexto().await;

        let mut partes = vec![
            "📊 CONTEXTO FINANCIERO DEL ORQUESTADOR".to_string(),
            "═══════════════════════════════════════".to_string(),
            String::new(),
            format!(
                "• BTC: ${:.2} | Cambio: {:.2}% | Tendencia: {}",
                contexto.btc.precio_actual, contexto.btc.cambio_porcentual, contexto.btc.tendencia
            ),
            format!(
                "• Volumen BTC: {:.2} | Sentimiento: {}",
                contexto.btc.volumen_24h, contexto.btc.sentimiento
            ),
        ];

        for activo in &contexto.otros {
            partes.push(format!(
                "• {}: ${:.2} | Cambio: {:.2}% | Tendencia: {}",
                activo.simbolo, activo.precio_actual, activo.cambio_porcentual, activo.tendencia
            ));
        }

        if !contexto.senales_recientes.is_empty() {
            partes.push(String::new());
            partes.push("📡 SEÑALES RECIENTES:".to_string());
            for senal in &contexto.senales_recientes {
                partes.push(format!(
                    "  {} {} @ ${:.2} (confianza: {:.0}%) — {}",
                    senal.accion,
                    senal.simbolo,
                    senal.precio_actual,
                    senal.confianza * 100.0,
                    senal.fundamento
                ));
            }
        }

        partes.push(String::new());
        partes.push("═ FIN CONTEXTO FINANCIERO ═".to_string());

        partes.join("\n")
    }
}

impl Default for MotorMercado {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ventana_ticks_tendencia_lateral() {
        let mut ventana = VentanaTicks::new(20);
        for i in 0..20 {
            ventana.push(TickMercado {
                simbolo: "BTC".to_string(),
                precio: 50000.0 + (i as f64 * 0.1),
                volumen: 1.0,
                timestamp: i as u64,
            });
        }
        assert_eq!(ventana.detectar_tendencia(), TendenciaMercado::Lateral);
    }

    #[test]
    fn test_ventana_ticks_tendencia_alcista() {
        let mut ventana = VentanaTicks::new(20);
        for i in 0..20 {
            ventana.push(TickMercado {
                simbolo: "BTC".to_string(),
                precio: 50000.0 + (i as f64 * 100.0),
                volumen: 1.0,
                timestamp: i as u64,
            });
        }
        assert_eq!(ventana.detectar_tendencia(), TendenciaMercado::Alcista);
    }

    #[test]
    fn test_ventana_ticks_tendencia_bajista() {
        let mut ventana = VentanaTicks::new(20);
        for i in 0..20 {
            ventana.push(TickMercado {
                simbolo: "BTC".to_string(),
                precio: 52000.0 - (i as f64 * 100.0),
                volumen: 1.0,
                timestamp: i as u64,
            });
        }
        assert_eq!(ventana.detectar_tendencia(), TendenciaMercado::Bajista);
    }

    #[test]
    fn test_motor_mercado_default() {
        let motor = MotorMercado::new();
        assert!(!motor.activo);
        assert!(motor.senales.is_empty());
    }
}
