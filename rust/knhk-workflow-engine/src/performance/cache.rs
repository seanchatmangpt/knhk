//! Caching layer for workflow engine

use crate::case::{Case, CaseId};
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tracing::warn;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum workflow spec cache size
    pub max_workflow_specs: usize,
    /// Maximum case cache size
    pub max_cases: usize,
    /// Maximum pattern result cache size
    pub max_pattern_results: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_workflow_specs: 1000,
            max_cases: 10000,
            max_pattern_results: 10000,
        }
    }
}

/// Workflow cache
pub struct WorkflowCache {
    workflow_specs: Arc<Mutex<LruCache<WorkflowSpecId, Arc<WorkflowSpec>>>>,
    cases: Arc<Mutex<LruCache<CaseId, Arc<Case>>>>,
    config: CacheConfig,
}

impl WorkflowCache {
    /// Create a new workflow cache
    pub fn new(config: CacheConfig) -> WorkflowResult<Self> {
        let max_specs = NonZeroUsize::new(config.max_workflow_specs).ok_or_else(|| {
            WorkflowError::Validation("max_workflow_specs must be greater than 0".to_string())
        })?;
        let max_cases = NonZeroUsize::new(config.max_cases).ok_or_else(|| {
            WorkflowError::Validation("max_cases must be greater than 0".to_string())
        })?;

        Ok(Self {
            workflow_specs: Arc::new(Mutex::new(LruCache::new(max_specs))),
            cases: Arc::new(Mutex::new(LruCache::new(max_cases))),
            config,
        })
    }

    /// Get workflow spec from cache
    pub fn get_workflow_spec(&self, spec_id: &WorkflowSpecId) -> Option<Arc<WorkflowSpec>> {
        self.workflow_specs
            .lock()
            .ok()
            .and_then(|mut cache| cache.get(spec_id).cloned())
    }

    /// Put workflow spec in cache
    pub fn put_workflow_spec(&self, spec: WorkflowSpec) {
        if let Ok(mut cache) = self.workflow_specs.lock() {
            cache.put(spec.id, Arc::new(spec));
        } else {
            warn!("Failed to acquire workflow_specs lock in put_workflow_spec");
        }
    }

    /// Get case from cache
    pub fn get_case(&self, case_id: &CaseId) -> Option<Arc<Case>> {
        self.cases
            .lock()
            .ok()
            .and_then(|mut cache| cache.get(case_id).cloned())
    }

    /// Put case in cache
    pub fn put_case(&self, case: Case) {
        if let Ok(mut cache) = self.cases.lock() {
            cache.put(case.id, Arc::new(case));
        } else {
            warn!("Failed to acquire cases lock in put_case");
        }
    }

    /// Invalidate workflow spec cache
    pub fn invalidate_workflow_spec(&self, spec_id: &WorkflowSpecId) {
        if let Ok(mut cache) = self.workflow_specs.lock() {
            cache.pop(spec_id);
        } else {
            warn!("Failed to acquire workflow_specs lock in invalidate_workflow_spec");
        }
    }

    /// Invalidate case cache
    pub fn invalidate_case(&self, case_id: &CaseId) {
        if let Ok(mut cache) = self.cases.lock() {
            cache.pop(case_id);
        } else {
            warn!("Failed to acquire cases lock in invalidate_case");
        }
    }

    /// Clear all caches
    pub fn clear(&self) {
        if let Ok(mut specs) = self.workflow_specs.lock() {
            specs.clear();
        } else {
            warn!("Failed to acquire workflow_specs lock in clear");
        }
        if let Ok(mut cases) = self.cases.lock() {
            cases.clear();
        } else {
            warn!("Failed to acquire cases lock in clear");
        }
    }
}

impl Default for WorkflowCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
            .expect("Default CacheConfig should have valid non-zero values")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_cache() {
        let cache = WorkflowCache::default();
        let spec = WorkflowSpec {
            id: WorkflowSpecId::new(),
            name: "test".to_string(),
            tasks: std::collections::HashMap::new(),
            conditions: std::collections::HashMap::new(),
            start_condition: None,
            end_condition: None,
        };

        cache.put_workflow_spec(spec.clone());
        let cached = cache.get_workflow_spec(&spec.id);
        assert!(cached.is_some());
    }
}
