// rust/knhk-etl/tests/chicago_tdd_runtime_class.rs
// Chicago TDD tests for Runtime Class
// Focus: Behavior verification using AAA pattern (Arrange, Act, Assert)

use knhk_etl::runtime_class::RuntimeClass;

#[test]
fn test_runtime_class_r1_operations() {
    // Arrange: R1 operations (ASK_SP, COUNT_SP_GE, etc.)
    let operations = vec!["ASK_SP", "COUNT_SP_GE", "COUNT_SP_EQ", "COMPARE_O_EQ"];

    // Act & Assert: All R1 operations classify correctly
    for op in operations {
        let class = RuntimeClass::classify_operation(op, 5).expect("Should classify operation");
        assert_eq!(class, RuntimeClass::R1, "Operation {} should be R1", op);
    }
}

#[test]
fn test_runtime_class_w1_operations() {
    // Arrange: W1 operations (CONSTRUCT8, etc.)

    // Act: Classify CONSTRUCT8
    let class =
        RuntimeClass::classify_operation("CONSTRUCT8", 5).expect("Should classify operation");

    // Assert: CONSTRUCT8 is W1
    assert_eq!(class, RuntimeClass::W1);
}

#[test]
fn test_runtime_class_data_size_limit() {
    // Arrange: R1 operation with different data sizes

    // Act: Classify with size <= 8 (R1)
    let r1_class = RuntimeClass::classify_operation("ASK_SP", 8).expect("Should classify");

    // Act: Classify with size > 8 (exceeds R1 limit, may fail or return C1)
    let r1_class_large = RuntimeClass::classify_operation("ASK_SP", 9);

    // Assert: Size <= 8 is R1, size > 8 may fail or be C1
    assert_eq!(r1_class, RuntimeClass::R1);
    // Size > 8 for R1 operation may fail classification or return C1
    if let Ok(class) = r1_class_large {
        // If it succeeds, it should be C1 (exceeds R1 limit)
        assert_eq!(class, RuntimeClass::C1);
    }
    // If it fails, that's also acceptable (size exceeds R1 limit)
}
