#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Worklets System - Dynamic Workflow Adaptation
//!
//! Implements YAWL-style worklets for dynamic workflow changes at runtime.
//! Worklets are reusable workflow fragments that can replace or extend
//! workflow tasks dynamically based on context, exceptions, or rules.
//!
//! # TRIZ Patterns Applied
//!
//! - TRIZ Principle 24: Intermediary - RDR selection plan
//! - TRIZ Principle 19: Periodic Action - Periodic repository sync
/// WorkletExecutionBackend implementation for WorkflowEngine
pub mod backend_impl;
/// Ripple Down Rules (RDR) engine for worklet selection
pub mod rdr;
/// YAWL worklet service implementation
pub mod yawl_worklet;

// # YAWL Worklet System
//
// - `yawl_worklet.rs`: YAWL worklet service with RDR selection and TRIZ enhancements

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use async_trait::async_trait;
pub use rdr::{ExceptionContext, RDREngine, RDRRule, RDRSelectionPlan};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use uuid::Uuid;
pub use yawl_worklet::{RdrNode, RdrTree, WorkletContext, YawlWorkletService};

/// Worklet identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkletId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl WorkletId {
    /// Create new worklet ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WorkletId {
    fn default() -> Self {
        Self::new()
    }
}

/// Worklet selection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkletRule {
    /// Rule identifier
    pub id: String,
    /// Rule name
    pub name: String,
    /// Condition expression (evaluated against context)
    pub condition: String,
    /// Worklet ID to select if condition matches
    pub worklet_id: WorkletId,
    /// Priority (higher = evaluated first)
    pub priority: u32,
}

/// Worklet metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkletMetadata {
    /// Worklet identifier
    pub id: WorkletId,
    /// Worklet name
    pub name: String,
    /// Worklet description
    pub description: String,
    /// Worklet version
    pub version: String,
    /// Applicable exception types
    pub exception_types: Vec<String>,
    /// Required context variables
    pub required_context: Vec<String>,
    /// Pattern IDs used in this worklet
    pub pattern_ids: Vec<PatternId>,
    /// Tags for discovery
    pub tags: Vec<String>,
}

/// Worklet (reusable workflow fragment)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worklet {
    /// Worklet metadata
    pub metadata: WorkletMetadata,
    /// Workflow specification (sub-workflow)
    pub workflow_spec: WorkflowSpec,
    /// Selection rules
    pub rules: Vec<WorkletRule>,
}

/// Worklet repository
///
/// TRIZ Principle 19: Periodic Action - Periodic reconciliation instead of continuous monitoring
pub struct WorkletRepository {
    /// Stored worklets
    worklets: Arc<RwLock<HashMap<WorkletId, Worklet>>>,
    /// Worklets by exception type
    exception_index: Arc<RwLock<HashMap<String, Vec<WorkletId>>>>,
    /// Worklets by tag
    tag_index: Arc<RwLock<HashMap<String, Vec<WorkletId>>>>,
    /// RDR engine for rule-based selection (TRIZ Principle 24: Intermediary)
    rdr_engine: Arc<RwLock<RDREngine>>,
    /// Last reconciliation time (TRIZ Principle 19: Periodic Action)
    last_reconciliation: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl WorkletRepository {
    /// Create new worklet repository
    pub fn new() -> Self {
        Self {
            worklets: Arc::new(RwLock::new(HashMap::new())),
            exception_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            rdr_engine: Arc::new(RwLock::new(RDREngine::new())),
            last_reconciliation: Arc::new(RwLock::new(None)),
        }
    }

    /// Start periodic reconciliation (TRIZ Principle 19: Periodic Action)
    ///
    /// Reconciles worklet repository state periodically instead of continuously
    pub fn start_periodic_reconciliation(&self, interval_secs: u64) {
        let worklets = self.worklets.clone();
        let exception_index = self.exception_index.clone();
        let tag_index = self.tag_index.clone();
        let last_reconciliation = self.last_reconciliation.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;

                // Reconcile indices
                let mut new_exception_index: HashMap<String, Vec<WorkletId>> = HashMap::new();
                let mut new_tag_index: HashMap<String, Vec<WorkletId>> = HashMap::new();

                let worklets_read = worklets.read().await;
                for (worklet_id, worklet) in worklets_read.iter() {
                    // Rebuild exception index
                    for exception_type in &worklet.metadata.exception_types {
                        new_exception_index
                            .entry(exception_type.clone())
                            .or_insert_with(Vec::new)
                            .push(*worklet_id);
                    }

                    // Rebuild tag index
                    for tag in &worklet.metadata.tags {
                        new_tag_index
                            .entry(tag.clone())
                            .or_insert_with(Vec::new)
                            .push(*worklet_id);
                    }
                }
                drop(worklets_read);

                // Update indices
                *exception_index.write().await = new_exception_index;
                *tag_index.write().await = new_tag_index;

                // Update reconciliation time
                *last_reconciliation.write().await = Some(chrono::Utc::now());

                tracing::debug!("Worklet repository reconciled");
            }
        });
    }

    /// Get RDR engine
    pub async fn get_rdr_engine(&self) -> Arc<RwLock<RDREngine>> {
        self.rdr_engine.clone()
    }

    /// Select worklet using RDR (TRIZ Principle 24: Intermediary)
    pub async fn select_worklet_rdr(
        &self,
        context: &rdr::ExceptionContext,
    ) -> WorkflowResult<Option<WorkletId>> {
        let rdr = self.rdr_engine.read().await;
        let plan = rdr.select_worklet(context)?;
        // Find worklet by ID string in repository
        if let Some(worklet_id_str) = plan.selected_worklet {
            let worklets = self.worklets.read().await;
            for (id, worklet) in worklets.iter() {
                if worklet.metadata.name == worklet_id_str || id.0.to_string() == worklet_id_str {
                    return Ok(Some(*id));
                }
            }
        }
        Ok(None)
    }

    /// Register a worklet
    pub async fn register(&self, worklet: Worklet) -> WorkflowResult<()> {
        let mut worklets = self.worklets.write().await;
        let worklet_id = worklet.metadata.id;

        // Index by exception types
        let mut exception_index = self.exception_index.write().await;
        for exception_type in &worklet.metadata.exception_types {
            exception_index
                .entry(exception_type.clone())
                .or_insert_with(Vec::new)
                .push(worklet_id);
        }

        // Index by tags
        let mut tag_index = self.tag_index.write().await;
        for tag in &worklet.metadata.tags {
            tag_index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(worklet_id);
        }

        worklets.insert(worklet_id, worklet);
        Ok(())
    }

    /// Get worklet by ID
    pub async fn get(&self, worklet_id: WorkletId) -> WorkflowResult<Worklet> {
        let worklets = self.worklets.read().await;
        worklets.get(&worklet_id).cloned().ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Worklet {} not found", worklet_id.0))
        })
    }

    /// Find worklets by exception type
    pub async fn find_by_exception(&self, exception_type: &str) -> Vec<WorkletId> {
        let exception_index = self.exception_index.read().await;
        exception_index
            .get(exception_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Find worklets by tag
    pub async fn find_by_tag(&self, tag: &str) -> Vec<WorkletId> {
        let tag_index = self.tag_index.read().await;
        tag_index.get(tag).cloned().unwrap_or_default()
    }

    /// Select worklet based on context and rules
    pub async fn select_worklet(
        &self,
        context: &PatternExecutionContext,
        exception_type: Option<&str>,
    ) -> WorkflowResult<Option<WorkletId>> {
        let worklets = self.worklets.read().await;

        // First, try to find worklets by exception type
        let candidate_ids = if let Some(exception_type) = exception_type {
            self.find_by_exception(exception_type).await
        } else {
            // If no exception, get all worklets
            worklets.keys().copied().collect()
        };

        // Evaluate rules for each candidate
        let mut matches: Vec<(WorkletId, u32)> = Vec::new();
        for worklet_id in candidate_ids {
            if let Some(worklet) = worklets.get(&worklet_id) {
                for rule in &worklet.rules {
                    if self.evaluate_rule(rule, context)? {
                        matches.push((worklet_id, rule.priority));
                        break; // Use first matching rule
                    }
                }
            }
        }

        // Sort by priority (descending) and return highest priority match
        matches.sort_by_key(|(_, priority)| std::cmp::Reverse(*priority));
        Ok(matches.first().map(|(id, _)| *id))
    }

    /// Evaluate a worklet selection rule
    fn evaluate_rule(
        &self,
        rule: &WorkletRule,
        context: &PatternExecutionContext,
    ) -> WorkflowResult<bool> {
        // Evaluate condition expression against context
        // Support simple boolean expressions and variable checks

        let condition = rule.condition.trim();

        // Handle simple boolean literals
        if condition == "true" {
            return Ok(true);
        }
        if condition == "false" {
            return Ok(false);
        }

        // Handle variable existence checks: "variable_name"
        if !condition.contains(' ') && !condition.contains('(') {
            return Ok(context.variables.contains_key(condition));
        }

        // Handle simple comparisons: "variable == value" or "variable != value"
        if let Some((left, right)) = condition.split_once("==") {
            let left = left.trim();
            let right = right.trim().trim_matches('"').trim_matches('\'');
            if let Some(value) = context.variables.get(left) {
                return Ok(*value == right);
            }
            return Ok(false);
        }

        if let Some((left, right)) = condition.split_once("!=") {
            let left = left.trim();
            let right = right.trim().trim_matches('"').trim_matches('\'');
            if let Some(value) = context.variables.get(left) {
                return Ok(*value != right);
            }
            return Ok(true); // Variable doesn't exist, so != is true
        }

        // Handle numeric comparisons: "variable > value", "variable < value", etc.
        if let Some((left, right)) = condition.split_once(">") {
            let left = left.trim();
            let right = right.trim();
            if let Some(value) = context.variables.get(left) {
                if let (Ok(left_num), Ok(right_num)) =
                    (value.to_string().parse::<f64>(), right.parse::<f64>())
                {
                    return Ok(left_num > right_num);
                }
            }
            return Ok(false);
        }

        if let Some((left, right)) = condition.split_once("<") {
            let left = left.trim();
            let right = right.trim();
            if let Some(value) = context.variables.get(left) {
                if let (Ok(left_num), Ok(right_num)) =
                    (value.to_string().parse::<f64>(), right.parse::<f64>())
                {
                    return Ok(left_num < right_num);
                }
            }
            return Ok(false);
        }

        // Default: evaluate as boolean expression (simple AND/OR)
        if condition.contains("&&") {
            let parts: Vec<&str> = condition.split("&&").collect();
            for part in parts {
                if !self.evaluate_rule(
                    &WorkletRule {
                        id: rule.id.clone(),
                        name: rule.name.clone(),
                        condition: part.trim().to_string(),
                        worklet_id: rule.worklet_id,
                        priority: rule.priority,
                    },
                    context,
                )? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        if condition.contains("||") {
            let parts: Vec<&str> = condition.split("||").collect();
            for part in parts {
                if self.evaluate_rule(
                    &WorkletRule {
                        id: rule.id.clone(),
                        name: rule.name.clone(),
                        condition: part.trim().to_string(),
                        worklet_id: rule.worklet_id,
                        priority: rule.priority,
                    },
                    context,
                )? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        // Unknown condition format - cannot evaluate
        Err(crate::error::WorkflowError::Internal(format!(
            "Cannot evaluate condition: {}",
            condition
        )))
    }

    /// List all worklets
    pub async fn list(&self) -> Vec<WorkletMetadata> {
        let worklets = self.worklets.read().await;
        worklets.values().map(|w| w.metadata.clone()).collect()
    }

    /// Search worklets
    pub async fn search(&self, query: &str) -> Vec<WorkletMetadata> {
        let worklets = self.worklets.read().await;
        worklets
            .values()
            .filter(|w| {
                w.metadata.name.contains(query)
                    || w.metadata.description.contains(query)
                    || w.metadata.tags.iter().any(|t| t.contains(query))
            })
            .map(|w| w.metadata.clone())
            .collect()
    }
}

impl Default for WorkletRepository {
    fn default() -> Self {
        Self::new()
    }
}

/// Worklet executor
/// Trait for worklet execution backend - breaks circular dependency
///
/// This trait allows WorkletExecutor to execute worklets without directly
/// depending on WorkflowEngine, enabling dependency injection.
#[async_trait::async_trait]
pub trait WorkletExecutionBackend: Send + Sync {
    /// Create a case for a workflow specification
    async fn create_case(
        &self,
        spec_id: crate::parser::WorkflowSpecId,
        data: serde_json::Value,
    ) -> WorkflowResult<crate::case::CaseId>;

    /// Execute a case
    async fn execute_case(&self, case_id: crate::case::CaseId) -> WorkflowResult<()>;

    /// Get case by ID
    async fn get_case(&self, case_id: crate::case::CaseId) -> WorkflowResult<crate::case::Case>;
}

pub struct WorkletExecutor {
    /// Worklet repository
    repository: Arc<WorkletRepository>,
}

impl WorkletExecutor {
    /// Create new worklet executor
    pub fn new(repository: Arc<WorkletRepository>) -> Self {
        Self { repository }
    }

    /// Execute worklet as replacement for a task
    ///
    /// TRIZ Principle 24: Intermediary - Uses execution plan instead of direct engine call
    /// This breaks the circular dependency by using dependency injection
    ///
    /// # Arguments
    /// * `worklet_id` - Worklet identifier
    /// * `context` - Pattern execution context
    /// * `executor_fn` - Function to execute the worklet's workflow spec (dependency injection)
    ///   Takes (spec_id, data) and returns (case_id, case_state)
    pub async fn execute_worklet<B: WorkletExecutionBackend>(
        &self,
        worklet_id: WorkletId,
        context: PatternExecutionContext,
        backend: &B,
    ) -> WorkflowResult<PatternExecutionResult> {
        let worklet = self.repository.get(worklet_id).await?;

        // Convert context variables to JSON Value
        let data = serde_json::json!(context.variables);

        // Create a case for the worklet's workflow spec using backend
        let case_id = backend.create_case(worklet.workflow_spec.id, data).await?;

        // Execute the case using backend
        backend.execute_case(case_id).await?;

        // Get the case to check its state using backend
        let case = backend.get_case(case_id).await?;

        // Convert case result to pattern execution result
        Ok(PatternExecutionResult {
            success: matches!(case.state, crate::case::CaseState::Completed),
            next_state: None,
            next_activities: vec![], // Worklet execution completes the pattern
            variables: context.variables.clone(),
            cancel_activities: vec![],
            terminates: false,
            updates: Some(serde_json::json!({
                "worklet_id": worklet_id.0,
                "case_id": case_id,
                "state": format!("{:?}", case.state),
            })),
        })
    }

    /// Handle exception with worklet
    ///
    /// Uses dependency injection via `WorkletExecutionBackend` trait.
    ///
    /// # Arguments
    /// * `exception_type` - Exception type to handle
    /// * `context` - Pattern execution context
    /// * `backend` - Execution backend (typically the workflow engine)
    pub async fn handle_exception<B: WorkletExecutionBackend>(
        &self,
        exception_type: &str,
        context: PatternExecutionContext,
        backend: &B,
    ) -> WorkflowResult<Option<PatternExecutionResult>> {
        // Select appropriate worklet for exception
        if let Some(worklet_id) = self
            .repository
            .select_worklet(&context, Some(exception_type))
            .await?
        {
            let result = self.execute_worklet(worklet_id, context, backend).await?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::WorkflowSpecId;

    #[tokio::test]
    async fn test_worklet_registration() {
        let repository = WorkletRepository::new();

        let worklet = Worklet {
            metadata: WorkletMetadata {
                id: WorkletId::new(),
                name: "Test Worklet".to_string(),
                description: "Test worklet for exception handling".to_string(),
                version: "1.0.0".to_string(),
                exception_types: vec!["timeout".to_string()],
                required_context: vec![],
                pattern_ids: vec![PatternId(1)],
                tags: vec!["test".to_string(), "exception".to_string()],
            },
            workflow_spec: WorkflowSpec {
                id: WorkflowSpecId::new(),
                name: "Test Worklet Spec".to_string(),
                tasks: HashMap::new(),
                conditions: HashMap::new(),
                flows: Vec::new(),
                start_condition: None,
                end_condition: None,
                source_turtle: None,
            },
            rules: vec![WorkletRule {
                id: "rule1".to_string(),
                name: "Rule 1".to_string(),
                condition: "true".to_string(),
                worklet_id: WorkletId::new(),
                priority: 100,
            }],
        };

        repository.register(worklet).await.unwrap();

        let worklets = repository.list().await;
        assert_eq!(worklets.len(), 1);
    }

    #[tokio::test]
    async fn test_worklet_selection_by_exception() {
        let repository = WorkletRepository::new();

        let worklet = Worklet {
            metadata: WorkletMetadata {
                id: WorkletId::new(),
                name: "Timeout Handler".to_string(),
                description: "Handles timeout exceptions".to_string(),
                version: "1.0.0".to_string(),
                exception_types: vec!["timeout".to_string()],
                required_context: vec![],
                pattern_ids: vec![PatternId(20)],
                tags: vec!["exception".to_string()],
            },
            workflow_spec: WorkflowSpec {
                id: WorkflowSpecId::new(),
                name: "Timeout Handler Spec".to_string(),
                tasks: HashMap::new(),
                conditions: HashMap::new(),
                flows: Vec::new(),
                start_condition: None,
                end_condition: None,
                source_turtle: None,
            },
            rules: vec![WorkletRule {
                id: "rule1".to_string(),
                name: "Timeout Rule".to_string(),
                condition: "true".to_string(),
                worklet_id: WorkletId::new(),
                priority: 100,
            }],
        };

        repository.register(worklet).await.unwrap();

        let context = PatternExecutionContext {
            case_id: crate::case::CaseId::new(),
            workflow_id: WorkflowSpecId::new(),
            variables: HashMap::new(),
            arrived_from: std::collections::HashSet::new(),
            scope_id: String::new(),
        };

        let selected = repository
            .select_worklet(&context, Some("timeout"))
            .await
            .unwrap();
        assert!(selected.is_some());
    }
}
