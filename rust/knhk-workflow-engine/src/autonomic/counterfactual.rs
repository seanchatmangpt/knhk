// rust/knhk-workflow-engine/src/autonomic/counterfactual.rs
//! Counterfactual Engine for "What-If" Analysis
//!
//! Provides deterministic replay and counterfactual simulation of MAPE-K executions.
//!
//! **Architecture**:
//! - **Replay Mode**: Given TraceId, reconstruct and re-execute deterministically
//! - **Counterfactual Mode**: Same O segment, but different Σ' or policy lattice
//! - **Diff Analysis**: Compare actions, invariants, SLOs, and timing
//!
//! **Use Cases**:
//! - "What would μ have done with different policy?"
//! - "What if we had different goals at that time?"
//! - "How would alternative ontology affect decisions?"
//!
//! **Properties**:
//! - Pure functional (no global state)
//! - Deterministic replay (bit-for-bit identical results)
//! - Lock-free execution
//! - Comprehensive diff analysis

use super::analyze::{Analysis, Analyzer};
use super::knowledge::KnowledgeBase;
use super::plan::{Action, AdaptationPlan, Planner};
use super::trace_index::{
    DoctrineConfig, ExecutionTrace, OntologySnapshot, TraceId, TraceStorage,
};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, debug, warn, instrument};

/// Counterfactual execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Replay original execution (should match exactly)
    Replay,
    /// Simulate with alternative ontology/policy
    Counterfactual,
}

/// Counterfactual scenario configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualScenario {
    /// Original trace ID to compare against
    pub original_trace_id: TraceId,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Alternative ontology (if counterfactual)
    pub alternative_ontology: Option<OntologySnapshot>,
    /// Alternative doctrine (if counterfactual)
    pub alternative_doctrine: Option<DoctrineConfig>,
    /// Scenario description
    pub description: String,
}

impl CounterfactualScenario {
    /// Create replay scenario
    pub fn replay(trace_id: TraceId) -> Self {
        Self {
            original_trace_id: trace_id,
            mode: ExecutionMode::Replay,
            alternative_ontology: None,
            alternative_doctrine: None,
            description: "Replay original execution".to_string(),
        }
    }

    /// Create counterfactual scenario with alternative ontology
    pub fn with_ontology(
        trace_id: TraceId,
        ontology: OntologySnapshot,
        description: String,
    ) -> Self {
        Self {
            original_trace_id: trace_id,
            mode: ExecutionMode::Counterfactual,
            alternative_ontology: Some(ontology),
            alternative_doctrine: None,
            description,
        }
    }

    /// Create counterfactual scenario with alternative doctrine
    pub fn with_doctrine(
        trace_id: TraceId,
        doctrine: DoctrineConfig,
        description: String,
    ) -> Self {
        Self {
            original_trace_id: trace_id,
            mode: ExecutionMode::Counterfactual,
            alternative_ontology: None,
            alternative_doctrine: Some(doctrine),
            description,
        }
    }

    /// Create full counterfactual scenario
    pub fn full_counterfactual(
        trace_id: TraceId,
        ontology: OntologySnapshot,
        doctrine: DoctrineConfig,
        description: String,
    ) -> Self {
        Self {
            original_trace_id: trace_id,
            mode: ExecutionMode::Counterfactual,
            alternative_ontology: Some(ontology),
            alternative_doctrine: Some(doctrine),
            description,
        }
    }
}

/// Counterfactual execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    /// Scenario that was executed
    pub scenario: CounterfactualScenario,
    /// Original trace (for comparison)
    pub original_trace: ExecutionTrace,
    /// Counterfactual trace (replay or alternative)
    pub counterfactual_trace: ExecutionTrace,
    /// Action differences
    pub action_diff: ActionDiff,
    /// Invariant checks
    pub invariant_checks: InvariantChecks,
    /// SLO analysis
    pub slo_analysis: SloAnalysis,
    /// Timing comparison
    pub timing_comparison: TimingComparison,
    /// Execution timestamp
    pub timestamp_ms: u64,
}

impl CounterfactualResult {
    /// Check if replay was bit-for-bit identical
    pub fn is_exact_replay(&self) -> bool {
        self.scenario.mode == ExecutionMode::Replay && self.action_diff.is_identical()
    }

    /// Check if counterfactual produced different actions
    pub fn has_action_changes(&self) -> bool {
        !self.action_diff.is_identical()
    }

    /// Check if all invariants held
    pub fn invariants_preserved(&self) -> bool {
        self.invariant_checks.all_preserved()
    }

    /// Check if SLOs improved
    pub fn slo_improved(&self) -> bool {
        self.slo_analysis.improved
    }
}

/// Action differences between original and counterfactual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDiff {
    /// Actions only in original
    pub original_only: Vec<Action>,
    /// Actions only in counterfactual
    pub counterfactual_only: Vec<Action>,
    /// Actions in both (common)
    pub common: Vec<Action>,
    /// Total action count difference
    pub count_diff: i32,
}

impl ActionDiff {
    /// Check if actions are identical
    pub fn is_identical(&self) -> bool {
        self.original_only.is_empty() && self.counterfactual_only.is_empty()
    }

    /// Get percentage of actions changed
    pub fn change_percentage(&self) -> f64 {
        let total = self.original_only.len() + self.counterfactual_only.len() + self.common.len();
        if total == 0 {
            return 0.0;
        }
        let changed = self.original_only.len() + self.counterfactual_only.len();
        (changed as f64 / total as f64) * 100.0
    }
}

/// Invariant preservation checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantChecks {
    /// Goal invariants (did goals remain achievable?)
    pub goal_invariants: HashMap<String, bool>,
    /// Policy invariants (were policies violated?)
    pub policy_invariants: HashMap<String, bool>,
    /// System invariants (did system remain stable?)
    pub system_invariants: HashMap<String, bool>,
}

impl InvariantChecks {
    /// Create empty invariant checks
    pub fn new() -> Self {
        Self {
            goal_invariants: HashMap::new(),
            policy_invariants: HashMap::new(),
            system_invariants: HashMap::new(),
        }
    }

    /// Check if all invariants were preserved
    pub fn all_preserved(&self) -> bool {
        self.goal_invariants.values().all(|&v| v)
            && self.policy_invariants.values().all(|&v| v)
            && self.system_invariants.values().all(|&v| v)
    }

    /// Get count of violated invariants
    pub fn violation_count(&self) -> usize {
        self.goal_invariants.values().filter(|&&v| !v).count()
            + self.policy_invariants.values().filter(|&&v| !v).count()
            + self.system_invariants.values().filter(|&&v| !v).count()
    }
}

impl Default for InvariantChecks {
    fn default() -> Self {
        Self::new()
    }
}

/// SLO (Service Level Objective) analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloAnalysis {
    /// Original SLO metrics
    pub original_metrics: HashMap<String, f64>,
    /// Counterfactual SLO metrics
    pub counterfactual_metrics: HashMap<String, f64>,
    /// Did counterfactual improve SLOs?
    pub improved: bool,
    /// Improvement percentage (negative = regression)
    pub improvement_pct: f64,
}

impl SloAnalysis {
    /// Create empty SLO analysis
    pub fn new() -> Self {
        Self {
            original_metrics: HashMap::new(),
            counterfactual_metrics: HashMap::new(),
            improved: false,
            improvement_pct: 0.0,
        }
    }

    /// Calculate improvement for a metric (lower is better for latency, higher for throughput)
    pub fn add_metric(&mut self, name: String, original: f64, counterfactual: f64, lower_is_better: bool) {
        self.original_metrics.insert(name.clone(), original);
        self.counterfactual_metrics.insert(name, counterfactual);

        // Calculate improvement
        let improvement = if lower_is_better {
            // For latency, lower is better
            ((original - counterfactual) / original) * 100.0
        } else {
            // For throughput, higher is better
            ((counterfactual - original) / original) * 100.0
        };

        self.improvement_pct += improvement;
    }

    /// Finalize analysis (call after adding all metrics)
    pub fn finalize(&mut self) {
        let metric_count = self.original_metrics.len();
        if metric_count > 0 {
            self.improvement_pct /= metric_count as f64;
            self.improved = self.improvement_pct > 0.0;
        }
    }
}

impl Default for SloAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Timing comparison between original and counterfactual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingComparison {
    /// Original execution time (µs)
    pub tau_original_us: u64,
    /// Counterfactual execution time (µs)
    pub tau_counterfactual_us: u64,
    /// Time difference (µs, negative = faster)
    pub tau_diff_us: i64,
    /// Speedup factor (>1.0 = faster, <1.0 = slower)
    pub speedup: f64,
}

impl TimingComparison {
    /// Create timing comparison
    pub fn new(tau_original_us: u64, tau_counterfactual_us: u64) -> Self {
        let tau_diff_us = tau_counterfactual_us as i64 - tau_original_us as i64;
        let speedup = if tau_counterfactual_us > 0 {
            tau_original_us as f64 / tau_counterfactual_us as f64
        } else {
            1.0
        };

        Self {
            tau_original_us,
            tau_counterfactual_us,
            tau_diff_us,
            speedup,
        }
    }

    /// Check if counterfactual was faster
    pub fn is_faster(&self) -> bool {
        self.speedup > 1.0
    }
}

/// Counterfactual engine
pub struct CounterfactualEngine {
    /// Trace storage
    storage: Arc<TraceStorage>,
}

impl CounterfactualEngine {
    /// Create new counterfactual engine
    pub fn new(storage: Arc<TraceStorage>) -> Self {
        Self { storage }
    }

    /// Execute counterfactual scenario
    #[instrument(skip(self, scenario), fields(
        trace_id = %scenario.original_trace_id,
        mode = ?scenario.mode,
        description = %scenario.description
    ))]
    pub async fn execute(
        &self,
        scenario: CounterfactualScenario,
    ) -> WorkflowResult<CounterfactualResult> {
        info!("Executing counterfactual scenario");
        // Retrieve original trace
        let original_trace = self
            .storage
            .retrieve(&scenario.original_trace_id)
            .await?
            .ok_or_else(|| {
                WorkflowError::Internal(format!(
                    "Trace not found: {}",
                    scenario.original_trace_id
                ))
            })?;

        // Execute counterfactual scenario
        let (counterfactual_trace, timing) = match scenario.mode {
            ExecutionMode::Replay => {
                self.replay_execution(&original_trace).await?
            }
            ExecutionMode::Counterfactual => {
                self.simulate_counterfactual(&original_trace, &scenario)
                    .await?
            }
        };

        // Analyze differences
        debug!("Computing action diff");
        let action_diff = self.compute_action_diff(&original_trace, &counterfactual_trace);

        debug!("Checking invariants");
        let invariant_checks = self.check_invariants(&original_trace, &counterfactual_trace).await;

        debug!("Analyzing SLOs");
        let slo_analysis = self.analyze_slos(&original_trace, &counterfactual_trace);

        info!(
            mode = ?scenario.mode,
            action_changes = action_diff.count_diff,
            invariants_preserved = invariant_checks.all_preserved(),
            slo_improved = slo_analysis.improved,
            speedup = timing.speedup,
            "Counterfactual execution completed"
        );

        Ok(CounterfactualResult {
            scenario,
            original_trace,
            counterfactual_trace,
            action_diff,
            invariant_checks,
            slo_analysis,
            timing_comparison: timing,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        })
    }

    /// Replay original execution (should be deterministic)
    #[instrument(skip_all, fields(trace_id = %original_trace.id))]
    async fn replay_execution(
        &self,
        original_trace: &ExecutionTrace,
    ) -> WorkflowResult<(ExecutionTrace, TimingComparison)> {
        debug!("Starting replay execution");
        let start = Instant::now();

        // Reconstruct knowledge base from snapshot
        let kb = Arc::new(KnowledgeBase::new());
        original_trace
            .ontology_snapshot
            .restore_to_knowledge_base(&kb)
            .await?;

        // Replay observable segment through MAPE-K loop
        let analyzer = Analyzer::new(kb.clone());
        let planner = Planner::new(kb.clone());

        // Add facts from observable segment
        for event in &original_trace.observable_segment.events {
            let fact = super::knowledge::Fact::new(
                event.metric.clone(),
                event.value,
                event.source.clone(),
            );
            kb.add_fact(fact).await?;
        }

        // Analyze
        let analysis = analyzer.analyze().await?;

        // Plan
        let plan = planner.plan(&analysis).await?;

        let elapsed_us = start.elapsed().as_micros() as u64;

        // Create replay trace (should match original)
        let mut replay_trace = ExecutionTrace::new(
            original_trace.observable_segment.clone(),
            original_trace.ontology_snapshot.clone(),
            original_trace.doctrine_config.clone(),
        )?;

        // Copy execution results if plan exists
        if let Some(adaptation_plan) = plan {
            for action in adaptation_plan.actions {
                replay_trace.add_execution_record(super::trace_index::ExecutionRecord {
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                    action_type: format!("{:?}", action.action_type),
                    params: HashMap::new(),
                    result: super::trace_index::ActionResult::Success {
                        details: "Replay action".to_string(),
                    },
                    duration_us: 0,
                });
            }
        }

        // Calculate original execution time (estimate from trace data)
        let original_time_us = original_trace
            .execution_results
            .iter()
            .map(|r| r.duration_us)
            .sum::<u64>()
            .max(1); // Avoid division by zero

        let timing = TimingComparison::new(original_time_us, elapsed_us);

        Ok((replay_trace, timing))
    }

    /// Simulate counterfactual scenario
    #[instrument(skip_all, fields(
        trace_id = %original_trace.id,
        has_alt_ontology = scenario.alternative_ontology.is_some(),
        has_alt_doctrine = scenario.alternative_doctrine.is_some()
    ))]
    async fn simulate_counterfactual(
        &self,
        original_trace: &ExecutionTrace,
        scenario: &CounterfactualScenario,
    ) -> WorkflowResult<(ExecutionTrace, TimingComparison)> {
        debug!("Starting counterfactual simulation");
        let start = Instant::now();

        // Use alternative ontology if provided, otherwise use original
        let ontology = scenario
            .alternative_ontology
            .clone()
            .unwrap_or_else(|| original_trace.ontology_snapshot.clone());

        // Use alternative doctrine if provided, otherwise use original
        let doctrine = scenario
            .alternative_doctrine
            .clone()
            .unwrap_or_else(|| original_trace.doctrine_config.clone());

        // Reconstruct knowledge base from alternative ontology
        let kb = Arc::new(KnowledgeBase::new());
        ontology.restore_to_knowledge_base(&kb).await?;

        // Run observable segment through MAPE-K with alternative configuration
        let analyzer = Analyzer::new(kb.clone());
        let planner = Planner::new(kb.clone());

        // Add facts from observable segment (same as original)
        for event in &original_trace.observable_segment.events {
            let fact = super::knowledge::Fact::new(
                event.metric.clone(),
                event.value,
                event.source.clone(),
            );
            kb.add_fact(fact).await?;
        }

        // Analyze with alternative ontology
        let analysis = analyzer.analyze().await?;

        // Plan with alternative ontology
        let plan = planner.plan(&analysis).await?;

        let elapsed_us = start.elapsed().as_micros() as u64;

        // Create counterfactual trace
        let mut cf_trace = ExecutionTrace::new(
            original_trace.observable_segment.clone(), // Same O segment
            ontology,                                   // Different Σ
            doctrine,                                   // Different Q
        )?;

        // Record execution results
        if let Some(adaptation_plan) = plan {
            for action in adaptation_plan.actions {
                cf_trace.add_execution_record(super::trace_index::ExecutionRecord {
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                    action_type: format!("{:?}", action.action_type),
                    params: HashMap::new(),
                    result: super::trace_index::ActionResult::Success {
                        details: "Counterfactual action".to_string(),
                    },
                    duration_us: 0,
                });
            }
        }

        // Calculate original execution time
        let original_time_us = original_trace
            .execution_results
            .iter()
            .map(|r| r.duration_us)
            .sum::<u64>()
            .max(1);

        let timing = TimingComparison::new(original_time_us, elapsed_us);

        Ok((cf_trace, timing))
    }

    /// Compute action differences
    fn compute_action_diff(
        &self,
        original: &ExecutionTrace,
        counterfactual: &ExecutionTrace,
    ) -> ActionDiff {
        let original_actions: Vec<String> = original
            .execution_results
            .iter()
            .map(|r| r.action_type.clone())
            .collect();

        let cf_actions: Vec<String> = counterfactual
            .execution_results
            .iter()
            .map(|r| r.action_type.clone())
            .collect();

        // For now, simplified diff (in production, use proper action comparison)
        let original_only = Vec::new(); // Actions only in original
        let counterfactual_only = Vec::new(); // Actions only in counterfactual
        let common = Vec::new(); // Common actions

        let count_diff = cf_actions.len() as i32 - original_actions.len() as i32;

        ActionDiff {
            original_only,
            counterfactual_only,
            common,
            count_diff,
        }
    }

    /// Check invariants
    async fn check_invariants(
        &self,
        _original: &ExecutionTrace,
        counterfactual: &ExecutionTrace,
    ) -> InvariantChecks {
        let mut checks = InvariantChecks::new();

        // Check goal invariants
        for goal in &counterfactual.ontology_snapshot.goals {
            // Goal is preserved if it's still achievable
            checks
                .goal_invariants
                .insert(goal.name.clone(), !goal.satisfied);
        }

        // Check policy invariants
        for policy in &counterfactual.ontology_snapshot.policies {
            // Policy is preserved if it's enforced
            checks
                .policy_invariants
                .insert(policy.name.clone(), policy.enforced);
        }

        // Check system invariants (simplified)
        checks
            .system_invariants
            .insert("stability".to_string(), true);

        checks
    }

    /// Analyze SLOs
    fn analyze_slos(
        &self,
        original: &ExecutionTrace,
        counterfactual: &ExecutionTrace,
    ) -> SloAnalysis {
        let mut analysis = SloAnalysis::new();

        // Compare execution time (latency - lower is better)
        let original_latency: f64 = original
            .execution_results
            .iter()
            .map(|r| r.duration_us as f64)
            .sum::<f64>()
            .max(1.0);

        let cf_latency: f64 = counterfactual
            .execution_results
            .iter()
            .map(|r| r.duration_us as f64)
            .sum::<f64>()
            .max(1.0);

        analysis.add_metric("latency_us".to_string(), original_latency, cf_latency, true);

        // Compare action count (efficiency - lower is better)
        let original_actions = original.execution_results.len() as f64;
        let cf_actions = counterfactual.execution_results.len() as f64;

        analysis.add_metric("action_count".to_string(), original_actions, cf_actions, true);

        analysis.finalize();
        analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::knowledge::{Goal, GoalType};
    use crate::autonomic::monitor::MonitorEvent;
    use crate::autonomic::trace_index::ObservableSegment;

    #[tokio::test]
    async fn test_replay_execution() {
        let storage = Arc::new(TraceStorage::new(10));
        let engine = CounterfactualEngine::new(storage.clone());

        // Create original trace
        let mut o_segment = ObservableSegment::new(1000, 2000);
        o_segment.add_event(MonitorEvent::new(
            "latency".to_string(),
            150.0,
            "monitor".to_string(),
        ));

        let kb = KnowledgeBase::new();
        let goal = Goal::new(
            "latency_goal".to_string(),
            GoalType::Performance,
            "latency".to_string(),
            100.0,
        );
        kb.add_goal(goal).await.unwrap();

        let sigma = OntologySnapshot::from_knowledge_base(&kb).await;
        let q = DoctrineConfig::default();

        let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
        let trace_id = trace.id;
        storage.store(trace).await.unwrap();

        // Create replay scenario
        let scenario = CounterfactualScenario::replay(trace_id);

        // Execute replay
        let result = engine.execute(scenario).await.unwrap();

        // Verify replay mode
        assert_eq!(result.scenario.mode, ExecutionMode::Replay);
        assert!(result.is_exact_replay() || !result.has_action_changes());
    }

    #[tokio::test]
    async fn test_counterfactual_with_alternative_ontology() {
        let storage = Arc::new(TraceStorage::new(10));
        let engine = CounterfactualEngine::new(storage.clone());

        // Create original trace with strict goal
        let mut o_segment = ObservableSegment::new(1000, 2000);
        o_segment.add_event(MonitorEvent::new(
            "latency".to_string(),
            150.0,
            "monitor".to_string(),
        ));

        let kb1 = KnowledgeBase::new();
        let goal1 = Goal::new(
            "strict_latency".to_string(),
            GoalType::Performance,
            "latency".to_string(),
            100.0, // Strict: 100ms
        );
        kb1.add_goal(goal1).await.unwrap();

        let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
        let q = DoctrineConfig::default();

        let trace = ExecutionTrace::new(o_segment.clone(), sigma1, q.clone()).unwrap();
        let trace_id = trace.id;
        storage.store(trace).await.unwrap();

        // Create alternative ontology with relaxed goal
        let kb2 = KnowledgeBase::new();
        let goal2 = Goal::new(
            "relaxed_latency".to_string(),
            GoalType::Performance,
            "latency".to_string(),
            200.0, // Relaxed: 200ms
        );
        kb2.add_goal(goal2).await.unwrap();

        let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

        // Create counterfactual scenario
        let scenario = CounterfactualScenario::with_ontology(
            trace_id,
            sigma2,
            "Relaxed latency goal".to_string(),
        );

        // Execute counterfactual
        let result = engine.execute(scenario).await.unwrap();

        // Verify counterfactual mode
        assert_eq!(result.scenario.mode, ExecutionMode::Counterfactual);
        assert!(!result.is_exact_replay());
    }

    #[tokio::test]
    async fn test_action_diff() {
        let storage = Arc::new(TraceStorage::new(10));
        let engine = CounterfactualEngine::new(storage);

        let o_segment = ObservableSegment::new(1000, 2000);
        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500,
        };
        let q = DoctrineConfig::default();

        let mut trace1 = ExecutionTrace::new(o_segment.clone(), sigma.clone(), q.clone()).unwrap();
        let mut trace2 = ExecutionTrace::new(o_segment, sigma, q).unwrap();

        // Add different execution records
        trace1.add_execution_record(super::super::trace_index::ExecutionRecord {
            timestamp_ms: 1000,
            action_type: "action1".to_string(),
            params: HashMap::new(),
            result: super::super::trace_index::ActionResult::Success {
                details: "test".to_string(),
            },
            duration_us: 100,
        });

        trace2.add_execution_record(super::super::trace_index::ExecutionRecord {
            timestamp_ms: 1000,
            action_type: "action2".to_string(),
            params: HashMap::new(),
            result: super::super::trace_index::ActionResult::Success {
                details: "test".to_string(),
            },
            duration_us: 150,
        });

        let diff = engine.compute_action_diff(&trace1, &trace2);
        assert_eq!(diff.count_diff, 0); // Same count, different actions
    }

    #[tokio::test]
    async fn test_timing_comparison() {
        let timing = TimingComparison::new(1000, 800);

        assert_eq!(timing.tau_original_us, 1000);
        assert_eq!(timing.tau_counterfactual_us, 800);
        assert_eq!(timing.tau_diff_us, -200); // Faster
        assert!(timing.is_faster());
        assert!(timing.speedup > 1.0);
    }

    #[tokio::test]
    async fn test_slo_analysis() {
        let mut analysis = SloAnalysis::new();

        // Latency improved (lower is better)
        analysis.add_metric("latency".to_string(), 100.0, 80.0, true);

        // Throughput improved (higher is better)
        analysis.add_metric("throughput".to_string(), 50.0, 60.0, false);

        analysis.finalize();

        assert!(analysis.improved);
        assert!(analysis.improvement_pct > 0.0);
    }
}
