#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Caching layer for workflow engine

use crate::case::{Case, CaseId};
use crate::error::WorkflowResult;
use crate::parser::{WorkflowSpec, WorkflowSpecId};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

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
    pub fn new(config: CacheConfig) -> Self {
        Self {
            workflow_specs: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(config.max_workflow_specs).unwrap(),
            ))),
            cases: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(config.max_cases).unwrap(),
            ))),
            config,
        }
    }

    /// Get workflow spec from cache
    pub fn get_workflow_spec(&self, spec_id: &WorkflowSpecId) -> Option<Arc<WorkflowSpec>> {
        let mut cache = self.workflow_specs.lock().unwrap();
        cache.get(spec_id).cloned()
    }

    /// Put workflow spec in cache
    pub fn put_workflow_spec(&self, spec: WorkflowSpec) {
        let mut cache = self.workflow_specs.lock().unwrap();
        cache.put(spec.id, Arc::new(spec));
    }

    /// Get case from cache
    pub fn get_case(&self, case_id: &CaseId) -> Option<Arc<Case>> {
        let mut cache = self.cases.lock().unwrap();
        cache.get(case_id).cloned()
    }

    /// Put case in cache
    pub fn put_case(&self, case: Case) {
        let mut cache = self.cases.lock().unwrap();
        cache.put(case.id, Arc::new(case));
    }

    /// Invalidate workflow spec cache
    pub fn invalidate_workflow_spec(&self, spec_id: &WorkflowSpecId) {
        let mut cache = self.workflow_specs.lock().unwrap();
        cache.pop(spec_id);
    }

    /// Invalidate case cache
    pub fn invalidate_case(&self, case_id: &CaseId) {
        let mut cache = self.cases.lock().unwrap();
        cache.pop(case_id);
    }

    /// Clear all caches
    pub fn clear(&self) {
        let mut specs = self.workflow_specs.lock().unwrap();
        specs.clear();
        let mut cases = self.cases.lock().unwrap();
        cases.clear();
    }
}

impl Default for WorkflowCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
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
