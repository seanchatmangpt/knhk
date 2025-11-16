// rust/knhk-workflow-engine/src/autonomic/mod.rs
//! MAPE-K Autonomic Computing Framework
//!
//! Implements the MAPE-K (Monitor, Analyze, Plan, Execute - Knowledge) reference
//! model for autonomic computing, enabling self-managing workflow systems.
//!
//! **Architecture**:
//! - **Monitor**: Collect runtime metrics from workflow execution
//! - **Analyze**: Detect anomalies and evaluate goals
//! - **Plan**: Generate adaptation plans based on analysis
//! - **Execute**: Apply adaptations to the running system
//! - **Knowledge**: Shared knowledge base for all MAPE components
//!
//! **Integration Points**:
//! - Dark Matter 80/20 coverage tracker (predicate access patterns)
//! - Multi-instance patterns (dynamic scaling)
//! - Cancellation regions (failure recovery)
//! - Compensation handlers (rollback on errors)
//! - OTEL telemetry (metric collection)
//! - Session-scoped autonomic runtime (per-workflow adaptation)
//! - Doctrine-aware failure modes (safe degradation)
//!
//! **Session-Scoped Adaptation**:
//! - Per-workflow session tracking with isolation
//! - Lock-free atomic metrics for hot-path operations
//! - Local adaptation with global Q (doctrine) compliance
//! - Aggregation of session metrics to global MAPE-K
//! - Millions of concurrent sessions supported
//!
//! **Failure Modes**:
//! - Safe ceiling behavior when MAPE-K loop degrades
//! - Automatic mode degradation based on health signals
//! - Mode-aware policy enforcement and action filtering
//! - Observable mode changes with telemetry receipts

pub mod knowledge;
pub mod monitor;
pub mod analyze;
pub mod plan;
pub mod execute;
pub mod loop_controller;
pub mod session;
pub mod session_adapter;
pub mod failure_modes;
pub mod mode_policy;
pub mod trace_index;
pub mod counterfactual;
pub mod delta_sigma;
pub mod overlay_validator;

pub mod policy_lattice;
pub mod doctrine;
pub use knowledge::{Knowledge, KnowledgeBase, Goal, GoalType, KnowledgeId, Policy, Rule, Fact};
pub use monitor::{Monitor, MetricCollector, MonitorEvent};
pub use analyze::{Analyzer, Anomaly, AnomalyType, Analysis, HealthStatus};
pub use plan::{Planner, AdaptationPlan, Action, ActionType};
pub use execute::{Executor, ExecutionResult};
pub use loop_controller::{MapeKController, ControllerConfig, ControllerState};
pub use session::{
    SessionHandle, SessionId, SessionMetrics, SessionMetricsSnapshot, SessionState, SessionTable,
    SessionContext, TenantId, SessionTableStats,
};
pub use session_adapter::{
    SessionAdapter, SessionAdapterConfig, SessionAction, SessionDecision, SessionEvent,
    SessionAggregator, AggregatedMetrics, GlobalQ, SessionAdapterStats,
};
pub use failure_modes::{
    AutonomicMode, HealthSignal, ComponentType, HealthMetrics, ModeManager, ModeChangeEvent,
};
pub use mode_policy::{
    ModePolicyFilter, MinimumMode, ActionAnnotation, ActionPattern, RejectedAction,
    ModeAwareAdaptationPlan,
};
pub use trace_index::{
    TraceId, ObservableSegment, OntologySnapshot, DoctrineConfig,
    ExecutionTrace, TraceStorage, ExecutionRecord, ActionResult,
};
pub use counterfactual::{
    CounterfactualEngine, CounterfactualScenario, CounterfactualResult,
    ExecutionMode, ActionDiff, InvariantChecks, SloAnalysis, TimingComparison,
};
pub use policy_lattice::{
    PolicyElement, PolicyLattice, Lattice,
    LatencyBound, FailureRateBound, GuardStrictness, GuardStrictnessLevel,
    CapacityEnvelope, Strictness, PolicyId,
};
pub use doctrine::{
    Doctrine, ExecutionMetrics, DoctrineAction,
    MAX_EXEC_TICKS, MAX_RUN_LEN, MAX_CALL_DEPTH,
};
pub use delta_sigma::{
    DeltaSigma, OverlayId, OverlayScope, OverlayChange, ProofObligation,
    ProofState, Unproven, ProofPending, Proven, ValidationEffort, OverlayComposition,
    CompositionStrategy,
};
pub use overlay_validator::{
    OverlayValidator, OverlayProof, ValidationResult, ObligationResult,
    TestResults, PerformanceMetrics,
};

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// MAPE-K cycle statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleStats {
    /// Total cycles executed
    pub total_cycles: u64,
    /// Successful adaptations
    pub successful_adaptations: u64,
    /// Failed adaptations
    pub failed_adaptations: u64,
    /// Average cycle duration (ms)
    pub avg_cycle_duration_ms: f64,
    /// Anomalies detected
    pub anomalies_detected: u64,
    /// Plans generated
    pub plans_generated: u64,
    /// Actions executed
    pub actions_executed: u64,
}

impl Default for CycleStats {
    fn default() -> Self {
        Self {
            total_cycles: 0,
            successful_adaptations: 0,
            failed_adaptations: 0,
            avg_cycle_duration_ms: 0.0,
            anomalies_detected: 0,
            plans_generated: 0,
            actions_executed: 0,
        }
    }
}

/// Autonomic property
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomicProperty {
    /// Self-configuration: Automatic configuration based on high-level policies
    SelfConfiguration,
    /// Self-healing: Automatic detection and recovery from failures
    SelfHealing,
    /// Self-optimization: Continuous improvement of performance
    SelfOptimization,
    /// Self-protection: Proactive defense against attacks
    SelfProtection,
}

/// Autonomic manager interface
pub trait AutonomicManager: Send + Sync {
    /// Get supported autonomic properties
    fn properties(&self) -> Vec<AutonomicProperty>;

    /// Start autonomic management
    fn start(&mut self) -> WorkflowResult<()>;

    /// Stop autonomic management
    fn stop(&mut self) -> WorkflowResult<()>;

    /// Get cycle statistics
    fn stats(&self) -> CycleStats;
}
