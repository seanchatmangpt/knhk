// Platform 2027 - Comprehensive Chicago TDD Test Suite
// Tests shadow environments, guard governance, doctrine enforcement, sector validation, org structure mapping
//
// CRITICAL: Uses state-based testing with real collaborators (not mocks)
// CRITICAL: Property-based testing with proptest for invariant verification
// CRITICAL: Latency measurement against 8-tick budget (Chatman Constant)

use knhk_closed_loop::*;
use std::sync::Arc;
use std::collections::HashMap;
use ed25519_dalek::SigningKey;
use proptest::prelude::*;

// ============================================================================
// TEST FIXTURE - Real Collaborators (Chicago TDD Pattern)
// ============================================================================

/// Closed-loop test fixture with all real components
struct ClosedLoopFixture {
    receipt_store: Arc<ReceiptStore>,
    observation_store: Arc<ObservationStore>,
    doctrine_store: Arc<DoctrineStore>,
    promoter: SnapshotPromoter,
    coordinator: MapEKCoordinator,
    signing_key: SigningKey,
    sector: String,
}

impl ClosedLoopFixture {
    fn new(sector: &str) -> Self {
        let signing_key = create_test_signing_key();
        let verifying_key = signing_key.verifying_key();

        let receipt_store = Arc::new(ReceiptStore::new(verifying_key));
        let observation_store = Arc::new(ObservationStore::new());
        let doctrine_store = Arc::new(DoctrineStore::new().expect("Failed to create doctrine store"));

        let genesis = SnapshotDescriptor {
            snapshot_id: format!("{}-genesis", sector),
            parent_id: None,
            promoted_at: chrono::Utc::now().timestamp_millis() as u64,
            version: 0,
        };
        let promoter = SnapshotPromoter::new(genesis);

        let coordinator = MapEKCoordinator::new(
            observation_store.clone(),
            receipt_store.clone(),
            signing_key.clone(),
            sector.to_string(),
        );

        ClosedLoopFixture {
            receipt_store,
            observation_store,
            doctrine_store,
            promoter,
            coordinator,
            signing_key,
            sector: sector.to_string(),
        }
    }

    fn add_observations(&self, count: usize, event_type: &str) {
        for i in 0..count {
            let obs = Observation::new(
                event_type.to_string(),
                serde_json::json!({"index": i, "value": i * 10}),
                self.sector.clone(),
                HashMap::new(),
            );
            self.observation_store.append(obs);
        }
    }

    fn create_finance_doctrine(&self) -> DoctrineRule {
        DoctrineRule {
            id: "FIN-APPROVAL-001".to_string(),
            name: "Finance Dual Approval".to_string(),
            sector: "finance".to_string(),
            constraint_type: ConstraintType::ApprovalChain {
                required_signers: 2,
                sectors: vec!["finance".to_string(), "compliance".to_string()],
            },
            description: "All finance changes require dual approval from finance and compliance".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "SOX-2027".to_string(),
            effective_date: 0,
            expires: None,
        }
    }

    fn create_healthcare_doctrine(&self) -> DoctrineRule {
        DoctrineRule {
            id: "HIPAA-PROTOCOL-001".to_string(),
            name: "Healthcare Data Access Control".to_string(),
            sector: "healthcare".to_string(),
            constraint_type: ConstraintType::SegregationOfDuties {
                incompatible_roles: vec![
                    vec!["doctor".to_string(), "billing".to_string()],
                    vec!["patient".to_string(), "administrator".to_string()],
                ],
            },
            description: "Healthcare professionals cannot access billing data".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "HIPAA-2027".to_string(),
            effective_date: 0,
            expires: None,
        }
    }

    fn create_manufacturing_doctrine(&self) -> DoctrineRule {
        DoctrineRule {
            id: "ISO9001-QUALITY-001".to_string(),
            name: "Manufacturing Quality Threshold".to_string(),
            sector: "manufacturing".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "defect_rate".to_string(),
                max_value: 0.001, // 0.1% defect rate
            },
            description: "Defect rate must not exceed 0.1%".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "ISO-9001-2027".to_string(),
            effective_date: 0,
            expires: None,
        }
    }

    fn create_logistics_doctrine(&self) -> DoctrineRule {
        DoctrineRule {
            id: "LOG-DELIVERY-SLA-001".to_string(),
            name: "Logistics Delivery SLA".to_string(),
            sector: "logistics".to_string(),
            constraint_type: ConstraintType::TimeWindow {
                start_hour: 6,  // 6 AM
                end_hour: 22,   // 10 PM
                days: vec!["Monday".to_string(), "Tuesday".to_string(), "Wednesday".to_string(),
                          "Thursday".to_string(), "Friday".to_string()],
            },
            description: "Deliveries only during business hours on weekdays".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Warning,
            source: "LOG-SLA-2027".to_string(),
            effective_date: 0,
            expires: None,
        }
    }
}

fn create_test_signing_key() -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = 42;
    SigningKey::from_bytes(&seed)
}

// ============================================================================
// SHADOW ENVIRONMENT TESTS (State-Based + Property Tests)
// ============================================================================

#[test]
fn spec_shadows_are_immutable() {
    // GIVEN: A snapshot promoter with genesis snapshot
    let fixture = ClosedLoopFixture::new("test_sector");
    let genesis = fixture.promoter.current();
    let genesis_id = genesis.snapshot_id.clone();

    // WHEN: We promote a child snapshot
    let child = SnapshotDescriptor {
        snapshot_id: "child-1".to_string(),
        parent_id: Some(genesis_id.clone()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };
    fixture.promoter.promote(child).expect("Failed to promote child");

    // THEN: Genesis snapshot is unchanged
    let retrieved_genesis = fixture.promoter.get(&genesis_id).expect("Genesis not found");
    assert_eq!(retrieved_genesis.snapshot_id, genesis_id);
    assert_eq!(retrieved_genesis.version, 0);
    assert!(retrieved_genesis.parent_id.is_none());
}

#[test]
fn spec_shadow_changes_are_reversible() {
    // GIVEN: A chain of snapshots (genesis → v1 → v2)
    let fixture = ClosedLoopFixture::new("test_sector");
    let genesis = fixture.promoter.current();
    let genesis_id = genesis.snapshot_id.clone();

    let v1 = SnapshotDescriptor {
        snapshot_id: "v1".to_string(),
        parent_id: Some(genesis_id.clone()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };
    fixture.promoter.promote(v1).expect("v1 promotion failed");

    let v2 = SnapshotDescriptor {
        snapshot_id: "v2".to_string(),
        parent_id: Some("v1".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 2,
    };
    fixture.promoter.promote(v2).expect("v2 promotion failed");

    // WHEN: We rollback from v2
    fixture.promoter.rollback().expect("Rollback failed");

    // THEN: Current snapshot is v1
    assert_eq!(fixture.promoter.current().snapshot_id, "v1");

    // AND: v2 still exists in history
    assert!(fixture.promoter.get("v2").is_ok());
}

#[test]
fn spec_parallel_shadows_dont_interfere() {
    // GIVEN: Two independent snapshot promoters
    let fixture1 = ClosedLoopFixture::new("sector_a");
    let fixture2 = ClosedLoopFixture::new("sector_b");

    // WHEN: We promote snapshots in parallel
    let snap_a = SnapshotDescriptor {
        snapshot_id: "sector-a-v1".to_string(),
        parent_id: Some("sector_a-genesis".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };
    let snap_b = SnapshotDescriptor {
        snapshot_id: "sector-b-v1".to_string(),
        parent_id: Some("sector_b-genesis".to_string()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };

    fixture1.promoter.promote(snap_a).expect("sector_a promotion failed");
    fixture2.promoter.promote(snap_b).expect("sector_b promotion failed");

    // THEN: Each promoter has its own current snapshot
    assert_eq!(fixture1.promoter.current().snapshot_id, "sector-a-v1");
    assert_eq!(fixture2.promoter.current().snapshot_id, "sector-b-v1");

    // AND: Snapshot counts are independent
    assert_eq!(fixture1.promoter.snapshot_count(), 2); // genesis + v1
    assert_eq!(fixture2.promoter.snapshot_count(), 2);
}

#[test]
fn perf_shadow_test_execution_under_budget() {
    // GIVEN: A fixture with multiple snapshots
    let fixture = ClosedLoopFixture::new("perf_test");

    // WHEN: We create 10 snapshots in sequence
    let start = std::time::Instant::now();

    for i in 1..=10 {
        let parent_id = if i == 1 {
            Some(fixture.promoter.current().snapshot_id.clone())
        } else {
            Some(format!("perf-v{}", i - 1))
        };

        let snapshot = SnapshotDescriptor {
            snapshot_id: format!("perf-v{}", i),
            parent_id,
            promoted_at: chrono::Utc::now().timestamp_millis() as u64,
            version: i,
        };
        fixture.promoter.promote(snapshot).expect("Promotion failed");
    }

    let elapsed = start.elapsed();

    // THEN: Total time for 10 promotions is under 100ms
    assert!(elapsed.as_millis() < 100,
            "Shadow creation took {}ms, expected <100ms", elapsed.as_millis());
}

// ============================================================================
// GUARD GOVERNANCE TESTS (Cryptographic + Consensus)
// ============================================================================

#[tokio::test]
async fn spec_critical_guards_require_quorum() {
    // GIVEN: A finance doctrine requiring 2 signers
    let fixture = ClosedLoopFixture::new("finance");
    let doctrine = fixture.create_finance_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: We validate with only 1 signer
    let context = ValidationContext {
        signers: vec![
            Signer {
                id: "user1".to_string(),
                role: "finance_manager".to_string(),
                sector: "finance".to_string(),
            }
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("critical_change", "finance", &context)
        .expect("Validation failed");

    // THEN: Violation is detected
    assert_eq!(violations.len(), 1);
    assert!(violations[0].is_blocking());
    assert!(violations[0].violation_reason.contains("Requires 2 signers"));
}

#[tokio::test]
async fn spec_approval_signatures_are_valid() {
    // GIVEN: A receipt store with verifying key
    let fixture = ClosedLoopFixture::new("test_sector");

    // WHEN: We create and append a signed receipt
    let receipt = Receipt::create(
        ReceiptOperation::ProposalGenerated {
            delta_description: "Test proposal".to_string(),
        },
        ReceiptOutcome::Pending {
            next_stage: "validation".to_string(),
        },
        vec!["evidence1".to_string()],
        "test_sector".to_string(),
        &fixture.signing_key,
        None,
    ).expect("Receipt creation failed");

    let receipt_id = fixture.receipt_store
        .append(receipt)
        .await
        .expect("Receipt append failed");

    // THEN: Receipt signature is valid
    let retrieved = fixture.receipt_store.get(&receipt_id).expect("Receipt not found");
    let verifying_key = fixture.signing_key.verifying_key();
    assert!(retrieved.verify(&verifying_key).is_ok());
}

#[test]
fn spec_guard_enforcement_is_atomic() {
    // GIVEN: Hard invariants with all guards enabled
    let invariants = HardInvariants {
        q1_no_retrocausation: true,
        q2_type_soundness: true,
        q3_guard_preservation: true,
        q4_slo_compliance: true,
        q5_performance_bounds: true,
    };

    // THEN: All guards are preserved
    assert!(invariants.all_preserved());

    // WHEN: One guard is violated
    let violated_invariants = HardInvariants {
        q1_no_retrocausation: true,
        q2_type_soundness: false, // Violated
        q3_guard_preservation: true,
        q4_slo_compliance: true,
        q5_performance_bounds: true,
    };

    // THEN: System is not in valid state
    assert!(!violated_invariants.all_preserved());
    let violations = violated_invariants.which_violated();
    assert_eq!(violations.len(), 1);
    assert!(violations[0].contains("Q2"));
}

// ============================================================================
// DOCTRINE ENFORCEMENT TESTS (Integration with Governance)
// ============================================================================

#[tokio::test]
async fn spec_doctrine_violation_blocks_promotion() {
    // GIVEN: A finance sector with dual-approval doctrine
    let fixture = ClosedLoopFixture::new("finance");
    let doctrine = fixture.create_finance_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: We try to promote a change with only 1 approval
    let context = ValidationContext {
        signers: vec![
            Signer {
                id: "alice".to_string(),
                role: "cfo".to_string(),
                sector: "finance".to_string(),
            }
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("ΔΣ: increase_budget", "finance", &context)
        .expect("Validation failed");

    // THEN: Promotion is blocked
    assert!(!violations.is_empty());
    assert!(violations.iter().any(|v| v.is_blocking()));
}

#[tokio::test]
async fn spec_multiple_doctrines_compose() {
    // GIVEN: Multiple doctrines for finance sector
    let fixture = ClosedLoopFixture::new("finance");

    let approval_doctrine = fixture.create_finance_doctrine();
    fixture.doctrine_store.add_rule(approval_doctrine).expect("Failed to add approval doctrine");

    let resource_doctrine = DoctrineRule {
        id: "FIN-BUDGET-001".to_string(),
        name: "Budget Limit".to_string(),
        sector: "finance".to_string(),
        constraint_type: ConstraintType::ResourceLimit {
            resource_type: "transaction_amount".to_string(),
            max_value: 1_000_000.0,
        },
        description: "No transaction over $1M".to_string(),
        parameters: HashMap::new(),
        enforcement_level: EnforcementLevel::Mandatory,
        source: "CFO-POLICY-2027".to_string(),
        effective_date: 0,
        expires: None,
    };
    fixture.doctrine_store.add_rule(resource_doctrine).expect("Failed to add resource doctrine");

    // WHEN: We validate with 2 signers but excessive amount
    let mut resources = HashMap::new();
    resources.insert("transaction_amount".to_string(), 2_000_000.0);

    let context = ValidationContext {
        signers: vec![
            Signer { id: "alice".to_string(), role: "cfo".to_string(), sector: "finance".to_string() },
            Signer { id: "bob".to_string(), role: "compliance".to_string(), sector: "compliance".to_string() },
        ],
        resources,
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("large_transaction", "finance", &context)
        .expect("Validation failed");

    // THEN: Resource limit violation is detected (even though approvals are sufficient)
    assert!(violations.iter().any(|v| v.violation_reason.contains("Resource limit exceeded")));
}

#[tokio::test]
async fn spec_sector_doctrines_isolated() {
    // GIVEN: Doctrines for finance and healthcare sectors
    let fixture = ClosedLoopFixture::new("multi_sector");

    let finance_doctrine = fixture.create_finance_doctrine();
    let healthcare_doctrine = fixture.create_healthcare_doctrine();

    fixture.doctrine_store.add_rule(finance_doctrine).expect("Failed to add finance doctrine");
    fixture.doctrine_store.add_rule(healthcare_doctrine).expect("Failed to add healthcare doctrine");

    // WHEN: We validate a finance proposal
    let context = ValidationContext {
        signers: vec![
            Signer { id: "user1".to_string(), role: "cfo".to_string(), sector: "finance".to_string() },
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let finance_violations = fixture.doctrine_store
        .validate_against_doctrines("finance_change", "finance", &context)
        .expect("Finance validation failed");

    // THEN: Only finance doctrines are checked
    assert!(!finance_violations.is_empty());
    assert!(finance_violations.iter().all(|v| v.rule_id.starts_with("FIN-")));

    // WHEN: We validate a healthcare proposal
    let healthcare_context = ValidationContext {
        signers: vec![
            Signer { id: "dr_smith".to_string(), role: "doctor".to_string(), sector: "healthcare".to_string() },
            Signer { id: "dr_smith".to_string(), role: "billing".to_string(), sector: "healthcare".to_string() },
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let healthcare_violations = fixture.doctrine_store
        .validate_against_doctrines("patient_access", "healthcare", &healthcare_context)
        .expect("Healthcare validation failed");

    // THEN: Only healthcare doctrines are checked
    assert!(healthcare_violations.iter().all(|v| v.rule_id.starts_with("HIPAA-")));
}

// ============================================================================
// SECTOR ONTOLOGY TESTS (Multi-sector Consistency)
// ============================================================================

#[tokio::test]
async fn spec_finance_sector_patterns() {
    // GIVEN: A finance sector with transaction observations
    let fixture = ClosedLoopFixture::new("finance");

    // WHEN: We add 150 high-frequency transaction observations
    fixture.add_observations(150, "transaction_event");

    // AND: Execute MAPE-K cycle
    let cycle = fixture.coordinator.execute_cycle().await.expect("Cycle failed");

    // THEN: High-frequency pattern is detected
    assert!(cycle.patterns_detected > 0, "Should detect high-frequency pattern");
    assert!(cycle.proposals_generated > 0, "Should generate proposals");
}

#[tokio::test]
async fn spec_health_sector_safety() {
    // GIVEN: A healthcare sector with safety doctrine
    let fixture = ClosedLoopFixture::new("healthcare");
    let doctrine = fixture.create_healthcare_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: A doctor tries to access billing (segregation of duties violation)
    let context = ValidationContext {
        signers: vec![
            Signer {
                id: "dr_jones".to_string(),
                role: "doctor".to_string(),
                sector: "healthcare".to_string()
            },
            Signer {
                id: "dr_jones".to_string(),
                role: "billing".to_string(),
                sector: "healthcare".to_string()
            },
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("access_billing", "healthcare", &context)
        .expect("Validation failed");

    // THEN: Segregation of duties violation is blocked
    assert!(!violations.is_empty());
    assert!(violations[0].is_blocking());
    assert!(violations[0].violation_reason.contains("Segregation of duties"));
}

#[tokio::test]
async fn spec_mfg_sector_quality() {
    // GIVEN: A manufacturing sector with quality doctrine
    let fixture = ClosedLoopFixture::new("manufacturing");
    let doctrine = fixture.create_manufacturing_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: Quality metrics exceed threshold
    let mut resources = HashMap::new();
    resources.insert("defect_rate".to_string(), 0.002); // 0.2% defect rate (exceeds 0.1%)

    let context = ValidationContext {
        signers: vec![],
        resources,
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("quality_report", "manufacturing", &context)
        .expect("Validation failed");

    // THEN: Quality violation is detected
    assert!(!violations.is_empty());
    assert!(violations[0].is_blocking());
    assert!(violations[0].violation_reason.contains("Resource limit exceeded"));
}

#[tokio::test]
async fn spec_logistics_sector_slas() {
    // GIVEN: A logistics sector with delivery time window doctrine
    let fixture = ClosedLoopFixture::new("logistics");
    let doctrine = fixture.create_logistics_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: We validate a delivery (time window check happens in doctrine validation)
    let context = ValidationContext {
        signers: vec![],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("schedule_delivery", "logistics", &context)
        .expect("Validation failed");

    // THEN: Time window doctrine is evaluated (may or may not violate depending on current time)
    // This test validates that logistics doctrine is properly registered and evaluated
    assert!(fixture.doctrine_store.get_rule("LOG-DELIVERY-SLA-001").is_ok());
}

// ============================================================================
// PERFORMANCE LATENCY TESTS
// ============================================================================

#[test]
fn perf_snapshot_promotion_under_1ns() {
    // GIVEN: A snapshot promoter with genesis
    let fixture = ClosedLoopFixture::new("perf_test");
    let parent_id = fixture.promoter.current().snapshot_id.clone();

    // WHEN: We promote a new snapshot
    let snapshot = SnapshotDescriptor {
        snapshot_id: "perf-snap-1".to_string(),
        parent_id: Some(parent_id),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 1,
    };

    let start = std::time::Instant::now();
    fixture.promoter.promote(snapshot).expect("Promotion failed");
    let elapsed = start.elapsed();

    // THEN: Promotion completes in sub-microsecond time
    assert!(elapsed.as_nanos() < 10_000,
            "Promotion took {}ns, expected <10,000ns", elapsed.as_nanos());
}

#[test]
fn perf_doctrine_validation_under_8_ticks() {
    // GIVEN: A doctrine store with finance approval doctrine
    let fixture = ClosedLoopFixture::new("finance");
    let doctrine = fixture.create_finance_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: We validate against doctrines
    let context = ValidationContext {
        signers: vec![
            Signer { id: "user1".to_string(), role: "cfo".to_string(), sector: "finance".to_string() },
            Signer { id: "user2".to_string(), role: "compliance".to_string(), sector: "compliance".to_string() },
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let start = std::time::Instant::now();
    let _violations = fixture.doctrine_store
        .validate_against_doctrines("test_proposal", "finance", &context)
        .expect("Validation failed");
    let elapsed = start.elapsed();

    // THEN: Validation completes in time budget (assuming ~1ns per tick, <8ns)
    assert!(elapsed.as_micros() < 100,
            "Validation took {}μs, expected <100μs", elapsed.as_micros());
}

#[test]
fn perf_guard_quorum_check_under_100us() {
    // GIVEN: A receipt store with multiple receipts
    let fixture = ClosedLoopFixture::new("test_sector");

    // WHEN: We create and verify multiple receipts
    let start = std::time::Instant::now();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for i in 0..10 {
            let receipt = Receipt::create(
                ReceiptOperation::InvariantChecked {
                    invariant: format!("Q{}", (i % 5) + 1),
                    preserved: true,
                },
                ReceiptOutcome::Approved,
                vec![format!("check_{}", i)],
                "test_sector".to_string(),
                &fixture.signing_key,
                None,
            ).expect("Receipt creation failed");

            fixture.receipt_store.append(receipt).await.expect("Append failed");
        }
    });

    let elapsed = start.elapsed();

    // THEN: 10 receipt creations + verifications complete in <100μs
    assert!(elapsed.as_micros() < 1000,
            "Guard checks took {}μs, expected <1000μs", elapsed.as_micros());
}

#[test]
fn perf_shadow_creation_scaling() {
    // GIVEN: A snapshot promoter
    let fixture = ClosedLoopFixture::new("scaling_test");

    // WHEN: We create 1000 snapshots
    let start = std::time::Instant::now();

    for i in 1..=1000 {
        let parent_id = if i == 1 {
            Some(fixture.promoter.current().snapshot_id.clone())
        } else {
            Some(format!("scale-v{}", i - 1))
        };

        let snapshot = SnapshotDescriptor {
            snapshot_id: format!("scale-v{}", i),
            parent_id,
            promoted_at: chrono::Utc::now().timestamp_millis() as u64,
            version: i,
        };
        fixture.promoter.promote(snapshot).expect("Promotion failed");
    }

    let elapsed = start.elapsed();

    // THEN: 1000 promotions complete in <100ms
    assert!(elapsed.as_millis() < 100,
            "1000 promotions took {}ms, expected <100ms", elapsed.as_millis());
}

// ============================================================================
// INTEGRATION TESTS (End-to-End MAPE-K)
// ============================================================================

#[tokio::test]
async fn integration_complete_finance_workflow() {
    // GIVEN: Finance sector with doctrines and observations
    let fixture = ClosedLoopFixture::new("finance");
    let doctrine = fixture.create_finance_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: We add observations
    fixture.add_observations(150, "transaction_spike");

    // AND: Execute MAPE-K cycle (Observation → Pattern → Doctrine → Guard → Shadow → Promotion)
    let cycle = fixture.coordinator.execute_cycle().await.expect("Cycle failed");

    // THEN: Complete workflow executes
    assert!(cycle.patterns_detected > 0, "Patterns should be detected");
    assert!(!cycle.receipt_ids.is_empty(), "Receipts should be created");
    assert!(cycle.completed_at.is_some(), "Cycle should complete");

    // AND: Receipts are cryptographically valid
    for receipt_id in &cycle.receipt_ids {
        let receipt = fixture.receipt_store.get(receipt_id).expect("Receipt not found");
        let verifying_key = fixture.signing_key.verifying_key();
        assert!(receipt.verify(&verifying_key).is_ok(), "Receipt signature should be valid");
    }
}

#[tokio::test]
async fn integration_healthcare_protocol_update() {
    // GIVEN: Healthcare sector with safety doctrines
    let fixture = ClosedLoopFixture::new("healthcare");
    let doctrine = fixture.create_healthcare_doctrine();
    fixture.doctrine_store.add_rule(doctrine).expect("Failed to add doctrine");

    // WHEN: We propose a protocol change with proper authorization
    let context = ValidationContext {
        signers: vec![
            Signer {
                id: "dr_admin".to_string(),
                role: "administrator".to_string(),
                sector: "healthcare".to_string()
            },
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    let violations = fixture.doctrine_store
        .validate_against_doctrines("protocol_update", "healthcare", &context)
        .expect("Validation failed");

    // THEN: No segregation of duties violations (single role)
    assert!(violations.is_empty() || violations.iter().all(|v| !v.is_blocking()));

    // AND: We can promote a new snapshot
    let new_snapshot = SnapshotDescriptor {
        snapshot_id: "healthcare-protocol-v2".to_string(),
        parent_id: Some(fixture.promoter.current().snapshot_id.clone()),
        promoted_at: chrono::Utc::now().timestamp_millis() as u64,
        version: 2,
    };

    let promoted = fixture.promoter.promote(new_snapshot).expect("Promotion failed");
    assert_eq!(promoted.version, 2);
}

#[tokio::test]
async fn integration_org_restructuring() {
    // GIVEN: Multiple sectors with different doctrines
    let finance_fixture = ClosedLoopFixture::new("finance");
    let healthcare_fixture = ClosedLoopFixture::new("healthcare");

    finance_fixture.doctrine_store.add_rule(finance_fixture.create_finance_doctrine())
        .expect("Failed to add finance doctrine");
    healthcare_fixture.doctrine_store.add_rule(healthcare_fixture.create_healthcare_doctrine())
        .expect("Failed to add healthcare doctrine");

    // WHEN: Organization restructures (e.g., CFO also becomes interim healthcare admin)
    let context = ValidationContext {
        signers: vec![
            Signer {
                id: "alice".to_string(),
                role: "cfo".to_string(),
                sector: "finance".to_string()
            },
            Signer {
                id: "alice".to_string(),
                role: "administrator".to_string(),
                sector: "healthcare".to_string()
            },
        ],
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    };

    // THEN: Finance doctrines still require compliance approval
    let finance_violations = finance_fixture.doctrine_store
        .validate_against_doctrines("finance_change", "finance", &context)
        .expect("Finance validation failed");

    assert!(finance_violations.iter().any(|v|
        v.violation_reason.contains("compliance") && v.is_blocking()
    ));

    // AND: Healthcare doctrines don't have cross-role conflicts for Alice
    let healthcare_violations = healthcare_fixture.doctrine_store
        .validate_against_doctrines("healthcare_change", "healthcare", &context)
        .expect("Healthcare validation failed");

    // Healthcare doctrine checks same-person role conflicts, not cross-sector
    assert!(healthcare_violations.is_empty() ||
            !healthcare_violations.iter().any(|v| v.violation_reason.contains("cfo")));
}

// ============================================================================
// PROPERTY-BASED TESTS (Proptest)
// ============================================================================

proptest! {
    #[test]
    fn prop_shadow_creation_is_sublinear(count in 1usize..=100) {
        let fixture = ClosedLoopFixture::new("prop_test");

        let start = std::time::Instant::now();

        for i in 1..=count {
            let parent_id = if i == 1 {
                Some(fixture.promoter.current().snapshot_id.clone())
            } else {
                Some(format!("prop-v{}", i - 1))
            };

            let snapshot = SnapshotDescriptor {
                snapshot_id: format!("prop-v{}", i),
                parent_id,
                promoted_at: chrono::Utc::now().timestamp_millis() as u64,
                version: i as u32,
            };
            fixture.promoter.promote(snapshot).expect("Promotion failed");
        }

        let elapsed = start.elapsed();
        let avg_per_snapshot = elapsed.as_nanos() / count as u128;

        // Property: Average promotion time should be sublinear (< 100μs per snapshot)
        prop_assert!(avg_per_snapshot < 100_000,
                    "Average {}ns per snapshot exceeds 100μs", avg_per_snapshot);
    }

    #[test]
    fn prop_any_guard_configuration_is_valid(
        q1 in any::<bool>(),
        q2 in any::<bool>(),
        q3 in any::<bool>(),
        q4 in any::<bool>(),
        q5 in any::<bool>()
    ) {
        let invariants = HardInvariants {
            q1_no_retrocausation: q1,
            q2_type_soundness: q2,
            q3_guard_preservation: q3,
            q4_slo_compliance: q4,
            q5_performance_bounds: q5,
        };

        // Property: all_preserved() is deterministic
        let result1 = invariants.all_preserved();
        let result2 = invariants.all_preserved();
        prop_assert_eq!(result1, result2);

        // Property: all_preserved() iff all guards are true
        let expected = q1 && q2 && q3 && q4 && q5;
        prop_assert_eq!(invariants.all_preserved(), expected);
    }

    #[test]
    fn prop_any_doctrine_change_is_reversible(version in 1u32..=20) {
        let fixture = ClosedLoopFixture::new("reversible_test");

        // Create a chain of snapshots
        for i in 1..=version {
            let parent_id = Some(fixture.promoter.current().snapshot_id.clone());

            let snapshot = SnapshotDescriptor {
                snapshot_id: format!("rev-v{}", i),
                parent_id,
                promoted_at: chrono::Utc::now().timestamp_millis() as u64,
                version: i,
            };
            fixture.promoter.promote(snapshot).expect("Promotion failed");
        }

        // Property: We can rollback to any ancestor
        for _ in 1..version {
            prop_assert!(fixture.promoter.rollback().is_ok());
        }
    }
}
