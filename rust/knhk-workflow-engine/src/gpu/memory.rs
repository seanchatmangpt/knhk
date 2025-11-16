//! GPU memory management and optimization
//!
//! Provides utilities for:
//! - Pinned memory allocation for faster transfers
//! - Async memory transfers
//! - Zero-copy operations where possible
//! - Memory pool management

use super::*;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Memory transfer statistics
#[derive(Debug, Clone, Default)]
pub struct TransferStats {
    /// Total bytes transferred to device
    pub bytes_to_device: usize,
    /// Total bytes transferred from device
    pub bytes_from_device: usize,
    /// Number of transfers to device
    pub transfers_to_device: usize,
    /// Number of transfers from device
    pub transfers_from_device: usize,
    /// Total transfer time in microseconds
    pub total_transfer_time_us: u64,
}

/// Memory pool for reusing buffers
pub struct MemoryPool {
    pools: Arc<Mutex<Vec<Vec<u8>>>>,
    max_pool_size: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pools: Arc::new(Mutex::new(Vec::new())),
            max_pool_size,
        }
    }

    /// Acquire a buffer from the pool or allocate new
    pub async fn acquire(&self, size: usize) -> Vec<u8> {
        let mut pools = self.pools.lock().await;

        // Try to find a buffer of suitable size
        if let Some(pos) = pools.iter().position(|buf| buf.capacity() >= size) {
            let mut buf = pools.swap_remove(pos);
            buf.clear();
            buf.resize(size, 0);
            return buf;
        }

        // Allocate new buffer
        vec![0u8; size]
    }

    /// Return a buffer to the pool
    pub async fn release(&self, mut buffer: Vec<u8>) {
        let mut pools = self.pools.lock().await;

        if pools.len() < self.max_pool_size {
            buffer.clear();
            pools.push(buffer);
        }
        // Otherwise drop the buffer
    }
}

/// Pinned memory allocator for faster CPU-GPU transfers
pub struct PinnedMemory {
    data: Vec<u8>,
}

impl PinnedMemory {
    /// Allocate pinned memory
    pub fn new(size: usize) -> WorkflowResult<Self> {
        // In a real implementation, this would allocate page-locked memory
        // For now, we use regular allocation
        Ok(Self {
            data: vec![0u8; size],
        })
    }

    /// Get a mutable slice of the data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get an immutable slice of the data
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

/// Async memory transfer helper
pub struct AsyncTransfer {
    stats: Arc<Mutex<TransferStats>>,
}

impl AsyncTransfer {
    /// Create a new async transfer helper
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(TransferStats::default())),
        }
    }

    /// Transfer data to device asynchronously
    pub async fn transfer_to_device<T: Copy + Send>(
        &self,
        data: &[T],
    ) -> WorkflowResult<Vec<T>> {
        let start = std::time::Instant::now();

        // Simulate async transfer with tokio::spawn
        let data_copy = data.to_vec();
        let result = tokio::task::spawn_blocking(move || data_copy)
            .await
            .map_err(|e| {
                WorkflowError::ExecutionError(format!("Transfer failed: {}", e))
            })?;

        let elapsed = start.elapsed().as_micros() as u64;

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.bytes_to_device += std::mem::size_of::<T>() * data.len();
        stats.transfers_to_device += 1;
        stats.total_transfer_time_us += elapsed;

        Ok(result)
    }

    /// Transfer data from device asynchronously
    pub async fn transfer_from_device<T: Copy + Send>(
        &self,
        data: Vec<T>,
    ) -> WorkflowResult<Vec<T>> {
        let start = std::time::Instant::now();

        // Simulate async transfer
        let result = tokio::task::spawn_blocking(move || data)
            .await
            .map_err(|e| {
                WorkflowError::ExecutionError(format!("Transfer failed: {}", e))
            })?;

        let elapsed = start.elapsed().as_micros() as u64;

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.bytes_from_device += std::mem::size_of::<T>() * result.len();
        stats.transfers_from_device += 1;
        stats.total_transfer_time_us += elapsed;

        Ok(result)
    }

    /// Get transfer statistics
    pub async fn stats(&self) -> TransferStats {
        self.stats.lock().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = TransferStats::default();
    }
}

impl Default for AsyncTransfer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new(10);

        let buf1 = pool.acquire(1024).await;
        assert_eq!(buf1.len(), 1024);

        pool.release(buf1).await;

        let buf2 = pool.acquire(1024).await;
        assert_eq!(buf2.len(), 1024);
    }

    #[tokio::test]
    async fn test_async_transfer() {
        let transfer = AsyncTransfer::new();

        let data = vec![1u32, 2, 3, 4, 5];
        let result = transfer.transfer_to_device(&data).await.unwrap();
        assert_eq!(result, data);

        let result2 = transfer.transfer_from_device(result).await.unwrap();
        assert_eq!(result2, data);

        let stats = transfer.stats().await;
        assert_eq!(stats.transfers_to_device, 1);
        assert_eq!(stats.transfers_from_device, 1);
    }

    #[test]
    fn test_pinned_memory() {
        let mut pinned = PinnedMemory::new(1024).unwrap();
        let slice = pinned.as_mut_slice();
        slice[0] = 42;
        assert_eq!(pinned.as_slice()[0], 42);
    }
}
