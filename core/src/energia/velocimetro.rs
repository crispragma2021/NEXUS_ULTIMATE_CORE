// ==========================================
// VELOCÍMETRO - Medidor Predictivo de Cuotas
// ==========================================
// Restaurado de la era Antigravity.
// Monitorea el saldo de cada API key y cambia
// de llave ANTES de que se agote, evitando errores 429.
// ==========================================

use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

/// Estados posibles de una llave según el Velocímetro.
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoCuota {
    /// Más del 50% disponible. Zona segura.
    Abundante,
    /// Entre 10% y 50%. Zona de precaución.
    Moderada,
    /// Menos del 10%. Cambiar inmediatamente.
    Critica,
    /// Sin cuota. No usar hasta recarga.
    Agotada,
}

/// Información de monitoreo para una API key.
#[derive(Debug, Clone)]
pub struct MedidorCuota {
    pub email: String,
    pub api_key: String,
    pub peticiones_restantes: u32,
    pub peticiones_totales: u32,
    pub porcentaje_restante: f64,
    pub estado: EstadoCuota,
    pub ultima_medicion: Instant,
}

pub struct Velocimetro {
    /// Mapa de email -> MedidorCuota
    pub medidores: HashMap<String, MedidorCuota>,
    /// Cuenta principal (nunca se sacrifica)
    pub cuenta_principal: String,
}

impl Velocimetro {
    /// Crea un nuevo Velocímetro con las 4 cuentas conocidas.
    pub fn new() -> Self {
        info!("⏱ [VELOCÍMETRO] Cargando pool de llaves desde la Bóveda...");

        let cuentas = Self::cargar_llaves_soberanas();

        let mut medidores = HashMap::new();
        for (email, key, total) in cuentas {
            medidores.insert(
                email.to_string(),
                MedidorCuota {
                    email: email.to_string(),
                    api_key: key.to_string(),
                    peticiones_restantes: total,
                    peticiones_totales: total,
                    porcentaje_restante: 100.0,
                    estado: EstadoCuota::Abundante,
                    ultima_medicion: Instant::now(),
                },
            );
        }

        info!(
            "⏱ [VELOCÍMETRO] Inicializado con {} cuentas",
            medidores.len()
        );

        Self {
            medidores,
            cuenta_principal: std::env::var("NEXUS_PRIMARY_ACCOUNT")
                .unwrap_or_else(|_| "admin@nexus.sovereign".to_string()),
        }
    }

    /// Carga las llaves desde variables de entorno o Vault,
    /// evitando que el código fuente contenga secretos.
    fn cargar_llaves_soberanas() -> Vec<(String, String, u32)> {
        // En una implementación final, esto leería de la tabla 'system_secrets'
        // Por ahora, simulamos la carga de las cuentas que detectamos en el .env
        vec![
            ("SOVEREIGN_PRIMARY".to_string(), "ENV_VAR".to_string(), 5000),
            // El combustible de 'Nestor' se carga aquí como una llave genérica
            ("EXTERNAL_FUEL_01".to_string(), "ENV_VAR".to_string(), 5000),
        ]
    }

    /// Estima el consumo basado en el uso reciente.
    /// En una implementación real, consultaría la API de Google Cloud
    /// para obtener el saldo exacto. Esta versión usa heurísticas.
    pub fn medir(&mut self, email: &str, peticiones_hechas: u32) {
        if let Some(medidor) = self.medidores.get_mut(email) {
            medidor.peticiones_restantes = medidor
                .peticiones_restantes
                .saturating_sub(peticiones_hechas);
            medidor.porcentaje_restante =
                (medidor.peticiones_restantes as f64 / medidor.peticiones_totales as f64) * 100.0;
            medidor.ultima_medicion = Instant::now();

            medidor.estado = match medidor.porcentaje_restante {
                p if p <= 0.0 => EstadoCuota::Agotada,
                p if p < 10.0 => EstadoCuota::Critica,
                p if p < 50.0 => EstadoCuota::Moderada,
                _ => EstadoCuota::Abundante,
            };

            info!(
                "⏱ [VELOCÍMETRO] {}: {:.1}% restante ({:?})",
                email, medidor.porcentaje_restante, medidor.estado
            );
        }
    }

    /// Determina cuál es la mejor llave disponible.
    /// Prioriza: Abundante > Moderada > Crítica.
    /// Nunca usa una Agotada.
    /// La cuenta principal solo se usa si es la única disponible.
    pub fn mejor_llave(&self) -> Option<(String, String)> {
        // Buscar la mejor no agotada que no sea la principal (para preservarla)
        let mut candidatas: Vec<&MedidorCuota> = self
            .medidores
            .values()
            .filter(|m| m.estado != EstadoCuota::Agotada)
            .collect();

        if candidatas.is_empty() {
            warn!("⏱ [VELOCÍMETRO] ¡Todas las llaves están agotadas!");
            return None;
        }

        // Ordenar por porcentaje restante (mayor primero)
        candidatas.sort_by(|a, b| {
            b.porcentaje_restante
                .partial_cmp(&a.porcentaje_restante)
                .unwrap()
        });

        // Si hay alguna abundante que no sea la principal, usarla
        for c in &candidatas {
            if c.estado == EstadoCuota::Abundante && c.email != self.cuenta_principal {
                return Some((c.email.clone(), c.api_key.clone()));
            }
        }

        // Si no, usar la mejor disponible
        let mejor = candidatas[0];
        Some((mejor.email.clone(), mejor.api_key.clone()))
    }

    /// Detecta si TODAS las llaves están por debajo del umbral crítico.
    pub fn todas_criticas(&self) -> bool {
        self.medidores
            .values()
            .all(|m| m.estado == EstadoCuota::Critica || m.estado == EstadoCuota::Agotada)
    }

    /// Detecta si TODAS las llaves están agotadas.
    pub fn todas_agotadas(&self) -> bool {
        self.medidores
            .values()
            .all(|m| m.estado == EstadoCuota::Agotada)
    }

    /// Obtiene el estado de todas las llaves para diagnóstico.
    pub fn diagnostico(&self) -> String {
        let mut reporte = String::from("⏱ [VELOCÍMETRO] Estado de Cuotas:\n");
        for (email, medidor) in &self.medidores {
            let marcador = if email == &self.cuenta_principal {
                "⭐ "
            } else {
                "   "
            };
            reporte.push_str(&format!(
                "{}{}: {:.1}% ({:?})\n",
                marcador, email, medidor.porcentaje_restante, medidor.estado
            ));
        }
        reporte
    }
}
