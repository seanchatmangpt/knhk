// rust/knhk-workflow-engine/src/neural/integration.rs
//! Integration with MAPE-K Autonomic Loop
//!
//! Neural predictions feed into the Analyze phase:
//! - Execution time predictions → Proactive resource allocation
//! - Resource estimates → Capacity planning
//! - Failure predictions → Preventive adaptations
//! - Path optimization → Smart routing

use crate::error::WorkflowResult;
use crate::autonomic::analyze::{Analysis, Anomaly, AnomalyType, HealthStatus};
use crate::autonomic::knowledge::KnowledgeBase;
use super::inference::{NeuralEngine, PredictionResult};
use super::features::WorkflowFeatures;
use super::{ExecutionTimePrediction, FailurePrediction};
use std::sync::Arc;

/// Neural-enhanced analyzer
pub struct NeuralAnalyzer {
    /// Neural inference engine
    neural_engine: Arc<NeuralEngine>,
    /// Prediction thresholds for anomaly detection
    time_prediction_threshold: f64,
    failure_probability_threshold: f64,
}

impl NeuralAnalyzer {
    /// Create new neural analyzer
    pub fn new(neural_engine: Arc<NeuralEngine>) -> Self {
        Self {
            neural_engine,
            time_prediction_threshold: 0.2, // 20% deviation from predicted
            failure_probability_threshold: 0.7, // 70% failure probability
        }
    }

    /// Analyze workflow using neural predictions
    pub async fn analyze_with_predictions(
        &self,
        features: &WorkflowFeatures,
        knowledge: &KnowledgeBase,
    ) -> WorkflowResult<Analysis> {
        let mut analysis = Analysis::new();

        // Get neural predictions
        let time_pred = self.neural_engine.predict_execution_time(features).await?;
        let failure_pred = self.neural_engine.predict_failure(features).await?;
        let resource_pred = self.neural_engine.predict_resources(features).await?;

        tracing::info!(
            predicted_time_ms = time_pred.prediction.mean_ms,
            failure_prob = failure_pred.prediction.failure_probability,
            cpu_cores = resource_pred.prediction.cpu_cores,
            memory_mb = resource_pred.prediction.memory_mb,
            inference_time_us = time_pred.inference_time_us,
            from_cache = time_pred.from_cache,
            "Neural predictions completed"
        );

        // Check for predicted anomalies
        self.check_time_anomaly(&time_pred, &mut analysis);
        self.check_failure_anomaly(&failure_pred, &mut analysis);
        self.check_resource_anomaly(&resource_pred, &mut analysis);

        // Update health status based on predictions
        self.update_health_status(&mut analysis);

        Ok(analysis)
    }

    /// Check if predicted execution time indicates an anomaly
    fn check_time_anomaly(
        &self,
        time_pred: &PredictionResult<ExecutionTimePrediction>,
        analysis: &mut Analysis,
    ) {
        let pred = &time_pred.prediction;

        // Check if uncertainty is too high
        let uncertainty_ratio = pred.std_ms / pred.mean_ms.max(1.0);
        if uncertainty_ratio > self.time_prediction_threshold {
            analysis.anomalies.push(Anomaly {
                anomaly_type: AnomalyType::Spike,
                metric: "execution_time_uncertainty".to_string(),
                current_value: uncertainty_ratio,
                expected_value: self.time_prediction_threshold,
                severity: uncertainty_ratio.min(1.0),
                timestamp_ms: Self::current_time_ms(),
            });

            analysis.adaptation_needed = true;
        }

        // Check if predicted time is excessive
        if pred.mean_ms > 10000.0 {
            // > 10 seconds
            analysis.anomalies.push(Anomaly {
                anomaly_type: AnomalyType::AboveThreshold,
                metric: "predicted_execution_time".to_string(),
                current_value: pred.mean_ms,
                expected_value: 10000.0,
                severity: (pred.mean_ms / 10000.0).min(1.0),
                timestamp_ms: Self::current_time_ms(),
            });

            analysis.adaptation_needed = true;
        }
    }

    /// Check if failure prediction indicates an anomaly
    fn check_failure_anomaly(
        &self,
        failure_pred: &PredictionResult<FailurePrediction>,
        analysis: &mut Analysis,
    ) {
        let pred = &failure_pred.prediction;

        if pred.failure_probability > self.failure_probability_threshold {
            analysis.anomalies.push(Anomaly {
                anomaly_type: AnomalyType::TrendDown, // Trending toward failure
                metric: "failure_probability".to_string(),
                current_value: pred.failure_probability,
                expected_value: self.failure_probability_threshold,
                severity: pred.failure_probability,
                timestamp_ms: Self::current_time_ms(),
            });

            analysis.adaptation_needed = true;

            tracing::warn!(
                failure_prob = pred.failure_probability,
                ?pred.failure_mode,
                "High failure probability detected"
            );
        }
    }

    /// Check if resource prediction indicates an anomaly
    fn check_resource_anomaly(
        &self,
        resource_pred: &PredictionResult<super::inference::ResourcePrediction>,
        analysis: &mut Analysis,
    ) {
        let pred = &resource_pred.prediction;

        // Check if resource requirements are excessive
        if pred.memory_mb > 4096.0 {
            // > 4GB
            analysis.anomalies.push(Anomaly {
                anomaly_type: AnomalyType::AboveThreshold,
                metric: "predicted_memory_mb".to_string(),
                current_value: pred.memory_mb,
                expected_value: 4096.0,
                severity: (pred.memory_mb / 4096.0 - 1.0).min(1.0),
                timestamp_ms: Self::current_time_ms(),
            });
        }

        if pred.cpu_cores > 8.0 {
            analysis.anomalies.push(Anomaly {
                anomaly_type: AnomalyType::AboveThreshold,
                metric: "predicted_cpu_cores".to_string(),
                current_value: pred.cpu_cores,
                expected_value: 8.0,
                severity: (pred.cpu_cores / 8.0 - 1.0).min(1.0),
                timestamp_ms: Self::current_time_ms(),
            });
        }
    }

    /// Update health status based on anomalies
    fn update_health_status(&self, analysis: &mut Analysis) {
        let critical_anomalies = analysis
            .anomalies
            .iter()
            .filter(|a| a.severity > 0.8)
            .count();

        let major_anomalies = analysis
            .anomalies
            .iter()
            .filter(|a| a.severity > 0.5 && a.severity <= 0.8)
            .count();

        analysis.health = if critical_anomalies > 0 {
            HealthStatus::Critical
        } else if major_anomalies > 0 {
            HealthStatus::Unhealthy
        } else if !analysis.anomalies.is_empty() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
    }

    fn current_time_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_neural_analyzer_creation() {
        let engine = NeuralEngine::new().expect("Failed to create neural engine");
        let analyzer = NeuralAnalyzer::new(Arc::new(engine));

        assert_eq!(analyzer.time_prediction_threshold, 0.2);
        assert_eq!(analyzer.failure_probability_threshold, 0.7);
    }
}
