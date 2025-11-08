//! Timer service for time-based workflow patterns
//!
//! Handles:
//! - Pattern 30: Transient Trigger (one-shot timers)
//! - Pattern 31: Persistent Trigger (recurring timers)
//!
//! Timer events are sent via async channel to the engine for pattern execution.

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::PatternId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant as TokioInstant};

/// Timer fired event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerFired {
    /// Pattern ID (30 for transient, 31 for persistent)
    pub pattern_id: u32,
    /// Case ID
    pub case_id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// Timer key/identifier
    pub key: String,
    /// Fired timestamp
    pub fired_at: DateTime<Utc>,
}

/// Timer kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerKind {
    /// One-shot timer (Pattern 30)
    Transient,
    /// Recurring timer (Pattern 31)
    Persistent,
}

/// Timer entry
#[derive(Debug, Clone)]
struct TimerEntry {
    /// Timer ID
    id: String,
    /// Pattern ID
    pattern_id: PatternId,
    /// Case ID
    case_id: String,
    /// Workflow ID
    workflow_id: String,
    /// Timer key
    key: String,
    /// Timer kind
    kind: TimerKind,
    /// Due time
    due_at: DateTime<Utc>,
    /// Recurrence rule (for persistent timers)
    rrule: Option<String>,
    /// Active flag
    active: bool,
}

/// Timer service
pub struct TimerService {
    /// Timer event sender
    timer_tx: mpsc::Sender<TimerFired>,
    /// Active timers
    timers: Arc<tokio::sync::RwLock<HashMap<String, TimerEntry>>>,
}

impl TimerService {
    /// Create a new timer service
    pub fn new(timer_tx: mpsc::Sender<TimerFired>) -> Self {
        let timers = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let service = Self {
            timer_tx,
            timers: timers.clone(),
        };

        // Start timer loop
        let timers_clone = timers.clone();
        let tx_clone = service.timer_tx.clone();
        tokio::spawn(async move {
            Self::timer_loop(timers_clone, tx_clone).await;
        });

        service
    }

    /// Register a transient timer (Pattern 30)
    pub async fn register_transient(
        &self,
        case_id: String,
        workflow_id: String,
        key: String,
        due_at: DateTime<Utc>,
    ) -> WorkflowResult<String> {
        let timer_id = format!("{}:{}:{}", case_id, workflow_id, key);
        let entry = TimerEntry {
            id: timer_id.clone(),
            pattern_id: PatternId(30),
            case_id,
            workflow_id,
            key,
            kind: TimerKind::Transient,
            due_at,
            rrule: None,
            active: true,
        };

        let mut timers = self.timers.write().await;
        timers.insert(timer_id.clone(), entry);
        Ok(timer_id)
    }

    /// Register a persistent timer (Pattern 31)
    pub async fn register_persistent(
        &self,
        case_id: String,
        workflow_id: String,
        key: String,
        due_at: DateTime<Utc>,
        rrule: Option<String>,
    ) -> WorkflowResult<String> {
        let timer_id = format!("{}:{}:{}", case_id, workflow_id, key);
        let entry = TimerEntry {
            id: timer_id.clone(),
            pattern_id: PatternId(31),
            case_id,
            workflow_id,
            key,
            kind: TimerKind::Persistent,
            due_at,
            rrule,
            active: true,
        };

        let mut timers = self.timers.write().await;
        timers.insert(timer_id.clone(), entry);
        Ok(timer_id)
    }

    /// Cancel a timer
    pub async fn cancel(&self, timer_id: &str) -> WorkflowResult<()> {
        let mut timers = self.timers.write().await;
        if let Some(entry) = timers.get_mut(timer_id) {
            entry.active = false;
            timers.remove(timer_id);
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Timer {} not found",
                timer_id
            )))
        }
    }

    /// Cancel all timers for a case
    pub async fn cancel_case_timers(&self, case_id: &str) -> WorkflowResult<()> {
        let mut timers = self.timers.write().await;
        timers.retain(|id, entry| {
            if entry.case_id == case_id {
                false
            } else {
                true
            }
        });
        Ok(())
    }

    /// Timer loop - checks timers and fires events
    async fn timer_loop(
        timers: Arc<tokio::sync::RwLock<HashMap<String, TimerEntry>>>,
        tx: mpsc::Sender<TimerFired>,
    ) {
        loop {
            // Check every 100ms
            sleep(Duration::from_millis(100)).await;

            let now = Utc::now();
            let mut to_fire = Vec::new();
            let mut to_update = Vec::new();

            {
                let timers_read = timers.read().await;
                for (id, entry) in timers_read.iter() {
                    if !entry.active {
                        continue;
                    }

                    if entry.due_at <= now {
                        // Timer fired
                        to_fire.push(TimerFired {
                            pattern_id: entry.pattern_id.0 as u32,
                            case_id: entry.case_id.clone(),
                            workflow_id: entry.workflow_id.clone(),
                            key: entry.key.clone(),
                            fired_at: now,
                        });

                        // For persistent timers, calculate next occurrence
                        if entry.kind == TimerKind::Persistent {
                            // FUTURE: Parse RRULE and calculate next occurrence
                            // For now, remove persistent timers after firing
                            to_update.push(id.clone());
                        } else {
                            // Transient timers are removed after firing
                            to_update.push(id.clone());
                        }
                    }
                }
            }

            // Fire timer events
            for event in to_fire {
                let _ = tx.send(event).await;
            }

            // Remove fired timers
            if !to_update.is_empty() {
                let mut timers_write = timers.write().await;
                for id in to_update {
                    timers_write.remove(&id);
                }
            }
        }
    }
}

impl Clone for TimerService {
    fn clone(&self) -> Self {
        Self {
            timer_tx: self.timer_tx.clone(),
            timers: Arc::clone(&self.timers),
        }
    }
}
