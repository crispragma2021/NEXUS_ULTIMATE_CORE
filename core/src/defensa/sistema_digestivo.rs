// ──────────────────────────────────────────────
// 🍽️ SISTEMA DIGESTIVO — Filtro de herramientas y código
// Estómago → Hígado → Colon
// Migrado desde legacy/nexus-orquestador/src/sistema_digestivo.rs
// ──────────────────────────────────────────────

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Nutriente resultante del análisis estomacal de una herramienta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutrienteTool {
    pub nombre: String,
    pub descripcion: String,
    pub componentes: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Evaluación hepática con decisión de absorción o rechazo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluacionHigado {
    pub alineado_proposito: bool,
    pub valor_nutricional: f32,
    pub compatible: bool,
    pub eficiencia_energetica: f32,
    pub sinergia: f32,
    pub potencial_evolutivo: f32,
    pub decision: DecisionHigado,
    pub razon: String,
}

/// Decisiones del hígado: qué hacer con cada herramienta
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionHigado {
    /// Absorber completamente — herramienta valiosa
    Absorber,
    /// Filtrar parcialmente, extraer lo útil
    Desintoxicar,
    /// Desechar por inservible
    Excretar,
    /// Rechazo inmediato por peligro
    RechazoInmediato,
}

/// Sistema Digestivo: pipeline de análisis estómago → hígado → colon
pub struct SistemaDigestivo;

impl SistemaDigestivo {
    /// 🍽️ ESTÓMAGO: Descompone un tool en sus componentes básicos
    pub async fn estomago_analisis(&self, tool: &str) -> anyhow::Result<NutrienteTool> {
        info!("🍽️ [ESTÓMAGO] Descomponiendo herramienta: {}", tool);

        // Análisis heurístico de componentes
        let line_count = tool.lines().count();
        let import_count = tool.matches("use ").count();
        let fn_count = tool.matches("fn ").count();
        let struct_count = tool.matches("struct ").count();
        let impl_count = tool.matches("impl ").count();

        let nombre = tool
            .lines()
            .find(|l| l.contains("pub struct") || l.contains("pub fn"))
            .map(|l| {
                l.trim()
                    .trim_start_matches("pub struct ")
                    .trim_start_matches("pub fn ")
                    .split('(')
                    .next()
                    .unwrap_or("Desconocido")
                    .to_string()
            })
            .unwrap_or_else(|| "Tool anónimo".to_string());

        let mut componentes = Vec::new();
        if import_count > 0 {
            componentes.push(format!("{} imports", import_count));
        }
        if fn_count > 0 {
            componentes.push(format!("{} funciones", fn_count));
        }
        if struct_count > 0 {
            componentes.push(format!("{} structs", struct_count));
        }
        if impl_count > 0 {
            componentes.push(format!("{} impls", impl_count));
        }
        componentes.push(format!("{} líneas totales", line_count));

        let metadata = serde_json::json!({
            "lineas": line_count,
            "imports": import_count,
            "funciones": fn_count,
            "structs": struct_count,
            "impls": impl_count,
        });

        Ok(NutrienteTool {
            nombre,
            descripcion: format!(
                "Herramienta analizada con {} líneas y {} dependencias",
                line_count, import_count
            ),
            componentes,
            metadata,
        })
    }

    /// 🧪 HÍGADO: Evalúa si un nutriente debe ser absorbido o rechazado
    pub async fn higado_filtrado(&self, nutriente: &NutrienteTool) -> EvaluacionHigado {
        info!("🧪 [HÍGADO] Evaluando '{}'...", nutriente.nombre);

        // Heurísticas de filtrado
        let alineado = !nutriente.nombre.contains("test")
            && !nutriente.nombre.contains("mock")
            && !nutriente.nombre.contains("example");

        let valor = if nutriente.componentes.len() >= 3 {
            0.8
        } else if nutriente.componentes.len() >= 2 {
            0.6
        } else {
            0.3
        };

        let metas = &nutriente.metadata;
        let lineas = metas.get("lineas").and_then(|v| v.as_u64()).unwrap_or(0);
        let funciones = metas.get("funciones").and_then(|v| v.as_u64()).unwrap_or(0);

        let compatible = lineas > 0;
        let eficiencia = if funciones > 0 && lineas > 0 {
            (funciones as f32 / lineas as f32).min(1.0)
        } else {
            0.5
        };

        let potencial = if alineado && valor > 0.5 { 0.7 } else { 0.2 };

        let (decision, razon) = match (alineado, valor) {
            (true, v) if v > 0.7 => (
                DecisionHigado::Absorber,
                "Herramienta alineada y nutritiva".into(),
            ),
            (true, v) if v > 0.4 => (
                DecisionHigado::Desintoxicar,
                "Potencial útil, requiere filtrado".into(),
            ),
            (true, _) => (DecisionHigado::Excretar, "Bajo valor nutricional".into()),
            (false, _) => (
                DecisionHigado::RechazoInmediato,
                "No alineado con el propósito OMEGA".into(),
            ),
        };

        info!("✅ [HÍGADO] Decisión: {:?} — {}", decision, razon);
        EvaluacionHigado {
            alineado_proposito: alineado,
            valor_nutricional: valor,
            compatible,
            eficiencia_energetica: eficiencia,
            sinergia: valor,
            potencial_evolutivo: potencial,
            decision,
            razon,
        }
    }

    /// 💩 COLON: Registra y desecha herramientas inservibles
    pub fn colon_excrecion(&self, nombre: &str, razon: &str) {
        warn!("💩 [COLON] Excretando '{}'. Razón: {}", nombre, razon);
    }

    /// Pipeline completo: estómago → hígado → colon (si aplica)
    pub async fn digerir(&self, tool: &str) -> anyhow::Result<EvaluacionHigado> {
        let nutriente = self.estomago_analisis(tool).await?;
        let evaluacion = self.higado_filtrado(&nutriente).await;

        if evaluacion.decision == DecisionHigado::Excretar
            || evaluacion.decision == DecisionHigado::RechazoInmediato
        {
            self.colon_excrecion(&nutriente.nombre, &evaluacion.razon);
        }

        Ok(evaluacion)
    }
}
