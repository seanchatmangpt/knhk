#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Worklets System - Dynamic Workflow Adaptation
//!
//! Implements YAWL-style worklets for dynamic workflow changes at runtime.
//! Worklets are reusable workflow fragments that can replace or extend
//! workflow tasks dynamically based on context, exceptions, or rules.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

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
pub struct WorkletRepository {
    /// Stored worklets
    worklets: Arc<RwLock<HashMap<WorkletId, Worklet>>>,
    /// Worklets by exception type
    exception_index: Arc<RwLock<HashMap<String, Vec<WorkletId>>>>,
    /// Worklets by tag
    tag_index: Arc<RwLock<HashMap<String, Vec<WorkletId>>>>,
}

impl WorkletRepository {
    /// Create new worklet repository
    pub fn new() -> Self {
        Self {
            worklets: Arc::new(RwLock::new(HashMap::new())),
            exception_index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
        }
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
        // Simple condition evaluation (in production, would use proper expression evaluator)
        // For now, check if condition is a context variable name and if it exists
        if context.variables.contains_key(&rule.condition) || rule.condition == "true" {
            Ok(true)
        } else if rule.condition == "false" {
            Ok(false)
        } else {
            // Default: evaluate as boolean expression
            // FUTURE: Implement proper expression evaluation
            Ok(false)
        }
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
    pub async fn execute_worklet(
        &self,
        worklet_id: WorkletId,
        context: PatternExecutionContext,
    ) -> WorkflowResult<PatternExecutionResult> {
        let _worklet = self.repository.get(worklet_id).await?;

        // Execute worklet workflow specification
        // This is a simplified implementation - in production would execute
        // the full workflow spec through the workflow engine
        Ok(PatternExecutionResult {
            success: true,
            next_state: Some(format!("worklet:{}:completed", worklet_id.0)),
            next_activities: Vec::new(),
            variables: context.variables,
            updates: None,
            cancel_activities: Vec::new(),
            terminates: false,
        })
    }

    /// Handle exception with worklet
    pub async fn handle_exception(
        &self,
        exception_type: &str,
        context: PatternExecutionContext,
    ) -> WorkflowResult<Option<PatternExecutionResult>> {
        // Select appropriate worklet for exception
        if let Some(worklet_id) = self
            .repository
            .select_worklet(&context, Some(exception_type))
            .await?
        {
            let result = self.execute_worklet(worklet_id, context).await?;
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
                start_condition: None,
                end_condition: None,
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
                start_condition: None,
                end_condition: None,
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
