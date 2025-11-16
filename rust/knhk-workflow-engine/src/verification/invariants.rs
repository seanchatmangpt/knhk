//! Runtime Invariant Checking
//!
//! Validates that system invariants hold during execution:
//! - Session isolation (no cross-session data leakage)
//! - Policy consistency (lattice laws preserved)
//! - Mode safety (actions in Frozen mode cannot execute)
//! - Trace determinism (same TraceId → same execution)
//!
//! **Design Principles**:
//! - Fast-path optimization: cached invariants check in < 1ms
//! - Fail-fast: invariant violations halt execution immediately
//! - Observable: all violations emit telemetry
//! - Recoverable: violations trigger autonomic adaptation

use crate::autonomic::failure_modes::AutonomicMode;
use crate::autonomic::policy_lattice::{Lattice, PolicyElement};
use crate::autonomic::session::SessionId;
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Invariant definition
pub trait Invariant: Send + Sync {
    /// Invariant name (for logging)
    fn name(&self) -> &str;

    /// Check if invariant holds
    fn check(&self, context: &InvariantContext) -> WorkflowResult<bool>;

    /// Get severity level
    fn severity(&self) -> InvariantSeverity {
        InvariantSeverity::Critical
    }

    /// Whether to fail-fast on violation
    fn fail_fast(&self) -> bool {
        self.severity() == InvariantSeverity::Critical
    }
}

/// Invariant severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvariantSeverity {
    /// Warning only - log but continue
    Warning,
    /// Error - log and trigger adaptation
    Error,
    /// Critical - log and halt execution
    Critical,
}

/// Invariant checking context
#[derive(Debug, Clone)]
pub struct InvariantContext {
    /// Current session ID
    pub session_id: Option<SessionId>,
    /// Current autonomic mode
    pub mode: AutonomicMode,
    /// Active policy
    pub policy: Option<PolicyElement>,
    /// Session data (for isolation checks)
    pub session_data: HashMap<SessionId, Vec<u8>>,
    /// Trace ID
    pub trace_id: Option<String>,
    /// Timestamp
    pub timestamp_ms: u64,
}

impl InvariantContext {
    /// Create new context
    pub fn new() -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            session_id: None,
            mode: AutonomicMode::Normal,
            policy: None,
            session_data: HashMap::new(),
            trace_id: None,
            timestamp_ms,
        }
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set mode
    pub fn with_mode(mut self, mode: AutonomicMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set policy
    pub fn with_policy(mut self, policy: PolicyElement) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Set trace ID
    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

impl Default for InvariantContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Session isolation invariant
///
/// Ensures no cross-session data leakage
pub struct SessionInvariant {
    /// Allowed session IDs
    allowed_sessions: HashSet<SessionId>,
}

impl SessionInvariant {
    pub fn new(allowed_sessions: HashSet<SessionId>) -> Self {
        Self { allowed_sessions }
    }
}

impl Invariant for SessionInvariant {
    fn name(&self) -> &str {
        "session_isolation"
    }

    fn check(&self, context: &InvariantContext) -> WorkflowResult<bool> {
        if let Some(session_id) = &context.session_id {
            // Check session is allowed
            if !self.allowed_sessions.contains(session_id) {
                return Ok(false);
            }

            // Check no data leakage to other sessions
            for (sid, _) in &context.session_data {
                if sid != session_id && !self.allowed_sessions.contains(sid) {
                    tracing::warn!(
                        current_session = ?session_id,
                        leaked_session = ?sid,
                        "Session isolation violation detected"
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn severity(&self) -> InvariantSeverity {
        InvariantSeverity::Critical
    }
}

/// Policy consistency invariant
///
/// Verifies lattice laws are preserved
pub struct PolicyConsistencyInvariant;

impl Invariant for PolicyConsistencyInvariant {
    fn name(&self) -> &str {
        "policy_consistency"
    }

    fn check(&self, context: &InvariantContext) -> WorkflowResult<bool> {
        if let Some(policy) = &context.policy {
            // Check idempotence: a ⊓ a = a
            let meet_self = policy.meet(policy);
            if policy.partial_cmp_lattice(&meet_self) != Some(std::cmp::Ordering::Equal) {
                tracing::warn!(
                    policy = ?policy,
                    "Policy lattice idempotence violated: a ⊓ a ≠ a"
                );
                return Ok(false);
            }

            // Check join idempotence: a ⊔ a = a
            let join_self = policy.join(policy);
            if policy.partial_cmp_lattice(&join_self) != Some(std::cmp::Ordering::Equal) {
                tracing::warn!(
                    policy = ?policy,
                    "Policy lattice idempotence violated: a ⊔ a ≠ a"
                );
                return Ok(false);
            }

            // Check bottom absorption: a ⊓ ⊥ = ⊥
            let bottom = PolicyElement::bottom();
            let meet_bottom = policy.meet(&bottom);
            if !meet_bottom.is_bottom() {
                tracing::warn!(
                    policy = ?policy,
                    "Policy lattice absorption violated: a ⊓ ⊥ ≠ ⊥"
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn severity(&self) -> InvariantSeverity {
        InvariantSeverity::Critical
    }
}

/// Mode safety invariant
///
/// Ensures no actions execute in Frozen mode
pub struct ModeSafetyInvariant {
    /// Whether any action was attempted
    action_attempted: bool,
}

impl ModeSafetyInvariant {
    pub fn new(action_attempted: bool) -> Self {
        Self { action_attempted }
    }
}

impl Invariant for ModeSafetyInvariant {
    fn name(&self) -> &str {
        "mode_safety"
    }

    fn check(&self, context: &InvariantContext) -> WorkflowResult<bool> {
        if context.mode == AutonomicMode::Frozen && self.action_attempted {
            tracing::error!(
                mode = ?context.mode,
                "Mode safety violation: action attempted in Frozen mode"
            );
            return Ok(false);
        }

        Ok(true)
    }

    fn severity(&self) -> InvariantSeverity {
        InvariantSeverity::Critical
    }
}

/// Trace determinism invariant
///
/// Ensures same TraceId produces same execution
pub struct TraceDeterminismInvariant {
    /// Historical trace executions
    trace_history: HashMap<String, Vec<String>>,
}

impl TraceDeterminismInvariant {
    pub fn new() -> Self {
        Self {
            trace_history: HashMap::new(),
        }
    }

    /// Record execution for trace
    pub fn record_execution(&mut self, trace_id: String, execution_hash: String) {
        self.trace_history
            .entry(trace_id)
            .or_insert_with(Vec::new)
            .push(execution_hash);
    }

    /// Check if trace has consistent executions
    pub fn is_consistent(&self, trace_id: &str) -> bool {
        if let Some(executions) = self.trace_history.get(trace_id) {
            if executions.len() > 1 {
                // All executions should be identical
                let first = &executions[0];
                return executions.iter().all(|e| e == first);
            }
        }
        true
    }
}

impl Default for TraceDeterminismInvariant {
    fn default() -> Self {
        Self::new()
    }
}

impl Invariant for TraceDeterminismInvariant {
    fn name(&self) -> &str {
        "trace_determinism"
    }

    fn check(&self, context: &InvariantContext) -> WorkflowResult<bool> {
        if let Some(trace_id) = &context.trace_id {
            if !self.is_consistent(trace_id) {
                tracing::warn!(
                    trace_id = %trace_id,
                    "Trace determinism violation: inconsistent executions"
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn severity(&self) -> InvariantSeverity {
        InvariantSeverity::Error
    }
}

/// Runtime invariant
///
/// Generic runtime check
pub struct RuntimeInvariant<F>
where
    F: Fn(&InvariantContext) -> WorkflowResult<bool> + Send + Sync,
{
    name: String,
    check_fn: F,
    severity: InvariantSeverity,
}

impl<F> RuntimeInvariant<F>
where
    F: Fn(&InvariantContext) -> WorkflowResult<bool> + Send + Sync,
{
    pub fn new(name: String, severity: InvariantSeverity, check_fn: F) -> Self {
        Self {
            name,
            check_fn,
            severity,
        }
    }
}

impl<F> Invariant for RuntimeInvariant<F>
where
    F: Fn(&InvariantContext) -> WorkflowResult<bool> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self, context: &InvariantContext) -> WorkflowResult<bool> {
        (self.check_fn)(context)
    }

    fn severity(&self) -> InvariantSeverity {
        self.severity
    }
}

/// Invariant violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantViolation {
    /// Invariant name
    pub invariant: String,
    /// Severity
    pub severity: InvariantSeverity,
    /// Violation message
    pub message: String,
    /// Session ID (if applicable)
    pub session_id: Option<SessionId>,
    /// Trace ID (if applicable)
    pub trace_id: Option<String>,
    /// Timestamp
    pub timestamp_ms: u64,
}

impl InvariantViolation {
    pub fn new(invariant: String, severity: InvariantSeverity, message: String) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            invariant,
            severity,
            message,
            session_id: None,
            trace_id: None,
            timestamp_ms,
        }
    }

    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/// Invariant checker
pub struct InvariantChecker {
    /// Registered invariants
    invariants: Vec<Arc<dyn Invariant>>,
    /// Violation history
    violations: Vec<InvariantViolation>,
    /// Maximum violations to track
    max_violations: usize,
}

impl InvariantChecker {
    /// Create new checker
    pub fn new() -> Self {
        Self {
            invariants: Vec::new(),
            violations: Vec::new(),
            max_violations: 1000,
        }
    }

    /// Register invariant
    pub fn register<I: Invariant + 'static>(&mut self, invariant: I) {
        self.invariants.push(Arc::new(invariant));
    }

    /// Check all invariants
    pub fn check_all(&mut self, context: &InvariantContext) -> WorkflowResult<()> {
        let start = Instant::now();
        let mut critical_failures = Vec::new();

        for invariant in &self.invariants {
            match invariant.check(context) {
                Ok(true) => {
                    tracing::trace!(invariant = %invariant.name(), "Invariant check passed");
                }
                Ok(false) => {
                    let violation = InvariantViolation::new(
                        invariant.name().to_string(),
                        invariant.severity(),
                        format!("Invariant '{}' violated", invariant.name()),
                    );

                    tracing::warn!(
                        invariant = %invariant.name(),
                        severity = ?invariant.severity(),
                        "Invariant violation detected"
                    );

                    if invariant.fail_fast() {
                        critical_failures.push(violation.clone());
                    }

                    self.record_violation(violation);
                }
                Err(e) => {
                    tracing::error!(
                        invariant = %invariant.name(),
                        error = %e,
                        "Invariant check failed with error"
                    );

                    if invariant.fail_fast() {
                        return Err(e);
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        tracing::debug!(
            invariants_checked = self.invariants.len(),
            duration_ms = duration_ms,
            violations = critical_failures.len(),
            "Invariant check completed"
        );

        if !critical_failures.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "{} critical invariant violations",
                critical_failures.len()
            )));
        }

        Ok(())
    }

    /// Record violation
    fn record_violation(&mut self, violation: InvariantViolation) {
        self.violations.push(violation);

        // Trim history
        if self.violations.len() > self.max_violations {
            self.violations
                .drain(0..self.violations.len() - self.max_violations);
        }
    }

    /// Get recent violations
    pub fn get_violations(&self, limit: usize) -> &[InvariantViolation] {
        let start = self.violations.len().saturating_sub(limit);
        &self.violations[start..]
    }

    /// Get violation count
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }

    /// Clear violations
    pub fn clear_violations(&mut self) {
        self.violations.clear();
    }

    /// Get registered invariant count
    pub fn invariant_count(&self) -> usize {
        self.invariants.len()
    }
}

impl Default for InvariantChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_invariant() {
        let session_id = SessionId::new();
        let mut allowed = HashSet::new();
        allowed.insert(session_id);

        let invariant = SessionInvariant::new(allowed);
        let context = InvariantContext::new().with_session(session_id);

        assert!(invariant.check(&context).unwrap());
    }

    #[test]
    fn test_session_invariant_violation() {
        let allowed_session = SessionId::new();
        let forbidden_session = SessionId::new();

        let mut allowed = HashSet::new();
        allowed.insert(allowed_session);

        let invariant = SessionInvariant::new(allowed);
        let context = InvariantContext::new().with_session(forbidden_session);

        assert!(!invariant.check(&context).unwrap());
    }

    #[test]
    fn test_policy_consistency_invariant() {
        use crate::autonomic::policy_lattice::{LatencyBound, Strictness};

        let invariant = PolicyConsistencyInvariant;
        let policy = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Hard).unwrap());
        let context = InvariantContext::new().with_policy(policy);

        assert!(invariant.check(&context).unwrap());
    }

    #[test]
    fn test_mode_safety_invariant() {
        let invariant = ModeSafetyInvariant::new(true); // Action attempted
        let context = InvariantContext::new().with_mode(AutonomicMode::Frozen);

        assert!(!invariant.check(&context).unwrap());
    }

    #[test]
    fn test_mode_safety_invariant_normal() {
        let invariant = ModeSafetyInvariant::new(true); // Action attempted
        let context = InvariantContext::new().with_mode(AutonomicMode::Normal);

        assert!(invariant.check(&context).unwrap());
    }

    #[test]
    fn test_runtime_invariant() {
        let invariant = RuntimeInvariant::new(
            "test_invariant".to_string(),
            InvariantSeverity::Warning,
            |_ctx| Ok(true),
        );

        let context = InvariantContext::new();
        assert!(invariant.check(&context).unwrap());
        assert_eq!(invariant.name(), "test_invariant");
    }

    #[test]
    fn test_invariant_checker() {
        let mut checker = InvariantChecker::new();

        checker.register(PolicyConsistencyInvariant);
        checker.register(ModeSafetyInvariant::new(false));

        let context = InvariantContext::new();
        assert!(checker.check_all(&context).is_ok());
        assert_eq!(checker.invariant_count(), 2);
    }

    #[test]
    fn test_invariant_checker_violation() {
        let mut checker = InvariantChecker::new();

        checker.register(ModeSafetyInvariant::new(true)); // Will violate in Frozen mode

        let context = InvariantContext::new().with_mode(AutonomicMode::Frozen);
        assert!(checker.check_all(&context).is_err());
        assert!(checker.violation_count() > 0);
    }

    #[test]
    fn test_trace_determinism() {
        let mut invariant = TraceDeterminismInvariant::new();

        invariant.record_execution("trace1".to_string(), "hash1".to_string());
        invariant.record_execution("trace1".to_string(), "hash1".to_string());

        assert!(invariant.is_consistent("trace1"));

        invariant.record_execution("trace1".to_string(), "hash2".to_string());
        assert!(!invariant.is_consistent("trace1"));
    }
}
