//! Gemba System - Real-Time Workflow Observation
//!
//! Implements Toyota Production System's Gemba concept:
//! - "Go to the real place" - observe workflow execution where it happens
//! - Real-time workflow state observation
//! - Execution context capture
//! - Performance metrics collection
//! - Integration with Andon for alerting
//!
//! # Gemba Principles
//!
//! 1. **Go to the real place**: Observe execution at the engine level
//! 2. **See the actual situation**: Capture real execution state
//! 3. **Understand the process**: Analyze workflow patterns
//! 4. **Take action**: Feed insights back to Andon system
//!
//! # TRIZ Principles Applied
//!
//! - **Principle 10: Prior Action**: Pre-configured observation points
//! - **Principle 15: Dynamics**: Real-time state capture
//! - **Principle 24: Intermediary**: Gemba as intermediary between execution and analysis

use crate::case::{Case, CaseId, CaseState};
use crate::error::WorkflowResult;
use crate::monitoring::andon::{AndonAlert, AndonAlertType, AndonState, AndonSystem};
use crate::parser::WorkflowSpecId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Gemba observation point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObservationPoint {
    /// Workflow execution start
    WorkflowStart,
    /// Task execution
    TaskExecution,
    /// Pattern execution
    PatternExecution,
    /// Guard evaluation
    GuardEvaluation,
    /// Case state transition
    CaseStateTransition,
    /// Workflow completion
    WorkflowCompletion,
    /// Error occurrence
    ErrorOccurrence,
}

/// Gemba observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GembaObservation {
    /// Observation point
    pub point: ObservationPoint,
    /// Workflow ID
    pub workflow_id: WorkflowSpecId,
    /// Case ID (if applicable)
    pub case_id: Option<CaseId>,
    /// Task ID (if applicable)
    pub task_id: Option<String>,
    /// Pattern ID (if applicable)
    pub pattern_id: Option<u32>,
    /// Observation timestamp (Unix epoch milliseconds)
    pub timestamp_ms: u64,
    /// Execution context
    pub context: ObservationContext,
    /// Performance metrics
    pub metrics: ObservationMetrics,
}

/// Observation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationContext {
    /// Case state (if applicable)
    pub case_state: Option<CaseState>,
    /// Variables at observation point
    pub variables: HashMap<String, String>,
    /// Execution stack depth
    pub stack_depth: usize,
    /// Active tasks
    pub active_tasks: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Observation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationMetrics {
    /// Execution latency (microseconds)
    pub latency_us: u64,
    /// Ticks used
    pub ticks_used: u32,
    /// Memory usage (bytes, if available)
    pub memory_bytes: Option<u64>,
    /// CPU usage (percentage, if available)
    pub cpu_percent: Option<f64>,
}

/// Gemba walker - observes workflow execution
pub struct GembaWalker {
    /// Andon system for alerting
    andon: Arc<AndonSystem>,
    /// Observation history
    observations: Arc<RwLock<Vec<GembaObservation>>>,
    /// Observation window (max observations to keep)
    max_observations: usize,
    /// Performance tracking
    performance_tracker: Arc<RwLock<PerformanceTracker>>,
}

/// Performance tracker
#[derive(Debug, Default)]
struct PerformanceTracker {
    /// Total observations
    total_observations: u64,
    /// Total latency (microseconds)
    total_latency_us: u64,
    /// Total ticks
    total_ticks: u64,
    /// Error count
    error_count: u64,
    /// Last observation time (Unix epoch milliseconds)
    last_observation_ms: Option<u64>,
}

impl GembaWalker {
    /// Create new gemba walker
    pub fn new(andon: Arc<AndonSystem>, max_observations: usize) -> Self {
        Self {
            andon,
            observations: Arc::new(RwLock::new(Vec::new())),
            max_observations,
            performance_tracker: Arc::new(RwLock::new(PerformanceTracker::default())),
        }
    }

    /// Observe workflow execution at a point
    pub async fn observe(
        &self,
        point: ObservationPoint,
        workflow_id: WorkflowSpecId,
        context: ObservationContext,
        metrics: ObservationMetrics,
    ) -> WorkflowResult<()> {
        let observation = GembaObservation {
            point,
            workflow_id,
            case_id: None, // Case ID not available in this context
            task_id: context.active_tasks.first().cloned(),
            pattern_id: None, // Would be set from context
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            context,
            metrics,
        };

        // Store observation
        {
            let mut observations = self.observations.write().await;
            observations.push(observation.clone());

            // Trim to max size
            if observations.len() > self.max_observations {
                observations.remove(0);
            }
        }

        // Update performance tracker
        {
            let mut tracker = self.performance_tracker.write().await;
            tracker.total_observations += 1;
            tracker.total_latency_us += metrics.latency_us;
            tracker.total_ticks += metrics.ticks_used as u64;
            tracker.last_observation_ms = Some(observation.timestamp_ms);
        }

        // Check for issues and alert via Andon
        self.check_and_alert(&observation).await?;

        debug!("GEMBA observation: {:?} at workflow {}", point, workflow_id);

        Ok(())
    }

    /// Check observation for issues and alert via Andon
    async fn check_and_alert(&self, observation: &GembaObservation) -> WorkflowResult<()> {
        // Check tick budget
        if observation.metrics.ticks_used > 8 {
            self.andon
                .check_tick_budget(observation.metrics.ticks_used)
                .await?;
        }

        // Check latency
        if observation.metrics.latency_us > 1_000_000 {
            // Critical latency (>1s)
            self.andon
                .raise_alert(
                    AndonAlert::new(
                        AndonAlertType::PerformanceDegradation,
                        AndonState::Red,
                        format!(
                            "Critical latency observed: {}μs at {:?}",
                            observation.metrics.latency_us, observation.point
                        ),
                    )
                    .with_context(
                        "latency_us".to_string(),
                        observation.metrics.latency_us.to_string(),
                    )
                    .with_context(
                        "observation_point".to_string(),
                        format!("{:?}", observation.point),
                    ),
                )
                .await?;
        }

        // Check for error states
        if let Some(ref case_state) = observation.context.case_state {
            if matches!(case_state, CaseState::Failed | CaseState::Cancelled) {
                self.andon
                    .raise_alert(
                        AndonAlert::new(
                            AndonAlertType::HighErrorRate,
                            AndonState::Yellow,
                            format!("Case in error state: {:?}", case_state),
                        )
                        .with_case(observation.case_id.unwrap_or_else(
                            || {
                                // Create a temporary case ID for alerting
                                CaseId::default()
                            },
                        )),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    /// Get recent observations
    pub async fn get_recent_observations(&self, count: usize) -> Vec<GembaObservation> {
        let observations = self.observations.read().await;
        observations.iter().rev().take(count).cloned().collect()
    }

    /// Get observations by point
    pub async fn get_observations_by_point(
        &self,
        point: ObservationPoint,
    ) -> Vec<GembaObservation> {
        self.observations
            .read()
            .await
            .iter()
            .filter(|o| o.point == point)
            .cloned()
            .collect()
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let tracker = self.performance_tracker.read().await;
        let observations = self.observations.read().await;

        let avg_latency_us = if tracker.total_observations > 0 {
            tracker.total_latency_us / tracker.total_observations
        } else {
            0
        };

        let avg_ticks = if tracker.total_observations > 0 {
            (tracker.total_ticks / tracker.total_observations) as u32
        } else {
            0
        };

        PerformanceSummary {
            total_observations: tracker.total_observations,
            avg_latency_us,
            avg_ticks,
            error_count: tracker.error_count,
            recent_observations: observations.len(),
            last_observation_ms: tracker.last_observation_ms,
        }
    }

    /// Analyze workflow execution patterns
    pub async fn analyze_patterns(&self) -> WorkflowResult<PatternAnalysis> {
        let observations = self.observations.read().await;

        // Count observations by point
        let mut point_counts: HashMap<ObservationPoint, usize> = HashMap::new();
        for obs in observations.iter() {
            *point_counts.entry(obs.point).or_insert(0) += 1;
        }

        // Calculate average metrics by point
        let mut point_metrics: HashMap<ObservationPoint, (u64, u32)> = HashMap::new();
        for obs in observations.iter() {
            let entry = point_metrics.entry(obs.point).or_insert((0, 0));
            entry.0 += obs.metrics.latency_us;
            entry.1 += obs.metrics.ticks_used;
        }

        // Calculate averages
        let mut avg_metrics: HashMap<ObservationPoint, (u64, u32)> = HashMap::new();
        for (point, (total_latency, total_ticks)) in point_metrics {
            let count = point_counts.get(&point).copied().unwrap_or(1);
            avg_metrics.insert(
                point,
                (total_latency / count as u64, total_ticks / count as u32),
            );
        }

        Ok(PatternAnalysis {
            point_counts,
            avg_metrics,
            total_observations: observations.len(),
        })
    }
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total observations
    pub total_observations: u64,
    /// Average latency (microseconds)
    pub avg_latency_us: u64,
    /// Average ticks
    pub avg_ticks: u32,
    /// Error count
    pub error_count: u64,
    /// Recent observations count
    pub recent_observations: usize,
    /// Last observation time (Unix epoch milliseconds)
    pub last_observation_ms: Option<u64>,
}

/// Pattern analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Observation counts by point
    pub point_counts: HashMap<ObservationPoint, usize>,
    /// Average metrics by point
    pub avg_metrics: HashMap<ObservationPoint, (u64, u32)>,
    /// Total observations analyzed
    pub total_observations: usize,
}
