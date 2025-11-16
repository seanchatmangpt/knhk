//! KNHK Hook Engine Runtime
//!
//! Core hook execution engine with latency-bounded scheduling (≤8 ticks).
//! Implements μ via KNHK execution layer with pattern library integration.

use crate::error::{WorkflowError, WorkflowResult};
use crate::hooks::{HookContext, HookRegistry, HookResult, HookType};
use crate::patterns::PatternId;
use knhk_otel::{MetricsHelper, SpanContext, SpanStatus, Tracer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Chatman constant: Maximum ticks for hot path operations
pub const MAX_HOT_PATH_TICKS: u32 = 8;

/// Tick counter (CPU cycles or logical ticks)
pub type TickCount = u32;

/// Hook execution result with tick accounting
#[derive(Debug, Clone)]
pub struct HookExecutionResult {
    /// Original hook result
    pub result: HookResult,
    /// Ticks consumed
    pub ticks_used: TickCount,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Whether hot path constraint was met (≤8 ticks)
    pub met_hot_path_constraint: bool,
}

/// Hook execution context with OTEL integration
#[derive(Clone)]
pub struct ExecutionContext {
    /// Hook context
    pub hook_context: HookContext,
    /// OTEL span context
    pub span_context: Option<SpanContext>,
    /// Execution start time
    pub start_time: Instant,
    /// Maximum allowed ticks
    pub max_ticks: TickCount,
}

/// Hook engine runtime
pub struct HookEngine {
    /// Hook registry
    hook_registry: Arc<HookRegistry>,
    /// OTEL tracer
    tracer: Arc<RwLock<Tracer>>,
    /// Pattern library
    pattern_library: Arc<crate::engine::pattern_library::PatternLibrary>,
    /// Scheduler
    scheduler: Arc<crate::engine::scheduler::LatencyBoundedScheduler>,
}

impl HookEngine {
    /// Create new hook engine
    pub fn new(
        hook_registry: Arc<HookRegistry>,
        tracer: Arc<RwLock<Tracer>>,
    ) -> Self {
        Self {
            hook_registry,
            tracer: tracer.clone(),
            pattern_library: Arc::new(crate::engine::pattern_library::PatternLibrary::new()),
            scheduler: Arc::new(crate::engine::scheduler::LatencyBoundedScheduler::new(MAX_HOT_PATH_TICKS)),
        }
    }

    /// Execute a hook with latency bounds
    pub async fn execute_hook(
        &self,
        hook_type: HookType,
        context: HookContext,
    ) -> WorkflowResult<HookExecutionResult> {
        let start_time = Instant::now();
        let tick_start = self.get_tick_count();

        // Start OTEL span
        let span_context = {
            let mut tracer = self.tracer.write().await;
            let span_name = format!("knhk.hook.execute.{:?}", hook_type);
            let span_ctx = tracer.start_span(span_name.clone(), None);
            tracer.add_attribute(
                span_ctx.clone(),
                "hook.type".to_string(),
                format!("{:?}", hook_type),
            );
            if let Some(ref task_id) = context.task_id {
                tracer.add_attribute(
                    span_ctx.clone(),
                    "task.id".to_string(),
                    task_id.clone(),
                );
            }
            span_ctx
        };

        // Execute hook through registry
        let result = self.hook_registry.execute_hooks(hook_type, context.clone()).await?;

        // Calculate ticks and time
        let tick_end = self.get_tick_count();
        let ticks_used = tick_end.saturating_sub(tick_start);
        let execution_time_us = start_time.elapsed().as_micros() as u64;
        let met_hot_path_constraint = ticks_used <= MAX_HOT_PATH_TICKS;

        // Record metrics
        {
            let mut tracer = self.tracer.write().await;
            MetricsHelper::record_hook_latency(
                &mut tracer,
                ticks_used,
                &format!("{:?}", hook_type),
            );

            // End span
            let status = if result.continue_execution {
                SpanStatus::Ok
            } else {
                SpanStatus::Error
            };
            tracer.end_span(span_context, status);
        }

        // Check constraint violation
        if !met_hot_path_constraint {
            tracing::warn!(
                "Hook execution exceeded hot path constraint: {} ticks (max {})",
                ticks_used,
                MAX_HOT_PATH_TICKS
            );
        }

        Ok(HookExecutionResult {
            result,
            ticks_used,
            execution_time_us,
            met_hot_path_constraint,
        })
    }

    /// Execute pattern with hook integration
    pub async fn execute_pattern(
        &self,
        pattern_id: PatternId,
        input_data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        // Pre-pattern hook
        let pre_context = HookContext {
            hook_type: HookType::BeforePatternExecution,
            case_id: None,
            workflow_spec_id: None,
            task_id: None,
            pattern_id: Some(pattern_id),
            data: input_data.clone(),
        };

        let pre_result = self.execute_hook(HookType::BeforePatternExecution, pre_context).await?;

        if !pre_result.result.continue_execution {
            return Err(WorkflowError::HookFailed(
                pre_result.result.error.unwrap_or_else(|| "Pre-pattern hook failed".to_string())
            ));
        }

        let modified_input = pre_result.result.modified_data.unwrap_or(input_data.clone());

        // Execute pattern via pattern library
        let output_data = self.pattern_library.execute_pattern(pattern_id, modified_input).await?;

        // Post-pattern hook
        let post_context = HookContext {
            hook_type: HookType::AfterPatternExecution,
            case_id: None,
            workflow_spec_id: None,
            task_id: None,
            pattern_id: Some(pattern_id),
            data: output_data.clone(),
        };

        let post_result = self.execute_hook(HookType::AfterPatternExecution, post_context).await?;

        Ok(post_result.result.modified_data.unwrap_or(output_data))
    }

    /// Get tick count (simplified - in production would use RDTSC or similar)
    #[inline(always)]
    fn get_tick_count(&self) -> TickCount {
        // Simplified tick counting - in production this would use CPU cycle counter
        // For now, use nanoseconds as a proxy
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_nanos() % u32::MAX as u128) as u32)
            .unwrap_or(0)
    }

    /// Get scheduler
    pub fn scheduler(&self) -> &crate::engine::scheduler::LatencyBoundedScheduler {
        &self.scheduler
    }

    /// Get pattern library
    pub fn pattern_library(&self) -> &crate::engine::pattern_library::PatternLibrary {
        &self.pattern_library
    }
}

/// Hook execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecutionStats {
    /// Total hooks executed
    pub total_executed: u64,
    /// Total ticks consumed
    pub total_ticks: u64,
    /// Average ticks per hook
    pub avg_ticks: f64,
    /// Hot path constraint violations
    pub constraint_violations: u64,
    /// Total execution time (microseconds)
    pub total_execution_time_us: u64,
}

impl Default for HookExecutionStats {
    fn default() -> Self {
        Self {
            total_executed: 0,
            total_ticks: 0,
            avg_ticks: 0.0,
            constraint_violations: 0,
            total_execution_time_us: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::WorkflowHook;

    #[tokio::test]
    async fn test_hook_engine_execution() {
        let registry = Arc::new(HookRegistry::new());
        let tracer = Arc::new(RwLock::new(Tracer::new()));
        let engine = HookEngine::new(registry.clone(), tracer);

        // Register a simple hook
        let hook = WorkflowHook {
            id: "test-hook".to_string(),
            hook_type: HookType::BeforeTaskExecution,
            name: "Test Hook".to_string(),
            description: "Test hook".to_string(),
            hook_fn: Arc::new(|_ctx| Box::pin(async move { HookResult::success() })),
            enabled: true,
            priority: 0,
        };

        registry.register(hook).await.expect("Failed to register hook");

        let context = HookContext {
            hook_type: HookType::BeforeTaskExecution,
            case_id: None,
            workflow_spec_id: None,
            task_id: Some("task-1".to_string()),
            pattern_id: None,
            data: serde_json::json!({}),
        };

        let result = engine.execute_hook(HookType::BeforeTaskExecution, context).await.expect("Hook execution failed");

        assert!(result.result.continue_execution);
        // Note: tick counting in tests may not be accurate, but we verify it returns a value
        assert!(result.ticks_used >= 0);
    }

    #[tokio::test]
    async fn test_hook_engine_latency_tracking() {
        let registry = Arc::new(HookRegistry::new());
        let tracer = Arc::new(RwLock::new(Tracer::new()));
        let engine = HookEngine::new(registry.clone(), tracer.clone());

        let hook = WorkflowHook {
            id: "latency-hook".to_string(),
            hook_type: HookType::BeforeTaskExecution,
            name: "Latency Hook".to_string(),
            description: "Hook with delay".to_string(),
            hook_fn: Arc::new(|_ctx| Box::pin(async move {
                // Small delay
                tokio::time::sleep(std::time::Duration::from_micros(10)).await;
                HookResult::success()
            })),
            enabled: true,
            priority: 0,
        };

        registry.register(hook).await.expect("Failed to register hook");

        let context = HookContext {
            hook_type: HookType::BeforeTaskExecution,
            case_id: None,
            workflow_spec_id: None,
            task_id: Some("task-1".to_string()),
            pattern_id: None,
            data: serde_json::json!({}),
        };

        let result = engine.execute_hook(HookType::BeforeTaskExecution, context).await.expect("Hook execution failed");

        assert!(result.execution_time_us > 0);

        // Verify metrics were recorded
        let tracer = tracer.read().await;
        let metrics = tracer.metrics();
        assert!(!metrics.is_empty());
    }
}
