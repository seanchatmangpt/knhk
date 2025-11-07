// rust/knhk-etl/tests/chicago_tdd_pipeline.rs
// Chicago TDD tests for Pipeline and ETL Stages
// Focus: Behavior verification using AAA pattern (Arrange, Act, Assert)

extern crate alloc;

use knhk_etl::*;
use alloc::vec::Vec;
use alloc::string::ToString;

#[test]
fn test_pipeline_creation() {
    // Arrange & Act: Create pipeline
    let pipeline = Pipeline::new(
        vec!["kafka_connector".to_string()],
        "urn:knhk:schema:test".to_string(),
        true,
        vec!["https://webhook.example.com".to_string()],
    );
    
    // Assert: Pipeline created with correct configuration
    assert_eq!(pipeline.load.max_run_len, 8);
    assert_eq!(pipeline.reflex.tick_budget, 8);
}

#[test]
fn test_load_stage_guard_enforcement() {
    // Arrange: Create load stage and transform result exceeding max_run_len
    let load = LoadStage::new();
    let transform_result = TransformResult {
        typed_triples: vec![
            TypedTriple {
                subject: 1,
                predicate: 2,
                object: 3,
                graph: None,
            }; 10  // Exceeds max_run_len (8)
        ],
        validation_errors: Vec::new(),
    };
    
    // Act: Try to load
    let result = load.load(transform_result);
    
    // Assert: Load fails due to guard violation
    assert!(result.is_err());
    if let Err(PipelineError::GuardViolation(msg)) = result {
        assert!(msg.contains("max_run_len") || msg.contains("8"));
    } else {
        panic!("Expected GuardViolation error");
    }
}

#[test]
fn test_load_stage_predicate_grouping() {
    // Arrange: Create load stage with triples having different predicates
    let load = LoadStage::new();
    let transform_result = TransformResult {
        typed_triples: vec![
            TypedTriple { subject: 1, predicate: 100, object: 10, graph: None },
            TypedTriple { subject: 2, predicate: 100, object: 20, graph: None },
            TypedTriple { subject: 3, predicate: 200, object: 30, graph: None },
        ],
        validation_errors: Vec::new(),
    };
    
    // Act: Load triples
    let result = load.load(transform_result);
    
    // Assert: Triples grouped by predicate into runs
    assert!(result.is_ok());
    let load_result = result.unwrap();
    assert_eq!(load_result.runs.len(), 2); // Two different predicates
    assert_eq!(load_result.runs[0].pred, 100);
    assert_eq!(load_result.runs[0].len, 2);
    assert_eq!(load_result.runs[1].pred, 200);
    assert_eq!(load_result.runs[1].len, 1);
}

#[test]
fn test_reflex_stage_tick_budget_enforcement() {
    // Arrange: Create reflex stage and load result
    let reflex = ReflexStage::new();
    
    let mut soa = SoAArrays::new();
    soa.s[0] = 1;
    soa.p[0] = 100;
    soa.o[0] = 10;
    
    let run = PredRun { pred: 100, off: 0, len: 1 };
    
    let load_result = LoadResult {
        soa_arrays: soa,
        runs: vec![run],
    };
    
    // Act: Execute reflex
    let result = reflex.reflex(load_result);
    
    // Assert: Reflex completes within tick budget
    assert!(result.is_ok());
    let reflex_result = result.unwrap();
    assert!(reflex_result.max_ticks <= 8);
    assert!(!reflex_result.receipts.is_empty());
}

#[test]
fn test_reflex_stage_receipt_generation() {
    // Arrange: Create reflex stage and load result
    let reflex = ReflexStage::new();
    
    let mut soa = SoAArrays::new();
    soa.s[0] = 1;
    soa.p[0] = 100;
    soa.o[0] = 10;
    
    let run = PredRun { pred: 100, off: 0, len: 1 };
    
    let load_result = LoadResult {
        soa_arrays: soa,
        runs: vec![run],
    };
    
    // Act: Execute reflex
    let result = reflex.reflex(load_result);
    
    // Assert: Receipts generated with required fields
    assert!(result.is_ok());
    let reflex_result = result.unwrap();
    assert!(!reflex_result.receipts.is_empty());
    
    let receipt = &reflex_result.receipts[0];
    assert!(!receipt.id.is_empty());
    assert!(receipt.ticks <= 8); // Within budget
    assert!(receipt.lanes > 0);
}

#[test]
fn test_receipt_merging() {
    // Arrange: Create two receipts
    let receipt1 = Receipt {
        id: "r1".to_string(),
        cycle_id: 1,
        shard_id: 1,
        hook_id: 1,
        ticks: 4,
        actual_ticks: 3,
        lanes: 8,
        span_id: 0x1234,
        a_hash: 0xABCD,
    };
    
    let receipt2 = Receipt {
        id: "r2".to_string(),
        cycle_id: 2,
        shard_id: 2,
        hook_id: 2,
        ticks: 6,
        actual_ticks: 5,
        lanes: 8,
        span_id: 0x5678,
        a_hash: 0xEF00,
    };
    
    // Act: Merge receipts
    let merged = ReflexStage::merge_receipts(&[receipt1, receipt2]);
    
    // Assert: Merged receipt has correct values
    assert_eq!(merged.ticks, 6); // Max ticks
    assert_eq!(merged.lanes, 16); // Sum lanes
    assert_eq!(merged.span_id, 0x1234 ^ 0x5678); // XOR merge
    assert_eq!(merged.a_hash, 0xABCD ^ 0xEF00); // XOR merge
}


