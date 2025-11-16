// benches/trace_counterfactual_benchmark.rs
//! Performance benchmarks for trace indexing and counterfactual analysis
//!
//! Measures:
//! - TraceId generation (hot path cost)
//! - Trace storage and retrieval
//! - Replay execution
//! - Counterfactual simulation
//! - Action diff computation

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_workflow_engine::autonomic::{
    CounterfactualEngine, CounterfactualScenario, DoctrineConfig, ExecutionTrace,
    Goal, GoalType, KnowledgeBase, MonitorEvent, ObservableSegment,
    OntologySnapshot, TraceId, TraceStorage,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark TraceId generation (hot path)
fn bench_trace_id_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("trace_id_generation", |b| {
        let o_segment = ObservableSegment::new(1000, 2000);
        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500,
        };
        let q = DoctrineConfig::default();

        b.iter(|| {
            let trace_id = TraceId::new(
                black_box(&o_segment),
                black_box(&sigma),
                black_box(&q),
            )
            .unwrap();
            black_box(trace_id);
        });
    });
}

/// Benchmark trace storage operations
fn bench_trace_storage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("trace_storage");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let storage = TraceStorage::new(size);

                    let o_segment = ObservableSegment::new(1000, 2000);
                    let sigma = OntologySnapshot {
                        goals: Vec::new(),
                        rules: Vec::new(),
                        facts: HashMap::new(),
                        policies: Vec::new(),
                        timestamp_ms: 1500,
                    };
                    let q = DoctrineConfig::default();

                    let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
                    let trace_id = trace.id;

                    // Store
                    storage.store(black_box(trace)).await.unwrap();

                    // Retrieve
                    let retrieved = storage.retrieve(black_box(&trace_id)).await.unwrap();
                    black_box(retrieved);
                });
            });
        });
    }

    group.finish();
}

/// Benchmark trace lookup performance
fn bench_trace_lookup(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("trace_lookup", |b| {
        let storage = rt.block_on(async {
            let storage = Arc::new(TraceStorage::new(100));

            // Pre-populate with 50 traces
            for i in 0..50 {
                let mut o_segment = ObservableSegment::new(1000 + i * 100, 2000 + i * 100);
                o_segment.metadata.insert("index".to_string(), i.to_string());

                let sigma = OntologySnapshot {
                    goals: Vec::new(),
                    rules: Vec::new(),
                    facts: HashMap::new(),
                    policies: Vec::new(),
                    timestamp_ms: 1500 + i * 100,
                };
                let q = DoctrineConfig::default();

                let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
                storage.store(trace).await.unwrap();
            }

            storage
        });

        let trace_ids: Vec<TraceId> = rt.block_on(async {
            storage.all_trace_ids().await
        });

        b.iter(|| {
            rt.block_on(async {
                for trace_id in &trace_ids {
                    let retrieved = storage.retrieve(black_box(trace_id)).await.unwrap();
                    black_box(retrieved);
                }
            });
        });
    });
}

/// Benchmark ontology snapshot creation
fn bench_ontology_snapshot(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("ontology_snapshot");

    for goal_count in [1, 10, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(goal_count),
            goal_count,
            |b, &goal_count| {
                b.iter(|| {
                    rt.block_on(async {
                        let kb = KnowledgeBase::new();

                        // Add goals
                        for i in 0..goal_count {
                            let goal = Goal::new(
                                format!("goal_{}", i),
                                GoalType::Performance,
                                format!("metric_{}", i),
                                100.0,
                            );
                            kb.add_goal(goal).await.unwrap();
                        }

                        let snapshot = OntologySnapshot::from_knowledge_base(black_box(&kb)).await;
                        black_box(snapshot);
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark replay execution
fn bench_replay_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("replay_execution", |b| {
        let (storage, trace_id) = rt.block_on(async {
            let storage = Arc::new(TraceStorage::new(10));

            let mut o_segment = ObservableSegment::new(1000, 2000);
            o_segment.add_event(MonitorEvent::new(
                "latency".to_string(),
                150.0,
                "monitor".to_string(),
            ));

            let kb = KnowledgeBase::new();
            let goal = Goal::new(
                "latency_goal".to_string(),
                GoalType::Performance,
                "latency".to_string(),
                100.0,
            );
            kb.add_goal(goal).await.unwrap();

            let sigma = OntologySnapshot::from_knowledge_base(&kb).await;
            let q = DoctrineConfig::default();

            let trace = ExecutionTrace::new(o_segment, sigma, q).unwrap();
            let trace_id = trace.id;
            storage.store(trace).await.unwrap();

            (storage, trace_id)
        });

        let engine = CounterfactualEngine::new(storage);

        b.iter(|| {
            rt.block_on(async {
                let scenario = CounterfactualScenario::replay(black_box(trace_id));
                let result = engine.execute(black_box(scenario)).await.unwrap();
                black_box(result);
            });
        });
    });
}

/// Benchmark counterfactual simulation
fn bench_counterfactual_simulation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("counterfactual_simulation", |b| {
        let (storage, trace_id, sigma2) = rt.block_on(async {
            let storage = Arc::new(TraceStorage::new(10));

            let mut o_segment = ObservableSegment::new(1000, 2000);
            o_segment.add_event(MonitorEvent::new(
                "latency".to_string(),
                150.0,
                "monitor".to_string(),
            ));

            // Original ontology
            let kb1 = KnowledgeBase::new();
            let goal1 = Goal::new(
                "strict_latency".to_string(),
                GoalType::Performance,
                "latency".to_string(),
                100.0,
            );
            kb1.add_goal(goal1).await.unwrap();

            let sigma1 = OntologySnapshot::from_knowledge_base(&kb1).await;
            let q = DoctrineConfig::default();

            let trace = ExecutionTrace::new(o_segment, sigma1, q).unwrap();
            let trace_id = trace.id;
            storage.store(trace).await.unwrap();

            // Alternative ontology
            let kb2 = KnowledgeBase::new();
            let goal2 = Goal::new(
                "relaxed_latency".to_string(),
                GoalType::Performance,
                "latency".to_string(),
                200.0,
            );
            kb2.add_goal(goal2).await.unwrap();

            let sigma2 = OntologySnapshot::from_knowledge_base(&kb2).await;

            (storage, trace_id, sigma2)
        });

        let engine = CounterfactualEngine::new(storage);

        b.iter(|| {
            rt.block_on(async {
                let scenario = CounterfactualScenario::with_ontology(
                    black_box(trace_id),
                    black_box(sigma2.clone()),
                    "Test scenario".to_string(),
                );
                let result = engine.execute(black_box(scenario)).await.unwrap();
                black_box(result);
            });
        });
    });
}

/// Benchmark trace ID hex encoding
fn bench_trace_id_hex_encoding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let trace_id = rt.block_on(async {
        let o_segment = ObservableSegment::new(1000, 2000);
        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500,
        };
        let q = DoctrineConfig::default();

        TraceId::new(&o_segment, &sigma, &q).unwrap()
    });

    c.bench_function("trace_id_to_hex", |b| {
        b.iter(|| {
            let hex = black_box(&trace_id).to_hex();
            black_box(hex);
        });
    });

    let hex = trace_id.to_hex();

    c.bench_function("trace_id_from_hex", |b| {
        b.iter(|| {
            let parsed = TraceId::from_hex(black_box(&hex)).unwrap();
            black_box(parsed);
        });
    });
}

criterion_group!(
    benches,
    bench_trace_id_generation,
    bench_trace_storage,
    bench_trace_lookup,
    bench_ontology_snapshot,
    bench_replay_execution,
    bench_counterfactual_simulation,
    bench_trace_id_hex_encoding,
);
criterion_main!(benches);
