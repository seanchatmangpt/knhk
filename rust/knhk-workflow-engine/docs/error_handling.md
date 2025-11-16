# Advanced Error Handling

This document describes the comprehensive error handling system in the KNHK Workflow Engine.

## Overview

The error handling system provides:

- **Rich Error Types**: Detailed error information with context
- **Automatic Recovery**: Transient errors are automatically retried
- **Error Chains**: Full error propagation tracking
- **User-Friendly Messages**: Clear error messages for end users
- **Severity Levels**: Error categorization (critical, error, warning, info)

## Error Hierarchy

```
WorkflowError (top-level)
├── Specification Errors
│   ├── SpecNotFound
│   └── InvalidSpecification
├── Case Errors
│   ├── CaseNotFound
│   ├── CaseExists
│   └── InvalidStateTransition
├── Pattern Errors
│   ├── PatternNotFound
│   └── PatternExecutionFailed
├── Resource Errors
│   ├── ResourceAllocationFailed
│   ├── ResourceUnavailable
│   ├── DeadlockDetected
│   └── Timeout
├── State Store Errors
│   └── StateStoreError
├── Connector Errors
│   ├── ConnectorError
│   └── ExternalSystem
├── Validation Errors
│   ├── RdfValidationError
│   └── Validation
└── Task Errors
    ├── TaskExecutionFailed
    └── CancellationFailed
```

## Basic Usage

### Simple Error Handling

```rust
use knhk_workflow_engine::error::{WorkflowError, WorkflowResult};

fn load_case(case_id: &str) -> WorkflowResult<Case> {
    get_case(case_id).ok_or_else(|| WorkflowError::CaseNotFound {
        case_id: case_id.to_string(),
    })
}
```

### Error Context

Add context to errors for better debugging:

```rust
use knhk_workflow_engine::error::ErrorContext;

async fn execute_workflow(case_id: &str) -> WorkflowResult<()> {
    let case = load_case(case_id)
        .context("Failed to load case for execution")?;

    execute(&case)
        .await
        .with_context(|| format!("Execution failed for case {}", case_id))?;

    Ok(())
}
```

## Recovery Strategies

### Automatic Retry

Transient errors are automatically retried with exponential backoff:

```rust
use knhk_workflow_engine::error::recover_from_error;

async fn connect_to_database() -> WorkflowResult<Connection> {
    let error = WorkflowError::Timeout {
        resource_type: "database".to_string(),
        duration_ms: 5000,
    };

    // Automatically retries up to 3 times with exponential backoff
    recover_from_error(&error, || {
        Box::pin(establish_connection())
    }).await
}
```

### Recovery Strategy Types

| Strategy | Use Case | Example |
|----------|----------|---------|
| **Retry** | Transient failures | Timeouts, network errors |
| **Fallback** | Alternative resources | Secondary database |
| **Wait** | Resource contention | Waiting for lock |
| **Degrade** | Graceful degradation | Reduced functionality |
| **FailFast** | Permanent failures | Parse errors |

### Custom Recovery

```rust
use knhk_workflow_engine::error::{Recoverable, RecoveryStrategy};

impl Recoverable for MyCustomError {
    fn is_recoverable(&self) -> bool {
        matches!(self, MyCustomError::Temporary(_))
    }

    fn recovery_strategy(&self) -> RecoveryStrategy {
        RecoveryStrategy::Retry {
            max_attempts: 5,
            backoff_ms: 1000,
            max_backoff_ms: 10000,
        }
    }
}
```

## Error Chains

Track error propagation through the call stack:

```rust
use knhk_workflow_engine::error::ErrorChain;

fn complex_operation() -> WorkflowResult<()> {
    database_operation().map_err(|e| {
        let chain = ErrorChain::new(e)
            .add_context("Database operation failed".to_string())
            .add_context("Complex operation failed".to_string());

        println!("Error chain:\n{}", chain.display_chain());
        println!("With backtrace:\n{}", chain.display_with_backtrace());

        WorkflowError::Internal {
            message: chain.display_chain(),
        }
    })
}
```

### Error Chain Output

```
0: Internal error: database connection failed
1: Database operation failed
2: Complex operation failed

Backtrace:
   0: std::backtrace::Backtrace::capture
   1: knhk_workflow_engine::error::ErrorChain::new
   2: complex_operation
   ...
```

## Try Blocks

Clean error handling with try blocks:

```rust
use knhk_workflow_engine::error::try_execute;

async fn workflow_execution(case_id: &str) -> WorkflowResult<String> {
    try_execute(async {
        let case = get_case(case_id).await?;
        validate_case(&case)?;
        let result = execute_case(&case).await?;
        save_result(&result).await?;
        Ok(result)
    }).await
}
```

### Try Execute with Recovery

```rust
use knhk_workflow_engine::error::try_execute_with_recovery;

async fn execute_with_auto_recovery(case_id: &str) -> WorkflowResult<()> {
    // Automatically retries on recoverable errors
    try_execute_with_recovery(async {
        risky_operation(case_id).await
    }).await
}
```

### Batch Operations

```rust
use knhk_workflow_engine::error::try_execute_all_or_fail;

async fn process_all_cases(case_ids: Vec<&str>) -> WorkflowResult<Vec<Case>> {
    let operations = case_ids
        .into_iter()
        .map(|id| get_case(id))
        .collect();

    // Fails fast on first error
    try_execute_all_or_fail(operations).await
}
```

## User-Friendly Messages

Convert technical errors to user-friendly messages:

```rust
use knhk_workflow_engine::error::WorkflowError;

let error = WorkflowError::DeadlockDetected {
    cycles_count: 2,
    cycles: vec![...],
};

// Technical message
println!("Technical: {}", error);
// Output: Deadlock detected: 2 cycles found

// User-friendly message
println!("User message: {}", error.user_message());
// Output: Deadlock detected in workflow: 2 resource cycles found. Please review resource allocation.
```

## Error Severity

Categorize errors by severity:

```rust
use knhk_workflow_engine::error::WorkflowError;

let error = WorkflowError::DeadlockDetected { ... };
match error.severity() {
    "critical" => alert_ops_team(&error),
    "error" => log_error(&error),
    "warning" => log_warning(&error),
    "info" => log_info(&error),
    _ => {}
}
```

## Error Sources

### State Store Errors

```rust
use knhk_workflow_engine::error::StateStoreError;

fn load_from_store(key: &str) -> Result<Data, StateStoreError> {
    store.get(key)?
        .ok_or(StateStoreError::KeyNotFound(key.to_string()))
}
```

### RDF Validation Errors

```rust
use knhk_workflow_engine::error::RdfValidationError;

fn validate_triple(triple: &Triple) -> Result<(), RdfValidationError> {
    if !triple.is_valid() {
        return Err(RdfValidationError::InvalidTriple(
            format!("Invalid triple: {:?}", triple)
        ));
    }
    Ok(())
}
```

### Connector Errors

```rust
use knhk_workflow_engine::error::ConnectorError;

async fn call_external_api() -> Result<Response, ConnectorError> {
    if rate_limited {
        return Err(ConnectorError::RateLimitExceeded {
            retry_after_ms: 5000,
        });
    }
    Ok(response)
}
```

## Best Practices

### 1. Use Specific Error Types

```rust
// ❌ Bad: Generic error
Err(WorkflowError::Internal {
    message: "something failed".to_string(),
})

// ✅ Good: Specific error
Err(WorkflowError::CaseNotFound {
    case_id: case_id.to_string(),
})
```

### 2. Add Context at Each Layer

```rust
// ❌ Bad: No context
database_operation()?;

// ✅ Good: Add context
database_operation()
    .context("Failed to persist workflow state")?;
```

### 3. Use Recovery for Transient Errors

```rust
// ❌ Bad: No retry
connect_to_service().await?;

// ✅ Good: Auto-retry
try_execute_with_recovery(async {
    connect_to_service().await
}).await?;
```

### 4. Provide User-Friendly Messages

```rust
// ❌ Bad: Technical error shown to user
return Err(error);

// ✅ Good: User-friendly message
return Err(WorkflowError::Internal {
    message: error.user_message(),
});
```

### 5. Log Error Chains

```rust
// ❌ Bad: Lose error context
error!("Error: {}", error);

// ✅ Good: Full error chain
let chain = ErrorChain::new(error);
error!("Error chain:\n{}", chain.display_with_backtrace());
```

## Testing

### Test Error Types

```rust
#[test]
fn test_error_is_recoverable() {
    let timeout = WorkflowError::Timeout {
        resource_type: "lock".to_string(),
        duration_ms: 5000,
    };
    assert!(timeout.is_recoverable());
}
```

### Test Recovery Strategies

```rust
#[tokio::test]
async fn test_auto_retry() {
    let counter = Arc::new(AtomicU32::new(0));
    let result = recover_from_error(&error, || {
        let count = counter.fetch_add(1, Ordering::SeqCst);
        if count < 2 {
            Box::pin(async { Err(error) })
        } else {
            Box::pin(async { Ok(()) })
        }
    }).await;

    assert!(result.is_ok());
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}
```

### Test Error Chains

```rust
#[test]
fn test_error_chain() {
    let error = WorkflowError::Internal {
        message: "db error".to_string(),
    };
    let chain = ErrorChain::new(error)
        .add_context("save failed".to_string());

    assert_eq!(chain.depth(), 2);
    assert!(chain.display_chain().contains("db error"));
}
```

## Conversion Guide

### From Old Error System

```rust
// Old system
#[derive(Error, Debug, Clone)]
pub enum WorkflowError {
    #[error("Case {0} not found")]
    CaseNotFound(String),
}

// New system
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Case not found: {case_id}")]
    CaseNotFound {
        case_id: String,
    },
}
```

### Migration Steps

1. **Replace error variants** with structured versions
2. **Add error context** at each layer
3. **Implement recovery** for transient errors
4. **Add error chains** for debugging
5. **Test all error paths** comprehensively

## Performance

The error handling system is designed for minimal overhead:

- **Zero-cost abstractions**: Error types use `#[inline]` where appropriate
- **Lazy evaluation**: Backtraces captured only when needed
- **Efficient recovery**: Retry logic uses async/await efficiently
- **Minimal allocations**: Error messages use `&str` where possible

## Summary

The advanced error handling system provides:

- ✅ Rich error types with context
- ✅ Automatic recovery strategies
- ✅ Full error chain tracking
- ✅ User-friendly error messages
- ✅ Severity-based categorization
- ✅ Comprehensive testing support
- ✅ Minimal performance overhead

For more examples, see the [tests/error_handling_tests.rs](../tests/error_handling_tests.rs) file.
