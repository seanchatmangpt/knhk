//! Execute Phase - Applies adaptation plans via shadow deployment

use super::{AdaptationPlan, AdaptationResult};
use crate::engine::HookEngine;
use crate::error::WorkflowResult;
use crate::guards::InvariantChecker;
use crate::snapshots::SnapshotVersioning;
use std::sync::Arc;

/// Execute phase applies adaptations using shadow deployment strategy
pub struct ExecutePhase {
    snapshot_versioning: Arc<SnapshotVersioning>,
    hook_engine: Arc<HookEngine>,
    invariant_checker: Arc<InvariantChecker>,
}

impl ExecutePhase {
    pub fn new(
        snapshot_versioning: Arc<SnapshotVersioning>,
        hook_engine: Arc<HookEngine>,
        invariant_checker: Arc<InvariantChecker>,
    ) -> Self {
        Self {
            snapshot_versioning,
            hook_engine,
            invariant_checker,
        }
    }

    /// Apply adaptation plans using shadow deployment
    ///
    /// This implements the "Execute" phase of MAPE-K:
    /// 1. Create shadow Σ' with proposed changes
    /// 2. Validate shadow deployment
    /// 3. Gradually promote (A/B test)
    /// 4. Atomic pointer update if successful
    pub async fn apply_adaptations(
        &self,
        plans: Vec<AdaptationPlan>,
    ) -> WorkflowResult<AdaptationResult> {
        let mut adaptations_applied = 0;
        let mut errors = Vec::new();
        let mut new_sigma_id = None;

        for plan in plans {
            match self.apply_single_adaptation(&plan).await {
                Ok(sigma_id) => {
                    adaptations_applied += 1;
                    new_sigma_id = Some(sigma_id);
                }
                Err(e) => {
                    errors.push(format!("Plan {}: {:?}", plan.plan_id, e));
                }
            }
        }

        Ok(AdaptationResult {
            adaptations_applied,
            new_sigma_id,
            errors,
        })
    }

    /// Apply a single adaptation plan
    async fn apply_single_adaptation(&self, plan: &AdaptationPlan) -> WorkflowResult<String> {
        // 1. Create shadow snapshot with proposed changes
        let shadow_id = self
            .snapshot_versioning
            .create_shadow_snapshot(plan.sigma_delta.as_deref(), plan.config_delta.as_ref())?;

        // 2. Validate shadow snapshot
        self.validate_shadow(&shadow_id)?;

        // 3. Run shadow tests (would run actual workflow executions here)
        // For now, we just verify invariants

        // 4. If successful, promote shadow to production
        let new_id = self.snapshot_versioning.promote_shadow(&shadow_id)?;

        tracing::info!(
            "Successfully applied adaptation plan {}: {} → {}",
            plan.plan_id,
            self.snapshot_versioning.current_id(),
            new_id
        );

        Ok(new_id)
    }

    /// Validate shadow snapshot
    fn validate_shadow(&self, shadow_id: &str) -> WorkflowResult<()> {
        // Load shadow snapshot
        let snapshot = self.snapshot_versioning.load_snapshot(shadow_id)?;

        // Validate invariants
        self.invariant_checker.validate_snapshot(&snapshot)?;

        // Additional validation could include:
        // - SHACL shape validation
        // - Pattern permutation checks
        // - Performance simulation

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_applies_adaptations() {
        let snapshot_versioning = Arc::new(SnapshotVersioning::new("./test_snapshots"));
        let hook_engine = Arc::new(HookEngine::new());
        let invariant_checker = Arc::new(InvariantChecker::new());

        let execute = ExecutePhase::new(snapshot_versioning, hook_engine, invariant_checker);

        let plan = AdaptationPlan {
            plan_id: "test-plan".to_string(),
            plan_type: super::super::AdaptationType::ConfigurationTuning,
            target_symptom: super::super::SymptomType::PerformanceDegradation,
            sigma_delta: None,
            config_delta: Some(
                vec![("test_key".to_string(), "test_value".to_string())]
                    .into_iter()
                    .collect(),
            ),
            expected_improvement: 0.3,
        };

        let result = execute.apply_adaptations(vec![plan]).await.unwrap();
        assert_eq!(result.adaptations_applied, 1);
    }
}
