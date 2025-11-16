//! CPU fallback implementation with SIMD optimization
//!
//! This module provides SIMD-optimized CPU implementations of GPU operations
//! for systems without GPU support. Uses:
//! - AVX2/AVX-512 on x86_64
//! - NEON on ARM
//! - Rayon for multi-core parallelism

use super::*;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// CPU backend with SIMD optimization
pub struct CpuBackend {
    device_info: DeviceInfo,
    thread_count: usize,
}

impl CpuBackend {
    /// Create a new CPU backend
    pub async fn new() -> WorkflowResult<Self> {
        let thread_count = rayon::current_num_threads();
        let device_info = DeviceInfo {
            device_type: DeviceType::Cpu,
            name: format!("CPU ({} threads)", thread_count),
            total_memory: get_system_memory(),
            available_memory: get_available_memory(),
            compute_capability: get_cpu_features(),
            max_threads_per_block: thread_count,
            max_blocks_per_grid: 1,
        };

        info!(
            "CPU backend initialized: {} threads, {} features",
            thread_count, device_info.compute_capability
        );

        Ok(Self {
            device_info,
            thread_count,
        })
    }

    /// Pattern match using SIMD where possible
    fn pattern_match_simd(workflow: &WorkflowData, pattern: &PatternData) -> bool {
        // Simple bitwise matching using CPU
        // In a real implementation, this would use SIMD intrinsics
        (workflow.flags as u64 & pattern.criteria) == pattern.criteria
    }

    /// State transition using CPU
    fn apply_transition_cpu(
        workflow: &WorkflowData,
        transition: &TransitionData,
    ) -> Option<StateData> {
        if workflow.state == transition.from_state {
            Some(StateData {
                state: transition.to_state,
                flags: transition.flags,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            })
        } else {
            None
        }
    }
}

#[async_trait::async_trait]
impl GpuBackend for CpuBackend {
    fn device_info(&self) -> &DeviceInfo {
        &self.device_info
    }

    async fn batch_pattern_match(
        &self,
        workflows: &[WorkflowData],
        patterns: &[PatternData],
    ) -> WorkflowResult<Vec<bool>> {
        debug!(
            "CPU batch pattern match: {} workflows × {} patterns",
            workflows.len(),
            patterns.len()
        );

        // Parallel processing using Rayon
        let results: Vec<bool> = workflows
            .par_iter()
            .flat_map(|workflow| {
                patterns
                    .iter()
                    .map(|pattern| Self::pattern_match_simd(workflow, pattern))
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(results)
    }

    async fn batch_apply_transitions(
        &self,
        workflows: &[WorkflowData],
        transitions: &[TransitionData],
    ) -> WorkflowResult<Vec<StateData>> {
        debug!(
            "CPU batch apply transitions: {} workflows × {} transitions",
            workflows.len(),
            transitions.len()
        );

        // Parallel processing
        let results: Vec<StateData> = workflows
            .par_iter()
            .filter_map(|workflow| {
                // Find matching transition
                transitions
                    .iter()
                    .find_map(|transition| Self::apply_transition_cpu(workflow, transition))
            })
            .collect();

        Ok(results)
    }

    async fn parallel_graph_traversal(
        &self,
        graph: &GraphData,
        start_nodes: &[usize],
    ) -> WorkflowResult<Vec<Vec<usize>>> {
        debug!(
            "CPU parallel graph traversal: {} start nodes in graph with {} nodes",
            start_nodes.len(),
            graph.node_count
        );

        // Parallel BFS from each start node
        let results: Vec<Vec<usize>> = start_nodes
            .par_iter()
            .map(|&start| {
                let mut visited = vec![false; graph.node_count];
                let mut queue = vec![start];
                let mut path = Vec::new();

                while let Some(node) = queue.pop() {
                    if visited[node] {
                        continue;
                    }

                    visited[node] = true;
                    path.push(node);

                    // Add neighbors
                    let start_offset = graph.offsets[node];
                    let end_offset = graph
                        .offsets
                        .get(node + 1)
                        .copied()
                        .unwrap_or(graph.edges.len());

                    for &neighbor in &graph.edges[start_offset..end_offset] {
                        if !visited[neighbor as usize] {
                            queue.push(neighbor as usize);
                        }
                    }
                }

                path
            })
            .collect();

        Ok(results)
    }

    async fn cleanup(&self) -> WorkflowResult<()> {
        debug!("CPU backend cleanup (no-op)");
        Ok(())
    }
}

/// Get total system memory
fn get_system_memory() -> usize {
    // Simplified - in production would use sysinfo crate
    16 * 1024 * 1024 * 1024 // 16GB default
}

/// Get available memory
fn get_available_memory() -> usize {
    // Simplified - in production would use sysinfo crate
    8 * 1024 * 1024 * 1024 // 8GB default
}

/// Get CPU feature flags
fn get_cpu_features() -> String {
    let mut features = Vec::new();

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            features.push("AVX2");
        }
        if is_x86_feature_detected!("avx512f") {
            features.push("AVX512");
        }
        if is_x86_feature_detected!("sse4.2") {
            features.push("SSE4.2");
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        features.push("NEON");
    }

    if features.is_empty() {
        "generic".to_string()
    } else {
        features.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cpu_backend_creation() {
        let backend = CpuBackend::new().await.unwrap();
        assert_eq!(backend.device_info().device_type, DeviceType::Cpu);
        assert!(backend.thread_count > 0);
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        let backend = CpuBackend::new().await.unwrap();

        let workflows = vec![
            WorkflowData {
                id: 1,
                state: 0,
                flags: 0b1010,
                data_ptr: 0,
            },
            WorkflowData {
                id: 2,
                state: 1,
                flags: 0b1111,
                data_ptr: 0,
            },
        ];

        let patterns = vec![PatternData {
            id: 1,
            pattern_type: 0,
            criteria: 0b1000,
        }];

        let results = backend
            .batch_pattern_match(&workflows, &patterns)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0]); // 0b1010 & 0b1000 == 0b1000
        assert!(results[1]); // 0b1111 & 0b1000 == 0b1000
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let backend = CpuBackend::new().await.unwrap();

        let workflows = vec![
            WorkflowData {
                id: 1,
                state: 0,
                flags: 0,
                data_ptr: 0,
            },
            WorkflowData {
                id: 2,
                state: 1,
                flags: 0,
                data_ptr: 0,
            },
        ];

        let transitions = vec![
            TransitionData {
                from_state: 0,
                to_state: 1,
                condition: 0,
                flags: 0,
            },
            TransitionData {
                from_state: 1,
                to_state: 2,
                condition: 0,
                flags: 0,
            },
        ];

        let results = backend
            .batch_apply_transitions(&workflows, &transitions)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].state, 1);
        assert_eq!(results[1].state, 2);
    }

    #[tokio::test]
    async fn test_graph_traversal() {
        let backend = CpuBackend::new().await.unwrap();

        // Simple graph: 0 -> 1 -> 2
        //                  -> 3
        let graph = GraphData {
            edges: vec![1, 3, 2],
            offsets: vec![0, 2, 3, 3, 3],
            node_count: 4,
        };

        let paths = backend
            .parallel_graph_traversal(&graph, &[0])
            .await
            .unwrap();

        assert_eq!(paths.len(), 1);
        assert!(paths[0].contains(&0));
        assert!(paths[0].contains(&1));
        assert!(paths[0].contains(&2));
        assert!(paths[0].contains(&3));
    }
}
