//! YAWL Work Item Implementation
//!
//! Implements YWorkItem from YAWL Java with TRIZ Principle 32: Color Changes
//! - Type-level markers for execution phases
//! - Compile-time state machine enforcement
//!
//! Based on: org.yawlfoundation.yawl.engine.YWorkItem

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::types::WorkflowSpecId;
use chrono::{DateTime, Utc};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Work item status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkItemStatus {
    /// Enabled (ready to be allocated)
    Enabled,
    /// Fired (triggered but not started)
    Fired,
    /// Executing (in progress)
    Executing,
    /// Suspended (paused)
    Suspended,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
    /// Deadlocked
    Deadlocked,
}

/// Type-level phase markers (TRIZ Principle 32: Color Changes)
pub enum Enabled {}
pub enum Allocated {}
pub enum Executing {}
pub enum Completed {}
pub enum Cancelled {}

/// Work item ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkItemId {
    /// Task ID
    pub task_id: String,
    /// Unique ID
    pub unique_id: String,
}

impl WorkItemId {
    /// Create a new work item ID
    pub fn new(task_id: String) -> Self {
        Self {
            task_id: task_id.clone(),
            unique_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Get full ID string
    pub fn to_string(&self) -> String {
        format!("{}!{}", self.task_id, self.unique_id)
    }
}

/// YAWL Work Item with type-level phase markers (TRIZ Principle 32: Color Changes)
///
/// The generic parameter `Phase` enforces compile-time state transitions:
/// - `WorkItem<Enabled>` - Can only transition to Allocated
/// - `WorkItem<Allocated>` - Can only transition to Executing
/// - `WorkItem<Executing>` - Can only transition to Completed or Cancelled
pub struct WorkItem<Phase> {
    /// Work item ID
    pub id: WorkItemId,
    /// Specification ID
    pub spec_id: WorkflowSpecId,
    /// Task ID
    pub task_id: String,
    /// Status
    pub status: WorkItemStatus,
    /// Enablement time
    pub enablement_time: DateTime<Utc>,
    /// Firing time (if fired)
    pub firing_time: Option<DateTime<Utc>>,
    /// Start time (if started)
    pub start_time: Option<DateTime<Utc>>,
    /// Completion time (if completed)
    pub completion_time: Option<DateTime<Utc>>,
    /// Work item data
    pub data: serde_json::Value,
    /// Phase marker (TRIZ Principle 32: Color Changes)
    _phase: PhantomData<Phase>,
}

impl WorkItem<Enabled> {
    /// Create a new enabled work item
    pub fn new_enabled(spec_id: WorkflowSpecId, task_id: String, data: serde_json::Value) -> Self {
        Self {
            id: WorkItemId::new(task_id.clone()),
            spec_id,
            task_id,
            status: WorkItemStatus::Enabled,
            enablement_time: Utc::now(),
            firing_time: None,
            start_time: None,
            completion_time: None,
            data,
            _phase: PhantomData,
        }
    }

    /// Allocate this work item (transition to Allocated phase)
    ///
    /// TRIZ Principle 32: Type-level state transition enforced at compile time
    pub fn allocate(self, resource_id: String) -> WorkItem<Allocated> {
        WorkItem {
            id: self.id,
            spec_id: self.spec_id,
            task_id: self.task_id,
            status: WorkItemStatus::Fired,
            enablement_time: self.enablement_time,
            firing_time: Some(Utc::now()),
            start_time: None,
            completion_time: None,
            data: self.data,
            _phase: PhantomData,
        }
    }
}

impl WorkItem<Allocated> {
    /// Start execution (transition to Executing phase)
    ///
    /// TRIZ Principle 32: Type-level state transition enforced at compile time
    pub fn start(self) -> WorkItem<Executing> {
        WorkItem {
            id: self.id,
            spec_id: self.spec_id,
            task_id: self.task_id,
            status: WorkItemStatus::Executing,
            enablement_time: self.enablement_time,
            firing_time: self.firing_time,
            start_time: Some(Utc::now()),
            completion_time: None,
            data: self.data,
            _phase: PhantomData,
        }
    }
}

impl WorkItem<Executing> {
    /// Complete this work item (transition to Completed phase)
    ///
    /// TRIZ Principle 32: Type-level state transition enforced at compile time
    pub fn complete(self, result_data: serde_json::Value) -> WorkItem<Completed> {
        WorkItem {
            id: self.id,
            spec_id: self.spec_id,
            task_id: self.task_id,
            status: WorkItemStatus::Completed,
            enablement_time: self.enablement_time,
            firing_time: self.firing_time,
            start_time: self.start_time,
            completion_time: Some(Utc::now()),
            data: result_data,
            _phase: PhantomData,
        }
    }

    /// Cancel this work item (transition to Cancelled phase)
    ///
    /// TRIZ Principle 32: Type-level state transition enforced at compile time
    pub fn cancel(self) -> WorkItem<Cancelled> {
        WorkItem {
            id: self.id,
            spec_id: self.spec_id,
            task_id: self.task_id,
            status: WorkItemStatus::Cancelled,
            enablement_time: self.enablement_time,
            firing_time: self.firing_time,
            start_time: self.start_time,
            completion_time: Some(Utc::now()),
            data: self.data,
            _phase: PhantomData,
        }
    }

    /// Suspend execution
    pub async fn suspend(self) -> WorkflowResult<WorkItem<Executing>> {
        // Suspension doesn't change phase, just status
        Ok(WorkItem {
            id: self.id,
            spec_id: self.spec_id,
            task_id: self.task_id,
            status: WorkItemStatus::Suspended,
            enablement_time: self.enablement_time,
            firing_time: self.firing_time,
            start_time: self.start_time,
            completion_time: None,
            data: self.data,
            _phase: PhantomData,
        })
    }
}

impl<Phase> WorkItem<Phase> {
    /// Get work item ID
    pub fn get_id(&self) -> &WorkItemId {
        &self.id
    }

    /// Get specification ID
    pub fn get_spec_id(&self) -> &WorkflowSpecId {
        &self.spec_id
    }

    /// Get task ID
    pub fn get_task_id(&self) -> &str {
        &self.task_id
    }

    /// Get status
    pub fn get_status(&self) -> WorkItemStatus {
        self.status
    }

    /// Get enablement time
    pub fn get_enablement_time(&self) -> DateTime<Utc> {
        self.enablement_time
    }

    /// Get work item data
    pub fn get_data(&self) -> &serde_json::Value {
        &self.data
    }

    /// Update work item data
    pub fn update_data(&mut self, data: serde_json::Value) {
        self.data = data;
    }
}

/// Work item repository for managing work items
pub struct WorkItemRepository {
    /// Work items by ID
    items: Arc<RwLock<std::collections::HashMap<String, WorkItemStatus>>>,
}

impl WorkItemRepository {
    /// Create a new work item repository
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Store a work item (type-erased for storage)
    pub async fn store(&self, item_id: &WorkItemId, status: WorkItemStatus) {
        let mut items = self.items.write().await;
        items.insert(item_id.to_string(), status);
    }

    /// Get work item status
    pub async fn get_status(&self, item_id: &WorkItemId) -> Option<WorkItemStatus> {
        let items = self.items.read().await;
        items.get(&item_id.to_string()).copied()
    }

    /// Remove work item
    pub async fn remove(&self, item_id: &WorkItemId) {
        let mut items = self.items.write().await;
        items.remove(&item_id.to_string());
    }
}

impl Default for WorkItemRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_item_creation() {
        let spec_id = WorkflowSpecId::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let task_id = "task1".to_string();
        let data = serde_json::json!({"key": "value"});

        let item: WorkItem<Enabled> = WorkItem::new_enabled(spec_id.clone(), task_id.clone(), data);

        assert_eq!(item.get_spec_id(), &spec_id);
        assert_eq!(item.get_task_id(), &task_id);
        assert_eq!(item.get_status(), WorkItemStatus::Enabled);
    }

    #[test]
    fn test_work_item_state_transitions() {
        let spec_id = WorkflowSpecId::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let task_id = "task1".to_string();
        let data = serde_json::json!({"key": "value"});

        // Create enabled work item
        let enabled: WorkItem<Enabled> = WorkItem::new_enabled(spec_id, task_id, data);

        // Allocate (Enabled -> Allocated)
        let allocated = enabled.allocate("resource1".to_string());
        assert_eq!(allocated.get_status(), WorkItemStatus::Fired);

        // Start (Allocated -> Executing)
        let executing = allocated.start();
        assert_eq!(executing.get_status(), WorkItemStatus::Executing);

        // Complete (Executing -> Completed)
        let result_data = serde_json::json!({"result": "success"});
        let completed = executing.complete(result_data);
        assert_eq!(completed.get_status(), WorkItemStatus::Completed);
    }

    #[test]
    fn test_work_item_cancellation() {
        let spec_id = WorkflowSpecId::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let task_id = "task1".to_string();
        let data = serde_json::json!({"key": "value"});

        let enabled: WorkItem<Enabled> = WorkItem::new_enabled(spec_id, task_id, data);
        let allocated = enabled.allocate("resource1".to_string());
        let executing = allocated.start();

        // Cancel (Executing -> Cancelled)
        let cancelled = executing.cancel();
        assert_eq!(cancelled.get_status(), WorkItemStatus::Cancelled);
    }
}
