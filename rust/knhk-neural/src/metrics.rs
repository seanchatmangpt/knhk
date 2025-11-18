//! Neural Training Metrics and Monitoring
//!
//! Comprehensive metrics tracking for neural network training,
//! including loss curves, accuracy, convergence, and performance.
//!
//! DOCTRINE ALIGNMENT:
//! - Covenant 6: All observations drive everything
//! - Principle: O (Observation) - Full telemetry coverage

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Training metrics aggregated across epochs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    /// Current epoch
    pub epoch: usize,

    /// Training loss
    pub train_loss: f32,

    /// Validation loss
    pub validation_loss: Option<f32>,

    /// Training accuracy (0.0-1.0)
    pub train_accuracy: f32,

    /// Validation accuracy (0.0-1.0)
    pub validation_accuracy: Option<f32>,

    /// Learning rate at this epoch
    pub learning_rate: f32,

    /// Epoch duration
    pub epoch_duration: Duration,

    /// Gradient norm (for monitoring gradient flow)
    pub gradient_norm: f32,

    /// Number of parameter updates
    pub num_updates: usize,
}

/// Learning curve tracker
pub struct LearningCurve {
    /// Training loss history
    train_losses: VecDeque<f32>,

    /// Validation loss history
    val_losses: VecDeque<f32>,

    /// Maximum history length
    max_length: usize,

    /// Best validation loss seen
    best_val_loss: f32,

    /// Epoch of best validation loss
    best_epoch: usize,
}

impl LearningCurve {
    /// Create new learning curve tracker
    pub fn new(max_length: usize) -> Self {
        Self {
            train_losses: VecDeque::with_capacity(max_length),
            val_losses: VecDeque::with_capacity(max_length),
            max_length,
            best_val_loss: f32::MAX,
            best_epoch: 0,
        }
    }

    /// Add training loss
    pub fn add_train_loss(&mut self, loss: f32) {
        if self.train_losses.len() >= self.max_length {
            self.train_losses.pop_front();
        }
        self.train_losses.push_back(loss);
    }

    /// Add validation loss
    pub fn add_val_loss(&mut self, loss: f32, epoch: usize) {
        if self.val_losses.len() >= self.max_length {
            self.val_losses.pop_front();
        }
        self.val_losses.push_back(loss);

        if loss < self.best_val_loss {
            self.best_val_loss = loss;
            self.best_epoch = epoch;
        }
    }

    /// Check if training is improving
    pub fn is_improving(&self, patience: usize) -> bool {
        if self.val_losses.len() < patience {
            return true;
        }

        let recent = self.val_losses.iter().rev().take(patience);
        let min_recent = recent.clone().fold(f32::MAX, |a, &b| a.min(b));

        min_recent <= self.best_val_loss * 1.01 // 1% tolerance
    }

    /// Get convergence score (0.0 = diverging, 1.0 = converged)
    pub fn convergence_score(&self) -> f32 {
        if self.train_losses.len() < 10 {
            return 0.0;
        }

        // Calculate variance of recent losses
        let recent: Vec<f32> = self.train_losses.iter().rev().take(10).copied().collect();
        let mean = recent.iter().sum::<f32>() / recent.len() as f32;
        let variance = recent.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / recent.len() as f32;

        // Low variance = high convergence
        (1.0 / (1.0 + variance)).min(1.0)
    }

    /// Get best validation loss and epoch
    pub fn best(&self) -> (f32, usize) {
        (self.best_val_loss, self.best_epoch)
    }
}

/// Performance tracker for training speed and resource usage
pub struct PerformanceTracker {
    /// Samples per second
    throughput_history: VecDeque<f32>,

    /// Memory usage samples (MB)
    memory_usage: VecDeque<f32>,

    /// Start time of tracking
    start_time: Instant,

    /// Total samples processed
    total_samples: usize,
}

impl PerformanceTracker {
    /// Create new performance tracker
    pub fn new() -> Self {
        Self {
            throughput_history: VecDeque::with_capacity(100),
            memory_usage: VecDeque::with_capacity(100),
            start_time: Instant::now(),
            total_samples: 0,
        }
    }

    /// Record batch processing
    pub fn record_batch(&mut self, batch_size: usize, duration: Duration) {
        let samples_per_sec = batch_size as f32 / duration.as_secs_f32();

        if self.throughput_history.len() >= 100 {
            self.throughput_history.pop_front();
        }
        self.throughput_history.push_back(samples_per_sec);

        self.total_samples += batch_size;
    }

    /// Get average throughput (samples/sec)
    pub fn avg_throughput(&self) -> f32 {
        if self.throughput_history.is_empty() {
            return 0.0;
        }

        self.throughput_history.iter().sum::<f32>() / self.throughput_history.len() as f32
    }

    /// Get total training time
    pub fn total_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get total samples processed
    pub fn total_samples(&self) -> usize {
        self.total_samples
    }

    /// Estimate time remaining (given total epochs)
    pub fn estimate_remaining(&self, current_epoch: usize, total_epochs: usize) -> Duration {
        if current_epoch == 0 {
            return Duration::from_secs(0);
        }

        let elapsed = self.total_time();
        let epoch_duration = elapsed / current_epoch as u32;
        let remaining_epochs = total_epochs.saturating_sub(current_epoch);

        epoch_duration * remaining_epochs as u32
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Gradient statistics for monitoring training health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStats {
    /// L2 norm of gradients
    pub norm: f32,

    /// Maximum absolute gradient value
    pub max_abs: f32,

    /// Minimum absolute gradient value
    pub min_abs: f32,

    /// Mean absolute gradient value
    pub mean_abs: f32,

    /// Percentage of zero gradients
    pub zero_percentage: f32,
}

impl GradientStats {
    /// Compute gradient statistics
    pub fn compute(gradients: &[f32]) -> Self {
        if gradients.is_empty() {
            return Self {
                norm: 0.0,
                max_abs: 0.0,
                min_abs: 0.0,
                mean_abs: 0.0,
                zero_percentage: 0.0,
            };
        }

        let norm = gradients.iter().map(|g| g * g).sum::<f32>().sqrt();
        let abs_grads: Vec<f32> = gradients.iter().map(|g| g.abs()).collect();

        let max_abs = abs_grads.iter().copied().fold(0.0f32, f32::max);
        let min_abs = abs_grads.iter().copied().fold(f32::MAX, f32::min);
        let mean_abs = abs_grads.iter().sum::<f32>() / abs_grads.len() as f32;

        let zero_count = abs_grads.iter().filter(|&&g| g < 1e-8).count();
        let zero_percentage = (zero_count as f32 / abs_grads.len() as f32) * 100.0;

        Self {
            norm,
            max_abs,
            min_abs,
            mean_abs,
            zero_percentage,
        }
    }

    /// Check for vanishing gradients
    pub fn has_vanishing_gradients(&self) -> bool {
        self.mean_abs < 1e-6 || self.zero_percentage > 90.0
    }

    /// Check for exploding gradients
    pub fn has_exploding_gradients(&self, threshold: f32) -> bool {
        self.norm > threshold || self.max_abs > threshold
    }
}

/// Complete metrics summary for an epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochSummary {
    pub epoch: usize,
    pub train_loss: f32,
    pub val_loss: Option<f32>,
    pub train_accuracy: f32,
    pub val_accuracy: Option<f32>,
    pub learning_rate: f32,
    pub duration: Duration,
    pub throughput: f32,
    pub gradient_stats: GradientStats,
    pub convergence_score: f32,
}

impl EpochSummary {
    /// Format summary as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Epoch {:3} | Loss: {:.4} (val: {:.4}) | Acc: {:.2}% (val: {:.2}%) | LR: {:.6} | {:?} | {:.0} samples/s | Conv: {:.2}",
            self.epoch,
            self.train_loss,
            self.val_loss.unwrap_or(0.0),
            self.train_accuracy * 100.0,
            self.val_accuracy.unwrap_or(0.0) * 100.0,
            self.learning_rate,
            self.duration,
            self.throughput,
            self.convergence_score
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_curve() {
        let mut curve = LearningCurve::new(100);

        // Simulate improving training
        for i in 0..50 {
            let loss = 10.0 * (-(i as f32) / 10.0).exp();
            curve.add_train_loss(loss);
            curve.add_val_loss(loss * 1.1, i);
        }

        assert!(curve.convergence_score() > 0.8);
        assert!(curve.is_improving(5));

        let (best_loss, best_epoch) = curve.best();
        assert!(best_loss < 1.0);
        assert_eq!(best_epoch, 49);
    }

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();

        // Simulate batch processing
        for _ in 0..10 {
            tracker.record_batch(32, Duration::from_millis(10));
        }

        assert!(tracker.avg_throughput() > 0.0);
        assert_eq!(tracker.total_samples(), 320);
    }

    #[test]
    fn test_gradient_stats() {
        let gradients = vec![0.1, -0.2, 0.05, -0.15, 0.3];
        let stats = GradientStats::compute(&gradients);

        assert!(stats.norm > 0.0);
        assert_eq!(stats.max_abs, 0.3);
        assert!(!stats.has_vanishing_gradients());
        assert!(!stats.has_exploding_gradients(10.0));
    }

    #[test]
    fn test_gradient_vanishing() {
        let gradients = vec![1e-7, -1e-8, 1e-9];
        let stats = GradientStats::compute(&gradients);

        assert!(stats.has_vanishing_gradients());
    }

    #[test]
    fn test_gradient_exploding() {
        let gradients = vec![100.0, -200.0, 150.0];
        let stats = GradientStats::compute(&gradients);

        assert!(stats.has_exploding_gradients(50.0));
    }
}
