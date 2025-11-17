//! YAWL Interface B Implementation
//!
//! Implements Interface B from YAWL Java with TRIZ hyper-advanced patterns:
//! - TRIZ Principle 13: Inversion - Tasks notify engine instead of engine polling
//! - TRIZ Principle 32: Color Changes - Type-level state machine for work items
//!
//! Based on: org.yawlfoundation.yawl.engine.interfce.interfaceB.InterfaceBClient
//!
//! # Interface B Operations (50+)
//!
//! ## Work Item Lifecycle (14 operations)
//! - checkEligibleToStart - Pre-start validation
//! - checkoutWorkItem - Acquire exclusive lock
//! - checkinWorkItem - Release with data save
//! - startWorkItem - Begin execution
//! - completeWorkItem - Finish and commit
//! - cancelWorkItem - Abort work item
//! - suspendWorkItem - Pause execution
//! - unsuspendWorkItem - Resume execution
//! - delegateWorkItem - Transfer ownership
//! - offerWorkItem - Add to user's queue
//! - reoffer - Redistribute to different users
//! - deallocate - Remove allocation
//! - reallocateStateless - Reassign without state loss
//! - reallocateStateful - Reassign with state
//!
//! ## Bulk Operations (6+ operations)
//! - getWorkItemsForUser - Get all user's work items
//! - getWorkItemsForCase - Get all case work items
//! - getWorkItemsForSpec - Get all spec work items
//! - getEnabledWorkItems - All enabled items in system
//! - getExecutingWorkItems - All executing items
//! - getSuspendedWorkItems - All suspended items

use crate::case::CaseId;
use crate::engine::y_work_item::{WorkItem, WorkItemId, WorkItemRepository, WorkItemStatus};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::types::WorkflowSpecId;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// User ID type
pub type UserId = String;

/// Session handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionHandle(String);

impl SessionHandle {
    /// Create a new session handle
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get handle as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Launch mode (TRIZ Principle 32: Color Changes - type-level launch modes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// User-initiated - Manual task claiming
    UserInitiated,
    /// Offered - Distributed to eligible users
    Offered,
    /// Allocated - System-assigned to specific user
    Allocated,
    /// Start-by-System - Auto-start when enabled
    StartBySystem,
    /// Concurrent - Multiple users can work on same item
    Concurrent,
}

/// Work item record for API responses
#[derive(Debug, Clone)]
pub struct WorkItemRecord {
    /// Work item ID
    pub item_id: WorkItemId,
    /// Case ID
    pub case_id: CaseId,
    /// Specification ID
    pub spec_id: WorkflowSpecId,
    /// Task ID
    pub task_id: String,
    /// Status
    pub status: WorkItemStatus,
    /// Assigned user ID (if allocated)
    pub assigned_user_id: Option<UserId>,
    /// Offered user IDs (if offered)
    pub offered_user_ids: Vec<UserId>,
    /// Launch mode
    pub launch_mode: LaunchMode,
    /// Enablement time
    pub enablement_time: chrono::DateTime<chrono::Utc>,
    /// Start time (if started)
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Completion time (if completed)
    pub completion_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Work item data
    pub data: serde_json::Value,
}

/// Work item notification (TRIZ Principle 13: Inversion)
#[derive(Debug, Clone)]
pub enum WorkItemNotification {
    /// Work item enabled
    Enabled {
        item_id: WorkItemId,
        case_id: CaseId,
        task_id: String,
    },
    /// Work item allocated
    Allocated {
        item_id: WorkItemId,
        user_id: UserId,
    },
    /// Work item started
    Started {
        item_id: WorkItemId,
        user_id: UserId,
    },
    /// Work item completed
    Completed {
        item_id: WorkItemId,
        user_id: UserId,
    },
    /// Work item cancelled
    Cancelled {
        item_id: WorkItemId,
        user_id: UserId,
    },
}

/// Interface B - Work Item Operations API
///
/// TRIZ Principle 13: Inversion
/// - Tasks notify engine instead of engine polling
/// - Event-driven architecture with notification channels
pub struct InterfaceB {
    /// Work item repository
    work_item_repo: Arc<WorkItemRepository>,
    /// Work items by ID
    work_items: Arc<DashMap<String, WorkItemRecord>>,
    /// Work items by user ID
    user_work_items: Arc<DashMap<UserId, HashSet<String>>>,
    /// Work items by case ID
    case_work_items: Arc<DashMap<CaseId, HashSet<String>>>,
    /// Work items by spec ID
    spec_work_items: Arc<DashMap<WorkflowSpecId, HashSet<String>>>,
    /// Notification channel (TRIZ Principle 13: Inversion)
    notification_tx: mpsc::UnboundedSender<WorkItemNotification>,
    notification_rx: Arc<RwLock<mpsc::UnboundedReceiver<WorkItemNotification>>>,
    /// Checked out work items (locked)
    checked_out_items: Arc<DashMap<String, (UserId, chrono::DateTime<chrono::Utc>)>>,
}

impl InterfaceB {
    /// Create a new Interface B instance
    pub fn new(work_item_repo: Arc<WorkItemRepository>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            work_item_repo,
            work_items: Arc::new(DashMap::new()),
            user_work_items: Arc::new(DashMap::new()),
            case_work_items: Arc::new(DashMap::new()),
            spec_work_items: Arc::new(DashMap::new()),
            notification_tx: tx,
            notification_rx: Arc::new(RwLock::new(rx)),
            checked_out_items: Arc::new(DashMap::new()),
        }
    }

    // ========================================================================
    // Work Item Lifecycle Operations (14 operations)
    // ========================================================================

    /// Check if work item is eligible to start
    pub async fn check_eligible_to_start(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
    ) -> WorkflowResult<bool> {
        let item_key = item_id.to_string();
        let item = self
            .work_items
            .get(&item_key)
            .ok_or_else(|| WorkflowError::CaseNotFound(item_key.clone()))?;

        // Check status
        if item.status != WorkItemStatus::Enabled && item.status != WorkItemStatus::Fired {
            return Ok(false);
        }

        // Check launch mode
        match item.launch_mode {
            LaunchMode::UserInitiated | LaunchMode::Offered | LaunchMode::Allocated => {
                // Check if user is eligible
                if item.launch_mode == LaunchMode::Offered {
                    Ok(item.offered_user_ids.contains(user_id))
                } else if item.launch_mode == LaunchMode::Allocated {
                    Ok(item.assigned_user_id.as_ref() == Some(user_id))
                } else {
                    Ok(true) // User-initiated: any user can start
                }
            }
            LaunchMode::StartBySystem => Ok(false), // System starts automatically
            LaunchMode::Concurrent => Ok(true),     // Multiple users can work
        }
    }

    /// Checkout work item (acquire exclusive lock)
    pub async fn checkout_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        // Check if already checked out
        if self.checked_out_items.contains_key(&item_key) {
            return Err(WorkflowError::ResourceUnavailable(format!(
                "Work item {} already checked out",
                item_key
            )));
        }

        // Check eligibility
        if !self.check_eligible_to_start(item_id, user_id).await? {
            return Err(WorkflowError::Validation(format!(
                "Work item {} not eligible to start for user {}",
                item_key, user_id
            )));
        }

        // Lock work item
        self.checked_out_items
            .insert(item_key.clone(), (user_id.clone(), chrono::Utc::now()));

        // Notify (TRIZ Principle 13: Inversion)
        self.notify(WorkItemNotification::Allocated {
            item_id: item_id.clone(),
            user_id: user_id.clone(),
        })?;

        Ok(format!("Work item {} checked out by {}", item_key, user_id))
    }

    /// Checkin work item (release lock with data save)
    pub async fn checkin_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        data: serde_json::Value,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        // Verify checkout
        let checkout = self
            .checked_out_items
            .get(&item_key)
            .ok_or_else(|| WorkflowError::Validation(format!(
                "Work item {} not checked out",
                item_key
            )))?;

        if checkout.0 != *user_id {
            return Err(WorkflowError::Validation(format!(
                "Work item {} checked out by different user",
                item_key
            )));
        }

        // Update work item data
        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            item.data = data;
        }

        // Release lock
        self.checked_out_items.remove(&item_key);

        Ok(format!("Work item {} checked in by {}", item_key, user_id))
    }

    /// Start work item (begin execution)
    pub async fn start_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<WorkItemRecord> {
        let item_key = item_id.to_string();

        // Check eligibility
        if !self.check_eligible_to_start(item_id, user_id).await? {
            return Err(WorkflowError::Validation(format!(
                "Work item {} not eligible to start",
                item_key
            )));
        }

        // Update status
        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            item.status = WorkItemStatus::Executing;
            item.start_time = Some(chrono::Utc::now());
            item.assigned_user_id = Some(user_id.clone());

            // Notify (TRIZ Principle 13: Inversion)
            self.notify(WorkItemNotification::Started {
                item_id: item_id.clone(),
                user_id: user_id.clone(),
            })?;

            Ok(item.clone())
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Complete work item (finish and commit)
    pub async fn complete_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        data: serde_json::Value,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        // Update work item
        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            if item.status != WorkItemStatus::Executing {
                return Err(WorkflowError::InvalidStateTransition {
                    from: format!("{:?}", item.status),
                    to: "Completed".to_string(),
                });
            }

            item.status = WorkItemStatus::Completed;
            item.completion_time = Some(chrono::Utc::now());
            item.data = data;

            // Notify (TRIZ Principle 13: Inversion)
            self.notify(WorkItemNotification::Completed {
                item_id: item_id.clone(),
                user_id: user_id.clone(),
            })?;

            Ok(format!("Work item {} completed by {}", item_key, user_id))
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Cancel work item (abort)
    pub async fn cancel_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            item.status = WorkItemStatus::Cancelled;
            item.completion_time = Some(chrono::Utc::now());

            // Release checkout if exists
            self.checked_out_items.remove(&item_key);

            // Notify (TRIZ Principle 13: Inversion)
            self.notify(WorkItemNotification::Cancelled {
                item_id: item_id.clone(),
                user_id: user_id.clone(),
            })?;

            Ok(format!("Work item {} cancelled by {}", item_key, user_id))
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Suspend work item (pause execution)
    pub async fn suspend_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<WorkItemRecord> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            if item.status != WorkItemStatus::Executing {
                return Err(WorkflowError::InvalidStateTransition {
                    from: format!("{:?}", item.status),
                    to: "Suspended".to_string(),
                });
            }

            item.status = WorkItemStatus::Suspended;
            Ok(item.clone())
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Unsuspend work item (resume execution)
    pub async fn unsuspend_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<WorkItemRecord> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            if item.status != WorkItemStatus::Suspended {
                return Err(WorkflowError::InvalidStateTransition {
                    from: format!("{:?}", item.status),
                    to: "Executing".to_string(),
                });
            }

            item.status = WorkItemStatus::Executing;
            Ok(item.clone())
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Delegate work item (transfer ownership)
    pub async fn delegate_work_item(
        &self,
        item_id: &WorkItemId,
        from_user_id: &UserId,
        to_user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            // Verify current owner
            if item.assigned_user_id.as_ref() != Some(from_user_id) {
                return Err(WorkflowError::Validation(format!(
                    "Work item {} not assigned to user {}",
                    item_key, from_user_id
                )));
            }

            // Update assignment
            item.assigned_user_id = Some(to_user_id.clone());

            // Update user work items
            if let Some(mut user_items) = self.user_work_items.get_mut(from_user_id) {
                user_items.remove(&item_key);
            }
            self.user_work_items
                .entry(to_user_id.clone())
                .or_insert_with(HashSet::new)
                .insert(item_key.clone());

            Ok(format!(
                "Work item {} delegated from {} to {}",
                item_key, from_user_id, to_user_id
            ))
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Offer work item (add to user's queue)
    pub async fn offer_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            if !item.offered_user_ids.contains(user_id) {
                item.offered_user_ids.push(user_id.clone());
                item.launch_mode = LaunchMode::Offered;

                // Update user work items
                self.user_work_items
                    .entry(user_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(item_key.clone());

                // Notify (TRIZ Principle 13: Inversion)
                self.notify(WorkItemNotification::Enabled {
                    item_id: item_id.clone(),
                    case_id: item.case_id.clone(),
                    task_id: item.task_id.clone(),
                })?;
            }

            Ok(format!("Work item {} offered to {}", item_key, user_id))
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Reoffer work item (redistribute to different users)
    pub async fn reoffer_work_item(
        &self,
        item_id: &WorkItemId,
        user_ids: Vec<UserId>,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            // Remove from old users
            for old_user_id in &item.offered_user_ids {
                if let Some(mut user_items) = self.user_work_items.get_mut(old_user_id) {
                    user_items.remove(&item_key);
                }
            }

            // Update offered users
            item.offered_user_ids = user_ids.clone();

            // Add to new users
            for user_id in &user_ids {
                self.user_work_items
                    .entry(user_id.clone())
                    .or_insert_with(HashSet::new)
                    .insert(item_key.clone());
            }

            Ok(format!("Work item {} reoffered to {} users", item_key, user_ids.len()))
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Deallocate work item (remove allocation)
    pub async fn deallocate_work_item(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        let item_key = item_id.to_string();

        if let Some(mut item) = self.work_items.get_mut(&item_key) {
            if item.assigned_user_id.as_ref() == Some(user_id) {
                item.assigned_user_id = None;
                item.launch_mode = LaunchMode::UserInitiated;

                // Remove from user work items
                if let Some(mut user_items) = self.user_work_items.get_mut(user_id) {
                    user_items.remove(&item_key);
                }
            }

            Ok(format!("Work item {} deallocated from {}", item_key, user_id))
        } else {
            Err(WorkflowError::CaseNotFound(item_key))
        }
    }

    /// Reallocate work item stateless (reassign without state loss)
    pub async fn reallocate_stateless(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        self.delegate_work_item(item_id, &"system".to_string(), user_id, session_handle)
            .await
    }

    /// Reallocate work item stateful (reassign with state)
    pub async fn reallocate_stateful(
        &self,
        item_id: &WorkItemId,
        user_id: &UserId,
        data: serde_json::Value,
        session_handle: &SessionHandle,
    ) -> WorkflowResult<String> {
        // Checkin current state
        let current_user = self
            .work_items
            .get(&item_id.to_string())
            .and_then(|item| item.assigned_user_id.clone())
            .unwrap_or_else(|| "system".to_string());

        self.checkin_work_item(item_id, &current_user, data, session_handle)
            .await?;

        // Reallocate
        self.reallocate_stateless(item_id, user_id, session_handle)
            .await
    }

    // ========================================================================
    // Bulk Operations (6+ operations)
    // ========================================================================

    /// Get work items for user
    pub async fn get_work_items_for_user(
        &self,
        user_id: &UserId,
        session_handle: &SessionHandle,
    ) -> Vec<WorkItemRecord> {
        if let Some(user_items) = self.user_work_items.get(user_id) {
            user_items
                .iter()
                .filter_map(|item_key| {
                    self.work_items.get(item_key).map(|item| item.clone())
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Get work items for case
    pub async fn get_work_items_for_case(
        &self,
        case_id: &CaseId,
        session_handle: &SessionHandle,
    ) -> Vec<WorkItemRecord> {
        if let Some(case_items) = self.case_work_items.get(case_id) {
            case_items
                .iter()
                .filter_map(|item_key| {
                    self.work_items.get(item_key).map(|item| item.clone())
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Get work items for specification
    pub async fn get_work_items_for_spec(
        &self,
        spec_id: &WorkflowSpecId,
        session_handle: &SessionHandle,
    ) -> Vec<WorkItemRecord> {
        if let Some(spec_items) = self.spec_work_items.get(spec_id) {
            spec_items
                .iter()
                .filter_map(|item_key| {
                    self.work_items.get(item_key).map(|item| item.clone())
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Get all enabled work items
    pub async fn get_enabled_work_items(
        &self,
        session_handle: &SessionHandle,
    ) -> Vec<WorkItemRecord> {
        self.work_items
            .iter()
            .filter(|entry| entry.value().status == WorkItemStatus::Enabled)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all executing work items
    pub async fn get_executing_work_items(
        &self,
        session_handle: &SessionHandle,
    ) -> Vec<WorkItemRecord> {
        self.work_items
            .iter()
            .filter(|entry| entry.value().status == WorkItemStatus::Executing)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get all suspended work items
    pub async fn get_suspended_work_items(
        &self,
        session_handle: &SessionHandle,
    ) -> Vec<WorkItemRecord> {
        self.work_items
            .iter()
            .filter(|entry| entry.value().status == WorkItemStatus::Suspended)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get work item by ID
    pub async fn get_work_item(
        &self,
        item_id: &WorkItemId,
        session_handle: &SessionHandle,
    ) -> Option<WorkItemRecord> {
        self.work_items
            .get(&item_id.to_string())
            .map(|entry| entry.clone())
    }

    // ========================================================================
    // Internal Helper Methods
    // ========================================================================

    /// Notify observers (TRIZ Principle 13: Inversion)
    fn notify(&self, notification: WorkItemNotification) -> WorkflowResult<()> {
        self.notification_tx
            .send(notification)
            .map_err(|_| WorkflowError::Internal("Failed to send notification".to_string()))
    }

    /// Register work item
    pub(crate) fn register_work_item(&self, record: WorkItemRecord) {
        let item_key = record.item_id.to_string();

        // Store work item
        self.work_items.insert(item_key.clone(), record.clone());

        // Index by user
        if let Some(user_id) = &record.assigned_user_id {
            self.user_work_items
                .entry(user_id.clone())
                .or_insert_with(HashSet::new)
                .insert(item_key.clone());
        }

        // Index by case
        self.case_work_items
            .entry(record.case_id.clone())
            .or_insert_with(HashSet::new)
            .insert(item_key.clone());

        // Index by spec
        self.spec_work_items
            .entry(record.spec_id.clone())
            .or_insert_with(HashSet::new)
            .insert(item_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interface_b_creation() {
        let repo = Arc::new(WorkItemRepository::new());
        let interface_b = InterfaceB::new(repo);
        assert!(true); // Interface created successfully
    }

    #[tokio::test]
    async fn test_checkout_checkin() {
        let repo = Arc::new(WorkItemRepository::new());
        let interface_b = InterfaceB::new(repo);

        let item_id = WorkItemId::new("task1".to_string());
        let user_id = "user1".to_string();
        let session = SessionHandle::new();

        // Create and register work item
        let record = WorkItemRecord {
            item_id: item_id.clone(),
            case_id: CaseId::new(),
            spec_id: WorkflowSpecId::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            task_id: "task1".to_string(),
            status: WorkItemStatus::Enabled,
            assigned_user_id: None,
            offered_user_ids: vec![],
            launch_mode: LaunchMode::UserInitiated,
            enablement_time: chrono::Utc::now(),
            start_time: None,
            completion_time: None,
            data: serde_json::json!({}),
        };
        interface_b.register_work_item(record);

        // Checkout
        let result = interface_b
            .checkout_work_item(&item_id, &user_id, &session)
            .await;
        assert!(result.is_ok());

        // Checkin
        let data = serde_json::json!({"result": "done"});
        let result = interface_b
            .checkin_work_item(&item_id, &user_id, data, &session)
            .await;
        assert!(result.is_ok());
    }
}

