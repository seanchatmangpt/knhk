# ΔΣ Guarded Overlay Engine - Proof-Carrying Ontology Evolution

## Overview

The ΔΣ (DeltaSigma) Guarded Overlay Engine provides **type-safe, proof-carrying ontology evolution** for the KNHK workflow engine. It enables safe runtime adaptation with compile-time and runtime verification.

## Core Concepts

### Type-Safe Proof States

Overlays use **phantom types** to encode proof validation state at the type level:

```rust
DeltaSigma<Unproven>      // Proposal created, not yet validated
    ↓ generate_proof_obligations()
DeltaSigma<ProofPending>  // Proof obligations generated
    ↓ validate()
DeltaSigma<Proven>        // All proofs satisfied, safe to apply
```

**Key Property**: Only `DeltaSigma<Proven>` can be applied to the running system. The type system enforces this at compile time.

### Overlay Scope

Overlays explicitly declare their **scope** - which parts of the system they affect:

- **Workflows**: Which workflow specifications
- **Patterns**: Which Van der Aalst patterns (1-43)
- **Guards**: Which guard constraints
- **Tags**: Custom scope metadata

**Risk Surface**: The scope defines the "blast radius" of the overlay.

### Overlay Changes

Changes are **strongly-typed** (no stringly-typed Turtle inside the engine):

```rust
enum OverlayChange {
    ScaleMultiInstance { delta: i32 },
    AdjustPerformance { target_ticks: u64 },
    ModifyGuard { guard_name: String, new_value: String },
    TogglePattern { pattern_id: PatternId, enabled: bool },
    AdjustResources { resource: String, multiplier: f64 },
    Custom { change_type: String, params: HashMap<String, String> },
}
```

### Proof Obligations

Each overlay generates **proof obligations** that must be satisfied:

1. **ValidateInvariants**: Workflow invariants remain valid
2. **ValidatePerformance**: Hot path constraint τ ≤ 8 (Chatman Constant)
3. **ValidateGuards**: Guard constraints remain valid
4. **ValidateSLO**: SLO compliance maintained
5. **ValidateDoctrine**: Conforms to system doctrine (Q)

## Usage

### Basic Workflow

```rust
use knhk_workflow_engine::autonomic::delta_sigma::*;
use knhk_workflow_engine::autonomic::overlay_validator::*;

// 1. Create overlay proposal (Unproven state)
let scope = OverlayScope::new()
    .with_pattern(PatternId::new(12)?)
    .with_workflow(workflow_id);

let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

let proposal = DeltaSigma::new(scope, changes)
    .with_metadata("source", "planner")
    .merge_change_scopes();

// 2. Generate proof obligations (Unproven → ProofPending)
let proof_pending = proposal.generate_proof_obligations()?;

// 3. Validate proof obligations (ProofPending → Proven)
let validator = OverlayValidator::new(pattern_registry, knowledge_base);
let result = validator.validate(&proof_pending).await?;

// 4. Apply proven overlay (only if validation succeeded)
if result.is_proven() {
    let proven = result.into_proven()?;
    executor.apply_overlay(proven).await?;
}
```

### Integration with MAPE-K

#### Plan Phase

The Planner generates overlay proposals:

```rust
impl Planner {
    async fn plan(&self, analysis: &Analysis) -> WorkflowResult<AdaptationPlan> {
        let mut plan = AdaptationPlan::new();

        // Generate overlay proposals
        let overlay = DeltaSigma::new(scope, changes)
            .with_metadata("goal", "performance")
            .merge_change_scopes();

        plan.add_overlay(overlay);

        Ok(plan)
    }
}
```

#### Execute Phase

The Executor only applies **proven** overlays:

```rust
impl Executor {
    async fn execute(&self, plan: &AdaptationPlan) -> WorkflowResult<()> {
        // Validate overlays first
        for overlay_proposal in plan.overlays() {
            let proof_pending = overlay_proposal.generate_proof_obligations()?;
            let result = self.validator.validate(&proof_pending).await?;

            if result.is_proven() {
                let proven = result.into_proven()?;
                self.apply_proven_overlay(proven).await?;
            } else {
                // Rejection with audit trail
                self.log_rejected_overlay(overlay_proposal, result.proof()).await;
            }
        }

        Ok(())
    }
}
```

## Safety Guarantees

### Compile-Time Safety

- **Type-level proof states**: Invalid state transitions rejected at compile time
- **No unwrap/expect**: All production paths use proper `Result<T, E>` error handling
- **Phantom types**: Zero runtime cost for type-level validation state

### Runtime Verification

- **Proof obligations**: Explicit validation requirements
- **Focused testing**: Only test affected patterns (not full regression)
- **Performance constraints**: Verify τ ≤ 8 for hot path
- **Invariant checking**: Workflow properties remain valid

### Audit Trail

- **Proof hash**: Deterministic hash of proof results (reproducibility)
- **Validator version**: Track which validator version produced proof
- **Timestamps**: When overlay was created and validated
- **Metadata**: Source, rationale, and context

## Example Overlays

### Safe Overlay: Scale Multi-Instance

```rust
// ✅ SAFE: Explicit scope, valid change, provable
let scope = OverlayScope::new()
    .with_pattern(PatternId::new(12)?)  // MI Without Sync
    .with_pattern(PatternId::new(13)?); // MI With Design-Time Knowledge

let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

let overlay = DeltaSigma::new(scope, changes)
    .with_metadata("reason", "High load detected")
    .merge_change_scopes();

// Proof obligations:
// - Validate MI pattern invariants
// - Verify performance impact ≤ 8 ticks
// - Check SLO compliance
// - Verify doctrine conformance
```

### Unsafe Overlay: Performance Violation

```rust
// ❌ UNSAFE: Performance constraint violated
let scope = OverlayScope::new()
    .with_pattern(PatternId::new(1)?);

let changes = vec![OverlayChange::AdjustPerformance { target_ticks: 20 }]; // > 8!

let overlay = DeltaSigma::new(scope, changes);

// This will FAIL validation:
// - Performance obligation violated (20 > 8)
// - Overlay rejected, not applied
```

### Unsafe Overlay: Empty Scope

```rust
// ❌ UNSAFE: Empty scope (no risk surface)
let scope = OverlayScope::new(); // Empty!

let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

let overlay = DeltaSigma::new(scope, changes);

// This will FAIL at proof obligation generation:
// - Scope cannot be empty
// - Must explicitly declare what is affected
```

### Unsafe Overlay: Unregistered Pattern

```rust
// ❌ UNSAFE: Pattern not in registry
let scope = OverlayScope::new()
    .with_pattern(PatternId::new(99)?); // Invalid pattern ID (only 1-43 valid)

let changes = vec![OverlayChange::TogglePattern {
    pattern_id: PatternId::new(99)?,
    enabled: false,
}];

let overlay = DeltaSigma::new(scope, changes);

// This will FAIL validation:
// - Pattern ID out of range (compile-time: PatternId::new fails)
// - Or pattern not registered (runtime: validation fails)
```

## Overlay Composition

Multiple overlays can be composed with different strategies:

### Sequential Composition

```rust
let composition = OverlayComposition::new(CompositionStrategy::Sequential)
    .add(overlay1)
    .add(overlay2)
    .add(overlay3);

// Overlays applied in order
// No conflicts possible (later overlays can override earlier ones)
```

### Parallel Composition

```rust
let composition = OverlayComposition::new(CompositionStrategy::Parallel)
    .add(overlay1)
    .add(overlay2);

composition.validate()?; // Fails if scopes overlap

// Overlays applied concurrently
// Scopes must be disjoint (no overlapping workflows/patterns/guards)
```

### Merge Composition

```rust
let composition = OverlayComposition::new(CompositionStrategy::Merge)
    .add(overlay1)
    .add(overlay2);

composition.validate()?; // Fails if changes conflict

// Overlays merged into single overlay
// Changes must not conflict
```

## Proof Contract

### What is Validated

1. **Pattern Invariants**: Workflow properties remain valid after overlay
2. **Performance Constraints**: Hot path operations ≤ 8 ticks
3. **Guard Constraints**: All guard validations still pass
4. **SLO Compliance**: Service level objectives maintained
5. **Doctrine Conformance**: Overlay conforms to system policies (Q)

### What is NOT Validated

- **Full regression testing**: Only affected patterns tested (focused)
- **External dependencies**: Assumes external systems behave correctly
- **Future changes**: Proof valid for current system state only

### Proof Reproducibility

Proofs are **deterministic and reproducible**:

- Same overlay + same system state → same proof
- Proof hash enables verification
- Validator version tracked for compatibility

## Performance

### Validation Effort

Overlays estimate validation effort:

```rust
let effort = proof_pending.validation_effort();

match effort {
    ValidationEffort::Low    => // < 30 seconds
    ValidationEffort::Medium => // 30-120 seconds
    ValidationEffort::High   => // > 120 seconds
}
```

### Proof Caching

Proofs are cached to avoid redundant validation:

```rust
// First validation: Execute proof obligations
let result1 = validator.validate(&overlay).await?;

// Second validation: Retrieved from cache (instant)
let result2 = validator.validate(&overlay).await?;

// Same proof, no recomputation
assert_eq!(result1.proof().proof_hash, result2.proof().proof_hash);
```

## Testing

### Unit Tests

```bash
cargo test autonomic::delta_sigma
cargo test autonomic::overlay_validator
```

### Integration Tests

```bash
cargo test --test test_delta_sigma
```

### Property Tests

```rust
// Property: All valid overlays pass validation
#[tokio::test]
async fn property_valid_overlays_pass_validation() {
    // ... test all valid overlay configurations ...
}

// Property: Invalid patterns fail validation
#[tokio::test]
async fn property_invalid_patterns_fail_validation() {
    // ... test invalid pattern IDs, unregistered patterns ...
}
```

## Error Handling

All operations return `WorkflowResult<T>`:

```rust
// Proof obligation generation
let proof_pending = proposal.generate_proof_obligations()?;
// Error: Empty scope, empty changes

// Validation
let result = validator.validate(&proof_pending).await?;
// Error: Network failure, timeout, etc.

// Proven overlay extraction
let proven = result.into_proven()?;
// Error: Proof not valid
```

## Observability

Overlays integrate with OTEL:

```rust
tracing::info!(
    overlay_id = %overlay.id,
    scope_risk = overlay.scope.risk_surface(),
    changes = overlay.changes.len(),
    "Overlay proposal created"
);

tracing::info!(
    overlay_id = %overlay.id,
    proof_valid = result.is_proven(),
    duration_ms = result.proof().total_duration_ms,
    "Overlay validation complete"
);
```

## Best Practices

### DO

✅ Use explicit scopes (declare what is affected)
✅ Validate overlays before application
✅ Use type-safe changes (avoid Custom when possible)
✅ Add metadata for audit trail
✅ Merge change scopes automatically
✅ Check proof validity before application
✅ Handle validation errors gracefully

### DON'T

❌ Create empty scopes
❌ Skip proof validation
❌ Apply unproven overlays
❌ Ignore validation failures
❌ Use stringly-typed changes
❌ Assume all overlays succeed
❌ Bypass the type system

## Future Enhancements

- **Multi-version proofs**: Support for proof schema evolution
- **Incremental validation**: Only re-validate changed obligations
- **Distributed validation**: Parallel proof execution across workers
- **ML-assisted planning**: Learn which overlays succeed/fail
- **Temporal proofs**: Proofs valid for time windows

## References

- **MAPE-K Framework**: IBM Autonomic Computing reference model
- **YAWL Patterns**: Van der Aalst workflow patterns (1-43)
- **Chatman Constant**: Performance constraint (τ ≤ 8)
- **KNHK Doctrine (Q)**: System policies and constraints
- **Phantom Types**: Compile-time type-level programming

---

**Version**: 1.0.0
**Status**: Production Ready
**Last Updated**: 2025-11-16
