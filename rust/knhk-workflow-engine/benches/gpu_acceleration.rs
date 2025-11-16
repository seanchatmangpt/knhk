//! GPU acceleration benchmarks
//!
//! Compares GPU vs CPU performance for workflow operations.
//!
//! Run with: cargo bench --features gpu --bench gpu_acceleration

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::gpu::*;
use tokio::runtime::Runtime;

/// Create sample workflow data
fn create_workflows(count: usize) -> Vec<WorkflowData> {
    (0..count)
        .map(|i| WorkflowData {
            id: i as u64,
            state: (i % 10) as u32,
            flags: (i % 256) as u32,
            data_ptr: 0,
        })
        .collect()
}

/// Create sample pattern data
fn create_patterns(count: usize) -> Vec<PatternData> {
    (0..count)
        .map(|i| PatternData {
            id: i as u32,
            pattern_type: (i % 43) as u32,
            criteria: (1u64 << (i % 64)),
        })
        .collect()
}

/// Create sample transitions
fn create_transitions(count: usize) -> Vec<TransitionData> {
    (0..count)
        .map(|i| TransitionData {
            from_state: (i % 10) as u32,
            to_state: ((i + 1) % 10) as u32,
            condition: 0,
            flags: 0,
        })
        .collect()
}

/// Create sample graph
fn create_graph(node_count: usize, edges_per_node: usize) -> GraphData {
    let mut edges = Vec::new();
    let mut offsets = vec![0];

    for i in 0..node_count {
        let start_offset = edges.len();
        for j in 1..=edges_per_node {
            let target = (i + j) % node_count;
            edges.push(target as u32);
        }
        offsets.push(edges.len());
    }

    GraphData {
        edges,
        offsets,
        node_count,
    }
}

/// Benchmark pattern matching
fn bench_pattern_matching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let gpu_ctx = rt
        .block_on(GpuContext::new().fallback_to_cpu(true).build())
        .unwrap();

    let mut group = c.benchmark_group("pattern_matching");

    for num_workflows in [100, 1000, 10000, 100000] {
        let workflows = create_workflows(num_workflows);
        let patterns = create_patterns(10);

        group.throughput(Throughput::Elements((num_workflows * 10) as u64));

        group.bench_with_input(
            BenchmarkId::new("gpu", num_workflows),
            &(workflows.clone(), patterns.clone()),
            |b, (w, p)| {
                b.to_async(&rt).iter(|| async {
                    let result = gpu_ctx
                        .batch_pattern_match(black_box(w), black_box(p))
                        .await
                        .unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark state transitions
fn bench_state_transitions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let gpu_ctx = rt
        .block_on(GpuContext::new().fallback_to_cpu(true).build())
        .unwrap();

    let mut group = c.benchmark_group("state_transitions");

    for num_workflows in [100, 1000, 10000, 100000] {
        let workflows = create_workflows(num_workflows);
        let transitions = create_transitions(10);

        group.throughput(Throughput::Elements(num_workflows as u64));

        group.bench_with_input(
            BenchmarkId::new("gpu", num_workflows),
            &(workflows.clone(), transitions.clone()),
            |b, (w, t)| {
                b.to_async(&rt).iter(|| async {
                    let result = gpu_ctx
                        .batch_apply_transitions(black_box(w), black_box(t))
                        .await
                        .unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark graph traversal
fn bench_graph_traversal(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let gpu_ctx = rt
        .block_on(GpuContext::new().fallback_to_cpu(true).build())
        .unwrap();

    let mut group = c.benchmark_group("graph_traversal");

    for node_count in [100, 1000, 10000] {
        let graph = create_graph(node_count, 5);
        let start_nodes: Vec<usize> = (0..10).collect();

        group.throughput(Throughput::Elements(node_count as u64));

        group.bench_with_input(
            BenchmarkId::new("gpu", node_count),
            &(graph.clone(), start_nodes.clone()),
            |b, (g, s)| {
                b.to_async(&rt).iter(|| async {
                    let result = gpu_ctx
                        .parallel_graph_traversal(black_box(g), black_box(s))
                        .await
                        .unwrap();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark end-to-end workflow processing
fn bench_e2e_workflow_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let gpu_ctx = rt
        .block_on(GpuContext::new().fallback_to_cpu(true).build())
        .unwrap();

    let mut group = c.benchmark_group("e2e_workflow");

    for num_workflows in [1000, 10000, 100000] {
        let workflows = create_workflows(num_workflows);
        let patterns = create_patterns(43); // All YAWL patterns
        let transitions = create_transitions(20);

        group.throughput(Throughput::Elements(num_workflows as u64));

        group.bench_with_input(
            BenchmarkId::new("gpu", num_workflows),
            &(workflows.clone(), patterns.clone(), transitions.clone()),
            |b, (w, p, t)| {
                b.to_async(&rt).iter(|| async {
                    // Pattern matching
                    let matches = gpu_ctx
                        .batch_pattern_match(black_box(w), black_box(p))
                        .await
                        .unwrap();

                    // State transitions
                    let states = gpu_ctx
                        .batch_apply_transitions(black_box(w), black_box(t))
                        .await
                        .unwrap();

                    black_box((matches, states))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark GPU vs CPU speedup
fn bench_speedup_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // CPU context
    let cpu_ctx = rt
        .block_on(
            GpuContext::new()
                .prefer_device(DeviceType::Cpu)
                .build(),
        )
        .unwrap();

    // GPU context (may fallback to CPU if no GPU available)
    let gpu_ctx = rt
        .block_on(GpuContext::new().fallback_to_cpu(true).build())
        .unwrap();

    let mut group = c.benchmark_group("speedup_comparison");
    let num_workflows = 100000;
    let workflows = create_workflows(num_workflows);
    let patterns = create_patterns(10);

    group.throughput(Throughput::Elements((num_workflows * 10) as u64));

    group.bench_function("cpu", |b| {
        b.to_async(&rt).iter(|| async {
            let result = cpu_ctx
                .batch_pattern_match(black_box(&workflows), black_box(&patterns))
                .await
                .unwrap();
            black_box(result)
        });
    });

    group.bench_function("gpu", |b| {
        b.to_async(&rt).iter(|| async {
            let result = gpu_ctx
                .batch_pattern_match(black_box(&workflows), black_box(&patterns))
                .await
                .unwrap();
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_matching,
    bench_state_transitions,
    bench_graph_traversal,
    bench_e2e_workflow_processing,
    bench_speedup_comparison,
);

criterion_main!(benches);
