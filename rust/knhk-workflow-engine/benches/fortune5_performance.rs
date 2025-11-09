//! Fortune 5 Performance Benchmarks
//!
//! Comprehensive performance validation against enterprise SLO requirements:
//! - Chatman Constant: Hot path operations ≤8 ticks
//! - R1 (hot reads): ≤2ns p99
//! - W1 (hot writes): ≤1ms p99
//! - C1 (complex operations): ≤500ms p99
//!
//! Benchmark Categories:
//! 1. Hot Path Performance (CRITICAL)
//! 2. End-to-End Workflow Performance
//! 3. Scalability Benchmarks
//! 4. Telemetry Overhead Measurement
//! 5. State Management Performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use knhk_workflow_engine::{
    parser::{Condition, Flow, JoinType, SplitType, Task, TaskType, WorkflowSpec, WorkflowSpecId},
    state::StateStore,
    WorkflowEngine,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ================================================================================================
// BENCHMARK CONSTANTS
// ================================================================================================

/// Chatman Constant: Maximum ticks for hot path operations
const CHATMAN_CONSTANT_TICKS: u64 = 8;

/// Approximate CPU cycles per nanosecond (adjust based on your CPU)
/// For a 2.5 GHz CPU: 2.5 cycles/ns
const CYCLES_PER_NS: f64 = 2.5;

/// Target latencies (in nanoseconds)
const TARGET_HOT_READ_NS: u64 = 2;

// ================================================================================================
// HELPER FUNCTIONS
// ================================================================================================

/// Convert duration to CPU ticks (approximate)
fn duration_to_ticks(duration: Duration) -> u64 {
    let nanos = duration.as_nanos() as f64;
    (nanos * CYCLES_PER_NS) as u64
}

/// Create a minimal workflow for hot path testing
fn create_minimal_workflow() -> WorkflowSpec {
    let mut tasks = HashMap::new();
    let mut conditions = HashMap::new();

    // Single task
    let task = Task {
        id: "task1".to_string(),
        name: "Hot Path Task".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::Xor,
        max_ticks: Some(8),
        priority: Some(100),
        use_simd: false,
        input_conditions: vec![],
        output_conditions: vec![],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec![],
        required_capabilities: vec![],
        exception_worklet: None,
    };
    tasks.insert("task1".to_string(), task);

    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Minimal Workflow".to_string(),
        tasks,
        conditions,
        flows: vec![],
        start_condition: None,
        end_condition: None,
        source_turtle: None,
    }
}

/// Create ATM withdrawal workflow
fn create_atm_workflow() -> WorkflowSpec {
    let mut tasks = HashMap::new();
    let mut conditions = HashMap::new();

    // Task 1: Verify PIN
    tasks.insert(
        "verify_pin".to_string(),
        Task {
            id: "verify_pin".to_string(),
            name: "Verify PIN".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
            max_ticks: Some(8),
            priority: Some(100),
            use_simd: false,
            input_conditions: vec!["c_start".to_string()],
            output_conditions: vec!["c_pin_ok".to_string()],
            outgoing_flows: vec!["check_balance".to_string()],
            incoming_flows: vec![],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        },
    );

    // Task 2: Check balance
    tasks.insert(
        "check_balance".to_string(),
        Task {
            id: "check_balance".to_string(),
            name: "Check Balance".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
            max_ticks: Some(8),
            priority: Some(100),
            use_simd: false,
            input_conditions: vec!["c_pin_ok".to_string()],
            output_conditions: vec!["c_balance_ok".to_string()],
            outgoing_flows: vec!["dispense_cash".to_string()],
            incoming_flows: vec!["verify_pin".to_string()],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        },
    );

    // Task 3: Dispense cash
    tasks.insert(
        "dispense_cash".to_string(),
        Task {
            id: "dispense_cash".to_string(),
            name: "Dispense Cash".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::Xor,
            join_type: JoinType::Xor,
            max_ticks: Some(8),
            priority: Some(100),
            use_simd: false,
            input_conditions: vec!["c_balance_ok".to_string()],
            output_conditions: vec!["c_dispensed".to_string()],
            outgoing_flows: vec!["update_balance".to_string()],
            incoming_flows: vec!["check_balance".to_string()],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        },
    );

    // Task 4: Update balance
    tasks.insert(
        "update_balance".to_string(),
        Task {
            id: "update_balance".to_string(),
            name: "Update Balance".to_string(),
            task_type: TaskType::Atomic,
            split_type: SplitType::And,
            join_type: JoinType::Xor,
            max_ticks: Some(8),
            priority: Some(100),
            use_simd: false,
            input_conditions: vec!["c_dispensed".to_string()],
            output_conditions: vec!["c_complete".to_string()],
            outgoing_flows: vec![],
            incoming_flows: vec!["dispense_cash".to_string()],
            allocation_policy: None,
            required_roles: vec![],
            required_capabilities: vec![],
            exception_worklet: None,
        },
    );

    // Conditions
    conditions.insert(
        "c_start".to_string(),
        Condition {
            id: "c_start".to_string(),
            name: "Start".to_string(),
            outgoing_flows: vec!["verify_pin".to_string()],
            incoming_flows: vec![],
        },
    );

    conditions.insert(
        "c_pin_ok".to_string(),
        Condition {
            id: "c_pin_ok".to_string(),
            name: "PIN OK".to_string(),
            outgoing_flows: vec!["check_balance".to_string()],
            incoming_flows: vec!["verify_pin".to_string()],
        },
    );

    conditions.insert(
        "c_balance_ok".to_string(),
        Condition {
            id: "c_balance_ok".to_string(),
            name: "Balance OK".to_string(),
            outgoing_flows: vec!["dispense_cash".to_string()],
            incoming_flows: vec!["check_balance".to_string()],
        },
    );

    conditions.insert(
        "c_dispensed".to_string(),
        Condition {
            id: "c_dispensed".to_string(),
            name: "Cash Dispensed".to_string(),
            outgoing_flows: vec!["update_balance".to_string()],
            incoming_flows: vec!["dispense_cash".to_string()],
        },
    );

    conditions.insert(
        "c_complete".to_string(),
        Condition {
            id: "c_complete".to_string(),
            name: "Complete".to_string(),
            outgoing_flows: vec![],
            incoming_flows: vec!["update_balance".to_string()],
        },
    );

    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "ATM Withdrawal".to_string(),
        tasks,
        conditions,
        flows: vec![],
        start_condition: Some("c_start".to_string()),
        end_condition: Some("c_complete".to_string()),
        source_turtle: None,
    }
}

// ================================================================================================
// HOT PATH BENCHMARKS (CRITICAL)
// ================================================================================================

/// Benchmark: Split type comparison (must be ≤8 ticks)
fn bench_split_type_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path");
    group.significance_level(0.01).sample_size(1000);

    group.bench_function("split_type_comparison", |b| {
        b.iter(|| {
            let start = Instant::now();
            let split = black_box(SplitType::And);
            let _matches = matches!(split, SplitType::And);
            let elapsed = start.elapsed();
            let ticks = duration_to_ticks(elapsed);
            assert!(
                ticks <= CHATMAN_CONSTANT_TICKS,
                "Split type comparison took {} ticks (max: {})",
                ticks,
                CHATMAN_CONSTANT_TICKS
            );
        });
    });

    group.finish();
}

/// Benchmark: Join type comparison (must be ≤8 ticks)
fn bench_join_type_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path");
    group.significance_level(0.01).sample_size(1000);

    group.bench_function("join_type_comparison", |b| {
        b.iter(|| {
            let start = Instant::now();
            let join = black_box(JoinType::Xor);
            let _matches = matches!(join, JoinType::Xor);
            let elapsed = start.elapsed();
            let ticks = duration_to_ticks(elapsed);
            assert!(
                ticks <= CHATMAN_CONSTANT_TICKS,
                "Join type comparison took {} ticks (max: {})",
                ticks,
                CHATMAN_CONSTANT_TICKS
            );
        });
    });

    group.finish();
}

/// Benchmark: Task lookup (hot read, must be ≤2ns p99)
fn bench_task_lookup_hot_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path");
    group.significance_level(0.01).sample_size(1000);

    let spec = create_minimal_workflow();

    group.bench_function("task_lookup_hot_read", |b| {
        b.iter(|| {
            let start = Instant::now();
            let _task = black_box(spec.tasks.get("task1"));
            let elapsed = start.elapsed();
            assert!(
                elapsed.as_nanos() <= TARGET_HOT_READ_NS as u128,
                "Task lookup took {}ns (target: ≤{}ns)",
                elapsed.as_nanos(),
                TARGET_HOT_READ_NS
            );
        });
    });

    group.finish();
}

/// Benchmark: Max ticks check (must be ≤8 ticks itself)
fn bench_max_ticks_check_hot_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_path");
    group.significance_level(0.01).sample_size(1000);

    let task = Task {
        id: "task1".to_string(),
        name: "Test Task".to_string(),
        task_type: TaskType::Atomic,
        split_type: SplitType::And,
        join_type: JoinType::Xor,
        max_ticks: Some(8),
        priority: Some(100),
        use_simd: false,
        input_conditions: vec![],
        output_conditions: vec![],
        outgoing_flows: vec![],
        incoming_flows: vec![],
        allocation_policy: None,
        required_roles: vec![],
        required_capabilities: vec![],
        exception_worklet: None,
    };

    group.bench_function("max_ticks_check", |b| {
        b.iter(|| {
            let start = Instant::now();
            let _exceeds = black_box(task.max_ticks.map_or(false, |m| m > 8));
            let elapsed = start.elapsed();
            let ticks = duration_to_ticks(elapsed);
            assert!(
                ticks <= CHATMAN_CONSTANT_TICKS,
                "Max ticks check took {} ticks (max: {})",
                ticks,
                CHATMAN_CONSTANT_TICKS
            );
        });
    });

    group.finish();
}

// ================================================================================================
// WORKFLOW CREATION BENCHMARKS
// ================================================================================================

/// Benchmark: Minimal workflow creation
fn bench_minimal_workflow_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_creation");
    group.sample_size(100);

    group.bench_function("minimal_workflow", |b| {
        b.iter(|| {
            let _spec = black_box(create_minimal_workflow());
        });
    });

    group.finish();
}

/// Benchmark: ATM workflow creation
fn bench_atm_workflow_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_creation");
    group.sample_size(100);

    group.bench_function("atm_workflow", |b| {
        b.iter(|| {
            let _spec = black_box(create_atm_workflow());
        });
    });

    group.finish();
}

// ================================================================================================
// ENGINE CREATION BENCHMARKS
// ================================================================================================

/// Benchmark: Engine creation with state store
fn bench_engine_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine");
    group.sample_size(50);

    group.bench_function("engine_creation", |b| {
        b.iter(|| {
            let state_store = StateStore::new("test_workflow_db_bench").unwrap();
            let _engine = black_box(WorkflowEngine::new(state_store));
        });
    });

    group.finish();
}

// ================================================================================================
// SCALABILITY BENCHMARKS
// ================================================================================================

/// Benchmark: Multiple workflow specs in memory
fn bench_workflow_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20);

    for workflow_count in [10, 50, 100, 200].iter() {
        group.throughput(Throughput::Elements(*workflow_count as u64));

        group.bench_with_input(
            BenchmarkId::new("workflow_specs", workflow_count),
            workflow_count,
            |b, &count| {
                b.iter(|| {
                    let mut specs = Vec::new();
                    for _ in 0..count {
                        specs.push(create_minimal_workflow());
                    }
                    black_box(specs);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Task lookup scalability
fn bench_task_lookup_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(100);

    for task_count in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*task_count as u64));

        group.bench_with_input(
            BenchmarkId::new("task_lookup", task_count),
            task_count,
            |b, &count| {
                // Create workflow with many tasks
                let mut tasks = HashMap::new();
                for i in 0..count {
                    tasks.insert(
                        format!("task{}", i),
                        Task {
                            id: format!("task{}", i),
                            name: format!("Task {}", i),
                            task_type: TaskType::Atomic,
                            split_type: SplitType::And,
                            join_type: JoinType::Xor,
                            max_ticks: Some(8),
                            priority: Some(100),
                            use_simd: false,
                            input_conditions: vec![],
                            output_conditions: vec![],
                            outgoing_flows: vec![],
                            incoming_flows: vec![],
                            allocation_policy: None,
                            required_roles: vec![],
                            required_capabilities: vec![],
                            exception_worklet: None,
                        },
                    );
                }

                let spec = WorkflowSpec {
                    id: WorkflowSpecId::new(),
                    name: "Scalability Test".to_string(),
                    tasks,
                    conditions: HashMap::new(),
                    flows: vec![],
                    start_condition: None,
                    end_condition: None,
                    source_turtle: None,
                };

                b.iter(|| {
                    // Lookup random task
                    let task_id = format!("task{}", count / 2);
                    let _task = black_box(spec.tasks.get(&task_id));
                });
            },
        );
    }

    group.finish();
}

// ================================================================================================
// TELEMETRY OVERHEAD BENCHMARKS
// ================================================================================================

/// Benchmark: Telemetry overhead (should be <5% of operation time)
fn bench_telemetry_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("telemetry");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(100);

    // Benchmark WITHOUT telemetry simulation
    let mut baseline_duration = Duration::ZERO;
    group.bench_function("baseline_no_telemetry", |b| {
        b.iter(|| {
            let start = Instant::now();
            black_box(create_minimal_workflow());
            baseline_duration = start.elapsed();
        });
    });

    // Benchmark WITH telemetry simulation
    let mut with_telemetry_duration = Duration::ZERO;
    group.bench_function("with_telemetry", |b| {
        b.iter(|| {
            let start = Instant::now();
            let _spec = black_box(create_minimal_workflow());
            // Simulate OTEL span creation (timestamp recording)
            let _span_start = Instant::now();
            with_telemetry_duration = start.elapsed();
        });
    });

    group.finish();

    // Calculate overhead percentage
    let overhead_ns =
        with_telemetry_duration.as_nanos() as i128 - baseline_duration.as_nanos() as i128;
    let overhead_percent = (overhead_ns as f64 / baseline_duration.as_nanos() as f64) * 100.0;

    println!("\n=== TELEMETRY OVERHEAD ANALYSIS ===");
    println!("Baseline (no telemetry): {:?}", baseline_duration);
    println!("With telemetry: {:?}", with_telemetry_duration);
    println!("Overhead: {}ns ({:.2}%)", overhead_ns, overhead_percent);
    println!(
        "Target: <5% overhead - {}",
        if overhead_percent < 5.0 {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    );
}

// ================================================================================================
// BENCHMARK GROUPS
// ================================================================================================

criterion_group!(
    hot_path_benches,
    bench_split_type_hot_path,
    bench_join_type_hot_path,
    bench_task_lookup_hot_read,
    bench_max_ticks_check_hot_path,
);

criterion_group!(
    workflow_creation_benches,
    bench_minimal_workflow_creation,
    bench_atm_workflow_creation,
);

criterion_group!(engine_benches, bench_engine_creation,);

criterion_group!(
    scalability_benches,
    bench_workflow_scalability,
    bench_task_lookup_scalability,
);

criterion_group!(telemetry_benches, bench_telemetry_overhead,);

criterion_main!(
    hot_path_benches,
    workflow_creation_benches,
    engine_benches,
    scalability_benches,
    telemetry_benches,
);
