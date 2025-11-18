//! Petri net state structures
//!
//! YAWL is based on Petri nets; this module represents net states and tokens.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

/// A token in the workflow net
///
/// Represents a unit of control flow in the Petri net.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Token {
    /// Unique token ID
    pub id: String,

    /// Current position (task ID)
    pub position: String,

    /// Token data (workflow variables)
    pub data: HashMap<String, String>,
}

impl Token {
    /// Create a new token at the given position
    #[must_use]
    pub fn new(position: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            position: position.into(),
            data: HashMap::new(),
        }
    }

    /// Move token to a new position
    pub fn move_to(&mut self, new_position: impl Into<String>) {
        self.position = new_position.into();
    }
}

/// An arc in the workflow net
///
/// Represents a directed edge between tasks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Arc {
    /// Source task ID
    pub from_task: String,

    /// Target task ID
    pub to_task: String,

    /// Flow label (optional)
    pub flow_label: Option<String>,
}

impl Arc {
    /// Create a new arc
    #[must_use]
    pub fn new(from_task: impl Into<String>, to_task: impl Into<String>) -> Self {
        Self {
            from_task: from_task.into(),
            to_task: to_task.into(),
            flow_label: None,
        }
    }

    /// Create a new arc with a label
    #[must_use]
    pub fn with_label(
        from_task: impl Into<String>,
        to_task: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            from_task: from_task.into(),
            to_task: to_task.into(),
            flow_label: Some(label.into()),
        }
    }
}

impl fmt::Display for Arc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from_task, self.to_task)?;
        if let Some(ref label) = self.flow_label {
            write!(f, " [{}]", label)?;
        }
        Ok(())
    }
}

/// State of a workflow net at a point in time
///
/// Represents the marking (token distribution) and execution history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetState {
    /// Currently active tasks (task ID -> set of token IDs)
    pub active_tasks: HashMap<String, HashSet<String>>,

    /// All tokens in the net (indexed by token ID)
    pub tokens: HashMap<String, Token>,

    /// Execution history (task ID -> completion timestamps)
    pub history: Vec<HistoryEntry>,
}

/// A single entry in the execution history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HistoryEntry {
    /// Task that was executed
    pub task_id: String,

    /// Timestamp (ISO 8601)
    pub timestamp: String,

    /// Event type (started, completed, failed, etc.)
    pub event_type: HistoryEventType,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Type of history event
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HistoryEventType {
    /// Task started execution
    Started,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

impl NetState {
    /// Create a new empty net state
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_tasks: HashMap::new(),
            tokens: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Add a token to the net at the given position
    pub fn add_token(&mut self, position: impl Into<String>) -> String {
        let token = Token::new(position);
        let token_id = token.id.clone();
        let position_str = token.position.clone();

        self.tokens.insert(token_id.clone(), token);
        self.active_tasks
            .entry(position_str)
            .or_default()
            .insert(token_id.clone());

        token_id
    }

    /// Move a token to a new position
    pub fn move_token(&mut self, token_id: &str, new_position: impl Into<String>) {
        if let Some(token) = self.tokens.get_mut(token_id) {
            let old_position = token.position.clone();
            let new_position_str = new_position.into();

            token.move_to(new_position_str.clone());

            // Update active tasks
            if let Some(active) = self.active_tasks.get_mut(&old_position) {
                active.remove(token_id);
            }
            self.active_tasks
                .entry(new_position_str)
                .or_default()
                .insert(token_id.to_string());
        }
    }

    /// Remove a token from the net
    pub fn remove_token(&mut self, token_id: &str) {
        if let Some(token) = self.tokens.remove(token_id) {
            if let Some(active) = self.active_tasks.get_mut(&token.position) {
                active.remove(token_id);
            }
        }
    }

    /// Add a history entry
    pub fn record_event(
        &mut self,
        task_id: impl Into<String>,
        event_type: HistoryEventType,
        metadata: HashMap<String, String>,
    ) {
        let entry = HistoryEntry {
            task_id: task_id.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type,
            metadata,
        };
        self.history.push(entry);
    }
}

impl Default for NetState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new("task1");
        assert_eq!(token.position, "task1");
        assert!(!token.id.is_empty());
    }

    #[test]
    fn test_token_movement() {
        let mut token = Token::new("task1");
        token.move_to("task2");
        assert_eq!(token.position, "task2");
    }

    #[test]
    fn test_arc_creation() {
        let arc = Arc::new("task1", "task2");
        assert_eq!(arc.from_task, "task1");
        assert_eq!(arc.to_task, "task2");
        assert_eq!(arc.flow_label, None);

        let labeled_arc = Arc::with_label("task1", "task2", "success");
        assert_eq!(labeled_arc.flow_label, Some("success".to_string()));
    }

    #[test]
    fn test_net_state() {
        let mut state = NetState::new();
        let token_id = state.add_token("task1");

        assert_eq!(state.tokens.len(), 1);
        assert!(state.active_tasks.contains_key("task1"));

        state.move_token(&token_id, "task2");
        assert!(state.active_tasks.contains_key("task2"));
        // Task1 might still have an empty set in active_tasks
        // Check that the token is actually in task2
        assert!(state.active_tasks.get("task2").map_or(false, |s| !s.is_empty()));

        state.remove_token(&token_id);
        assert_eq!(state.tokens.len(), 0);
    }

    #[test]
    fn test_history_recording() {
        let mut state = NetState::new();
        state.record_event("task1", HistoryEventType::Started, HashMap::new());
        state.record_event("task1", HistoryEventType::Completed, HashMap::new());

        assert_eq!(state.history.len(), 2);
        assert_eq!(state.history[0].event_type, HistoryEventType::Started);
        assert_eq!(state.history[1].event_type, HistoryEventType::Completed);
    }
}
