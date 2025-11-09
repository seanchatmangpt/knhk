//! Hot path performance tests
//!
//! Verifies all hot path operations execute in ≤8 ticks (Chatman Constant: 2ns at 4GHz).
//!
//! **CRITICAL GAP IDENTIFIED**: Previous tests were placeholders that simulated operations
//! instead of calling actual hot path code. These tests now call real `knhk-hot` operations
//! and measure with RDTSC for cycle-accurate validation.

use knhk_workflow_engine::constants::HOT_PATH_MAX_TICKS;
use knhk_hot::cycle_counter::{read_cycles, cycles_to_ticks};
use knhk_hot::kernels::{KernelExecutor, KernelType};
use chicago_tdd_tools::{assert_within_tick_budget, assert_ok};

/// Test ASK_SP operation with actual hot path code
///
/// **GAP FIXED**: Now calls actual `KernelExecutor::execute()` instead of simulating.
#[test]
fn test_hot_path_ask_operation() {
    // Arrange: Create SoA arrays with ≤8 triples (MAX_RUN_LEN constraint)
    let n_rows = 8;
    let s_lane: [u64; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let p_lane: [u64; 8] = [10, 10, 10, 10, 10, 10, 10, 10]; // Same predicate
    let o_lane: [u64; 8] = [100, 200, 300, 400, 500, 600, 700, 800];

    // Act: Execute actual ASK_SP operation and measure with RDTSC
    let result = KernelExecutor::execute(
        KernelType::AskSp,
        &s_lane[..n_rows],
        &p_lane[..n_rows],
        &o_lane[..n_rows],
        n_rows,
    );
    
    // Assert: Operation succeeds and completes within 8 ticks
    assert_ok!(&result, "ASK_SP operation should succeed");
    let (cycles, _mask) = result.unwrap();
    let ticks = cycles_to_ticks(cycles);
    
    assert_within_tick_budget!(ticks, "ASK_SP operation");
}

/// Test COUNT_SP_GE operation with actual hot path code
///
/// **GAP FIXED**: Now calls actual `KernelExecutor::execute()` instead of simulating.
#[test]
fn test_hot_path_count_operation() {
    // Arrange: Create SoA arrays with ≤8 triples
    let n_rows = 5;
    let s_lane: [u64; 8] = [1, 2, 3, 4, 5, 0, 0, 0];
    let p_lane: [u64; 8] = [10, 10, 10, 10, 10, 0, 0, 0]; // Same predicate
    let o_lane: [u64; 8] = [100, 200, 300, 400, 500, 0, 0, 0];

    // Act: Execute actual COUNT_SP_GE operation and measure with RDTSC
    let result = KernelExecutor::execute(
        KernelType::CountSpGe,
        &s_lane[..n_rows],
        &p_lane[..n_rows],
        &o_lane[..n_rows],
        n_rows,
    );
    
    // Assert: Operation succeeds and completes within 8 ticks
    assert_ok!(&result, "COUNT_SP_GE operation should succeed");
    let (cycles, _mask) = result.unwrap();
    let ticks = cycles_to_ticks(cycles);
    
    assert_within_tick_budget!(ticks, "COUNT_SP_GE operation");
}

/// Test ASK_SPO operation with actual hot path code
///
/// **GAP FIXED**: Now calls actual `KernelExecutor::execute()` instead of simulating.
#[test]
fn test_hot_path_ask_spo_operation() {
    // Arrange: Create SoA arrays with ≤8 triples
    let n_rows = 3;
    let s_lane: [u64; 8] = [1, 2, 3, 0, 0, 0, 0, 0];
    let p_lane: [u64; 8] = [10, 20, 30, 0, 0, 0, 0, 0];
    let o_lane: [u64; 8] = [100, 200, 300, 0, 0, 0, 0, 0];

    // Act: Execute actual ASK_SPO operation and measure with RDTSC
    let result = KernelExecutor::execute(
        KernelType::AskSpo,
        &s_lane[..n_rows],
        &p_lane[..n_rows],
        &o_lane[..n_rows],
        n_rows,
    );
    
    // Assert: Operation succeeds and completes within 8 ticks
    assert_ok!(&result, "ASK_SPO operation should succeed");
    let (cycles, _mask) = result.unwrap();
    let ticks = cycles_to_ticks(cycles);
    
    assert_within_tick_budget!(ticks, "ASK_SPO operation");
}

/// Test COMPARE_O operation with actual hot path code
///
/// **GAP FIXED**: Now calls actual `KernelExecutor::execute()` instead of simulating.
#[test]
fn test_hot_path_compare_operation() {
    // Arrange: Create SoA arrays with ≤8 triples
    let n_rows = 4;
    let s_lane: [u64; 8] = [1, 2, 3, 4, 0, 0, 0, 0];
    let p_lane: [u64; 8] = [10, 10, 10, 10, 0, 0, 0, 0];
    let o_lane: [u64; 8] = [100, 200, 300, 400, 0, 0, 0, 0];

    // Act: Execute actual COMPARE_O operation and measure with RDTSC
    let result = KernelExecutor::execute(
        KernelType::CompareO,
        &s_lane[..n_rows],
        &p_lane[..n_rows],
        &o_lane[..n_rows],
        n_rows,
    );
    
    // Assert: Operation succeeds and completes within 8 ticks
    assert_ok!(&result, "COMPARE_O operation should succeed");
    let (cycles, _mask) = result.unwrap();
    let ticks = cycles_to_ticks(cycles);
    
    assert_within_tick_budget!(ticks, "COMPARE_O operation");
}

