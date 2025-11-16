//! Performance Metrics Collection for Self-Executing Workflows
//!
//! Tracks and reports performance metrics across all layers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{DateTime, Utc};

/// Performance metrics collector
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// Record workflow execution
    pub fn record_execution(&self, workflow_id: &str, ticks: u32, success: bool) {
        let mut metrics = self.metrics.write();
        metrics.total_executions += 1;

        if success {
            metrics.successful_executions += 1;
        } else {
            metrics.failed_executions += 1;
        }

        // Update tick statistics
        metrics.total_ticks += ticks as u64;
        metrics.avg_ticks = metrics.total_ticks as f64 / metrics.total_executions as f64;

        if ticks > metrics.max_ticks {
            metrics.max_ticks = ticks;
        }

        if ticks < metrics.min_ticks {
            metrics.min_ticks = ticks;
        }

        // Track Chatman Constant violations
        if ticks > 8 {
            metrics.chatman_violations += 1;
        }

        // Track per-workflow stats
        let workflow_stats = metrics.per_workflow_stats
            .entry(workflow_id.to_string())
            .or_insert_with(WorkflowStats::default);

        workflow_stats.executions += 1;
        workflow_stats.total_ticks += ticks as u64;
        workflow_stats.avg_ticks = workflow_stats.total_ticks as f64 / workflow_stats.executions as f64;
    }

    /// Record MAPE-K cycle
    pub fn record_mape_k_cycle(&self, duration_ms: i64, adaptations: usize) {
        let mut metrics = self.metrics.write();
        metrics.mape_k_cycles += 1;
        metrics.total_mape_k_duration_ms += duration_ms;
        metrics.avg_mape_k_duration_ms =
            metrics.total_mape_k_duration_ms as f64 / metrics.mape_k_cycles as f64;
        metrics.total_adaptations += adaptations;
    }

    /// Record pattern selection
    pub fn record_pattern_selection(&self, pattern: &str, duration_ns: u64) {
        let mut metrics = self.metrics.write();
        let pattern_stats = metrics.pattern_selection_stats
            .entry(pattern.to_string())
            .or_insert_with(PatternSelectionStats::default);

        pattern_stats.selections += 1;
        pattern_stats.total_duration_ns += duration_ns;
        pattern_stats.avg_duration_ns =
            pattern_stats.total_duration_ns as f64 / pattern_stats.selections as f64;
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().clone()
    }

    /// Get metrics report as JSON
    pub fn get_report(&self) -> serde_json::Value {
        let metrics = self.metrics.read();
        serde_json::to_value(&*metrics).unwrap_or_else(|_| serde_json::Value::Null)
    }

    /// Reset all metrics
    pub fn reset(&self) {
        let mut metrics = self.metrics.write();
        *metrics = PerformanceMetrics::default();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total workflow executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Total ticks used
    pub total_ticks: u64,
    /// Average ticks per execution
    pub avg_ticks: f64,
    /// Maximum ticks in single execution
    pub max_ticks: u32,
    /// Minimum ticks in single execution
    pub min_ticks: u32,
    /// Chatman Constant violations (>8 ticks)
    pub chatman_violations: u64,
    /// MAPE-K cycles run
    pub mape_k_cycles: u64,
    /// Total MAPE-K duration (ms)
    pub total_mape_k_duration_ms: i64,
    /// Average MAPE-K duration (ms)
    pub avg_mape_k_duration_ms: f64,
    /// Total adaptations applied
    pub total_adaptations: usize,
    /// Per-workflow statistics
    pub per_workflow_stats: HashMap<String, WorkflowStats>,
    /// Pattern selection statistics
    pub pattern_selection_stats: HashMap<String, PatternSelectionStats>,
    /// Collection start time
    pub collection_start: DateTime<Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_ticks: 0,
            avg_ticks: 0.0,
            max_ticks: 0,
            min_ticks: u32::MAX,
            chatman_violations: 0,
            mape_k_cycles: 0,
            total_mape_k_duration_ms: 0,
            avg_mape_k_duration_ms: 0.0,
            total_adaptations: 0,
            per_workflow_stats: HashMap::new(),
            pattern_selection_stats: HashMap::new(),
            collection_start: Utc::now(),
        }
    }
}

/// Per-workflow statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowStats {
    pub executions: u64,
    pub total_ticks: u64,
    pub avg_ticks: f64,
}

/// Pattern selection statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternSelectionStats {
    pub selections: u64,
    pub total_duration_ns: u64,
    pub avg_duration_ns: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        collector.record_execution("workflow-1", 5, true);
        collector.record_execution("workflow-1", 7, true);
        collector.record_execution("workflow-2", 3, true);

        let metrics = collector.get_metrics();

        assert_eq!(metrics.total_executions, 3);
        assert_eq!(metrics.successful_executions, 3);
        assert_eq!(metrics.failed_executions, 0);
        assert_eq!(metrics.total_ticks, 15);
        assert_eq!(metrics.avg_ticks, 5.0);
        assert_eq!(metrics.chatman_violations, 0);
    }

    #[test]
    fn test_chatman_violations() {
        let collector = MetricsCollector::new();

        collector.record_execution("workflow-1", 9, true); // Violation
        collector.record_execution("workflow-1", 10, true); // Violation
        collector.record_execution("workflow-1", 5, true); // OK

        let metrics = collector.get_metrics();
        assert_eq!(metrics.chatman_violations, 2);
    }

    #[test]
    fn test_mape_k_tracking() {
        let collector = MetricsCollector::new();

        collector.record_mape_k_cycle(1000, 2);
        collector.record_mape_k_cycle(1500, 3);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.mape_k_cycles, 2);
        assert_eq!(metrics.total_mape_k_duration_ms, 2500);
        assert_eq!(metrics.avg_mape_k_duration_ms, 1250.0);
        assert_eq!(metrics.total_adaptations, 5);
    }
}
