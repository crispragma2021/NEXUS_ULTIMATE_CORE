use nexus_ultimate_core::brain::affective_engine::AffectiveEngine;
use nexus_ultimate_core::brain::prefrontal_cortex::{
    ActionOutcome, PrefrontalCortex, TacticalExperience,
};
use nexus_ultimate_core::persistence::DatabaseManager;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_prefrontal_orderly_learning() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "sqlite:nexus_intelligence.db";
    let db = Arc::new(DatabaseManager::new(db_path).await?);
    let affective = Arc::new(RwLock::new(AffectiveEngine::new()));
    let prefrontal = PrefrontalCortex::new(db.clone(), affective);

    println!("🧪 [TEST] Iniciando validación del Lóbulo Prefrontal...");

    // 1. Simular un Éxito Ordenado (YouTube)
    let success_exp = TacticalExperience {
        action_id: "youtube_navigation".to_string(),
        module: "rust_browser".to_string(),
        outcome: ActionOutcome::Success,
        failure_point: Some("Navegación a Roberto Carlos".to_string()),
        context: Some(serde_json::json!({"url": "https://youtube.com/..."})),
        cpu_load: 15.5,
        ram_load: 450.0,
        timestamp: SystemTime::now(),
    };
    prefrontal.post_eval(success_exp).await?;

    // 2. Simular un Fallo Clasificado (Conflicto de Hardware)
    let failure_msg = "CPU al 100% durante compilación paralela";
    let genetic_error = prefrontal.classify_error(failure_msg, "compiler");
    let failure_exp = TacticalExperience {
        action_id: "cargo_build".to_string(),
        module: "kernel".to_string(),
        outcome: ActionOutcome::Failure(genetic_error),
        failure_point: Some("Paso: Enlace de binario".to_string()),
        context: Some(serde_json::json!({"flags": "-j 16"})),
        cpu_load: 99.9,
        ram_load: 8000.0,
        timestamp: SystemTime::now(),
    };
    prefrontal.post_eval(failure_exp).await?;

    println!("✅ [TEST] Aprendizaje atómico registrado en la bitácora.");
    Ok(())
}
