# knhk-patterns

Van der Aalst workflow patterns for KNHK pipeline orchestration.

## Overview

`knhk-patterns` provides a production-ready implementation of the critical 20% of workflow patterns that deliver 80% of orchestration value. Based on the van der Aalst workflow pattern catalog and inspired by BitFlow's workflow engine, this crate enables complex pipeline orchestration beyond linear execution.

## Core Philosophy

- **80/20 Focus**: Implement critical patterns (1, 2, 3, 4, 5, 6, 9, 10, 11, 16, 20, 21) that cover 85% of real-world workflows
- **Performance First**: All patterns respect the Chatman Constant (≤8 ticks) for hot path execution
- **Ingress Validation**: Guards enforce constraints ONCE at pattern creation, not during execution
- **Zero Overhead**: No runtime measurement or validation in hot path
- **Production Ready**: No placeholders, real implementations with proper error handling

## Supported Patterns

### Basic Control Flow (Patterns 1-5)

1. **Sequence** (1 tick) - Sequential execution of activities
2. **Parallel Split** (2 ticks, SIMD) - Splits execution into multiple parallel branches
3. **Synchronization** (3 ticks, SIMD) - Synchronizes multiple parallel branches
4. **Exclusive Choice** (2 ticks) - Selects one branch from multiple alternatives (XOR-split)
5. **Simple Merge** (1 tick) - Merges alternative branches without synchronization (XOR-join)

### Advanced Patterns (Patterns 6, 9, 10, 11, 16)

6. **Multi-Choice** (3 ticks, SIMD) - Selects one or more branches based on conditions (OR-split)
9. **Discriminator** (3 ticks, SIMD) - First-wins pattern (race condition, first branch to complete wins)
10. **Arbitrary Cycles** (2 ticks) - Supports loops and retry logic
11. **Implicit Termination** (2 ticks) - Workflow completion detection (waits for all branches)
16. **Deferred Choice** (3 ticks) - Event-driven choice made by environment

### Production Patterns (Patterns 20, 21)

20. **Timeout** (2 ticks) - Executes branch with timeout and optional fallback
21. **Cancellation** (1 tick) - Cancellable execution with pre/post-execution checks

## Quick Start

### Basic Pattern Usage

```rust
use knhk_patterns::*;
use std::sync::Arc;

// Pattern 1: Sequence - Execute branches in order
let branch1 = Arc::new(|mut data: i32| {
    data *= 2;
    Ok(data)
});

let branch2 = Arc::new(|mut data: i32| {
    data += 10;
    Ok(data)
});

let pattern = SequencePattern::new(vec![branch1, branch2])?;
let results = pattern.execute(5)?;
assert_eq!(results[0], 20); // (5 * 2) + 10

// Pattern 2: Parallel Split - Execute all branches concurrently
let branches = vec![
    Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
];

let pattern = ParallelSplitPattern::new(branches)?;
let results = pattern.execute(10)?;
assert_eq!(results.len(), 2); // [20, 30]

// Pattern 4: Exclusive Choice - Select one branch based on condition
let choices = vec![
    (
        Arc::new(|data: &i32| *data < 0) as ConditionFn<i32>,
        Arc::new(|mut data: i32| { data = -1; Ok(data) }) as BranchFn<i32>,
    ),
    (
        Arc::new(|data: &i32| *data >= 0),
        Arc::new(|mut data: i32| { data = 100; Ok(data) }),
    ),
];

let pattern = ExclusiveChoicePattern::new(choices)?;
let results = pattern.execute(5)?;
assert_eq!(results[0], 100);
```

### Pattern Composition

Build complex workflows from simple primitives using the fluent API:

```rust
use knhk_patterns::composition::PatternBuilder;

let workflow = PatternBuilder::new()
    .then(Arc::new(|mut data: i32| { data += 1; Ok(data) }))
    .parallel(vec![
        Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
        Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
    ])
    .retry(
        Arc::new(|mut data: i32| { data += 1; Ok(data) }),
        Arc::new(|data: &i32| *data < 10),
        100,
    )
    .build();

let results = workflow.execute(5)?;
```

Or compose patterns directly:

```rust
use knhk_patterns::composition::CompositePattern;

let parallel = ParallelSplitPattern::new(branches)?;
let sequential = SequencePattern::new(vec![branch])?;

let composite = CompositePattern::Sequence(vec![
    Box::new(parallel) as Box<dyn Pattern<i32>>,
    Box::new(sequential) as Box<dyn Pattern<i32>>,
]);

let results = composite.execute(input)?;
```

## Pipeline Integration

Extend KNHK pipelines with workflow patterns:

```rust
use knhk_patterns::PipelinePatternExt;
use knhk_etl::{Pipeline, EmitResult};

let mut pipeline = Pipeline::new(...);

// Execute with parallel validation
let results = pipeline.execute_parallel(vec![
    |result: EmitResult| { /* validator 1 */ Ok(result) },
    |result: EmitResult| { /* validator 2 */ Ok(result) },
])?;

// Execute with conditional routing
let results = pipeline.execute_conditional(vec![
    (|result: &EmitResult| result.receipts_written > 100, |result: EmitResult| { 
        /* process large */ Ok(result) 
    }),
    (|_| true, |result: EmitResult| { 
        /* process normal */ Ok(result) 
    }),
])?;

// Execute with retry
let result = pipeline.execute_with_retry(
    |result: EmitResult| { /* processor */ Ok(result) },
    |result: &EmitResult| result.receipts_written == 0,
    3,
)?;
```

## Knowledge Hook Integration

Orchestrate knowledge hook execution within the Reflex stage using workflow patterns:

```rust
use knhk_patterns::hook_patterns::*;
use knhk_etl::hook_registry::HookRegistry;
use knhk_etl::hook_orchestration::HookExecutionContext;

let registry = HookRegistry::new();
// Register hooks for predicates
registry.register_hook(pred1, KernelType::ValidateSp, guard1, invariants1)?;
registry.register_hook(pred2, KernelType::ValidateSp, guard2, invariants2)?;

// Create execution context
let context = create_hook_context_from_components(
    registry,
    predicate_runs,
    soa_arrays,
    8, // tick budget
);

// Execute hooks in parallel
let pattern = HookParallelPattern::new(vec![pred1, pred2])?;
let results = pattern.execute_hooks(&context)?;

// Execute hooks conditionally
let choices = vec![
    (|ctx: &HookExecutionContext| ctx.predicate_runs.len() > 1, pred1),
    (|_| true, pred2),
];
let pattern = HookChoicePattern::new(choices)?;
let results = pattern.execute_hooks(&context)?;

// Execute hooks with retry
let pattern = HookRetryPattern::new(
    pred1,
    |receipt: &Receipt| receipt.ticks == 0, // Should retry if failed
    3, // max attempts
)?;
let results = pattern.execute_hooks(&context)?;
```

**Hook Pattern Types:**
- **HookSequencePattern** - Sequential hook execution
- **HookParallelPattern** - Parallel hook execution (SIMD-optimized)
- **HookChoicePattern** - Conditional hook routing
- **HookRetryPattern** - Retry logic for transient failures

See [docs/HOOK_INTEGRATION.md](docs/HOOK_INTEGRATION.md) for comprehensive hook integration guide.

## Hybrid Hot/Cold Path Patterns

Orchestrate both hot path (HookRegistry) and cold path (unrdf) hooks together:

```rust
use knhk_patterns::hybrid_patterns::*;
use knhk_etl::hook_registry::HookRegistry;

let hot_registry = HookRegistry::new();
let hot_predicates = vec![pred1, pred2];
let cold_hook_ids = vec!["policy-validation".to_string()];

// Sequential: Hot path first, then cold path
let pattern = HybridSequencePattern::new(
    hot_predicates.clone(),
    cold_hook_ids.clone(),
    hot_registry.clone(),
)?;
let results = pattern.execute(&context)?;

// Parallel: Hot and cold paths concurrently
let pattern = HybridParallelPattern::new(
    hot_predicates,
    cold_hook_ids.clone(),
    hot_registry.clone(),
)?;
let results = pattern.execute(&context)?;

// Conditional: Route based on context
let pattern = HybridChoicePattern::new(
    |ctx: &HookExecutionContext| ctx.predicate_runs.len() > 8,
    hot_registry,
    cold_hook_ids,
)?;
let results = pattern.execute(&context)?;
```

**Note:** Cold path patterns require the `unrdf` feature to be enabled.

## Cold Path Hook Patterns (unrdf feature)

Execute SPARQL-based hooks for complex policy validation:

```rust
#[cfg(feature = "unrdf")]
use knhk_patterns::unrdf_patterns::*;

// Sequential cold path hook execution
let pattern = UnrdfSequencePattern::new(vec![
    "check-permission".to_string(),
    "validate-schema".to_string(),
])?;
let results = pattern.execute_hooks(turtle_data)?;

// Parallel cold path hook execution
let pattern = UnrdfParallelPattern::new(vec![
    "policy1".to_string(),
    "policy2".to_string(),
])?;
let results = pattern.execute_hooks(turtle_data)?;
```

## Performance Characteristics

All patterns are designed for hot path execution:

| Pattern | Tick Budget | SIMD Support | Complexity |
|---------|-------------|--------------|------------|
| Sequence | 1 | No | O(n) |
| Parallel Split | 2 | Yes | O(n) |
| Synchronization | 3 | Yes | O(n) |
| Exclusive Choice | 2 | No | O(n) |
| Simple Merge | 1 | No | O(1) |
| Multi-Choice | 3 | Yes | O(n) |
| Discriminator | 3 | Yes | O(n) |
| Arbitrary Cycles | 2 | No | O(k*n) |
| Implicit Termination | 2 | No | O(n) |
| Deferred Choice | 3 | No | O(n) |
| Timeout | 2 | No | O(1) |
| Cancellation | 1 | No | O(1) |

**Key Performance Principles:**
- Ingress validation happens ONCE at pattern creation (cold path)
- Hot path execution has zero validation overhead
- SIMD optimization for parallelizable patterns
- Branchless operations where possible
- No runtime measurement or tick counting in hot path

## Error Handling

All patterns return `PatternResult<T>` with comprehensive error types:

```rust
pub enum PatternError {
    ValidationFailed(String),      // Ingress validation failed
    ExecutionFailed(String),       // Runtime execution error
    TooManyBranches,               // Branch limit exceeded (max 1024)
    InvalidConfiguration(String),  // Invalid pattern configuration
}
```

Patterns validate constraints at creation time (ingress validation), not during execution. This ensures zero overhead in the hot path.

## Type System

Patterns are generic over data type `T` where `T: Clone + Send + Sync`:

```rust
pub trait Pattern<T>: Send + Sync {
    fn pattern_type(&self) -> PatternType;
    fn execute(&self, input: T) -> PatternResult<Vec<T>>;
    fn tick_budget(&self) -> u32;
}
```

**Type Aliases:**
- `BranchFn<T>` - `Arc<dyn Fn(T) -> PatternResult<T> + Send + Sync>`
- `ConditionFn<T>` - `Arc<dyn Fn(&T) -> bool + Send + Sync>`
- `PatternResult<T>` - `Result<T, PatternError>`

## Testing

Tests follow Chicago TDD methodology - testing behavior, not implementation:

```bash
cargo test
```

All tests verify:
- Pattern behavior (what patterns do)
- Error handling
- Ingress validation
- Tick budget compliance (≤8 ticks)
- Zero overhead execution (no runtime measurement)

Tests use the AAA pattern (Arrange, Act, Assert) and focus on observable behavior rather than implementation details.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed design documentation.

## Pattern Reference

See [docs/PATTERNS.md](docs/PATTERNS.md) for complete pattern descriptions and examples.

## Dependencies

- `knhk-etl` - Pipeline integration
- `knhk-config` - Configuration management
- `rayon` - Parallel execution
- `crossbeam-channel` - Lock-free channels
- `knhk-unrdf` (optional) - Cold path hook support (unrdf feature)

## Features

- `unrdf` - Enable cold path hook patterns (requires `knhk-unrdf`)

## License

Part of the KNHK project.

## References

- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)
- [YAWL Foundation](https://www.yawlfoundation.org/)
- [BitFlow Workflow Engine](https://github.com/bitflow-engine/bitflow)
