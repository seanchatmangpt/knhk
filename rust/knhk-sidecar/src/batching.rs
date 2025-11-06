// rust/knhk-sidecar/src/batching.rs
// Request batching for grouping multiple operations

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct Batch<T> {
    pub items: Vec<T>,
    pub created_at: Instant,
}

pub struct Batcher<T> {
    batch_size: usize,
    timeout: Duration,
    sender: mpsc::UnboundedSender<Batch<T>>,
    current_batch: Arc<Mutex<Option<Batch<T>>>>,
}

impl<T> Batcher<T>
where
    T: Send + 'static,
{
    pub fn new(batch_size: usize, timeout: Duration) -> (Self, mpsc::UnboundedReceiver<Batch<T>>) {
        let (sender, receiver) = mpsc::unbounded_channel();

        let batcher = Self {
            batch_size,
            timeout,
            sender,
            current_batch: Arc::new(Mutex::new(None)),
        };

        (batcher, receiver)
    }

    pub fn add(&self, item: T) -> Result<(), crate::error::SidecarError> {
        let mut batch = self.current_batch.lock()
            .map_err(|e| crate::error::SidecarError::internal_error(
                format!("Failed to acquire batch lock: {}", e)
            ))?;

        let now = Instant::now();

        let should_flush = if let Some(ref b) = *batch {
            b.items.len() >= self.batch_size || now.duration_since(b.created_at) >= self.timeout
        } else {
            false
        };

        if should_flush {
            if let Some(b) = batch.take() {
                self.sender.send(b)
                    .map_err(|e| crate::error::SidecarError::Internal(
                        format!("Failed to send batch: {}", e)
                    ))?;
            }
        }

        if batch.is_none() {
            *batch = Some(Batch {
                items: Vec::with_capacity(self.batch_size),
                created_at: now,
            });
        }

        if let Some(ref mut b) = *batch {
            b.items.push(item);

            if b.items.len() >= self.batch_size {
                let batch_to_send = batch.take().expect("Batch should exist when items.len() >= batch_size");
                self.sender.send(batch_to_send)
                    .map_err(|e| crate::error::SidecarError::Internal(
                        format!("Failed to send batch: {}", e)
                    ))?;
            }
        }

        Ok(())
    }

    pub fn flush(&self) -> Result<(), crate::error::SidecarError> {
        let mut batch = self.current_batch.lock()
            .map_err(|e| crate::error::SidecarError::internal_error(
                format!("Failed to acquire batch lock: {}", e)
            ))?;

        if let Some(b) = batch.take() {
            if !b.items.is_empty() {
                self.sender.send(b)
                    .map_err(|e| crate::error::SidecarError::Internal(
                        format!("Failed to send batch: {}", e)
                    ))?;
            }
        }

        Ok(())
    }
}

