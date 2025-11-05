// rust/knhk-etl/src/lib.rs
// ETL Pipeline Stages
// Implements: Ingest → Transform → Load → Reflex → Emit

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Module declarations
pub mod types;
pub mod ingest;
pub mod transform;
pub mod load;
pub mod reflex;
pub mod emit;
pub mod pipeline;
pub mod integration;

// Re-export main types
pub use types::{PipelineStage, PipelineMetrics, PipelineError};
pub use ingest::{IngestStage, IngestResult, RawTriple};
pub use transform::{TransformStage, TransformResult, TypedTriple};
pub use load::{LoadStage, LoadResult, SoAArrays, PredRun, HookOperation};
pub use reflex::{ReflexStage, ReflexResult, Action, Receipt};
pub use emit::{EmitStage, EmitResult};
pub use pipeline::Pipeline;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;

    #[test]
    fn test_pipeline_creation() {
        #[cfg(feature = "std")]
        {
        let pipeline = Pipeline::new(
            vec!["kafka_connector".to_string()],
            "urn:knhk:schema:test".to_string(),
            true,
            vec!["https://webhook.example.com".to_string()],
        );

        assert_eq!(pipeline.load.max_run_len, 8);
        assert_eq!(pipeline.reflex.tick_budget, 8);
        }
    }

    #[test]
    fn test_load_stage_guard() {
        let load = LoadStage::new();
        let transform_result = TransformResult {
            typed_triples: vec![TypedTriple {
                subject: 1,
                predicate: 2,
                object: 3,
                graph: None,
            }; 10], // Exceeds max_run_len
            validation_errors: Vec::new(),
        };

        assert!(load.load(transform_result).is_err());
    }

    #[test]
    fn test_ingest_stage_rdf_parsing() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = "<http://example.org/subject> <http://example.org/predicate> <http://example.org/object> .";
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].subject, "http://example.org/subject");
        assert_eq!(triples[0].predicate, "http://example.org/predicate");
        assert_eq!(triples[0].object, "http://example.org/object");
    }

    #[test]
    fn test_transform_stage_hashing() {
        let transform = TransformStage::new("urn:knhk:schema:test".to_string(), false);
        
        let ingest_result = IngestResult {
            triples: vec![
                RawTriple {
                    subject: "http://example.org/subject".to_string(),
                    predicate: "http://example.org/predicate".to_string(),
                    object: "http://example.org/object".to_string(),
                    graph: None,
                }
            ],
            metadata: BTreeMap::new(),
        };
        
        let result = transform.transform(ingest_result);
        assert!(result.is_ok());
        
        let transform_result = result.unwrap();
        assert_eq!(transform_result.typed_triples.len(), 1);
        assert!(transform_result.typed_triples[0].subject > 0);
        assert!(transform_result.typed_triples[0].predicate > 0);
        assert!(transform_result.typed_triples[0].object > 0);
    }

    #[test]
    fn test_load_stage_predicate_grouping() {
        let load = LoadStage::new();
        
        let transform_result = TransformResult {
            typed_triples: vec![
                TypedTriple { subject: 1, predicate: 100, object: 10, graph: None },
                TypedTriple { subject: 2, predicate: 100, object: 20, graph: None },
                TypedTriple { subject: 3, predicate: 200, object: 30, graph: None },
            ],
            validation_errors: Vec::new(),
        };
        
        let result = load.load(transform_result);
        assert!(result.is_ok());
        
        let load_result = result.unwrap();
        assert_eq!(load_result.runs.len(), 2); // Two different predicates
        assert_eq!(load_result.runs[0].pred, 100);
        assert_eq!(load_result.runs[0].len, 2);
        assert_eq!(load_result.runs[1].pred, 200);
        assert_eq!(load_result.runs[1].len, 1);
    }

    #[test]
    fn test_receipt_merging() {
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
        
        let merged = ReflexStage::merge_receipts(&[receipt1, receipt2]);
        
        assert_eq!(merged.ticks, 6); // Max ticks
        assert_eq!(merged.lanes, 16); // Sum lanes
        assert_eq!(merged.span_id, 0x1234 ^ 0x5678); // XOR merge
        assert_eq!(merged.a_hash, 0xABCD ^ 0xEF00); // XOR merge
    }
}
