# Policy Lattice Quick Start Guide

**5-Minute Introduction to Doctrine-Bound Policy Lattice**

---

## What is it?

The Policy Lattice Kernel ensures all autonomic adaptations in KNHK are **mathematically lawful** and satisfy μ-kernel constraints (τ ≤ 8 ticks).

**Key Concept**: Actions carry **policy elements** that are validated against a **doctrine** before execution.

---

## Quick Examples

### Example 1: Create a Simple Policy

```rust
use knhk_workflow_engine::autonomic::{
    LatencyBound, PolicyElement, Strictness
};

// Create a latency constraint: "P99 must be ≤ 100ms"
let latency = LatencyBound::new(100.0, Strictness::Hard)?;
let policy = PolicyElement::Latency(latency);
```

### Example 2: Create an Action with Policy

```rust
use knhk_workflow_engine::autonomic::{
    Action, ActionType, LatencyBound, PolicyElement, Strictness
};

// Create action with latency constraint
let policy = PolicyElement::Latency(
    LatencyBound::new(80.0, Strictness::Soft)?
);

let action = Action::with_policy(
    ActionType::ScaleInstances { delta: 2 },
    policy
);
```

### Example 3: Execute with Automatic Validation

```rust
use knhk_workflow_engine::autonomic::{Executor, AdaptationPlan};

let executor = Executor::new();
let mut plan = AdaptationPlan::new();
plan.actions.push(action);

// Automatic policy validation happens here
let results = executor.execute(&plan).await?;

assert!(results[0].policy_validated); // Policy was checked
assert!(results[0].success);          // Action succeeded
```

### Example 4: Multiple Constraints

```rust
use knhk_workflow_engine::autonomic::{
    LatencyBound, FailureRateBound, CapacityEnvelope,
    PolicyElement, Strictness, Lattice
};

// Create three constraints
let latency = PolicyElement::Latency(
    LatencyBound::new(100.0, Strictness::Hard)?
);
let failure = PolicyElement::FailureRate(
    FailureRateBound::new(0.01)?
);
let capacity = PolicyElement::Capacity(
    CapacityEnvelope::new(100, 32)?
);

// Combine them (meet operation = all must be satisfied)
let policy = latency.meet(&failure).meet(&capacity);

let action = Action::with_policy(
    ActionType::OptimizePattern { pattern_id: 12 },
    policy
);
```

### Example 5: System-Wide Policy

```rust
use knhk_workflow_engine::autonomic::KnowledgeBase;
use std::sync::Arc;

let kb = Arc::new(KnowledgeBase::new());

// Set system-wide latency constraint
let system_policy = PolicyElement::Latency(
    LatencyBound::new(50.0, Strictness::Hard)?
);
kb.strengthen_policy(system_policy).await?;

// All actions are now validated against this constraint
let action_policy = PolicyElement::Latency(
    LatencyBound::new(80.0, Strictness::Soft)?
);

let is_valid = kb.validate_action_policy(&action_policy).await?;
// is_valid = false (80ms exceeds 50ms system constraint)
```

---

## Policy Types

| Type | Purpose | Example |
|------|---------|---------|
| **LatencyBound** | P99 latency limits | `LatencyBound::new(100.0, Strictness::Hard)` |
| **FailureRateBound** | Error rate limits | `FailureRateBound::new(0.01)` (1% max) |
| **GuardStrictness** | Guard evaluation mode | `GuardStrictness::new(GuardStrictnessLevel::Tighten)` |
| **CapacityEnvelope** | Concurrency limits | `CapacityEnvelope::new(100, 32)` |
| **Conjunction** | Multiple constraints | `policy1.meet(&policy2)` |
| **Bottom** | No actions allowed | `PolicyElement::Bottom` |

---

## Lattice Operations

| Operation | Symbol | Meaning | Example |
|-----------|--------|---------|---------|
| **Meet** | ⊓ | Stricter (intersection) | `policy1.meet(&policy2)` |
| **Join** | ⊔ | More relaxed (union) | `policy1.join(&policy2)` |
| **Bottom** | ⊥ | Strictest (rejects all) | `PolicyElement::Bottom` |

**Example**:
```rust
let strict = LatencyBound::new(50.0, Strictness::Hard)?;
let relaxed = LatencyBound::new(100.0, Strictness::Soft)?;

let meet = strict.meet(&relaxed);
// meet = LatencyBound { target_p99_ms: 50.0, strictness: Hard }
// (stricter of the two)

let join = strict.join(&relaxed);
// join = LatencyBound { target_p99_ms: 100.0, strictness: Soft }
// (more relaxed of the two)
```

---

## Doctrine (Q)

The **doctrine** defines what actions are lawful. Default doctrine enforces:

- **τ ≤ 8 ticks** (Chatman Constant)
- **max_run_len ≤ 8** (consecutive operations)
- **max_call_depth ≤ 8** (stack depth)
- **max_latency ≤ 100ms** (P99)
- **max_error_rate ≤ 1%**

**Custom Doctrine**:
```rust
use knhk_workflow_engine::autonomic::Doctrine;

// Relaxed for development
let dev_doctrine = Doctrine::relaxed();

// Strict for production
let prod_doctrine = Doctrine::strict();

// Custom
let custom_doctrine = Doctrine {
    max_exec_ticks: 16,
    max_hot_path_latency_ms: 50.0,
    ..Doctrine::new()
};

let executor = Executor::with_doctrine(custom_doctrine);
```

---

## Common Patterns

### Pattern 1: Simple Action with Policy

```rust
// 1. Create policy
let policy = PolicyElement::Latency(
    LatencyBound::new(100.0, Strictness::Soft)?
);

// 2. Create action
let action = Action::with_policy(
    ActionType::ScaleInstances { delta: 2 },
    policy
);

// 3. Execute
let executor = Executor::new();
let result = executor.execute_action(&action).await?;
```

### Pattern 2: System-Wide Constraint + Action

```rust
let kb = Arc::new(KnowledgeBase::new());

// 1. Set system constraint
let system_policy = PolicyElement::Latency(
    LatencyBound::new(50.0, Strictness::Hard)?
);
kb.strengthen_policy(system_policy).await?;

// 2. Create action (validated against system policy)
let action_policy = PolicyElement::Latency(
    LatencyBound::new(40.0, Strictness::Soft)?
);

// 3. Validate
let is_valid = kb.validate_action_policy(&action_policy).await?;
assert!(is_valid); // 40ms < 50ms system constraint
```

### Pattern 3: Multi-Constraint Action

```rust
// 1. Create constraints
let latency = PolicyElement::Latency(
    LatencyBound::new(100.0, Strictness::Hard)?
);
let failure = PolicyElement::FailureRate(
    FailureRateBound::new(0.01)?
);

// 2. Combine
let policy = latency.meet(&failure);

// 3. Create action
let action = Action::with_policy(
    ActionType::OptimizePattern { pattern_id: 5 },
    policy
);

// 4. Execute (both constraints validated)
let executor = Executor::new();
let result = executor.execute_action(&action).await?;
```

---

## Error Handling

All operations return `Result<T, WorkflowError>`:

```rust
use knhk_workflow_engine::error::WorkflowError;

// Invalid policy creation
match LatencyBound::new(-10.0, Strictness::Soft) {
    Ok(bound) => { /* ... */ }
    Err(WorkflowError::Validation(msg)) => {
        eprintln!("Invalid policy: {}", msg);
    }
    Err(e) => { /* ... */ }
}

// Policy validation failure
match executor.execute(&plan).await? {
    results if !results[0].policy_validated => {
        eprintln!("Policy violation: {}", results[0].error.unwrap());
    }
    results => { /* ... */ }
}
```

---

## Checking Results

```rust
let result = executor.execute_action(&action).await?;

// Check if policy was validated
if result.policy_validated {
    println!("✓ Policy validated");
} else {
    println!("✗ Policy validation failed: {}", result.error.unwrap());
}

// Check execution success
if result.success {
    println!("✓ Action executed successfully");
    println!("  Impact: {:?}", result.actual_impact);
    println!("  Duration: {}ms", result.duration_ms);
} else {
    println!("✗ Action failed: {}", result.error.unwrap());
}

// Check μ-kernel compliance
if let Some(metrics) = &result.metrics {
    println!("Execution metrics:");
    println!("  Ticks: {}", metrics.exec_ticks);
    println!("  Run length: {}", metrics.run_len);
    println!("  Call depth: {}", metrics.call_depth);
}
```

---

## Testing

```rust
#[tokio::test]
async fn test_my_policy() {
    let executor = Executor::new();

    let policy = PolicyElement::Latency(
        LatencyBound::new(100.0, Strictness::Hard).unwrap()
    );

    let action = Action::with_policy(
        ActionType::ScaleInstances { delta: 1 },
        policy
    );

    let mut plan = AdaptationPlan::new();
    plan.actions.push(action);

    let results = executor.execute(&plan).await.unwrap();
    assert!(results[0].policy_validated);
    assert!(results[0].success);
}
```

---

## Complete Example

```rust
use knhk_workflow_engine::autonomic::{
    Action, ActionType, AdaptationPlan, Executor,
    KnowledgeBase, LatencyBound, FailureRateBound,
    PolicyElement, Strictness, Lattice
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create knowledge base with system constraints
    let kb = Arc::new(KnowledgeBase::new());
    let system_latency = PolicyElement::Latency(
        LatencyBound::new(50.0, Strictness::Hard)?
    );
    kb.strengthen_policy(system_latency).await?;

    // 2. Create executor
    let executor = Executor::new();

    // 3. Create action with multiple constraints
    let action_latency = PolicyElement::Latency(
        LatencyBound::new(40.0, Strictness::Soft)?
    );
    let action_failure = PolicyElement::FailureRate(
        FailureRateBound::new(0.005)?
    );
    let policy = action_latency.meet(&action_failure);

    let action = Action::with_policy(
        ActionType::ScaleInstances { delta: 2 },
        policy
    );

    // 4. Validate against knowledge base
    let is_valid = kb.validate_action_policy(
        action.get_policy().unwrap()
    ).await?;
    println!("Policy valid: {}", is_valid);

    // 5. Execute
    let mut plan = AdaptationPlan::new();
    plan.actions.push(action);

    let results = executor.execute(&plan).await?;

    // 6. Check results
    for result in &results {
        println!("Action: {:?}", result.action_id);
        println!("  Policy validated: {}", result.policy_validated);
        println!("  Success: {}", result.success);
        println!("  Duration: {}ms", result.duration_ms);
        if let Some(metrics) = &result.metrics {
            println!("  Ticks: {}", metrics.exec_ticks);
        }
    }

    Ok(())
}
```

---

## Key Takeaways

1. **Actions carry policies** (`Action::with_policy()`)
2. **Policies are validated automatically** before execution
3. **Lattice operations combine constraints** (`meet`, `join`)
4. **Doctrine enforces μ-kernel limits** (τ ≤ 8 ticks)
5. **Knowledge base stores system-wide policies**
6. **All operations return Result** (no unwrap/expect)

---

## Need Help?

- **Architecture Docs**: `/docs/architecture/doctrine-policy-lattice.md`
- **Implementation Summary**: `/docs/architecture/IMPLEMENTATION_SUMMARY.md`
- **API Docs**: Run `cargo doc --open`
- **Tests**: `/rust/knhk-workflow-engine/tests/autonomic_policy_lattice_integration.rs`

---

**Quick Reference Version**: 1.0.0
**Last Updated**: 2025-11-16
