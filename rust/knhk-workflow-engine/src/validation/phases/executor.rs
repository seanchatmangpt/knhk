//! Phase Executor - Concurrent Phase Execution
//!
//! Implements parallel and sequential phase execution with:
//! - Async/await for concurrent execution
//! - Dependency resolution
//! - Error handling and recovery
//! - Performance tracking

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use tokio::task::JoinSet;

use super::core::{Phase, PhaseContext, PhaseResult, PhaseStatus};
use crate::error::{WorkflowError, WorkflowResult};

/// Phase executor for concurrent phase execution
pub struct PhaseExecutor {
    /// Maximum number of concurrent phases
    max_concurrent: usize,
    /// Enable parallel execution
    parallel: bool,
}

impl PhaseExecutor {
    /// Create a new phase executor
    pub fn new() -> Self {
        Self {
            max_concurrent: num_cpus::get(),
            parallel: true,
        }
    }

    /// Set maximum concurrent phases
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max.max(1);
        self
    }

    /// Enable or disable parallel execution
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }

    /// Execute a single phase
    pub async fn execute_phase<T, P>(
        &self,
        phase: &P,
        ctx: PhaseContext,
    ) -> WorkflowResult<PhaseResult<T>>
    where
        T: Clone + Debug + Send + Sync + 'static,
        P: Phase<T> + ?Sized,
    {
        let start = Instant::now();

        // Pre-execute hook
        phase.pre_execute(&ctx).await?;

        // Execute phase
        let mut result = phase.execute(ctx.clone()).await?;

        // Post-execute hook
        phase.post_execute(&ctx, &result).await?;

        // Ensure duration is set
        if result.duration.as_secs() == 0 {
            result.duration = start.elapsed();
        }

        Ok(result)
    }

    /// Execute multiple phases concurrently
    ///
    /// # Type Parameters
    /// - T: Output type (must be the same for all phases)
    ///
    /// # Returns
    /// Vector of phase results in execution order
    pub async fn execute_phases<T>(
        &self,
        phases: Vec<(String, Arc<dyn Phase<T, Output = WorkflowResult<PhaseResult<T>>> + Send + Sync>)>,
        ctx: PhaseContext,
    ) -> WorkflowResult<Vec<PhaseResult<T>>>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        if !self.parallel || phases.len() == 1 {
            // Sequential execution
            return self.execute_sequential(phases, ctx).await;
        }

        // Parallel execution with semaphore for concurrency control
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
        let mut join_set = JoinSet::new();

        for (name, phase) in phases {
            let ctx = ctx.clone();
            let semaphore = semaphore.clone();

            join_set.spawn(async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.map_err(|e| {
                    WorkflowError::InvalidSpecification(format!("Semaphore error: {}", e))
                })?;

                // Execute phase
                let start = Instant::now();
                let result = phase.execute(ctx).await;

                // Add execution time to metrics
                match result {
                    Ok(mut r) => {
                        r.duration = start.elapsed();
                        Ok((name, r))
                    }
                    Err(e) => Err(e),
                }
            });
        }

        // Collect results
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((_name, phase_result))) => {
                    results.push(phase_result);
                }
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(WorkflowError::InvalidSpecification(format!(
                        "Task join error: {}",
                        e
                    )))
                }
            }
        }

        Ok(results)
    }

    /// Execute phases sequentially
    async fn execute_sequential<T>(
        &self,
        phases: Vec<(String, Arc<dyn Phase<T, Output = WorkflowResult<PhaseResult<T>>> + Send + Sync>)>,
        ctx: PhaseContext,
    ) -> WorkflowResult<Vec<PhaseResult<T>>>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        let mut results = Vec::new();

        for (_name, phase) in phases {
            let start = Instant::now();
            let mut result = phase.execute(ctx.clone()).await?;
            result.duration = start.elapsed();
            results.push(result);
        }

        Ok(results)
    }

    /// Execute phases with dependency resolution
    ///
    /// Phases are executed in topological order respecting dependencies.
    /// Independent phases can run in parallel.
    pub async fn execute_with_dependencies<T>(
        &self,
        phases: Vec<(String, Arc<dyn Phase<T, Output = WorkflowResult<PhaseResult<T>>> + Send + Sync>, Vec<String>)>,
        ctx: PhaseContext,
    ) -> WorkflowResult<HashMap<String, PhaseResult<T>>>
    where
        T: Clone + Debug + Send + Sync + 'static,
    {
        // Build dependency graph
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut phase_map: HashMap<String, Arc<dyn Phase<T, Output = WorkflowResult<PhaseResult<T>>> + Send + Sync>> =
            HashMap::new();

        for (name, phase, deps) in phases {
            graph.insert(name.clone(), deps.clone());
            in_degree.insert(name.clone(), deps.len());
            phase_map.insert(name.clone(), phase);
        }

        // Topological sort with parallel execution per level
        let mut results: HashMap<String, PhaseResult<T>> = HashMap::new();
        let mut ready: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        while !ready.is_empty() {
            // Execute all ready phases in parallel
            let mut join_set = JoinSet::new();

            for name in ready.drain(..) {
                if let Some(phase) = phase_map.remove(&name) {
                    let ctx = ctx.clone();
                    join_set.spawn(async move {
                        let start = Instant::now();
                        let mut result = phase.execute(ctx).await?;
                        result.duration = start.elapsed();
                        Ok::<(String, PhaseResult<T>), WorkflowError>((name, result))
                    });
                }
            }

            // Collect results from this level
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok((name, phase_result))) => {
                        results.insert(name.clone(), phase_result);

                        // Update in-degrees for dependent phases
                        for (dependent, deps) in &graph {
                            if deps.contains(&name) {
                                if let Some(degree) = in_degree.get_mut(dependent) {
                                    *degree -= 1;
                                    if *degree == 0 {
                                        ready.push(dependent.clone());
                                    }
                                }
                            }
                        }
                    }
                    Ok(Err(e)) => return Err(e),
                    Err(e) => {
                        return Err(WorkflowError::InvalidSpecification(format!(
                            "Task join error: {}",
                            e
                        )))
                    }
                }
            }
        }

        // Check for cycles (remaining phases with in_degree > 0)
        let remaining: Vec<_> = in_degree
            .iter()
            .filter(|(_, &degree)| degree > 0)
            .map(|(name, _)| name.as_str())
            .collect();

        if !remaining.is_empty() {
            return Err(WorkflowError::InvalidSpecification(format!(
                "Circular dependencies detected in phases: {:?}",
                remaining
            )));
        }

        Ok(results)
    }
}

impl Default for PhaseExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateStore;
    use crate::WorkflowEngine;

    #[derive(Debug, Clone)]
    struct TestData {
        value: i32,
    }

    struct TestPhase {
        value: i32,
    }

    impl Phase<TestData> for TestPhase {
        fn metadata() -> super::super::core::PhaseMetadata {
            super::super::core::PhaseMetadata {
                name: "test_phase",
                description: "Test phase",
                version: "1.0.0",
                dependencies: &[],
                parallel: true,
            }
        }

        async fn execute(&self, _ctx: PhaseContext) -> WorkflowResult<PhaseResult<TestData>> {
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            Ok(PhaseResult::new(
                "test_phase",
                PhaseStatus::Pass,
                TestData { value: self.value },
            ))
        }
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = PhaseExecutor::new();
        assert!(executor.max_concurrent > 0);
        assert!(executor.parallel);
    }

    #[tokio::test]
    async fn test_execute_single_phase() {
        let executor = PhaseExecutor::new();
        let phase = TestPhase { value: 42 };

        let state_store = StateStore::new_in_memory();
        let engine = Arc::new(WorkflowEngine::new(state_store));
        let ctx = PhaseContext::new(engine, crate::parser::WorkflowSpecId::default());

        let result = executor.execute_phase(&phase, ctx).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.data.value, 42);
        assert!(result.duration.as_millis() >= 10);
    }
}
