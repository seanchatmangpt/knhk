# Error Handling

KNHK uses comprehensive error handling with structured diagnostics.

## Overview

All fallible operations return `Result<T, E>` with:
- Structured error types
- Error codes
- Error messages
- Retryability checking
- OTEL correlation

## Error Types

### ConnectorError

```rust
#[derive(Error, Debug)]
pub enum ConnectorError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
```

### ProcessingError

```rust
#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("Guard violation: {0}")]
    GuardViolation(String),
    
    #[error("Performance budget exceeded: {0} ticks")]
    PerformanceBudgetExceeded(u64),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidationFailed(String),
}
```

## Best Practices

### Zero Unwrap/Expect

**Policy**: No `.unwrap()` or `.expect()` in production code.

**Enforcement**:
- Clippy lints: `#![deny(clippy::unwrap_used)]`
- Pre-commit hook: Blocks commits with unwrap/expect
- Pre-push hook: Validates entire codebase

### Acceptable Exceptions

| Pattern | Rationale | Documentation Required |
|---------|-----------|----------------------|
| Mutex poisoning | Unrecoverable error | `#![allow(clippy::expect_used)]` + link to Rust docs |
| Singleton init failure | Unrecoverable deployment error | `#![allow(clippy::expect_used)]` + explanation |
| Default trait fallback | Cannot return Result | `#![allow(clippy::expect_used)]` + explanation |

## Error Propagation

### Using `?` Operator

```rust
fn process(input: &str) -> Result<u64, ProcessingError> {
    let parsed = input.parse()?;
    let validated = validate(parsed)?;
    Ok(validated)
}
```

### Error Context

```rust
use anyhow::Context;

let result = operation()
    .context("Failed to process input")?;
```

## OTEL Integration

Errors are correlated with OTEL spans:

```rust
use knhk_otel::Tracer;

let span = tracer.start_span("operation".to_string(), None);
let result = operation();
if let Err(e) = result {
    tracer.record_error(&span, &e);
}
```

## Related Documentation

- [Chicago TDD](chicago-tdd.md) - Testing methodology
- [Code Organization](code-organization.md) - Code structure
