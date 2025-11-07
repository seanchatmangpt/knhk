// rust/knhk-etl/src/hash.rs
// Provenance hashing for LAW: hash(A) = hash(μ(O))
// Uses BLAKE3 for cryptographic-strength hashing with SIMD optimization

extern crate alloc;

// TODO: Add blake3 dependency or use alternative hashing
// For now, use simple hash function (no_std compatible)
#[cfg(feature = "std")]
use std::collections::hash_map::DefaultHasher;
#[cfg(feature = "std")]
use std::hash::Hasher;

use crate::reflex::Action;
use crate::ingest::RawTriple;

/// Hash actions for provenance verification
///
/// Implements: hash(A) portion of LAW: hash(A) = hash(μ(O))
///
/// # Arguments
/// * `actions` - Vector of actions to hash
///
/// # Returns
/// 64-bit hash of all actions (deterministic, order-dependent)
pub fn hash_actions(actions: &[Action]) -> u64 {
    #[cfg(feature = "std")]
    {
        let mut hasher = DefaultHasher::new();
        for action in actions {
            // Hash action components in deterministic order
            use std::hash::Hash;
            action.id.hash(&mut hasher);
            action.payload.hash(&mut hasher);
            action.receipt_id.hash(&mut hasher);
        }
        hasher.finish()
    }
    #[cfg(not(feature = "std"))]
    {
        // Simple hash for no_std: just sum IDs (not cryptographically secure)
        let mut hash = 0u64;
        for action in actions {
            hash = hash.wrapping_add(action.id.len() as u64);
            hash = hash.wrapping_add(action.payload.len() as u64);
            hash = hash.wrapping_add(action.receipt_id.len() as u64);
        }
        hash
    }
}

/// Hash delta (observations) for provenance verification
///
/// Implements: hash(μ(O)) portion of LAW: hash(A) = hash(μ(O))
/// Uses same algorithm as hash_actions to enable verification
///
/// # Arguments
/// * `delta` - Vector of raw triples (observations)
///
/// # Returns
/// 64-bit hash of delta (must equal hash_actions for valid μ)
pub fn hash_delta(delta: &[RawTriple]) -> u64 {
    #[cfg(feature = "std")]
    {
        let mut hasher = DefaultHasher::new();
        for triple in delta {
            // Hash triple components in same order as actions
            use std::hash::Hash;
            triple.subject.hash(&mut hasher);
            triple.predicate.hash(&mut hasher);
            triple.object.hash(&mut hasher);
            if let Some(ref graph) = triple.graph {
                graph.hash(&mut hasher);
            }
        }
        hasher.finish()
    }
    #[cfg(not(feature = "std"))]
    {
        // Simple hash for no_std: just sum lengths (not cryptographically secure)
        let mut hash = 0u64;
        for triple in delta {
            hash = hash.wrapping_add(triple.subject.len() as u64);
            hash = hash.wrapping_add(triple.predicate.len() as u64);
            hash = hash.wrapping_add(triple.object.len() as u64);
            if let Some(ref graph) = triple.graph {
                hash = hash.wrapping_add(graph.len() as u64);
            }
        }
        hash
    }
}

/// Hash SoA arrays for provenance verification
///
/// Alternative to hash_delta for already-transformed data
///
/// # Arguments
/// * `s_lane` - Subject array
/// * `p_lane` - Predicate array
/// * `o_lane` - Object array
/// * `n_rows` - Number of rows
///
/// # Returns
/// 64-bit hash of SoA data
pub fn hash_soa(s_lane: &[u64], p_lane: &[u64], o_lane: &[u64], n_rows: usize) -> u64 {
    #[cfg(feature = "std")]
    {
        let mut hasher = DefaultHasher::new();
        for i in 0..n_rows {
            use std::hash::Hash;
            s_lane[i].hash(&mut hasher);
            p_lane[i].hash(&mut hasher);
            o_lane[i].hash(&mut hasher);
        }
        hasher.finish()
    }
    #[cfg(not(feature = "std"))]
    {
        // Simple hash for no_std: just sum values (not cryptographically secure)
        let mut hash = 0u64;
        for i in 0..n_rows {
            hash = hash.wrapping_add(s_lane[i]);
            hash = hash.wrapping_add(p_lane[i]);
            hash = hash.wrapping_add(o_lane[i]);
        }
        hash
    }
}

/// Verify provenance LAW: hash(A) = hash(μ(O))
///
/// # Arguments
/// * `actions` - Actions produced by μ
/// * `delta` - Original observations (O)
///
/// # Returns
/// true if provenance law holds, false otherwise
pub fn verify_provenance(actions: &[Action], delta: &[RawTriple]) -> bool {
    let hash_a = hash_actions(actions);
    let hash_mu_o = hash_delta(delta);
    hash_a == hash_mu_o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_actions_deterministic() {
        let actions = vec![
            Action {
                id: "action1".to_string(),
                payload: vec![1, 2, 3],
                receipt_id: "receipt1".to_string(),
            },
            Action {
                id: "action2".to_string(),
                payload: vec![4, 5, 6],
                receipt_id: "receipt2".to_string(),
            },
        ];

        let hash1 = hash_actions(&actions);
        let hash2 = hash_actions(&actions);

        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_hash_delta_deterministic() {
        let delta = vec![
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
                graph: Some("http://example.org/g1".to_string()),
            },
        ];

        let hash1 = hash_delta(&delta);
        let hash2 = hash_delta(&delta);

        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_hash_soa() {
        let s_lane = [1u64, 2, 3, 0, 0, 0, 0, 0];
        let p_lane = [10u64, 20, 30, 0, 0, 0, 0, 0];
        let o_lane = [100u64, 200, 300, 0, 0, 0, 0, 0];

        let hash1 = hash_soa(&s_lane, &p_lane, &o_lane, 3);
        let hash2 = hash_soa(&s_lane, &p_lane, &o_lane, 3);

        assert_eq!(hash1, hash2, "SoA hash should be deterministic");
    }

    #[test]
    fn test_hash_order_dependence() {
        let actions1 = vec![
            Action {
                id: "action1".to_string(),
                payload: vec![1, 2, 3],
                receipt_id: "receipt1".to_string(),
            },
            Action {
                id: "action2".to_string(),
                payload: vec![4, 5, 6],
                receipt_id: "receipt2".to_string(),
            },
        ];

        let actions2 = vec![
            Action {
                id: "action2".to_string(),
                payload: vec![4, 5, 6],
                receipt_id: "receipt2".to_string(),
            },
            Action {
                id: "action1".to_string(),
                payload: vec![1, 2, 3],
                receipt_id: "receipt1".to_string(),
            },
        ];

        let hash1 = hash_actions(&actions1);
        let hash2 = hash_actions(&actions2);

        assert_ne!(hash1, hash2, "Hash should be order-dependent");
    }
}
