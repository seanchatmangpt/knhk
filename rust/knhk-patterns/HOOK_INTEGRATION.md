# Hook Integration Guide

## Overview

This guide explains how `knhk-patterns` workflow patterns integrate with knowledge hooks to enable pattern-based hook orchestration within the Reflex stage of the KNHK pipeline.

## Architecture

### Knowledge Hooks

Knowledge hooks are compiled interfaces between ontological laws and runtime reconciliation. They execute during the Reflex stage (≤8 ticks per hook) and enforce invariants on data admitted into the knowledge graph.

**Hook Definition:**
- **Predicate**: The RDF predicate this hook validates
- **Guard**: Function that validates triples against invariants
- **Action**: Kernel operation executed (ASK_SP, COUNT_SP_GE, etc.)

**Formal Definition:**
```
hook(p, q, a): Δ ⊨ Qp  ⇒  μ(O ⊔ Δ) = μ(O) ⊔ μ(Δ)
```

### Current Hook Execution

Currently, hooks execute sequentially in the Reflex stage:

```rust
// Sequential execution (current)
for run in &input.runs {
    let receipt = self.execute_hook(&input.soa_arrays, run)?;
    receipts.push(receipt);
}
```

**Limitations:**
- No parallel execution for independent predicates
- No conditional routing based on hook results
- No retry logic for transient failures
- No orchestration beyond simple iteration

### Pattern-Based Hook Orchestration

Workflow patterns enable sophisticated hook orchestration:

1. **Sequential Execution**: Execute hooks in a specific order
2. **Parallel Execution**: Execute independent hooks concurrently
3. **Conditional Routing**: Route hook execution based on conditions
4. **Retry Logic**: Retry failed hooks with backoff

## Integration Points

### 1. Hook Execution Context

Hooks execute within a context that includes:

```rust
pub struct HookExecutionContext {
    pub hook_registry: HookRegistry,  // Predicate → Kernel mapping
    pub predicate_runs: Vec<PredRun>, // Predicate runs to execute
    pub soa_arrays: SoAArrays,        // Data arrays
    pub tick_budget: u32,              // ≤8 ticks per hook
}
```

### 2. Pattern Configuration

Patterns configure hook execution:

```rust
pub enum HookExecutionPattern {
    Sequence(Vec<u64>),                    // Predicate IDs in sequence
    Parallel(Vec<u64>),                    // Predicate IDs in parallel
    Choice(Vec<(HookCondition, u64)>),     // Conditional routing
    Retry { predicate: u64, max_attempts: u32 },
}
```

### 3. Receipt Aggregation

Patterns aggregate receipts from hook execution:

- Merge receipts using ⊕ operator
- Track max ticks across all hooks
- Aggregate actions from all hooks
- Verify hash(A) = hash(μ(O))

## Pattern Types for Hooks

### 1. Sequential Hook Execution

Execute hooks in a specific order:

```rust
use knhk_patterns::hook_patterns::HookSequencePattern;
use knhk_etl::hook_registry::HookRegistry;

let registry = HookRegistry::new();
let predicates = vec![predicate1, predicate2, predicate3];

let pattern = HookSequencePattern::new(predicates)?;
let results = pattern.execute_hooks(&context)?;
```

**Use Cases:**
- Dependent hooks (hook2 depends on hook1)
- Ordered validation (schema → constraints → business rules)
- Sequential transformations

### 2. Parallel Hook Execution

Execute independent hooks concurrently:

```rust
use knhk_patterns::hook_patterns::HookParallelPattern;

let predicates = vec![predicate1, predicate2, predicate3];
let pattern = HookParallelPattern::new(predicates)?;
let results = pattern.execute_hooks(&context)?;
```

**Use Cases:**
- Independent validations
- Parallel schema checks
- Concurrent constraint validation

**Performance:**
- Uses Rayon for parallel execution
- Respects tick budget (≤8 ticks per hook)
- Aggregates receipts efficiently

### 3. Conditional Hook Routing

Route hook execution based on conditions:

```rust
use knhk_patterns::hook_patterns::HookChoicePattern;

let choices = vec![
    (|receipt: &Receipt| receipt.ticks > 4, predicate1),
    (|receipt: &Receipt| receipt.ticks <= 4, predicate2),
];

let pattern = HookChoicePattern::new(choices)?;
let results = pattern.execute_hooks(&context)?;
```

**Use Cases:**
- Conditional validation paths
- Error handling routes
- Performance-based routing

### 4. Retry Logic

Retry failed hooks with exponential backoff:

```rust
use knhk_patterns::hook_patterns::HookRetryPattern;

let pattern = HookRetryPattern::new(
    predicate,
    |receipt: &Receipt| receipt.ticks == 0, // Should retry if failed
    3, // max attempts
)?;
let results = pattern.execute_hooks(&context)?;
```

**Use Cases:**
- Transient failures
- Network timeouts
- Resource contention

## Integration with Reflex Stage

### Extending ReflexStage

The Reflex stage can be extended to support pattern-based hook execution:

```rust
impl ReflexStage {
    /// Execute reflex with pattern-based hook orchestration
    pub fn reflex_with_patterns(
        &self,
        input: LoadResult,
        pattern: HookExecutionPattern,
    ) -> Result<ReflexResult, PipelineError> {
        // Execute hooks using pattern
        // Aggregate receipts
        // Return results
    }
}
```

### Hook Orchestrator

A dedicated orchestrator manages pattern-based hook execution:

```rust
use knhk_etl::hook_orchestration::HookOrchestrator;

let orchestrator = HookOrchestrator::new(hook_registry);
let results = orchestrator.execute_with_pattern(
    &context,
    HookExecutionPattern::Parallel(predicates),
)?;
```

## Pipeline Integration

### Using Patterns in Pipeline

Patterns can be integrated into the pipeline execution:

```rust
use knhk_patterns::PipelinePatternExt;

let mut pipeline = Pipeline::new();
let hook_registry = HookRegistry::new();

// Execute hooks in parallel
let results = pipeline.execute_hooks_parallel(
    &hook_registry,
    vec![predicate1, predicate2, predicate3],
)?;

// Execute hooks conditionally
let results = pipeline.execute_hooks_conditional(
    &hook_registry,
    vec![
        (|r| r.ticks > 4, predicate1),
        (|_| true, predicate2),
    ],
)?;

// Execute hooks with retry
let result = pipeline.execute_hooks_with_retry(
    &hook_registry,
    predicate,
    |r| r.ticks == 0,
    3,
)?;
```

## Performance Considerations

### Parallel Execution

- **Rayon**: Uses Rayon thread pool for parallel execution
- **Tick Budget**: Each hook must complete in ≤8 ticks
- **Receipt Aggregation**: Efficient merging using ⊕ operator

### Conditional Routing

- **Early Exit**: First matching condition executes immediately
- **Overhead**: Minimize condition evaluation overhead
- **Caching**: Cache condition results where possible

### Retry Logic

- **Exponential Backoff**: Backoff between retry attempts
- **Max Attempts**: Respect maximum attempt limits
- **Tracking**: Track retry attempts in receipts

## Examples

### Example 1: Parallel Validation

Execute multiple independent validations in parallel:

```rust
use knhk_patterns::hook_patterns::HookParallelPattern;
use knhk_etl::hook_registry::HookRegistry;

let registry = HookRegistry::new();
// Register hooks for predicates
registry.register_hook(pred1, KernelType::ValidateSp, guard1, invariants1)?;
registry.register_hook(pred2, KernelType::ValidateSp, guard2, invariants2)?;
registry.register_hook(pred3, KernelType::ValidateSp, guard3, invariants3)?;

let pattern = HookParallelPattern::new(vec![pred1, pred2, pred3])?;
let context = HookExecutionContext {
    hook_registry: registry,
    predicate_runs: runs,
    soa_arrays: soa,
    tick_budget: 8,
};

let results = pattern.execute_hooks(&context)?;
// All three hooks execute in parallel
// Receipts are aggregated
```

### Example 2: Conditional Routing

Route hook execution based on previous results:

```rust
use knhk_patterns::hook_patterns::HookChoicePattern;

let choices = vec![
    (
        |receipt: &Receipt| receipt.ticks > 4, // High latency
        expensive_predicate, // Use expensive validation
    ),
    (
        |receipt: &Receipt| receipt.ticks <= 4, // Low latency
        fast_predicate, // Use fast validation
    ),
];

let pattern = HookChoicePattern::new(choices)?;
let results = pattern.execute_hooks(&context)?;
```

### Example 3: Retry Logic

Retry failed hooks with exponential backoff:

```rust
use knhk_patterns::hook_patterns::HookRetryPattern;

let pattern = HookRetryPattern::new(
    predicate,
    |receipt: &Receipt| receipt.ticks == 0, // Retry if failed
    3, // max attempts
)?;

let results = pattern.execute_hooks(&context)?;
// Hook retries up to 3 times if it fails
```

## Best Practices

### 1. Use Sequential for Dependencies

When hooks depend on each other, use sequential execution:

```rust
// ✅ Good: Dependent hooks
let pattern = HookSequencePattern::new(vec![schema_hook, constraint_hook, business_rule_hook])?;
```

### 2. Use Parallel for Independence

When hooks are independent, use parallel execution:

```rust
// ✅ Good: Independent hooks
let pattern = HookParallelPattern::new(vec![validation1, validation2, validation3])?;
```

### 3. Use Conditional for Routing

When routing depends on conditions, use conditional execution:

```rust
// ✅ Good: Conditional routing
let pattern = HookChoicePattern::new(vec![
    (|r| r.is_error(), error_handler),
    (|r| r.is_success(), success_handler),
])?;
```

### 4. Use Retry for Transient Failures

When failures are transient, use retry logic:

```rust
// ✅ Good: Retry transient failures
let pattern = HookRetryPattern::new(predicate, |r| r.is_transient(), 3)?;
```

## Migration Guide

### From Sequential to Pattern-Based

**Before:**
```rust
for run in &input.runs {
    let receipt = self.execute_hook(&input.soa_arrays, run)?;
    receipts.push(receipt);
}
```

**After:**
```rust
let pattern = HookParallelPattern::new(predicates)?;
let results = pattern.execute_hooks(&context)?;
```

### Backward Compatibility

The existing `reflex()` method remains unchanged for backward compatibility:

```rust
// Still works
let result = reflex_stage.reflex(load_result)?;

// New pattern-based execution
let result = reflex_stage.reflex_with_patterns(load_result, pattern)?;
```

## References

- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)
- [KNHK Reflex Architecture](book/src/architecture/reflex.md)
- [Hook Registry Documentation](rust/knhk-etl/src/hook_registry.rs)
- [Pattern Documentation](rust/knhk-patterns/PATTERNS.md)

