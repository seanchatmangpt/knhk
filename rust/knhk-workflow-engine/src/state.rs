//! State persistence for workflow engine

use crate::case::Case;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use sled::Db;
use std::path::Path;

/// State store for workflow engine
pub struct StateStore {
    db: Db,
}

impl StateStore {
    /// Create a new state store
    pub fn new<P: AsRef<Path>>(path: P) -> WorkflowResult<Self> {
        let db = sled::open(path)
            .map_err(|e| WorkflowError::StatePersistence(format!("Failed to open database: {}", e)))?;
        Ok(Self { db })
    }

    /// Save a workflow specification
    pub fn save_spec(&self, spec: &WorkflowSpec) -> WorkflowResult<()> {
        let key = format!("spec:{}", spec.id);
        let value = serde_json::to_vec(spec)
            .map_err(|e| WorkflowError::StatePersistence(format!("Serialization error: {}", e)))?;
        self.db
            .insert(key.as_bytes(), value)
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {}", e)))?;
        Ok(())
    }

    /// Load a workflow specification
    pub fn load_spec(&self, spec_id: &crate::parser::WorkflowSpecId) -> WorkflowResult<Option<WorkflowSpec>> {
        let key = format!("spec:{}", spec_id);
        match self.db.get(key.as_bytes())
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {}", e)))?
        {
            Some(value) => {
                let spec: WorkflowSpec = serde_json::from_slice(&value)
                    .map_err(|e| WorkflowError::StatePersistence(format!("Deserialization error: {}", e)))?;
                Ok(Some(spec))
            }
            None => Ok(None),
        }
    }

    /// Save a case
    pub fn save_case(&self, case_id: crate::case::CaseId, case: &Case) -> WorkflowResult<()> {
        let key = format!("case:{}", case_id);
        let value = serde_json::to_vec(case)
            .map_err(|e| WorkflowError::StatePersistence(format!("Serialization error: {}", e)))?;
        self.db
            .insert(key.as_bytes(), value)
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {}", e)))?;
        Ok(())
    }

    /// Load a case
    pub fn load_case(&self, case_id: &crate::case::CaseId) -> WorkflowResult<Option<Case>> {
        let key = format!("case:{}", case_id);
        match self.db.get(key.as_bytes())
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {}", e)))?
        {
            Some(value) => {
                let case: Case = serde_json::from_slice(&value)
                    .map_err(|e| WorkflowError::StatePersistence(format!("Deserialization error: {}", e)))?;
                Ok(Some(case))
            }
            None => Ok(None),
        }
    }

    /// List all cases for a workflow specification
    pub fn list_cases(&self, spec_id: &crate::parser::WorkflowSpecId) -> WorkflowResult<Vec<crate::case::CaseId>> {
        let prefix = format!("case:");
        let mut cases = Vec::new();
        
        for result in self.db.scan_prefix(prefix.as_bytes())
            .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {}", e)))?
        {
            let (key, value) = result
                .map_err(|e| WorkflowError::StatePersistence(format!("Database error: {}", e)))?;
            
            let case: Case = serde_json::from_slice(&value)
                .map_err(|e| WorkflowError::StatePersistence(format!("Deserialization error: {}", e)))?;
            
            if case.spec_id == *spec_id {
                cases.push(case.id);
            }
        }
        
        Ok(cases)
    }
}

