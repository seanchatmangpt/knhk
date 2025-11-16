// rust/knhk-workflow-engine/src/neural/ab_testing.rs
//! A/B Testing Framework for Model Versioning
//!
//! Enables:
//! - Multi-model deployment (A/B/C testing)
//! - Traffic splitting
//! - Performance comparison
//! - Automatic winner selection

use crate::error::{WorkflowError, WorkflowResult};
use super::inference::{NeuralEngine, PredictionResult};
use super::features::WorkflowFeatures;
use super::models::{ModelType, ModelMetrics};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;

/// Model variant for A/B testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVariant {
    /// Variant name (e.g., "model_v1", "model_v2")
    pub name: String,
    /// Model version
    pub version: String,
    /// Traffic weight (0.0-1.0)
    pub traffic_weight: f64,
    /// Whether this is the control group
    pub is_control: bool,
    /// Model directory path
    pub model_dir: String,
}

/// A/B test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    /// Test name
    pub test_name: String,
    /// Model type being tested
    pub model_type: ModelType,
    /// Model variants
    pub variants: Vec<ModelVariant>,
    /// Minimum samples before evaluation
    pub min_samples: usize,
    /// Confidence threshold for winner selection (0.0-1.0)
    pub confidence_threshold: f64,
}

/// A/B test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResults {
    /// Test name
    pub test_name: String,
    /// Variant metrics
    pub variant_metrics: HashMap<String, VariantMetrics>,
    /// Statistical significance
    pub statistical_significance: f64,
    /// Recommended winner
    pub recommended_winner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMetrics {
    /// Total predictions made
    pub total_predictions: u64,
    /// Average inference time in microseconds
    pub avg_inference_time_us: f64,
    /// Average prediction accuracy (compared to actual)
    pub avg_accuracy: f64,
    /// Error rate
    pub error_rate: f64,
}

/// A/B testing engine
pub struct ABTestingEngine {
    /// Test configuration
    config: ABTestConfig,
    /// Neural engines for each variant
    engines: HashMap<String, Arc<NeuralEngine>>,
    /// Variant metrics
    metrics: Arc<RwLock<HashMap<String, VariantMetrics>>>,
    /// Random number generator for traffic splitting
    rng: Arc<parking_lot::Mutex<fastrand::Rng>>,
}

impl ABTestingEngine {
    /// Create new A/B testing engine
    pub fn new(config: ABTestConfig) -> WorkflowResult<Self> {
        // Validate traffic weights sum to 1.0
        let total_weight: f64 = config.variants.iter().map(|v| v.traffic_weight).sum();
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(WorkflowError::Configuration(format!(
                "Traffic weights must sum to 1.0, got {}",
                total_weight
            )));
        }

        let mut engines = HashMap::new();
        let mut metrics = HashMap::new();

        // Create neural engine for each variant
        for variant in &config.variants {
            let engine = NeuralEngine::new()?;
            // In production, load model from variant.model_dir
            engines.insert(variant.name.clone(), Arc::new(engine));

            metrics.insert(
                variant.name.clone(),
                VariantMetrics {
                    total_predictions: 0,
                    avg_inference_time_us: 0.0,
                    avg_accuracy: 0.0,
                    error_rate: 0.0,
                },
            );
        }

        Ok(Self {
            config,
            engines,
            metrics: Arc::new(RwLock::new(metrics)),
            rng: Arc::new(parking_lot::Mutex::new(fastrand::Rng::new())),
        })
    }

    /// Select variant based on traffic weights
    fn select_variant(&self) -> &ModelVariant {
        let random_value = self.rng.lock().f64();
        let mut cumulative = 0.0;

        for variant in &self.config.variants {
            cumulative += variant.traffic_weight;
            if random_value <= cumulative {
                return variant;
            }
        }

        // Fallback to first variant
        &self.config.variants[0]
    }

    /// Run prediction with A/B testing
    pub async fn predict_with_ab_test(
        &self,
        features: &WorkflowFeatures,
    ) -> WorkflowResult<(String, PredictionResult<crate::neural::ExecutionTimePrediction>)> {
        let variant = self.select_variant();
        let engine = self
            .engines
            .get(&variant.name)
            .ok_or_else(|| WorkflowError::Configuration(format!("Variant {} not found", variant.name)))?;

        let result = engine.predict_execution_time(features).await?;

        // Update metrics
        let mut metrics = self.metrics.write();
        if let Some(variant_metrics) = metrics.get_mut(&variant.name) {
            variant_metrics.total_predictions += 1;

            // Update average inference time (exponential moving average)
            let alpha = 0.1;
            variant_metrics.avg_inference_time_us = (1.0 - alpha) * variant_metrics.avg_inference_time_us
                + alpha * result.inference_time_us as f64;
        }

        tracing::debug!(
            variant = %variant.name,
            inference_time_us = result.inference_time_us,
            "A/B test prediction"
        );

        Ok((variant.name.clone(), result))
    }

    /// Update accuracy metrics (called after actual execution)
    pub fn update_accuracy(&self, variant_name: &str, predicted: f64, actual: f64) {
        let mut metrics = self.metrics.write();
        if let Some(variant_metrics) = metrics.get_mut(variant_name) {
            let error = (predicted - actual).abs() / actual.max(1.0);
            let alpha = 0.1;

            // Update accuracy (lower error = higher accuracy)
            let accuracy = 1.0 - error.min(1.0);
            variant_metrics.avg_accuracy =
                (1.0 - alpha) * variant_metrics.avg_accuracy + alpha * accuracy;

            // Update error rate
            variant_metrics.error_rate = (1.0 - alpha) * variant_metrics.error_rate + alpha * error;
        }
    }

    /// Evaluate A/B test and determine winner
    pub fn evaluate(&self) -> WorkflowResult<ABTestResults> {
        let metrics = self.metrics.read();

        // Check if we have enough samples
        let total_samples: u64 = metrics.values().map(|m| m.total_predictions).sum();
        if total_samples < self.config.min_samples as u64 {
            return Err(WorkflowError::Configuration(format!(
                "Not enough samples for evaluation: {} < {}",
                total_samples, self.config.min_samples
            )));
        }

        // Find best variant by accuracy
        let mut best_variant: Option<(String, f64)> = None;
        for (name, m) in metrics.iter() {
            if let Some((_, best_accuracy)) = &best_variant {
                if m.avg_accuracy > *best_accuracy {
                    best_variant = Some((name.clone(), m.avg_accuracy));
                }
            } else {
                best_variant = Some((name.clone(), m.avg_accuracy));
            }
        }

        // Calculate statistical significance (simplified)
        let control_variant = self
            .config
            .variants
            .iter()
            .find(|v| v.is_control)
            .map(|v| v.name.as_str());

        let statistical_significance = if let Some(control_name) = control_variant {
            if let (Some(control_metrics), Some((winner_name, _))) =
                (metrics.get(control_name), &best_variant)
            {
                if let Some(winner_metrics) = metrics.get(winner_name) {
                    // Simple significance: relative improvement
                    (winner_metrics.avg_accuracy - control_metrics.avg_accuracy)
                        / control_metrics.avg_accuracy.max(0.01)
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let recommended_winner = if statistical_significance >= self.config.confidence_threshold {
            best_variant.map(|(name, _)| name)
        } else {
            None
        };

        Ok(ABTestResults {
            test_name: self.config.test_name.clone(),
            variant_metrics: metrics.clone(),
            statistical_significance,
            recommended_winner,
        })
    }

    /// Get current metrics for all variants
    pub fn get_metrics(&self) -> HashMap<String, VariantMetrics> {
        self.metrics.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ab_test_config_validation() {
        // Valid config
        let config = ABTestConfig {
            test_name: "execution_time_v1_vs_v2".to_string(),
            model_type: ModelType::ExecutionTime,
            variants: vec![
                ModelVariant {
                    name: "model_v1".to_string(),
                    version: "1.0".to_string(),
                    traffic_weight: 0.5,
                    is_control: true,
                    model_dir: "models/v1".to_string(),
                },
                ModelVariant {
                    name: "model_v2".to_string(),
                    version: "2.0".to_string(),
                    traffic_weight: 0.5,
                    is_control: false,
                    model_dir: "models/v2".to_string(),
                },
            ],
            min_samples: 1000,
            confidence_threshold: 0.05,
        };

        let engine = ABTestingEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_invalid_traffic_weights() {
        let config = ABTestConfig {
            test_name: "test".to_string(),
            model_type: ModelType::ExecutionTime,
            variants: vec![
                ModelVariant {
                    name: "v1".to_string(),
                    version: "1.0".to_string(),
                    traffic_weight: 0.6, // Total > 1.0
                    is_control: true,
                    model_dir: "models/v1".to_string(),
                },
                ModelVariant {
                    name: "v2".to_string(),
                    version: "2.0".to_string(),
                    traffic_weight: 0.6,
                    is_control: false,
                    model_dir: "models/v2".to_string(),
                },
            ],
            min_samples: 100,
            confidence_threshold: 0.05,
        };

        let engine = ABTestingEngine::new(config);
        assert!(engine.is_err());
    }

    #[test]
    fn test_variant_selection() {
        let config = ABTestConfig {
            test_name: "test".to_string(),
            model_type: ModelType::ExecutionTime,
            variants: vec![
                ModelVariant {
                    name: "v1".to_string(),
                    version: "1.0".to_string(),
                    traffic_weight: 0.7,
                    is_control: true,
                    model_dir: "models/v1".to_string(),
                },
                ModelVariant {
                    name: "v2".to_string(),
                    version: "2.0".to_string(),
                    traffic_weight: 0.3,
                    is_control: false,
                    model_dir: "models/v2".to_string(),
                },
            ],
            min_samples: 100,
            confidence_threshold: 0.05,
        };

        let engine = ABTestingEngine::new(config).unwrap();

        // Test variant selection distribution
        let mut counts = HashMap::new();
        for _ in 0..1000 {
            let variant = engine.select_variant();
            *counts.entry(variant.name.clone()).or_insert(0) += 1;
        }

        // v1 should get ~70% of traffic
        let v1_count = counts.get("v1").copied().unwrap_or(0);
        assert!(v1_count > 600 && v1_count < 800, "v1 count: {}", v1_count);
    }
}
