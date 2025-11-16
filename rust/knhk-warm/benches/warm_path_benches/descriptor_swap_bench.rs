// benches/warm_path_benches/descriptor_swap_bench.rs - Benchmark descriptor swap performance
// Phase 3: Measure hot-swap latency and reader impact

#![feature(test)]
extern crate test;

use knhk_warm::kernel::{
    DescriptorManager, Descriptor, DescriptorContent, DescriptorVersion,
    Rule, Constraints,
};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use test::Bencher;
use crossbeam::epoch;

fn create_descriptor(version: u64, rule_count: usize) -> Descriptor {
    let mut rules = Vec::with_capacity(rule_count);
    for i in 0..rule_count {
        rules.push(Rule {
            id: format!("rule-{}", i),
            condition: format!("condition-{}", i),
            action: format!("action-{}", i),
            priority: (i % 10) as u8,
        });
    }

    let content = DescriptorContent {
        id: format!("bench-{}", version),
        schema_version: "1.0.0".to_string(),
        rules,
        patterns: vec![],
        constraints: Constraints {
            max_execution_time_us: 1000,
            max_memory_bytes: 1024 * 1024,
            required_capabilities: vec![],
            forbidden_operations: vec![],
        },
        metadata: HashMap::new(),
    };

    let hash = blake3::hash(serde_json::to_string(&content).unwrap().as_bytes()).into();

    Descriptor {
        version: DescriptorVersion {
            version,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            hash,
            parent_version: if version > 1 { Some(version - 1) } else { None },
            author: "bench".to_string(),
            message: format!("Benchmark version {}", version),
            tags: vec![],
        },
        content,
        compiled: None,
    }
}

#[bench]
fn bench_descriptor_swap_small(b: &mut Bencher) {
    let initial = create_descriptor(1, 10);
    let manager = DescriptorManager::new(initial);
    let mut version = 2;

    b.iter(|| {
        let descriptor = create_descriptor(version, 10);
        manager.hot_swap(descriptor).unwrap();
        version += 1;
    });
}

#[bench]
fn bench_descriptor_swap_large(b: &mut Bencher) {
    let initial = create_descriptor(1, 1000);
    let manager = DescriptorManager::new(initial);
    let mut version = 2;

    b.iter(|| {
        let descriptor = create_descriptor(version, 1000);
        manager.hot_swap(descriptor).unwrap();
        version += 1;
    });
}

#[bench]
fn bench_descriptor_read(b: &mut Bencher) {
    let descriptor = create_descriptor(1, 100);
    let manager = DescriptorManager::new(descriptor);
    let current = manager.get_current();

    b.iter(|| {
        let guard = &epoch::pin();
        test::black_box(current.load(guard));
    });
}

#[bench]
fn bench_concurrent_reads(b: &mut Bencher) {
    let descriptor = create_descriptor(1, 100);
    let manager = Arc::new(DescriptorManager::new(descriptor));

    b.iter(|| {
        let threads: Vec<_> = (0..4)
            .map(|_| {
                let mgr = Arc::clone(&manager);
                thread::spawn(move || {
                    let guard = &epoch::pin();
                    for _ in 0..100 {
                        test::black_box(mgr.get_current().load(guard));
                    }
                })
            })
            .collect();

        for t in threads {
            t.join().unwrap();
        }
    });
}

#[bench]
fn bench_version_rollback(b: &mut Bencher) {
    let initial = create_descriptor(1, 50);
    let manager = DescriptorManager::new(initial);

    // Build history
    for v in 2..=10 {
        manager.hot_swap(create_descriptor(v, 50)).unwrap();
    }

    b.iter(|| {
        manager.rollback().unwrap();
        manager.hot_swap(create_descriptor(11, 50)).unwrap();
    });
}

#[bench]
fn bench_atomic_transition(b: &mut Bencher) {
    let initial = create_descriptor(1, 50);
    let manager = DescriptorManager::new(initial);

    b.iter(|| {
        manager.atomic_transition(|content| {
            let mut new = content.clone();
            new.rules.push(Rule {
                id: "new".to_string(),
                condition: "true".to_string(),
                action: "allow".to_string(),
                priority: 5,
            });
            Ok(new)
        }).unwrap();
    });
}