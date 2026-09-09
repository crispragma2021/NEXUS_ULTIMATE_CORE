use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct DetectorEstancamiento {
    pub intentos_actuales: u32,
    pub inicio_bucle: Instant,
    pub umbral_intentos: u32,
    pub umbral_tiempo: Duration,
}

impl DetectorEstancamiento {
    pub fn new(intentos: u32, tiempo_secs: u64) -> Self {
        Self {
            intentos_actuales: 0,
            inicio_bucle: Instant::now(),
            umbral_intentos: intentos,
            umbral_tiempo: Duration::from_secs(tiempo_secs),
        }
    }

    pub fn registrar_falla(&mut self) {
        self.intentos_actuales += 1;
    }

    pub fn resetear(&mut self) {
        self.intentos_actuales = 0;
        self.inicio_bucle = Instant::now();
    }

    pub fn esta_atascado(&self) -> bool {
        self.intentos_actuales >= self.umbral_intentos
            || self.inicio_bucle.elapsed() > self.umbral_tiempo
    }
}
