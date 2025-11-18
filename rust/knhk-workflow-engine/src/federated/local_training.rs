//! Local training coordinator implementation (stub)
//!
//! This module provides a skeleton for local training.
//! Full implementation requires neural network model, which is beyond
//! the scope of this initial specification.

use super::traits::{ExperienceBuffer, LocalModel, LocalTrainer, Optimizer};
use super::types::{Experience, FederatedError, Gradients, LocalTrainingMetrics};
use async_trait::async_trait;
use std::fmt::Debug;
use tracing::instrument;

/// Simple circular experience buffer
#[derive(Debug)]
pub struct CircularBuffer {
    experiences: Vec<Experience>,
    capacity: usize,
    next_idx: usize,
}

impl CircularBuffer {
    /// Create a new circular buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            experiences: Vec::with_capacity(capacity),
            capacity,
            next_idx: 0,
        }
    }
}

impl ExperienceBuffer for CircularBuffer {
    fn push(&mut self, experience: Experience) {
        if self.experiences.len() < self.capacity {
            self.experiences.push(experience);
        } else {
            self.experiences[self.next_idx] = experience;
            self.next_idx = (self.next_idx + 1) % self.capacity;
        }
    }

    fn sample(&self, batch_size: usize) -> Vec<Experience> {
        if self.experiences.is_empty() {
            return vec![];
        }

        let actual_batch_size = batch_size.min(self.experiences.len());

        // Simple random sampling using fastrand
        (0..actual_batch_size)
            .map(|_| {
                let idx = fastrand::usize(0..self.experiences.len());
                self.experiences[idx].clone()
            })
            .collect()
    }

    fn len(&self) -> usize {
        self.experiences.len()
    }

    fn is_empty(&self) -> bool {
        self.experiences.is_empty()
    }
}

/// Stub local model (placeholder for actual neural network)
///
/// **TODO**: Implement actual neural network model (2-layer MLP)
#[derive(Debug)]
pub struct StubLocalModel {
    agent_id: String,
    params: Vec<f32>,
}

impl StubLocalModel {
    pub fn new(agent_id: String, input_dim: usize, hidden_dim: usize, output_dim: usize) -> Self {
        let total_params = input_dim * hidden_dim + hidden_dim + hidden_dim * output_dim + output_dim;

        Self {
            agent_id,
            params: vec![0.5; total_params], // Initialize with 0.5
        }
    }
}

impl LocalModel for StubLocalModel {
    fn compute_loss(&self, _batch: &[Experience]) -> Result<f64, FederatedError> {
        // Stub: return random loss
        Ok(fastrand::f64() * 0.5 + 0.1)
    }

    fn compute_gradients(&self, _batch: &[Experience]) -> Result<Gradients, FederatedError> {
        // Stub: return random gradients
        let gradients = Gradients {
            values: self.params.iter().map(|_| fastrand::f32()).collect(),
            agent_id: self.agent_id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            round: 0,
        };

        Ok(gradients)
    }

    fn apply_gradients(&mut self, gradients: &Gradients) -> Result<(), FederatedError> {
        if gradients.values.len() != self.params.len() {
            return Err(FederatedError::ModelError(format!(
                "Gradient dimension mismatch: expected {}, got {}",
                self.params.len(),
                gradients.values.len()
            )));
        }

        // Simple gradient descent: θ ← θ - lr × ∇L
        let lr = 0.01;
        for (param, grad) in self.params.iter_mut().zip(gradients.values.iter()) {
            *param -= lr * grad;
        }

        Ok(())
    }

    fn predict(&self, _state: &[f32]) -> Result<usize, FederatedError> {
        // Stub: return random action
        Ok(fastrand::usize(0..10))
    }

    fn serialize_params(&self) -> Result<Vec<f32>, FederatedError> {
        Ok(self.params.clone())
    }

    fn load_params(&mut self, params: &[f32]) -> Result<(), FederatedError> {
        if params.len() != self.params.len() {
            return Err(FederatedError::ModelError(format!(
                "Parameter dimension mismatch: expected {}, got {}",
                self.params.len(),
                params.len()
            )));
        }

        self.params.copy_from_slice(params);
        Ok(())
    }
}

/// SGD optimizer with momentum (stub)
#[derive(Debug)]
pub struct SGDOptimizer {
    learning_rate: f64,
    momentum: f64,
    velocity: Vec<f32>,
}

impl SGDOptimizer {
    pub fn new(learning_rate: f64, momentum: f64, param_count: usize) -> Self {
        Self {
            learning_rate,
            momentum,
            velocity: vec![0.0; param_count],
        }
    }
}

impl Optimizer for SGDOptimizer {
    fn step(&mut self, gradients: &Gradients, model: &mut dyn LocalModel) -> Result<(), FederatedError> {
        // SGD with momentum: v ← β×v + ∇L, θ ← θ - lr×v
        for (v, g) in self.velocity.iter_mut().zip(gradients.values.iter()) {
            *v = (self.momentum as f32) * (*v) + g;
        }

        let momentum_gradients = Gradients {
            values: self
                .velocity
                .iter()
                .map(|v| (self.learning_rate as f32) * v)
                .collect(),
            agent_id: gradients.agent_id.clone(),
            timestamp: gradients.timestamp,
            round: gradients.round,
        };

        model.apply_gradients(&momentum_gradients)
    }

    fn learning_rate(&self) -> f64 {
        self.learning_rate
    }

    fn set_learning_rate(&mut self, lr: f64) {
        self.learning_rate = lr;
    }
}

/// Local training coordinator (stub implementation)
#[derive(Debug)]
pub struct LocalTrainingCoordinator {
    agent_id: String,
    model: StubLocalModel,
    buffer: CircularBuffer,
    optimizer: SGDOptimizer,
}

impl LocalTrainingCoordinator {
    pub fn new(agent_id: String) -> Self {
        let input_dim = 100;
        let hidden_dim = 64;
        let output_dim = 10;
        let total_params = input_dim * hidden_dim + hidden_dim + hidden_dim * output_dim + output_dim;

        Self {
            agent_id: agent_id.clone(),
            model: StubLocalModel::new(agent_id, input_dim, hidden_dim, output_dim),
            buffer: CircularBuffer::new(10000),
            optimizer: SGDOptimizer::new(0.01, 0.9, total_params),
        }
    }
}

#[async_trait]
impl LocalTrainer for LocalTrainingCoordinator {
    #[instrument(
        name = "federated.learning.local_training",
        skip(self),
        fields(
            agent_id = %self.agent_id,
            num_epochs = num_epochs,
            batch_size = batch_size,
        )
    )]
    async fn train_local_round(
        &mut self,
        num_epochs: usize,
        batch_size: usize,
    ) -> Result<LocalTrainingMetrics, FederatedError> {
        let start = std::time::Instant::now();

        if self.buffer.is_empty() {
            return Err(FederatedError::TrainingError(
                "Experience buffer is empty".into(),
            ));
        }

        let mut total_loss = 0.0;

        // Training loop
        for _epoch in 0..num_epochs {
            // Sample batch
            let batch = self.buffer.sample(batch_size);

            // Compute loss
            let loss = self.model.compute_loss(&batch)?;
            total_loss += loss;

            // Compute gradients
            let gradients = self.model.compute_gradients(&batch)?;

            // Optimizer step
            self.optimizer.step(&gradients, &mut self.model)?;
        }

        let avg_loss = total_loss / num_epochs as f64;
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            agent_id = %self.agent_id,
            loss = avg_loss,
            duration_ms = duration_ms,
            "Local training completed"
        );

        Ok(LocalTrainingMetrics {
            loss: avg_loss,
            epochs: num_epochs,
            batch_size,
            duration_ms,
            experiences_count: self.buffer.len(),
        })
    }

    fn model(&self) -> &dyn LocalModel {
        &self.model
    }

    fn model_mut(&mut self) -> &mut dyn LocalModel {
        &mut self.model
    }

    fn buffer(&self) -> &dyn ExperienceBuffer {
        &self.buffer
    }

    fn buffer_mut(&mut self) -> &mut dyn ExperienceBuffer {
        &mut self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_buffer() {
        let mut buffer = CircularBuffer::new(10);

        // Add experiences
        for i in 0..15 {
            buffer.push(Experience {
                state: vec![i as f32],
                action: i,
                reward: i as f64,
                next_state: vec![(i + 1) as f32],
                done: false,
            });
        }

        // Buffer should have 10 items (capacity)
        assert_eq!(buffer.len(), 10);

        // Sample
        let batch = buffer.sample(5);
        assert_eq!(batch.len(), 5);
    }

    #[tokio::test]
    async fn test_local_training() {
        let mut coordinator = LocalTrainingCoordinator::new("test_agent".into());

        // Populate buffer
        for i in 0..100 {
            coordinator.buffer_mut().push(Experience {
                state: vec![i as f32; 100],
                action: i % 10,
                reward: (i as f64) / 100.0,
                next_state: vec![(i + 1) as f32; 100],
                done: false,
            });
        }

        // Train
        let metrics = coordinator
            .train_local_round(10, 32)
            .await
            .unwrap();

        assert_eq!(metrics.epochs, 10);
        assert_eq!(metrics.batch_size, 32);
        assert!(metrics.loss >= 0.0);
        assert!(metrics.duration_ms > 0);
    }
}
