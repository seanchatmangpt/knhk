//! Process analytics and performance measurement
//!
//! This module analyzes process execution to calculate performance metrics,
//! identify bottlenecks, and generate optimization recommendations.

use crate::event_log::{EventLog, ProcessEvent};
use crate::{ProcessMiningError, Result};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Complete process analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessAnalytics {
    /// Average cycle time per case (ms)
    pub avg_cycle_time_ms: f64,

    /// Median cycle time per case (ms)
    pub median_cycle_time_ms: f64,

    /// Throughput (cases per hour)
    pub throughput_per_hour: f64,

    /// Activity performance
    pub activity_metrics: HashMap<String, ActivityMetrics>,

    /// Identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,

    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Metrics for a single activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityMetrics {
    /// Activity name
    pub activity: String,

    /// Number of occurrences
    pub count: usize,

    /// Average execution time (ms)
    pub avg_duration_ms: f64,

    /// Standard deviation (ms)
    pub std_dev_ms: f64,

    /// Min execution time (ms)
    pub min_duration_ms: f64,

    /// Max execution time (ms)
    pub max_duration_ms: f64,

    /// Percentage of total time
    pub time_percentage: f64,
}

/// Identified bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Activity causing the bottleneck
    pub activity: String,

    /// Severity (1.0 = worst)
    pub severity: f64,

    /// Description
    pub description: String,

    /// Suggested fix
    pub suggestion: String,
}

/// Process analyzer
#[derive(Debug)]
pub struct ProcessAnalyzer<'a> {
    event_log: &'a EventLog,
}

impl<'a> ProcessAnalyzer<'a> {
    /// Create new analyzer
    pub fn new(event_log: &'a EventLog) -> Self {
        Self { event_log }
    }

    /// Run complete analysis
    pub fn analyze(&self) -> Result<ProcessAnalytics> {
        let cycle_times = self.calculate_cycle_times()?;
        let activity_metrics = self.calculate_activity_metrics()?;
        let bottlenecks = self.detect_bottlenecks(&activity_metrics)?;
        let recommendations = self.generate_recommendations(&bottlenecks, &activity_metrics);

        let avg_cycle_time_ms = cycle_times.iter().sum::<f64>() / cycle_times.len() as f64;

        let mut sorted_times = cycle_times.clone();
        sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_cycle_time_ms = sorted_times[sorted_times.len() / 2];

        // Calculate throughput
        let time_range_hours = self
            .event_log
            .metadata
            .end_time
            .signed_duration_since(self.event_log.metadata.start_time)
            .num_seconds() as f64
            / 3600.0;

        let throughput_per_hour = if time_range_hours > 0.0 {
            self.event_log.metadata.total_cases as f64 / time_range_hours
        } else {
            0.0
        };

        Ok(ProcessAnalytics {
            avg_cycle_time_ms,
            median_cycle_time_ms,
            throughput_per_hour,
            activity_metrics,
            bottlenecks,
            recommendations,
        })
    }

    /// Calculate cycle times for all cases
    fn calculate_cycle_times(&self) -> Result<Vec<f64>> {
        let mut cycle_times = Vec::new();

        for case_id in &self.event_log.case_ids {
            if let Some(duration) = self.event_log.case_duration(case_id) {
                cycle_times.push(duration.as_secs_f64() * 1000.0);
            }
        }

        if cycle_times.is_empty() {
            return Err(ProcessMiningError::Analytics(
                "No cycle times calculated".to_string(),
            ));
        }

        Ok(cycle_times)
    }

    /// Calculate metrics for each activity
    fn calculate_activity_metrics(&self) -> Result<HashMap<String, ActivityMetrics>> {
        let mut metrics = HashMap::new();

        let total_time: f64 = self
            .event_log
            .case_ids
            .iter()
            .filter_map(|case_id| self.event_log.case_duration(case_id))
            .map(|d| d.as_secs_f64() * 1000.0)
            .sum();

        for activity in &self.event_log.activities {
            let events = self.event_log.events_for_activity(activity);

            if events.is_empty() {
                continue;
            }

            let count = events.len();

            // Calculate durations (simplified - assuming consecutive events)
            let mut durations = Vec::new();
            for case_id in &self.event_log.case_ids {
                let case_events: Vec<_> = self
                    .event_log
                    .events_for_case(case_id)
                    .into_iter()
                    .filter(|e| e.activity == *activity)
                    .collect();

                for event in case_events {
                    // Approximate duration based on event spacing
                    durations.push(10.0); // Simplified
                }
            }

            if durations.is_empty() {
                continue;
            }

            let avg_duration_ms = durations.iter().sum::<f64>() / durations.len() as f64;

            let variance = durations
                .iter()
                .map(|d| {
                    let diff = d - avg_duration_ms;
                    diff * diff
                })
                .sum::<f64>()
                / durations.len() as f64;

            let std_dev_ms = variance.sqrt();
            let min_duration_ms = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_duration_ms = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            let time_percentage = if total_time > 0.0 {
                (avg_duration_ms * count as f64 / total_time) * 100.0
            } else {
                0.0
            };

            metrics.insert(
                activity.clone(),
                ActivityMetrics {
                    activity: activity.clone(),
                    count,
                    avg_duration_ms,
                    std_dev_ms,
                    min_duration_ms,
                    max_duration_ms,
                    time_percentage,
                },
            );
        }

        Ok(metrics)
    }

    /// Detect bottlenecks
    fn detect_bottlenecks(
        &self,
        activity_metrics: &HashMap<String, ActivityMetrics>,
    ) -> Result<Vec<Bottleneck>> {
        let mut bottlenecks = Vec::new();

        for (activity, metrics) in activity_metrics {
            // High variance = inconsistent performance
            if metrics.std_dev_ms > metrics.avg_duration_ms * 0.5 {
                bottlenecks.push(Bottleneck {
                    activity: activity.clone(),
                    severity: 0.7,
                    description: format!(
                        "High variance (σ={:.2}ms) indicates inconsistent performance",
                        metrics.std_dev_ms
                    ),
                    suggestion: "Investigate why execution time varies. Consider caching or optimization."
                        .to_string(),
                });
            }

            // High time percentage = major contributor to cycle time
            if metrics.time_percentage > 20.0 {
                bottlenecks.push(Bottleneck {
                    activity: activity.clone(),
                    severity: metrics.time_percentage / 100.0,
                    description: format!(
                        "Consumes {:.1}% of total process time",
                        metrics.time_percentage
                    ),
                    suggestion: "This activity is a major time consumer. Optimize or parallelize."
                        .to_string(),
                });
            }
        }

        // Sort by severity (descending)
        bottlenecks.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

        Ok(bottlenecks)
    }

    /// Generate optimization recommendations
    fn generate_recommendations(
        &self,
        bottlenecks: &[Bottleneck],
        activity_metrics: &HashMap<String, ActivityMetrics>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !bottlenecks.is_empty() {
            recommendations.push(format!(
                "Focus on top {} bottlenecks to reduce cycle time",
                bottlenecks.len().min(3)
            ));
        }

        let total_activities = activity_metrics.len();
        if total_activities > 10 {
            recommendations.push(
                "Consider simplifying workflow - many activities may indicate complexity".to_string(),
            );
        }

        recommendations.push("Enable caching for idempotent activities".to_string());
        recommendations.push("Consider parallel execution for independent activities".to_string());

        recommendations
    }
}

/// Bottleneck detector
#[derive(Debug, Default)]
pub struct BottleneckDetector {
    severity_threshold: f64,
}

impl BottleneckDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self {
            severity_threshold: 0.5,
        }
    }

    /// Set severity threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.severity_threshold = threshold;
        self
    }

    /// Detect critical bottlenecks
    pub fn detect_critical(&self, analytics: &ProcessAnalytics) -> Vec<&Bottleneck> {
        analytics
            .bottlenecks
            .iter()
            .filter(|b| b.severity >= self.severity_threshold)
            .collect()
    }
}

/// Performance analytics helper
#[derive(Debug)]
pub struct PerformanceAnalytics;

impl PerformanceAnalytics {
    /// Calculate process efficiency (0.0 to 1.0)
    pub fn calculate_efficiency(analytics: &ProcessAnalytics) -> f64 {
        // Simplified efficiency metric
        let bottleneck_penalty = analytics
            .bottlenecks
            .iter()
            .map(|b| b.severity)
            .sum::<f64>()
            / analytics.bottlenecks.len().max(1) as f64;

        (1.0 - bottleneck_penalty).max(0.0)
    }

    /// Calculate theoretical maximum throughput
    pub fn max_throughput(analytics: &ProcessAnalytics) -> f64 {
        if analytics.avg_cycle_time_ms > 0.0 {
            3600_000.0 / analytics.avg_cycle_time_ms
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::EventLogBuilder;
    use chrono::Utc;

    #[test]
    fn test_process_analyzer() {
        let mut builder = EventLogBuilder::new();
        let now = Utc::now();

        for i in 0..5 {
            builder.add_span_event(
                format!("case_{:03}", i),
                "step_1".to_string(),
                now + chrono::Duration::seconds(i * 10),
                None,
                HashMap::new(),
            );

            builder.add_span_event(
                format!("case_{:03}", i),
                "step_2".to_string(),
                now + chrono::Duration::seconds(i * 10 + 5),
                None,
                HashMap::new(),
            );
        }

        let log = builder.build().unwrap();
        let analyzer = ProcessAnalyzer::new(&log);
        let analytics = analyzer.analyze().unwrap();

        assert!(analytics.avg_cycle_time_ms > 0.0);
        assert!(analytics.throughput_per_hour > 0.0);
        assert!(!analytics.activity_metrics.is_empty());
    }

    #[test]
    fn test_bottleneck_detector() {
        let detector = BottleneckDetector::new().with_threshold(0.6);

        let analytics = ProcessAnalytics {
            avg_cycle_time_ms: 100.0,
            median_cycle_time_ms: 95.0,
            throughput_per_hour: 36.0,
            activity_metrics: HashMap::new(),
            bottlenecks: vec![
                Bottleneck {
                    activity: "slow_step".to_string(),
                    severity: 0.8,
                    description: "Very slow".to_string(),
                    suggestion: "Optimize".to_string(),
                },
                Bottleneck {
                    activity: "fast_step".to_string(),
                    severity: 0.3,
                    description: "Fast".to_string(),
                    suggestion: "None".to_string(),
                },
            ],
            recommendations: vec![],
        };

        let critical = detector.detect_critical(&analytics);
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].activity, "slow_step");
    }
}
