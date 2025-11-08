# knhk-patterns Architecture

## Overview

`knhk-patterns` implements a layered architecture that bridges high-level Rust APIs with low-level C hot path implementations, following the 80/20 principle for workflow orchestration.

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│              Application Layer (Rust)                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Patterns   │  │ Composition   │  │ Pipeline Ext │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              FFI Layer (Rust ↔ C)                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  PatternType, PatternContext, PatternResult      │  │
│  │  Safe wrappers for C functions                    │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Hot Path Layer (C)                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │  knhk-hot/workflow_patterns.c                     │  │
│  │  SIMD-optimized, ≤8 tick implementations          │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Pattern Trait (`patterns.rs`)

The foundational trait that all patterns implement:

```rust
pub trait Pattern<T>: Send + Sync {
    fn pattern_type(&self) -> PatternType;
    fn execute(&self, input: T) -> PatternResult<Vec<T>>;
    fn tick_budget(&self) -> u32;
}
```

**Design Principles:**
- Generic over data type `T` (must be `Clone + Send + Sync`)
- Returns `Vec<T>` to support patterns that produce multiple outputs
- Tick budget query for ingress validation
- Thread-safe (`Send + Sync`)

### 2. Pattern Implementations (`patterns.rs`)

Each pattern is a struct implementing `Pattern<T>`:

- **SequencePattern**: Sequential execution
- **ParallelSplitPattern**: Parallel execution with Rayon
- **SynchronizationPattern**: Synchronization point
- **ExclusiveChoicePattern**: XOR-split with conditions
- **SimpleMergePattern**: XOR-join
- **MultiChoicePattern**: OR-split with conditions
- **DiscriminatorPattern**: First-wins pattern (race condition)
- **ArbitraryCyclesPattern**: Loop/retry logic
- **ImplicitTerminationPattern**: Workflow completion detection
- **DeferredChoicePattern**: Event-driven choice
- **TimeoutPattern**: Timeout with optional fallback
- **CancellationPattern**: Cancellable execution

**Key Implementation Details:**

1. **Ingress Validation**: All patterns validate constraints at creation:
   ```rust
   PatternType::ParallelSplit.validate_ingress(branches.len() as u32)?;
   ```

2. **Error Handling**: All operations return `PatternResult<T>`:
   ```rust
   pub type PatternResult<T> = Result<T, PatternError>;
   ```

3. **Parallel Execution**: Uses Rayon for parallel patterns:
   ```rust
   self.branches.par_iter().map(|branch| branch(input.clone())).collect()
   ```

### 3. Composition Layer (`composition.rs`)

Enables building complex workflows from simple patterns:

**CompositePattern Enum:**
- `Sequence(Vec<Box<dyn Pattern<T>>>)` - Sequential composition
- `Parallel(Vec<Box<dyn Pattern<T>>>)` - Parallel composition
- `Choice(Vec<(ConditionFn<T>, Box<dyn Pattern<T>>)>)` - Conditional routing
- `MultiChoice(...)` - Multi-branch conditional
- `Retry { pattern, should_continue, max_attempts }` - Retry logic
- `Timeout { pattern, timeout_ms }` - Timeout wrapper
- `Atomic(Box<dyn Pattern<T>>)` - Leaf pattern

**PatternBuilder Fluent API:**
```rust
PatternBuilder::new()
    .then(branch)
    .parallel(branches)
    .choice(choices)
    .build()
```

### 4. FFI Layer (`ffi.rs`)

Bridges Rust and C implementations:

**C Types:**
- `PatternContext` - Execution context
- `PatternResult` - C result type
- `PatternType` - Enum of pattern types
- `BranchFn` / `ConditionFn` - C function pointers

**Safe Wrappers:**
- `PatternType::validate_ingress()` - Ingress validation
- `PatternType::tick_budget()` - Tick budget query
- `PatternResult::into_result()` - Convert to Rust Result

### 5. Pipeline Integration (`pipeline_ext.rs`)

Extension trait for KNHK pipelines:

```rust
pub trait PipelinePatternExt {
    fn execute_parallel<F>(&mut self, processors: Vec<F>) -> PatternResult<Vec<EmitResult>>;
    fn execute_conditional<F, C>(&mut self, choices: Vec<(C, F)>) -> PatternResult<Vec<EmitResult>>;
    fn execute_with_retry<F, C>(&mut self, processor: F, should_retry: C, max_attempts: u32) -> PatternResult<EmitResult>;
    fn execute_hooks_parallel(&mut self, hook_registry: &HookRegistry, predicates: Vec<u64>) -> PatternResult<HookExecutionResult>;
    fn execute_hooks_conditional(&mut self, hook_registry: &HookRegistry, choices: Vec<(HookCondition, u64)>) -> PatternResult<HookExecutionResult>;
    fn execute_hooks_with_retry(&mut self, hook_registry: &HookRegistry, predicate: u64, should_retry: HookRetryCondition, max_attempts: u32) -> PatternResult<HookExecutionResult>;
}
```

## Performance Architecture

### Ingress Validation (Cold Path)

Validation happens ONCE at pattern creation:

```rust
// ✅ Cold path: Validate at creation
let pattern = ParallelSplitPattern::new(branches)?; // Validates here

// ✅ Hot path: Zero validation overhead
let results = pattern.execute(input)?; // No validation checks
```

**Validation Rules:**
- Branch count limits (max 1024)
- Pattern-specific constraints
- Tick budget compliance (≤8 ticks)

### Hot Path Execution

Zero-overhead execution:

1. **No Runtime Validation**: Guards checked at ingress only
2. **SIMD Optimization**: Parallel patterns use SIMD when available
3. **Branchless Operations**: Where possible, avoid branches
4. **Lock-Free**: Use atomic operations for synchronization

### Tick Budget Enforcement

Each pattern has a tick budget (≤8 ticks):

| Pattern | Tick Budget | Rationale |
|---------|-------------|-----------|
| Sequence | 1 | Direct flow transition |
| Parallel Split | 2 | Branch initialization |
| Synchronization | 3 | Atomic counter operations |
| Exclusive Choice | 2 | Condition evaluation |
| Simple Merge | 1 | Direct merge |
| Multi-Choice | 3 | Multiple condition checks |
| Discriminator | 3 | First-wins with atomic coordination |
| Arbitrary Cycles | 2 | Loop overhead |
| Implicit Termination | 2 | Track active branches |
| Deferred Choice | 3 | Event polling |
| Timeout | 2 | Check timeout + execute |
| Cancellation | 1 | Atomic cancel check |

## Error Handling Architecture

### Error Types

```rust
pub enum PatternError {
    ValidationFailed(String),      // Ingress validation error
    ExecutionFailed(String),       // Runtime execution error
    TooManyBranches,               // Branch limit exceeded
    InvalidConfiguration(String),  // Invalid configuration
}
```

### Error Propagation

1. **Ingress Errors**: Returned during pattern creation
2. **Execution Errors**: Returned during `execute()`
3. **Composition Errors**: Propagated through composite patterns

## Thread Safety

All patterns are `Send + Sync`:

- **Send**: Patterns can be moved between threads
- **Sync**: Patterns can be shared between threads
- **Arc**: Branch functions use `Arc` for shared ownership
- **Rayon**: Parallel execution uses Rayon's thread pool

## Memory Management

- **Zero-Copy**: Use references where possible (`&T` over `T`)
- **Clone on Demand**: Only clone when necessary (parallel execution)
- **RAII**: Automatic cleanup via Rust's ownership system
- **No Leaks**: All resources managed by Rust

## Testing Architecture

Tests follow Chicago TDD methodology:

1. **Behavior Testing**: Test what patterns do, not how
2. **AAA Pattern**: Arrange, Act, Assert
3. **No Implementation Details**: Tests don't depend on internals
4. **Performance Verification**: Verify tick budgets, not measure ticks

## Integration Points

### With knhk-etl

```rust
use knhk_patterns::PipelinePatternExt;
use knhk_etl::Pipeline;

// Extend Pipeline with patterns
impl PipelinePatternExt for Pipeline { ... }
```

### With knhk-hot

```rust
use knhk_patterns::ffi::*;

// Call C hot path implementations
unsafe {
    knhk_pattern_parallel_split(ctx, branches.as_ptr(), num_branches);
}
```

### With knhk-config

Patterns can be configured via `knhk-config`:

```rust
use knhk_config::Config;

let config = Config::load()?;
let max_branches = config.get::<u32>("patterns.max_branches")?;
```

### With Knowledge Hooks (Hook Orchestration)

Patterns orchestrate knowledge hook execution within the Reflex stage:

```rust
use knhk_patterns::hook_patterns::*;
use knhk_etl::hook_orchestration::HookOrchestrator;

let orchestrator = HookOrchestrator::new(hook_registry);
let results = orchestrator.execute_with_pattern(
    &context,
    HookExecutionPattern::Parallel(predicates),
)?;
```

**Hook Orchestration Architecture:**

```
┌─────────────────────────────────────────────────────────┐
│              Hook Orchestration Layer                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ HookSequence │  │ HookParallel │  │ HookChoice   │ │
│  │   Pattern    │  │   Pattern    │  │   Pattern    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│  ┌──────────────┐                                      │
│  │ HookRetry    │                                      │
│  │   Pattern    │                                      │
│  └──────────────┘                                      │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Hook Execution Context                      │
│  ┌──────────────────────────────────────────────────┐  │
│  │  HookRegistry, PredicateRuns, SoAArrays          │  │
│  │  Tick Budget (≤8 ticks per hook)                │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Reflex Stage (knhk-etl)                     │
│  ┌──────────────────────────────────────────────────┐  │
│  │  execute_hook() → C hot path API                 │  │
│  │  Receipt aggregation (⊕)                         │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Key Components:**

1. **Hook Pattern Types** (`hook_patterns.rs`):
   - `HookSequencePattern` - Sequential hook execution
   - `HookParallelPattern` - Parallel hook execution (SIMD-optimized)
   - `HookChoicePattern` - Conditional hook routing
   - `HookRetryPattern` - Retry logic for transient failures

2. **Hook Orchestrator** (`hook_orchestration.rs`):
   - Wraps `HookRegistry` for pattern-based execution
   - Manages hook execution context
   - Aggregates receipts from pattern execution

3. **Hook Execution Context**:
   - Contains hook registry, predicate runs, SoA arrays
   - Enforces tick budget (≤8 ticks per hook)
   - Provides execution environment for patterns

**Execution Flow:**

1. **Pattern Configuration**: Define hook execution pattern
2. **Context Creation**: Create execution context with registry and data
3. **Pattern Execution**: Execute hooks according to pattern
4. **Receipt Aggregation**: Merge receipts using ⊕ operator
5. **Result Return**: Return aggregated results

**Performance Characteristics:**

- **Parallel Execution**: Uses Rayon for concurrent hook execution
- **Tick Budget**: Each hook must complete in ≤8 ticks
- **Receipt Aggregation**: Efficient merging using ⊕ operator
- **Conditional Routing**: Early exit on first matching condition

See [HOOK_INTEGRATION.md](HOOK_INTEGRATION.md) for detailed hook integration guide.

### 6. Hybrid Patterns (`hybrid_patterns.rs`)

Hybrid patterns orchestrate both hot path (HookRegistry) and cold path (unrdf) hooks together:

**Hybrid Pattern Types:**
- `HybridSequencePattern` - Execute hot path hooks, then cold path hooks sequentially
- `HybridParallelPattern` - Execute hot and cold path hooks concurrently
- `HybridChoicePattern` - Route between hot and cold paths based on condition

**Key Features:**
- Feature-gated cold path support (`#[cfg(feature = "unrdf")]`)
- Conditional routing based on execution context
- Receipt aggregation from both paths
- Tick budget enforcement (≤8 ticks for hot path)

### 7. Unrdf Patterns (`unrdf_patterns.rs`)

Cold path hook patterns for SPARQL-based hook execution (feature-gated):

**Unrdf Pattern Types:**
- `UnrdfSequencePattern` - Sequential cold path hook execution
- `UnrdfParallelPattern` - Parallel cold path hook execution
- `UnrdfChoicePattern` - Conditional cold path hook routing
- `UnrdfRetryPattern` - Retry logic for cold path hooks

**Key Features:**
- SPARQL ASK query evaluation
- Epoch-based ordering (≺-total order)
- Batch evaluation support
- Native Rust implementation (oxigraph)

## Future Extensions

### Planned Patterns

- Pattern 7: Structured Synchronizing Merge
- Pattern 8: Multi-Merge
- Pattern 12: Multiple Instances Without Synchronization
- Pattern 13: Multiple Instances With a Priori Design-Time Knowledge

### Performance Optimizations

- SIMD vectorization for more patterns
- Lock-free data structures
- NUMA-aware scheduling
- Cache-conscious layouts

## Design Decisions

### Why Generic Over T?

Allows patterns to work with any data type:
- `Pattern<i32>` - Simple integers
- `Pattern<LoadResult>` - Pipeline results
- `Pattern<CustomType>` - User-defined types

### Why Vec<T> Output?

Some patterns produce multiple outputs:
- Parallel Split: One output per branch
- Multi-Choice: One output per matching branch
- Sequence: Single output (but consistent API)

### Why Arc for Functions?

Enables shared ownership:
- Multiple patterns can share branch functions
- Thread-safe sharing
- Zero-cost when not shared

### Why Ingress Validation?

Performance optimization:
- Validate once at creation
- Zero overhead in hot path
- Fail fast on invalid configurations

## References

- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)
- [BitFlow Architecture](~/cns/bitflow/architecture/)
- [KNHK Hot Path Design](rust/knhk-hot/README.md)

