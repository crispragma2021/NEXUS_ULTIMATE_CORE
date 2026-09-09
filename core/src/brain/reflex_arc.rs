// ==========================================
// 🧠 REFLEX ARC — Arco reflejo (médula espinal)
// ==========================================
// Señales de reflejo rápidas entre órganos sensoriales y motores.
// ==========================================

/// Señal refleja rápida del sistema nervioso periférico.
#[derive(Debug, Clone, PartialEq)]
pub enum ReflexSignal {
    /// Pico de temperatura detectado.
    HeatSpike(i32),
    /// Cambio propioceptivo (posición corporal).
    ProprioceptiveShift(String),
    /// Señal de socorro / anomalía crítica.
    Distress(String),
}
