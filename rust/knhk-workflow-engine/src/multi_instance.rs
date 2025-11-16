// rust/knhk-workflow-engine/src/multi_instance.rs
//! Multi-Instance Task Execution
//!
//! Implements YAWL-style multi-instance patterns for parallel task execution
//! with configurable synchronization strategies.
//!
//! **YAWL Patterns Supported**:
//! - Pattern 12: Multiple Instances Without Synchronization
//! - Pattern 13: Multiple Instances With a Priori Design Time Knowledge
//! - Pattern 14: Multiple Instances With a Priori Runtime Knowledge
//! - Pattern 15: Multiple Instances Without a Priori Runtime Knowledge
//!
//! **Architecture**:
//! - Dynamic instance creation based on runtime data
//! - Configurable completion conditions (all, one, threshold)
//! - Lock-free progress tracking for hot path performance

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{PatternExecutionContext, PatternExecutionResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Multi-instance task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MultiInstanceId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl MultiInstanceId {
    /// Create new multi-instance ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MultiInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Instance identifier (within a multi-instance task)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(pub usize);

/// Multi-instance creation strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreationStrategy {
    /// Fixed number of instances (Pattern 13)
    Static(usize),
    /// Number determined at runtime from data (Pattern 14)
    Dynamic {
        /// Variable name containing instance count or collection
        variable: String,
    },
    /// Create instances on-demand without prior knowledge (Pattern 15)
    OnDemand {
        /// Maximum instances allowed
        max_instances: usize,
    },
}

/// Multi-instance completion condition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionCondition {
    /// Wait for all instances to complete
    All,
    /// Complete when any one instance completes
    One,
    /// Complete when N instances complete
    Threshold(usize),
    /// Complete when N% of instances complete
    Percentage(u8), // 0-100
    /// Custom condition expression
    Custom(String),
}

/// Multi-instance synchronization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynchronizationMode {
    /// No synchronization - instances run independently (Pattern 12)
    None,
    /// Synchronize all instances at completion
    Full,
    /// Partial synchronization based on condition
    Partial,
}

/// Multi-instance specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceSpec {
    /// Multi-instance identifier
    pub id: MultiInstanceId,
    /// Task ID for the multi-instance task
    pub task_id: String,
    /// Creation strategy
    pub creation: CreationStrategy,
    /// Completion condition
    pub completion: CompletionCondition,
    /// Synchronization mode
    pub synchronization: SynchronizationMode,
    /// Input data splitting strategy
    pub input_split: Option<String>, // Variable name for split
    /// Output data merging strategy
    pub output_merge: Option<String>, // Expression for merge
}

/// Instance state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceState {
    /// Instance is ready to execute
    Ready,
    /// Instance is currently executing
    Running,
    /// Instance completed successfully
    Completed,
    /// Instance failed
    Failed,
    /// Instance was cancelled
    Cancelled,
}

/// Multi-instance execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceContext {
    /// Instance identifier
    pub instance_id: InstanceId,
    /// Instance state
    pub state: InstanceState,
    /// Instance-specific input data
    pub input_data: HashMap<String, String>,
    /// Instance-specific output data
    pub output_data: Option<HashMap<String, String>>,
    /// Execution start time (ms since epoch)
    pub start_time_ms: Option<u64>,
    /// Execution end time (ms since epoch)
    pub end_time_ms: Option<u64>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl InstanceContext {
    /// Create new instance context
    pub fn new(instance_id: InstanceId, input_data: HashMap<String, String>) -> Self {
        Self {
            instance_id,
            state: InstanceState::Ready,
            input_data,
            output_data: None,
            start_time_ms: None,
            end_time_ms: None,
            error: None,
        }
    }
}

/// Multi-instance tracker (per multi-instance task)
pub struct MultiInstanceTracker {
    /// Multi-instance specification
    spec: MultiInstanceSpec,
    /// Instance contexts
    instances: Arc<RwLock<HashMap<InstanceId, InstanceContext>>>,
    /// Completed instance count (atomic for hot path checks)
    completed_count: Arc<AtomicUsize>,
    /// Failed instance count
    failed_count: Arc<AtomicUsize>,
    /// Total instance count
    total_count: Arc<AtomicUsize>,
    /// Whether multi-instance execution is complete
    complete: Arc<AtomicBool>,
}

use std::sync::atomic::AtomicBool;

impl MultiInstanceTracker {
    /// Create new multi-instance tracker
    pub fn new(spec: MultiInstanceSpec) -> Self {
        Self {
            spec,
            instances: Arc::new(RwLock::new(HashMap::new())),
            completed_count: Arc::new(AtomicUsize::new(0)),
            failed_count: Arc::new(AtomicUsize::new(0)),
            total_count: Arc::new(AtomicUsize::new(0)),
            complete: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create instances based on strategy
    pub async fn create_instances(
        &self,
        context: &PatternExecutionContext,
    ) -> WorkflowResult<Vec<InstanceId>> {
        let count = match &self.spec.creation {
            CreationStrategy::Static(n) => *n,
            CreationStrategy::Dynamic { variable } => {
                // Get count from context variable
                if let Some(value) = context.variables.get(variable) {
                    value.parse::<usize>().map_err(|e| {
                        WorkflowError::Validation(format!(
                            "Cannot parse instance count from '{}': {}",
                            variable, e
                        ))
                    })?
                } else {
                    return Err(WorkflowError::Validation(format!(
                        "Variable '{}' not found for dynamic instance creation",
                        variable
                    )));
                }
            }
            CreationStrategy::OnDemand { max_instances } => {
                // Start with 1 instance, more can be added dynamically
                *max_instances.min(&1)
            }
        };

        let mut instance_ids = Vec::new();
        let mut instances = self.instances.write().await;

        for i in 0..count {
            let instance_id = InstanceId(i);

            // Split input data if configured
            let input_data = if let Some(split_var) = &self.spec.input_split {
                if let Some(collection) = context.variables.get(split_var) {
                    // TODO: Implement proper collection splitting
                    // For now, pass the whole collection to each instance
                    let mut data = HashMap::new();
                    data.insert(split_var.clone(), collection.clone());
                    data.insert("instance_index".to_string(), i.to_string());
                    data
                } else {
                    HashMap::new()
                }
            } else {
                let mut data = HashMap::new();
                data.insert("instance_index".to_string(), i.to_string());
                data
            };

            let instance_context = InstanceContext::new(instance_id, input_data);
            instances.insert(instance_id, instance_context);
            instance_ids.push(instance_id);
        }

        self.total_count.store(count, Ordering::Release);

        Ok(instance_ids)
    }

    /// Mark instance as running
    pub async fn start_instance(&self, instance_id: InstanceId) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.state = InstanceState::Running;
            instance.start_time_ms = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| WorkflowError::Internal(format!("Time error: {}", e)))?
                    .as_millis() as u64,
            );
            Ok(())
        } else {
            Err(WorkflowError::Internal(format!(
                "Instance {:?} not found",
                instance_id
            )))
        }
    }

    /// Mark instance as completed
    pub async fn complete_instance(
        &self,
        instance_id: InstanceId,
        output_data: HashMap<String, String>,
    ) -> WorkflowResult<bool> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.state = InstanceState::Completed;
            instance.output_data = Some(output_data);
            instance.end_time_ms = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| WorkflowError::Internal(format!("Time error: {}", e)))?
                    .as_millis() as u64,
            );

            let completed = self.completed_count.fetch_add(1, Ordering::AcqRel) + 1;

            // Check if completion condition is met
            let is_complete = self.check_completion_condition(completed).await;
            if is_complete {
                self.complete.store(true, Ordering::Release);
            }

            Ok(is_complete)
        } else {
            Err(WorkflowError::Internal(format!(
                "Instance {:?} not found",
                instance_id
            )))
        }
    }

    /// Mark instance as failed
    pub async fn fail_instance(
        &self,
        instance_id: InstanceId,
        error: String,
    ) -> WorkflowResult<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(&instance_id) {
            instance.state = InstanceState::Failed;
            instance.error = Some(error);
            instance.end_time_ms = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| WorkflowError::Internal(format!("Time error: {}", e)))?
                    .as_millis() as u64,
            );

            self.failed_count.fetch_add(1, Ordering::AcqRel);

            Ok(())
        } else {
            Err(WorkflowError::Internal(format!(
                "Instance {:?} not found",
                instance_id
            )))
        }
    }

    /// Check if completion condition is met
    async fn check_completion_condition(&self, completed: usize) -> bool {
        let total = self.total_count.load(Ordering::Acquire);

        match &self.spec.completion {
            CompletionCondition::All => completed >= total,
            CompletionCondition::One => completed >= 1,
            CompletionCondition::Threshold(n) => completed >= *n,
            CompletionCondition::Percentage(pct) => {
                let threshold = (total as f64 * (*pct as f64 / 100.0)).ceil() as usize;
                completed >= threshold
            }
            CompletionCondition::Custom(_expr) => {
                // TODO: Evaluate custom expression
                completed >= total
            }
        }
    }

    /// Check if multi-instance execution is complete (hot path)
    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::Relaxed)
    }

    /// Get completion statistics
    pub async fn get_stats(&self) -> MultiInstanceStats {
        let instances = self.instances.read().await;

        let total = self.total_count.load(Ordering::Acquire);
        let completed = self.completed_count.load(Ordering::Acquire);
        let failed = self.failed_count.load(Ordering::Acquire);
        let running = instances
            .values()
            .filter(|i| i.state == InstanceState::Running)
            .count();

        MultiInstanceStats {
            total_instances: total,
            completed_instances: completed,
            failed_instances: failed,
            running_instances: running,
            pending_instances: total - completed - failed - running,
        }
    }

    /// Merge instance outputs
    pub async fn merge_outputs(&self) -> HashMap<String, Vec<String>> {
        let instances = self.instances.read().await;
        let mut merged: HashMap<String, Vec<String>> = HashMap::new();

        for instance in instances.values() {
            if let Some(output) = &instance.output_data {
                for (key, value) in output {
                    merged
                        .entry(key.clone())
                        .or_insert_with(Vec::new)
                        .push(value.clone());
                }
            }
        }

        merged
    }

    /// Get all instance contexts
    pub async fn get_instances(&self) -> Vec<InstanceContext> {
        let instances = self.instances.read().await;
        instances.values().cloned().collect()
    }
}

/// Multi-instance execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceStats {
    /// Total number of instances
    pub total_instances: usize,
    /// Number of completed instances
    pub completed_instances: usize,
    /// Number of failed instances
    pub failed_instances: usize,
    /// Number of running instances
    pub running_instances: usize,
    /// Number of pending instances
    pub pending_instances: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_static_instance_creation() {
        let spec = MultiInstanceSpec {
            id: MultiInstanceId::new(),
            task_id: "task1".to_string(),
            creation: CreationStrategy::Static(5),
            completion: CompletionCondition::All,
            synchronization: SynchronizationMode::Full,
            input_split: None,
            output_merge: None,
        };

        let tracker = MultiInstanceTracker::new(spec);

        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let instances = tracker.create_instances(&context).await.unwrap();
        assert_eq!(instances.len(), 5);
    }

    #[tokio::test]
    async fn test_completion_all() {
        let spec = MultiInstanceSpec {
            id: MultiInstanceId::new(),
            task_id: "task1".to_string(),
            creation: CreationStrategy::Static(3),
            completion: CompletionCondition::All,
            synchronization: SynchronizationMode::Full,
            input_split: None,
            output_merge: None,
        };

        let tracker = MultiInstanceTracker::new(spec);

        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let instances = tracker.create_instances(&context).await.unwrap();

        // Complete instances one by one
        assert!(!tracker.is_complete());

        tracker
            .complete_instance(instances[0], HashMap::new())
            .await
            .unwrap();
        assert!(!tracker.is_complete());

        tracker
            .complete_instance(instances[1], HashMap::new())
            .await
            .unwrap();
        assert!(!tracker.is_complete());

        let is_complete = tracker
            .complete_instance(instances[2], HashMap::new())
            .await
            .unwrap();
        assert!(is_complete);
        assert!(tracker.is_complete());
    }

    #[tokio::test]
    async fn test_completion_one() {
        let spec = MultiInstanceSpec {
            id: MultiInstanceId::new(),
            task_id: "task1".to_string(),
            creation: CreationStrategy::Static(5),
            completion: CompletionCondition::One,
            synchronization: SynchronizationMode::None,
            input_split: None,
            output_merge: None,
        };

        let tracker = MultiInstanceTracker::new(spec);

        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let instances = tracker.create_instances(&context).await.unwrap();

        // Complete just one instance
        let is_complete = tracker
            .complete_instance(instances[0], HashMap::new())
            .await
            .unwrap();

        assert!(is_complete);
        assert!(tracker.is_complete());
    }

    #[tokio::test]
    async fn test_completion_threshold() {
        let spec = MultiInstanceSpec {
            id: MultiInstanceId::new(),
            task_id: "task1".to_string(),
            creation: CreationStrategy::Static(10),
            completion: CompletionCondition::Threshold(7),
            synchronization: SynchronizationMode::Partial,
            input_split: None,
            output_merge: None,
        };

        let tracker = MultiInstanceTracker::new(spec);

        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let instances = tracker.create_instances(&context).await.unwrap();

        // Complete 7 instances
        for i in 0..7 {
            tracker
                .complete_instance(instances[i], HashMap::new())
                .await
                .unwrap();
        }

        assert!(tracker.is_complete());

        let stats = tracker.get_stats().await;
        assert_eq!(stats.completed_instances, 7);
        assert_eq!(stats.total_instances, 10);
    }

    #[tokio::test]
    async fn test_output_merging() {
        let spec = MultiInstanceSpec {
            id: MultiInstanceId::new(),
            task_id: "task1".to_string(),
            creation: CreationStrategy::Static(3),
            completion: CompletionCondition::All,
            synchronization: SynchronizationMode::Full,
            input_split: None,
            output_merge: None,
        };

        let tracker = MultiInstanceTracker::new(spec);

        let context = PatternExecutionContext {
            case_id: CaseId::new(),
            workflow_id: crate::parser::WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let instances = tracker.create_instances(&context).await.unwrap();

        // Complete instances with different outputs
        for (i, instance_id) in instances.iter().enumerate() {
            let mut output = HashMap::new();
            output.insert("result".to_string(), format!("value_{}", i));
            tracker
                .complete_instance(*instance_id, output)
                .await
                .unwrap();
        }

        let merged = tracker.merge_outputs().await;
        assert_eq!(merged.get("result").unwrap().len(), 3);
        assert!(merged.get("result").unwrap().contains(&"value_0".to_string()));
        assert!(merged.get("result").unwrap().contains(&"value_1".to_string()));
        assert!(merged.get("result").unwrap().contains(&"value_2".to_string()));
    }
}
