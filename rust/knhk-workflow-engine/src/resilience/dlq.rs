#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Dead letter queue for failed workflow operations

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Dead letter queue entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DLQEntry {
    /// Unique entry ID
    pub id: Uuid,
    /// Case ID (if applicable)
    pub case_id: Option<CaseId>,
    /// Workflow spec ID
    pub spec_id: Option<WorkflowSpecId>,
    /// Error that caused the failure
    pub error: String,
    /// Error type
    pub error_type: String,
    /// Failed operation data
    pub operation_data: serde_json::Value,
    /// Timestamp when entry was created
    pub created_at: DateTime<Utc>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Next retry timestamp
    pub next_retry_at: Option<DateTime<Utc>>,
}

/// Dead letter queue for failed operations
pub struct DeadLetterQueue {
    entries: Arc<Mutex<VecDeque<DLQEntry>>>,
    max_size: usize,
}

impl DeadLetterQueue {
    /// Create a new dead letter queue
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }

    /// Add an entry to the DLQ
    pub fn add(
        &self,
        case_id: Option<CaseId>,
        spec_id: Option<WorkflowSpecId>,
        error: WorkflowError,
        operation_data: serde_json::Value,
    ) -> WorkflowResult<Uuid> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| WorkflowError::Internal(format!("Failed to acquire DLQ lock: {}", e)))?;

        // Remove oldest entries if at capacity
        while entries.len() >= self.max_size {
            entries.pop_front();
        }

        let entry = DLQEntry {
            id: Uuid::new_v4(),
            case_id,
            spec_id,
            error: error.to_string(),
            error_type: format!("{:?}", error),
            operation_data,
            created_at: Utc::now(),
            retry_count: 0,
            next_retry_at: None,
        };

        let id = entry.id;
        entries.push_back(entry);

        Ok(id)
    }

    /// Get entries ready for retry
    pub fn get_retryable_entries(&self) -> WorkflowResult<Vec<DLQEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| WorkflowError::Internal(format!("Failed to acquire DLQ lock: {}", e)))?;

        let now = Utc::now();
        let retryable: Vec<DLQEntry> = entries
            .iter()
            .filter(|entry| {
                entry.retry_count < 3
                    && entry
                        .next_retry_at
                        .map(|retry_at| retry_at <= now)
                        .unwrap_or(true)
            })
            .cloned()
            .collect();

        Ok(retryable)
    }

    /// Remove an entry from the DLQ
    pub fn remove(&self, id: Uuid) -> WorkflowResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| WorkflowError::Internal(format!("Failed to acquire DLQ lock: {}", e)))?;

        entries.retain(|entry| entry.id != id);
        Ok(())
    }

    /// Get all entries
    pub fn list(&self) -> WorkflowResult<Vec<DLQEntry>> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| WorkflowError::Internal(format!("Failed to acquire DLQ lock: {}", e)))?;

        Ok(entries.iter().cloned().collect())
    }

    /// Get entry count
    pub fn len(&self) -> WorkflowResult<usize> {
        let entries = self
            .entries
            .lock()
            .map_err(|e| WorkflowError::Internal(format!("Failed to acquire DLQ lock: {}", e)))?;

        Ok(entries.len())
    }

    /// Check if DLQ is empty
    pub fn is_empty(&self) -> WorkflowResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Update retry count and next retry time
    pub fn update_retry(&self, id: Uuid, next_retry_at: DateTime<Utc>) -> WorkflowResult<()> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|e| WorkflowError::Internal(format!("Failed to acquire DLQ lock: {}", e)))?;

        if let Some(entry) = entries.iter_mut().find(|e| e.id == id) {
            entry.retry_count += 1;
            entry.next_retry_at = Some(next_retry_at);
        }

        Ok(())
    }
}

impl Default for DeadLetterQueue {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlq_add_entry() {
        let dlq = DeadLetterQueue::new(100);
        let error = WorkflowError::Timeout;

        let id = dlq.add(None, None, error, serde_json::json!({})).unwrap();

        assert_eq!(dlq.len().unwrap(), 1);
        let entries = dlq.list().unwrap();
        assert_eq!(entries[0].id, id);
    }

    #[test]
    fn test_dlq_max_size() {
        let dlq = DeadLetterQueue::new(2);
        let error = WorkflowError::Timeout;

        dlq.add(None, None, error.clone(), serde_json::json!({}))
            .unwrap();
        dlq.add(None, None, error.clone(), serde_json::json!({}))
            .unwrap();
        let id3 = dlq.add(None, None, error, serde_json::json!({})).unwrap();

        assert_eq!(dlq.len().unwrap(), 2);
        let entries = dlq.list().unwrap();
        assert!(entries.iter().any(|e| e.id == id3));
    }
}
