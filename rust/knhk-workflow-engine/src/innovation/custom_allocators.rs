//! Custom Allocators: Zero-Allocation Workflow Execution
//!
//! Arena allocators for batch allocation/deallocation.
//! Bump allocators for append-only workflows.
//! Pool allocators for fixed-size objects.
//! Stack-based allocators for temporary data.

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{self, NonNull};
use std::alloc::System;

/// Arena allocator - batch allocation with single deallocation
pub struct Arena {
    chunks: UnsafeCell<Vec<Chunk>>,
    chunk_size: usize,
}

struct Chunk {
    data: NonNull<u8>,
    size: usize,
    used: usize,
}

impl Arena {
    /// Create arena with default chunk size (4KB)
    pub fn new() -> Self {
        Self::with_chunk_size(4096)
    }

    /// Create arena with custom chunk size
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: UnsafeCell::new(Vec::new()),
            chunk_size,
        }
    }

    /// Allocate memory from arena
    pub fn alloc<T>(&self) -> Option<&mut T>
    where
        T: Default,
    {
        unsafe {
            let layout = Layout::new::<T>();
            let ptr = self.alloc_raw(layout)?;
            let typed_ptr = ptr.as_ptr() as *mut T;
            ptr::write(typed_ptr, T::default());
            Some(&mut *typed_ptr)
        }
    }

    /// Allocate raw memory
    pub fn alloc_raw(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe {
            let chunks = &mut *self.chunks.get();

            // Try to allocate from existing chunk
            if let Some(chunk) = chunks.last_mut() {
                if let Some(ptr) = chunk.allocate(layout.size(), layout.align()) {
                    return Some(ptr);
                }
            }

            // Need new chunk
            let new_size = self.chunk_size.max(layout.size());
            let new_chunk = Chunk::new(new_size)?;
            let ptr = new_chunk.allocate(layout.size(), layout.align())?;
            chunks.push(new_chunk);
            Some(ptr)
        }
    }

    /// Allocate slice
    pub fn alloc_slice<T>(&self, count: usize) -> Option<&mut [T]>
    where
        T: Default,
    {
        unsafe {
            let layout = Layout::array::<T>(count).ok()?;
            let ptr = self.alloc_raw(layout)?;
            let slice_ptr = core::slice::from_raw_parts_mut(ptr.as_ptr() as *mut T, count);

            // Initialize elements
            for i in 0..count {
                ptr::write(slice_ptr.as_mut_ptr().add(i), T::default());
            }

            Some(slice_ptr)
        }
    }

    /// Clear arena (dealloc all at once)
    pub fn clear(&mut self) {
        unsafe {
            let chunks = self.chunks.get_mut();
            for chunk in chunks.drain(..) {
                chunk.dealloc();
            }
        }
    }

    /// Get total allocated bytes
    pub fn allocated(&self) -> usize {
        unsafe {
            let chunks = &*self.chunks.get();
            chunks.iter().map(|chunk| chunk.used).sum()
        }
    }

    /// Get total capacity
    pub fn capacity(&self) -> usize {
        unsafe {
            let chunks = &*self.chunks.get();
            chunks.iter().map(|chunk| chunk.size).sum()
        }
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Chunk {
    fn new(size: usize) -> Option<Self> {
        unsafe {
            let layout = Layout::from_size_align(size, 8).ok()?;
            let data = NonNull::new(System.alloc(layout))?;
            Some(Self {
                data,
                size,
                used: 0,
            })
        }
    }

    fn allocate(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let addr = self.data.as_ptr() as usize + self.used;
        let aligned = (addr + align - 1) & !(align - 1);
        let offset = aligned - self.data.as_ptr() as usize;

        if offset + size <= self.size {
            self.used = offset + size;
            Some(unsafe { NonNull::new_unchecked(self.data.as_ptr().add(offset)) })
        } else {
            None
        }
    }

    fn dealloc(self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, 8);
            System.dealloc(self.data.as_ptr(), layout);
        }
    }
}

/// Bump allocator - append-only, ultra-fast allocation
pub struct BumpAllocator {
    buffer: Vec<u8>,
    offset: UnsafeCell<usize>,
}

impl BumpAllocator {
    /// Create bump allocator with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            offset: UnsafeCell::new(0),
        }
    }

    /// Allocate from bump allocator
    pub fn alloc<T>(&self) -> Option<&mut T>
    where
        T: Default,
    {
        unsafe {
            let layout = Layout::new::<T>();
            let offset = self.offset.get();
            let current = *offset;

            // Align offset
            let aligned = (current + layout.align() - 1) & !(layout.align() - 1);

            if aligned + layout.size() > self.buffer.len() {
                return None; // Out of space
            }

            *offset = aligned + layout.size();
            let ptr = self.buffer.as_ptr().add(aligned) as *mut T;
            ptr::write(ptr, T::default());
            Some(&mut *ptr)
        }
    }

    /// Reset allocator (reuse buffer)
    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Get used bytes
    pub fn used(&self) -> usize {
        unsafe { *self.offset.get() }
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.used()
    }
}

/// Object pool - reusable fixed-size objects
pub struct ObjectPool<T> {
    objects: UnsafeCell<Vec<Option<T>>>,
    free_list: UnsafeCell<Vec<usize>>,
}

impl<T> ObjectPool<T> {
    /// Create pool with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut objects = Vec::with_capacity(capacity);
        objects.resize_with(capacity, || None);

        let free_list = (0..capacity).collect();

        Self {
            objects: UnsafeCell::new(objects),
            free_list: UnsafeCell::new(free_list),
        }
    }

    /// Acquire object from pool
    pub fn acquire(&self) -> Option<PooledObject<T>>
    where
        T: Default,
    {
        unsafe {
            let free_list = &mut *self.free_list.get();
            let objects = &mut *self.objects.get();

            let index = free_list.pop()?;
            objects[index] = Some(T::default());

            Some(PooledObject {
                pool: self as *const Self,
                index,
            })
        }
    }

    /// Release object back to pool
    fn release(&self, index: usize) {
        unsafe {
            let free_list = &mut *self.free_list.get();
            let objects = &mut *self.objects.get();

            objects[index] = None;
            free_list.push(index);
        }
    }

    /// Get object by index
    unsafe fn get(&self, index: usize) -> Option<&mut T> {
        let objects = &mut *self.objects.get();
        objects[index].as_mut()
    }

    /// Get pool utilization (0.0 - 1.0)
    pub fn utilization(&self) -> f64 {
        unsafe {
            let free_list = &*self.free_list.get();
            let objects = &*self.objects.get();
            let used = objects.len() - free_list.len();
            used as f64 / objects.len() as f64
        }
    }
}

/// Pooled object - automatically returned to pool on drop
pub struct PooledObject<T> {
    pool: *const ObjectPool<T>,
    index: usize,
}

impl<T> PooledObject<T> {
    /// Get reference to pooled object
    pub fn get(&self) -> Option<&T> {
        unsafe {
            let pool = &*self.pool;
            pool.get(self.index).map(|r| &*r)
        }
    }

    /// Get mutable reference to pooled object
    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            let pool = &*self.pool;
            pool.get(self.index)
        }
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        unsafe {
            let pool = &*self.pool;
            pool.release(self.index);
        }
    }
}

/// Stack-based allocator - uses pre-allocated stack buffer
pub struct StackAllocator<const SIZE: usize> {
    buffer: [u8; SIZE],
    offset: UnsafeCell<usize>,
}

impl<const SIZE: usize> StackAllocator<SIZE> {
    /// Create stack allocator
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; SIZE],
            offset: UnsafeCell::new(0),
        }
    }

    /// Allocate from stack
    pub fn alloc<T>(&self) -> Option<&mut T>
    where
        T: Default,
    {
        unsafe {
            let layout = Layout::new::<T>();
            let offset = self.offset.get();
            let current = *offset;

            let aligned = (current + layout.align() - 1) & !(layout.align() - 1);

            if aligned + layout.size() > SIZE {
                return None;
            }

            *offset = aligned + layout.size();
            let ptr = self.buffer.as_ptr().add(aligned) as *mut T;
            ptr::write(ptr, T::default());
            Some(&mut *ptr)
        }
    }

    /// Reset allocator
    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }
}

/// Allocator statistics
pub struct AllocatorStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub bytes_allocated: usize,
    pub bytes_freed: usize,
    pub peak_usage: usize,
}

impl AllocatorStats {
    pub const fn new() -> Self {
        Self {
            allocations: 0,
            deallocations: 0,
            bytes_allocated: 0,
            bytes_freed: 0,
            peak_usage: 0,
        }
    }

    pub fn record_alloc(&mut self, size: usize) {
        self.allocations += 1;
        self.bytes_allocated += size;
        let current = self.bytes_allocated - self.bytes_freed;
        if current > self.peak_usage {
            self.peak_usage = current;
        }
    }

    pub fn record_dealloc(&mut self, size: usize) {
        self.deallocations += 1;
        self.bytes_freed += size;
    }

    pub fn current_usage(&self) -> usize {
        self.bytes_allocated - self.bytes_freed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocator() {
        let arena = Arena::new();

        let x = arena.alloc::<i32>().unwrap();
        *x = 42;
        assert_eq!(*x, 42);

        let y = arena.alloc::<i32>().unwrap();
        *y = 100;
        assert_eq!(*y, 100);

        assert!(arena.allocated() > 0);
    }

    #[test]
    fn test_arena_slice() {
        let arena = Arena::new();
        let slice = arena.alloc_slice::<u32>(10).unwrap();
        assert_eq!(slice.len(), 10);

        for i in 0..10 {
            slice[i] = i as u32 * 2;
        }

        assert_eq!(slice[5], 10);
    }

    #[test]
    fn test_bump_allocator() {
        let bump = BumpAllocator::with_capacity(1024);

        let x = bump.alloc::<i32>().unwrap();
        *x = 42;

        let y = bump.alloc::<i64>().unwrap();
        *y = 1000;

        assert!(bump.used() > 0);
        assert!(bump.remaining() < 1024);

        bump.reset();
        assert_eq!(bump.used(), 0);
    }

    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::<i32>::with_capacity(10);

        let mut obj1 = pool.acquire().unwrap();
        *obj1.get_mut().unwrap() = 42;
        assert_eq!(*obj1.get().unwrap(), 42);

        let obj2 = pool.acquire().unwrap();
        assert!(obj2.get().is_some());

        drop(obj1); // Returns to pool

        let obj3 = pool.acquire().unwrap();
        assert!(obj3.get().is_some());

        assert!(pool.utilization() > 0.0);
        assert!(pool.utilization() <= 1.0);
    }

    #[test]
    fn test_stack_allocator() {
        let stack = StackAllocator::<256>::new();

        let x = stack.alloc::<i32>().unwrap();
        *x = 42;

        let y = stack.alloc::<i64>().unwrap();
        *y = 100;

        stack.reset();

        let z = stack.alloc::<i32>().unwrap();
        *z = 10;
        assert_eq!(*z, 10);
    }

    #[test]
    fn test_allocator_stats() {
        let mut stats = AllocatorStats::new();

        stats.record_alloc(100);
        assert_eq!(stats.current_usage(), 100);

        stats.record_alloc(50);
        assert_eq!(stats.current_usage(), 150);
        assert_eq!(stats.peak_usage, 150);

        stats.record_dealloc(100);
        assert_eq!(stats.current_usage(), 50);
        assert_eq!(stats.peak_usage, 150); // Peak remains

        assert_eq!(stats.allocations, 2);
        assert_eq!(stats.deallocations, 1);
    }

    #[test]
    fn test_arena_clear() {
        let mut arena = Arena::new();

        let _x = arena.alloc::<i32>();
        let _y = arena.alloc::<i64>();

        let before = arena.allocated();
        assert!(before > 0);

        arena.clear();

        let after = arena.allocated();
        assert_eq!(after, 0);
    }

    #[test]
    fn test_pool_exhaustion() {
        let pool = ObjectPool::<i32>::with_capacity(2);

        let _obj1 = pool.acquire().unwrap();
        let _obj2 = pool.acquire().unwrap();

        // Pool exhausted
        let obj3 = pool.acquire();
        assert!(obj3.is_none());
    }
}
