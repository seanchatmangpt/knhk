//! Supervision trees and fault tolerance strategies
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: Q (Invariants enforcement via supervision)
//! - Covenant: Covenant 2 (Invariants are law - supervision enforces them)
//! - Validation: Fault recovery < 100ms (warm path SLO)

use crate::engine::actor::{ActorError, ActorHandle, ActorState};
use crate::engine::messages::TaskId;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// Supervision strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// Restart failed child only
    OneForOne,

    /// Restart all children when one fails
    OneForAll,

    /// Restart failed child and all started after it
    RestForOne,
}

/// Restart strategy
#[derive(Debug, Clone)]
pub struct RestartStrategy {
    /// Maximum number of restarts allowed
    pub max_restarts: u32,

    /// Time window for restart counting
    pub window: Duration,

    /// Backoff strategy between restarts
    pub backoff: BackoffStrategy,
}

impl Default for RestartStrategy {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            window: Duration::from_secs(60),
            backoff: BackoffStrategy::Exponential {
                initial: Duration::from_millis(100),
                max: Duration::from_secs(30),
                factor: 2.0,
            },
        }
    }
}

/// Backoff strategy for restarts
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between restarts
    Fixed(Duration),

    /// Exponential backoff with maximum
    Exponential {
        initial: Duration,
        max: Duration,
        factor: f64,
    },
}

impl BackoffStrategy {
    /// Calculate delay for given restart attempt
    pub fn delay(&self, attempt: u32) -> Duration {
        match self {
            Self::Fixed(d) => *d,
            Self::Exponential { initial, max, factor } => {
                let delay_ms = initial.as_millis() as f64 * factor.powi(attempt as i32);
                let delay = Duration::from_millis(delay_ms as u64);
                delay.min(*max)
            }
        }
    }
}

/// Supervision tree errors
#[derive(Debug, Error)]
pub enum SupervisionError {
    #[error("Max restarts exceeded")]
    MaxRestartsExceeded,

    #[error("Child restart failed: {0}")]
    RestartFailed(String),

    #[error("No such child: {0}")]
    ChildNotFound(TaskId),
}

/// Supervision tree node
pub struct SupervisionTree {
    /// Supervision strategy
    strategy: SupervisionStrategy,

    /// Restart strategy
    restart_strategy: RestartStrategy,

    /// Children and their restart counts
    children: Vec<(TaskId, ActorHandle, u32)>,

    /// State
    state: ActorState,
}

impl SupervisionTree {
    pub fn new(strategy: SupervisionStrategy, restart_strategy: RestartStrategy) -> Self {
        Self {
            strategy,
            restart_strategy,
            children: Vec::new(),
            state: ActorState::Initializing,
        }
    }

    /// Add child to supervision tree
    pub fn add_child(&mut self, task_id: TaskId, handle: ActorHandle) {
        self.children.push((task_id, handle, 0));
        info!(task_id = %task_id, "Child added to supervision tree");
    }

    /// Handle child failure with supervision strategy
    #[tracing::instrument(skip(self))]
    pub async fn handle_failure(&mut self, failed_task: TaskId) -> Result<(), SupervisionError> {
        // Find failed child
        let child_idx = self
            .children
            .iter()
            .position(|(id, _, _)| *id == failed_task)
            .ok_or(SupervisionError::ChildNotFound(failed_task))?;

        match self.strategy {
            SupervisionStrategy::OneForOne => {
                self.restart_child(child_idx).await?;
            }
            SupervisionStrategy::OneForAll => {
                self.restart_all_children().await?;
            }
            SupervisionStrategy::RestForOne => {
                self.restart_from_index(child_idx).await?;
            }
        }

        Ok(())
    }

    /// Restart a single child
    #[tracing::instrument(skip(self))]
    async fn restart_child(&mut self, idx: usize) -> Result<(), SupervisionError> {
        let (task_id, handle, restart_count) = &mut self.children[idx];

        // Check restart limits
        if *restart_count >= self.restart_strategy.max_restarts {
            error!(
                task_id = %task_id,
                restarts = *restart_count,
                "Child exceeded max restarts"
            );
            return Err(SupervisionError::MaxRestartsExceeded);
        }

        // Calculate backoff delay
        let delay = self.restart_strategy.backoff.delay(*restart_count);
        warn!(
            task_id = %task_id,
            attempt = *restart_count + 1,
            delay_ms = delay.as_millis(),
            "Restarting child with backoff"
        );

        // Apply backoff
        sleep(delay).await;

        // Increment restart count
        *restart_count += 1;

        // TODO: Implement actual restart logic
        // For now, just update state
        handle.set_state(ActorState::Running);

        info!(task_id = %task_id, "Child restarted successfully");
        Ok(())
    }

    /// Restart all children (OneForAll strategy)
    #[tracing::instrument(skip(self))]
    async fn restart_all_children(&mut self) -> Result<(), SupervisionError> {
        warn!("Restarting all children (OneForAll)");

        for idx in 0..self.children.len() {
            self.restart_child(idx).await?;
        }

        Ok(())
    }

    /// Restart children from index onwards (RestForOne strategy)
    #[tracing::instrument(skip(self))]
    async fn restart_from_index(&mut self, start_idx: usize) -> Result<(), SupervisionError> {
        warn!(start_idx, "Restarting children from index (RestForOne)");

        for idx in start_idx..self.children.len() {
            self.restart_child(idx).await?;
        }

        Ok(())
    }

    /// Shutdown all children gracefully
    #[tracing::instrument(skip(self))]
    pub async fn shutdown_all(&mut self) {
        info!("Shutting down all supervised children");

        for (task_id, handle, _) in &self.children {
            match handle.send(crate::engine::messages::WorkflowMessage::Shutdown).await {
                Ok(_) => info!(task_id = %task_id, "Child shutdown initiated"),
                Err(e) => warn!(task_id = %task_id, error = ?e, "Child shutdown failed"),
            }
        }

        self.children.clear();
        self.state = ActorState::Stopped;

        info!("All children shut down");
    }

    /// Get current state
    pub fn get_state(&self) -> ActorState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_backoff_fixed() {
        let backoff = BackoffStrategy::Fixed(Duration::from_millis(100));

        assert_eq!(backoff.delay(0), Duration::from_millis(100));
        assert_eq!(backoff.delay(5), Duration::from_millis(100));
    }

    #[test]
    fn test_backoff_exponential() {
        let backoff = BackoffStrategy::Exponential {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(10),
            factor: 2.0,
        };

        assert_eq!(backoff.delay(0), Duration::from_millis(100));
        assert_eq!(backoff.delay(1), Duration::from_millis(200));
        assert_eq!(backoff.delay(2), Duration::from_millis(400));

        // Should cap at max
        assert!(backoff.delay(10) <= Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_supervision_tree_add_child() {
        let mut tree = SupervisionTree::new(
            SupervisionStrategy::OneForOne,
            RestartStrategy::default(),
        );

        let task_id = TaskId::new();
        let (tx, _rx) = mpsc::channel(10);
        let handle = ActorHandle::new(tx);

        tree.add_child(task_id, handle);

        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].0, task_id);
        assert_eq!(tree.children[0].2, 0); // Restart count
    }
}
