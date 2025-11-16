//! Doctrine Module for Policy Enforcement
//!
//! Implements **doctrine-bound policy projection** with μ-kernel constraint enforcement.
//!
//! # Doctrine (Q)
//!
//! The doctrine Q represents the **lawful subset** of the action space. It encodes:
//! - μ-kernel constraints (τ ≤ 8 ticks, max_run_len ≤ 8)
//! - System invariants that must never be violated
//! - Safety properties required for correct operation
//!
//! # Doctrine Projection
//!
//! **Q ∧ policy → policy'**
//!
//! Projects a policy element through the doctrine to determine if it's lawful:
//! - If policy' ≠ ⊥ → Action is allowed (satisfies doctrine)
//! - If policy' = ⊥ → Action is illegal (violates doctrine)
//!
//! # μ-Kernel Constraints
//!
//! The **Chatman Constant** (8 ticks) defines hard limits:
//! - **τ ≤ 8 ticks**: Maximum execution time for hot path operations
//! - **max_run_len ≤ 8**: Maximum consecutive operations without yielding
//! - **max_depth ≤ 8**: Maximum call stack depth for recursive operations
//!
//! These constraints ensure **bounded latency** and **livelock freedom**.

use super::policy_lattice::{
    CapacityEnvelope, FailureRateBound, GuardStrictness, GuardStrictnessLevel, LatencyBound,
    Lattice, PolicyElement, Strictness,
};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// μ-Kernel Constants (Chatman Constant = 8)
// ============================================================================

/// Maximum execution ticks for hot path operations (Chatman Constant)
pub const MAX_EXEC_TICKS: u64 = 8;

/// Maximum consecutive operations without yielding
pub const MAX_RUN_LEN: usize = 8;

/// Maximum call stack depth for recursive operations
pub const MAX_CALL_DEPTH: usize = 8;

/// Maximum concurrency for safe operation
pub const MAX_SAFE_CONCURRENCY: u32 = 256;

/// Maximum parallelism for safe operation
pub const MAX_SAFE_PARALLELISM: u32 = 64;

/// Maximum acceptable error rate (1% for production)
pub const MAX_SAFE_ERROR_RATE: f64 = 0.01;

/// Maximum latency for hot path (100ms P99)
pub const MAX_HOT_PATH_LATENCY_MS: f64 = 100.0;

// ============================================================================
// Doctrine Definition
// ============================================================================

/// Doctrine (Q) - Defines lawful action space
///
/// The doctrine encodes all invariants that must hold for system correctness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doctrine {
    /// μ-kernel execution tick limit
    pub max_exec_ticks: u64,
    /// Maximum run length without yielding
    pub max_run_len: usize,
    /// Maximum call depth
    pub max_call_depth: usize,
    /// Maximum safe concurrency
    pub max_safe_concurrency: u32,
    /// Maximum safe parallelism
    pub max_safe_parallelism: u32,
    /// Maximum acceptable error rate
    pub max_safe_error_rate: f64,
    /// Maximum hot path latency (ms)
    pub max_hot_path_latency_ms: f64,
    /// Whether to enforce μ-kernel constraints strictly
    pub enforce_mu_kernel: bool,
}

impl Doctrine {
    /// Create default doctrine with μ-kernel constraints
    pub fn new() -> Self {
        Self {
            max_exec_ticks: MAX_EXEC_TICKS,
            max_run_len: MAX_RUN_LEN,
            max_call_depth: MAX_CALL_DEPTH,
            max_safe_concurrency: MAX_SAFE_CONCURRENCY,
            max_safe_parallelism: MAX_SAFE_PARALLELISM,
            max_safe_error_rate: MAX_SAFE_ERROR_RATE,
            max_hot_path_latency_ms: MAX_HOT_PATH_LATENCY_MS,
            enforce_mu_kernel: true,
        }
    }

    /// Create relaxed doctrine (for testing/development)
    pub fn relaxed() -> Self {
        Self {
            max_exec_ticks: MAX_EXEC_TICKS * 4,
            max_run_len: MAX_RUN_LEN * 2,
            max_call_depth: MAX_CALL_DEPTH * 2,
            max_safe_concurrency: MAX_SAFE_CONCURRENCY * 2,
            max_safe_parallelism: MAX_SAFE_PARALLELISM * 2,
            max_safe_error_rate: MAX_SAFE_ERROR_RATE * 2.0,
            max_hot_path_latency_ms: MAX_HOT_PATH_LATENCY_MS * 2.0,
            enforce_mu_kernel: false,
        }
    }

    /// Create strict doctrine (production hardening)
    pub fn strict() -> Self {
        Self {
            max_exec_ticks: MAX_EXEC_TICKS,
            max_run_len: MAX_RUN_LEN,
            max_call_depth: MAX_CALL_DEPTH,
            max_safe_concurrency: MAX_SAFE_CONCURRENCY / 2,
            max_safe_parallelism: MAX_SAFE_PARALLELISM / 2,
            max_safe_error_rate: MAX_SAFE_ERROR_RATE / 2.0,
            max_hot_path_latency_ms: MAX_HOT_PATH_LATENCY_MS / 2.0,
            enforce_mu_kernel: true,
        }
    }

    /// Project policy through doctrine: Q ∧ policy → policy'
    ///
    /// Returns:
    /// - Ok(Some(policy')) if action is lawful
    /// - Ok(None) if action violates doctrine (policy' = ⊥)
    /// - Err(_) if projection fails
    pub fn project(&self, policy: &PolicyElement) -> WorkflowResult<Option<PolicyElement>> {
        match policy {
            // Bottom always remains bottom
            PolicyElement::Bottom => Ok(Some(PolicyElement::Bottom)),

            // Project latency bound
            PolicyElement::Latency(latency) => self.project_latency(latency),

            // Project failure rate bound
            PolicyElement::FailureRate(failure) => self.project_failure_rate(failure),

            // Project guard strictness
            PolicyElement::GuardStrictness(guard) => self.project_guard_strictness(guard),

            // Project capacity envelope
            PolicyElement::Capacity(capacity) => self.project_capacity(capacity),

            // Project conjunction (all elements must satisfy doctrine)
            PolicyElement::Conjunction(policies) => {
                let mut projected = Vec::new();

                for p in policies {
                    match self.project(p)? {
                        Some(p_prime) => {
                            if p_prime.is_bottom() {
                                // Any bottom element makes entire conjunction bottom
                                return Ok(None);
                            }
                            projected.push(p_prime);
                        }
                        None => {
                            // Any violation makes entire conjunction invalid
                            return Ok(None);
                        }
                    }
                }

                if projected.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(PolicyElement::Conjunction(projected)))
                }
            }
        }
    }

    /// Project latency bound through doctrine
    fn project_latency(&self, latency: &LatencyBound) -> WorkflowResult<Option<PolicyElement>> {
        // Check against μ-kernel hot path constraint
        if self.enforce_mu_kernel && latency.target_p99_ms > self.max_hot_path_latency_ms {
            // Latency exceeds doctrine - meet with doctrine bound
            let doctrine_bound =
                LatencyBound::new(self.max_hot_path_latency_ms, Strictness::Hard)?;
            let projected = latency.meet(&doctrine_bound);

            // If projected is effectively zero, reject
            if projected.target_p99_ms <= 0.0 {
                Ok(None)
            } else {
                Ok(Some(PolicyElement::Latency(projected)))
            }
        } else {
            // Within doctrine bounds
            Ok(Some(PolicyElement::Latency(latency.clone())))
        }
    }

    /// Project failure rate bound through doctrine
    fn project_failure_rate(
        &self,
        failure: &FailureRateBound,
    ) -> WorkflowResult<Option<PolicyElement>> {
        // Check against maximum safe error rate
        if failure.max_error_rate > self.max_safe_error_rate {
            // Error rate exceeds doctrine - meet with doctrine bound
            let doctrine_bound = FailureRateBound::new(self.max_safe_error_rate)?;
            let projected = failure.meet(&doctrine_bound);

            // If projected error rate is zero (impossible), reject
            if projected.max_error_rate <= 0.0 {
                Ok(None)
            } else {
                Ok(Some(PolicyElement::FailureRate(projected)))
            }
        } else {
            // Within doctrine bounds
            Ok(Some(PolicyElement::FailureRate(failure.clone())))
        }
    }

    /// Project guard strictness through doctrine
    fn project_guard_strictness(
        &self,
        guard: &GuardStrictness,
    ) -> WorkflowResult<Option<PolicyElement>> {
        // Doctrine always prefers tightened guards for safety
        if self.enforce_mu_kernel && guard.level == GuardStrictnessLevel::Relax {
            let doctrine_guard = GuardStrictness::new(GuardStrictnessLevel::Tighten);
            let projected = guard.meet(&doctrine_guard);
            Ok(Some(PolicyElement::GuardStrictness(projected)))
        } else {
            Ok(Some(PolicyElement::GuardStrictness(guard.clone())))
        }
    }

    /// Project capacity envelope through doctrine
    fn project_capacity(
        &self,
        capacity: &CapacityEnvelope,
    ) -> WorkflowResult<Option<PolicyElement>> {
        // Check against maximum safe concurrency/parallelism
        if capacity.max_concurrency > self.max_safe_concurrency
            || capacity.max_parallelism > self.max_safe_parallelism
        {
            // Capacity exceeds doctrine - meet with doctrine bounds
            let doctrine_capacity =
                CapacityEnvelope::new(self.max_safe_concurrency, self.max_safe_parallelism)?;
            let projected = capacity.meet(&doctrine_capacity);

            // If projected capacity is zero, reject
            if projected.max_concurrency == 0 || projected.max_parallelism == 0 {
                Ok(None)
            } else {
                Ok(Some(PolicyElement::Capacity(projected)))
            }
        } else {
            // Within doctrine bounds
            Ok(Some(PolicyElement::Capacity(capacity.clone())))
        }
    }

    /// Validate that an action satisfies doctrine
    pub fn validate(&self, policy: &PolicyElement) -> WorkflowResult<bool> {
        match self.project(policy)? {
            Some(policy_prime) => Ok(!policy_prime.is_bottom()),
            None => Ok(false),
        }
    }

    /// Validate execution metrics against μ-kernel constraints
    pub fn validate_execution_metrics(&self, metrics: &ExecutionMetrics) -> WorkflowResult<bool> {
        if !self.enforce_mu_kernel {
            return Ok(true);
        }

        let violations = vec![
            (
                metrics.exec_ticks > self.max_exec_ticks,
                format!(
                    "Execution ticks {} exceeds limit {}",
                    metrics.exec_ticks, self.max_exec_ticks
                ),
            ),
            (
                metrics.run_len > self.max_run_len,
                format!(
                    "Run length {} exceeds limit {}",
                    metrics.run_len, self.max_run_len
                ),
            ),
            (
                metrics.call_depth > self.max_call_depth,
                format!(
                    "Call depth {} exceeds limit {}",
                    metrics.call_depth, self.max_call_depth
                ),
            ),
        ];

        let failed: Vec<_> = violations.into_iter().filter(|(v, _)| *v).collect();

        if failed.is_empty() {
            Ok(true)
        } else {
            let errors: Vec<String> = failed.into_iter().map(|(_, msg)| msg).collect();
            Err(WorkflowError::Validation(format!(
                "μ-kernel constraint violations: {}",
                errors.join("; ")
            )))
        }
    }
}

impl Default for Doctrine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Execution Metrics
// ============================================================================

/// Execution metrics for doctrine validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Number of execution ticks consumed
    pub exec_ticks: u64,
    /// Consecutive operations run without yielding
    pub run_len: usize,
    /// Current call stack depth
    pub call_depth: usize,
    /// Actual latency observed (ms)
    pub latency_ms: f64,
    /// Actual error rate observed
    pub error_rate: f64,
    /// Timestamp of measurement
    pub timestamp_ms: u64,
}

impl ExecutionMetrics {
    pub fn new() -> Self {
        Self {
            exec_ticks: 0,
            run_len: 0,
            call_depth: 0,
            latency_ms: 0.0,
            error_rate: 0.0,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create metrics from duration
    pub fn from_duration(duration: Duration) -> Self {
        Self {
            exec_ticks: (duration.as_micros() / 100) as u64, // Approximate tick count
            run_len: 1,
            call_depth: 1,
            latency_ms: duration.as_secs_f64() * 1000.0,
            error_rate: 0.0,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Doctrine-Aware Action Wrapper
// ============================================================================

/// Action that carries its policy element for doctrine checking
///
/// This wrapper ensures that every action is validated against the doctrine
/// before execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineAction<T> {
    /// The action payload
    pub action: T,
    /// Policy element governing this action
    pub policy: PolicyElement,
    /// Whether action has been validated against doctrine
    pub validated: bool,
}

impl<T> DoctrineAction<T> {
    /// Create new doctrine-aware action
    pub fn new(action: T, policy: PolicyElement) -> Self {
        Self {
            action,
            policy,
            validated: false,
        }
    }

    /// Validate action against doctrine
    pub fn validate(&mut self, doctrine: &Doctrine) -> WorkflowResult<bool> {
        let is_valid = doctrine.validate(&self.policy)?;
        self.validated = is_valid;
        Ok(is_valid)
    }

    /// Project action policy through doctrine
    pub fn project(&mut self, doctrine: &Doctrine) -> WorkflowResult<Option<PolicyElement>> {
        let projected = doctrine.project(&self.policy)?;
        if let Some(ref policy_prime) = projected {
            self.policy = policy_prime.clone();
            self.validated = !policy_prime.is_bottom();
        } else {
            self.validated = false;
        }
        Ok(projected)
    }

    /// Check if action is validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }

    /// Unwrap action (consumes self)
    pub fn into_action(self) -> T {
        self.action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctrine_creation() {
        let doctrine = Doctrine::new();
        assert_eq!(doctrine.max_exec_ticks, MAX_EXEC_TICKS);
        assert_eq!(doctrine.max_run_len, MAX_RUN_LEN);
        assert!(doctrine.enforce_mu_kernel);

        let relaxed = Doctrine::relaxed();
        assert!(relaxed.max_exec_ticks > doctrine.max_exec_ticks);
        assert!(!relaxed.enforce_mu_kernel);

        let strict = Doctrine::strict();
        assert!(strict.max_safe_concurrency < doctrine.max_safe_concurrency);
    }

    #[test]
    fn test_doctrine_project_latency_within_bounds() {
        let doctrine = Doctrine::new();
        let latency = LatencyBound::new(50.0, Strictness::Soft).unwrap();
        let policy = PolicyElement::Latency(latency);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());
        assert!(!projected.unwrap().is_bottom());
    }

    #[test]
    fn test_doctrine_project_latency_exceeds_bounds() {
        let doctrine = Doctrine::new();
        let latency = LatencyBound::new(200.0, Strictness::Soft).unwrap(); // Exceeds 100ms
        let policy = PolicyElement::Latency(latency);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());

        // Should be clamped to doctrine bound
        match projected.unwrap() {
            PolicyElement::Latency(bound) => {
                assert!(bound.target_p99_ms <= doctrine.max_hot_path_latency_ms);
            }
            _ => panic!("Expected Latency policy"),
        }
    }

    #[test]
    fn test_doctrine_project_failure_rate_within_bounds() {
        let doctrine = Doctrine::new();
        let failure = FailureRateBound::new(0.005).unwrap(); // 0.5% < 1%
        let policy = PolicyElement::FailureRate(failure);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());
        assert!(!projected.unwrap().is_bottom());
    }

    #[test]
    fn test_doctrine_project_failure_rate_exceeds_bounds() {
        let doctrine = Doctrine::new();
        let failure = FailureRateBound::new(0.05).unwrap(); // 5% > 1%
        let policy = PolicyElement::FailureRate(failure);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());

        // Should be clamped to doctrine bound
        match projected.unwrap() {
            PolicyElement::FailureRate(bound) => {
                assert!(bound.max_error_rate <= doctrine.max_safe_error_rate);
            }
            _ => panic!("Expected FailureRate policy"),
        }
    }

    #[test]
    fn test_doctrine_project_capacity_within_bounds() {
        let doctrine = Doctrine::new();
        let capacity = CapacityEnvelope::new(100, 32).unwrap();
        let policy = PolicyElement::Capacity(capacity);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());
        assert!(!projected.unwrap().is_bottom());
    }

    #[test]
    fn test_doctrine_project_capacity_exceeds_bounds() {
        let doctrine = Doctrine::new();
        let capacity = CapacityEnvelope::new(1000, 200).unwrap(); // Exceeds limits
        let policy = PolicyElement::Capacity(capacity);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());

        // Should be clamped to doctrine bounds
        match projected.unwrap() {
            PolicyElement::Capacity(envelope) => {
                assert!(envelope.max_concurrency <= doctrine.max_safe_concurrency);
                assert!(envelope.max_parallelism <= doctrine.max_safe_parallelism);
            }
            _ => panic!("Expected Capacity policy"),
        }
    }

    #[test]
    fn test_doctrine_project_guard_strictness() {
        let doctrine = Doctrine::new();
        let guard = GuardStrictness::new(GuardStrictnessLevel::Relax);
        let policy = PolicyElement::GuardStrictness(guard);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());

        // Should be tightened by doctrine
        match projected.unwrap() {
            PolicyElement::GuardStrictness(g) => {
                assert_eq!(g.level, GuardStrictnessLevel::Tighten);
            }
            _ => panic!("Expected GuardStrictness policy"),
        }
    }

    #[test]
    fn test_doctrine_project_conjunction() {
        let doctrine = Doctrine::new();
        let latency = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());
        let failure = PolicyElement::FailureRate(FailureRateBound::new(0.005).unwrap());
        let policy = PolicyElement::Conjunction(vec![latency, failure]);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some());

        match projected.unwrap() {
            PolicyElement::Conjunction(policies) => {
                assert_eq!(policies.len(), 2);
            }
            _ => panic!("Expected Conjunction policy"),
        }
    }

    #[test]
    fn test_doctrine_project_conjunction_with_violation() {
        let doctrine = Doctrine::new();
        let latency = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());
        let bad_failure = PolicyElement::FailureRate(FailureRateBound::new(1.0).unwrap()); // 100% error rate
        let policy = PolicyElement::Conjunction(vec![latency, bad_failure]);

        let projected = doctrine.project(&policy).unwrap();
        assert!(projected.is_some()); // Still returns a projection, but clamped
    }

    #[test]
    fn test_doctrine_validate() {
        let doctrine = Doctrine::new();
        let good_latency =
            PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());
        assert!(doctrine.validate(&good_latency).unwrap());

        let bottom = PolicyElement::Bottom;
        assert!(!doctrine.validate(&bottom).unwrap());
    }

    #[test]
    fn test_execution_metrics_validation() {
        let doctrine = Doctrine::new();

        // Valid metrics
        let mut metrics = ExecutionMetrics::new();
        metrics.exec_ticks = 5;
        metrics.run_len = 3;
        metrics.call_depth = 2;
        assert!(doctrine.validate_execution_metrics(&metrics).unwrap());

        // Violates tick constraint
        metrics.exec_ticks = 20;
        assert!(doctrine.validate_execution_metrics(&metrics).is_err());
    }

    #[test]
    fn test_doctrine_action_creation() {
        let action = "scale_instances";
        let policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());
        let doctrine_action = DoctrineAction::new(action, policy);

        assert_eq!(doctrine_action.action, "scale_instances");
        assert!(!doctrine_action.is_validated());
    }

    #[test]
    fn test_doctrine_action_validation() {
        let doctrine = Doctrine::new();
        let action = "scale_instances";
        let policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft).unwrap());
        let mut doctrine_action = DoctrineAction::new(action, policy);

        let is_valid = doctrine_action.validate(&doctrine).unwrap();
        assert!(is_valid);
        assert!(doctrine_action.is_validated());
    }

    #[test]
    fn test_doctrine_action_projection() {
        let doctrine = Doctrine::new();
        let action = "scale_instances";
        let policy = PolicyElement::Latency(LatencyBound::new(200.0, Strictness::Soft).unwrap()); // Exceeds bounds
        let mut doctrine_action = DoctrineAction::new(action, policy);

        let projected = doctrine_action.project(&doctrine).unwrap();
        assert!(projected.is_some());

        // Policy should be clamped
        match &doctrine_action.policy {
            PolicyElement::Latency(bound) => {
                assert!(bound.target_p99_ms <= doctrine.max_hot_path_latency_ms);
            }
            _ => panic!("Expected Latency policy"),
        }
    }

    #[test]
    fn test_execution_metrics_from_duration() {
        let duration = Duration::from_millis(10);
        let metrics = ExecutionMetrics::from_duration(duration);

        assert_eq!(metrics.latency_ms, 10.0);
        assert!(metrics.exec_ticks > 0);
    }

    #[test]
    fn test_mu_kernel_constants() {
        assert_eq!(MAX_EXEC_TICKS, 8);
        assert_eq!(MAX_RUN_LEN, 8);
        assert_eq!(MAX_CALL_DEPTH, 8);
    }

    #[test]
    fn test_relaxed_doctrine_allows_more() {
        let standard = Doctrine::new();
        let relaxed = Doctrine::relaxed();

        let high_latency =
            PolicyElement::Latency(LatencyBound::new(150.0, Strictness::Soft).unwrap());

        // Standard doctrine clamps it
        let projected_standard = standard.project(&high_latency).unwrap().unwrap();
        match projected_standard {
            PolicyElement::Latency(bound) => {
                assert!(bound.target_p99_ms <= standard.max_hot_path_latency_ms);
            }
            _ => panic!("Expected Latency"),
        }

        // Relaxed doctrine allows it
        let projected_relaxed = relaxed.project(&high_latency).unwrap().unwrap();
        match projected_relaxed {
            PolicyElement::Latency(bound) => {
                assert_eq!(bound.target_p99_ms, 150.0);
            }
            _ => panic!("Expected Latency"),
        }
    }

    #[test]
    fn test_strict_doctrine_tighter_bounds() {
        let strict = Doctrine::strict();
        assert!(strict.max_safe_concurrency < MAX_SAFE_CONCURRENCY);
        assert!(strict.max_safe_error_rate < MAX_SAFE_ERROR_RATE);
    }
}
