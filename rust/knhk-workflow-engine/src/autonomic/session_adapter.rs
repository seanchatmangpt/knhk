// rust/knhk-workflow-engine/src/autonomic/session_adapter.rs
//! Session-Local Adaptation
//!
//! Implements per-session adaptation logic while respecting global Q (doctrine).
//!
//! **Key Principles**:
//! - Session decisions are **isolated** from other sessions
//! - Local adaptations **never violate** global Q
//! - Session-scoped actions are **reversible** and **safe**
//! - Aggregation to global MAPE-K preserves consistency
//!
//! **Adaptation Flow**:
//! 1. Monitor session-local metrics
//! 2. Analyze against session-local thresholds and global Q
//! 3. Plan session-scoped actions (subset of global actions)
//! 4. Execute adaptations only if global Q is satisfied
//! 5. Emit session-level events for global aggregation

use super::knowledge::KnowledgeBase;
use super::plan::{Action, ActionType, AdaptationPlan};
use super::session::{SessionHandle, SessionId, SessionMetricsSnapshot, SessionState};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session-scoped action (subset of global actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionAction {
    /// Retry failed task
    RetryTask { task_id: String, backoff_ms: u64 },
    /// Switch to degraded mode
    DegradePerformance { factor: f64 },
    /// Request additional resources (subject to global approval)
    RequestResources { amount: f64 },
    /// Cancel non-critical tasks
    CancelOptionalTasks,
    /// Activate compensation handlers
    TriggerCompensation { scope: String },
    /// Log warning and continue
    LogAndContinue { message: String },
}

/// Session adaptation decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDecision {
    /// Session ID
    pub session_id: SessionId,
    /// Decision timestamp
    pub timestamp_ms: u64,
    /// Session action to execute
    pub action: SessionAction,
    /// Reason for adaptation
    pub reason: String,
    /// Expected local impact
    pub expected_impact: f64,
    /// Whether global Q was checked
    pub global_q_verified: bool,
}

/// Session-level event (for global aggregation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// Session started
    Started {
        session_id: SessionId,
        pattern_id: Option<u8>,
    },
    /// Session completed
    Completed {
        session_id: SessionId,
        duration_ms: u64,
        task_count: u64,
    },
    /// Session failed
    Failed {
        session_id: SessionId,
        error: String,
    },
    /// Session adapted
    Adapted {
        session_id: SessionId,
        action: SessionAction,
        reason: String,
    },
    /// Violation detected
    ViolationDetected {
        session_id: SessionId,
        metric: String,
        value: f64,
        threshold: f64,
    },
    /// Threshold exceeded
    ThresholdExceeded {
        session_id: SessionId,
        metric: String,
        current: f64,
        limit: f64,
    },
}

/// Session adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAdapterConfig {
    /// Maximum retry count before escalation
    pub max_retries: u64,
    /// Maximum violation rate before adaptation (0.0-1.0)
    pub max_violation_rate: f64,
    /// Maximum latency (microseconds) before adaptation
    pub max_latency_us: u64,
    /// Enable automatic degradation
    pub enable_auto_degrade: bool,
    /// Enable global Q verification (should always be true in production)
    pub verify_global_q: bool,
}

impl Default for SessionAdapterConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            max_violation_rate: 0.1, // 10% violation rate threshold
            max_latency_us: 100_000,  // 100ms
            enable_auto_degrade: true,
            verify_global_q: true,
        }
    }
}

/// Global Q (doctrine) - invariants that must never be violated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQ {
    /// Maximum total resource usage (0.0-1.0)
    pub max_total_resources: f64,
    /// Maximum concurrent adaptations
    pub max_concurrent_adaptations: u64,
    /// Minimum SLO compliance rate (0.0-1.0)
    pub min_slo_compliance: f64,
    /// Maximum failure rate (0.0-1.0)
    pub max_failure_rate: f64,
}

impl Default for GlobalQ {
    fn default() -> Self {
        Self {
            max_total_resources: 0.9,      // 90% max resource usage
            max_concurrent_adaptations: 10, // Max 10 simultaneous adaptations
            min_slo_compliance: 0.95,       // 95% SLO compliance
            max_failure_rate: 0.05,         // 5% max failure rate
        }
    }
}

/// Session adapter (per-session adaptation logic)
pub struct SessionAdapter {
    /// Configuration
    config: SessionAdapterConfig,
    /// Global Q (doctrine)
    global_q: Arc<RwLock<GlobalQ>>,
    /// Knowledge base (for global context)
    knowledge: Arc<KnowledgeBase>,
    /// Session event buffer (for aggregation)
    event_buffer: Arc<RwLock<Vec<SessionEvent>>>,
    /// Active adaptations count
    active_adaptations: Arc<RwLock<u64>>,
    /// Decision history (per session)
    decision_history: Arc<RwLock<HashMap<SessionId, Vec<SessionDecision>>>>,
}

impl SessionAdapter {
    /// Create new session adapter
    pub fn new(knowledge: Arc<KnowledgeBase>) -> Self {
        Self {
            config: SessionAdapterConfig::default(),
            global_q: Arc::new(RwLock::new(GlobalQ::default())),
            knowledge,
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            active_adaptations: Arc::new(RwLock::new(0)),
            decision_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: SessionAdapterConfig, knowledge: Arc<KnowledgeBase>) -> Self {
        Self {
            config,
            global_q: Arc::new(RwLock::new(GlobalQ::default())),
            knowledge,
            event_buffer: Arc::new(RwLock::new(Vec::new())),
            active_adaptations: Arc::new(RwLock::new(0)),
            decision_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update global Q
    pub async fn update_global_q(&self, q: GlobalQ) {
        let mut global_q = self.global_q.write().await;
        *global_q = q;
    }

    /// Analyze session and generate local decision
    pub async fn analyze_session(
        &self,
        handle: &SessionHandle,
    ) -> WorkflowResult<Option<SessionDecision>> {
        let snapshot = handle.snapshot();

        // Check if session needs adaptation
        if let Some(action) = self.should_adapt(&snapshot).await? {
            // Verify global Q before adapting
            if self.config.verify_global_q {
                if !self.check_global_q().await? {
                    // Global Q violation - cannot adapt
                    tracing::warn!(
                        session_id = %handle.id,
                        "Cannot adapt session: global Q would be violated"
                    );
                    return Ok(None);
                }
            }

            let decision = SessionDecision {
                session_id: handle.id,
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                action: action.clone(),
                reason: self.get_adaptation_reason(&snapshot, &action),
                expected_impact: 0.5, // Estimated impact
                global_q_verified: self.config.verify_global_q,
            };

            // Record decision
            let mut history = self.decision_history.write().await;
            history
                .entry(handle.id)
                .or_insert_with(Vec::new)
                .push(decision.clone());

            // Emit adaptation event
            self.emit_event(SessionEvent::Adapted {
                session_id: handle.id,
                action: action.clone(),
                reason: decision.reason.clone(),
            })
            .await;

            Ok(Some(decision))
        } else {
            Ok(None)
        }
    }

    /// Execute session decision
    pub async fn execute_decision(
        &self,
        decision: &SessionDecision,
        handle: &SessionHandle,
    ) -> WorkflowResult<()> {
        // Increment active adaptations
        let mut active = self.active_adaptations.write().await;
        *active += 1;
        drop(active);

        // Execute action
        match &decision.action {
            SessionAction::RetryTask { task_id, backoff_ms } => {
                tracing::info!(
                    session_id = %decision.session_id,
                    task_id = %task_id,
                    backoff_ms = backoff_ms,
                    "Retrying task"
                );
                handle.record_retry();
            }
            SessionAction::DegradePerformance { factor } => {
                tracing::info!(
                    session_id = %decision.session_id,
                    factor = factor,
                    "Degrading performance"
                );
                handle.mark_adapted();
            }
            SessionAction::RequestResources { amount } => {
                tracing::info!(
                    session_id = %decision.session_id,
                    amount = amount,
                    "Requesting additional resources"
                );
            }
            SessionAction::CancelOptionalTasks => {
                tracing::info!(
                    session_id = %decision.session_id,
                    "Cancelling optional tasks"
                );
                handle.mark_adapted();
            }
            SessionAction::TriggerCompensation { scope } => {
                tracing::info!(
                    session_id = %decision.session_id,
                    scope = %scope,
                    "Triggering compensation"
                );
            }
            SessionAction::LogAndContinue { message } => {
                tracing::warn!(
                    session_id = %decision.session_id,
                    message = %message,
                    "Session warning"
                );
            }
        }

        // Decrement active adaptations
        let mut active = self.active_adaptations.write().await;
        *active = active.saturating_sub(1);

        Ok(())
    }

    /// Check if session should adapt
    async fn should_adapt(
        &self,
        snapshot: &SessionMetricsSnapshot,
    ) -> WorkflowResult<Option<SessionAction>> {
        // Check retry threshold
        if snapshot.retry_count > self.config.max_retries {
            return Ok(Some(SessionAction::CancelOptionalTasks));
        }

        // Check violation rate
        if snapshot.violation_rate() > self.config.max_violation_rate {
            if self.config.enable_auto_degrade {
                return Ok(Some(SessionAction::DegradePerformance { factor: 0.8 }));
            }
        }

        // Check latency
        if let Some(avg_latency) = snapshot.avg_latency_us() {
            if avg_latency > self.config.max_latency_us {
                return Ok(Some(SessionAction::RequestResources { amount: 0.2 }));
            }
        }

        // Check state-specific conditions
        match snapshot.state {
            SessionState::Failed => {
                return Ok(Some(SessionAction::TriggerCompensation {
                    scope: "session".to_string(),
                }));
            }
            _ => {}
        }

        Ok(None)
    }

    /// Check global Q (doctrine)
    async fn check_global_q(&self) -> WorkflowResult<bool> {
        let global_q = self.global_q.read().await;
        let active_adaptations = *self.active_adaptations.read().await;

        // Check concurrent adaptations limit
        if active_adaptations >= global_q.max_concurrent_adaptations {
            return Ok(false);
        }

        // Check SLO compliance (from knowledge base)
        let facts = self.knowledge.get_facts().await;
        if let Some(slo_fact) = facts.get("slo_compliance_rate") {
            if slo_fact.value < global_q.min_slo_compliance {
                return Ok(false);
            }
        }

        // Check failure rate
        if let Some(failure_fact) = facts.get("failure_rate") {
            if failure_fact.value > global_q.max_failure_rate {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get adaptation reason
    fn get_adaptation_reason(
        &self,
        snapshot: &SessionMetricsSnapshot,
        action: &SessionAction,
    ) -> String {
        match action {
            SessionAction::RetryTask { .. } => {
                format!("Retry count exceeded: {}", snapshot.retry_count)
            }
            SessionAction::DegradePerformance { .. } => {
                format!(
                    "Violation rate too high: {:.2}%",
                    snapshot.violation_rate() * 100.0
                )
            }
            SessionAction::RequestResources { .. } => {
                format!(
                    "Latency too high: {}μs",
                    snapshot.avg_latency_us().unwrap_or(0)
                )
            }
            SessionAction::CancelOptionalTasks => {
                format!("Too many retries: {}", snapshot.retry_count)
            }
            SessionAction::TriggerCompensation { .. } => "Session failed".to_string(),
            SessionAction::LogAndContinue { message } => message.clone(),
        }
    }

    /// Emit session event (for global aggregation)
    async fn emit_event(&self, event: SessionEvent) {
        let mut buffer = self.event_buffer.write().await;
        buffer.push(event);
    }

    /// Drain events for aggregation
    pub async fn drain_events(&self) -> Vec<SessionEvent> {
        let mut buffer = self.event_buffer.write().await;
        std::mem::take(&mut *buffer)
    }

    /// Get decision history for session
    pub async fn get_session_history(&self, session_id: SessionId) -> Vec<SessionDecision> {
        let history = self.decision_history.read().await;
        history.get(&session_id).cloned().unwrap_or_default()
    }

    /// Clear decision history for completed sessions
    pub async fn clear_completed_sessions(&self, session_ids: &[SessionId]) {
        let mut history = self.decision_history.write().await;
        for id in session_ids {
            history.remove(id);
        }
    }

    /// Get statistics
    pub async fn stats(&self) -> SessionAdapterStats {
        let active_adaptations = *self.active_adaptations.read().await;
        let history = self.decision_history.read().await;
        let events_pending = self.event_buffer.read().await.len();

        let total_decisions: usize = history.values().map(|v| v.len()).sum();

        SessionAdapterStats {
            active_adaptations,
            total_decisions,
            tracked_sessions: history.len(),
            pending_events: events_pending,
        }
    }
}

/// Session adapter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAdapterStats {
    pub active_adaptations: u64,
    pub total_decisions: usize,
    pub tracked_sessions: usize,
    pub pending_events: usize,
}

/// Session aggregator (rolls up session metrics to global MAPE-K)
pub struct SessionAggregator {
    /// Knowledge base
    knowledge: Arc<KnowledgeBase>,
}

impl SessionAggregator {
    /// Create new aggregator
    pub fn new(knowledge: Arc<KnowledgeBase>) -> Self {
        Self { knowledge }
    }

    /// Aggregate session metrics to global knowledge base
    pub async fn aggregate_sessions(
        &self,
        sessions: &[SessionHandle],
    ) -> WorkflowResult<AggregatedMetrics> {
        let mut total_latency_us = 0u64;
        let mut total_completions = 0u64;
        let mut total_violations = 0u64;
        let mut total_retries = 0u64;
        let mut total_adaptations = 0u64;
        let mut active_count = 0usize;
        let mut completed_count = 0usize;
        let mut failed_count = 0usize;

        for session in sessions {
            let snapshot = session.snapshot();

            total_latency_us += snapshot.total_latency_us;
            total_completions += snapshot.task_completions;
            total_violations += snapshot.violation_count;
            total_retries += snapshot.retry_count;
            total_adaptations += snapshot.adaptation_count;

            match snapshot.state {
                SessionState::Active | SessionState::Adapted => active_count += 1,
                SessionState::Completed => completed_count += 1,
                SessionState::Failed => failed_count += 1,
                _ => {}
            }
        }

        // Calculate aggregated metrics
        let avg_latency_us = if total_completions > 0 {
            total_latency_us / total_completions
        } else {
            0
        };

        let violation_rate = if total_completions > 0 {
            total_violations as f64 / total_completions as f64
        } else {
            0.0
        };

        let retry_rate = if total_completions > 0 {
            total_retries as f64 / total_completions as f64
        } else {
            0.0
        };

        let failure_rate = if sessions.len() > 0 {
            failed_count as f64 / sessions.len() as f64
        } else {
            0.0
        };

        let adaptation_rate = if sessions.len() > 0 {
            total_adaptations as f64 / sessions.len() as f64
        } else {
            0.0
        };

        // Update knowledge base with aggregated facts
        self.knowledge
            .add_fact(super::knowledge::Fact::new(
                "session_avg_latency_us".to_string(),
                avg_latency_us as f64,
                "session_aggregator".to_string(),
            ))
            .await?;

        self.knowledge
            .add_fact(super::knowledge::Fact::new(
                "session_violation_rate".to_string(),
                violation_rate,
                "session_aggregator".to_string(),
            ))
            .await?;

        self.knowledge
            .add_fact(super::knowledge::Fact::new(
                "session_failure_rate".to_string(),
                failure_rate,
                "session_aggregator".to_string(),
            ))
            .await?;

        Ok(AggregatedMetrics {
            total_sessions: sessions.len(),
            active_sessions: active_count,
            completed_sessions: completed_count,
            failed_sessions: failed_count,
            avg_latency_us,
            violation_rate,
            retry_rate,
            failure_rate,
            adaptation_rate,
            total_completions,
        })
    }
}

/// Aggregated metrics (global view from all sessions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub completed_sessions: usize,
    pub failed_sessions: usize,
    pub avg_latency_us: u64,
    pub violation_rate: f64,
    pub retry_rate: f64,
    pub failure_rate: f64,
    pub adaptation_rate: f64,
    pub total_completions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::session::{CaseId, SessionTable, TenantId};
    use std::time::Duration;

    #[tokio::test]
    async fn test_session_adapter() {
        let kb = Arc::new(KnowledgeBase::new());
        let adapter = SessionAdapter::new(kb.clone());

        let table = SessionTable::new();
        let case_id = CaseId::new();
        let handle = table.create_session(case_id, TenantId::default_tenant());

        handle.start();

        // Trigger adaptation by exceeding retry threshold
        for _ in 0..5 {
            handle.record_retry();
        }

        let decision = adapter.analyze_session(&handle).await.unwrap();
        assert!(decision.is_some());

        let decision = decision.unwrap();
        assert!(matches!(
            decision.action,
            SessionAction::CancelOptionalTasks
        ));
    }

    #[tokio::test]
    async fn test_global_q_enforcement() {
        let kb = Arc::new(KnowledgeBase::new());
        let adapter = SessionAdapter::new(kb.clone());

        // Set strict global Q
        let mut q = GlobalQ::default();
        q.max_concurrent_adaptations = 0; // No adaptations allowed
        adapter.update_global_q(q).await;

        let table = SessionTable::new();
        let case_id = CaseId::new();
        let handle = table.create_session(case_id, TenantId::default_tenant());

        handle.start();

        // Trigger adaptation
        for _ in 0..5 {
            handle.record_retry();
        }

        // Should be rejected by global Q
        let decision = adapter.analyze_session(&handle).await.unwrap();
        assert!(decision.is_none());
    }

    #[tokio::test]
    async fn test_session_aggregation() {
        let kb = Arc::new(KnowledgeBase::new());
        let aggregator = SessionAggregator::new(kb.clone());

        let table = SessionTable::new();

        // Create multiple sessions
        let mut handles = Vec::new();
        for _ in 0..10 {
            let case_id = CaseId::new();
            let handle = table.create_session(case_id, TenantId::default_tenant());
            handle.start();
            handle.record_task_execution(Duration::from_micros(1000));
            handle.record_violation();
            handles.push(handle);
        }

        // Complete some sessions
        handles[0].complete();
        handles[1].complete();
        handles[2].fail();

        // Aggregate
        let metrics = aggregator.aggregate_sessions(&handles).await.unwrap();

        assert_eq!(metrics.total_sessions, 10);
        assert!(metrics.active_sessions > 0);
        assert_eq!(metrics.completed_sessions, 2);
        assert_eq!(metrics.failed_sessions, 1);
        assert!(metrics.avg_latency_us > 0);
    }

    #[tokio::test]
    async fn test_session_event_emission() {
        let kb = Arc::new(KnowledgeBase::new());
        let adapter = SessionAdapter::new(kb.clone());

        let table = SessionTable::new();
        let case_id = CaseId::new();
        let handle = table.create_session(case_id, TenantId::default_tenant());

        handle.start();

        // Trigger adaptation
        for _ in 0..5 {
            handle.record_retry();
        }

        let decision = adapter.analyze_session(&handle).await.unwrap().unwrap();
        adapter.execute_decision(&decision, &handle).await.unwrap();

        // Check events were emitted
        let events = adapter.drain_events().await;
        assert!(!events.is_empty());
    }
}
