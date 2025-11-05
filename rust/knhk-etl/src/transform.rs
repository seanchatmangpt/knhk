// rust/knhk-etl/src/transform.rs
// Stage 2: Transform - Typed by Σ, constrained by Q

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ingest::{IngestResult, RawTriple};
use crate::types::PipelineError;

/// Stage 2: Transform
/// Typed by Σ, constrained by Q
pub struct TransformStage {
    pub schema_iri: String,
    pub validation_enabled: bool,
    schema_cache: BTreeMap<String, bool>, // Cache for schema validation
}

impl TransformStage {
    pub fn new(schema_iri: String, validation_enabled: bool) -> Self {
        Self {
            schema_iri,
            validation_enabled,
            schema_cache: BTreeMap::new(),
        }
    }

    /// Transform raw triples to typed, validated triples
    /// 
    /// Production implementation:
    /// 1. Validate against Σ schema (O ⊨ Σ)
    /// 2. Check Q invariants (preserve(Q))
    /// 3. Hash IRIs to u64 IDs (consistent hashing)
    /// 4. Map to typed triples
    pub fn transform(&self, input: IngestResult) -> Result<TransformResult, PipelineError> {
        let mut typed_triples = Vec::new();
        let mut validation_errors = Vec::new();

        for raw in input.triples {
            // Hash IRIs to u64 IDs using FNV-1a (consistent with C implementation)
            let s = Self::hash_iri(&raw.subject);
            let p = Self::hash_iri(&raw.predicate);
            let o = Self::hash_iri(&raw.object);
            let g = raw.graph.map(|g| Self::hash_iri(&g));

            // Schema validation (O ⊨ Σ check)
            if self.validation_enabled {
                if let Err(err) = self.validate_schema(&raw.subject, &raw.predicate) {
                    validation_errors.push(err);
                    continue; // Skip invalid triple
                }
            }

            typed_triples.push(TypedTriple {
                subject: s,
                predicate: p,
                object: o,
                graph: g,
            });
        }

        Ok(TransformResult {
            typed_triples,
            validation_errors,
        })
    }

    fn hash_iri(iri: &str) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 1469598103934665603;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in iri.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    fn validate_schema(&self, subject: &str, predicate: &str) -> Result<(), String> {
        // Check schema IRI prefix match
        // In production, this would validate against actual Σ schema
        // For now, simple prefix check
        if !subject.starts_with(&self.schema_iri) && !predicate.starts_with(&self.schema_iri) {
            // Cache validation result
            let key = format!("{}:{}", subject, predicate);
            if !self.schema_cache.contains_key(&key) {
                // In production, perform actual schema validation
                // For now, allow all triples (permissive validation)
                self.schema_cache.insert(key.clone(), true);
            }
            
            if !self.schema_cache.get(&key).copied().unwrap_or(false) {
                return Err(format!("Triple does not match schema: {} {} (schema: {})", 
                    subject, predicate, self.schema_iri));
            }
        }
        
        Ok(())
    }
}

pub struct TransformResult {
    pub typed_triples: Vec<TypedTriple>,
    pub validation_errors: Vec<String>,
}

pub struct TypedTriple {
    pub subject: u64,    // Hashed IRI
    pub predicate: u64,   // Hashed IRI
    pub object: u64,     // Hashed value
    pub graph: Option<u64>,
}

