//! CUDA backend for NVIDIA GPUs
//!
//! Provides high-performance GPU acceleration using CUDA for workflow operations.
//! Requires NVIDIA GPU with CUDA support (compute capability 5.0+).

#![cfg(feature = "gpu-cuda")]

use super::*;
use cudarc::driver::*;
use std::sync::Arc;

/// CUDA GPU backend
pub struct CudaBackend {
    device: Arc<CudaDevice>,
    device_info: DeviceInfo,
}

impl CudaBackend {
    /// Create a new CUDA backend
    pub async fn new(min_memory_mb: usize) -> WorkflowResult<Self> {
        // Initialize CUDA
        let device = CudaDevice::new(0).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to initialize CUDA device: {}", e))
        })?;

        // Get device properties
        let device_name = device.name().map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to get device name: {}", e))
        })?;

        let total_memory = device.total_memory().map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to get total memory: {}", e))
        })?;

        let free_memory = device.free_memory().map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to get free memory: {}", e))
        })?;

        // Check minimum memory requirement
        if free_memory < (min_memory_mb * 1024 * 1024) {
            return Err(WorkflowError::ExecutionError(format!(
                "Insufficient GPU memory: {}MB available, {}MB required",
                free_memory / 1024 / 1024,
                min_memory_mb
            )));
        }

        let compute_capability = format!(
            "{}.{}",
            device.compute_capability().0,
            device.compute_capability().1
        );

        let device_info = DeviceInfo {
            device_type: DeviceType::Cuda,
            name: device_name,
            total_memory,
            available_memory: free_memory,
            compute_capability,
            max_threads_per_block: 1024, // Common for modern GPUs
            max_blocks_per_grid: 65535,
        };

        info!(
            "CUDA backend initialized: {} (compute {}, {}MB free)",
            device_info.name,
            device_info.compute_capability,
            device_info.available_memory / 1024 / 1024
        );

        Ok(Self {
            device: Arc::new(device),
            device_info,
        })
    }

    /// Compile and load CUDA kernel
    fn load_kernel(&self, kernel_name: &str, ptx_code: &str) -> WorkflowResult<CudaFunction> {
        self.device
            .load_ptx(
                ptx_code.into(),
                kernel_name,
                &[kernel_name],
            )
            .map_err(|e| {
                WorkflowError::ExecutionError(format!("Failed to load kernel {}: {}", kernel_name, e))
            })?;

        self.device
            .get_func(kernel_name, kernel_name)
            .ok_or_else(|| {
                WorkflowError::ExecutionError(format!("Kernel {} not found", kernel_name))
            })
    }
}

#[async_trait::async_trait]
impl GpuBackend for CudaBackend {
    fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    async fn batch_pattern_match(
        &self,
        workflows: &[WorkflowData],
        patterns: &[PatternData],
    ) -> WorkflowResult<Vec<bool>> {
        debug!(
            "CUDA batch pattern match: {} workflows × {} patterns",
            workflows.len(),
            patterns.len()
        );

        let num_workflows = workflows.len();
        let num_patterns = patterns.len();
        let total_ops = num_workflows * num_patterns;

        // Allocate device memory
        let workflows_dev = self.device.htod_copy(workflows).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to copy workflows to device: {}", e))
        })?;

        let patterns_dev = self.device.htod_copy(patterns).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to copy patterns to device: {}", e))
        })?;

        let mut results = vec![0u32; total_ops];
        let results_dev = self.device.htod_copy(&results).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to allocate results buffer: {}", e))
        })?;

        // Load and launch kernel
        let kernel = self.load_kernel("pattern_match_kernel", PATTERN_MATCH_KERNEL)?;

        let threads_per_block = 256;
        let num_blocks = (total_ops + threads_per_block - 1) / threads_per_block;

        let config = LaunchConfig {
            grid_dim: (num_blocks as u32, 1, 1),
            block_dim: (threads_per_block as u32, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            kernel
                .launch(
                    config,
                    (
                        &workflows_dev,
                        &patterns_dev,
                        &results_dev,
                        num_workflows,
                        num_patterns,
                    ),
                )
                .map_err(|e| {
                    WorkflowError::ExecutionError(format!("Failed to launch kernel: {}", e))
                })?;
        }

        // Copy results back
        self.device.dtoh_sync_copy_into(&results_dev, &mut results).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to copy results from device: {}", e))
        })?;

        // Convert to bool
        Ok(results.iter().map(|&r| r != 0).collect())
    }

    async fn batch_apply_transitions(
        &self,
        workflows: &[WorkflowData],
        transitions: &[TransitionData],
    ) -> WorkflowResult<Vec<StateData>> {
        debug!(
            "CUDA batch apply transitions: {} workflows × {} transitions",
            workflows.len(),
            transitions.len()
        );

        let num_workflows = workflows.len();
        let num_transitions = transitions.len();

        // Allocate device memory
        let workflows_dev = self.device.htod_copy(workflows).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to copy workflows to device: {}", e))
        })?;

        let transitions_dev = self.device.htod_copy(transitions).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to copy transitions to device: {}", e))
        })?;

        let mut results = vec![
            StateData {
                state: 0,
                flags: 0,
                timestamp: 0
            };
            num_workflows
        ];
        let results_dev = self.device.htod_copy(&results).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to allocate results buffer: {}", e))
        })?;

        // Load and launch kernel
        let kernel = self.load_kernel("state_transition_kernel", STATE_TRANSITION_KERNEL)?;

        let threads_per_block = 256;
        let num_blocks = (num_workflows + threads_per_block - 1) / threads_per_block;

        let config = LaunchConfig {
            grid_dim: (num_blocks as u32, 1, 1),
            block_dim: (threads_per_block as u32, 1, 1),
            shared_mem_bytes: 0,
        };

        unsafe {
            kernel
                .launch(
                    config,
                    (
                        &workflows_dev,
                        &transitions_dev,
                        &results_dev,
                        num_workflows,
                        num_transitions,
                    ),
                )
                .map_err(|e| {
                    WorkflowError::ExecutionError(format!("Failed to launch kernel: {}", e))
                })?;
        }

        // Copy results back
        self.device.dtoh_sync_copy_into(&results_dev, &mut results).map_err(|e| {
            WorkflowError::ExecutionError(format!("Failed to copy results from device: {}", e))
        })?;

        Ok(results)
    }

    async fn parallel_graph_traversal(
        &self,
        graph: &GraphData,
        start_nodes: &[usize],
    ) -> WorkflowResult<Vec<Vec<usize>>> {
        debug!(
            "CUDA parallel graph traversal: {} start nodes in graph with {} nodes",
            start_nodes.len(),
            graph.node_count
        );

        // For graph traversal, we'll use a simplified approach
        // Real implementation would use GPU-optimized BFS/DFS algorithms

        // For now, fall back to CPU for complex graph operations
        warn!("Graph traversal falling back to CPU (GPU implementation pending)");

        let cpu_backend = super::cpu_fallback::CpuBackend::new().await?;
        cpu_backend.parallel_graph_traversal(graph, start_nodes).await
    }

    async fn cleanup(&self) -> WorkflowResult<()> {
        debug!("CUDA backend cleanup");
        // CudaDevice handles cleanup automatically via Drop
        Ok(())
    }
}

/// CUDA kernel for pattern matching
const PATTERN_MATCH_KERNEL: &str = r#"
extern "C" __global__ void pattern_match_kernel(
    const unsigned long long* workflows,
    const unsigned long long* patterns,
    unsigned int* results,
    int num_workflows,
    int num_patterns
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    int total_ops = num_workflows * num_patterns;

    if (idx >= total_ops) return;

    int workflow_idx = idx / num_patterns;
    int pattern_idx = idx % num_patterns;

    // Workflow structure: [id, state, flags, data_ptr]
    unsigned int workflow_flags = ((unsigned int*)workflows)[workflow_idx * 4 + 2];

    // Pattern structure: [id, pattern_type, criteria_low, criteria_high]
    unsigned long long pattern_criteria = patterns[pattern_idx * 2 + 1];
    unsigned int criteria = (unsigned int)(pattern_criteria & 0xFFFFFFFF);

    // Match if workflow flags contain all pattern criteria bits
    results[idx] = ((workflow_flags & criteria) == criteria) ? 1 : 0;
}
"#;

/// CUDA kernel for state transitions
const STATE_TRANSITION_KERNEL: &str = r#"
extern "C" __global__ void state_transition_kernel(
    const unsigned long long* workflows,
    const unsigned int* transitions,
    unsigned long long* results,
    int num_workflows,
    int num_transitions
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;

    if (idx >= num_workflows) return;

    // Workflow structure: [id, state, flags, data_ptr]
    unsigned int current_state = ((unsigned int*)workflows)[idx * 4 + 1];

    // Find matching transition
    for (int i = 0; i < num_transitions; i++) {
        // Transition structure: [from_state, to_state, condition, flags]
        unsigned int from_state = transitions[i * 4];
        unsigned int to_state = transitions[i * 4 + 1];
        unsigned int flags = transitions[i * 4 + 3];

        if (current_state == from_state) {
            // Result structure: [state, flags, timestamp]
            ((unsigned int*)results)[idx * 4] = to_state;
            ((unsigned int*)results)[idx * 4 + 1] = flags;
            ((unsigned long long*)results)[idx * 2 + 1] = 0; // timestamp (filled by host)
            break;
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run if CUDA is available
    async fn test_cuda_backend_creation() {
        let result = CudaBackend::new(256).await;
        if result.is_ok() {
            let backend = result.unwrap();
            assert_eq!(backend.device_info().device_type, DeviceType::Cuda);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_cuda_pattern_matching() {
        if let Ok(backend) = CudaBackend::new(256).await {
            let workflows = vec![WorkflowData {
                id: 1,
                state: 0,
                flags: 0b1111,
                data_ptr: 0,
            }];

            let patterns = vec![PatternData {
                id: 1,
                pattern_type: 0,
                criteria: 0b1000,
            }];

            let results = backend
                .batch_pattern_match(&workflows, &patterns)
                .await
                .unwrap();

            assert_eq!(results.len(), 1);
            assert!(results[0]);
        }
    }
}
