//! Plan Phase - Generates adaptation plans from symptoms

use super::{AdaptationPlan, AdaptationType, KnowledgeBase, Symptom, SymptomType};
use crate::error::WorkflowResult;
use crate::snapshots::SnapshotVersioning;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Plan phase generates adaptation strategies
pub struct PlanPhase {
    knowledge: Arc<RwLock<KnowledgeBase>>,
    snapshot_versioning: Arc<SnapshotVersioning>,
}

impl PlanPhase {
    pub fn new(
        knowledge: Arc<RwLock<KnowledgeBase>>,
        snapshot_versioning: Arc<SnapshotVersioning>,
    ) -> Self {
        Self {
            knowledge,
            snapshot_versioning,
        }
    }

    /// Generate adaptation plans from symptoms
    ///
    /// This implements the "Plan" phase of MAPE-K, creating
    /// structured change proposals.
    pub async fn generate_plans(
        &self,
        symptoms: &[Symptom],
    ) -> WorkflowResult<Vec<AdaptationPlan>> {
        let mut plans = Vec::new();

        for symptom in symptoms {
            if let Some(plan) = self.plan_for_symptom(symptom).await? {
                plans.push(plan);
            }
        }

        Ok(plans)
    }

    /// Generate a plan for a specific symptom
    async fn plan_for_symptom(&self, symptom: &Symptom) -> WorkflowResult<Option<AdaptationPlan>> {
        match symptom.symptom_type {
            SymptomType::PerformanceDegradation => {
                Ok(Some(self.plan_performance_optimization(symptom)))
            }
            SymptomType::GuardFailureSpike => Ok(Some(self.plan_guard_relaxation(symptom))),
            SymptomType::UnexpectedBehavior => Ok(Some(self.plan_pattern_adaptation(symptom))),
            _ => Ok(None),
        }
    }

    /// Plan performance optimization
    fn plan_performance_optimization(&self, symptom: &Symptom) -> AdaptationPlan {
        let mut config_delta = HashMap::new();

        // Suggest enabling caching or batch processing
        config_delta.insert("enable_caching".to_string(), "true".to_string());
        config_delta.insert("batch_size".to_string(), "10".to_string());

        AdaptationPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            plan_type: AdaptationType::ConfigurationTuning,
            target_symptom: symptom.symptom_type.clone(),
            sigma_delta: None,
            config_delta: Some(config_delta),
            expected_improvement: 0.3, // Expect 30% improvement
        }
    }

    /// Plan guard relaxation (when guards are failing too often)
    fn plan_guard_relaxation(&self, symptom: &Symptom) -> AdaptationPlan {
        // Propose relaxing guards or adjusting thresholds
        let sigma_delta = r#"
            # Relax guard thresholds
            :AmountGuard knhk:maxValue "20000"^^xsd:decimal .
        "#
        .to_string();

        AdaptationPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            plan_type: AdaptationType::GuardModification,
            target_symptom: symptom.symptom_type.clone(),
            sigma_delta: Some(sigma_delta),
            config_delta: None,
            expected_improvement: 0.5, // Expect 50% reduction in failures
        }
    }

    /// Plan pattern adaptation
    fn plan_pattern_adaptation(&self, symptom: &Symptom) -> AdaptationPlan {
        // Propose pattern change (e.g., parallel split instead of sequence)
        let sigma_delta = r#"
            # Change to parallel pattern for better throughput
            :ProcessTask yawl:pattern "ParallelSplit" .
        "#
        .to_string();

        AdaptationPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            plan_type: AdaptationType::PatternChange,
            target_symptom: symptom.symptom_type.clone(),
            sigma_delta: Some(sigma_delta),
            config_delta: None,
            expected_improvement: 0.4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_plans_for_symptoms() {
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
        let snapshot_versioning = Arc::new(SnapshotVersioning::new("./test_snapshots"));
        let plan_phase = PlanPhase::new(knowledge, snapshot_versioning);

        let symptom = Symptom {
            symptom_type: SymptomType::PerformanceDegradation,
            severity: 0.3,
            description: "High tick usage".to_string(),
            observations: vec![],
        };

        let plans = plan_phase.generate_plans(&[symptom]).await.unwrap();
        assert!(!plans.is_empty());
        assert_eq!(plans[0].plan_type, AdaptationType::ConfigurationTuning);
    }
}
