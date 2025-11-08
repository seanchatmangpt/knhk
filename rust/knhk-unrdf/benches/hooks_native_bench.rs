// knhk-unrdf: Benchmarks for native hooks engine
// Performance benchmarks using criterion
#![allow(clippy::expect_used)]

#[cfg(feature = "native")]
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
#[cfg(feature = "native")]
use knhk_unrdf::hooks_native::*;
#[cfg(feature = "native")]
use knhk_unrdf::types::HookDefinition;
#[cfg(feature = "native")]
use rand::Rng;
#[cfg(feature = "native")]
use serde_json::Value as JsonValue;

#[cfg(feature = "native")]
fn generate_test_data(count: usize) -> String {
    let mut turtle = String::from(
        "@prefix ex: <http://example.org/> .\n@prefix foaf: <http://xmlns.com/foaf/0.1/> .\n\n",
    );
    for i in 0..count {
        turtle.push_str(&format!("ex:person{} foaf:name \"Person {}\" .\n", i, i));
    }
    turtle
}

#[cfg(feature = "native")]
fn generate_hook(id: usize, query: &str) -> HookDefinition {
    HookDefinition {
        id: format!("hook-{}", id),
        name: format!("Hook {}", id),
        hook_type: "sparql-ask".to_string(),
        definition: {
            let mut def = serde_json::Map::new();
            let mut when = serde_json::Map::new();
            when.insert(
                "kind".to_string(),
                JsonValue::String("sparql-ask".to_string()),
            );
            when.insert("query".to_string(), JsonValue::String(query.to_string()));
            def.insert("when".to_string(), JsonValue::Object(when));
            JsonValue::Object(def)
        },
    }
}

#[cfg(feature = "native")]
fn benchmark_single_hook_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_hook_execution");

    // Benchmark with different data sizes
    for size in [10, 100, 1000, 10000].iter() {
        let turtle_data = generate_test_data(*size);
        let hook = generate_hook(1, "ASK { ?s ?p ?o }");

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_triples", size)),
            &(hook, turtle_data),
            |b, (hook, data)| {
                b.iter(|| {
                    black_box(evaluate_hook_native(black_box(hook), black_box(data))).unwrap()
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "native")]
fn benchmark_batch_hook_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_hook_evaluation");

    // Benchmark with different batch sizes
    for batch_size in [1, 10, 100, 1000].iter() {
        let turtle_data = generate_test_data(1000);
        let hooks: Vec<HookDefinition> = (0..*batch_size)
            .map(|i| generate_hook(i, "ASK { ?s ?p ?o }"))
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_hooks", batch_size)),
            &hooks,
            |b, hooks| {
                b.iter(|| {
                    black_box(evaluate_hooks_batch_native(
                        black_box(hooks),
                        black_box(&turtle_data),
                    ))
                    .unwrap()
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "native")]
fn benchmark_hook_registry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_registry_operations");

    let registry = NativeHookRegistry::new();

    // Benchmark register
    group.bench_function("register", |b| {
        let mut rng = rand::thread_rng();
        b.iter(|| {
            let mut hook = generate_hook(1, "ASK { ?s ?p ?o }");
            hook.id = format!("hook-{}", black_box(rng.gen::<u64>()));
            black_box(registry.register(black_box(hook))).unwrap();
        });
    });

    // Benchmark get
    let hook = generate_hook(1, "ASK { ?s ?p ?o }");
    registry.register(hook.clone()).unwrap();
    group.bench_function("get", |b| {
        b.iter(|| {
            black_box(registry.get(black_box("hook-1"))).unwrap();
        });
    });

    // Benchmark list
    group.bench_function("list", |b| {
        b.iter(|| {
            black_box(registry.list()).unwrap();
        });
    });

    group.finish();
}

#[cfg(feature = "native")]
fn benchmark_hook_receipt_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_receipt_generation");

    let turtle_data = generate_test_data(100);
    let hook = generate_hook(1, "ASK { ?s ?p ?o }");

    group.bench_function("generate_receipt", |b| {
        b.iter(|| {
            let result = black_box(evaluate_hook_native(
                black_box(&hook),
                black_box(&turtle_data),
            ))
            .unwrap();
            black_box(result.receipt);
        });
    });

    group.finish();
}

#[cfg(feature = "native")]
fn benchmark_query_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_complexity");

    let turtle_data = generate_test_data(1000);

    let queries = vec![
        ("simple", "ASK { ?s ?p ?o }"),
        ("with_filter", "ASK { ?s ?p ?o . FILTER(?o > \"100\") }"),
        ("multi_pattern", "ASK { ?s ?p ?o . ?s ?p2 ?o2 }"),
        (
            "complex",
            "ASK { ?s ?p ?o . ?s ?p2 ?o2 . FILTER(?o != ?o2) }",
        ),
    ];

    for (name, query) in queries {
        let hook = generate_hook(1, query);
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(evaluate_hook_native(
                    black_box(&hook),
                    black_box(&turtle_data),
                ))
                .unwrap();
            });
        });
    }

    group.finish();
}

#[cfg(feature = "native")]
criterion_group!(
    benches,
    benchmark_single_hook_execution,
    benchmark_batch_hook_evaluation,
    benchmark_hook_registry_operations,
    benchmark_hook_receipt_generation,
    benchmark_query_complexity
);

#[cfg(feature = "native")]
criterion_main!(benches);
