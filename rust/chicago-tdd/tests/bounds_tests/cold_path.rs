//! Cold path tests
//!
//! Cold path operations have no hard bound (diagnostic only).

use chicago_tdd::{PerformanceHarness, OperationType};

#[test]
fn test_cold_path_large_allocation() {
    let mut harness = PerformanceHarness::with_iterations(10, 100, 10);

    let result = harness.measure("large_vec_alloc", OperationType::ColdPath, || {
        let v: Vec<u64> = Vec::with_capacity(10000);
        v
    });

    // Cold path - just measure, don't enforce
    println!("Large allocation: p99={}ns", result.statistics.p99);
    assert!(!result.bounds_violated); // Cold path never violates
}

#[test]
fn test_cold_path_file_metadata() {
    let mut harness = PerformanceHarness::with_iterations(10, 100, 10);

    let result = harness.measure("file_metadata", OperationType::ColdPath, || {
        std::fs::metadata("/tmp").ok()
    });

    println!("File metadata: p99={}ns", result.statistics.p99);
    assert!(!result.bounds_violated);
}

#[test]
fn test_cold_path_thread_spawn() {
    let mut harness = PerformanceHarness::with_iterations(10, 100, 10);

    let result = harness.measure("thread_spawn", OperationType::ColdPath, || {
        std::thread::spawn(|| {
            // Minimal work
            42
        }).join().unwrap()
    });

    println!("Thread spawn: p99={}ns", result.statistics.p99);
    assert!(!result.bounds_violated);
}

#[test]
fn test_cold_path_large_json_parse() {
    let mut harness = PerformanceHarness::with_iterations(10, 100, 10);
    let json_str = r#"{
        "workflow": "complex",
        "tasks": [
            {"id": 1, "name": "task1", "priority": 100},
            {"id": 2, "name": "task2", "priority": 90},
            {"id": 3, "name": "task3", "priority": 80}
        ],
        "variables": {
            "x": 42,
            "y": 58,
            "z": 100
        }
    }"#;

    let result = harness.measure("large_json_parse", OperationType::ColdPath, || {
        serde_json::from_str::<serde_json::Value>(json_str).unwrap()
    });

    println!("Large JSON parse: p99={}ns", result.statistics.p99);
    assert!(!result.bounds_violated);
}
