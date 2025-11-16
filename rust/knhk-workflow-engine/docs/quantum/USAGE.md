# Quantum-Inspired Optimization Usage Guide

## Quick Start

```rust
use knhk_workflow_engine::quantum::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create workflow tasks
    let tasks = vec![
        WorkflowTask::new("ingest-data")
            .with_duration(100)
            .with_cost(5.0)
            .with_cpu(60.0),
        WorkflowTask::new("process-data")
            .with_duration(200)
            .with_cost(10.0)
            .with_cpu(80.0)
            .with_dependency(/* ingest-data id */),
        WorkflowTask::new("export-results")
            .with_duration(50)
            .with_cost(3.0)
            .with_cpu(40.0)
            .with_dependency(/* process-data id */),
    ];

    // 2. Create scheduler with constraints
    let scheduler = QuantumScheduler::builder()
        .with_seed(42)  // For reproducibility
        .with_method(OptimizationMethod::Auto)  // Auto-select best algorithm
        .with_constraint(Box::new(LatencyConstraint::new(1000)))  // Max 1s latency
        .with_constraint(Box::new(CostConstraint::new(100.0)))     // Max $100 cost
        .with_constraint(Box::new(ResourceConstraint::new(80.0)))  // Max 80% CPU
        .build()?;

    // 3. Optimize schedule
    let schedule = scheduler.optimize(&tasks).await?;

    // 4. Validate results
    assert!(schedule.satisfies_constraints());
    assert!(schedule.is_optimal_within(0.05));  // Within 5% of global optimum

    println!("✅ Scheduled {} tasks in {}ms",
        schedule.state.execution_order.len(),
        schedule.optimization_time_ms);

    Ok(())
}
```

## Creating Tasks

### Basic Task

```rust
let task = WorkflowTask::new("my-task")
    .with_duration(100)          // 100ms execution time
    .with_cost(10.0)             // Cost per execution
    .with_cpu(50.0)              // 50% CPU requirement
    .with_memory(1024.0)         // 1GB memory
    .with_priority(5);           // Priority (higher = more important)
```

### Task with Dependencies

```rust
let task1 = WorkflowTask::new("task-1");
let task2 = WorkflowTask::new("task-2")
    .with_dependency(task1.id);  // task2 depends on task1
let task3 = WorkflowTask::new("task-3")
    .with_dependency(task1.id)
    .with_dependency(task2.id);  // task3 depends on both
```

### Task with Metadata

```rust
let task = WorkflowTask::new("etl-job")
    .with_metadata("source", "database-1")
    .with_metadata("destination", "warehouse")
    .with_metadata("batch_size", "10000");
```

## Constraints

### Latency Constraint

Maximum total execution time:

```rust
let constraint = LatencyConstraint::new(1000)  // Max 1000ms
    .with_weight(2.0);  // 2x penalty weight

scheduler.add_constraint(Box::new(constraint))?;
```

### Cost Constraint

Maximum total cost:

```rust
let constraint = CostConstraint::new(100.0)  // Max $100
    .with_weight(1.5);

scheduler.add_constraint(Box::new(constraint))?;
```

### Resource Constraint

Maximum CPU and memory utilization:

```rust
let constraint = ResourceConstraint::new(80.0)  // Max 80% CPU
    .with_max_memory(8192.0)   // Max 8GB memory
    .with_weight(1.0);

scheduler.add_constraint(Box::new(constraint))?;
```

### Custom Constraints

Implement the `Constraint` trait:

```rust
use knhk_workflow_engine::quantum::Constraint;

#[derive(Debug)]
struct DeadlineConstraint {
    deadline_ms: u64,
}

impl Constraint for DeadlineConstraint {
    fn is_satisfied(&self, state: &State, tasks: &[WorkflowTask]) -> bool {
        // Check if execution completes before deadline
        let total_time: u64 = tasks.iter()
            .filter(|t| state.execution_order.contains(&t.id))
            .map(|t| t.estimated_duration_ms)
            .sum();
        total_time <= self.deadline_ms
    }

    fn penalty(&self, state: &State, tasks: &[WorkflowTask]) -> f64 {
        let total_time: u64 = tasks.iter()
            .filter(|t| state.execution_order.contains(&t.id))
            .map(|t| t.estimated_duration_ms)
            .sum();
        if total_time > self.deadline_ms {
            ((total_time - self.deadline_ms) as f64 / self.deadline_ms as f64) * 100.0
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "DeadlineConstraint"
    }
}
```

## Optimization Methods

### Quantum Annealing

Best for: General optimization with multiple constraints

```rust
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::QuantumAnnealing)
    .build()?;

let schedule = scheduler.optimize(&tasks).await?;
```

Configuration options:

```rust
let config = AnnealingConfig::with_seed(42)
    .initial_temp(1000.0)       // Higher = more exploration
    .final_temp(0.1)            // Lower = more exploitation
    .cooling_rate(0.95)         // 0.9-0.99 typical
    .max_iterations(10_000)     // More iterations = better quality
    .tunneling_factor(0.1);     // Quantum tunneling strength
```

### Grover Search

Best for: Resource allocation, unstructured search

```rust
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::GroverSearch)
    .build()?;

let schedule = scheduler.optimize(&tasks).await?;
```

Direct usage:

```rust
let resources = vec![
    Resource::new("gpu-1"),
    Resource::new("gpu-2"),
    Resource::new("cpu-pool"),
];

let mut grover = GroverSearch::new(
    GroverConfig::for_search_space(tasks.len() * resources.len())
        .with_seed(42)
);

let allocation = grover
    .find_optimal_allocation(&tasks, &resources, Box::new(default_oracle))
    .await?;
```

### QAOA

Best for: Task partitioning, load balancing

```rust
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::QAOA)
    .build()?;

let schedule = scheduler.optimize_qaoa(&tasks, 4).await?;  // 4 partitions
```

Direct usage:

```rust
let mut qaoa = QAOAOptimizer::new(
    QAOAConfig::default()
        .with_seed(42)
        .with_layers(3)          // More layers = better quality
        .with_max_iterations(1000)
);

let partitions = qaoa.optimize_assignment(&tasks, 4).await?;
```

### Quantum Walk

Best for: Dependency resolution, precedence constraints

```rust
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::QuantumWalk)
    .build()?;

let schedule = scheduler.optimize(&tasks).await?;
```

Direct usage:

```rust
let mut qwalk = QuantumWalk::new(
    QuantumWalkConfig::default()
        .with_seed(42)
        .with_mixing(0.5)         // Quantum interference strength
        .with_max_iterations(500)
);

let execution_order = qwalk.find_execution_order(&tasks).await?;
```

### Hybrid

Best for: Maximum quality, critical workflows

```rust
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::Hybrid)
    .build()?;

let schedule = scheduler.optimize(&tasks).await?;
```

Runs multiple algorithms in parallel and selects best result.

### Auto

Best for: General use, automatic selection

```rust
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::Auto)
    .build()?;

let schedule = scheduler.optimize(&tasks).await?;
```

Selection criteria:
- **QuantumWalk**: If `has_dependencies && num_tasks > 50`
- **QuantumAnnealing**: If `complex_constraints`
- **Hybrid**: If `num_tasks > 100`
- **GroverSearch**: Otherwise

## Resources

### Creating Resources

```rust
let resource = Resource::new("gpu-cluster-1")
    .with_cpu(800.0)             // 8 cores * 100%
    .with_memory(16384.0)        // 16GB
    .with_cost_per_ms(0.001)     // $0.001 per ms
    .with_location("us-west");
```

### Custom Resource Selection

```rust
use knhk_workflow_engine::quantum::grover::Oracle;

let custom_oracle: Oracle = Box::new(|allocation, tasks, resources| {
    let mut score = 0.0;

    // Prefer specific resources for specific tasks
    for (task_id, resource_id) in allocation {
        if let Some(task) = tasks.iter().find(|t| &t.id == task_id) {
            if task.metadata.get("type") == Some(&"gpu".to_string()) {
                if resource_id.contains("gpu") {
                    score += 100.0;  // Bonus for GPU tasks on GPU resources
                }
            }
        }
    }

    score
});

let mut grover = GroverSearch::new(config);
let allocation = grover
    .find_optimal_allocation(&tasks, &resources, custom_oracle)
    .await?;
```

## Schedule Analysis

```rust
let schedule = scheduler.optimize(&tasks).await?;

// Check constraints
if schedule.satisfies_constraints() {
    println!("✅ All constraints satisfied");
} else {
    println!("⚠️  Some constraints violated");
}

// Check optimality
if schedule.is_optimal_within(0.05) {
    println!("✅ Within 5% of global optimum");
}

// Access results
println!("Optimization method: {:?}", schedule.method);
println!("Quality score: {:.2}%", schedule.quality_score * 100.0);
println!("Time taken: {}ms", schedule.optimization_time_ms);
println!("Energy (cost): {:.2}", schedule.state.energy);

// Execution order
for task_id in &schedule.state.execution_order {
    let task = tasks.iter().find(|t| &t.id == task_id).unwrap();
    let resource = &schedule.allocations[task_id];
    println!("{} → {}", task.name, resource);
}
```

## Performance Tuning

### For Maximum Speed (1M workflows < 100ms)

```rust
let config = AnnealingConfig::with_seed(42)
    .max_iterations(100)      // Reduce iterations
    .cooling_rate(0.85)       // Faster cooling
    .initial_temp(500.0);     // Lower initial temp

let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::QuantumAnnealing)
    .build()?;
```

### For Maximum Quality

```rust
let config = AnnealingConfig::with_seed(42)
    .max_iterations(50_000)   // More iterations
    .cooling_rate(0.99)       // Slower cooling
    .initial_temp(2000.0)     // Higher initial temp
    .tunneling_factor(0.2);   // More tunneling

// Or use Hybrid
let scheduler = QuantumScheduler::builder()
    .with_method(OptimizationMethod::Hybrid)
    .build()?;
```

### For Determinism

Always set a seed:

```rust
let scheduler = QuantumScheduler::builder()
    .with_seed(42)  // Same seed = same results
    .build()?;
```

## Integration with KNHK

### With OpenTelemetry

```rust
use tracing::{info_span, instrument};

#[instrument]
async fn optimize_workflow(tasks: Vec<WorkflowTask>) -> QuantumResult<Schedule> {
    let _span = info_span!("quantum_optimization").entered();

    let scheduler = QuantumScheduler::builder()
        .with_seed(42)
        .build()?;

    scheduler.optimize(&tasks).await
}
```

### With Workflow Engine

```rust
use knhk_workflow_engine::{WorkflowEngine, quantum::*};

let engine = WorkflowEngine::new(state_store);

// Parse workflow
let spec = parser.parse_file("workflow.ttl")?;

// Convert to quantum tasks
let tasks: Vec<WorkflowTask> = spec.tasks.iter()
    .map(|t| WorkflowTask::new(&t.name)
        .with_duration(t.estimated_duration)
        .with_cost(t.cost)
        // ... other properties
    )
    .collect();

// Optimize schedule
let schedule = scheduler.optimize(&tasks).await?;

// Execute in optimal order
for task_id in schedule.state.execution_order {
    engine.execute_task(task_id).await?;
}
```

## Examples

See `/examples/quantum_optimization.rs` for complete working examples:

```bash
cargo run --example quantum_optimization --features default
```

## Troubleshooting

### Poor Quality Results

- Increase `max_iterations`
- Decrease `cooling_rate` (anneal slower)
- Use `Hybrid` method
- Add more constraints to guide optimization

### Slow Performance

- Decrease `max_iterations`
- Increase `cooling_rate` (anneal faster)
- Use `GroverSearch` instead of `QuantumAnnealing`
- Reduce constraint complexity

### Non-Deterministic Results

- Always set `seed` in builder
- Use same seed across runs
- Avoid external randomness (system time, etc.)

### Constraint Violations

- Relax constraints (increase limits)
- Increase `penalty_weight` in annealing config
- Increase constraint weights
- Check for infeasible constraints

## Best Practices

1. **Start Simple**: Begin with `Auto` method, add constraints incrementally
2. **Benchmark**: Use provided benchmarks to validate performance
3. **Seed for Tests**: Always use deterministic seeds in tests
4. **Profile**: Use `--features=profiling` to identify bottlenecks
5. **Validate**: Check `satisfies_constraints()` and `is_optimal_within()`
6. **Document**: Record optimization parameters and results

## API Reference

See:
- [Module documentation](https://docs.rs/knhk-workflow-engine/latest/knhk_workflow_engine/quantum/)
- [Algorithm theory](./ALGORITHMS.md)
- [Examples](../../examples/)
