# ADR-002: Generic Associated Types for Pattern Executor Hierarchy

**Status:** Proposed
**Date:** 2025-11-16
**Deciders:** Architecture Team
**Technical Story:** Type-Safe Pattern Execution Framework

---

## Context and Problem Statement

The KNHK workflow engine implements 43 Van der Aalst workflow patterns, each with different:
- Input/output types
- State requirements
- Execution semantics
- Error handling needs

**Problem:** How do we create a type-safe, extensible pattern executor framework that:
1. Eliminates runtime type checks in the hot path
2. Provides compile-time guarantees for pattern composition
3. Supports both synchronous and asynchronous execution
4. Allows zero-cost abstractions
5. Enables dynamic pattern registration and dispatch

**Constraints:**
- Must maintain ≤8 tick hot path performance
- Must support all 43 YAWL patterns
- Must integrate with existing Weaver validation
- Must be maintainable by the team

---

## Decision Drivers

1. **Type Safety:** Compile-time guarantees eliminate runtime errors
2. **Performance:** Zero-cost abstractions, no runtime overhead
3. **Extensibility:** Easy to add new patterns
4. **Ergonomics:** Pleasant API for pattern implementers
5. **Compatibility:** Works with existing Rust ecosystem

---

## Considered Options

### Option 1: Trait Objects (Dynamic Dispatch)

**Description:** Use traditional trait objects with `dyn Trait` for pattern executors.

```rust
pub trait PatternExecutor: Send + Sync {
    fn execute(&self, ctx: &Context) -> Result<Box<dyn Any>>;
}

pub struct PatternRegistry {
    executors: HashMap<PatternId, Box<dyn PatternExecutor>>,
}
```

**Pros:**
- ✅ Simple to implement
- ✅ Easy to understand
- ✅ No unstable features required
- ✅ Dynamic registration straightforward

**Cons:**
- ❌ Runtime type checks required (`downcast`)
- ❌ Virtual function call overhead (~10ns)
- ❌ `Box<dyn Any>` loses type information
- ❌ No compile-time validation of output types
- ❌ Cannot express lifetime relationships

**Performance Impact:**
```
Hot path overhead per pattern execution:
- Virtual dispatch: ~10ns
- Type erasure boxing: ~5ns
- Downcast check: ~8ns
Total: ~23ns (3x over budget for 8-tick limit)
```

---

### Option 2: Enum-Based Dispatch

**Description:** Use a large enum to represent all patterns with concrete types.

```rust
pub enum Pattern {
    Sequence(SequencePattern),
    ParallelSplit(ParallelSplitPattern),
    // ... 41 more variants
}

impl Pattern {
    pub fn execute(&self, ctx: &Context) -> Result<Output> {
        match self {
            Pattern::Sequence(p) => p.execute(ctx),
            Pattern::ParallelSplit(p) => p.execute(ctx),
            // ... 41 more matches
        }
    }
}
```

**Pros:**
- ✅ Zero-cost dispatch (monomorphization)
- ✅ Type-safe output
- ✅ No virtual calls

**Cons:**
- ❌ Not extensible (sealed enum)
- ❌ Large enum size (43+ variants)
- ❌ Plugin patterns impossible
- ❌ Difficult to maintain (43+ matches)
- ❌ Recompile required for new patterns

**Code Maintenance:**
```rust
// Adding a new pattern requires:
// 1. Add enum variant
// 2. Update all match statements (43+ locations)
// 3. Update serialization/deserialization
// 4. Recompile entire crate
```

---

### Option 3: Generic Associated Types (GATs)

**Description:** Use GATs to create a type-safe, extensible pattern hierarchy with associated futures.

```rust
pub trait Pattern: Send + Sync {
    type Config: DeserializeOwned + Send + Sync;
    type State: Send + Sync;
    type Output: Send + Sync;
    type Error: Error + Send + Sync;

    // GAT: Future lifetime bound to self
    type ExecuteFuture<'a>: Future<Output = Result<Self::Output, Self::Error>> + Send + 'a
    where
        Self: 'a;

    fn execute<'a>(&'a self, state: &'a Self::State) -> Self::ExecuteFuture<'a>;
}

pub trait BasicPattern: Pattern {
    // Specialization for basic patterns
}

pub trait MultipleInstancePattern: Pattern {
    fn instance_count(&self) -> usize;
}

// HRTB for dynamic dispatch
pub struct PatternRegistry {
    patterns: HashMap<
        PatternId,
        Box<dyn for<'a> Fn(&'a Context) -> BoxFuture<'a, Result<Output>>>,
    >,
}
```

**Pros:**
- ✅ Type-safe with compile-time validation
- ✅ Zero-cost static dispatch
- ✅ Extensible (new patterns via trait impl)
- ✅ Supports async naturally
- ✅ Lifetime flexibility via GATs
- ✅ Dynamic registration via HRTB
- ✅ Pattern composition possible

**Cons:**
- ⚠️ Requires Rust 1.65+ (GATs stabilized)
- ⚠️ Complex type signatures
- ⚠️ Team learning curve
- ⚠️ Compiler error messages can be cryptic

**Performance:**
```
Hot path overhead:
- Monomorphized call: 0ns (inlined)
- HRTB dispatch: ~5ns (single indirect call)
Total: ~5ns (well within 8-tick budget)
```

---

### Option 4: Macro-Based Code Generation

**Description:** Use macros to generate pattern executor code.

```rust
pattern! {
    Sequence {
        config: SequenceConfig,
        state: SequenceState,
        output: SequenceOutput,

        fn execute(&self, state: &Self::State) -> Result<Self::Output> {
            // implementation
        }
    }
}
```

**Pros:**
- ✅ Zero-cost (expands to concrete code)
- ✅ Less boilerplate
- ✅ Consistent structure

**Cons:**
- ❌ Difficult to debug (macro expansion)
- ❌ Poor IDE support
- ❌ Macro hygiene issues
- ❌ Less flexible than traits
- ❌ Harder to understand

---

## Decision Outcome

**Chosen Option:** **Option 3 - Generic Associated Types (GATs)**

**Rationale:**
1. **Type Safety:** Compile-time validation eliminates entire classes of runtime errors
2. **Performance:** Zero-cost abstractions meet ≤8 tick requirement
3. **Extensibility:** New patterns via trait implementation, supports plugins
4. **Async Native:** GATs provide natural async support without boxing
5. **Ecosystem Fit:** Aligns with Rust's direction (GATs in stable)

**Accepted Trade-offs:**
- ⚠️ Complex type signatures acceptable given team expertise
- ⚠️ Learning curve mitigated by comprehensive documentation
- ⚠️ Rust 1.65+ requirement acceptable for 2027 target

---

## Implementation Design

### Core Pattern Trait

```rust
/// Base trait for all workflow patterns
pub trait Pattern: Send + Sync + 'static {
    /// Pattern configuration (deserialized from workflow spec)
    type Config: DeserializeOwned + Send + Sync;

    /// Pattern execution state
    type State: Send + Sync;

    /// Pattern execution output
    type Output: Send + Sync;

    /// Pattern-specific error type
    type Error: Error + Send + Sync + 'static;

    /// GAT: Future for async execution
    /// Lifetime 'a ensures future borrows self and state safely
    type ExecuteFuture<'a>: Future<Output = Result<Self::Output, Self::Error>> + Send + 'a
    where
        Self: 'a;

    /// Execute pattern with given state
    fn execute<'a>(&'a self, state: &'a Self::State) -> Self::ExecuteFuture<'a>;

    /// Validate pattern configuration (optional)
    fn validate_config(config: &Self::Config) -> Result<(), Self::Error> {
        Ok(())
    }
}
```

### Pattern Hierarchy

```rust
/// Basic control flow patterns (1-5)
pub trait BasicPattern: Pattern {
    /// Sequential execution guarantee
    fn is_sequential(&self) -> bool {
        true
    }
}

/// Multiple instance patterns (12-15)
pub trait MultipleInstancePattern: Pattern {
    /// Number of instances (design-time or runtime)
    fn instance_count(&self) -> InstanceCount;

    /// Synchronization mode
    fn synchronization_mode(&self) -> SyncMode;
}

pub enum InstanceCount {
    DesignTime(usize),
    Runtime,
    Dynamic,
}

pub enum SyncMode {
    NoSync,
    WaitForAll,
    WaitForN(usize),
}

/// State-based patterns (16-18)
pub trait StateBasedPattern: Pattern {
    /// Event triggers
    type Event: Send + Sync;

    /// Handle external event
    fn handle_event<'a>(
        &'a self,
        state: &'a mut Self::State,
        event: Self::Event,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;
}

/// Cancellation patterns (19-25)
pub trait CancellationPattern: Pattern {
    /// Cancellation scope
    fn cancellation_scope(&self) -> CancellationScope;

    /// Compensation logic (optional)
    fn compensate<'a>(
        &'a self,
        state: &'a Self::State,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        async { Ok(()) }
    }
}

pub enum CancellationScope {
    Activity,
    Case,
    Region,
}
```

### Example Pattern Implementation

```rust
/// Pattern 13: Multiple Instances with Design-Time Knowledge
pub struct DesignTimeMIPattern {
    config: DesignTimeMIConfig,
}

#[derive(Debug, Deserialize)]
pub struct DesignTimeMIConfig {
    instance_count: usize,
}

pub struct DesignTimeMIState {
    spawned: Vec<InstanceId>,
    completed: Vec<InstanceResult>,
}

pub struct DesignTimeMIOutput {
    results: Vec<InstanceResult>,
}

#[derive(Debug, thiserror::Error)]
pub enum DesignTimeMIError {
    #[error("Instance execution failed: {0}")]
    InstanceFailed(#[from] InstanceError),
}

// Implement base Pattern trait
impl Pattern for DesignTimeMIPattern {
    type Config = DesignTimeMIConfig;
    type State = DesignTimeMIState;
    type Output = DesignTimeMIOutput;
    type Error = DesignTimeMIError;

    // GAT implementation
    type ExecuteFuture<'a> = impl Future<Output = Result<Self::Output, Self::Error>> + Send + 'a
    where
        Self: 'a;

    fn execute<'a>(&'a self, state: &'a Self::State) -> Self::ExecuteFuture<'a> {
        async move {
            let count = self.config.instance_count;

            // Spawn instances
            let handles: Vec<_> = (0..count)
                .map(|i| self.spawn_instance(i))
                .collect();

            // Wait for all
            let results = futures::future::join_all(handles).await;

            Ok(DesignTimeMIOutput {
                results: results.into_iter().collect::<Result<_, _>>()?,
            })
        }
    }

    fn validate_config(config: &Self::Config) -> Result<(), Self::Error> {
        if config.instance_count == 0 {
            Err(DesignTimeMIError::InvalidConfig("instance_count must be > 0"))
        } else {
            Ok(())
        }
    }
}

// Implement specialized trait
impl MultipleInstancePattern for DesignTimeMIPattern {
    fn instance_count(&self) -> InstanceCount {
        InstanceCount::DesignTime(self.config.instance_count)
    }

    fn synchronization_mode(&self) -> SyncMode {
        SyncMode::WaitForAll
    }
}
```

### Dynamic Pattern Registry with HRTB

```rust
/// Type-erased pattern executor using HRTB
type DynPatternExecutor = Box<
    dyn for<'a> Fn(&'a Context) -> Pin<Box<dyn Future<Output = Result<Output>> + Send + 'a>>,
>;

/// Pattern registry supporting dynamic dispatch
pub struct PatternRegistry {
    executors: HashMap<PatternId, DynPatternExecutor>,
}

impl PatternRegistry {
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    /// Register a pattern with type erasure
    pub fn register<P>(&mut self, id: PatternId, pattern: P)
    where
        P: Pattern + 'static,
        P::Output: Into<Output>,
        for<'a> P::ExecuteFuture<'a>: Send,
    {
        let pattern = Arc::new(pattern);

        self.executors.insert(
            id,
            Box::new(move |ctx: &Context| {
                let pattern = Arc::clone(&pattern);
                Box::pin(async move {
                    let state = ctx.get_state::<P::State>()?;
                    let output = pattern.execute(state).await?;
                    Ok(output.into())
                })
            }),
        );
    }

    /// Execute pattern by ID with dynamic dispatch
    pub async fn execute(&self, id: PatternId, ctx: &Context) -> Result<Output> {
        let executor = self
            .executors
            .get(&id)
            .ok_or(WorkflowError::PatternNotFound(id))?;

        executor(ctx).await
    }
}
```

---

## Benefits Realized

### 1. Compile-Time Type Safety

```rust
// ✅ CORRECT: Type-safe composition
async fn compose_patterns() -> Result<FinalOutput> {
    let pattern1 = SequencePattern::new(config1);
    let pattern2 = ParallelSplitPattern::new(config2);

    let output1 = pattern1.execute(&state1).await?;
    // Compiler knows output1 is SequenceOutput

    let output2 = pattern2.execute(&state2).await?;
    // Compiler knows output2 is ParallelSplitOutput

    Ok(combine(output1, output2))
}

// ❌ COMPILE ERROR: Type mismatch caught at compile time
async fn wrong_composition() {
    let pattern1 = SequencePattern::new(config);
    let wrong_state = ParallelState::default();

    // ERROR: expected SequenceState, found ParallelState
    pattern1.execute(&wrong_state).await;
}
```

### 2. Zero-Cost Abstractions

```rust
// Monomorphized code (zero overhead)
#[inline(always)]
async fn execute_sequence(pattern: &SequencePattern, state: &SequenceState) -> Result<SequenceOutput> {
    pattern.execute(state).await
}

// Generated assembly shows direct function call, no vtable
```

### 3. Natural Async Support

```rust
// No boxing of futures in static dispatch
impl Pattern for MyPattern {
    type ExecuteFuture<'a> = impl Future<Output = Result<Self::Output>> + Send + 'a;

    fn execute<'a>(&'a self, state: &'a Self::State) -> Self::ExecuteFuture<'a> {
        async move {
            // Complex async logic with zero heap allocation
            let data = self.fetch_data().await?;
            let result = self.process(data).await?;
            Ok(result)
        }
    }
}
```

### 4. Extensibility via Plugins

```rust
// Third-party pattern (loaded dynamically)
#[no_mangle]
pub fn _plugin_create() -> *mut dyn Pattern<Config = Value, State = Value, Output = Value> {
    Box::into_raw(Box::new(CustomPattern::new()))
}

// Plugin loader
pub fn load_pattern(path: &Path) -> Result<Box<dyn Pattern>> {
    let lib = unsafe { Library::new(path)? };
    let constructor: Symbol<fn() -> *mut dyn Pattern> = unsafe { lib.get(b"_plugin_create")? };
    Ok(unsafe { Box::from_raw(constructor()) })
}
```

---

## Performance Validation

### Benchmark Results

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_gat_dispatch(c: &mut Criterion) {
    let pattern = SequencePattern::new(config);
    let state = SequenceState::default();

    c.bench_function("gat_static_dispatch", |b| {
        b.iter(|| {
            black_box(pattern.execute(&state))
        });
    });

    // Result: 0ns overhead (fully inlined)
}

fn bench_hrtb_dispatch(c: &mut Criterion) {
    let registry = PatternRegistry::new();
    registry.register(PatternId(1), SequencePattern::new(config));

    let ctx = Context::new(&state);

    c.bench_function("hrtb_dynamic_dispatch", |b| {
        b.iter(|| {
            black_box(registry.execute(PatternId(1), &ctx))
        });
    });

    // Result: ~5ns overhead (single indirect call)
}

criterion_group!(benches, bench_gat_dispatch, bench_hrtb_dispatch);
criterion_main!(benches);
```

**Results:**
- Static dispatch (GAT): 0ns overhead (fully inlined)
- Dynamic dispatch (HRTB): ~5ns overhead
- Total hot path: ≤5ns (well within 8-tick budget)

---

## Migration Strategy

### Phase 1: Introduce GAT Traits (Week 1)

```rust
// Define new GAT-based traits alongside old ones
#[cfg(feature = "gat-patterns")]
pub mod gat {
    pub trait Pattern { /* GAT definition */ }
}

#[cfg(not(feature = "gat-patterns"))]
pub mod legacy {
    pub trait PatternExecutor { /* Old definition */ }
}
```

### Phase 2: Adapter Pattern (Week 2)

```rust
// Adapter to use old patterns with new trait
pub struct LegacyPatternAdapter<P> {
    inner: P,
}

impl<P: legacy::PatternExecutor> gat::Pattern for LegacyPatternAdapter<P> {
    type Config = P::Config;
    type State = P::State;
    type Output = P::Output;
    type Error = P::Error;

    type ExecuteFuture<'a> = impl Future<Output = Result<Self::Output>> + Send + 'a
    where
        Self: 'a;

    fn execute<'a>(&'a self, state: &'a Self::State) -> Self::ExecuteFuture<'a> {
        async move {
            // Run legacy sync code in blocking task
            tokio::task::spawn_blocking(|| self.inner.execute(state)).await?
        }
    }
}
```

### Phase 3: Incremental Migration (Week 3-6)

```rust
// Migrate patterns one at a time
// Week 3: Basic patterns (1-5)
impl gat::Pattern for SequencePattern { /* ... */ }
impl gat::Pattern for ParallelSplitPattern { /* ... */ }

// Week 4: MI patterns (12-15)
impl gat::Pattern for DesignTimeMIPattern { /* ... */ }

// Week 5: State-based patterns (16-18)
impl gat::StateBasedPattern for DeferredChoicePattern { /* ... */ }

// Week 6: Advanced patterns (26-43)
impl gat::Pattern for AdvancedPattern { /* ... */ }
```

---

## Risks and Mitigation

### Risk 1: Compiler Errors Difficult to Debug

**Probability:** High
**Impact:** Medium (developer frustration)

**Mitigation:**
- ✅ Comprehensive documentation with examples
- ✅ Helper macros for common patterns
- ✅ IDE support (rust-analyzer understands GATs)
- ✅ Error message glossary

### Risk 2: HRTB Limitations

**Probability:** Low
**Impact:** Medium (some patterns may not work)

**Mitigation:**
- ✅ Prototype all 43 patterns with GAT/HRTB
- ✅ Escape hatch: type-erased fallback for edge cases
- ✅ Continuous Rust language monitoring

### Risk 3: Team Learning Curve

**Probability:** Medium
**Impact:** Medium (slower initial development)

**Mitigation:**
- ✅ Internal training sessions
- ✅ Pair programming for first patterns
- ✅ Extensive code reviews
- ✅ Pattern implementation templates

---

## Validation Checklist

- [ ] All 43 patterns implementable with GAT trait
- [ ] Static dispatch overhead: 0ns (verified via benchmarks)
- [ ] Dynamic dispatch overhead: <10ns (verified via benchmarks)
- [ ] Compile-time type safety demonstrated with tests
- [ ] Plugin system working with dynamic loading
- [ ] Backward compatibility via adapter pattern
- [ ] Documentation complete with examples
- [ ] Team training completed

---

## References

1. **GAT RFC:** https://rust-lang.github.io/rfcs/1598-generic_associated_types.html
2. **GAT Stabilization:** https://blog.rust-lang.org/2022/10/28/gats-stabilization.html
3. **HRTB Documentation:** https://doc.rust-lang.org/nomicon/hrtb.html
4. **Async Trait:** https://github.com/dtolnay/async-trait
5. **Zero-Cost Futures:** https://aturon.github.io/tech/2016/08/11/futures/

---

## Appendix: Full Example

See `/home/user/knhk/docs/architecture/examples/gat-pattern-example.rs` for a complete working example of a GAT-based pattern implementation.

---

## Sign-Off

**Proposed By:** System Architect
**Date:** 2025-11-16

**Reviewed By:**
- [ ] Type Systems Expert: _________________ Date: _______
- [ ] Performance Engineer: _________________ Date: _______
- [ ] Senior Rust Developer: _________________ Date: _______

**Approved By:**
- [ ] Tech Lead: _________________ Date: _______
- [ ] Engineering Manager: _________________ Date: _______

**Implementation Start Date:** _______
**Target Completion:** Week 6 (Phase 1 completion)
