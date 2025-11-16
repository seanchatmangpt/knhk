//! Work-stealing executor integration for MI patterns
//!
//! Provides instance spawning and execution on work-stealing executor.

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::mi::{InstanceMetadata, InstanceStatus, InstanceTracker, SyncGate};
use crate::patterns::PatternId;
use std::sync::Arc;

#[cfg(feature = "async-v2")]
use crate::concurrency::WorkStealingExecutor;

#[cfg(feature = "rdf")]
use oxigraph::store::Store;
#[cfg(feature = "rdf")]
use tokio::sync::RwLock;

/// Instance executor for MI patterns
pub struct InstanceExecutor {
    #[cfg(feature = "async-v2")]
    executor: Arc<WorkStealingExecutor>,
    #[cfg(feature = "rdf")]
    instance_tracker: Arc<InstanceTracker>,
    #[cfg(feature = "rdf")]
    sync_gate: Arc<SyncGate>,
}

impl InstanceExecutor {
    /// Create new instance executor
    #[cfg(all(feature = "async-v2", feature = "rdf"))]
    pub fn new(
        executor: Arc<WorkStealingExecutor>,
        rdf_store: Arc<RwLock<Store>>,
    ) -> Self {
        let instance_tracker = Arc::new(InstanceTracker::new(rdf_store.clone()));
        let sync_gate = Arc::new(SyncGate::new(rdf_store));

        Self {
            executor,
            instance_tracker,
            sync_gate,
        }
    }

    #[cfg(not(all(feature = "async-v2", feature = "rdf")))]
    pub fn new() -> Self {
        Self {}
    }

    /// Get instance tracker
    #[cfg(feature = "rdf")]
    pub fn instance_tracker(&self) -> &InstanceTracker {
        &self.instance_tracker
    }

    /// Get sync gate
    #[cfg(feature = "rdf")]
    pub fn sync_gate(&self) -> &SyncGate {
        &self.sync_gate
    }
}

#[cfg(not(all(feature = "async-v2", feature = "rdf")))]
impl Default for InstanceExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn instances on work-stealing executor
///
/// For Pattern 12 (no sync): spawns all instances and returns immediately
/// For Pattern 13-14 (sync): spawns all instances and creates sync gate
#[cfg(all(feature = "async-v2", feature = "rdf"))]
pub async fn spawn_instances(
    executor: &WorkStealingExecutor,
    instance_tracker: &InstanceTracker,
    sync_gate: Option<&SyncGate>,
    case_id: CaseId,
    pattern_id: PatternId,
    instance_set_id: &str,
    instance_count: usize,
    instance_data: Vec<Option<serde_json::Value>>,
) -> WorkflowResult<Vec<String>> {
    let mut instance_ids = Vec::with_capacity(instance_count);

    // Create sync gate if needed (Pattern 13-14)
    let gate_id = if let Some(gate) = sync_gate {
        Some(gate.create_gate(instance_set_id, instance_count).await?)
    } else {
        None
    };

    // Create and spawn all instances
    for i in 0..instance_count {
        let metadata = InstanceMetadata {
            instance_id: i,
            case_id,
            pattern_id,
            status: InstanceStatus::Pending,
            input_data: instance_data.get(i).cloned().unwrap_or(None),
            output_data: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };

        let instance_id = instance_tracker
            .create_instance(instance_set_id, &metadata)
            .await?;

        instance_ids.push(instance_id.clone());

        // Spawn instance on executor
        let tracker = Arc::new(instance_tracker.clone());
        let gate_opt = gate_id.clone();
        let gate_ref = sync_gate.map(|g| Arc::new(g.clone()));
        let inst_id = instance_id.clone();

        executor.spawn(async move {
            // Execute instance (simplified - in real implementation would call task executor)
            if let Err(e) = execute_instance_impl(&tracker, &inst_id).await {
                tracing::error!("Instance {} failed: {:?}", inst_id, e);
                let _ = tracker
                    .update_status(&inst_id, InstanceStatus::Failed)
                    .await;
                return;
            }

            // Update sync gate if present
            if let (Some(gate_id), Some(gate)) = (gate_opt, gate_ref) {
                match gate.increment_and_check(&gate_id).await {
                    Ok(should_proceed) => {
                        if should_proceed {
                            tracing::info!("Sync gate {} completed - all instances done", gate_id);
                            // In real implementation, would trigger next activities here
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to update sync gate: {:?}", e);
                    }
                }
            }
        });
    }

    Ok(instance_ids)
}

#[cfg(not(all(feature = "async-v2", feature = "rdf")))]
pub async fn spawn_instances(
    _instance_count: usize,
) -> WorkflowResult<Vec<String>> {
    Ok(Vec::new())
}

/// Execute single instance (simplified implementation)
#[cfg(feature = "rdf")]
async fn execute_instance_impl(
    tracker: &InstanceTracker,
    instance_id: &str,
) -> WorkflowResult<()> {
    // Update status to Running
    tracker
        .update_status(instance_id, InstanceStatus::Running)
        .await?;

    // Simulate instance execution
    // In real implementation, would:
    // 1. Execute task logic
    // 2. Allocate resources
    // 3. Call worklets if needed
    // 4. Handle errors and retries
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Update status to Completed
    tracker
        .update_status(instance_id, InstanceStatus::Completed)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(feature = "async-v2", feature = "rdf"))]
    #[tokio::test]
    async fn test_spawn_instances_no_sync() {
        use crate::case::CaseId;
        use oxigraph::store::Store;
        use tokio::sync::RwLock;

        let executor = Arc::new(WorkStealingExecutor::new(2));
        let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));
        let tracker = InstanceTracker::new(rdf_store.clone());

        let case_id = CaseId::from("test-case");
        let pattern_id = PatternId(12);
        let instance_set_id = format!("{}:pattern_12:instances", case_id);

        // Create instance set
        tracker
            .create_instance_set(case_id, pattern_id, 5)
            .await
            .unwrap();

        // Spawn instances without sync
        let instance_ids = spawn_instances(
            &executor,
            &tracker,
            None, // No sync gate
            case_id,
            pattern_id,
            &instance_set_id,
            5,
            vec![None; 5],
        )
        .await
        .unwrap();

        assert_eq!(instance_ids.len(), 5);

        // Wait for instances to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        executor.shutdown().await;
    }

    #[cfg(all(feature = "async-v2", feature = "rdf"))]
    #[tokio::test]
    async fn test_spawn_instances_with_sync() {
        use crate::case::CaseId;
        use oxigraph::store::Store;
        use tokio::sync::RwLock;

        let executor = Arc::new(WorkStealingExecutor::new(2));
        let rdf_store = Arc::new(RwLock::new(Store::new().unwrap()));
        let tracker = InstanceTracker::new(rdf_store.clone());
        let gate = SyncGate::new(rdf_store);

        let case_id = CaseId::from("test-case-sync");
        let pattern_id = PatternId(13);
        let instance_set_id = format!("{}:pattern_13:instances", case_id);

        // Create instance set
        tracker
            .create_instance_set(case_id, pattern_id, 3)
            .await
            .unwrap();

        // Spawn instances with sync
        let instance_ids = spawn_instances(
            &executor,
            &tracker,
            Some(&gate), // With sync gate
            case_id,
            pattern_id,
            &instance_set_id,
            3,
            vec![None; 3],
        )
        .await
        .unwrap();

        assert_eq!(instance_ids.len(), 3);

        // Wait for instances to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check sync gate status
        let gate_id = format!("{}:sync_gate", instance_set_id);
        let (completed, target) = gate.get_progress(&gate_id).await.unwrap();
        assert_eq!(completed, target);

        executor.shutdown().await;
    }
}
