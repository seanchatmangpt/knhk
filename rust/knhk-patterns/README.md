# knhk-patterns

Van der Aalst workflow patterns for KNHK pipeline orchestration.

## Overview

`knhk-patterns` provides a production-ready implementation of the critical 20% of workflow patterns that deliver 80% of orchestration value. Based on the van der Aalst workflow pattern catalog and inspired by BitFlow's workflow engine, this crate enables complex pipeline orchestration beyond linear execution.

## Core Philosophy

- **80/20 Focus**: Implement critical patterns (1, 2, 3, 4, 5, 6, 10, 16) that cover 85% of real-world workflows
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

### Advanced Patterns (Patterns 6, 10, 16)

6. **Multi-Choice** (3 ticks, SIMD) - Selects one or more branches based on conditions (OR-split)
10. **Arbitrary Cycles** (2 ticks) - Supports loops and retry logic
16. **Deferred Choice** (3 ticks) - Event-driven choice made by environment

## Quick Start

```rust
use knhk_patterns::*;
use std::sync::Arc;

// Pattern 1: Sequence
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

// Pattern 2: Parallel Split
let branches = vec![
    Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
];

let pattern = ParallelSplitPattern::new(branches)?;
let results = pattern.execute(10)?;
assert_eq!(results.len(), 2); // [20, 30]

// Pattern 4: Exclusive Choice
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

## Pattern Composition

Build complex workflows from simple primitives:

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

## Pipeline Integration

Extend KNHK pipelines with workflow patterns:

```rust
use knhk_patterns::PipelinePatternExt;
use knhk_etl::Pipeline;

let mut pipeline = Pipeline::new(...);

// Execute with parallel validation
let results = pipeline.execute_parallel(vec![
    |result| { /* validator 1 */ Ok(result) },
    |result| { /* validator 2 */ Ok(result) },
])?;

// Execute with conditional routing
let results = pipeline.execute_conditional(vec![
    (|result| result.runs.len() > 100, |result| { /* process large */ Ok(result) }),
    (|_| true, |result| { /* process normal */ Ok(result) }),
])?;

// Execute with retry
let result = pipeline.execute_with_retry(
    |result| { /* processor */ Ok(result) },
    |result| result.runs.is_empty(),
    3,
)?;
```

## Knowledge Hook Integration

Orchestrate knowledge hook execution within the Reflex stage using workflow patterns:

```rust
use knhk_patterns::hook_patterns::*;
use knhk_etl::hook_registry::HookRegistry;

let registry = HookRegistry::new();
// Register hooks for predicates
registry.register_hook(pred1, KernelType::ValidateSp, guard1, invariants1)?;
registry.register_hook(pred2, KernelType::ValidateSp, guard2, invariants2)?;

// Execute hooks in parallel
let pattern = HookParallelPattern::new(vec![pred1, pred2])?;
let context = HookExecutionContext {
    hook_registry: registry,
    predicate_runs: runs,
    soa_arrays: soa,
    tick_budget: 8,
};
let results = pattern.execute_hooks(&context)?;

// Execute hooks conditionally
let choices = vec![
    (|receipt: &Receipt| receipt.ticks > 4, pred1),
    (|_| true, pred2),
];
let pattern = HookChoicePattern::new(choices)?;
let results = pattern.execute_hooks(&context)?;

// Execute hooks with retry
let pattern = HookRetryPattern::new(
    pred1,
    |receipt: &Receipt| receipt.ticks == 0,
    3, // max attempts
)?;
let results = pattern.execute_hooks(&context)?;
```

**Hook Pattern Types:**
- **HookSequencePattern** - Sequential hook execution
- **HookParallelPattern** - Parallel hook execution (SIMD-optimized)
- **HookChoicePattern** - Conditional hook routing
- **HookRetryPattern** - Retry logic for transient failures

See [HOOK_INTEGRATION.md](HOOK_INTEGRATION.md) for comprehensive hook integration guide.

## Performance Characteristics

All patterns are designed for hot path execution:

| Pattern | Tick Budget | SIMD Support | Complexity |
|---------|-------------|--------------|-------------|
| Sequence | 1 | No | O(n) |
| Parallel Split | 2 | Yes | O(n) |
| Synchronization | 3 | Yes | O(n) |
| Exclusive Choice | 2 | No | O(n) |
| Simple Merge | 1 | No | O(1) |
| Multi-Choice | 3 | Yes | O(n) |
| Arbitrary Cycles | 2 | No | O(k*n) |
| Deferred Choice | 3 | No | O(n) |

**Key Performance Principles:**
- Ingress validation happens ONCE at pattern creation
- Hot path execution has zero validation overhead
- SIMD optimization for parallelizable patterns
- Branchless operations where possible

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
- Zero overhead execution

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design documentation.

## Pattern Reference

See [PATTERNS.md](PATTERNS.md) for complete pattern descriptions and examples.

## Dependencies

- `knhk-hot` - Hot path implementations (C FFI)
- `knhk-etl` - Pipeline integration
- `knhk-config` - Configuration management
- `rayon` - Parallel execution
- `crossbeam-channel` - Lock-free channels

## License

Part of the KNHK project.

## References

- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)
- [YAWL Foundation](https://www.yawlfoundation.org/)
- [BitFlow Workflow Engine](https://github.com/bitflow-engine/bitflow)

