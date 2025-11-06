// rust/knhk-etl/tests/runtime_class_test.rs
// Tests for runtime class classification

use knhk_etl::runtime_class::RuntimeClass;

#[test]
fn test_r1_classification_ask() {
    assert_eq!(
        RuntimeClass::classify_operation("ASK_SP", 5).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("ASK_SPO", 8).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("ASK_OP", 3).unwrap(),
        RuntimeClass::R1
    );
}

#[test]
fn test_r1_classification_count() {
    assert_eq!(
        RuntimeClass::classify_operation("COUNT_SP_GE", 5).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("COUNT_SP_LE", 8).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("COUNT_SP_EQ", 1).unwrap(),
        RuntimeClass::R1
    );
}

#[test]
fn test_r1_classification_compare() {
    assert_eq!(
        RuntimeClass::classify_operation("COMPARE_O_EQ", 5).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("COMPARE_O_GT", 3).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("COMPARE_O_LT", 7).unwrap(),
        RuntimeClass::R1
    );
}

#[test]
fn test_r1_classification_validate() {
    assert_eq!(
        RuntimeClass::classify_operation("VALIDATE_DATATYPE_SP", 1).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("VALIDATE_DATATYPE_SPO", 5).unwrap(),
        RuntimeClass::R1
    );
    assert_eq!(
        RuntimeClass::classify_operation("UNIQUE_SP", 1).unwrap(),
        RuntimeClass::R1
    );
}

#[test]
fn test_r1_data_size_limit() {
    assert_eq!(
        RuntimeClass::classify_operation("ASK_SP", 8).unwrap(),
        RuntimeClass::R1
    );
    // Data size > 8 should not be R1
    assert_ne!(
        RuntimeClass::classify_operation("ASK_SP", 9).unwrap(),
        RuntimeClass::R1
    );
}

#[test]
fn test_w1_classification() {
    assert_eq!(
        RuntimeClass::classify_operation("CONSTRUCT8", 5).unwrap(),
        RuntimeClass::W1
    );
    assert_eq!(
        RuntimeClass::classify_operation("CONSTRUCT8", 10).unwrap(),
        RuntimeClass::W1
    );
    assert_eq!(
        RuntimeClass::classify_operation("PREBIND_TEMPLATE", 100).unwrap(),
        RuntimeClass::W1
    );
    assert_eq!(
        RuntimeClass::classify_operation("AOT_TRANSFORM", 50).unwrap(),
        RuntimeClass::W1
    );
}

#[test]
fn test_c1_classification() {
    assert_eq!(
        RuntimeClass::classify_operation("SPARQL_SELECT", 100).unwrap(),
        RuntimeClass::C1
    );
    assert_eq!(
        RuntimeClass::classify_operation("SHACL_VALIDATE", 50).unwrap(),
        RuntimeClass::C1
    );
    assert_eq!(
        RuntimeClass::classify_operation("JOIN_QUERY", 200).unwrap(),
        RuntimeClass::C1
    );
    assert_eq!(
        RuntimeClass::classify_operation("ANALYTICS_QUERY", 1000).unwrap(),
        RuntimeClass::C1
    );
}

#[test]
fn test_metadata_r1() {
    let meta = RuntimeClass::R1.metadata();
    assert_eq!(meta.budget_ns, 8);
    assert_eq!(meta.slo_p99_ns, 2);
    assert_eq!(meta.operation_type, "R1 Hot");
}

#[test]
fn test_metadata_w1() {
    let meta = RuntimeClass::W1.metadata();
    assert_eq!(meta.budget_ns, 500_000);
    assert_eq!(meta.slo_p99_ns, 1_000_000);
    assert_eq!(meta.operation_type, "W1 Warm");
}

#[test]
fn test_metadata_c1() {
    let meta = RuntimeClass::C1.metadata();
    assert_eq!(meta.budget_ns, 200_000_000);
    assert_eq!(meta.slo_p99_ns, 500_000_000);
    assert_eq!(meta.operation_type, "C1 Cold");
}

#[test]
fn test_invalid_classification() {
    let result = RuntimeClass::classify_operation("UNKNOWN_OP", 0);
    assert!(result.is_err());
}

