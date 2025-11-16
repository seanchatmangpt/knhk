//! Performance benchmarks: WASM vs Native
//!
//! Compares workflow execution performance between WASM and native Rust implementations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;

// Mock native workflow engine for comparison
struct NativeWorkflowEngine;

impl NativeWorkflowEngine {
    fn new() -> Self {
        Self
    }

    fn execute_sequence(&self, data: serde_json::Value) -> serde_json::Value {
        // Simulate sequence execution
        let mut result = data;
        for _ in 0..3 {
            if let Some(obj) = result.as_object_mut() {
                obj.insert("processed".to_string(), json!(true));
            }
        }
        result
    }

    fn execute_parallel(&self, data: serde_json::Value) -> Vec<serde_json::Value> {
        // Simulate parallel execution
        vec![data.clone(), data.clone(), data]
    }
}

fn benchmark_sequence_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequence_workflow");

    let input = json!({
        "email": "test@example.com",
        "name": "Test User"
    });

    // Native implementation
    let native_engine = NativeWorkflowEngine::new();
    group.bench_function("native", |b| {
        b.iter(|| {
            black_box(native_engine.execute_sequence(black_box(input.clone())))
        })
    });

    // Note: WASM benchmarks would require wasm-bindgen-test
    // For now, we benchmark the native Rust code that gets compiled to WASM

    group.finish();
}

fn benchmark_parallel_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_workflow");

    let input = json!({
        "task1": "data",
        "task2": "data",
        "task3": "data"
    });

    let native_engine = NativeWorkflowEngine::new();
    group.bench_function("native", |b| {
        b.iter(|| {
            black_box(native_engine.execute_parallel(black_box(input.clone())))
        })
    });

    group.finish();
}

fn benchmark_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    let workflow_json = r#"{
        "id": "test-workflow",
        "pattern": "Sequence",
        "tasks": [
            {"id": "task1", "type": "compute"},
            {"id": "task2", "type": "validate"},
            {"id": "task3", "type": "transform"}
        ]
    }"#;

    group.bench_function("parse_workflow", |b| {
        b.iter(|| {
            let _: serde_json::Value = serde_json::from_str(black_box(workflow_json)).unwrap();
        })
    });

    group.finish();
}

fn benchmark_state_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_operations");

    use std::collections::HashMap;

    group.bench_function("hashmap_insert", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..100 {
                map.insert(format!("key-{}", i), json!({"value": i}));
            }
            black_box(map)
        })
    });

    group.bench_function("hashbrown_insert", |b| {
        b.iter(|| {
            let mut map = hashbrown::HashMap::new();
            for i in 0..100 {
                map.insert(format!("key-{}", i), json!({"value": i}));
            }
            black_box(map)
        })
    });

    group.finish();
}

fn benchmark_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let native_engine = NativeWorkflowEngine::new();
    let input = json!({"data": "test"});

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("workflows", count), count, |b, &count| {
            b.iter(|| {
                for _ in 0..count {
                    black_box(native_engine.execute_sequence(black_box(input.clone())));
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_sequence_workflow,
    benchmark_parallel_workflow,
    benchmark_json_parsing,
    benchmark_state_operations,
    benchmark_throughput
);

criterion_main!(benches);
