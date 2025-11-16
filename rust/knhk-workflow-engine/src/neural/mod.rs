// rust/knhk-workflow-engine/src/neural/mod.rs
//! Neural Network-Based Workflow Prediction and Optimization
//!
//! Implements ML models for predicting workflow behavior:
//! - Execution time prediction (LSTM)
//! - Resource estimation (feed-forward NN)
//! - Failure probability (classification)
//! - Path optimization (reinforcement learning)
//!
//! **Integration with MAPE-K**:
//! - Monitor: Collect execution telemetry for training
//! - Analyze: Use ML predictions to detect potential issues
//! - Plan: Generate optimal configurations based on predictions
//! - Execute: Apply ML-guided adaptations
//!
//! **Performance**:
//! - Inference latency: <10ms (target)
//! - Prediction accuracy: 90%+ for execution time
//! - Model versioning and A/B testing support

#[cfg(feature = "neural")]
pub mod models;
#[cfg(feature = "neural")]
pub mod features;
#[cfg(feature = "neural")]
pub mod training;
#[cfg(feature = "neural")]
pub mod inference;
#[cfg(feature = "neural")]
pub mod integration;
#[cfg(feature = "neural")]
pub mod ab_testing;

#[cfg(feature = "neural")]
pub use self::features::{WorkflowFeatures, FeatureExtractor};
#[cfg(feature = "neural")]
pub use self::inference::{NeuralEngine, PredictionResult, ResourcePrediction};
#[cfg(feature = "neural")]
pub use self::models::{ModelType, ModelConfig};
#[cfg(feature = "neural")]
pub use self::training::{TrainingPipeline, OnlineLearner};

use crate::error::WorkflowResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Neural prediction for execution time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimePrediction {
    /// Mean predicted time in milliseconds
    pub mean_ms: f64,
    /// Standard deviation (uncertainty)
    pub std_ms: f64,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Model version used
    pub model_version: String,
}

/// Neural prediction for resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Predicted CPU cores needed
    pub cpu_cores: f64,
    /// Predicted memory in MB
    pub memory_mb: f64,
    /// Predicted I/O bandwidth in MB/s
    pub io_bandwidth_mbps: f64,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Neural prediction for failure probability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePrediction {
    /// Probability of failure (0.0-1.0)
    pub failure_probability: f64,
    /// Most likely failure mode
    pub failure_mode: Option<String>,
    /// Recommended mitigations
    pub mitigations: Vec<String>,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
}

/// Optimal workflow path recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathRecommendation {
    /// Recommended execution path
    pub recommended_path: Vec<String>,
    /// Expected success rate
    pub success_rate: f64,
    /// Expected total time in milliseconds
    pub expected_time_ms: f64,
    /// Expected resource usage
    pub expected_resources: ResourceRequirements,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model type
    pub model_type: String,
    /// Model version
    pub version: String,
    /// Accuracy or R² score
    pub accuracy: f64,
    /// Mean absolute error (for regression)
    pub mae: Option<f64>,
    /// Root mean squared error (for regression)
    pub rmse: Option<f64>,
    /// F1 score (for classification)
    pub f1_score: Option<f64>,
    /// AUC-ROC (for classification)
    pub auc_roc: Option<f64>,
    /// Average inference time in microseconds
    pub avg_inference_time_us: f64,
    /// Total predictions made
    pub total_predictions: u64,
}

/// Stub implementation when neural feature is disabled
#[cfg(not(feature = "neural"))]
pub struct NeuralEngine;

#[cfg(not(feature = "neural"))]
impl NeuralEngine {
    pub fn new() -> WorkflowResult<Self> {
        Err(crate::error::WorkflowError::Configuration(
            "Neural network feature not enabled. Compile with --features neural".to_string(),
        ))
    }

    pub async fn predict_execution_time(
        &self,
        _features: &WorkflowFeatures,
    ) -> WorkflowResult<ExecutionTimePrediction> {
        Err(crate::error::WorkflowError::Configuration(
            "Neural network feature not enabled".to_string(),
        ))
    }
}

#[cfg(not(feature = "neural"))]
#[derive(Debug, Clone)]
pub struct WorkflowFeatures;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_types_serialization() {
        let exec_pred = ExecutionTimePrediction {
            mean_ms: 1234.5,
            std_ms: 123.4,
            confidence: 0.92,
            model_version: "lstm_v2".to_string(),
        };

        let json = serde_json::to_string(&exec_pred).unwrap();
        let decoded: ExecutionTimePrediction = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.mean_ms, 1234.5);
        assert_eq!(decoded.confidence, 0.92);
    }

    #[test]
    fn test_resource_prediction_serialization() {
        let resources = ResourceRequirements {
            cpu_cores: 4.0,
            memory_mb: 2048.0,
            io_bandwidth_mbps: 100.0,
            confidence: 0.88,
        };

        let json = serde_json::to_string(&resources).unwrap();
        let decoded: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.cpu_cores, 4.0);
        assert_eq!(decoded.memory_mb, 2048.0);
    }
}
