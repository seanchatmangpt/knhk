// tests/warm_path_tests/descriptor_swap_test.rs - Test descriptor hot-swapping under load
// Phase 3: Verify atomic descriptor updates with minimal reader impact

use knhk_warm::kernel::{
    DescriptorManager, Descriptor, DescriptorContent, DescriptorVersion,
    Rule, Pattern, Constraints,
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use crossbeam::epoch;

fn create_test_descriptor(version: u64, rules_count: usize) -> Descriptor {
    let mut rules = Vec::new();
    for i in 0..rules_count {
        rules.push(Rule {
            id: format!("rule-{}", i),
            condition: format!("condition-{}", i),
            action: format!("action-{}", i),
            priority: (i % 10) as u8,
        });
    }

    let content = DescriptorContent {
        id: format!("descriptor-v{}", version),
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
            author: "test".to_string(),
            message: format!("Test version {}", version),
            tags: vec![],
        },
        content,
        compiled: None,
    }
}

#[test]
fn test_descriptor_swap_under_load() {
    let initial_descriptor = create_test_descriptor(1, 10);
    let manager = Arc::new(DescriptorManager::new(initial_descriptor));

    let reader_count = 10;
    let swap_count = 5;
    let reads_per_thread = 1000;

    // Track reader latencies
    let latencies = Arc::new(parking_lot::Mutex::new(Vec::new()));

    // Spawn reader threads
    let mut reader_threads = Vec::new();
    for thread_id in 0..reader_count {
        let manager_clone = Arc::clone(&manager);
        let latencies_clone = Arc::clone(&latencies);

        let handle = thread::spawn(move || {
            let guard = &epoch::pin();
            let mut local_latencies = Vec::new();

            for _ in 0..reads_per_thread {
                let start = Instant::now();
                let descriptor = manager_clone.get_current().load(guard);
                let duration = start.elapsed();

                unsafe {
                    if let Some(desc) = descriptor.as_ref() {
                        // Verify descriptor is valid
                        assert!(desc.version.version > 0);
                    }
                }

                local_latencies.push(duration.as_micros() as u64);
                thread::yield_now();
            }

            latencies_clone.lock().extend(local_latencies);
        });

        reader_threads.push(handle);
    }

    // Spawn swapper thread
    let manager_swap = Arc::clone(&manager);
    let swap_thread = thread::spawn(move || {
        for version in 2..=swap_count + 1 {
            thread::sleep(Duration::from_millis(50));

            let new_descriptor = create_test_descriptor(version, 20);
            let result = manager_swap.hot_swap(new_descriptor);

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), version);
        }
    });

    // Wait for all threads to complete
    swap_thread.join().unwrap();
    for handle in reader_threads {
        handle.join().unwrap();
    }

    // Analyze latencies
    let all_latencies = latencies.lock();
    let avg_latency = all_latencies.iter().sum::<u64>() / all_latencies.len() as u64;
    let max_latency = all_latencies.iter().max().copied().unwrap_or(0);

    println!("Reader latency - Average: {}µs, Max: {}µs", avg_latency, max_latency);

    // Verify latency SLA
    assert!(avg_latency < 100, "Average reader latency exceeds 100µs");
    assert!(max_latency < 1000, "Maximum reader latency exceeds 1ms");
}

#[test]
fn test_version_rollback() {
    let initial = create_test_descriptor(1, 5);
    let manager = DescriptorManager::new(initial);

    // Create version history
    for version in 2..=5 {
        let descriptor = create_test_descriptor(version, 10);
        manager.hot_swap(descriptor).unwrap();
    }

    // Verify current version
    assert_eq!(manager.get_current().version(), 5);

    // Rollback to version 3
    for _ in 0..2 {
        let result = manager.rollback();
        assert!(result.is_ok());
    }

    // Should be at version 3 now
    assert_eq!(manager.get_current().version(), 3);
}

#[test]
fn test_concurrent_swaps() {
    let initial = create_test_descriptor(1, 5);
    let manager = Arc::new(DescriptorManager::new(initial));

    let swap_threads = 5;
    let mut handles = Vec::new();

    for thread_id in 0..swap_threads {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let version = (thread_id + 2) * 100 + i;
                let descriptor = create_test_descriptor(version, 5);

                // Some swaps will fail due to version ordering
                let _ = manager_clone.hot_swap(descriptor);
                thread::yield_now();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state is consistent
    let final_version = manager.get_current().version();
    assert!(final_version > 1);

    // Verify transition log
    let transitions = manager.get_transitions();
    assert!(!transitions.is_empty());
}

#[test]
fn test_compatibility_checking() {
    let v1 = create_test_descriptor(1, 5);
    let manager = DescriptorManager::new(v1);

    // Create incompatible descriptor (major version change)
    let mut v2 = create_test_descriptor(2, 5);
    v2.content.schema_version = "2.0.0".to_string(); // Major version change

    let result = manager.hot_swap(v2);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Incompatible"));
}

#[test]
fn test_scheduled_updates() {
    let initial = create_test_descriptor(1, 5);
    let manager = DescriptorManager::new(initial);

    // Schedule update for 1 second in the future
    let future_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 1;
    let scheduled = create_test_descriptor(2, 10);
    manager.schedule_update(scheduled, future_time);

    // Process immediately - should not apply yet
    let results = manager.process_pending_updates();
    assert!(results.is_empty());

    // Wait and process again
    thread::sleep(Duration::from_secs(2));
    let results = manager.process_pending_updates();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());

    // Verify update was applied
    assert_eq!(manager.get_current().version(), 2);
}

#[test]
fn test_emergency_rollback() {
    let initial = create_test_descriptor(1, 5);
    let manager = DescriptorManager::new(initial);

    // Build version history
    for v in 2..=10 {
        manager.hot_swap(create_test_descriptor(v, 5)).unwrap();
    }

    // Emergency rollback to version 1
    let result = manager.emergency_rollback(1);
    assert!(result.is_ok());
    assert_eq!(manager.get_current().version(), 1);
}

#[test]
fn test_atomic_state_transition() {
    let initial = create_test_descriptor(1, 5);
    let manager = DescriptorManager::new(initial);

    // Perform atomic transition
    let result = manager.atomic_transition(|content| {
        let mut new_content = content.clone();
        new_content.rules.push(Rule {
            id: "new-rule".to_string(),
            condition: "always".to_string(),
            action: "allow".to_string(),
            priority: 10,
        });
        Ok(new_content)
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2);

    // Verify new rule was added
    let guard = &epoch::pin();
    let current = manager.get_current().load(guard);
    unsafe {
        let desc = current.as_ref().unwrap();
        assert_eq!(desc.content.rules.len(), 6); // Original 5 + 1 new
    }
}

#[test]
fn test_reader_writer_coordination() {
    let initial = create_test_descriptor(1, 100);
    let manager = Arc::new(DescriptorManager::new(initial));

    // Track successful reads
    let successful_reads = Arc::new(std::sync::atomic::AtomicU64::new(0));

    // Spawn aggressive reader
    let manager_reader = Arc::clone(&manager);
    let reads_clone = Arc::clone(&successful_reads);
    let reader = thread::spawn(move || {
        let guard = &epoch::pin();
        let end_time = Instant::now() + Duration::from_secs(2);

        while Instant::now() < end_time {
            let descriptor = manager_reader.get_current().load(guard);
            unsafe {
                if let Some(desc) = descriptor.as_ref() {
                    // Validate descriptor integrity
                    assert!(!desc.content.id.is_empty());
                    assert!(desc.version.version > 0);
                    reads_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    });

    // Spawn aggressive writer
    let manager_writer = Arc::clone(&manager);
    let writer = thread::spawn(move || {
        let end_time = Instant::now() + Duration::from_secs(2);
        let mut version = 2;

        while Instant::now() < end_time {
            let descriptor = create_test_descriptor(version, 50);
            let _ = manager_writer.hot_swap(descriptor);
            version += 1;
            thread::sleep(Duration::from_millis(10));
        }
    });

    reader.join().unwrap();
    writer.join().unwrap();

    let total_reads = successful_reads.load(std::sync::atomic::Ordering::Relaxed);
    println!("Successful reads during concurrent swaps: {}", total_reads);
    assert!(total_reads > 1000, "Should have many successful reads");
}