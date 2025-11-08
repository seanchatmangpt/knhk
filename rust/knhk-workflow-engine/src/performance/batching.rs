#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Batch processing for workflow operations

use crate::case::CaseId;
use crate::error::WorkflowResult;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Maximum wait time before flushing (milliseconds)
    pub max_wait_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_wait_ms: 10,
        }
    }
}

/// Batch item
#[derive(Debug, Clone)]
pub struct BatchItem {
    /// Item ID
    pub id: uuid::Uuid,
    /// Item data
    pub data: Value,
    /// Timestamp
    pub timestamp: Instant,
}

/// Batch processor
pub struct BatchProcessor {
    config: BatchConfig,
    batch: Arc<Mutex<Vec<BatchItem>>>,
    last_flush: Arc<Mutex<Instant>>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            batch: Arc::new(Mutex::new(Vec::new())),
            last_flush: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Add item to batch
    pub fn add_item(&self, data: Value) -> WorkflowResult<uuid::Uuid> {
        let item = BatchItem {
            id: uuid::Uuid::new_v4(),
            data,
            timestamp: Instant::now(),
        };

        let id = item.id;
        let mut batch = self.batch.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire batch lock: {}", e))
        })?;

        batch.push(item);

        // Check if batch should be flushed
        if batch.len() >= self.config.max_batch_size {
            drop(batch);
            self.flush()?;
        }

        Ok(id)
    }

    /// Flush batch
    pub fn flush(&self) -> WorkflowResult<Vec<BatchItem>> {
        let mut batch = self.batch.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire batch lock: {}", e))
        })?;

        let items = batch.drain(..).collect();
        let mut last_flush = self.last_flush.lock().unwrap();
        *last_flush = Instant::now();

        Ok(items)
    }

    /// Check if batch should be flushed (time-based)
    pub fn should_flush(&self) -> bool {
        let batch = self.batch.lock().unwrap();
        let last_flush = self.last_flush.lock().unwrap();

        !batch.is_empty() && last_flush.elapsed() >= Duration::from_millis(self.config.max_wait_ms)
    }

    /// Get current batch size
    pub fn batch_size(&self) -> usize {
        let batch = self.batch.lock().unwrap();
        batch.len()
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new(BatchConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor() {
        let processor = BatchProcessor::default();
        processor.add_item(serde_json::json!({})).unwrap();
        assert_eq!(processor.batch_size(), 1);
    }
}
