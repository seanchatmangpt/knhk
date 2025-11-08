//! Benchmark warm path queries (oxigraph)
//! Compare with cold path baseline (unrdf)
//! Validate ≤500ms warm path target

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_warm::{execute_ask, execute_construct, execute_select, WarmPathGraph};
use std::time::Instant;

fn bench_warm_path_select(c: &mut Criterion) {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Load test data
    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
        <s3> <p2> <o1> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o }";

    c.bench_function("warm_path_select", |b| {
        b.iter(|| {
            let result = execute_select(black_box(&graph), black_box(query));
            black_box(result)
        })
    });
}

fn bench_warm_path_ask(c: &mut Criterion) {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let test_data = r#"
        <s1> <p1> <o1> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "ASK { <s1> <p1> <o1> }";

    c.bench_function("warm_path_ask", |b| {
        b.iter(|| {
            let result = execute_ask(black_box(&graph), black_box(query));
            black_box(result)
        })
    });
}

fn bench_warm_path_construct(c: &mut Criterion) {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "CONSTRUCT { ?s <p1> <o1> } WHERE { ?s <p1> ?o }";

    c.bench_function("warm_path_construct", |b| {
        b.iter(|| {
            let result = execute_construct(black_box(&graph), black_box(query));
            black_box(result)
        })
    });
}

fn bench_cache_hit_performance(c: &mut Criterion) {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    let test_data = r#"
        <s1> <p1> <o1> .
        <s2> <p1> <o2> .
    "#;
    graph
        .load_from_turtle(test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s WHERE { ?s <p1> ?o }";

    // Warm up cache
    execute_select(&graph, query).expect("Query failed");

    c.bench_function("warm_path_cache_hit", |b| {
        b.iter(|| {
            let result = execute_select(black_box(&graph), black_box(query));
            black_box(result)
        })
    });
}

fn validate_500ms_target(c: &mut Criterion) {
    let graph = WarmPathGraph::new().expect("Failed to create graph");

    // Load larger dataset for realistic testing
    let mut test_data = String::new();
    for i in 0..1000 {
        test_data.push_str(&format!("<s{}> <p1> <o{}> .\n", i, i));
    }
    graph
        .load_from_turtle(&test_data)
        .expect("Failed to load data");

    let query = "SELECT ?s ?o WHERE { ?s <p1> ?o } LIMIT 100";

    c.bench_function("warm_path_large_dataset", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::from_secs(0);

            for _ in 0..iters {
                let start = Instant::now();
                let result = execute_select(&graph, query);
                let duration = start.elapsed();
                total_duration += duration;

                // Validate result
                assert!(result.is_ok(), "Query should succeed");

                // Validate latency target (≤500ms)
                assert!(
                    duration.as_millis() <= 500,
                    "Query exceeded 500ms target: {}ms",
                    duration.as_millis()
                );
            }

            total_duration
        })
    });
}

criterion_group!(
    benches,
    bench_warm_path_select,
    bench_warm_path_ask,
    bench_warm_path_construct,
    bench_cache_hit_performance,
    validate_500ms_target
);
criterion_main!(benches);
