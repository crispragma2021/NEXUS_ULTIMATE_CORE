// ==========================================
// REALITY ANCHOR - Anclaje a verdades inmutables
// ==========================================

use crate::infra::policy::RealityAnchor;

pub struct RealityAnchorDaemon {
    pub truths: Vec<String>, // ← HACER PÚBLICO
    threshold: f32,
}

impl RealityAnchorDaemon {
    pub fn new(config: RealityAnchor) -> Self {
        Self {
            truths: config.fundamental_truths,
            threshold: config.semantic_distance_threshold,
        }
    }

    pub async fn measure_distance(&self, statement: &str) -> f32 {
        let mut max_similarity = 0.0;

        for truth in &self.truths {
            let similarity = self.simple_similarity(statement, truth);
            if similarity > max_similarity {
                max_similarity = similarity;
            }
        }

        1.0 - max_similarity
    }

    fn simple_similarity(&self, a: &str, b: &str) -> f32 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
            return 0.9;
        }

        0.0
    }

    pub async fn check_alignment(&self, statement: &str) -> bool {
        let distance = self.measure_distance(statement).await;

        if distance > self.threshold {
            tracing::warn!(
                "⚠️ Afirmación se aleja de la verdad: dist={:.2} > {:.2}",
                distance,
                self.threshold
            );
            return false;
        }

        true
    }

    pub async fn quarantine(&self, statement: &str) {
        tracing::error!("🚨 CUARENTENA ACTIVADA para: {}", statement);
    }
}
