// knhk-sidecar: Request batching logic

// ACCEPTABLE: Mutex poisoning .expect() is allowed in this module (unrecoverable error)
#![allow(clippy::expect_used)]

use crate::error::{SidecarError, SidecarResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration, Instant};

/// Batched request with response channel
pub struct BatchedRequest<T, R> {
    pub request: T,
    pub response_tx: oneshot::Sender<SidecarResult<R>>,
}

/// Batch configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Batch window in milliseconds
    pub batch_window_ms: u64,

    /// Maximum batch size
    pub max_batch_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_window_ms: 10,
            max_batch_size: 100,
        }
    }
}

/// Batch collector for grouping requests
pub struct BatchCollector<T, R> {
    config: BatchConfig,
    pending: Arc<Mutex<Vec<BatchedRequest<T, R>>>>,
}

impl<T, R> BatchCollector<T, R> {
    /// Create new batch collector
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            pending: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add request to batch
    pub fn add_request(&self, request: T) -> oneshot::Receiver<SidecarResult<R>> {
        let (tx, rx) = oneshot::channel();

        // ACCEPTABLE: Mutex poisoning is an unrecoverable error. Panicking is appropriate.
        // See Rust std docs: https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
        let mut pending = self
            .pending
            .lock()
            .expect("Batch pending mutex poisoned - unrecoverable state");
        pending.push(BatchedRequest {
            request,
            response_tx: tx,
        });

        rx
    }

    /// Collect batch (non-blocking, returns immediately if batch is ready)
    pub fn collect_batch(&self) -> Option<Vec<BatchedRequest<T, R>>> {
        let mut pending = self
            .pending
            .lock()
            .expect("Batch pending mutex poisoned - unrecoverable state");

        if pending.is_empty() {
            return None;
        }

        // Check if batch is ready (size limit reached)
        if pending.len() >= self.config.max_batch_size {
            let batch = std::mem::take(&mut *pending);
            return Some(batch);
        }

        None
    }

    /// Collect batch with timeout (waits for batch window or max size)
    pub async fn collect_batch_with_timeout(&self) -> Vec<BatchedRequest<T, R>> {
        let start = Instant::now();
        let timeout = Duration::from_millis(self.config.batch_window_ms);

        loop {
            // Check if batch is ready
            if let Some(batch) = self.collect_batch() {
                return batch;
            }

            // Check if timeout reached
            if start.elapsed() >= timeout {
                let mut pending = self
                    .pending
                    .lock()
                    .expect("Batch pending mutex poisoned - unrecoverable state");
                if !pending.is_empty() {
                    return std::mem::take(&mut *pending);
                }
                return Vec::new();
            }

            // Wait a bit before checking again
            sleep(Duration::from_millis(1)).await;
        }
    }

    /// Get current pending count
    pub fn pending_count(&self) -> usize {
        self.pending
            .lock()
            .expect("Batch pending mutex poisoned - unrecoverable state")
            .len()
    }
}

/// Batch processor for handling batched requests
pub struct BatchProcessor<T, R> {
    config: BatchConfig,
    processor: Arc<
        dyn Fn(
                Vec<T>,
            )
                -> std::pin::Pin<Box<dyn std::future::Future<Output = SidecarResult<Vec<R>>> + Send>>
            + Send
            + Sync,
    >,
}

impl<T, R> BatchProcessor<T, R>
where
    T: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    /// Create new batch processor
    pub fn new<F, Fut>(config: BatchConfig, processor: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = SidecarResult<Vec<R>>> + Send + 'static,
    {
        Self {
            config,
            processor: Arc::new(move |requests| Box::pin(processor(requests))),
        }
    }

    /// Process batch of requests
    pub async fn process_batch(&self, requests: Vec<T>) -> SidecarResult<Vec<R>> {
        // Validate batch size
        if requests.len() > self.config.max_batch_size {
            return Err(SidecarError::BatchError {
                context: crate::error::ErrorContext::new(
                    "SIDECAR_BATCH_SIZE_EXCEEDED",
                    format!(
                        "Batch size {} exceeds maximum {}",
                        requests.len(),
                        self.config.max_batch_size
                    ),
                ),
            });
        }

        // Process batch
        (self.processor)(requests).await
    }
}

/// Batch manager for coordinating collection and processing
pub struct BatchManager<T, R> {
    collector: Arc<BatchCollector<T, R>>,
    processor: Arc<BatchProcessor<T, R>>,
}

impl<T, R> BatchManager<T, R>
where
    T: Send + Sync + Clone + 'static,
    R: Send + Sync + 'static,
{
    /// Create new batch manager
    pub fn new<F, Fut>(config: BatchConfig, processor: F) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = SidecarResult<Vec<R>>> + Send + 'static,
    {
        Self {
            collector: Arc::new(BatchCollector::new(config.clone())),
            processor: Arc::new(BatchProcessor::new(config, processor)),
        }
    }

    /// Submit request for batching
    pub fn submit(&self, request: T) -> oneshot::Receiver<SidecarResult<R>> {
        self.collector.add_request(request)
    }

    /// Process batches continuously
    pub async fn process_loop(&self) -> SidecarResult<()> {
        loop {
            // Collect batch with timeout
            let batched_requests = self.collector.collect_batch_with_timeout().await;

            if batched_requests.is_empty() {
                continue;
            }

            // Extract requests
            let requests: Vec<T> = batched_requests
                .iter()
                .map(|br| br.request.clone())
                .collect();
            let response_channels: Vec<oneshot::Sender<SidecarResult<R>>> = batched_requests
                .into_iter()
                .map(|br| br.response_tx)
                .collect();

            // Process batch
            match self.processor.process_batch(requests).await {
                Ok(responses) => {
                    // Send responses back
                    for (tx, response) in response_channels.into_iter().zip(responses.into_iter()) {
                        let _ = tx.send(Ok(response));
                    }
                }
                Err(e) => {
                    // Send error to all channels
                    for tx in response_channels {
                        let _ = tx.send(Err(e.clone()));
                    }
                }
            }
        }
    }
}
