// tests/warm_path_integration_test.rs - Comprehensive warm path integration tests
// Phase 3: End-to-end testing of warm path with descriptor management

use knhk_warm::kernel::{
    WarmPathExecutor, WorkItem,
    DescriptorManager, Descriptor, DescriptorContent, DescriptorVersion,
    Rule, Constraints,
    VersionGraph, RollbackManager, TimeTravelExecutor,
    TelemetryPipeline, TraceContext,
    ChannelManager, CoordinationMessage, BackpressureController,
    DegradationManager, DegradationStrategy,
    KnowledgeBase, MAPEKIntegration, MAPEKPhase, HookPoint, TriggerCondition,
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

#[test]
fn test_end_to_end_warm_path_execution() {
    // Initialize all components
    let warm_executor = Arc::new(WarmPathExecutor::new(1000, 100, 1024 * 1024));
    let descriptor_mgr = Arc::new(DescriptorManager::new(create_initial_descriptor()));
    let telemetry = Arc::new(TelemetryPipeline::new(10000, 100000, Duration::from_secs(60)));
    let coordination = Arc::new(ChannelManager::new(100));
    let degradation = Arc::new(DegradationManager::new());
    let knowledge = Arc::new(KnowledgeBase::new());
    let mapek = Arc::new(MAPEKIntegration::new(Arc::clone(&knowledge)));

    // Register MAPE-K hooks
    mapek.register_hook(HookPoint {
        id: "warm-execute".to_string(),
        phase: MAPEKPhase::Execute,
        location: "warm_path".to_string(),
        trigger_condition: TriggerCondition::Always,
    });

    // Start telemetry batching
    let _telemetry_handle = telemetry.start_auto_batching(Duration::from_millis(100));

    // Spawn work producer
    let executor_producer = Arc::clone(&warm_executor);
    let producer = thread::spawn(move || {
        for i in 0..100 {
            let work = WorkItem {
                id: i,
                payload: vec![0; 100],
                priority: (i % 10) as u8,
                created_at: Instant::now(),
                deadline: Some(Instant::now() + Duration::from_millis(100)),
            };

            let _ = executor_producer.submit_work(work);
            thread::sleep(Duration::from_micros(100));
        }
    });

    // Spawn warm path executor
    let executor_worker = Arc::clone(&warm_executor);
    let telemetry_worker = Arc::clone(&telemetry);
    let mapek_worker = Arc::clone(&mapek);
    let worker = thread::spawn(move || {
        let mut successful = 0;
        let mut degraded = 0;

        for _ in 0..50 {
            let result = mapek_worker.execute_hook("warm-execute", || {
                Ok(executor_worker.execute())
            }).unwrap();

            match result {
                knhk_warm::kernel::WarmPathResult::Success { items_processed, .. } => {
                    successful += items_processed;
                }
                knhk_warm::kernel::WarmPathResult::Degraded { .. } => {
                    degraded += 1;
                }
                _ => {}
            }

            // Flush telemetry periodically
            if successful % 10 == 0 {
                let (receipts, metrics, events) = executor_worker.flush_telemetry();
                for receipt in receipts {
                    let _ = telemetry_worker.process_receipt(receipt);
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        (successful, degraded)
    });

    // Spawn descriptor updater
    let descriptor_updater = Arc::clone(&descriptor_mgr);
    let updater = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));

        for version in 2..=3 {
            let new_descriptor = create_versioned_descriptor(version);
            let result = descriptor_updater.hot_swap(new_descriptor);
            assert!(result.is_ok());
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Wait for completion
    producer.join().unwrap();
    let (successful, degraded) = worker.join().unwrap();
    updater.join().unwrap();

    // Verify results
    assert!(successful > 0, "Should have processed some items");
    println!("Processed {} items, {} degradations", successful, degraded);

    // Check telemetry
    let stats = telemetry.get_stats();
    assert!(stats.receipts_processed > 0);

    // Check knowledge base
    let kb_stats = knowledge.get_stats();
    assert!(kb_stats.patterns_learned > 0 || kb_stats.predictions_made > 0);

    // Verify descriptor was updated
    assert_eq!(descriptor_mgr.get_current().version(), 3);
}

#[test]
fn test_degradation_and_recovery() {
    let warm_executor = Arc::new(WarmPathExecutor::new(100, 100, 1024 * 1024));
    let degradation = Arc::new(DegradationManager::new());

    // Simulate increasing load
    let loads = vec![0.3, 0.6, 0.88, 0.96, 0.99, 0.85, 0.70, 0.40];

    for load in loads {
        let decision = degradation.update_degradation("warm_path".to_string(), load);

        // Submit work based on degradation
        for i in 0..10 {
            let work = WorkItem {
                id: i,
                payload: vec![0; 1000],
                priority: (i % 10) as u8,
                created_at: Instant::now(),
                deadline: None,
            };

            if degradation.should_accept_work("warm_path", work.priority) {
                let _ = warm_executor.submit_work(work);
            }
        }

        // Execute with degradation applied
        let result = warm_executor.execute();

        println!("Load: {:.2}, Decision: {:?}, Result: {:?}",
            load, decision.action, result);

        thread::sleep(Duration::from_millis(50));
    }

    let stats = degradation.get_metrics();
    assert!(stats.degradation_events > 0);
    assert!(stats.recovery_events > 0);
}

#[test]
fn test_version_time_travel() {
    let graph = Arc::new(VersionGraph::new());
    let rollback_mgr = Arc::new(RollbackManager::new(Arc::clone(&graph)));
    let executor = TimeTravelExecutor::new(Arc::clone(&graph), Arc::clone(&rollback_mgr));

    // Create version history
    for i in 1..=5 {
        let version = create_version(i);
        graph.add_version(version).unwrap();

        // Create snapshot at each version
        let state = vec![i as u8; 100];
        executor.create_snapshot(state).unwrap();
    }

    // Tag important versions
    graph.tag_version(3, "stable".to_string()).unwrap();
    graph.tag_version(5, "latest".to_string()).unwrap();

    // Travel back to version 3
    let state = executor.travel_to(3).unwrap();
    assert_eq!(state[0], 3);

    // Get timeline
    let timeline = executor.get_timeline(10);
    assert_eq!(timeline.len(), 5);

    // Test rollback
    let changes = rollback_mgr.rollback_to(2).unwrap();
    assert!(!changes.is_empty());
}

#[test]
fn test_circuit_breaker_protection() {
    let degradation = DegradationManager::new();
    let breaker = degradation.get_circuit_breaker("critical_path");

    let mut successes = 0;
    let mut failures = 0;

    for i in 0..20 {
        let result: Result<(), String> = breaker.call(|| {
            if i < 5 || i > 15 {
                Ok(())
            } else {
                Err("Simulated failure".to_string())
            }
        });

        match result {
            Ok(_) => successes += 1,
            Err(_) => failures += 1,
        }

        thread::sleep(Duration::from_millis(10));
    }

    println!("Circuit breaker - Successes: {}, Failures: {}", successes, failures);
    assert!(successes > 0);
    assert!(failures > 0);
}

#[test]
fn test_coordination_with_backpressure() {
    let coordination = Arc::new(ChannelManager::new(100));
    let backpressure = Arc::new(BackpressureController::new());

    // Producer with backpressure awareness
    let coord_producer = Arc::clone(&coordination);
    let bp_producer = Arc::clone(&backpressure);
    let producer = thread::spawn(move || {
        let mut sent = 0;
        let mut dropped = 0;

        for i in 0..200 {
            // Update backpressure based on queue state
            let queue_size = coord_producer.get_stats().multiplex_queue_size;
            bp_producer.update_queue_depth("work_queue".to_string(), queue_size, 100);

            let priority = (i % 10) as u8;
            if bp_producer.should_accept_work(priority) {
                let msg = CoordinationMessage::WorkRequest {
                    priority,
                    estimated_cost: 100,
                };

                if coord_producer.send_multiplex(msg).is_ok() {
                    sent += 1;
                } else {
                    dropped += 1;
                }
            } else {
                dropped += 1;
            }

            if i % 50 == 0 {
                thread::sleep(Duration::from_millis(10));
            }
        }

        (sent, dropped)
    });

    // Consumer
    let coord_consumer = Arc::clone(&coordination);
    let consumer = thread::spawn(move || {
        let mut received = 0;
        let end_time = Instant::now() + Duration::from_secs(2);

        while Instant::now() < end_time {
            if let Some(_) = coord_consumer.recv_multiplex() {
                received += 1;
            }
            thread::sleep(Duration::from_millis(5));
        }

        received
    });

    let (sent, dropped) = producer.join().unwrap();
    let received = consumer.join().unwrap();

    println!("Backpressure test - Sent: {}, Dropped: {}, Received: {}",
        sent, dropped, received);

    assert!(sent > 0);
    assert!(received > 0);
    assert!(dropped > 0); // Some should be dropped due to backpressure
}

// Helper functions

fn create_initial_descriptor() -> Descriptor {
    create_versioned_descriptor(1)
}

fn create_versioned_descriptor(version: u64) -> Descriptor {
    let content = DescriptorContent {
        id: format!("test-v{}", version),
        schema_version: "1.0.0".to_string(),
        rules: vec![
            Rule {
                id: "rule1".to_string(),
                condition: "true".to_string(),
                action: "allow".to_string(),
                priority: 10,
            },
        ],
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
            message: format!("Version {}", version),
            tags: vec![],
        },
        content,
        compiled: None,
    }
}

fn create_version(id: u64) -> knhk_warm::kernel::versioning::Version {
    knhk_warm::kernel::versioning::Version {
        id,
        tag: None,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        parent: if id > 1 { Some(id - 1) } else { None },
        author: knhk_warm::kernel::versioning::VersionAuthor {
            id: "test".to_string(),
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
            public_key: None,
        },
        changes: vec![],
        signature: None,
        hash: [0; 32],
        dependencies: vec![],
        compatibility: knhk_warm::kernel::versioning::CompatibilityInfo {
            breaking_changes: false,
            forward_compatible: true,
            backward_compatible: true,
            min_compatible_version: 1,
            migration_required: false,
        },
    }
}