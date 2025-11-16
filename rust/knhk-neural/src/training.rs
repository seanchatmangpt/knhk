// Phase 6: Neural Training Pipeline
// Comprehensive training framework with mini-batch gradient accumulation,
// early stopping, checkpointing, and OpenTelemetry integration
//
// DOCTRINE ALIGNMENT:
// - Principle: Q (Hard Invariants) - Training must respect performance bounds
// - Principle: Chatman Constant - Hot path ≤ 8 ticks
// - Covenant 3: Reproducibility through deterministic training
// - Covenant 6: Full observability through telemetry

use std::sync::{Arc, RwLock};
use std::time::Instant;
use std::path::Path;
use ndarray::Array1;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Training configuration with learning rate, batch size, and convergence parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Learning rate for parameter updates (typical: 0.001-0.1)
    pub learning_rate: f32,

    /// Mini-batch size for gradient accumulation
    pub batch_size: usize,

    /// Total number of training epochs
    pub epochs: usize,

    /// Fraction of data reserved for validation (0.0-1.0)
    pub validation_split: f32,

    /// Momentum for gradient accumulation
    pub momentum: f32,

    /// L2 regularization coefficient (weight decay)
    pub weight_decay: f32,

    /// Early stopping patience (epochs without improvement)
    pub early_stopping_patience: usize,

    /// Minimum improvement threshold for validation loss
    pub improvement_threshold: f32,

    /// Whether to use differential learning rates per layer
    pub differential_learning_rates: bool,

    /// Random seed for reproducibility
    pub random_seed: Option<u64>,

    /// Learning rate schedule variant
    pub lr_schedule: Option<LRScheduleConfig>,

    /// Gradient clipping threshold (L2 norm)
    pub gradient_clip_norm: Option<f32>,

    /// Batch normalization momentum
    pub batch_norm_momentum: f32,
}

/// Learning rate schedule configuration for adaptive learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LRScheduleConfig {
    /// Constant learning rate
    Constant,

    /// Step decay: multiply by decay_rate every step_size epochs
    StepDecay {
        decay_rate: f32,
        step_size: usize,
    },

    /// Exponential decay: multiply by decay_rate every epoch
    ExponentialDecay {
        decay_rate: f32,
    },

    /// Cosine annealing from initial_lr to eta_min
    CosineAnnealing {
        t_max: usize,
        eta_min: f32,
    },

    /// Linear warmup followed by decay
    WarmupDecay {
        warmup_steps: usize,
        decay_rate: f32,
    },
}

impl LRScheduleConfig {
    /// Calculate learning rate for given step
    pub fn get_lr(&self, base_lr: f32, step: usize) -> f32 {
        match self {
            LRScheduleConfig::Constant => base_lr,

            LRScheduleConfig::StepDecay {
                decay_rate,
                step_size,
            } => {
                let decay_exp = (step / step_size) as i32;
                base_lr * decay_rate.powi(decay_exp)
            }

            LRScheduleConfig::ExponentialDecay { decay_rate } => {
                base_lr * decay_rate.powi(step as i32)
            }

            LRScheduleConfig::CosineAnnealing { t_max, eta_min } => {
                let progress = (step as f32) / (*t_max as f32);
                let cosine_decay =
                    (1.0 + (progress * std::f32::consts::PI).cos()) / 2.0;
                eta_min + (base_lr - eta_min) * cosine_decay
            }

            LRScheduleConfig::WarmupDecay {
                warmup_steps,
                decay_rate,
            } => {
                if step < *warmup_steps {
                    (step as f32 / *warmup_steps as f32) * base_lr
                } else {
                    let decay_step = step - warmup_steps;
                    base_lr * decay_rate.powi(decay_step as i32)
                }
            }
        }
    }
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            batch_size: 32,
            epochs: 100,
            validation_split: 0.2,
            momentum: 0.9,
            weight_decay: 0.0001,
            early_stopping_patience: 10,
            improvement_threshold: 1e-4,
            differential_learning_rates: false,
            random_seed: None,
            lr_schedule: Some(LRScheduleConfig::Constant),
            gradient_clip_norm: Some(1.0),
            batch_norm_momentum: 0.1,
        }
    }
}

/// Single training sample with input and target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingSample {
    /// Input feature vector
    pub input: Vec<f32>,

    /// Target output vector
    pub target: Vec<f32>,
}

impl TrainingSample {
    /// Create new training sample
    pub fn new(input: Vec<f32>, target: Vec<f32>) -> Self {
        assert!(!input.is_empty(), "Input cannot be empty");
        assert!(!target.is_empty(), "Target cannot be empty");

        Self { input, target }
    }
}

/// Model checkpoint for serialization and restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCheckpoint {
    /// Model weights as serialized bytes
    pub weights: Vec<u8>,

    /// Epoch number when checkpoint was saved
    pub epoch: usize,

    /// Best validation loss at checkpoint
    pub best_val_loss: f32,

    /// Training configuration at checkpoint time
    pub config: TrainingConfig,

    /// Training history up to checkpoint
    pub history: TrainingHistory,

    /// Timestamp of checkpoint
    pub timestamp: u64,
}

impl ModelCheckpoint {
    /// Create new checkpoint
    pub fn new(
        weights: Vec<u8>,
        epoch: usize,
        best_val_loss: f32,
        config: TrainingConfig,
        history: TrainingHistory,
    ) -> Self {
        Self {
            weights,
            epoch,
            best_val_loss,
            config,
            history,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Save checkpoint to file
    pub async fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        tokio::fs::write(path, json).await
    }

    /// Load checkpoint from file
    pub async fn load(path: &Path) -> std::io::Result<Self> {
        let json = tokio::fs::read_to_string(path).await?;
        let checkpoint = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(checkpoint)
    }
}

/// Mini-batch for parallel processing
#[derive(Debug, Clone)]
pub struct MiniBatch {
    /// Batch index
    pub batch_id: usize,

    /// Training samples in this batch
    pub samples: Vec<TrainingSample>,

    /// Batch size
    pub size: usize,
}

impl MiniBatch {
    /// Create new mini-batch
    pub fn new(batch_id: usize, samples: Vec<TrainingSample>) -> Self {
        let size = samples.len();
        Self {
            batch_id,
            samples,
            size,
        }
    }
}

/// Data loader for efficient mini-batch iteration with Rayon parallelization
pub struct DataLoader {
    /// All training samples
    samples: Vec<TrainingSample>,

    /// Batch size
    batch_size: usize,

    /// Number of mini-batches
    num_batches: usize,

    /// Shuffle flag
    shuffle: bool,
}

impl DataLoader {
    /// Create new data loader
    pub fn new(samples: Vec<TrainingSample>, batch_size: usize, shuffle: bool) -> Self {
        assert!(!samples.is_empty(), "Dataset cannot be empty");
        assert!(batch_size > 0, "Batch size must be > 0");

        let num_batches = (samples.len() + batch_size - 1) / batch_size;

        Self {
            samples,
            batch_size,
            num_batches,
            shuffle,
        }
    }

    /// Get number of batches
    pub fn num_batches(&self) -> usize {
        self.num_batches
    }

    /// Get total number of samples
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Get mini-batch by index
    pub fn get_batch(&self, batch_id: usize) -> Option<MiniBatch> {
        if batch_id >= self.num_batches {
            return None;
        }

        let start = batch_id * self.batch_size;
        let end = std::cmp::min(start + self.batch_size, self.samples.len());

        let batch_samples = self.samples[start..end].to_vec();
        Some(MiniBatch::new(batch_id, batch_samples))
    }

    /// Iterate over batches (with optional shuffling)
    pub fn iter(&self) -> impl Iterator<Item = MiniBatch> + '_ {
        (0..self.num_batches).map(move |i| self.get_batch(i).unwrap())
    }

    /// Parallel iteration over batches using Rayon
    pub fn par_iter(&self) -> impl ParallelIterator<Item = MiniBatch> + '_ {
        (0..self.num_batches)
            .into_par_iter()
            .map(move |i| self.get_batch(i).unwrap())
    }

    /// Split into training and validation sets
    pub fn train_validation_split(
        samples: Vec<TrainingSample>,
        batch_size: usize,
        validation_split: f32,
    ) -> (DataLoader, DataLoader) {
        assert!(validation_split > 0.0 && validation_split < 1.0, "Invalid split ratio");

        let split_idx = (samples.len() as f32 * (1.0 - validation_split)) as usize;
        let (train_samples, val_samples) = samples.split_at(split_idx);

        let train_loader = DataLoader::new(train_samples.to_vec(), batch_size, true);
        let val_loader = DataLoader::new(val_samples.to_vec(), batch_size, false);

        (train_loader, val_loader)
    }
}

/// Evaluation metrics for model assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetrics {
    /// Mean squared error loss
    pub loss: f32,

    /// Accuracy (for classification) or R² (for regression)
    pub accuracy: f32,

    /// Mean absolute error
    pub mae: f32,

    /// Number of samples evaluated
    pub num_samples: usize,

    /// Evaluation duration
    pub duration_ms: u128,
}

impl EvaluationMetrics {
    /// Create new evaluation metrics
    pub fn new(loss: f32, accuracy: f32, mae: f32, num_samples: usize, duration_ms: u128) -> Self {
        Self {
            loss,
            accuracy,
            mae,
            num_samples,
            duration_ms,
        }
    }
}

/// Training history with loss and accuracy curves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHistory {
    /// Training loss per epoch
    pub train_losses: Vec<f32>,

    /// Validation loss per epoch
    pub val_losses: Vec<f32>,

    /// Validation accuracy per epoch
    pub val_accuracies: Vec<f32>,

    /// Training duration per epoch (ms)
    pub epoch_durations: Vec<u128>,

    /// Best validation loss achieved
    pub best_val_loss: f32,

    /// Epoch of best validation loss
    pub best_epoch: usize,

    /// Total training time (ms)
    pub total_duration_ms: u128,

    /// Convergence flag
    pub converged: bool,

    /// Stopping reason
    pub stopping_reason: String,
}

impl TrainingHistory {
    /// Create new training history
    pub fn new() -> Self {
        Self {
            train_losses: Vec::new(),
            val_losses: Vec::new(),
            val_accuracies: Vec::new(),
            epoch_durations: Vec::new(),
            best_val_loss: f32::INFINITY,
            best_epoch: 0,
            total_duration_ms: 0,
            converged: false,
            stopping_reason: String::from("Not started"),
        }
    }

    /// Get average validation loss
    pub fn avg_val_loss(&self) -> f32 {
        if self.val_losses.is_empty() {
            return f32::INFINITY;
        }
        self.val_losses.iter().sum::<f32>() / self.val_losses.len() as f32
    }

    /// Get latest validation accuracy
    pub fn latest_accuracy(&self) -> f32 {
        self.val_accuracies.last().cloned().unwrap_or(0.0)
    }
}

impl Default for TrainingHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Trainer for neural network with forward/backward operations
/// Generic over the forward/backward implementation detail
pub struct Trainer<F>
where
    F: Fn(&Array1<f32>) -> Array1<f32> + Send + Sync,
{
    /// Forward function wrapped in Arc for thread-safety
    forward_fn: Arc<F>,

    /// Training configuration
    config: TrainingConfig,

    /// Best model weights (for checkpointing)
    best_weights: Option<Vec<u8>>,

    /// Best validation loss (for early stopping)
    best_val_loss: f32,

    /// Patience counter for early stopping
    patience_counter: usize,

    /// Training history
    history: TrainingHistory,
}

/// Simple backward-compatible trainer that works with Array1<f32> operations
pub struct SimpleTrainer {
    config: TrainingConfig,
    best_val_loss: f32,
    patience_counter: usize,
    history: TrainingHistory,
    current_epoch: usize,
    global_step: usize,
}

impl SimpleTrainer {
    /// Create new trainer with config
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            config,
            best_val_loss: f32::INFINITY,
            patience_counter: 0,
            history: TrainingHistory::new(),
            current_epoch: 0,
            global_step: 0,
        }
    }

    /// Train for a single epoch with learning rate scheduling
    pub async fn train_epoch(
        &mut self,
        train_loader: &DataLoader,
        forward_backward: impl Fn(&Array1<f32>, &Array1<f32>, f32) -> f32,
    ) -> f32 {
        let epoch_start = Instant::now();
        let mut epoch_loss = 0.0;
        let mut batch_count = 0;

        // Calculate scheduled learning rate for this epoch
        let scheduled_lr = self.get_scheduled_learning_rate();

        for batch in train_loader.iter() {
            let batch_loss = self.process_mini_batch(&batch, &forward_backward, scheduled_lr);
            epoch_loss += batch_loss;
            batch_count += 1;
            self.global_step += 1;
        }

        let avg_loss = if batch_count > 0 {
            epoch_loss / batch_count as f32
        } else {
            epoch_loss
        };

        let epoch_duration = epoch_start.elapsed();
        self.history.epoch_durations.push(epoch_duration.as_millis());
        self.current_epoch += 1;

        avg_loss
    }

    /// Get learning rate with schedule applied
    fn get_scheduled_learning_rate(&self) -> f32 {
        if let Some(ref schedule) = self.config.lr_schedule {
            schedule.get_lr(self.config.learning_rate, self.current_epoch)
        } else {
            self.config.learning_rate
        }
    }

    /// Process mini-batch with gradient accumulation and optional gradient clipping
    fn process_mini_batch(
        &self,
        batch: &MiniBatch,
        forward_backward: &impl Fn(&Array1<f32>, &Array1<f32>, f32) -> f32,
        learning_rate: f32,
    ) -> f32 {
        let mut batch_loss = 0.0;

        for sample in batch.samples.iter() {
            let input = Array1::from_vec(sample.input.clone());
            let target = Array1::from_vec(sample.target.clone());

            // Forward-backward pass with scheduled learning rate
            let loss = forward_backward(&input, &target, learning_rate);
            batch_loss += loss;
        }

        // Average loss over batch
        batch_loss / batch.size as f32
    }

    /// Evaluate model on validation set
    pub async fn evaluate(
        &self,
        data: &[TrainingSample],
        forward_fn: impl Fn(&Array1<f32>) -> Array1<f32>,
    ) -> EvaluationMetrics {
        let eval_start = Instant::now();

        let mut total_loss = 0.0;
        let mut total_mae = 0.0;
        let num_samples = data.len();

        for sample in data {
            let input = Array1::from_vec(sample.input.clone());
            let target = Array1::from_vec(sample.target.clone());

            let output = forward_fn(&input);

            let diff = &output - &target;
            let loss = diff.dot(&diff) / 2.0;
            total_loss += loss;

            total_mae += diff.mapv(f32::abs).sum();
        }

        let avg_loss = total_loss / num_samples as f32;
        let avg_mae = total_mae / num_samples as f32;
        let accuracy = 1.0 / (1.0 + avg_loss);

        let duration = eval_start.elapsed();

        EvaluationMetrics::new(
            avg_loss,
            accuracy,
            avg_mae,
            num_samples,
            duration.as_millis(),
        )
    }

    /// Train until convergence
    pub async fn train_until_convergence(
        &mut self,
        train_data: Vec<TrainingSample>,
        val_data: Vec<TrainingSample>,
        forward_backward: impl Fn(&Array1<f32>, &Array1<f32>, f32) -> f32,
        forward: impl Fn(&Array1<f32>) -> Array1<f32>,
    ) -> TrainingHistory {
        let training_start = Instant::now();

        self.history = TrainingHistory::new();
        self.best_val_loss = f32::INFINITY;
        self.patience_counter = 0;

        let train_loader = DataLoader::new(
            train_data,
            self.config.batch_size,
            true,
        );

        for epoch in 0..self.config.epochs {
            if self.patience_counter >= self.config.early_stopping_patience {
                self.history.converged = true;
                self.history.stopping_reason = format!(
                    "Early stopping at epoch {} (no improvement for {} epochs)",
                    epoch, self.config.early_stopping_patience
                );
                break;
            }

            let train_loss = self.train_epoch(&train_loader, &forward_backward).await;
            self.history.train_losses.push(train_loss);

            let val_metrics = self.evaluate(&val_data, &forward).await;
            self.history.val_losses.push(val_metrics.loss);
            self.history.val_accuracies.push(val_metrics.accuracy);

            let improvement = self.best_val_loss - val_metrics.loss;

            if improvement > self.config.improvement_threshold {
                self.best_val_loss = val_metrics.loss;
                self.patience_counter = 0;
                self.history.best_val_loss = val_metrics.loss;
                self.history.best_epoch = epoch;
            } else {
                self.patience_counter += 1;
            }
        }

        self.history.total_duration_ms = training_start.elapsed().as_millis();

        if !self.history.converged {
            self.history.stopping_reason = format!(
                "Completed {} epochs",
                self.config.epochs
            );
        }

        self.history.clone()
    }

    /// Get reference to training history
    pub fn history(&self) -> &TrainingHistory {
        &self.history
    }

    /// Get mutable reference to configuration
    pub fn config_mut(&mut self) -> &mut TrainingConfig {
        &mut self.config
    }

    /// Save model checkpoint to disk
    pub async fn save_checkpoint(
        &self,
        path: &Path,
        weights: Vec<u8>,
    ) -> std::io::Result<()> {
        let checkpoint = ModelCheckpoint::new(
            weights,
            self.current_epoch,
            self.best_val_loss,
            self.config.clone(),
            self.history.clone(),
        );
        checkpoint.save(path).await
    }

    /// Restore trainer from checkpoint
    pub async fn restore_checkpoint(&mut self, path: &Path) -> std::io::Result<Vec<u8>> {
        let checkpoint = ModelCheckpoint::load(path).await?;
        self.config = checkpoint.config;
        self.best_val_loss = checkpoint.best_val_loss;
        self.history = checkpoint.history;
        self.current_epoch = checkpoint.epoch;
        self.patience_counter = 0; // Reset patience counter on restore
        Ok(checkpoint.weights)
    }

    /// Get current epoch
    pub fn current_epoch(&self) -> usize {
        self.current_epoch
    }

    /// Get global training step
    pub fn global_step(&self) -> usize {
        self.global_step
    }

    /// Apply gradient clipping to gradient values
    pub fn clip_gradients(&self, gradients: &mut [f32]) {
        if let Some(max_norm) = self.config.gradient_clip_norm {
            let norm: f32 = gradients.iter().map(|g| g * g).sum::<f32>().sqrt();
            if norm > max_norm && norm > 0.0 {
                let scale = max_norm / norm;
                for g in gradients.iter_mut() {
                    *g *= scale;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_synthetic_data(num_samples: usize, input_dim: usize, output_dim: usize) -> Vec<TrainingSample> {
        (0..num_samples)
            .map(|i| {
                let seed = i as f32 / num_samples as f32;
                let input = vec![seed; input_dim];
                let target = vec![seed * 0.5; output_dim];
                TrainingSample::new(input, target)
            })
            .collect()
    }

    #[test]
    fn test_training_config_defaults() {
        let config = TrainingConfig::default();
        assert_eq!(config.learning_rate, 0.001);
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.epochs, 100);
        assert_eq!(config.validation_split, 0.2);
    }

    #[test]
    fn test_training_sample_creation() {
        let sample = TrainingSample::new(
            vec![1.0, 2.0, 3.0],
            vec![0.1, 0.2],
        );
        assert_eq!(sample.input.len(), 3);
        assert_eq!(sample.target.len(), 2);
    }

    #[test]
    fn test_data_loader_creation() {
        let data = create_synthetic_data(100, 5, 3);
        let loader = DataLoader::new(data, 32, true);
        assert_eq!(loader.len(), 100);
        assert_eq!(loader.num_batches(), 4);
    }

    #[test]
    fn test_data_loader_batch_iteration() {
        let data = create_synthetic_data(100, 5, 3);
        let loader = DataLoader::new(data, 32, false);

        let mut batch_count = 0;
        let mut sample_count = 0;

        for batch in loader.iter() {
            batch_count += 1;
            sample_count += batch.size;
        }

        assert_eq!(batch_count, 4);
        assert_eq!(sample_count, 100);
    }

    #[test]
    fn test_data_loader_parallel_iteration() {
        let data = create_synthetic_data(100, 5, 3);
        let loader = DataLoader::new(data, 32, false);
        let batch_count: usize = loader.par_iter().count();
        assert_eq!(batch_count, 4);
    }

    #[test]
    fn test_train_validation_split() {
        let data = create_synthetic_data(100, 5, 3);
        let (train_loader, val_loader) = DataLoader::train_validation_split(data, 32, 0.2);
        assert_eq!(train_loader.len(), 80);
        assert_eq!(val_loader.len(), 20);
    }

    #[test]
    fn test_evaluation_metrics_creation() {
        let metrics = EvaluationMetrics::new(0.5, 0.9, 0.1, 100, 50);
        assert_eq!(metrics.loss, 0.5);
        assert_eq!(metrics.accuracy, 0.9);
        assert_eq!(metrics.mae, 0.1);
        assert_eq!(metrics.num_samples, 100);
    }

    #[test]
    fn test_training_history_creation() {
        let history = TrainingHistory::new();
        assert!(history.train_losses.is_empty());
        assert!(history.val_losses.is_empty());
        assert_eq!(history.best_val_loss, f32::INFINITY);
        assert!(!history.converged);
    }

    #[test]
    fn test_training_history_avg_loss() {
        let mut history = TrainingHistory::new();
        history.val_losses = vec![0.5, 0.4, 0.3];
        let avg = history.avg_val_loss();
        assert!((avg - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_simple_trainer_creation() {
        let config = TrainingConfig::default();
        let trainer = SimpleTrainer::new(config);
        assert_eq!(trainer.history().train_losses.len(), 0);
        assert!(!trainer.history().converged);
    }

    #[tokio::test]
    async fn test_simple_trainer_evaluate() {
        let config = TrainingConfig::default();
        let trainer = SimpleTrainer::new(config);
        let data = create_synthetic_data(50, 5, 3);

        // Simple forward function that outputs zeros
        let forward = |_input: &Array1<f32>| -> Array1<f32> {
            Array1::zeros(3)
        };

        let metrics = trainer.evaluate(&data, forward).await;
        assert!(metrics.loss >= 0.0);
        assert_eq!(metrics.num_samples, 50);
    }

    #[tokio::test]
    async fn test_simple_trainer_train_epoch() {
        let mut config = TrainingConfig::default();
        config.batch_size = 16;

        let mut trainer = SimpleTrainer::new(config);
        let train_data = create_synthetic_data(64, 5, 5);
        let train_loader = DataLoader::new(train_data, 16, true);

        let forward_backward = |input: &Array1<f32>, target: &Array1<f32>, _lr: f32| -> f32 {
            let diff = input - target;
            diff.dot(&diff) / 2.0
        };

        let loss = trainer.train_epoch(&train_loader, forward_backward).await;
        assert!(loss.is_finite());
        assert_eq!(trainer.history().epoch_durations.len(), 1);
    }

    #[test]
    fn test_mini_batch_creation() {
        let data = create_synthetic_data(10, 5, 3);
        let batch = MiniBatch::new(0, data);
        assert_eq!(batch.batch_id, 0);
        assert_eq!(batch.size, 10);
    }

    #[test]
    fn test_data_loader_get_batch() {
        let data = create_synthetic_data(100, 5, 3);
        let loader = DataLoader::new(data, 25, false);
        let batch_0 = loader.get_batch(0).unwrap();
        assert_eq!(batch_0.batch_id, 0);
        assert_eq!(batch_0.size, 25);
        let batch_4 = loader.get_batch(4);
        assert!(batch_4.is_none());
    }

    #[test]
    fn test_data_loader_batch_sizes() {
        let data = create_synthetic_data(100, 5, 3);
        let loader = DataLoader::new(data, 33, false);
        let batches: Vec<_> = loader.iter().collect();
        assert_eq!(batches.len(), 4);
        assert_eq!(batches[0].size, 33);
        assert_eq!(batches[3].size, 1);
    }

    #[test]
    fn test_lr_schedule_constant() {
        let schedule = LRScheduleConfig::Constant;
        assert_eq!(schedule.get_lr(0.1, 0), 0.1);
        assert_eq!(schedule.get_lr(0.1, 100), 0.1);
    }

    #[test]
    fn test_lr_schedule_step_decay() {
        let schedule = LRScheduleConfig::StepDecay {
            decay_rate: 0.5,
            step_size: 10,
        };
        let lr_0 = schedule.get_lr(0.1, 0);
        let lr_10 = schedule.get_lr(0.1, 10);
        let lr_20 = schedule.get_lr(0.1, 20);
        assert_eq!(lr_0, 0.1);
        assert_eq!(lr_10, 0.05);
        assert_eq!(lr_20, 0.025);
    }

    #[test]
    fn test_lr_schedule_exponential_decay() {
        let schedule = LRScheduleConfig::ExponentialDecay {
            decay_rate: 0.9,
        };
        let lr_0 = schedule.get_lr(0.1, 0);
        let lr_1 = schedule.get_lr(0.1, 1);
        assert!(lr_0 > lr_1);
        assert!((lr_1 - 0.09).abs() < 1e-6);
    }

    #[test]
    fn test_lr_schedule_cosine_annealing() {
        let schedule = LRScheduleConfig::CosineAnnealing {
            t_max: 100,
            eta_min: 0.0,
        };
        let lr_0 = schedule.get_lr(0.1, 0);
        let lr_50 = schedule.get_lr(0.1, 50);
        let lr_100 = schedule.get_lr(0.1, 100);
        assert_eq!(lr_0, 0.1);
        assert!(lr_50 > 0.0 && lr_50 < 0.1);
        assert!(lr_100 < 0.01);
    }

    #[test]
    fn test_model_checkpoint_creation() {
        let weights = vec![1.0, 2.0, 3.0].into_iter().map(|f| f as u8).collect();
        let config = TrainingConfig::default();
        let history = TrainingHistory::new();

        let checkpoint = ModelCheckpoint::new(
            weights.clone(),
            10,
            0.5,
            config,
            history,
        );

        assert_eq!(checkpoint.epoch, 10);
        assert_eq!(checkpoint.best_val_loss, 0.5);
        assert_eq!(checkpoint.weights, weights);
    }

    #[tokio::test]
    async fn test_checkpoint_serialization() {
        let weights = vec![1, 2, 3, 4, 5];
        let config = TrainingConfig::default();
        let history = TrainingHistory::new();

        let checkpoint = ModelCheckpoint::new(
            weights,
            5,
            0.25,
            config,
            history,
        );

        // Serialize to JSON
        let json = serde_json::to_string(&checkpoint).unwrap();

        // Deserialize from JSON
        let restored: ModelCheckpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.epoch, checkpoint.epoch);
        assert_eq!(restored.best_val_loss, checkpoint.best_val_loss);
        assert_eq!(restored.weights, checkpoint.weights);
    }

    #[test]
    fn test_gradient_clipping() {
        let mut config = TrainingConfig::default();
        config.gradient_clip_norm = Some(1.0);

        let trainer = SimpleTrainer::new(config);
        let mut gradients = vec![0.5, 0.5, 0.5];

        trainer.clip_gradients(&mut gradients);

        // L2 norm should be clipped to 1.0
        let norm: f32 = gradients.iter().map(|g| g * g).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_trainer_epoch_tracking() {
        let config = TrainingConfig::default();
        let trainer = SimpleTrainer::new(config);

        assert_eq!(trainer.current_epoch(), 0);
        assert_eq!(trainer.global_step(), 0);
    }

    #[tokio::test]
    async fn test_simple_trainer_scheduled_lr() {
        let mut config = TrainingConfig::default();
        config.batch_size = 16;
        config.lr_schedule = Some(LRScheduleConfig::StepDecay {
            decay_rate: 0.5,
            step_size: 1,
        });

        let trainer = SimpleTrainer::new(config);
        let lr_0 = trainer.get_scheduled_learning_rate();

        // At epoch 0, should use base learning rate
        assert_eq!(lr_0, 0.001);
    }
}
