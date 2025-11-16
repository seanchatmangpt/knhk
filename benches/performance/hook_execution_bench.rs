///! Hook Execution Benchmarks
//! Measures performance of hook execution and coordination overhead

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

/// Simulated hook types
#[derive(Debug, Clone)]
enum HookType {
    PreTask,
    PostTask,
    PreEdit,
    PostEdit,
    SessionStart,
    SessionEnd,
}

/// Simulated hook context
struct HookContext {
    hook_type: HookType,
    metadata: Vec<(String, String)>,
}

impl HookContext {
    fn new(hook_type: HookType) -> Self {
        Self {
            hook_type,
            metadata: Vec::new(),
        }
    }

    fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.push((key, value));
    }

    fn execute(&self) -> HookResult {
        // Simulated hook execution
        let overhead = match self.hook_type {
            HookType::PreTask => 10,
            HookType::PostTask => 15,
            HookType::PreEdit => 5,
            HookType::PostEdit => 20,
            HookType::SessionStart => 50,
            HookType::SessionEnd => 30,
        };

        HookResult {
            success: true,
            duration_ns: overhead * 1000,
        }
    }
}

struct HookResult {
    success: bool,
    duration_ns: u64,
}

fn benchmark_pre_task_hook(c: &mut Criterion) {
    c.bench_function("hook_pre_task", |b| {
        b.iter(|| {
            let mut ctx = HookContext::new(HookType::PreTask);
            ctx.add_metadata("task_id".to_string(), "task_123".to_string());
            ctx.add_metadata("description".to_string(), "test task".to_string());
            black_box(ctx.execute())
        });
    });
}

fn benchmark_post_task_hook(c: &mut Criterion) {
    c.bench_function("hook_post_task", |b| {
        b.iter(|| {
            let mut ctx = HookContext::new(HookType::PostTask);
            ctx.add_metadata("task_id".to_string(), "task_123".to_string());
            ctx.add_metadata("result".to_string(), "success".to_string());
            black_box(ctx.execute())
        });
    });
}

fn benchmark_pre_edit_hook(c: &mut Criterion) {
    c.bench_function("hook_pre_edit", |b| {
        b.iter(|| {
            let mut ctx = HookContext::new(HookType::PreEdit);
            ctx.add_metadata("file".to_string(), "test.rs".to_string());
            black_box(ctx.execute())
        });
    });
}

fn benchmark_post_edit_hook(c: &mut Criterion) {
    c.bench_function("hook_post_edit", |b| {
        b.iter(|| {
            let mut ctx = HookContext::new(HookType::PostEdit);
            ctx.add_metadata("file".to_string(), "test.rs".to_string());
            ctx.add_metadata("memory_key".to_string(), "swarm/agent/step".to_string());
            black_box(ctx.execute())
        });
    });
}

fn benchmark_hook_with_varying_metadata(c: &mut Criterion) {
    let mut group = c.benchmark_group("hook_metadata_scaling");

    for metadata_count in [0, 1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(metadata_count),
            metadata_count,
            |b, &count| {
                b.iter(|| {
                    let mut ctx = HookContext::new(HookType::PostTask);
                    for i in 0..count {
                        ctx.add_metadata(
                            format!("key_{}", i),
                            format!("value_{}", i),
                        );
                    }
                    black_box(ctx.execute())
                });
            },
        );
    }

    group.finish();
}

fn benchmark_hook_overhead(c: &mut Criterion) {
    c.bench_function("hook_total_overhead", |b| {
        b.iter(|| {
            // Simulate complete hook lifecycle
            let pre_ctx = HookContext::new(HookType::PreTask);
            let pre_result = pre_ctx.execute();

            // Simulated work
            let _work_result = black_box(42 * 42);

            let mut post_ctx = HookContext::new(HookType::PostTask);
            post_ctx.add_metadata("result".to_string(), "success".to_string());
            let post_result = post_ctx.execute();

            black_box((pre_result, post_result))
        });
    });
}

criterion_group! {
    name = hook_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(1000);
    targets = benchmark_pre_task_hook,
              benchmark_post_task_hook,
              benchmark_pre_edit_hook,
              benchmark_post_edit_hook,
              benchmark_hook_with_varying_metadata,
              benchmark_hook_overhead
}

criterion_main!(hook_benches);
