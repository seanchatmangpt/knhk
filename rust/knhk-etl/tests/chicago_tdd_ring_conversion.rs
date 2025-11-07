// rust/knhk-etl/tests/chicago_tdd_ring_conversion.rs
// Chicago TDD tests for Ring Conversion
// Focus: Behavior verification using AAA pattern (Arrange, Act, Assert)

extern crate alloc;

use knhk_etl::ring_conversion::{raw_triples_to_soa, soa_to_raw_triples};
use knhk_etl::RawTriple;
use alloc::vec::Vec;
use alloc::string::ToString;

#[test]
fn test_ring_conversion_raw_to_soa() {
    // Arrange: Create raw triples
    let triples = vec![
        RawTriple {
            subject: "http://example.org/s1".to_string(),
            predicate: "http://example.org/p1".to_string(),
            object: "http://example.org/o1".to_string(),
            graph: None,
        },
        RawTriple {
            subject: "http://example.org/s2".to_string(),
            predicate: "http://example.org/p1".to_string(),
            object: "http://example.org/o2".to_string(),
            graph: None,
        },
    ];
    
    // Act: Convert to SoA
    #[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
    let (S, P, O) = raw_triples_to_soa(&triples).expect("Should convert");
    
    // Assert: SoA arrays have correct length and values
    assert_eq!(S.len(), 2);
    assert_eq!(P.len(), 2);
    assert_eq!(O.len(), 2);
    assert!(S[0] > 0); // Hashed IRI
    assert!(S[1] > 0);
    assert_eq!(P[0], P[1]); // Same predicate
}

#[test]
fn test_ring_conversion_soa_to_raw() {
    // Arrange: Create SoA arrays
    #[allow(non_snake_case)] // RDF naming convention
    let S = vec![100u64, 200u64];
    #[allow(non_snake_case)] // RDF naming convention
    let P = vec![50u64, 50u64];
    #[allow(non_snake_case)] // RDF naming convention
    let O = vec![10u64, 20u64];
    
    // Act: Convert to raw triples
    let triples = soa_to_raw_triples(&S, &P, &O);
    
    // Assert: Raw triples have correct length
    assert_eq!(triples.len(), 2);
    // Note: Subject/predicate/object values are hashed IRIs, so we can't check exact values
    // But we can verify structure
    assert!(!triples[0].subject.is_empty());
    assert!(!triples[0].predicate.is_empty());
    assert!(!triples[0].object.is_empty());
}

#[test]
fn test_ring_conversion_empty_input() {
    // Arrange: Empty triples
    let triples = Vec::new();
    
    // Act: Convert to SoA
    let result = raw_triples_to_soa(&triples);
    
    // Assert: Returns empty arrays
    assert!(result.is_ok());
    #[allow(non_snake_case)] // RDF naming convention
    let (S, P, O) = result.unwrap();
    assert_eq!(S.len(), 0);
    assert_eq!(P.len(), 0);
    assert_eq!(O.len(), 0);
}

#[test]
fn test_ring_conversion_max_run_len() {
    // Arrange: Create exactly 8 triples (max_run_len)
    let triples: Vec<RawTriple> = (0..8)
        .map(|i| RawTriple {
            subject: format!("http://example.org/s{}", i),
            predicate: "http://example.org/p1".to_string(),
            object: format!("http://example.org/o{}", i),
            graph: None,
        })
        .collect();
    
    // Act: Convert to SoA
    let result = raw_triples_to_soa(&triples);
    
    // Assert: Conversion succeeds (within max_run_len)
    assert!(result.is_ok());
    #[allow(non_snake_case)] // RDF naming convention
    let (S, P, O) = result.unwrap();
    assert_eq!(S.len(), 8);
    assert_eq!(P.len(), 8);
    assert_eq!(O.len(), 8);
}


