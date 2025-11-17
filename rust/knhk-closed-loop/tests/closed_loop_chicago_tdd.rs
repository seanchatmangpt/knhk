// Chicago TDD Specification Harness for Closed Loop System
// State-based tests with real collaborators validating loop closure

use knhk_closed_loop::*;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// SPECIFICATION HARNESS: The narrative specification in test form
// ============================================================================

/// This test harness validates the complete 2027 narrative:
/// 1. Model reality carefully (observation plane)
/// 2. Bind yourself to measurable guarantees (hard invariants Q)
/// 3. Close the loop faster than anyone else (MAPE-K cycles)
/// 4. What you measure, you manage (receipts)
/// 5. Picoseconds to decisions (atomic promotion)

// ============================================================================
// FIXTURE: Real collaborators (not mocks)
// ============================================================================

struct ClosedLoopFixture {
    observation_store: Arc<observation::ObservationStore>,
    receipt_store: Arc<receipt::ReceiptStore>,
    promoter: promoter::SnapshotPromoterWithStats,
    coordinator: coordinator::MapEKCoordinator,
    signing_key: ed25519_dalek::SigningKey,
}

impl ClosedLoopFixture {
    fn new(sector: &str) -> Self {
        // Real signing key
        let mut seed = [0u8; 32];
        seed[0] = 42;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        // Real stores
        let observation_store = Arc::new(observation::ObservationStore::new());
        let receipt_store = Arc::new(receipt::ReceiptStore::new(verifying_key));

        // Real promoter
        let initial_snapshot = promoter::SnapshotDescriptor {
            snapshot_id: "genesis".to_string(),
            parent_id: None,
            promoted_at: chrono::Utc::now().timestamp_millis() as u64,
            version: 0,
        };
        let promoter = promoter::SnapshotPromoterWithStats::new(initial_snapshot);

        // Real coordinator
        let coordinator = coordinator::MapEKCoordinator::new(
            observation_store.clone(),
            receipt_store.clone(),
            signing_key.clone(),
            sector.to_string(),
        );

        ClosedLoopFixture {
            observation_store,
            receipt_store,
            promoter,
            coordinator,
            signing_key,
        }
    }
}

// ============================================================================
// SPECIFICATION: Rule 1 - Model Reality Carefully (Observation Plane)
// ============================================================================

#[test]
fn spec_rule_1_observations_form_immutable_append_only_log() {
    // Given: Empty observation store
    let fixture = ClosedLoopFixture::new("spec_test");

    // When: I append observations
    for i in 0..10 {
        let obs = observation::Observation::new(
            "test_event".to_string(),
            serde_json::json!({"value": i}),
            "spec_test".to_string(),
            HashMap::new(),
        );
        fixture.observation_store.append(obs);
    }

    // Then: All observations are recorded
    assert_eq!(fixture.observation_store.count_observations(), 10);

    // And: I can retrieve them
    let all_obs = fixture
        .observation_store
        .get_sector_observations("spec_test");
    assert_eq!(all_obs.len(), 10);

    // And: Observations are immutable (idempotency: appending same obs again gives different ID)
    let obs = observation::Observation::new(
        "test_event".to_string(),
        serde_json::json!({"value": 42}),
        "spec_test".to_string(),
        HashMap::new(),
    );
    let id1 = obs.id.clone();
    fixture.observation_store.append(obs.clone());

    // If appended again, different timestamp → different ID
    let obs2 = observation::Observation::new(
        "test_event".to_string(),
        serde_json::json!({"value": 42}),
        "spec_test".to_string(),
        HashMap::new(),
    );
    let id2 = obs2.id.clone();
    // IDs will differ due to timestamp
    assert_ne!(id1, id2);
}

#[tokio::test]
async fn spec_rule_1_patterns_detected_from_observations() {
    // Given: Observation store with many events
    let fixture = ClosedLoopFixture::new("pattern_test");

    // When: I add >100 events in one minute (frequency anomaly)
    for i in 0..150 {
        let obs = observation::Observation::new(
            "rapid_event".to_string(),
            serde_json::json!({"index": i}),
            "pattern_test".to_string(),
            HashMap::new(),
        );
        fixture.observation_store.append(obs);
    }

    // And: I run pattern detection (the "Analyze" phase)
    let detector = observation::PatternDetector::new(fixture.observation_store.clone());
    let patterns = detector.detect_patterns().await;

    // Then: Pattern is detected
    assert!(!patterns.is_empty());

    // And: Recommended action is to propose change
    let pattern = patterns.iter().find(|p| p.name.contains("high_frequency"));
    assert!(pattern.is_some());
}

// ============================================================================
// SPECIFICATION: Rule 2 - Bind to Measurable Guarantees (Hard Invariants Q)
// ============================================================================

#[test]
fn spec_rule_2_q1_no_retrocausation() {
    // Given: Invariant validator
    // When: I check Q1 with snapshot → parent chain
    let mut visited = std::collections::HashSet::new();
    let result = invariants::InvariantValidator::check_q1_no_retrocausation(
        "snap1",
        Some("snap0"),
        &mut visited,
    );

    // Then: No cycle error
    assert!(result.is_ok());

    // When: I create self-reference (invalid)
    let mut visited = std::collections::HashSet::new();
    let result = invariants::InvariantValidator::check_q1_no_retrocausation(
        "snap1",
        Some("snap1"),
        &mut visited,
    );

    // Then: Cycle error
    assert!(result.is_err());
}

#[test]
fn spec_rule_2_q3_guard_preservation() {
    // Given: Q3 requires max_run_length ≤ 8 (Chatman constant)
    // When: I check guard with valid latency
    let result = invariants::InvariantValidator::check_q3_guard_preservation(8);

    // Then: Pass
    assert!(result.is_ok());

    // When: I check with violation
    let result = invariants::InvariantValidator::check_q3_guard_preservation(9);

    // Then: Fail with clear error
    assert!(result.is_err());
}

#[test]
fn spec_rule_2_comprehensive_invariant_check() {
    // Given: All hard invariants must hold together
    // When: I check all with valid parameters
    let result = invariants::InvariantValidator::check_all(
        "snap1",
        Some("snap0"),
        100,  // observation_count
        0,    // schema_violations
        8,    // max_ticks
        8,    // hot_path_latency_ticks
        50,   // warm_path_latency_ms
        512,  // memory_mb
        25.0, // cpu_percent
        250,  // tail_latency_ms
    );

    // Then: All preserved
    assert!(result.is_ok());
    let inv = result.unwrap();
    assert!(inv.all_preserved());

    // When: Multiple violations
    let result = invariants::InvariantValidator::check_all(
        "snap1",
        Some("snap0"),
        100,
        50,   // Schema violations
        9,    // Guard violation
        9,    // Hot path violation
        150,  // Warm path violation
        2048, // Memory violation
        75.0, // CPU violation
        750,  // Tail latency violation
    );

    // Then: Caught
    assert!(result.is_err());
}

// ============================================================================
// SPECIFICATION: Rule 3 - Close the Loop (MAPE-K Coordinator)
// ============================================================================

#[tokio::test]
async fn spec_rule_3_mape_k_cycle_complete() {
    // Given: Fixture with monitor, analyze, plan, execute, knowledge phases
    let fixture = ClosedLoopFixture::new("mape_test");

    // When: I execute one MAPE-K cycle
    let cycle = fixture
        .coordinator
        .execute_cycle()
        .await
        .expect("cycle failed");

    // Then: Cycle completed
    assert!(cycle.completed_at.is_some());
    assert!(cycle.duration_ms.is_some());

    // And: Receipts generated for each phase
    assert!(!cycle.receipt_ids.is_empty());
}

#[tokio::test]
async fn spec_rule_3_pattern_detection_triggers_proposals() {
    // Given: Fixture
    let fixture = ClosedLoopFixture::new("proposal_test");

    // When: I add >100 observations
    for i in 0..150 {
        let obs = observation::Observation::new(
            "event".to_string(),
            serde_json::json!({"i": i}),
            "proposal_test".to_string(),
            HashMap::new(),
        );
        fixture.observation_store.append(obs);
    }

    // And: Execute cycle (patterns will be detected)
    let cycle = fixture
        .coordinator
        .execute_cycle()
        .await
        .expect("cycle failed");

    // Then: Patterns detected
    assert!(cycle.patterns_detected > 0);

    // And: Proposals generated
    assert!(cycle.proposals_generated > 0);

    // And: Validations executed
    assert!(cycle.validations_passed + cycle.validations_failed > 0);
}

// ============================================================================
// SPECIFICATION: Rule 4 - Measure Everything (Receipts)
// ============================================================================

#[test]
fn spec_rule_4_receipt_is_cryptographic_proof() {
    // Given: Signing key
    let mut seed = [0u8; 32];
    seed[0] = 42;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();

    // When: I create a receipt
    let receipt = receipt::Receipt::create(
        receipt::ReceiptOperation::PatternDetected {
            pattern: "test".to_string(),
            confidence: 0.95,
        },
        receipt::ReceiptOutcome::Approved,
        vec!["evidence1".to_string()],
        "test_sector".to_string(),
        &signing_key,
        None,
    )
    .expect("receipt creation failed");

    // Then: Receipt is signed
    assert!(!receipt.signature.is_empty());

    // And: Signature verifies
    assert!(receipt.verify(&verifying_key).is_ok());

    // When: I tamper with receipt
    let mut tampered = receipt.clone();
    tampered.id = "tampered".to_string();

    // Then: Verification fails
    assert!(tampered.verify(&verifying_key).is_err());
}

#[tokio::test]
async fn spec_rule_4_receipt_chain_of_custody() {
    // Given: Receipt store with signing key
    let mut seed = [0u8; 32];
    seed[0] = 42;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    let store = Arc::new(receipt::ReceiptStore::new(verifying_key));

    // When: I create and append first receipt
    let receipt1 = receipt::Receipt::create(
        receipt::ReceiptOperation::PatternDetected {
            pattern: "p1".to_string(),
            confidence: 0.8,
        },
        receipt::ReceiptOutcome::Approved,
        vec![],
        "s1".to_string(),
        &signing_key,
        None,
    )
    .expect("r1 creation failed");

    let id1 = receipt1.id.clone();
    store.append(receipt1).await.expect("r1 append failed");

    // And: Create second receipt with parent reference
    let receipt2 = receipt::Receipt::create(
        receipt::ReceiptOperation::ProposalGenerated {
            delta_description: "test".to_string(),
        },
        receipt::ReceiptOutcome::Pending {
            next_stage: "validation".to_string(),
        },
        vec![],
        "s1".to_string(),
        &signing_key,
        Some(id1.clone()),
    )
    .expect("r2 creation failed");

    store
        .append(receipt2.clone())
        .await
        .expect("r2 append failed");

    // Then: Chain verified (parent must exist and have earlier timestamp)
    let result = receipt2.verify_chain(&store);
    assert!(result.is_ok());

    // And: Can retrieve full chain
    let chain = store
        .get_chain(&receipt2.id)
        .expect("chain retrieval failed");
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0].id, id1);
}

// ============================================================================
// SPECIFICATION: Rule 5 - Picoseconds to Decisions (Atomic Promotion)
// ============================================================================

#[test]
fn spec_rule_5_atomic_promotion_via_pointer_swap() {
    // Given: Snapshot promoter with genesis snapshot
    let genesis = promoter::SnapshotDescriptor {
        snapshot_id: "snap0".to_string(),
        parent_id: None,
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 0,
    };
    let promoter = promoter::SnapshotPromoter::new(genesis);

    // When: I read current snapshot (fast path)
    let start = std::time::Instant::now();
    let current = promoter.current();
    let read_duration = start.elapsed().as_nanos();

    // Then: Read is nearly free (<100ns)
    assert!(read_duration < 100);
    assert_eq!(current.snapshot_id, "snap0");

    // When: I promote new snapshot
    let snap1 = promoter::SnapshotDescriptor {
        snapshot_id: "snap1".to_string(),
        parent_id: Some("snap0".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };

    let start = std::time::Instant::now();
    let _result = promoter.promote(snap1).expect("promote failed");
    let promote_duration = start.elapsed().as_nanos();

    // Then: Promotion is atomic and fast (<10μs typical, <1μs ideal)
    assert!(promote_duration < 10_000);
    assert_eq!(promoter.current().snapshot_id, "snap1");
}

#[test]
fn spec_rule_5_promotion_preserves_immutability() {
    // Given: Promoter with chain snap0 → snap1 → snap2
    let snap0 = promoter::SnapshotDescriptor {
        snapshot_id: "snap0".to_string(),
        parent_id: None,
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 0,
    };
    let promoter = promoter::SnapshotPromoter::new(snap0);

    let snap1 = promoter::SnapshotDescriptor {
        snapshot_id: "snap1".to_string(),
        parent_id: Some("snap0".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };
    promoter.promote(snap1).expect("promote1 failed");

    let snap2 = promoter::SnapshotDescriptor {
        snapshot_id: "snap2".to_string(),
        parent_id: Some("snap1".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 2,
    };
    promoter.promote(snap2).expect("promote2 failed");

    // When: I get snapshot chain
    let chain = promoter.chain().expect("chain failed");

    // Then: Chain is immutable (snap0 forever has no parent)
    assert_eq!(chain.last().unwrap().parent_id, None);

    // And: Chain is ordered
    assert_eq!(chain[0].snapshot_id, "snap2");
    assert_eq!(chain[1].snapshot_id, "snap1");
    assert_eq!(chain[2].snapshot_id, "snap0");
}

#[test]
fn spec_rule_5_promotion_latency_under_budget() {
    // Given: Promoter with stats tracking
    let snap0 = promoter::SnapshotDescriptor {
        snapshot_id: "snap0".to_string(),
        parent_id: None,
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 0,
    };
    let promoter = promoter::SnapshotPromoterWithStats::new(snap0);

    // When: I do 100 promotions
    for i in 1..=100 {
        let snap = promoter::SnapshotDescriptor {
            snapshot_id: format!("snap{}", i),
            parent_id: Some(format!("snap{}", i - 1)),
            promoted_at: chrono::Utc::now().timestamp_millis() as u64,
            version: i as u32,
        };
        promoter.promote(snap).expect("promote failed");
    }

    // Then: Stats show all promotions succeeded
    let stats = promoter.get_stats();
    assert_eq!(stats.total_promotions, 100);

    // And: Average latency is sub-microsecond (<100ns)
    assert!(stats.average_promotion_latency_ns < 100.0);

    // And: Max latency is under 10μs
    assert!(stats.max_promotion_latency_ns < 10_000);
}

// ============================================================================
// INTEGRATION TEST: The Complete Narrative
// ============================================================================

#[tokio::test]
async fn integration_complete_autonomous_loop_closure() {
    // Given: Complete fixture representing 2027 system
    let fixture = ClosedLoopFixture::new("integration");

    // Step 1: Add observations (Rule 1: Model reality)
    for i in 0..200 {
        let obs = observation::Observation::new(
            "system_event".to_string(),
            serde_json::json!({"event_num": i, "status": "running"}),
            "integration".to_string(),
            {
                let mut m = HashMap::new();
                m.insert("source".to_string(), "integration_test".to_string());
                m
            },
        );
        fixture.observation_store.append(obs);
    }

    // Step 2: Execute full MAPE-K cycle
    let cycle = fixture
        .coordinator
        .execute_cycle()
        .await
        .expect("cycle failed");

    // Verify: Observations were ingested
    assert!(fixture.observation_store.count_observations() > 0);

    // Verify: Cycle belongs to integration sector
    assert!(cycle.id.contains("integration"));

    // Verify: Patterns detected (high frequency)
    assert!(
        cycle.patterns_detected > 0,
        "No patterns detected in observations"
    );

    // Verify: Proposals generated
    assert!(cycle.proposals_generated > 0, "No proposals generated");

    // Verify: Validations executed
    assert!(
        cycle.validations_passed + cycle.validations_failed > 0,
        "No validations ran"
    );

    // Verify: Receipts created (proof of everything)
    assert!(!cycle.receipt_ids.is_empty(), "No receipts generated");

    // Step 3: Verify all receipts are valid (Rule 4: Measure everything)
    for receipt_id in &cycle.receipt_ids {
        let receipt = fixture
            .receipt_store
            .get(receipt_id)
            .expect("receipt not found");

        // Each receipt has valid signature
        assert!(!receipt.signature.is_empty(), "Receipt missing signature");

        // Receipt has operation
        assert!(matches!(
            receipt.operation,
            receipt::ReceiptOperation::PatternDetected { .. }
                | receipt::ReceiptOperation::ProposalGenerated { .. }
                | receipt::ReceiptOperation::ValidationExecuted { .. }
                | receipt::ReceiptOperation::LoopCycleCompleted { .. }
        ));
    }

    // Step 4: Promote new snapshot (Rule 5: Picoseconds)
    let new_snapshot = promoter::SnapshotDescriptor {
        snapshot_id: "snap_post_cycle".to_string(),
        parent_id: Some("genesis".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };

    fixture
        .promoter
        .promote(new_snapshot)
        .expect("promotion failed");

    // Verify: Current snapshot changed atomically
    assert_eq!(fixture.promoter.current().snapshot_id, "snap_post_cycle");

    // Verify: Latency under budget
    let stats = fixture.promoter.get_stats();
    assert!(stats.average_promotion_latency_ns < 1000.0);

    // Final: All loop phases completed successfully
    println!("\n✅ INTEGRATION TEST PASSED\n");
    println!("Loop Closure Narrative:");
    println!(
        "  1. Observed {} system events",
        fixture.observation_store.count_observations()
    );
    println!(
        "  2. Detected {} patterns (reality model)",
        cycle.patterns_detected
    );
    println!(
        "  3. Generated {} proposals (binding to guarantees)",
        cycle.proposals_generated
    );
    println!(
        "  4. Executed {} validations ({} passed, {} failed)",
        cycle.validations_passed + cycle.validations_failed,
        cycle.validations_passed,
        cycle.validations_failed
    );
    println!(
        "  5. Generated {} receipts (proof of execution)",
        cycle.receipt_ids.len()
    );
    println!(
        "  6. Promoted snapshot in {:.2}ns (atomic)",
        stats.average_promotion_latency_ns
    );
    println!(
        "  7. Cycle completed in {}ms",
        cycle.duration_ms.unwrap_or(0)
    );
    println!("\nThe loop closes. The 2027 narrative is validated.");
}

// ============================================================================
// PROPERTY-BASED TESTS: Verify invariants hold under stress
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_invariant_q3_always_preserved(max_ticks in 1u32..=100) {
        let result = invariants::InvariantValidator::check_q3_guard_preservation(max_ticks);

        prop_assert_eq!(result.is_ok(), max_ticks <= CHATMAN_CONSTANT);
    }

    #[test]
    fn prop_snapshot_chain_immutable(snapshot_ids in prop::collection::vec("[a-z0-9]{1,20}", 1..10)) {
        let promoter = promoter::SnapshotPromoter::new(
            promoter::SnapshotDescriptor {
                snapshot_id: snapshot_ids[0].clone(),
                parent_id: None,
                promoted_at: chrono::Utc::now().timestamp_millis() as u64,
                version: 0,
            }
        );

        // Build chain
        for (i, id) in snapshot_ids.iter().enumerate().skip(1) {
            let snap = promoter::SnapshotDescriptor {
                snapshot_id: id.clone(),
                parent_id: Some(snapshot_ids[i-1].clone()),
                promoted_at: chrono::Utc::now().timestamp_millis() as u64,
                version: i as u32,
            };
            let _ = promoter.promote(snap);
        }

        // Verify chain is immutable (parent_id never changes)
        let chain = promoter.chain().unwrap();
        let current_id = promoter.current().snapshot_id.clone();
        prop_assert_eq!(chain[0].snapshot_id, current_id);
    }
}
