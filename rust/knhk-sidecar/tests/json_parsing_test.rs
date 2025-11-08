// rust/knhk-sidecar/tests/json_parsing_test.rs
// Unit tests for JSON parsing with simdjson

use knhk_sidecar::json_parser::{parse_json_triples, JsonDelta, JsonParseError};

#[test]
fn test_parse_simple_json_delta() {
    let json = r#"{
        "additions": [
            {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
        ],
        "removals": []
    }"#;

    let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
    assert_eq!(delta.additions.len(), 1);
    assert_eq!(delta.removals.len(), 0);
    assert_eq!(delta.additions[0].s, "http://example.org/s");
    assert_eq!(delta.additions[0].p, "http://example.org/p");
    assert_eq!(delta.additions[0].o, "http://example.org/o");
}

#[test]
fn test_parse_json_array() {
    let json = r#"[
        {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
    ]"#;

    let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
    assert_eq!(delta.additions.len(), 1);
}

#[test]
fn test_to_raw_triples() {
    let json = r#"{
        "additions": [
            {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o", "g": "http://example.org/g"}
        ],
        "removals": []
    }"#;

    let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
    let raw_triples = delta.to_raw_triples().unwrap();
    assert_eq!(raw_triples.len(), 1);
    assert_eq!(raw_triples[0].subject, "http://example.org/s");
    assert_eq!(raw_triples[0].predicate, "http://example.org/p");
    assert_eq!(raw_triples[0].object, "http://example.org/o");
    assert_eq!(
        raw_triples[0].graph,
        Some("http://example.org/g".to_string())
    );
}

#[test]
fn test_parse_empty_json() {
    let json = b"{}";
    let result = JsonDelta::from_bytes(json);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_json() {
    let json = b"{ invalid json }";
    let result = JsonDelta::from_bytes(json);
    assert!(result.is_err());
}

#[test]
fn test_parse_json_with_numeric_values() {
    let json = r#"{
        "additions": [
            {"s": "123", "p": "456", "o": "789"}
        ],
        "removals": []
    }"#;

    let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
    assert_eq!(delta.additions.len(), 1);
    assert_eq!(delta.additions[0].s, "123");
    assert_eq!(delta.additions[0].p, "456");
    assert_eq!(delta.additions[0].o, "789");
}

#[test]
fn test_parse_json_ld_format() {
    let json = r#"{
        "@graph": [
            {
                "@id": "http://example.org/s",
                "http://example.org/p": "http://example.org/o"
            }
        ]
    }"#;

    let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
    assert!(!delta.additions.is_empty());
}

#[test]
fn test_parse_json_triples_convenience() {
    let json = r#"[
        {"s": "http://example.org/s", "p": "http://example.org/p", "o": "http://example.org/o"}
    ]"#;

    let result = parse_json_triples(json.as_bytes());
    assert!(result.is_ok());
    let triples = result.unwrap();
    assert_eq!(triples.len(), 1);
    assert_eq!(triples[0].subject, "http://example.org/s");
}

#[test]
fn test_parse_json_missing_fields() {
    let json = r#"{
        "additions": [
            {"s": "http://example.org/s"}
        ],
        "removals": []
    }"#;

    let result = JsonDelta::from_bytes(json.as_bytes());
    assert!(result.is_err());
}

#[test]
fn test_parse_json_multiple_triples() {
    let json = r#"{
        "additions": [
            {"s": "http://example.org/s1", "p": "http://example.org/p1", "o": "http://example.org/o1"},
            {"s": "http://example.org/s2", "p": "http://example.org/p2", "o": "http://example.org/o2"}
        ],
        "removals": []
    }"#;

    let delta = JsonDelta::from_bytes(json.as_bytes()).unwrap();
    assert_eq!(delta.additions.len(), 2);
    let raw_triples = delta.to_raw_triples().unwrap();
    assert_eq!(raw_triples.len(), 2);
}
