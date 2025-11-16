//! High-level GPU operations for workflow processing
//!
//! Provides convenient APIs for common workflow operations with automatic
//! batching and optimization.

use super::*;
use crate::case::Case;
use crate::patterns::PatternId;

/// Batch size for automatic batching
const DEFAULT_BATCH_SIZE: usize = 10000;

/// High-level GPU operations
pub struct GpuOperations {
    context: Arc<GpuContext>,
    batch_size: usize,
}

impl GpuOperations {
    /// Create new GPU operations handler
    pub fn new(context: GpuContext) -> Self {
        Self {
            context: Arc::new(context),
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }

    /// Set batch size for operations
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Check which workflows match a pattern
    pub async fn find_matching_workflows(
        &self,
        workflows: &[Case],
        pattern: PatternId,
    ) -> WorkflowResult<Vec<bool>> {
        // Convert workflows to GPU format
        let workflow_data: Vec<WorkflowData> = workflows
            .iter()
            .map(|w| WorkflowData {
                id: w.id.0 as u64,
                state: 0, // Simplified
                flags: 0,
                data_ptr: 0,
            })
            .collect();

        let pattern_data = vec![PatternData {
            id: pattern as u32,
            pattern_type: pattern as u32,
            criteria: 0,
        }];

        self.context
            .batch_pattern_match(&workflow_data, &pattern_data)
            .await
    }

    /// Apply state transitions to multiple workflows in parallel
    pub async fn apply_transitions_batch(
        &self,
        workflows: &[WorkflowData],
        transitions: &[TransitionData],
    ) -> WorkflowResult<Vec<StateData>> {
        // Process in batches to avoid GPU memory exhaustion
        let mut results = Vec::with_capacity(workflows.len());

        for chunk in workflows.chunks(self.batch_size) {
            let batch_results = self
                .context
                .batch_apply_transitions(chunk, transitions)
                .await?;
            results.extend(batch_results);
        }

        Ok(results)
    }

    /// Find execution paths through workflow graph
    pub async fn find_execution_paths(
        &self,
        graph: &GraphData,
        start_states: &[usize],
    ) -> WorkflowResult<Vec<Vec<usize>>> {
        self.context
            .parallel_graph_traversal(graph, start_states)
            .await
    }

    /// Get GPU device information
    pub fn device_info(&self) -> &DeviceInfo {
        self.context.device_info()
    }

    /// Estimate speedup vs CPU for given operation size
    pub fn estimate_speedup(&self, num_operations: usize) -> f64 {
        match self.context.device_info().device_type {
            #[cfg(feature = "gpu-cuda")]
            DeviceType::Cuda => {
                // CUDA typically provides 100-250x speedup for large batches
                if num_operations < 1000 {
                    1.0 // Too small, overhead dominates
                } else if num_operations < 10000 {
                    10.0 + (num_operations as f64 / 1000.0) * 5.0
                } else {
                    100.0 + ((num_operations as f64 / 10000.0).ln() * 50.0).min(150.0)
                }
            }
            #[cfg(feature = "gpu-vulkan")]
            DeviceType::Vulkan => {
                // Vulkan typically 50-150x speedup
                if num_operations < 1000 {
                    1.0
                } else if num_operations < 10000 {
                    5.0 + (num_operations as f64 / 1000.0) * 3.0
                } else {
                    50.0 + ((num_operations as f64 / 10000.0).ln() * 30.0).min(100.0)
                }
            }
            DeviceType::Cpu => {
                // CPU parallelism provides 2-8x speedup
                let num_threads = rayon::current_num_threads() as f64;
                (num_threads * 0.8).min(8.0) // 80% efficiency
            }
        }
    }

    /// Check if GPU acceleration is worthwhile for given size
    pub fn should_use_gpu(&self, num_operations: usize) -> bool {
        match self.context.device_info().device_type {
            DeviceType::Cpu => false, // Already on CPU
            _ => num_operations >= 1000, // Use GPU for 1K+ operations
        }
    }
}

/// Workflow execution planner with GPU optimization
pub struct ExecutionPlanner {
    gpu_ops: Arc<GpuOperations>,
}

impl ExecutionPlanner {
    /// Create new execution planner
    pub fn new(gpu_ops: GpuOperations) -> Self {
        Self {
            gpu_ops: Arc::new(gpu_ops),
        }
    }

    /// Plan optimal execution strategy
    pub async fn plan_execution(
        &self,
        num_workflows: usize,
        num_operations_per_workflow: usize,
    ) -> ExecutionPlan {
        let total_ops = num_workflows * num_operations_per_workflow;
        let estimated_speedup = self.gpu_ops.estimate_speedup(total_ops);
        let use_gpu = self.gpu_ops.should_use_gpu(total_ops);

        ExecutionPlan {
            use_gpu,
            batch_size: if use_gpu { 10000 } else { 1000 },
            estimated_speedup,
            device_type: self.gpu_ops.device_info().device_type,
        }
    }
}

/// Execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Whether to use GPU
    pub use_gpu: bool,
    /// Optimal batch size
    pub batch_size: usize,
    /// Estimated speedup vs single-threaded CPU
    pub estimated_speedup: f64,
    /// Device type to use
    pub device_type: DeviceType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_operations() {
        let ctx = GpuContext::new()
            .fallback_to_cpu(true)
            .build()
            .await
            .unwrap();

        let ops = GpuOperations::new(ctx);

        // Test speedup estimation
        let speedup_small = ops.estimate_speedup(100);
        let speedup_large = ops.estimate_speedup(100000);

        assert!(speedup_large > speedup_small);
    }

    #[tokio::test]
    async fn test_execution_planner() {
        let ctx = GpuContext::new()
            .fallback_to_cpu(true)
            .build()
            .await
            .unwrap();

        let ops = GpuOperations::new(ctx);
        let planner = ExecutionPlanner::new(ops);

        let plan = planner.plan_execution(10000, 10).await;
        assert!(plan.batch_size > 0);
        assert!(plan.estimated_speedup >= 1.0);
    }

    #[tokio::test]
    async fn test_should_use_gpu() {
        let ctx = GpuContext::new()
            .fallback_to_cpu(true)
            .build()
            .await
            .unwrap();

        let ops = GpuOperations::new(ctx);

        // CPU backend should never recommend GPU
        if ops.device_info().device_type == DeviceType::Cpu {
            assert!(!ops.should_use_gpu(10000));
        }
    }
}
