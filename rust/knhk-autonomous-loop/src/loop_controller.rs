//! Main autonomous loop controller - orchestrates the continuous evolution cycle

use crate::{
    adaptive_strategy::AdaptiveStrategy,
    audit_trail::{AuditEvent, AuditTrail},
    feedback_system::FeedbackSystem,
    self_healing::SelfHealer,
    CompiledProjection, DeltaSigmaProposal, DetectedPatterns, RecoveryStrategy, Result,
    SigmaSnapshotId, ValidationResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// Main orchestrator for the autonomous evolution loop
///
/// This controller manages the complete cycle:
/// 1. Pattern detection from observations
/// 2. ΔΣ proposal generation
/// 3. Validation of proposals
/// 4. Compilation of projections
/// 5. Promotion to production
/// 6. Feedback and adaptation
#[derive(Clone)]
pub struct AutonomousLoopController {
    /// Current state of the loop
    state: Arc<RwLock<LoopState>>,

    /// Feedback system (triggers evolution)
    feedback: Arc<FeedbackSystem>,

    /// Self-healing (recovery)
    healer: Arc<SelfHealer>,

    /// Adaptive strategy (learning)
    adaptive: Arc<AdaptiveStrategy>,

    /// Audit trail (immutable log)
    audit_trail: Arc<AuditTrail>,

    /// Configuration
    config: LoopConfig,
}

/// Current state of the autonomous loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopState {
    /// Is the loop currently running?
    pub is_running: bool,

    /// Timestamp of last completed cycle
    pub last_cycle: Option<SystemTime>,

    /// Total number of cycles completed
    pub cycle_count: u64,

    /// Number of successful cycles
    pub success_count: u64,

    /// Number of failed cycles
    pub failure_count: u64,

    /// Current change rate (proposals per second)
    pub change_rate: f64,

    /// Last error message (if any)
    pub last_error: Option<String>,
}

impl Default for LoopState {
    fn default() -> Self {
        Self {
            is_running: true,
            last_cycle: None,
            cycle_count: 0,
            success_count: 0,
            failure_count: 0,
            change_rate: 0.0,
            last_error: None,
        }
    }
}

/// Configuration for the autonomous loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopConfig {
    /// Maximum number of proposals to generate per cycle
    pub max_proposals: usize,

    /// Time to wait between cycles
    pub cycle_interval: Duration,

    /// Maximum change rate (proposals per second)
    pub max_change_rate: f64,

    /// Auto-pause if failure rate exceeds this threshold
    pub failure_threshold: f64,

    /// Recovery strategy on promotion failure
    pub recovery_strategy: RecoveryStrategy,
}

impl Default for LoopConfig {
    fn default() -> Self {
        Self {
            max_proposals: 10,
            cycle_interval: Duration::from_secs(60),
            max_change_rate: 1.0,
            failure_threshold: 0.5,
            recovery_strategy: RecoveryStrategy::Rollback,
        }
    }
}

/// Result of a single evolution cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleResult {
    /// Number of proposals generated
    pub proposals_generated: usize,

    /// Number of proposals that passed validation
    pub proposals_validated: usize,

    /// Snapshots that were promoted
    pub promoted: Vec<SigmaSnapshotId>,

    /// Duration of the cycle in milliseconds
    pub duration_ms: u64,
}

impl CycleResult {
    pub fn no_changes() -> Self {
        Self {
            proposals_generated: 0,
            proposals_validated: 0,
            promoted: Vec::new(),
            duration_ms: 0,
        }
    }
}

impl AutonomousLoopController {
    /// Create a new autonomous loop controller
    #[instrument(skip(config))]
    pub async fn new(config: LoopConfig) -> Result<Self> {
        info!("Initializing autonomous loop controller");

        let state = Arc::new(RwLock::new(LoopState::default()));
        let feedback = Arc::new(FeedbackSystem::new().await?);
        let healer = Arc::new(SelfHealer::new(config.recovery_strategy.clone()).await?);
        let adaptive = Arc::new(AdaptiveStrategy::new(state.clone()));
        let audit_trail = Arc::new(AuditTrail::new().await?);

        Ok(Self {
            state,
            feedback,
            healer,
            adaptive,
            audit_trail,
            config,
        })
    }

    /// Run the main autonomous loop continuously
    ///
    /// This is the primary entry point. The loop will run until:
    /// - Explicitly stopped via `stop()`
    /// - Failure rate exceeds threshold
    /// - Unrecoverable error occurs
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        info!("Starting autonomous evolution loop");

        loop {
            // Check if we should continue
            if !self.should_continue().await? {
                info!("Loop stopped");
                break;
            }

            // Execute one cycle
            let result = self.cycle().await;

            match result {
                Ok(cycle_result) => {
                    self.log_success(&cycle_result).await?;
                }
                Err(e) => {
                    self.log_failure(&e).await?;

                    // Self-healing check
                    if self.healer.should_heal(&self.state).await? {
                        info!("Triggering self-healing");
                        self.healer.heal().await?;
                    }
                }
            }

            // Adaptive strategy adjustment
            let mut config = self.config.clone();
            self.adaptive.adjust_interval(&mut config).await?;

            // Wait for next cycle
            tokio::time::sleep(config.cycle_interval).await;
        }

        Ok(())
    }

    /// Execute one complete evolution cycle
    #[instrument(skip(self))]
    async fn cycle(&self) -> Result<CycleResult> {
        let start = Instant::now();
        let cycle_num = {
            let mut state = self.state.write().await;
            state.cycle_count += 1;
            state.cycle_count
        };

        info!(cycle = cycle_num, "Starting evolution cycle");

        // 1. Check if evolution should be triggered
        let trigger_reason = self.feedback.should_trigger_evolution().await?;
        debug!(?trigger_reason, "Checked trigger conditions");

        if matches!(trigger_reason, crate::TriggerReason::None) {
            info!("No triggers detected, skipping cycle");
            return Ok(CycleResult::no_changes());
        }

        self.audit_trail
            .record(AuditEvent::CycleStarted {
                cycle_number: cycle_num,
                trigger: trigger_reason,
            })
            .await?;

        // 2. Detect patterns
        let patterns = self.detect_patterns().await?;
        debug!(pattern_count = patterns.count(), "Patterns detected");

        self.audit_trail
            .record(AuditEvent::PatternsDetected(patterns.clone()))
            .await?;

        if patterns.is_empty() {
            info!("No patterns detected, skipping cycle");
            return Ok(CycleResult::no_changes());
        }

        // 3. Generate proposals
        let proposals = self.generate_proposals(&patterns).await?;
        let proposal_count = proposals.len();
        info!(count = proposal_count, "Generated proposals");

        if proposals.is_empty() {
            return Ok(CycleResult::no_changes());
        }

        for proposal in &proposals {
            self.audit_trail
                .record(AuditEvent::ProposalGenerated(proposal.clone()))
                .await?;
        }

        // 4. Validate proposals (in parallel)
        let validated = self.validate_proposals(proposals).await?;
        let validated_count = validated.len();
        info!(count = validated_count, "Validated proposals");

        // 5. Compile projections
        let compiled = self.compile_all_validated(&validated).await?;
        info!(count = compiled.len(), "Compiled projections");

        // 6. Promote to production
        let promoted_ids = self.promote_all(&compiled).await?;
        info!(count = promoted_ids.len(), "Promoted snapshots");

        let duration = start.elapsed();

        Ok(CycleResult {
            proposals_generated: proposal_count,
            proposals_validated: validated_count,
            promoted: promoted_ids,
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Check if the loop should continue running
    async fn should_continue(&self) -> Result<bool> {
        let state = self.state.read().await;

        if !state.is_running {
            return Ok(false);
        }

        // Check failure rate
        if state.cycle_count > 0 {
            let failure_rate = state.failure_count as f64 / state.cycle_count as f64;
            if failure_rate > self.config.failure_threshold {
                warn!(
                    failure_rate,
                    threshold = self.config.failure_threshold,
                    "Failure rate exceeded threshold, pausing loop"
                );

                drop(state);
                let mut state = self.state.write().await;
                state.is_running = false;
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Detect patterns from recent observations
    #[instrument(skip(self))]
    async fn detect_patterns(&self) -> Result<DetectedPatterns> {
        // TODO: Integration with knhk-pattern-miner
        // For now, return mock patterns
        debug!("Detecting patterns from observations");

        // Placeholder: Would call pattern miner here
        Ok(DetectedPatterns::default())
    }

    /// Generate ΔΣ proposals from detected patterns
    #[instrument(skip(self, patterns))]
    async fn generate_proposals(
        &self,
        patterns: &DetectedPatterns,
    ) -> Result<Vec<DeltaSigmaProposal>> {
        // TODO: Integration with knhk-change-engine
        debug!(pattern_count = patterns.count(), "Generating proposals");

        // Placeholder: Would call change engine here
        Ok(Vec::new())
    }

    /// Validate proposals in parallel
    #[instrument(skip(self, proposals))]
    async fn validate_proposals(
        &self,
        proposals: Vec<DeltaSigmaProposal>,
    ) -> Result<Vec<DeltaSigmaProposal>> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let mut validation_tasks = FuturesUnordered::new();

        for proposal in proposals {
            let audit = self.audit_trail.clone();
            validation_tasks.push(async move {
                let result = Self::validate_single_proposal(&proposal).await;
                (proposal, result, audit)
            });
        }

        let mut validated = Vec::new();

        while let Some((proposal, result, audit)) = validation_tasks.next().await {
            match result {
                Ok(validation_result) if validation_result.passed => {
                    audit
                        .record(AuditEvent::ValidationPassed(
                            proposal.snapshot_id.clone(),
                        ))
                        .await?;
                    validated.push(proposal);
                }
                Ok(validation_result) => {
                    let reason = format!("Validation failed: {:?}", validation_result.violations);
                    audit
                        .record(AuditEvent::ValidationFailed(reason))
                        .await?;
                }
                Err(e) => {
                    audit
                        .record(AuditEvent::ValidationFailed(e.to_string()))
                        .await?;
                }
            }
        }

        Ok(validated)
    }

    /// Validate a single proposal
    async fn validate_single_proposal(
        proposal: &DeltaSigmaProposal,
    ) -> Result<ValidationResult> {
        // TODO: Integration with knhk-change-engine validator
        debug!(proposal_id = %proposal.proposal_id, "Validating proposal");

        // Placeholder: Would call validator here
        Ok(ValidationResult {
            passed: false,
            snapshot_id: proposal.snapshot_id.clone(),
            violations: Vec::new(),
            warnings: Vec::new(),
            validated_at: SystemTime::now(),
        })
    }

    /// Compile all validated proposals into projections
    #[instrument(skip(self, proposals))]
    async fn compile_all_validated(
        &self,
        proposals: &[DeltaSigmaProposal],
    ) -> Result<Vec<CompiledProjection>> {
        // TODO: Integration with knhk-projections
        debug!(count = proposals.len(), "Compiling projections");

        // Placeholder: Would call projection compiler here
        Ok(Vec::new())
    }

    /// Promote all compiled projections to production
    #[instrument(skip(self, compiled))]
    async fn promote_all(
        &self,
        compiled: &[CompiledProjection],
    ) -> Result<Vec<SigmaSnapshotId>> {
        // TODO: Integration with knhk-promotion
        debug!(count = compiled.len(), "Promoting snapshots");

        let mut promoted = Vec::new();

        for projection in compiled {
            self.audit_trail
                .record(AuditEvent::PromotionStarted(projection.snapshot_id.clone()))
                .await?;

            // Placeholder: Would call promotion system here
            // If successful:
            self.audit_trail
                .record(AuditEvent::PromotionSucceeded(
                    projection.snapshot_id.clone(),
                ))
                .await?;

            promoted.push(projection.snapshot_id.clone());
        }

        Ok(promoted)
    }

    /// Log a successful cycle
    async fn log_success(&self, result: &CycleResult) -> Result<()> {
        let mut state = self.state.write().await;
        state.success_count += 1;
        state.last_cycle = Some(SystemTime::now());
        state.last_error = None;

        info!(
            cycle = state.cycle_count,
            success = state.success_count,
            proposals = result.proposals_generated,
            promoted = result.promoted.len(),
            duration_ms = result.duration_ms,
            "Cycle completed successfully"
        );

        Ok(())
    }

    /// Log a failed cycle
    async fn log_failure(&self, error: &crate::AutonomousLoopError) -> Result<()> {
        let mut state = self.state.write().await;
        state.failure_count += 1;
        state.last_error = Some(error.to_string());

        error!(
            cycle = state.cycle_count,
            failures = state.failure_count,
            error = %error,
            "Cycle failed"
        );

        Ok(())
    }

    /// Stop the loop gracefully
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping autonomous loop");
        let mut state = self.state.write().await;
        state.is_running = false;
        Ok(())
    }

    /// Get current state (for monitoring)
    pub async fn get_state(&self) -> LoopState {
        self.state.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_loop_controller_creation() {
        let config = LoopConfig::default();
        let controller = AutonomousLoopController::new(config).await.unwrap();
        let state = controller.get_state().await;
        assert!(state.is_running);
        assert_eq!(state.cycle_count, 0);
    }

    #[tokio::test]
    async fn test_loop_stop() {
        let config = LoopConfig::default();
        let controller = AutonomousLoopController::new(config).await.unwrap();
        controller.stop().await.unwrap();
        let state = controller.get_state().await;
        assert!(!state.is_running);
    }

    #[tokio::test]
    async fn test_failure_threshold() {
        let config = LoopConfig {
            failure_threshold: 0.5,
            ..Default::default()
        };
        let controller = AutonomousLoopController::new(config).await.unwrap();

        // Simulate failures
        {
            let mut state = controller.state.write().await;
            state.cycle_count = 10;
            state.failure_count = 6; // 60% failure rate
        }

        let should_continue = controller.should_continue().await.unwrap();
        assert!(!should_continue);
    }
}
