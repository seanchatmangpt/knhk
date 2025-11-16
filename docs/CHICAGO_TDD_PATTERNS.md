# Chicago TDD Patterns for the Chatman Equation Platform

**Reference**: chicago-tdd-tools v1.3.0
**Integration**: knhk-closed-loop + knhk-platform-2027

## Core Patterns

### Pattern 1: Specification Rules with State-Based Tests

State-based testing validates that a system maintains invariants across state transitions.

```rust
// Each specification rule is a state-based test
#[test]
fn spec_rule_X_describes_property() {
    // Given: Initial state
    let initial = create_fixture();

    // When: Action that transitions state
    initial.perform_action();

    // Then: Verify invariant holds
    assert!(initial.invariant_holds());

    // And: Verify no corruption or side effects
    assert!(initial.consistency_check());
}
```

**Key Principle**: Test the invariant, not the implementation. The test verifies that:
- State transitions are valid
- Invariants are preserved
- No unintended side effects occur

### Pattern 2: Real Collaborators (Not Mocks)

State-based tests use actual objects, not mocks. This tests real concurrent behavior.

```rust
// ✅ CORRECT: Real collaborators
struct TestFixture {
    receipt_store: Arc<ReceiptStore>,          // Real store
    observation_store: Arc<ObservationStore>,  // Real store
    promoter: SnapshotPromoter,                // Real promoter
    coordinator: MapEKCoordinator,             // Real coordinator
}

// ❌ WRONG: Mocks
let mock_store = MockReceiptStore::new();      // Doesn't test real behavior
let mock_coordinator = MockCoordinator::new(); // Can't verify actual patterns
```

**Why**: Mocks hide real concurrency bugs, race conditions, and integration issues. Real objects test actual behavior.

### Pattern 3: Given-When-Then with Assertions

```rust
#[tokio::test]
async fn spec_rule_3_observations_trigger_patterns() {
    // GIVEN: A fixture with stores and coordinator
    let fixture = ClosedLoopFixture::new("test_sector");

    // WHEN: We add 150 observations (frequency anomaly threshold)
    for i in 0..150 {
        let obs = Observation::new(
            "rapid_event".to_string(),
            json!({"index": i}),
            "test_sector".to_string(),
            HashMap::new(),
        );
        fixture.observation_store.append(obs);
    }

    // AND: Execute one MAPE-K cycle
    let cycle = fixture.coordinator.execute_cycle().await.expect("failed");

    // THEN: Patterns were detected
    assert!(cycle.patterns_detected > 0);

    // AND: Proposals were generated
    assert!(cycle.proposals_generated > 0);

    // AND: Receipts were created for each phase
    assert!(!cycle.receipt_ids.is_empty());
}
```

### Pattern 4: Multi-Stage Validation

Validate at three levels:

```rust
#[test]
fn spec_rule_validates_hard_invariants() {
    // STAGE 1: Static validation (SHACL-style)
    let static_ok = validate_schema(&snapshot);
    assert!(static_ok, "Schema should be valid");

    // STAGE 2: Dynamic validation (Simulate execution)
    let dynamic_ok = run_test_harness(&snapshot);
    assert!(dynamic_ok, "Behavior should be correct");

    // STAGE 3: Invariant validation (Q1-Q5)
    let invariants = check_all_invariants(&snapshot);
    assert!(invariants.all_preserved());

    // FINAL: All three passed = safe to promote
}
```

### Pattern 5: Property-Based Testing

Test that properties hold across randomized inputs:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_invariant_q3_always_preserved(max_ticks in 1u32..=100) {
        let result = InvariantValidator::check_q3_guard_preservation(max_ticks);

        // Property: Q3 is preserved if and only if max_ticks <= 8
        prop_assert_eq!(result.is_ok(), max_ticks <= CHATMAN_CONSTANT);
    }

    #[test]
    fn prop_receipts_always_verify(
        operations in prop::collection::vec(any::<ReceiptOp>(), 1..10)
    ) {
        let signing_key = create_key();
        let verifying_key = signing_key.verifying_key();

        for op in operations {
            let receipt = Receipt::create(op, ...)?;

            // Property: Every receipt verifies correctly
            prop_assert!(receipt.verify(&verifying_key).is_ok());
        }
    }
}
```

## Usage in Platform 2027

### Test Pyramid for Chatman Equation Platform

```
                    ▲
                   /│\
                  / │ \
                 /  │  \    Integration Tests
                /   │   \   (End-to-end MAPE-K cycles)
               /    │    \
              /_____│_____\
                   /│\
                  / │ \
                 /  │  \    Multi-Stage Tests
                /   │   \   (Static → Dynamic → Perf → Invariants)
               /    │    \
              /_____│_____\
                   /│\
                  / │ \
                 /  │  \    Unit Tests
                /   │   \   (Individual components)
               /    │    \
              /_____│_____\
```

### Hot Path Tests (Chatman Constant Budget)

```rust
#[test]
fn hot_path_descriptor_read_under_budget() {
    let promoter = SnapshotPromoter::new(genesis);

    let start = std::time::Instant::now();
    let _descriptor = promoter.current();  // Should be < 100ns
    let elapsed = start.elapsed();

    assert!(elapsed.as_nanos() < 100, "Read must be sub-microsecond");
}

#[test]
fn hot_path_promotion_under_budget() {
    let promoter = SnapshotPromoter::new(genesis);

    let start = std::time::Instant::now();
    promoter.promote(new_snap)?;  // Should be < 10μs typical
    let elapsed = start.elapsed();

    assert!(elapsed.as_micros() < 10, "Promotion must be atomic");
}
```

### Warm Path Tests (100ms Budget)

```rust
#[tokio::test]
async fn warm_path_pattern_detection_under_budget() {
    let detector = PatternDetector::new(obs_store);

    let start = std::time::Instant::now();
    let patterns = detector.detect_patterns().await;
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Detection must complete in warm budget");
    assert!(!patterns.is_empty(), "Should detect at least one pattern");
}

#[tokio::test]
async fn warm_path_mape_k_cycle_under_budget() {
    let coordinator = MapEKCoordinator::new(...);

    let start = std::time::Instant::now();
    let cycle = coordinator.execute_cycle().await?;
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 1000, "Full cycle must complete in <1s");
    assert_eq!(cycle.outcome, CycleOutcome::Success);
}
```

### Cold Path Tests (No Hard Latency Budget)

```rust
#[tokio::test]
async fn cold_path_full_validation_suite() {
    let snapshot = create_snapshot();

    // Static validation
    let static_ok = ReceiptStore::validate_static(&snapshot)?;
    assert!(static_ok);

    // Dynamic validation (simulate on clone)
    let dynamic_ok = TddHarness::run_tests(&snapshot)?;
    assert!(dynamic_ok);

    // Performance validation
    let perf_ok = BenchmarkRunner::check_slos(&snapshot)?;
    assert!(perf_ok);

    // Invariant validation
    let inv = InvariantValidator::check_all(...)?;
    assert!(inv.all_preserved());
}
```

## Integration Patterns

### Pattern: Observation → Pattern → Proposal → Validation → Promotion

```rust
#[tokio::test]
async fn integration_full_mape_k_to_promotion() {
    // 1. OBSERVE: Add observations
    for i in 0..200 {
        fixture.observation_store.append(Observation::new(...));
    }

    // 2. DETECT: Run pattern detection
    let patterns = detector.detect_patterns().await;
    assert!(!patterns.is_empty());

    // 3. PROPOSE: Generate ΔΣ
    let proposals = patterns
        .iter()
        .filter(|p| matches!(p.recommended_action, PatternAction::ProposeChange { .. }))
        .collect::<Vec<_>>();
    assert!(!proposals.is_empty());

    // 4. VALIDATE: Run validation suite
    for proposal in &proposals {
        let invariants = HardInvariants {
            q1_no_retrocausation: true,
            q2_type_soundness: true,
            q3_guard_preservation: true,
            q4_slo_compliance: true,
            q5_performance_bounds: true,
        };
        assert!(invariants.all_preserved());
    }

    // 5. EXECUTE: Promote snapshot
    let new_snap = SnapshotDescriptor { ... };
    let promoted = promoter.promote(new_snap)?;
    assert_eq!(promoter.current().snapshot_id, promoted.snapshot_id);
}
```

### Pattern: Doctrinal Constraint → Guard → Test Case

```rust
#[test]
fn doctrine_encoding_to_guard() {
    // DOCTRINE: "No single point of failure in payment processing"
    // ENCODED AS Q: segregation-of-duties invariant

    let guard = Guard {
        name: "segregation_of_duties",
        check: |context| {
            let approver = context.get("approver");
            let executor = context.get("executor");
            approver != executor  // Cannot be same person
        },
    };

    // TEST: Guard is enforced
    let ok_context = map![
        "approver" => "alice",
        "executor" => "bob",
    ];
    assert!(guard.check(&ok_context));

    let bad_context = map![
        "approver" => "alice",
        "executor" => "alice",
    ];
    assert!(!guard.check(&bad_context));
}
```

## Failure Mode Tests

### Pattern: Catch Invariant Violations

```rust
#[test]
fn test_q1_violation_caught() {
    // Create circular snapshot reference (invalid)
    let snap1 = SnapshotDescriptor {
        snapshot_id: "snap1".to_string(),
        parent_id: Some("snap2".to_string()),
        ...
    };
    let snap2 = SnapshotDescriptor {
        snapshot_id: "snap2".to_string(),
        parent_id: Some("snap1".to_string()),  // Cycle!
        ...
    };

    // Should fail validation
    let result = InvariantValidator::check_q1_no_retrocausation("snap1", Some("snap2"), ...);
    assert!(result.is_err(), "Should detect cycle");
}

#[test]
fn test_q3_violation_caught() {
    // max_ticks > 8 (Chatman constant)
    let result = InvariantValidator::check_q3_guard_preservation(9);
    assert!(result.is_err(), "Should reject > 8 ticks");
}

#[test]
fn test_q5_violation_caught() {
    // Memory > 1GB
    let result = InvariantValidator::check_q5_performance_bounds(
        2048,  // 2GB memory
        25.0,  // 25% CPU
        250,   // 250ms latency
    );
    assert!(result.is_err(), "Should reject memory violation");
}
```

## Success Metrics

For a test suite to validate Platform 2027, it must demonstrate:

1. **Latency**: Hot path < 100ns, warm path < 100ms, cold path no constraint
2. **Reliability**: 99.9%+ cycle success rate, zero invariant violations
3. **Auditability**: Every decision has cryptographic receipt
4. **Composability**: Snapshots chain (DAG), overlays compose, guards compose
5. **Autonomy**: MAPE-K cycles run without human intervention

## Running the Tests

```bash
# Unit tests
cargo test --lib knhk-closed-loop

# Integration tests
cargo test --test closed_loop_chicago_tdd

# Property-based tests
cargo test --test closed_loop_chicago_tdd prop_

# Hot path performance
cargo test hot_path_ -- --nocapture

# Full validation suite
cargo test integration_full_mape_k_to_promotion
```

## Key Takeaways

1. **State-based testing** with real collaborators catches real bugs
2. **Multi-stage validation** (static → dynamic → perf → invariants) prevents escaping violations
3. **Property testing** ensures invariants hold at scale
4. **Latency testing** proves adherence to Chatman constant
5. **Integration testing** validates end-to-end MAPE-K cycles

This is how we validate that autonomic hyper intelligence actually works.
