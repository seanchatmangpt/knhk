//! Multiple Instance Patterns (12-15) - Phase 3 Implementation
//!
//! True parallel execution using work-stealing executor and RDF-based instance tracking.
//! This module provides the full implementation with:
//! - Pattern 12: MI Without Synchronization (fire-and-forget)
//! - Pattern 13: MI Design-Time Knowledge (known count, synchronized)
//! - Pattern 14: MI Runtime Knowledge (runtime count, synchronized)
//! - Pattern 15: MI Dynamic (unbounded, dynamic creation)

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::mi::{
    InstanceExecutor, InstanceMetadata, InstanceStatus, InstanceTracker, SyncGate,
};
use crate::patterns::{
    PatternExecutionContext, PatternExecutionResult, PatternExecutor, PatternId,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

#[cfg(all(feature = "async-v2", feature = "rdf"))]
use crate::concurrency::WorkStealingExecutor;
#[cfg(all(feature = "async-v2", feature = "rdf"))]
use oxigraph::store::Store;
#[cfg(all(feature = "async-v2", feature = "rdf"))]
use tokio::sync::RwLock;

/// Extended context for MI pattern execution with executor and RDF store
#[cfg(all(feature = "async-v2", feature = "rdf"))]
pub struct MIExecutionContext {
    pub base: PatternExecutionContext,
    pub executor: Arc<WorkStealingExecutor>,
    pub rdf_store: Arc<RwLock<Store>>,
}

/// Pattern 12: Multiple Instances Without Synchronization
///
/// Spawns multiple instances in parallel without waiting for completion.
/// Instances execute independently on the work-stealing executor.
pub struct MultipleInstanceWithoutSyncV2;

impl PatternExecutor for MultipleInstanceWithoutSyncV2 {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Extract instance count from context variables
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        // Create metadata for instance spawning
        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "no_sync".to_string());
        variables.insert("instance_count".to_string(), instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "false".to_string());
        variables.insert("mi_pattern".to_string(), "12".to_string());
        variables.insert(
            "mi_instance_set_id".to_string(),
            format!("{}:pattern_12:instances", ctx.case_id),
        );

        // Create update JSON with instance spawning metadata
        let updates = json!({
            "pattern": 12,
            "mi_mode": "no_sync",
            "instance_count": instance_count,
            "wait_for_completion": false,
            "requires_executor": true,
            "requires_rdf": true,
            "instance_set_id": format!("{}:pattern_12:instances", ctx.case_id),
        });

        PatternExecutionResult {
            success: true,
            next_state: Some(format!("pattern:{}:spawning", 12)),
            next_activities: Vec::new(), // Instances spawned asynchronously
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

#[cfg(all(feature = "async-v2", feature = "rdf"))]
impl MultipleInstanceWithoutSyncV2 {
    /// Execute with full async support and executor integration
    pub async fn execute_async(&self, ctx: &MIExecutionContext) -> WorkflowResult<PatternExecutionResult> {
        let instance_count: usize = ctx
            .base
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let instance_set_id = format!("{}:pattern_12:instances", ctx.base.case_id);
        let pattern_id = PatternId(12);

        // Create instance tracker and spawn instances
        let tracker = InstanceTracker::new(ctx.rdf_store.clone());

        // Create instance set in RDF
        tracker
            .create_instance_set(ctx.base.case_id, pattern_id, instance_count)
            .await?;

        // Spawn instances on work-stealing executor (no sync gate for Pattern 12)
        let instance_ids = crate::patterns::mi::executor_integration::spawn_instances(
            &ctx.executor,
            &tracker,
            None, // No synchronization for Pattern 12
            ctx.base.case_id,
            pattern_id,
            &instance_set_id,
            instance_count,
            vec![None; instance_count], // No input data by default
        )
        .await?;

        // Return result with instance IDs
        let mut variables = ctx.base.variables.clone();
        variables.insert("instances_spawned".to_string(), instance_ids.len().to_string());
        variables.insert("instance_set_id".to_string(), instance_set_id.clone());

        let updates = json!({
            "pattern": 12,
            "instance_set_id": instance_set_id,
            "instances_spawned": instance_ids.len(),
            "instance_ids": instance_ids,
        });

        Ok(PatternExecutionResult {
            success: true,
            next_state: Some("pattern:12:spawned".to_string()),
            next_activities: Vec::new(), // Continue immediately without waiting
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        })
    }
}

/// Pattern 13: Multiple Instances With a Priori Design-Time Knowledge
///
/// Spawns a known number of instances (determined at design time) and waits for all to complete.
pub struct MultipleInstanceDesignTimeV2;

impl PatternExecutor for MultipleInstanceDesignTimeV2 {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "design_time".to_string());
        variables.insert("instance_count".to_string(), instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "true".to_string());
        variables.insert("mi_pattern".to_string(), "13".to_string());
        variables.insert(
            "mi_instance_set_id".to_string(),
            format!("{}:pattern_13:instances", ctx.case_id),
        );

        let updates = json!({
            "pattern": 13,
            "mi_mode": "design_time",
            "instance_count": instance_count,
            "wait_for_completion": true,
            "requires_executor": true,
            "requires_rdf": true,
            "requires_sync_gate": true,
            "instance_set_id": format!("{}:pattern_13:instances", ctx.case_id),
        });

        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:13:spawning".to_string()),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

#[cfg(all(feature = "async-v2", feature = "rdf"))]
impl MultipleInstanceDesignTimeV2 {
    /// Execute with synchronization gate
    pub async fn execute_async(&self, ctx: &MIExecutionContext) -> WorkflowResult<PatternExecutionResult> {
        let instance_count: usize = ctx
            .base
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let instance_set_id = format!("{}:pattern_13:instances", ctx.base.case_id);
        let pattern_id = PatternId(13);

        // Create instance tracker and sync gate
        let tracker = InstanceTracker::new(ctx.rdf_store.clone());
        let sync_gate = SyncGate::new(ctx.rdf_store.clone());

        // Create instance set in RDF
        tracker
            .create_instance_set(ctx.base.case_id, pattern_id, instance_count)
            .await?;

        // Spawn instances with sync gate (Pattern 13 waits for all)
        let instance_ids = crate::patterns::mi::executor_integration::spawn_instances(
            &ctx.executor,
            &tracker,
            Some(&sync_gate), // WITH synchronization for Pattern 13
            ctx.base.case_id,
            pattern_id,
            &instance_set_id,
            instance_count,
            vec![None; instance_count],
        )
        .await?;

        let sync_gate_id = format!("{}:sync_gate", instance_set_id);

        let mut variables = ctx.base.variables.clone();
        variables.insert("instances_spawned".to_string(), instance_ids.len().to_string());
        variables.insert("instance_set_id".to_string(), instance_set_id.clone());
        variables.insert("sync_gate_id".to_string(), sync_gate_id.clone());

        let updates = json!({
            "pattern": 13,
            "instance_set_id": instance_set_id,
            "sync_gate_id": sync_gate_id,
            "instances_spawned": instance_ids.len(),
            "instance_ids": instance_ids,
            "synchronization": "required",
        });

        Ok(PatternExecutionResult {
            success: true,
            next_state: Some("pattern:13:executing".to_string()),
            next_activities: Vec::new(), // Will proceed when sync gate opens
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        })
    }
}

/// Pattern 14: Multiple Instances With a Priori Runtime Knowledge
///
/// Spawns instances based on runtime case data (e.g., array length).
pub struct MultipleInstanceRuntimeV2;

impl PatternExecutor for MultipleInstanceRuntimeV2 {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        // Try to determine instance count from runtime data
        let instance_count: usize = ctx
            .variables
            .get("instance_count")
            .and_then(|v| v.parse().ok())
            .or_else(|| {
                // Parse runtime instance data array
                ctx.variables
                    .get("runtime_instance_data")
                    .and_then(|data| {
                        serde_json::from_str::<Vec<serde_json::Value>>(data)
                            .ok()
                            .map(|arr| arr.len())
                    })
            })
            .unwrap_or(1);

        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "runtime".to_string());
        variables.insert("instance_count".to_string(), instance_count.to_string());
        variables.insert("mi_wait_for_completion".to_string(), "true".to_string());
        variables.insert("mi_pattern".to_string(), "14".to_string());
        variables.insert(
            "mi_instance_set_id".to_string(),
            format!("{}:pattern_14:instances", ctx.case_id),
        );

        let updates = json!({
            "pattern": 14,
            "mi_mode": "runtime",
            "instance_count": instance_count,
            "wait_for_completion": true,
            "requires_executor": true,
            "requires_rdf": true,
            "requires_sync_gate": true,
            "instance_set_id": format!("{}:pattern_14:instances", ctx.case_id),
        });

        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:14:spawning".to_string()),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

#[cfg(all(feature = "async-v2", feature = "rdf"))]
impl MultipleInstanceRuntimeV2 {
    /// Execute with runtime-determined instance count
    pub async fn execute_async(&self, ctx: &MIExecutionContext) -> WorkflowResult<PatternExecutionResult> {
        // Parse runtime instance data
        let instance_data: Vec<Option<serde_json::Value>> = ctx
            .base
            .variables
            .get("runtime_instance_data")
            .and_then(|data| serde_json::from_str::<Vec<serde_json::Value>>(data).ok())
            .map(|arr| arr.into_iter().map(Some).collect())
            .unwrap_or_else(|| vec![None]);

        let instance_count = instance_data.len();
        let instance_set_id = format!("{}:pattern_14:instances", ctx.base.case_id);
        let pattern_id = PatternId(14);

        // Create instance tracker and sync gate
        let tracker = InstanceTracker::new(ctx.rdf_store.clone());
        let sync_gate = SyncGate::new(ctx.rdf_store.clone());

        // Create instance set in RDF
        tracker
            .create_instance_set(ctx.base.case_id, pattern_id, instance_count)
            .await?;

        // Spawn instances with input data from runtime array
        let instance_ids = crate::patterns::mi::executor_integration::spawn_instances(
            &ctx.executor,
            &tracker,
            Some(&sync_gate),
            ctx.base.case_id,
            pattern_id,
            &instance_set_id,
            instance_count,
            instance_data,
        )
        .await?;

        let sync_gate_id = format!("{}:sync_gate", instance_set_id);

        let mut variables = ctx.base.variables.clone();
        variables.insert("instances_spawned".to_string(), instance_ids.len().to_string());
        variables.insert("instance_set_id".to_string(), instance_set_id.clone());
        variables.insert("sync_gate_id".to_string(), sync_gate_id.clone());

        let updates = json!({
            "pattern": 14,
            "instance_set_id": instance_set_id,
            "sync_gate_id": sync_gate_id,
            "instances_spawned": instance_ids.len(),
            "instance_ids": instance_ids,
            "runtime_determined": true,
            "synchronization": "required",
        });

        Ok(PatternExecutionResult {
            success: true,
            next_state: Some("pattern:14:executing".to_string()),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        })
    }
}

/// Pattern 15: Multiple Instances Without a Priori Runtime Knowledge
///
/// Dynamically spawns instances as needed (unbounded count).
pub struct MultipleInstanceDynamicV2;

impl PatternExecutor for MultipleInstanceDynamicV2 {
    fn execute(&self, ctx: &PatternExecutionContext) -> PatternExecutionResult {
        let initial_instance_count: usize = ctx
            .variables
            .get("initial_instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let allow_dynamic_spawning = ctx
            .variables
            .get("allow_dynamic_spawning")
            .map(|v| v == "true")
            .unwrap_or(true);

        let mut variables = ctx.variables.clone();
        variables.insert("mi_mode".to_string(), "dynamic".to_string());
        variables.insert(
            "instance_count".to_string(),
            initial_instance_count.to_string(),
        );
        variables.insert("mi_wait_for_completion".to_string(), "true".to_string());
        variables.insert(
            "allow_dynamic_spawning".to_string(),
            allow_dynamic_spawning.to_string(),
        );
        variables.insert("mi_pattern".to_string(), "15".to_string());
        variables.insert(
            "mi_instance_set_id".to_string(),
            format!("{}:pattern_15:instances", ctx.case_id),
        );

        let updates = json!({
            "pattern": 15,
            "mi_mode": "dynamic",
            "initial_instance_count": initial_instance_count,
            "wait_for_completion": true,
            "allow_dynamic_spawning": allow_dynamic_spawning,
            "requires_executor": true,
            "requires_rdf": true,
            "requires_sync_gate": true,
            "instance_set_id": format!("{}:pattern_15:instances", ctx.case_id),
        });

        PatternExecutionResult {
            success: true,
            next_state: Some("pattern:15:spawning".to_string()),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        }
    }
}

#[cfg(all(feature = "async-v2", feature = "rdf"))]
impl MultipleInstanceDynamicV2 {
    /// Execute with dynamic instance creation
    pub async fn execute_async(&self, ctx: &MIExecutionContext) -> WorkflowResult<PatternExecutionResult> {
        let initial_instance_count: usize = ctx
            .base
            .variables
            .get("initial_instance_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let instance_set_id = format!("{}:pattern_15:instances", ctx.base.case_id);
        let pattern_id = PatternId(15);

        // Create instance tracker and sync gate
        let tracker = InstanceTracker::new(ctx.rdf_store.clone());
        let sync_gate = SyncGate::new(ctx.rdf_store.clone());

        // Create instance set with initial count (can grow dynamically)
        tracker
            .create_instance_set(ctx.base.case_id, pattern_id, initial_instance_count)
            .await?;

        // Spawn initial instances
        let instance_ids = if initial_instance_count > 0 {
            crate::patterns::mi::executor_integration::spawn_instances(
                &ctx.executor,
                &tracker,
                Some(&sync_gate),
                ctx.base.case_id,
                pattern_id,
                &instance_set_id,
                initial_instance_count,
                vec![None; initial_instance_count],
            )
            .await?
        } else {
            Vec::new()
        };

        let sync_gate_id = format!("{}:sync_gate", instance_set_id);

        let mut variables = ctx.base.variables.clone();
        variables.insert(
            "instances_spawned".to_string(),
            instance_ids.len().to_string(),
        );
        variables.insert("instance_set_id".to_string(), instance_set_id.clone());
        variables.insert("sync_gate_id".to_string(), sync_gate_id.clone());
        variables.insert("allow_dynamic_spawning".to_string(), "true".to_string());

        let updates = json!({
            "pattern": 15,
            "instance_set_id": instance_set_id,
            "sync_gate_id": sync_gate_id,
            "instances_spawned": instance_ids.len(),
            "instance_ids": instance_ids,
            "dynamic": true,
            "can_add_instances": true,
            "synchronization": "required",
        });

        Ok(PatternExecutionResult {
            success: true,
            next_state: Some("pattern:15:executing".to_string()),
            next_activities: Vec::new(),
            variables,
            updates: Some(updates),
            cancel_activities: Vec::new(),
            terminates: false,
        })
    }
}

/// Pattern factory functions (V2 implementations)
pub fn create_pattern_12_v2() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(12), Box::new(MultipleInstanceWithoutSyncV2))
}

pub fn create_pattern_13_v2() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(13), Box::new(MultipleInstanceDesignTimeV2))
}

pub fn create_pattern_14_v2() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(14), Box::new(MultipleInstanceRuntimeV2))
}

pub fn create_pattern_15_v2() -> (PatternId, Box<dyn PatternExecutor>) {
    (PatternId(15), Box::new(MultipleInstanceDynamicV2))
}
