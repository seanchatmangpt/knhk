//! Execution Engine - Core workflow execution with improved architecture
//!
//! Provides:
//! - Async pattern execution
//! - Execution pipeline
//! - Pattern composition
//! - Execution context management

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternId, PatternRegistry,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution engine for workflow patterns
pub struct ExecutionEngine {
    /// Pattern registry
    pattern_registry: Arc<PatternRegistry>,
    /// Execution pipeline
    pipeline: Arc<ExecutionPipeline>,
    /// Active executions
    active_executions: Arc<RwLock<std::collections::HashMap<String, ExecutionHandle>>>,
}

/// Execution handle for tracking active executions
#[derive(Debug, Clone)]
pub struct ExecutionHandle {
    /// Execution ID
    pub execution_id: String,
    /// Pattern ID being executed
    pub pattern_id: PatternId,
    /// Start time
    pub started_at: std::time::Instant,
    /// Status
    pub status: ExecutionStatus,
}

/// Execution status
#[derive(Debug, Clone)]
pub enum ExecutionStatus {
    /// Execution is pending
    Pending,
    /// Execution is running
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed(String),
    /// Execution was cancelled
    Cancelled,
}

impl ExecutionEngine {
    /// Create new execution engine
    pub fn new(pattern_registry: Arc<PatternRegistry>) -> Self {
        Self {
            pattern_registry,
            pipeline: Arc::new(ExecutionPipeline::new()),
            active_executions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Execute a pattern asynchronously
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        let execution_id = format!("{}:{}", context.case_id, pattern_id.0);

        // Create execution handle
        let handle = ExecutionHandle {
            execution_id: execution_id.clone(),
            pattern_id,
            started_at: std::time::Instant::now(),
            status: ExecutionStatus::Pending,
        };

        // Register execution
        {
            let mut executions = self.active_executions.write().await;
            executions.insert(execution_id.clone(), handle);
        }

        // Execute through pipeline
        let result = self
            .pipeline
            .execute(&self.pattern_registry, pattern_id, context)
            .await;

        // Update execution status
        {
            let mut executions = self.active_executions.write().await;
            if let Some(handle) = executions.get_mut(&execution_id) {
                handle.status = match &result {
                    Ok(r) if r.success => ExecutionStatus::Completed,
                    Ok(_) => {
                        ExecutionStatus::Failed("Pattern execution returned failure".to_string())
                    }
                    Err(e) => ExecutionStatus::Failed(e.to_string()),
                };
            }
        }

        result
    }

    /// Cancel an execution
    pub async fn cancel_execution(&self, execution_id: &str) -> WorkflowResult<()> {
        let mut executions = self.active_executions.write().await;
        if let Some(handle) = executions.get_mut(execution_id) {
            handle.status = ExecutionStatus::Cancelled;
            Ok(())
        } else {
            Err(WorkflowError::CaseNotFound(execution_id.to_string()))
        }
    }

    /// Get execution status
    pub async fn get_execution_status(&self, execution_id: &str) -> Option<ExecutionHandle> {
        let executions = self.active_executions.read().await;
        executions.get(execution_id).cloned()
    }
}

/// Execution pipeline for pattern composition and optimization
pub struct ExecutionPipeline {
    /// Pipeline stages
    stages: Vec<Box<dyn PipelineStage>>,
}

impl ExecutionPipeline {
    /// Create new execution pipeline
    pub fn new() -> Self {
        Self {
            stages: vec![
                Box::new(ValidationStage),
                Box::new(OptimizationStage),
                Box::new(ExecutionStage),
            ],
        }
    }

    /// Execute pattern through pipeline
    pub async fn execute(
        &self,
        registry: &PatternRegistry,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        let mut ctx = context;

        // Run through pipeline stages
        for stage in &self.stages {
            ctx = stage.process(registry, pattern_id, ctx).await?;
        }

        // Final execution
        let executor = registry
            .get(&pattern_id)
            .ok_or_else(|| WorkflowError::PatternNotFound(pattern_id.0))?;

        Ok(executor.execute(&ctx))
    }
}

impl Default for ExecutionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline stage trait
pub trait PipelineStage: Send + Sync {
    /// Process execution context through this stage
    fn process(
        &self,
        registry: &PatternRegistry,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WorkflowResult<PatternExecutionContext>> + Send + '_>,
    >;
}

/// Validation stage - validates execution context
struct ValidationStage;

impl PipelineStage for ValidationStage {
    fn process(
        &self,
        _registry: &PatternRegistry,
        pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WorkflowResult<PatternExecutionContext>> + Send + '_>,
    > {
        Box::pin(async move {
            // Validate pattern ID
            if !(1..=43).contains(&pattern_id.0) {
                return Err(WorkflowError::PatternNotFound(pattern_id.0));
            }

            // Validate context
            if context.variables.is_empty() && pattern_id.0 > 1 {
                // Some patterns require variables
                // FUTURE: Add pattern-specific validation
            }

            Ok(context)
        })
    }
}

/// Optimization stage - optimizes execution context
struct OptimizationStage;

impl PipelineStage for OptimizationStage {
    async fn process(
        &self,
        _registry: &PatternRegistry,
        _pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionContext> {
        // FUTURE: Add optimizations like:
        // - Variable caching
        // - Pattern composition
        // - Execution plan optimization
        Ok(context)
    }
}

/// Execution stage - prepares for execution
struct ExecutionStage;

impl PipelineStage for ExecutionStage {
    fn process(
        &self,
        _registry: &PatternRegistry,
        _pattern_id: PatternId,
        context: PatternExecutionContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WorkflowResult<PatternExecutionContext>> + Send + '_>,
    > {
        Box::pin(async move {
            // FUTURE: Add execution preparation like:
            // - Resource pre-allocation
            // - Dependency resolution
            // - Execution plan creation
            Ok(context)
        })
    }
}
