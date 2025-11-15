//! Process Mining Integration Template
//!
//! Complete template for integrating process mining with KNHK workflows.
//! Shows telemetry → event log → analysis pipeline with real event handling,
//! metrics calculation, and report generation.
//!
//! This template provides:
//! - OTEL span collection and event log extraction
//! - Process discovery and pattern validation
//! - Performance analytics and bottleneck detection
//! - Automated reporting and optimization recommendations
//!
//! Copy and adapt this template for your workflow analysis needs.

use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// ============================================================================
// 1. Telemetry Integration - Collect Spans
// ============================================================================

/// Represents an OpenTelemetry span (simplified)
#[derive(Debug, Clone)]
struct OtelSpan {
    span_id: String,
    trace_id: String,
    name: String,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    attributes: HashMap<String, String>,
}

/// Telemetry collector - captures OTEL spans from workflow execution
struct TelemetryCollector {
    spans: Arc<Mutex<Vec<OtelSpan>>>,
}

impl TelemetryCollector {
    fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a span from workflow execution
    fn record_span(&self, span: OtelSpan) {
        self.spans.lock().unwrap().push(span);
    }

    /// Get all collected spans
    fn get_spans(&self) -> Vec<OtelSpan> {
        self.spans.lock().unwrap().clone()
    }

    /// Clear all spans
    fn clear(&self) {
        self.spans.lock().unwrap().clear();
    }
}

// ============================================================================
// 2. Event Log Extraction - Convert Spans to Events
// ============================================================================

#[derive(Debug, Clone)]
struct ProcessEvent {
    case_id: String,
    activity: String,
    timestamp: DateTime<Utc>,
    resource: Option<String>,
    duration_ms: i64,
}

struct EventLogExtractor;

impl EventLogExtractor {
    /// Extract event log from OTEL spans
    fn extract_from_spans(spans: &[OtelSpan]) -> EventLog {
        let mut events = Vec::new();

        for span in spans {
            // Extract case_id from trace_id
            let case_id = span.trace_id.clone();

            // Extract resource from attributes
            let resource = span.attributes.get("resource").cloned();

            // Calculate duration
            let duration_ms = span
                .end_time
                .signed_duration_since(span.start_time)
                .num_milliseconds();

            events.push(ProcessEvent {
                case_id,
                activity: span.name.clone(),
                timestamp: span.start_time,
                resource,
                duration_ms,
            });
        }

        // Sort by timestamp
        events.sort_by_key(|e| e.timestamp);

        EventLog { events }
    }
}

#[derive(Debug, Clone)]
struct EventLog {
    events: Vec<ProcessEvent>,
}

impl EventLog {
    fn get_unique_cases(&self) -> Vec<String> {
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

    fn events_for_case(&self, case_id: &str) -> Vec<&ProcessEvent> {
        self.events
            .iter()
            .filter(|e| e.case_id == case_id)
            .collect()
    }
}

// ============================================================================
// 3. Process Discovery - Discover Workflow Structure
// ============================================================================

#[derive(Debug, Clone)]
struct ProcessGraph {
    transitions: HashMap<(String, String), usize>,
    activities: Vec<String>,
}

impl ProcessGraph {
    fn discover(event_log: &EventLog) -> Self {
        let mut transitions = HashMap::new();
        let mut activities = std::collections::HashSet::new();

        for case_id in event_log.get_unique_cases() {
            let case_events = event_log.events_for_case(&case_id);

            for event in &case_events {
                activities.insert(event.activity.clone());
            }

            for window in case_events.windows(2) {
                let from = window[0].activity.clone();
                let to = window[1].activity.clone();

                *transitions.entry((from, to)).or_insert(0) += 1;
            }
        }

        let mut activities: Vec<_> = activities.into_iter().collect();
        activities.sort();

        Self {
            transitions,
            activities,
        }
    }

    fn print_summary(&self) {
        println!("\n📊 Discovered Process Structure:");
        println!("  Activities: {:?}", self.activities);
        println!("  Transitions: {} unique paths", self.transitions.len());
    }
}

// ============================================================================
// 4. Performance Analytics - Analyze Metrics
// ============================================================================

#[derive(Debug, Clone)]
struct PerformanceReport {
    avg_cycle_time_ms: f64,
    throughput_per_hour: f64,
    activity_durations: HashMap<String, f64>,
    bottlenecks: Vec<String>,
}

struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    fn analyze(event_log: &EventLog) -> PerformanceReport {
        // Calculate cycle times
        let mut cycle_times = Vec::new();
        for case_id in event_log.get_unique_cases() {
            let case_events = event_log.events_for_case(&case_id);

            if let (Some(first), Some(last)) = (case_events.first(), case_events.last()) {
                let duration = last
                    .timestamp
                    .signed_duration_since(first.timestamp)
                    .num_milliseconds();
                cycle_times.push(duration);
            }
        }

        let avg_cycle_time_ms = if !cycle_times.is_empty() {
            cycle_times.iter().sum::<i64>() as f64 / cycle_times.len() as f64
        } else {
            0.0
        };

        // Calculate throughput (simplified)
        let throughput_per_hour = if avg_cycle_time_ms > 0.0 {
            3600_000.0 / avg_cycle_time_ms
        } else {
            0.0
        };

        // Calculate activity durations
        let mut activity_durations: HashMap<String, Vec<i64>> = HashMap::new();
        for event in &event_log.events {
            activity_durations
                .entry(event.activity.clone())
                .or_default()
                .push(event.duration_ms);
        }

        let activity_avg_durations: HashMap<String, f64> = activity_durations
            .into_iter()
            .map(|(activity, durations)| {
                let avg = durations.iter().sum::<i64>() as f64 / durations.len() as f64;
                (activity, avg)
            })
            .collect();

        // Identify bottlenecks (activities with avg duration > 100ms)
        let bottlenecks: Vec<String> = activity_avg_durations
            .iter()
            .filter(|(_, &avg)| avg > 100.0)
            .map(|(activity, _)| activity.clone())
            .collect();

        PerformanceReport {
            avg_cycle_time_ms,
            throughput_per_hour,
            activity_durations: activity_avg_durations,
            bottlenecks,
        }
    }

    fn print_report(report: &PerformanceReport) {
        println!("\n📈 Performance Analytics Report:");
        println!("  Avg Cycle Time: {:.2}ms", report.avg_cycle_time_ms);
        println!(
            "  Throughput: {:.2} cases/hour",
            report.throughput_per_hour
        );

        println!("\n  Activity Durations:");
        for (activity, avg) in &report.activity_durations {
            println!("    {}: {:.2}ms", activity, avg);
        }

        if !report.bottlenecks.is_empty() {
            println!("\n  ⚠️  Bottlenecks Detected:");
            for activity in &report.bottlenecks {
                println!("    - {}", activity);
            }
        }
    }
}

// ============================================================================
// 5. Workflow Execution (Example)
// ============================================================================

struct WorkflowEngine {
    telemetry: Arc<TelemetryCollector>,
}

impl WorkflowEngine {
    fn new(telemetry: Arc<TelemetryCollector>) -> Self {
        Self { telemetry }
    }

    /// Execute a workflow and emit telemetry
    fn execute_workflow(&self, workflow_id: &str) {
        let trace_id = format!("trace_{}", workflow_id);

        // Step 1: Validate
        self.execute_step(&trace_id, "validate_input", 15, "validator");

        // Step 2: Fetch data
        self.execute_step(&trace_id, "fetch_data", 50, "data_service");

        // Step 3: Process (slower - potential bottleneck)
        self.execute_step(&trace_id, "process_data", 120, "processor");

        // Step 4: Save
        self.execute_step(&trace_id, "save_result", 30, "storage");
    }

    fn execute_step(&self, trace_id: &str, step_name: &str, duration_ms: u64, resource: &str) {
        let start_time = Utc::now();

        // Simulate work
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));

        let end_time = Utc::now();

        // Record span
        let mut attributes = HashMap::new();
        attributes.insert("resource".to_string(), resource.to_string());

        self.telemetry.record_span(OtelSpan {
            span_id: format!("{}_{}_{}", trace_id, step_name, start_time.timestamp_millis()),
            trace_id: trace_id.to_string(),
            name: step_name.to_string(),
            start_time,
            end_time,
            attributes,
        });
    }
}

// ============================================================================
// 6. Complete Integration Example
// ============================================================================

fn main() {
    println!("=== Process Mining Integration Template ===\n");

    let overall_start = Instant::now();

    // Step 1: Set up telemetry collection
    println!("1️⃣  Setting up telemetry collection...");
    let telemetry = Arc::new(TelemetryCollector::new());
    println!("  ✅ Telemetry collector initialized\n");

    // Step 2: Execute workflows
    println!("2️⃣  Executing workflows...");
    let engine = WorkflowEngine::new(Arc::clone(&telemetry));

    for i in 0..5 {
        let workflow_id = format!("workflow_{:03}", i);
        engine.execute_workflow(&workflow_id);
    }

    let spans = telemetry.get_spans();
    println!("  ✅ Executed 5 workflows ({} spans collected)\n", spans.len());

    // Step 3: Extract event log
    println!("3️⃣  Extracting event log from telemetry...");
    let event_log = EventLogExtractor::extract_from_spans(&spans);
    println!(
        "  ✅ Event log extracted ({} events)\n",
        event_log.events.len()
    );

    // Step 4: Discover process
    println!("4️⃣  Discovering process structure...");
    let process_graph = ProcessGraph::discover(&event_log);
    println!("  ✅ Process discovered");
    process_graph.print_summary();

    // Step 5: Analyze performance
    println!("\n5️⃣  Analyzing performance...");
    let performance_report = PerformanceAnalyzer::analyze(&event_log);
    println!("  ✅ Performance analyzed");
    PerformanceAnalyzer::print_report(&performance_report);

    // Step 6: Generate recommendations
    println!("\n6️⃣  Generating optimization recommendations...\n");

    if !performance_report.bottlenecks.is_empty() {
        println!("  💡 Recommendations:");
        println!("  - Optimize bottleneck activities: {:?}", performance_report.bottlenecks);
        println!("  - Consider caching for frequently called activities");
        println!("  - Evaluate parallel execution opportunities");
    } else {
        println!("  ✅ No bottlenecks detected - process is well-optimized!");
    }

    println!("\n⏱️  Total Processing Time: {:?}", overall_start.elapsed());

    // Step 7: Integration checklist
    println!("\n=== Integration Checklist ===\n");
    println!("✅ 1. Telemetry collection configured");
    println!("✅ 2. Workflow execution instrumented");
    println!("✅ 3. Event log extraction implemented");
    println!("✅ 4. Process discovery configured");
    println!("✅ 5. Performance analytics enabled");
    println!("✅ 6. Bottleneck detection active");
    println!("✅ 7. Optimization recommendations generated");

    println!("\n=== Production Deployment Steps ===\n");
    println!("1. Connect to OTEL collector for real telemetry");
    println!("2. Configure persistent storage for event logs");
    println!("3. Set up scheduled analysis jobs");
    println!("4. Integrate with monitoring dashboards");
    println!("5. Enable alerting on performance degradation");
    println!("6. Implement A/B testing for optimizations");
    println!("7. Document discovered process patterns");

    println!("\n=== Next Steps ===\n");
    println!("- See examples/process-mining-workflow-analysis.rs for detailed analysis");
    println!("- See examples/process-mining-discovery.rs for pattern discovery");
    println!("- See examples/process-mining-performance-analytics.rs for advanced metrics");
    println!("- Read docs/papers/how-to-guides/13-analyze-workflows-with-process-mining.md");
    println!();
}

// ============================================================================
// Template Customization Guide
// ============================================================================

// CUSTOMIZATION POINTS:
//
// 1. TelemetryCollector:
//    - Replace with OTEL SDK integration
//    - Configure OTLP exporter
//    - Add custom span attributes
//
// 2. EventLogExtractor:
//    - Customize case_id extraction logic
//    - Add business-specific attributes
//    - Implement filtering for specific workflows
//
// 3. ProcessGraph:
//    - Add edge weights and probabilities
//    - Implement pattern detection
//    - Export to DOT/BPMN formats
//
// 4. PerformanceAnalyzer:
//    - Add custom metrics (SLA compliance, etc.)
//    - Implement trend analysis
//    - Configure alerting thresholds
//
// 5. WorkflowEngine:
//    - Replace with your actual workflow engine
//    - Ensure proper span context propagation
//    - Add error handling and retry logic

// PRODUCTION CONSIDERATIONS:
//
// - Use async/await for non-blocking execution
// - Implement proper error handling with Result types
// - Add comprehensive logging and tracing
// - Set up monitoring and alerting
// - Implement data retention policies
// - Consider privacy/compliance requirements
// - Add authentication/authorization
// - Enable distributed tracing across services
