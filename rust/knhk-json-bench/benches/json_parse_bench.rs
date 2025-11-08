//! JSON parsing benchmarks using knhk framework

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use knhk_json_bench::parse_json;

const SIMPLE_JSON: &str = r#"{"key": "value", "number": 42}"#;
const NESTED_JSON: &str = r#"{"array": [1, 2, 3, 4, 5], "object": {"nested": true}}"#;
const TWITTER_LIKE: &str = r#"{
    "statuses": [
        {"text": "First tweet", "user": {"screen_name": "user1"}, "retweet_count": 10},
        {"text": "Second tweet", "user": {"screen_name": "user2"}, "retweet_count": 20}
    ]
}"#;

fn bench_knhk_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_json");

    group.bench_function(BenchmarkId::new("knhk", "simple"), |b| {
        b.iter(|| {
            let result = parse_json(SIMPLE_JSON.as_bytes().to_vec()).unwrap();
            black_box(result);
        });
    });

    group.bench_function(BenchmarkId::new("knhk", "nested"), |b| {
        b.iter(|| {
            let result = parse_json(NESTED_JSON.as_bytes().to_vec()).unwrap();
            black_box(result);
        });
    });

    group.bench_function(BenchmarkId::new("knhk", "twitter"), |b| {
        b.iter(|| {
            let result = parse_json(TWITTER_LIKE.as_bytes().to_vec()).unwrap();
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_knhk_parse);
criterion_main!(benches);
