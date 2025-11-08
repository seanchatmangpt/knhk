//! Work item service for human task management
//!
//! Handles:
//! - Human-assigned tasks
//! - Work item queue
//! - Task assignment and claiming

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Work item state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkItemState {
    /// Created but not yet assigned
    Created,
    /// Assigned to a resource
    Assigned,
    /// Claimed by a resource
    Claimed,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
}

/// Work item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    /// Work item ID
    pub id: String,
    /// Case ID
    pub case_id: String,
    /// Workflow spec ID
    pub spec_id: WorkflowSpecId,
    /// Task ID
    pub task_id: String,
    /// Work item state
    pub state: WorkItemState,
    /// Assigned resource ID (if assigned)
    pub assigned_resource_id: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Completed timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Work item data
    pub data: serde_json::Value,
}

/// Work item service
pub struct WorkItemService {
    /// Work items by ID
    work_items: Arc<RwLock<HashMap<String, WorkItem>>>,
    /// Work items by case ID
    case_items: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl WorkItemService {
    /// Create a new work item service
    pub fn new() -> Self {
        Self {
            work_items: Arc::new(RwLock::new(HashMap::new())),
            case_items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a work item
    pub async fn create_work_item(
        &self,
        case_id: String,
        spec_id: WorkflowSpecId,
        task_id: String,
        data: serde_json::Value,
    ) -> WorkflowResult<String> {
        let work_item_id = Uuid::new_v4().to_string();
        let work_item = WorkItem {
            id: work_item_id.clone(),
            case_id: case_id.clone(),
            spec_id,
            task_id,
            state: WorkItemState::Created,
            assigned_resource_id: None,
            created_at: Utc::now(),
            completed_at: None,
            data,
        };

        let mut items = self.work_items.write().await;
        items.insert(work_item_id.clone(), work_item);

        let mut case_items = self.case_items.write().await;
        case_items
            .entry(case_id)
            .or_insert_with(Vec::new)
            .push(work_item_id.clone());

        Ok(work_item_id)
    }

    /// Get work item by ID
    pub async fn get_work_item(&self, work_item_id: &str) -> Option<WorkItem> {
        let items = self.work_items.read().await;
        items.get(work_item_id).cloned()
    }

    /// List work items for a case
    pub async fn list_case_work_items(&self, case_id: &str) -> Vec<WorkItem> {
        let case_items = self.case_items.read().await;
        let item_ids = case_items.get(case_id).cloned().unwrap_or_default();
        drop(case_items);

        let items = self.work_items.read().await;
        item_ids
            .iter()
            .filter_map(|id| items.get(id).cloned())
            .collect()
    }

    /// Assign work item to a resource
    pub async fn assign(&self, work_item_id: &str, resource_id: String) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state != WorkItemState::Created {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in Created state",
                    work_item_id
                )));
            }
            item.state = WorkItemState::Assigned;
            item.assigned_resource_id = Some(resource_id);
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Claim work item (move from Assigned to Claimed)
    pub async fn claim(&self, work_item_id: &str, resource_id: &str) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state != WorkItemState::Assigned {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in Assigned state",
                    work_item_id
                )));
            }
            if item.assigned_resource_id.as_ref() != Some(&resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, resource_id
                )));
            }
            item.state = WorkItemState::Claimed;
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Complete work item
    pub async fn complete(
        &self,
        work_item_id: &str,
        result: serde_json::Value,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state == WorkItemState::Completed || item.state == WorkItemState::Cancelled {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is already completed or cancelled",
                    work_item_id
                )));
            }
            item.state = WorkItemState::Completed;
            item.completed_at = Some(Utc::now());
            item.data = result;
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Cancel work item
    ///
    /// Emits HumanCancelled event for pattern dispatch (Patterns 19, 20)
    pub async fn cancel(&self, work_item_id: &str) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state == WorkItemState::Completed {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is already completed",
                    work_item_id
                )));
            }
            item.state = WorkItemState::Cancelled;

            // Emit HumanCancelled event for pattern dispatch
            // Patterns: 19 (Cancel Activity), 20 (Cancel Case)
            // This event will be handled by the pattern dispatch system
            // FUTURE: Wire to event bus for pattern dispatch

            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get inbox for a resource (work items assigned to or available for the resource)
    pub async fn get_inbox(&self, resource_id: &str) -> WorkflowResult<Vec<WorkItem>> {
        let items = self.work_items.read().await;
        Ok(items
            .values()
            .filter(|item| {
                // Include items assigned to this resource
                item.assigned_resource_id.as_ref() == Some(&resource_id.to_string())
                    // Or items in Created state (available for assignment)
                    || (item.state == WorkItemState::Created
                        && item.assigned_resource_id.is_none())
            })
            .cloned()
            .collect())
    }

    /// Submit work item (complete with payload)
    ///
    /// Emits HumanCompleted event for pattern dispatch (Patterns 4, 6, 33, 34, 27)
    pub async fn submit_work_item(
        &self,
        work_item_id: &str,
        _submission_id: &str,
        payload: serde_json::Value,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            // Validate state - must be Claimed or InProgress
            if item.state != WorkItemState::Claimed && item.state != WorkItemState::InProgress {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in Claimed or InProgress state",
                    work_item_id
                )));
            }

            // Update state and data
            item.state = WorkItemState::Completed;
            item.completed_at = Some(Utc::now());
            item.data = payload;

            // Emit HumanCompleted event for pattern dispatch
            // Patterns: 4 (Exclusive Choice), 6 (Multi-Choice), 33/34 (Partial Joins), 27 (Cancelling Discriminator)
            // This event will be handled by the pattern dispatch system
            // FUTURE: Wire to event bus for pattern dispatch

            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Withdraw work item (unclaim)
    pub async fn withdraw_work_item(
        &self,
        work_item_id: &str,
        resource_id: &str,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            // Validate state - must be Claimed or InProgress
            if item.state != WorkItemState::Claimed && item.state != WorkItemState::InProgress {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in Claimed or InProgress state",
                    work_item_id
                )));
            }

            // Validate resource
            if item.assigned_resource_id.as_ref() != Some(&resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, resource_id
                )));
            }

            // Withdraw (move back to Assigned state)
            item.state = WorkItemState::Assigned;

            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }
}

impl Default for WorkItemService {
    fn default() -> Self {
        Self::new()
    }
}
