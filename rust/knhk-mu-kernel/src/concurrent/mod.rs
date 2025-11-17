//! Lock-free concurrent data structures
//!
//! This module requires std and is only available with the `concurrent-structures` feature
//! or when testing.
//!
//! This module provides cutting-edge lock-free data structures for 2027-grade concurrency:
//!
//! ## Data Structures
//!
//! - **Skip List**: Lock-free ordered map with O(log n) expected time
//! - **HAMT**: Concurrent Hash Array Mapped Trie with persistent sharing
//! - **Stack/Queue**: Treiber stack and Michael-Scott queue
//! - **Epoch**: Epoch-based memory reclamation
//! - **Atomic Arc**: Lock-free atomic reference counting

// Enable std for this module
extern crate std;

mod lib_compat;

// ## Lock-Free Guarantees
//
// All data structures provide:
// - **Lock-freedom**: System-wide progress (at least one thread makes progress)
// - **Linearizability**: Operations appear to occur atomically
// - **ABA mitigation**: Tagged pointers prevent ABA problems
// - **Memory safety**: Safe reclamation via epoch-based or hazard pointers
//
// ## Memory Reclamation
//
// Two reclamation strategies are provided:
//
// 1. **Hazard Pointers** (skip list): Thread announces pointers it's using
// 2. **Epoch-Based** (others): Grace period based on epoch advancement
//
// ## Performance Characteristics
//
// | Structure | Insert | Remove | Search | Memory |
// |-----------|--------|--------|--------|--------|
// | Skip List | O(log n) | O(log n) | O(log n) wait-free | O(n) |
// | HAMT | O(log32 n) | O(log32 n) | O(log32 n) | O(n) persistent |
// | Stack | O(1) | O(1) | - | O(n) |
// | Queue | O(1) | O(1) | - | O(n) |
//
// ## Usage Example
//
// ```rust,ignore
// use knhk_mu_kernel::concurrent::{LockFreeSkipList, Guard};
//
// let list = LockFreeSkipList::new();
// list.insert(5);
// list.insert(3);
// list.insert(7);
//
// assert!(list.contains(&5));
// assert!(!list.contains(&10));
//
// list.remove(&3);
// assert!(!list.contains(&3));
// ```
//
// ## Integration with Epoch Reclamation
//
// ```rust,ignore
// use knhk_mu_kernel::concurrent::{Atomic, Guard};
//
// let atomic = Atomic::new(42);
// let guard = Guard::pin();
//
// // Protected read
// if let Some(value) = atomic.load(&guard) {
//     println!("Value: {}", value);
// }
//
// // Safe update with deferred reclamation
// atomic.store(100, &guard);
// ```

pub mod arc_atomic;
pub mod epoch;
pub mod hamt;
pub mod skiplist;
pub mod stack_queue;

// Re-export main types
pub use arc_atomic::{AtomicArc, AtomicArcCell, WeakArc};
pub use epoch::{Atomic, Guard};
pub use hamt::ConcurrentHAMT;
pub use skiplist::{HazardGuard, LockFreeSkipList};
pub use stack_queue::{MichaelScottQueue, TreiberStack};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_skip_list_with_epoch() {
        let list = Arc::new(LockFreeSkipList::new());
        let mut handles = vec![];

        // Concurrent insertions
        for t in 0..4 {
            let list = Arc::clone(&list);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    list.insert(t * 100 + i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(list.len(), 400);

        // Verify all insertions
        for i in 0..400 {
            assert!(list.contains(&i));
        }
    }

    #[test]
    fn test_hamt_concurrent() {
        let map = Arc::new(ConcurrentHAMT::new());
        let mut handles = vec![];

        // Concurrent insertions and updates
        for t in 0..4 {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    map.insert(format!("key-{}", i), t * 100 + i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all keys exist
        for i in 0..50 {
            assert!(map.contains_key(&format!("key-{}", i)));
        }
    }

    #[test]
    fn test_stack_queue_interaction() {
        let stack = Arc::new(TreiberStack::new());
        let queue = Arc::new(MichaelScottQueue::new());

        let mut handles = vec![];

        // Producer: push to stack, pop and enqueue to queue
        for _ in 0..2 {
            let stack = Arc::clone(&stack);
            let queue = Arc::clone(&queue);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    stack.push(i);
                    if let Some(val) = stack.pop() {
                        queue.enqueue(val);
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_atomic_arc_with_epoch() {
        let cell = Arc::new(AtomicArcCell::new(AtomicArc::new(0)));
        let mut handles = vec![];

        for t in 0..4 {
            let cell = Arc::clone(&cell);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    cell.store(AtomicArc::new(t * 100 + i));
                    cell.load();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(cell.load().is_some());
    }
}
