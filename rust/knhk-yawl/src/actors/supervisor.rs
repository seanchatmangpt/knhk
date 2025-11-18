//! Workflow Supervisor - Root of Actor Hierarchy
//!
//! Implements Erlang-style supervision tree with restart strategies.

use super::*;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info, instrument};

/// Workflow supervisor - manages case actors
pub struct WorkflowSupervisor {
    ctx: ActorContext,
    receiver: mpsc::Receiver<SupervisorMessage>,
    children: HashMap<ActorId, ChildState>,
    strategy: SupervisionStrategy,
}

#[derive(Debug)]
struct ChildState {
    actor_id: ActorId,
    actor_type: ActorType,
    restart_count: usize,
    max_restarts: usize,
}

impl WorkflowSupervisor {
    pub fn new(strategy: SupervisionStrategy) -> (Self, ActorHandle<SupervisorMessage>) {
        let ctx = ActorContext::new(ActorType::Supervisor);
        let (tx, rx) = mpsc::channel(1000);

        let handle = ActorHandle {
            actor_id: ctx.actor_id,
            sender: tx,
        };

        let supervisor = Self {
            ctx,
            receiver: rx,
            children: HashMap::new(),
            strategy,
        };

        (supervisor, handle)
    }

    #[instrument(skip(self))]
    async fn handle_message(&mut self, msg: SupervisorMessage) {
        match msg {
            SupervisorMessage::RegisterChild {
                child_id,
                child_type,
            } => {
                info!(?child_id, ?child_type, "Registering child actor");

                self.children.insert(
                    child_id,
                    ChildState {
                        actor_id: child_id,
                        actor_type: child_type,
                        restart_count: 0,
                        max_restarts: 3,
                    },
                );

                self.ctx.add_child(child_id);
            }

            SupervisorMessage::ChildFailed { child_id, error } => {
                error!(?child_id, %error, "Child actor failed");

                if let Some(child) = self.children.get_mut(&child_id) {
                    match self.strategy {
                        SupervisionStrategy::Restart => {
                            if child.restart_count < child.max_restarts {
                                info!(?child_id, "Restarting child actor");
                                child.restart_count += 1;
                                // TODO: Actually restart the actor
                            } else {
                                error!(?child_id, "Max restarts exceeded, stopping child");
                                self.children.remove(&child_id);
                            }
                        }
                        SupervisionStrategy::Stop => {
                            info!(?child_id, "Stopping child actor");
                            self.children.remove(&child_id);
                        }
                        SupervisionStrategy::Resume => {
                            info!(?child_id, "Resuming child actor (ignoring error)");
                        }
                        SupervisionStrategy::Escalate => {
                            error!("Escalating failure to parent (not implemented)");
                        }
                    }
                }
            }

            SupervisorMessage::RestartChild { child_id } => {
                info!(?child_id, "Manual restart requested");
                // TODO: Implement restart logic
            }

            SupervisorMessage::ShutdownAll { reply } => {
                info!("Shutting down all child actors");
                self.children.clear();
                let _ = reply.send(());
            }
        }
    }
}

#[async_trait]
impl Actor for WorkflowSupervisor {
    fn actor_id(&self) -> ActorId {
        self.ctx.actor_id
    }

    fn actor_type(&self) -> ActorType {
        self.ctx.actor_type
    }

    #[instrument(skip(self))]
    async fn run(mut self) {
        info!("Workflow supervisor started");

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }

        info!("Workflow supervisor stopped");
    }

    async fn shutdown(&mut self) {
        info!("Supervisor shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_creation() {
        let (supervisor, handle) = WorkflowSupervisor::new(SupervisionStrategy::Restart);

        assert_eq!(supervisor.actor_type(), ActorType::Supervisor);
        assert_eq!(handle.actor_id, supervisor.actor_id());
    }
}
