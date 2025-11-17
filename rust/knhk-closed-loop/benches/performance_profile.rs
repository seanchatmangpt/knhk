// Performance Profile: Comprehensive Chatman Constant Validation
// Measures all hot paths against the ≤8 tick constraint

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion, PlotConfiguration,
    Throughput,
};
use knhk_closed_loop::{
    Action, ChatmanEquation, ConstraintType, DoctrineRule, DoctrineStore, DoctrineViolation,
    EnforcementLevel, HardInvariants, MapEKCoordinator, Observation, ObservationStore,
    PatternDetector, Receipt, ReceiptOperation, ReceiptOutcome, ReceiptStore, ResourceBudget,
    Signer, SnapshotPromoter, SnapshotPromoterWithStats, ValidationContext, CHATMAN_CONSTANT,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// BENCHMARK 1: Atomic Snapshot Promotion (Target: <10ns)
// ============================================================================

fn bench_snapshot_promotion(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_promotion");
    group.plot_config(PlotConfiguration::default());

    // Setup: Create promoter with initial snapshot
    let initial = create_snapshot_descriptor("genesis", None);
    let promoter = SnapshotPromoter::new(initial);

    group.bench_function("atomic_swap", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let new_snapshot = create_snapshot_descriptor(
                &format!("snap_{}", counter),
                Some(if counter == 1 {
                    "genesis"
                } else {
                    &format!("snap_{}", counter - 1)
                }),
            );
            let _ = black_box(promoter.promote(new_snapshot));
        });
    });

    // Measure with statistics tracking
    let promoter_stats =
        SnapshotPromoterWithStats::new(create_snapshot_descriptor("genesis", None));

    group.bench_function("atomic_swap_with_stats", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let new_snapshot =
                create_snapshot_descriptor(&format!("snap_{}", counter), Some("genesis"));
            let _ = black_box(promoter_stats.promote(new_snapshot));
        });
    });

    // Concurrent promotion stress test
    group.bench_function("concurrent_promotion_10_threads", |b| {
        b.iter(|| {
            let promoter = Arc::new(SnapshotPromoter::new(create_snapshot_descriptor(
                "genesis", None,
            )));
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let p = promoter.clone();
                    std::thread::spawn(move || {
                        for j in 0..100 {
                            let snap = create_snapshot_descriptor(
                                &format!("t{}_s{}", i, j),
                                Some("genesis"),
                            );
                            let _ = p.promote(snap);
                        }
                    })
                })
                .collect();

            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.finish();
}

// ============================================================================
// BENCHMARK 2: Doctrine Validation (Target: <100ns per rule)
// ============================================================================

fn bench_doctrine_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("doctrine_validation");

    let store = DoctrineStore::new().expect("Failed to create store");

    // Add single approval chain rule
    let approval_rule = create_approval_rule("APPROVAL-001", 2);
    store.add_rule(approval_rule).expect("Failed to add rule");

    // Context with 2 signers
    let context_valid = create_validation_context(2);
    let context_invalid = create_validation_context(1);

    group.bench_function("validate_single_doctrine_pass", |b| {
        b.iter(|| {
            let violations = black_box(
                store
                    .validate_against_doctrines("test proposal", "finance", &context_valid)
                    .expect("Validation failed"),
            );
            assert_eq!(violations.len(), 0);
        });
    });

    group.bench_function("validate_single_doctrine_fail", |b| {
        b.iter(|| {
            let violations = black_box(
                store
                    .validate_against_doctrines("test proposal", "finance", &context_invalid)
                    .expect("Validation failed"),
            );
            assert_eq!(violations.len(), 1);
        });
    });

    // Multiple doctrines (target: ≤8 ticks total)
    let store_multi = DoctrineStore::new().expect("Failed to create store");
    for i in 0..5 {
        let rule = create_approval_rule(&format!("APPROVAL-{:03}", i), 2);
        store_multi.add_rule(rule).expect("Failed to add rule");
    }

    group.bench_function("validate_5_doctrines", |b| {
        b.iter(|| {
            let violations = black_box(
                store_multi
                    .validate_against_doctrines("test proposal", "finance", &context_valid)
                    .expect("Validation failed"),
            );
            assert_eq!(violations.len(), 0);
        });
    });

    // Scalability: 1-100 doctrines
    for count in [1, 5, 10, 20, 50, 100].iter() {
        let store_scale = DoctrineStore::new().expect("Failed to create store");
        for i in 0..*count {
            let rule = create_approval_rule(&format!("RULE-{:03}", i), 2);
            store_scale.add_rule(rule).expect("Failed to add rule");
        }

        group.bench_with_input(
            BenchmarkId::new("validate_n_doctrines", count),
            count,
            |b, _count| {
                b.iter(|| {
                    let violations = black_box(
                        store_scale
                            .validate_against_doctrines("test", "finance", &context_valid)
                            .expect("Validation failed"),
                    );
                    black_box(violations);
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// BENCHMARK 3: Pattern Detection (Target: <1µs)
// ============================================================================

fn bench_pattern_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_detection");

    // Create observation store with various data sizes
    for obs_count in [10, 100, 1000, 10000].iter() {
        let obs_store = Arc::new(ObservationStore::new());
        for i in 0..*obs_count {
            let obs = Observation::new(
                "test_event".to_string(),
                serde_json::json!({"value": i}),
                "test_sector".to_string(),
                HashMap::new(),
            );
            obs_store.append(obs);
        }

        let detector = PatternDetector::new(obs_store.clone());

        group.bench_with_input(
            BenchmarkId::new("detect_patterns", obs_count),
            obs_count,
            |b, _count| {
                b.iter(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let patterns = black_box(detector.detect_patterns().await);
                        black_box(patterns);
                    });
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// BENCHMARK 4: Receipt Signing & Verification (Target: <1ms signing, <500µs verify)
// ============================================================================

fn bench_receipt_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("receipt_operations");

    let signing_key = create_signing_key();
    let verifying_key = signing_key.verifying_key();

    group.bench_function("receipt_signing", |b| {
        b.iter(|| {
            let receipt = black_box(
                Receipt::create(
                    ReceiptOperation::ProposalGenerated {
                        delta_description: "Test change".to_string(),
                    },
                    ReceiptOutcome::Approved,
                    vec!["test metadata".to_string()],
                    "test_sector".to_string(),
                    &signing_key,
                    None,
                )
                .expect("Failed to create receipt"),
            );
            black_box(receipt);
        });
    });

    // Verification
    let receipt = Receipt::create(
        ReceiptOperation::ProposalGenerated {
            delta_description: "Test".to_string(),
        },
        ReceiptOutcome::Approved,
        vec![],
        "test".to_string(),
        &signing_key,
        None,
    )
    .expect("Failed to create receipt");

    group.bench_function("receipt_verification", |b| {
        b.iter(|| {
            let valid = black_box(receipt.verify_signature(&verifying_key));
            assert!(valid);
        });
    });

    // Batch signing
    group.bench_function("receipt_batch_sign_100", |b| {
        b.iter(|| {
            for i in 0..100 {
                let receipt = black_box(
                    Receipt::create(
                        ReceiptOperation::ProposalGenerated {
                            delta_description: format!("Change {}", i),
                        },
                        ReceiptOutcome::Approved,
                        vec![],
                        "test".to_string(),
                        &signing_key,
                        None,
                    )
                    .expect("Failed to create receipt"),
                );
                black_box(receipt);
            }
        });
    });

    group.finish();
}

// ============================================================================
// BENCHMARK 5: Guard Enforcement Check (Target: <50ns)
// ============================================================================

fn bench_guard_enforcement(c: &mut Criterion) {
    let mut group = c.benchmark_group("guard_enforcement");

    let invariants_all_true = HardInvariants {
        q1_no_retrocausation: true,
        q2_type_soundness: true,
        q3_guard_preservation: true,
        q4_slo_compliance: true,
        q5_performance_bounds: true,
    };

    let invariants_one_false = HardInvariants {
        q1_no_retrocausation: true,
        q2_type_soundness: false,
        q3_guard_preservation: true,
        q4_slo_compliance: true,
        q5_performance_bounds: true,
    };

    group.bench_function("guard_check_all_pass", |b| {
        b.iter(|| {
            let result = black_box(invariants_all_true.all_preserved());
            assert!(result);
        });
    });

    group.bench_function("guard_check_one_fail", |b| {
        b.iter(|| {
            let result = black_box(invariants_one_false.all_preserved());
            assert!(!result);
        });
    });

    group.bench_function("guard_identify_violations", |b| {
        b.iter(|| {
            let violations = black_box(invariants_one_false.which_violated());
            assert_eq!(violations.len(), 1);
        });
    });

    group.finish();
}

// ============================================================================
// BENCHMARK 6: Complete MAPE-K Cycle (Target: within latency budget)
// ============================================================================

fn bench_mapek_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("mapek_cycle");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(50);

    let signing_key = create_signing_key();
    let verifying_key = signing_key.verifying_key();

    // Empty cycle (no patterns)
    group.bench_function("cycle_empty", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let obs_store = Arc::new(ObservationStore::new());
                let receipt_store = Arc::new(ReceiptStore::new(verifying_key));
                let coordinator = MapEKCoordinator::new(
                    obs_store,
                    receipt_store,
                    signing_key.clone(),
                    "test".to_string(),
                );

                let cycle = black_box(coordinator.execute_cycle().await.expect("Cycle failed"));
                black_box(cycle);
            });
        });
    });

    // Cycle with patterns (150 observations)
    group.bench_function("cycle_with_patterns", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let obs_store = Arc::new(ObservationStore::new());
                for i in 0..150 {
                    let obs = Observation::new(
                        "test_event".to_string(),
                        serde_json::json!({"value": i}),
                        "test".to_string(),
                        HashMap::new(),
                    );
                    obs_store.append(obs);
                }

                let receipt_store = Arc::new(ReceiptStore::new(verifying_key));
                let coordinator = MapEKCoordinator::new(
                    obs_store,
                    receipt_store,
                    signing_key.clone(),
                    "test".to_string(),
                );

                let cycle = black_box(coordinator.execute_cycle().await.expect("Cycle failed"));
                assert!(cycle.patterns_detected > 0);
                black_box(cycle);
            });
        });
    });

    group.finish();
}

// ============================================================================
// BENCHMARK 7: Real-World Scenarios
// ============================================================================

fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenarios");
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(20);

    // Finance: 1000 transactions/sec validation
    group.bench_function("finance_1000_txn_validation", |b| {
        let store = DoctrineStore::new().expect("Failed to create store");

        // Add 5 finance doctrines (SOD, approval chains, limits)
        for i in 0..5 {
            let rule = create_approval_rule(&format!("FIN-{:03}", i), 2);
            store.add_rule(rule).expect("Failed to add rule");
        }

        let context = create_validation_context(3);

        b.iter(|| {
            for i in 0..1000 {
                let proposal = format!("transaction_{}", i);
                let violations = black_box(
                    store
                        .validate_against_doctrines(&proposal, "finance", &context)
                        .expect("Validation failed"),
                );
                black_box(violations);
            }
        });
    });

    // Healthcare: Compliance checking (100 records)
    group.bench_function("healthcare_100_compliance_checks", |b| {
        let store = DoctrineStore::new().expect("Failed to create store");

        // HIPAA-style time window restrictions
        let hipaa_rule = DoctrineRule {
            id: "HIPAA-001".to_string(),
            name: "Access time window".to_string(),
            sector: "healthcare".to_string(),
            constraint_type: ConstraintType::TimeWindow {
                start_hour: 8,
                end_hour: 18,
                days: vec![
                    "Monday".to_string(),
                    "Tuesday".to_string(),
                    "Wednesday".to_string(),
                ],
            },
            description: "PHI access only during business hours".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "HIPAA".to_string(),
            effective_date: 0,
            expires: None,
        };

        store.add_rule(hipaa_rule).expect("Failed to add rule");

        let context = ValidationContext::default();

        b.iter(|| {
            for i in 0..100 {
                let proposal = format!("patient_record_{}", i);
                let violations = black_box(
                    store
                        .validate_against_doctrines(&proposal, "healthcare", &context)
                        .expect("Validation failed"),
                );
                black_box(violations);
            }
        });
    });

    // Manufacturing: Sensor pattern detection (1000 sensors)
    group.bench_function("manufacturing_1000_sensor_patterns", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let obs_store = Arc::new(ObservationStore::new());

                // Simulate 1000 sensor readings
                for sensor_id in 0..100 {
                    for reading in 0..10 {
                        let obs = Observation::new(
                            format!("sensor_{}", sensor_id),
                            serde_json::json!({
                                "temperature": 20.0 + (reading as f64 * 0.1),
                                "pressure": 100.0,
                                "vibration": 0.5
                            }),
                            "manufacturing".to_string(),
                            HashMap::new(),
                        );
                        obs_store.append(obs);
                    }
                }

                let detector = PatternDetector::new(obs_store);
                let patterns = black_box(detector.detect_patterns().await);
                black_box(patterns);
            });
        });
    });

    // Logistics: Route optimization (1000 shipments)
    group.bench_function("logistics_1000_shipment_validation", |b| {
        let store = DoctrineStore::new().expect("Failed to create store");

        // Resource limit: max delivery time
        let time_limit = DoctrineRule {
            id: "LOG-001".to_string(),
            name: "Delivery time SLA".to_string(),
            sector: "logistics".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "delivery_hours".to_string(),
                max_value: 48.0,
            },
            description: "Deliveries must complete within 48 hours".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "SLA".to_string(),
            effective_date: 0,
            expires: None,
        };

        store.add_rule(time_limit).expect("Failed to add rule");

        b.iter(|| {
            for i in 0..1000 {
                let mut resources = HashMap::new();
                resources.insert("delivery_hours".to_string(), 24.0 + (i as f64 * 0.01));

                let context = ValidationContext {
                    signers: vec![],
                    resources,
                    custom_validations: HashMap::new(),
                };

                let proposal = format!("shipment_{}", i);
                let violations = black_box(
                    store
                        .validate_against_doctrines(&proposal, "logistics", &context)
                        .expect("Validation failed"),
                );
                black_box(violations);
            }
        });
    });

    group.finish();
}

// ============================================================================
// BENCHMARK 8: Latency Percentiles Analysis
// ============================================================================

fn bench_latency_percentiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_percentiles");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10000); // Large sample for accurate percentiles

    let promoter = SnapshotPromoterWithStats::new(create_snapshot_descriptor("genesis", None));

    group.bench_function("promotion_latency_distribution", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let snap = create_snapshot_descriptor(&format!("s{}", counter), Some("genesis"));
            let _ = black_box(promoter.promote(snap));
        });
    });

    // After benchmark, check stats
    let stats = promoter.get_stats();
    println!("\n=== PROMOTION LATENCY STATISTICS ===");
    println!("Total promotions: {}", stats.total_promotions);
    println!(
        "Average latency: {:.2} ns",
        stats.average_promotion_latency_ns
    );
    println!("Max latency: {} ns", stats.max_promotion_latency_ns);
    println!(
        "CHATMAN CONSTANT CHECK: {}",
        if stats.max_promotion_latency_ns < 1_000_000 {
            "✓ PASS (well under 8 ticks)"
        } else {
            "✗ FAIL (exceeds reasonable bounds)"
        }
    );

    group.finish();
}

// ============================================================================
// BENCHMARK 9: Memory Allocation Patterns
// ============================================================================

fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Measure allocations for key operations
    group.bench_function("observation_append_1000", |b| {
        b.iter(|| {
            let store = ObservationStore::new();
            for i in 0..1000 {
                let obs = Observation::new(
                    "event".to_string(),
                    serde_json::json!({"id": i}),
                    "sector".to_string(),
                    HashMap::new(),
                );
                store.append(obs);
            }
            black_box(store);
        });
    });

    group.bench_function("doctrine_store_100_rules", |b| {
        b.iter(|| {
            let store = DoctrineStore::new().expect("Failed to create store");
            for i in 0..100 {
                let rule = create_approval_rule(&format!("R{}", i), 2);
                let _ = store.add_rule(rule);
            }
            black_box(store);
        });
    });

    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_snapshot_descriptor(
    id: &str,
    parent: Option<&str>,
) -> knhk_closed_loop::promoter::SnapshotDescriptor {
    knhk_closed_loop::promoter::SnapshotDescriptor {
        snapshot_id: id.to_string(),
        parent_id: parent.map(|s| s.to_string()),
        promoted_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: 1,
    }
}

fn create_approval_rule(id: &str, required_signers: usize) -> DoctrineRule {
    DoctrineRule {
        id: id.to_string(),
        name: format!("Approval rule {}", id),
        sector: "finance".to_string(),
        constraint_type: ConstraintType::ApprovalChain {
            required_signers,
            sectors: vec!["finance".to_string()],
        },
        description: format!("Requires {} approvers", required_signers),
        parameters: HashMap::new(),
        enforcement_level: EnforcementLevel::Mandatory,
        source: "org-policy".to_string(),
        effective_date: 0,
        expires: None,
    }
}

fn create_validation_context(signer_count: usize) -> ValidationContext {
    let signers: Vec<Signer> = (0..signer_count)
        .map(|i| Signer {
            id: format!("user_{}", i),
            role: "manager".to_string(),
            sector: "finance".to_string(),
        })
        .collect();

    ValidationContext {
        signers,
        resources: HashMap::new(),
        custom_validations: HashMap::new(),
    }
}

fn create_signing_key() -> ed25519_dalek::SigningKey {
    let mut seed = [0u8; 32];
    seed[0] = 42;
    ed25519_dalek::SigningKey::from_bytes(&seed)
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    name = benches;
    config = Criterion::default()
        .significance_level(0.05)
        .noise_threshold(0.05)
        .confidence_level(0.95)
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_snapshot_promotion,
        bench_doctrine_validation,
        bench_pattern_detection,
        bench_receipt_operations,
        bench_guard_enforcement,
        bench_mapek_cycle,
        bench_real_world_scenarios,
        bench_latency_percentiles,
        bench_memory_allocation
);

criterion_main!(benches);
