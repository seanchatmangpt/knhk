# Error Migration Guide

## Overview

This guide helps migrate from the old error system to the new advanced error handling system.

## Breaking Changes

### 1. Error Variant Structure Changes

**Old Format** (tuple variant):
```rust
WorkflowError::Internal(String)
WorkflowError::CaseNotFound(String)
WorkflowError::InvalidStateTransition { from: String, to: String }
```

**New Format** (struct variant with named fields):
```rust
WorkflowError::Internal { message: String }
WorkflowError::CaseNotFound { case_id: String }
WorkflowError::InvalidStateTransition { from: String, to: String }
```

### 2. Migration Script

Use this sed script to perform bulk migrations:

```bash
# Backup files first
find src -name "*.rs" -exec cp {} {}.backup \;

# Migrate Internal errors
find src -name "*.rs" -exec sed -i 's/WorkflowError::Internal(\([^)]*\))/WorkflowError::Internal { message: \1 }/g' {} \;

# Migrate CaseNotFound errors
find src -name "*.rs" -exec sed -i 's/WorkflowError::CaseNotFound(\([^)]*\))/WorkflowError::CaseNotFound { case_id: \1 }/g' {} \;

# Migrate Parse errors
find src -name "*.rs" -exec sed -i 's/WorkflowError::Parse(\([^)]*\))/WorkflowError::Parse { message: \1 }/g' {} \;

# Migrate Timeout errors (more complex - manual review needed)
# WorkflowError::Timeout -> WorkflowError::Timeout { resource_type: "unknown", duration_ms: 5000 }
```

### 3. Common Migration Patterns

#### Pattern 1: Simple Internal Error

```rust
// Old
Err(WorkflowError::Internal("database error".to_string()))

// New
Err(WorkflowError::Internal {
    message: "database error".to_string(),
})
```

#### Pattern 2: Case Not Found

```rust
// Old
Err(WorkflowError::CaseNotFound(case_id.to_string()))

// New
Err(WorkflowError::CaseNotFound {
    case_id: case_id.to_string(),
})
```

#### Pattern 3: Timeout Error

```rust
// Old
Err(WorkflowError::Timeout)

// New
Err(WorkflowError::Timeout {
    resource_type: "database".to_string(),
    duration_ms: 5000,
})
```

#### Pattern 4: Pattern Execution Error

```rust
// Old
Err(WorkflowError::PatternNotFound(pattern_id))

// New
Err(WorkflowError::PatternNotFound {
    pattern_id,
})
```

### 4. New Error Types to Use

When encountering errors that don't fit the old types, use these new variants:

```rust
// Resource allocation failures
Err(WorkflowError::ResourceAllocationFailed {
    resource_id: "resource-1".to_string(),
    reason: "max capacity reached".to_string(),
})

// Deadlock detection
Err(WorkflowError::DeadlockDetected {
    cycles_count: 2,
    cycles: vec![/* ... */],
})

// External system errors
Err(WorkflowError::ExternalSystem {
    system_name: "payment-gateway".to_string(),
    message: "connection timeout".to_string(),
})

// Recoverable errors
Err(WorkflowError::Recoverable {
    message: "temporary failure".to_string(),
})
```

### 5. Error Context Migration

Add context to errors for better debugging:

```rust
// Old
load_case(case_id)?;

// New
use knhk_workflow_engine::error::ErrorContext;

load_case(case_id)
    .context("Failed to load case for execution")?;
```

### 6. Recovery Strategy Migration

Use automatic recovery for transient errors:

```rust
// Old
let result = risky_operation().await?;

// New
use knhk_workflow_engine::error::try_execute_with_recovery;

let result = try_execute_with_recovery(async {
    risky_operation().await
}).await?;
```

## File-by-File Migration Checklist

### High Priority Files

1. `/home/user/knhk/rust/knhk-workflow-engine/src/executor/case.rs`
2. `/home/user/knhk/rust/knhk-workflow-engine/src/executor/workflow_execution.rs`
3. `/home/user/knhk/rust/knhk-workflow-engine/src/patterns/mod.rs`
4. `/home/user/knhk/rust/knhk-workflow-engine/src/state/manager.rs`

### Medium Priority Files

5. `/home/user/knhk/rust/knhk-workflow-engine/src/api/service/workflow.rs`
6. `/home/user/knhk/rust/knhk-workflow-engine/src/resource/allocation/allocator.rs`
7. `/home/user/knhk/rust/knhk-workflow-engine/src/security/auth.rs`

### Low Priority Files

8. Visualization, logging, and utility modules

## Testing After Migration

After migrating each file, run:

```bash
# Check compilation
cargo check --lib --no-default-features --features rdf,storage

# Run tests
cargo test --lib error::

# Run integration tests
cargo test --test error_handling_tests
```

## Rollback Procedure

If issues occur:

```bash
# Restore from backups
find src -name "*.rs.backup" -exec bash -c 'mv "$1" "${1%.backup}"' _ {} \;

# Or use git
git checkout -- src/
```

## Expected Benefits After Migration

1. **Better Error Messages**: Structured fields provide clearer context
2. **Automatic Recovery**: Transient errors automatically retried
3. **Error Chains**: Full error propagation tracking
4. **User-Friendly Messages**: `error.user_message()` for end users
5. **Severity Levels**: `error.severity()` for categorization

## Support

For migration issues, see:
- [docs/error_handling.md](./error_handling.md) - Full error handling guide
- [tests/error_handling_tests.rs](../tests/error_handling_tests.rs) - Test examples
- [src/error/mod.rs](../src/error/mod.rs) - Error type definitions
