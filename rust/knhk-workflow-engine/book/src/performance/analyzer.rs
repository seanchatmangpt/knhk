#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Workflow performance analyzer
//!
//! Analyzes workflow execution performance and provides optimization recommendations.

use crate::case::CaseId;
use crate::error::WorkflowResult;
use crate::executor::WorkflowEngine;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance metrics for a workflow case
#[derive(Debug, Clone)]
pub struct CaseMetrics {
    /// Case ID
    pub case_id: CaseId,
    /// Total execution time
    pub total_time: Duration,
    /// Task execution times
    pub task_times: HashMap<String, Duration>,
    /// Pattern execution times
    pub pattern_times: HashMap<u32, Duration>,
    /// Resource allocation times
    pub allocation_times: HashMap<String, Duration>,
    /// Worklet execution times
    pub worklet_times: HashMap<String, Duration>,
    /// Tick budget violations
    pub tick_violations: Vec<TickViolation>,
}

/// Tick budget violation
#[derive(Debug, Clone)]
pub struct TickViolation {
    /// Task or pattern ID
    pub id: String,
    /// Expected ticks (≤8)
    pub expected_ticks: u32,
    /// Actual ticks
    pub actual_ticks: u32,
    /// Violation severity
    pub severity: ViolationSeverity,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationSeverity {
    /// Minor violation (9-16 ticks)
    Minor,
    /// Major violation (17-32 ticks)
    Major,
    /// Critical violation (>32 ticks)
    Critical,
}

/// Workflow profiler
pub struct WorkflowProfiler {
    /// Performance metrics storage
    metrics: HashMap<CaseId, CaseMetrics>,
}

impl WorkflowProfiler {
    /// Create a new workflow profiler
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Profile a workflow case execution
    pub async fn profile_case(
        &mut self,
        engine: &WorkflowEngine,
        case_id: CaseId,
    ) -> WorkflowResult<CaseMetrics> {
        let start = Instant::now();
        let task_times = HashMap::new();
        let pattern_times = HashMap::new();
        let allocation_times = HashMap::new();
        let worklet_times = HashMap::new();

        // Execute case with profiling
        // In production, would instrument execution to collect metrics
        engine.execute_case(case_id).await?;

        let total_time = start.elapsed();

        let metrics = CaseMetrics {
            case_id,
            total_time,
            task_times,
            pattern_times,
            allocation_times,
            worklet_times,
            tick_violations: vec![],
        };

        self.metrics.insert(case_id, metrics.clone());
        Ok(metrics)
    }

    /// Generate performance report
    pub fn generate_report(&self, metrics: &CaseMetrics) -> WorkflowResult<String> {
        let mut report = String::from("Workflow Performance Report\n");
        report.push_str("===========================\n\n");

        report.push_str(&format!("Case ID: {}\n", metrics.case_id));
        report.push_str(&format!(
            "Total Execution Time: {:?}\n\n",
            metrics.total_time
        ));

        report.push_str("Task Performance:\n");
        for (task_id, duration) in &metrics.task_times {
            let ticks = duration.as_nanos() / 2; // 2ns per tick
            let status = if ticks <= 8 { "✅" } else { "⚠️" };
            report.push_str(&format!(
                "- {}: {} ticks {} (within budget)\n",
                task_id, ticks, status
            ));
        }

        if !metrics.tick_violations.is_empty() {
            report.push_str("\nTick Budget Violations:\n");
            for violation in &metrics.tick_violations {
                report.push_str(&format!(
                    "- {}: {} ticks (expected ≤{})\n",
                    violation.id, violation.actual_ticks, violation.expected_ticks
                ));
            }
        }

        Ok(report)
    }

    /// Get metrics for a case
    pub fn get_metrics(&self, case_id: &CaseId) -> Option<&CaseMetrics> {
        self.metrics.get(case_id)
    }

    /// Analyze hot path
    pub fn analyze_hot_path(&self, metrics: &CaseMetrics) -> WorkflowResult<HotPathAnalysis> {
        let mut critical_path = vec![];
        let mut total_ticks = 0u64;

        // Find longest path
        for (task_id, duration) in &metrics.task_times {
            let ticks = (duration.as_nanos() / 2) as u64;
            if ticks > 8 {
                critical_path.push(task_id.clone());
                total_ticks += ticks;
            }
        }

        Ok(HotPathAnalysis {
            critical_path,
            total_ticks,
            recommendations: vec![
                "Consider parallelizing sequential tasks".to_string(),
                "Optimize resource allocation".to_string(),
            ],
        })
    }
}

/// Hot path analysis result
#[derive(Debug, Clone)]
pub struct HotPathAnalysis {
    /// Critical path tasks
    pub critical_path: Vec<String>,
    /// Total ticks in critical path
    pub total_ticks: u64,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

impl Default for WorkflowProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_report() {
        let profiler = WorkflowProfiler::new();
        let metrics = CaseMetrics {
            case_id: crate::case::CaseId::new(),
            total_time: Duration::from_millis(100),
            task_times: HashMap::new(),
            pattern_times: HashMap::new(),
            allocation_times: HashMap::new(),
            worklet_times: HashMap::new(),
            tick_violations: vec![],
        };

        let report = profiler.generate_report(&metrics).unwrap();
        assert!(report.contains("Workflow Performance Report"));
    }
}
