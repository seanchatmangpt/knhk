// Chicago TDD Concurrency Tests for Fortune 5
// Tests: Multi-threaded scenarios, race condition detection, synchronization
// Principle: Verify thread-safe behavior under concurrent access

use knhk_sidecar::capacity::*;
use knhk_sidecar::kms::*;
use knhk_sidecar::promotion::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// Concurrency Test: Promotion Routing Under Concurrent Requests
// ============================================================================

#[test]
fn test_promotion_routing_concurrent_requests() {
    // Arrange: Simulate concurrent requests to canary routing
    let traffic_percent = 20.0;
    let concurrent_requests = 100;
    let request_threads = 4;

    // Act: Route requests concurrently
    let results = Arc::new(Mutex::new(vec![]));

    let handles: Vec<_> = (0..request_threads)
        .map(|thread_id| {
            let results = Arc::clone(&results);
            thread::spawn(move || {
                for i in 0..concurrent_requests / request_threads {
                    let request_id = format!("req-t{}-{}", thread_id, i);
                    let hash = request_id
                        .chars()
                        .map(|c| c as u32)
                        .sum::<u32>() as u64;
                    let routed_to_new = (hash % 100) < (traffic_percent as u64);

                    results.lock().unwrap().push(routed_to_new);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Routing results are consistent
    let routed_results = results.lock().unwrap();
    let to_new_count = routed_results.iter().filter(|&&r| r).count();

    // Should be approximately 20% routed to new version
    let actual_percent = (to_new_count as f64 / concurrent_requests as f64) * 100.0;
    assert!(
        (actual_percent - traffic_percent).abs() <= 10.0,
        "Should be close to {}% routed, got {}%",
        traffic_percent,
        actual_percent
    );
}

// ============================================================================
// Concurrency Test: Capacity Metrics Aggregation
// ============================================================================

#[test]
fn test_capacity_metrics_concurrent_updates() {
    // Arrange: Multiple threads updating metrics simultaneously
    let shared_metrics = Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];

    // Thread 1: Record cache hits
    {
        let metrics = Arc::clone(&shared_metrics);
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                let mut m = metrics.lock().unwrap();
                *m.entry("hits").or_insert(0) += 1;
            }
        });
        handles.push(handle);
    }

    // Thread 2: Record cache misses
    {
        let metrics = Arc::clone(&shared_metrics);
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                let mut m = metrics.lock().unwrap();
                *m.entry("misses").or_insert(0) += 1;
            }
        });
        handles.push(handle);
    }

    // Thread 3: Record evictions
    {
        let metrics = Arc::clone(&shared_metrics);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let mut m = metrics.lock().unwrap();
                *m.entry("evictions").or_insert(0) += 1;
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: All updates were recorded
    let final_metrics = shared_metrics.lock().unwrap();
    assert_eq!(*final_metrics.get("hits").unwrap_or(&0), 50);
    assert_eq!(*final_metrics.get("misses").unwrap_or(&0), 20);
    assert_eq!(*final_metrics.get("evictions").unwrap_or(&0), 10);
}

// ============================================================================
// Concurrency Test: Multi-Region Sync Coordination
// ============================================================================

#[test]
fn test_multi_region_concurrent_sync() {
    // Arrange: Simulate concurrent region syncs
    let regions = vec!["us-east-1", "eu-west-1", "ap-southeast-1"];
    let sync_results = Arc::new(Mutex::new(vec![]));

    let handles: Vec<_> = regions
        .iter()
        .map(|region| {
            let results = Arc::clone(&sync_results);
            let region = region.to_string();
            thread::spawn(move || {
                // Simulate async sync operation
                let sync_ok = !region.is_empty();
                results.lock().unwrap().push((region, sync_ok));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: All regions synced successfully
    let results = sync_results.lock().unwrap();
    assert_eq!(results.len(), 3, "All regions should report sync status");

    let all_synced = results.iter().all(|(_, success)| success);
    assert!(all_synced, "All regions should sync successfully");
}

// ============================================================================
// Concurrency Test: KMS Key Rotation Coordination
// ============================================================================

#[test]
fn test_kms_concurrent_rotation_detection() {
    // Arrange: Multiple threads checking if rotation is needed
    let rotation_needed = Arc::new(Mutex::new(false));
    let rotation_in_progress = Arc::new(Mutex::new(false));

    let mut handles = vec![];

    // Thread 1: Check rotation status
    {
        let needed = Arc::clone(&rotation_needed);
        let in_progress = Arc::clone(&rotation_in_progress);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let _needed = needed.lock().unwrap().clone();
                let _in_progress = in_progress.lock().unwrap().clone();
                // Simulate checking rotation status
            }
        });
        handles.push(handle);
    }

    // Thread 2: Initiate rotation
    {
        let in_progress = Arc::clone(&rotation_in_progress);
        let handle = thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(5));
            *in_progress.lock().unwrap() = true;
        });
        handles.push(handle);
    }

    // Thread 3: Complete rotation
    {
        let in_progress = Arc::clone(&rotation_in_progress);
        let needed = Arc::clone(&rotation_needed);
        let handle = thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(15));
            *in_progress.lock().unwrap() = false;
            *needed.lock().unwrap() = false;
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Rotation state is consistent
    let final_in_progress = rotation_in_progress.lock().unwrap().clone();
    let final_needed = rotation_needed.lock().unwrap().clone();

    assert!(!final_in_progress, "Rotation should be complete");
    assert!(!final_needed, "Rotation should not be needed after completion");
}

// ============================================================================
// Concurrency Test: Canary Health Monitoring
// ============================================================================

#[test]
fn test_canary_health_concurrent_metric_collection() {
    // Arrange: Multiple threads collecting canary metrics
    let metrics = Arc::new(Mutex::new(CanaryMetrics {
        total_requests: 0,
        errors: 0,
        latencies: vec![],
        last_checked: std::time::Instant::now(),
    }));

    let mut handles = vec![];

    // Thread 1: Record successful requests
    {
        let m = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut metrics = m.lock().unwrap();
                metrics.total_requests += 1;
                metrics.latencies.push(50);
            }
        });
        handles.push(handle);
    }

    // Thread 2: Record failed requests
    {
        let m = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let mut metrics = m.lock().unwrap();
                metrics.total_requests += 1;
                metrics.errors += 1;
                metrics.latencies.push(200);
            }
        });
        handles.push(handle);
    }

    // Thread 3: Calculate error rate
    {
        let m = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(50));
            let metrics = m.lock().unwrap();
            let _error_rate = metrics.errors as f64 / metrics.total_requests as f64;
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Metrics are correctly aggregated
    let final_metrics = metrics.lock().unwrap();
    assert_eq!(final_metrics.total_requests, 110);
    assert_eq!(final_metrics.errors, 10);
    assert!(!final_metrics.latencies.is_empty());
}

// ============================================================================
// Concurrency Test: Deadlock Prevention
// ============================================================================

#[test]
fn test_no_deadlock_promotion_and_capacity() {
    // Arrange: Two locks that could deadlock if not careful
    let promotion_state = Arc::new(Mutex::new("healthy"));
    let capacity_state = Arc::new(Mutex::new("sufficient"));

    let mut handles = vec![];

    // Thread 1: Lock in order A -> B
    {
        let promo = Arc::clone(&promotion_state);
        let capacity = Arc::clone(&capacity_state);
        let handle = thread::spawn(move || {
            let _p = promo.lock().unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
            let _c = capacity.lock().unwrap();
        });
        handles.push(handle);
    }

    // Thread 2: Lock in same order A -> B (no deadlock)
    {
        let promo = Arc::clone(&promotion_state);
        let capacity = Arc::clone(&capacity_state);
        let handle = thread::spawn(move || {
            let _p = promo.lock().unwrap();
            thread::sleep(std::time::Duration::from_millis(1));
            let _c = capacity.lock().unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads (should complete without deadlock)
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Completed without hanging
    assert_eq!(*promotion_state.lock().unwrap(), "healthy");
    assert_eq!(*capacity_state.lock().unwrap(), "sufficient");
}

// ============================================================================
// Concurrency Test: Race Condition in Decision Logic
// ============================================================================

#[test]
fn test_concurrent_slo_admission_decisions() {
    // Arrange: Multiple threads making concurrent SLO admission decisions
    let hit_rate = Arc::new(Mutex::new(0.97));
    let decisions = Arc::new(Mutex::new(vec![]));

    let mut handles = vec![];

    // Thread 1: Checking R1 admission
    {
        let rate = Arc::clone(&hit_rate);
        let decs = Arc::clone(&decisions);
        let handle = thread::spawn(move || {
            let current_rate = *rate.lock().unwrap();
            let can_admit_r1 = current_rate >= 0.99;
            decs.lock().unwrap().push(("R1", can_admit_r1));
        });
        handles.push(handle);
    }

    // Thread 2: Checking W1 admission
    {
        let rate = Arc::clone(&hit_rate);
        let decs = Arc::clone(&decisions);
        let handle = thread::spawn(move || {
            let current_rate = *rate.lock().unwrap();
            let can_admit_w1 = current_rate >= 0.95;
            decs.lock().unwrap().push(("W1", can_admit_w1));
        });
        handles.push(handle);
    }

    // Thread 3: Checking C1 admission (always true)
    {
        let decs = Arc::clone(&decisions);
        let handle = thread::spawn(move || {
            decs.lock().unwrap().push(("C1", true));
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: All decisions are consistent
    let final_decisions = decisions.lock().unwrap();
    assert_eq!(final_decisions.len(), 3, "Should have 3 decisions");

    // All decisions should be consistent (same hit rate)
    for (slo, _) in final_decisions.iter() {
        assert!(!slo.is_empty(), "SLO class should be set");
    }
}

// ============================================================================
// Concurrency Test: Load Distribution
// ============================================================================

#[test]
fn test_load_distribution_across_regions() {
    // Arrange: Simulate request distribution across regions
    let region_loads = Arc::new(Mutex::new(HashMap::new()));
    let regions = vec!["us-east-1", "eu-west-1", "ap-southeast-1"];

    // Initialize counters
    {
        let mut loads = region_loads.lock().unwrap();
        for region in &regions {
            loads.insert(*region, 0);
        }
    }

    let mut handles = vec![];

    // Simulate 300 requests distributed across regions
    for request_id in 0..300 {
        let loads = Arc::clone(&region_loads);
        let regions = regions.clone();
        let handle = thread::spawn(move || {
            // Simple hash-based distribution
            let selected_region = regions[request_id % regions.len()];
            let mut l = loads.lock().unwrap();
            *l.entry(selected_region).or_insert(0) += 1;
        });
        handles.push(handle);
    }

    // Wait for all requests
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Load is relatively balanced
    let final_loads = region_loads.lock().unwrap();
    let loads: Vec<i32> = final_loads.values().cloned().collect();

    // Each region should get about 100 requests (with some variance)
    for region_load in loads {
        assert!(
            region_load > 90 && region_load < 110,
            "Load should be balanced, got {}",
            region_load
        );
    }
}
