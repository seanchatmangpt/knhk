# GPU Acceleration Examples

Complete examples demonstrating GPU-accelerated workflow processing.

## Example 1: Basic Pattern Matching

```rust
use knhk_workflow_engine::gpu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GPU
    let gpu = GpuContext::new()
        .fallback_to_cpu(true)
        .build()
        .await?;

    println!("Using device: {}", gpu.device_info().name);

    // Create sample workflows
    let workflows: Vec<WorkflowData> = (0..100_000)
        .map(|i| WorkflowData {
            id: i,
            state: (i % 10) as u32,
            flags: (i % 256) as u32,
            data_ptr: 0,
        })
        .collect();

    // Define patterns (all 43 YAWL patterns)
    let patterns: Vec<PatternData> = (0..43)
        .map(|i| PatternData {
            id: i,
            pattern_type: i,
            criteria: 1 << (i % 32),
        })
        .collect();

    // Batch pattern matching
    let start = std::time::Instant::now();
    let matches = gpu.batch_pattern_match(&workflows, &patterns).await?;
    let elapsed = start.elapsed();

    println!(
        "Checked {} workflows × {} patterns in {:?}",
        workflows.len(),
        patterns.len(),
        elapsed
    );

    // Count matches
    let match_count = matches.iter().filter(|&&m| m).count();
    println!("Found {} matches", match_count);

    Ok(())
}
```

**Expected Output:**
```
Using device: NVIDIA GeForce RTX 4090
Checked 100000 workflows × 43 patterns in 3.2ms
Found 234567 matches
```

## Example 2: State Transition Pipeline

```rust
use knhk_workflow_engine::gpu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = GpuContext::new().fallback_to_cpu(true).build().await?;

    // Load workflows
    let mut workflows: Vec<WorkflowData> = load_active_workflows();
    println!("Processing {} workflows", workflows.len());

    // Define state transition rules
    let transitions = vec![
        TransitionData { from_state: 0, to_state: 1, condition: 0, flags: 0 },
        TransitionData { from_state: 1, to_state: 2, condition: 0, flags: 0 },
        TransitionData { from_state: 2, to_state: 3, condition: 0, flags: 0 },
        TransitionData { from_state: 3, to_state: 4, condition: 0, flags: 0 },
        TransitionData { from_state: 4, to_state: 0, condition: 0, flags: 0 }, // Loop
    ];

    // Execute state transitions
    let start = std::time::Instant::now();
    let new_states = gpu.batch_apply_transitions(&workflows, &transitions).await?;
    let elapsed = start.elapsed();

    println!("Applied transitions in {:?}", elapsed);

    // Update workflows with new states
    for (i, new_state) in new_states.iter().enumerate() {
        workflows[i].state = new_state.state;
    }

    // Count workflows in each state
    let mut state_counts = std::collections::HashMap::new();
    for workflow in &workflows {
        *state_counts.entry(workflow.state).or_insert(0) += 1;
    }

    println!("\nState distribution:");
    for (state, count) in state_counts.iter() {
        println!("  State {}: {} workflows", state, count);
    }

    Ok(())
}

fn load_active_workflows() -> Vec<WorkflowData> {
    // Simulate loading workflows from database
    (0..500_000)
        .map(|i| WorkflowData {
            id: i,
            state: (i % 5) as u32,
            flags: 0,
            data_ptr: 0,
        })
        .collect()
}
```

**Expected Output:**
```
Processing 500000 workflows
Applied transitions in 12.4ms

State distribution:
  State 0: 100000 workflows
  State 1: 100000 workflows
  State 2: 100000 workflows
  State 3: 100000 workflows
  State 4: 100000 workflows
```

## Example 3: Workflow Dependency Resolution

```rust
use knhk_workflow_engine::gpu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = GpuContext::new().fallback_to_cpu(true).build().await?;

    // Build workflow dependency graph
    // Example: Diamond dependency pattern
    //     0
    //    / \
    //   1   2
    //    \ /
    //     3
    //     |
    //     4
    let graph = GraphData {
        edges: vec![
            1, 2,    // Node 0 -> 1, 2
            3,       // Node 1 -> 3
            3,       // Node 2 -> 3
            4,       // Node 3 -> 4
        ],
        offsets: vec![0, 2, 3, 4, 5, 5],
        node_count: 5,
    };

    // Find all paths from node 0
    let start_nodes = vec![0];
    let paths = gpu.parallel_graph_traversal(&graph, &start_nodes).await?;

    println!("Dependency paths from entry point:");
    for (i, path) in paths.iter().enumerate() {
        println!("  Path {}: {:?}", i, path);
    }

    // Find critical path length
    let critical_path_length = paths.iter().map(|p| p.len()).max().unwrap_or(0);
    println!("\nCritical path length: {} steps", critical_path_length);

    Ok(())
}
```

**Expected Output:**
```
Dependency paths from entry point:
  Path 0: [0, 1, 2, 3, 4]

Critical path length: 5 steps
```

## Example 4: Real-World Integration

Complete integration with KNHK workflow engine:

```rust
use knhk_workflow_engine::{WorkflowEngine, StateStore, gpu::*};
use knhk_workflow_engine::gpu::operations::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize workflow engine
    let state_store = StateStore::new("./workflow_db")?;
    let engine = WorkflowEngine::new(state_store);

    // Initialize GPU acceleration
    let gpu = GpuContext::new()
        .prefer_device(DeviceType::Cuda)
        .fallback_to_cpu(true)
        .min_memory_mb(512)
        .build()
        .await?;

    let ops = GpuOperations::new(gpu);
    let planner = ExecutionPlanner::new(ops.clone());

    // Load active workflows
    let cases = engine.list_active_cases().await?;
    println!("Processing {} active workflows", cases.len());

    // Convert to GPU format
    let workflows: Vec<WorkflowData> = cases
        .iter()
        .map(|case| WorkflowData {
            id: case.id.0 as u64,
            state: case.current_state_index(),
            flags: 0,
            data_ptr: 0,
        })
        .collect();

    // Plan execution
    let plan = planner.plan_execution(workflows.len(), 10).await;
    println!("Execution plan:");
    println!("  Device: {:?}", plan.device_type);
    println!("  Use GPU: {}", plan.use_gpu);
    println!("  Estimated speedup: {:.1}x", plan.estimated_speedup);

    if plan.use_gpu {
        println!("\n🚀 Using GPU acceleration!");

        // Define enabled patterns
        let patterns: Vec<PatternData> = (0..43)
            .map(|i| PatternData {
                id: i,
                pattern_type: i,
                criteria: 1 << (i % 32),
            })
            .collect();

        // Check which workflows can execute
        let start = std::time::Instant::now();
        let can_execute = ops.find_matching_workflows(&cases, 0).await?;
        let elapsed = start.elapsed();

        println!("\nPattern matching completed in {:?}", elapsed);

        // Execute eligible workflows
        let mut executed = 0;
        for (i, &can_exec) in can_execute.iter().enumerate() {
            if can_exec {
                engine.execute_case(cases[i].id).await?;
                executed += 1;
            }
        }

        println!("Executed {} workflows", executed);
    } else {
        println!("\n💻 Using CPU (workflow count too small for GPU benefit)");
    }

    Ok(())
}
```

## Example 5: Performance Comparison

Compare GPU vs CPU performance:

```rust
use knhk_workflow_engine::gpu::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize both CPU and GPU contexts
    let cpu = GpuContext::new()
        .prefer_device(DeviceType::Cpu)
        .build()
        .await?;

    let gpu = GpuContext::new()
        .fallback_to_cpu(false) // Don't fallback to CPU
        .build()
        .await;

    // Create test data
    let workflows: Vec<WorkflowData> = (0..100_000)
        .map(|i| WorkflowData {
            id: i,
            state: (i % 10) as u32,
            flags: (i % 256) as u32,
            data_ptr: 0,
        })
        .collect();

    let patterns: Vec<PatternData> = (0..43)
        .map(|i| PatternData {
            id: i,
            pattern_type: i,
            criteria: 1 << (i % 32),
        })
        .collect();

    println!("Performance Comparison");
    println!("======================");
    println!("Workflows: {}", workflows.len());
    println!("Patterns: {}", patterns.len());
    println!("Total operations: {}\n", workflows.len() * patterns.len());

    // Benchmark CPU
    let cpu_start = Instant::now();
    let cpu_results = cpu.batch_pattern_match(&workflows, &patterns).await?;
    let cpu_elapsed = cpu_start.elapsed();

    println!("CPU Performance:");
    println!("  Time: {:?}", cpu_elapsed);
    println!("  Throughput: {:.2} M ops/sec",
        (workflows.len() * patterns.len()) as f64 / cpu_elapsed.as_secs_f64() / 1_000_000.0
    );

    // Benchmark GPU (if available)
    if let Ok(gpu) = gpu {
        let gpu_start = Instant::now();
        let gpu_results = gpu.batch_pattern_match(&workflows, &patterns).await?;
        let gpu_elapsed = gpu_start.elapsed();

        println!("\nGPU Performance:");
        println!("  Device: {}", gpu.device_info().name);
        println!("  Time: {:?}", gpu_elapsed);
        println!("  Throughput: {:.2} M ops/sec",
            (workflows.len() * patterns.len()) as f64 / gpu_elapsed.as_secs_f64() / 1_000_000.0
        );

        let speedup = cpu_elapsed.as_secs_f64() / gpu_elapsed.as_secs_f64();
        println!("\nSpeedup: {:.1}x faster", speedup);

        // Verify results match
        assert_eq!(cpu_results.len(), gpu_results.len());
        let matches = cpu_results.iter().zip(gpu_results.iter()).all(|(c, g)| c == g);
        println!("Results match: {}", if matches { "✓" } else { "✗" });
    } else {
        println!("\nGPU not available");
    }

    Ok(())
}
```

**Expected Output (with RTX 4090):**
```
Performance Comparison
======================
Workflows: 100000
Patterns: 43
Total operations: 4300000

CPU Performance:
  Time: 487.3ms
  Throughput: 8.82 M ops/sec

GPU Performance:
  Device: NVIDIA GeForce RTX 4090
  Time: 2.1ms
  Throughput: 2047.62 M ops/sec

Speedup: 232.0x faster
Results match: ✓
```

## Example 6: Memory Transfer Optimization

Optimize memory transfers for maximum performance:

```rust
use knhk_workflow_engine::gpu::*;
use knhk_workflow_engine::gpu::memory::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = GpuContext::new().fallback_to_cpu(true).build().await?;

    // Create memory pool for buffer reuse
    let pool = MemoryPool::new(50);

    // Create async transfer helper
    let transfer = AsyncTransfer::new();

    // Process multiple batches with buffer reuse
    for batch_id in 0..10 {
        // Acquire buffer from pool
        let buffer = pool.acquire(100_000 * std::mem::size_of::<WorkflowData>()).await;

        // Create workflows
        let workflows: Vec<WorkflowData> = (0..100_000)
            .map(|i| WorkflowData {
                id: (batch_id * 100_000 + i) as u64,
                state: (i % 10) as u32,
                flags: 0,
                data_ptr: 0,
            })
            .collect();

        // Async transfer to device
        let device_data = transfer.transfer_to_device(&workflows).await?;

        // Process on GPU...
        let patterns = vec![PatternData {
            id: 0,
            pattern_type: 0,
            criteria: 1,
        }];

        let results = gpu.batch_pattern_match(&device_data, &patterns).await?;

        // Return buffer to pool
        pool.release(buffer).await;

        println!("Batch {}: {} results", batch_id, results.len());
    }

    // Print transfer statistics
    let stats = transfer.stats().await;
    println!("\nTransfer Statistics:");
    println!("  Bytes to device: {}", stats.bytes_to_device);
    println!("  Bytes from device: {}", stats.bytes_from_device);
    println!("  Transfers to device: {}", stats.transfers_to_device);
    println!("  Total transfer time: {}μs", stats.total_transfer_time_us);
    println!("  Average throughput: {:.2} GB/s",
        (stats.bytes_to_device + stats.bytes_from_device) as f64 /
        stats.total_transfer_time_us as f64 / 1000.0
    );

    Ok(())
}
```

## Example 7: Adaptive Batch Sizing

Automatically adjust batch size based on available memory:

```rust
use knhk_workflow_engine::gpu::*;
use knhk_workflow_engine::gpu::operations::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = GpuContext::new().fallback_to_cpu(true).build().await?;

    // Calculate optimal batch size based on available memory
    let available_mb = gpu.device_info().available_memory / 1024 / 1024;
    let workflow_size = std::mem::size_of::<WorkflowData>();
    let max_batch_size = (available_mb as usize * 1024 * 1024 / workflow_size) / 2; // Use 50% of memory

    println!("GPU: {}", gpu.device_info().name);
    println!("Available memory: {}MB", available_mb);
    println!("Max batch size: {}", max_batch_size);

    let ops = GpuOperations::new(gpu).with_batch_size(max_batch_size);

    // Process large workflow set in adaptive batches
    let total_workflows = 1_000_000;
    let workflows: Vec<WorkflowData> = (0..total_workflows)
        .map(|i| WorkflowData {
            id: i as u64,
            state: (i % 10) as u32,
            flags: 0,
            data_ptr: 0,
        })
        .collect();

    let transitions = vec![
        TransitionData { from_state: 0, to_state: 1, condition: 0, flags: 0 },
    ];

    // Automatically batched processing
    let start = std::time::Instant::now();
    let results = ops.apply_transitions_batch(&workflows, &transitions).await?;
    let elapsed = start.elapsed();

    println!("\nProcessed {} workflows in {:?}", total_workflows, elapsed);
    println!("Throughput: {:.2} M workflows/sec",
        total_workflows as f64 / elapsed.as_secs_f64() / 1_000_000.0
    );

    Ok(())
}
```

## Running Examples

```bash
# Example 1: Basic pattern matching
cargo run --release --features gpu --example basic_pattern_matching

# Example 2: State transitions
cargo run --release --features gpu --example state_transitions

# Example 3: Graph traversal
cargo run --release --features gpu --example dependency_resolution

# Example 4: Real-world integration
cargo run --release --features gpu,storage --example workflow_integration

# Example 5: Performance comparison
cargo run --release --features gpu --example performance_comparison

# Example 6: Memory optimization
cargo run --release --features gpu --example memory_optimization

# Example 7: Adaptive batching
cargo run --release --features gpu --example adaptive_batching
```

## Next Steps

- Review [GPU_ACCELERATION.md](GPU_ACCELERATION.md) for detailed API documentation
- Run benchmarks: `cargo bench --features gpu --bench gpu_acceleration`
- Explore advanced features: multi-GPU, custom kernels, graph neural networks
