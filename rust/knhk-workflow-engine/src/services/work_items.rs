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

    /// Start work item (move from Claimed to InProgress)
    pub async fn start_work_item(
        &self,
        work_item_id: &str,
        resource_id: &str,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state != WorkItemState::Claimed {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in Claimed state",
                    work_item_id
                )));
            }

            if item.assigned_resource_id.as_ref() != Some(&resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, resource_id
                )));
            }

            item.state = WorkItemState::InProgress;
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Suspend work item (pause execution)
    pub async fn suspend_work_item(
        &self,
        work_item_id: &str,
        resource_id: &str,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state != WorkItemState::InProgress {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in InProgress state",
                    work_item_id
                )));
            }

            if item.assigned_resource_id.as_ref() != Some(&resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, resource_id
                )));
            }

            // Note: Suspend is not a separate state in WorkItemState enum
            // In production, would add WorkItemState::Suspended
            // For now, keep in InProgress but mark in data
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.insert("suspended".to_string(), serde_json::Value::Bool(true));
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Resume work item (resume from suspended)
    pub async fn resume_work_item(
        &self,
        work_item_id: &str,
        resource_id: &str,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.state != WorkItemState::InProgress {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not in InProgress state",
                    work_item_id
                )));
            }

            if item.assigned_resource_id.as_ref() != Some(&resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, resource_id
                )));
            }

            // Remove suspended flag
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.remove("suspended");
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Reassign work item to different resource
    pub async fn reassign_work_item(
        &self,
        work_item_id: &str,
        from_resource_id: &str,
        to_resource_id: String,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.assigned_resource_id.as_ref() != Some(&from_resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, from_resource_id
                )));
            }

            if item.state == WorkItemState::Completed || item.state == WorkItemState::Cancelled {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is in final state {:?}",
                    work_item_id, item.state
                )));
            }

            item.assigned_resource_id = Some(to_resource_id);
            // Reset to Assigned state if was Claimed or InProgress
            if item.state == WorkItemState::Claimed || item.state == WorkItemState::InProgress {
                item.state = WorkItemState::Assigned;
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work items by state
    pub async fn get_work_items_by_state(&self, state: WorkItemState) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| item.state == state)
            .cloned()
            .collect()
    }

    /// Get work items by task ID
    pub async fn get_work_items_by_task(&self, task_id: &str) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| item.task_id == task_id)
            .cloned()
            .collect()
    }

    /// Get work items by workflow spec ID
    pub async fn get_work_items_by_spec(&self, spec_id: &WorkflowSpecId) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| item.spec_id == *spec_id)
            .cloned()
            .collect()
    }

    /// Update work item data
    pub async fn update_work_item_data(
        &self,
        work_item_id: &str,
        resource_id: &str,
        data: serde_json::Value,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.assigned_resource_id.as_ref() != Some(&resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, resource_id
                )));
            }

            if item.state == WorkItemState::Completed || item.state == WorkItemState::Cancelled {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is in final state {:?}",
                    work_item_id, item.state
                )));
            }

            item.data = data;
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work item statistics
    pub async fn get_statistics(&self) -> WorkflowResult<serde_json::Value> {
        let items = self.work_items.read().await;
        let mut stats = std::collections::HashMap::new();
        let mut state_counts = std::collections::HashMap::new();

        for item in items.values() {
            *state_counts.entry(format!("{:?}", item.state)).or_insert(0) += 1;
        }

        stats.insert(
            "total".to_string(),
            serde_json::Value::Number(items.len().into()),
        );
        stats.insert(
            "by_state".to_string(),
            serde_json::to_value(state_counts)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        );

        Ok(
            serde_json::to_value(stats)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        )
    }

    /// Bulk assign work items
    pub async fn bulk_assign(
        &self,
        work_item_ids: Vec<String>,
        resource_id: String,
    ) -> WorkflowResult<Vec<String>> {
        let mut assigned = Vec::new();
        let mut failed = Vec::new();

        for work_item_id in work_item_ids {
            match self.assign(&work_item_id, resource_id.clone()).await {
                Ok(()) => assigned.push(work_item_id),
                Err(e) => {
                    failed.push(format!("{}: {}", work_item_id, e));
                }
            }
        }

        if !failed.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Failed to assign some work items: {:?}",
                failed
            )));
        }

        Ok(assigned)
    }

    /// Bulk claim work items
    pub async fn bulk_claim(
        &self,
        work_item_ids: Vec<String>,
        resource_id: &str,
    ) -> WorkflowResult<Vec<String>> {
        let mut claimed = Vec::new();
        let mut failed = Vec::new();

        for work_item_id in work_item_ids {
            match self.claim(&work_item_id, resource_id).await {
                Ok(()) => claimed.push(work_item_id),
                Err(e) => {
                    failed.push(format!("{}: {}", work_item_id, e));
                }
            }
        }

        if !failed.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Failed to claim some work items: {:?}",
                failed
            )));
        }

        Ok(claimed)
    }

    /// Get work items with filters
    pub async fn get_work_items_filtered(
        &self,
        case_id: Option<String>,
        task_id: Option<String>,
        state: Option<WorkItemState>,
        resource_id: Option<String>,
    ) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| {
                if let Some(ref cid) = case_id {
                    if item.case_id != *cid {
                        return false;
                    }
                }
                if let Some(ref tid) = task_id {
                    if item.task_id != *tid {
                        return false;
                    }
                }
                if let Some(ref s) = state {
                    if item.state != *s {
                        return false;
                    }
                }
                if let Some(ref rid) = resource_id {
                    if item.assigned_resource_id.as_ref() != Some(rid) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Delegate work item to another resource
    pub async fn delegate_work_item(
        &self,
        work_item_id: &str,
        from_resource_id: &str,
        to_resource_id: String,
    ) -> WorkflowResult<()> {
        // Delegation is similar to reassignment but preserves original assignee
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if item.assigned_resource_id.as_ref() != Some(&from_resource_id.to_string()) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is not assigned to resource {}",
                    work_item_id, from_resource_id
                )));
            }

            if item.state == WorkItemState::Completed || item.state == WorkItemState::Cancelled {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} is in final state {:?}",
                    work_item_id, item.state
                )));
            }

            // Store original assignee in data
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.insert(
                    "original_assignee".to_string(),
                    serde_json::Value::String(from_resource_id.to_string()),
                );
            }

            item.assigned_resource_id = Some(to_resource_id);
            if item.state == WorkItemState::Claimed || item.state == WorkItemState::InProgress {
                item.state = WorkItemState::Assigned;
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work item history
    pub async fn get_work_item_history(
        &self,
        work_item_id: &str,
    ) -> WorkflowResult<Vec<serde_json::Value>> {
        // In production, would maintain a separate history log
        // For now, return empty history
        let _items = self.work_items.read().await;
        if _items.contains_key(work_item_id) {
            Ok(Vec::new())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Set work item priority
    pub async fn set_priority(&self, work_item_id: &str, priority: u8) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.insert(
                    "priority".to_string(),
                    serde_json::Value::Number(priority.into()),
                );
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work item priority
    pub async fn get_priority(&self, work_item_id: &str) -> WorkflowResult<u8> {
        let items = self.work_items.read().await;
        if let Some(item) = items.get(work_item_id) {
            if let Some(priority) = item.data.get("priority") {
                if let Some(priority_num) = priority.as_u64() {
                    return Ok(priority_num.min(255) as u8);
                }
            }
            Ok(100) // Default priority
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Set work item deadline
    pub async fn set_deadline(
        &self,
        work_item_id: &str,
        deadline: DateTime<Utc>,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.insert(
                    "deadline".to_string(),
                    serde_json::Value::String(deadline.to_rfc3339()),
                );
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work item deadline
    pub async fn get_deadline(&self, work_item_id: &str) -> WorkflowResult<Option<DateTime<Utc>>> {
        let items = self.work_items.read().await;
        if let Some(item) = items.get(work_item_id) {
            if let Some(deadline_str) = item.data.get("deadline") {
                if let Some(deadline_str) = deadline_str.as_str() {
                    if let Ok(deadline) = DateTime::parse_from_rfc3339(deadline_str) {
                        return Ok(Some(deadline.with_timezone(&Utc)));
                    }
                }
            }
            Ok(None)
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get overdue work items
    pub async fn get_overdue_work_items(&self) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        let now = Utc::now();
        items
            .values()
            .filter(|item| {
                if let Some(deadline_str) = item.data.get("deadline") {
                    if let Some(deadline_str) = deadline_str.as_str() {
                        if let Ok(deadline) = DateTime::parse_from_rfc3339(deadline_str) {
                            let deadline_utc = deadline.with_timezone(&Utc);
                            return deadline_utc < now
                                && item.state != WorkItemState::Completed
                                && item.state != WorkItemState::Cancelled;
                        }
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Get work items due soon
    pub async fn get_due_soon_work_items(&self, hours: u32) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        let now = Utc::now();
        let threshold = now + chrono::Duration::hours(hours as i64);
        items
            .values()
            .filter(|item| {
                if let Some(deadline_str) = item.data.get("deadline") {
                    if let Some(deadline_str) = deadline_str.as_str() {
                        if let Ok(deadline) = DateTime::parse_from_rfc3339(deadline_str) {
                            let deadline_utc = deadline.with_timezone(&Utc);
                            return deadline_utc <= threshold
                                && deadline_utc > now
                                && item.state != WorkItemState::Completed
                                && item.state != WorkItemState::Cancelled;
                        }
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Get work items by date range
    pub async fn get_work_items_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| item.created_at >= start_date && item.created_at <= end_date)
            .cloned()
            .collect()
    }

    /// Get work items by completion date range
    pub async fn get_work_items_by_completion_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| {
                if let Some(completed_at) = item.completed_at {
                    completed_at >= start_date && completed_at <= end_date
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Get work items count by state
    pub async fn get_count_by_state(&self, state: WorkItemState) -> usize {
        let items = self.work_items.read().await;
        items.values().filter(|item| item.state == state).count()
    }

    /// Get total work items count
    pub async fn get_total_count(&self) -> usize {
        let items = self.work_items.read().await;
        items.len()
    }

    /// Get average completion time
    pub async fn get_average_completion_time(&self) -> WorkflowResult<Option<chrono::Duration>> {
        let items = self.work_items.read().await;
        let completed_items: Vec<_> = items
            .values()
            .filter(|item| item.completed_at.is_some())
            .collect();

        if completed_items.is_empty() {
            return Ok(None);
        }

        let total_duration: i64 = completed_items
            .iter()
            .map(|item| {
                item.completed_at
                    .map(|completed_at| (completed_at - item.created_at).num_seconds())
                    .unwrap_or(0)
            })
            .sum();

        let avg_seconds = total_duration / completed_items.len() as i64;
        Ok(Some(chrono::Duration::seconds(avg_seconds)))
    }

    /// Get work items by resource workload
    pub async fn get_work_items_by_resource_workload(
        &self,
        resource_id: &str,
        min_workload: u32,
        max_workload: u32,
    ) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| {
                if item.assigned_resource_id.as_ref() == Some(&resource_id.to_string()) {
                    // Note: Would need to query ResourceAllocator for actual workload
                    // For now, use queue_length from work item data if available
                    if let Some(queue_len) = item.data.get("queue_length") {
                        if let Some(queue_len) = queue_len.as_u64() {
                            let queue_len = queue_len as u32;
                            return queue_len >= min_workload && queue_len <= max_workload;
                        }
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Get work items requiring attention (overdue or high priority)
    pub async fn get_work_items_requiring_attention(&self) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        let now = Utc::now();
        items
            .values()
            .filter(|item| {
                // Overdue
                if let Some(deadline_str) = item.data.get("deadline") {
                    if let Some(deadline_str) = deadline_str.as_str() {
                        if let Ok(deadline) = DateTime::parse_from_rfc3339(deadline_str) {
                            let deadline_utc = deadline.with_timezone(&Utc);
                            if deadline_utc < now
                                && item.state != WorkItemState::Completed
                                && item.state != WorkItemState::Cancelled
                            {
                                return true;
                            }
                        }
                    }
                }
                // High priority
                if let Some(priority) = item.data.get("priority") {
                    if let Some(priority) = priority.as_u64() {
                        if priority >= 200 {
                            return true;
                        }
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Get work items by tags
    pub async fn get_work_items_by_tags(&self, tags: Vec<String>) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| {
                if let Some(item_tags) = item.data.get("tags") {
                    if let Some(item_tags) = item_tags.as_array() {
                        for tag in &tags {
                            if item_tags.iter().any(|t| t.as_str() == Some(tag)) {
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Add tags to work item
    pub async fn add_tags(&self, work_item_id: &str, tags: Vec<String>) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if let Some(data_obj) = item.data.as_object_mut() {
                let mut existing_tags = if let Some(tags_val) = data_obj.get("tags") {
                    if let Some(tags_arr) = tags_val.as_array() {
                        tags_arr
                            .iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                for tag in tags {
                    if !existing_tags.contains(&tag) {
                        existing_tags.push(tag);
                    }
                }

                data_obj.insert(
                    "tags".to_string(),
                    serde_json::to_value(existing_tags)
                        .unwrap_or(serde_json::Value::Array(Vec::new())),
                );
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Remove tags from work item
    pub async fn remove_tags(&self, work_item_id: &str, tags: Vec<String>) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if let Some(data_obj) = item.data.as_object_mut() {
                if let Some(tags_val) = data_obj.get_mut("tags") {
                    if let Some(tags_arr) = tags_val.as_array_mut() {
                        tags_arr.retain(|t| {
                            if let Some(tag_str) = t.as_str() {
                                !tags.contains(&tag_str.to_string())
                            } else {
                                true
                            }
                        });
                    }
                }
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work items by custom query
    pub async fn query_work_items(
        &self,
        query: serde_json::Value,
    ) -> WorkflowResult<Vec<WorkItem>> {
        // Simple query implementation - in production would use a query engine
        let items = self.work_items.read().await;
        let mut results = Vec::new();

        for item in items.values() {
            // Match query against work item data
            if let Some(query_obj) = query.as_object() {
                let mut matches = true;
                for (key, value) in query_obj {
                    if let Some(item_value) = item.data.get(key) {
                        if item_value != value {
                            matches = false;
                            break;
                        }
                    } else {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    results.push(item.clone());
                }
            }
        }

        Ok(results)
    }

    /// Get work items with pagination
    pub async fn get_work_items_paginated(
        &self,
        page: usize,
        page_size: usize,
    ) -> WorkflowResult<(Vec<WorkItem>, usize)> {
        let items = self.work_items.read().await;
        let total = items.len();
        let start = page * page_size;
        let end = (start + page_size).min(total);

        let work_items: Vec<WorkItem> = items
            .values()
            .skip(start)
            .take(end - start)
            .cloned()
            .collect();

        Ok((work_items, total))
    }

    /// Get work items sorted by field
    pub async fn get_work_items_sorted(&self, sort_field: &str, ascending: bool) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        let mut work_items: Vec<WorkItem> = items.values().cloned().collect();

        work_items.sort_by(|a, b| {
            let ordering = match sort_field {
                "created_at" => {
                    let a_ts = a.created_at.timestamp();
                    let b_ts = b.created_at.timestamp();
                    if ascending {
                        a_ts.cmp(&b_ts)
                    } else {
                        b_ts.cmp(&a_ts)
                    }
                }
                "completed_at" => {
                    let a_ts = a.completed_at.map(|d| d.timestamp()).unwrap_or(i64::MIN);
                    let b_ts = b.completed_at.map(|d| d.timestamp()).unwrap_or(i64::MIN);
                    if ascending {
                        a_ts.cmp(&b_ts)
                    } else {
                        b_ts.cmp(&a_ts)
                    }
                }
                "task_id" => {
                    if ascending {
                        a.task_id.cmp(&b.task_id)
                    } else {
                        b.task_id.cmp(&a.task_id)
                    }
                }
                "case_id" => {
                    if ascending {
                        a.case_id.cmp(&b.case_id)
                    } else {
                        b.case_id.cmp(&a.case_id)
                    }
                }
                "state" => {
                    let a_state = format!("{:?}", a.state);
                    let b_state = format!("{:?}", b.state);
                    if ascending {
                        a_state.cmp(&b_state)
                    } else {
                        b_state.cmp(&a_state)
                    }
                }
                _ => std::cmp::Ordering::Equal,
            };
            ordering
        });

        work_items
    }

    /// Get work items by multiple case IDs
    pub async fn get_work_items_by_cases(&self, case_ids: Vec<String>) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| case_ids.contains(&item.case_id))
            .cloned()
            .collect()
    }

    /// Get work items by multiple task IDs
    pub async fn get_work_items_by_tasks(&self, task_ids: Vec<String>) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| task_ids.contains(&item.task_id))
            .cloned()
            .collect()
    }

    /// Get work items by multiple resource IDs
    pub async fn get_work_items_by_resources(&self, resource_ids: Vec<String>) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| {
                if let Some(ref rid) = item.assigned_resource_id {
                    resource_ids.contains(rid)
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Get work items by multiple states
    pub async fn get_work_items_by_states(&self, states: Vec<WorkItemState>) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| states.contains(&item.state))
            .cloned()
            .collect()
    }

    /// Bulk complete work items
    pub async fn bulk_complete(
        &self,
        work_item_ids: Vec<String>,
        _resource_id: &str,
        results: Vec<serde_json::Value>,
    ) -> WorkflowResult<Vec<String>> {
        if work_item_ids.len() != results.len() {
            return Err(WorkflowError::Validation(
                "Work item IDs and results must have same length".to_string(),
            ));
        }

        let mut completed = Vec::new();
        let mut failed = Vec::new();

        for (work_item_id, result) in work_item_ids.into_iter().zip(results.into_iter()) {
            match self.complete(&work_item_id, result).await {
                Ok(()) => completed.push(work_item_id),
                Err(e) => {
                    failed.push(format!("{}: {}", work_item_id, e));
                }
            }
        }

        if !failed.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Failed to complete some work items: {:?}",
                failed
            )));
        }

        Ok(completed)
    }

    /// Bulk cancel work items
    pub async fn bulk_cancel(&self, work_item_ids: Vec<String>) -> WorkflowResult<Vec<String>> {
        let mut cancelled = Vec::new();
        let mut failed = Vec::new();

        for work_item_id in work_item_ids {
            match self.cancel(&work_item_id).await {
                Ok(()) => cancelled.push(work_item_id),
                Err(e) => {
                    failed.push(format!("{}: {}", work_item_id, e));
                }
            }
        }

        if !failed.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Failed to cancel some work items: {:?}",
                failed
            )));
        }

        Ok(cancelled)
    }

    /// Get work items with full text search
    pub async fn search_work_items(&self, query: &str) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        let query_lower = query.to_lowercase();
        items
            .values()
            .filter(|item| {
                item.task_id.to_lowercase().contains(&query_lower)
                    || item.case_id.to_lowercase().contains(&query_lower)
                    || item.id.to_lowercase().contains(&query_lower)
                    || (if let Some(name) = item.data.get("name") {
                        if let Some(name) = name.as_str() {
                            name.to_lowercase().contains(&query_lower)
                        } else {
                            false
                        }
                    } else {
                        false
                    })
            })
            .cloned()
            .collect()
    }

    /// Get work items by custom filter function
    pub async fn filter_work_items<F>(&self, filter: F) -> Vec<WorkItem>
    where
        F: Fn(&WorkItem) -> bool,
    {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| filter(item))
            .cloned()
            .collect()
    }

    /// Export work items to JSON
    pub async fn export_to_json(&self) -> WorkflowResult<String> {
        let items = self.work_items.read().await;
        let work_items: Vec<&WorkItem> = items.values().collect();
        serde_json::to_string(&work_items)
            .map_err(|e| WorkflowError::Internal(format!("Failed to serialize work items: {}", e)))
    }

    /// Import work items from JSON
    pub async fn import_from_json(&self, json: &str) -> WorkflowResult<usize> {
        let work_items: Vec<WorkItem> = serde_json::from_str(json).map_err(|e| {
            WorkflowError::Internal(format!("Failed to deserialize work items: {}", e))
        })?;

        let mut items = self.work_items.write().await;
        let mut imported = 0;

        for work_item in work_items {
            items.insert(work_item.id.clone(), work_item);
            imported += 1;
        }

        Ok(imported)
    }

    /// Get work items by exception type
    pub async fn get_work_items_by_exception(&self, exception_type: &str) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| {
                if let Some(exc_type) = item.data.get("exception_type") {
                    if let Some(exc_type) = exc_type.as_str() {
                        return exc_type == exception_type;
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Set work item exception
    pub async fn set_exception(
        &self,
        work_item_id: &str,
        exception_type: String,
        exception_data: serde_json::Value,
    ) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.insert(
                    "exception_type".to_string(),
                    serde_json::Value::String(exception_type),
                );
                data_obj.insert("exception_data".to_string(), exception_data);
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Clear work item exception
    pub async fn clear_exception(&self, work_item_id: &str) -> WorkflowResult<()> {
        let mut items = self.work_items.write().await;
        if let Some(item) = items.get_mut(work_item_id) {
            if let Some(data_obj) = item.data.as_object_mut() {
                data_obj.remove("exception_type");
                data_obj.remove("exception_data");
            }
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} not found",
                work_item_id
            )))
        }
    }

    /// Get work items with exceptions
    pub async fn get_work_items_with_exceptions(&self) -> Vec<WorkItem> {
        let items = self.work_items.read().await;
        items
            .values()
            .filter(|item| item.data.get("exception_type").is_some())
            .cloned()
            .collect()
    }
}

impl Default for WorkItemService {
    fn default() -> Self {
        Self::new()
    }
}
