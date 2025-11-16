# KNHK Hook Engine Integration Guide

## Overview

The KNHK Hook Engine provides a comprehensive execution layer (μ via KNHK) with:
- Hook execution with latency bounds (≤8 ticks)
- 43 YAWL pattern library
- Guard enforcement (Q invariants)
- Receipt generation system
- Snapshot versioning (Σ)
- Full OTEL integration

## Architecture

### Components

1. **Hook Engine** (`engine/hook_engine.rs`)
   - Core hook execution with tick accounting
   - OTEL span integration
   - Pattern execution coordination

2. **Pattern Library** (`engine/pattern_library.rs`)
   - All 43 Van der Aalst workflow patterns
   - O(1) pattern lookup
   - Hot path eligibility tracking

3. **Latency-Bounded Scheduler** (`engine/scheduler.rs`)
   - Enforces Chatman constant (≤8 ticks)
   - Priority-based execution
   - Statistics tracking

4. **Guard Enforcement** (`guards/`)
   - Invariant checking (Q)
   - SHACL validation
   - Precondition/postcondition enforcement

5. **Receipt System** (`receipts/`)
   - Cryptographic receipt generation
   - Immutable log storage
   - Query API

6. **Snapshot System** (`snapshots/`)
   - Σ versioning with SHA-256 hashing
   - Atomic pointer updates
   - Rollback mechanism

## Quick Start

### Basic Usage

```rust
use knhk_workflow_engine::{
    engine::{HookEngine, PatternLibrary},
    hooks::{HookRegistry, HookContext, HookType},
    guards::InvariantChecker,
    receipts::{ReceiptGenerator, ReceiptStore},
    snapshots::SnapshotVersioning,
};
use knhk_otel::Tracer;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let hook_registry = Arc::new(HookRegistry::new());
    let tracer = Arc::new(RwLock::new(Tracer::new()));
    let hook_engine = HookEngine::new(hook_registry.clone(), tracer.clone());

    // Initialize pattern library
    let pattern_library = hook_engine.pattern_library();
    pattern_library.initialize().await?;

    // Initialize guards
    let invariant_checker = InvariantChecker::new();

    // Initialize receipts
    let receipt_generator = ReceiptGenerator::new();
    let receipt_store = ReceiptStore::new();

    // Initialize snapshots
    let snapshot_versioning = SnapshotVersioning::new();

    // Execute a hook
    let context = HookContext {
        hook_type: HookType::BeforeTaskExecution,
        case_id: None,
        workflow_spec_id: None,
        task_id: Some("task-1".to_string()),
        pattern_id: None,
        data: serde_json::json!({"input": "data"}),
    };

    let result = hook_engine.execute_hook(
        HookType::BeforeTaskExecution,
        context
    ).await?;

    println!("Hook executed in {} ticks", result.ticks_used);
    println!("Met hot path constraint: {}", result.met_hot_path_constraint);

    Ok(())
}
```

### Pattern Execution

```rust
use knhk_workflow_engine::engine::HookEngine;

// Execute pattern with hooks
let input_data = serde_json::json!({"task": "process"});
let pattern_id = 1; // Sequence pattern

let output = hook_engine.execute_pattern(
    pattern_id,
    input_data
).await?;

println!("Pattern output: {}", output);
```

### Guard Enforcement

```rust
use knhk_workflow_engine::guards::{InvariantChecker, Invariant, InvariantType};
use std::sync::Arc;

let checker = InvariantChecker::new();

// Register invariant
let invariant = Invariant {
    id: "positive-balance".to_string(),
    invariant_type: InvariantType::Precondition,
    name: "Positive Balance".to_string(),
    description: "Account balance must be positive".to_string(),
    predicate: Arc::new(|state| Box::pin(async move {
        state.get("balance")
            .and_then(|v| v.as_f64())
            .map(|b| b > 0.0)
            .unwrap_or(false)
    })),
    severity: 8,
};

checker.register_invariant(invariant).await?;

// Check preconditions
let state = serde_json::json!({"balance": 100.0});
let results = checker.check_preconditions(&state).await?;

// Validate all passed
InvariantChecker::validate_results(&results)?;
```

### Receipt Generation

```rust
use knhk_workflow_engine::receipts::{ReceiptGenerator, ReceiptStore};

let generator = ReceiptGenerator::new();
let store = ReceiptStore::new();

// Generate receipt
let receipt = generator.generate_receipt(
    "sigma-123".to_string(),
    &serde_json::json!({"input": "data"}),
    &serde_json::json!({"output": "result"}),
    vec!["guard1".to_string()],
    vec![], // No failures
    5, // Ticks used
)?;

// Store receipt
store.store(receipt.clone()).await?;

// Verify receipt
assert!(receipt.verify_signature());
assert!(receipt.is_valid());

// Query receipts
let results = store.get_by_sigma("sigma-123").await;
println!("Found {} receipts", results.len());

// Get statistics
let stats = store.get_stats().await;
println!("Total receipts: {}", stats.total_receipts);
println!("Average ticks: {}", stats.avg_ticks);
```

### Snapshot Management

```rust
use knhk_workflow_engine::snapshots::SnapshotVersioning;

let versioning = SnapshotVersioning::new();

// Create snapshots
let v1_content = serde_json::json!({"state": "v1"});
let v1_id = versioning.create_snapshot(v1_content).await?;

let v2_content = serde_json::json!({"state": "v2"});
let v2_id = versioning.create_snapshot(v2_content).await?;

// Get current snapshot
let current = versioning.get_current_snapshot().await.expect("No current");
println!("Current version: {}", current.metadata.version);

// Rollback
let previous_id = versioning.rollback().await?;
println!("Rolled back to: {}", previous_id);

// Get history
let history = versioning.get_history().await;
println!("Snapshot history: {} versions", history.len());
```

## Performance Constraints

### Chatman Constant (≤8 Ticks)

All hot path operations must complete within 8 ticks:

```rust
use knhk_workflow_engine::engine::{HookEngine, MAX_HOT_PATH_TICKS};

let result = hook_engine.execute_hook(hook_type, context).await?;

if result.ticks_used <= MAX_HOT_PATH_TICKS {
    println!("✅ Hot path constraint met");
} else {
    println!("⚠️  Exceeded hot path: {} ticks", result.ticks_used);
}
```

### Pattern Hot Path Eligibility

Check if a pattern is hot path eligible:

```rust
let pattern = pattern_library.get_pattern(pattern_id).await.expect("Not found");

if pattern.hot_path_eligible {
    println!("Pattern {} is hot path eligible", pattern.name);
}
```

## OTEL Integration

### Automatic Span Creation

Hook execution automatically creates OTEL spans:

```rust
// Spans are created with attributes:
// - hook.type
// - task.id (if present)
// - pattern.id (if present)

// Metrics are recorded:
// - knhk.hook.latency.ticks
// - knhk.receipt.generated
// - knhk.guard.violation
```

### Weaver Validation

All telemetry is validated against Weaver schemas:

```bash
# Check schemas
weaver registry check -r registry/

# Live validation
weaver registry live-check --registry registry/
```

## Testing

### Unit Tests

All components include comprehensive tests:

```bash
# Run hook engine tests
cargo test --package knhk-workflow-engine engine::

# Run guard tests
cargo test --package knhk-workflow-engine guards::

# Run receipt tests
cargo test --package knhk-workflow-engine receipts::

# Run snapshot tests
cargo test --package knhk-workflow-engine snapshots::
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_workflow() {
    let registry = Arc::new(HookRegistry::new());
    let tracer = Arc::new(RwLock::new(Tracer::new()));
    let engine = HookEngine::new(registry, tracer);

    // Initialize pattern library
    engine.pattern_library().initialize().await.expect("Init failed");

    // Execute pattern
    let result = engine.execute_pattern(
        1,
        serde_json::json!({"test": "data"})
    ).await.expect("Execution failed");

    assert!(result.is_object());
}
```

## Best Practices

### 1. Always Use Guards

Enforce preconditions and postconditions:

```rust
// Before operation
let pre_results = checker.check_preconditions(&state).await?;
InvariantChecker::validate_results(&pre_results)?;

// Execute operation
let new_state = perform_operation(state)?;

// After operation
let post_results = checker.check_postconditions(&new_state).await?;
InvariantChecker::validate_results(&post_results)?;
```

### 2. Generate Receipts for Audit

Always generate receipts for auditable operations:

```rust
let receipt = generator.generate_receipt(
    sigma_id,
    &input,
    &output,
    guards_checked,
    guards_failed,
    ticks_used,
)?;

store.store(receipt).await?;
```

### 3. Use Snapshots for State Management

Create snapshots at critical points:

```rust
// Before major operation
let before_id = versioning.create_snapshot(current_state).await?;

// Perform operation
let result = risky_operation()?;

if result.is_err() {
    // Rollback on failure
    versioning.rollback_to(&before_id).await?;
}
```

### 4. Monitor Performance

Track tick consumption:

```rust
let stats = scheduler.get_stats();

if stats.violation_rate > 0.05 {
    tracing::warn!("High constraint violation rate: {:.2}%", stats.violation_rate * 100.0);
}
```

## Error Handling

All operations return `WorkflowResult<T>`:

```rust
use knhk_workflow_engine::error::{WorkflowError, WorkflowResult};

fn my_operation() -> WorkflowResult<()> {
    // Guards failed
    if guards_failed {
        return Err(WorkflowError::GuardViolation("Precondition failed".to_string()));
    }

    // Receipt generation failed
    if !receipt.verify_signature() {
        return Err(WorkflowError::ReceiptGenerationFailed("Invalid signature".to_string()));
    }

    // Snapshot error
    if snapshot_invalid {
        return Err(WorkflowError::SnapshotError("Integrity check failed".to_string()));
    }

    Ok(())
}
```

## Advanced Features

### Custom Hook Registration

```rust
use knhk_workflow_engine::hooks::{WorkflowHook, HookFn};

let custom_hook = WorkflowHook {
    id: "custom-validator".to_string(),
    hook_type: HookType::BeforeTaskExecution,
    name: "Custom Validator".to_string(),
    description: "Validates task input".to_string(),
    hook_fn: Arc::new(|ctx| Box::pin(async move {
        // Custom validation logic
        if ctx.data.get("valid").and_then(|v| v.as_bool()).unwrap_or(false) {
            HookResult::success()
        } else {
            HookResult::failure("Validation failed".to_string())
        }
    })),
    enabled: true,
    priority: 10,
};

registry.register(custom_hook).await?;
```

### SHACL Validation

```rust
use knhk_workflow_engine::guards::shacl_validator::{ShaclValidator, ShaclShape};

let mut validator = ShaclValidator::new();

let shape = ShaclShape {
    id: "WorkflowShape".to_string(),
    target_class: Some("Workflow".to_string()),
    properties: /* ... */,
};

validator.register_shape(shape)?;

let result = validator.validate(&graph_data)?;
if !result.conforms {
    for violation in result.violations {
        eprintln!("Violation: {}", violation.message);
    }
}
```

## Troubleshooting

### Hook Execution Exceeds Ticks

**Problem**: Hooks exceeding 8 tick constraint.

**Solution**:
- Review hook implementation for efficiency
- Check if hook is doing I/O or async operations
- Consider marking as low priority if not hot path critical

### Guard Failures

**Problem**: Guards consistently failing.

**Solution**:
- Review guard predicates for correctness
- Check input state format
- Verify guard severity levels

### Receipt Signature Failures

**Problem**: Receipt verification failing.

**Solution**:
- Ensure receipt is not modified after generation
- Check SHA-256 hash computation
- Verify all required fields are populated

### Snapshot Integrity Issues

**Problem**: Snapshot integrity checks failing.

**Solution**:
- Ensure content is not modified after snapshot creation
- Check serialization/deserialization
- Verify SHA-256 hash matches

## License

MIT License

## See Also

- [Workflow Engine Guide](../docs/WORKFLOW_ENGINE.md)
- [YAWL Integration](../docs/YAWL_INTEGRATION.md)
- [Testing Guide](../docs/TESTING.md)
- [KNHK OTEL Library](../knhk-otel/README.md)
