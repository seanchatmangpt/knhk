//! WASM-compatible workflow runtime
//!
//! This module provides a workflow runtime optimized for WebAssembly execution.

use crate::{WasmEngineConfig, WasmError, WasmResult, EngineStats};
use crate::state::WasmStateStore;
use crate::parser::WorkflowParser;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

/// WASM-compatible workflow runtime
pub struct WasmWorkflowRuntime {
    config: WasmEngineConfig,
    state_store: WasmStateStore,
    parser: WorkflowParser,
    stats: RuntimeStats,
}

#[derive(Debug, Default)]
struct RuntimeStats {
    total_executed: u64,
    failed_workflows: u64,
    total_execution_time_ms: u64,
}

impl WasmWorkflowRuntime {
    /// Create a new WASM workflow runtime
    pub fn new(config: WasmEngineConfig) -> WasmResult<Self> {
        Ok(Self {
            config,
            state_store: WasmStateStore::new(),
            parser: WorkflowParser::new(),
            stats: RuntimeStats::default(),
        })
    }

    /// Execute a workflow from Turtle/RDF definition
    pub async fn execute(
        &mut self,
        workflow_def: &str,
        input: JsonValue,
    ) -> WasmResult<JsonValue> {
        let start = instant::Instant::now();

        // Parse workflow definition
        let spec = self.parser.parse_turtle(workflow_def)?;

        // Create a new case ID
        let case_id = Uuid::new_v4().to_string();

        // Store initial state
        self.state_store.create_case(&case_id, &spec, input.clone())?;

        // Execute workflow (simplified for WASM)
        let result = self.execute_workflow_internal(&case_id, &spec, input).await?;

        // Update statistics
        let elapsed = start.elapsed();
        self.stats.total_executed += 1;
        self.stats.total_execution_time_ms += elapsed.as_millis() as u64;

        Ok(result)
    }

    /// Execute a workflow from JSON specification
    pub async fn execute_json(
        &mut self,
        workflow_json: &str,
        input: JsonValue,
    ) -> WasmResult<JsonValue> {
        let spec = self.parser.parse_json(workflow_json)?;
        let case_id = Uuid::new_v4().to_string();

        self.state_store.create_case(&case_id, &spec, input.clone())?;
        self.execute_workflow_internal(&case_id, &spec, input).await
    }

    /// Validate a workflow definition
    pub fn validate(&self, workflow_def: &str) -> WasmResult<()> {
        let _spec = self.parser.parse_turtle(workflow_def)?;
        Ok(())
    }

    /// Get engine statistics
    pub fn get_stats(&self) -> EngineStats {
        let avg_time = if self.stats.total_executed > 0 {
            self.stats.total_execution_time_ms as f64 / self.stats.total_executed as f64
        } else {
            0.0
        };

        EngineStats {
            total_executed: self.stats.total_executed,
            running_workflows: self.state_store.running_count(),
            failed_workflows: self.stats.failed_workflows,
            avg_execution_time_ms: avg_time,
            memory_usage_bytes: self.state_store.memory_usage(),
        }
    }

    /// Reset the runtime state
    pub fn reset(&mut self) -> WasmResult<()> {
        self.state_store.clear();
        self.stats = RuntimeStats::default();
        Ok(())
    }

    /// Internal workflow execution logic
    async fn execute_workflow_internal(
        &mut self,
        case_id: &str,
        spec: &WorkflowSpec,
        input: JsonValue,
    ) -> WasmResult<JsonValue> {
        use tracing::{info, error};

        info!(case_id = %case_id, workflow = %spec.id, "Starting workflow execution");

        // Check timeout
        let timeout = std::time::Duration::from_millis(self.config.timeout_ms as u64);
        let start = instant::Instant::now();

        // Execute based on workflow pattern
        let result = match spec.pattern.as_str() {
            "Sequence" => self.execute_sequence(case_id, spec, input).await,
            "Parallel" => self.execute_parallel(case_id, spec, input).await,
            "Choice" => self.execute_choice(case_id, spec, input).await,
            "Loop" => self.execute_loop(case_id, spec, input).await,
            _ => {
                error!(pattern = %spec.pattern, "Unsupported workflow pattern");
                Err(WasmError::UnsupportedPattern(spec.pattern.clone()))
            }
        };

        // Check if we exceeded timeout
        if start.elapsed() > timeout {
            error!(case_id = %case_id, "Workflow execution timeout");
            self.stats.failed_workflows += 1;
            return Err(WasmError::ExecutionTimeout(timeout.as_millis() as u32));
        }

        match result {
            Ok(output) => {
                info!(case_id = %case_id, "Workflow completed successfully");
                self.state_store.mark_completed(case_id)?;
                Ok(output)
            }
            Err(e) => {
                error!(case_id = %case_id, error = %e, "Workflow execution failed");
                self.stats.failed_workflows += 1;
                self.state_store.mark_failed(case_id, &e.to_string())?;
                Err(e)
            }
        }
    }

    /// Execute a sequence workflow pattern
    async fn execute_sequence(
        &mut self,
        case_id: &str,
        spec: &WorkflowSpec,
        mut data: JsonValue,
    ) -> WasmResult<JsonValue> {
        for task in &spec.tasks {
            data = self.execute_task(case_id, task, data).await?;
        }
        Ok(data)
    }

    /// Execute a parallel workflow pattern
    async fn execute_parallel(
        &mut self,
        case_id: &str,
        spec: &WorkflowSpec,
        data: JsonValue,
    ) -> WasmResult<JsonValue> {
        // In WASM, we simulate parallelism by executing tasks concurrently
        // using wasm-bindgen-futures
        let mut results = Vec::new();

        for task in &spec.tasks {
            let result = self.execute_task(case_id, task, data.clone()).await?;
            results.push(result);
        }

        // Merge results
        Ok(JsonValue::Array(results))
    }

    /// Execute a choice workflow pattern
    async fn execute_choice(
        &mut self,
        case_id: &str,
        spec: &WorkflowSpec,
        data: JsonValue,
    ) -> WasmResult<JsonValue> {
        // Evaluate condition and pick the right branch
        for task in &spec.tasks {
            if self.evaluate_condition(task, &data)? {
                return self.execute_task(case_id, task, data).await;
            }
        }

        Err(WasmError::NoMatchingBranch)
    }

    /// Execute a loop workflow pattern
    async fn execute_loop(
        &mut self,
        case_id: &str,
        spec: &WorkflowSpec,
        mut data: JsonValue,
    ) -> WasmResult<JsonValue> {
        let max_iterations = 1000; // Safety limit
        let mut iteration = 0;

        while iteration < max_iterations {
            // Check loop condition
            if !self.evaluate_loop_condition(spec, &data)? {
                break;
            }

            // Execute loop body
            for task in &spec.tasks {
                data = self.execute_task(case_id, task, data).await?;
            }

            iteration += 1;
        }

        if iteration >= max_iterations {
            return Err(WasmError::MaxIterationsExceeded);
        }

        Ok(data)
    }

    /// Execute a single task
    async fn execute_task(
        &mut self,
        case_id: &str,
        task: &Task,
        data: JsonValue,
    ) -> WasmResult<JsonValue> {
        use tracing::info;

        info!(case_id = %case_id, task = %task.id, "Executing task");

        // Mark task as running
        self.state_store.start_task(case_id, &task.id)?;

        // Execute task logic (simplified for WASM)
        // In a real implementation, this would call host functions or execute task code
        let result = match task.task_type.as_str() {
            "transform" => self.execute_transform(task, data),
            "validate" => self.execute_validate(task, data),
            "compute" => self.execute_compute(task, data),
            _ => Ok(data), // Pass-through for unknown types
        };

        // Mark task as completed
        self.state_store.complete_task(case_id, &task.id)?;

        result
    }

    fn execute_transform(&self, task: &Task, mut data: JsonValue) -> WasmResult<JsonValue> {
        // Simple transformation example
        if let Some(transform) = &task.config {
            if let Some(obj) = data.as_object_mut() {
                for (key, value) in transform.as_object().unwrap_or(&serde_json::Map::new()) {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }
        Ok(data)
    }

    fn execute_validate(&self, task: &Task, data: JsonValue) -> WasmResult<JsonValue> {
        // Simple validation example
        if let Some(schema) = &task.config {
            // In a real implementation, this would use JSON Schema validation
            // For now, just check if required fields exist
            if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
                for field in required {
                    let field_name = field.as_str().ok_or(WasmError::InvalidConfig)?;
                    if data.get(field_name).is_none() {
                        return Err(WasmError::ValidationFailed(
                            format!("Missing required field: {}", field_name)
                        ));
                    }
                }
            }
        }
        Ok(data)
    }

    fn execute_compute(&self, _task: &Task, data: JsonValue) -> WasmResult<JsonValue> {
        // Simple computation example
        // In a real implementation, this would evaluate expressions
        Ok(data)
    }

    fn evaluate_condition(&self, task: &Task, data: &JsonValue) -> WasmResult<bool> {
        // Simple condition evaluation
        if let Some(condition) = &task.condition {
            // Evaluate simple field existence checks
            if let Some(field) = condition.as_str() {
                return Ok(data.get(field).is_some());
            }
        }
        Ok(true)
    }

    fn evaluate_loop_condition(&self, spec: &WorkflowSpec, data: &JsonValue) -> WasmResult<bool> {
        // Simple loop condition evaluation
        if let Some(condition) = &spec.loop_condition {
            if let Some(field) = condition.as_str() {
                return Ok(data.get(field).and_then(|v| v.as_bool()).unwrap_or(false));
            }
        }
        Ok(false)
    }
}

/// Simplified workflow specification for WASM
#[derive(Debug, Clone)]
pub struct WorkflowSpec {
    pub id: String,
    pub pattern: String,
    pub tasks: Vec<Task>,
    pub loop_condition: Option<JsonValue>,
}

/// Task definition
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub task_type: String,
    pub config: Option<JsonValue>,
    pub condition: Option<JsonValue>,
}

use instant;
