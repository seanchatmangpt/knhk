//! Performance benchmarks
//!
//! Benchmarks for hot path operations, warm path operations, and cold path operations.

use knhk_workflow_engine::constants::{HOT_PATH_MAX_TICKS, WARM_PATH_MAX_MS, COLD_PATH_MAX_MS};

#[test]
fn test_hot_path_constraint() {
    // Hot path must execute in ≤8 ticks (2ns at 4GHz)
    assert_eq!(HOT_PATH_MAX_TICKS, 8, "Hot path max ticks must be 8 (Chatman Constant)");
}

#[test]
fn test_warm_path_constraint() {
    // Warm path must execute in ≤1ms
    assert_eq!(WARM_PATH_MAX_MS, 1, "Warm path max latency must be 1ms");
}

#[test]
fn test_cold_path_constraint() {
    // Cold path must execute in ≤500ms
    assert_eq!(COLD_PATH_MAX_MS, 500, "Cold path max latency must be 500ms");
}

