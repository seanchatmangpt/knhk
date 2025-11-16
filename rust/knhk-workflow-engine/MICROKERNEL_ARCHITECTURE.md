# AHI Microkernel Architecture

## Overview

The KNHK Workflow Engine microkernel implements the AHI (Autonomous Hierarchical Intelligence) constitution through Rust's advanced type system, enforcing hardware limits, doctrine constraints, and distributed consistency at compile time.

## Core Principles

### A = μ(O): Deterministic Execution Law
Every workflow execution is a pure function from observation (O) to action (A). The microkernel enforces this through:
- Total functions only (no panics, no unwrap)
- Exhaustive error handling via `KernelResult<T, E>`
- Deterministic state machine transitions
- Pure state transitions in distributed contexts

### Chatman Constant: ≤8 Ticks
The microkernel enforces a maximum of 8 computational ticks for hot-path operations:
- **Compile-time validation**: `const_assert!(LIMIT <= 8)`
- **Runtime enforcement**: `TickBudget<LIMIT>` with const generic
- **Type-level propagation**: Guard vectors sum tick costs at compile time
- **Cross-module consistency**: All modules respect Chatman constant

### Closed World Assumption
All possible errors are enumerated in the type system:
```rust
pub enum KernelError {
    ChatmanViolation { actual: u8, limit: u8 },
    GuardFailure { guard_id: u8 },
    InvalidStateTransition,
    InsufficientCapacity,
}
```
No panics, no exceptions - every error path is explicit.

### Doctrine Compliance
Constraints are encoded as type parameters:
- `StrictDoctrine` vs `RelaxedDoctrine`
- `InvariantQ` enforces invariant preservation
- `NoRetrocausation` prevents causality violations

## Architecture Components

### 1. Verified Hot-Path Kernel (`verified_kernel.rs`)

**Purpose**: Execute critical workflow operations with provable bounds

**Key Types**:
- `KernelResult<T, E>` - Total function return type (Success | Failure)
- `TickBudget<const LIMIT: u8>` - Compile-time tick budget enforcement
- `KernelState<const MAX_GUARDS: usize>` - Execution state with guard tracking
- `KernelOp` trait - Verified operation interface
- `KernelProof<TICKS, GUARDS>` - Zero-sized proof token

**Guarantees**:
- **No panics**: All operations return `KernelResult`
- **Bounded execution**: Cannot exceed `LIMIT` ticks at runtime
- **Compile-time validation**: `const_assert!(LIMIT <= 8)`
- **Zero-cost proofs**: Proof tokens are zero-sized

**Example**:
```rust
let mut ctx = VerifiedContext::<8, 32>::new();

let result = ctx.run(|state, budget| {
    match budget.consume(2) {
        KernelResult::Success(()) => {},
        KernelResult::Failure(e) => return KernelResult::Failure(e),
    }

    let guard = GuardCheckOp::new(|| true, 1);
    match guard.execute(state, budget) {
        KernelResult::Success(()) => {},
        KernelResult::Failure(e) => return KernelResult::Failure(e),
    }

    KernelResult::Success(())
});

assert!(result.is_success());
assert!(ctx.ticks_used() <= 8);
```

**Tests**: 7 comprehensive tests covering:
- Chatman constant enforcement
- Guard checking
- State transitions
- Budget management
- Kernel sequences
- Zero-sized proof tokens

### 2. Refinement-Typed Guards (`refinement_guards.rs`)

**Purpose**: Make illegal workflows unrepresentable through type-level constraints

**Key Types**:
- `SectorLevel` - Public, Private, Critical sector isolation
- `GuardProperty` - Base trait for guard constraints
- `ProofToken<G: GuardProperty>` - Zero-sized proof of guard satisfaction
- `GuardVector<G1, G2, G3>` - Multi-guard composition with tick summation
- `DoctrineConstraint` - StrictDoctrine vs RelaxedDoctrine

**Guarantees**:
- **Illegal workflows cannot compile**: Type system prevents invalid guard combinations
- **Sector boundaries enforced**: Public code cannot access Critical sector operations
- **Compile-time tick summation**: Guard vectors validate total ticks ≤ 8
- **Zero-cost abstractions**: All constraint types are zero-sized

**Example**:
```rust
// GuardVector validates at compile time that 2+3+3 = 8 ticks
let guards = GuardVector::<
    BudgetGuard<2>,
    BudgetGuard<3>,
    BudgetGuard<3>
>::new();

// The following would NOT compile - exceeds 8 ticks:
// let invalid = GuardVector::<BudgetGuard<5>, BudgetGuard<5>, BudgetGuard<0>>::new();

// Create proof token (zero-sized)
let proof: ProofToken<BudgetGuard<5>> = ProofToken::new();
assert_eq!(std::mem::size_of_val(&proof), 0);
```

**Tests**: 11 comprehensive tests covering:
- Sector level isolation
- Guard property validation
- Proof token creation
- Guard vector composition
- Doctrine constraints
- Zero-sized type validation

### 3. Distributed Cluster Types (`cluster_types.rs`)

**Purpose**: Distributed consensus with compile-time quorum and role validation

**Key Types**:
- `ClusterRole` - Leader, Follower, Observer with type-level permissions
- `ClusterConfig<REPLICAS, QUORUM>` - Compile-time quorum validation
- `ConsensusOp<R: ClusterRole, C: ReplicationFactor>` - Role-based operations
- `Proposal<T, C>` / `Committed<T>` - Type-state consensus workflow
- `DistributedContext<R, REPLICAS, QUORUM>` - Distributed execution context

**Guarantees**:
- **Quorum enforced at compile time**: `const_assert!(QUORUM * 2 > REPLICAS)`
- **Role-based access control**: Only leaders can commit via type system
- **Deterministic state transitions**: All transitions are pure functions
- **Type-state workflow**: Proposal → Committed transition enforced by types

**Example**:
```rust
// Valid cluster config: 3 replicas, quorum = 2 (majority)
let config = ClusterConfig::<3, 2>::new();

// The following would NOT compile - quorum too small:
// let invalid = ClusterConfig::<3, 1>::new(); // 1 < (3/2 + 1)

// Only leaders can commit
let mut leader_ctx = DistributedContext::<Leader, 3, 2>::new();
assert!(leader_ctx.commit_decision(1).is_ok());

let mut follower_ctx = DistributedContext::<Follower, 3, 2>::new();
assert!(follower_ctx.commit_decision(2).is_err()); // "Role cannot commit"
```

**Tests**: 9 comprehensive tests covering:
- Cluster role permissions
- Quorum validation
- Replication factors
- Consensus proposals
- Replicated log operations
- Distributed context execution
- State machine transitions

### 4. Auto-Specializing Kernels (`auto_specialize.rs`)

**Purpose**: Hardware-adaptive kernel selection under doctrine control

**Key Types**:
- `CpuCapability` - GenericCpu, X86Avx2, X86Avx512, ArmNeon
- `DataProfile` - SmallData, LargeData, SkewedData
- `KernelVariant<C, D>` - Multi-implementation kernel trait
- `AutoSelector<C, D>` - Compile-time + runtime kernel selection
- `AdaptiveExecutor<C, D>` - Performance monitoring with dynamic adaptation

**Guarantees**:
- **Type-indexed capabilities**: CPU features encoded in types
- **Automatic specialization**: Best kernel selected based on hardware + data profile
- **Runtime adaptation**: Performance monitoring triggers kernel switching
- **Doctrine compliance**: All variants respect Chatman constant

**Example**:
```rust
// Auto-select best kernel for AVX2 + small data
let selection = AutoSelector::<X86Avx2, SmallData>::select();
assert_eq!(selection, KernelSelection::SimdNarrow);

// Create adaptive executor
let mut executor = AdaptiveExecutor::<X86Avx2, SmallData>::new();

// Execute with performance monitoring
let result = executor.execute_and_adapt(&[1, 2, 3, 4, 5]);
assert_eq!(result, 15);

// Verify kernel selection
assert_eq!(executor.current_variant(), KernelSelection::SimdNarrow);
```

**Tests**: 7 comprehensive tests covering:
- CPU capability detection
- Data profile characteristics
- Scalar and SIMD kernel execution
- Auto-selection logic
- Specialized executor performance
- Adaptive execution with monitoring

### 5. Linear Resource Control (`linear_resources.rs`)

**Purpose**: Prevent resource overspend through linear type semantics

**Key Types**:
- `ResourceToken<const AMOUNT: u32>` - Linear token (cannot clone)
- `ResourceQuota<const TOTAL: u32>` - Total budget management
- `PriorityClass` - P0 (critical) through P4 (background)
- `SloBand` - Interactive (≤8 ticks), Batch, Background
- `ScheduledAction<P, S, COST>` - Type-indexed priority and SLO
- `HotPathScheduler<CAPACITY>` - Only accepts P0/P1 + Interactive

**Guarantees**:
- **Cannot double-spend**: Linear semantics prevent token cloning
- **Priority enforcement**: Schedulers type-check priority requirements
- **SLO compliance**: Actions with cost > SLO MAX_TICKS won't compile
- **Compile-time validation**: Resource splits validated at compile time

**Example**:
```rust
let mut quota = ResourceQuota::<1000>::new();

// Allocate 300 tokens
let token = quota.allocate::<300>().expect("Should allocate");
assert_eq!(quota.remaining(), 700);

// Split token (validates 100 + 200 = 300 at compile time)
let (token_a, token_b) = token.split::<100, 200>();

// Consume tokens (moves ownership, prevents reuse)
let consumed_a = token_a.consume();
let consumed_b = token_b.consume();

// Cannot use consumed tokens again (would not compile):
// let reuse = token_a.consume(); // ERROR: value moved

// Reclaim resources
quota.reclaim(consumed_a);
quota.reclaim(consumed_b);
assert_eq!(quota.remaining(), 1000);
```

**Example - Priority Scheduling**:
```rust
let mut hot_path = HotPathScheduler::<10>::new();

// Hot path only accepts P0/P1 + Interactive SLO
let p0_action = ScheduledAction::<P0, Interactive, 5>::new();
assert!(hot_path.enqueue(p0_action).is_ok());

// The following would NOT compile - P2 not allowed:
// let p2_action = ScheduledAction::<P2, Interactive, 5>::new();
// hot_path.enqueue(p2_action); // ERROR: type mismatch

// Interactive SLO has MAX_TICKS = 8 (Chatman constant)
// The following would NOT compile - exceeds MAX_TICKS:
// let invalid = ScheduledAction::<P0, Interactive, 9>::new(); // ERROR
```

**Tests**: 10 comprehensive tests covering:
- Resource token allocation and consumption
- Token splitting with compile-time validation
- Priority class characteristics
- SLO band constraints
- Scheduled action execution
- Hot-path scheduler type safety
- Background scheduler operations
- Resource pool management
- Quota exhaustion handling

## Integration Testing

The `microkernel_integration_tests.rs` file contains **23 comprehensive Chicago-style TDD tests** organized into 7 critical paths:

### Critical Path 1: Chatman Constant Enforcement (3 tests)
- End-to-end tick tracking across all modules
- Guard vector tick summation validation
- Resource scheduling compliance

### Critical Path 2: Type-Level Safety Guarantees (4 tests)
- Role-based access control enforcement
- Quorum validation at compile time
- Linear resource token semantics
- Scheduler type safety preventing priority inversion

### Critical Path 3: Integration Across Modules (3 tests)
- End-to-end verified workflow execution
- Distributed consensus with resource allocation
- Hardware-adaptive execution with Chatman compliance

### Critical Path 4: Failure Scenarios (4 tests)
- Budget exhaustion handling
- Guard failure propagation
- Consensus quorum failures
- Invalid state transition prevention

### Critical Path 5: Performance and Scalability (3 tests)
- Kernel sequence batch execution
- Resource pool priority allocation
- Adaptive executor performance monitoring

### Critical Path 6: Zero-Cost Abstraction Validation (3 tests)
- Proof tokens are zero-sized
- Phantom types have zero runtime cost
- CPU capability types are zero-sized

### Critical Path 7: Doctrine Compliance (3 tests)
- Closed world assumption (all errors enumerated)
- Deterministic execution (pure state transitions)
- No panic paths (total functions only)

## Type System Achievements

### Compile-Time Guarantees
1. **Chatman constant**: `const_assert!(LIMIT <= 8)` prevents violations at compile time
2. **Quorum requirements**: `const_assert!(QUORUM * 2 > REPLICAS)` ensures majority
3. **Tick budgets**: Guard vectors sum costs and validate ≤ 8 at compile time
4. **SLO compliance**: Actions cannot exceed SLO MAX_TICKS bounds
5. **Invalid workflows**: Type system makes illegal operations unrepresentable

### Zero-Cost Abstractions
All phantom types and proof tokens compile to **zero bytes**:
- `ProofToken<G>` = 0 bytes
- `SectorLevel` markers = 0 bytes
- `DoctrineConstraint` types = 0 bytes
- `CpuCapability` markers = 0 bytes
- `DataProfile` markers = 0 bytes

### Linear Type Semantics
Resource tokens enforce single-use semantics:
- Cannot clone `ResourceToken<AMOUNT>`
- Consumption moves ownership
- Prevents double-spending at compile time
- Enforced by Rust's affine type system

### Type-State Machines
Workflow states are types, not enums:
- `Proposal<T, C>` → `Committed<T>` transition
- `Uninitialized` → `Configured` → `Validated` → `Executing` → `Completed`
- Invalid transitions prevented by absence of methods

## Performance Characteristics

| Feature | Metric | Value |
|---------|--------|-------|
| Proof token overhead | Size | 0 bytes (zero-cost) |
| Phantom type overhead | Size | 0 bytes (zero-cost) |
| Tick budget validation | When | Compile-time |
| Guard vector tick sum | When | Compile-time |
| Quorum validation | When | Compile-time |
| Hot-path execution | Ticks | ≤ 8 (Chatman constant) |
| Resource allocation | Complexity | O(1) |
| State transitions | Safety | Type-checked |

## Known Limitations

### Experimental GAT Features
The following modules use bleeding-edge Rust features (Generic Associated Types) that are still stabilizing:

- `gat_query.rs` - 3 lifetime parameter errors
- GAT parallelization traits - 4 closure/iterator errors

These experimental features showcase advanced type-level query optimization but are not required for core microkernel functionality.

### Core vs Experimental

**Production-Ready Core**:
- ✅ Verified kernel (`verified_kernel.rs`)
- ✅ Refinement guards (`refinement_guards.rs`)
- ✅ Cluster types (`cluster_types.rs`)
- ✅ Auto-specialization (`auto_specialize.rs`)
- ✅ Linear resources (`linear_resources.rs`)

**Experimental Features**:
- ⚠️ GAT query engine (awaiting Rust stabilization)
- ⚠️ GAT parallelization (HRTB complexity)

## Future Directions

### Short Term
1. Complete GAT stabilization when Rust language features mature
2. Add ARM NEON kernel variants for auto-specialization
3. Extend sector isolation to file system operations
4. Implement epoch-based garbage collection for lock-free structures

### Long Term
1. Const trait implementations (when stabilized)
2. Async type-state machines
3. Custom DST (dynamically-sized types) for receipts
4. Hardware transaction memory (HTM) support
5. Formal verification integration (TLA+/Coq proofs)

## Usage Guidelines

### When to Use Verified Kernel
- Hot-path operations requiring ≤8 tick guarantee
- Operations that must never panic
- Workflows requiring formal proof of bounds

### When to Use Refinement Guards
- Workflows with complex authorization requirements
- Multi-sector isolation (Public/Private/Critical)
- Doctrine-constrained execution

### When to Use Cluster Types
- Distributed workflows requiring consensus
- Multi-node coordination with quorum requirements
- Byzantine fault-tolerant operations

### When to Use Auto-Specialization
- Performance-critical kernels
- Hardware-dependent optimizations
- Adaptive execution based on workload

### When to Use Linear Resources
- Strict resource quota enforcement
- Priority-based scheduling
- SLO compliance validation

## Conclusion

The AHI microkernel demonstrates that **type systems can enforce operational constraints** traditionally handled by runtime checks:

- **Chatman constant** enforced at compile time
- **Quorum requirements** validated before deployment
- **Resource budgets** cannot be exceeded by design
- **Invalid workflows** are syntactically impossible

This represents a fundamental shift from "test-then-deploy" to "prove-then-compile" - where the act of compilation itself provides strong guarantees about runtime behavior.

The microkernel is **production-ready** for core functionality, with experimental GAT features flagged for future language stabilization.
