//! GPU-accelerated workflow processing
//!
//! This module provides GPU acceleration for embarrassingly parallel workflow operations:
//! - Batch pattern matching (100K+ workflows in parallel)
//! - Parallel state transitions
//! - Vector operations on workflow data
//! - Parallel graph traversal
//!
//! Supports multiple backends with graceful degradation:
//! - CUDA (NVIDIA GPUs) - best performance on NVIDIA hardware
//! - Vulkan Compute (cross-platform) - good performance on all modern GPUs
//! - CPU fallback with SIMD - automatic fallback when GPU unavailable
//!
//! # Performance Targets
//!
//! | Operation | CPU (32-core) | GPU (RTX 4090) | Speedup |
//! |-----------|---------------|----------------|---------|
//! | 100K pattern matches | 500ms | 2ms | 250x |
//! | 1M state transitions | 2s | 10ms | 200x |
//! | Graph traversal (10K nodes) | 100ms | 1ms | 100x |
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::gpu::{GpuContext, DeviceType};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize GPU context with automatic backend selection
//! let gpu = GpuContext::new()
//!     .prefer_device(DeviceType::Cuda)
//!     .fallback_to_cpu(true)
//!     .build().await?;
//!
//! // Batch pattern matching
//! let workflows = vec![/* ... */];
//! let patterns = vec![/* ... */];
//! let matches = gpu.batch_pattern_match(&workflows, &patterns).await?;
//!
//! // Parallel state transitions
//! let transitions = vec![/* ... */];
//! let new_states = gpu.batch_apply_transitions(&workflows, &transitions).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[cfg(feature = "gpu-cuda")]
pub mod cuda;
#[cfg(feature = "gpu-vulkan")]
pub mod vulkan;
pub mod cpu_fallback;
pub mod memory;
pub mod operations;

/// GPU device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// NVIDIA CUDA devices
    #[cfg(feature = "gpu-cuda")]
    Cuda,
    /// Vulkan compute (cross-platform)
    #[cfg(feature = "gpu-vulkan")]
    Vulkan,
    /// CPU fallback with SIMD
    Cpu,
}

/// GPU device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// Device type
    pub device_type: DeviceType,
    /// Device name
    pub name: String,
    /// Total memory in bytes
    pub total_memory: usize,
    /// Available memory in bytes
    pub available_memory: usize,
    /// Compute capability (CUDA) or API version (Vulkan)
    pub compute_capability: String,
    /// Maximum threads per block
    pub max_threads_per_block: usize,
    /// Maximum blocks per grid
    pub max_blocks_per_grid: usize,
}

/// GPU context for workflow acceleration
pub struct GpuContext {
    backend: Arc<RwLock<Box<dyn GpuBackend>>>,
    device_info: DeviceInfo,
}

/// Trait for GPU backends (CUDA, Vulkan, CPU)
#[async_trait::async_trait]
pub trait GpuBackend: Send + Sync {
    /// Get device information
    fn device_info(&self) -> &DeviceInfo;

    /// Batch pattern matching
    async fn batch_pattern_match(
        &self,
        workflows: &[WorkflowData],
        patterns: &[PatternData],
    ) -> WorkflowResult<Vec<bool>>;

    /// Batch state transitions
    async fn batch_apply_transitions(
        &self,
        workflows: &[WorkflowData],
        transitions: &[TransitionData],
    ) -> WorkflowResult<Vec<StateData>>;

    /// Parallel graph traversal
    async fn parallel_graph_traversal(
        &self,
        graph: &GraphData,
        start_nodes: &[usize],
    ) -> WorkflowResult<Vec<Vec<usize>>>;

    /// Cleanup resources
    async fn cleanup(&self) -> WorkflowResult<()>;
}

/// Workflow data for GPU processing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "gpu-cuda", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct WorkflowData {
    /// Workflow ID
    pub id: u64,
    /// Current state index
    pub state: u32,
    /// Flags bitfield
    pub flags: u32,
    /// Data pointer (for complex data)
    pub data_ptr: u64,
}

/// Pattern data for GPU processing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "gpu-cuda", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct PatternData {
    /// Pattern ID
    pub id: u32,
    /// Pattern type
    pub pattern_type: u32,
    /// Match criteria bitfield
    pub criteria: u64,
}

/// State transition data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "gpu-cuda", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct TransitionData {
    /// Source state
    pub from_state: u32,
    /// Target state
    pub to_state: u32,
    /// Transition condition
    pub condition: u32,
    /// Transition flags
    pub flags: u32,
}

/// State data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "gpu-cuda", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[repr(C)]
pub struct StateData {
    /// State index
    pub state: u32,
    /// State flags
    pub flags: u32,
    /// Timestamp
    pub timestamp: u64,
}

/// Graph data for traversal
#[derive(Debug, Clone)]
pub struct GraphData {
    /// Adjacency list (flattened)
    pub edges: Vec<u32>,
    /// Offsets into edges array for each node
    pub offsets: Vec<usize>,
    /// Node count
    pub node_count: usize,
}

/// GPU context builder
pub struct GpuContextBuilder {
    preferred_device: Option<DeviceType>,
    fallback_to_cpu: bool,
    min_memory_mb: usize,
}

impl GpuContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            preferred_device: None,
            fallback_to_cpu: true,
            min_memory_mb: 512, // 512MB minimum
        }
    }

    /// Prefer a specific device type
    pub fn prefer_device(mut self, device_type: DeviceType) -> Self {
        self.preferred_device = Some(device_type);
        self
    }

    /// Enable/disable CPU fallback
    pub fn fallback_to_cpu(mut self, enable: bool) -> Self {
        self.fallback_to_cpu = enable;
        self
    }

    /// Set minimum required memory in MB
    pub fn min_memory_mb(mut self, mb: usize) -> Self {
        self.min_memory_mb = mb;
        self
    }

    /// Build the GPU context
    pub async fn build(self) -> WorkflowResult<GpuContext> {
        // Try preferred device first
        if let Some(preferred) = self.preferred_device {
            match self.try_create_backend(preferred).await {
                Ok(backend) => {
                    let device_info = backend.device_info().clone();
                    info!(
                        "GPU context initialized: {} ({})",
                        device_info.name, device_info.compute_capability
                    );
                    return Ok(GpuContext {
                        backend: Arc::new(RwLock::new(backend)),
                        device_info,
                    });
                }
                Err(e) => {
                    warn!("Failed to initialize preferred device {:?}: {}", preferred, e);
                }
            }
        }

        // Try all available backends
        #[cfg(feature = "gpu-cuda")]
        {
            match self.try_create_backend(DeviceType::Cuda).await {
                Ok(backend) => {
                    let device_info = backend.device_info().clone();
                    info!(
                        "GPU context initialized (CUDA): {} ({})",
                        device_info.name, device_info.compute_capability
                    );
                    return Ok(GpuContext {
                        backend: Arc::new(RwLock::new(backend)),
                        device_info,
                    });
                }
                Err(e) => {
                    debug!("CUDA not available: {}", e);
                }
            }
        }

        #[cfg(feature = "gpu-vulkan")]
        {
            match self.try_create_backend(DeviceType::Vulkan).await {
                Ok(backend) => {
                    let device_info = backend.device_info().clone();
                    info!(
                        "GPU context initialized (Vulkan): {} ({})",
                        device_info.name, device_info.compute_capability
                    );
                    return Ok(GpuContext {
                        backend: Arc::new(RwLock::new(backend)),
                        device_info,
                    });
                }
                Err(e) => {
                    debug!("Vulkan not available: {}", e);
                }
            }
        }

        // Fallback to CPU
        if self.fallback_to_cpu {
            match self.try_create_backend(DeviceType::Cpu).await {
                Ok(backend) => {
                    let device_info = backend.device_info().clone();
                    info!("GPU context initialized (CPU fallback): {}", device_info.name);
                    return Ok(GpuContext {
                        backend: Arc::new(RwLock::new(backend)),
                        device_info,
                    });
                }
                Err(e) => {
                    return Err(WorkflowError::ExecutionError(format!(
                        "Failed to initialize CPU fallback: {}",
                        e
                    )));
                }
            }
        }

        Err(WorkflowError::ExecutionError(
            "No GPU backend available and CPU fallback disabled".to_string(),
        ))
    }

    async fn try_create_backend(
        &self,
        device_type: DeviceType,
    ) -> WorkflowResult<Box<dyn GpuBackend>> {
        match device_type {
            #[cfg(feature = "gpu-cuda")]
            DeviceType::Cuda => {
                let backend = cuda::CudaBackend::new(self.min_memory_mb).await?;
                Ok(Box::new(backend))
            }
            #[cfg(feature = "gpu-vulkan")]
            DeviceType::Vulkan => {
                let backend = vulkan::VulkanBackend::new(self.min_memory_mb).await?;
                Ok(Box::new(backend))
            }
            DeviceType::Cpu => {
                let backend = cpu_fallback::CpuBackend::new().await?;
                Ok(Box::new(backend))
            }
            #[allow(unreachable_patterns)]
            _ => Err(WorkflowError::ExecutionError(format!(
                "Device type {:?} not enabled (missing feature flag)",
                device_type
            ))),
        }
    }
}

impl Default for GpuContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuContext {
    /// Create a new GPU context builder
    pub fn new() -> GpuContextBuilder {
        GpuContextBuilder::new()
    }

    /// Get device information
    pub fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    /// Batch pattern matching
    pub async fn batch_pattern_match(
        &self,
        workflows: &[WorkflowData],
        patterns: &[PatternData],
    ) -> WorkflowResult<Vec<bool>> {
        let backend = self.backend.read().await;
        backend.batch_pattern_match(workflows, patterns).await
    }

    /// Batch state transitions
    pub async fn batch_apply_transitions(
        &self,
        workflows: &[WorkflowData],
        transitions: &[TransitionData],
    ) -> WorkflowResult<Vec<StateData>> {
        let backend = self.backend.read().await;
        backend.batch_apply_transitions(workflows, transitions).await
    }

    /// Parallel graph traversal
    pub async fn parallel_graph_traversal(
        &self,
        graph: &GraphData,
        start_nodes: &[usize],
    ) -> WorkflowResult<Vec<Vec<usize>>> {
        let backend = self.backend.read().await;
        backend.parallel_graph_traversal(graph, start_nodes).await
    }

    /// Cleanup GPU resources
    pub async fn cleanup(self) -> WorkflowResult<()> {
        let backend = self.backend.write().await;
        backend.cleanup().await
    }
}

impl Default for GpuContext {
    fn default() -> Self {
        // This will panic if called, but we keep it for trait implementation
        // Users should use GpuContext::new().build().await instead
        panic!("GpuContext must be created using GpuContext::new().build().await")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_context_builder() {
        let result = GpuContext::new()
            .fallback_to_cpu(true)
            .min_memory_mb(256)
            .build()
            .await;

        // Should at least get CPU fallback
        assert!(result.is_ok());
        let ctx = result.unwrap();
        assert!(ctx.device_info().name.len() > 0);
    }

    #[tokio::test]
    async fn test_cpu_fallback() {
        let ctx = GpuContext::new()
            .prefer_device(DeviceType::Cpu)
            .build()
            .await
            .unwrap();

        assert_eq!(ctx.device_info().device_type, DeviceType::Cpu);
    }
}
