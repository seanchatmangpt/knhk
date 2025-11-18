// knhk-yawl/benches/pattern_performance.rs
// Performance benchmarks for YAWL patterns (Chatman constant validation)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_yawl::patterns::basic::SequencePattern;
use knhk_yawl::patterns::{ExecutionContext, YawlPattern};
use serde_json::json;

fn benchmark_sequence_pattern(c: &mut Criterion) {
    let pattern = SequencePattern::new(vec![
        "task1".to_string(),
        "task2".to_string(),
        "task3".to_string(),
    ]);

    let context = ExecutionContext::new("workflow1", "seq1")
        .with_data(json!({"value": 42}));

    c.bench_function("sequence_3_tasks", |b| {
        b.iter(|| {
            pattern.execute(black_box(&context)).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_sequence_pattern);
criterion_main!(benches);
