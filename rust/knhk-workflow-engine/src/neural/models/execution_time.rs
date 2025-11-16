// rust/knhk-workflow-engine/src/neural/models/execution_time.rs
//! Execution Time Predictor using LSTM/Transformer
//!
//! **Architecture**:
//! ```text
//! Input: [workflow_pattern_embedding(64), data_size(1), num_tasks(1), dependencies(1)]
//!        ↓
//!    LSTM(128 hidden) × 2 layers
//!        ↓
//!    Attention layer
//!        ↓
//!    Dense(64) → ReLU
//!        ↓
//!    Dense(32) → ReLU
//!        ↓
//!    Dense(1) → Output (predicted time in ms)
//! ```
//!
//! **Training**:
//! - Loss: Mean Squared Error (MSE)
//! - Optimizer: Adam (lr=0.001)
//! - Target: RMSE < 50ms (for workflows <5s), R² > 0.90

use crate::error::{WorkflowError, WorkflowResult};
use super::ModelConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

#[cfg(feature = "neural")]
use ndarray::{Array1, Array2};

/// Execution time prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePrediction {
    /// Mean predicted time in milliseconds
    pub mean_ms: f64,
    /// Standard deviation (uncertainty estimate)
    pub std_ms: f64,
    /// Lower bound (95% confidence interval)
    pub lower_bound_ms: f64,
    /// Upper bound (95% confidence interval)
    pub upper_bound_ms: f64,
    /// Model confidence (0.0-1.0)
    pub confidence: f64,
}

/// LSTM-based execution time predictor
pub struct ExecutionTimePredictor {
    config: ModelConfig,
    #[cfg(feature = "neural")]
    model: Arc<RwLock<Option<tract_onnx::prelude::SimplePlan<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>, tract_onnx::prelude::Graph<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>>>>>>,
    /// Running statistics for normalization
    feature_mean: Arc<RwLock<Vec<f64>>>,
    feature_std: Arc<RwLock<Vec<f64>>>,
    /// Model performance metrics
    total_predictions: Arc<parking_lot::Mutex<u64>>,
    total_inference_time_us: Arc<parking_lot::Mutex<u64>>,
}

impl ExecutionTimePredictor {
    /// Create new execution time predictor
    pub fn new(config: ModelConfig) -> WorkflowResult<Self> {
        Ok(Self {
            config,
            #[cfg(feature = "neural")]
            model: Arc::new(RwLock::new(None)),
            feature_mean: Arc::new(RwLock::new(vec![0.0; 67])), // 64 + 3 features
            feature_std: Arc::new(RwLock::new(vec![1.0; 67])),
            total_predictions: Arc::new(parking_lot::Mutex::new(0)),
            total_inference_time_us: Arc::new(parking_lot::Mutex::new(0)),
        })
    }

    /// Load model from ONNX file
    #[cfg(feature = "neural")]
    pub fn load_model(&self, model_path: &std::path::Path) -> WorkflowResult<()> {
        use tract_onnx::prelude::*;

        let start = std::time::Instant::now();

        // Load ONNX model
        let model = tract_onnx::onnx()
            .model_for_path(model_path)
            .map_err(|e| WorkflowError::Configuration(format!("Failed to load ONNX model: {}", e)))?
            .into_optimized()
            .map_err(|e| WorkflowError::Configuration(format!("Failed to optimize model: {}", e)))?
            .into_runnable()
            .map_err(|e| WorkflowError::Configuration(format!("Failed to create runnable model: {}", e)))?;

        *self.model.write() = Some(model);

        tracing::info!(
            model_path = ?model_path,
            load_time_ms = start.elapsed().as_millis(),
            "Loaded execution time predictor model"
        );

        Ok(())
    }

    /// Stub for when neural feature is disabled
    #[cfg(not(feature = "neural"))]
    pub fn load_model(&self, _model_path: &std::path::Path) -> WorkflowResult<()> {
        Err(WorkflowError::Configuration(
            "Neural network feature not enabled".to_string(),
        ))
    }

    /// Predict execution time for given features
    #[cfg(feature = "neural")]
    pub fn predict(&self, features: &[f64]) -> WorkflowResult<TimePrediction> {
        use tract_onnx::prelude::*;

        let start = std::time::Instant::now();

        // Normalize features
        let normalized = self.normalize_features(features);

        // Convert to tensor
        let input_shape = &[1, features.len()]; // batch_size=1
        let input_tensor = tract_ndarray::Array2::from_shape_vec(
            *input_shape,
            normalized.clone(),
        )
        .map_err(|e| WorkflowError::Configuration(format!("Failed to create input tensor: {}", e)))?
        .into_dyn();

        // Run inference
        let model_guard = self.model.read();
        let model = model_guard.as_ref()
            .ok_or_else(|| WorkflowError::Configuration("Model not loaded".to_string()))?;

        let result = model
            .run(tvec![input_tensor.into()])
            .map_err(|e| WorkflowError::Configuration(format!("Inference failed: {}", e)))?;

        // Extract prediction
        let output = result[0]
            .to_array_view::<f32>()
            .map_err(|e| WorkflowError::Configuration(format!("Failed to extract output: {}", e)))?;

        let predicted_time_ms = output[[0, 0]] as f64;

        // Estimate uncertainty (simple heuristic: 10% of predicted time)
        let std_ms = predicted_time_ms * 0.1;

        // Calculate confidence based on feature quality
        let confidence = self.calculate_confidence(&normalized);

        // Update metrics
        let inference_time_us = start.elapsed().as_micros() as u64;
        *self.total_predictions.lock() += 1;
        *self.total_inference_time_us.lock() += inference_time_us;

        tracing::debug!(
            predicted_time_ms,
            confidence,
            inference_time_us,
            "Execution time prediction"
        );

        Ok(TimePrediction {
            mean_ms: predicted_time_ms,
            std_ms,
            lower_bound_ms: predicted_time_ms - 1.96 * std_ms, // 95% CI
            upper_bound_ms: predicted_time_ms + 1.96 * std_ms,
            confidence,
        })
    }

    /// Stub for when neural feature is disabled
    #[cfg(not(feature = "neural"))]
    pub fn predict(&self, _features: &[f64]) -> WorkflowResult<TimePrediction> {
        Err(WorkflowError::Configuration(
            "Neural network feature not enabled".to_string(),
        ))
    }

    /// Normalize features using running statistics
    fn normalize_features(&self, features: &[f64]) -> Vec<f64> {
        let mean = self.feature_mean.read();
        let std = self.feature_std.read();

        features
            .iter()
            .zip(mean.iter().zip(std.iter()))
            .map(|(f, (m, s))| (f - m) / s.max(1e-8))
            .collect()
    }

    /// Calculate confidence score based on feature quality
    fn calculate_confidence(&self, _normalized_features: &[f64]) -> f64 {
        // Simple heuristic: high confidence for now
        // In production, this would be based on:
        // - Distance from training distribution
        // - Model ensemble agreement
        // - Feature quality metrics
        0.90
    }

    /// Update normalization statistics (for online learning)
    pub fn update_statistics(&self, features: &[f64]) {
        let mut mean = self.feature_mean.write();
        let mut std = self.feature_std.write();

        // Simple exponential moving average (alpha=0.01)
        let alpha = 0.01;
        for (i, f) in features.iter().enumerate() {
            if i < mean.len() {
                mean[i] = (1.0 - alpha) * mean[i] + alpha * f;
                let diff = f - mean[i];
                std[i] = ((1.0 - alpha) * std[i].powi(2) + alpha * diff.powi(2)).sqrt();
            }
        }
    }

    /// Get model performance metrics
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
    fn test_predictor_creation() {
        let config = ModelConfig::execution_time_default();
        let predictor = ExecutionTimePredictor::new(config);
        assert!(predictor.is_ok());
    }

    #[test]
    fn test_normalization() {
        let config = ModelConfig::execution_time_default();
        let predictor = ExecutionTimePredictor::new(config).unwrap();

        let features = vec![1.0, 2.0, 3.0];
        let normalized = predictor.normalize_features(&features);

        // Should normalize around mean=0, std=1
        assert_eq!(normalized.len(), 3);
    }

    #[test]
    fn test_metrics_tracking() {
        let config = ModelConfig::execution_time_default();
        let predictor = ExecutionTimePredictor::new(config).unwrap();

        let (preds, avg_time) = predictor.get_metrics();
        assert_eq!(preds, 0);
        assert_eq!(avg_time, 0.0);
    }
}
