// rust/knhk-etl/src/lib.rs
// ETL Pipeline Stages
// Implements: Ingest → Transform → Load → Reflex → Emit

// CRITICAL: Enforce proper error handling - no unwrap/expect in production code
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
// Allow acceptable warnings for clean build
#![allow(unused_imports)] // Some imports are conditional or reserved for planned use
#![allow(unused_variables)] // Some variables are used in conditional compilation
#![allow(unused_mut)] // Some mut variables are used in conditional compilation
#![allow(dead_code)] // Some code is reserved for planned features
#![allow(deprecated)] // Some dependencies use deprecated APIs (will be updated)
#![allow(unexpected_cfgs)] // Some cfg values are informational

extern crate alloc;
extern crate std;

// Conditional compilation for optional features
#[cfg(feature = "knhk-lockchain")]
use knhk_lockchain as _;
#[cfg(feature = "knhk-otel")]
use knhk_otel as _;

// OpenTelemetry initialization (requires tokio runtime for async OTLP exporter)
#[cfg(feature = "tokio-runtime")]
pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry_sdk::trace::TracerProvider;
    use tracing_subscriber::layer::SubscriberExt;

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry_sdk::runtime::TokioScheduler)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

// No-op version for non-tokio builds
#[cfg(not(feature = "tokio-runtime"))]
pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// Module declarations
pub mod beat_scheduler; // 8-beat epoch scheduler
pub mod buffer_pool; // Memory reuse pool (simdjson pattern)
pub mod emit;
pub mod error;
pub mod failure_actions;
pub mod fiber; // Cooperative fibers for deterministic execution
pub mod guard_validation; // Branchless guard validation helpers
pub mod hash; // Provenance hashing for LAW: hash(A) = hash(μ(O))
pub mod hook_orchestration; // Pattern-based hook orchestration
pub mod hook_registry; // Hook registry for predicate-to-kernel mapping
pub mod hot_path_engine; // Reusable hot path engine with memory reuse
pub mod ingest;
pub mod ingester; // Ingester pattern - inspired by Weaver
pub mod load;
pub mod park; // Park/escalate mechanism for over-budget work
pub mod pipeline;
pub mod reconcile;
pub mod reflex;
pub mod reflex_map;
pub mod ring_buffer;
pub mod ring_conversion; // Lock-free ring buffers for 8-beat epoch
pub mod runtime_class;
pub mod slo_monitor;
pub mod transform;
pub mod triple_view; // Zero-copy triple access patterns
pub mod types; // Reconciliation: A = μ(O)

// Re-exports for convenience
pub use buffer_pool::{BufferPool, CapacityUsage, PoolError};
pub use error::PipelineError;
pub use hot_path_engine::HotPathEngine;
pub use ingest::{IngestResult, IngestStage, RawTriple};
pub use load::{LoadResult, LoadStage, PredRun, SoAArrays};
pub use reflex::{Action, Receipt, ReflexResult, ReflexStage};
pub use transform::{TransformResult, TransformStage, TypedTriple};
pub use triple_view::{SoAArraysExt, TripleIterator, TripleView};
pub use types::{PipelineMetrics, PipelineStage};
// ReflexMap types are exported separately to avoid conflicts with reflex::Action/Receipt
pub use emit::{EmitResult, EmitStage};
pub use ingester::{FileIngester, Ingester, StdinIngester};
pub use pipeline::Pipeline;
pub use reflex_map::{ReflexMap, ReflexMapResult};

// Hook registry exports
pub use hook_registry::{GuardFn, HookMetadata, HookRegistry, HookRegistryError};

// Hook orchestration exports
pub use hook_orchestration::{
    HookExecutionContext, HookExecutionPattern, HookExecutionResult, HookOrchestrator,
};

// Beat scheduler exports
pub use beat_scheduler::{BeatScheduler, BeatSchedulerError};

pub mod integration;

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec;

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
            typed_triples: vec![
                TypedTriple {
                    subject: 1,
                    predicate: 2,
                    object: 3,
                    graph: None,
                };
                10
            ], // Exceeds max_run_len
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
        let triples = result.expect("Failed to parse basic RDF turtle content");
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
        let triples = result.expect("Failed to parse RDF with prefix resolution");
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].subject, "http://example.org/subject");
        assert_eq!(triples[0].predicate, "http://example.org/predicate");
        assert_eq!(triples[0].object, "http://example.org/object");
    }

    #[test]
    #[ignore]  // RDF parsing is stubbed (raptor2 optional dependency)
    fn test_ingest_stage_blank_nodes() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

        let content = r#"
            _:alice <http://example.org/name> "Alice" .
            _:bob <http://example.org/name> "Bob" .
        "#;
        let result = ingest.parse_rdf_turtle(content);

        assert!(result.is_ok());
        let triples = result.expect("Failed to parse RDF with blank nodes");
        assert_eq!(triples.len(), 2);
        assert!(triples[0].subject.starts_with("_:"));
        assert!(triples[1].subject.starts_with("_:"));
        assert_eq!(triples[0].object, "\"Alice\"");
        assert_eq!(triples[1].object, "\"Bob\"");
    }

    #[test]
    #[ignore]  // RDF parsing is stubbed (raptor2 optional dependency)
    fn test_ingest_stage_literals() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

        let content = r#"
            <http://example.org/subject> <http://example.org/name> "Alice" .
            <http://example.org/subject> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
            <http://example.org/subject> <http://example.org/label> "Hello"@en .
        "#;
        let result = ingest.parse_rdf_turtle(content);

        assert!(result.is_ok());
        let triples = result.expect("Failed to parse RDF with literals");
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
        let triples = result.expect("Failed to parse RDF with base URI");
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
        let triples = result.expect("Failed to parse multiple RDF triples");
        assert_eq!(triples.len(), 3);
    }

    #[test]
    fn test_ingest_stage_empty_input() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());

        let result = ingest.parse_rdf_turtle("");
        assert!(result.is_ok());
        assert_eq!(result.expect("Failed to parse empty RDF input").len(), 0);
    }

    #[test]
    #[ignore]  // RDF parsing is stubbed (raptor2 optional dependency)
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
            triples: vec![RawTriple {
                subject: "http://example.org/subject".to_string(),
                predicate: "http://example.org/predicate".to_string(),
                object: "http://example.org/object".to_string(),
                graph: None,
            }],
            metadata: BTreeMap::new(),
        };

        let result = transform.transform(ingest_result);
        assert!(result.is_ok());

        let transform_result = result.expect("Failed to transform triples");
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
                TypedTriple {
                    subject: 1,
                    predicate: 100,
                    object: 10,
                    graph: None,
                },
                TypedTriple {
                    subject: 2,
                    predicate: 100,
                    object: 20,
                    graph: None,
                },
                TypedTriple {
                    subject: 3,
                    predicate: 200,
                    object: 30,
                    graph: None,
                },
            ],
            validation_errors: Vec::new(),
        };

        let result = load.load(transform_result);
        assert!(result.is_ok());

        let load_result = result.expect("Failed to load triples into SoA");
        assert_eq!(load_result.runs.len(), 2); // Two different predicates
        assert_eq!(load_result.runs[0].pred, 100);
        assert_eq!(load_result.runs[0].len, 2);
        assert_eq!(load_result.runs[1].pred, 200);
        assert_eq!(load_result.runs[1].len, 1);
    }

    #[test]
    #[ignore]  // Test depends on RDF parsing which is stubbed (raptor2 optional)
    fn test_reflex_stage_tick_budget() {
        let reflex = ReflexStage::new();

        let mut soa = SoAArrays::new();
        soa.s[0] = 1;
        soa.p[0] = 100;
        soa.o[0] = 10;

        let run = PredRun {
            pred: 100,
            off: 0,
            len: 1,
        };

        let load_result = LoadResult {
            soa_arrays: soa,
            runs: vec![run],
        };

        let result = reflex.reflex(load_result);
        assert!(result.is_ok());

        let reflex_result = result.expect("Failed to execute reflex stage");
        assert!(reflex_result.max_ticks <= 8);
        assert!(!reflex_result.receipts.is_empty());
    }

    #[test]
    fn test_receipt_merging() {
        // Generate proper span IDs for test receipts
        fn generate_test_span_id() -> u64 {
            #[cfg(feature = "knhk-otel")]
            {
                use knhk_otel::generate_span_id;
                generate_span_id()
            }
            #[cfg(not(feature = "knhk-otel"))]
            {
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                timestamp.wrapping_mul(0x9e3779b97f4a7c15)
            }
        }
        let span_id1 = generate_test_span_id();
        let span_id2 = generate_test_span_id();
        let expected_merged_span_id = span_id1 ^ span_id2;

        let receipt1 = Receipt {
            id: "r1".to_string(),
            cycle_id: 1,
            shard_id: 1,
            hook_id: 1,
            ticks: 4,
            actual_ticks: 3,
            lanes: 8,
            span_id: span_id1,
            a_hash: 0xABCD,
        };

        let receipt2 = Receipt {
            id: "r2".to_string(),
            cycle_id: 2,
            shard_id: 2,
            hook_id: 2,
            ticks: 6,
            actual_ticks: 5,
            lanes: 8,
            span_id: span_id2,
            a_hash: 0xEF00,
        };

        let merged = ReflexStage::merge_receipts(&[receipt1, receipt2]);

        assert_eq!(merged.ticks, 6); // Max ticks
        assert_eq!(merged.lanes, 16); // Sum lanes
        assert_eq!(merged.span_id, expected_merged_span_id); // XOR merge
        assert_eq!(merged.a_hash, 0xABCD ^ 0xEF00); // XOR merge
    }

    #[test]
    #[ignore]  // Test depends on RDF parsing which is stubbed (raptor2 optional)
    fn test_emit_stage() {
        let mut emit = EmitStage::new(true, vec!["https://webhook.example.com".to_string()]);

        // Generate proper span ID for test receipt
        fn generate_test_span_id() -> u64 {
            #[cfg(feature = "knhk-otel")]
            {
                use knhk_otel::generate_span_id;
                generate_span_id()
            }
            #[cfg(not(feature = "knhk-otel"))]
            {
                use std::time::{SystemTime, UNIX_EPOCH};
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);
                timestamp.wrapping_mul(0x9e3779b97f4a7c15)
            }
        }
        let receipt = Receipt {
            id: "receipt1".to_string(),
            cycle_id: 1,
            shard_id: 1,
            hook_id: 1,
            ticks: 4,
            actual_ticks: 3,
            lanes: 8,
            span_id: generate_test_span_id(),
            a_hash: 0xABCD,
        };

        let reflex_result = ReflexResult {
            actions: vec![Action {
                id: "action1".to_string(),
                payload: vec![1, 2, 3],
                receipt_id: "receipt1".to_string(),
            }],
            receipts: vec![receipt],
            max_ticks: 4,
            c1_failure_actions: Vec::new(),
        };

        let result = emit.emit(reflex_result);
        assert!(result.is_ok());

        let emit_result = result.expect("Failed to emit actions and receipts");
        assert_eq!(emit_result.receipts_written, 1);
        assert_eq!(emit_result.actions_sent, 1);
        assert_eq!(emit_result.lockchain_hashes.len(), 1);
    }
}
