//! Core types for the MAPE-K autonomic system
//!
//! These types map directly to the MAPE-K ontology defined in
//! `ontology/mape-k-autonomic.ttl`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// Autonomic controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// How often the MAPE-K loop executes
    pub loop_frequency: Duration,

    /// Path to persistent knowledge store
    pub knowledge_path: String,

    /// Maximum latency for hot path operations (ticks)
    pub max_hot_path_ticks: u64,

    /// Enable self-healing
    pub enable_self_healing: bool,

    /// Enable self-optimizing
    pub enable_self_optimizing: bool,

    /// Enable self-learning
    pub enable_self_learning: bool,

    /// Enable self-protecting
    pub enable_self_protecting: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            loop_frequency: Duration::from_millis(crate::DEFAULT_LOOP_FREQUENCY_MS),
            knowledge_path: "./autonomic_knowledge.db".to_string(),
            max_hot_path_ticks: crate::CHATMAN_CONSTANT_TICKS,
            enable_self_healing: true,
            enable_self_optimizing: true,
            enable_self_learning: true,
            enable_self_protecting: false,
        }
    }
}

impl Config {
    /// Set loop frequency
    pub fn with_loop_frequency(mut self, frequency: Duration) -> Self {
        self.loop_frequency = frequency;
        self
    }

    /// Set knowledge path
    pub fn with_knowledge_path(mut self, path: impl Into<String>) -> Self {
        self.knowledge_path = path.into();
        self
    }
}

/// Type of metric being monitored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Performance metrics (latency, throughput)
    Performance,
    /// Reliability metrics (error rate, success rate)
    Reliability,
    /// Resource metrics (CPU, memory, network)
    Resource,
    /// Quality metrics (data quality, accuracy)
    Quality,
    /// Security metrics (unauthorized access, anomalies)
    Security,
}

/// Workflow metric being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Unique identifier
    pub id: Uuid,

    /// Human-readable name
    pub name: String,

    /// Type of metric
    pub metric_type: MetricType,

    /// Current measured value
    pub current_value: f64,

    /// Expected/target value
    pub expected_value: f64,

    /// Unit of measurement
    pub unit: String,

    /// Threshold for anomaly detection
    pub anomaly_threshold: f64,

    /// Whether current value is anomalous
    pub is_anomalous: bool,

    /// Trend direction
    pub trend: TrendDirection,

    /// When metric was last updated
    pub timestamp: DateTime<Utc>,
}

/// Direction of metric trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Metric is improving
    Improving,
    /// Metric is degrading
    Degrading,
    /// Metric is stable
    Stable,
}

/// Type of event observed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// Task started
    TaskStarted,
    /// Task completed successfully
    TaskCompleted,
    /// Task failed
    TaskFailed,
    /// Task timeout
    TaskTimeout,
    /// Task slowdown detected
    TaskSlowdown,
    /// Data corruption detected
    DataCorruption,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Policy violation
    PolicyViolation,
    /// Unexpected behavior
    UnexpectedBehavior,
    /// Anomaly detected
    AnomalyDetected,
}

/// Severity of observation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Low severity (minor issue)
    Low,
    /// Medium severity (notable deviation)
    Medium,
    /// High severity (significant degradation)
    High,
    /// Critical severity (workflow may fail)
    Critical,
}

/// Observation of an event or state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier
    pub id: Uuid,

    /// When observation was made
    pub observed_at: DateTime<Utc>,

    /// Type of event
    pub event_type: EventType,

    /// Severity of observation
    pub severity: Severity,

    /// Element that was observed (task ID, condition ID, etc.)
    pub observed_element: String,

    /// Additional context
    pub context: serde_json::Value,
}

/// Type of analysis rule
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// Performance degradation detected
    PerformanceDegradation,
    /// High error rate detected
    HighErrorRate,
    /// Resource starvation detected
    ResourceStarvation,
    /// Deadlock risk detected
    DeadlockRisk,
    /// SLA violation detected
    SLAViolation,
    /// Security threat detected
    SecurityThreat,
    /// Data quality issue detected
    DataQualityIssue,
    /// Optimization opportunity detected
    OptimizationOpportunity,
}

/// Result of analyzing metrics and observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    /// Unique identifier
    pub id: Uuid,

    /// When analysis was performed
    pub timestamp: DateTime<Utc>,

    /// Problem identified
    pub problem: String,

    /// Root cause analysis
    pub root_cause: String,

    /// Affected elements
    pub affected_elements: Vec<String>,

    /// Recommended actions
    pub recommended_actions: Vec<Uuid>,

    /// Confidence in analysis (0.0-1.0)
    pub confidence: f64,

    /// Type of rule that matched
    pub rule_type: RuleType,
}

/// Type of autonomic action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Heal (fix or recover from failure)
    Heal,
    /// Optimize (improve performance or resource usage)
    Optimize,
    /// Configure (adapt to changing conditions)
    Configure,
    /// Protect (prevent security threats)
    Protect,
    /// Prevent (prevent predicted problems)
    Prevent,
}

/// Risk level of an action
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// No risk
    NoRisk,
    /// Low risk
    LowRisk,
    /// Medium risk
    MediumRisk,
    /// High risk (may require approval)
    HighRisk,
}

/// Action to be executed by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Unique identifier
    pub id: Uuid,

    /// Type of action
    pub action_type: ActionType,

    /// Human-readable description
    pub description: String,

    /// Target element (task, condition, etc.)
    pub target: String,

    /// Implementation reference (code, service, script)
    pub implementation: String,

    /// Estimated impact
    pub estimated_impact: String,

    /// Risk level
    pub risk_level: RiskLevel,
}

/// Autonomic policy (when to take actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique identifier
    pub id: Uuid,

    /// Policy name
    pub name: String,

    /// SPARQL query that triggers policy
    pub trigger: String,

    /// Actions to execute when triggered
    pub actions: Vec<Uuid>,

    /// Priority (higher = execute first)
    pub priority: i32,

    /// Additional condition (SPARQL WHERE clause)
    pub condition: Option<String>,
}

/// Execution plan (ordered sequence of actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Unique identifier
    pub id: Uuid,

    /// Ordered list of action IDs
    pub actions: Vec<Uuid>,

    /// Rationale for this plan
    pub rationale: String,

    /// Expected outcome
    pub expected_outcome: String,

    /// When plan was created
    pub created_at: DateTime<Utc>,
}

/// Status of action execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution successful
    Successful,
    /// Execution failed
    Failed,
    /// Partial success
    PartialSuccess,
    /// Execution cancelled
    Cancelled,
}

/// Record of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionExecution {
    /// Unique identifier
    pub id: Uuid,

    /// Action that was executed
    pub action_id: Uuid,

    /// When execution started
    pub start_time: DateTime<Utc>,

    /// When execution ended
    pub end_time: DateTime<Utc>,

    /// Execution status
    pub status: ExecutionStatus,

    /// Output from execution
    pub output: String,

    /// Error message if failed
    pub error: Option<String>,

    /// Metrics after execution
    pub metrics_after: Vec<Metric>,

    /// Analysis of what changed
    pub impact_analysis: String,
}

/// Complete MAPE-K feedback cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackCycle {
    /// Cycle sequence number
    pub cycle_number: u64,

    /// When cycle started
    pub start_time: DateTime<Utc>,

    /// When cycle completed
    pub end_time: DateTime<Utc>,

    /// Observations from monitor phase
    pub observations: Vec<Observation>,

    /// Analysis from analyze phase
    pub analysis: Option<Analysis>,

    /// Plan from plan phase
    pub plan: Option<Plan>,

    /// Executions from execute phase
    pub executions: Vec<ActionExecution>,

    /// Summary of cycle outcome
    pub outcome: String,

    /// How effective was this cycle (0.0-1.0)
    pub effectiveness: f64,
}

/// Pattern learned from experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Unique identifier
    pub id: Uuid,

    /// Pattern description
    pub description: String,

    /// How many times observed
    pub frequency: u64,

    /// Success rate (0.0-1.0)
    pub reliability: f64,

    /// Actions that work for this pattern
    pub associated_actions: Vec<Uuid>,

    /// When pattern was first learned
    pub first_seen: DateTime<Utc>,

    /// When pattern was last observed
    pub last_seen: DateTime<Utc>,
}

/// Memory of what actions worked in what situations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessMemory {
    /// Unique identifier
    pub id: Uuid,

    /// Situation description
    pub situation: String,

    /// Actions that succeeded
    pub successful_actions: Vec<Uuid>,

    /// Success rate (0.0-1.0)
    pub success_rate: f64,

    /// Number of times tried
    pub attempts: u64,

    /// Number of successes
    pub successes: u64,
}
