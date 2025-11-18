//! Core traits for federated learning components

use super::types::*;
use async_trait::async_trait;
use std::fmt::Debug;

/// Local model trained by each agent
///
/// # Trait Safety
/// This trait is object-safe (dyn compatible) - no async methods.
/// All methods are synchronous to allow `dyn LocalModel` usage.
pub trait LocalModel: Send + Sync + Debug {
    /// Compute loss on a batch of experiences
    fn compute_loss(&self, batch: &[Experience]) -> Result<f64, FederatedError>;

    /// Compute gradients via backpropagation
    fn compute_gradients(&self, batch: &[Experience]) -> Result<Gradients, FederatedError>;

    /// Apply gradient updates to model
    fn apply_gradients(&mut self, gradients: &Gradients) -> Result<(), FederatedError>;

    /// Predict action for given state
    fn predict(&self, state: &[f32]) -> Result<usize, FederatedError>;

    /// Serialize model parameters
    fn serialize_params(&self) -> Result<Vec<f32>, FederatedError>;

    /// Deserialize model parameters
    fn load_params(&mut self, params: &[f32]) -> Result<(), FederatedError>;
}

/// Experience buffer for replay
pub trait ExperienceBuffer: Send + Sync + Debug {
    /// Add experience to buffer
    fn push(&mut self, experience: Experience);

    /// Sample batch for training
    fn sample(&self, batch_size: usize) -> Vec<Experience>;

    /// Current buffer size
    fn len(&self) -> usize;

    /// Check if buffer is empty
    fn is_empty(&self) -> bool;
}

/// Optimizer for gradient descent
pub trait Optimizer: Send + Sync + Debug {
    /// Apply optimization step
    fn step(&mut self, gradients: &Gradients, model: &mut dyn LocalModel) -> Result<(), FederatedError>;

    /// Get current learning rate
    fn learning_rate(&self) -> f64;

    /// Set learning rate (for decay)
    fn set_learning_rate(&mut self, lr: f64);
}

/// Byzantine-robust aggregator
#[async_trait]
pub trait ByzantineRobustAggregator: Send + Sync + Debug {
    /// Aggregate gradients with Byzantine tolerance
    ///
    /// # Guarantees
    /// - Tolerates up to f < n/3 Byzantine gradients
    /// - Uses median for robust aggregation
    /// - Detects and reports Byzantine agents
    async fn aggregate(
        &self,
        gradients: Vec<Gradients>,
        quorum_size: usize,
    ) -> Result<AggregatedGradients, FederatedError>;
}

/// Convergence validator
///
/// # Trait Safety
/// This trait is object-safe (dyn compatible) - no async methods.
pub trait ConvergenceValidator: Send + Sync + Debug {
    /// Check if model has converged
    ///
    /// # Convergence Criteria
    /// - KL divergence < 0.01 between old and new model
    /// - At least MIN_ROUNDS (10) completed
    fn check_convergence(
        &self,
        old_params: &[f32],
        new_params: &[f32],
        round: u64,
    ) -> Result<ConvergenceStatus, FederatedError>;
}

/// Local training coordinator for each agent
#[async_trait]
pub trait LocalTrainer: Send + Sync + Debug {
    /// Run one local training round
    ///
    /// # Process
    /// 1. Sample experiences from buffer
    /// 2. Compute loss on local model
    /// 3. Backpropagate gradients
    /// 4. Apply optimizer step
    /// 5. Emit telemetry
    ///
    /// # Performance
    /// - Target: <100ms per round (async, off hot-path)
    async fn train_local_round(
        &mut self,
        num_epochs: usize,
        batch_size: usize,
    ) -> Result<LocalTrainingMetrics, FederatedError>;

    /// Get current local model (immutable)
    fn model(&self) -> &dyn LocalModel;

    /// Get mutable local model
    fn model_mut(&mut self) -> &mut dyn LocalModel;

    /// Get experience buffer (immutable)
    fn buffer(&self) -> &dyn ExperienceBuffer;

    /// Get mutable experience buffer
    fn buffer_mut(&mut self) -> &mut dyn ExperienceBuffer;
}

/// Main federated learning coordinator
#[async_trait]
pub trait FederatedCoordinator: Send + Sync + Debug {
    /// Run one complete federated learning round
    ///
    /// # Process
    /// 1. All agents train locally (parallel)
    /// 2. Collect gradients from quorum
    /// 3. Byzantine-robust aggregation
    /// 4. Validate convergence
    /// 5. Broadcast global model
    /// 6. Integrate with MAPE-K
    ///
    /// # Performance
    /// - Target: <150ms total round time
    async fn run_federated_round(&mut self) -> Result<FederatedRoundMetrics, FederatedError>;

    /// Start continuous federated learning loop
    async fn start_continuous_learning(
        &mut self,
        interval_ms: u64,
    ) -> Result<(), FederatedError>;

    /// Get current global model (immutable)
    fn global_model(&self) -> &dyn LocalModel;

    /// Get convergence status
    fn convergence_status(&self) -> &ConvergenceStatus;
}
