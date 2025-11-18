//! Case Actor - Manages Single Workflow Instance
//!
//! Each case (workflow instance) is an independent actor.
//! Responsible for task scheduling, state management, and pattern execution.

use super::*;
use crate::core::*;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info, instrument};

/// Case actor - executes a workflow instance
pub struct CaseActor {
    ctx: ActorContext,
    receiver: mpsc::Receiver<CaseMessage>,
    case_id: CaseId,
    workflow: Arc<Workflow>,
    state: CaseSnapshot,
    execution_context: Arc<parking_lot::RwLock<ExecutionContext>>,
    task_handles: HashMap<TaskId, ActorHandle<TaskMessage>>,
}

impl CaseActor {
    pub fn new(
        case_id: CaseId,
        workflow: Arc<Workflow>,
    ) -> (Self, ActorHandle<CaseMessage>) {
        let ctx = ActorContext::new(ActorType::CaseActor);
        let (tx, rx) = mpsc::channel(1000);

        let handle = ActorHandle {
            actor_id: ctx.actor_id,
            sender: tx,
        };

        let state = CaseSnapshot::new(case_id, workflow.id);
        let execution_context = Arc::new(parking_lot::RwLock::new(ExecutionContext::new(
            case_id,
            workflow.id,
        )));

        let actor = Self {
            ctx,
            receiver: rx,
            case_id,
            workflow,
            state,
            execution_context,
            task_handles: HashMap::new(),
        };

        (actor, handle)
    }

    #[instrument(skip(self))]
    async fn handle_message(&mut self, msg: CaseMessage) {
        match msg {
            CaseMessage::Start {
                workflow,
                initial_data,
                reply,
            } => {
                info!(case_id = %self.case_id, "Starting case execution");

                // Set initial data
                if let serde_json::Value::Object(map) = initial_data {
                    for (key, value) in map {
                        self.execution_context.write().set_data(key, value);
                    }
                }

                // Transition to Running state
                match self.state.transition(CaseState::Running) {
                    Ok(new_state) => {
                        self.state = new_state;

                        // Start initial task
                        let start_task = workflow.start_task;
                        let result = self.schedule_task(start_task).await;

                        let _ = reply.send(result.map(|_| self.case_id));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }

            CaseMessage::ExecuteTask { task_id, reply } => {
                info!(?task_id, "Executing task");

                let result = self.execute_task(task_id).await;
                let _ = reply.send(result);
            }

            CaseMessage::Suspend { reply } => {
                info!("Suspending case");

                match self.state.transition(CaseState::Suspended) {
                    Ok(new_state) => {
                        self.state = new_state;
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }

            CaseMessage::Resume { reply } => {
                info!("Resuming case");

                match self.state.transition(CaseState::Running) {
                    Ok(new_state) => {
                        self.state = new_state;
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }

            CaseMessage::Cancel { reply } => {
                info!("Cancelling case");

                match self.state.transition(CaseState::Cancelled) {
                    Ok(new_state) => {
                        self.state = new_state;
                        // TODO: Cancel all active tasks
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }

            CaseMessage::GetState { reply } => {
                let _ = reply.send(self.state.clone());
            }

            CaseMessage::Shutdown => {
                info!("Case actor shutting down");
            }
        }
    }

    async fn schedule_task(&mut self, task_id: TaskId) -> Result<(), ExecutionError> {
        // TODO: Create task actor and schedule execution
        Ok(())
    }

    async fn execute_task(&mut self, task_id: TaskId) -> Result<ExecutionResult, ExecutionError> {
        // TODO: Delegate to task actor
        Ok(ExecutionResult {
            success: true,
            ticks_used: 1,
            output_data: None,
            activated_arcs: Vec::new(),
        })
    }
}

#[async_trait]
impl Actor for CaseActor {
    fn actor_id(&self) -> ActorId {
        self.ctx.actor_id
    }

    fn actor_type(&self) -> ActorType {
        self.ctx.actor_type
    }

    #[instrument(skip(self))]
    async fn run(mut self) {
        info!(case_id = %self.case_id, "Case actor started");

        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg).await;
        }

        info!(case_id = %self.case_id, "Case actor stopped");
    }

    async fn shutdown(&mut self) {
        info!("Case actor shutting down");
    }
}
