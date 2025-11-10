# Van der Aalst Pattern OTEL Best Practices

## Overview

This document outlines best practices for OpenTelemetry instrumentation of Van der Aalst workflow patterns (1-43).

## Core Principles

### 1. Use Pattern-Specific Spans

Always use `PatternOtelHelper` for pattern execution spans, not generic `execute_pattern` spans.

**❌ Bad:**
```rust
let span_ctx = otel.start_execute_pattern_span(&pattern_id, &case_id).await?;
```

**✅ Good:**
```rust
use crate::integration::{PatternOtelHelper, PatternAttributes};

let pattern_attrs = PatternAttributes {
    branch_count: Some(3),  // For parallel patterns
    instance_count: Some(5), // For MI patterns
    chosen_branch: Some("approve".to_string()), // For choice patterns
    ..Default::default()
};

let span_ctx = PatternOtelHelper::start_pattern_span_with_attrs(
    &otel,
    &pattern_id,
    &case_id,
    pattern_attrs,
).await?;
```

### 2. Include Pattern Metadata

Every pattern span must include:
- `knhk.workflow_engine.pattern_id` (required)
- `knhk.workflow_engine.pattern_name` (required)
- `knhk.workflow_engine.pattern_category` (required)
- `knhk.workflow_engine.case_id` (required)
- `knhk.workflow_engine.success` (required)
- `knhk.workflow_engine.latency_ms` (required)

### 3. Pattern-Specific Attributes

Add pattern-specific attributes based on pattern type:

#### Parallel Patterns (2, 3, 7, 8, 9, 24, 25)
- `knhk.workflow_engine.branch_count` - Number of parallel branches
- `knhk.workflow_engine.synchronized_count` - Number of synchronized branches

#### Choice Patterns (4, 6, 16)
- `knhk.workflow_engine.chosen_branch` - Single chosen branch (exclusive choice)
- `knhk.workflow_engine.chosen_branches` - Multiple chosen branches (multi-choice)

#### Multiple Instance Patterns (12-15)
- `knhk.workflow_engine.instance_count` - Number of instances executed

#### Cancellation Patterns (19-25)
- `knhk.workflow_engine.cancelled_activity` - Activity that was cancelled

### 4. Span Naming Convention

Use pattern-specific span names that match Weaver schema:

- Pattern 1: `knhk.workflow_engine.pattern.sequence`
- Pattern 2: `knhk.workflow_engine.pattern.parallel_split`
- Pattern 3: `knhk.workflow_engine.pattern.synchronization`
- Pattern 12: `knhk.workflow_engine.pattern.mi_without_sync`
- Pattern 19: `knhk.workflow_engine.pattern.cancel_activity`

### 5. Error Handling

Always end spans with appropriate status:

```rust
PatternOtelHelper::end_pattern_span(
    &otel,
    span_ctx,
    result.success,
).await?;
```

### 6. Latency Tracking

Track latency for all pattern executions:

```rust
let start_time = Instant::now();
// ... pattern execution ...
let latency_ms = start_time.elapsed().as_millis();

otel.add_attribute(
    span_ctx.clone(),
    "knhk.workflow_engine.latency_ms".to_string(),
    latency_ms.to_string(),
).await?;
```

### 7. Pattern Context Extraction

Extract pattern-specific context from execution results:

```rust
let pattern_attrs = match pattern_id.0 {
    2 | 3 => PatternAttributes {
        branch_count: Some(result.next_activities.len() as u32),
        synchronized_count: Some(context.arrived_from.len() as u32),
        ..Default::default()
    },
    4 | 6 | 16 => PatternAttributes {
        chosen_branch: result.next_activities.first().cloned(),
        ..Default::default()
    },
    12..=15 => PatternAttributes {
        instance_count: Some(result.variables.get("instance_count")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0)),
        ..Default::default()
    },
    19..=25 => PatternAttributes {
        cancelled_activity: result.cancel_activities.first().cloned(),
        ..Default::default()
    },
    _ => PatternAttributes::default(),
};
```

## Implementation Checklist

- [ ] Use `PatternOtelHelper::start_pattern_span_with_attrs` for all pattern executions
- [ ] Include pattern_name and pattern_category attributes
- [ ] Add pattern-specific attributes based on pattern type
- [ ] Track latency for all pattern executions
- [ ] End spans with appropriate status
- [ ] Extract pattern-specific context from execution results
- [ ] Use pattern-specific span names matching Weaver schema

## Examples

### Pattern 2: Parallel Split

```rust
let pattern_attrs = PatternAttributes {
    branch_count: Some(3),
    ..Default::default()
};

let span_ctx = PatternOtelHelper::start_pattern_span_with_attrs(
    &otel,
    &PatternId(2),
    &case_id,
    pattern_attrs,
).await?;

// Execute pattern...
let result = registry.execute(&PatternId(2), &context)?;

// Add success and latency
otel.add_attribute(span_ctx.clone(), "knhk.workflow_engine.success".to_string(), result.success.to_string()).await?;
otel.add_attribute(span_ctx.clone(), "knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string()).await?;

PatternOtelHelper::end_pattern_span(&otel, span_ctx, result.success).await?;
```

### Pattern 12: MI Without Sync

```rust
let pattern_attrs = PatternAttributes {
    instance_count: Some(5),
    ..Default::default()
};

let span_ctx = PatternOtelHelper::start_pattern_span_with_attrs(
    &otel,
    &PatternId(12),
    &case_id,
    pattern_attrs,
).await?;

// Execute pattern...
let result = registry.execute(&PatternId(12), &context)?;

// Add success and latency
otel.add_attribute(span_ctx.clone(), "knhk.workflow_engine.success".to_string(), result.success.to_string()).await?;
otel.add_attribute(span_ctx.clone(), "knhk.workflow_engine.latency_ms".to_string(), latency_ms.to_string()).await?;

PatternOtelHelper::end_pattern_span(&otel, span_ctx, result.success).await?;
```

## Weaver Schema Compliance

All spans must match the Weaver schema defined in `registry/knhk-workflow-engine.yaml`:

- Span names must match schema definitions
- Attributes must match schema attribute IDs
- Attribute types must match schema types
- Required attributes must be present

## Performance Considerations

- Pattern spans should be lightweight (≤8 ticks for hot path)
- Use conditional instrumentation (only if OTEL is enabled)
- Batch attribute additions when possible
- Avoid blocking on span operations




