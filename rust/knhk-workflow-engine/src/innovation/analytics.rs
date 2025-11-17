//! Workflow Analytics Engine
//!
//! Query and analyze Γ(O) receipt history to derive insights about
//! workflow behavior, performance trends, and optimization opportunities.
//!
//! # Philosophy
//!
//! Data-driven decisions require sophisticated analytics. This engine
//! transforms raw receipts into actionable intelligence.

use std::collections::HashMap;
use std::sync::Arc;

use crate::execution::{Receipt, ReceiptStatistics, ReceiptStore, SnapshotId};
use serde::{Deserialize, Serialize};

/// Time-series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: u64,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// Analytics query builder
pub struct QueryBuilder {
    workflow_id: Option<String>,
    snapshot_id: Option<SnapshotId>,
    time_range: Option<(u64, u64)>,
    success_only: bool,
    failures_only: bool,
    violations_only: bool,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            workflow_id: None,
            snapshot_id: None,
            time_range: None,
            success_only: false,
            failures_only: false,
            violations_only: false,
        }
    }

    pub fn workflow(mut self, id: String) -> Self {
        self.workflow_id = Some(id);
        self
    }

    pub fn snapshot(mut self, id: SnapshotId) -> Self {
        self.snapshot_id = Some(id);
        self
    }

    pub fn time_range(mut self, start: u64, end: u64) -> Self {
        self.time_range = Some((start, end));
        self
    }

    pub fn successes_only(mut self) -> Self {
        self.success_only = true;
        self
    }

    pub fn failures_only(mut self) -> Self {
        self.failures_only = true;
        self
    }

    pub fn violations_only(mut self) -> Self {
        self.violations_only = true;
        self
    }

    pub fn execute(&self, store: &ReceiptStore) -> Result<Vec<Receipt>, String> {
        let mut receipts = if let Some(ref workflow_id) = self.workflow_id {
            store.get_by_workflow(workflow_id)?
        } else if let Some(ref snapshot_id) = self.snapshot_id {
            store.get_by_snapshot(snapshot_id)?
        } else {
            store.get_all()?
        };

        // Apply filters
        if let Some((start, end)) = self.time_range {
            receipts.retain(|r| r.timestamp >= start && r.timestamp <= end);
        }

        if self.success_only {
            receipts.retain(|r| r.success);
        }

        if self.failures_only {
            receipts.retain(|r| !r.success);
        }

        if self.violations_only {
            receipts.retain(|r| r.ticks_used > 8);
        }

        Ok(receipts)
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Analytics engine
pub struct AnalyticsEngine {
    receipt_store: Arc<ReceiptStore>,
}

impl AnalyticsEngine {
    pub fn new(receipt_store: Arc<ReceiptStore>) -> Self {
        Self { receipt_store }
    }

    /// Create a query builder
    pub fn query(&self) -> QueryBuilder {
        QueryBuilder::new()
    }

    /// Get latency time-series
    pub fn latency_timeseries(&self, workflow_id: &str) -> Result<Vec<DataPoint>, String> {
        let receipts = self.receipt_store.get_by_workflow(workflow_id)?;

        Ok(receipts
            .into_iter()
            .map(|r| DataPoint {
                timestamp: r.timestamp,
                value: r.ticks_used as f64,
                metadata: [
                    ("workflow_id".to_string(), r.workflow_instance_id),
                    ("success".to_string(), r.success.to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            })
            .collect())
    }

    /// Get success rate time-series (windowed)
    pub fn success_rate_timeseries(
        &self,
        workflow_id: &str,
        window_size: usize,
    ) -> Result<Vec<DataPoint>, String> {
        let receipts = self.receipt_store.get_by_workflow(workflow_id)?;

        let mut timeseries = Vec::new();

        for window in receipts.windows(window_size) {
            let successes = window.iter().filter(|r| r.success).count();
            let rate = successes as f64 / window_size as f64;

            // Use middle timestamp of window
            let mid_timestamp = window[window_size / 2].timestamp;

            timeseries.push(DataPoint {
                timestamp: mid_timestamp,
                value: rate,
                metadata: [("window_size".to_string(), window_size.to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            });
        }

        Ok(timeseries)
    }

    /// Detect anomalies in latency
    pub fn detect_latency_anomalies(
        &self,
        workflow_id: &str,
        threshold_stddev: f64,
    ) -> Result<Vec<Receipt>, String> {
        let receipts = self.receipt_store.get_by_workflow(workflow_id)?;

        if receipts.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate mean and stddev
        let latencies: Vec<f64> = receipts.iter().map(|r| r.ticks_used as f64).collect();
        let mean = latencies.iter().sum::<f64>() / latencies.len() as f64;

        let variance =
            latencies.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / latencies.len() as f64;
        let stddev = variance.sqrt();

        // Find anomalies (values beyond threshold * stddev from mean)
        let anomalies = receipts
            .into_iter()
            .filter(|r| {
                let deviation = (r.ticks_used as f64 - mean).abs();
                deviation > threshold_stddev * stddev
            })
            .collect();

        Ok(anomalies)
    }

    /// Analyze guard failures
    pub fn analyze_guard_failures(
        &self,
        workflow_id: &str,
    ) -> Result<GuardFailureAnalysis, String> {
        let receipts = self.receipt_store.get_by_workflow(workflow_id)?;

        let mut guard_counts: HashMap<String, usize> = HashMap::new();
        let mut guard_timeseries: HashMap<String, Vec<DataPoint>> = HashMap::new();

        for receipt in &receipts {
            for guard in &receipt.guards_failed {
                *guard_counts.entry(guard.clone()).or_insert(0) += 1;

                guard_timeseries
                    .entry(guard.clone())
                    .or_insert_with(Vec::new)
                    .push(DataPoint {
                        timestamp: receipt.timestamp,
                        value: 1.0,
                        metadata: HashMap::new(),
                    });
            }
        }

        let mut ranked_failures: Vec<(String, usize)> = guard_counts.into_iter().collect();
        ranked_failures.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(GuardFailureAnalysis {
            total_failures: receipts.iter().filter(|r| !r.success).count(),
            unique_guards: ranked_failures.len(),
            top_failures: ranked_failures.into_iter().take(10).collect(),
            failure_timeseries: guard_timeseries,
        })
    }

    /// Compare snapshots
    pub fn compare_snapshots(
        &self,
        snapshot_a: &SnapshotId,
        snapshot_b: &SnapshotId,
    ) -> Result<SnapshotComparison, String> {
        let receipts_a = self.receipt_store.get_by_snapshot(snapshot_a)?;
        let receipts_b = self.receipt_store.get_by_snapshot(snapshot_b)?;

        let metrics_a = Self::calculate_snapshot_metrics(&receipts_a);
        let metrics_b = Self::calculate_snapshot_metrics(&receipts_b);

        let latency_improvement = if metrics_a.avg_latency > 0.0 {
            ((metrics_a.avg_latency - metrics_b.avg_latency) / metrics_a.avg_latency) * 100.0
        } else {
            0.0
        };

        let success_rate_change = (metrics_b.success_rate - metrics_a.success_rate) * 100.0;

        Ok(SnapshotComparison {
            snapshot_a: snapshot_a.clone(),
            snapshot_b: snapshot_b.clone(),
            metrics_a,
            metrics_b,
            latency_improvement_percent: latency_improvement,
            success_rate_change_percent: success_rate_change,
            sample_size_a: receipts_a.len(),
            sample_size_b: receipts_b.len(),
        })
    }

    /// Calculate snapshot metrics
    fn calculate_snapshot_metrics(receipts: &[Receipt]) -> SnapshotMetrics {
        if receipts.is_empty() {
            return SnapshotMetrics {
                avg_latency: 0.0,
                p50_latency: 0,
                p95_latency: 0,
                p99_latency: 0,
                success_rate: 0.0,
                chatman_violation_rate: 0.0,
            };
        }

        let total = receipts.len();
        let successes = receipts.iter().filter(|r| r.success).count();
        let violations = receipts.iter().filter(|r| r.ticks_used > 8).count();

        let mut latencies: Vec<u32> = receipts.iter().map(|r| r.ticks_used).collect();
        latencies.sort_unstable();

        let avg = latencies.iter().sum::<u32>() as f64 / total as f64;
        let p50 = latencies[total / 2];
        let p95 = latencies[(total as f64 * 0.95) as usize];
        let p99 = latencies[(total as f64 * 0.99) as usize];

        SnapshotMetrics {
            avg_latency: avg,
            p50_latency: p50,
            p95_latency: p95,
            p99_latency: p99,
            success_rate: successes as f64 / total as f64,
            chatman_violation_rate: violations as f64 / total as f64,
        }
    }

    /// Generate executive summary
    pub fn executive_summary(&self, workflow_id: &str) -> Result<ExecutiveSummary, String> {
        let receipts = self.receipt_store.get_by_workflow(workflow_id)?;

        if receipts.is_empty() {
            return Err("No data available for workflow".to_string());
        }

        let stats = self.receipt_store.get_statistics()?;
        let anomalies = self.detect_latency_anomalies(workflow_id, 2.0)?;
        let guard_analysis = self.analyze_guard_failures(workflow_id)?;

        Ok(ExecutiveSummary {
            total_executions: stats.total_receipts,
            success_rate: stats.successful_executions as f64 / stats.total_receipts as f64,
            average_latency: stats.average_ticks,
            chatman_compliance: 1.0
                - (stats.chatman_violations as f64 / stats.total_receipts as f64),
            anomaly_count: anomalies.len(),
            top_failure_guard: guard_analysis
                .top_failures
                .first()
                .map(|(name, _)| name.clone()),
            health_score: Self::calculate_health_score(&stats, &anomalies, &guard_analysis),
        })
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(
        stats: &ReceiptStatistics,
        anomalies: &[Receipt],
        guard_analysis: &GuardFailureAnalysis,
    ) -> f64 {
        let success_score =
            (stats.successful_executions as f64 / stats.total_receipts as f64) * 40.0;

        let chatman_score =
            (1.0 - (stats.chatman_violations as f64 / stats.total_receipts as f64)) * 30.0;

        let anomaly_score = if stats.total_receipts > 0 {
            (1.0 - (anomalies.len() as f64 / stats.total_receipts as f64)) * 20.0
        } else {
            0.0
        };

        let failure_diversity_score = if guard_analysis.unique_guards < 5 {
            10.0
        } else {
            5.0
        };

        (success_score + chatman_score + anomaly_score + failure_diversity_score).min(100.0)
    }
}

/// Guard failure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardFailureAnalysis {
    pub total_failures: usize,
    pub unique_guards: usize,
    pub top_failures: Vec<(String, usize)>,
    pub failure_timeseries: HashMap<String, Vec<DataPoint>>,
}

/// Snapshot comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotComparison {
    pub snapshot_a: SnapshotId,
    pub snapshot_b: SnapshotId,
    pub metrics_a: SnapshotMetrics,
    pub metrics_b: SnapshotMetrics,
    pub latency_improvement_percent: f64,
    pub success_rate_change_percent: f64,
    pub sample_size_a: usize,
    pub sample_size_b: usize,
}

/// Snapshot metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetrics {
    pub avg_latency: f64,
    pub p50_latency: u32,
    pub p95_latency: u32,
    pub p99_latency: u32,
    pub success_rate: f64,
    pub chatman_violation_rate: f64,
}

/// Executive summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub total_executions: usize,
    pub success_rate: f64,
    pub average_latency: f64,
    pub chatman_compliance: f64,
    pub anomaly_count: usize,
    pub top_failure_guard: Option<String>,
    pub health_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::Receipt;

    #[test]
    fn test_query_builder() {
        let store = Arc::new(ReceiptStore::new());
        let engine = AnalyticsEngine::new(store.clone());

        // Create test receipts
        for i in 0..10 {
            let snapshot_id = SnapshotId::from_string("test".to_string());
            let mut receipt = Receipt::new(
                snapshot_id,
                b"input",
                b"output",
                "test-workflow".to_string(),
            );
            receipt.set_ticks(i);
            store.append(receipt).unwrap();
        }

        let query = engine.query().workflow("test-workflow".to_string());
        let results = query.execute(&store).unwrap();

        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_latency_timeseries() {
        let store = Arc::new(ReceiptStore::new());
        let engine = AnalyticsEngine::new(store.clone());

        for i in 0..10 {
            let snapshot_id = SnapshotId::from_string("test".to_string());
            let mut receipt = Receipt::new(
                snapshot_id,
                b"input",
                b"output",
                "test-workflow".to_string(),
            );
            receipt.set_ticks(i);
            store.append(receipt).unwrap();
        }

        let timeseries = engine.latency_timeseries("test-workflow").unwrap();
        assert_eq!(timeseries.len(), 10);
    }

    #[test]
    fn test_anomaly_detection() {
        let store = Arc::new(ReceiptStore::new());
        let engine = AnalyticsEngine::new(store.clone());

        // Create receipts with one anomaly
        for i in 0..10 {
            let snapshot_id = SnapshotId::from_string("test".to_string());
            let mut receipt = Receipt::new(
                snapshot_id,
                b"input",
                b"output",
                "test-workflow".to_string(),
            );
            let ticks = if i == 5 { 100 } else { 5 }; // Anomaly at index 5
            receipt.set_ticks(ticks);
            store.append(receipt).unwrap();
        }

        let anomalies = engine
            .detect_latency_anomalies("test-workflow", 2.0)
            .unwrap();
        assert!(!anomalies.is_empty());
    }

    #[test]
    fn test_executive_summary() {
        let store = Arc::new(ReceiptStore::new());
        let engine = AnalyticsEngine::new(store.clone());

        for i in 0..100 {
            let snapshot_id = SnapshotId::from_string("test".to_string());
            let mut receipt = Receipt::new(
                snapshot_id,
                b"input",
                b"output",
                "test-workflow".to_string(),
            );
            receipt.set_ticks((i % 10) as u32);
            store.append(receipt).unwrap();
        }

        let summary = engine.executive_summary("test-workflow").unwrap();
        assert_eq!(summary.total_executions, 100);
        assert!(summary.health_score >= 0.0 && summary.health_score <= 100.0);
    }
}
