# How to Analyze Workflows with Process Mining

**Goal**: Discover process patterns and optimize workflow performance using process mining techniques

**Time**: 2-3 hours | **Level**: Advanced

---

## Prerequisites

Before you begin, ensure you have:

- ✅ Working KNHK installation with workflow engine
- ✅ Telemetry configured and collecting OTEL spans
- ✅ Basic understanding of process mining concepts
- ✅ Familiarity with workflow patterns
- ✅ Understanding of performance metrics (cycle time, throughput)

**Related Guides**:
- [How to Emit Proper Telemetry](07-emit-proper-telemetry.md)
- [How to Implement Workflow Patterns](11-implement-workflow-patterns.md)
- [How to Optimize Performance](08-optimize-performance.md)

---

## What is Process Mining?

Process mining extracts knowledge from event logs to:
1. **Discover** actual process structure from execution traces
2. **Analyze** performance metrics and identify bottlenecks
3. **Validate** that workflows match expected patterns
4. **Optimize** processes based on data-driven insights

---

## Step 1: Set Up Telemetry for Workflow Events

**Goal**: Configure telemetry to capture workflow execution as event logs

### 1.1 Add Process Mining Dependencies

Update `/home/user/knhk/rust/Cargo.toml`:

```toml
[dependencies]
knhk-process-mining = { workspace = true }
knhk-workflow-engine = { workspace = true }
knhk-otel = { workspace = true }
```

### 1.2 Instrument Workflow Activities

```rust
use tracing::{info_span, Instrument};
use chrono::Utc;

async fn execute_workflow_step(step_name: &str) {
    let span = info_span!(
        "workflow.step",
        step.name = step_name,
        step.timestamp = %Utc::now()
    );

    async {
        // Your step logic here
        perform_step_work().await;
    }
    .instrument(span)
    .await;
}
```

### 1.3 Configure Span Attributes

Add these attributes to each workflow span:
- `case.id` - Unique workflow instance ID
- `activity.name` - Activity/step name
- `resource` - Agent/service executing the step
- `lifecycle` - Event lifecycle (start, complete, etc.)

**Verification**:
```bash
# Check that spans are being collected
cargo run --bin workflow-trace-viewer

# Should show workflow spans with all required attributes
```

---

## Step 2: Extract Event Log from Telemetry

**Goal**: Convert OTEL spans into process mining event log format

### 2.1 Create Event Log Builder

```rust
use knhk_process_mining::{EventLogBuilder, ProcessEvent};
use hashbrown::HashMap;

let mut builder = EventLogBuilder::new();

for span in telemetry_spans {
    builder.add_span_event(
        span.trace_id.clone(),        // case_id
        span.name.clone(),              // activity
        span.start_time,                // timestamp
        span.attributes.get("resource").cloned(), // resource
        span.attributes.clone(),        // attributes
    );
}

let event_log = builder.build()?;
```

### 2.2 Verify Event Log

```rust
println!("Event Log Summary:");
println!("  Total Cases: {}", event_log.metadata.total_cases);
println!("  Total Events: {}", event_log.metadata.total_events);
println!("  Unique Activities: {}", event_log.activities.len());
```

**Expected Output**:
```
Event Log Summary:
  Total Cases: 50
  Total Events: 200
  Unique Activities: 4
```

---

## Step 3: Discover Process Structure

**Goal**: Extract the actual process graph from execution traces

### 3.1 Run Process Discovery

```rust
use knhk_process_mining::DiscoveryEngine;

let discovery = DiscoveryEngine::new()
    .with_min_frequency(2)
    .with_min_confidence(0.5);

let process_graph = discovery.discover(&event_log)?;

// Print discovered structure
println!("Discovered Activities: {:?}", process_graph.nodes);
println!("Transitions: {} paths", process_graph.edges.len());
```

### 3.2 Analyze Discovered Patterns

```rust
for pattern in &process_graph.patterns {
    println!("{:?}: {} (confidence: {:.2})",
        pattern.pattern_type,
        pattern.description,
        pattern.confidence
    );
}
```

**Example Output**:
```
Sequence: validate → fetch (confidence: 0.95)
Sequence: fetch → process (confidence: 0.98)
Loop: validate ↔ retry (confidence: 0.30)
```

### 3.3 Export Process Graph

```rust
// Export to visualization format
let dot_graph = process_graph.to_dot_format()?;
std::fs::write("process_graph.dot", dot_graph)?;

// Convert to image with Graphviz
// dot -Tpng process_graph.dot -o process_graph.png
```

---

## Step 4: Analyze Performance Metrics

**Goal**: Calculate cycle times, throughput, and identify bottlenecks

### 4.1 Run Performance Analysis

```rust
use knhk_process_mining::ProcessAnalyzer;

let analyzer = ProcessAnalyzer::new(&event_log);
let analytics = analyzer.analyze()?;

println!("Performance Metrics:");
println!("  Avg Cycle Time: {:.2}ms", analytics.avg_cycle_time_ms);
println!("  Median: {:.2}ms", analytics.median_cycle_time_ms);
println!("  Throughput: {:.2} cases/hour", analytics.throughput_per_hour);
```

### 4.2 Analyze Activity Performance

```rust
for (activity, metrics) in &analytics.activity_metrics {
    println!("\nActivity: {}", activity);
    println!("  Count: {}", metrics.count);
    println!("  Avg Duration: {:.2}ms", metrics.avg_duration_ms);
    println!("  Time %: {:.1}%", metrics.time_percentage);
}
```

### 4.3 Review Time Breakdown

```rust
let processing = event_log.total_processing_time();
let waiting = event_log.total_waiting_time();
let ratio = processing / (processing + waiting);

println!("Time Breakdown:");
println!("  Processing: {:.2}ms ({:.1}%)", processing, ratio * 100.0);
println!("  Waiting: {:.2}ms ({:.1}%)", waiting, (1.0 - ratio) * 100.0);
```

---

## Step 5: Identify Bottlenecks

**Goal**: Find activities that limit workflow performance

### 5.1 Run Bottleneck Detection

```rust
use knhk_process_mining::BottleneckDetector;

let detector = BottleneckDetector::new()
    .with_threshold(0.5);  // Severity threshold

let critical = detector.detect_critical(&analytics);

println!("Critical Bottlenecks:");
for bottleneck in critical {
    println!("  Activity: {}", bottleneck.activity);
    println!("  Severity: {:.2}", bottleneck.severity);
    println!("  Description: {}", bottleneck.description);
    println!("  Suggestion: {}", bottleneck.suggestion);
}
```

### 5.2 Prioritize Bottlenecks

```rust
// Bottlenecks are sorted by severity (descending)
let top_3 = critical.iter().take(3);

println!("\nFocus on these bottlenecks first:");
for (i, b) in top_3.enumerate() {
    println!("{}. {} - {}", i + 1, b.activity, b.suggestion);
}
```

**Example Output**:
```
Focus on these bottlenecks first:
1. process_data - Optimize or parallelize (consumes 45% of time)
2. fetch_external_api - High variance indicates inconsistent performance
3. enrich_metadata - Long average duration (150ms)
```

---

## Step 6: Compare with Expected Process

**Goal**: Validate that actual execution matches designed process

### 6.1 Define Expected Patterns

```rust
use knhk_process_mining::{PatternValidator, ExpectedPattern, PatternType};

let expected = vec![
    ExpectedPattern {
        pattern_type: PatternType::Sequence,
        activities: vec!["validate".into(), "fetch".into()],
        required: true,
    },
    ExpectedPattern {
        pattern_type: PatternType::Sequence,
        activities: vec!["fetch".into(), "process".into()],
        required: true,
    },
];
```

### 6.2 Run Conformance Checking

```rust
let validator = PatternValidator::new(expected);
let report = validator.validate(&process_graph)?;

println!("Conformance Report:");
println!("  Conformance Score: {:.1}%", report.conformance * 100.0);
println!("  Matched: {}", report.matched.len());
println!("  Missing: {}", report.missing.len());
println!("  Unexpected: {}", report.unexpected.len());
```

### 6.3 Review Deviations

```rust
if !report.unexpected.is_empty() {
    println!("\n⚠️  Unexpected patterns found:");
    for pattern in &report.unexpected {
        println!("  - {}", pattern.description);
    }
}
```

---

## Step 7: Generate Performance Report

**Goal**: Create comprehensive analysis report for stakeholders

### 7.1 Generate Report

```rust
use knhk_process_mining::ReportGenerator;

let report = ReportGenerator::new()
    .with_event_log(&event_log)
    .with_analytics(&analytics)
    .with_process_graph(&process_graph)
    .with_conformance(&conformance_report)
    .generate()?;

// Save to file
std::fs::write("workflow_analysis_report.md", report.to_markdown())?;
std::fs::write("workflow_analysis_report.json", report.to_json()?)?;
```

### 7.2 Report Contents

The report should include:
- Executive summary with key metrics
- Process graph visualization
- Performance statistics
- Bottleneck analysis
- Conformance assessment
- Optimization recommendations

---

## Step 8: Implement Optimizations

**Goal**: Apply data-driven improvements to workflow

### 8.1 Address Top Bottlenecks

Based on bottleneck analysis:

```rust
// Example: Add caching for slow activity
#[cached(time = 300)] // Cache for 5 minutes
async fn fetch_external_data(id: &str) -> Result<Data> {
    // Slow external API call
    external_api::fetch(id).await
}
```

### 8.2 Parallelize Independent Activities

```rust
use tokio::join;

// Before: Sequential (slow)
let a = fetch_data().await?;
let b = fetch_metadata().await?;

// After: Parallel (fast)
let (a, b) = join!(
    fetch_data(),
    fetch_metadata()
);
```

### 8.3 Measure Improvement

```rust
// Run analysis before and after optimization
let before_analytics = analyze_baseline_period();
let after_analytics = analyze_optimized_period();

let improvement = (before_analytics.avg_cycle_time_ms
    - after_analytics.avg_cycle_time_ms)
    / before_analytics.avg_cycle_time_ms
    * 100.0;

println!("Improvement: {:.1}% faster", improvement);
```

---

## Verification

Confirm your process mining setup is working:

### ✅ Event Log Extraction
```bash
cargo run --example process-mining-workflow-analysis
# Should extract event log and show metrics
```

### ✅ Process Discovery
```bash
cargo run --example process-mining-discovery
# Should discover process graph and patterns
```

### ✅ Performance Analytics
```bash
cargo run --example process-mining-performance-analytics
# Should calculate metrics and identify bottlenecks
```

### ✅ Integration Template
```bash
cargo run --bin templates/process-mining-integration-template
# Should run complete analysis pipeline
```

---

## Troubleshooting

### Issue: Event log is empty

**Cause**: Telemetry spans not being collected

**Solution**:
1. Verify OTEL collector is running
2. Check that spans have required attributes (`case.id`, `activity.name`)
3. Ensure spans are being exported correctly

```bash
# Check OTEL collector logs
journalctl -u otel-collector -f
```

### Issue: Process graph has unexpected structure

**Cause**: Missing or duplicate events in event log

**Solution**:
1. Review event log for data quality issues
2. Check for missing lifecycle events (start/complete)
3. Verify case IDs are consistent within workflows

```rust
// Validate event log
let validation = event_log.validate()?;
println!("Validation: {:?}", validation);
```

### Issue: Performance metrics seem incorrect

**Cause**: Incorrect timestamp handling or time zone issues

**Solution**:
1. Ensure all timestamps are in UTC
2. Verify clock synchronization across services
3. Check for timestamp precision loss

```rust
// Verify timestamp precision
for event in &event_log.events {
    println!("{}: {}", event.activity, event.timestamp.to_rfc3339());
}
```

---

## Related Documentation

**How-to Guides**:
- [How to Emit Proper Telemetry](07-emit-proper-telemetry.md)
- [How to Implement Workflow Patterns](11-implement-workflow-patterns.md)
- [How to Optimize Performance](08-optimize-performance.md)

**Reference**:
- [Process Mining Checklist](../../reference/cards/PROCESS_MINING_CHECKLIST.md)
- [Workflow Engine Documentation](../../reference/WORKFLOW_ENGINE.md)

**Examples**:
- `/home/user/knhk/examples/process-mining-workflow-analysis.rs`
- `/home/user/knhk/examples/process-mining-discovery.rs`
- `/home/user/knhk/examples/process-mining-performance-analytics.rs`

**Templates**:
- `/home/user/knhk/templates/process-mining-integration-template.rs`

---

## Best Practices

### 1. Event Log Quality
- ✅ Ensure complete lifecycle events (start, complete)
- ✅ Use consistent case IDs within workflows
- ✅ Include relevant business attributes
- ✅ Handle concurrent workflow instances correctly

### 2. Performance Analysis
- ✅ Collect sufficient data (>30 instances minimum)
- ✅ Analyze multiple time periods for trends
- ✅ Consider both average and percentile metrics (P95, P99)
- ✅ Account for business hours vs. off-hours differences

### 3. Process Discovery
- ✅ Set appropriate frequency thresholds
- ✅ Filter noise and rare variants
- ✅ Validate discovered patterns with domain experts
- ✅ Document expected vs. actual deviations

### 4. Optimization
- ✅ Prioritize high-impact bottlenecks
- ✅ Measure before and after improvements
- ✅ Use A/B testing for validation
- ✅ Monitor for regressions

---

## Advanced Topics

### Continuous Process Mining

Set up automated analysis:

```rust
// Schedule analysis every hour
#[tokio::main]
async fn main() {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));

    loop {
        interval.tick().await;

        let event_log = collect_recent_events().await?;
        let analytics = analyze_performance(&event_log)?;

        if analytics.has_degradation() {
            send_alert(&analytics).await?;
        }
    }
}
```

### Process Simulation

Use discovered process for what-if analysis:

```rust
let simulator = ProcessSimulator::new(&process_graph);
simulator.simulate_change(
    "process_data",
    OptimizationStrategy::ReduceDuration(0.5)
)?;

let predicted_improvement = simulator.estimate_impact()?;
println!("Predicted improvement: {:.1}%", predicted_improvement);
```

---

## Summary

You've learned how to:
- ✅ Extract event logs from workflow telemetry
- ✅ Discover actual process structure from execution traces
- ✅ Analyze performance metrics and identify bottlenecks
- ✅ Validate processes against expected patterns
- ✅ Generate data-driven optimization recommendations
- ✅ Implement and measure improvements

**Next Steps**:
1. Set up continuous process mining for your workflows
2. Integrate with monitoring dashboards
3. Establish baseline metrics and SLAs
4. Implement alerting on performance degradation

---

**Last Updated**: 2025-11-15
**KNHK Version**: 1.1.0
**Guide**: 13 of 13 (How-to Guides Complete!)
