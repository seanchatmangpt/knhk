# Compile-Time Workflow Validation

## Overview

KNHK's compile-time workflow validation system uses Rust's advanced type system and procedural macros to catch workflow errors **at compile time** rather than runtime. This eliminates entire classes of bugs before code even runs.

## Key Features

### 1. Workflow DSL with Procedural Macros

Define workflows in Rust code with full IDE support and compile-time validation:

```rust
use knhk_workflow_macros::workflow;

workflow! {
    name: UserRegistration,
    patterns: [Sequence, ExclusiveChoice, ParallelSplit, Synchronization],

    states: {
        Initial -> ValidateEmail,
        ValidateEmail -> [CreateAccount, RejectInvalid],  // Exclusive choice
        CreateAccount -> [SendWelcome, CreateProfile],    // Parallel split
        [SendWelcome, CreateProfile] -> Complete,         // Synchronization
        RejectInvalid -> Complete,
    },

    constraints: {
        max_duration: 30_000,  // 30 seconds
        max_concurrency: 100,
    },
}
```

### 2. Compile-Time Checks

The macro performs these validations **at compile time**:

- **Deadlock Detection**: Analyzes workflow graph for cycles using Petri net analysis
- **Reachability**: Ensures all states are reachable from the initial state
- **Completeness**: Verifies all paths lead to terminal states
- **Pattern Compliance**: Validates declared YAWL patterns match actual usage
- **Type Safety**: Ensures state transitions are valid
- **Resource Bounds**: Checks concurrency and duration limits

### 3. Type-Level State Machines

Use phantom types to enforce valid state transitions:

```rust
use knhk_workflow_engine::compile_time::*;

// Create workflow in Initial state
let workflow = TypedWorkflow::<Initial>::new("wf-123".to_string());

// Type-safe transitions
let workflow = workflow.validate_email();     // → EmailValidated
let workflow = workflow.create_account();     // → AccountCreated
let workflow = workflow.complete();           // → Complete

// These would be compile errors:
// workflow.validate_email();  // ERROR: method not found for TypedWorkflow<Complete>
// workflow.create_account();  // ERROR: already past that state!
```

### 4. Const Evaluation

Analyze workflow properties at compile time:

```rust
type MyWorkflow = CompileTimeWorkflow<5, 6>;  // 5 states, 6 transitions

const COMPLEXITY: usize = MyWorkflow::complexity();
const IS_SIMPLE: bool = MyWorkflow::is_simple();
const MEMORY: usize = MyWorkflow::memory_usage();

// Compile-time assertions
const_assert!(COMPLEXITY <= 100);
const_assert!(MEMORY < 1_000_000);
```

### 5. GADT-Style Workflows

Use Generalized Algebraic Data Types for stage-specific data:

```rust
// Each stage can have different data types
let workflow = WorkflowStage::<stage::Initial>::new();

let workflow = workflow.validate_email("user@example.com".to_string());
assert_eq!(workflow.email(), "user@example.com");  // Only available in EmailValidation stage

let workflow = workflow.create_account("username".to_string());
let (email, username) = workflow.account_details();  // Only available in AccountCreation stage

let workflow = workflow.complete(12345);
let (user_id, email, username) = workflow.account_data();  // Only available in Complete stage
```

## Compile Error Examples

### Deadlock Detection

```rust
workflow! {
    name: DeadlockWorkflow,
    patterns: [Sequence],
    states: {
        A -> B,
        B -> C,
        C -> A,  // Cycle!
    },
}
```

**Compile Error:**
```
error: Workflow contains deadlock cycle: A -> B -> C -> A
  --> src/workflows.rs:5:9
   |
5  |         C -> A,
   |         ^^^^^^ deadlock detected
   |
   = help: Add an exit condition, timeout, or cancellation pattern
```

### Unreachable State

```rust
workflow! {
    name: UnreachableWorkflow,
    patterns: [Sequence],
    states: {
        Initial -> StateA,
        StateA -> Complete,
        StateB -> Complete,  // Unreachable!
    },
}
```

**Compile Error:**
```
error: State 'StateB' is unreachable
  --> src/workflows.rs:8:9
   |
8  |         StateB -> Complete,
   |         ^^^^^^ unreachable state
   |
   = help: Add a transition path from Initial to StateB
```

### Invalid Pattern

```rust
workflow! {
    name: InvalidWorkflow,
    patterns: [NonExistentPattern],  // Not a valid YAWL pattern!
    states: {
        Initial -> Complete,
    },
}
```

**Compile Error:**
```
error: Unknown workflow pattern: NonExistentPattern
  --> src/workflows.rs:3:16
   |
3  |     patterns: [NonExistentPattern],
   |                ^^^^^^^^^^^^^^^^^ unknown pattern
   |
   = help: Valid patterns: Sequence, ParallelSplit, Synchronization, ExclusiveChoice, SimpleMerge, ...
```

## Type-Level Programming

### Type-Level Natural Numbers

```rust
use knhk_workflow_engine::compile_time::const_eval::nat::*;

type Zero = N0;
type One = N1;
type ChatmanConstant = N8;  // Maximum ticks for hot path

assert_eq!(ChatmanConstant::VALUE, 8);
```

### Type-Level Lists

```rust
use knhk_workflow_engine::compile_time::type_level::list::*;

type StateList = Cons<Initial, Cons<Running, Cons<Complete, Nil>>>;

const LENGTH: usize = StateList::LEN;  // 3
const HAS_RUNNING: bool = StateList::contains::<Running>::VALUE;  // true
```

### Type-Level Witnesses

```rust
use knhk_workflow_engine::compile_time::type_level::{Witness, properties::*};

// Prove that workflow is acyclic
fn validate_workflow() -> Witness<Acyclic> {
    // Validation logic here
    Witness::new()  // Only create if proven
}

// Prove that workflow terminates
fn prove_termination() -> Witness<Terminating> {
    Witness::new()
}
```

## Performance Estimation

Estimate workflow performance at compile time:

```rust
const TRANSITIONS: usize = 4;
const AVG_TICKS_PER_TRANSITION: usize = 2;
const ESTIMATED_TICKS: usize = estimate_execution_ticks(TRANSITIONS, AVG_TICKS_PER_TRANSITION);

// Enforce Chatman Constant at compile time
const_assert!(ESTIMATED_TICKS <= 8, "Exceeds Chatman Constant (8 ticks)");
```

## Integration with Runtime Validation

Compile-time validation **complements** runtime validation:

1. **Compile-time**: Catches structural errors (deadlocks, unreachable states, invalid patterns)
2. **Runtime**: Validates actual data and execution (SHACL constraints, business rules, telemetry)
3. **Weaver validation**: Proves runtime behavior matches schema (source of truth)

### Validation Hierarchy

```
┌─────────────────────────────────────────┐
│  Compile-Time Validation                │
│  - Deadlock detection                   │
│  - Type safety                          │
│  - Pattern compliance                   │
│  └─> Prevents invalid workflows from    │
│      being compiled                     │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  Runtime Validation                     │
│  - SHACL constraints                    │
│  - Business rules                       │
│  - Data validation                      │
│  └─> Validates actual execution         │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  Weaver Validation (Source of Truth)    │
│  - Schema conformance                   │
│  - Telemetry validation                 │
│  - Live runtime behavior                │
│  └─> Proves feature actually works      │
└─────────────────────────────────────────┘
```

## Benefits

### Developer Experience

1. **IDE Integration**: Full autocomplete and type checking in IDEs
2. **Early Error Detection**: Catch bugs at compile time, not production
3. **Self-Documenting**: Workflow structure is clear from code
4. **Refactoring Safety**: Invalid refactorings caught by compiler

### Performance

1. **Zero Runtime Overhead**: All validation compiled away
2. **Optimized Code Generation**: Macro generates optimal runtime code
3. **Const Evaluation**: Complexity calculations at compile time
4. **Type Erasure**: Phantom types have zero size

### Correctness

1. **Mathematical Guarantees**: Type system proves correctness
2. **No Invalid States**: Impossible to construct invalid workflows
3. **Exhaustive Checking**: Compiler verifies all paths
4. **Property Witnesses**: Type-level proofs of workflow properties

## Example: Full Workflow Lifecycle

```rust
use knhk_workflow_macros::workflow;
use knhk_workflow_engine::compile_time::*;

// 1. Define workflow with compile-time validation
workflow! {
    name: OrderProcessing,
    patterns: [Sequence, ExclusiveChoice, ParallelSplit],

    states: {
        Initial -> ValidateOrder,
        ValidateOrder -> [ProcessPayment, RejectOrder],
        ProcessPayment -> [ShipOrder, SendConfirmation],
        [ShipOrder, SendConfirmation] -> Complete,
        RejectOrder -> Complete,
    },

    constraints: {
        max_duration: 60_000,
        max_concurrency: 50,
    },
}

fn main() {
    // 2. Create workflow (type-safe)
    let workflow = OrderProcessing::new("order-123".to_string());

    // 3. Check compile-time properties
    const COMPLEXITY: usize = OrderProcessing::CALCULATED_COMPLEXITY;
    const HAS_DEADLOCK: bool = OrderProcessing::HAS_DEADLOCK;

    assert!(COMPLEXITY > 0);
    assert!(!HAS_DEADLOCK);

    // 4. Use type-safe state machine
    let typed_workflow = TypedWorkflow::<Initial>::new("order-456".to_string());
    let typed_workflow = typed_workflow
        .with_context("order_id".to_string(), serde_json::json!(456))
        .with_context("customer_id".to_string(), serde_json::json!(789));

    // 5. Execute workflow with compile-time guarantees
    // Type system ensures only valid transitions are possible
    let typed_workflow = typed_workflow.validate_email();
    let typed_workflow = typed_workflow.create_account();
    let typed_workflow = typed_workflow.complete();

    println!("Workflow completed in state: {}", typed_workflow.state_name());
}
```

## Comparison with Traditional Approaches

| Feature | Traditional Runtime | KNHK Compile-Time |
|---------|-------------------|------------------|
| Error Detection | Runtime | Compile-time |
| Type Safety | Minimal | Complete |
| Performance | Runtime overhead | Zero overhead |
| Refactoring | Risky | Safe |
| IDE Support | Limited | Full |
| Deadlock Detection | Runtime analysis | Compile-time proof |
| Invalid States | Possible | Impossible |
| Documentation | Separate | Built-in |

## Advanced Topics

### Custom State Types

```rust
// Define custom states
pub struct MyCustomState;

impl WorkflowState for MyCustomState {
    fn name() -> &'static str {
        "MyCustomState"
    }
}

// Use in type-safe workflow
impl TypedWorkflow<SomeState> {
    pub fn transition_to_custom(self) -> TypedWorkflow<MyCustomState> {
        TypedWorkflow {
            state: PhantomData,
            data: self.data,
        }
    }
}
```

### Conditional Compilation

```rust
// Different workflows for different targets
#[cfg(feature = "enterprise")]
workflow! {
    name: EnterpriseWorkflow,
    patterns: [Sequence, ParallelSplit, Synchronization],
    states: {
        Initial -> ParallelProcessing,
        ParallelProcessing -> [Task1, Task2, Task3],
        [Task1, Task2, Task3] -> Complete,
    },
}

#[cfg(not(feature = "enterprise"))]
workflow! {
    name: BasicWorkflow,
    patterns: [Sequence],
    states: {
        Initial -> Processing,
        Processing -> Complete,
    },
}
```

## See Also

- [YAWL Patterns Documentation](./yawl-patterns.md)
- [Type-Level Programming in Rust](https://doc.rust-lang.org/nomicon/phantom-data.html)
- [Procedural Macros Guide](https://doc.rust-lang.org/reference/procedural-macros.html)
- [Weaver Validation](./weaver-validation.md)
