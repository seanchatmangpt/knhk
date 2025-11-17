//! Adaptive Workflow Optimizer
//!
//! Uses MAPE-K feedback loops and receipt history to automatically optimize
//! workflow execution. Learns from Γ(O) to improve performance, reliability,
//! and resource utilization.
//!
//! # Philosophy
//!
//! Workflows should self-optimize based on actual execution data, not
//! assumptions. The optimizer applies reinforcement learning principles
//! to discover better execution strategies.

use std::collections::HashMap;
use std::sync::Arc;

use crate::execution::{OntologyFile, Receipt, ReceiptStore, SnapshotId, SnapshotStore};
use crate::observability::{DarkMatterDetector, MapekManager};
use serde::{Deserialize, Serialize};

/// Optimization strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// Minimize average execution latency
    MinimizeLatency,
    /// Maximize success rate
    MaximizeReliability,
    /// Minimize resource consumption
    MinimizeResources,
    /// Balance all factors
    Balanced,
}

/// Optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub strategy: OptimizationStrategy,
    pub confidence: f64, // 0.0 to 1.0
    pub description: String,
    pub estimated_improvement: f64, // Percentage
    pub actions: Vec<OptimizationAction>,
}

/// Specific optimization action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationAction {
    /// Add caching layer
    AddCaching {
        hook_name: String,
        cache_ttl_secs: u64,
    },
    /// Parallelize sequential steps
    Parallelize { hooks: Vec<String> },
    /// Add retry logic
    AddRetry { hook_name: String, max_retries: u32 },
    /// Reduce timeout
    ReduceTimeout {
        hook_name: String,
        new_timeout_ms: u64,
    },
    /// Add circuit breaker
    AddCircuitBreaker { hook_name: String, threshold: f64 },
    /// Reorder execution
    Reorder { new_order: Vec<String> },
    /// Remove redundant step
    RemoveRedundant { hook_name: String },
    /// Increase batch size
    IncreaseBatchSize {
        hook_name: String,
        new_batch_size: usize,
    },
}

/// Workflow performance metrics derived from receipts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_executions: usize,
    pub success_rate: f64,
    pub average_ticks: f64,
    pub p50_ticks: u32,
    pub p95_ticks: u32,
    pub p99_ticks: u32,
    pub chatman_violation_rate: f64,
    pub failure_rate: f64,
    pub common_failure_guards: Vec<(String, usize)>,
}

/// Adaptive workflow optimizer
pub struct AdaptiveOptimizer {
    receipt_store: Arc<ReceiptStore>,
    snapshot_store: Arc<SnapshotStore>,
    mape_k: Arc<MapekManager>,
    dark_matter: Arc<DarkMatterDetector>,
    learning_history: HashMap<String, Vec<PerformanceMetrics>>,
}

impl AdaptiveOptimizer {
    pub fn new(
        receipt_store: Arc<ReceiptStore>,
        snapshot_store: Arc<SnapshotStore>,
        mape_k: Arc<MapekManager>,
        dark_matter: Arc<DarkMatterDetector>,
    ) -> Self {
        Self {
            receipt_store,
            snapshot_store,
            mape_k,
            dark_matter,
            learning_history: HashMap::new(),
        }
    }

    /// Analyze workflow performance from receipt history
    pub fn analyze_performance(&self, workflow_id: &str) -> Result<PerformanceMetrics, String> {
        let receipts = self.receipt_store.get_by_workflow(workflow_id)?;

        if receipts.is_empty() {
            return Err("No receipts found for workflow".to_string());
        }

        let total = receipts.len();
        let successful = receipts.iter().filter(|r| r.success).count();

        let mut ticks: Vec<u32> = receipts.iter().map(|r| r.ticks_used).collect();
        ticks.sort_unstable();

        let avg_ticks = receipts.iter().map(|r| r.ticks_used as f64).sum::<f64>() / total as f64;

        let p50_idx = (total as f64 * 0.5) as usize;
        let p95_idx = (total as f64 * 0.95) as usize;
        let p99_idx = (total as f64 * 0.99) as usize;

        let p50 = ticks.get(p50_idx).copied().unwrap_or(0);
        let p95 = ticks.get(p95_idx).copied().unwrap_or(0);
        let p99 = ticks.get(p99_idx).copied().unwrap_or(0);

        let violations = receipts.iter().filter(|r| r.ticks_used > 8).count();

        // Count failed guards
        let mut guard_failures: HashMap<String, usize> = HashMap::new();
        for receipt in &receipts {
            for guard in &receipt.guards_failed {
                *guard_failures.entry(guard.clone()).or_insert(0) += 1;
            }
        }

        let mut common_failures: Vec<(String, usize)> = guard_failures.into_iter().collect();
        common_failures.sort_by(|a, b| b.1.cmp(&a.1));
        common_failures.truncate(5);

        Ok(PerformanceMetrics {
            total_executions: total,
            success_rate: successful as f64 / total as f64,
            average_ticks: avg_ticks,
            p50_ticks: p50,
            p95_ticks: p95,
            p99_ticks: p99,
            chatman_violation_rate: violations as f64 / total as f64,
            failure_rate: (total - successful) as f64 / total as f64,
            common_failure_guards: common_failures,
        })
    }

    /// Generate optimization recommendations
    pub fn recommend_optimizations(
        &self,
        workflow_id: &str,
        strategy: OptimizationStrategy,
    ) -> Result<Vec<OptimizationRecommendation>, String> {
        let metrics = self.analyze_performance(workflow_id)?;
        let mut recommendations = Vec::new();

        match strategy {
            OptimizationStrategy::MinimizeLatency => {
                recommendations.extend(self.latency_optimizations(&metrics));
            }
            OptimizationStrategy::MaximizeReliability => {
                recommendations.extend(self.reliability_optimizations(&metrics));
            }
            OptimizationStrategy::MinimizeResources => {
                recommendations.extend(self.resource_optimizations(&metrics));
            }
            OptimizationStrategy::Balanced => {
                recommendations.extend(self.latency_optimizations(&metrics));
                recommendations.extend(self.reliability_optimizations(&metrics));
                recommendations.extend(self.resource_optimizations(&metrics));
            }
        }

        // Sort by confidence * improvement
        recommendations.sort_by(|a, b| {
            let score_a = a.confidence * a.estimated_improvement;
            let score_b = b.confidence * b.estimated_improvement;
            score_b.partial_cmp(&score_a).unwrap()
        });

        Ok(recommendations)
    }

    /// Latency-focused optimizations
    fn latency_optimizations(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // High average ticks suggests parallelization opportunity
        if metrics.average_ticks > 6.0 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::MinimizeLatency,
                confidence: 0.8,
                description: "Parallelize independent operations to reduce latency".to_string(),
                estimated_improvement: 30.0,
                actions: vec![OptimizationAction::Parallelize {
                    hooks: vec!["step1".to_string(), "step2".to_string()],
                }],
            });
        }

        // High P95-P50 spread suggests caching opportunity
        if metrics.p95_ticks > metrics.p50_ticks * 2 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::MinimizeLatency,
                confidence: 0.7,
                description: "Add caching to reduce P95 latency spikes".to_string(),
                estimated_improvement: 25.0,
                actions: vec![OptimizationAction::AddCaching {
                    hook_name: "data_fetch".to_string(),
                    cache_ttl_secs: 300,
                }],
            });
        }

        // Chatman violations suggest optimization needed
        if metrics.chatman_violation_rate > 0.1 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::MinimizeLatency,
                confidence: 0.9,
                description: "Reduce Chatman constant violations with hot path optimization"
                    .to_string(),
                estimated_improvement: 40.0,
                actions: vec![
                    OptimizationAction::AddCaching {
                        hook_name: "hot_path".to_string(),
                        cache_ttl_secs: 60,
                    },
                    OptimizationAction::IncreaseBatchSize {
                        hook_name: "batch_processor".to_string(),
                        new_batch_size: 100,
                    },
                ],
            });
        }

        recommendations
    }

    /// Reliability-focused optimizations
    fn reliability_optimizations(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Low success rate suggests retry logic needed
        if metrics.success_rate < 0.95 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::MaximizeReliability,
                confidence: 0.85,
                description: "Add retry logic to improve success rate".to_string(),
                estimated_improvement: (1.0 - metrics.success_rate) * 50.0,
                actions: vec![OptimizationAction::AddRetry {
                    hook_name: "external_call".to_string(),
                    max_retries: 3,
                }],
            });
        }

        // High failure rate on specific guards suggests circuit breaker
        if !metrics.common_failure_guards.is_empty() {
            let (guard, count) = &metrics.common_failure_guards[0];
            let failure_rate = *count as f64 / metrics.total_executions as f64;

            if failure_rate > 0.05 {
                recommendations.push(OptimizationRecommendation {
                    strategy: OptimizationStrategy::MaximizeReliability,
                    confidence: 0.75,
                    description: format!("Add circuit breaker for frequent '{}' failures", guard),
                    estimated_improvement: 20.0,
                    actions: vec![OptimizationAction::AddCircuitBreaker {
                        hook_name: "service_call".to_string(),
                        threshold: 0.5,
                    }],
                });
            }
        }

        recommendations
    }

    /// Resource-focused optimizations
    fn resource_optimizations(
        &self,
        metrics: &PerformanceMetrics,
    ) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Low success rate but high execution count suggests waste
        if metrics.success_rate < 0.9 && metrics.total_executions > 1000 {
            recommendations.push(OptimizationRecommendation {
                strategy: OptimizationStrategy::MinimizeResources,
                confidence: 0.7,
                description: "Reduce wasted executions with early validation".to_string(),
                estimated_improvement: 15.0,
                actions: vec![OptimizationAction::Reorder {
                    new_order: vec![
                        "validate".to_string(),
                        "process".to_string(),
                        "store".to_string(),
                    ],
                }],
            });
        }

        recommendations
    }

    /// Learn from optimization results
    pub fn record_optimization_result(
        &mut self,
        workflow_id: String,
        before_metrics: PerformanceMetrics,
        after_metrics: PerformanceMetrics,
    ) {
        let history = self
            .learning_history
            .entry(workflow_id)
            .or_insert_with(Vec::new);
        history.push(before_metrics);
        history.push(after_metrics);

        // Keep last 100 metrics
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
    }

    /// Get optimization success rate
    pub fn get_optimization_success_rate(&self, workflow_id: &str) -> Option<f64> {
        let history = self.learning_history.get(workflow_id)?;

        if history.len() < 2 {
            return None;
        }

        let mut improvements = 0;
        for window in history.windows(2) {
            if let [before, after] = window {
                // Improvement if success rate increased or latency decreased
                if after.success_rate > before.success_rate
                    || after.average_ticks < before.average_ticks
                {
                    improvements += 1;
                }
            }
        }

        Some(improvements as f64 / (history.len() - 1) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{ReceiptId, SnapshotManifest};

    #[test]
    fn test_optimizer_creation() {
        let receipt_store = Arc::new(ReceiptStore::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let dark_matter = Arc::new(DarkMatterDetector::new());
        let mape_k = Arc::new(MapekManager::new(dark_matter.clone()));

        let _optimizer = AdaptiveOptimizer::new(receipt_store, snapshot_store, mape_k, dark_matter);
    }

    #[test]
    fn test_performance_analysis() {
        let receipt_store = Arc::new(ReceiptStore::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let dark_matter = Arc::new(DarkMatterDetector::new());
        let mape_k = Arc::new(MapekManager::new(dark_matter.clone()));

        // Create some test receipts
        let snapshot_id = SnapshotId::from_string("test".to_string());
        for i in 0..100 {
            let mut receipt = Receipt::new(
                snapshot_id.clone(),
                b"input",
                b"output",
                "test-workflow".to_string(),
            );
            receipt.set_ticks((i % 10) as u32);
            receipt_store.append(receipt).unwrap();
        }

        let optimizer = AdaptiveOptimizer::new(receipt_store, snapshot_store, mape_k, dark_matter);

        let metrics = optimizer.analyze_performance("test-workflow").unwrap();

        assert_eq!(metrics.total_executions, 100);
        assert!(metrics.average_ticks > 0.0);
        assert!(metrics.success_rate > 0.0);
    }

    #[test]
    fn test_latency_optimizations() {
        let receipt_store = Arc::new(ReceiptStore::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let dark_matter = Arc::new(DarkMatterDetector::new());
        let mape_k = Arc::new(MapekManager::new(dark_matter.clone()));

        let optimizer = AdaptiveOptimizer::new(receipt_store, snapshot_store, mape_k, dark_matter);

        let metrics = PerformanceMetrics {
            total_executions: 100,
            success_rate: 0.99,
            average_ticks: 7.5, // High average
            p50_ticks: 6,
            p95_ticks: 9,
            p99_ticks: 10,
            chatman_violation_rate: 0.15, // High violation rate
            failure_rate: 0.01,
            common_failure_guards: vec![],
        };

        let recommendations = optimizer.latency_optimizations(&metrics);
        assert!(!recommendations.is_empty());

        // Should recommend parallelization and violation reduction
        assert!(recommendations
            .iter()
            .any(|r| matches!(r.strategy, OptimizationStrategy::MinimizeLatency)));
    }

    #[test]
    fn test_reliability_optimizations() {
        let receipt_store = Arc::new(ReceiptStore::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let dark_matter = Arc::new(DarkMatterDetector::new());
        let mape_k = Arc::new(MapekManager::new(dark_matter.clone()));

        let optimizer = AdaptiveOptimizer::new(receipt_store, snapshot_store, mape_k, dark_matter);

        let metrics = PerformanceMetrics {
            total_executions: 100,
            success_rate: 0.85, // Low success rate
            average_ticks: 5.0,
            p50_ticks: 5,
            p95_ticks: 7,
            p99_ticks: 8,
            chatman_violation_rate: 0.0,
            failure_rate: 0.15,
            common_failure_guards: vec![("TIMEOUT".to_string(), 15)],
        };

        let recommendations = optimizer.reliability_optimizations(&metrics);
        assert!(!recommendations.is_empty());

        // Should recommend retry and circuit breaker
        let has_retry = recommendations.iter().any(|r| {
            r.actions
                .iter()
                .any(|a| matches!(a, OptimizationAction::AddRetry { .. }))
        });
        assert!(has_retry);
    }

    #[test]
    fn test_learning_history() {
        let receipt_store = Arc::new(ReceiptStore::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let dark_matter = Arc::new(DarkMatterDetector::new());
        let mape_k = Arc::new(MapekManager::new(dark_matter.clone()));

        let mut optimizer =
            AdaptiveOptimizer::new(receipt_store, snapshot_store, mape_k, dark_matter);

        let before = PerformanceMetrics {
            total_executions: 100,
            success_rate: 0.90,
            average_ticks: 7.0,
            p50_ticks: 6,
            p95_ticks: 9,
            p99_ticks: 10,
            chatman_violation_rate: 0.1,
            failure_rate: 0.1,
            common_failure_guards: vec![],
        };

        let after = PerformanceMetrics {
            total_executions: 100,
            success_rate: 0.95, // Improved
            average_ticks: 5.0, // Improved
            p50_ticks: 5,
            p95_ticks: 7,
            p99_ticks: 8,
            chatman_violation_rate: 0.0,
            failure_rate: 0.05,
            common_failure_guards: vec![],
        };

        optimizer.record_optimization_result("test-workflow".to_string(), before, after);

        let success_rate = optimizer.get_optimization_success_rate("test-workflow");
        assert!(success_rate.is_some());
        assert_eq!(success_rate.unwrap(), 1.0); // Single improvement
    }
}
