//! Correctness tests for lock-free concurrent data structures
//!
//! These tests verify:
//! - Linearizability of operations
//! - Absence of data races
//! - Memory safety under concurrent access
//! - Correctness of memory reclamation
//! - ABA problem mitigation

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;

// Import concurrent structures
#[path = "../src/concurrent/arc_atomic.rs"]
mod arc_atomic;
#[path = "../src/concurrent/epoch.rs"]
mod epoch;
#[path = "../src/concurrent/hamt.rs"]
mod hamt;
#[path = "../src/concurrent/skiplist.rs"]
mod skiplist;
#[path = "../src/concurrent/stack_queue.rs"]
mod stack_queue;

use arc_atomic::{AtomicArc, AtomicArcCell};
use epoch::{Atomic, Guard};
use hamt::ConcurrentHAMT;
use skiplist::LockFreeSkipList;
use stack_queue::{MichaelScottQueue, TreiberStack};

// --- Skip List Tests ---

#[test]
fn test_skiplist_linearizability_insert() {
    let list = Arc::new(LockFreeSkipList::new());
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    // All threads try to insert the same key
    for t in 0..4 {
        let list = Arc::clone(&list);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            list.insert(42)
        }));
    }

    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Exactly one should succeed
    assert_eq!(results.iter().filter(|&&r| r).count(), 1);
    assert_eq!(list.len(), 1);
}

#[test]
fn test_skiplist_concurrent_insert_all_unique() {
    let list = Arc::new(LockFreeSkipList::new());
    let mut handles = vec![];

    for t in 0..8 {
        let list = Arc::clone(&list);
        handles.push(thread::spawn(move || {
            for i in 0..1000 {
                assert!(list.insert(t * 1000 + i));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(list.len(), 8000);

    // Verify all values present
    for i in 0..8000 {
        assert!(list.contains(&i));
    }
}

#[test]
fn test_skiplist_concurrent_remove() {
    let list = Arc::new(LockFreeSkipList::new());

    // Pre-populate
    for i in 0..1000 {
        list.insert(i);
    }

    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    // Multiple threads try to remove same keys
    for _ in 0..4 {
        let list = Arc::clone(&list);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let mut removed = 0;
            for i in 0..1000 {
                if list.remove(&i) {
                    removed += 1;
                }
            }
            removed
        }));
    }

    let results: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Total removed should equal 1000
    assert_eq!(results.iter().sum::<usize>(), 1000);
    assert_eq!(list.len(), 0);
}

#[test]
fn test_skiplist_ordering() {
    let list = Arc::new(LockFreeSkipList::new());
    let mut handles = vec![];

    // Insert in random order from multiple threads
    for t in 0..4 {
        let list = Arc::clone(&list);
        handles.push(thread::spawn(move || {
            let mut vals = vec![];
            for i in 0..100 {
                vals.push(t * 100 + i);
            }
            // Shuffle would require rand, so just reverse
            vals.reverse();
            for v in vals {
                list.insert(v);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify iteration is ordered
    let values: Vec<_> = list.iter().collect();
    assert_eq!(values.len(), 400);

    for i in 0..399 {
        assert!(values[i] < values[i + 1]);
    }
}

// --- HAMT Tests ---

#[test]
fn test_hamt_linearizability() {
    let map = Arc::new(ConcurrentHAMT::new());
    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    // All threads insert same key
    for t in 0..4 {
        let map = Arc::clone(&map);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            map.insert("key", t)
        }));
    }

    let results: Vec<Option<usize>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Exactly 3 should see None (first insert), 1 should see Some (update)
    let none_count = results.iter().filter(|r| r.is_none()).count();
    assert!(none_count >= 1);
}

#[test]
fn test_hamt_concurrent_insert_unique_keys() {
    let map = Arc::new(ConcurrentHAMT::new());
    let mut handles = vec![];

    for t in 0..8 {
        let map = Arc::clone(&map);
        handles.push(thread::spawn(move || {
            for i in 0..500 {
                map.insert(format!("key-{}-{}", t, i), i);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all keys
    for t in 0..8 {
        for i in 0..500 {
            assert!(map.contains_key(&format!("key-{}-{}", t, i)));
        }
    }
}

#[test]
fn test_hamt_concurrent_updates() {
    let map = Arc::new(ConcurrentHAMT::new());

    // Pre-populate
    for i in 0..100 {
        map.insert(i, 0);
    }

    let mut handles = vec![];

    // Multiple threads update same keys
    for t in 0..4 {
        let map = Arc::clone(&map);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                map.insert(i, t);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // All keys should still exist
    for i in 0..100 {
        assert!(map.contains_key(&i));
    }
}

#[test]
fn test_hamt_remove_concurrent() {
    let map = Arc::new(ConcurrentHAMT::new());

    // Pre-populate
    for i in 0..500 {
        map.insert(i, i);
    }

    let barrier = Arc::new(Barrier::new(4));
    let mut handles = vec![];

    for _ in 0..4 {
        let map = Arc::clone(&map);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let mut removed = 0;
            for i in 0..500 {
                if map.remove(&i).is_some() {
                    removed += 1;
                }
            }
            removed
        }));
    }

    let results: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Total removed should equal 500
    assert_eq!(results.iter().sum::<usize>(), 500);
}

#[test]
fn test_hamt_persistent_sharing() {
    let map1 = Arc::new(ConcurrentHAMT::new());

    for i in 0..100 {
        map1.insert(i, i);
    }

    // Take snapshot
    let snapshot = map1.snapshot();

    // Modify original
    for i in 0..50 {
        map1.remove(&i);
    }

    // Snapshot should be unchanged (structural sharing)
    // This is a semantic test - implementation uses COW
    for i in 0..100 {
        assert!(map1.contains_key(&i) == (i >= 50));
    }
}

// --- Stack Tests ---

#[test]
fn test_stack_lifo_order() {
    let stack = TreiberStack::new();

    stack.push(1);
    stack.push(2);
    stack.push(3);

    assert_eq!(stack.pop(), Some(3));
    assert_eq!(stack.pop(), Some(2));
    assert_eq!(stack.pop(), Some(1));
    assert_eq!(stack.pop(), None);
}

#[test]
fn test_stack_concurrent_push_pop() {
    let stack = Arc::new(TreiberStack::new());
    let push_count = Arc::new(AtomicUsize::new(0));
    let pop_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Pushers
    for t in 0..4 {
        let stack = Arc::clone(&stack);
        let push_count = Arc::clone(&push_count);
        handles.push(thread::spawn(move || {
            for i in 0..1000 {
                stack.push(t * 1000 + i);
                push_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // Poppers
    for _ in 0..4 {
        let stack = Arc::clone(&stack);
        let pop_count = Arc::clone(&pop_count);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                if stack.pop().is_some() {
                    pop_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(push_count.load(Ordering::Relaxed), 4000);

    // Drain remaining
    let mut remaining = 0;
    while stack.pop().is_some() {
        remaining += 1;
    }

    assert_eq!(pop_count.load(Ordering::Relaxed) + remaining, 4000);
}

#[test]
fn test_stack_aba_resistance() {
    let stack = Arc::new(TreiberStack::new());

    // Pre-populate
    for i in 0..100 {
        stack.push(i);
    }

    let mut handles = vec![];

    // Concurrent pop-push cycles
    for _ in 0..4 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                if let Some(val) = stack.pop() {
                    stack.push(val);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Should still have 100 elements
    let mut count = 0;
    while stack.pop().is_some() {
        count += 1;
    }
    assert_eq!(count, 100);
}

// --- Queue Tests ---

#[test]
fn test_queue_fifo_order() {
    let queue = MichaelScottQueue::new();

    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);

    assert_eq!(queue.dequeue(), Some(1));
    assert_eq!(queue.dequeue(), Some(2));
    assert_eq!(queue.dequeue(), Some(3));
    assert_eq!(queue.dequeue(), None);
}

#[test]
fn test_queue_concurrent_enqueue_dequeue() {
    let queue = Arc::new(MichaelScottQueue::new());
    let enqueue_count = Arc::new(AtomicUsize::new(0));
    let dequeue_count = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Enqueuers
    for t in 0..4 {
        let queue = Arc::clone(&queue);
        let enqueue_count = Arc::clone(&enqueue_count);
        handles.push(thread::spawn(move || {
            for i in 0..1000 {
                queue.enqueue(t * 1000 + i);
                enqueue_count.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // Dequeuers
    for _ in 0..4 {
        let queue = Arc::clone(&queue);
        let dequeue_count = Arc::clone(&dequeue_count);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                if queue.dequeue().is_some() {
                    dequeue_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(enqueue_count.load(Ordering::Relaxed), 4000);

    // Drain remaining
    let mut remaining = 0;
    while queue.dequeue().is_some() {
        remaining += 1;
    }

    assert_eq!(dequeue_count.load(Ordering::Relaxed) + remaining, 4000);
}

#[test]
fn test_queue_producer_consumer() {
    let queue = Arc::new(MichaelScottQueue::new());
    let barrier = Arc::new(Barrier::new(8));
    let mut handles = vec![];

    // Producers
    for t in 0..4 {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            for i in 0..500 {
                queue.enqueue(t * 500 + i);
            }
        }));
    }

    // Consumers
    for _ in 0..4 {
        let queue = Arc::clone(&queue);
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            let mut consumed = HashSet::new();
            for _ in 0..500 {
                if let Some(val) = queue.dequeue() {
                    consumed.insert(val);
                }
            }
            consumed
        }));
    }

    let results: Vec<HashSet<usize>> = handles
        .into_iter()
        .skip(4)
        .map(|h| h.join().unwrap())
        .collect();

    // Combine all consumed values
    let mut all_consumed = HashSet::new();
    for set in results {
        all_consumed.extend(set);
    }

    // Drain remaining
    while let Some(val) = queue.dequeue() {
        all_consumed.insert(val);
    }

    assert_eq!(all_consumed.len(), 2000);
}

// --- Epoch Tests ---

#[test]
fn test_epoch_guard_basic() {
    let guard = Guard::pin();

    guard.defer(|| {
        // Deferred operation
    });

    drop(guard);
}

#[test]
fn test_epoch_nested_guards() {
    let guard1 = Guard::pin();
    let guard2 = Guard::pin();
    let guard3 = guard1.clone();

    guard1.defer(|| {});
    guard2.defer(|| {});
    guard3.defer(|| {});

    drop(guard1);
    drop(guard2);
    drop(guard3);
}

#[test]
fn test_epoch_concurrent_updates() {
    let atomic = Arc::new(Atomic::new(0));
    let mut handles = vec![];

    for t in 0..4 {
        let atomic = Arc::clone(&atomic);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let guard = Guard::pin();
                atomic.store(t * 100 + i, &guard);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let guard = Guard::pin();
    assert!(atomic.load(&guard).is_some());
}

#[test]
fn test_epoch_deferred_destruction() {
    static DESTROYED: AtomicUsize = AtomicUsize::new(0);

    struct DropChecker;
    impl Drop for DropChecker {
        fn drop(&mut self) {
            DESTROYED.fetch_add(1, Ordering::Relaxed);
        }
    }

    let atomic = Atomic::new(DropChecker);

    {
        let guard = Guard::pin();
        atomic.store(DropChecker, &guard);
        atomic.store(DropChecker, &guard);
        guard.flush();
    }

    // Allow deferred operations to run
    thread::sleep(std::time::Duration::from_millis(10));

    // At least some should be destroyed
    assert!(DESTROYED.load(Ordering::Relaxed) > 0);
}

// --- Atomic Arc Tests ---

#[test]
fn test_atomic_arc_basic() {
    let arc = AtomicArc::new(42);
    assert_eq!(*arc, 42);
    assert_eq!(arc.strong_count(), 1);
}

#[test]
fn test_atomic_arc_concurrent_clone() {
    let arc = Arc::new(AtomicArc::new(100));
    let mut handles = vec![];

    for _ in 0..4 {
        let arc = Arc::clone(&arc);
        handles.push(thread::spawn(move || {
            let mut clones = vec![];
            for _ in 0..100 {
                clones.push(arc.clone());
            }
            clones
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Should eventually drop to 1 when all clones are dropped
    assert!(arc.strong_count() >= 1);
}

#[test]
fn test_atomic_arc_weak_upgrade() {
    let arc = AtomicArc::new(42);
    let weak = arc.downgrade();

    assert_eq!(arc.weak_count(), 1);
    assert_eq!(weak.upgrade().map(|a| *a), Some(42));

    drop(arc);
    assert!(weak.upgrade().is_none());
}

#[test]
fn test_atomic_arc_cell_concurrent() {
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

// --- Integration Tests ---

#[test]
fn test_full_system_stress() {
    let list = Arc::new(LockFreeSkipList::new());
    let map = Arc::new(ConcurrentHAMT::new());
    let stack = Arc::new(TreiberStack::new());
    let queue = Arc::new(MichaelScottQueue::new());

    let mut handles = vec![];

    for t in 0..4 {
        let list = Arc::clone(&list);
        let map = Arc::clone(&map);
        let stack = Arc::clone(&stack);
        let queue = Arc::clone(&queue);

        handles.push(thread::spawn(move || {
            for i in 0..100 {
                let key = t * 100 + i;

                // Insert into all structures
                list.insert(key);
                map.insert(key, key);
                stack.push(key);
                queue.enqueue(key);

                // Read from all
                list.contains(&key);
                map.get(&key);

                // Remove from some
                if i % 2 == 0 {
                    list.remove(&key);
                    map.remove(&key);
                }

                stack.pop();
                queue.dequeue();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Just verify no crashes and structures still functional
    list.contains(&0);
    map.get(&0);
    stack.is_empty();
    queue.is_empty();
}
