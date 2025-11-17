# Van der Aalst Production OTEL Requirements

## Overview

Based on Van der Aalst's process mining and workflow analysis methodology, this document outlines what he would want from production OpenTelemetry instrumentation.

## Core Requirements

### 1. **Case Correlation (Trace Reconstruction)**

**Requirement**: All spans must be linked by `case_id` to enable complete trace reconstruction.

**Why**: Process mining requires complete event logs per case (trace) to discover process models and check conformance.

**OTEL Implementation**:
- Every span must include `knhk.workflow_engine.case_id`
- Use trace context to link all spans for a case
- Support trace export to XES format for ProM analysis

**Example**:
```rust
// All spans for a case must share the same trace_id
otel.add_attribute(span_ctx, "knhk.workflow_engine.case_id", case_id.to_string());
```

### 2. **Event Ordering (Temporal Analysis)**

**Requirement**: Precise timestamps for all events to enable temporal analysis and bottleneck detection.

**Why**: Process mining algorithms (Alpha+++, Inductive Miner) require accurate event ordering to discover process models.

**OTEL Implementation**:
- Every span must include `time:timestamp` (ISO 8601)
- Track `start_time` and `end_time` for each operation
- Support millisecond precision (required for bottleneck analysis)

**Example**:
```rust
otel.add_attribute(span_ctx, "time:timestamp", start_time.to_rfc3339());
otel.add_attribute(span_ctx, "knhk.workflow_engine.latency_ms", latency_ms.to_string());
```

### 3. **Lifecycle Transitions (XES Compatibility)**

**Requirement**: Track lifecycle transitions (start/complete/cancel) for all activities.

**Why**: XES standard requires `lifecycle:transition` for process mining tools (ProM, Disco).

**OTEL Implementation**:
- Add `lifecycle:transition` attribute to all task/pattern spans
- Values: `start`, `complete`, `cancel`
- Map to OTEL span status: `start` → span start, `complete` → `Ok`, `cancel` → `Error`

**Example**:
```rust
otel.add_attribute(span_ctx, "lifecycle:transition", "start");
// ... execution ...
otel.add_attribute(span_ctx, "lifecycle:transition", "complete");
otel.end_span(span_ctx, SpanStatus::Ok);
```

### 4. **Pattern Discovery (Which Patterns Are Used)**

**Requirement**: Track which Van der Aalst patterns (1-43) are actually used in production.

**Why**: Pattern discovery enables understanding of actual workflow behavior vs. design.

**OTEL Implementation**:
- Every pattern execution must include `knhk.workflow_engine.pattern_id`
- Track pattern usage metrics (count, frequency, success rate)
- Support pattern composition analysis (which patterns combine)

**Example**:
```rust
otel.add_attribute(span_ctx, "knhk.workflow_engine.pattern_id", pattern_id.0.to_string());
otel.add_attribute(span_ctx, "knhk.workflow_engine.pattern_name", "Parallel Split");
otel.add_attribute(span_ctx, "knhk.workflow_engine.pattern_category", "Basic Control Flow");
```

### 5. **Conformance Checking (Design vs. Execution)**

**Requirement**: Enable comparison of actual execution to workflow specification.

**Why**: Conformance checking verifies that execution matches design (fitness, precision, generalization).

**OTEL Implementation**:
- Include `knhk.workflow_engine.spec_id` in all spans
- Track expected vs. actual pattern execution
- Support alignment-based conformance checking

**Example**:
```rust
otel.add_attribute(span_ctx, "knhk.workflow_engine.spec_id", spec_id.to_string());
// Compare actual pattern execution to spec
```

### 6. **Performance Analysis (Bottleneck Detection)**

**Requirement**: Track latency and resource utilization for bottleneck detection.

**Why**: Performance analysis identifies optimization opportunities and bottlenecks.

**OTEL Implementation**:
- Track `knhk.workflow_engine.latency_ms` for all operations
- Track resource utilization per pattern/task
- Support percentile analysis (p50, p95, p99)

**Example**:
```rust
otel.add_attribute(span_ctx, "knhk.workflow_engine.latency_ms", latency_ms.to_string());
otel.add_attribute(span_ctx, "knhk.workflow_engine.resource_utilization", utilization.to_string());
```

### 7. **Deadlock Detection (Runtime Verification)**

**Requirement**: Enable runtime deadlock detection from execution traces.

**Why**: Deadlock detection verifies workflow properties (safety, liveness).

**OTEL Implementation**:
- Track task dependencies and execution order
- Detect circular dependencies in traces
- Flag potential deadlocks in spans

**Example**:
```rust
otel.add_attribute(span_ctx, "knhk.workflow_engine.task_dependencies", deps.to_string());
// Analyze traces for deadlock patterns
```

### 8. **Resource Tracking (Who/What Executed)**

**Requirement**: Track resource assignment for organizational analysis.

**Why**: Resource tracking enables organizational mining and workload analysis.

**OTEL Implementation**:
- Include `org:resource` attribute (XES standard)
- Track resource roles and capabilities
- Support resource utilization metrics

**Example**:
```rust
otel.add_attribute(span_ctx, "org:resource", resource_id.to_string());
otel.add_attribute(span_ctx, "org:role", role.to_string());
```

### 9. **Process Discovery (Model Reconstruction)**

**Requirement**: Enable process model reconstruction from execution traces.

**Why**: Process discovery reveals actual process models from execution logs.

**OTEL Implementation**:
- Export traces to XES format for ProM analysis
- Support trace aggregation for process discovery
- Include all required XES attributes

**Example**:
```rust
// Export case traces to XES
let xes_log = engine.export_case_to_xes(case_id).await?;
// Import into ProM for process discovery
```

### 10. **Anomaly Detection (Deviation Analysis)**

**Requirement**: Detect deviations from expected patterns and workflows.

**Why**: Anomaly detection identifies process deviations and compliance violations.

**OTEL Implementation**:
- Compare actual execution to expected patterns
- Flag unexpected pattern compositions
- Track deviation frequency and severity

**Example**:
```rust
otel.add_attribute(span_ctx, "knhk.workflow_engine.deviation_detected", "true");
otel.add_attribute(span_ctx, "knhk.workflow_engine.deviation_type", "unexpected_pattern");
```

## Required OTEL Attributes

### Core Attributes (Required for All Spans)

1. **`knhk.workflow_engine.case_id`** (string) - Case identifier for trace correlation
2. **`knhk.workflow_engine.spec_id`** (string) - Workflow specification identifier
3. **`time:timestamp`** (string) - Event timestamp (ISO 8601)
4. **`lifecycle:transition`** (string) - Event lifecycle (start/complete/cancel)
5. **`knhk.workflow_engine.latency_ms`** (int) - Operation latency

### Pattern-Specific Attributes

6. **`knhk.workflow_engine.pattern_id`** (int) - Van der Aalst pattern ID (1-43)
7. **`knhk.workflow_engine.pattern_name`** (string) - Pattern name
8. **`knhk.workflow_engine.pattern_category`** (string) - Pattern category
9. **`knhk.workflow_engine.success`** (boolean) - Execution success

### Task-Specific Attributes

10. **`knhk.workflow_engine.task_id`** (string) - Task identifier
11. **`org:resource`** (string) - Resource assignment (XES standard)
12. **`org:role`** (string) - Resource role

### Performance Attributes

13. **`knhk.workflow_engine.resource_utilization`** (float) - Resource utilization
14. **`knhk.workflow_engine.bottleneck_detected`** (boolean) - Bottleneck flag

### Conformance Attributes

15. **`knhk.workflow_engine.expected_pattern`** (int) - Expected pattern ID
16. **`knhk.workflow_engine.conformance_violation`** (boolean) - Conformance flag

## XES Compatibility

All OTEL spans must be exportable to XES format for ProM analysis:

**XES Mapping**:
- `knhk.workflow_engine.case_id` → `concept:name` (trace)
- `knhk.workflow_engine.task_id` → `concept:name` (event)
- `time:timestamp` → `time:timestamp` (event)
- `lifecycle:transition` → `lifecycle:transition` (event)
- `org:resource` → `org:resource` (event)
- `knhk.workflow_engine.pattern_id` → `pattern:id` (KNHK extension)

## Process Mining Integration

### Export to XES

```rust
// Export case traces to XES for ProM
let xes_log = engine.export_case_to_xes(case_id).await?;
// All OTEL spans for the case are converted to XES events
```

### Process Discovery

```rust
// Discover process model from traces
let discovered_model = process_mining::discover_process(xes_log, Algorithm::AlphaPlusPlus)?;
// Compare to specification
let conformance = process_mining::check_conformance(spec, discovered_model)?;
```

### Conformance Metrics

```rust
// Calculate Van der Aalst conformance metrics
let fitness = process_mining::calculate_fitness(spec, xes_log)?;
let precision = process_mining::calculate_precision(spec, xes_log)?;
let generalization = process_mining::calculate_generalization(spec, xes_log)?;
```

## Implementation Checklist

- [ ] All spans include `case_id` for trace correlation
- [ ] All spans include precise timestamps (ISO 8601)
- [ ] All spans include `lifecycle:transition` (start/complete/cancel)
- [ ] All pattern spans include `pattern_id`, `pattern_name`, `pattern_category`
- [ ] All task spans include `task_id` and `org:resource`
- [ ] All spans include `latency_ms` for performance analysis
- [ ] Support trace export to XES format
- [ ] Support process discovery from traces
- [ ] Support conformance checking (fitness, precision, generalization)
- [ ] Support bottleneck detection from latency metrics
- [ ] Support deadlock detection from trace analysis
- [ ] Support anomaly detection (deviation analysis)

## Summary

Van der Aalst would want production OTEL that enables:

1. **Complete Trace Reconstruction**: All spans linked by `case_id`
2. **Process Mining**: Export to XES for ProM analysis
3. **Conformance Checking**: Compare actual execution to design
4. **Performance Analysis**: Bottleneck detection and optimization
5. **Pattern Discovery**: Track which patterns are actually used
6. **Deadlock Detection**: Runtime verification of workflow properties
7. **Anomaly Detection**: Identify deviations from expected behavior

The key is that OTEL spans must be **exportable to XES format** and **support process mining analysis** to enable Van der Aalst's methodology.





