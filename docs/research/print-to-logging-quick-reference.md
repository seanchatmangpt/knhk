# Quick Reference: Print → Logging Migration Guide

## Quick Decision Tree

```
Is it an error/warning?
├─ Yes → Use tracing::error! or tracing::warn!
│   └─ Include structured fields (error.kind, error.message, etc.)
│
├─ No → Is it user-facing output?
│   ├─ Yes → Keep println! for CLI output, add tracing::info! for logs
│   │
│   └─ No → Is it test output?
│       ├─ Yes → Use tracing::warn! (gaps) or tracing::info! (results)
│       │   └─ Add test verification with LogCapture
│       │
│       └─ No → Is it debug/development?
│           ├─ Yes → Use tracing::debug! or tracing::trace!
│           │   └─ Or remove if not needed
│           │
│           └─ No → Remove println!
```

## Migration Patterns

### Error Reporting

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

### Test Gap Reporting

**Before:**
```rust
eprintln!("GAP: DoD compliance below target");
eprintln!("  Current: {:.1}%", dod_compliance);
```

**After:**
```rust
tracing::warn!(
    gap_type = "dod_compliance",
    current_percent = dod_compliance,
    target_percent = 85.0,
    "DoD compliance below target"
);
```

### Operation Logging

**Before:**
```rust
println!("Processing {} triples", count);
```

**After:**
```rust
tracing::info!(
    triple_count = count,
    "Processing triples"
);
```

### Function Instrumentation

**Before:**
```rust
fn process_workflow(id: &str) -> Result<()> {
    println!("Processing workflow: {}", id);
    // ... code ...
}
```

**After:**
```rust
#[tracing::instrument(fields(workflow_id = %id))]
fn process_workflow(id: &str) -> Result<()> {
    // ... code ...
    // Function name, args, and result automatically logged
}
```

## Log Levels

| Level | Use Case | Example |
|-------|----------|---------|
| `error!` | Critical failures | Initialization failures, unrecoverable errors |
| `warn!` | Recoverable issues | Configuration fallbacks, gap analysis |
| `info!` | Important events | Workflow started, operation completed |
| `debug!` | Diagnostic info | Detailed operation state, test progress |
| `trace!` | Very verbose | Hot path entry/exit, detailed state dumps |

## Structured Fields

### Standard Field Names

- `error.kind` - Error type identifier
- `error.message` - Error message (use `%` for Display)
- `knhk.operation.type` - Operation type
- `knhk.workflow.id` - Workflow identifier
- `knhk.case.id` - Case identifier
- `knhk.guard.constraint` - Guard constraint name
- `knhk.guard.value` - Guard constraint value

### Field Formatting

```rust
// Display formatting (for types implementing Display)
tracing::info!(workflow_id = %id, "Workflow started");

// Debug formatting (for any type)
tracing::debug!(state = ?workflow_state, "Workflow state");

// String formatting (avoid if possible)
tracing::info!(message = format!("Processing {}", count), "Operation");
```

## Testing Logging

### Basic Test Pattern

```rust
use chicago_tdd_tools::otel::LogCapture;

#[test]
fn test_error_logged() {
    // Arrange
    let log_capture = LogCapture::new();
    
    // Act
    execute_operation_that_logs();
    
    // Assert
    log_capture.assert_log_contains("expected message");
}
```

### OTEL Span Verification

```rust
#[test]
fn test_span_created() {
    // Arrange
    let mut tracer = knhk_otel::Tracer::new();
    
    // Act
    execute_operation();
    
    // Assert
    let spans = tracer.spans();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "expected.span.name");
    knhk_otel::validation::validate_span_structure(&spans[0])
        .expect("Span should be valid");
}
```

## Common Mistakes

### ❌ Don't Do This

```rust
// String formatting in log message
tracing::info!("Processing {} triples", count);

// Logging in hot path
#[tracing::instrument]
fn hot_path_operation() { /* ... */ }

// Using println! for errors
println!("Error: {}", e);
```

### ✅ Do This Instead

```rust
// Structured fields
tracing::info!(triple_count = count, "Processing triples");

// Log outside hot path
fn hot_path_operation() { /* ... */ }
fn execute_with_logging() {
    tracing::debug!("Starting hot path");
    hot_path_operation();
    tracing::debug!("Hot path completed");
}

// Use tracing::error! for errors
tracing::error!(error.message = %e, "Operation failed");
```

## File-by-File Checklist

### High Priority (Error Reporting)

- [ ] `rust/knhk-cli/src/main.rs` - Error handling
- [ ] `rust/knhk-cli/src/tracing.rs` - Initialization errors
- [ ] All command modules - Command errors

### Medium Priority (Test Output)

- [ ] `rust/knhk-workflow-engine/tests/dflss_validation.rs` - Gap analysis
- [ ] All test files with `eprintln!` - Test output

### Low Priority (Debug Output)

- [ ] Example files - Debug output
- [ ] Benchmark files - Performance output
- [ ] Development tools - Diagnostic output

## Verification Commands

```bash
# Find all println! statements
grep -r "println!" rust/ --include="*.rs" | wc -l

# Find all eprintln! statements
grep -r "eprintln!" rust/ --include="*.rs" | wc -l

# Find all tracing:: macros (after migration)
grep -r "tracing::" rust/ --include="*.rs" | wc -l

# Run tests to verify logging behavior
make test-rust

# Run Weaver validation
weaver registry live-check --registry registry/
```

