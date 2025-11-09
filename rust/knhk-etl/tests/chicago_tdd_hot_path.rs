// rust/knhk-etl/tests/chicago_tdd_hot_path.rs
// Chicago TDD tests for HotPathEngine public APIs
// Focus: Behavior verification using AAA pattern (Arrange, Act, Assert)
// Real collaborators, state-based verification, no mocks

extern crate alloc;

use knhk_etl::{HotPathEngine, PipelineError};

/// Test: HotPathEngine creation with default capacity
/// Chicago TDD: Verify state after creation (capacity = 8)
#[test]
fn test_hot_path_engine_new_default_capacity() {
    // Arrange: No setup needed

    // Act: Create engine with default capacity
    let engine = HotPathEngine::new();

    // Assert: Verify default capacity is 8 (Chatman Constant)
    assert_eq!(engine.current_capacity(), 8);
    assert_eq!(engine.max_capacity(), 8);
}

/// Test: HotPathEngine creation with custom capacity
/// Chicago TDD: Verify state after creation with valid capacity
#[test]
fn test_hot_path_engine_with_max_capacity_valid() {
    // Arrange: Valid capacity values
    let valid_capacities = vec![1, 4, 8];

    for capacity in valid_capacities {
        // Act: Create engine with custom capacity
        let engine = HotPathEngine::with_max_capacity(capacity).unwrap();

        // Assert: Verify capacity is set correctly
        assert_eq!(engine.current_capacity(), capacity);
        assert_eq!(engine.max_capacity(), capacity);
    }
}

/// Test: HotPathEngine creation with invalid capacity (guard violation)
/// Chicago TDD: Verify error path when capacity exceeds MAX_RUN_LEN
#[test]
fn test_hot_path_engine_with_max_capacity_guard_violation() {
    // Arrange: Invalid capacity values (> 8)
    let invalid_capacities = vec![9, 10, 100];

    for capacity in invalid_capacities {
        // Act: Try to create engine with invalid capacity
        let result = HotPathEngine::with_max_capacity(capacity);

        // Assert: Verify guard violation error
        assert!(result.is_err());
        if let Err(PipelineError::GuardViolation(msg)) = result {
            assert!(msg.contains("max_run_len") || msg.contains("8"));
        } else {
            panic!("Expected GuardViolation error, got: {:?}", result.err());
        }
    }
}

/// Test: HotPathEngine load_triples with valid input
/// Chicago TDD: Verify state after loading triples
#[test]
fn test_hot_path_engine_load_triples_valid() {
    // Arrange: Create engine and valid triples
    let mut engine = HotPathEngine::new();
    let triples = vec![(1, 100, 1000), (2, 100, 2000), (3, 100, 3000)];

    // Act: Load triples
    let buffers = engine.load_triples(&triples).unwrap();

    // Assert: Verify triples loaded correctly
    assert_eq!(buffers.s[0], 1);
    assert_eq!(buffers.p[0], 100);
    assert_eq!(buffers.o[0], 1000);
    assert_eq!(buffers.s[1], 2);
    assert_eq!(buffers.p[1], 100);
    assert_eq!(buffers.o[1], 2000);
    assert_eq!(buffers.s[2], 3);
    assert_eq!(buffers.p[2], 100);
    assert_eq!(buffers.o[2], 3000);
}

/// Test: HotPathEngine load_triples with guard violation
/// Chicago TDD: Verify error path when triple count exceeds capacity
#[test]
fn test_hot_path_engine_load_triples_guard_violation() {
    // Arrange: Create engine with capacity 4 and triples exceeding capacity
    let mut engine = HotPathEngine::with_max_capacity(4).unwrap();
    let triples = vec![(1, 100, 1000); 5]; // 5 triples > 4 capacity

    // Act: Try to load triples
    let result = engine.load_triples(&triples);

    // Assert: Verify guard violation error
    assert!(result.is_err());
    if let Err(PipelineError::GuardViolation(msg)) = result {
        assert!(msg.contains("exceeds") || msg.contains("capacity"));
    } else {
        panic!("Expected GuardViolation error, got: {:?}", result);
    }
}

/// Test: HotPathEngine buffer reuse
/// Chicago TDD: Verify buffers are reused across operations (memory efficiency)
#[test]
fn test_hot_path_engine_buffer_reuse() {
    // Arrange: Create engine
    let mut engine = HotPathEngine::new();

    // Act: Load first set of triples
    let triples1 = vec![(1, 100, 1000)];
    let buffers1 = engine.load_triples(&triples1).unwrap();
    let buffer1_ptr = buffers1 as *const _ as usize;

    // Drop first reference
    drop(buffers1);

    // Act: Load second set of triples (should reuse buffers)
    let triples2 = vec![(2, 200, 2000)];
    let buffers2 = engine.load_triples(&triples2).unwrap();
    let buffer2_ptr = buffers2 as *const _ as usize;

    // Assert: Verify buffers were reused (same memory location)
    assert_eq!(buffer1_ptr, buffer2_ptr);
    assert_eq!(buffers2.s[0], 2);
    assert_eq!(buffers2.p[0], 200);
    assert_eq!(buffers2.o[0], 2000);
}

/// Test: HotPathEngine set_max_capacity with valid value
/// Chicago TDD: Verify state change after setting capacity
#[test]
fn test_hot_path_engine_set_max_capacity_valid() {
    // Arrange: Create engine with default capacity
    let mut engine = HotPathEngine::new();
    assert_eq!(engine.max_capacity(), 8);

    // Act: Set capacity to 4
    engine.set_max_capacity(4).unwrap();

    // Assert: Verify capacity changed
    assert_eq!(engine.max_capacity(), 4);
    assert_eq!(engine.current_capacity(), 4);
}

/// Test: HotPathEngine set_max_capacity with guard violation
/// Chicago TDD: Verify error path when capacity exceeds MAX_RUN_LEN
#[test]
fn test_hot_path_engine_set_max_capacity_guard_violation() {
    // Arrange: Create engine
    let mut engine = HotPathEngine::new();

    // Act: Try to set invalid capacity
    let result = engine.set_max_capacity(9);

    // Assert: Verify guard violation error
    assert!(result.is_err());
    if let Err(PipelineError::GuardViolation(msg)) = result {
        assert!(msg.contains("max_run_len") || msg.contains("8"));
    } else {
        panic!("Expected GuardViolation error, got: {:?}", result);
    }

    // Assert: Verify capacity unchanged
    assert_eq!(engine.max_capacity(), 8);
}

/// Test: HotPathEngine clear buffers
/// Chicago TDD: Verify state after clearing buffers
#[test]
fn test_hot_path_engine_clear_buffers() {
    // Arrange: Create engine and load triples
    let mut engine = HotPathEngine::new();
    let triples = vec![(1, 100, 1000), (2, 200, 2000)];
    engine.load_triples(&triples).unwrap();

    // Act: Clear buffers
    engine.clear();

    // Assert: Verify buffers are zeroed
    let buffers = engine.get_buffers();
    assert_eq!(buffers.s[0], 0);
    assert_eq!(buffers.p[0], 0);
    assert_eq!(buffers.o[0], 0);
    assert_eq!(buffers.s[1], 0);
}

/// Test: HotPathEngine get_buffers_mut
/// Chicago TDD: Verify mutable access to buffers
#[test]
fn test_hot_path_engine_get_buffers_mut() {
    // Arrange: Create engine
    let mut engine = HotPathEngine::new();

    // Act: Get mutable reference and modify buffers
    let buffers = engine.get_buffers_mut();
    buffers.s[0] = 42;
    buffers.p[0] = 100;
    buffers.o[0] = 1000;

    // Assert: Verify modifications persisted
    let buffers_immut = engine.get_buffers();
    assert_eq!(buffers_immut.s[0], 42);
    assert_eq!(buffers_immut.p[0], 100);
    assert_eq!(buffers_immut.o[0], 1000);
}

/// Test: HotPathEngine load_triples with maximum capacity (8 triples)
/// Chicago TDD: Verify boundary condition (exactly 8 triples)
#[test]
fn test_hot_path_engine_load_triples_max_capacity() {
    // Arrange: Create engine and exactly 8 triples
    let mut engine = HotPathEngine::new();
    let triples: Vec<(u64, u64, u64)> = (1..=8).map(|i| (i, i * 100, i * 1000)).collect();

    // Act: Load triples
    let buffers = engine.load_triples(&triples).unwrap();

    // Assert: Verify all 8 triples loaded
    for i in 0..8 {
        assert_eq!(buffers.s[i], (i + 1) as u64);
        assert_eq!(buffers.p[i], ((i + 1) * 100) as u64);
        assert_eq!(buffers.o[i], ((i + 1) * 1000) as u64);
    }
}

/// Test: HotPathEngine load_triples with empty input
/// Chicago TDD: Verify behavior with empty triples
#[test]
fn test_hot_path_engine_load_triples_empty() {
    // Arrange: Create engine and empty triples
    let mut engine = HotPathEngine::new();
    let triples = vec![];

    // Act: Load empty triples
    let buffers = engine.load_triples(&triples).unwrap();

    // Assert: Verify buffers are cleared (all zeros)
    for i in 0..8 {
        assert_eq!(buffers.s[i], 0);
        assert_eq!(buffers.p[i], 0);
        assert_eq!(buffers.o[i], 0);
    }
}

/// Test: HotPathEngine capacity reduction
/// Chicago TDD: Verify current_capacity adjusts when max_capacity is reduced
#[test]
fn test_hot_path_engine_capacity_reduction() {
    // Arrange: Create engine with capacity 8
    let mut engine = HotPathEngine::new();
    assert_eq!(engine.current_capacity(), 8);
    assert_eq!(engine.max_capacity(), 8);

    // Act: Reduce capacity to 4
    engine.set_max_capacity(4).unwrap();

    // Assert: Verify current_capacity adjusted
    assert_eq!(engine.current_capacity(), 4);
    assert_eq!(engine.max_capacity(), 4);
}

/// Test: HotPathEngine error context preservation
/// Chicago TDD: Verify error messages contain context
#[test]
fn test_hot_path_engine_error_context() {
    // Arrange: Create engine with capacity 4
    let mut engine = HotPathEngine::with_max_capacity(4).unwrap();
    let triples = vec![(1, 100, 1000); 5];

    // Act: Try to load triples exceeding capacity
    let result = engine.load_triples(&triples);

    // Assert: Verify error message contains useful context
    assert!(result.is_err());
    if let Err(PipelineError::GuardViolation(msg)) = result {
        assert!(msg.contains("5") || msg.contains("exceeds"));
        assert!(msg.contains("4") || msg.contains("capacity"));
    } else {
        panic!("Expected GuardViolation error with context");
    }
}
