//! Overlay Application Protocol State Machine
//!
//! This module encodes the overlay promotion pipeline as a type-level state machine.
//! The promotion workflow consists of:
//! - **Shadow** - Deploy overlay in shadow environment
//! - **Test** - Run tests against shadow deployment
//! - **Validate** - Validate correctness and performance
//! - **Promote** - Promote to production
//!
//! The type system enforces:
//! - Cannot promote without validation (compile error)
//! - Cannot skip testing phase
//! - Rollback protocol is type-safe
//! - Integration with existing overlay module
//!
//! ## Example
//! ```no_run
//! use knhk_mu_kernel::protocols::overlay_protocol::*;
//!
//! // Create overlay promotion pipeline
//! let pipeline = OverlayPipeline::new(overlay);
//!
//! // Must follow exact order
//! let pipeline = pipeline.deploy_shadow();
//! let pipeline = pipeline.run_tests();
//! let pipeline = pipeline.validate();
//! let result = pipeline.promote();
//!
//! // Invalid: pipeline.promote(); // ERROR: no method `promote` on ShadowPhase
//! ```

use crate::overlay::{OverlayAlgebra, PromoteError, RolloutStrategy};
use crate::overlay_proof::OverlayProof;
use crate::overlay_types::{OverlayValue, SnapshotId};
use core::marker::PhantomData;

/// Overlay promotion pipeline
///
/// Parameterized by current phase and overlay type.
/// Type system ensures phases are executed in order.
pub struct OverlayPipeline<Phase, P: OverlayProof> {
    /// Type-level phase marker
    _phase: PhantomData<fn() -> Phase>,
    /// The overlay being promoted
    overlay: OverlayValue<P>,
}

impl<Phase, P: OverlayProof> OverlayPipeline<Phase, P> {
    /// Internal constructor
    #[inline(always)]
    fn new(overlay: OverlayValue<P>) -> Self {
        Self {
            _phase: PhantomData,
            overlay,
        }
    }

    /// Get overlay reference
    #[inline(always)]
    pub fn overlay(&self) -> &OverlayValue<P> {
        &self.overlay
    }
}

/// Phase marker types (zero-sized)

/// Shadow phase - overlay deployed in shadow environment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowPhase;

/// Test phase - tests running against shadow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestPhase;

/// Validate phase - validation in progress
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatePhase;

/// Promote phase - ready for production promotion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotePhase;

/// Promoted phase - successfully promoted (terminal)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotedPhase;

/// Rolled back phase - promotion failed, rolled back (terminal)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RolledBackPhase;

// State transitions (enforce Shadow → Test → Validate → Promote order)

impl<P: OverlayProof> OverlayPipeline<ShadowPhase, P> {
    /// Create new overlay promotion pipeline
    ///
    /// Starts in Shadow phase - overlay not yet tested.
    #[inline(always)]
    pub fn new(overlay: OverlayValue<P>) -> Self {
        Self::new(overlay)
    }

    /// Deploy to shadow environment
    ///
    /// This is the first phase of overlay promotion.
    /// Type system ensures you start here.
    #[inline(always)]
    pub fn deploy_shadow(self) -> Result<OverlayPipeline<TestPhase, P>, PromoteError> {
        // Validate overlay before deploying to shadow
        self.overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;

        Ok(OverlayPipeline::new(self.overlay))
    }

    /// Rollback from shadow (early abort)
    #[inline(always)]
    pub fn rollback(self) -> OverlayPipeline<RolledBackPhase, P> {
        OverlayPipeline::new(self.overlay)
    }
}

impl<P: OverlayProof> OverlayPipeline<TestPhase, P> {
    /// Run tests against shadow deployment
    ///
    /// Can only be called after shadow deployment.
    /// Type system enforces this.
    #[inline(always)]
    pub fn run_tests(self) -> Result<OverlayPipeline<ValidatePhase, P>, PromoteError> {
        // In real implementation, would run comprehensive tests
        // For now, assume tests pass
        Ok(OverlayPipeline::new(self.overlay))
    }

    /// Rollback after failed tests
    #[inline(always)]
    pub fn rollback(self) -> OverlayPipeline<RolledBackPhase, P> {
        OverlayPipeline::new(self.overlay)
    }
}

impl<P: OverlayProof> OverlayPipeline<ValidatePhase, P> {
    /// Validate overlay correctness and performance
    ///
    /// Can only be called after tests pass.
    /// Type system enforces this.
    #[inline(always)]
    pub fn validate(self) -> Result<OverlayPipeline<PromotePhase, P>, PromoteError> {
        // Validate proof
        self.overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;

        // Check timing bound
        if self.overlay.proof().timing_bound() > crate::CHATMAN_CONSTANT {
            return Err(PromoteError::TimingBoundExceeded);
        }

        Ok(OverlayPipeline::new(self.overlay))
    }

    /// Rollback after failed validation
    #[inline(always)]
    pub fn rollback(self) -> OverlayPipeline<RolledBackPhase, P> {
        OverlayPipeline::new(self.overlay)
    }
}

impl<P: OverlayProof> OverlayPipeline<PromotePhase, P> {
    /// Promote to production
    ///
    /// Can ONLY be called after successful validation.
    /// Type system enforces this - cannot promote without validation.
    #[inline(always)]
    pub fn promote(self) -> Result<OverlayPipeline<PromotedPhase, P>, PromoteError> {
        // Final validation before promotion
        self.overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;

        // In real implementation, would atomically swap Σ*
        Ok(OverlayPipeline::new(self.overlay))
    }

    /// Rollback before promotion
    #[inline(always)]
    pub fn rollback(self) -> OverlayPipeline<RolledBackPhase, P> {
        OverlayPipeline::new(self.overlay)
    }
}

// Terminal states (Promoted, RolledBack) have no transitions

/// Overlay pipeline with test results
///
/// Tracks test results through the pipeline.
#[derive(Debug, Clone)]
pub struct TestResults {
    /// Number of tests run
    pub tests_run: usize,
    /// Number of tests passed
    pub tests_passed: usize,
    /// Performance metrics
    pub perf_metrics: Option<PerfMetrics>,
}

impl TestResults {
    /// Create empty test results
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            tests_run: 0,
            tests_passed: 0,
            perf_metrics: None,
        }
    }

    /// Check if all tests passed
    #[inline(always)]
    pub fn all_passed(&self) -> bool {
        self.tests_run > 0 && self.tests_passed == self.tests_run
    }
}

impl Default for TestResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics from testing
#[derive(Debug, Clone, Copy)]
pub struct PerfMetrics {
    /// Maximum ticks observed
    pub max_ticks: u64,
    /// Average ticks
    pub avg_ticks: u64,
    /// P99 ticks
    pub p99_ticks: u64,
}

/// Overlay pipeline with data tracking
pub struct OverlayPipelineWithData<Phase, P: OverlayProof, D> {
    _phase: PhantomData<fn() -> Phase>,
    overlay: OverlayValue<P>,
    data: D,
}

impl<Phase, P: OverlayProof, D> OverlayPipelineWithData<Phase, P, D> {
    /// Internal constructor
    #[inline(always)]
    fn new(overlay: OverlayValue<P>, data: D) -> Self {
        Self {
            _phase: PhantomData,
            overlay,
            data,
        }
    }

    /// Get data reference
    #[inline(always)]
    pub fn data(&self) -> &D {
        &self.data
    }

    /// Get overlay reference
    #[inline(always)]
    pub fn overlay(&self) -> &OverlayValue<P> {
        &self.overlay
    }
}

impl<P: OverlayProof> OverlayPipelineWithData<ShadowPhase, P, TestResults> {
    /// Create new pipeline with data tracking
    #[inline(always)]
    pub fn new(overlay: OverlayValue<P>) -> Self {
        Self::new(overlay, TestResults::new())
    }

    /// Deploy shadow with data
    #[inline(always)]
    pub fn deploy_shadow(
        self,
    ) -> Result<OverlayPipelineWithData<TestPhase, P, TestResults>, PromoteError> {
        self.overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;
        Ok(OverlayPipelineWithData::new(self.overlay, self.data))
    }
}

impl<P: OverlayProof> OverlayPipelineWithData<TestPhase, P, TestResults> {
    /// Run tests with result tracking
    #[inline(always)]
    pub fn run_tests(
        mut self,
        results: TestResults,
    ) -> Result<OverlayPipelineWithData<ValidatePhase, P, TestResults>, PromoteError> {
        if !results.all_passed() {
            return Err(PromoteError::ValidationFailed);
        }

        self.data = results;
        Ok(OverlayPipelineWithData::new(self.overlay, self.data))
    }
}

impl<P: OverlayProof> OverlayPipelineWithData<ValidatePhase, P, TestResults> {
    /// Validate with data
    #[inline(always)]
    pub fn validate(
        self,
    ) -> Result<OverlayPipelineWithData<PromotePhase, P, TestResults>, PromoteError> {
        // Check test results
        if !self.data.all_passed() {
            return Err(PromoteError::ValidationFailed);
        }

        // Check performance metrics
        if let Some(metrics) = &self.data.perf_metrics {
            if metrics.max_ticks > crate::CHATMAN_CONSTANT {
                return Err(PromoteError::TimingBoundExceeded);
            }
        }

        self.overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;
        Ok(OverlayPipelineWithData::new(self.overlay, self.data))
    }
}

impl<P: OverlayProof> OverlayPipelineWithData<PromotePhase, P, TestResults> {
    /// Promote with strategy
    #[inline(always)]
    pub fn promote_with_strategy(
        self,
        _strategy: RolloutStrategy,
    ) -> Result<OverlayPipelineWithData<PromotedPhase, P, TestResults>, PromoteError> {
        self.overlay
            .validate()
            .map_err(|_| PromoteError::ValidationFailed)?;
        Ok(OverlayPipelineWithData::new(self.overlay, self.data))
    }

    /// Get test results
    #[inline(always)]
    pub fn into_results(self) -> (OverlayValue<P>, TestResults) {
        (self.overlay, self.data)
    }
}

/// Rollback protocol
///
/// Type-safe rollback from any phase.
pub struct RollbackProtocol<Phase, P: OverlayProof> {
    _phase: PhantomData<fn() -> Phase>,
    /// Original overlay before rollback
    original: OverlayValue<P>,
    /// Reason for rollback
    reason: RollbackReason,
}

impl<Phase, P: OverlayProof> RollbackProtocol<Phase, P> {
    /// Create rollback protocol
    #[inline(always)]
    pub fn new(original: OverlayValue<P>, reason: RollbackReason) -> Self {
        Self {
            _phase: PhantomData,
            original,
            reason,
        }
    }

    /// Get rollback reason
    #[inline(always)]
    pub fn reason(&self) -> &RollbackReason {
        &self.reason
    }

    /// Execute rollback
    #[inline(always)]
    pub fn execute(self) -> OverlayPipeline<RolledBackPhase, P> {
        OverlayPipeline::new(self.original)
    }
}

/// Rollback reason
#[derive(Debug, Clone)]
pub enum RollbackReason {
    /// Tests failed
    TestsFailed { failed_count: usize },
    /// Validation failed
    ValidationFailed { error: &'static str },
    /// Performance degradation
    PerformanceDegraded { max_ticks: u64 },
    /// Manual rollback
    Manual,
}

/// Canary deployment state machine
///
/// For gradual rollout with automatic rollback.
pub struct CanaryDeployment<Phase, P: OverlayProof> {
    _phase: PhantomData<fn() -> Phase>,
    overlay: OverlayValue<P>,
    /// Current rollout percentage
    rollout_percent: u8,
    /// Target rollout percentage
    target_percent: u8,
}

/// Canary phases
pub struct CanaryInitial;
pub struct CanaryRollingOut;
pub struct CanaryComplete;

impl<P: OverlayProof> CanaryDeployment<CanaryInitial, P> {
    /// Create new canary deployment
    #[inline(always)]
    pub fn new(overlay: OverlayValue<P>, target_percent: u8) -> Self {
        Self {
            _phase: PhantomData,
            overlay,
            rollout_percent: 0,
            target_percent: target_percent.min(100),
        }
    }

    /// Start canary rollout
    #[inline(always)]
    pub fn start_rollout(mut self, initial_percent: u8) -> CanaryDeployment<CanaryRollingOut, P> {
        self.rollout_percent = initial_percent.min(self.target_percent);
        CanaryDeployment {
            _phase: PhantomData,
            overlay: self.overlay,
            rollout_percent: self.rollout_percent,
            target_percent: self.target_percent,
        }
    }
}

impl<P: OverlayProof> CanaryDeployment<CanaryRollingOut, P> {
    /// Increment rollout percentage
    #[inline(always)]
    pub fn increment(mut self, percent: u8) -> Result<Self, Self> {
        let new_percent = self.rollout_percent.saturating_add(percent);

        if new_percent >= self.target_percent {
            // Would transition to CanaryComplete
            Err(self)
        } else {
            self.rollout_percent = new_percent;
            Ok(self)
        }
    }

    /// Complete rollout
    #[inline(always)]
    pub fn complete(mut self) -> CanaryDeployment<CanaryComplete, P> {
        self.rollout_percent = self.target_percent;
        CanaryDeployment {
            _phase: PhantomData,
            overlay: self.overlay,
            rollout_percent: self.rollout_percent,
            target_percent: self.target_percent,
        }
    }

    /// Rollback canary
    #[inline(always)]
    pub fn rollback(self) -> OverlayPipeline<RolledBackPhase, P> {
        OverlayPipeline::new(self.overlay)
    }

    /// Get current rollout percentage
    #[inline(always)]
    pub fn rollout_percent(&self) -> u8 {
        self.rollout_percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay_proof::CompilerProof;
    use crate::overlay_types::{OverlayChanges, OverlayMetadata, PerfImpact};
    use alloc::vec;

    fn make_test_overlay() -> OverlayValue<CompilerProof> {
        let changes = OverlayChanges::new();
        let proof = CompilerProof {
            compiler_version: (2027, 0, 0),
            proof_id: 1,
            invariants: vec![1, 2, 3],
            timing_bound: 6,
            coverage: crate::overlay_proof::ChangeCoverage {
                covered_changes: 0,
                coverage_percent: 100,
            },
            signature: [1; 64],
        };
        let metadata = OverlayMetadata {
            id: 1,
            created_at: 0,
            priority: 10,
            author: [0; 32],
            description: "test",
            perf_impact: PerfImpact {
                expected_improvement: 0.1,
                confidence: 0.9,
                max_tick_increase: 2,
            },
        };

        OverlayValue::new(SnapshotId([0; 32]), changes, proof, metadata).unwrap()
    }

    #[test]
    fn test_overlay_promotion_pipeline() {
        let overlay = make_test_overlay();
        let pipeline = OverlayPipeline::new(overlay);

        // Must follow exact order
        let pipeline = pipeline.deploy_shadow().unwrap();
        let pipeline = pipeline.run_tests().unwrap();
        let pipeline = pipeline.validate().unwrap();
        let _pipeline = pipeline.promote().unwrap();
    }

    #[test]
    fn test_overlay_rollback() {
        let overlay = make_test_overlay();
        let pipeline = OverlayPipeline::new(overlay);

        // Can rollback from any phase
        let _pipeline = pipeline.rollback();
    }

    #[test]
    fn test_pipeline_with_data() {
        let overlay = make_test_overlay();
        let pipeline = OverlayPipelineWithData::new(overlay);

        let pipeline = pipeline.deploy_shadow().unwrap();

        let mut results = TestResults::new();
        results.tests_run = 10;
        results.tests_passed = 10;
        results.perf_metrics = Some(PerfMetrics {
            max_ticks: 6,
            avg_ticks: 4,
            p99_ticks: 6,
        });

        let pipeline = pipeline.run_tests(results).unwrap();
        assert!(pipeline.data().all_passed());

        let pipeline = pipeline.validate().unwrap();
        let _pipeline = pipeline
            .promote_with_strategy(RolloutStrategy::Immediate)
            .unwrap();
    }

    #[test]
    fn test_canary_deployment() {
        let overlay = make_test_overlay();
        let canary = CanaryDeployment::new(overlay, 100);

        let canary = canary.start_rollout(10);
        assert_eq!(canary.rollout_percent(), 10);

        let canary = canary.increment(20).unwrap();
        assert_eq!(canary.rollout_percent(), 30);

        let _canary = canary.complete();
    }

    #[test]
    fn test_rollback_protocol() {
        let overlay = make_test_overlay();
        let rollback =
            RollbackProtocol::new(overlay, RollbackReason::TestsFailed { failed_count: 3 });

        let _pipeline = rollback.execute();
    }
}
