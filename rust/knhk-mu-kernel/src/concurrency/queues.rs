//! Lock-Free Work Queues
//!
//! Implements deterministic, cache-friendly work queues for multi-core scheduling.

use core::sync::atomic::{AtomicU64, AtomicUsize, AtomicPtr, Ordering};
use core::ptr;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::concurrency::logical_time::{Timestamp, TimestampedEvent};
use crate::concurrency::types::CoreLocal;

/// Maximum queue capacity
const MAX_QUEUE_SIZE: usize = 4096;

/// Queue errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    /// Queue is full
    Full,
    /// Queue is empty
    Empty,
    /// Invalid operation
    InvalidOperation,
}

/// Lock-free work queue (per-core, SPSC)
///
/// Single-Producer, Single-Consumer lock-free queue.
/// Used for core-local task execution.
///
/// # Properties
///
/// - **Lock-free**: No blocking operations
/// - **Cache-friendly**: Aligned to cache line boundaries
/// - **Bounded**: Fixed capacity for determinism
/// - **FIFO**: Preserves enqueue order
///
/// # Usage
///
/// ```rust,no_run
/// use knhk_mu_kernel::concurrency::WorkQueue;
///
/// let queue = WorkQueue::<u64, 1024>::new();
///
/// // Producer (core-local)
/// queue.enqueue(42)?;
///
/// // Consumer (same core)
/// let item = queue.dequeue()?;
/// ```
#[repr(C, align(128))]  // Cache-line aligned (avoid false sharing)
pub struct WorkQueue<T, const CAPACITY: usize> {
    /// Head index (consumer)
    head: AtomicUsize,
    /// Padding to prevent false sharing
    _pad1: [u64; 7],
    /// Tail index (producer)
    tail: AtomicUsize,
    /// Padding to prevent false sharing
    _pad2: [u64; 7],
    /// Ring buffer (fixed size for determinism)
    buffer: [Option<T>; CAPACITY],
}

// Manual implementation since we use const generics with options
impl<T, const CAPACITY: usize> WorkQueue<T, CAPACITY> {
    /// Create new work queue
    pub fn new() -> CoreLocal<Self> {
        assert!(CAPACITY > 0 && CAPACITY <= MAX_QUEUE_SIZE);
        assert!(CAPACITY.is_power_of_two(), "Capacity must be power of 2");

        // Create buffer filled with None
        // SAFETY: All None values are valid
        let buffer: [Option<T>; CAPACITY] = unsafe {
            let mut arr: [core::mem::MaybeUninit<Option<T>>; CAPACITY] =
                core::mem::MaybeUninit::uninit().assume_init();

            for elem in &mut arr[..] {
                elem.write(None);
            }

            core::mem::transmute_copy(&arr)
        };

        CoreLocal::new(Self {
            head: AtomicUsize::new(0),
            _pad1: [0; 7],
            tail: AtomicUsize::new(0),
            _pad2: [0; 7],
            buffer,
        })
    }

    /// Enqueue item (producer)
    #[inline]
    pub fn enqueue(&mut self, item: T) -> Result<(), QueueError> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);

        let next_tail = (tail + 1) & (CAPACITY - 1);  // Wrap using mask

        // Check if full
        if next_tail == head {
            return Err(QueueError::Full);
        }

        // SAFETY: We checked capacity, index is valid
        unsafe {
            let slot = &mut *(&mut self.buffer[tail] as *mut Option<T>);
            *slot = Some(item);
        }

        self.tail.store(next_tail, Ordering::Release);

        Ok(())
    }

    /// Dequeue item (consumer)
    #[inline]
    pub fn dequeue(&mut self) -> Result<T, QueueError> {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);

        // Check if empty
        if head == tail {
            return Err(QueueError::Empty);
        }

        // SAFETY: We checked that queue is not empty
        let item = unsafe {
            let slot = &mut *(&mut self.buffer[head] as *mut Option<T>);
            slot.take().ok_or(QueueError::Empty)?
        };

        let next_head = (head + 1) & (CAPACITY - 1);
        self.head.store(next_head, Ordering::Release);

        Ok(item)
    }

    /// Check if queue is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head == tail
    }

    /// Get current size (approximate)
    #[inline]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        if tail >= head {
            tail - head
        } else {
            CAPACITY - head + tail
        }
    }
}

/// Globally ordered queue (total order via timestamps)
///
/// Uses priority queue with logical timestamps for deterministic ordering.
///
/// # Properties
///
/// - **Total Order**: All events totally ordered by timestamp
/// - **Deterministic**: Same timestamps → same order (core_id tie-break)
/// - **Thread-safe**: Can be accessed from multiple cores
///
/// # Usage
///
/// ```rust,no_run
/// use knhk_mu_kernel::concurrency::{GlobalOrdered, Timestamp};
///
/// let queue = GlobalOrdered::new();
///
/// queue.enqueue(Timestamp::from_raw(10), 0, "event1");
/// queue.enqueue(Timestamp::from_raw(5), 0, "event2");
///
/// // Dequeues in timestamp order
/// let (ts, event) = queue.dequeue()?;  // timestamp=5, "event2"
/// ```
#[repr(C, align(64))]
pub struct GlobalOrdered<T> {
    /// Heap of timestamped events (min-heap)
    heap: AtomicPtr<Vec<TimestampedEvent<T>>>,
    /// Lock for heap modifications (spinlock via atomic)
    lock: AtomicU64,
}

impl<T> GlobalOrdered<T> {
    /// Create new globally ordered queue
    pub fn new() -> Self {
        let heap = Box::new(Vec::new());
        Self {
            heap: AtomicPtr::new(Box::into_raw(heap)),
            lock: AtomicU64::new(0),
        }
    }

    /// Acquire lock (spinlock)
    #[inline]
    fn acquire(&self) {
        while self
            .lock
            .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    /// Release lock
    #[inline]
    fn release(&self) {
        self.lock.store(0, Ordering::Release);
    }

    /// Enqueue item with timestamp
    pub fn enqueue(&self, timestamp: Timestamp, core_id: u8, item: T) -> Result<(), QueueError> {
        self.acquire();

        let heap_ptr = self.heap.load(Ordering::Relaxed);
        let heap = unsafe { &mut *heap_ptr };

        if heap.len() >= MAX_QUEUE_SIZE {
            self.release();
            return Err(QueueError::Full);
        }

        let event = TimestampedEvent::new(timestamp, core_id, item);
        heap.push(event);

        // Bubble up (min-heap property)
        let mut idx = heap.len() - 1;
        while idx > 0 {
            let parent = (idx - 1) / 2;
            if heap[idx] >= heap[parent] {
                break;
            }
            heap.swap(idx, parent);
            idx = parent;
        }

        self.release();
        Ok(())
    }

    /// Dequeue item (minimum timestamp)
    pub fn dequeue(&self) -> Result<(Timestamp, T), QueueError> {
        self.acquire();

        let heap_ptr = self.heap.load(Ordering::Relaxed);
        let heap = unsafe { &mut *heap_ptr };

        if heap.is_empty() {
            self.release();
            return Err(QueueError::Empty);
        }

        // Extract min (root)
        let min_event = heap.swap_remove(0);

        // Bubble down
        if !heap.is_empty() {
            let mut idx = 0;
            loop {
                let left = 2 * idx + 1;
                let right = 2 * idx + 2;

                let mut smallest = idx;

                if left < heap.len() && heap[left] < heap[smallest] {
                    smallest = left;
                }

                if right < heap.len() && heap[right] < heap[smallest] {
                    smallest = right;
                }

                if smallest == idx {
                    break;
                }

                heap.swap(idx, smallest);
                idx = smallest;
            }
        }

        self.release();
        Ok((min_event.timestamp, min_event.event))
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.acquire();
        let heap_ptr = self.heap.load(Ordering::Relaxed);
        let is_empty = unsafe { (*heap_ptr).is_empty() };
        self.release();
        is_empty
    }
}

impl<T> Drop for GlobalOrdered<T> {
    fn drop(&mut self) {
        let heap_ptr = self.heap.load(Ordering::Relaxed);
        if !heap_ptr.is_null() {
            unsafe {
                drop(Box::from_raw(heap_ptr));
            }
        }
    }
}

/// Best-effort queue (no ordering guarantees)
///
/// Lock-free MPMC queue for tasks that don't require deterministic ordering.
///
/// # Usage
///
/// For logging, metrics, or other non-critical paths.
pub struct BestEffort<T> {
    /// Lock-free linked list of items
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

struct Node<T> {
    data: Option<T>,
    next: AtomicPtr<Node<T>>,
}

impl<T> BestEffort<T> {
    /// Create new best-effort queue
    pub fn new() -> Self {
        let sentinel = Box::into_raw(Box::new(Node {
            data: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        Self {
            head: AtomicPtr::new(sentinel),
            tail: AtomicPtr::new(sentinel),
        }
    }

    /// Enqueue item (lock-free)
    pub fn enqueue(&self, item: T) {
        let node = Box::into_raw(Box::new(Node {
            data: Some(item),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*tail).next.load(Ordering::Acquire) };

            if next.is_null() {
                if unsafe { (*tail).next.compare_exchange(
                    ptr::null_mut(),
                    node,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() } {
                    let _ = self.tail.compare_exchange(
                        tail,
                        node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                    return;
                }
            } else {
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            }
        }
    }

    /// Dequeue item (lock-free)
    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { (*head).next.load(Ordering::Acquire) };

            if head == tail {
                if next.is_null() {
                    return None;  // Empty
                }
                let _ = self.tail.compare_exchange(
                    tail,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            } else {
                if next.is_null() {
                    return None;
                }

                let data = unsafe { (*next).data.take() };

                if self.head.compare_exchange(
                    head,
                    next,
                    Ordering::Release,
                    Ordering::Relaxed,
                ).is_ok() {
                    unsafe { drop(Box::from_raw(head)); }
                    return data;
                }
            }
        }
    }
}

impl<T> Drop for BestEffort<T> {
    fn drop(&mut self) {
        while self.dequeue().is_some() {}

        let head = self.head.load(Ordering::Relaxed);
        if !head.is_null() {
            unsafe { drop(Box::from_raw(head)); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_queue_creation() {
        let queue: CoreLocal<WorkQueue<u64, 16>> = WorkQueue::new();
        queue.with(|q| {
            assert!(q.is_empty());
            assert_eq!(q.len(), 0);
        });
    }

    #[test]
    fn test_work_queue_enqueue_dequeue() {
        let queue = WorkQueue::<u64, 16>::new();

        queue.with_mut(|q| {
            assert!(q.enqueue(42).is_ok());
            assert!(q.enqueue(43).is_ok());

            assert_eq!(q.len(), 2);
            assert!(!q.is_empty());

            assert_eq!(q.dequeue().unwrap(), 42);
            assert_eq!(q.dequeue().unwrap(), 43);

            assert!(q.is_empty());
        });
    }

    #[test]
    fn test_work_queue_full() {
        let queue = WorkQueue::<u64, 4>::new();

        queue.with_mut(|q| {
            assert!(q.enqueue(1).is_ok());
            assert!(q.enqueue(2).is_ok());
            assert!(q.enqueue(3).is_ok());

            // Queue is full (capacity - 1 due to ring buffer)
            assert_eq!(q.enqueue(4), Err(QueueError::Full));
        });
    }

    #[test]
    fn test_global_ordered_queue() {
        let queue = GlobalOrdered::new();

        // Enqueue out of order
        queue.enqueue(Timestamp::from_raw(30), 0, "event3").unwrap();
        queue.enqueue(Timestamp::from_raw(10), 0, "event1").unwrap();
        queue.enqueue(Timestamp::from_raw(20), 0, "event2").unwrap();

        // Dequeue in timestamp order
        let (ts1, e1) = queue.dequeue().unwrap();
        assert_eq!(ts1.as_raw(), 10);
        assert_eq!(e1, "event1");

        let (ts2, e2) = queue.dequeue().unwrap();
        assert_eq!(ts2.as_raw(), 20);
        assert_eq!(e2, "event2");

        let (ts3, e3) = queue.dequeue().unwrap();
        assert_eq!(ts3.as_raw(), 30);
        assert_eq!(e3, "event3");

        assert!(queue.is_empty());
    }

    #[test]
    fn test_best_effort_queue() {
        let queue = BestEffort::new();

        queue.enqueue(42);
        queue.enqueue(43);
        queue.enqueue(44);

        assert_eq!(queue.dequeue(), Some(42));
        assert_eq!(queue.dequeue(), Some(43));
        assert_eq!(queue.dequeue(), Some(44));
        assert_eq!(queue.dequeue(), None);
    }
}
