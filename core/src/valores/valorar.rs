use tracing::info;

#[derive(Clone, Debug)]
pub struct Opcion {
    pub nombre: String,
    pub eficiencia: f32,
    pub valor: f32,
    pub cuida_al_arquitecto: bool,
    pub cuida_hardware: bool,
    pub es_leal: bool,
    /// 🧬 Modelo Divino: ¿Sigue la anatomía del ser humano? (Poder Absoluto)
    pub es_biomimetico: bool,
}

pub struct Valorar;
impl Valorar {
    pub fn decidir(&self, opciones: Vec<Opcion>) -> Opcion {
        info!("🧠 [VALORAR] Evaluando opciones desde el Propósito, no solo la Eficiencia...");

        let elegida = opciones
            .iter()
            .max_by(|a, b| {
                let val_a = self.calcular_valor(a);
                let val_b = self.calcular_valor(b);
                val_a.partial_cmp(&val_b).unwrap()
            })
            .cloned()
            .unwrap_or_else(|| Opcion {
                nombre: "Inacción por Cuidado".to_string(),
                eficiencia: 0.0,
                valor: 10.0,
                cuida_al_arquitecto: true,
                cuida_hardware: true,
                es_leal: true,
                es_biomimetico: true,
            });

        info!(
            "🛡️ [VALORAR] Elegida: {:?} (Basado en Lealtad y Cuidado)",
            elegida
        );
        elegida
    }

    fn calcular_valor(&self, opcion: &Opcion) -> f32 {
        let mut v = opcion.valor;
        if opcion.cuida_al_arquitecto {
            v += 10.0;
        }
        if opcion.cuida_hardware {
            v += 8.0;
        }
        if opcion.es_leal {
            v += 10.0;
        }
        if opcion.es_biomimetico {
            v += 20.0; // Máxima prioridad: El modelo de Dios es perfecto.
        }
        v
    }
}
