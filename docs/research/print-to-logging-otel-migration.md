# Research: Replacing Print Statements with Logging, OTEL, and Testing

## Executive Summary

This document provides comprehensive research on replacing all `println!`, `eprintln!`, `print!`, and `eprint!` statements in the KNHK codebase with proper structured logging using the `tracing` crate, OpenTelemetry integration, and testable verification patterns.

**Current State:**
- 259 files contain `println!` statements
- 43 files contain `eprintln!` statements
- Codebase already has OTEL infrastructure (`knhk-otel` crate)
- `tracing` crate is already integrated
- Tests exist but don't verify log output behavior

**Goal:**
- Replace all print statements with appropriate `tracing` macros
- Ensure all logs are captured in OTEL spans
- Create testable patterns for verifying logging behavior
- Follow Chicago TDD principles: test behavior, not implementation

---

## 1. Current State Analysis

### 1.1 Print Statement Categories

Based on codebase analysis, print statements fall into these categories:

#### A. Error Reporting (`eprintln!`)
**Location**: `rust/knhk-cli/src/main.rs`, `rust/knhk-cli/src/tracing.rs`
**Pattern**: Error messages, warnings, initialization failures
**Example**:
```rust
eprintln!("Warning: Failed to initialize tracing: {}", e);
```

#### B. Test Output (`println!`, `eprintln!`)
**Location**: Test files, especially `rust/knhk-workflow-engine/tests/dflss_validation.rs`
**Pattern**: Test progress, gap analysis, validation results
**Example**:
```rust
eprintln!("GAP: DoD compliance below target");
eprintln!("  Current: {:.1}% ({} criteria met)", dod_compliance, collector.dod_criteria_met);
```

#### C. Debug/Development Output (`println!`)
**Location**: Examples, benchmarks, development tools
**Pattern**: Debug information, progress indicators
**Example**:
```rust
println!("[TEST] Workflow Events Captured in XES Format");
```

#### D. User-Facing Output (`println!`)
**Location**: CLI commands, main binaries
**Pattern**: Command results, user feedback
**Example**: (Need to verify actual CLI output patterns)

### 1.2 Existing Logging Infrastructure

#### Tracing Setup (`rust/knhk-cli/src/tracing.rs`)
- ✅ `tracing-subscriber` configured with `EnvFilter`
- ✅ OTEL integration via `knhk-otel::init_tracer`
- ✅ JSON formatting support
- ✅ Environment-based log level control (`KNHK_TRACE`)

#### OTEL Infrastructure (`rust/knhk-otel/src/lib.rs`)
- ✅ `Tracer` struct with span/metric management
- ✅ OTLP exporter support
- ✅ Weaver schema validation
- ✅ Span attributes and events

#### Test Infrastructure (`rust/chicago-tdd-tools/`)
- ✅ Chicago TDD macros (`chicago_test!`, `chicago_async_test!`)
- ✅ OTEL validation utilities (`otel` feature)
- ✅ Weaver live validation (`weaver` feature)
- ⚠️ **Gap**: No log capture/verification utilities

---

## 2. Best Practices for Rust Logging with Tracing

### 2.1 Log Level Hierarchy

```rust
use tracing::{error, warn, info, debug, trace};

// Error: Critical failures requiring immediate attention
tracing::error!("Failed to initialize OTEL: {}", e);

// Warning: Recoverable issues or deprecations
tracing::warn!("Configuration file not found, using defaults");

// Info: Important operational events
tracing::info!("Workflow engine started", service.name = "knhk-workflow");

// Debug: Detailed diagnostic information
tracing::debug!("Processing {} triples", triple_count);

// Trace: Very verbose diagnostic information
tracing::trace!("Entering hot path operation");
```

### 2.2 Structured Logging with Fields

**❌ Bad: String formatting**
```rust
println!("Processing {} triples in {} ticks", count, ticks);
```

**✅ Good: Structured fields**
```rust
tracing::info!(
    triple_count = count,
    tick_count = ticks,
    "Processing triples"
);
```

**Benefits:**
- Fields are queryable in log aggregation systems
- Better performance (no string formatting unless logged)
- OTEL automatically captures fields as span attributes

### 2.3 Context Propagation with Spans

**❌ Bad: Standalone log**
```rust
eprintln!("GAP: DoD compliance below target");
```

**✅ Good: Span with attributes**
```rust
tracing::warn!(
    gap_type = "dod_compliance",
    current_percent = dod_compliance,
    target_percent = 85.0,
    criteria_met = collector.dod_criteria_met,
    "DoD compliance below target"
);
```

### 2.4 Instrumentation Macros

**For functions:**
```rust
#[tracing::instrument]
fn process_workflow(workflow_id: &str) -> Result<()> {
    // Function name, args automatically logged
    // Return value logged on success/error
}
```

**With custom fields:**
```rust
#[tracing::instrument(
    fields(
        workflow_id = %workflow_id,
        case_count = case_count
    )
)]
fn execute_workflow(workflow_id: &str, case_count: usize) -> Result<()> {
    // Custom fields added to span
}
```

**With skip fields:**
```rust
#[tracing::instrument(skip(sensitive_data))]
fn process_data(sensitive_data: &[u8]) -> Result<()> {
    // sensitive_data not logged
}
```

---

## 3. OTEL Integration Patterns

### 3.1 Span Creation for Operations

**Pattern: Hot Path Operations**
```rust
use tracing::{info_span, Instrument};

let span = info_span!(
    "knhk.operation.execute",
    knhk.operation.type = "ASK_SP",
    knhk.operation.predicate = %predicate_id,
    knhk.operation.ticks = tick_count,
    knhk.operation.runtime_class = "R1"
);

let _guard = span.enter();
// Operation code here
// Span automatically ends when guard drops
```

**Pattern: Workflow Operations**
```rust
let span = info_span!(
    "knhk.workflow.execute",
    knhk.workflow.id = %workflow_id,
    knhk.case.id = %case_id,
    knhk.pattern.id = pattern_id
);
```

### 3.2 Events for Discrete Occurrences

**Pattern: State Changes**
```rust
tracing::event!(
    tracing::Level::INFO,
    knhk.event.type = "fiber.parked",
    knhk.fiber.shard_id = shard_id,
    knhk.fiber.cause = %cause,
    "Fiber parked to W1"
);
```

**Pattern: Errors**
```rust
tracing::error!(
    error.kind = "guard_violation",
    error.message = %error_msg,
    knhk.guard.constraint = "max_run_len",
    knhk.guard.value = run_len,
    knhk.guard.limit = 8,
    "Guard constraint violated"
);
```

### 3.3 Metrics Recording

**Pattern: Counter Metrics**
```rust
use knhk_otel::MetricsHelper;

MetricsHelper::record_operation(&mut tracer, "workflow.execute", true);
MetricsHelper::record_config_load(&mut tracer, "file");
```

**Pattern: Histogram Metrics**
```rust
tracing::info!(
    knhk.metric.name = "operation.duration",
    knhk.metric.value = duration_ms,
    knhk.metric.unit = "ms",
    "Operation completed"
);
```

---

## 4. Testing Strategies for Logging

### 4.1 Current Gap: No Log Capture in Tests

**Problem**: Tests use `println!`/`eprintln!` but don't verify log output behavior.

**Example from `dflss_validation.rs`:**
```rust
eprintln!("GAP: DoD compliance below target");
// No verification that this log was emitted
```

### 4.2 Solution: Test Subscriber Pattern

**Pattern 1: Using `tracing-subscriber::fmt::TestWriter`**

```rust
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[test]
fn test_logging_behavior() {
    // Arrange: Set up test subscriber that captures logs
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new("debug"))
        .with(fmt::layer().with_test_writer());
    
    let _ = tracing::subscriber::set_default(subscriber);
    
    // Act: Execute code that logs
    tracing::warn!("Test warning message");
    
    // Assert: Verify log was emitted (check subscriber output)
    // Note: This requires custom test utilities
}
```

**Pattern 2: Using `tracing::collect` (tracing 0.1 API)**

```rust
use tracing::{collect::with_default, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

#[test]
fn test_span_creation() {
    // Arrange: Create test collector
    let collector = Registry::default()
        .with(tracing_subscriber::fmt::layer().with_test_writer());
    
    // Act & Assert: Verify span creation
    with_default(collector, || {
        let span = tracing::info_span!("test.operation");
        let _guard = span.enter();
        
        tracing::info!("Operation started");
        
        // Verify span exists in collector
        // Note: Requires custom test utilities to inspect collector state
    });
}
```

**Pattern 3: Custom Test Utilities (Recommended)**

Create test utilities in `chicago-tdd-tools`:

```rust
// rust/chicago-tdd-tools/src/otel/log_capture.rs

use std::sync::{Arc, Mutex};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct LogCapture {
    logs: Arc<Mutex<Vec<String>>>,
}

impl LogCapture {
    pub fn new() -> Self {
        let logs = Arc::new(Mutex::new(Vec::new()));
        
        // Set up subscriber that captures logs
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("debug"))
            .with(fmt::layer()
                .with_writer(TestWriter::new(logs.clone()))
                .json());
        
        let _ = tracing::subscriber::set_default(subscriber);
        
        Self { logs }
    }
    
    pub fn assert_log_contains(&self, pattern: &str) {
        let logs = self.logs.lock().unwrap();
        assert!(
            logs.iter().any(|log| log.contains(pattern)),
            "Expected log containing '{}' not found",
            pattern
        );
    }
    
    pub fn assert_log_count(&self, expected: usize) {
        let logs = self.logs.lock().unwrap();
        assert_eq!(
            logs.len(),
            expected,
            "Expected {} logs, found {}",
            expected,
            logs.len()
        );
    }
}
```

### 4.3 OTEL Span Verification in Tests

**Pattern: Verify Span Attributes**

```rust
use chicago_tdd_tools::otel::validate_span;

#[test]
fn test_operation_creates_span() {
    // Arrange: Set up OTEL test environment
    let mut tracer = knhk_otel::Tracer::new();
    
    // Act: Execute operation
    let span_ctx = tracer.start_span(
        "knhk.operation.execute".to_string(),
        None
    );
    tracer.add_attribute(
        span_ctx.clone(),
        "knhk.operation.type".to_string(),
        "ASK_SP".to_string()
    );
    tracer.end_span(span_ctx, knhk_otel::SpanStatus::Ok);
    
    // Assert: Verify span structure
    let spans = tracer.spans();
    assert_eq!(spans.len(), 1);
    
    let span = &spans[0];
    assert_eq!(span.name, "knhk.operation.execute");
    assert_eq!(
        span.attributes.get("knhk.operation.type"),
        Some(&"ASK_SP".to_string())
    );
    
    // Validate against Weaver schema
    knhk_otel::validation::validate_span_structure(span)
        .expect("Span should be valid");
}
```

**Pattern: Verify Log Events in Spans**

```rust
#[test]
fn test_error_logs_create_span_events() {
    // Arrange: Set up test environment
    let mut tracer = knhk_otel::Tracer::new();
    
    // Act: Log error with context
    tracing::error!(
        error.kind = "guard_violation",
        knhk.guard.constraint = "max_run_len",
        "Guard violation occurred"
    );
    
    // Assert: Verify error was captured as span event
    // Note: Requires integration with tracing-opentelemetry layer
}
```

### 4.4 Weaver Validation in Tests

**Pattern: Live Validation (E2E)**

```rust
#[test]
#[ignore] // Requires Weaver binary and OTLP collector
fn test_telemetry_validates_against_weaver() {
    // Arrange: Start Weaver live-check
    let weaver = knhk_otel::WeaverLiveCheck::new()
        .with_otlp_port(4317)
        .with_admin_port(4320);
    
    // Act: Execute operation that emits telemetry
    execute_workflow("test-workflow");
    
    // Assert: Weaver validates telemetry against schema
    // Weaver will exit with non-zero code if validation fails
    let validation_result = weaver.wait_for_validation();
    assert!(validation_result.is_ok(), "Weaver validation should pass");
}
```

---

## 5. Migration Patterns

### 5.1 Error Reporting Migration

**Before:**
```rust
eprintln!("Warning: Failed to initialize tracing: {}", e);
```

**After:**
```rust
tracing::warn!(
    error.message = %e,
    component = "tracing",
    "Failed to initialize tracing"
);
```

### 5.2 Test Output Migration

**Before:**
```rust
eprintln!("GAP: DoD compliance below target");
eprintln!("  Current: {:.1}% ({} criteria met)", dod_compliance, collector.dod_criteria_met);
eprintln!("  Target: ≥85% (≥28 criteria required)");
```

**After:**
```rust
tracing::warn!(
    gap_type = "dod_compliance",
    current_percent = dod_compliance,
    target_percent = 85.0,
    criteria_met = collector.dod_criteria_met,
    criteria_required = 28,
    "DoD compliance below target"
);
```

**With Test Verification:**
```rust
#[test]
fn test_dod_compliance_gap_logged() {
    // Arrange: Set up log capture
    let log_capture = LogCapture::new();
    let collector = create_test_collector();
    
    // Act: Execute validation
    validate_dod_compliance(&collector);
    
    // Assert: Verify gap was logged
    log_capture.assert_log_contains("DoD compliance below target");
    log_capture.assert_log_contains(&format!("{}", collector.dod_criteria_met));
}
```

### 5.3 Debug Output Migration

**Before:**
```rust
println!("[TEST] Workflow Events Captured in XES Format");
```

**After:**
```rust
tracing::debug!(
    test.name = "workflow_events_captured_in_xes",
    "Workflow events captured in XES format"
);
```

**Or remove entirely if not needed:**
- Test names are already in function names
- Use `tracing::instrument` macro for automatic logging

### 5.4 User-Facing Output Migration

**Before:**
```rust
println!("Extracted {} triples in {} ticks", count, ticks);
```

**After:**
```rust
tracing::info!(
    triple_count = count,
    tick_count = ticks,
    "Extracted triples"
);
```

**For CLI output specifically:**
```rust
// Use tracing for logs, but keep user-facing output separate
// Option 1: Use tracing for logs, println for user output
println!("Extracted {} triples", count);
tracing::info!(triple_count = count, tick_count = ticks, "Triple extraction completed");

// Option 2: Use structured output format (JSON) for programmatic consumption
if output_format == "json" {
    println!("{}", serde_json::json!({
        "triple_count": count,
        "tick_count": ticks
    }));
} else {
    println!("Extracted {} triples in {} ticks", count, ticks);
}
tracing::info!(triple_count = count, tick_count = ticks, "Triple extraction completed");
```

---

## 6. Implementation Recommendations

### 6.1 Phase 1: Create Test Utilities

**Priority: HIGH**

Create log capture utilities in `chicago-tdd-tools`:

1. **`LogCapture` struct** - Captures logs during test execution
2. **Assertion macros** - `assert_log_contains!`, `assert_log_count!`
3. **OTEL span verification** - Verify spans created from logs
4. **Weaver integration** - Test utilities for Weaver validation

**Files to create:**
- `rust/chicago-tdd-tools/src/otel/log_capture.rs`
- `rust/chicago-tdd-tools/src/otel/assertions.rs`

### 6.2 Phase 2: Migrate Error Reporting

**Priority: HIGH**

Replace all `eprintln!` statements with `tracing::error!` or `tracing::warn!`:

1. CLI error handling (`rust/knhk-cli/src/main.rs`, `rust/knhk-cli/src/tracing.rs`)
2. Initialization failures
3. Configuration errors
4. Guard violations

**Files to update:**
- `rust/knhk-cli/src/main.rs`
- `rust/knhk-cli/src/tracing.rs`
- All command modules with error reporting

### 6.3 Phase 3: Migrate Test Output

**Priority: MEDIUM**

Replace test `println!`/`eprintln!` with structured logging:

1. Test progress indicators → `tracing::debug!`
2. Gap analysis → `tracing::warn!` with structured fields
3. Validation results → `tracing::info!` with metrics

**Files to update:**
- `rust/knhk-workflow-engine/tests/dflss_validation.rs`
- All test files with print statements

### 6.4 Phase 4: Migrate Debug/Development Output

**Priority: LOW**

Replace development `println!` statements:

1. Examples → `tracing::debug!` or remove
2. Benchmarks → Keep minimal output, use `tracing::trace!`
3. Development tools → Structured logging

### 6.5 Phase 5: Add Test Verification

**Priority: HIGH**

Add tests that verify logging behavior:

1. Test that errors are logged correctly
2. Test that spans are created with correct attributes
3. Test that Weaver validation passes
4. Test that log levels are respected

---

## 7. Testing Examples

### 7.1 Test: Error Logging Behavior

```rust
use chicago_tdd_tools::otel::LogCapture;

#[test]
fn test_error_logged_on_initialization_failure() {
    // Arrange
    let log_capture = LogCapture::new();
    
    // Act: Simulate initialization failure
    let result = init_tracing();
    
    // Assert: Error was logged
    if result.is_err() {
        log_capture.assert_log_contains("Failed to initialize");
        log_capture.assert_log_contains("tracing");
    }
}
```

### 7.2 Test: Span Creation from Logs

```rust
#[test]
fn test_operation_logs_create_spans() {
    // Arrange
    let mut tracer = knhk_otel::Tracer::new();
    
    // Act: Log operation with structured fields
    tracing::info!(
        knhk.operation.type = "ASK_SP",
        knhk.operation.predicate = "http://example.org/p",
        "Operation executed"
    );
    
    // Assert: Span was created with correct attributes
    // Note: Requires tracing-opentelemetry integration
    let spans = tracer.spans();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "knhk.operation.execute");
}
```

### 7.3 Test: Weaver Schema Validation

```rust
#[test]
#[ignore] // Requires Weaver binary
fn test_telemetry_validates_against_weaver_schema() {
    // Arrange: Start Weaver live-check
    let weaver = knhk_otel::WeaverLiveCheck::new()
        .with_otlp_port(4317);
    
    // Act: Execute operation
    execute_hot_path_operation();
    
    // Assert: Weaver validates telemetry
    let result = weaver.wait_for_validation();
    assert_ok!(&result, "Weaver validation should pass");
}
```

---

## 8. Dependencies and Features

### 8.1 Required Dependencies

**Already present:**
- `tracing = "0.1"`
- `tracing-subscriber = "0.3"`
- `tracing-opentelemetry = "0.32"`
- `opentelemetry = "0.31"`
- `opentelemetry_sdk = "0.31"`
- `opentelemetry-otlp = "0.31"`

**May need to add:**
- `tracing-appender` - For test log capture (if needed)
- `tracing-test` - Alternative test utilities (evaluate)

### 8.2 Feature Gates

Ensure proper feature gating:

```rust
#[cfg(feature = "otel")]
use tracing::{info, warn, error};

#[cfg(not(feature = "otel"))]
// Fallback to no-op or basic logging
```

---

## 9. Performance Considerations

### 9.1 Zero-Cost When Disabled

Tracing macros are zero-cost when log level is disabled:

```rust
tracing::debug!("Expensive computation: {}", expensive_format());
// If log level is INFO or higher, expensive_format() is never called
```

### 9.2 Hot Path Considerations

For hot path operations (≤8 ticks), minimize logging:

```rust
// ❌ Bad: Logging in hot path
#[tracing::instrument] // Adds overhead
fn hot_path_operation() {
    tracing::debug!("Hot path"); // String formatting overhead
}

// ✅ Good: Log outside hot path
fn hot_path_operation() {
    // No logging in hot path
}

fn execute_with_logging() {
    tracing::debug!("Starting hot path operation");
    let result = hot_path_operation();
    tracing::debug!(result = ?result, "Hot path completed");
}
```

---

## 10. Migration Checklist

### 10.1 Pre-Migration

- [ ] Create `LogCapture` utilities in `chicago-tdd-tools`
- [ ] Add log assertion macros
- [ ] Document logging patterns for team
- [ ] Set up Weaver schema validation in CI

### 10.2 Migration Steps

- [ ] Phase 1: Create test utilities
- [ ] Phase 2: Migrate error reporting (`eprintln!` → `tracing::error!`/`warn!`)
- [ ] Phase 3: Migrate test output (`println!`/`eprintln!` → structured logging)
- [ ] Phase 4: Migrate debug output (`println!` → `tracing::debug!` or remove)
- [ ] Phase 5: Add test verification for logging behavior

### 10.3 Post-Migration

- [ ] Verify all tests pass
- [ ] Verify Weaver validation passes
- [ ] Verify log aggregation works (if applicable)
- [ ] Update documentation with logging patterns
- [ ] Remove all `println!`/`eprintln!` statements

---

## 11. References

### 11.1 Documentation

- [tracing crate documentation](https://docs.rs/tracing/)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/)
- [OpenTelemetry Rust documentation](https://opentelemetry.io/docs/instrumentation/rust/)
- [Weaver documentation](https://github.com/open-telemetry/semantic-conventions)

### 11.2 Codebase References

- `rust/knhk-cli/src/tracing.rs` - Tracing initialization
- `rust/knhk-otel/src/lib.rs` - OTEL infrastructure
- `rust/knhk-otel/tests/chicago_tdd_otel_integration.rs` - OTEL test patterns
- `rust/chicago-tdd-tools/src/otel/` - OTEL test utilities (to be created)

---

## 12. Next Steps

1. **Create test utilities** - Implement `LogCapture` and assertion macros
2. **Migrate high-priority files** - Start with error reporting in CLI
3. **Add test verification** - Ensure logging behavior is tested
4. **Iterate** - Migrate remaining files following established patterns
5. **Validate** - Run Weaver validation to ensure schema compliance

---

**Document Status**: Research Complete
**Last Updated**: 2025-01-XX
**Author**: AI Assistant
**Review Status**: Pending Team Review

