//! YAWL Case Number Store Implementation
//!
//! Implements YCaseNbrStore from YAWL Java
//! - Manages case number generation and persistence
//! - Thread-safe case number allocation
//!
//! Based on: org.yawlfoundation.yawl.engine.YCaseNbrStore

use crate::error::{WorkflowError, WorkflowResult};
use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Case number store
///
/// Manages case number generation with persistence support
pub struct CaseStore {
    /// Current case number counter
    case_counter: Arc<AtomicU32>,
    /// Case number to case ID mapping (for persistence)
    case_number_map: Arc<DashMap<u32, String>>,
    /// Case ID to case number mapping
    case_id_map: Arc<DashMap<String, u32>>,
    /// Next case number (for allocation)
    next_case_number: Arc<RwLock<u32>>,
}

impl CaseStore {
    /// Create a new case store
    pub fn new() -> Self {
        Self {
            case_counter: Arc::new(AtomicU32::new(1)),
            case_number_map: Arc::new(DashMap::new()),
            case_id_map: Arc::new(DashMap::new()),
            next_case_number: Arc::new(RwLock::new(1)),
        }
    }

    /// Generate next case number
    pub async fn generate_case_number(&self) -> u32 {
        let number = self.case_counter.fetch_add(1, Ordering::SeqCst);
        number
    }

    /// Register a case with a case number
    pub async fn register_case(&self, case_id: String, case_number: u32) -> WorkflowResult<()> {
        self.case_number_map.insert(case_number, case_id.clone());
        self.case_id_map.insert(case_id, case_number);
        Ok(())
    }

    /// Get case ID by case number
    pub fn get_case_id(&self, case_number: u32) -> Option<String> {
        self.case_number_map
            .get(&case_number)
            .map(|entry| entry.clone())
    }

    /// Get case number by case ID
    pub fn get_case_number(&self, case_id: &str) -> Option<u32> {
        self.case_id_map.get(case_id).map(|entry| *entry)
    }

    /// Get current case number
    pub fn get_current_case_number(&self) -> u32 {
        self.case_counter.load(Ordering::SeqCst)
    }

    /// Set case number (for restoration from persistence)
    pub async fn set_case_number(&self, case_number: u32) -> WorkflowResult<()> {
        let mut next = self.next_case_number.write().await;
        *next = case_number.max(*next);
        self.case_counter.store(
            case_number.max(self.case_counter.load(Ordering::SeqCst)),
            Ordering::SeqCst,
        );
        Ok(())
    }
}

impl Default for CaseStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_case_number_generation() {
        let store = CaseStore::new();
        let number1 = store.generate_case_number().await;
        let number2 = store.generate_case_number().await;
        assert_eq!(number1, 1);
        assert_eq!(number2, 2);
    }

    #[tokio::test]
    async fn test_case_registration() {
        let store = CaseStore::new();
        let case_number = store.generate_case_number().await;
        store
            .register_case("case-123".to_string(), case_number)
            .await
            .unwrap();

        assert_eq!(store.get_case_id(case_number), Some("case-123".to_string()));
        assert_eq!(store.get_case_number("case-123"), Some(case_number));
    }
}
