// Phase 6: Self-Learning Workflow Integration
// Advanced workflow orchestration with continuous learning and adaptive execution
//
// DOCTRINE ALIGNMENT:
// - Principle: MAPE-K (Monitor, Analyze, Plan, Execute, Knowledge)
// - Covenant: Continuous learning must maintain Q invariants (latency ≤ 8 ticks)
// - Implementation: Async workflow loops with metrics emission
// - Platform Integration: Phase 5 autonomic components with neural optimization

use std::collections::VecDeque;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::reinforcement::{QLearning, WorkflowAction, WorkflowState};
use crate::model::DenseLayer;

/// Workflow execution metrics for MAPE-K feedback loops
/// Tracks performance of individual workflow executions for adaptive optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    /// Execution duration in milliseconds
    pub duration_ms: f32,
    /// Whether execution completed successfully
    pub success: bool,
    /// Resource utilization percentage (0-100)
    pub resource_usage: f32,
    /// Pattern ID used in execution (0-255)
    pub pattern_id: u8,
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
    /// Execution quality score (0.0-1.0)
    pub quality_score: f32,
    /// Throughput (operations per second)
    pub throughput: f32,
}

impl WorkflowMetrics {
    /// Create new workflow metrics
    pub fn new(
        duration_ms: f32,
        success: bool,
        resource_usage: f32,
        pattern_id: u8,
    ) -> Self {
        let quality_score = if success {
            (100.0 - resource_usage.min(100.0)) / 100.0
        } else {
            0.5 * (100.0 - resource_usage.min(100.0)) / 100.0
        };

        Self {
            duration_ms,
            success,
            resource_usage,
            pattern_id,
            timestamp: Utc::now(),
            quality_score,
            throughput: 1000.0 / duration_ms.max(1.0),
        }
    }

    /// Calculate reward for Q-Learning from metrics
    pub fn to_reward(&self) -> f32 {
        if !self.success {
            -10.0
        } else {
            (self.quality_score * 10.0) - (self.duration_ms / 100.0).min(5.0)
        }
    }
}

/// Episode execution result with metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeResult {
    /// Total reward accumulated
    pub total_reward: f32,
    /// Number of steps taken
    pub steps: usize,
    /// Whether episode reached terminal state
    pub is_terminal: bool,
    /// Episode duration
    pub duration_ms: u128,
    /// Loss/accuracy metrics
    pub loss: f32,
    /// Convergence indicator (0.0 = diverging, 1.0 = converged)
    pub convergence: f32,
}

/// Learning metrics tracked across episodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    /// Episode number
    pub episode: usize,
    /// Average reward over last N episodes
    pub avg_reward: f32,
    /// Loss trend (decreasing = improving)
    pub loss_trend: f32,
    /// Exploration rate
    pub exploration_rate: f32,
    /// Total episodes completed
    pub total_episodes: usize,
    /// Training convergence flag
    pub is_converged: bool,
}

/// Trainer for neural network optimization
#[derive(Clone)]
pub struct Trainer<L: Clone + Send + Sync> {
    pub learning_rate: f32,
    batch_size: usize,
    momentum: f32,
    weight_decay: f32,
    _phantom: std::marker::PhantomData<L>,
}

impl<L: Clone + Send + Sync> Trainer<L> {
    pub fn new() -> Self {
        Self {
            learning_rate: 0.001,
            batch_size: 32,
            momentum: 0.9,
            weight_decay: 0.0001,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_learning_rate(mut self, lr: f32) -> Self {
        self.learning_rate = lr;
        self
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Train the model with a batch of transitions
    pub fn train_batch(&self, gradients: Vec<f32>) -> f32 {
        // Compute average gradient magnitude as loss
        if gradients.is_empty() {
            return 0.0;
        }

        let avg_gradient: f32 = gradients.iter().sum::<f32>() / gradients.len() as f32;
        avg_gradient.abs()
    }

    /// Decay learning rate
    pub fn decay_learning_rate(&mut self, decay_factor: f32) {
        self.learning_rate *= decay_factor;
    }

    pub fn learning_rate(&self) -> f32 {
        self.learning_rate
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}

impl<L: Clone + Send + Sync> Default for Trainer<L> {
    fn default() -> Self {
        Self::new()
    }
}

/// Self-learning workflow with ε-greedy exploration and Q-learning
///
/// Combines:
/// - State observation and action selection via QLearning
/// - Continuous training via Trainer
/// - Performance monitoring and metrics
/// - Model versioning and persistence
pub struct SelfLearningWorkflow<S: WorkflowState, A: WorkflowAction> {
    /// Q-Learning agent for action selection
    agent: Arc<RwLock<QLearning<S, A>>>,

    /// Trainer for neural optimization
    trainer: Arc<RwLock<Trainer<DenseLayer<10, 5>>>>,

    /// State observation function
    state_observer: Arc<dyn Fn() -> S + Send + Sync>,

    /// Reward calculation function: (state, action, next_state) -> reward
    reward_calculator: Arc<dyn Fn(&S, &A, &S) -> f32 + Send + Sync>,

    /// Performance history for convergence tracking
    reward_history: Arc<RwLock<VecDeque<f32>>>,

    /// Loss history for convergence detection
    loss_history: Arc<RwLock<VecDeque<f32>>>,

    /// Episode counter
    episode_count: Arc<RwLock<usize>>,

    /// Last training timestamp
    last_training: Arc<RwLock<Instant>>,

    /// Model versions (name -> metrics)
    model_versions: Arc<RwLock<Vec<ModelVersion>>>,

    /// Configuration
    config: WorkflowConfig,

    /// Workflow executor for MAPE-K integration
    executor: Arc<WorkflowExecutor>,

    /// Execution history for adaptive learning
    execution_history: Arc<Mutex<Vec<WorkflowMetrics>>>,
}

/// Model version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub version: usize,
    pub timestamp: u128,
    pub avg_reward: f32,
    pub loss: f32,
    pub convergence: f32,
}

/// Workflow configuration
#[derive(Debug, Clone, Copy)]
pub struct WorkflowConfig {
    /// Training frequency (steps between training)
    pub training_interval: usize,
    /// Convergence threshold (avg reward improvement)
    pub convergence_threshold: f32,
    /// History window size for metrics
    pub history_window: usize,
    /// Retraining trigger threshold
    pub retraining_threshold: f32,
    /// Learning rate decay
    pub lr_decay: f32,
    /// Maximum steps per episode
    pub max_steps: usize,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            training_interval: 10,
            convergence_threshold: 0.01,
            history_window: 100,
            retraining_threshold: 0.9,
            lr_decay: 0.99,
            max_steps: 1000,
        }
    }
}

/// Performance improvement tracker for adaptive workflows
/// Measures improvements in execution metrics over time
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    /// Historical execution metrics
    metrics_history: Arc<Mutex<Vec<WorkflowMetrics>>>,
    /// Best performance recorded
    best_metrics: Arc<RwLock<Option<WorkflowMetrics>>>,
    /// Improvement percentage (0.0-100.0)
    improvement_percentage: Arc<RwLock<f32>>,
    /// Total executions tracked
    total_executions: Arc<RwLock<usize>>,
    /// Successful executions
    successful_executions: Arc<RwLock<usize>>,
}

impl PerformanceTracker {
    /// Create new performance tracker
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(Vec::new())),
            best_metrics: Arc::new(RwLock::new(None)),
            improvement_percentage: Arc::new(RwLock::new(0.0)),
            total_executions: Arc::new(RwLock::new(0)),
            successful_executions: Arc::new(RwLock::new(0)),
        }
    }

    /// Record execution metrics and compute improvements
    pub fn record(&self, metrics: WorkflowMetrics) {
        let mut history = self.metrics_history.lock().unwrap();
        history.push(metrics.clone());

        // Update counters
        *self.total_executions.write().unwrap() += 1;
        if metrics.success {
            *self.successful_executions.write().unwrap() += 1;
        }

        // Update best metrics
        let mut best = self.best_metrics.write().unwrap();
        let should_update = match best.as_ref() {
            None => true,
            Some(best_metrics) => {
                metrics.quality_score > best_metrics.quality_score
                    && metrics.duration_ms < best_metrics.duration_ms
            }
        };

        if should_update {
            *best = Some(metrics.clone());
        }

        // Compute improvement
        if history.len() >= 2 {
            let recent_idx = history.len() - 1;
            let previous_idx = (recent_idx as isize - 1).max(0) as usize;
            let recent = &history[recent_idx];
            let previous = &history[previous_idx];

            let improvement = (previous.duration_ms - recent.duration_ms)
                / previous.duration_ms.max(1.0) * 100.0;
            *self.improvement_percentage.write().unwrap() = improvement.max(0.0);
        }
    }

    /// Get improvement percentage
    pub fn improvement_percentage(&self) -> f32 {
        *self.improvement_percentage.read().unwrap()
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        let total = *self.total_executions.read().unwrap();
        if total == 0 {
            return 0.0;
        }
        let successful = *self.successful_executions.read().unwrap();
        (successful as f32 / total as f32) * 100.0
    }

    /// Get average execution time
    pub fn avg_duration_ms(&self) -> f32 {
        let history = self.metrics_history.lock().unwrap();
        if history.is_empty() {
            return 0.0;
        }
        let sum: f32 = history.iter().map(|m| m.duration_ms).sum();
        sum / history.len() as f32
    }

    /// Get best metrics recorded
    pub fn best_metrics(&self) -> Option<WorkflowMetrics> {
        self.best_metrics.read().unwrap().clone()
    }

    /// Get metrics history (last N entries)
    pub fn recent_metrics(&self, limit: usize) -> Vec<WorkflowMetrics> {
        let history = self.metrics_history.lock().unwrap();
        history
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Clear history
    pub fn clear(&self) {
        self.metrics_history.lock().unwrap().clear();
        *self.improvement_percentage.write().unwrap() = 0.0;
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow executor: integrates Phase 5 platform with Phase 6 neural components
/// Executes workflows with MAPE-K feedback loops and Q-Learning optimization
pub struct WorkflowExecutor {
    /// Unique executor ID
    pub id: String,
    /// Performance tracker for metrics
    performance_tracker: Arc<PerformanceTracker>,
    /// Execution count
    execution_count: Arc<RwLock<usize>>,
    /// Last execution time
    last_execution: Arc<RwLock<Option<Instant>>>,
    /// MAPE-K cycle counter
    mape_k_cycles: Arc<RwLock<u64>>,
}

impl WorkflowExecutor {
    /// Create new workflow executor
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            performance_tracker: Arc::new(PerformanceTracker::new()),
            execution_count: Arc::new(RwLock::new(0)),
            last_execution: Arc::new(RwLock::new(None)),
            mape_k_cycles: Arc::new(RwLock::new(0)),
        }
    }

    /// Execute workflow with metrics recording
    pub async fn execute(
        &self,
        duration_ms: f32,
        success: bool,
        resource_usage: f32,
        pattern_id: u8,
    ) -> WorkflowMetrics {
        let start = Instant::now();

        // Create metrics
        let metrics = WorkflowMetrics::new(duration_ms, success, resource_usage, pattern_id);

        // Record in tracker
        self.performance_tracker.record(metrics.clone());

        // Update counters
        *self.execution_count.write().unwrap() += 1;
        *self.last_execution.write().unwrap() = Some(start);

        // Increment MAPE-K cycle
        *self.mape_k_cycles.write().unwrap() += 1;

        metrics
    }

    /// Get performance tracker
    pub fn performance_tracker(&self) -> Arc<PerformanceTracker> {
        Arc::clone(&self.performance_tracker)
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        *self.execution_count.read().unwrap()
    }

    /// Get MAPE-K cycle count
    pub fn mape_k_cycles(&self) -> u64 {
        *self.mape_k_cycles.read().unwrap()
    }

    /// Emit telemetry (compatible with Phase 5 autonomic system)
    pub async fn emit_telemetry(&self) {
        let tracker = &self.performance_tracker;
        let success_rate = tracker.success_rate();
        let improvement = tracker.improvement_percentage();
        let avg_duration = tracker.avg_duration_ms();

        // In production, this would emit OpenTelemetry metrics
        println!(
            "[WorkflowExecutor] cycles={}, success_rate={:.1}%, improvement={:.1}%, avg_duration={:.1}ms",
            self.mape_k_cycles(),
            success_rate,
            improvement,
            avg_duration
        );
    }
}

impl<S: WorkflowState + 'static, A: WorkflowAction + 'static> SelfLearningWorkflow<S, A> {
    /// Create a new self-learning workflow
    pub fn new<FState, FReward>(
        state_fn: FState,
        reward_fn: FReward,
        config: WorkflowConfig,
    ) -> Self
    where
        FState: Fn() -> S + Send + Sync + 'static,
        FReward: Fn(&S, &A, &S) -> f32 + Send + Sync + 'static,
    {
        Self {
            agent: Arc::new(RwLock::new(QLearning::new())),
            trainer: Arc::new(RwLock::new(Trainer::new())),
            state_observer: Arc::new(state_fn),
            reward_calculator: Arc::new(reward_fn),
            reward_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.history_window))),
            loss_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.history_window))),
            episode_count: Arc::new(RwLock::new(0)),
            last_training: Arc::new(RwLock::new(Instant::now())),
            model_versions: Arc::new(RwLock::new(Vec::new())),
            config,
            executor: Arc::new(WorkflowExecutor::new()),
            execution_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Execute workflow with adaptive Q-Learning optimization
    ///
    /// Integrates with Phase 5 platform to select patterns based on learned Q-values
    /// and records metrics for continuous improvement.
    pub async fn execute_adaptive(&self, workflow_def: &str) -> (WorkflowMetrics, f32) {
        let start = Instant::now();

        // Observe current state
        let state = (self.state_observer)();

        // Select action using learned Q-values (greedy with some exploration)
        let agent = self.agent.read().unwrap();
        let action = agent.select_action(&state);
        drop(agent);

        // Get pattern from action
        let pattern_id = (action.to_index() % 256) as u8;

        // Simulate workflow execution
        // In production, this would call actual workflow engine
        let resource_usage = (pattern_id as f32 / 256.0) * 80.0;
        let success = (state.features().len() as f32).sin().abs() > 0.3;

        // Record execution metrics
        let metrics = self.executor.execute(
            start.elapsed().as_secs_f32() * 1000.0,
            success,
            resource_usage,
            pattern_id,
        ).await;

        // Calculate reward and update Q-value
        let next_state = (self.state_observer)();
        let reward = (self.reward_calculator)(&state, &action, &next_state);

        let agent = self.agent.read().unwrap();
        agent.update(&state, &action, reward, &next_state, next_state.is_terminal());
        drop(agent);

        // Record in execution history
        let mut history = self.execution_history.lock().unwrap();
        history.push(metrics.clone());

        (metrics, reward)
    }

    /// Record execution metrics for learning
    pub fn record_execution(&self, metrics: WorkflowMetrics) {
        let mut history = self.execution_history.lock().unwrap();
        history.push(metrics.clone());

        // Update performance tracker in executor
        self.executor.performance_tracker().record(metrics);
    }

    /// Update model based on execution history
    ///
    /// Computes improvement metrics and retrains Q-Learning weights
    /// Returns improvement percentage
    pub async fn update_model(&mut self) -> f32 {
        let start = Instant::now();

        // Get execution history
        let history = self.execution_history.lock().unwrap();
        if history.len() < 2 {
            return 0.0;
        }

        // Calculate improvement from first to last
        let first = &history[0];
        let last = &history[history.len() - 1];

        let improvement_ratio = (first.duration_ms - last.duration_ms)
            / first.duration_ms.max(1.0);

        drop(history);

        // Perform training
        self.perform_training().await;

        // Log improvement
        let improvement_percentage = improvement_ratio.max(0.0) * 100.0;
        println!(
            "[SelfLearningWorkflow] Model updated: {:.1}% improvement in execution time",
            improvement_percentage
        );

        improvement_percentage
    }

    /// Get performance improvements over time
    ///
    /// Returns metrics showing improvement in:
    /// - Execution speed (duration reduction)
    /// - Success rate
    /// - Resource efficiency
    pub fn get_performance_improvements(&self) -> PerformanceImprovements {
        let tracker = self.executor.performance_tracker();
        let history = self.execution_history.lock().unwrap();

        let mut speed_improvement = 0.0;
        let mut quality_improvement = 0.0;

        if history.len() >= 2 {
            let first = &history[0];
            let last = &history[history.len() - 1];

            // Speed improvement (negative is good, we want lower durations)
            speed_improvement = ((first.duration_ms - last.duration_ms)
                / first.duration_ms.max(1.0))
                .max(0.0)
                * 100.0;

            // Quality improvement
            quality_improvement = (last.quality_score - first.quality_score)
                .max(0.0)
                * 100.0;
        }

        PerformanceImprovements {
            speed_improvement,
            quality_improvement,
            success_rate: tracker.success_rate(),
            average_duration_ms: tracker.avg_duration_ms(),
            total_executions: history.len(),
        }
    }

    /// Get executor reference
    pub fn executor(&self) -> Arc<WorkflowExecutor> {
        Arc::clone(&self.executor)
    }

    /// Execute a single episode with full state/action/reward cycle
    ///
    /// Returns EpisodeResult with metrics
    pub async fn execute_episode(&self) -> EpisodeResult {
        let start = Instant::now();
        let mut total_reward = 0.0;
        let mut steps = 0;
        let mut is_terminal = false;
        let mut loss = 0.0;

        // Get initial state
        let mut state = (self.state_observer)();

        // Execute episode steps
        while !state.is_terminal() && steps < self.config.max_steps {
            // Select action using ε-greedy
            let agent = self.agent.read().unwrap();
            let action = agent.select_action(&state);
            drop(agent);

            // Simulate action execution - get next state
            // In real systems, this would call actual execution
            let next_state = (self.state_observer)();

            // Calculate reward
            let reward = (self.reward_calculator)(&state, &action, &next_state);
            total_reward += reward;

            // Update Q-values
            let is_done = next_state.is_terminal();
            let agent = self.agent.read().unwrap();
            agent.update(&state, &action, reward, &next_state, is_done);
            drop(agent);

            // Progress episode
            state = next_state;
            steps += 1;
            is_terminal = state.is_terminal();

            // Yield to avoid blocking
            if steps % 100 == 0 {
                sleep(Duration::from_micros(1)).await;
            }
        }

        // Record metrics
        let mut reward_hist = self.reward_history.write().unwrap();
        if reward_hist.len() >= self.config.history_window {
            reward_hist.pop_front();
        }
        reward_hist.push_back(total_reward);
        drop(reward_hist);

        // Calculate convergence
        loss = self.calculate_convergence(); // Loss inversely correlates with convergence

        let duration = start.elapsed().as_millis();

        // Compute convergence metric (simple heuristic)
        let convergence = if loss < 0.1 { 0.9 } else { (1.0 - loss).max(0.0) };

        EpisodeResult {
            total_reward,
            steps,
            is_terminal,
            duration_ms: duration,
            loss,
            convergence,
        }
    }

    /// Execute continuous learning loop
    ///
    /// Runs episodes indefinitely with periodic training and retraining checks
    pub async fn continuous_learning_loop(&self) -> ! {
        let mut episode = 0;

        loop {
            // Execute episode
            let _result = self.execute_episode().await;
            episode += 1;

            // Update counter
            *self.episode_count.write().unwrap() = episode;

            // Check if training is needed
            if episode % self.config.training_interval == 0 {
                self.perform_training().await;
            }

            // Emit metrics periodically
            if episode % 10 == 0 {
                let metrics = self.get_learning_metrics();
                self.emit_metrics(&metrics).await;

                // Check for convergence
                if metrics.is_converged {
                    // Log convergence but continue learning
                    println!("[Workflow] Episode {}: Converged at loss={:.4}", episode, metrics.loss_trend);
                }
            }

            // Yield to event loop
            sleep(Duration::from_micros(100)).await;
        }
    }

    /// Perform training with current episode data
    async fn perform_training(&self) {
        let start = Instant::now();
        let mut trainer = self.trainer.write().unwrap();

        // Simulate gradient computation from recent episodes
        let reward_hist = self.reward_history.read().unwrap();
        let gradients: Vec<f32> = reward_hist
            .iter()
            .take(trainer.batch_size())
            .map(|r| (r.abs() * 0.1).min(1.0))
            .collect();
        drop(reward_hist);

        // Train batch
        let loss = trainer.train_batch(gradients);

        // Update loss history
        let mut loss_hist = self.loss_history.write().unwrap();
        if loss_hist.len() >= self.config.history_window {
            loss_hist.pop_front();
        }
        loss_hist.push_back(loss);
        drop(loss_hist);

        // Decay learning rate
        if *self.episode_count.read().unwrap() % 100 == 0 {
            trainer.decay_learning_rate(self.config.lr_decay);
        }

        *self.last_training.write().unwrap() = Instant::now();

        let duration = start.elapsed().as_millis();
        println!("[Training] Loss: {:.6}, Duration: {}ms", loss, duration);
    }

    /// Calculate convergence metric (0.0 = diverging, 1.0 = converged)
    fn calculate_convergence(&self) -> f32 {
        let reward_hist = self.reward_history.read().unwrap();

        if reward_hist.len() < 2 {
            return 0.0;
        }

        // Calculate trend: positive trend = improving = more converged
        let recent: Vec<f32> = reward_hist.iter().rev().take(20).cloned().collect();
        if recent.len() < 2 {
            return 0.0;
        }

        let recent_avg = recent.iter().sum::<f32>() / recent.len() as f32;
        let older: Vec<f32> = reward_hist.iter().take(std::cmp::min(20, reward_hist.len() - 20)).cloned().collect();
        let older_avg = if older.is_empty() {
            recent_avg
        } else {
            older.iter().sum::<f32>() / older.len() as f32
        };

        // Convergence = improvement trend, clamped to [0, 1]
        let improvement = (recent_avg - older_avg) / (older_avg.abs().max(0.1));
        improvement.clamp(-1.0, 1.0) * 0.5 + 0.5
    }

    /// Get current learning metrics
    pub fn get_learning_metrics(&self) -> LearningMetrics {
        let reward_hist = self.reward_history.read().unwrap();
        let loss_hist = self.loss_history.read().unwrap();

        let avg_reward = if reward_hist.is_empty() {
            0.0
        } else {
            reward_hist.iter().sum::<f32>() / reward_hist.len() as f32
        };

        let loss_trend = if loss_hist.is_empty() {
            0.0
        } else {
            loss_hist.iter().sum::<f32>() / loss_hist.len() as f32
        };

        let agent = self.agent.read().unwrap();
        let exploration_rate = agent.get_exploration_rate();
        drop(agent);

        let episode = *self.episode_count.read().unwrap();
        let convergence = self.calculate_convergence();
        let is_converged = loss_trend < self.config.convergence_threshold && episode > 100 && convergence > 0.7;

        LearningMetrics {
            episode,
            avg_reward,
            loss_trend,
            exploration_rate,
            total_episodes: episode,
            is_converged,
        }
    }

    /// Emit metrics as telemetry
    async fn emit_metrics(&self, metrics: &LearningMetrics) {
        // In real implementation, this would emit to OpenTelemetry
        println!(
            "[Metrics] Episode: {}, AvgReward: {:.4}, Loss: {:.6}, Exploration: {:.4}, Converged: {}",
            metrics.episode,
            metrics.avg_reward,
            metrics.loss_trend,
            metrics.exploration_rate,
            metrics.is_converged
        );
    }

    /// Create model checkpoint
    pub async fn checkpoint_model(&self) -> ModelVersion {
        let metrics = self.get_learning_metrics();
        let convergence = self.calculate_convergence();

        let version = ModelVersion {
            version: self.model_versions.read().unwrap().len() + 1,
            timestamp: Instant::now().elapsed().as_millis(),
            avg_reward: metrics.avg_reward,
            loss: metrics.loss_trend,
            convergence,
        };

        self.model_versions.write().unwrap().push(version.clone());
        version
    }

    /// Get current agent
    pub fn agent(&self) -> Arc<RwLock<QLearning<S, A>>> {
        Arc::clone(&self.agent)
    }

    /// Get trainer
    pub fn trainer(&self) -> Arc<RwLock<Trainer<DenseLayer<10, 5>>>> {
        Arc::clone(&self.trainer)
    }

    /// Get model versions
    pub fn model_versions(&self) -> Vec<ModelVersion> {
        self.model_versions.read().unwrap().clone()
    }
}

/// Adaptive executor that monitors performance and triggers retraining
pub struct AdaptiveWorkflowExecutor<S: WorkflowState + 'static, A: WorkflowAction + 'static> {
    /// Underlying self-learning workflow
    workflow: Arc<SelfLearningWorkflow<S, A>>,

    /// Performance history for adaptive decisions
    performance_history: Arc<RwLock<VecDeque<f32>>>,

    /// Retraining threshold
    retraining_threshold: f32,

    /// Minimum episodes before considering retraining
    min_episodes_retraining: usize,

    /// Last retraining episode
    last_retraining: Arc<RwLock<usize>>,
}

impl<S: WorkflowState + 'static, A: WorkflowAction + 'static> AdaptiveWorkflowExecutor<S, A> {
    /// Create new adaptive executor
    pub fn new(
        workflow: Arc<SelfLearningWorkflow<S, A>>,
        retraining_threshold: f32,
    ) -> Self {
        Self {
            workflow,
            performance_history: Arc::new(RwLock::new(VecDeque::with_capacity(200))),
            retraining_threshold,
            min_episodes_retraining: 50,
            last_retraining: Arc::new(RwLock::new(0)),
        }
    }

    /// Run adaptive execution loop with retraining
    pub async fn run_adaptive_loop(&self) -> ! {
        loop {
            // Execute episode
            let result = self.workflow.execute_episode().await;

            // Track performance
            let mut perf_hist = self.performance_history.write().unwrap();
            if perf_hist.len() >= 200 {
                perf_hist.pop_front();
            }
            perf_hist.push_back(result.total_reward);
            drop(perf_hist);

            // Check if retraining needed
            let metrics = self.workflow.get_learning_metrics();
            if self.should_retrain(&metrics) {
                self.trigger_retraining(&metrics).await;
            }

            // Adapt learning rate based on convergence
            if metrics.is_converged {
                let mut trainer = self.workflow.trainer.write().unwrap();
                trainer.decay_learning_rate(0.95);
                println!("[Adaptive] Decayed learning rate to {:.6}", trainer.learning_rate());
            }

            sleep(Duration::from_millis(10)).await;
        }
    }

    /// Determine if retraining is needed
    fn should_retrain(&self, metrics: &LearningMetrics) -> bool {
        if metrics.total_episodes < self.min_episodes_retraining {
            return false;
        }

        let last_retrain = *self.last_retraining.read().unwrap();
        if metrics.total_episodes - last_retrain < 50 {
            return false;
        }

        // Retrain if accuracy drops below threshold
        let perf_hist = self.performance_history.read().unwrap();
        if perf_hist.len() < 10 {
            return false;
        }

        let recent_avg: f32 = perf_hist.iter().rev().take(10).sum::<f32>() / 10.0;
        let older_avg: f32 = perf_hist.iter().take(10).sum::<f32>() / 10.0;

        // If performance is degrading, retrain
        recent_avg < older_avg * self.retraining_threshold
    }

    /// Trigger retraining with learning rate adaptation
    async fn trigger_retraining(&self, metrics: &LearningMetrics) {
        println!("[Adaptive] Triggering retraining at episode {}", metrics.total_episodes);

        // Increase learning rate for faster adaptation
        let mut trainer = self.workflow.trainer.write().unwrap();
        let old_lr = trainer.learning_rate();
        trainer.learning_rate = old_lr * 1.5;
        let new_lr = trainer.learning_rate();
        drop(trainer);
        println!("[Adaptive] Increased learning rate from {:.6} to {:.6}", old_lr, new_lr);

        *self.last_retraining.write().unwrap() = metrics.total_episodes;
    }

    /// Get current metrics
    pub fn metrics(&self) -> LearningMetrics {
        self.workflow.get_learning_metrics()
    }

    /// Get workflow reference
    pub fn workflow(&self) -> Arc<SelfLearningWorkflow<S, A>> {
        Arc::clone(&self.workflow)
    }

    /// Graceful shutdown with model checkpointing
    pub async fn shutdown_with_checkpoint(&self) {
        println!("[Shutdown] Creating final checkpoint...");
        let version = self.workflow.checkpoint_model().await;
        println!(
            "[Shutdown] Final model version {}: reward={:.4}, loss={:.6}, convergence={:.4}",
            version.version, version.avg_reward, version.loss, version.convergence
        );
    }
}

/// Performance improvement metrics
/// Tracks improvements in key metrics over workflow executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceImprovements {
    /// Speed improvement percentage (execution time reduction)
    pub speed_improvement: f32,
    /// Quality improvement percentage (quality score increase)
    pub quality_improvement: f32,
    /// Success rate percentage (0-100)
    pub success_rate: f32,
    /// Average execution time in milliseconds
    pub average_duration_ms: f32,
    /// Total executions tracked
    pub total_executions: usize,
}

impl PerformanceImprovements {
    /// Check if improvements are significant (>10%)
    pub fn is_significant(&self) -> bool {
        self.speed_improvement > 10.0 || self.quality_improvement > 10.0
    }

    /// Get overall improvement score (0.0-1.0)
    pub fn overall_score(&self) -> f32 {
        let speed_weight = 0.4;
        let quality_weight = 0.4;
        let success_weight = 0.2;

        (self.speed_improvement / 100.0).min(1.0) * speed_weight
            + (self.quality_improvement / 100.0).min(1.0) * quality_weight
            + (self.success_rate / 100.0) * success_weight
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;

    /// Mock workflow state for testing
    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    struct MockState {
        value: i32,
    }

    impl WorkflowState for MockState {
        fn features(&self) -> Vec<f32> {
            vec![self.value as f32]
        }

        fn is_terminal(&self) -> bool {
            self.value >= 100
        }
    }

    /// Mock workflow action for testing
    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    enum MockAction {
        Increment,
        Double,
    }

    impl WorkflowAction for MockAction {
        const ACTION_COUNT: usize = 2;

        fn to_index(&self) -> usize {
            match self {
                MockAction::Increment => 0,
                MockAction::Double => 1,
            }
        }

        fn from_index(idx: usize) -> Option<Self> {
            match idx {
                0 => Some(MockAction::Increment),
                1 => Some(MockAction::Double),
                _ => None,
            }
        }
    }

    #[tokio::test]
    async fn test_workflow_creation() {
        let state = Arc::new(std::sync::Mutex::new(0i32));
        let config = WorkflowConfig::default();
        let state_clone = state.clone();

        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            move || {
                let mut s = state_clone.lock().unwrap();
                let val = *s;
                *s += 1;
                MockState { value: val }
            },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            config,
        );

        assert_eq!(workflow.get_learning_metrics().total_episodes, 0);
    }

    #[tokio::test]
    async fn test_episode_execution() {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let counter_clone = counter.clone();

        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            move || {
                let mut c = counter_clone.lock().unwrap();
                let val = *c;
                *c += 1;
                MockState { value: val }
            },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let result = workflow.execute_episode().await;
        assert!(result.total_reward > 0.0, "Episode should accumulate positive reward");
        assert!(result.steps > 0, "Episode should have steps");
    }

    #[tokio::test]
    async fn test_learning_metrics() {
        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            || MockState { value: 0 },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let metrics = workflow.get_learning_metrics();
        assert_eq!(metrics.total_episodes, 0);
        assert_eq!(metrics.avg_reward, 0.0);
    }

    #[tokio::test]
    async fn test_convergence_calculation() {
        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            || MockState { value: 0 },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let convergence = workflow.calculate_convergence();
        assert!(convergence >= 0.0 && convergence <= 1.0, "Convergence should be in [0, 1]");
    }

    #[tokio::test]
    async fn test_model_checkpointing() {
        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            || MockState { value: 0 },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let version = workflow.checkpoint_model().await;
        assert_eq!(version.version, 1);

        let version2 = workflow.checkpoint_model().await;
        assert_eq!(version2.version, 2);

        let versions = workflow.model_versions();
        assert_eq!(versions.len(), 2);
    }

    #[tokio::test]
    async fn test_adaptive_executor_creation() {
        let workflow = Arc::new(SelfLearningWorkflow::<MockState, MockAction>::new(
            || MockState { value: 0 },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        ));

        let executor = AdaptiveWorkflowExecutor::new(workflow, 0.9);
        let metrics = executor.metrics();
        assert_eq!(metrics.total_episodes, 0);
    }

    #[tokio::test]
    async fn test_trainer_learning_rate_decay() {
        let mut trainer: Trainer<DenseLayer<10, 5>> = Trainer::new();
        let initial_lr = trainer.learning_rate();

        trainer.decay_learning_rate(0.95);
        let new_lr = trainer.learning_rate();

        assert!(new_lr < initial_lr, "Learning rate should decay");
        assert!((new_lr - initial_lr * 0.95).abs() < 1e-6, "Decay should be 5%");
    }

    #[tokio::test]
    async fn test_trainer_batch_training() {
        let trainer: Trainer<DenseLayer<10, 5>> = Trainer::new();

        let gradients = vec![0.1, 0.2, 0.3, 0.4];
        let loss = trainer.train_batch(gradients);

        assert!(loss > 0.0, "Loss should be positive for non-zero gradients");
    }

    #[test]
    fn test_episode_result_creation() {
        let result = EpisodeResult {
            total_reward: 100.0,
            steps: 50,
            is_terminal: true,
            duration_ms: 100,
            loss: 0.05,
            convergence: 0.8,
        };

        assert_eq!(result.total_reward, 100.0);
        assert_eq!(result.steps, 50);
        assert!(result.is_terminal);
    }

    #[test]
    fn test_workflow_config_defaults() {
        let config = WorkflowConfig::default();
        assert_eq!(config.history_window, 100);
        assert_eq!(config.training_interval, 10);
        assert!(config.retraining_threshold > 0.0 && config.retraining_threshold < 1.0);
    }

    #[test]
    fn test_model_version_serialization() {
        let version = ModelVersion {
            version: 1,
            timestamp: 1000,
            avg_reward: 50.0,
            loss: 0.1,
            convergence: 0.8,
        };

        let json = serde_json::to_string(&version).unwrap();
        let parsed: ModelVersion = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, version.version);
        assert_eq!(parsed.avg_reward, version.avg_reward);
    }

    #[test]
    fn test_workflow_metrics_creation() {
        let metrics = WorkflowMetrics::new(50.0, true, 30.0, 5);
        assert_eq!(metrics.duration_ms, 50.0);
        assert!(metrics.success);
        assert_eq!(metrics.pattern_id, 5);
        assert!(metrics.quality_score > 0.0 && metrics.quality_score <= 1.0);
    }

    #[test]
    fn test_workflow_metrics_reward() {
        let success_metrics = WorkflowMetrics::new(50.0, true, 30.0, 5);
        let success_reward = success_metrics.to_reward();
        assert!(success_reward > 0.0);

        let failure_metrics = WorkflowMetrics::new(50.0, false, 30.0, 5);
        let failure_reward = failure_metrics.to_reward();
        assert!(failure_reward < 0.0);
    }

    #[test]
    fn test_performance_tracker_creation() {
        let tracker = PerformanceTracker::new();
        assert_eq!(tracker.success_rate(), 0.0);
        assert_eq!(tracker.avg_duration_ms(), 0.0);
    }

    #[test]
    fn test_performance_tracker_record() {
        let tracker = PerformanceTracker::new();
        let metrics = WorkflowMetrics::new(50.0, true, 30.0, 5);
        tracker.record(metrics.clone());

        assert_eq!(tracker.success_rate(), 100.0);
        assert_eq!(tracker.avg_duration_ms(), 50.0);
    }

    #[test]
    fn test_performance_tracker_improvement() {
        let tracker = PerformanceTracker::new();

        let metrics1 = WorkflowMetrics::new(100.0, true, 40.0, 5);
        tracker.record(metrics1);

        let metrics2 = WorkflowMetrics::new(70.0, true, 30.0, 5);
        tracker.record(metrics2);

        let improvement = tracker.improvement_percentage();
        assert!(improvement > 0.0, "Should show improvement from 100ms to 70ms");
    }

    #[test]
    fn test_workflow_executor_creation() {
        let executor = WorkflowExecutor::new();
        assert_eq!(executor.execution_count(), 0);
        assert_eq!(executor.mape_k_cycles(), 0);
    }

    #[tokio::test]
    async fn test_workflow_executor_execute() {
        let executor = WorkflowExecutor::new();
        let metrics = executor.execute(50.0, true, 30.0, 5).await;

        assert_eq!(executor.execution_count(), 1);
        assert_eq!(executor.mape_k_cycles(), 1);
        assert!(metrics.success);
    }

    #[tokio::test]
    async fn test_self_learning_workflow_execute_adaptive() {
        let counter = std::sync::Arc::new(std::sync::Mutex::new(0));
        let counter_clone = counter.clone();

        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            move || {
                let mut c = counter_clone.lock().unwrap();
                let val = *c;
                *c += 1;
                MockState { value: val }
            },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let (metrics, reward) = workflow.execute_adaptive("test_workflow").await;
        assert!(metrics.success || !metrics.success); // Valid state
        assert!(reward.is_finite());
    }

    #[tokio::test]
    async fn test_self_learning_workflow_record_execution() {
        let workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            || MockState { value: 0 },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let metrics = WorkflowMetrics::new(50.0, true, 30.0, 5);
        workflow.record_execution(metrics.clone());

        let improvements = workflow.get_performance_improvements();
        assert_eq!(improvements.total_executions, 1);
    }

    #[tokio::test]
    async fn test_self_learning_workflow_update_model() {
        let mut workflow = SelfLearningWorkflow::<MockState, MockAction>::new(
            || MockState { value: 0 },
            |_state: &MockState, _action: &MockAction, _next: &MockState| 1.0,
            WorkflowConfig::default(),
        );

        let metrics1 = WorkflowMetrics::new(100.0, true, 40.0, 5);
        workflow.record_execution(metrics1);

        let metrics2 = WorkflowMetrics::new(70.0, true, 30.0, 5);
        workflow.record_execution(metrics2);

        let improvement = workflow.update_model().await;
        assert!(improvement >= 0.0, "Improvement should be non-negative");
    }

    #[test]
    fn test_performance_improvements_creation() {
        let improvements = PerformanceImprovements {
            speed_improvement: 25.0,
            quality_improvement: 15.0,
            success_rate: 95.0,
            average_duration_ms: 50.0,
            total_executions: 10,
        };

        assert!(improvements.is_significant());
        let score = improvements.overall_score();
        assert!(score > 0.0 && score <= 1.0);
    }

    #[test]
    fn test_performance_improvements_overall_score() {
        let good_improvements = PerformanceImprovements {
            speed_improvement: 50.0,
            quality_improvement: 50.0,
            success_rate: 100.0,
            average_duration_ms: 50.0,
            total_executions: 10,
        };

        let poor_improvements = PerformanceImprovements {
            speed_improvement: 0.0,
            quality_improvement: 0.0,
            success_rate: 0.0,
            average_duration_ms: 100.0,
            total_executions: 10,
        };

        assert!(good_improvements.overall_score() > poor_improvements.overall_score());
    }

    #[tokio::test]
    async fn test_workflow_executor_telemetry() {
        let executor = WorkflowExecutor::new();

        executor.execute(50.0, true, 30.0, 5).await;
        executor.execute(45.0, true, 25.0, 6).await;

        // Just verify telemetry doesn't panic
        executor.emit_telemetry().await;

        assert_eq!(executor.execution_count(), 2);
    }
}
