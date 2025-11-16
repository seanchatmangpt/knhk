// rust/knhk-workflow-engine/src/cancellation.rs
//! Cancellation Regions and Sets
//!
//! Implements YAWL-style cancellation regions for workflow control flow.
//! Cancellation regions allow tasks to cancel other tasks or entire regions
//! when specific conditions are met.
//!
//! **YAWL Patterns Supported**:
//! - Pattern 19: Cancel Task
//! - Pattern 20: Cancel Case
//! - Pattern 25: Cancel Region
//!
//! **Architecture**:
//! - Lock-free cancellation flags for hot path performance
//! - Hierarchical region containment
//! - Ripple-down exception propagation

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Cancellation region identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegionId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl RegionId {
    /// Create new region ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RegionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Cancellation scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancellationScope {
    /// Cancel a single task
    Task,
    /// Cancel all tasks in a region
    Region,
    /// Cancel the entire case (workflow instance)
    Case,
}

/// Cancellation region definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationRegion {
    /// Region identifier
    pub id: RegionId,
    /// Region name
    pub name: String,
    /// Parent region (for hierarchical regions)
    pub parent: Option<RegionId>,
    /// Tasks in this region
    pub tasks: HashSet<String>,
    /// Child regions (for hierarchical containment)
    pub children: HashSet<RegionId>,
    /// Cancellation condition (evaluated against context)
    pub cancel_condition: Option<String>,
}

impl CancellationRegion {
    /// Create new cancellation region
    pub fn new(name: String) -> Self {
        Self {
            id: RegionId::new(),
            name,
            parent: None,
            tasks: HashSet::new(),
            children: HashSet::new(),
            cancel_condition: None,
        }
    }

    /// Add task to region
    pub fn add_task(&mut self, task_id: String) {
        self.tasks.insert(task_id);
    }

    /// Add child region
    pub fn add_child(&mut self, child_id: RegionId) {
        self.children.insert(child_id);
    }

    /// Check if region contains task
    pub fn contains_task(&self, task_id: &str) -> bool {
        self.tasks.contains(task_id)
    }
}

/// Cancellation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationEvent {
    /// Event timestamp (milliseconds since epoch)
    pub timestamp_ms: u64,
    /// Scope of cancellation
    pub scope: CancellationScope,
    /// Region ID (if scope is Region)
    pub region_id: Option<RegionId>,
    /// Task ID (if scope is Task)
    pub task_id: Option<String>,
    /// Case ID
    pub case_id: CaseId,
    /// Reason for cancellation
    pub reason: String,
    /// Whether to ripple down to child regions/tasks
    pub ripple_down: bool,
}

/// Cancellation registry (per case)
pub struct CancellationRegistry {
    /// Cancellation flag (atomic for hot path checks)
    cancelled: Arc<AtomicBool>,
    /// Cancellation regions
    regions: Arc<RwLock<HashMap<RegionId, CancellationRegion>>>,
    /// Task -> Region mapping
    task_regions: Arc<RwLock<HashMap<String, RegionId>>>,
    /// Cancelled tasks (for precise cancellation)
    cancelled_tasks: Arc<RwLock<HashSet<String>>>,
    /// Cancelled regions (for region cancellation)
    cancelled_regions: Arc<RwLock<HashSet<RegionId>>>,
    /// Cancellation events history
    events: Arc<RwLock<Vec<CancellationEvent>>>,
}

impl CancellationRegistry {
    /// Create new cancellation registry
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            regions: Arc::new(RwLock::new(HashMap::new())),
            task_regions: Arc::new(RwLock::new(HashMap::new())),
            cancelled_tasks: Arc::new(RwLock::new(HashSet::new())),
            cancelled_regions: Arc::new(RwLock::new(HashSet::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a cancellation region
    pub async fn register_region(&self, region: CancellationRegion) -> WorkflowResult<()> {
        let mut regions = self.regions.write().await;
        let mut task_regions = self.task_regions.write().await;

        // Map tasks to region
        for task_id in &region.tasks {
            task_regions.insert(task_id.clone(), region.id);
        }

        regions.insert(region.id, region);
        Ok(())
    }

    /// Check if case is cancelled (hot path - lock-free)
    #[inline(always)]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Check if specific task is cancelled (hot path - single lock)
    pub async fn is_task_cancelled(&self, task_id: &str) -> bool {
        // Fast path: check case cancellation
        if self.is_cancelled() {
            return true;
        }

        // Check task-specific cancellation
        let cancelled_tasks = self.cancelled_tasks.read().await;
        if cancelled_tasks.contains(task_id) {
            return true;
        }

        // Check if task's region is cancelled
        let task_regions = self.task_regions.read().await;
        if let Some(region_id) = task_regions.get(task_id) {
            let cancelled_regions = self.cancelled_regions.read().await;
            return cancelled_regions.contains(region_id);
        }

        false
    }

    /// Cancel entire case (Pattern 20)
    pub async fn cancel_case(
        &self,
        case_id: CaseId,
        reason: String,
    ) -> WorkflowResult<()> {
        self.cancelled.store(true, Ordering::Release);

        let event = CancellationEvent {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| WorkflowError::Internal(format!("Time error: {}", e)))?
                .as_millis() as u64,
            scope: CancellationScope::Case,
            region_id: None,
            task_id: None,
            case_id,
            reason,
            ripple_down: true,
        };

        let mut events = self.events.write().await;
        events.push(event);

        Ok(())
    }

    /// Cancel specific task (Pattern 19)
    pub async fn cancel_task(
        &self,
        case_id: CaseId,
        task_id: String,
        reason: String,
    ) -> WorkflowResult<()> {
        let mut cancelled_tasks = self.cancelled_tasks.write().await;
        cancelled_tasks.insert(task_id.clone());

        let event = CancellationEvent {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| WorkflowError::Internal(format!("Time error: {}", e)))?
                .as_millis() as u64,
            scope: CancellationScope::Task,
            region_id: None,
            task_id: Some(task_id),
            case_id,
            reason,
            ripple_down: false,
        };

        let mut events = self.events.write().await;
        events.push(event);

        Ok(())
    }

    /// Cancel region and optionally ripple down (Pattern 25)
    pub async fn cancel_region(
        &self,
        case_id: CaseId,
        region_id: RegionId,
        reason: String,
        ripple_down: bool,
    ) -> WorkflowResult<()> {
        let mut cancelled_regions = self.cancelled_regions.write().await;
        cancelled_regions.insert(region_id);

        // If ripple down, cancel all child regions
        if ripple_down {
            let regions = self.regions.read().await;
            if let Some(region) = regions.get(&region_id) {
                // Cancel all tasks in region
                let mut cancelled_tasks = self.cancelled_tasks.write().await;
                for task_id in &region.tasks {
                    cancelled_tasks.insert(task_id.clone());
                }

                // Recursively cancel child regions
                for child_id in &region.children {
                    cancelled_regions.insert(*child_id);
                    // TODO: Recursively cancel children's children
                }
            }
        }

        let event = CancellationEvent {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| WorkflowError::Internal(format!("Time error: {}", e)))?
                .as_millis() as u64,
            scope: CancellationScope::Region,
            region_id: Some(region_id),
            task_id: None,
            case_id,
            reason,
            ripple_down,
        };

        let mut events = self.events.write().await;
        events.push(event);

        Ok(())
    }

    /// Get cancellation events
    pub async fn get_events(&self) -> Vec<CancellationEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// Get region by ID
    pub async fn get_region(&self, region_id: RegionId) -> Option<CancellationRegion> {
        let regions = self.regions.read().await;
        regions.get(&region_id).cloned()
    }

    /// List all regions
    pub async fn list_regions(&self) -> Vec<CancellationRegion> {
        let regions = self.regions.read().await;
        regions.values().cloned().collect()
    }

    /// Reset cancellation state (for testing)
    pub async fn reset(&self) {
        self.cancelled.store(false, Ordering::Release);
        let mut cancelled_tasks = self.cancelled_tasks.write().await;
        cancelled_tasks.clear();
        let mut cancelled_regions = self.cancelled_regions.write().await;
        cancelled_regions.clear();
        let mut events = self.events.write().await;
        events.clear();
    }
}

impl Default for CancellationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_case_cancellation() {
        let registry = CancellationRegistry::new();
        let case_id = CaseId::new();

        assert!(!registry.is_cancelled());

        registry
            .cancel_case(case_id, "Test cancellation".to_string())
            .await
            .unwrap();

        assert!(registry.is_cancelled());

        let events = registry.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].scope, CancellationScope::Case);
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let registry = CancellationRegistry::new();
        let case_id = CaseId::new();

        registry
            .cancel_task(case_id, "task1".to_string(), "Test".to_string())
            .await
            .unwrap();

        assert!(registry.is_task_cancelled("task1").await);
        assert!(!registry.is_task_cancelled("task2").await);
    }

    #[tokio::test]
    async fn test_region_cancellation() {
        let registry = CancellationRegistry::new();

        // Create region with tasks
        let mut region = CancellationRegion::new("test_region".to_string());
        region.add_task("task1".to_string());
        region.add_task("task2".to_string());

        let region_id = region.id;
        registry.register_region(region).await.unwrap();

        // Cancel region with ripple down
        let case_id = CaseId::new();
        registry
            .cancel_region(case_id, region_id, "Test".to_string(), true)
            .await
            .unwrap();

        // Tasks in region should be cancelled
        assert!(registry.is_task_cancelled("task1").await);
        assert!(registry.is_task_cancelled("task2").await);
        assert!(!registry.is_task_cancelled("task3").await);
    }

    #[tokio::test]
    async fn test_hierarchical_regions() {
        let registry = CancellationRegistry::new();

        // Create parent region
        let mut parent = CancellationRegion::new("parent".to_string());
        parent.add_task("task1".to_string());
        let parent_id = parent.id;

        // Create child region
        let mut child = CancellationRegion::new("child".to_string());
        child.parent = Some(parent_id);
        child.add_task("task2".to_string());
        let child_id = child.id;

        parent.add_child(child_id);

        registry.register_region(parent).await.unwrap();
        registry.register_region(child).await.unwrap();

        // Cancel parent with ripple down
        let case_id = CaseId::new();
        registry
            .cancel_region(case_id, parent_id, "Test".to_string(), true)
            .await
            .unwrap();

        // Both parent and child tasks should be cancelled
        assert!(registry.is_task_cancelled("task1").await);
        assert!(registry.is_task_cancelled("task2").await);
    }
}
