//! Process Mining Workflow Analysis Example
//!
//! Demonstrates how to extract event logs from workflow execution,
//! analyze the process with process mining, identify bottlenecks,
//! and generate actionable recommendations.
//!
//! This example shows:
//! - Extracting event logs from telemetry spans
//! - Building event log from workflow execution
//! - Running process analytics
//! - Identifying performance bottlenecks
//! - Generating optimization recommendations
//!
//! Run: `cargo run --example process-mining-workflow-analysis`

use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use std::time::Instant;

// Simplified imports (in real code, use knhk-process-mining crate)
use serde::{Deserialize, Serialize};

// ============================================================================
// Workflow Execution Simulator
// ============================================================================

#[derive(Debug, Clone)]
struct WorkflowExecution {
    workflow_id: String,
    steps: Vec<ExecutionStep>,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
struct ExecutionStep {
    step_name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_ms: u64,
    resource: String,
    attributes: HashMap<String, String>,
}

impl WorkflowExecution {
    fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            steps: Vec::new(),
            start_time: Utc::now(),
            end_time: None,
        }
    }

    fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    fn complete(&mut self) {
        self.end_time = Some(Utc::now());
    }

    fn total_duration_ms(&self) -> u64 {
        self.steps.iter().map(|s| s.duration_ms).sum()
    }
}

// ============================================================================
// Event Log Representation
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessEvent {
    case_id: String,
    activity: String,
    timestamp: DateTime<Utc>,
    resource: Option<String>,
    attributes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct EventLog {
    events: Vec<ProcessEvent>,
}

impl EventLog {
    fn from_workflow(execution: &WorkflowExecution) -> Self {
        let mut events = Vec::new();

        for step in &execution.steps {
            events.push(ProcessEvent {
                case_id: execution.workflow_id.clone(),
                activity: step.step_name.clone(),
                timestamp: step.start_time,
                resource: Some(step.resource.clone()),
                attributes: step.attributes.clone(),
            });
        }

        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);

        Self { events }
    }

    fn merge(logs: Vec<EventLog>) -> Self {
        let mut all_events = Vec::new();

        for log in logs {
            all_events.extend(log.events);
        }

        all_events.sort_by_key(|e| e.timestamp);

        Self { events: all_events }
    }

    fn unique_cases(&self) -> Vec<String> {
        let mut cases: Vec<_> = self
            .events
            .iter()
            .map(|e| e.case_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cases.sort();
        cases
    }

    fn unique_activities(&self) -> Vec<String> {
        let mut activities: Vec<_> = self
            .events
            .iter()
            .map(|e| e.activity.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        activities.sort();
        activities
    }

    fn events_for_case(&self, case_id: &str) -> Vec<&ProcessEvent> {
        self.events.iter().filter(|e| e.case_id == case_id).collect()
    }
}

// ============================================================================
// Process Analytics
// ============================================================================

#[derive(Debug, Clone)]
struct ActivityMetrics {
    activity: String,
    count: usize,
    total_duration_ms: u64,
    avg_duration_ms: f64,
    min_duration_ms: u64,
    max_duration_ms: u64,
    time_percentage: f64,
}

#[derive(Debug, Clone)]
struct Bottleneck {
    activity: String,
    severity: f64,
    description: String,
    recommendation: String,
}

struct ProcessAnalytics {
    total_cases: usize,
    total_events: usize,
    avg_cycle_time_ms: f64,
    median_cycle_time_ms: f64,
    throughput_per_hour: f64,
    activity_metrics: HashMap<String, ActivityMetrics>,
    bottlenecks: Vec<Bottleneck>,
}

impl ProcessAnalytics {
    fn analyze(executions: &[WorkflowExecution]) -> Self {
        let total_cases = executions.len();

        // Calculate cycle times
        let mut cycle_times: Vec<f64> = executions
            .iter()
            .map(|e| e.total_duration_ms() as f64)
            .collect();

        let avg_cycle_time_ms = if !cycle_times.is_empty() {
            cycle_times.iter().sum::<f64>() / cycle_times.len() as f64
        } else {
            0.0
        };

        cycle_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median_cycle_time_ms = if !cycle_times.is_empty() {
            cycle_times[cycle_times.len() / 2]
        } else {
            0.0
        };

        // Calculate throughput
        let time_span = if let (Some(first), Some(last)) = (executions.first(), executions.last()) {
            if let Some(last_end) = last.end_time {
                last_end
                    .signed_duration_since(first.start_time)
                    .num_seconds() as f64
                    / 3600.0
            } else {
                1.0
            }
        } else {
            1.0
        };

        let throughput_per_hour = if time_span > 0.0 {
            total_cases as f64 / time_span
        } else {
            0.0
        };

        // Calculate activity metrics
        let mut activity_durations: HashMap<String, Vec<u64>> = HashMap::new();
        let total_time: u64 = executions.iter().map(|e| e.total_duration_ms()).sum();

        for execution in executions {
            for step in &execution.steps {
                activity_durations
                    .entry(step.step_name.clone())
                    .or_default()
                    .push(step.duration_ms);
            }
        }

        let mut activity_metrics = HashMap::new();

        for (activity, durations) in &activity_durations {
            let count = durations.len();
            let total_duration_ms: u64 = durations.iter().sum();
            let avg_duration_ms = total_duration_ms as f64 / count as f64;
            let min_duration_ms = *durations.iter().min().unwrap_or(&0);
            let max_duration_ms = *durations.iter().max().unwrap_or(&0);
            let time_percentage = if total_time > 0 {
                (total_duration_ms as f64 / total_time as f64) * 100.0
            } else {
                0.0
            };

            activity_metrics.insert(
                activity.clone(),
                ActivityMetrics {
                    activity: activity.clone(),
                    count,
                    total_duration_ms,
                    avg_duration_ms,
                    min_duration_ms,
                    max_duration_ms,
                    time_percentage,
                },
            );
        }

        // Identify bottlenecks
        let mut bottlenecks = Vec::new();

        for (activity, metrics) in &activity_metrics {
            // High time percentage = bottleneck
            if metrics.time_percentage > 20.0 {
                bottlenecks.push(Bottleneck {
                    activity: activity.clone(),
                    severity: metrics.time_percentage / 100.0,
                    description: format!(
                        "Consumes {:.1}% of total execution time",
                        metrics.time_percentage
                    ),
                    recommendation: "Consider optimizing or parallelizing this activity".to_string(),
                });
            }

            // High variance = inconsistent performance
            let variance_ratio = if metrics.avg_duration_ms > 0.0 {
                (metrics.max_duration_ms as f64 - metrics.min_duration_ms as f64)
                    / metrics.avg_duration_ms
            } else {
                0.0
            };

            if variance_ratio > 1.0 {
                bottlenecks.push(Bottleneck {
                    activity: activity.clone(),
                    severity: variance_ratio.min(1.0),
                    description: format!(
                        "High variance: min={}ms, max={}ms, avg={:.0}ms",
                        metrics.min_duration_ms, metrics.max_duration_ms, metrics.avg_duration_ms
                    ),
                    recommendation: "Investigate inconsistent performance - consider caching or resource allocation".to_string(),
                });
            }
        }

        // Sort bottlenecks by severity
        bottlenecks.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

        let total_events = executions.iter().map(|e| e.steps.len()).sum();

        Self {
            total_cases,
            total_events,
            avg_cycle_time_ms,
            median_cycle_time_ms,
            throughput_per_hour,
            activity_metrics,
            bottlenecks,
        }
    }

    fn print_report(&self) {
        println!("\n=== Process Analytics Report ===\n");

        println!("📊 Overall Metrics:");
        println!("  Total Cases: {}", self.total_cases);
        println!("  Total Events: {}", self.total_events);
        println!("  Avg Cycle Time: {:.2}ms", self.avg_cycle_time_ms);
        println!("  Median Cycle Time: {:.2}ms", self.median_cycle_time_ms);
        println!("  Throughput: {:.2} cases/hour", self.throughput_per_hour);

        println!("\n📈 Activity Performance:");
        let mut sorted_activities: Vec<_> = self.activity_metrics.values().collect();
        sorted_activities.sort_by(|a, b| {
            b.time_percentage
                .partial_cmp(&a.time_percentage)
                .unwrap()
        });

        for metrics in sorted_activities.iter().take(10) {
            println!("\n  Activity: {}", metrics.activity);
            println!("    Count: {}", metrics.count);
            println!("    Avg Duration: {:.2}ms", metrics.avg_duration_ms);
            println!(
                "    Range: {}ms - {}ms",
                metrics.min_duration_ms, metrics.max_duration_ms
            );
            println!("    Time %: {:.1}%", metrics.time_percentage);
        }

        println!("\n🔍 Bottlenecks Identified:");
        if self.bottlenecks.is_empty() {
            println!("  ✅ No significant bottlenecks detected");
        } else {
            for (i, bottleneck) in self.bottlenecks.iter().enumerate() {
                println!("\n  Bottleneck #{}: {}", i + 1, bottleneck.activity);
                println!("    Severity: {:.2}", bottleneck.severity);
                println!("    Description: {}", bottleneck.description);
                println!("    Recommendation: {}", bottleneck.recommendation);
            }
        }
    }
}

// ============================================================================
// Workflow Simulator
// ============================================================================

fn simulate_workflow(workflow_id: &str, include_delays: bool) -> WorkflowExecution {
    let mut execution = WorkflowExecution::new(workflow_id.to_string());
    let mut current_time = Utc::now();

    let steps = vec![
        ("validate_input", 10, "validator"),
        ("fetch_data", 50, "data_service"),
        ("process_data", 100, "processor"),
        ("enrich_data", 80, "enrichment_service"),
        ("save_result", 30, "storage"),
    ];

    for (step_name, base_duration, resource) in steps {
        let duration_ms = if include_delays && step_name == "process_data" {
            base_duration + 200 // Simulate bottleneck
        } else {
            base_duration
        };

        let step_end = current_time + Duration::milliseconds(duration_ms as i64);

        execution.add_step(ExecutionStep {
            step_name: step_name.to_string(),
            start_time: current_time,
            end_time: step_end,
            duration_ms,
            resource: resource.to_string(),
            attributes: HashMap::new(),
        });

        current_time = step_end;
    }

    execution.complete();
    execution
}

// ============================================================================
// Main Example
// ============================================================================

fn main() {
    println!("=== Process Mining Workflow Analysis Example ===\n");

    let start = Instant::now();

    // Simulate multiple workflow executions
    println!("🔄 Simulating workflow executions...");

    let mut executions = Vec::new();

    // Normal executions
    for i in 0..5 {
        executions.push(simulate_workflow(&format!("workflow_{:03}", i), false));
    }

    // Executions with delays (bottleneck)
    for i in 5..8 {
        executions.push(simulate_workflow(&format!("workflow_{:03}", i), true));
    }

    println!("  ✅ Executed {} workflows\n", executions.len());

    // Extract event log
    println!("📋 Extracting event log from executions...");
    let event_logs: Vec<_> = executions.iter().map(EventLog::from_workflow).collect();
    let merged_log = EventLog::merge(event_logs);

    println!("  ✅ Event log extracted:");
    println!("     Cases: {}", merged_log.unique_cases().len());
    println!("     Activities: {}", merged_log.unique_activities().len());
    println!("     Events: {}\n", merged_log.events.len());

    // Analyze process
    println!("🔍 Analyzing process performance...");
    let analytics = ProcessAnalytics::analyze(&executions);
    println!("  ✅ Analysis complete\n");

    // Print detailed report
    analytics.print_report();

    // Recommendations
    println!("\n💡 Optimization Recommendations:\n");

    if analytics.bottlenecks.is_empty() {
        println!("  ✅ Process is well-optimized!");
        println!("  Consider monitoring for future regressions.");
    } else {
        println!("  Focus on the top {} bottleneck(s):", analytics.bottlenecks.len().min(3));

        for (i, bottleneck) in analytics.bottlenecks.iter().take(3).enumerate() {
            println!("  {}. {}: {}", i + 1, bottleneck.activity, bottleneck.recommendation);
        }

        println!("\n  General recommendations:");
        println!("  - Enable caching for idempotent activities");
        println!("  - Consider parallel execution for independent steps");
        println!("  - Monitor resource allocation for high-variance activities");
        println!("  - Set up alerting for throughput degradation");
    }

    println!("\n⏱️  Analysis Time: {:?}", start.elapsed());

    println!("\n=== Key Insights ===\n");
    println!("1. Event Log Extraction:");
    println!("   - Extracted {} events from {} workflow executions", merged_log.events.len(), executions.len());
    println!("   - Event log provides complete audit trail");
    println!("   - Can be exported to XES format for external tools\n");

    println!("2. Performance Analysis:");
    println!("   - Identified {} activities across all workflows", analytics.activity_metrics.len());
    println!("   - Calculated cycle time statistics (avg, median)");
    println!("   - Measured throughput: {:.2} workflows/hour\n", analytics.throughput_per_hour);

    println!("3. Bottleneck Detection:");
    println!("   - Found {} bottlenecks using time % and variance", analytics.bottlenecks.len());
    println!("   - Prioritized by severity for actionable insights");
    println!("   - Provided specific optimization recommendations\n");

    println!("4. Production Integration:");
    println!("   - Extract event logs from OTEL spans");
    println!("   - Run analytics on historical workflow data");
    println!("   - Continuous monitoring and optimization");
    println!("   - Validate against expected process patterns\n");

    println!("=== Next Steps ===\n");
    println!("1. See process-mining-discovery.rs for process structure discovery");
    println!("2. See process-mining-performance-analytics.rs for advanced metrics");
    println!("3. See templates/process-mining-integration-template.rs for integration code");
    println!("4. Read docs/papers/how-to-guides/13-analyze-workflows-with-process-mining.md\n");
}
