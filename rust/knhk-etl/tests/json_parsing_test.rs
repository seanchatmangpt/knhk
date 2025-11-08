// rust/knhk-etl/tests/json_parsing_test.rs
// Unit tests for JSON parsing in IngestStage

use knhk_etl::ingest::{IngestStage, RawTriple};

#[test]
fn test_parse_json_delta_simple() {
    let ingest = IngestStage::new(vec!["test".to_string()], "json".to_string());

    let json = r#"{
        "additions": [
            {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
        ]
    }"#;

    let result = ingest.parse_json_delta(json.as_bytes());
    assert!(result.is_ok());
    let triples = result.unwrap();
    assert_eq!(triples.len(), 1);
    assert_eq!(triples[0].subject, "http://example.org/s");
    assert_eq!(triples[0].predicate, "http://example.org/p");
    assert_eq!(triples[0].object, "http://example.org/o");
}

#[test]
fn test_parse_json_delta_array() {
    let ingest = IngestStage::new(vec!["test".to_string()], "json".to_string());

    let json = r#"[
        {"s": "http://example.org/s1", "p": "http://example.org/p1", "o": "http://example.org/o1"},
        {"s": "http://example.org/s2", "p": "http://example.org/p2", "o": "http://example.org/o2"}
    ]"#;

    let result = ingest.parse_json_delta(json.as_bytes());
    assert!(result.is_ok());
    let triples = result.unwrap();
    assert_eq!(triples.len(), 2);
}

#[test]
fn test_parse_json_delta_empty() {
    let ingest = IngestStage::new(vec!["test".to_string()], "json".to_string());

    let json = b"";
    let result = ingest.parse_json_delta(json);
    assert!(result.is_ok());
    let triples = result.unwrap();
    assert_eq!(triples.len(), 0);
}

#[test]
fn test_parse_json_delta_with_graph() {
    let ingest = IngestStage::new(vec!["test".to_string()], "json".to_string());

    let json = r#"{
        "additions": [
            {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o", "g": "http://example.org/g"}
        ]
    }"#;

    let result = ingest.parse_json_delta(json.as_bytes());
    assert!(result.is_ok());
    let triples = result.unwrap();
    assert_eq!(triples.len(), 1);
    assert_eq!(triples[0].graph, Some("http://example.org/g".to_string()));
}

#[test]
fn test_parse_json_delta_invalid_json() {
    let ingest = IngestStage::new(vec!["test".to_string()], "json".to_string());

    let json = b"{ invalid json }";
    let result = ingest.parse_json_delta(json);
    assert!(result.is_err());
}
