// tests/warm_path_tests/telemetry_coordination_test.rs - Test telemetry and coordination
// Phase 3: Verify telemetry pipeline and coordination don't block hot path

use knhk_warm::kernel::{
    TelemetryPipeline, TelemetryReceipt, TraceContext, CorrelatedEvent,
    ChannelManager, CoordinationMessage, BackpressureController, BackpressureLevel,
    ShutdownCoordinator, HealthMonitor, HealthStatus,
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn test_telemetry_pipeline_throughput() {
    let pipeline = Arc::new(TelemetryPipeline::new(10000, 100000, Duration::from_secs(60)));

    let producer_count = 5;
    let receipts_per_producer = 10000;
    let start = Instant::now();

    let mut producers = Vec::new();
    for producer_id in 0..producer_count {
        let pipeline_clone = Arc::clone(&pipeline);

        let handle = thread::spawn(move || {
            for i in 0..receipts_per_producer {
                let receipt = TelemetryReceipt {
                    id: format!("receipt-{}-{}", producer_id, i),
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    execution_time_us: 100,
                    trace_id: format!("trace-{}", producer_id),
                    span_id: format!("span-{}-{}", producer_id, i),
                    parent_span_id: None,
                    attributes: HashMap::new(),
                    status: knhk_warm::kernel::telemetry_pipeline::ReceiptStatus::Success,
                };

                let _ = pipeline_clone.process_receipt(receipt);
            }
        });

        producers.push(handle);
    }

    // Start auto-batching
    let _running = pipeline.start_auto_batching(Duration::from_millis(100));

    // Collect batches
    let mut total_receipts = 0;
    let consumer_pipeline = Arc::clone(&pipeline);
    let consumer = thread::spawn(move || {
        let mut batch_count = 0;
        let end_time = Instant::now() + Duration::from_secs(5);

        while Instant::now() < end_time {
            match consumer_pipeline.receive_batch() {
                Ok(batch) => {
                    batch_count += 1;
                    total_receipts += batch.receipts.len();
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }

        (batch_count, total_receipts)
    });

    // Wait for producers
    for handle in producers {
        handle.join().unwrap();
    }

    // Final flush
    pipeline.flush_batch().ok();

    // Get consumer results
    let (batches, receipts) = consumer.join().unwrap();

    let elapsed = start.elapsed();
    let throughput = (receipts as f64) / elapsed.as_secs_f64();

    println!("Telemetry pipeline throughput: {:.0} receipts/sec", throughput);
    println!("Total batches: {}, Total receipts: {}", batches, receipts);

    // Verify throughput meets requirements
    assert!(throughput > 10000.0, "Pipeline throughput should exceed 10k/sec");

    // Check statistics
    let stats = pipeline.get_stats();
    assert!(stats.receipts_processed > 0);
    assert!(stats.batches_sent > 0);
}

#[test]
fn test_coordination_channels() {
    let manager = Arc::new(ChannelManager::new(100));

    // Test control channel
    let msg = CoordinationMessage::HealthCheck {
        requester_id: "test".to_string(),
    };
    assert!(manager.send_control(msg.clone()).is_ok());
    assert!(matches!(manager.recv_control(), Ok(_)));

    // Test work channel with multiple producers
    let producer_count = 3;
    let messages_per_producer = 100;

    let mut producers = Vec::new();
    for producer_id in 0..producer_count {
        let manager_clone = Arc::clone(&manager);

        let handle = thread::spawn(move || {
            for i in 0..messages_per_producer {
                let msg = CoordinationMessage::WorkRequest {
                    priority: (i % 10) as u8,
                    estimated_cost: 100,
                };
                let _ = manager_clone.send_work(msg);
                thread::yield_now();
            }
        });

        producers.push(handle);
    }

    // Consumer thread
    let manager_consumer = Arc::clone(&manager);
    let consumer = thread::spawn(move || {
        let mut received = 0;
        let timeout = Duration::from_millis(100);
        let end_time = Instant::now() + Duration::from_secs(2);

        while Instant::now() < end_time {
            match manager_consumer.recv_work_timeout(timeout) {
                Ok(_) => received += 1,
                Err(_) => continue,
            }
        }

        received
    });

    for handle in producers {
        handle.join().unwrap();
    }

    let total_received = consumer.join().unwrap();
    assert!(total_received > 0);

    let stats = manager.get_stats();
    println!("Channel stats - Sent: {}, Received: {}",
        stats.messages_sent, stats.messages_received);
}

#[test]
fn test_backpressure_controller() {
    let controller = Arc::new(BackpressureController::new());

    // Simulate increasing load
    let queue_capacity = 100;

    controller.update_queue_depth("test_queue".to_string(), 30, queue_capacity);
    assert_eq!(controller.get_level(), BackpressureLevel::None);
    assert!(controller.should_accept_work(1));

    controller.update_queue_depth("test_queue".to_string(), 60, queue_capacity);
    assert_eq!(controller.get_level(), BackpressureLevel::Medium);
    assert!(!controller.should_accept_work(2));
    assert!(controller.should_accept_work(6));

    controller.update_queue_depth("test_queue".to_string(), 90, queue_capacity);
    assert_eq!(controller.get_level(), BackpressureLevel::High);
    assert!(!controller.should_accept_work(6));
    assert!(controller.should_accept_work(8));

    controller.update_queue_depth("test_queue".to_string(), 98, queue_capacity);
    assert_eq!(controller.get_level(), BackpressureLevel::Critical);
    assert!(!controller.should_accept_work(8));
    assert!(controller.should_accept_work(10));
}

#[test]
fn test_graceful_shutdown() {
    let coordinator = Arc::new(ShutdownCoordinator::new());

    // Register components with dependencies
    coordinator.register_component("database".to_string(), vec![]);
    coordinator.register_component("cache".to_string(), vec![]);
    coordinator.register_component("api".to_string(), vec!["database".to_string(), "cache".to_string()]);
    coordinator.register_component("worker".to_string(), vec!["database".to_string()]);

    // Initiate graceful shutdown
    coordinator.initiate_shutdown(true, Duration::from_secs(5));
    assert!(coordinator.is_shutdown_requested());

    // Simulate component shutdown in correct order
    coordinator.notify_component_stopping("api");
    coordinator.notify_component_stopped("api");

    coordinator.notify_component_stopping("worker");
    coordinator.notify_component_stopped("worker");

    coordinator.notify_component_stopping("cache");
    coordinator.notify_component_stopped("cache");

    coordinator.notify_component_stopping("database");
    coordinator.notify_component_stopped("database");

    assert!(coordinator.is_shutdown_complete());

    let (stopped, total) = coordinator.get_shutdown_progress();
    assert_eq!(stopped, total);
}

#[test]
fn test_health_monitoring() {
    let monitor = Arc::new(HealthMonitor::new(Duration::from_millis(100)));

    // Register health checks
    let healthy_count = Arc::new(AtomicU64::new(0));
    let healthy_clone = Arc::clone(&healthy_count);
    monitor.register_health_check("component_a".to_string(), move || {
        healthy_clone.fetch_add(1, Ordering::Relaxed);
        HealthStatus::Healthy
    });

    let degraded_count = Arc::new(AtomicU64::new(0));
    let degraded_clone = Arc::clone(&degraded_count);
    monitor.register_health_check("component_b".to_string(), move || {
        degraded_clone.fetch_add(1, Ordering::Relaxed);
        if degraded_clone.load(Ordering::Relaxed) > 3 {
            HealthStatus::Degraded("High load".to_string())
        } else {
            HealthStatus::Healthy
        }
    });

    // Run health checks multiple times
    for _ in 0..5 {
        monitor.run_health_checks();
        thread::sleep(Duration::from_millis(150));
    }

    // Verify health status
    let overall = monitor.get_health();
    assert!(matches!(overall, HealthStatus::Degraded(_)));

    let component_a_health = monitor.get_component_health("component_a");
    assert!(matches!(component_a_health, Some(HealthStatus::Healthy)));

    let component_b_health = monitor.get_component_health("component_b");
    assert!(matches!(component_b_health, Some(HealthStatus::Degraded(_))));
}

#[test]
fn test_telemetry_rate_limiting() {
    let pipeline = TelemetryPipeline::new(100, 100, Duration::from_secs(60));

    // Flood with receipts
    let mut accepted = 0;
    let mut rejected = 0;

    for i in 0..200 {
        let receipt = TelemetryReceipt {
            id: format!("receipt-{}", i),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            execution_time_us: 50,
            trace_id: "test".to_string(),
            span_id: format!("span-{}", i),
            parent_span_id: None,
            attributes: HashMap::new(),
            status: knhk_warm::kernel::telemetry_pipeline::ReceiptStatus::Success,
        };

        match pipeline.process_receipt(receipt) {
            Ok(_) => accepted += 1,
            Err(_) => rejected += 1,
        }
    }

    println!("Rate limiting - Accepted: {}, Rejected: {}", accepted, rejected);
    assert!(rejected > 0, "Some receipts should be rate limited");

    let stats = pipeline.get_stats();
    assert!(stats.rate_limited_items > 0);
}

#[test]
fn test_event_correlation() {
    let pipeline = TelemetryPipeline::new(1000, 10000, Duration::from_secs(60)));

    let trace_id = "trace-123";
    let correlation_id = "corr-456";

    // Create correlated events
    for i in 0..5 {
        let event = CorrelatedEvent {
            id: format!("event-{}", i),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            event_type: "test".to_string(),
            trace_id: trace_id.to_string(),
            correlation_id: correlation_id.to_string(),
            data: HashMap::new(),
            related_events: vec![],
        };

        pipeline.process_event(event);
    }

    // Flush and verify correlation
    let _ = pipeline.flush_batch();

    let stats = pipeline.get_stats();
    assert_eq!(stats.events_correlated, 5);
}

#[test]
fn test_coordination_under_load() {
    let manager = Arc::new(ChannelManager::new(1000));
    let message_count = 10000;
    let start = Instant::now();

    // Producer thread
    let manager_producer = Arc::clone(&manager);
    let producer = thread::spawn(move || {
        for i in 0..message_count {
            let msg = CoordinationMessage::LoadReport {
                current: (i % 100) as f64,
                predicted: ((i + 10) % 100) as f64,
                capacity: 100.0,
            };

            let _ = manager_producer.send_multiplex(msg);

            if i % 100 == 0 {
                thread::yield_now();
            }
        }
    });

    // Consumer thread
    let manager_consumer = Arc::clone(&manager);
    let consumer = thread::spawn(move || {
        let mut received = 0;
        let end_time = Instant::now() + Duration::from_secs(5);

        while received < message_count && Instant::now() < end_time {
            if let Some(_) = manager_consumer.recv_multiplex() {
                received += 1;
            } else {
                thread::yield_now();
            }
        }

        received
    });

    producer.join().unwrap();
    let total_received = consumer.join().unwrap();

    let elapsed = start.elapsed();
    let throughput = total_received as f64 / elapsed.as_secs_f64();

    println!("Coordination throughput: {:.0} msgs/sec", throughput);
    assert!(throughput > 1000.0, "Should handle >1000 msgs/sec");

    let stats = manager.get_stats();
    assert_eq!(stats.messages_sent, message_count as u64);
    assert_eq!(stats.messages_received, total_received as u64);
}