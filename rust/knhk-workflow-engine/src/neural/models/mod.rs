// rust/knhk-workflow-engine/src/neural/models/mod.rs
//! Neural Network Model Implementations
//!
//! Provides four specialized models:
//! 1. Execution Time Predictor (LSTM/Transformer)
//! 2. Resource Estimator (Feed-forward NN)
//! 3. Failure Predictor (Classification)
//! 4. Path Optimizer (Reinforcement Learning)

pub mod execution_time;
pub mod resource_estimator;
pub mod failure_predictor;
pub mod path_optimizer;

pub use execution_time::ExecutionTimePredictor;
pub use resource_estimator::ResourceEstimator;
pub use failure_predictor::FailurePredictor;
pub use path_optimizer::PathOptimizer;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Model type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    /// Execution time prediction (LSTM)
    ExecutionTime,
    /// Resource estimation (feed-forward)
    ResourceEstimation,
    /// Failure prediction (classification)
    FailurePrediction,
    /// Path optimization (RL policy)
    PathOptimization,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::ExecutionTime => "execution_time",
            ModelType::ResourceEstimation => "resource_estimation",
            ModelType::FailurePrediction => "failure_prediction",
            ModelType::PathOptimization => "path_optimization",
        }
    }
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model type
    pub model_type: ModelType,
    /// Model version
    pub version: String,
    /// Path to model file (ONNX format)
    pub model_path: PathBuf,
    /// Input dimension
    pub input_dim: usize,
    /// Output dimension
    pub output_dim: usize,
    /// Hidden layer sizes
    pub hidden_sizes: Vec<usize>,
    /// Whether to use GPU acceleration
    pub use_gpu: bool,
    /// Batch size for inference
    pub batch_size: usize,
}

impl ModelConfig {
    /// Create default config for execution time predictor
    pub fn execution_time_default() -> Self {
        Self {
            model_type: ModelType::ExecutionTime,
            version: "v1.0".to_string(),
            model_path: PathBuf::from("models/execution_time_lstm_v1.onnx"),
            input_dim: 64, // workflow embedding size
            output_dim: 1, // predicted time
            hidden_sizes: vec![128, 128], // 2 LSTM layers
            use_gpu: false,
            batch_size: 32,
        }
    }

    /// Create default config for resource estimator
    pub fn resource_estimator_default() -> Self {
        Self {
            model_type: ModelType::ResourceEstimation,
            version: "v1.0".to_string(),
            model_path: PathBuf::from("models/resource_estimator_v1.onnx"),
            input_dim: 64,
            output_dim: 3, // CPU, memory, I/O
            hidden_sizes: vec![64, 32],
            use_gpu: false,
            batch_size: 64,
        }
    }

    /// Create default config for failure predictor
    pub fn failure_predictor_default() -> Self {
        Self {
            model_type: ModelType::FailurePrediction,
            version: "v1.0".to_string(),
            model_path: PathBuf::from("models/failure_classifier_v1.onnx"),
            input_dim: 64,
            output_dim: 2, // binary classification (success/failure)
            hidden_sizes: vec![64, 32, 16],
            use_gpu: false,
            batch_size: 64,
        }
    }

    /// Create default config for path optimizer
    pub fn path_optimizer_default() -> Self {
        Self {
            model_type: ModelType::PathOptimization,
            version: "v1.0".to_string(),
            model_path: PathBuf::from("models/path_optimizer_policy_v1.onnx"),
            input_dim: 128, // state embedding
            output_dim: 64, // action space
            hidden_sizes: vec![256, 128],
            use_gpu: false,
            batch_size: 32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_type_str() {
        assert_eq!(ModelType::ExecutionTime.as_str(), "execution_time");
        assert_eq!(ModelType::ResourceEstimation.as_str(), "resource_estimation");
        assert_eq!(ModelType::FailurePrediction.as_str(), "failure_prediction");
        assert_eq!(ModelType::PathOptimization.as_str(), "path_optimization");
    }

    #[test]
    fn test_default_configs() {
        let exec_cfg = ModelConfig::execution_time_default();
        assert_eq!(exec_cfg.model_type, ModelType::ExecutionTime);
        assert_eq!(exec_cfg.input_dim, 64);
        assert_eq!(exec_cfg.output_dim, 1);
        assert_eq!(exec_cfg.hidden_sizes, vec![128, 128]);

        let resource_cfg = ModelConfig::resource_estimator_default();
        assert_eq!(resource_cfg.output_dim, 3); // CPU, memory, I/O

        let failure_cfg = ModelConfig::failure_predictor_default();
        assert_eq!(failure_cfg.output_dim, 2); // binary classification
    }
}
