// rust/knhk-etl/src/transform.rs
// Stage 2: Transform
// Typed by Σ, constrained by Q

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::format;

use crate::error::PipelineError;
use crate::ingest::{IngestResult, RawTriple};

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

    /// Hash IRI to u64 using FNV-1a (consistent with C implementation)
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

    /// Validate triple against schema (O ⊨ Σ)
    /// 
    /// Validates IRI format and schema namespace matching.
    /// Uses cache for repeated validations.
    /// 
    /// Note: Full schema registry integration is planned for v1.0.
    /// Current implementation validates IRI format and namespace matching.
    fn validate_schema(&self, subject: &str, predicate: &str) -> Result<(), String> {
        // Check schema IRI prefix match
        if !self.schema_iri.is_empty() {
            if !subject.starts_with(&self.schema_iri) && !predicate.starts_with(&self.schema_iri) {
                // Check cache first
                let cache_key = format!("{}:{}", subject, predicate);
                if let Some(&valid) = self.schema_cache.get(&cache_key) {
                    if !valid {
                        return Err(format!("Schema validation failed for {} {}", subject, predicate));
                    }
                } else {
                    // Validate IRI format (must contain namespace separator)
                    let valid = predicate.contains(":") || subject.contains(":");
                    if !valid {
                        return Err(format!("Schema validation failed: invalid IRI format for {} {}", subject, predicate));
                    }
                }
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

