//! JSON parsing benchmarks comparing knhk-json-bench to SimdJSON and serde_json

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_json_bench::JsonTokenizer;

const SIMPLE_JSON: &str = r#"{"key": "value", "number": 42}"#;
const NESTED_JSON: &str = r#"{"array": [1, 2, 3, 4, 5], "object": {"nested": true}}"#;
const TWITTER_LIKE: &str = r#"{
    "statuses": [
        {"text": "First tweet", "user": {"screen_name": "user1"}, "retweet_count": 10},
        {"text": "Second tweet", "user": {"screen_name": "user2"}, "retweet_count": 20}
    ]
}"#;

fn bench_knhk_tokenize(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize");

    group.bench_function(BenchmarkId::new("knhk", "simple"), |b| {
        b.iter(|| {
            let mut tokenizer = JsonTokenizer::new(SIMPLE_JSON.as_bytes().to_vec());
            let count = tokenizer.tokenize().unwrap();
            black_box(count);
        });
    });

    group.bench_function(BenchmarkId::new("knhk", "nested"), |b| {
        b.iter(|| {
            let mut tokenizer = JsonTokenizer::new(NESTED_JSON.as_bytes().to_vec());
            let count = tokenizer.tokenize().unwrap();
            black_box(count);
        });
    });

    group.bench_function(BenchmarkId::new("knhk", "twitter"), |b| {
        b.iter(|| {
            let mut tokenizer = JsonTokenizer::new(TWITTER_LIKE.as_bytes().to_vec());
            let count = tokenizer.tokenize().unwrap();
            black_box(count);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_knhk_tokenize);
criterion_main!(benches);
