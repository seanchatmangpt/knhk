//! State Manager - Improved state management with event sourcing
//!
//! Provides:
//! - Event sourcing for auditability
//! - State caching for performance
//! - State snapshots for recovery

use crate::case::{Case, CaseId};
use crate::error::WorkflowResult;
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use crate::state::store::StateStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// State manager with event sourcing and caching
#[derive(Clone)]
pub struct StateManager {
    /// State store
    store: Arc<StateStore>,
    /// In-memory cache for specs
    spec_cache: Arc<RwLock<HashMap<WorkflowSpecId, WorkflowSpec>>>,
    /// In-memory cache for cases
    case_cache: Arc<RwLock<HashMap<CaseId, Case>>>,
    /// Event log
    event_log: Arc<RwLock<Vec<StateEvent>>>,
}

/// State event for event sourcing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum StateEvent {
    /// Workflow spec registered
    SpecRegistered {
        spec_id: WorkflowSpecId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Case created
    CaseCreated {
        case_id: CaseId,
        spec_id: WorkflowSpecId,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Case state changed
    CaseStateChanged {
        case_id: CaseId,
        old_state: String,
        new_state: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Task started execution
    TaskStarted {
        case_id: CaseId,
        task_id: String,
        task_name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Task completed execution
    TaskCompleted {
        case_id: CaseId,
        task_id: String,
        task_name: String,
        duration_ms: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl StateEvent {
    /// Get case_id from event if it exists
    pub fn case_id(&self) -> Option<CaseId> {
        match self {
            StateEvent::SpecRegistered { .. } => None,
            StateEvent::CaseCreated { case_id, .. } => Some(*case_id),
            StateEvent::CaseStateChanged { case_id, .. } => Some(*case_id),
            StateEvent::TaskStarted { case_id, .. } => Some(*case_id),
            StateEvent::TaskCompleted { case_id, .. } => Some(*case_id),
        }
    }

    /// Get timestamp from event
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            StateEvent::SpecRegistered { timestamp, .. } => *timestamp,
            StateEvent::CaseCreated { timestamp, .. } => *timestamp,
            StateEvent::CaseStateChanged { timestamp, .. } => *timestamp,
            StateEvent::TaskStarted { timestamp, .. } => *timestamp,
            StateEvent::TaskCompleted { timestamp, .. } => *timestamp,
        }
    }
}

impl StateManager {
    /// Create new state manager
    pub fn new(store: Arc<StateStore>) -> Self {
        Self {
            store,
            spec_cache: Arc::new(RwLock::new(HashMap::new())),
            case_cache: Arc::new(RwLock::new(HashMap::new())),
            event_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Save workflow spec with event logging
    pub async fn save_spec(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Save to store
        self.store.save_spec(spec)?;

        // Update cache
        {
            let mut cache = self.spec_cache.write().await;
            cache.insert(spec.id, spec.clone());
        }

        // Log event
        let event = StateEvent::SpecRegistered {
            spec_id: spec.id,
            timestamp: chrono::Utc::now(),
        };
        {
            let mut log = self.event_log.write().await;
            log.push(event.clone());
        }
        // Persist event to store (for audit trail)
        // Note: Spec events don't have a case_id, so we don't persist them to case history

        Ok(())
    }

    /// Load workflow spec with caching
    pub async fn load_spec(
        &self,
        spec_id: &WorkflowSpecId,
    ) -> WorkflowResult<Option<WorkflowSpec>> {
        // Check cache first
        {
            let cache = self.spec_cache.read().await;
            if let Some(spec) = cache.get(spec_id) {
                return Ok(Some(spec.clone()));
            }
        }

        // Load from store
        let spec = self.store.load_spec(spec_id)?;

        // Update cache if found
        if let Some(ref spec) = spec {
            let mut cache = self.spec_cache.write().await;
            cache.insert(*spec_id, spec.clone());
        }

        Ok(spec)
    }

    /// Save case with event logging
    pub async fn save_case(&self, case: &Case) -> WorkflowResult<()> {
        // Get old state for event logging
        let _old_state = {
            let cache = self.case_cache.read().await;
            cache.get(&case.id).map(|c| c.state.to_string())
        };

        // Save to store
        self.store.save_case(case.id, case)?;

        // Update cache
        {
            let mut cache = self.case_cache.write().await;
            let old_state_str = cache
                .get(&case.id)
                .map(|c| c.state.to_string())
                .unwrap_or_else(|| "None".to_string());
            cache.insert(case.id, case.clone());

            // Log state change event
            if old_state_str != case.state.to_string() {
                let event = StateEvent::CaseStateChanged {
                    case_id: case.id,
                    old_state: old_state_str,
                    new_state: case.state.to_string(),
                    timestamp: chrono::Utc::now(),
                };
                let mut log = self.event_log.write().await;
                log.push(event.clone());
                // Persist event to store (for audit trail)
                drop(log);
                self.store.save_case_history_event(&case.id, &event)?;
            }
        }

        Ok(())
    }

    /// Load case with caching
    pub async fn load_case(&self, case_id: &CaseId) -> WorkflowResult<Option<Case>> {
        // Check cache first
        {
            let cache = self.case_cache.read().await;
            if let Some(case) = cache.get(case_id) {
                return Ok(Some(case.clone()));
            }
        }

        // Load from store
        let case = self.store.load_case(case_id)?;

        // Update cache if found
        if let Some(ref case) = case {
            let mut cache = self.case_cache.write().await;
            cache.insert(*case_id, case.clone());
        }

        Ok(case)
    }

    /// Get event log
    pub async fn get_events(&self, limit: Option<usize>) -> Vec<StateEvent> {
        let log = self.event_log.read().await;
        if let Some(limit) = limit {
            log.iter().rev().take(limit).cloned().collect()
        } else {
            log.clone()
        }
    }

    /// Get case history (all events for a specific case)
    pub async fn get_case_history(&self, case_id: CaseId) -> Vec<StateEvent> {
        // Try to load from store first (persistent history)
        let events = match self.store.load_case_history(&case_id) {
            Ok(loaded_events) if !loaded_events.is_empty() => loaded_events,
            _ => Vec::new(),
        };

        // Merge with in-memory events (for recent events not yet persisted)
        let log = self.event_log.read().await;
        let in_memory_events: Vec<StateEvent> = log
            .iter()
            .filter(|event| event.case_id() == Some(case_id))
            .cloned()
            .collect();

        // Merge and deduplicate (prefer in-memory for recent events)
        // Use HashMap to deduplicate by timestamp + event type
        let mut dedup_map: std::collections::HashMap<
            (chrono::DateTime<chrono::Utc>, String),
            StateEvent,
        > = std::collections::HashMap::new();

        // First add persisted events
        for event in events {
            let key = (
                event.timestamp(),
                format!("{:?}", std::mem::discriminant(&event)),
            );
            dedup_map.insert(key, event);
        }

        // Then add in-memory events (will overwrite duplicates, preferring newer)
        for event in in_memory_events {
            let key = (
                event.timestamp(),
                format!("{:?}", std::mem::discriminant(&event)),
            );
            dedup_map.insert(key, event);
        }

        // Convert back to Vec and sort by timestamp
        let mut result: Vec<StateEvent> = dedup_map.into_values().collect();
        result.sort_by_key(|a| a.timestamp());
        result
    }

    /// Log task started event
    pub async fn log_task_started(
        &self,
        case_id: CaseId,
        task_id: String,
        task_name: String,
    ) -> WorkflowResult<()> {
        let event = StateEvent::TaskStarted {
            case_id,
            task_id,
            task_name,
            timestamp: chrono::Utc::now(),
        };
        {
            let mut log = self.event_log.write().await;
            log.push(event.clone());
        }
        // Persist event to store (for audit trail)
        self.store.save_case_history_event(&case_id, &event)?;
        Ok(())
    }

    /// Log task completed event
    pub async fn log_task_completed(
        &self,
        case_id: CaseId,
        task_id: String,
        task_name: String,
        duration_ms: u64,
    ) -> WorkflowResult<()> {
        let event = StateEvent::TaskCompleted {
            case_id,
            task_id,
            task_name,
            duration_ms,
            timestamp: chrono::Utc::now(),
        };
        {
            let mut log = self.event_log.write().await;
            log.push(event.clone());
        }
        // Persist event to store (for audit trail)
        self.store.save_case_history_event(&case_id, &event)?;
        Ok(())
    }

    /// Clear cache (for testing/debugging)
    pub async fn clear_cache(&self) {
        let mut spec_cache = self.spec_cache.write().await;
        spec_cache.clear();
        let mut case_cache = self.case_cache.write().await;
        case_cache.clear();
    }
}
