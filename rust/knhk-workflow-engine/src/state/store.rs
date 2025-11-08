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
    cache: Arc<ReflexCache>,
}

impl StateStore {
    /// Create a new state store
    pub fn new<P: AsRef<Path>>(path: P) -> WorkflowResult<Self> {
        let db = sled::open(path).map_err(|e| {
            WorkflowError::StatePersistence(format!("Failed to open database: {:?}", e))
        })?;
        Ok(Self {
            db,
            cache: Arc::new(ReflexCache::new()),
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
        // Update cache first (hot path)
        let cache = self.cache.blocking_read();
        cache.insert_case(case_id.clone(), Arc::new(case.clone()));
        drop(cache);

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
        // Try cache first (hot path)
        let cache = self.cache.blocking_read();
        if let Some(case) = cache.get_case(case_id) {
            return Ok(Some((*case).clone()));
        }
        drop(cache);

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

                // Update cache
                let cache = self.cache.blocking_write();
                cache.insert_case(case.id.clone(), Arc::new(case.clone()));
                drop(cache);

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
}
