// rust/knhk-etl/src/lib.rs
// ETL Pipeline Stages
// Implements: Ingest → Transform → Load → Reflex → Emit

extern crate alloc;
extern crate std;

// Silence unused crate warnings (proper feature gating)
// Note: These crates are always available when features are enabled
#[cfg(feature = "knhk-otel")]
use knhk_otel as _;
#[cfg(feature = "knhk-lockchain")]
use knhk_lockchain as _;

// Module declarations
pub mod types;
pub mod error;
pub mod ingest;
pub mod ingester; // Ingester pattern - inspired by Weaver
pub mod transform;
pub mod load;
pub mod reflex;
pub mod reflex_map;
pub mod emit;
pub mod pipeline;
pub mod runtime_class;
pub mod slo_monitor;
pub mod failure_actions;

// Re-exports for convenience
pub use types::{PipelineStage, PipelineMetrics};
pub use error::PipelineError;
pub use ingest::{IngestStage, IngestResult, RawTriple};
pub use transform::{TransformStage, TransformResult, TypedTriple};
pub use load::{LoadStage, LoadResult, SoAArrays, PredRun};
pub use reflex::{ReflexStage, ReflexResult, Action, Receipt};
// ReflexMap types are exported separately to avoid conflicts with reflex::Action/Receipt
pub use reflex_map::{ReflexMap, ReflexMapResult};
pub use emit::{EmitStage, EmitResult};
pub use pipeline::Pipeline;

pub mod integration;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::vec;
    use alloc::string::ToString;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = Pipeline::new(
            vec!["kafka_connector".to_string()],
            "urn:knhk:schema:test".to_string(),
            true,
            vec!["https://webhook.example.com".to_string()],
        );

        assert_eq!(pipeline.load.max_run_len, 8);
        assert_eq!(pipeline.reflex.tick_budget, 8);
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
    fn test_ingest_stage_prefix_resolution() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = r#"
            @prefix ex: <http://example.org/> .
            ex:subject ex:predicate ex:object .
        "#;
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].subject, "http://example.org/subject");
        assert_eq!(triples[0].predicate, "http://example.org/predicate");
        assert_eq!(triples[0].object, "http://example.org/object");
    }

    #[test]
    fn test_ingest_stage_blank_nodes() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = r#"
            _:alice <http://example.org/name> "Alice" .
            _:bob <http://example.org/name> "Bob" .
        "#;
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2);
        assert!(triples[0].subject.starts_with("_:"));
        assert!(triples[1].subject.starts_with("_:"));
        assert_eq!(triples[0].object, "\"Alice\"");
        assert_eq!(triples[1].object, "\"Bob\"");
    }

    #[test]
    fn test_ingest_stage_literals() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = r#"
            <http://example.org/subject> <http://example.org/name> "Alice" .
            <http://example.org/subject> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
            <http://example.org/subject> <http://example.org/label> "Hello"@en .
        "#;
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 3);
        assert_eq!(triples[0].object, "\"Alice\"");
        assert!(triples[1].object.contains("integer"));
        assert!(triples[2].object.contains("@en"));
    }

    #[test]
    fn test_ingest_stage_base_uri() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = r#"
            @base <http://example.org/> .
            <subject> <predicate> <object> .
        "#;
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].subject, "http://example.org/subject");
        assert_eq!(triples[0].predicate, "http://example.org/predicate");
        assert_eq!(triples[0].object, "http://example.org/object");
    }

    #[test]
    fn test_ingest_stage_multiple_triples() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = r#"
            <http://example.org/alice> <http://example.org/name> "Alice" .
            <http://example.org/alice> <http://example.org/age> "30" .
            <http://example.org/bob> <http://example.org/name> "Bob" .
        "#;
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_ok());
        let triples = result.unwrap();
        assert_eq!(triples.len(), 3);
    }

    #[test]
    fn test_ingest_stage_empty_input() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let result = ingest.parse_rdf_turtle("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_ingest_stage_invalid_syntax() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let content = "<http://example.org/subject> <http://example.org/predicate>";
        let result = ingest.parse_rdf_turtle(content);
        
        assert!(result.is_err());
        if let Err(PipelineError::IngestError(msg)) = result {
            assert!(msg.contains("parse error"));
        } else {
            panic!("Expected IngestError");
        }
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
    fn test_reflex_stage_tick_budget() {
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
        
        let result = reflex.reflex(load_result);
        assert!(result.is_ok());
        
        let reflex_result = result.unwrap();
        assert!(reflex_result.max_ticks <= 8);
        assert!(!reflex_result.receipts.is_empty());
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

    #[test]
    fn test_emit_stage() {
        let mut emit = EmitStage::new(true, vec!["https://webhook.example.com".to_string()]);

        let receipt = Receipt {
            id: "receipt1".to_string(),
            ticks: 4,
            lanes: 8,
            span_id: 0x1234,
            a_hash: 0xABCD,
        };

        let reflex_result = ReflexResult {
            actions: vec![
                Action {
                    id: "action1".to_string(),
                    payload: vec![1, 2, 3],
                    receipt_id: "receipt1".to_string(),
                }
            ],
            receipts: vec![receipt],
            max_ticks: 4,
            c1_failure_actions: Vec::new(),
        };

        let result = emit.emit(reflex_result);
        assert!(result.is_ok());
        
        let emit_result = result.unwrap();
        assert_eq!(emit_result.receipts_written, 1);
        assert_eq!(emit_result.actions_sent, 1);
        assert_eq!(emit_result.lockchain_hashes.len(), 1);
    }
}
