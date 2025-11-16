//! Benchmarks for lock-free concurrent data structures
//!
//! This benchmark suite measures:
//! - Throughput under different thread counts
//! - Scalability of lock-free operations
//! - Performance vs standard library structures
//! - Memory reclamation overhead

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

// Import our concurrent structures
// Note: Adjust paths as needed based on your project structure
#[path = "../src/concurrent/skiplist.rs"]
mod skiplist;
#[path = "../src/concurrent/hamt.rs"]
mod hamt;
#[path = "../src/concurrent/stack_queue.rs"]
mod stack_queue;
#[path = "../src/concurrent/epoch.rs"]
mod epoch;
#[path = "../src/concurrent/arc_atomic.rs"]
mod arc_atomic;

use skiplist::LockFreeSkipList;
use hamt::ConcurrentHAMT;
use stack_queue::{TreiberStack, MichaelScottQueue};
use epoch::{Guard, Atomic};
use arc_atomic::{AtomicArc, AtomicArcCell};

fn bench_skip_list_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("skiplist_insert");

    for thread_count in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(10000));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    let list = Arc::new(LockFreeSkipList::new());
                    let mut handles = vec![];

                    for t in 0..threads {
                        let list = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            let ops_per_thread = 10000 / threads;
                            for i in 0..ops_per_thread {
                                list.insert(t * ops_per_thread + i);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_skip_list_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("skiplist_search");

    // Pre-populate list
    let list = Arc::new(LockFreeSkipList::new());
    for i in 0..10000 {
        list.insert(i);
    }

    for thread_count in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(10000));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    let list = Arc::clone(&list);
                    let mut handles = vec![];

                    for t in 0..threads {
                        let list = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            let ops_per_thread = 10000 / threads;
                            for i in 0..ops_per_thread {
                                list.contains(&(t * ops_per_thread + i));
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_skip_list_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("skiplist_mixed");

    for thread_count in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(10000));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    let list = Arc::new(LockFreeSkipList::new());

                    // Pre-populate
                    for i in 0..5000 {
                        list.insert(i);
                    }

                    let mut handles = vec![];

                    for t in 0..threads {
                        let list = Arc::clone(&list);
                        handles.push(thread::spawn(move || {
                            let ops_per_thread = 10000 / threads;
                            for i in 0..ops_per_thread {
                                let key = t * ops_per_thread + i;
                                if key % 3 == 0 {
                                    list.insert(key);
                                } else if key % 3 == 1 {
                                    list.remove(&key);
                                } else {
                                    list.contains(&key);
                                }
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_hamt_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamt_insert");

    for thread_count in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(1000));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(ConcurrentHAMT::new());
                    let mut handles = vec![];

                    for t in 0..threads {
                        let map = Arc::clone(&map);
                        handles.push(thread::spawn(move || {
                            let ops_per_thread = 1000 / threads;
                            for i in 0..ops_per_thread {
                                map.insert(t * ops_per_thread + i, i);
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_hamt_vs_mutex_hashmap(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamt_vs_mutex");
    group.throughput(Throughput::Elements(1000));

    // Benchmark HAMT
    group.bench_function("hamt", |b| {
        b.iter(|| {
            let map = Arc::new(ConcurrentHAMT::new());
            let mut handles = vec![];

            for t in 0..4 {
                let map = Arc::clone(&map);
                handles.push(thread::spawn(move || {
                    for i in 0..250 {
                        map.insert(t * 250 + i, i);
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // Benchmark Mutex<HashMap>
    group.bench_function("mutex_hashmap", |b| {
        b.iter(|| {
            let map = Arc::new(Mutex::new(HashMap::new()));
            let mut handles = vec![];

            for t in 0..4 {
                let map = Arc::clone(&map);
                handles.push(thread::spawn(move || {
                    for i in 0..250 {
                        map.lock().unwrap().insert(t * 250 + i, i);
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_stack_push_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_operations");

    for thread_count in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(10000));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    let stack = Arc::new(TreiberStack::new());
                    let mut handles = vec![];

                    for t in 0..threads {
                        let stack = Arc::clone(&stack);
                        handles.push(thread::spawn(move || {
                            let ops_per_thread = 10000 / threads;
                            for i in 0..ops_per_thread {
                                stack.push(t * ops_per_thread + i);
                                stack.pop();
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_queue_enqueue_dequeue(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue_operations");

    for thread_count in [1, 2, 4, 8] {
        group.throughput(Throughput::Elements(10000));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &threads| {
                b.iter(|| {
                    let queue = Arc::new(MichaelScottQueue::new());
                    let mut handles = vec![];

                    for t in 0..threads {
                        let queue = Arc::clone(&queue);
                        handles.push(thread::spawn(move || {
                            let ops_per_thread = 10000 / threads;
                            for i in 0..ops_per_thread {
                                queue.enqueue(t * ops_per_thread + i);
                                queue.dequeue();
                            }
                        }));
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_epoch_reclamation(c: &mut Criterion) {
    let mut group = c.benchmark_group("epoch_reclamation");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("atomic_updates", |b| {
        b.iter(|| {
            let atomic = Arc::new(Atomic::new(0));
            let mut handles = vec![];

            for t in 0..4 {
                let atomic = Arc::clone(&atomic);
                handles.push(thread::spawn(move || {
                    for i in 0..250 {
                        let guard = Guard::pin();
                        atomic.store(t * 250 + i, &guard);
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_atomic_arc(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_arc");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("clone_drop", |b| {
        b.iter(|| {
            let arc = Arc::new(AtomicArc::new(vec![1, 2, 3, 4, 5]));
            let mut handles = vec![];

            for _ in 0..4 {
                let arc = Arc::clone(&arc);
                handles.push(thread::spawn(move || {
                    for _ in 0..250 {
                        let _clone = arc.clone();
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.bench_function("weak_upgrade", |b| {
        b.iter(|| {
            let arc = Arc::new(AtomicArc::new(42));
            let weak = arc.downgrade();
            let weak_arc = Arc::new(weak);
            let mut handles = vec![];

            for _ in 0..4 {
                let weak = Arc::clone(&weak_arc);
                handles.push(thread::spawn(move || {
                    for _ in 0..250 {
                        weak.upgrade();
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_atomic_arc_cell(c: &mut Criterion) {
    let mut group = c.benchmark_group("atomic_arc_cell");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("store_load", |b| {
        b.iter(|| {
            let cell = Arc::new(AtomicArcCell::new(AtomicArc::new(0)));
            let mut handles = vec![];

            for t in 0..4 {
                let cell = Arc::clone(&cell);
                handles.push(thread::spawn(move || {
                    for i in 0..250 {
                        cell.store(AtomicArc::new(t * 250 + i));
                        cell.load();
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_skip_list_insert,
    bench_skip_list_search,
    bench_skip_list_mixed,
    bench_hamt_insert,
    bench_hamt_vs_mutex_hashmap,
    bench_stack_push_pop,
    bench_queue_enqueue_dequeue,
    bench_epoch_reclamation,
    bench_atomic_arc,
    bench_atomic_arc_cell,
);

criterion_main!(benches);
