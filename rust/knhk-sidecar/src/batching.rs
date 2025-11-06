// rust/knhk-sidecar/src/batching.rs
// Request batching manager

use crate::error::{Result, SidecarError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Batched transaction request
pub type BatchedTransaction = Vec<u8>; // Serialized Transaction protobuf

/// Batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size (guard: ≤ 8)
    pub max_batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 8,
            batch_timeout_ms: 100,
        }
    }
}

impl BatchConfig {
    /// Validate batch configuration
    pub fn validate(&self) -> Result<()> {
        if self.max_batch_size > 8 {
            return Err(SidecarError::ValidationFailed(
                format!("max_batch_size {} exceeds guard constraint (8)", self.max_batch_size)
            ));
        }
        if self.max_batch_size == 0 {
            return Err(SidecarError::ValidationFailed(
                "max_batch_size must be > 0".to_string()
            ));
        }
        Ok(())
    }
}

/// Batch accumulator
pub struct BatchAccumulator {
    config: BatchConfig,
    batch: Vec<BatchedTransaction>,
    last_flush: Instant,
    flush_tx: mpsc::Sender<Vec<BatchedTransaction>>,
}

impl BatchAccumulator {
    /// Create new batch accumulator
    pub fn new(
        config: BatchConfig,
        flush_tx: mpsc::Sender<Vec<BatchedTransaction>>,
    ) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            batch: Vec::new(),
            last_flush: Instant::now(),
            flush_tx,
        })
    }

    /// Add transaction to batch
    pub fn add(&mut self, transaction: BatchedTransaction) -> Result<()> {
        // Check if batch is full
        if self.batch.len() >= self.config.max_batch_size {
            self.flush()?;
        }
        
        self.batch.push(transaction);
        Ok(())
    }

    /// Flush batch if timeout exceeded
    pub fn check_timeout(&mut self) -> Result<()> {
        let elapsed = self.last_flush.elapsed();
        let timeout = Duration::from_millis(self.config.batch_timeout_ms);
        
        if elapsed >= timeout && !self.batch.is_empty() {
            self.flush()?;
        }
        
        Ok(())
    }

    /// Flush current batch
    pub fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }
        
        let batch = std::mem::take(&mut self.batch);
        self.last_flush = Instant::now();
        
        // Send batch to flush channel (non-blocking)
        self.flush_tx.try_send(batch)
            .map_err(|e| SidecarError::BatchError(
                format!("Failed to send batch: {}", e)
            ))?;
        
        Ok(())
    }

    /// Get current batch size
    pub fn len(&self) -> usize {
        self.batch.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }
}

/// Batch manager - coordinates batching and flushing
pub struct BatchManager {
    accumulator: Arc<Mutex<BatchAccumulator>>,
    config: BatchConfig,
}

impl BatchManager {
    /// Create new batch manager
    pub fn new(config: BatchConfig) -> Result<(Self, mpsc::Receiver<Vec<BatchedTransaction>>)> {
        config.validate()?;
        
        let (flush_tx, flush_rx) = mpsc::channel(100);
        let accumulator = Arc::new(Mutex::new(
            BatchAccumulator::new(config.clone(), flush_tx)?
        ));
        
        let manager = Self {
            accumulator,
            config,
        };
        
        Ok((manager, flush_rx))
    }

    /// Add transaction to batch
    pub fn add(&self, transaction: BatchedTransaction) -> Result<()> {
        let mut acc = self.accumulator.lock()
            .map_err(|e| SidecarError::InternalError(
                format!("Failed to lock accumulator: {}", e)
            ))?;
        
        acc.add(transaction)
    }

    /// Start background flush task
    pub fn start_flush_task(&self) -> tokio::task::JoinHandle<()> {
        let accumulator = Arc::clone(&self.accumulator);
        let timeout_ms = self.config.batch_timeout_ms;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(timeout_ms));
            
            loop {
                interval.tick().await;
                
                if let Ok(mut acc) = accumulator.lock() {
                    let _ = acc.check_timeout();
                }
            }
        })
    }

    /// Force flush current batch
    pub fn flush(&self) -> Result<()> {
        let mut acc = self.accumulator.lock()
            .map_err(|e| SidecarError::InternalError(
                format!("Failed to lock accumulator: {}", e)
            ))?;
        
        acc.flush()
    }

    /// Get current batch size
    pub fn len(&self) -> usize {
        if let Ok(acc) = self.accumulator.lock() {
            acc.len()
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_accumulation() {
        let config = BatchConfig::default();
        let (manager, mut flush_rx) = BatchManager::new(config).unwrap();
        
        // Add transactions
        for i in 0..5 {
            manager.add(vec![i]).unwrap();
        }
        
        assert_eq!(manager.len(), 5);
        
        // Flush
        manager.flush().unwrap();
        
        // Receive batch
        let batch = flush_rx.recv().await.unwrap();
        assert_eq!(batch.len(), 5);
    }

    #[tokio::test]
    async fn test_batch_full_flush() {
        let config = BatchConfig {
            max_batch_size: 3,
            batch_timeout_ms: 1000,
        };
        let (manager, mut flush_rx) = BatchManager::new(config).unwrap();
        
        // Add transactions up to batch size
        for i in 0..3 {
            manager.add(vec![i]).unwrap();
        }
        
        // Next add should trigger flush
        manager.add(vec![3]).unwrap();
        
        // Receive first batch
        let batch1 = flush_rx.recv().await.unwrap();
        assert_eq!(batch1.len(), 3);
        
        // Check remaining batch
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_batch_config_validation() {
        // Valid config
        let config = BatchConfig {
            max_batch_size: 8,
            batch_timeout_ms: 100,
        };
        assert!(config.validate().is_ok());
        
        // Invalid: exceeds guard constraint
        let config = BatchConfig {
            max_batch_size: 9,
            batch_timeout_ms: 100,
        };
        assert!(config.validate().is_err());
        
        // Invalid: zero batch size
        let config = BatchConfig {
            max_batch_size: 0,
            batch_timeout_ms: 100,
        };
        assert!(config.validate().is_err());
    }
}

