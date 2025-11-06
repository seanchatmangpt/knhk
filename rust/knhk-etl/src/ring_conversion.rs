// rust/knhk-etl/src/ring_conversion.rs
// Conversion utilities for RawTriple ↔ SoA arrays
// Uses hash-based IRI encoding (placeholder until MPHF/registry available)

use crate::ingest::RawTriple;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Convert RawTriple to SoA arrays (S, P, O as u64)
/// Uses hash-based encoding: hash(IRI) → u64
/// Note: This is a placeholder until MPHF or IRI registry is available
pub fn raw_triples_to_soa(triples: &[RawTriple]) -> Result<(Vec<u64>, Vec<u64>, Vec<u64>), String> {
    if triples.len() > 8 {
        return Err(format!("Triple count {} exceeds max_run_len 8", triples.len()));
    }

    let mut S = Vec::with_capacity(triples.len());
    let mut P = Vec::with_capacity(triples.len());
    let mut O = Vec::with_capacity(triples.len());

    for triple in triples {
        S.push(hash_iri(&triple.subject)?);
        P.push(hash_iri(&triple.predicate)?);
        O.push(hash_iri(&triple.object)?);
    }

    Ok((S, P, O))
}

/// Convert SoA arrays back to RawTriple
/// Uses hex representation of u64 as IRI
/// Note: This is a placeholder until reverse lookup from IRI registry is available
pub fn soa_to_raw_triples(S: &[u64], P: &[u64], O: &[u64]) -> Vec<RawTriple> {
    if S.len() != P.len() || P.len() != O.len() {
        return Vec::new();
    }

    S.iter()
        .zip(P.iter())
        .zip(O.iter())
        .map(|((&s, &p), &o)| RawTriple {
            subject: format!("urn:hash:{:x}", s),
            predicate: format!("urn:hash:{:x}", p),
            object: format!("urn:hash:{:x}", o),
            graph: None,
        })
        .collect()
}

/// Hash IRI string to u64
/// Uses std::collections::hash_map::DefaultHasher
/// Note: This is a placeholder until MPHF or IRI registry is available
fn hash_iri(iri: &str) -> Result<u64, String> {
    if iri.is_empty() {
        return Err("IRI cannot be empty".to_string());
    }

    let mut hasher = DefaultHasher::new();
    iri.hash(&mut hasher);
    Ok(hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_triples_to_soa() {
        let triples = vec![
            RawTriple {
                subject: "http://example.org/s1".to_string(),
                predicate: "http://example.org/p1".to_string(),
                object: "http://example.org/o1".to_string(),
                graph: None,
            },
            RawTriple {
                subject: "http://example.org/s2".to_string(),
                predicate: "http://example.org/p2".to_string(),
                object: "http://example.org/o2".to_string(),
                graph: None,
            },
        ];

        let result = raw_triples_to_soa(&triples);
        assert!(result.is_ok());
        let (S, P, O) = match result {
            Ok(v) => v,
            Err(e) => panic!("Failed to convert triples to SoA: {}", e),
        };
        assert_eq!(S.len(), 2);
        assert_eq!(P.len(), 2);
        assert_eq!(O.len(), 2);
    }

    #[test]
    fn test_soa_to_raw_triples() {
        let S = vec![0x1234, 0x5678];
        let P = vec![0xabcd, 0xef00];
        let O = vec![0x1111, 0x2222];

        let triples = soa_to_raw_triples(&S, &P, &O);
        assert_eq!(triples.len(), 2);
        assert!(triples[0].subject.starts_with("urn:hash:"));
    }

    #[test]
    fn test_hash_iri() {
        let result = hash_iri("http://example.org/test");
        assert!(result.is_ok());
        let hash = match result {
            Ok(v) => v,
            Err(e) => panic!("Failed to hash IRI: {}", e),
        };
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_hash_iri_empty() {
        let result = hash_iri("");
        assert!(result.is_err());
    }
}

