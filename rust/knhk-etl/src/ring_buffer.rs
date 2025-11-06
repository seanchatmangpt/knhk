// rust/knhk-etl/src/ring_buffer.rs
// Lock-free ring buffers for 8-beat epoch system
// Power-of-two sized, atomic indices, branchless enqueue/dequeue

use core::sync::atomic::{AtomicU64, Ordering};
use core::cell::UnsafeCell;
use alloc::vec::Vec;

/// Ring buffer error types
#[derive(Debug, Clone, PartialEq)]
pub enum RingError {
    Full,
    Empty,
    InvalidCapacity,
}

/// Lock-free single-producer single-consumer ring buffer
/// Size must be power-of-two for branchless modulo
#[derive(Debug)]
pub struct RingBuffer<T> {
    /// Producer head (write position)
    head: AtomicU64,
    /// Consumer tail (read position)
    tail: AtomicU64,
    /// Capacity mask (capacity - 1, must be power-of-two)
    mask: u64,
    /// Fixed-size buffer (UnsafeCell for interior mutability in lock-free context)
    buffer: UnsafeCell<Vec<Option<T>>>,
    /// Capacity (must be power-of-two)
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// Create new ring buffer with power-of-two capacity
    /// Panics if capacity is not power-of-two
    pub fn new(capacity: usize) -> Result<Self, RingError> {
        // Validate power-of-two
        if capacity == 0 || (capacity & (capacity - 1)) != 0 {
            return Err(RingError::InvalidCapacity);
        }

        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);

        Ok(Self {
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            mask: (capacity - 1) as u64,
            buffer: UnsafeCell::new(buffer),
            capacity,
        })
    }

    /// Enqueue item (producer side)
    /// Returns Ok(()) on success, Err(RingError::Full) if buffer is full
    pub fn enqueue(&self, item: T) -> Result<(), RingError> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        // Check if full: (head + 1) & mask == tail & mask
        let next_head = (head + 1) & self.mask;
        if next_head == (tail & self.mask) {
            return Err(RingError::Full);
        }

        // Store item at head position
        let slot = (head & self.mask) as usize;
        unsafe {
            (&mut *self.buffer.get())[slot] = Some(item);
        }

        // Advance head (release semantics for consumer visibility)
        self.head.store(head + 1, Ordering::Release);

        Ok(())
    }

    /// Dequeue item (consumer side)
    /// Returns Some(item) on success, None if buffer is empty
    pub fn dequeue(&self) -> Option<T> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        // Check if empty: tail & mask == head & mask
        if (tail & self.mask) == (head & self.mask) {
            return None;
        }

        // Load item from tail position
        let slot = (tail & self.mask) as usize;
        let item = unsafe {
            (&mut *self.buffer.get())[slot].take()
        }?;

        // Advance tail (release semantics for producer visibility)
        self.tail.store(tail + 1, Ordering::Release);

        Some(item)
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        (tail & self.mask) == (head & self.mask)
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        let next_head = (head + 1) & self.mask;
        next_head == (tail & self.mask)
    }

    /// Get current size (approximate, may be slightly stale)
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        let diff = head.wrapping_sub(tail);
        (diff & self.mask) as usize
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear buffer (reset head and tail)
    /// Note: This is not thread-safe if used concurrently
    pub fn clear(&mut self) {
        self.head.store(0, Ordering::Relaxed);
        self.tail.store(0, Ordering::Relaxed);
        unsafe {
            for slot in (&mut *self.buffer.get()).iter_mut() {
                *slot = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let ring = RingBuffer::<u32>::new(8).unwrap();
        assert_eq!(ring.capacity(), 8);
        assert!(ring.is_empty());
    }

    #[test]
    fn test_ring_buffer_invalid_capacity() {
        assert_eq!(
            RingBuffer::<u32>::new(7),
            Err(RingError::InvalidCapacity)
        );
        assert_eq!(
            RingBuffer::<u32>::new(0),
            Err(RingError::InvalidCapacity)
        );
    }

    #[test]
    fn test_ring_buffer_enqueue_dequeue() {
        let ring = RingBuffer::<u32>::new(8).unwrap();
        
        assert!(ring.enqueue(1).is_ok());
        assert!(ring.enqueue(2).is_ok());
        
        assert_eq!(ring.dequeue(), Some(1));
        assert_eq!(ring.dequeue(), Some(2));
        assert_eq!(ring.dequeue(), None);
    }

    #[test]
    fn test_ring_buffer_full() {
        let ring = RingBuffer::<u32>::new(8).unwrap();
        
        // Fill buffer (capacity is 8, so we can store 7 items before full)
        for i in 0..7 {
            assert!(ring.enqueue(i).is_ok());
        }
        
        // Next enqueue should fail (full)
        assert_eq!(ring.enqueue(7), Err(RingError::Full));
    }

    #[test]
    fn test_ring_buffer_wrap_around() {
        let ring = RingBuffer::<u32>::new(8).unwrap();
        
        // Fill and drain to test wrap-around
        for i in 0..7 {
            assert!(ring.enqueue(i).is_ok());
        }
        
        for i in 0..7 {
            assert_eq!(ring.dequeue(), Some(i));
        }
        
        // Should be able to enqueue again after wrap-around
        assert!(ring.enqueue(10).is_ok());
        assert_eq!(ring.dequeue(), Some(10));
    }

    #[test]
    fn test_ring_buffer_len() {
        let ring = RingBuffer::<u32>::new(8).unwrap();
        
        assert_eq!(ring.len(), 0);
        
        ring.enqueue(1).unwrap();
        ring.enqueue(2).unwrap();
        assert_eq!(ring.len(), 2);
        
        ring.dequeue();
        assert_eq!(ring.len(), 1);
    }
}

