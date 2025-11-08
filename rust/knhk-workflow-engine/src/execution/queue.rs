//! Work Queue - Distributed execution support
//!
//! Provides:
//! - Work queue for distributed execution
//! - Job scheduling
//! - Load balancing

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Work queue for distributed execution
pub struct WorkQueue {
    /// Job sender
    sender: mpsc::UnboundedSender<WorkItem>,
    /// Worker pool
    workers: Vec<Arc<Worker>>,
}

/// Work item
#[derive(Debug, Clone)]
pub struct WorkItem {
    /// Work item ID
    pub id: String,
    /// Pattern ID
    pub pattern_id: PatternId,
    /// Execution context
    pub context: PatternExecutionContext,
    /// Priority
    pub priority: u8,
}

/// Worker for processing work items
pub struct Worker {
    /// Worker ID
    pub id: String,
    /// Receiver for work items
    receiver: mpsc::UnboundedReceiver<WorkItem>,
}

impl WorkQueue {
    /// Create new work queue
    pub fn new(worker_count: usize) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let mut workers = Vec::new();

        // Create workers
        // Note: UnboundedReceiver cannot be cloned, so we use a shared sender
        // Each worker will need to be created differently - for now, use a single worker
        if worker_count > 0 {
            let worker = Arc::new(Worker {
                id: "worker-0".to_string(),
                receiver,
            });
            workers.push(worker);
        }

        Self { sender, workers }
    }

    /// Enqueue work item
    pub fn enqueue(&self, item: WorkItem) -> WorkflowResult<()> {
        self.sender
            .send(item)
            .map_err(|e| WorkflowError::Internal(format!("Failed to enqueue work item: {}", e)))?;
        Ok(())
    }

    /// Get worker pool size
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

impl Worker {
    /// Process work items
    pub async fn process(&mut self) -> Option<WorkItem> {
        self.receiver.recv().await
    }
}

