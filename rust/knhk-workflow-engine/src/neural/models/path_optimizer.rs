// rust/knhk-workflow-engine/src/neural/models/path_optimizer.rs
//! Path Optimizer using Reinforcement Learning Policy Network
//!
//! **Architecture**:
//! ```text
//! Input: [state_embedding(128)] (workflow state + context)
//!        ↓
//!    Dense(256) → ReLU
//!        ↓
//!    Dense(128) → ReLU
//!        ↓
//!    Dense(64) → Softmax (action probabilities)
//! ```
//!
//! **Training**:
//! - Algorithm: Proximal Policy Optimization (PPO)
//! - Reward: -execution_time - 0.1 * resource_usage + 10 * success
//! - Target: 95% optimal path selection

use crate::error::{WorkflowError, WorkflowResult};
use super::ModelConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Path optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathOptimization {
    /// Recommended execution path (sequence of task IDs)
    pub recommended_path: Vec<String>,
    /// Action probabilities for each possible next step
    pub action_probabilities: Vec<(String, f64)>,
    /// Expected total execution time in milliseconds
    pub expected_time_ms: f64,
    /// Expected success rate (0.0-1.0)
    pub expected_success_rate: f64,
    /// Expected resource usage score
    pub expected_resource_score: f64,
    /// Confidence in recommendation (0.0-1.0)
    pub confidence: f64,
}

/// State representation for RL policy
#[derive(Debug, Clone)]
pub struct WorkflowState {
    /// Current task ID
    pub current_task: String,
    /// Completed tasks
    pub completed_tasks: Vec<String>,
    /// Available next tasks
    pub available_tasks: Vec<String>,
    /// Current resource usage
    pub resource_usage: f64,
    /// Elapsed time in milliseconds
    pub elapsed_time_ms: u64,
    /// State embedding (128-dim vector)
    pub embedding: Vec<f64>,
}

/// RL policy network for path optimization
pub struct PathOptimizer {
    config: ModelConfig,
    #[cfg(feature = "neural")]
    model: Arc<RwLock<Option<tract_onnx::prelude::SimplePlan<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>, tract_onnx::prelude::Graph<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>>>>>>,
    state_mean: Arc<RwLock<Vec<f64>>>,
    state_std: Arc<RwLock<Vec<f64>>>,
    /// Action space (task IDs)
    action_space: Arc<RwLock<Vec<String>>>,
    total_predictions: Arc<parking_lot::Mutex<u64>>,
    total_inference_time_us: Arc<parking_lot::Mutex<u64>>,
}

impl PathOptimizer {
    /// Create new path optimizer
    pub fn new(config: ModelConfig) -> WorkflowResult<Self> {
        Ok(Self {
            config,
            #[cfg(feature = "neural")]
            model: Arc::new(RwLock::new(None)),
            state_mean: Arc::new(RwLock::new(vec![0.0; 128])),
            state_std: Arc::new(RwLock::new(vec![1.0; 128])),
            action_space: Arc::new(RwLock::new(Vec::new())),
            total_predictions: Arc::new(parking_lot::Mutex::new(0)),
            total_inference_time_us: Arc::new(parking_lot::Mutex::new(0)),
        })
    }

    /// Load model from ONNX file
    #[cfg(feature = "neural")]
    pub fn load_model(&self, model_path: &std::path::Path) -> WorkflowResult<()> {
        use tract_onnx::prelude::*;

        let model = tract_onnx::onnx()
            .model_for_path(model_path)
            .map_err(|e| WorkflowError::Configuration(format!("Failed to load ONNX model: {}", e)))?
            .into_optimized()
            .map_err(|e| WorkflowError::Configuration(format!("Failed to optimize model: {}", e)))?
            .into_runnable()
            .map_err(|e| WorkflowError::Configuration(format!("Failed to create runnable model: {}", e)))?;

        *self.model.write() = Some(model);

        tracing::info!(model_path = ?model_path, "Loaded path optimizer model");
        Ok(())
    }

    #[cfg(not(feature = "neural"))]
    pub fn load_model(&self, _model_path: &std::path::Path) -> WorkflowResult<()> {
        Err(WorkflowError::Configuration("Neural network feature not enabled".to_string()))
    }

    /// Set action space (available tasks)
    pub fn set_action_space(&self, actions: Vec<String>) {
        *self.action_space.write() = actions;
    }

    /// Optimize execution path given current state
    #[cfg(feature = "neural")]
    pub fn optimize(&self, state: &WorkflowState) -> WorkflowResult<PathOptimization> {
        use tract_onnx::prelude::*;

        let start = std::time::Instant::now();

        let normalized = self.normalize_state(&state.embedding);

        let input_tensor = tract_ndarray::Array2::from_shape_vec(
            [1, state.embedding.len()],
            normalized.clone(),
        )
        .map_err(|e| WorkflowError::Configuration(format!("Failed to create tensor: {}", e)))?
        .into_dyn();

        let model_guard = self.model.read();
        let model = model_guard.as_ref()
            .ok_or_else(|| WorkflowError::Configuration("Model not loaded".to_string()))?;

        let result = model
            .run(tvec![input_tensor.into()])
            .map_err(|e| WorkflowError::Configuration(format!("Inference failed: {}", e)))?;

        let output = result[0]
            .to_array_view::<f32>()
            .map_err(|e| WorkflowError::Configuration(format!("Failed to extract output: {}", e)))?;

        // Output: action probabilities (softmax)
        let action_space = self.action_space.read();
        let mut action_probs: Vec<(String, f64)> = action_space
            .iter()
            .enumerate()
            .take(output.shape()[1])
            .map(|(i, action)| (action.clone(), output[[0, i]] as f64))
            .collect();

        // Sort by probability (descending)
        action_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Greedy path: select highest probability action
        let recommended_path = if let Some((action, _)) = action_probs.first() {
            let mut path = state.completed_tasks.clone();
            path.push(action.clone());
            path
        } else {
            state.completed_tasks.clone()
        };

        // Estimate metrics (simplified)
        let expected_time_ms = self.estimate_time(&recommended_path);
        let expected_success_rate = if action_probs.is_empty() {
            0.5
        } else {
            action_probs[0].1.min(0.98)
        };
        let expected_resource_score = 0.75; // Baseline

        let confidence = if action_probs.is_empty() {
            0.5
        } else {
            action_probs[0].1
        };

        let inference_time_us = start.elapsed().as_micros() as u64;
        *self.total_predictions.lock() += 1;
        *self.total_inference_time_us.lock() += inference_time_us;

        tracing::debug!(
            ?recommended_path,
            expected_time_ms,
            expected_success_rate,
            confidence,
            inference_time_us,
            "Path optimization"
        );

        Ok(PathOptimization {
            recommended_path,
            action_probabilities: action_probs,
            expected_time_ms,
            expected_success_rate,
            expected_resource_score,
            confidence,
        })
    }

    #[cfg(not(feature = "neural"))]
    pub fn optimize(&self, _state: &WorkflowState) -> WorkflowResult<PathOptimization> {
        Err(WorkflowError::Configuration("Neural network feature not enabled".to_string()))
    }

    fn normalize_state(&self, state_embedding: &[f64]) -> Vec<f64> {
        let mean = self.state_mean.read();
        let std = self.state_std.read();

        state_embedding
            .iter()
            .zip(mean.iter().zip(std.iter()))
            .map(|(s, (m, st))| (s - m) / st.max(1e-8))
            .collect()
    }

    fn estimate_time(&self, _path: &[String]) -> f64 {
        // Simplified time estimation (in production, use execution time predictor)
        500.0 // 500ms baseline
    }

    pub fn update_statistics(&self, state_embedding: &[f64]) {
        let mut mean = self.state_mean.write();
        let mut std = self.state_std.write();

        let alpha = 0.01;
        for (i, s) in state_embedding.iter().enumerate() {
            if i < mean.len() {
                mean[i] = (1.0 - alpha) * mean[i] + alpha * s;
                let diff = s - mean[i];
                std[i] = ((1.0 - alpha) * std[i].powi(2) + alpha * diff.powi(2)).sqrt();
            }
        }
    }

    pub fn get_metrics(&self) -> (u64, f64) {
        let total_preds = *self.total_predictions.lock();
        let total_time = *self.total_inference_time_us.lock();
        let avg_time_us = if total_preds > 0 {
            total_time as f64 / total_preds as f64
        } else {
            0.0
        };
        (total_preds, avg_time_us)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_optimizer_creation() {
        let config = ModelConfig::path_optimizer_default();
        let optimizer = PathOptimizer::new(config);
        assert!(optimizer.is_ok());
    }

    #[test]
    fn test_action_space() {
        let config = ModelConfig::path_optimizer_default();
        let optimizer = PathOptimizer::new(config).unwrap();

        let actions = vec!["task1".to_string(), "task2".to_string(), "task3".to_string()];
        optimizer.set_action_space(actions.clone());

        let action_space = optimizer.action_space.read();
        assert_eq!(*action_space, actions);
    }
}
