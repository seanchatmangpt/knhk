//! Workflow A/B Testing Framework
//!
//! Shadow execution framework for safely testing workflow changes.
//! Runs experimental workflows in parallel with production, comparing
//! results without affecting production traffic.
//!
//! # Philosophy
//!
//! Never deploy untested workflows to production. Shadow execution
//! provides statistical confidence before promotion.

use std::sync::Arc;
use std::collections::HashMap;

use crate::execution::{
    SelfExecutingWorkflow, SnapshotId, Receipt, ReceiptStore, ReceiptId,
    OntologyFile, HookFn,
};
use serde::{Deserialize, Serialize};

/// A/B test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub test_name: String,
    pub control_snapshot: SnapshotId,
    pub experiment_snapshot: SnapshotId,
    pub traffic_split: f64, // 0.0 to 1.0, percentage to experiment
    pub min_sample_size: usize,
    pub confidence_threshold: f64, // Statistical significance threshold
    pub max_duration_hours: u64,
}

/// A/B test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResults {
    pub control_metrics: TestVariantMetrics,
    pub experiment_metrics: TestVariantMetrics,
    pub statistical_significance: f64,
    pub winner: TestVariant,
    pub recommendation: TestRecommendation,
    pub sample_size: usize,
}

/// Test variant (control or experiment)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestVariant {
    Control,
    Experiment,
    Inconclusive,
}

/// Metrics for a test variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVariantMetrics {
    pub execution_count: usize,
    pub success_rate: f64,
    pub average_latency: f64,
    pub p95_latency: f64,
    pub chatman_violations: usize,
    pub error_rate: f64,
}

/// Test recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestRecommendation {
    PromoteExperiment { confidence: f64, improvement: f64 },
    KeepControl { reason: String },
    ContinueTesting { samples_needed: usize },
}

/// A/B test orchestrator
pub struct ABTestOrchestrator {
    workflow: Arc<SelfExecutingWorkflow>,
    active_tests: HashMap<String, ABTestConfig>,
    test_results: HashMap<String, Vec<(SnapshotId, ReceiptId)>>,
}

impl ABTestOrchestrator {
    pub fn new(workflow: Arc<SelfExecutingWorkflow>) -> Self {
        Self {
            workflow,
            active_tests: HashMap::new(),
            test_results: HashMap::new(),
        }
    }

    /// Start an A/B test
    pub fn start_test(&mut self, config: ABTestConfig) -> Result<(), String> {
        // Validate config
        if config.traffic_split < 0.0 || config.traffic_split > 1.0 {
            return Err("Traffic split must be between 0.0 and 1.0".to_string());
        }

        if config.min_sample_size < 30 {
            return Err("Minimum sample size must be at least 30 for statistical validity".to_string());
        }

        self.active_tests.insert(config.test_name.clone(), config);
        Ok(())
    }

    /// Execute workflow with A/B testing
    pub fn execute_with_test(
        &mut self,
        test_name: &str,
        hook_name: &str,
        observation: Vec<u8>,
        workflow_instance_id: String,
    ) -> Result<ReceiptId, String> {
        let config = self.active_tests.get(test_name)
            .ok_or_else(|| "Test not found".to_string())?
            .clone();

        // Determine variant based on traffic split
        let variant = self.select_variant(config.traffic_split);

        // Select snapshot based on variant
        let snapshot_id = match variant {
            TestVariant::Control => config.control_snapshot,
            TestVariant::Experiment => config.experiment_snapshot,
            TestVariant::Inconclusive => return Err("Invalid variant".to_string()),
        };

        // Set active snapshot
        self.workflow.set_active_snapshot(snapshot_id.clone())?;

        // Execute workflow
        let receipt_id = self.workflow.execute(
            hook_name,
            observation,
            workflow_instance_id,
        )?;

        // Record result
        self.test_results
            .entry(test_name.to_string())
            .or_insert_with(Vec::new)
            .push((snapshot_id, receipt_id.clone()));

        Ok(receipt_id)
    }

    /// Execute shadow test (both variants, compare results)
    pub fn execute_shadow_test(
        &mut self,
        test_name: &str,
        hook_name: &str,
        observation: Vec<u8>,
        workflow_instance_id: String,
    ) -> Result<ShadowTestResult, String> {
        let config = self.active_tests.get(test_name)
            .ok_or_else(|| "Test not found".to_string())?
            .clone();

        // Execute control
        self.workflow.set_active_snapshot(config.control_snapshot.clone())?;
        let control_id = self.workflow.execute(
            hook_name,
            observation.clone(),
            format!("{}-control", workflow_instance_id),
        )?;

        // Execute experiment (shadow)
        self.workflow.set_active_snapshot(config.experiment_snapshot.clone())?;
        let experiment_id = self.workflow.execute(
            hook_name,
            observation,
            format!("{}-experiment", workflow_instance_id),
        )?;

        // Get receipts
        let control_receipt = self.workflow.get_receipt(&control_id)?;
        let experiment_receipt = self.workflow.get_receipt(&experiment_id)?;

        // Compare results
        let results_match = control_receipt.o_in_hash == experiment_receipt.o_in_hash
            && control_receipt.success == experiment_receipt.success;

        let latency_delta = experiment_receipt.ticks_used as i32 - control_receipt.ticks_used as i32;

        Ok(ShadowTestResult {
            control_receipt_id: control_id,
            experiment_receipt_id: experiment_id,
            results_match,
            latency_delta,
            control_success: control_receipt.success,
            experiment_success: experiment_receipt.success,
        })
    }

    /// Analyze test results
    pub fn analyze_test(&self, test_name: &str) -> Result<ABTestResults, String> {
        let config = self.active_tests.get(test_name)
            .ok_or_else(|| "Test not found".to_string())?;

        let results = self.test_results.get(test_name)
            .ok_or_else(|| "No results yet".to_string())?;

        // Separate control and experiment results
        let mut control_receipts = Vec::new();
        let mut experiment_receipts = Vec::new();

        for (snapshot_id, receipt_id) in results {
            let receipt = self.workflow.get_receipt(receipt_id)?;
            if snapshot_id == &config.control_snapshot {
                control_receipts.push(receipt);
            } else {
                experiment_receipts.push(receipt);
            }
        }

        // Calculate metrics
        let control_metrics = Self::calculate_metrics(&control_receipts);
        let experiment_metrics = Self::calculate_metrics(&experiment_receipts);

        // Determine statistical significance (simplified z-test)
        let significance = self.calculate_significance(&control_metrics, &experiment_metrics);

        // Determine winner
        let winner = if significance > config.confidence_threshold {
            if experiment_metrics.success_rate > control_metrics.success_rate
                || experiment_metrics.average_latency < control_metrics.average_latency
            {
                TestVariant::Experiment
            } else {
                TestVariant::Control
            }
        } else {
            TestVariant::Inconclusive
        };

        // Generate recommendation
        let recommendation = self.generate_recommendation(
            &control_metrics,
            &experiment_metrics,
            &winner,
            significance,
            config,
        );

        Ok(ABTestResults {
            control_metrics,
            experiment_metrics,
            statistical_significance: significance,
            winner,
            recommendation,
            sample_size: results.len(),
        })
    }

    /// Select variant based on traffic split
    fn select_variant(&self, traffic_split: f64) -> TestVariant {
        use fastrand;
        if fastrand::f64() < traffic_split {
            TestVariant::Experiment
        } else {
            TestVariant::Control
        }
    }

    /// Calculate metrics for a set of receipts
    fn calculate_metrics(receipts: &[Receipt]) -> TestVariantMetrics {
        if receipts.is_empty() {
            return TestVariantMetrics {
                execution_count: 0,
                success_rate: 0.0,
                average_latency: 0.0,
                p95_latency: 0.0,
                chatman_violations: 0,
                error_rate: 1.0,
            };
        }

        let total = receipts.len();
        let successful = receipts.iter().filter(|r| r.success).count();
        let violations = receipts.iter().filter(|r| r.ticks_used > 8).count();

        let mut latencies: Vec<u32> = receipts.iter().map(|r| r.ticks_used).collect();
        latencies.sort_unstable();

        let avg_latency = latencies.iter().sum::<u32>() as f64 / total as f64;
        let p95_idx = (total as f64 * 0.95) as usize;
        let p95_latency = latencies.get(p95_idx).copied().unwrap_or(0) as f64;

        TestVariantMetrics {
            execution_count: total,
            success_rate: successful as f64 / total as f64,
            average_latency: avg_latency,
            p95_latency,
            chatman_violations: violations,
            error_rate: (total - successful) as f64 / total as f64,
        }
    }

    /// Calculate statistical significance (simplified)
    fn calculate_significance(
        &self,
        control: &TestVariantMetrics,
        experiment: &TestVariantMetrics,
    ) -> f64 {
        // Simplified z-test for proportions
        if control.execution_count < 30 || experiment.execution_count < 30 {
            return 0.0; // Not enough samples
        }

        let p1 = control.success_rate;
        let p2 = experiment.success_rate;
        let n1 = control.execution_count as f64;
        let n2 = experiment.execution_count as f64;

        let p_pool = (p1 * n1 + p2 * n2) / (n1 + n2);
        let se = (p_pool * (1.0 - p_pool) * (1.0 / n1 + 1.0 / n2)).sqrt();

        if se == 0.0 {
            return 0.0;
        }

        let z = ((p1 - p2) / se).abs();

        // Convert z-score to approximate confidence level
        // z > 1.96 ≈ 95% confidence
        // z > 2.58 ≈ 99% confidence
        if z > 2.58 {
            0.99
        } else if z > 1.96 {
            0.95
        } else if z > 1.64 {
            0.90
        } else {
            z / 2.58 * 0.9 // Linear approximation for lower z-scores
        }
    }

    /// Generate recommendation based on results
    fn generate_recommendation(
        &self,
        control: &TestVariantMetrics,
        experiment: &TestVariantMetrics,
        winner: &TestVariant,
        significance: f64,
        config: &ABTestConfig,
    ) -> TestRecommendation {
        match winner {
            TestVariant::Experiment => {
                let improvement = if control.average_latency > 0.0 {
                    ((control.average_latency - experiment.average_latency) / control.average_latency) * 100.0
                } else {
                    0.0
                };

                TestRecommendation::PromoteExperiment {
                    confidence: significance,
                    improvement,
                }
            }
            TestVariant::Control => {
                let reason = if experiment.error_rate > control.error_rate {
                    "Experiment has higher error rate".to_string()
                } else if experiment.average_latency > control.average_latency * 1.1 {
                    "Experiment is significantly slower".to_string()
                } else {
                    "Control performs better overall".to_string()
                };

                TestRecommendation::KeepControl { reason }
            }
            TestVariant::Inconclusive => {
                let current_samples = control.execution_count + experiment.execution_count;
                let samples_needed = config.min_sample_size.saturating_sub(current_samples);

                TestRecommendation::ContinueTesting { samples_needed }
            }
        }
    }

    /// Stop test and clean up
    pub fn stop_test(&mut self, test_name: &str) -> Result<ABTestResults, String> {
        let results = self.analyze_test(test_name)?;
        self.active_tests.remove(test_name);
        self.test_results.remove(test_name);
        Ok(results)
    }
}

/// Shadow test comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowTestResult {
    pub control_receipt_id: ReceiptId,
    pub experiment_receipt_id: ReceiptId,
    pub results_match: bool,
    pub latency_delta: i32, // Positive = experiment slower
    pub control_success: bool,
    pub experiment_success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ab_config_validation() {
        let mut orchestrator = ABTestOrchestrator::new(Arc::new(SelfExecutingWorkflow::new()));

        let invalid_config = ABTestConfig {
            test_name: "test".to_string(),
            control_snapshot: SnapshotId::from_string("control".to_string()),
            experiment_snapshot: SnapshotId::from_string("experiment".to_string()),
            traffic_split: 1.5, // Invalid
            min_sample_size: 100,
            confidence_threshold: 0.95,
            max_duration_hours: 24,
        };

        assert!(orchestrator.start_test(invalid_config).is_err());
    }

    #[test]
    fn test_variant_selection() {
        let orchestrator = ABTestOrchestrator::new(Arc::new(SelfExecutingWorkflow::new()));

        // Test traffic split
        let mut experiment_count = 0;
        let mut control_count = 0;

        for _ in 0..1000 {
            match orchestrator.select_variant(0.5) {
                TestVariant::Experiment => experiment_count += 1,
                TestVariant::Control => control_count += 1,
                _ => {}
            }
        }

        // Should be roughly 50/50 (allow 40-60% range)
        assert!(experiment_count > 400 && experiment_count < 600);
        assert!(control_count > 400 && control_count < 600);
    }

    #[test]
    fn test_metrics_calculation() {
        let mut receipts = Vec::new();

        // Create test receipts
        for i in 0..100 {
            let snapshot_id = SnapshotId::from_string("test".to_string());
            let mut receipt = Receipt::new(
                snapshot_id,
                b"input",
                b"output",
                format!("workflow-{}", i),
            );
            receipt.set_ticks((i % 10) as u32);
            receipts.push(receipt);
        }

        let metrics = ABTestOrchestrator::calculate_metrics(&receipts);

        assert_eq!(metrics.execution_count, 100);
        assert!(metrics.success_rate > 0.0);
        assert!(metrics.average_latency > 0.0);
    }

    #[test]
    fn test_statistical_significance() {
        let orchestrator = ABTestOrchestrator::new(Arc::new(SelfExecutingWorkflow::new()));

        let control = TestVariantMetrics {
            execution_count: 100,
            success_rate: 0.90,
            average_latency: 5.0,
            p95_latency: 7.0,
            chatman_violations: 0,
            error_rate: 0.10,
        };

        let experiment = TestVariantMetrics {
            execution_count: 100,
            success_rate: 0.95, // Significant improvement
            average_latency: 4.0,
            p95_latency: 6.0,
            chatman_violations: 0,
            error_rate: 0.05,
        };

        let significance = orchestrator.calculate_significance(&control, &experiment);
        assert!(significance > 0.0);
    }
}
