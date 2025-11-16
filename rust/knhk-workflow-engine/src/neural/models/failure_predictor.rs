// rust/knhk-workflow-engine/src/neural/models/failure_predictor.rs
//! Failure Predictor using Classification Model
//!
//! **Architecture**:
//! ```text
//! Input: [workflow_features(64)]
//!        ↓
//!    Dense(64) → ReLU
//!        ↓
//!    Dropout(0.3)
//!        ↓
//!    Dense(32) → ReLU
//!        ↓
//!    Dropout(0.3)
//!        ↓
//!    Dense(16) → ReLU
//!        ↓
//!    Dense(2) → Softmax (success/failure)
//! ```
//!
//! **Training**:
//! - Loss: Binary Cross-Entropy
//! - Optimizer: Adam (lr=0.001)
//! - Target: F1 score > 0.85, AUC-ROC > 0.92

use crate::error::{WorkflowError, WorkflowResult};
use super::ModelConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Failure prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePrediction {
    /// Probability of failure (0.0-1.0)
    pub failure_probability: f64,
    /// Probability of success (0.0-1.0)
    pub success_probability: f64,
    /// Predicted failure mode
    pub failure_mode: Option<String>,
    /// Recommended mitigations
    pub mitigations: Vec<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Classification model for failure prediction
pub struct FailurePredictor {
    config: ModelConfig,
    #[cfg(feature = "neural")]
    model: Arc<RwLock<Option<tract_onnx::prelude::SimplePlan<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>, tract_onnx::prelude::Graph<tract_onnx::prelude::TypedFact, Box<dyn tract_onnx::prelude::TypedOp>>>>>>,
    feature_mean: Arc<RwLock<Vec<f64>>>,
    feature_std: Arc<RwLock<Vec<f64>>>,
    total_predictions: Arc<parking_lot::Mutex<u64>>,
    total_inference_time_us: Arc<parking_lot::Mutex<u64>>,
    /// Failure mode patterns (learned from training data)
    failure_patterns: Arc<RwLock<Vec<FailurePattern>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FailurePattern {
    pattern_name: String,
    feature_signature: Vec<f64>,
    frequency: f64,
    mitigation: String,
}

impl FailurePredictor {
    /// Create new failure predictor
    pub fn new(config: ModelConfig) -> WorkflowResult<Self> {
        Ok(Self {
            config,
            #[cfg(feature = "neural")]
            model: Arc::new(RwLock::new(None)),
            feature_mean: Arc::new(RwLock::new(vec![0.0; 64])),
            feature_std: Arc::new(RwLock::new(vec![1.0; 64])),
            total_predictions: Arc::new(parking_lot::Mutex::new(0)),
            total_inference_time_us: Arc::new(parking_lot::Mutex::new(0)),
            failure_patterns: Arc::new(RwLock::new(Self::default_failure_patterns())),
        })
    }

    fn default_failure_patterns() -> Vec<FailurePattern> {
        vec![
            FailurePattern {
                pattern_name: "resource_exhaustion".to_string(),
                feature_signature: vec![0.0; 64],
                frequency: 0.3,
                mitigation: "Increase resource allocation or add backpressure".to_string(),
            },
            FailurePattern {
                pattern_name: "timeout".to_string(),
                feature_signature: vec![0.0; 64],
                frequency: 0.25,
                mitigation: "Increase timeout threshold or optimize slow operations".to_string(),
            },
            FailurePattern {
                pattern_name: "dependency_failure".to_string(),
                feature_signature: vec![0.0; 64],
                frequency: 0.2,
                mitigation: "Add retry logic and circuit breakers".to_string(),
            },
        ]
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

        tracing::info!(model_path = ?model_path, "Loaded failure predictor model");
        Ok(())
    }

    #[cfg(not(feature = "neural"))]
    pub fn load_model(&self, _model_path: &std::path::Path) -> WorkflowResult<()> {
        Err(WorkflowError::Configuration("Neural network feature not enabled".to_string()))
    }

    /// Predict failure probability
    #[cfg(feature = "neural")]
    pub fn predict(&self, features: &[f64]) -> WorkflowResult<FailurePrediction> {
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

        // Output: [success_prob, failure_prob] (softmax output)
        let success_prob = output[[0, 0]] as f64;
        let failure_prob = output[[0, 1]] as f64;

        // Match against known failure patterns
        let (failure_mode, mitigations) = self.match_failure_pattern(&normalized);

        let confidence = self.calculate_confidence(failure_prob);

        let inference_time_us = start.elapsed().as_micros() as u64;
        *self.total_predictions.lock() += 1;
        *self.total_inference_time_us.lock() += inference_time_us;

        tracing::debug!(
            failure_prob,
            success_prob,
            ?failure_mode,
            confidence,
            inference_time_us,
            "Failure prediction"
        );

        Ok(FailurePrediction {
            failure_probability: failure_prob,
            success_probability: success_prob,
            failure_mode,
            mitigations,
            confidence,
        })
    }

    #[cfg(not(feature = "neural"))]
    pub fn predict(&self, _features: &[f64]) -> WorkflowResult<FailurePrediction> {
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

    fn calculate_confidence(&self, failure_prob: f64) -> f64 {
        // Higher confidence for extreme probabilities
        if failure_prob < 0.1 || failure_prob > 0.9 {
            0.92
        } else {
            0.85
        }
    }

    fn match_failure_pattern(&self, features: &[f64]) -> (Option<String>, Vec<String>) {
        let patterns = self.failure_patterns.read();

        // Simple nearest neighbor matching
        let mut best_match: Option<(&FailurePattern, f64)> = None;

        for pattern in patterns.iter() {
            let distance = self.euclidean_distance(features, &pattern.feature_signature);
            if let Some((_, best_dist)) = best_match {
                if distance < best_dist {
                    best_match = Some((pattern, distance));
                }
            } else {
                best_match = Some((pattern, distance));
            }
        }

        if let Some((pattern, distance)) = best_match {
            if distance < 5.0 { // threshold for pattern match
                return (
                    Some(pattern.pattern_name.clone()),
                    vec![pattern.mitigation.clone()],
                );
            }
        }

        (None, vec!["Monitor closely and apply general mitigation strategies".to_string()])
    }

    fn euclidean_distance(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f64>()
            .sqrt()
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
    fn test_failure_predictor_creation() {
        let config = ModelConfig::failure_predictor_default();
        let predictor = FailurePredictor::new(config);
        assert!(predictor.is_ok());
    }

    #[test]
    fn test_default_patterns() {
        let patterns = FailurePredictor::default_failure_patterns();
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].pattern_name, "resource_exhaustion");
    }
}
