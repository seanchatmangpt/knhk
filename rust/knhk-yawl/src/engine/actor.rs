//! Task actor implementation
//!
//! TaskActors execute individual tasks within a workflow.

use crate::core::Task;
use std::fmt;

/// Task actor
///
/// Represents an executing task instance.
/// In a full implementation, this would be an actor in an actor system.
#[derive(Debug, Clone)]
pub struct TaskActor {
    /// Task being executed
    pub task: Task,

    /// Actor ID (unique per actor instance)
    pub actor_id: String,

    /// Supervisor actor ID (for supervision hierarchies)
    pub supervisor_id: Option<String>,
}

impl TaskActor {
    /// Create a new task actor
    #[must_use]
    pub fn new(task: Task, actor_id: impl Into<String>) -> Self {
        Self {
            task,
            actor_id: actor_id.into(),
            supervisor_id: None,
        }
    }

    /// Set supervisor
    #[must_use]
    pub fn with_supervisor(mut self, supervisor_id: impl Into<String>) -> Self {
        self.supervisor_id = Some(supervisor_id.into());
        self
    }

    /// Execute the task
    ///
    /// # Errors
    /// Returns error if task execution fails
    #[tracing::instrument(skip(self))]
    pub fn execute(&self) -> crate::Result<()> {
        tracing::info!("Executing task: {} (actor: {})", self.task.id, self.actor_id);

        // In real implementation, would execute the actual task logic
        // For now, just log and return success

        tracing::info!("Task completed: {}", self.task.id);
        Ok(())
    }
}

impl fmt::Display for TaskActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TaskActor(task={}, actor_id={})",
            self.task.id, self.actor_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TaskType;

    #[test]
    fn test_task_actor_creation() {
        let task = Task::new("task1", "Test Task", TaskType::Atomic);
        let actor = TaskActor::new(task, "actor1");

        assert_eq!(actor.actor_id, "actor1");
        assert_eq!(actor.supervisor_id, None);
    }

    #[test]
    fn test_task_actor_with_supervisor() {
        let task = Task::new("task1", "Test Task", TaskType::Atomic);
        let actor = TaskActor::new(task, "actor1").with_supervisor("supervisor1");

        assert_eq!(actor.supervisor_id, Some("supervisor1".to_string()));
    }

    #[test]
    fn test_task_actor_execution() {
        let task = Task::new("task1", "Test Task", TaskType::Atomic);
        let actor = TaskActor::new(task, "actor1");

        assert!(actor.execute().is_ok());
    }
}
