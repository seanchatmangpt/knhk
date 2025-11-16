// rust/knhk-workflow-engine/src/neural/models/resource_estimator.rs
//! Resource Estimator using Feed-Forward Neural Network
//!
//! **Architecture**:
//! ```text
//! Input: [workflow_features(64)]
//!        ↓
//!    Dense(64) → ReLU
//!        ↓
//!    Dropout(0.2)
//!        ↓
//!    Dense(32) → ReLU
//!        ↓
//!    Dropout(0.2)
//!        ↓
//!    Dense(3) → Output (CPU cores, Memory MB, I/O bandwidth)
//! ```
//!
//! **Training**:
//! - Loss: Mean Absolute Error (MAE)
//! - Optimizer: Adam (lr=0.001)
//! - Target: MAE < 10% of actual usage

use crate::error::{WorkflowError, WorkflowResult};
use super::ModelConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Resource prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    /// Predicted CPU cores needed
    pub cpu_cores: f64,
    /// Predicted memory in MB
    pub memory_mb: f64,
    /// Predicted I/O bandwidth in MB/s
    pub io_bandwidth_mbps: f64,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Feed-forward neural network for resource estimation
pub struct ResourceEstimator {
    config: ModelConfig,
    #[cfg(feature = "neural")]
    model: Arc<RwLock<Option<tract_onnx::prelude::SimplePlan<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>, tract_onnx::prelude::Graph<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>>>>>>,
    feature_mean: Arc<RwLock<Vec<f64>>>,
    feature_std: Arc<RwLock<Vec<f64>>>,
    total_predictions: Arc<parking_lot::Mutex<u64>>,
    total_inference_time_us: Arc<parking_lot::Mutex<u64>>,
}

impl ResourceEstimator {
    /// Create new resource estimator
    pub fn new(config: ModelConfig) -> WorkflowResult<Self> {
        Ok(Self {
            config,
            #[cfg(feature = "neural")]
            model: Arc::new(RwLock::new(None)),
            feature_mean: Arc::new(RwLock::new(vec![0.0; 64])),
            feature_std: Arc::new(RwLock::new(vec![1.0; 64])),
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

        tracing::info!(model_path = ?model_path, "Loaded resource estimator model");
        Ok(())
    }

    #[cfg(not(feature = "neural"))]
    pub fn load_model(&self, _model_path: &std::path::Path) -> WorkflowResult<()> {
        Err(WorkflowError::Configuration("Neural network feature not enabled".to_string()))
    }

    /// Predict resource requirements
    #[cfg(feature = "neural")]
    pub fn predict(&self, features: &[f64]) -> WorkflowResult<ResourcePrediction> {
        use tract_onnx::prelude::*;

        let start = std::time::Instant::now();

        let normalized = self.normalize_features(features);

        let input_tensor = tract_ndarray::Array2::from_shape_vec(
            [1, features.len()],
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

        // Output: [CPU cores, Memory MB, I/O bandwidth MB/s]
        let cpu_cores = output[[0, 0]].max(0.5) as f64; // At least 0.5 cores
        let memory_mb = output[[0, 1]].max(128.0) as f64; // At least 128 MB
        let io_bandwidth_mbps = output[[0, 2]].max(1.0) as f64; // At least 1 MB/s

        let confidence = self.calculate_confidence(&normalized);

        let inference_time_us = start.elapsed().as_micros() as u64;
        *self.total_predictions.lock() += 1;
        *self.total_inference_time_us.lock() += inference_time_us;

        tracing::debug!(
            cpu_cores,
            memory_mb,
            io_bandwidth_mbps,
            confidence,
            inference_time_us,
            "Resource prediction"
        );

        Ok(ResourcePrediction {
            cpu_cores,
            memory_mb,
            io_bandwidth_mbps,
            confidence,
        })
    }

    #[cfg(not(feature = "neural"))]
    pub fn predict(&self, _features: &[f64]) -> WorkflowResult<ResourcePrediction> {
        Err(WorkflowError::Configuration("Neural network feature not enabled".to_string()))
    }

    fn normalize_features(&self, features: &[f64]) -> Vec<f64> {
        let mean = self.feature_mean.read();
        let std = self.feature_std.read();

        features
            .iter()
            .zip(mean.iter().zip(std.iter()))
            .map(|(f, (m, s))| (f - m) / s.max(1e-8))
            .collect()
    }

    fn calculate_confidence(&self, _normalized_features: &[f64]) -> f64 {
        0.88 // Baseline confidence
    }

    pub fn update_statistics(&self, features: &[f64]) {
        let mut mean = self.feature_mean.write();
        let mut std = self.feature_std.write();

        let alpha = 0.01;
        for (i, f) in features.iter().enumerate() {
            if i < mean.len() {
                mean[i] = (1.0 - alpha) * mean[i] + alpha * f;
                let diff = f - mean[i];
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
    fn test_resource_estimator_creation() {
        let config = ModelConfig::resource_estimator_default();
        let estimator = ResourceEstimator::new(config);
        assert!(estimator.is_ok());
    }

    #[test]
    fn test_metrics_tracking() {
        let config = ModelConfig::resource_estimator_default();
        let estimator = ResourceEstimator::new(config).unwrap();

        let (preds, avg_time) = estimator.get_metrics();
        assert_eq!(preds, 0);
        assert_eq!(avg_time, 0.0);
    }
}
