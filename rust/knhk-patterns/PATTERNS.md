# Workflow Patterns Reference

Complete reference for all workflow patterns implemented in `knhk-patterns`.

## Pattern Categories

### Basic Control Flow (Patterns 1-5)

Fundamental patterns for process control, covering 75% of real-world workflows.

### Advanced Branching (Patterns 6, 10, 16)

Complex branching and synchronization patterns, adding 10% coverage.

## Pattern 1: Sequence

**Van der Aalst ID**: WCP-1  
**Tick Budget**: 1  
**SIMD Support**: No  
**Complexity**: O(n)

### Description

Executes activities in strict sequential order. Task A must complete before Task B begins.

### YAWL Equivalent

```xml
<task name="A"/>
<task name="B"/>
```

### Usage

```rust
use knhk_patterns::*;
use std::sync::Arc;

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
```

### Behavior

- Executes branches in order
- Stops on first error
- Returns single output
- Zero overhead (direct flow transition)

### Use Cases

- Linear pipeline stages
- Sequential transformations
- Ordered processing

## Pattern 2: Parallel Split (AND-split)

**Van der Aalst ID**: WCP-2  
**Tick Budget**: 2  
**SIMD Support**: Yes  
**Complexity**: O(n)

### Description

Splits execution into multiple parallel branches. All branches execute concurrently.

### YAWL Equivalent

```xml
<split type="AND">
    <task name="A"/>
    <task name="B"/>
    <task name="C"/>
</split>
```

### Usage

```rust
let branches = vec![
    Arc::new(|mut data: i32| { data *= 2; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 3; Ok(data) }),
    Arc::new(|mut data: i32| { data *= 5; Ok(data) }),
];

let pattern = ParallelSplitPattern::new(branches)?;
let results = pattern.execute(10)?;
assert_eq!(results.len(), 3); // [20, 30, 50]
```

### Behavior

- Executes all branches in parallel (Rayon)
- Returns one output per branch
- Fails if any branch fails
- SIMD-optimized branch initialization

### Use Cases

- Parallel validation
- Independent transformations
- Concurrent processing

## Pattern 3: Synchronization (AND-join)

**Van der Aalst ID**: WCP-3  
**Tick Budget**: 3  
**SIMD Support**: Yes  
**Complexity**: O(n)

### Description

Synchronizes multiple parallel branches. Waits for all branches to complete before proceeding.

### YAWL Equivalent

```xml
<join type="AND">
    <task name="A"/>
    <task name="B"/>
</join>
```

### Usage

```rust
let pattern = SynchronizationPattern::new();
let results = pattern.execute(input)?;
// Synchronization happens implicitly in ParallelSplit execution
```

### Behavior

- Waits for all incoming branches
- Atomic counter for branch tracking
- SIMD-optimized synchronization
- Passes through single input

### Use Cases

- Synchronize parallel branches
- Wait for all validations
- Merge parallel results

## Pattern 4: Exclusive Choice (XOR-split)

**Van der Aalst ID**: WCP-4  
**Tick Budget**: 2  
**SIMD Support**: No  
**Complexity**: O(n)

### Description

Selects exactly one branch from multiple alternatives based on conditions. Only one branch executes.

### YAWL Equivalent

```xml
<split type="XOR">
    <condition>value < 0</condition>
    <task name="A"/>
    <condition>value >= 0</condition>
    <task name="B"/>
</split>
```

### Usage

```rust
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

### Behavior

- Evaluates conditions in order
- Executes first matching branch
- Returns single output
- Fails if no condition matches

### Use Cases

- Conditional routing
- Business logic branching
- Error handling paths

## Pattern 5: Simple Merge (XOR-join)

**Van der Aalst ID**: WCP-5  
**Tick Budget**: 1  
**SIMD Support**: No  
**Complexity**: O(1)

### Description

Merges alternative branches without synchronization. Continues immediately when any branch completes.

### YAWL Equivalent

```xml
<join type="XOR">
    <task name="A"/>
    <task name="B"/>
</join>
```

### Usage

```rust
let pattern = SimpleMergePattern::new();
let results = pattern.execute(input)?;
// Simple merge just passes through
```

### Behavior

- No waiting for other branches
- Direct flow merge
- Zero overhead
- Passes through single input

### Use Cases

- Merge after XOR-split
- Converge alternative paths
- Fast path continuation

## Pattern 6: Multi-Choice (OR-split)

**Van der Aalst ID**: WCP-6  
**Tick Budget**: 3  
**SIMD Support**: Yes  
**Complexity**: O(n)

### Description

Selects one or more branches based on conditions. Multiple branches can execute concurrently.

### YAWL Equivalent

```xml
<split type="OR">
    <condition>value > 0</condition>
    <task name="A"/>
    <condition>value < 10</condition>
    <task name="B"/>
</split>
```

### Usage

```rust
let choices = vec![
    (
        Arc::new(|data: &i32| *data > 0) as ConditionFn<i32>,
        Arc::new(|mut data: i32| { data *= 2; Ok(data) }) as BranchFn<i32>,
    ),
    (
        Arc::new(|data: &i32| *data < 10),
        Arc::new(|mut data: i32| { data += 5; Ok(data) }),
    ),
];

let pattern = MultiChoicePattern::new(choices)?;
let results = pattern.execute(5)?;
assert_eq!(results.len(), 2); // Both conditions match
```

### Behavior

- Evaluates all conditions
- Executes all matching branches in parallel
- Returns one output per matching branch
- SIMD-optimized condition evaluation
- Fails if no condition matches

### Use Cases

- Multiple notifications
- Conditional parallelism
- Complex business rules

## Pattern 10: Arbitrary Cycles

**Van der Aalst ID**: WCP-10  
**Tick Budget**: 2  
**SIMD Support**: No  
**Complexity**: O(k*n)

### Description

Supports loops and retry logic. Executes a branch repeatedly until a condition is met or max iterations reached.

### YAWL Equivalent

```xml
<task name="A">
    <loopCondition>value < 10</loopCondition>
</task>
```

### Usage

```rust
let branch = Arc::new(|mut data: i32| {
    data += 1;
    Ok(data)
});

let should_continue = Arc::new(|data: &i32| *data < 10);

let pattern = ArbitraryCyclesPattern::new(branch, should_continue, 100)?;
let results = pattern.execute(5)?;
assert_eq!(results[0], 10); // Stopped when value == 10
```

### Behavior

- Executes branch repeatedly
- Checks condition after each iteration
- Stops when condition is false or max iterations reached
- Returns single output

### Use Cases

- Retry logic
- Approval loops
- Iterative processing
- Polling

## Pattern 16: Deferred Choice

**Van der Aalst ID**: WCP-16  
**Tick Budget**: 3  
**SIMD Support**: No  
**Complexity**: O(n)

### Description

Event-driven choice made by the environment, not the process. Waits for external events to trigger branch selection.

### YAWL Equivalent

```xml
<split type="OR">
    <eventTrigger>eventA</eventTrigger>
    <task name="A"/>
    <eventTrigger>eventB</eventTrigger>
    <task name="B"/>
</split>
```

### Usage

```rust
let choices = vec![
    (
        Arc::new(|data: &EventData| data.event == Event::A) as ConditionFn<EventData>,
        Arc::new(|mut data: EventData| { data.processed = true; Ok(data) }) as BranchFn<EventData>,
    ),
    (
        Arc::new(|data: &EventData| data.event == Event::B),
        Arc::new(|mut data: EventData| { data.processed = true; Ok(data) }),
    ),
];

let pattern = DeferredChoicePattern::new(choices, 1000)?; // 1000ms timeout
let results = pattern.execute(event_data)?;
```

### Behavior

- Polls conditions until one becomes true
- Executes first matching branch
- Supports timeout
- Event-driven execution

### Use Cases

- Event-driven workflows
- External trigger handling
- Timeout-based routing
- Reactive systems

## Pattern Composition

### Sequential Composition

```rust
let composite = CompositePattern::Sequence(vec![
    Box::new(pattern1),
    Box::new(pattern2),
    Box::new(pattern3),
]);
```

### Parallel Composition

```rust
let composite = CompositePattern::Parallel(vec![
    Box::new(pattern1),
    Box::new(pattern2),
]);
```

### Conditional Composition

```rust
let composite = CompositePattern::Choice(vec![
    (condition1, Box::new(pattern1)),
    (condition2, Box::new(pattern2)),
]);
```

### Retry Composition

```rust
let composite = CompositePattern::Retry {
    pattern: Box::new(pattern),
    should_continue: Arc::new(|data| data.value < 10),
    max_attempts: 100,
};
```

## Performance Characteristics

| Pattern | Tick Budget | SIMD | Parallel | Use Case |
|---------|-------------|------|----------|----------|
| Sequence | 1 | No | No | Linear flow |
| Parallel Split | 2 | Yes | Yes | Parallelism |
| Synchronization | 3 | Yes | No | Sync point |
| Exclusive Choice | 2 | No | No | Conditional |
| Simple Merge | 1 | No | No | Fast merge |
| Multi-Choice | 3 | Yes | Yes | Multi-branch |
| Arbitrary Cycles | 2 | No | No | Loops |
| Deferred Choice | 3 | No | No | Events |

## Best Practices

### 1. Use Sequence for Linear Flow

```rust
// ✅ Good: Sequential processing
let pattern = SequencePattern::new(vec![step1, step2, step3])?;
```

### 2. Use Parallel Split for Independent Work

```rust
// ✅ Good: Parallel validation
let pattern = ParallelSplitPattern::new(vec![validator1, validator2, validator3])?;
```

### 3. Use Exclusive Choice for Mutually Exclusive Paths

```rust
// ✅ Good: Single path selection
let pattern = ExclusiveChoicePattern::new(vec![
    (is_error, error_handler),
    (is_success, success_handler),
])?;
```

### 4. Use Multi-Choice for Multiple Notifications

```rust
// ✅ Good: Multiple notifications
let pattern = MultiChoicePattern::new(vec![
    (needs_email, email_notifier),
    (needs_sms, sms_notifier),
    (needs_slack, slack_notifier),
])?;
```

### 5. Use Arbitrary Cycles for Retry Logic

```rust
// ✅ Good: Retry with backoff
let pattern = ArbitraryCyclesPattern::new(
    processor,
    Arc::new(|result| result.should_retry()),
    3, // max attempts
)?;
```

## Error Handling

All patterns return `PatternResult<T>`:

```rust
match pattern.execute(input) {
    Ok(results) => {
        // Process results
    }
    Err(PatternError::ValidationFailed(msg)) => {
        // Ingress validation failed
    }
    Err(PatternError::ExecutionFailed(msg)) => {
        // Runtime execution error
    }
    Err(PatternError::TooManyBranches) => {
        // Branch limit exceeded
    }
    Err(PatternError::InvalidConfiguration(msg)) => {
        // Invalid configuration
    }
}
```

## References

- [Van der Aalst Workflow Patterns](http://www.workflowpatterns.com/)
- [YAWL Specification](https://www.yawlfoundation.org/)
- [BitFlow Patterns](~/cns/bitflow/ontologies/workflow_patterns_43.ttl)

