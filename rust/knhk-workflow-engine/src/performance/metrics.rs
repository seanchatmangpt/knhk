//! Performance metrics collection
//!
//! Provides performance metrics collection and analysis for workflow operations.

// Metrics module - HashMap will be used when implementing metrics aggregation
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Performance metric
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    /// Operation name
    pub operation: String,
    /// Latency in nanoseconds
    pub latency_ns: u64,
    /// Ticks consumed
    pub ticks: u32,
    /// Success flag
    pub success: bool,
    /// Timestamp
    pub timestamp: Instant,
}

/// Performance metrics collector
pub struct PerformanceMetrics {
    metrics: Arc<RwLock<Vec<PerformanceMetric>>>,
    max_samples: usize,
}

impl PerformanceMetrics {
    /// Create new metrics collector
    pub fn new(max_samples: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            max_samples,
        }
    }

    /// Record metric
    pub async fn record(&self, metric: PerformanceMetric) {
        let mut metrics = self.metrics.write().await;
        metrics.push(metric);
        if metrics.len() > self.max_samples {
            metrics.remove(0);
        }
    }

    /// Get statistics for operation
    pub async fn get_stats(&self, operation: &str) -> Option<OperationStats> {
        let metrics = self.metrics.read().await;
        let operation_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.operation == operation)
            .collect();

        if operation_metrics.is_empty() {
            return None;
        }

        let latencies: Vec<u64> = operation_metrics.iter().map(|m| m.latency_ns).collect();
        let ticks: Vec<u32> = operation_metrics.iter().map(|m| m.ticks).collect();

        let avg_latency = latencies.iter().sum::<u64>() / latencies.len() as u64;
        let p50_latency = Self::percentile(&latencies, 0.50);
        let p99_latency = Self::percentile(&latencies, 0.99);
        let max_latency = *latencies.iter().max().unwrap_or(&0);

        let avg_ticks = ticks.iter().sum::<u32>() / ticks.len() as u32;
        let max_ticks = *ticks.iter().max().unwrap_or(&0);

        let success_count = operation_metrics.iter().filter(|m| m.success).count();
        let success_rate = success_count as f64 / operation_metrics.len() as f64;

        Some(OperationStats {
            operation: operation.to_string(),
            count: operation_metrics.len(),
            avg_latency_ns: avg_latency,
            p50_latency_ns: p50_latency,
            p99_latency_ns: p99_latency,
            max_latency_ns: max_latency,
            avg_ticks,
            max_ticks,
            success_rate,
        })
    }

    /// Calculate percentile
    fn percentile(values: &[u64], percentile: f64) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let index = ((sorted.len() as f64) * percentile).ceil() as usize - 1;
        sorted
            .get(index.min(sorted.len() - 1))
            .copied()
            .unwrap_or(0)
    }

    /// Get all operation names
    pub async fn get_operations(&self) -> Vec<String> {
        let metrics = self.metrics.read().await;
        let mut operations: Vec<String> = metrics.iter().map(|m| m.operation.clone()).collect();
        operations.sort_unstable();
        operations.dedup();
        operations
    }
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStats {
    /// Operation name
    pub operation: String,
    /// Sample count
    pub count: usize,
    /// Average latency (nanoseconds)
    pub avg_latency_ns: u64,
    /// P50 latency (nanoseconds)
    pub p50_latency_ns: u64,
    /// P99 latency (nanoseconds)
    pub p99_latency_ns: u64,
    /// Maximum latency (nanoseconds)
    pub max_latency_ns: u64,
    /// Average ticks
    pub avg_ticks: u32,
    /// Maximum ticks
    pub max_ticks: u32,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
}
