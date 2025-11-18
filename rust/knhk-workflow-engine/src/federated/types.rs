//! Core data types for federated learning

use serde::{Deserialize, Serialize};

/// Experience sample from workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Workflow state representation
    pub state: Vec<f32>,
    /// Action taken (workflow decision)
    pub action: usize,
    /// Reward received (performance metric)
    pub reward: f64,
    /// Next state after action
    pub next_state: Vec<f32>,
    /// Whether episode terminated
    pub done: bool,
}

/// Gradient vector for model updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradients {
    /// Gradient values per parameter
    pub values: Vec<f32>,
    /// Agent ID that computed gradients
    pub agent_id: String,
    /// Timestamp of computation
    pub timestamp: i64,
    /// Training round number
    pub round: u64,
}

/// Aggregated gradients with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedGradients {
    /// Aggregated gradient values
    pub values: Vec<f32>,
    /// Number of agents contributed
    pub num_agents: usize,
    /// Detected Byzantine agents
    pub byzantine_agents: Vec<String>,
    /// Aggregation round
    pub round: u64,
    /// Timestamp
    pub timestamp: i64,
}

/// Convergence status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConvergenceStatus {
    /// Model has converged
    Converged {
        kl_divergence: f64,
        rounds_completed: u64,
    },
    /// Still training
    Training {
        kl_divergence: f64,
        rounds_completed: u64,
        estimated_rounds_remaining: u64,
    },
}

/// Metrics from local training round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTrainingMetrics {
    /// Training loss
    pub loss: f64,
    /// Number of epochs completed
    pub epochs: usize,
    /// Batch size used
    pub batch_size: usize,
    /// Training duration (ms)
    pub duration_ms: u64,
    /// Experiences trained on
    pub experiences_count: usize,
}

/// Metrics from federated learning round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedRoundMetrics {
    /// Round number
    pub round: u64,
    /// Total duration (ms)
    pub total_duration_ms: u64,
    /// Local training duration (ms)
    pub local_training_ms: u64,
    /// Aggregation duration (ms)
    pub aggregation_ms: u64,
    /// Convergence check duration (ms)
    pub convergence_ms: u64,
    /// Model distribution duration (ms)
    pub distribution_ms: u64,
    /// Number of agents participated
    pub agents_count: usize,
    /// Byzantine agents detected
    pub byzantine_count: usize,
    /// KL divergence from previous model
    pub kl_divergence: f64,
    /// Average local loss
    pub avg_local_loss: f64,
    /// Convergence status
    pub converged: bool,
}

/// Federated learning errors
#[derive(Debug, thiserror::Error)]
pub enum FederatedError {
    #[error("Byzantine attack detected: {0}")]
    ByzantineAttack(String),

    #[error("Insufficient quorum: got {got}, need {need}")]
    InsufficientQuorum { got: usize, need: usize },

    #[error("Convergence failed: KL divergence {kl} > threshold {threshold}")]
    ConvergenceFailed { kl: f64, threshold: f64 },

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Aggregation error: {0}")]
    AggregationError(String),

    #[error("Training error: {0}")]
    TrainingError(String),

    #[error("Experience buffer error: {0}")]
    BufferError(String),
}
