//! Warm path bounds tests
//!
//! Warm path operations should complete in ≤100ms.

use chicago_tdd::{OperationType, PerformanceHarness};

#[test]
fn test_warm_path_small_allocation() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);

    let result = harness.measure("small_vec_alloc", OperationType::WarmPath, || {
        let v: Vec<u32> = Vec::with_capacity(10);
        v
    });

    assert!(
        !result.bounds_violated,
        "Small allocation too slow: p99={}ns",
        result.statistics.p99
    );
}

#[test]
fn test_warm_path_string_format() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);

    let result = harness.measure("string_format", OperationType::WarmPath, || {
        format!("Task_{}", 42)
    });

    assert!(
        !result.bounds_violated,
        "String format too slow: p99={}ns",
        result.statistics.p99
    );
}

#[test]
fn test_warm_path_hash_insert() {
    use std::collections::HashMap;
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);
    let mut map = HashMap::new();
    let mut counter = 0;

    let result = harness.measure("hash_insert", OperationType::WarmPath, || {
        counter += 1;
        map.insert(counter, counter * 2);
    });

    assert!(
        !result.bounds_violated,
        "HashMap insert too slow: p99={}ns",
        result.statistics.p99
    );
}

#[test]
fn test_warm_path_small_iteration() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    let result = harness.measure("small_iter", OperationType::WarmPath, || {
        data.iter().filter(|&&x| x > 5).count()
    });

    assert!(
        !result.bounds_violated,
        "Small iteration too slow: p99={}ns",
        result.statistics.p99
    );
}

#[test]
fn test_warm_path_json_parse_small() {
    let mut harness = PerformanceHarness::with_iterations(100, 1000, 10);
    let json_str = r#"{"id": 42, "name": "test", "active": true}"#;

    let result = harness.measure("json_parse", OperationType::WarmPath, || {
        serde_json::from_str::<serde_json::Value>(json_str).unwrap()
    });

    assert!(
        !result.bounds_violated,
        "Small JSON parse too slow: p99={}ns",
        result.statistics.p99
    );
}
