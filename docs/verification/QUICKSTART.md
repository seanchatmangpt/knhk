# Formal Verification Quickstart Guide

## Installation

No additional dependencies required! The verification system is included in `knhk-workflow-engine`.

```toml
[dependencies]
knhk-workflow-engine = { path = "../knhk-workflow-engine" }
```

## Quick Examples

### 1. Verify a Policy (30 seconds)

```rust
use knhk_workflow_engine::{
    autonomic::policy_lattice::{LatencyBound, PolicyElement, Strictness},
    verification::PolicyVerifier,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create verifier
    let verifier = PolicyVerifier::new()?;

    // Create policy
    let policy = PolicyElement::Latency(
        LatencyBound::new(100.0, Strictness::Hard)?
    );

    // Verify
    let proof = verifier.verify_policy(&policy)?;

    if proof.is_valid() {
        println!("✅ Policy is valid!");
    } else {
        println!("❌ Policy invalid: {:?}", proof.counterexample());
    }

    Ok(())
}
```

### 2. Check Runtime Invariants (30 seconds)

```rust
use knhk_workflow_engine::verification::{
    InvariantChecker, InvariantContext, PolicyConsistencyInvariant,
};

let mut checker = InvariantChecker::new();
checker.register(PolicyConsistencyInvariant);

let context = InvariantContext::new()
    .with_policy(policy);

checker.check_all(&context)?;
println!("✅ All invariants satisfied!");
```

### 3. Type-Level Verification (30 seconds)

```rust
use knhk_workflow_engine::verification::type_level::{
    VerifiedMetrics, MuKernelTickBound,
};

// Valid metrics
let metrics = VerifiedMetrics::<MuKernelTickBound>::new(5)?;
println!("✅ Metrics valid: {} ticks", metrics.ticks());

// Invalid metrics (will error)
let invalid = VerifiedMetrics::<MuKernelTickBound>::new(10);
assert!(invalid.is_err());
```

## Running the Example

```bash
cd /home/user/knhk/rust/knhk-workflow-engine
cargo run --example formal_verification
```

Expected output:
```
🔬 KNHK Formal Verification Example
====================================

📋 Example 1: Policy Verification
----------------------------------
Policy: Latency(100ms, Hard)
✅ Policy is valid (SAT)
   Verification time: 2ms

🔄 Example 2: Overlay Verification
-----------------------------------
Overlay scope: 1 patterns
Changes: Scale multi-instance by 2
Proof obligations generated: 5
  1. Verify workflow invariants remain valid after overlay
  2. Verify hot path performance constraint (τ ≤ 8)
  3. Verify guard constraints remain valid
  4. Verify SLO compliance after overlay
  5. Verify overlay conforms to system doctrine (Q)
✅ Overlay is safe to apply

⚖️  Example 3: Doctrine Projection (Q ∧ policy → policy')
--------------------------------------------------------
Doctrine μ-kernel constraints:
  - max_exec_ticks: 8 (Chatman Constant)
  - max_run_len: 8
  - max_hot_path_latency_ms: 100ms

✅ Policy satisfies doctrine
   Original: Latency(50ms, Soft)
   Projected: Latency(50ms, Soft)

⚠️  Excessive policy clamped to doctrine bounds
   Original: Latency(200ms, Hard)
   Projected: Latency(100ms, Hard)

🔒 Example 4: Runtime Invariant Checking
----------------------------------------
Registered invariants: 1
✅ All invariants satisfied
   Violations: 0

💾 Example 5: Proof Caching
---------------------------
Cache configuration:
  - Max proofs: 100
  - TTL: 60 seconds

✅ Proof cached
   Proof ID: proof:a1b2c3d4...

✅ Proof retrieved from cache
   Cache hit! Verification time: < 1ms
   Proof valid: true

Cache statistics:
  - Total proofs: 1
  - TTL: 60000ms

✅ All verification examples completed successfully!
```

## Running Tests

### Unit Tests

```bash
# Test individual verification modules
cargo test --lib verification::smt_solver
cargo test --lib verification::invariants
cargo test --lib verification::proof_certificates
cargo test --lib verification::type_level
```

### Integration Tests

```bash
# Test complete verification workflows
cargo test --test verification
```

Expected output:
```
running 8 tests
test test_complete_verification_workflow ... ok
test test_overlay_verification_with_proofs ... ok
test test_proof_caching ... ok
test test_invariant_checking_workflow ... ok
test test_doctrine_projection_verification ... ok
test test_type_level_verification ... ok
test test_verified_policy_state_transitions ... ok
test test_performance_constraint_verification ... ok

test result: ok. 8 passed; 0 failed
```

## Common Use Cases

### Use Case 1: Verify Overlay Before Application

```rust
use knhk_workflow_engine::{
    DeltaSigma, OverlayChange, OverlayScope,
    verification::PolicyVerifier,
    patterns::PatternId,
};

// Create overlay
let scope = OverlayScope::new()
    .with_pattern(PatternId::new(12)?);
let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

let unproven = DeltaSigma::new(scope, changes);
let proof_pending = unproven.generate_proof_obligations()?;

// Verify
let verifier = PolicyVerifier::new()?;
let proof = verifier.verify_overlay(&proof_pending).await?;

if proof.is_valid() {
    // Safe to transition to Proven state
    let proven = proof_pending.into_proven()?;
    executor.apply_overlay(proven).await?;
} else {
    eprintln!("Overlay rejected: {:?}", proof.counterexample());
}
```

### Use Case 2: Check μ-Kernel Constraints

```rust
use knhk_workflow_engine::{
    autonomic::doctrine::ExecutionMetrics,
    verification::PolicyVerifier,
};

let verifier = PolicyVerifier::new()?;

let mut metrics = ExecutionMetrics::new();
metrics.exec_ticks = 5; // Within bounds

let proof = verifier.verify_metrics(&metrics)?;

if proof.is_valid() {
    println!("✅ Metrics satisfy μ-kernel constraints");
} else {
    println!("❌ Metrics violate constraints: {:?}", proof.counterexample());
}
```

### Use Case 3: Register Custom Invariants

```rust
use knhk_workflow_engine::verification::{
    RuntimeInvariant, InvariantSeverity, InvariantChecker,
};

let mut checker = InvariantChecker::new();

// Custom business invariant
let custom = RuntimeInvariant::new(
    "business_rule_42".to_string(),
    InvariantSeverity::Critical,
    |ctx| {
        // Your custom logic here
        Ok(ctx.session_id.is_some() && ctx.mode != AutonomicMode::Frozen)
    }
);

checker.register(custom);
```

### Use Case 4: Cache Proofs for Performance

```rust
use knhk_workflow_engine::verification::ProofCache;

let cache = ProofCache::new(1000, 300_000); // 1000 proofs, 5min TTL

// First verification (100ms)
let proof1 = verifier.verify_policy(&policy)?;
cache.put(proof1.clone()).await?;

// Subsequent verifications (< 1ms)
if let Some(cached) = cache.get(&subject).await {
    println!("Cache hit! Instant verification");
}
```

## Troubleshooting

### Problem: "Verification timeout"

**Solution**: Increase SMT solver timeout:

```rust
use knhk_workflow_engine::verification::VerificationConfig;

let mut config = VerificationConfig::default();
config.smt_timeout_ms = 500; // Increase from 100ms

let verifier = PolicyVerifier::with_config(config)?;
```

### Problem: "Invariant violation"

**Solution**: Check violation details:

```rust
match checker.check_all(&context) {
    Err(e) => {
        eprintln!("Invariant violation: {}", e);

        for violation in checker.get_violations(10) {
            eprintln!("  {}: {}", violation.invariant, violation.message);
        }
    }
    Ok(()) => { /* Success */ }
}
```

### Problem: "Proof not found in cache"

**Solution**: Proof may have expired. Check TTL:

```rust
let stats = cache.stats().await;
println!("Cache TTL: {}ms", stats.default_ttl_ms);

// Increase TTL if needed
let cache = ProofCache::new(1000, 3600_000); // 1 hour
```

## Performance Tips

1. **Enable Proof Caching**
   ```rust
   let mut config = VerificationConfig::default();
   config.enable_cache = true;
   ```

2. **Batch Verification**
   ```rust
   // Verify multiple policies in parallel
   let futures: Vec<_> = policies.iter()
       .map(|p| verifier.verify_policy(p))
       .collect();

   let proofs = futures::future::join_all(futures).await;
   ```

3. **Use Type-Level Verification**
   ```rust
   // Compile-time checking (zero runtime cost)
   const TICKS: Option<Bounded<MuKernelTickBound, 5>> =
       Bounded::<MuKernelTickBound, 5>::new();
   ```

## Next Steps

1. **Read Full Documentation**: `/docs/verification/FORMAL_VERIFICATION.md`
2. **Study Implementation**: `/docs/verification/IMPLEMENTATION_SUMMARY.md`
3. **Explore Examples**: `cargo run --example formal_verification`
4. **Run Tests**: `cargo test --test verification`
5. **Integrate with Your Code**: See integration examples above

## Support

For questions or issues:
1. Check documentation in `/docs/verification/`
2. Review example code in `examples/formal_verification.rs`
3. Run tests to verify setup: `cargo test --test verification`

## Quick Reference

| Task | Command |
|------|---------|
| Run example | `cargo run --example formal_verification` |
| Run all tests | `cargo test --test verification` |
| Check module | `cargo test --lib verification` |
| Build docs | `cargo doc --open` |
| View example code | `cat examples/formal_verification.rs` |

## Summary

You now have a working formal verification system that can:
- ✅ Verify policies using SMT solvers
- ✅ Check runtime invariants
- ✅ Store and cache proofs
- ✅ Provide type-level guarantees
- ✅ Integrate with KNHK governance layer

Start with the example (`cargo run --example formal_verification`) and gradually integrate verification into your autonomic workflows!
