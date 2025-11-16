// rust/knhk-workflow-engine/src/neural/inference.rs
//! Low-Latency Inference Engine with Caching
//!
//! **Performance Targets**:
//! - Inference latency: <10ms (95th percentile)
//! - Throughput: >1000 predictions/sec
//! - Cache hit rate: >80%

use crate::error::{WorkflowError, WorkflowResult};
use super::models::{
    ExecutionTimePredictor, ResourceEstimator, FailurePredictor, PathOptimizer,
    ModelConfig, ModelType,
};
use super::features::{WorkflowFeatures, FeatureExtractor};
use super::{ExecutionTimePrediction, ResourceRequirements, FailurePrediction, PathRecommendation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::path::Path;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;

/// Prediction result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult<T> {
    /// Prediction value
    pub prediction: T,
    /// Model version used
    pub model_version: String,
    /// Inference time in microseconds
    pub inference_time_us: u64,
    /// Whether result came from cache
    pub from_cache: bool,
}

/// Resource prediction from neural model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    pub cpu_cores: f64,
    pub memory_mb: f64,
    pub io_bandwidth_mbps: f64,
    pub confidence: f64,
}

/// Neural inference engine
pub struct NeuralEngine {
    /// Execution time predictor
    execution_time_predictor: Arc<ExecutionTimePredictor>,
    /// Resource estimator
    resource_estimator: Arc<ResourceEstimator>,
    /// Failure predictor
    failure_predictor: Arc<FailurePredictor>,
    /// Path optimizer
    path_optimizer: Arc<PathOptimizer>,
    /// Feature extractor
    feature_extractor: Arc<RwLock<FeatureExtractor>>,
    /// Prediction cache (feature hash -> prediction)
    execution_time_cache: Arc<RwLock<LruCache<u64, ExecutionTimePrediction>>>,
    resource_cache: Arc<RwLock<LruCache<u64, ResourcePrediction>>>,
    failure_cache: Arc<RwLock<LruCache<u64, FailurePrediction>>>,
    /// Cache statistics
    cache_hits: Arc<parking_lot::Mutex<u64>>,
    cache_misses: Arc<parking_lot::Mutex<u64>>,
}

impl NeuralEngine {
    /// Create new neural engine
    pub fn new() -> WorkflowResult<Self> {
        let cache_size = NonZeroUsize::new(1000).ok_or_else(|| {
            WorkflowError::Configuration("Invalid cache size".to_string())
        })?;

        Ok(Self {
            execution_time_predictor: Arc::new(ExecutionTimePredictor::new(
                ModelConfig::execution_time_default(),
            )?),
            resource_estimator: Arc::new(ResourceEstimator::new(
                ModelConfig::resource_estimator_default(),
            )?),
            failure_predictor: Arc::new(FailurePredictor::new(
                ModelConfig::failure_predictor_default(),
            )?),
            path_optimizer: Arc::new(PathOptimizer::new(
                ModelConfig::path_optimizer_default(),
            )?),
            feature_extractor: Arc::new(RwLock::new(FeatureExtractor::new())),
            execution_time_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            resource_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            failure_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            cache_hits: Arc::new(parking_lot::Mutex::new(0)),
            cache_misses: Arc::new(parking_lot::Mutex::new(0)),
        })
    }

    /// Load models from disk
    pub fn load_models(&self, model_dir: &Path) -> WorkflowResult<()> {
        #[cfg(feature = "neural")]
        {
            // Load execution time predictor
            let exec_time_path = model_dir.join("execution_time_lstm_v1.onnx");
            if exec_time_path.exists() {
                self.execution_time_predictor.load_model(&exec_time_path)?;
            } else {
                tracing::warn!(path = ?exec_time_path, "Execution time model not found");
            }

            // Load resource estimator
            let resource_path = model_dir.join("resource_estimator_v1.onnx");
            if resource_path.exists() {
                self.resource_estimator.load_model(&resource_path)?;
            } else {
                tracing::warn!(path = ?resource_path, "Resource estimator model not found");
            }

            // Load failure predictor
            let failure_path = model_dir.join("failure_classifier_v1.onnx");
            if failure_path.exists() {
                self.failure_predictor.load_model(&failure_path)?;
            } else {
                tracing::warn!(path = ?failure_path, "Failure predictor model not found");
            }

            // Load path optimizer
            let path_opt_path = model_dir.join("path_optimizer_policy_v1.onnx");
            if path_opt_path.exists() {
                self.path_optimizer.load_model(&path_opt_path)?;
            } else {
                tracing::warn!(path = ?path_opt_path, "Path optimizer model not found");
            }

            tracing::info!(model_dir = ?model_dir, "Loaded neural network models");
        }

        #[cfg(not(feature = "neural"))]
        {
            let _ = model_dir;
            tracing::warn!("Neural feature not enabled, models not loaded");
        }

        Ok(())
    }

    /// Predict execution time
    #[cfg(feature = "neural")]
    pub async fn predict_execution_time(
        &self,
        features: &WorkflowFeatures,
    ) -> WorkflowResult<PredictionResult<ExecutionTimePrediction>> {
        let start = std::time::Instant::now();
        let feature_hash = self.hash_features(features);

        // Check cache
        if let Some(cached) = self.execution_time_cache.write().get(&feature_hash) {
            *self.cache_hits.lock() += 1;
            return Ok(PredictionResult {
                prediction: cached.clone(),
                model_version: "lstm_v1".to_string(),
                inference_time_us: start.elapsed().as_micros() as u64,
                from_cache: true,
            });
        }

        *self.cache_misses.lock() += 1;

        // Run inference
        let feature_vec = features.to_vector();
        let time_pred = self.execution_time_predictor.predict(&feature_vec)?;

        let prediction = ExecutionTimePrediction {
            mean_ms: time_pred.mean_ms,
            std_ms: time_pred.std_ms,
            confidence: time_pred.confidence,
            model_version: "lstm_v1".to_string(),
        };

        // Cache result
        self.execution_time_cache.write().put(feature_hash, prediction.clone());

        Ok(PredictionResult {
            prediction,
            model_version: "lstm_v1".to_string(),
            inference_time_us: start.elapsed().as_micros() as u64,
            from_cache: false,
        })
    }

    #[cfg(not(feature = "neural"))]
    pub async fn predict_execution_time(
        &self,
        _features: &WorkflowFeatures,
    ) -> WorkflowResult<PredictionResult<ExecutionTimePrediction>> {
        Err(WorkflowError::Configuration("Neural feature not enabled".to_string()))
    }

    /// Predict resource requirements
    #[cfg(feature = "neural")]
    pub async fn predict_resources(
        &self,
        features: &WorkflowFeatures,
    ) -> WorkflowResult<PredictionResult<ResourcePrediction>> {
        let start = std::time::Instant::now();
        let feature_hash = self.hash_features(features);

        // Check cache
        if let Some(cached) = self.resource_cache.write().get(&feature_hash) {
            *self.cache_hits.lock() += 1;
            return Ok(PredictionResult {
                prediction: cached.clone(),
                model_version: "resource_v1".to_string(),
                inference_time_us: start.elapsed().as_micros() as u64,
                from_cache: true,
            });
        }

        *self.cache_misses.lock() += 1;

        // Run inference
        let feature_vec = features.to_vector();
        let resource_pred = self.resource_estimator.predict(&feature_vec)?;

        let prediction = ResourcePrediction {
            cpu_cores: resource_pred.cpu_cores,
            memory_mb: resource_pred.memory_mb,
            io_bandwidth_mbps: resource_pred.io_bandwidth_mbps,
            confidence: resource_pred.confidence,
        };

        // Cache result
        self.resource_cache.write().put(feature_hash, prediction.clone());

        Ok(PredictionResult {
            prediction,
            model_version: "resource_v1".to_string(),
            inference_time_us: start.elapsed().as_micros() as u64,
            from_cache: false,
        })
    }

    #[cfg(not(feature = "neural"))]
    pub async fn predict_resources(
        &self,
        _features: &WorkflowFeatures,
    ) -> WorkflowResult<PredictionResult<ResourcePrediction>> {
        Err(WorkflowError::Configuration("Neural feature not enabled".to_string()))
    }

    /// Predict failure probability
    #[cfg(feature = "neural")]
    pub async fn predict_failure(
        &self,
        features: &WorkflowFeatures,
    ) -> WorkflowResult<PredictionResult<FailurePrediction>> {
        let start = std::time::Instant::now();
        let feature_hash = self.hash_features(features);

        // Check cache
        if let Some(cached) = self.failure_cache.write().get(&feature_hash) {
            *self.cache_hits.lock() += 1;
            return Ok(PredictionResult {
                prediction: cached.clone(),
                model_version: "failure_v1".to_string(),
                inference_time_us: start.elapsed().as_micros() as u64,
                from_cache: true,
            });
        }

        *self.cache_misses.lock() += 1;

        // Run inference
        let feature_vec = features.to_vector();
        let failure_pred = self.failure_predictor.predict(&feature_vec)?;

        // Cache result
        self.failure_cache.write().put(feature_hash, failure_pred.clone());

        Ok(PredictionResult {
            prediction: failure_pred,
            model_version: "failure_v1".to_string(),
            inference_time_us: start.elapsed().as_micros() as u64,
            from_cache: false,
        })
    }

    #[cfg(not(feature = "neural"))]
    pub async fn predict_failure(
        &self,
        _features: &WorkflowFeatures,
    ) -> WorkflowResult<PredictionResult<FailurePrediction>> {
        Err(WorkflowError::Configuration("Neural feature not enabled".to_string()))
    }

    /// Hash features for cache key
    fn hash_features(&self, features: &WorkflowFeatures) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash feature vector (simplified - in production use better hashing)
        let vec = features.to_vector();
        for f in vec {
            f.to_bits().hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (u64, u64, f64) {
        let hits = *self.cache_hits.lock();
        let misses = *self.cache_misses.lock();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        (hits, misses, hit_rate)
    }

    /// Clear prediction caches
    pub fn clear_caches(&self) {
        self.execution_time_cache.write().clear();
        self.resource_cache.write().clear();
        self.failure_cache.write().clear();
        *self.cache_hits.lock() = 0;
        *self.cache_misses.lock() = 0;

        tracing::info!("Cleared prediction caches");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_engine_creation() {
        let engine = NeuralEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_cache_stats() {
        let engine = NeuralEngine::new().unwrap();
        let (hits, misses, hit_rate) = engine.get_cache_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(hit_rate, 0.0);
    }

    #[test]
    fn test_feature_hashing() {
        let engine = NeuralEngine::new().unwrap();
        let features1 = WorkflowFeatures::default_with_embedding(vec![1.0; 64]);
        let features2 = WorkflowFeatures::default_with_embedding(vec![1.0; 64]);
        let features3 = WorkflowFeatures::default_with_embedding(vec![2.0; 64]);

        let hash1 = engine.hash_features(&features1);
        let hash2 = engine.hash_features(&features2);
        let hash3 = engine.hash_features(&features3);

        // Same features should hash to same value
        assert_eq!(hash1, hash2);
        // Different features should hash to different values (usually)
        assert_ne!(hash1, hash3);
    }
}
