//! Benchmarks for Self-Executing Workflow System
//!
//! Measures performance of key components:
//! - Ontology parsing and execution
//! - Pattern selection (adaptive vs static)
//! - MAPE-K cycle performance
//! - Receipt generation
//! - Snapshot versioning

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_workflow_engine::ontology_executor::OntologyExecutor;
use knhk_workflow_engine::adaptive_patterns::{AdaptivePatternSelector, PatternSelectionContext};
use knhk_workflow_engine::mape::KnowledgeBase;
use knhk_workflow_engine::patterns::PatternId;
use serde_json::json;
use std::sync::Arc;
use parking_lot::RwLock;
use tempfile::TempDir;

fn benchmark_pattern_selection(c: &mut Criterion) {
    let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
    let mut selector = AdaptivePatternSelector::new(knowledge);

    let mut group = c.benchmark_group("pattern_selection");

    for data_size in [10, 100, 1000, 10000].iter() {
        let context = PatternSelectionContext {
            data_size: *data_size,
            concurrency_level: 4,
            requires_parallelism: *data_size > 100,
            requires_exclusive_choice: false,
            max_ticks: 8,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(data_size),
            data_size,
            |b, _| {
                b.iter(|| {
                    selector.select_pattern(black_box(&context))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_receipt_generation(c: &mut Criterion) {
    use knhk_workflow_engine::receipts::ReceiptGenerator;

    let generator = ReceiptGenerator::new();
    let input = json!({"test": "data"});
    let output = json!({"result": "success"});

    c.bench_function("receipt_generation", |b| {
        b.iter(|| {
            generator.generate(
                black_box("sigma-v1"),
                black_box(&input),
                black_box(&output),
                black_box(5),
                black_box(&[]),
                black_box(&[]),
            )
        });
    });
}

fn benchmark_snapshot_operations(c: &mut Criterion) {
    use knhk_workflow_engine::snapshots::SnapshotVersioning;

    let temp_dir = TempDir::new().unwrap();
    let versioning = SnapshotVersioning::new(temp_dir.path().to_str().unwrap());

    let content = "test ontology content";
    let config = std::collections::HashMap::new();

    c.bench_function("snapshot_create", |b| {
        b.iter(|| {
            versioning.create_snapshot(
                black_box(content),
                black_box(&config),
            )
        });
    });
}

fn benchmark_guard_validation(c: &mut Criterion) {
    use knhk_workflow_engine::guards::InvariantChecker;
    use knhk_workflow_engine::parser::WorkflowSpec;

    let checker = InvariantChecker::new();
    let spec = WorkflowSpec {
        id: "test".to_string(),
        name: "Test Workflow".to_string(),
        version: "1.0.0".to_string(),
        tasks: vec![],
        data_inputs: vec![],
        data_outputs: vec![],
        patterns: vec![],
    };

    c.bench_function("guard_validation", |b| {
        b.iter(|| {
            checker.validate_workflow_spec(black_box(&spec))
        });
    });
}

fn benchmark_hook_execution(c: &mut Criterion) {
    use knhk_workflow_engine::engine::HookEngine;

    let engine = HookEngine::new();

    c.bench_function("hook_before_execution", |b| {
        b.iter(|| {
            engine.before_hook(
                black_box("task-1"),
                black_box(&json!({"data": "test"})),
            )
        });
    });
}

fn benchmark_pattern_library_lookup(c: &mut Criterion) {
    use knhk_workflow_engine::engine::PatternLibrary;

    let library = PatternLibrary::new();

    let mut group = c.benchmark_group("pattern_lookup");

    for pattern in &[
        PatternId::Sequence,
        PatternId::ParallelSplit,
        PatternId::ExclusiveChoice,
        PatternId::MultiChoice,
    ] {
        group.bench_with_input(
            BenchmarkId::new("pattern", format!("{:?}", pattern)),
            pattern,
            |b, p| {
                b.iter(|| {
                    library.get_pattern(black_box(p))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_pattern_selection,
    benchmark_receipt_generation,
    benchmark_snapshot_operations,
    benchmark_guard_validation,
    benchmark_hook_execution,
    benchmark_pattern_library_lookup,
);

criterion_main!(benches);
