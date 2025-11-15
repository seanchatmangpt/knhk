# Poka-Yoke Patterns in KNHK Process Mining

## Overview

This document describes the Poka-Yoke (error prevention) patterns used in the KNHK process mining module. Poka-Yoke (ポカヨケ, "mistake-proofing") is a Japanese manufacturing technique that uses design to prevent errors before they can occur.

In Rust, we leverage the type system to make invalid states **impossible** rather than **unlikely**.

## Core Principle: Make Invalid States Impossible

Traditional error handling:
```rust
// ❌ Traditional: Runtime validation
fn create_case_id(id: u64) -> Result<CaseID, Error> {
    if id == 0 {
        return Err(Error::InvalidId);
    }
    Ok(CaseID(id))
}

// Problem: Can still accidentally use raw u64 as CaseID
let raw_id: u64 = 0;
// Oops, forgot to validate!
```

Poka-Yoke approach:
```rust
// ✅ Poka-Yoke: Compile-time prevention
pub struct CaseID(NonZeroU64);

impl CaseID {
    pub fn new(id: u64) -> Result<Self, InvalidIdError> {
        NonZeroU64::new(id)
            .map(CaseID)
            .ok_or(InvalidIdError::ZeroId)
    }
}

// Impossible to use raw u64 as CaseID (type mismatch)
// Impossible to create CaseID(0) (NonZeroU64 prevents it)
```

## Pattern 1: Validated Newtypes

### What It Prevents
- Using raw primitive types for domain concepts
- Creating invalid values (zero IDs, empty strings, out-of-range probabilities)
- Mixing different ID types (CaseID vs EventID)
- Skipping validation checks

### How It Works

**Type Safety Through Wrapping:**
```rust
// Each domain concept gets its own type
pub struct CaseID(NonZeroU64);    // Cannot be zero
pub struct EventID(NonZeroU32);   // Cannot be zero, different from CaseID
pub struct ActivityName(String);  // Cannot be empty
pub struct Probability(u32);      // Must be 0-100
```

**Validated Construction:**
```rust
impl ActivityName {
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidStringError> {
        let name = name.into();

        if name.is_empty() {
            return Err(InvalidStringError::EmptyString);
        }

        if name.len() > Self::MAX_LENGTH {
            return Err(InvalidStringError::TooLong { ... });
        }

        Ok(ActivityName(name))
    }
}
```

**TryFrom for Safe Conversion:**
```rust
impl TryFrom<u64> for CaseID {
    type Error = InvalidIdError;

    fn try_from(id: u64) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

// Usage:
let case_id = CaseID::try_from(42)?;  // Validated
// let case_id: CaseID = 42;          // Compiler error!
```

### Benefits
1. **Compile-Time Type Safety**: Cannot mix CaseID and EventID
2. **Runtime Validation**: Invalid values rejected at construction
3. **Self-Documenting**: Type name conveys meaning
4. **Zero Runtime Cost**: Newtypes are zero-cost abstractions

### Examples in Code

**File:** `src/types.rs`
- `CaseID` - Cannot be zero, distinct from other IDs
- `EventID` - Cannot be zero, different type from CaseID
- `ActivityName` - Cannot be empty, length-limited
- `Probability` - Must be 0-100
- `Count` - Cannot be zero, has increment operations
- `Timestamp` - Type-safe time values
- `Duration` - Type-safe intervals

## Pattern 2: Enum-Based State Machines

### What It Prevents
- Invalid state transitions (e.g., Initial -> Completed without Running)
- Accessing data from wrong state (e.g., getting duration from Initial state)
- Forgetting to handle states (exhaustive match required)
- Using stale data after state transitions

### How It Works

**State as Enum:**
```rust
pub enum WorkflowState {
    Initial { created_at: Timestamp },
    Running { started_at: Timestamp, case_count: u64 },
    Paused { paused_at: Timestamp, cases_processed: u64 },
    Completed { completed_at: Timestamp, total_cases: u64, total_duration: Duration },
    Error { error_at: Timestamp, error_message: String, ... },
}
```

**Consuming Transitions:**
```rust
impl WorkflowState {
    pub fn start(self) -> Result<Self, StateError> {
        match self {
            WorkflowState::Initial { .. } => {
                Ok(WorkflowState::Running {
                    started_at: Timestamp::now(),
                    case_count: 0,
                })
            }
            _ => Err(StateError::InvalidTransition { ... })
        }
    }
}
```

**Why Consuming?**
```rust
let workflow = WorkflowState::new();
let workflow = workflow.start().unwrap();  // Consumes old state

// COMPILER ERROR: workflow was moved
// workflow.start();  // Cannot use again!
```

### Valid Transitions

```text
Initial -> Running -> Completed ✓
Initial -> Running -> Paused -> Running -> Completed ✓
Running -> Error -> Running (with recovery) ✓

Initial -> Completed ✗ (compile error: no method)
Completed -> Running ✗ (value consumed, cannot use)
Paused -> Completed ✗ (must resume first)
```

### Benefits
1. **Impossible Invalid Transitions**: Compiler enforces valid transitions
2. **State-Specific Data**: Each state carries only relevant data
3. **Exhaustive Matching**: Cannot forget to handle a state
4. **No Stale Data**: Old state consumed, cannot access old data

### Examples in Code

**File:** `src/state_machine.rs`
- `WorkflowState` - Process workflow lifecycle
- State transitions: Initial -> Running -> Paused -> Completed
- Error recovery with state snapshots

## Pattern 3: Type-State Builders

### What It Prevents
- Building objects with missing required fields
- Setting the same field twice
- Skipping validation steps
- Forgetting to call `build()`

### How It Works

**Type States for Progress:**
```rust
pub struct Empty;
pub struct WithCaseID;
pub struct WithActivity;
pub struct Complete;

pub struct EventBuilder<State> {
    case_id: Option<CaseID>,
    activity: Option<ActivityName>,
    timestamp: Option<Timestamp>,
    _state: PhantomData<State>,
}
```

**Progressive Construction:**
```rust
impl EventBuilder<Empty> {
    pub fn new() -> Self { ... }

    pub fn with_case_id(self, case_id: CaseID) -> EventBuilder<WithCaseID> {
        // Transition to next state
    }
}

impl EventBuilder<WithCaseID> {
    pub fn with_activity(self, activity: ActivityName) -> EventBuilder<WithActivity> {
        // Next state
    }
}

impl EventBuilder<Complete> {
    pub fn build(self) -> Event {
        // Only available when complete!
        // All fields guaranteed to be Some
    }
}
```

**Usage:**
```rust
let event = EventBuilder::new()
    .with_case_id(case_id)        // -> WithCaseID
    .with_activity(activity)       // -> WithActivity
    .with_timestamp(timestamp)     // -> Complete
    .build();                      // Only available on Complete

// COMPILE ERRORS:
// EventBuilder::new().build();                    // build() not available
// EventBuilder::new().with_case_id(id).build();   // build() not available
```

### Benefits
1. **No Missing Fields**: Cannot build without all required fields
2. **No Duplicate Fields**: Each field can only be set once
3. **Compile-Time Validation**: Invalid builder usage is a compiler error
4. **Zero Runtime Cost**: PhantomData has zero size

### Examples in Code

**File:** `src/builders.rs`
- `EventBuilder<State>` - Type-state builder for events
- `ConfigBuilder` - Runtime validation for configuration
- Progressive type states: Empty -> WithCaseID -> WithActivity -> Complete

## Pattern 4: Resource Handles

### What It Prevents
- Using resources after closing (use-after-close)
- Modifying resources in wrong state
- Forgetting to close resources
- Accessing results before completion

### How It Works

**Lifecycle Type States:**
```rust
pub struct EventLogOpen;
pub struct EventLogClosed;

pub struct EventLog<State> {
    events: Vec<Event>,
    metadata: EventLogMetadata,
    _state: PhantomData<State>,
}
```

**State-Specific Operations:**
```rust
impl EventLog<EventLogOpen> {
    pub fn add_event(self, event: Event) -> Result<Self, Error> {
        // Only available when open
    }

    pub fn close(self) -> EventLog<EventLogClosed> {
        // Consumes open log, returns closed log
    }
}

impl EventLog<EventLogClosed> {
    pub fn analyze(&self) -> ProcessAnalytics {
        // Only available when closed
    }
}
```

**Usage:**
```rust
let mut log = EventLog::new();          // Open
log = log.add_event(event1).unwrap();   // Still open
log = log.add_event(event2).unwrap();   // Still open

let closed = log.close();               // Transition to closed

// COMPILE ERROR: log was moved
// log.add_event(event3);

let analytics = closed.analyze();       // Only works on closed
```

### Benefits
1. **No Use-After-Close**: Compiler prevents using closed resources
2. **State-Appropriate Operations**: Each state exposes only valid operations
3. **Data Consistency**: Analysis only on complete, immutable data
4. **Resource Safety**: Prevents common resource management bugs

### Examples in Code

**File:** `src/resource_handles.rs`
- `EventLog<State>` - Open/Closed lifecycle for event logs
- `ProcessAnalyzer<State>` - Configured/Running/Completed lifecycle
- State-specific operations prevent misuse

## Pattern 5: Type-Safe Pipelines

### What It Prevents
- Skipping pipeline steps
- Running steps out of order
- Accessing results before completion
- Re-running completed pipelines

### How It Works

**Sequential Type States:**
```rust
pub struct Empty;
pub struct Loaded;
pub struct Discovered;
pub struct Validated;
pub struct Completed;

pub struct ProcessMiningPipeline<State> {
    events: Vec<Event>,
    process_graph: Option<ProcessGraph>,
    validation_report: Option<ValidationReport>,
    _state: PhantomData<State>,
}
```

**Enforced Sequence:**
```rust
impl ProcessMiningPipeline<Empty> {
    pub fn load_events(self, events: Vec<Event>)
        -> Result<ProcessMiningPipeline<Loaded>, Error> { ... }
}

impl ProcessMiningPipeline<Loaded> {
    pub fn discover_process(self) -> ProcessMiningPipeline<Discovered> { ... }
}

impl ProcessMiningPipeline<Discovered> {
    pub fn validate_model(self) -> ProcessMiningPipeline<Validated> { ... }
}

impl ProcessMiningPipeline<Validated> {
    pub fn complete(self) -> ProcessMiningPipeline<Completed> { ... }
}

impl ProcessMiningPipeline<Completed> {
    pub fn results(&self) -> &PipelineResults { ... }  // Only here!
}
```

**Usage:**
```rust
let pipeline = ProcessMiningPipeline::new()
    .load_events(events)?        // Empty -> Loaded
    .discover_process()           // Loaded -> Discovered
    .validate_model()             // Discovered -> Validated
    .complete();                  // Validated -> Completed

let results = pipeline.results(); // Only available on Completed

// COMPILE ERRORS:
// ProcessMiningPipeline::new().discover_process();  // No such method on Empty
// ProcessMiningPipeline::new().results();            // No such method on Empty
```

### Benefits
1. **Enforced Ordering**: Cannot skip or reorder steps
2. **Progress Tracking**: Type indicates current stage
3. **Result Guarantee**: Completed type guarantees all steps done
4. **Compile-Time Validation**: Invalid pipelines won't compile

### Examples in Code

**File:** `src/typed_pipeline.rs`
- `ProcessMiningPipeline<State>` - Type-safe analysis pipeline
- Sequential states: Empty -> Loaded -> Discovered -> Validated -> Completed
- Each step only available at appropriate stage

## Pattern 6: Phantom Types

### What It Prevents
- Runtime overhead for type safety
- Mixing types with same underlying representation
- State tracking overhead

### How It Works

**Zero-Size Type Marker:**
```rust
use std::marker::PhantomData;

pub struct EventBuilder<State> {
    // Actual data
    case_id: Option<CaseID>,
    activity: Option<ActivityName>,

    // Zero-size type marker (compile-time only)
    _state: PhantomData<State>,
}
```

**Size Verification:**
```rust
assert_eq!(
    std::mem::size_of::<EventBuilder<Empty>>(),
    std::mem::size_of::<EventBuilder<Complete>>()
);
// Both have the same size! PhantomData<T> is zero bytes.
```

### Benefits
1. **Zero Runtime Cost**: PhantomData has no size or runtime impact
2. **Compile-Time Type Safety**: Type checking happens at compile time
3. **No Performance Penalty**: Same as using raw types
4. **Type-Level Programming**: Types carry information without data

### Examples in Code
- All type-state builders use `PhantomData<State>`
- All resource handles use `PhantomData<State>`
- All pipeline stages use `PhantomData<State>`

## Pattern 7: NonZero Types

### What It Prevents
- Zero values for IDs and counts
- Off-by-one errors
- Null/zero special cases

### How It Works

**Built-in NonZero Types:**
```rust
use std::num::{NonZeroU32, NonZeroU64};

pub struct CaseID(NonZeroU64);
pub struct EventID(NonZeroU32);
pub struct Count(NonZeroU32);
```

**Construction Guarantees:**
```rust
// Cannot construct NonZeroU64 with zero
let zero = NonZeroU64::new(0);  // Returns None

// Safe construction
let id = NonZeroU64::new(42).expect("non-zero");
```

### Benefits
1. **Guaranteed Non-Zero**: Compiler and runtime guarantee
2. **Niche Optimization**: `Option<NonZeroU64>` same size as `u64`
3. **Semantic Clarity**: Type conveys "never zero" constraint
4. **Division Safety**: Safe divisor in division operations

### Examples in Code

**File:** `src/types.rs`
- `CaseID(NonZeroU64)` - Case IDs cannot be zero
- `EventID(NonZeroU32)` - Event IDs cannot be zero
- `Count(NonZeroU32)` - Counts start at 1, not 0

## Integration Example: Complete Workflow

Here's how all patterns work together:

```rust
use knhk_process_mining::{
    builders::EventBuilder,
    types::{CaseID, ActivityName, Timestamp},
    resource_handles::EventLog,
    typed_pipeline::ProcessMiningPipeline,
};

// 1. Validated Newtypes: Type-safe domain objects
let case_id = CaseID::new(1)?;              // Cannot be zero
let activity = ActivityName::new("Task")?;  // Cannot be empty
let timestamp = Timestamp::now();            // Type-safe time

// 2. Type-State Builder: Compile-time required fields
let event = EventBuilder::new()
    .with_case_id(case_id)      // Required
    .with_activity(activity)     // Required
    .with_timestamp(timestamp)   // Required
    .build();                    // Only available when complete

// 3. Resource Handle: Lifecycle management
let mut log = EventLog::new();              // Open for writing
log = log.add_event(event)?;                // Add events
let closed_log = log.close();               // Close for analysis
let analytics = closed_log.analyze();       // Analyze closed log

// 4. Type-Safe Pipeline: Enforced ordering
let pipeline = ProcessMiningPipeline::new()
    .load_from_event_log(closed_log)?       // Load
    .discover_process()                      // Discover
    .validate_model()                        // Validate
    .complete();                             // Complete

let results = pipeline.results();            // Get results

// 5. State Machine: Workflow lifecycle
let workflow = WorkflowState::new()          // Initial
    .start()?                                // Running
    .increment_cases()?                      // Update
    .complete(duration)?;                    // Completed

// All compile-time checks, zero runtime overhead!
```

## Comparison: Traditional vs Poka-Yoke

### Traditional Approach

```rust
// ❌ Runtime validation, easy to forget
struct Event {
    case_id: u64,        // Could be zero
    activity: String,    // Could be empty
}

fn create_event(case_id: u64, activity: String) -> Result<Event, Error> {
    if case_id == 0 {
        return Err(Error::InvalidCaseId);
    }
    if activity.is_empty() {
        return Err(Error::EmptyActivity);
    }
    Ok(Event { case_id, activity })
}

// Problems:
// - Can still use raw u64 as case_id elsewhere
// - Can forget validation in other constructors
// - Can mix different ID types
// - Runtime overhead for every validation
```

### Poka-Yoke Approach

```rust
// ✅ Compile-time prevention, impossible to forget
pub struct CaseID(NonZeroU64);
pub struct ActivityName(String);

struct Event {
    case_id: CaseID,          // Cannot be zero (NonZeroU64)
    activity: ActivityName,   // Cannot be empty (validated in constructor)
}

// Benefits:
// - Cannot use raw u64 as CaseID (type error)
// - Cannot forget validation (only valid constructors)
// - Cannot mix ID types (CaseID ≠ EventID)
// - Zero runtime overhead (compile-time checks)
```

## Testing Poka-Yoke Patterns

### What to Test

1. **Positive Cases**: Valid constructions work
2. **Negative Cases**: Invalid constructions fail appropriately
3. **Compile Failures**: Document expected compiler errors

### Example Tests

```rust
#[test]
fn test_case_id_cannot_be_zero() {
    assert!(CaseID::new(0).is_err());
    assert!(CaseID::new(1).is_ok());
}

#[test]
fn test_activity_name_cannot_be_empty() {
    assert!(ActivityName::new("").is_err());
    assert!(ActivityName::new("Valid").is_ok());
}

#[test]
fn test_event_builder_requires_all_fields() {
    // This compiles and works:
    let event = EventBuilder::new()
        .with_case_id(case_id)
        .with_activity(activity)
        .with_timestamp(timestamp)
        .build();

    // These won't compile (document in comments):
    // let event = EventBuilder::new().build();           // ERROR
    // let event = EventBuilder::new()
    //     .with_case_id(case_id)
    //     .build();                                       // ERROR
}
```

## Performance Considerations

### Zero-Cost Abstractions

All Poka-Yoke patterns in this module are **zero-cost abstractions**:

1. **Newtypes**: Same size as underlying type
   ```rust
   assert_eq!(size_of::<CaseID>(), size_of::<NonZeroU64>());
   ```

2. **PhantomData**: Zero size
   ```rust
   assert_eq!(size_of::<PhantomData<State>>(), 0);
   ```

3. **Type States**: No runtime overhead
   ```rust
   assert_eq!(
       size_of::<EventBuilder<Empty>>(),
       size_of::<EventBuilder<Complete>>()
   );
   ```

4. **Enum Variants**: Only active variant stored
   ```rust
   // Size of largest variant, not sum of all variants
   assert!(size_of::<WorkflowState>() < size_of::<all_variants_combined>());
   ```

### Optimization

- Type checks happen at compile time (eliminated in release builds)
- No runtime validation for type-guaranteed invariants
- Compiler can optimize away abstraction layers
- Same performance as unsafe raw types, with safety

## Summary: Why Poka-Yoke?

### Traditional Error Handling
- **When**: Runtime validation
- **Where**: Scattered throughout code
- **Cost**: Runtime overhead, easy to forget
- **Safety**: Relies on programmer discipline

### Poka-Yoke Error Prevention
- **When**: Compile time + construction time
- **Where**: Centralized in type definitions
- **Cost**: Zero runtime overhead, impossible to forget
- **Safety**: Enforced by compiler

### The Guarantee

With Poka-Yoke patterns, we achieve:

> **If it compiles, the invariants are guaranteed.**

No runtime checks needed. No defensive programming. No "just in case" validations. The type system prevents errors before they can occur.

## References

- **Rust Type-State Pattern**: https://cliffle.com/blog/rust-typestate/
- **Phantom Types**: https://doc.rust-lang.org/std/marker/struct.PhantomData.html
- **NonZero Types**: https://doc.rust-lang.org/std/num/struct.NonZeroU64.html
- **Zero-Cost Abstractions**: https://blog.rust-lang.org/2015/05/11/traits.html

---

**Remember**: The best error is the one that never happens. Poka-Yoke makes errors **impossible**, not just **unlikely**.
