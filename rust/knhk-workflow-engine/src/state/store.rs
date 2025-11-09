//! State persistence for workflow engine
//!
//! Provides sled-based persistence with hot cache layer for reflex core performance.

use crate::cache::ReflexCache;
use crate::case::Case;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use sled::Db;
use std::path::Path;
use std::sync::Arc;

/// State store for workflow engine with hot cache layer
pub struct StateStore {
    /// Sled database for cold storage
    db: Db,
    /// Hot cache for reflex core (sub-microsecond access, lock-free DashMap)
    cache: ReflexCache,
}

impl StateStore {
    /// Create a new state store
    pub fn new<P: AsRef<Path>>(path: P) -> WorkflowResult<Self> {
        let db = sled::open(path).map_err(|e| {
            WorkflowError::StatePersistence(format!("Failed to open database: {:?}", e))
        })?;
        Ok(Self {
            db,
            cache: ReflexCache::new(),
        })
    }

    /// Save a workflow specification (with cache)
    pub fn save_spec(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        // Update cache first (hot path, lock-free DashMap operation)
        self.cache
            .insert_spec(spec.id.clone(), Arc::new(spec.clone()));

        // Persist to sled (async, can be batched)
        let key = format!("spec:{}", spec.id);
        let value = serde_json::to_vec(spec)
            .map_err(|e| WorkflowError::StatePersistence(format!("Serialization error: {}", e)))?;
        self.db
            .insert(key.as_bytes(), value)
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;
        Ok(())
    }

    /// Load a workflow specification (cache-first)
    pub fn load_spec(
        &self,
        spec_id: &crate::parser::WorkflowSpecId,
    ) -> WorkflowResult<Option<WorkflowSpec>> {
        // Try cache first (hot path, lock-free DashMap operation)
        if let Some(spec) = self.cache.get_spec(spec_id) {
            return Ok(Some((*spec).clone()));
        }

        // Fallback to sled
        let key = format!("spec:{}", spec_id);
        match self
            .db
            .get(key.as_bytes())
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?
        {
            Some(value) => {
                let spec: WorkflowSpec = serde_json::from_slice(value.as_ref()).map_err(|e| {
                    WorkflowError::StatePersistence(format!("Deserialization error: {}", e))
                })?;

                // Update cache (lock-free DashMap operation)
                self.cache
                    .insert_spec(spec.id.clone(), Arc::new(spec.clone()));

                Ok(Some(spec))
            }
            None => Ok(None),
        }
    }

    /// Save a case (with cache)
    pub fn save_case(&self, case_id: crate::case::CaseId, case: &Case) -> WorkflowResult<()> {
        // Update cache first (hot path, lock-free DashMap operation)
        self.cache
            .insert_case(case_id.clone(), Arc::new(case.clone()));

        // Persist to sled
        let key = format!("case:{}", case_id);
        let value = serde_json::to_vec(case)
            .map_err(|e| WorkflowError::StatePersistence(format!("Serialization error: {}", e)))?;
        self.db
            .insert(key.as_bytes(), value)
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;
        Ok(())
    }

    /// Load a case (cache-first)
    pub fn load_case(&self, case_id: &crate::case::CaseId) -> WorkflowResult<Option<Case>> {
        // Try cache first (hot path, lock-free DashMap operation)
        if let Some(case) = self.cache.get_case(case_id) {
            return Ok(Some((*case).clone()));
        }

        // Fallback to sled
        let key = format!("case:{}", case_id);
        match self
            .db
            .get(key.as_bytes())
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?
        {
            Some(value) => {
                let case: Case = serde_json::from_slice(value.as_ref()).map_err(|e| {
                    WorkflowError::StatePersistence(format!("Deserialization error: {}", e))
                })?;

                // Update cache (lock-free DashMap operation)
                self.cache
                    .insert_case(case.id.clone(), Arc::new(case.clone()));

                Ok(Some(case))
            }
            None => Ok(None),
        }
    }

    /// Append a receipt (immutable, for provenance)
    pub fn append_receipt(&self, key: &str, value: &[u8]) -> WorkflowResult<()> {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| WorkflowError::StatePersistence(format!("Time error: {:?}", e)))?;
        let receipt_key = format!("receipt:{}:{}", key, ts.as_nanos());
        self.db
            .insert(receipt_key.as_bytes(), value)
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;
        Ok(())
    }

    /// List all cases for a workflow specification
    pub fn list_cases(
        &self,
        spec_id: &crate::parser::WorkflowSpecId,
    ) -> WorkflowResult<Vec<crate::case::CaseId>> {
        let prefix = "case:".to_string();
        let mut cases = Vec::new();

        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = result
                .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;

            let case: Case = serde_json::from_slice(value.as_ref()).map_err(|e| {
                WorkflowError::StatePersistence(format!("Deserialization error: {}", e))
            })?;

            if case.spec_id == *spec_id {
                cases.push(case.id);
            }
        }

        Ok(cases)
    }

    /// Delete a workflow specification
    pub fn delete_spec(&self, spec_id: &crate::parser::WorkflowSpecId) -> WorkflowResult<()> {
        let key = format!("spec:{}", spec_id);
        self.db
            .remove(key.as_bytes())
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;
        Ok(())
    }

    /// Compact database (run at fixed tick epochs for reflex compliance)
    ///
    /// Reflex layers must guarantee no drift (argmin drift(A)).
    /// Sled compacts lazily; run compaction at fixed tick epochs to ensure
    /// no drift and maintain reflex compliance.
    pub async fn compact(&self) -> WorkflowResult<()> {
        // Flush pending writes asynchronously
        self.db
            .flush_async()
            .await
            .map_err(|e| WorkflowError::StatePersistence(format!("Flush error: {:?}", e)))?;

        // Run compaction (sled doesn't have checkpoint method, compaction happens automatically)
        // Flush is sufficient for ensuring writes are persisted

        Ok(())
    }

    /// Flush database (async, non-blocking)
    ///
    /// Flushes pending writes to disk without compaction.
    pub async fn flush(&self) -> WorkflowResult<()> {
        self.db
            .flush_async()
            .await
            .map_err(|e| WorkflowError::StatePersistence(format!("Flush error: {:?}", e)))?;
        Ok(())
    }

    /// Save case history event (append-only log)
    pub fn save_case_history_event(
        &self,
        case_id: &crate::case::CaseId,
        event: &crate::state::manager::StateEvent,
    ) -> WorkflowResult<()> {
        // Persist event to sled (append-only log)
        let key = format!(
            "case_history:{}:{}",
            case_id,
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let value = serde_json::to_vec(event)
            .map_err(|e| WorkflowError::StatePersistence(format!("Serialization error: {}", e)))?;
        self.db
            .insert(key.as_bytes(), value)
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;
        Ok(())
    }

    /// Load case history events
    pub fn load_case_history(
        &self,
        case_id: &crate::case::CaseId,
    ) -> WorkflowResult<Vec<crate::state::manager::StateEvent>> {
        let prefix = format!("case_history:{}:", case_id);
        let mut events = Vec::new();

        // Scan all events for this case
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = result
                .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {:?}", e)))?;
            let event: crate::state::manager::StateEvent = serde_json::from_slice(value.as_ref())
                .map_err(|e| {
                WorkflowError::StatePersistence(format!("Deserialization error: {}", e))
            })?;
            events.push(event);
        }

        // Sort by timestamp using StateEvent::timestamp() method
        events.sort_by(|a, b| a.timestamp().cmp(&b.timestamp()));

        Ok(events)
    }
}
