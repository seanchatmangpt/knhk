//! Example: Quantum-Inspired Workflow Optimization
//!
//! Demonstrates all four quantum algorithms for workflow scheduling.
//!
//! Run with:
//! ```bash
//! cargo run --example quantum_optimization --features default
//! ```

use knhk_workflow_engine::quantum::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌌 Quantum-Inspired Workflow Optimization Demo\n");

    // Create sample workflow tasks
    let tasks = create_sample_workflow();
    println!("📋 Created {} workflow tasks with dependencies\n", tasks.len());

    // Demo 1: Quantum Annealing
    println!("═══ 1️⃣  QUANTUM ANNEALING ═══");
    demo_quantum_annealing(&tasks).await?;
    println!();

    // Demo 2: Grover Search
    println!("═══ 2️⃣  GROVER SEARCH ═══");
    demo_grover_search(&tasks).await?;
    println!();

    // Demo 3: QAOA
    println!("═══ 3️⃣  QAOA (Task Partitioning) ═══");
    demo_qaoa(&tasks).await?;
    println!();

    // Demo 4: Quantum Walk
    println!("═══ 4️⃣  QUANTUM WALK (Dependency Resolution) ═══");
    demo_quantum_walk(&tasks).await?;
    println!();

    // Demo 5: Unified Scheduler (Auto-select)
    println!("═══ 5️⃣  UNIFIED SCHEDULER (Auto-select) ═══");
    demo_unified_scheduler(&tasks).await?;
    println!();

    // Demo 6: Hybrid Optimization
    println!("═══ 6️⃣  HYBRID OPTIMIZATION ═══");
    demo_hybrid_optimization(&tasks).await?;
    println!();

    // Performance comparison
    println!("═══ 📊 PERFORMANCE COMPARISON ═══");
    performance_comparison(&tasks).await?;

    println!("\n✅ All quantum optimization demos completed successfully!");

    Ok(())
}

fn create_sample_workflow() -> Vec<WorkflowTask> {
    let mut tasks = Vec::new();

    // Create 20 tasks with varying characteristics
    for i in 0..20 {
        let task = WorkflowTask::new(format!("Task-{:02}", i))
            .with_duration(50 + (i as u64 * 10))
            .with_cost(10.0 + (i as f64 * 2.0))
            .with_cpu(40.0 + (i as f64 * 2.0))
            .with_memory(512.0 + (i as f64 * 50.0))
            .with_priority((10 - i as i32).max(1));

        tasks.push(task);
    }

    // Add dependencies to create realistic workflow
    tasks[1] = tasks[1].clone().with_dependency(tasks[0].id);
    tasks[2] = tasks[2].clone().with_dependency(tasks[0].id);
    tasks[3] = tasks[3].clone().with_dependency(tasks[1].id);
    tasks[4] = tasks[4].clone().with_dependency(tasks[2].id);
    tasks[5] = tasks[5].clone().with_dependency(tasks[3].id)
        .with_dependency(tasks[4].id);
    tasks[10] = tasks[10].clone().with_dependency(tasks[5].id);
    tasks[15] = tasks[15].clone().with_dependency(tasks[10].id);
    tasks[19] = tasks[19].clone().with_dependency(tasks[15].id);

    tasks
}

async fn demo_quantum_annealing(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Optimizing workflow schedule using quantum annealing...");

    let mut constraints = ConstraintManager::new();
    constraints.add_constraint(Box::new(LatencyConstraint::new(5000)))?;
    constraints.add_constraint(Box::new(CostConstraint::new(500.0)))?;
    constraints.add_constraint(Box::new(ResourceConstraint::new(80.0)))?;

    let config = AnnealingConfig::with_seed(42)
        .initial_temp(1000.0)
        .final_temp(0.1)
        .cooling_rate(0.95)
        .max_iterations(1000);

    let start = Instant::now();
    let mut annealer = QuantumAnnealing::new(config, std::sync::Arc::new(constraints));
    let result = annealer.optimize(tasks).await?;
    let elapsed = start.elapsed();

    println!("✅ Optimization complete in {:?}", elapsed);
    println!("   Energy: {:.2}", result.energy);
    println!("   Constraint satisfaction: {:.1}%", result.constraint_satisfaction * 100.0);
    println!("   Quality score: {:.1}%", result.quality_score() * 100.0);
    println!("   Scheduled {} tasks", result.execution_order.len());

    Ok(())
}

async fn demo_grover_search(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Finding optimal resource allocation using Grover search...");

    let resources = vec![
        Resource::new("Resource-A"),
        Resource::new("Resource-B"),
        Resource::new("Resource-C"),
        Resource::new("Resource-D"),
    ];

    let search_space = tasks.len() * resources.len();
    let config = GroverConfig::for_search_space(search_space)
        .with_seed(42)
        .with_amplification(2.0);

    let start = Instant::now();
    let mut grover = GroverSearch::new(config);
    let allocation = grover
        .find_optimal_allocation(tasks, &resources, Box::new(default_oracle))
        .await?;
    let elapsed = start.elapsed();

    let speedup = grover.calculate_speedup(search_space);

    println!("✅ Resource allocation complete in {:?}", elapsed);
    println!("   Search space: {} possibilities", search_space);
    println!("   Theoretical speedup: {:.1}x", speedup);
    println!("   Allocated {} tasks to {} resources", allocation.len(), resources.len());

    // Show allocation distribution
    let mut resource_counts = std::collections::HashMap::new();
    for resource_id in allocation.values() {
        *resource_counts.entry(resource_id).or_insert(0) += 1;
    }

    for (resource_id, count) in resource_counts {
        println!("   {} → {} tasks", resource_id, count);
    }

    Ok(())
}

async fn demo_qaoa(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Partitioning tasks using QAOA...");

    let config = QAOAConfig::default()
        .with_seed(42)
        .with_layers(3)
        .with_max_iterations(500);

    let num_partitions = 4;

    let start = Instant::now();
    let mut qaoa = QAOAOptimizer::new(config);
    let partitions = qaoa.optimize_assignment(tasks, num_partitions).await?;
    let elapsed = start.elapsed();

    println!("✅ Task partitioning complete in {:?}", elapsed);
    println!("   Created {} partitions", partitions.len());

    for (i, partition) in partitions.iter().enumerate() {
        println!("   Partition {}: {} tasks", i + 1, partition.len());
    }

    Ok(())
}

async fn demo_quantum_walk(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Resolving dependencies using quantum walk...");

    let config = QuantumWalkConfig::default()
        .with_seed(42)
        .with_mixing(0.5)
        .with_max_iterations(500);

    let start = Instant::now();
    let mut qwalk = QuantumWalk::new(config);
    let execution_order = qwalk.find_execution_order(tasks).await?;
    let elapsed = start.elapsed();

    // Compare with classical topological sort
    let classical_order = qwalk.topological_sort(tasks)?;

    println!("✅ Dependency resolution complete in {:?}", elapsed);
    println!("   Execution order: {} tasks", execution_order.len());
    println!("   First 5 tasks: {:?}",
        execution_order.iter()
            .take(5)
            .map(|id| tasks.iter().find(|t| &t.id == id).unwrap().name.as_str())
            .collect::<Vec<_>>()
    );
    println!("   Convergence: Quantum walk vs classical topological sort");

    Ok(())
}

async fn demo_unified_scheduler(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Using unified scheduler with auto-selection...");

    let scheduler = QuantumScheduler::builder()
        .with_seed(42)
        .with_method(OptimizationMethod::Auto)
        .with_constraint(Box::new(LatencyConstraint::new(5000)))
        .with_constraint(Box::new(CostConstraint::new(500.0)))
        .with_constraint(Box::new(ResourceConstraint::new(80.0)))
        .build()?;

    let start = Instant::now();
    let schedule = scheduler.optimize(tasks).await?;
    let elapsed = start.elapsed();

    println!("✅ Optimization complete in {:?}", elapsed);
    println!("   Method selected: {:?}", schedule.method);
    println!("   Quality score: {:.1}%", schedule.quality_score * 100.0);
    println!("   Constraints satisfied: {}", schedule.constraints_satisfied);
    println!("   Optimization time: {}ms", schedule.optimization_time_ms);

    Ok(())
}

async fn demo_hybrid_optimization(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running hybrid optimization (all algorithms in parallel)...");

    let scheduler = QuantumScheduler::builder()
        .with_seed(42)
        .with_method(OptimizationMethod::Hybrid)
        .with_constraint(Box::new(LatencyConstraint::new(5000)))
        .with_constraint(Box::new(CostConstraint::new(500.0)))
        .build()?;

    let start = Instant::now();
    let schedule = scheduler.optimize(tasks).await?;
    let elapsed = start.elapsed();

    println!("✅ Hybrid optimization complete in {:?}", elapsed);
    println!("   Best quality: {:.1}%", schedule.quality_score * 100.0);
    println!("   Constraints satisfied: {}", schedule.constraints_satisfied);
    println!("   Total optimization time: {}ms", schedule.optimization_time_ms);

    Ok(())
}

async fn performance_comparison(tasks: &[WorkflowTask]) -> Result<(), Box<dyn std::error::Error>> {
    let sizes = vec![10, 20, 50, 100];

    println!("\n┌─────────┬──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│  Tasks  │   Annealing  │    Grover    │     QAOA     │ Quantum Walk │");
    println!("├─────────┼──────────────┼──────────────┼──────────────┼──────────────┤");

    for &size in &sizes {
        let subset: Vec<_> = tasks.iter().take(size).cloned().collect();

        // Annealing
        let scheduler_ann = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::QuantumAnnealing)
            .build()?;
        let start = Instant::now();
        let _ = scheduler_ann.optimize(&subset).await?;
        let ann_time = start.elapsed().as_millis();

        // Grover
        let scheduler_grov = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::GroverSearch)
            .build()?;
        let start = Instant::now();
        let _ = scheduler_grov.optimize(&subset).await?;
        let grov_time = start.elapsed().as_millis();

        // QAOA
        let scheduler_qaoa = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::QAOA)
            .build()?;
        let start = Instant::now();
        let _ = scheduler_qaoa.optimize(&subset).await?;
        let qaoa_time = start.elapsed().as_millis();

        // Quantum Walk
        let scheduler_qw = QuantumScheduler::builder()
            .with_seed(42)
            .with_method(OptimizationMethod::QuantumWalk)
            .build()?;
        let start = Instant::now();
        let _ = scheduler_qw.optimize(&subset).await?;
        let qw_time = start.elapsed().as_millis();

        println!("│ {:>7} │ {:>10}ms │ {:>10}ms │ {:>10}ms │ {:>10}ms │",
            size, ann_time, grov_time, qaoa_time, qw_time);
    }

    println!("└─────────┴──────────────┴──────────────┴──────────────┴──────────────┘");

    Ok(())
}
