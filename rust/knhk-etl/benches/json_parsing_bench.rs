// rust/knhk-etl/benches/json_parsing_bench.rs
// Performance benchmarks comparing simdjson vs serde_json vs oxigraph parsing

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_etl::ingest::IngestStage;

/// Create test JSON data with specified number of triples
fn create_test_json(num_triples: usize) -> String {
    let mut triples = Vec::new();
    for i in 0..num_triples {
        triples.push(format!(
            r#"{{"s": "http://example.org/s{}", "p": "http://example.org/p{}", "o": "http://example.org/o{}"}}"#,
            i, i, i
        ));
    }
    format!(
        r#"{{"additions": [{}], "removals": []}}"#,
        triples.join(", ")
    )
}

/// Create test Turtle data with specified number of triples
fn create_test_turtle(num_triples: usize) -> String {
    let mut triples = Vec::new();
    for i in 0..num_triples {
        triples.push(format!(
            "<http://example.org/s{}> <http://example.org/p{}> <http://example.org/o{}> .",
            i, i, i
        ));
    }
    triples.join("\n")
}

/// Benchmark simdjson parsing
fn bench_simdjson_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing_simdjson");

    for num_triples in [1, 4, 8] {
        let json_data = create_test_json(num_triples);
        let ingest = IngestStage::new(vec!["bench".to_string()], "json".to_string());

        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &json_data,
            |b, json| {
                b.iter(|| {
                    let result = ingest.parse_json_delta(black_box(json.as_bytes()));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark serde_json parsing (for comparison)
fn bench_serde_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing_serde_json");

    for num_triples in [1, 4, 8] {
        let json_data = create_test_json(num_triples);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &json_data,
            |b, json| {
                b.iter(|| {
                    let result: Result<serde_json::Value, _> =
                        serde_json::from_str(black_box(json));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark oxigraph Turtle parsing (for comparison)
fn bench_oxigraph_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("turtle_parsing_oxigraph");

    for num_triples in [1, 4, 8] {
        let turtle_data = create_test_turtle(num_triples);
        let ingest = IngestStage::new(vec!["bench".to_string()], "turtle".to_string());

        group.bench_with_input(
            BenchmarkId::from_parameter(num_triples),
            &turtle_data,
            |b, turtle| {
                b.iter(|| {
                    let result = ingest.parse_rdf_turtle(black_box(turtle));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark JSON vs Turtle parsing comparison
fn bench_json_vs_turtle(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_vs_turtle_parsing");

    for num_triples in [1, 4, 8] {
        let json_data = create_test_json(num_triples);
        let turtle_data = create_test_turtle(num_triples);
        let ingest_json = IngestStage::new(vec!["bench".to_string()], "json".to_string());
        let ingest_turtle = IngestStage::new(vec!["bench".to_string()], "turtle".to_string());

        group.bench_with_input(
            BenchmarkId::new("simdjson", num_triples),
            &json_data,
            |b, json| {
                b.iter(|| {
                    let result = ingest_json.parse_json_delta(black_box(json.as_bytes()));
                    black_box(result)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("oxigraph", num_triples),
            &turtle_data,
            |b, turtle| {
                b.iter(|| {
                    let result = ingest_turtle.parse_rdf_turtle(black_box(turtle));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simdjson_parsing,
    bench_serde_json_parsing,
    bench_oxigraph_parsing,
    bench_json_vs_turtle
);
criterion_main!(benches);
