// Phase 6: Neural Integration - Core Trait Definitions
// This file contains all trait boundaries for the neural integration layer

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Result type for neural operations
pub type NeuralResult<T> = Result<T, NeuralError>;

/// Error types for neural operations
#[derive(Debug, thiserror::Error)]
pub enum NeuralError {
    #[error("Model load failed: {0}")]
    ModelLoadError(String),

    #[error("Model save failed: {0}")]
    ModelSaveError(String),

    #[error("Inference failed: {0}")]
    InferenceError(String),

    #[error("Training failed: {0}")]
    TrainingError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),
}

// ============================================================================
// CORE NEURAL MODEL TRAIT
// ============================================================================

/// Base trait for all neural network models
///
/// All models MUST:
/// - Support async inference (<1ms latency)
/// - Implement Send + Sync for multi-threaded execution
/// - Provide serialization/deserialization
/// - Emit telemetry for all operations
pub trait NeuralModel: Send + Sync {
    /// Input type for the model
    type Input: Clone + Send;

    /// Output type for the model
    type Output: Clone + Send;

    /// Gradient type for backpropagation
    type Gradient: Clone + Send;

    /// Forward pass (inference)
    ///
    /// MUST complete in <1ms for hot path models
    fn forward(&mut self, input: &Self::Input) -> Self::Output;

    /// Backward pass (training)
    ///
    /// Only called during offline training, no latency constraints
    fn backward(&mut self, gradient: &Self::Gradient, learning_rate: f32);

    /// Save model to disk (async)
    async fn save(&self, path: &Path) -> NeuralResult<()>;

    /// Load model from disk (async)
    async fn load(path: &Path) -> NeuralResult<Self> where Self: Sized;

    /// Get model metadata
    fn metadata(&self) -> ModelMetadata;

    /// Reset model to initial state
    fn reset(&mut self);
}

/// Metadata for neural models
#[derive(Clone, Debug)]
pub struct ModelMetadata {
    pub name: String,
    pub version: String,
    pub parameters_count: usize,
    pub input_size: usize,
    pub output_size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_trained: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// REINFORCEMENT LEARNING TRAITS
// ============================================================================

/// State in a reinforcement learning problem
pub trait State: Clone + Send + Sync + std::hash::Hash + Eq {
    /// Convert state to feature vector for function approximation
    fn to_feature_vector(&self) -> Vec<f32>;

    /// Get state dimensionality
    fn dimension() -> usize;
}

/// Action in a reinforcement learning problem
pub trait Action: Clone + Send + Sync + std::hash::Hash + Eq {
    /// Convert action to index (for Q-table)
    fn to_idx(&self) -> usize;

    /// Create action from index
    fn from_idx(idx: usize) -> Self;

    /// Get total number of possible actions
    fn action_space_size() -> usize;

    /// Sample random action (for exploration)
    fn random() -> Self;
}

/// Transition in reinforcement learning (s, a, r, s')
#[derive(Clone, Debug)]
pub struct Transition<S: State, A: Action> {
    pub state: S,
    pub action: A,
    pub reward: f32,
    pub next_state: S,
}

/// SARSA transition (s, a, r, s', a')
#[derive(Clone, Debug)]
pub struct SARSATransition<S: State, A: Action> {
    pub state: S,
    pub action: A,
    pub reward: f32,
    pub next_state: S,
    pub next_action: A,
}

/// Base trait for reinforcement learning agents
pub trait RLAgent<S: State, A: Action>: Send + Sync {
    /// Select action given current state
    ///
    /// MUST complete in <1ms for hot path
    async fn select_action(&self, state: &S) -> A;

    /// Learn from a single transition
    ///
    /// Can be async/slow, runs in background
    async fn learn(&mut self, transition: &Transition<S, A>);

    /// Learn from batch of transitions (more efficient)
    async fn learn_batch(&mut self, transitions: &[Transition<S, A>]) {
        for transition in transitions {
            self.learn(transition).await;
        }
    }

    /// Get current exploration rate (ε)
    fn exploration_rate(&self) -> f32;

    /// Set exploration rate
    fn set_exploration_rate(&mut self, epsilon: f32);

    /// Get Q-value for state-action pair (if applicable)
    fn get_q_value(&self, state: &S, action: &A) -> Option<f32>;

    /// Get agent metadata
    fn metadata(&self) -> AgentMetadata;
}

/// Metadata for RL agents
#[derive(Clone, Debug)]
pub struct AgentMetadata {
    pub algorithm: String,
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub exploration_rate: f32,
    pub total_episodes: usize,
    pub total_steps: usize,
}

// ============================================================================
// PREDICTOR TRAITS
// ============================================================================

/// Base trait for predictive models
pub trait Predictor: Send + Sync {
    type Input: Clone + Send;
    type Output: Clone + Send;

    /// Make prediction
    ///
    /// MUST complete in <1ms for hot path predictors
    fn predict(&mut self, input: &Self::Input) -> Self::Output;

    /// Make prediction with confidence score
    fn predict_with_confidence(&mut self, input: &Self::Input) -> (Self::Output, f32) {
        (self.predict(input), 0.5)  // Default: 50% confidence
    }

    /// Train on batch of samples
    ///
    /// Offline operation, no latency constraints
    fn train(&mut self, samples: &[(Self::Input, Self::Output)], epochs: usize, learning_rate: f32);

    /// Evaluate accuracy on test set
    fn evaluate(&mut self, test_set: &[(Self::Input, Self::Output)]) -> f32;

    /// Get model size (bytes)
    fn size_bytes(&self) -> usize;
}

/// Duration predictor (regression)
pub trait DurationPredictor: Predictor<Input = WorkflowFeatures, Output = f32> {
    /// Predict duration with percentiles
    fn predict_percentiles(&mut self, input: &WorkflowFeatures) -> DurationPercentiles;
}

#[derive(Clone, Debug)]
pub struct DurationPercentiles {
    pub p50: f32,
    pub p95: f32,
    pub p99: f32,
}

/// Success predictor (binary classification)
pub trait SuccessPredictor: Predictor<Input = WorkflowFeatures, Output = f32> {
    /// Predict success probability (0.0-1.0)
    fn predict_success_probability(&mut self, input: &WorkflowFeatures) -> f32 {
        self.predict(input).clamp(0.0, 1.0)
    }
}

/// Anomaly detector
pub trait AnomalyDetector: Send + Sync {
    /// Check if input is anomalous
    ///
    /// Returns (is_anomaly, anomaly_score)
    fn is_anomaly(&mut self, input: &WorkflowFeatures) -> (bool, f32);

    /// Train on normal samples
    fn train_on_normal(&mut self, normal_samples: &[WorkflowFeatures], epochs: usize, learning_rate: f32);

    /// Set detection threshold
    fn set_threshold(&mut self, threshold: f32);

    /// Get current threshold
    fn threshold(&self) -> f32;
}

// ============================================================================
// EVOLVABLE DESCRIPTOR TRAITS
// ============================================================================

/// Trait for descriptor evolution
pub trait EvolvableDescriptor: Clone + Send + Sync {
    /// Create mutated variant
    ///
    /// mutation_rate: probability of each gene mutating (0.0-1.0)
    fn mutate(&self, mutation_rate: f32) -> Self;

    /// Crossover with another descriptor
    ///
    /// Returns child descriptor
    fn crossover(&self, other: &Self) -> Self;

    /// Compute fitness from execution history
    ///
    /// Higher fitness = better performance
    fn fitness(&self, history: &ExecutionHistory) -> f32;

    /// Random initialization (for population diversity)
    fn random() -> Self;

    /// Get genome representation
    fn genome(&self) -> DescriptorGenome;

    /// Set genome
    fn set_genome(&mut self, genome: DescriptorGenome);
}

/// Descriptor genome (evolvable parameters)
#[derive(Clone, Debug)]
pub struct DescriptorGenome {
    /// Task timeout multipliers (0.5-2.0)
    pub timeout_multipliers: Vec<f32>,

    /// Retry counts (0-10)
    pub retry_counts: Vec<u8>,

    /// Resource allocation levels
    pub resource_levels: Vec<ResourceLevel>,

    /// Parallelization strategies
    pub parallel_strategies: Vec<ParallelStrategy>,

    /// Backoff strategies
    pub backoff_strategies: Vec<BackoffStrategy>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParallelStrategy {
    Sequential,
    Parallel,
    Adaptive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackoffStrategy {
    Constant,
    Linear,
    Exponential,
}

// ============================================================================
// EXPERIENCE REPLAY TRAITS
// ============================================================================

/// Experience replay buffer for efficient learning
pub trait ExperienceReplay<S: State, A: Action>: Send + Sync {
    /// Add transition to buffer
    fn add(&mut self, transition: Transition<S, A>, priority: f32);

    /// Sample batch of transitions
    ///
    /// Returns (transition, importance_weight) pairs
    fn sample(&self, batch_size: usize) -> Vec<(Transition<S, A>, f32)>;

    /// Get buffer size
    fn len(&self) -> usize;

    /// Check if buffer is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get buffer capacity
    fn capacity(&self) -> usize;

    /// Clear buffer
    fn clear(&mut self);

    /// Update priorities (for prioritized replay)
    fn update_priorities(&mut self, indices: &[usize], priorities: &[f32]);
}

// ============================================================================
// FEDERATED LEARNING TRAITS
// ============================================================================

/// Federated learning coordinator
pub trait FederatedCoordinator<S: State, A: Action>: Send + Sync {
    /// Register local agent
    fn register_agent(&mut self, agent_id: String, agent: Arc<RwLock<dyn RLAgent<S, A>>>);

    /// Aggregate knowledge from all agents
    async fn aggregate(&mut self) -> NeuralResult<()>;

    /// Broadcast global model to all agents
    async fn broadcast(&mut self) -> NeuralResult<()>;

    /// Get aggregation strategy
    fn strategy(&self) -> AggregationStrategy;

    /// Set synchronization interval
    fn set_sync_interval(&mut self, interval: usize);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregationStrategy {
    /// Average Q-values across agents
    Average,

    /// Weight by agent experience
    WeightedByExperience,

    /// Take maximum Q-value
    MaxValue,

    /// Consensus-based (all agents must agree)
    Consensus,
}

// ============================================================================
// MAPE-K INTEGRATION TRAITS
// ============================================================================

/// MAPE-K Analyze stage (learning)
pub trait MAPEKAnalyze: Send + Sync {
    type ExecutionResult;
    type AnalysisResult;

    /// Analyze execution result and learn
    async fn analyze(&mut self, execution: &Self::ExecutionResult) -> NeuralResult<Self::AnalysisResult>;

    /// Get recommendations for next execution
    fn recommendations(&self) -> Vec<Recommendation>;
}

/// MAPE-K Plan stage (RL-driven planning)
pub trait MAPEKPlan: Send + Sync {
    type State: State;
    type Action: Action;

    /// Plan next action based on analysis
    async fn plan(&self, state: &Self::State, analysis: &AnalysisResult) -> Self::Action;

    /// Get plan confidence
    fn confidence(&self) -> f32;
}

/// MAPE-K Knowledge store
pub trait MAPEKKnowledge: Send + Sync {
    /// Persist model to knowledge store
    async fn persist_model(&self, model_name: &str, model: &dyn NeuralModel<Input = Vec<f32>, Output = Vec<f32>>) -> NeuralResult<()>;

    /// Load model from knowledge store
    async fn load_model(&self, model_name: &str) -> NeuralResult<Box<dyn NeuralModel<Input = Vec<f32>, Output = Vec<f32>>>>;

    /// Store execution vector for similarity search
    async fn store_execution_vector(&self, execution_id: &str, features: &[f32], metadata: serde_json::Value) -> NeuralResult<()>;

    /// Query similar executions
    async fn query_similar(&self, features: &[f32], k: usize) -> NeuralResult<Vec<SimilarExecution>>;
}

#[derive(Clone, Debug)]
pub struct SimilarExecution {
    pub execution_id: String,
    pub similarity: f32,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct Recommendation {
    pub action: String,
    pub confidence: f32,
    pub rationale: String,
}

#[derive(Clone, Debug)]
pub struct AnalysisResult {
    pub predicted_duration: f32,
    pub predicted_success: f32,
    pub is_anomaly: bool,
    pub anomaly_score: f32,
    pub recommended_action: Option<String>,
}

// ============================================================================
// TELEMETRY TRAITS
// ============================================================================

/// Telemetry emission for neural operations
pub trait NeuralTelemetry: Send + Sync {
    /// Emit telemetry for inference
    fn emit_inference(&self, model_name: &str, input_size: usize, output_size: usize, latency_us: u64);

    /// Emit telemetry for training
    fn emit_training(&self, model_name: &str, samples: usize, epochs: usize, loss: f32);

    /// Emit telemetry for RL action selection
    fn emit_rl_action(&self, state: &str, action: &str, q_value: f32, is_exploration: bool);

    /// Emit telemetry for learning
    fn emit_learning(&self, algorithm: &str, reward: f32, td_error: f32);

    /// Emit telemetry for anomaly detection
    fn emit_anomaly(&self, is_anomaly: bool, score: f32);

    /// Emit telemetry for evolution
    fn emit_evolution(&self, generation: u32, best_fitness: f32, avg_fitness: f32);
}

// ============================================================================
// PLACEHOLDER TYPES (to be defined elsewhere)
// ============================================================================

/// Workflow features (64-128 dimensions)
#[derive(Clone, Debug)]
pub struct WorkflowFeatures {
    // Task properties
    pub task_count: f32,
    pub avg_task_duration_ms: f32,
    // ... (64 total fields)
}

impl WorkflowFeatures {
    pub fn to_vector(&self) -> Vec<f32> {
        vec![self.task_count, self.avg_task_duration_ms]  // Simplified
    }
}

/// Execution history
#[derive(Clone, Debug)]
pub struct ExecutionHistory {
    // Placeholder
}

impl ExecutionHistory {
    pub fn max_duration(&self) -> f32 { 1000.0 }
    pub fn max_cost(&self) -> f32 { 100.0 }
    pub fn max_p99_duration(&self) -> f32 { 2000.0 }
}

// ============================================================================
// EXAMPLE IMPLEMENTATIONS
// ============================================================================

/// Example: Simple Q-Learning agent
pub struct QLearningAgent<S: State, A: Action> {
    pub q_table: Arc<RwLock<HashMap<S, Vec<f32>>>>,
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub epsilon: f32,
    pub episode_count: usize,
    pub step_count: usize,
}

#[async_trait::async_trait]
impl<S: State, A: Action> RLAgent<S, A> for QLearningAgent<S, A> {
    async fn select_action(&self, state: &S) -> A {
        if fastrand::f32() < self.epsilon {
            A::random()
        } else {
            let table = self.q_table.read().await;
            let q_values = table.get(state).cloned().unwrap_or_default();
            let best_idx = q_values.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            A::from_idx(best_idx)
        }
    }

    async fn learn(&mut self, transition: &Transition<S, A>) {
        let mut table = self.q_table.write().await;

        let q_next_max = table.get(&transition.next_state)
            .map(|v| v.iter().copied().fold(f32::NEG_INFINITY, f32::max))
            .unwrap_or(0.0);

        let target = transition.reward + self.discount_factor * q_next_max;

        let q_entry = table.entry(transition.state.clone())
            .or_insert_with(|| vec![0.0; A::action_space_size()]);
        let action_idx = transition.action.to_idx();
        q_entry[action_idx] = (1.0 - self.learning_rate) * q_entry[action_idx]
                             + self.learning_rate * target;

        self.step_count += 1;
    }

    fn exploration_rate(&self) -> f32 {
        self.epsilon
    }

    fn set_exploration_rate(&mut self, epsilon: f32) {
        self.epsilon = epsilon;
    }

    fn get_q_value(&self, state: &S, action: &A) -> Option<f32> {
        // Note: This is a blocking read, should be used carefully
        None  // Placeholder
    }

    fn metadata(&self) -> AgentMetadata {
        AgentMetadata {
            algorithm: "Q-Learning".to_string(),
            learning_rate: self.learning_rate,
            discount_factor: self.discount_factor,
            exploration_rate: self.epsilon,
            total_episodes: self.episode_count,
            total_steps: self.step_count,
        }
    }
}
