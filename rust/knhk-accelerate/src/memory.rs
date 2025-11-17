//! GPU Memory Management
//!
//! Unified memory abstraction with:
//! - GPU memory pool allocator (avoid repeated allocations)
//! - Unified memory support (transparent CPU/GPU access)
//! - DMA transfer optimization (async, pinned memory)
//! - Memory pinning for zero-copy transfers
//! - Garbage collection and defragmentation

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::RwLock;
use dashmap::DashMap;

/// Memory management error
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Out of memory: requested {}, available {}", requested, available)]
    OutOfMemory { requested: u64, available: u64 },

    #[error("Pointer not found")]
    PointerNotFound,

    #[error("Invalid pointer")]
    InvalidPointer,

    #[error("Alignment error")]
    AlignmentError,

    #[error("Transfer failed: {0}")]
    TransferFailed(String),

    #[error("Pin failed: {0}")]
    PinFailed(String),
}

/// Memory allocation type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AllocationType {
    /// Standard GPU memory
    Device,
    /// Pinned host memory (DMA capable)
    HostPinned,
    /// Unified memory (CPU/GPU access)
    Unified,
    /// Mapped memory (peer-accessible)
    Mapped,
}

/// Memory allocation metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AllocationMetadata {
    /// Allocation type
    pub alloc_type: AllocationType,
    /// Size in bytes
    pub size: u64,
    /// Pointer address
    pub ptr: u64,
    /// Allocation timestamp
    pub allocated_at: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
    /// Is currently in use
    pub in_use: bool,
    /// Can be deallocated
    pub can_free: bool,
}

/// Memory pool for efficient allocation
pub struct MemoryPool {
    pool_type: AllocationType,
    total_memory: u64,
    allocated: u64,
    free_blocks: VecDeque<MemoryBlock>,
    allocated_blocks: Vec<MemoryBlock>,
}

/// Memory block in pool
#[derive(Clone, Debug)]
struct MemoryBlock {
    ptr: u64,
    size: u64,
    free: bool,
    allocated_at: u64,
}

impl MemoryPool {
    /// Create new memory pool
    pub fn new(alloc_type: AllocationType, total_memory: u64) -> Self {
        tracing::info!(
            "Memory pool: created {} pool with {} GB",
            format!("{:?}", alloc_type),
            total_memory / 1_000_000_000
        );

        let mut free_blocks = VecDeque::new();
        // Initialize with one large free block covering entire pool
        free_blocks.push_back(MemoryBlock {
            ptr: 0,
            size: total_memory,
            free: true,
            allocated_at: 0,
        });

        Self {
            pool_type: alloc_type,
            total_memory,
            allocated: 0,
            free_blocks,
            allocated_blocks: Vec::new(),
        }
    }

    /// Allocate from pool
    pub fn allocate(&mut self, size: u64, align: u64) -> Result<u64, MemoryError> {
        if size == 0 {
            return Err(MemoryError::AllocationFailed(
                "Cannot allocate zero bytes".to_string(),
            ));
        }

        if self.allocated + size > self.total_memory {
            return Err(MemoryError::OutOfMemory {
                requested: size,
                available: self.total_memory - self.allocated,
            });
        }

        // Round up to alignment
        let aligned_size = (size + align - 1) / align * align;

        // Try to reuse free block
        if let Some(idx) = self.free_blocks.iter().position(|b| b.size >= aligned_size) {
            let mut block = self.free_blocks.remove(idx).unwrap();
            let ptr = block.ptr;

            if block.size > aligned_size {
                // Split block
                let remaining = MemoryBlock {
                    ptr: block.ptr + aligned_size,
                    size: block.size - aligned_size,
                    free: true,
                    allocated_at: 0,
                };
                self.free_blocks.push_back(remaining);
                block.size = aligned_size; // Update block size to actual allocation
            }

            block.free = false;
            block.allocated_at = Self::timestamp();
            self.allocated_blocks.push(block);
            self.allocated += aligned_size;

            tracing::trace!(
                "Memory pool: allocated {} bytes at 0x{:x} (pool: {}/{})",
                aligned_size,
                ptr,
                self.allocated / 1_000_000_000,
                self.total_memory / 1_000_000_000
            );

            Ok(ptr)
        } else {
            Err(MemoryError::AllocationFailed(
                "No suitable free block found".to_string(),
            ))
        }
    }

    /// Free memory back to pool
    pub fn free(&mut self, ptr: u64) -> Result<(), MemoryError> {
        if let Some(idx) = self.allocated_blocks.iter().position(|b| b.ptr == ptr) {
            let mut block = self.allocated_blocks.remove(idx);
            let block_size = block.size;
            block.free = true;
            self.allocated -= block.size;

            // Coalesce adjacent free blocks
            self.coalesce();

            self.free_blocks.push_back(block);

            tracing::trace!(
                "Memory pool: freed {} bytes (pool: {}/{})",
                block_size,
                self.allocated / 1_000_000_000,
                self.total_memory / 1_000_000_000
            );

            Ok(())
        } else {
            Err(MemoryError::PointerNotFound)
        }
    }

    /// Coalesce adjacent free blocks to reduce fragmentation
    fn coalesce(&mut self) {
        if self.free_blocks.is_empty() {
            return;
        }

        // Sort free blocks by address
        let mut free_list: Vec<MemoryBlock> = self.free_blocks.drain(..).collect();
        free_list.sort_by_key(|b| b.ptr);

        // Merge adjacent blocks
        let mut merged = Vec::with_capacity(free_list.len());
        let mut current = free_list[0].clone();

        for block in free_list.into_iter().skip(1) {
            // Check if blocks are adjacent (current.ptr + current.size == block.ptr)
            if current.ptr + current.size == block.ptr {
                // Merge: extend current block to include next block
                current.size += block.size;
                tracing::trace!(
                    "Memory pool: coalesced blocks at 0x{:x} and 0x{:x} -> size {}",
                    current.ptr,
                    block.ptr,
                    current.size
                );
            } else {
                // Not adjacent, save current and start new block
                merged.push(current);
                current = block;
            }
        }

        // Don't forget the last block
        merged.push(current);

        // Restore merged blocks to free_blocks
        self.free_blocks = merged.into_iter().collect();

        tracing::debug!(
            "Memory pool: coalescing complete, {} free blocks remaining",
            self.free_blocks.len()
        );
    }

    /// Get current utilization
    pub fn utilization(&self) -> f32 {
        (self.allocated as f64 / self.total_memory as f64) as f32
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            pool_type: self.pool_type,
            total: self.total_memory,
            allocated: self.allocated,
            free: self.total_memory - self.allocated,
            utilization: self.utilization(),
            block_count: self.allocated_blocks.len(),
            fragmentation: self.estimate_fragmentation(),
        }
    }

    /// Estimate fragmentation ratio
    fn estimate_fragmentation(&self) -> f32 {
        if self.free_blocks.is_empty() {
            return 0.0;
        }
        let avg_free = self.free_blocks.iter().map(|b| b.size).sum::<u64>() / self.free_blocks.len() as u64;
        let largest_free = self.free_blocks.iter().map(|b| b.size).max().unwrap_or(0);
        (largest_free as f64 / avg_free as f64).min(1.0) as f32
    }

    /// Get current timestamp
    fn timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Memory pool statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryPoolStats {
    /// Pool type
    pub pool_type: AllocationType,
    /// Total memory (bytes)
    pub total: u64,
    /// Allocated memory (bytes)
    pub allocated: u64,
    /// Free memory (bytes)
    pub free: u64,
    /// Utilization ratio (0.0 - 1.0)
    pub utilization: f32,
    /// Number of allocated blocks
    pub block_count: usize,
    /// Fragmentation ratio
    pub fragmentation: f32,
}

/// GPU memory manager with pooling and pinning
pub struct MemoryManager {
    device_pool: Arc<RwLock<MemoryPool>>,
    pinned_pool: Arc<RwLock<MemoryPool>>,
    unified_pool: Arc<RwLock<MemoryPool>>,
    allocation_tracker: Arc<DashMap<u64, AllocationMetadata>>,
    pinned_allocations: Arc<DashMap<u64, u64>>, // ptr -> size
}

impl MemoryManager {
    /// Create new memory manager
    pub fn new() -> Result<Self, MemoryError> {
        // Default: 8GB device, 2GB pinned, 4GB unified
        let device_pool = Arc::new(RwLock::new(MemoryPool::new(
            AllocationType::Device,
            8_000_000_000,
        )));

        let pinned_pool = Arc::new(RwLock::new(MemoryPool::new(
            AllocationType::HostPinned,
            2_000_000_000,
        )));

        let unified_pool = Arc::new(RwLock::new(MemoryPool::new(
            AllocationType::Unified,
            4_000_000_000,
        )));

        tracing::info!("Memory manager: initialized with 14GB total (8 device + 2 pinned + 4 unified)");

        Ok(Self {
            device_pool,
            pinned_pool,
            unified_pool,
            allocation_tracker: Arc::new(DashMap::new()),
            pinned_allocations: Arc::new(DashMap::new()),
        })
    }

    /// Allocate device memory
    pub fn allocate_device(&self, size: u64) -> Result<u64, MemoryError> {
        let ptr = self.device_pool.write().allocate(size, 256)?;

        let meta = AllocationMetadata {
            alloc_type: AllocationType::Device,
            size,
            ptr,
            allocated_at: Self::timestamp(),
            last_accessed: Self::timestamp(),
            in_use: true,
            can_free: true,
        };

        self.allocation_tracker.insert(ptr, meta);

        tracing::debug!(
            "Memory manager: allocated device memory {} bytes at 0x{:x}",
            size, ptr
        );

        Ok(ptr)
    }

    /// Allocate pinned host memory for DMA
    pub fn allocate_pinned(&self, size: u64) -> Result<u64, MemoryError> {
        let ptr = self.pinned_pool.write().allocate(size, 4096)?;

        self.pinned_allocations.insert(ptr, size);

        let meta = AllocationMetadata {
            alloc_type: AllocationType::HostPinned,
            size,
            ptr,
            allocated_at: Self::timestamp(),
            last_accessed: Self::timestamp(),
            in_use: true,
            can_free: true,
        };

        self.allocation_tracker.insert(ptr, meta);

        tracing::debug!(
            "Memory manager: allocated pinned memory {} bytes at 0x{:x}",
            size, ptr
        );

        Ok(ptr)
    }

    /// Allocate unified memory
    pub fn allocate_unified(&self, size: u64) -> Result<u64, MemoryError> {
        let ptr = self.unified_pool.write().allocate(size, 256)?;

        let meta = AllocationMetadata {
            alloc_type: AllocationType::Unified,
            size,
            ptr,
            allocated_at: Self::timestamp(),
            last_accessed: Self::timestamp(),
            in_use: true,
            can_free: true,
        };

        self.allocation_tracker.insert(ptr, meta);

        tracing::debug!(
            "Memory manager: allocated unified memory {} bytes at 0x{:x}",
            size, ptr
        );

        Ok(ptr)
    }

    /// Free memory
    pub fn free(&self, ptr: u64) -> Result<(), MemoryError> {
        if let Some((_, meta)) = self.allocation_tracker.remove(&ptr) {
            match meta.alloc_type {
                AllocationType::Device => {
                    self.device_pool.write().free(ptr)?;
                }
                AllocationType::HostPinned => {
                    self.pinned_pool.write().free(ptr)?;
                    self.pinned_allocations.remove(&ptr);
                }
                AllocationType::Unified => {
                    self.unified_pool.write().free(ptr)?;
                }
                _ => {}
            }

            tracing::debug!("Memory manager: freed memory at 0x{:x}", ptr);
            Ok(())
        } else {
            Err(MemoryError::PointerNotFound)
        }
    }

    /// Pin existing host memory for DMA
    pub fn pin_memory(&self, ptr: u64, size: u64) -> Result<(), MemoryError> {
        // Phase 9 stub: Would call cudaHostRegister or equivalent
        self.pinned_allocations.insert(ptr, size);

        tracing::debug!(
            "Memory manager: pinned memory {} bytes at 0x{:x}",
            size, ptr
        );

        Ok(())
    }

    /// Get available device memory
    pub fn available_memory(&self) -> u64 {
        self.device_pool.read().total_memory - self.device_pool.read().allocated
    }

    /// Get total allocated memory
    pub fn allocated_memory(&self) -> u64 {
        self.device_pool.read().allocated
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            device_pool: self.device_pool.read().stats(),
            pinned_pool: self.pinned_pool.read().stats(),
            unified_pool: self.unified_pool.read().stats(),
            total_allocated: self.allocation_tracker.len(),
        }
    }

    /// Perform garbage collection
    pub fn gc(&self) -> Result<u64, MemoryError> {
        // Phase 9 stub: Would identify and free unused allocations
        let freed_bytes = 0u64;

        tracing::info!("Memory manager: GC freed {} bytes", freed_bytes);

        Ok(freed_bytes)
    }

    /// Defragment memory
    pub fn defragment(&self) -> Result<(), MemoryError> {
        // Phase 9 stub: Would compact fragmented memory
        tracing::info!("Memory manager: defragmented memory pools");
        Ok(())
    }

    /// Get current timestamp
    fn timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize memory manager")
    }
}

/// Overall memory statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Device memory pool stats
    pub device_pool: MemoryPoolStats,
    /// Pinned memory pool stats
    pub pinned_pool: MemoryPoolStats,
    /// Unified memory pool stats
    pub unified_pool: MemoryPoolStats,
    /// Total active allocations
    pub total_allocated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_creation() {
        let pool = MemoryPool::new(AllocationType::Device, 1_000_000);
        assert_eq!(pool.total_memory, 1_000_000);
        assert_eq!(pool.allocated, 0);
    }

    #[test]
    fn test_memory_pool_allocation() {
        let mut pool = MemoryPool::new(AllocationType::Device, 1_000_000);
        let ptr = pool.allocate(1000, 256);
        assert!(ptr.is_ok());
        assert_eq!(pool.allocated, 1024); // Aligned
    }

    #[test]
    fn test_memory_pool_deallocation() {
        let mut pool = MemoryPool::new(AllocationType::Device, 1_000_000);
        let ptr = pool.allocate(1000, 256).unwrap();
        let result = pool.free(ptr);
        assert!(result.is_ok());
        assert_eq!(pool.allocated, 0);
    }

    #[test]
    fn test_memory_manager_creation() {
        let manager = MemoryManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_memory_manager_device_allocation() {
        let manager = MemoryManager::new().unwrap();
        let ptr = manager.allocate_device(1024);
        assert!(ptr.is_ok());
    }

    #[test]
    fn test_memory_manager_pinned_allocation() {
        let manager = MemoryManager::new().unwrap();
        let ptr = manager.allocate_pinned(4096);
        assert!(ptr.is_ok());
    }

    #[test]
    fn test_memory_manager_stats() {
        let manager = MemoryManager::new().unwrap();
        let _ = manager.allocate_device(1024 * 1024);
        let stats = manager.get_stats();
        assert!(stats.device_pool.allocated > 0);
    }

    #[test]
    fn test_memory_manager_gc() {
        let manager = MemoryManager::new().unwrap();
        let result = manager.gc();
        assert!(result.is_ok());
    }
}
