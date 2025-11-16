# Integration Guide: GPU-Accelerated Workflow Engine

Complete guide for integrating GPU acceleration into KNHK workflow engine.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Workflow Engine                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Parser     │  │   Executor   │  │     State    │      │
│  │              │  │              │  │    Manager   │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                  │              │
│         └─────────────────┼──────────────────┘              │
│                           │                                 │
│                  ┌────────▼────────┐                        │
│                  │  GPU Operations │                        │
│                  └────────┬────────┘                        │
│                           │                                 │
│         ┌─────────────────┼─────────────────┐              │
│         │                 │                 │              │
│  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐        │
│  │    CUDA     │  │   Vulkan    │  │   CPU SIMD  │        │
│  │   Backend   │  │   Backend   │  │   Fallback  │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## Step 1: Enable GPU Features

Add to your `Cargo.toml`:

```toml
[dependencies]
knhk-workflow-engine = { version = "1.0", features = ["gpu", "storage"] }

# Or specific backends:
[features]
default = ["gpu-cuda"]  # NVIDIA GPUs
# OR
default = ["gpu-vulkan"]  # Cross-platform (AMD, Intel, ARM)
# OR
default = ["gpu"]  # All backends
```

## Step 2: Initialize GPU Context

Create a shared GPU context at application startup:

```rust
use knhk_workflow_engine::gpu::*;
use std::sync::Arc;

pub struct WorkflowService {
    engine: WorkflowEngine,
    gpu: Arc<GpuContext>,
}

impl WorkflowService {
    pub async fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize workflow engine
        let state_store = StateStore::new(db_path)?;
        let engine = WorkflowEngine::new(state_store);

        // Initialize GPU with automatic backend selection
        let gpu = GpuContext::new()
            .prefer_device(DeviceType::Cuda)  // Prefer CUDA if available
            .fallback_to_cpu(true)            // Always fallback to CPU
            .min_memory_mb(512)               // Require 512MB minimum
            .build()
            .await?;

        info!("GPU initialized: {} ({})",
            gpu.device_info().name,
            gpu.device_info().compute_capability
        );

        Ok(Self {
            engine,
            gpu: Arc::new(gpu),
        })
    }
}
```

## Step 3: Integrate Pattern Matching

Accelerate workflow pattern matching:

```rust
use knhk_workflow_engine::gpu::operations::*;

impl WorkflowService {
    pub async fn check_executable_workflows(
        &self,
    ) -> Result<Vec<CaseId>, Box<dyn std::error::Error>> {
        // Get all active workflows
        let cases = self.engine.list_active_cases().await?;

        // Convert to GPU format
        let workflows: Vec<WorkflowData> = cases
            .iter()
            .map(|case| WorkflowData {
                id: case.id.0 as u64,
                state: case.current_state_index(),
                flags: self.encode_case_flags(case),
                data_ptr: 0,
            })
            .collect();

        // Define enabled patterns
        let patterns = self.get_enabled_patterns();

        // GPU pattern matching
        let ops = GpuOperations::new((*self.gpu).clone());
        let can_execute = ops
            .batch_pattern_match(&workflows, &patterns)
            .await?;

        // Return executable workflow IDs
        let executable_ids: Vec<CaseId> = cases
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                let workflow_idx = i / patterns.len();
                can_execute
                    .iter()
                    .skip(workflow_idx * patterns.len())
                    .take(patterns.len())
                    .any(|&matched| matched)
            })
            .map(|(_, case)| case.id)
            .collect();

        Ok(executable_ids)
    }

    fn encode_case_flags(&self, case: &Case) -> u32 {
        let mut flags = 0u32;

        if case.is_active() {
            flags |= 1 << 0;
        }
        if case.is_suspended() {
            flags |= 1 << 1;
        }
        // Add more flags as needed...

        flags
    }

    fn get_enabled_patterns(&self) -> Vec<PatternData> {
        (0..43)
            .map(|i| PatternData {
                id: i,
                pattern_type: i,
                criteria: self.pattern_criteria(i),
            })
            .collect()
    }

    fn pattern_criteria(&self, pattern_id: u32) -> u64 {
        // Define pattern matching criteria
        match pattern_id {
            0 => 0b0001, // Sequence
            1 => 0b0010, // Parallel Split
            2 => 0b0100, // Synchronization
            // ... more patterns
            _ => 0,
        }
    }
}
```

## Step 4: Accelerate State Transitions

Batch state transitions for concurrent workflows:

```rust
impl WorkflowService {
    pub async fn execute_batch_transitions(
        &self,
        case_ids: &[CaseId],
    ) -> Result<Vec<StateData>, Box<dyn std::error::Error>> {
        // Load workflows
        let cases: Vec<Case> = self.engine.get_cases(case_ids).await?;

        // Convert to GPU format
        let workflows: Vec<WorkflowData> = cases
            .iter()
            .map(|case| WorkflowData {
                id: case.id.0 as u64,
                state: case.current_state_index(),
                flags: self.encode_case_flags(case),
                data_ptr: 0,
            })
            .collect();

        // Define transitions
        let transitions = self.compute_transitions(&cases);

        // GPU batch transitions
        let new_states = self
            .gpu
            .batch_apply_transitions(&workflows, &transitions)
            .await?;

        // Update workflow states
        for (case, new_state) in cases.iter().zip(new_states.iter()) {
            self.engine
                .update_case_state(case.id, new_state.state)
                .await?;
        }

        Ok(new_states)
    }

    fn compute_transitions(&self, cases: &[Case]) -> Vec<TransitionData> {
        // Compute valid transitions for each case
        cases
            .iter()
            .flat_map(|case| {
                let current = case.current_state_index();
                self.get_valid_transitions(current)
            })
            .collect()
    }

    fn get_valid_transitions(&self, state: u32) -> Vec<TransitionData> {
        // Return valid transitions from current state
        // This is workflow-specific logic
        vec![TransitionData {
            from_state: state,
            to_state: state + 1,
            condition: 0,
            flags: 0,
        }]
    }
}
```

## Step 5: Optimize Dependency Resolution

Use GPU for parallel graph traversal:

```rust
impl WorkflowService {
    pub async fn resolve_dependencies(
        &self,
        workflow_spec: &WorkflowSpec,
    ) -> Result<Vec<Vec<usize>>, Box<dyn std::error::Error>> {
        // Build dependency graph
        let graph = self.build_dependency_graph(workflow_spec);

        // Find all entry points
        let entry_points = self.find_entry_points(workflow_spec);

        // GPU graph traversal
        let paths = self.gpu.parallel_graph_traversal(&graph, &entry_points).await?;

        Ok(paths)
    }

    fn build_dependency_graph(&self, spec: &WorkflowSpec) -> GraphData {
        let nodes = spec.tasks.len();
        let mut edges = Vec::new();
        let mut offsets = vec![0];

        for task in &spec.tasks {
            let start = edges.len();

            // Add edges to dependent tasks
            for dep in &task.dependencies {
                edges.push(*dep as u32);
            }

            offsets.push(edges.len());
        }

        GraphData {
            edges,
            offsets,
            node_count: nodes,
        }
    }

    fn find_entry_points(&self, spec: &WorkflowSpec) -> Vec<usize> {
        spec.tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| task.dependencies.is_empty())
            .map(|(i, _)| i)
            .collect()
    }
}
```

## Step 6: Automatic GPU Selection

Use execution planner to automatically choose GPU vs CPU:

```rust
use knhk_workflow_engine::gpu::operations::ExecutionPlanner;

impl WorkflowService {
    pub async fn smart_execute_workflows(
        &self,
        case_ids: &[CaseId],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ops = GpuOperations::new((*self.gpu).clone());
        let planner = ExecutionPlanner::new(ops.clone());

        // Plan execution based on workload size
        let plan = planner
            .plan_execution(case_ids.len(), 10) // 10 ops per workflow
            .await;

        info!("Execution plan:");
        info!("  Device: {:?}", plan.device_type);
        info!("  Use GPU: {}", plan.use_gpu);
        info!("  Estimated speedup: {:.1}x", plan.estimated_speedup);

        if plan.use_gpu {
            // Use GPU acceleration
            self.execute_batch_transitions(case_ids).await?;
        } else {
            // Use CPU (more efficient for small batches)
            for case_id in case_ids {
                self.engine.execute_case(*case_id).await?;
            }
        }

        Ok(())
    }
}
```

## Step 7: Performance Monitoring

Track GPU performance metrics:

```rust
use knhk_workflow_engine::gpu::memory::AsyncTransfer;

impl WorkflowService {
    pub async fn get_gpu_stats(&self) -> GpuStats {
        let device_info = self.gpu.device_info();

        GpuStats {
            device_name: device_info.name.clone(),
            device_type: device_info.device_type,
            available_memory_mb: device_info.available_memory / 1024 / 1024,
            total_memory_mb: device_info.total_memory / 1024 / 1024,
        }
    }
}

#[derive(Debug)]
pub struct GpuStats {
    pub device_name: String,
    pub device_type: DeviceType,
    pub available_memory_mb: usize,
    pub total_memory_mb: usize,
}
```

## Step 8: Error Handling

Implement graceful fallback on GPU errors:

```rust
impl WorkflowService {
    pub async fn resilient_pattern_matching(
        &self,
        workflows: &[WorkflowData],
        patterns: &[PatternData],
    ) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
        // Try GPU first
        match self.gpu.batch_pattern_match(workflows, patterns).await {
            Ok(results) => {
                info!("GPU pattern matching succeeded");
                Ok(results)
            }
            Err(e) => {
                warn!("GPU pattern matching failed, falling back to CPU: {}", e);

                // Fallback to CPU
                let cpu = GpuContext::new()
                    .prefer_device(DeviceType::Cpu)
                    .build()
                    .await?;

                cpu.batch_pattern_match(workflows, patterns).await
            }
        }
    }
}
```

## Step 9: Testing

Test GPU integration:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_pattern_matching() {
        let service = WorkflowService::new("test_db").await.unwrap();

        // Create test workflows
        let workflows = vec![WorkflowData {
            id: 1,
            state: 0,
            flags: 0b1111,
            data_ptr: 0,
        }];

        let patterns = vec![PatternData {
            id: 0,
            pattern_type: 0,
            criteria: 0b1000,
        }];

        let results = service
            .resilient_pattern_matching(&workflows, &patterns)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0]);
    }

    #[tokio::test]
    async fn test_gpu_state_transitions() {
        let service = WorkflowService::new("test_db").await.unwrap();

        // Create and execute workflow
        let spec_id = create_test_workflow(&service).await;
        let case_id = service.engine.create_case(spec_id, json!({})).await.unwrap();

        // Execute with GPU
        let results = service
            .execute_batch_transitions(&[case_id])
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].state, 1);
    }
}
```

## Step 10: Production Deployment

### Configuration

Create `gpu_config.toml`:

```toml
[gpu]
enabled = true
preferred_backend = "cuda"  # "cuda", "vulkan", or "cpu"
fallback_to_cpu = true
min_memory_mb = 512
batch_size = 10000

[gpu.thresholds]
min_workflows_for_gpu = 1000  # Use GPU when >= 1000 workflows
max_batch_size = 100000

[gpu.monitoring]
collect_stats = true
log_performance = true
alert_on_fallback = true
```

### Load Configuration

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct GpuConfig {
    enabled: bool,
    preferred_backend: String,
    fallback_to_cpu: bool,
    min_memory_mb: usize,
    batch_size: usize,
}

impl WorkflowService {
    pub async fn from_config(
        config_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config: GpuConfig = load_config(config_path)?;

        let gpu = if config.enabled {
            let backend = match config.preferred_backend.as_str() {
                "cuda" => DeviceType::Cuda,
                "vulkan" => DeviceType::Vulkan,
                _ => DeviceType::Cpu,
            };

            GpuContext::new()
                .prefer_device(backend)
                .fallback_to_cpu(config.fallback_to_cpu)
                .min_memory_mb(config.min_memory_mb)
                .build()
                .await?
        } else {
            GpuContext::new()
                .prefer_device(DeviceType::Cpu)
                .build()
                .await?
        };

        // ... rest of initialization
    }
}
```

## Performance Tuning

### Batch Size Optimization

```rust
// Find optimal batch size empirically
async fn find_optimal_batch_size(gpu: &GpuContext) -> usize {
    let test_sizes = [1000, 5000, 10000, 50000, 100000];
    let mut best_size = 10000;
    let mut best_throughput = 0.0;

    for &size in &test_sizes {
        let workflows = create_test_workflows(size);
        let patterns = create_test_patterns(10);

        let start = std::time::Instant::now();
        let _ = gpu.batch_pattern_match(&workflows, &patterns).await;
        let elapsed = start.elapsed().as_secs_f64();

        let throughput = (size * 10) as f64 / elapsed;

        if throughput > best_throughput {
            best_throughput = throughput;
            best_size = size;
        }
    }

    best_size
}
```

## Next Steps

1. Review [GPU_ACCELERATION.md](GPU_ACCELERATION.md) for API details
2. Run benchmarks: `cargo bench --features gpu`
3. Monitor GPU utilization in production
4. Tune batch sizes for your workload
5. Consider multi-GPU scaling for extreme loads

## Support

- Documentation: See `docs/gpu/` directory
- Examples: See `examples/gpu_*.rs` files
- Benchmarks: `cargo bench --features gpu --bench gpu_acceleration`
- Issues: Report GPU-specific issues with `[GPU]` tag
