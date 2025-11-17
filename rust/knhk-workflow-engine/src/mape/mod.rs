//! MAPE-K Autonomic Computing Engine
//!
//! Implements the Monitor-Analyze-Plan-Execute-Knowledge feedback loop for
//! self-adapting workflows per SELF_EXECUTING_WORKFLOWS.md specification.

use crate::engine::{HookEngine, PatternLibrary};
use crate::error::{WorkflowError, WorkflowResult};
use crate::guards::InvariantChecker;
use crate::receipts::{Receipt, ReceiptGenerator, ReceiptStore};
use crate::snapshots::SnapshotVersioning;
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub mod analyze;
pub mod execute;
pub mod knowledge;
pub mod monitor;
pub mod plan;

pub use analyze::AnalyzePhase;
pub use execute::ExecutePhase;
pub use knowledge::KnowledgeBase;
pub use monitor::MonitorPhase;
pub use plan::PlanPhase;

/// MAPE-K autonomic engine that implements the feedback loop
/// for self-executing workflows.
///
/// # Architecture
///
/// ```text
/// Observation(O) → Monitor → Analyze → Plan → Execute → Knowledge
///                     ↓         ↓        ↓        ↓         ↓
///                  Receipts   Symptoms  Δ-Plans  Σ_new   Patterns
/// ```
#[derive(Clone)]
pub struct MapeKEngine {
    /// Monitor phase - collects observations
    monitor: Arc<MonitorPhase>,
    /// Analyze phase - detects symptoms
    analyze: Arc<AnalyzePhase>,
    /// Plan phase - generates adaptation plans
    plan: Arc<PlanPhase>,
    /// Execute phase - applies adaptations
    execute: Arc<ExecutePhase>,
    /// Knowledge base - stores learned patterns
    knowledge: Arc<RwLock<KnowledgeBase>>,
    /// Receipt store for observation persistence
    receipt_store: Arc<ReceiptStore>,
    /// Snapshot versioning for Σ management
    snapshot_versioning: Arc<SnapshotVersioning>,
}

impl MapeKEngine {
    /// Create a new MAPE-K engine
    pub fn new(
        receipt_store: Arc<ReceiptStore>,
        snapshot_versioning: Arc<SnapshotVersioning>,
        hook_engine: Arc<HookEngine>,
        invariant_checker: Arc<InvariantChecker>,
    ) -> Self {
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));

        Self {
            monitor: Arc::new(MonitorPhase::new(receipt_store.clone(), knowledge.clone())),
            analyze: Arc::new(AnalyzePhase::new(knowledge.clone())),
            plan: Arc::new(PlanPhase::new(
                knowledge.clone(),
                snapshot_versioning.clone(),
            )),
            execute: Arc::new(ExecutePhase::new(
                snapshot_versioning.clone(),
                hook_engine,
                invariant_checker,
            )),
            knowledge,
            receipt_store,
            snapshot_versioning,
        }
    }

    /// Run one complete MAPE-K cycle
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the cycle completed successfully.
    /// Returns metrics about the cycle performance.
    pub async fn run_cycle(&self) -> WorkflowResult<MapeKCycleMetrics> {
        let start_time = Utc::now();

        // 1. Monitor: Collect observations from receipts
        let observations = self.monitor.collect_observations().await?;
        let monitor_duration = (Utc::now() - start_time).num_milliseconds();

        // 2. Analyze: Detect symptoms from observations
        let symptoms = self.analyze.detect_symptoms(&observations).await?;
        let analyze_duration = (Utc::now() - start_time).num_milliseconds() - monitor_duration;

        // 3. Plan: Generate adaptation plans
        let plans = self.plan.generate_plans(&symptoms).await?;
        let plan_duration =
            (Utc::now() - start_time).num_milliseconds() - analyze_duration - monitor_duration;

        // 4. Execute: Apply adaptations (shadow deploy + promote)
        let results = self.execute.apply_adaptations(plans).await?;
        let execute_duration = (Utc::now() - start_time).num_milliseconds()
            - plan_duration
            - analyze_duration
            - monitor_duration;

        // 5. Knowledge: Store learned patterns
        self.knowledge
            .write()
            .record_cycle(&observations, &symptoms, &results);

        let total_duration = (Utc::now() - start_time).num_milliseconds();

        Ok(MapeKCycleMetrics {
            total_duration_ms: total_duration,
            monitor_duration_ms: monitor_duration,
            analyze_duration_ms: analyze_duration,
            plan_duration_ms: plan_duration,
            execute_duration_ms: execute_duration,
            observations_count: observations.len(),
            symptoms_detected: symptoms.len(),
            plans_generated: results.adaptations_applied,
            cycle_timestamp: Utc::now(),
        })
    }

    /// Start continuous MAPE-K loop with specified interval
    pub async fn start_continuous_loop(&self, interval_ms: u64) -> WorkflowResult<()> {
        loop {
            match self.run_cycle().await {
                Ok(metrics) => {
                    tracing::info!(
                        "MAPE-K cycle completed in {}ms: {} observations, {} symptoms, {} adaptations",
                        metrics.total_duration_ms,
                        metrics.observations_count,
                        metrics.symptoms_detected,
                        metrics.plans_generated
                    );
                }
                Err(e) => {
                    tracing::error!("MAPE-K cycle failed: {:?}", e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
        }
    }

    /// Query knowledge base for learned patterns
    pub fn query_knowledge(&self, query: &str) -> Vec<String> {
        self.knowledge.read().query_patterns(query)
    }

    /// Get current snapshot version
    pub fn current_snapshot_id(&self) -> String {
        self.snapshot_versioning.current_id()
    }
}

/// Metrics for a MAPE-K cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapeKCycleMetrics {
    /// Total cycle duration in milliseconds
    pub total_duration_ms: i64,
    /// Monitor phase duration
    pub monitor_duration_ms: i64,
    /// Analyze phase duration
    pub analyze_duration_ms: i64,
    /// Plan phase duration
    pub plan_duration_ms: i64,
    /// Execute phase duration
    pub execute_duration_ms: i64,
    /// Number of observations collected
    pub observations_count: usize,
    /// Number of symptoms detected
    pub symptoms_detected: usize,
    /// Number of plans/adaptations applied
    pub plans_generated: usize,
    /// Timestamp of this cycle
    pub cycle_timestamp: chrono::DateTime<Utc>,
}

/// Observation from monitoring phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Receipt ID
    pub receipt_id: String,
    /// Sigma version
    pub sigma_id: String,
    /// Execution duration in ticks
    pub ticks_used: u32,
    /// Guards that were checked
    pub guards_checked: Vec<String>,
    /// Guards that failed
    pub guards_failed: Vec<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<Utc>,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Symptom detected by analyze phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symptom {
    /// Symptom type
    pub symptom_type: SymptomType,
    /// Severity (0.0 - 1.0)
    pub severity: f64,
    /// Description
    pub description: String,
    /// Related observations
    pub observations: Vec<String>,
}

/// Types of symptoms that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SymptomType {
    /// Performance degradation (approaching tick limit)
    PerformanceDegradation,
    /// SLO violation
    SloViolation,
    /// High guard failure rate
    GuardFailureSpike,
    /// Unexpected workflow path
    UnexpectedBehavior,
    /// Resource exhaustion
    ResourceExhaustion,
}

/// Adaptation plan generated by plan phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationPlan {
    /// Plan ID
    pub plan_id: String,
    /// Plan type
    pub plan_type: AdaptationType,
    /// Target symptom
    pub target_symptom: SymptomType,
    /// Proposed changes to Σ
    pub sigma_delta: Option<String>,
    /// Proposed config changes
    pub config_delta: Option<HashMap<String, String>>,
    /// Expected improvement
    pub expected_improvement: f64,
}

/// Types of adaptations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdaptationType {
    /// Add/modify guards
    GuardModification,
    /// Change pattern selection
    PatternChange,
    /// Resource scaling
    ResourceScaling,
    /// Workflow restructuring
    WorkflowRestructuring,
    /// Configuration tuning
    ConfigurationTuning,
}

/// Result of applying adaptations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationResult {
    /// Number of adaptations applied
    pub adaptations_applied: usize,
    /// New snapshot ID
    pub new_sigma_id: Option<String>,
    /// Errors encountered
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mape_k_engine_creation() {
        let receipt_store = Arc::new(ReceiptStore::in_memory());
        let snapshot_versioning = Arc::new(SnapshotVersioning::new("./test_snapshots"));
        let hook_engine = Arc::new(HookEngine::new());
        let invariant_checker = Arc::new(InvariantChecker::new());

        let engine = MapeKEngine::new(
            receipt_store,
            snapshot_versioning,
            hook_engine,
            invariant_checker,
        );

        // Engine should be created successfully
        assert!(!engine.current_snapshot_id().is_empty());
    }
}
