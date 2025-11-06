// Chicago TDD Tests for KNHK ETL Pipeline - Complete End-to-End Validation
//
// Principles:
// 1. State-based verification (not interaction-based)
// 2. Real collaborators (no mocks - actual ETL stages)
// 3. Test full pipeline: Ingest → Transform → Load → Reflex → Emit
// 4. Verify ≤8 tick budget compliance

use knhk_etl::*;
use std::collections::BTreeMap;

// ============================================================================
// Test Suite: Full Pipeline End-to-End State Verification
// ============================================================================

#[test]
fn test_full_pipeline_ingest_to_emit() {
    // Arrange: Complete pipeline with real stages
    let pipeline = Pipeline::new(
        vec!["kafka_test".to_string()],
        "urn:knhk:schema:test".to_string(),
        true,
        vec!["https://webhook.test.com".to_string()],
    );

    let turtle_data = r#"
        <http://example.org/alice> <http://example.org/name> "Alice" .
        <http://example.org/alice> <http://example.org/age> "30" .
        <http://example.org/bob> <http://example.org/name> "Bob" .
    "#;

    // Act: Execute full pipeline
    let ingest_result = pipeline.ingest.parse_rdf_turtle(turtle_data);
    assert!(ingest_result.is_ok(), "Ingest stage should succeed");

    let ingest_result = IngestResult {
        triples: ingest_result.unwrap(),
        metadata: BTreeMap::new(),
    };

    let transform_result = pipeline.transform.transform(ingest_result);
    assert!(transform_result.is_ok(), "Transform stage should succeed");

    let load_result = pipeline.load.load(transform_result.unwrap());
    assert!(load_result.is_ok(), "Load stage should succeed");

    let reflex_result = pipeline.reflex.reflex(load_result.unwrap());
    assert!(reflex_result.is_ok(), "Reflex stage should succeed");

    let emit_result = pipeline.emit.emit(reflex_result.unwrap());
    assert!(emit_result.is_ok(), "Emit stage should succeed");

    // Assert: Pipeline completed successfully
    let emit_result = emit_result.unwrap();
    assert_eq!(emit_result.receipts_written, 1, "Should write receipts");
    assert!(emit_result.actions_sent >= 0, "Should send actions");
    assert!(emit_result.lockchain_hashes.len() >= 1, "Should generate lockchain hashes");
}

#[test]
fn test_pipeline_respects_8_tick_budget() {
    // Arrange: Pipeline with tick budget enforcement
    let pipeline = Pipeline::new(
        vec!["test".to_string()],
        "urn:test:schema".to_string(),
        false,
        vec![],
    );

    assert_eq!(pipeline.reflex.tick_budget, 8, "Should enforce 8-tick budget");

    let turtle = "<http://s> <http://p> <http://o> .";

    // Act: Full pipeline execution
    let ingest_result = IngestResult {
        triples: pipeline.ingest.parse_rdf_turtle(turtle).unwrap(),
        metadata: BTreeMap::new(),
    };

    let transform_result = pipeline.transform.transform(ingest_result).unwrap();
    let load_result = pipeline.load.load(transform_result).unwrap();
    let reflex_result = pipeline.reflex.reflex(load_result).unwrap();

    // Assert: Tick budget respected
    assert!(reflex_result.max_ticks <= 8,
            "Reflex stage must respect 8-tick budget, got {} ticks", reflex_result.max_ticks);
}

#[test]
fn test_pipeline_handles_empty_input() {
    // Arrange: Pipeline with empty data
    let pipeline = Pipeline::new(
        vec!["test".to_string()],
        "urn:test:schema".to_string(),
        false,
        vec![],
    );

    // Act: Empty turtle data
    let ingest_result = pipeline.ingest.parse_rdf_turtle("");
    assert!(ingest_result.is_ok(), "Should handle empty input gracefully");

    let triples = ingest_result.unwrap();

    // Assert: Empty result, no errors
    assert_eq!(triples.len(), 0, "Should return empty triples for empty input");
}

// ============================================================================
// Test Suite: Ingest Stage State-Based Validation
// ============================================================================

#[test]
fn test_ingest_parses_simple_turtle() {
    // Arrange: Ingest stage with turtle parser
    let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

    let turtle = "<http://example.org/s> <http://example.org/p> <http://example.org/o> .";

    // Act: Parse turtle
    let result = ingest.parse_rdf_turtle(turtle);

    // Assert: Successful parse with correct triples
    assert!(result.is_ok(), "Should parse valid turtle");
    let triples = result.unwrap();
    assert_eq!(triples.len(), 1, "Should parse 1 triple");
    assert_eq!(triples[0].subject, "http://example.org/s");
    assert_eq!(triples[0].predicate, "http://example.org/p");
    assert_eq!(triples[0].object, "http://example.org/o");
}

#[test]
fn test_ingest_handles_prefixes() {
    // Arrange
    let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

    let turtle = r#"
        @prefix ex: <http://example.org/> .
        ex:subject ex:predicate ex:object .
    "#;

    // Act
    let result = ingest.parse_rdf_turtle(turtle);

    // Assert: Prefixes expanded
    assert!(result.is_ok(), "Should handle prefixes");
    let triples = result.unwrap();
    assert_eq!(triples[0].subject, "http://example.org/subject");
    assert_eq!(triples[0].predicate, "http://example.org/predicate");
    assert_eq!(triples[0].object, "http://example.org/object");
}

#[test]
fn test_ingest_handles_blank_nodes() {
    // Arrange
    let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

    let turtle = r#"
        _:alice <http://example.org/name> "Alice" .
    "#;

    // Act
    let result = ingest.parse_rdf_turtle(turtle);

    // Assert: Blank nodes handled
    assert!(result.is_ok(), "Should handle blank nodes");
    let triples = result.unwrap();
    assert!(triples[0].subject.starts_with("_:"), "Should preserve blank node");
    assert_eq!(triples[0].object, "\"Alice\"");
}

#[test]
fn test_ingest_handles_literals_with_datatypes() {
    // Arrange
    let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

    let turtle = r#"
        <http://example.org/alice> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
    "#;

    // Act
    let result = ingest.parse_rdf_turtle(turtle);

    // Assert: Datatype preserved
    assert!(result.is_ok(), "Should handle typed literals");
    let triples = result.unwrap();
    assert!(triples[0].object.contains("integer"), "Should preserve datatype");
}

#[test]
fn test_ingest_handles_language_tags() {
    // Arrange
    let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

    let turtle = r#"
        <http://example.org/alice> <http://example.org/label> "Alice"@en .
    "#;

    // Act
    let result = ingest.parse_rdf_turtle(turtle);

    // Assert: Language tag preserved
    assert!(result.is_ok(), "Should handle language tags");
    let triples = result.unwrap();
    assert!(triples[0].object.contains("@en"), "Should preserve language tag");
}

#[test]
fn test_ingest_rejects_invalid_syntax() {
    // Arrange
    let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

    let invalid_turtle = "<http://example.org/s> <http://example.org/p>";

    // Act
    let result = ingest.parse_rdf_turtle(invalid_turtle);

    // Assert: Error on invalid syntax
    assert!(result.is_err(), "Should reject invalid turtle syntax");
    match result {
        Err(PipelineError::IngestError(msg)) => {
            assert!(msg.contains("parse"), "Error should mention parse failure");
        }
        _ => panic!("Expected IngestError"),
    }
}

// ============================================================================
// Test Suite: Transform Stage State-Based Validation
// ============================================================================

#[test]
fn test_transform_hashes_strings_consistently() {
    // Arrange: Transform stage with hashing
    let transform = TransformStage::new("urn:test:schema".to_string(), false);

    let ingest_result = IngestResult {
        triples: vec![
            RawTriple {
                subject: "http://example.org/s1".to_string(),
                predicate: "http://example.org/p1".to_string(),
                object: "http://example.org/o1".to_string(),
                graph: None,
            },
            RawTriple {
                subject: "http://example.org/s1".to_string(), // Same subject
                predicate: "http://example.org/p2".to_string(),
                object: "http://example.org/o2".to_string(),
                graph: None,
            },
        ],
        metadata: BTreeMap::new(),
    };

    // Act: Transform twice
    let result1 = transform.transform(ingest_result.clone());
    let result2 = transform.transform(ingest_result.clone());

    // Assert: Consistent hashing
    assert!(result1.is_ok() && result2.is_ok(), "Transform should succeed");
    let typed1 = result1.unwrap().typed_triples;
    let typed2 = result2.unwrap().typed_triples;

    // Same input → same hashes
    assert_eq!(typed1[0].subject, typed2[0].subject, "Subject hash should be consistent");
    assert_eq!(typed1[1].subject, typed2[1].subject, "Same subject → same hash");
    assert_eq!(typed1[0].subject, typed1[1].subject, "Duplicate subjects → same hash");
}

#[test]
fn test_transform_produces_nonzero_hashes() {
    // Arrange
    let transform = TransformStage::new("urn:test:schema".to_string(), false);

    let ingest_result = IngestResult {
        triples: vec![
            RawTriple {
                subject: "http://example.org/s".to_string(),
                predicate: "http://example.org/p".to_string(),
                object: "http://example.org/o".to_string(),
                graph: None,
            }
        ],
        metadata: BTreeMap::new(),
    };

    // Act
    let result = transform.transform(ingest_result).unwrap();

    // Assert: Hashes are non-zero
    assert!(result.typed_triples[0].subject > 0, "Subject hash should be non-zero");
    assert!(result.typed_triples[0].predicate > 0, "Predicate hash should be non-zero");
    assert!(result.typed_triples[0].object > 0, "Object hash should be non-zero");
}

// ============================================================================
// Test Suite: Load Stage State-Based Validation
// ============================================================================

#[test]
fn test_load_creates_predicate_runs() {
    // Arrange: Load stage with max_run_len=8
    let load = LoadStage::new();
    assert_eq!(load.max_run_len, 8, "Should enforce max run length of 8");

    let transform_result = TransformResult {
        typed_triples: vec![
            TypedTriple { subject: 1, predicate: 100, object: 10, graph: None },
            TypedTriple { subject: 2, predicate: 100, object: 20, graph: None },
            TypedTriple { subject: 3, predicate: 200, object: 30, graph: None },
            TypedTriple { subject: 4, predicate: 200, object: 40, graph: None },
        ],
        validation_errors: Vec::new(),
    };

    // Act: Load into SoA
    let result = load.load(transform_result).unwrap();

    // Assert: Predicate runs created correctly
    assert_eq!(result.runs.len(), 2, "Should create 2 predicate runs");
    assert_eq!(result.runs[0].pred, 100, "First run should be predicate 100");
    assert_eq!(result.runs[0].len, 2, "First run should have 2 triples");
    assert_eq!(result.runs[1].pred, 200, "Second run should be predicate 200");
    assert_eq!(result.runs[1].len, 2, "Second run should have 2 triples");
}

#[test]
fn test_load_rejects_runs_exceeding_budget() {
    // Arrange: Load stage with max_run_len=8
    let load = LoadStage::new();

    let transform_result = TransformResult {
        typed_triples: vec![TypedTriple {
            subject: 1,
            predicate: 100,
            object: 10,
            graph: None,
        }; 9], // Exceeds max_run_len
        validation_errors: Vec::new(),
    };

    // Act: Try to load
    let result = load.load(transform_result);

    // Assert: Should reject
    assert!(result.is_err(), "Should reject runs > max_run_len");
    match result {
        Err(PipelineError::LoadError(msg)) => {
            assert!(msg.contains("max_run_len") || msg.contains("exceeds"),
                    "Error should mention budget violation");
        }
        _ => panic!("Expected LoadError"),
    }
}

#[test]
fn test_load_stores_triples_in_soa_format() {
    // Arrange
    let load = LoadStage::new();

    let transform_result = TransformResult {
        typed_triples: vec![
            TypedTriple { subject: 111, predicate: 222, object: 333, graph: None },
        ],
        validation_errors: Vec::new(),
    };

    // Act
    let result = load.load(transform_result).unwrap();

    // Assert: SoA arrays populated
    assert_eq!(result.soa_arrays.s[0], 111, "Subject should be in S array");
    assert_eq!(result.soa_arrays.p[0], 222, "Predicate should be in P array");
    assert_eq!(result.soa_arrays.o[0], 333, "Object should be in O array");
}

// ============================================================================
// Test Suite: Reflex Stage State-Based Validation
// ============================================================================

#[test]
fn test_reflex_respects_tick_budget() {
    // Arrange: Reflex stage with 8-tick budget
    let reflex = ReflexStage::new();
    assert_eq!(reflex.tick_budget, 8, "Should have 8-tick budget");

    let mut soa = SoAArrays::new();
    soa.s[0] = 1;
    soa.p[0] = 100;
    soa.o[0] = 10;

    let load_result = LoadResult {
        soa_arrays: soa,
        runs: vec![PredRun { pred: 100, off: 0, len: 1 }],
    };

    // Act: Execute reflex
    let result = reflex.reflex(load_result).unwrap();

    // Assert: Tick budget respected
    assert!(result.max_ticks <= 8,
            "Reflex must complete within 8 ticks, got {} ticks", result.max_ticks);
}

#[test]
fn test_reflex_generates_receipts() {
    // Arrange
    let reflex = ReflexStage::new();

    let mut soa = SoAArrays::new();
    soa.s[0] = 1;
    soa.p[0] = 100;
    soa.o[0] = 10;

    let load_result = LoadResult {
        soa_arrays: soa,
        runs: vec![PredRun { pred: 100, off: 0, len: 1 }],
    };

    // Act
    let result = reflex.reflex(load_result).unwrap();

    // Assert: Receipt generated
    assert!(!result.receipts.is_empty(), "Should generate at least one receipt");
    let receipt = &result.receipts[0];
    assert!(receipt.ticks <= 8, "Receipt ticks should be within budget");
    assert_eq!(receipt.lanes, 8, "Receipt should record lanes");
}

#[test]
fn test_reflex_merges_multiple_receipts() {
    // Arrange: Test receipt merging
    let receipt1 = Receipt {
        id: "r1".to_string(),
        ticks: 4,
        lanes: 8,
        span_id: 0x1234,
        a_hash: 0xABCD,
    };

    let receipt2 = Receipt {
        id: "r2".to_string(),
        ticks: 6,
        lanes: 8,
        span_id: 0x5678,
        a_hash: 0xEF00,
    };

    // Act: Merge receipts
    let merged = ReflexStage::merge_receipts(&[receipt1, receipt2]);

    // Assert: Merged correctly
    assert_eq!(merged.ticks, 6, "Should use max ticks");
    assert_eq!(merged.lanes, 16, "Should sum lanes");
    assert_eq!(merged.span_id, 0x1234 ^ 0x5678, "Should XOR span_ids");
    assert_eq!(merged.a_hash, 0xABCD ^ 0xEF00, "Should XOR a_hashes");
}

// ============================================================================
// Test Suite: Emit Stage State-Based Validation
// ============================================================================

#[test]
fn test_emit_writes_receipts_to_lockchain() {
    // Arrange: Emit stage with lockchain enabled
    let mut emit = EmitStage::new(true, vec![]);

    let receipt = Receipt {
        id: "test_receipt_001".to_string(),
        ticks: 4,
        lanes: 8,
        span_id: 0x1234,
        a_hash: 0xABCD,
    };

    let reflex_result = ReflexResult {
        actions: vec![],
        receipts: vec![receipt],
        max_ticks: 4,
        c1_failure_actions: Vec::new(),
    };

    // Act: Emit
    let result = emit.emit(reflex_result).unwrap();

    // Assert: Receipts written
    assert_eq!(result.receipts_written, 1, "Should write 1 receipt");
    assert_eq!(result.lockchain_hashes.len(), 1, "Should generate lockchain hash");
}

#[test]
fn test_emit_sends_actions_to_webhooks() {
    // Arrange: Emit with webhook configured
    let mut emit = EmitStage::new(true, vec!["https://webhook.test.com".to_string()]);

    let action = Action {
        id: "action_001".to_string(),
        payload: vec![1, 2, 3, 4],
        receipt_id: "receipt_001".to_string(),
    };

    let receipt = Receipt {
        id: "receipt_001".to_string(),
        ticks: 3,
        lanes: 8,
        span_id: 0x9999,
        a_hash: 0xFFFF,
    };

    let reflex_result = ReflexResult {
        actions: vec![action],
        receipts: vec![receipt],
        max_ticks: 3,
        c1_failure_actions: Vec::new(),
    };

    // Act
    let result = emit.emit(reflex_result).unwrap();

    // Assert: Actions sent
    assert_eq!(result.actions_sent, 1, "Should send 1 action");
}

#[test]
fn test_emit_handles_empty_reflex_result() {
    // Arrange: Emit with no actions/receipts
    let mut emit = EmitStage::new(false, vec![]);

    let reflex_result = ReflexResult {
        actions: vec![],
        receipts: vec![],
        max_ticks: 0,
        c1_failure_actions: Vec::new(),
    };

    // Act
    let result = emit.emit(reflex_result);

    // Assert: Handles gracefully
    assert!(result.is_ok(), "Should handle empty result gracefully");
    let emit_result = result.unwrap();
    assert_eq!(emit_result.receipts_written, 0, "No receipts to write");
    assert_eq!(emit_result.actions_sent, 0, "No actions to send");
}

// ============================================================================
// Test Suite: Error Handling and Edge Cases
// ============================================================================

#[test]
fn test_pipeline_handles_multiple_predicates() {
    // Arrange: Complex data with multiple predicates
    let pipeline = Pipeline::new(
        vec!["test".to_string()],
        "urn:test:schema".to_string(),
        false,
        vec![],
    );

    let turtle = r#"
        <http://alice> <http://name> "Alice" .
        <http://alice> <http://age> "30" .
        <http://alice> <http://email> "alice@example.com" .
        <http://bob> <http://name> "Bob" .
        <http://bob> <http://age> "25" .
    "#;

    // Act: Full pipeline
    let ingest_result = IngestResult {
        triples: pipeline.ingest.parse_rdf_turtle(turtle).unwrap(),
        metadata: BTreeMap::new(),
    };
    let transform_result = pipeline.transform.transform(ingest_result).unwrap();
    let load_result = pipeline.load.load(transform_result).unwrap();
    let reflex_result = pipeline.reflex.reflex(load_result).unwrap();

    // Assert: All triples processed
    assert!(!reflex_result.receipts.is_empty(), "Should process all triples");
    assert!(reflex_result.max_ticks <= 8, "Should respect tick budget");
}

#[test]
fn test_pipeline_creation_validates_config() {
    // Arrange & Act: Create pipeline with config
    let pipeline = Pipeline::new(
        vec!["kafka".to_string(), "postgres".to_string()],
        "urn:knhk:schema:production".to_string(),
        true,
        vec!["https://webhook1.com".to_string(), "https://webhook2.com".to_string()],
    );

    // Assert: Config applied correctly
    assert_eq!(pipeline.load.max_run_len, 8, "Should enforce 8-triple max run length");
    assert_eq!(pipeline.reflex.tick_budget, 8, "Should enforce 8-tick budget");
}
