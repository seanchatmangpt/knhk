//! Integration tests for Pattern Miner
//!
//! These tests verify the pattern detection logic works correctly across
//! various scenarios including schema mismatches, repeated structures,
//! guard violations, and performance regressions.

use knhk_change_engine::{
    PatternMiner,
    pattern_miner::{Receipt, Operation, OperationType, Triple, OperationResult, PerformanceMetrics},
};

#[test]
fn test_detect_schema_mismatches() {
    let mut miner = PatternMiner::new(100);

    // Create receipts with schema mismatches
    for i in 0..15 {
        let receipt = Receipt {
            id: format!("r{}", i),
            timestamp: i * 1000,
            operations: vec![
                Operation {
                    op_type: OperationType::Insert,
                    triple: Triple {
                        subject: format!("s{}", i),
                        predicate: "hasAge".to_string(),
                        object: "not_a_number".to_string(),
                    },
                    result: OperationResult::SchemaMismatch("type mismatch: expected xsd:integer".to_string()),
                }
            ],
            metrics: PerformanceMetrics {
                total_ticks: 5,
                ops_per_sec: 200.0,
            },
        };

        miner.scan_receipt(receipt);
    }

    let patterns = miner.detected_patterns();

    // Should detect schema mismatches
    assert!(patterns.schema_mismatches.len() > 0);

    // Find the hasAge mismatch
    let age_mismatch = patterns.schema_mismatches.iter()
        .find(|m| m.triple.predicate == "hasAge");

    assert!(age_mismatch.is_some());
    assert!(age_mismatch.unwrap().frequency >= 15);
}

#[test]
fn test_detect_repeated_structures() {
    let mut miner = PatternMiner::new(100);

    // Create receipts with repeated predicate usage
    for i in 0..20 {
        let receipt = Receipt {
            id: format!("r{}", i),
            timestamp: i * 1000,
            operations: vec![
                Operation {
                    op_type: OperationType::Insert,
                    triple: Triple {
                        subject: format!("user{}", i),
                        predicate: "hasEmail".to_string(),
                        object: format!("user{}@example.com", i),
                    },
                    result: OperationResult::Success,
                }
            ],
            metrics: PerformanceMetrics {
                total_ticks: 3,
                ops_per_sec: 300.0,
            },
        };

        miner.scan_receipt(receipt);
    }

    let patterns = miner.detected_patterns();

    // Should detect repeated structure (hasEmail appears 20 times)
    assert!(patterns.repeated_structures.len() > 0);

    let email_pattern = patterns.repeated_structures.iter()
        .find(|s| s.pattern.1 == "hasEmail");

    assert!(email_pattern.is_some());
    assert_eq!(email_pattern.unwrap().count, 20);
}

#[test]
fn test_detect_guard_violations() {
    let mut miner = PatternMiner::new(100);

    // Create receipts with guard violations
    for i in 0..5 {
        let receipt = Receipt {
            id: format!("r{}", i),
            timestamp: i * 1000,
            operations: vec![
                Operation {
                    op_type: OperationType::Insert,
                    triple: Triple {
                        subject: format!("user{}", i),
                        predicate: "salary".to_string(),
                        object: "100000".to_string(),
                    },
                    result: OperationResult::GuardViolation("pii_protection".to_string()),
                }
            ],
            metrics: PerformanceMetrics {
                total_ticks: 4,
                ops_per_sec: 250.0,
            },
        };

        miner.scan_receipt(receipt);
    }

    let patterns = miner.detected_patterns();

    // Should detect guard violations
    assert!(patterns.guard_violations.len() > 0);

    let pii_violation = patterns.guard_violations.iter()
        .find(|v| v.guard_name == "pii_protection");

    assert!(pii_violation.is_some());
    assert_eq!(pii_violation.unwrap().near_miss_count, 5);
}

#[test]
fn test_detect_performance_regressions() {
    let mut miner = PatternMiner::new(100);

    // Create receipt with performance regression (>8 ticks)
    let receipt = Receipt {
        id: "slow_op".to_string(),
        timestamp: 1000,
        operations: vec![
            Operation {
                op_type: OperationType::Query,
                triple: Triple {
                    subject: "s1".to_string(),
                    predicate: "p1".to_string(),
                    object: "o1".to_string(),
                },
                result: OperationResult::PerformanceIssue("slow query".to_string()),
            }
        ],
        metrics: PerformanceMetrics {
            total_ticks: 15, // Violates Chatman Constant (≤8 ticks)
            ops_per_sec: 66.0,
        },
    };

    miner.scan_receipt(receipt);

    let patterns = miner.detected_patterns();

    // Should detect performance regression
    assert!(patterns.performance_regressions.len() > 0);

    let regression = &patterns.performance_regressions[0];
    assert_eq!(regression.observed_latency_ticks, 15);
    assert_eq!(regression.expected_latency_ticks, 8);
    assert!(regression.regression_factor > 1.5);
}

#[test]
fn test_window_size_enforcement() {
    let window_size = 10;
    let mut miner = PatternMiner::new(window_size);

    // Add more receipts than window size
    for i in 0..20 {
        let receipt = Receipt {
            id: format!("r{}", i),
            timestamp: i * 1000,
            operations: vec![],
            metrics: PerformanceMetrics {
                total_ticks: 5,
                ops_per_sec: 200.0,
            },
        };

        miner.scan_receipt(receipt);
    }

    // Window should not exceed specified size
    assert_eq!(miner.window_occupancy(), window_size);
}

#[test]
fn test_acknowledge_patterns_resets_state() {
    let mut miner = PatternMiner::new(100);

    // Add some patterns
    let receipt = Receipt {
        id: "r1".to_string(),
        timestamp: 1000,
        operations: vec![
            Operation {
                op_type: OperationType::Insert,
                triple: Triple {
                    subject: "s1".to_string(),
                    predicate: "p1".to_string(),
                    object: "o1".to_string(),
                },
                result: OperationResult::SchemaMismatch("test".to_string()),
            }
        ],
        metrics: PerformanceMetrics {
            total_ticks: 5,
            ops_per_sec: 200.0,
        },
    };

    miner.scan_receipt(receipt);

    // Verify patterns detected
    assert!(miner.detected_patterns().schema_mismatches.len() > 0);

    // Acknowledge patterns
    miner.acknowledge_patterns();

    // Patterns should be reset
    assert_eq!(miner.detected_patterns().schema_mismatches.len(), 0);
}

#[test]
fn test_concurrent_pattern_updates() {
    use std::sync::Arc;
    use std::thread;

    let miner = Arc::new(std::sync::Mutex::new(PatternMiner::new(100)));
    let mut handles = vec![];

    // Spawn multiple threads adding receipts
    for thread_id in 0..5 {
        let miner_clone = miner.clone();
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let receipt = Receipt {
                    id: format!("t{}_r{}", thread_id, i),
                    timestamp: (thread_id * 100 + i) as u64,
                    operations: vec![
                        Operation {
                            op_type: OperationType::Insert,
                            triple: Triple {
                                subject: format!("s{}", i),
                                predicate: "concurrent".to_string(),
                                object: format!("o{}", i),
                            },
                            result: OperationResult::Success,
                        }
                    ],
                    metrics: PerformanceMetrics {
                        total_ticks: 4,
                        ops_per_sec: 250.0,
                    },
                };

                miner_clone.lock().unwrap().scan_receipt(receipt);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify patterns were collected from all threads
    let final_miner = miner.lock().unwrap();
    assert!(final_miner.window_occupancy() > 0);
}
