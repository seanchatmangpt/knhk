//! In-memory state store for WASM workflow execution

use crate::{WasmError, WasmResult};
use crate::runtime::WorkflowSpec;
use serde_json::Value as JsonValue;
use std::collections::HashMap as FastHashMap;

/// In-memory state store optimized for WASM
pub struct WasmStateStore {
    cases: FastHashMap<String, CaseState>,
}

#[derive(Debug, Clone)]
struct CaseState {
    case_id: String,
    workflow_id: String,
    status: CaseStatus,
    input: JsonValue,
    output: Option<JsonValue>,
    tasks: FastHashMap<String, TaskState>,
    created_at: u64,
    updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseStatus {
    Created,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
struct TaskState {
    task_id: String,
    status: TaskStatus,
    started_at: Option<u64>,
    completed_at: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl WasmStateStore {
    /// Create a new in-memory state store
    pub fn new() -> Self {
        Self {
            cases: FastHashMap::new(),
        }
    }

    /// Create a new case
    pub fn create_case(
        &mut self,
        case_id: &str,
        spec: &WorkflowSpec,
        input: JsonValue,
    ) -> WasmResult<()> {
        let now = Self::now_ms();

        let case = CaseState {
            case_id: case_id.to_string(),
            workflow_id: spec.id.clone(),
            status: CaseStatus::Created,
            input,
            output: None,
            tasks: FastHashMap::new(),
            created_at: now,
            updated_at: now,
        };

        self.cases.insert(case_id.to_string(), case);
        Ok(())
    }

    /// Start a task
    pub fn start_task(&mut self, case_id: &str, task_id: &str) -> WasmResult<()> {
        let case = self.cases.get_mut(case_id)
            .ok_or_else(|| WasmError::StateNotFound(case_id.to_string()))?;

        case.status = CaseStatus::Running;
        case.updated_at = Self::now_ms();

        case.tasks.insert(
            task_id.to_string(),
            TaskState {
                task_id: task_id.to_string(),
                status: TaskStatus::Running,
                started_at: Some(Self::now_ms()),
                completed_at: None,
            },
        );

        Ok(())
    }

    /// Complete a task
    pub fn complete_task(&mut self, case_id: &str, task_id: &str) -> WasmResult<()> {
        let case = self.cases.get_mut(case_id)
            .ok_or_else(|| WasmError::StateNotFound(case_id.to_string()))?;

        case.updated_at = Self::now_ms();

        if let Some(task) = case.tasks.get_mut(task_id) {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(Self::now_ms());
        }

        Ok(())
    }

    /// Mark case as completed
    pub fn mark_completed(&mut self, case_id: &str) -> WasmResult<()> {
        let case = self.cases.get_mut(case_id)
            .ok_or_else(|| WasmError::StateNotFound(case_id.to_string()))?;

        case.status = CaseStatus::Completed;
        case.updated_at = Self::now_ms();

        Ok(())
    }

    /// Mark case as failed
    pub fn mark_failed(&mut self, case_id: &str, _error: &str) -> WasmResult<()> {
        let case = self.cases.get_mut(case_id)
            .ok_or_else(|| WasmError::StateNotFound(case_id.to_string()))?;

        case.status = CaseStatus::Failed;
        case.updated_at = Self::now_ms();

        Ok(())
    }

    /// Get the number of running workflows
    pub fn running_count(&self) -> usize {
        self.cases.values()
            .filter(|c| c.status == CaseStatus::Running)
            .count()
    }

    /// Get approximate memory usage
    pub fn memory_usage(&self) -> usize {
        // Rough estimation
        self.cases.len() * std::mem::size_of::<CaseState>()
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.cases.clear();
    }

    /// Get current timestamp in milliseconds
    fn now_ms() -> u64 {
        instant::Instant::now().elapsed().as_millis() as u64
    }
}

use instant;
