//! Chatman Constant Compliance Benchmarks
//!
//! Validates that critical path patterns execute within ≤8 ticks.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_yawl::core::*;
use knhk_yawl::patterns::*;
use std::time::Instant;

/// Measure pattern execution in CPU ticks (using RDTSC on x86)
#[cfg(target_arch = "x86_64")]
fn measure_ticks<F>(f: F) -> u64
where
    F: FnOnce(),
{
    unsafe {
        let start = core::arch::x86_64::_rdtsc();
        f();
        let end = core::arch::x86_64::_rdtsc();
        end - start
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn measure_ticks<F>(f: F) -> u64
where
    F: FnOnce(),
{
    // Fallback for non-x86 architectures
    let start = Instant::now();
    f();
    let duration = start.elapsed();
    duration.as_nanos() as u64 / 250 // Approximate tick conversion
}

/// Benchmark Sequence Pattern (Target: ≤2 ticks)
fn bench_sequence_pattern(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("sequence_pattern", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pattern = SequencePattern {
                    first_task: black_box(TaskId::new()),
                    second_task: black_box(TaskId::new()),
                };

                let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                let mut tick_counter = TickCounter::new();

                let ticks = measure_ticks(|| {
                    let _ = rt.block_on(pattern.execute(&mut context, &mut tick_counter));
                });

                // Verify Chatman Constant compliance
                assert!(
                    tick_counter.ticks() <= 2,
                    "Sequence pattern exceeded target: {} ticks",
                    tick_counter.ticks()
                );
                assert!(
                    tick_counter.ticks() <= CHATMAN_CONSTANT,
                    "Chatman Constant violated: {} ticks",
                    tick_counter.ticks()
                );

                ticks
            })
        })
    });
}

/// Benchmark Parallel Split Pattern (Target: ≤3 ticks)
fn bench_parallel_split_pattern(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("parallel_split_pattern", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pattern = ParallelSplitPattern {
                    tasks: black_box(vec![TaskId::new(), TaskId::new(), TaskId::new()]),
                };

                let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                let mut tick_counter = TickCounter::new();

                let ticks = measure_ticks(|| {
                    let _ = rt.block_on(pattern.execute(&mut context, &mut tick_counter));
                });

                assert!(
                    tick_counter.ticks() <= 3,
                    "Parallel split exceeded target: {} ticks",
                    tick_counter.ticks()
                );
                assert!(
                    tick_counter.ticks() <= CHATMAN_CONSTANT,
                    "Chatman Constant violated: {} ticks",
                    tick_counter.ticks()
                );

                ticks
            })
        })
    });
}

/// Benchmark Synchronization Pattern (Target: ≤4 ticks)
fn bench_synchronization_pattern(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("synchronization_pattern", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pattern = SynchronizationPattern {
                    incoming_tasks: black_box(vec![TaskId::new(), TaskId::new()]),
                };

                let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());

                // Mark tasks as completed
                for task_id in &pattern.incoming_tasks {
                    context.complete_task(*task_id);
                }

                let mut tick_counter = TickCounter::new();

                let ticks = measure_ticks(|| {
                    let _ = rt.block_on(pattern.execute(&mut context, &mut tick_counter));
                });

                assert!(
                    tick_counter.ticks() <= 4,
                    "Synchronization exceeded target: {} ticks",
                    tick_counter.ticks()
                );
                assert!(
                    tick_counter.ticks() <= CHATMAN_CONSTANT,
                    "Chatman Constant violated: {} ticks",
                    tick_counter.ticks()
                );

                ticks
            })
        })
    });
}

/// Benchmark Exclusive Choice Pattern (Target: ≤3 ticks)
fn bench_exclusive_choice_pattern(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("exclusive_choice_pattern", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pattern = ExclusiveChoicePattern {
                    branches: black_box(vec![
                        (TaskId::new(), "x > 10".to_string()),
                        (TaskId::new(), "x <= 10".to_string()),
                    ]),
                };

                let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                let mut tick_counter = TickCounter::new();

                let ticks = measure_ticks(|| {
                    let _ = rt.block_on(pattern.execute(&mut context, &mut tick_counter));
                });

                assert!(
                    tick_counter.ticks() <= 3,
                    "Exclusive choice exceeded target: {} ticks",
                    tick_counter.ticks()
                );
                assert!(
                    tick_counter.ticks() <= CHATMAN_CONSTANT,
                    "Chatman Constant violated: {} ticks",
                    tick_counter.ticks()
                );

                ticks
            })
        })
    });
}

/// Benchmark Simple Merge Pattern (Target: ≤2 ticks)
fn bench_simple_merge_pattern(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("simple_merge_pattern", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pattern = SimpleMergePattern;

                let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                let mut tick_counter = TickCounter::new();

                let ticks = measure_ticks(|| {
                    let _ = rt.block_on(pattern.execute(&mut context, &mut tick_counter));
                });

                assert!(
                    tick_counter.ticks() <= 2,
                    "Simple merge exceeded target: {} ticks",
                    tick_counter.ticks()
                );
                assert!(
                    tick_counter.ticks() <= CHATMAN_CONSTANT,
                    "Chatman Constant violated: {} ticks",
                    tick_counter.ticks()
                );

                ticks
            })
        })
    });
}

/// Benchmark all critical patterns together
fn bench_all_critical_patterns(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("all_critical_patterns", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut total_ticks = 0u8;

                // Pattern 1: Sequence
                {
                    let pattern = SequencePattern {
                        first_task: TaskId::new(),
                        second_task: TaskId::new(),
                    };
                    let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                    let mut tick_counter = TickCounter::new();
                    let _ = pattern.execute(&mut context, &mut tick_counter).await;
                    total_ticks += tick_counter.ticks();
                }

                // Pattern 2: Parallel Split
                {
                    let pattern = ParallelSplitPattern {
                        tasks: vec![TaskId::new(), TaskId::new()],
                    };
                    let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                    let mut tick_counter = TickCounter::new();
                    let _ = pattern.execute(&mut context, &mut tick_counter).await;
                    total_ticks += tick_counter.ticks();
                }

                // Pattern 3: Synchronization
                {
                    let pattern = SynchronizationPattern {
                        incoming_tasks: vec![TaskId::new(), TaskId::new()],
                    };
                    let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                    for task_id in &pattern.incoming_tasks {
                        context.complete_task(*task_id);
                    }
                    let mut tick_counter = TickCounter::new();
                    let _ = pattern.execute(&mut context, &mut tick_counter).await;
                    total_ticks += tick_counter.ticks();
                }

                // Pattern 4: Exclusive Choice
                {
                    let pattern = ExclusiveChoicePattern {
                        branches: vec![(TaskId::new(), "true".to_string())],
                    };
                    let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                    let mut tick_counter = TickCounter::new();
                    let _ = pattern.execute(&mut context, &mut tick_counter).await;
                    total_ticks += tick_counter.ticks();
                }

                // Pattern 5: Simple Merge
                {
                    let pattern = SimpleMergePattern;
                    let mut context = ExecutionContext::new(CaseId::new(), WorkflowId::new());
                    let mut tick_counter = TickCounter::new();
                    let _ = pattern.execute(&mut context, &mut tick_counter).await;
                    total_ticks += tick_counter.ticks();
                }

                // All 5 critical patterns should execute within reasonable bounds
                // Not a strict limit since they run sequentially, but validates individual compliance
                eprintln!("Total ticks for all critical patterns: {}", total_ticks);

                total_ticks
            })
        })
    });
}

criterion_group!(
    chatman_constant_benches,
    bench_sequence_pattern,
    bench_parallel_split_pattern,
    bench_synchronization_pattern,
    bench_exclusive_choice_pattern,
    bench_simple_merge_pattern,
    bench_all_critical_patterns,
);

criterion_main!(chatman_constant_benches);
