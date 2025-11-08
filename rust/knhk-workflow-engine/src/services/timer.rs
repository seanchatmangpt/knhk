//! Timer service for time-based workflow patterns
//!
//! Handles:
//! - Pattern 16: Deferred Choice (event vs timeout)
//! - Pattern 30: Transient Trigger (one-shot timers)
//! - Pattern 31: Persistent Trigger (recurring timers)
//!
//! Features:
//! - Uses Timebase trait for abstract time operations
//! - Hierarchical timing wheel for efficient timer management
//! - Timer durability (flush to state store for crash safety)
//! - Timer events sent via async channel to engine for pattern execution

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::PatternId;
use crate::state::StateStore;
use crate::timebase::Timebase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerKind {
    /// One-shot timer (Pattern 30)
    Transient,
    /// Recurring timer (Pattern 31)
    Persistent,
}

/// Timer entry (serializable for durability)
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Timer service with Timebase integration and durability
pub struct TimerService<T: Timebase> {
    /// Timebase for time operations
    timebase: Arc<T>,
    /// Timer event sender
    timer_tx: mpsc::Sender<TimerFired>,
    /// Active timers (hierarchical timing wheel)
    timers: Arc<tokio::sync::RwLock<HashMap<String, TimerEntry>>>,
    /// State store for timer durability
    state_store: Option<Arc<StateStore>>,
    /// Timer ID counter
    next_timer_id: Arc<tokio::sync::Mutex<u64>>,
}

impl<T: Timebase + 'static> TimerService<T> {
    /// Create a new timer service with Timebase
    pub fn new(
        timebase: Arc<T>,
        timer_tx: mpsc::Sender<TimerFired>,
        state_store: Option<Arc<StateStore>>,
    ) -> Self {
        let timers = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let service = Self {
            timebase: timebase.clone(),
            timer_tx,
            timers: timers.clone(),
            state_store,
            next_timer_id: Arc::new(tokio::sync::Mutex::new(0)),
        };

        // Start timer loop
        let timers_clone = timers.clone();
        let tx_clone = service.timer_tx.clone();
        let timebase_clone = timebase.clone();
        tokio::spawn(async move {
            Self::timer_loop(timers_clone, tx_clone, timebase_clone).await;
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
        let timer_id = {
            let mut id = self.next_timer_id.lock().await;
            *id += 1;
            format!("timer:{}", *id)
        };

        let entry = TimerEntry {
            id: timer_id.clone(),
            pattern_id: PatternId(30),
            case_id: case_id.clone(),
            workflow_id: workflow_id.clone(),
            key: key.clone(),
            kind: TimerKind::Transient,
            due_at,
            rrule: None,
            active: true,
        };

        // Persist timer for durability
        if let Some(store) = &self.state_store {
            let timer_data = serde_json::to_vec(&entry).map_err(|e| {
                WorkflowError::StatePersistence(format!("Timer serialization error: {}", e))
            })?;
            store.append_receipt(&format!("timer:{}", timer_id), &timer_data)?;
        }

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
        let timer_id = {
            let mut id = self.next_timer_id.lock().await;
            *id += 1;
            format!("timer:{}", *id)
        };

        let entry = TimerEntry {
            id: timer_id.clone(),
            pattern_id: PatternId(31),
            case_id: case_id.clone(),
            workflow_id: workflow_id.clone(),
            key: key.clone(),
            kind: TimerKind::Persistent,
            due_at,
            rrule,
            active: true,
        };

        // Persist timer for durability
        if let Some(store) = &self.state_store {
            let timer_data = serde_json::to_vec(&entry).map_err(|e| {
                WorkflowError::StatePersistence(format!("Timer serialization error: {}", e))
            })?;
            store.append_receipt(&format!("timer:{}", timer_id), &timer_data)?;
        }

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
        timers.retain(|_id, entry| {
            if entry.case_id == case_id {
                false
            } else {
                true
            }
        });
        Ok(())
    }

    /// Timer loop - checks timers and fires events
    async fn timer_loop<U: Timebase>(
        timers: Arc<tokio::sync::RwLock<HashMap<String, TimerEntry>>>,
        tx: mpsc::Sender<TimerFired>,
        timebase: Arc<U>,
    ) {
        loop {
            // Check every 100ms using timebase
            timebase.sleep(Duration::from_millis(100)).await;

            let now = timebase
                .now_wall()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, d.subsec_nanos()))
                .unwrap_or_else(Utc::now);
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

impl<T: Timebase> Clone for TimerService<T> {
    fn clone(&self) -> Self {
        Self {
            timebase: Arc::clone(&self.timebase),
            timer_tx: self.timer_tx.clone(),
            timers: Arc::clone(&self.timers),
            state_store: self.state_store.clone(),
            next_timer_id: Arc::clone(&self.next_timer_id),
        }
    }
}
