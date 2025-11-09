# OTEL Helper Functions and Macros

## Overview

The workflow engine provides helper functions and macros to simplify OpenTelemetry instrumentation following Van der Aalst production OTEL requirements.

## Helper Functions

### `add_workflow_attributes()`

Adds common workflow attributes to a span.

```rust
use knhk_workflow_engine::integration::otel_helpers::add_workflow_attributes;

add_workflow_attributes(
    &otel,
    span_ctx,
    case_id: Some(&case_id),
    spec_id: Some(&spec_id),
    task_id: Some(&task.id),
    pattern_id: Some(&pattern_id),
).await?;
```

### `end_span_with_result()`

Ends a span with success/latency and lifecycle transition.

```rust
use knhk_workflow_engine::integration::otel_helpers::end_span_with_result;

end_span_with_result(
    &otel,
    span_ctx,
    success: true,
    start_time: task_start_time,
).await?;
```

### `check_bottleneck()`

Checks if latency exceeds threshold.

```rust
use knhk_workflow_engine::integration::otel_helpers::check_bottleneck;

let is_bottleneck = check_bottleneck(latency_ms, 1000);
```

### `add_bottleneck_if_detected()`

Adds bottleneck attribute if detected.

```rust
use knhk_workflow_engine::integration::otel_helpers::add_bottleneck_if_detected;

add_bottleneck_if_detected(
    &otel,
    span_ctx,
    latency_ms,
    threshold_ms: 1000,
).await?;
```

### `add_conformance_attributes()`

Adds conformance checking attributes.

```rust
use knhk_workflow_engine::integration::otel_helpers::add_conformance_attributes;

add_conformance_attributes(
    &otel,
    span_ctx,
    expected_pattern: Some(2),
    actual_pattern: pattern_id.0,
).await?;
```

### `create_trace_context()`

Creates a trace context from case_id for trace correlation.

```rust
use knhk_workflow_engine::integration::otel_helpers::create_trace_context;

let parent_ctx = create_trace_context(&case_id);
```

## Macros

### `otel_span!`

Starts an OTEL span with XES attributes.

```rust
let span_ctx = otel_span!(
    otel,
    "knhk.workflow_engine.execute_task",
    case_id: Some(&case_id),
    spec_id: Some(&spec_id),
    task_id: Some(&task.id),
    pattern_id: Some(&pattern_id),
    parent: Some(&parent_span), // Optional
).await?;
```

### `otel_span_end!`

Ends an OTEL span with lifecycle transition.

```rust
// With start_time (calculates latency automatically)
otel_span_end!(
    otel,
    span_ctx,
    success: true,
    start_time: task_start_time
).await?;

// With explicit latency_ms
otel_span_end!(
    otel,
    span_ctx,
    success: true,
    latency_ms: duration_ms
).await?;
```

### `otel_attr!`

Adds multiple attributes to a span.

```rust
otel_attr!(
    otel,
    span_ctx,
    "knhk.workflow_engine.resource_utilization" => utilization.to_string(),
    "knhk.workflow_engine.bottleneck_detected" => "true",
    "knhk.workflow_engine.case_state" => "Running"
).await?;
```

### `otel_with_span!`

Executes code within an OTEL span (automatically handles start/end).

```rust
let result = otel_with_span!(
    otel,
    "knhk.workflow_engine.execute_task",
    case_id: Some(&case_id),
    task_id: Some(&task.id),
    {
        // Your code here
        execute_task().await
    }
).await?;
```

### `otel_resource!`

Adds resource attributes to a span.

```rust
otel_resource!(
    otel,
    span_ctx,
    resource: Some("resource_123"),
    role: Some("executor")
).await?;
```

### `otel_bottleneck!`

Adds bottleneck detection attribute if detected.

```rust
otel_bottleneck!(
    otel,
    span_ctx,
    latency_ms: duration_ms,
    threshold_ms: 1000
).await?;
```

### `otel_conformance!`

Adds conformance checking attributes.

```rust
otel_conformance!(
    otel,
    span_ctx,
    expected_pattern: Some(2),
    actual_pattern: pattern_id.0
).await?;
```

## Usage Examples

### Example 1: Task Execution with Macros

```rust
use knhk_workflow_engine::otel_span;
use knhk_workflow_engine::otel_span_end;
use knhk_workflow_engine::otel_resource;
use knhk_workflow_engine::otel_bottleneck;

let start_time = Instant::now();

// Start span
let span_ctx = otel_span!(
    &engine.otel_integration,
    "knhk.workflow_engine.execute_task",
    case_id: Some(&case_id),
    task_id: Some(&task.id),
    pattern_id: Some(&pattern_id)
).await?;

// Add resource tracking
otel_resource!(
    &engine.otel_integration,
    span_ctx,
    resource: Some("resource_123"),
    role: Some("executor")
).await?;

// Execute task
let result = execute_task().await;

// Check bottleneck
let duration_ms = start_time.elapsed().as_millis();
otel_bottleneck!(
    &engine.otel_integration,
    span_ctx,
    latency_ms: duration_ms,
    threshold_ms: 1000
).await?;

// End span
otel_span_end!(
    &engine.otel_integration,
    span_ctx,
    success: result.is_ok(),
    start_time: start_time
).await?;
```

### Example 2: Using `otel_with_span!` Macro

```rust
use knhk_workflow_engine::otel_with_span;

let result = otel_with_span!(
    &engine.otel_integration,
    "knhk.workflow_engine.execute_pattern",
    case_id: Some(&case_id),
    pattern_id: Some(&pattern_id),
    {
        // Pattern execution code
        engine.pattern_registry.execute(&pattern_id, &context)
    }
).await?;
```

### Example 3: Pattern Execution with Conformance Checking

```rust
use knhk_workflow_engine::otel_span;
use knhk_workflow_engine::otel_span_end;
use knhk_workflow_engine::otel_conformance;

let start_time = Instant::now();

let span_ctx = otel_span!(
    &engine.otel_integration,
    "knhk.workflow_engine.execute_pattern",
    case_id: Some(&case_id),
    pattern_id: Some(&pattern_id)
).await?;

// Add conformance checking
let expected_pattern = get_expected_pattern_from_spec(&spec);
otel_conformance!(
    &engine.otel_integration,
    span_ctx,
    expected_pattern: expected_pattern,
    actual_pattern: pattern_id.0
).await?;

// Execute pattern
let result = engine.pattern_registry.execute(&pattern_id, &context)?;

// End span
otel_span_end!(
    &engine.otel_integration,
    span_ctx,
    success: result.success,
    start_time: start_time
).await?;
```

## Benefits

1. **Reduced Boilerplate**: Macros handle common patterns automatically
2. **Consistency**: Ensures all spans include required XES attributes
3. **Trace Correlation**: Automatic trace correlation by case_id
4. **Type Safety**: Compile-time checking of macro arguments
5. **Van der Aalst Compliance**: All macros follow production OTEL requirements

## Best Practices

1. Use `otel_with_span!` for simple operations that don't need custom error handling
2. Use `otel_span!` + `otel_span_end!` for operations that need custom error handling
3. Always include `case_id` for trace correlation
4. Use `otel_bottleneck!` after measuring latency
5. Use `otel_conformance!` for pattern execution when expected pattern is known

