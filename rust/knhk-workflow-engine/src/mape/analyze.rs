//! Analyze Phase - Detects symptoms from observations

use super::{KnowledgeBase, Observation, Symptom, SymptomType};
use crate::error::WorkflowResult;
use parking_lot::RwLock;
use std::sync::Arc;

/// Analyze phase detects symptoms from collected observations
pub struct AnalyzePhase {
    knowledge: Arc<RwLock<KnowledgeBase>>,
}

impl AnalyzePhase {
    pub fn new(knowledge: Arc<RwLock<KnowledgeBase>>) -> Self {
        Self { knowledge }
    }

    /// Detect symptoms from observations
    ///
    /// This implements the "Analyze" phase of MAPE-K, examining
    /// observations to identify problems or opportunities.
    pub async fn detect_symptoms(
        &self,
        observations: &[Observation],
    ) -> WorkflowResult<Vec<Symptom>> {
        let mut symptoms = Vec::new();

        // Analyze performance degradation (Chatman Constant violations)
        if let Some(symptom) = self.detect_performance_degradation(observations) {
            symptoms.push(symptom);
        }

        // Analyze guard failures
        if let Some(symptom) = self.detect_guard_failures(observations) {
            symptoms.push(symptom);
        }

        // Analyze unexpected behavior
        if let Some(symptom) = self.detect_unexpected_behavior(observations) {
            symptoms.push(symptom);
        }

        Ok(symptoms)
    }

    /// Detect performance degradation (approaching Chatman Constant)
    fn detect_performance_degradation(&self, observations: &[Observation]) -> Option<Symptom> {
        let near_misses: Vec<_> = observations
            .iter()
            .filter(|obs| obs.ticks_used >= 7) // Near the 8-tick limit
            .collect();

        if near_misses.len() > observations.len() / 10 {
            // More than 10% near-misses
            let severity = near_misses.len() as f64 / observations.len() as f64;
            Some(Symptom {
                symptom_type: SymptomType::PerformanceDegradation,
                severity,
                description: format!(
                    "{}% of operations approaching Chatman Constant (≤8 ticks)",
                    (severity * 100.0) as u32
                ),
                observations: near_misses.iter().map(|o| o.receipt_id.clone()).collect(),
            })
        } else {
            None
        }
    }

    /// Detect guard failure spikes
    fn detect_guard_failures(&self, observations: &[Observation]) -> Option<Symptom> {
        let failed_observations: Vec<_> = observations
            .iter()
            .filter(|obs| !obs.guards_failed.is_empty())
            .collect();

        if failed_observations.len() > observations.len() / 20 {
            // More than 5% failures
            let severity = failed_observations.len() as f64 / observations.len() as f64;
            Some(Symptom {
                symptom_type: SymptomType::GuardFailureSpike,
                severity,
                description: format!("Guard failure rate: {}%", (severity * 100.0) as u32),
                observations: failed_observations
                    .iter()
                    .map(|o| o.receipt_id.clone())
                    .collect(),
            })
        } else {
            None
        }
    }

    /// Detect unexpected behavior patterns
    fn detect_unexpected_behavior(&self, observations: &[Observation]) -> Option<Symptom> {
        // Check for unusual sigma version distribution
        let sigma_counts =
            observations
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, obs| {
                    *acc.entry(&obs.sigma_id).or_insert(0) += 1;
                    acc
                });

        // If we have multiple sigma versions with significant usage, that might be unexpected
        if sigma_counts.len() > 2 {
            Some(Symptom {
                symptom_type: SymptomType::UnexpectedBehavior,
                severity: 0.5,
                description: format!(
                    "Multiple active Σ versions detected: {}",
                    sigma_counts.len()
                ),
                observations: observations
                    .iter()
                    .take(10)
                    .map(|o| o.receipt_id.clone())
                    .collect(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_detect_performance_degradation() {
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
        let analyze = AnalyzePhase::new(knowledge);

        // Create observations with high tick usage
        let observations: Vec<Observation> = (0..100)
            .map(|i| Observation {
                receipt_id: format!("receipt-{}", i),
                sigma_id: "sigma-v1".to_string(),
                ticks_used: if i < 15 { 7 } else { 3 }, // 15% near-misses
                guards_checked: vec![],
                guards_failed: vec![],
                timestamp: Utc::now(),
                metrics: std::collections::HashMap::new(),
            })
            .collect();

        let symptoms = analyze.detect_symptoms(&observations).await.unwrap();

        // Should detect performance degradation
        assert!(symptoms
            .iter()
            .any(|s| s.symptom_type == SymptomType::PerformanceDegradation));
    }
}
