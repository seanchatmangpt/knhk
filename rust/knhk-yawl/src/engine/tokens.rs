//! Token (data flow) management for YAWL workflows
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: O (Observation plane - tokens are first-class data)
//! - Covenant: Covenant 1 (Turtle is definition and cause)
//! - Validation: Token lifecycle tracked via telemetry

use crate::engine::messages::{TaskId, TokenId, WorkflowId};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Token lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenLifecycle {
    /// Token created
    Created,
    /// Token being processed by task
    InTransit,
    /// Token consumed by task
    Consumed,
    /// Token cancelled
    Cancelled,
}

/// Token representing data flow in workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Unique token identifier
    pub id: TokenId,

    /// Workflow this token belongs to
    pub workflow_id: WorkflowId,

    /// Task that produced this token
    pub parent_task: Option<TaskId>,

    /// Task that will consume this token
    pub target_task: Option<TaskId>,

    /// Token data payload
    pub data: serde_json::Value,

    /// Current lifecycle state
    pub lifecycle: TokenLifecycle,

    /// When token was created
    pub created_at: DateTime<Utc>,

    /// When token was last updated
    pub updated_at: DateTime<Utc>,
}

impl Token {
    /// Create new token
    pub fn new(
        workflow_id: WorkflowId,
        parent_task: Option<TaskId>,
        data: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: TokenId::new(),
            workflow_id,
            parent_task,
            target_task: None,
            data,
            lifecycle: TokenLifecycle::Created,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark token as in transit to target task
    pub fn set_target(&mut self, target_task: TaskId) {
        self.target_task = Some(target_task);
        self.lifecycle = TokenLifecycle::InTransit;
        self.updated_at = Utc::now();
    }

    /// Mark token as consumed
    pub fn consume(&mut self) {
        self.lifecycle = TokenLifecycle::Consumed;
        self.updated_at = Utc::now();
    }

    /// Mark token as cancelled
    pub fn cancel(&mut self) {
        self.lifecycle = TokenLifecycle::Cancelled;
        self.updated_at = Utc::now();
    }
}

/// Concurrent token manager
pub struct TokenManager {
    /// Active tokens indexed by ID
    tokens: Arc<DashMap<TokenId, Token>>,

    /// Tokens indexed by workflow
    workflow_tokens: Arc<DashMap<WorkflowId, Vec<TokenId>>>,

    /// Tokens indexed by target task (for efficient lookup)
    task_tokens: Arc<DashMap<TaskId, Vec<TokenId>>>,
}

impl TokenManager {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(DashMap::new()),
            workflow_tokens: Arc::new(DashMap::new()),
            task_tokens: Arc::new(DashMap::new()),
        }
    }

    /// Create new token
    #[tracing::instrument(skip(self, data))]
    pub fn create_token(
        &self,
        workflow_id: WorkflowId,
        parent_task: Option<TaskId>,
        data: serde_json::Value,
    ) -> TokenId {
        let token = Token::new(workflow_id, parent_task, data);
        let token_id = token.id;

        // Store token
        self.tokens.insert(token_id, token);

        // Index by workflow
        self.workflow_tokens
            .entry(workflow_id)
            .or_insert_with(Vec::new)
            .push(token_id);

        info!(
            token_id = %token_id,
            workflow_id = %workflow_id,
            parent_task = ?parent_task,
            "Token created"
        );

        token_id
    }

    /// Route token to target task
    #[tracing::instrument(skip(self))]
    pub fn route_token(&self, token_id: TokenId, target_task: TaskId) {
        if let Some(mut token) = self.tokens.get_mut(&token_id) {
            token.set_target(target_task);

            // Index by target task
            self.task_tokens
                .entry(target_task)
                .or_insert_with(Vec::new)
                .push(token_id);

            info!(
                token_id = %token_id,
                target_task = %target_task,
                "Token routed"
            );
        }
    }

    /// Consume token
    #[tracing::instrument(skip(self))]
    pub fn consume_token(&self, token_id: TokenId) {
        if let Some(mut token) = self.tokens.get_mut(&token_id) {
            let target_task = token.target_task;
            token.consume();

            // Remove from task index
            if let Some(task_id) = target_task {
                if let Some(mut tokens) = self.task_tokens.get_mut(&task_id) {
                    tokens.retain(|&id| id != token_id);
                }
            }

            info!(token_id = %token_id, "Token consumed");
        }
    }

    /// Get tokens for a specific task
    pub fn get_task_tokens(&self, task_id: TaskId) -> Vec<Token> {
        if let Some(token_ids) = self.task_tokens.get(&task_id) {
            token_ids
                .iter()
                .filter_map(|id| self.tokens.get(id).map(|t| t.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get token by ID
    pub fn get_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens.get(&token_id).map(|t| t.clone())
    }

    /// Get all tokens for workflow
    pub fn get_workflow_tokens(&self, workflow_id: WorkflowId) -> Vec<Token> {
        if let Some(token_ids) = self.workflow_tokens.get(&workflow_id) {
            token_ids
                .iter()
                .filter_map(|id| self.tokens.get(id).map(|t| t.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get active tokens count for workflow
    pub fn get_active_token_count(&self, workflow_id: WorkflowId) -> usize {
        self.get_workflow_tokens(workflow_id)
            .iter()
            .filter(|t| !matches!(t.lifecycle, TokenLifecycle::Consumed | TokenLifecycle::Cancelled))
            .count()
    }

    /// Cancel all tokens for workflow
    #[tracing::instrument(skip(self))]
    pub fn cancel_workflow_tokens(&self, workflow_id: WorkflowId) {
        if let Some(token_ids) = self.workflow_tokens.get(&workflow_id) {
            for token_id in token_ids.iter() {
                if let Some(mut token) = self.tokens.get_mut(token_id) {
                    token.cancel();
                }
            }

            debug!(workflow_id = %workflow_id, "All workflow tokens cancelled");
        }
    }
}

impl Default for TokenManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let manager = TokenManager::new();
        let wf_id = WorkflowId::new();
        let task_id = TaskId::new();

        let token_id = manager.create_token(
            wf_id,
            Some(task_id),
            serde_json::json!({"value": 42}),
        );

        let token = manager.get_token(token_id).unwrap();
        assert_eq!(token.workflow_id, wf_id);
        assert_eq!(token.parent_task, Some(task_id));
        assert_eq!(token.lifecycle, TokenLifecycle::Created);
    }

    #[test]
    fn test_token_routing() {
        let manager = TokenManager::new();
        let wf_id = WorkflowId::new();
        let source_task = TaskId::new();
        let target_task = TaskId::new();

        let token_id = manager.create_token(
            wf_id,
            Some(source_task),
            serde_json::json!({}),
        );

        manager.route_token(token_id, target_task);

        let token = manager.get_token(token_id).unwrap();
        assert_eq!(token.target_task, Some(target_task));
        assert_eq!(token.lifecycle, TokenLifecycle::InTransit);

        let task_tokens = manager.get_task_tokens(target_task);
        assert_eq!(task_tokens.len(), 1);
        assert_eq!(task_tokens[0].id, token_id);
    }

    #[test]
    fn test_token_consumption() {
        let manager = TokenManager::new();
        let wf_id = WorkflowId::new();
        let target_task = TaskId::new();

        let token_id = manager.create_token(wf_id, None, serde_json::json!({}));
        manager.route_token(token_id, target_task);
        manager.consume_token(token_id);

        let token = manager.get_token(token_id).unwrap();
        assert_eq!(token.lifecycle, TokenLifecycle::Consumed);

        // Should be removed from task index
        let task_tokens = manager.get_task_tokens(target_task);
        assert_eq!(task_tokens.len(), 0);
    }

    #[test]
    fn test_workflow_token_cancellation() {
        let manager = TokenManager::new();
        let wf_id = WorkflowId::new();

        let token1 = manager.create_token(wf_id, None, serde_json::json!({}));
        let token2 = manager.create_token(wf_id, None, serde_json::json!({}));

        manager.cancel_workflow_tokens(wf_id);

        assert_eq!(
            manager.get_token(token1).unwrap().lifecycle,
            TokenLifecycle::Cancelled
        );
        assert_eq!(
            manager.get_token(token2).unwrap().lifecycle,
            TokenLifecycle::Cancelled
        );
        assert_eq!(manager.get_active_token_count(wf_id), 0);
    }
}
