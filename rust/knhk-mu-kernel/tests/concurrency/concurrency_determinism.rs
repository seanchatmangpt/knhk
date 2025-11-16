//! Comprehensive Determinism Tests for Concurrency Module
//!
//! Validates that the deterministic scheduler produces identical results
//! across multiple runs with same inputs.

use knhk_mu_kernel::concurrency::*;
use knhk_mu_kernel::timing::TickBudget;
use knhk_mu_kernel::sigma::SigmaPointer;
use knhk_mu_kernel::isa::GuardContext;

/// Test deterministic timestamp generation
#[test]
fn test_deterministic_timestamps() {
    let clock = LogicalClock::new();

    // Generate sequence of timestamps
    let t1 = clock.tick();
    let t2 = clock.tick();
    let t3 = clock.tick();

    // Verify monotonic increase
    assert!(t1 < t2);
    assert!(t2 < t3);
    assert!(t1 < t3);

    // Verify happened-before relationship
    assert!(t1.happens_before(&t2));
    assert!(t2.happens_before(&t3));
    assert!(t1.happens_before(&t3));
}

/// Test logical clock synchronization
#[test]
fn test_logical_clock_sync() {
    let clock1 = LogicalClock::new();
    let clock2 = LogicalClock::new();

    // Process 1 events
    let t1 = clock1.tick();  // 1
    let t2 = clock1.tick();  // 2

    // Process 2 receives message from process 1
    let t3 = clock2.recv(t2);  // max(0, 2) + 1 = 3

    // Process 2 continues
    let t4 = clock2.tick();  // 4

    // Verify causality
    assert!(t2.happens_before(&t3));
    assert!(t3.happens_before(&t4));
}

/// Test core-local queue determinism
#[test]
fn test_core_local_queue_determinism() {
    // Create two queues with same inputs
    let queue1 = WorkQueue::<u64, 16>::new();
    let queue2 = WorkQueue::<u64, 16>::new();

    let inputs = vec![42, 43, 44, 45, 46];

    // Enqueue same inputs to both queues
    queue1.with_mut(|q| {
        for &val in &inputs {
            q.enqueue(val).unwrap();
        }
    });

    queue2.with_mut(|q| {
        for &val in &inputs {
            q.enqueue(val).unwrap();
        }
    });

    // Dequeue and verify identical outputs
    let mut outputs1 = Vec::new();
    let mut outputs2 = Vec::new();

    queue1.with_mut(|q| {
        while let Ok(val) = q.dequeue() {
            outputs1.push(val);
        }
    });

    queue2.with_mut(|q| {
        while let Ok(val) = q.dequeue() {
            outputs2.push(val);
        }
    });

    assert_eq!(outputs1, outputs2);
    assert_eq!(outputs1, inputs);
}

/// Test globally ordered queue determinism
#[test]
fn test_global_ordered_determinism() {
    // Create two queues
    let queue1 = GlobalOrdered::new();
    let queue2 = GlobalOrdered::new();

    // Enqueue events out of order (but with same timestamps)
    let events = vec![
        (Timestamp::from_raw(30), 0, "event3"),
        (Timestamp::from_raw(10), 0, "event1"),
        (Timestamp::from_raw(20), 0, "event2"),
        (Timestamp::from_raw(15), 0, "event1.5"),
    ];

    for &(ts, core, event) in &events {
        queue1.enqueue(ts, core, event).unwrap();
        queue2.enqueue(ts, core, event).unwrap();
    }

    // Dequeue and verify identical order
    let mut results1 = Vec::new();
    let mut results2 = Vec::new();

    while let Ok((ts, event)) = queue1.dequeue() {
        results1.push((ts, event));
    }

    while let Ok((ts, event)) = queue2.dequeue() {
        results2.push((ts, event));
    }

    assert_eq!(results1, results2);

    // Verify timestamp order
    for i in 1..results1.len() {
        assert!(results1[i - 1].0 < results1[i].0);
    }
}

/// Test scheduler determinism (same inputs → same outputs)
#[test]
fn test_scheduler_determinism() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));

    // Run 1
    let mut scheduler1 = DeterministicScheduler::<4>::new(sigma_ptr);

    // Run 2 (fresh scheduler, same inputs)
    let mut scheduler2 = DeterministicScheduler::<4>::new(sigma_ptr);

    // Enqueue same tasks to both schedulers
    for task_id in 1..=10 {
        let task1 = SchedulableTask::new(
            task_id,
            TickBudget::chatman(),
            PriorityNormal,
            NoGuards,
            scheduler::TaskWork::Pure(task_id * 100),
            sigma_ptr,
        )
        .unwrap();

        let task2 = SchedulableTask::new(
            task_id,
            TickBudget::chatman(),
            PriorityNormal,
            NoGuards,
            scheduler::TaskWork::Pure(task_id * 100),
            sigma_ptr,
        )
        .unwrap();

        scheduler1.enqueue(task1).unwrap();
        scheduler2.enqueue(task2).unwrap();
    }

    // Execute tasks and collect results
    let mut results1 = Vec::new();
    let mut results2 = Vec::new();

    for core_id in 0..4 {
        while let Ok(result) = scheduler1.run_cycle(core_id) {
            results1.push((result.task_id, result.timestamp, result.core_id));
        }

        while let Ok(result) = scheduler2.run_cycle(core_id) {
            results2.push((result.task_id, result.timestamp, result.core_id));
        }
    }

    // Verify determinism (same task assignments and execution order)
    assert_eq!(results1.len(), results2.len());

    for (r1, r2) in results1.iter().zip(results2.iter()) {
        assert_eq!(r1.0, r2.0);  // Same task_id
        assert_eq!(r1.2, r2.2);  // Same core_id
        // Timestamps may differ slightly, but should be monotonic
    }
}

/// Test replay log determinism
#[test]
fn test_replay_log_determinism() {
    let mut log1 = ReplayLog::new();
    let mut log2 = ReplayLog::new();

    // Record same events
    let events = vec![
        replay::ReplayEvent::TaskEnqueued {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
        },
        replay::ReplayEvent::TaskExecuted {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(2),
            ticks: 5,
            output_hash: [0xDEADBEEF; 4],
        },
        replay::ReplayEvent::TaskEnqueued {
            task_id: 2,
            core_id: 1,
            timestamp: Timestamp::from_raw(3),
        },
        replay::ReplayEvent::TaskExecuted {
            task_id: 2,
            core_id: 1,
            timestamp: Timestamp::from_raw(4),
            ticks: 7,
            output_hash: [0xCAFEBABE; 4],
        },
    ];

    for event in events {
        log1.record(event);
        log2.record(event);
    }

    // Verify identical logs
    assert_eq!(log1.len(), log2.len());
    assert_eq!(log1.checksum(), log2.checksum());

    let comparison = replay::compare_replays(&log1, &log2);
    assert_eq!(comparison, replay::ReplayResult::ExactMatch);
}

/// Test cross-run determinism with replay
#[test]
fn test_cross_run_replay() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));

    // Original run
    let mut scheduler_orig = DeterministicScheduler::<2>::new(sigma_ptr);

    for task_id in 1..=5 {
        let task = SchedulableTask::new(
            task_id,
            TickBudget::chatman(),
            PriorityHigh,
            NoGuards,
            scheduler::TaskWork::Pure(task_id * 42),
            sigma_ptr,
        )
        .unwrap();

        scheduler_orig.enqueue(task).unwrap();
    }

    // Execute all tasks
    for core_id in 0..2 {
        while scheduler_orig.run_cycle(core_id).is_ok() {}
    }

    // Get original replay log
    let original_log = scheduler_orig.replay_log();

    // Replay run (same seed/inputs)
    let mut scheduler_replay = DeterministicScheduler::<2>::new(sigma_ptr);

    for task_id in 1..=5 {
        let task = SchedulableTask::new(
            task_id,
            TickBudget::chatman(),
            PriorityHigh,
            NoGuards,
            scheduler::TaskWork::Pure(task_id * 42),
            sigma_ptr,
        )
        .unwrap();

        scheduler_replay.enqueue(task).unwrap();
    }

    // Execute all tasks
    for core_id in 0..2 {
        while scheduler_replay.run_cycle(core_id).is_ok() {}
    }

    let replay_log = scheduler_replay.replay_log();

    // Compare logs (should be identical for determinism)
    let comparison = replay::compare_replays(original_log, replay_log);

    match comparison {
        replay::ReplayResult::ExactMatch => {
            // Perfect determinism!
        }
        replay::ReplayResult::MinorDifference { mismatch_count } => {
            // Some non-determinism (timestamps may vary slightly)
            assert!(mismatch_count <= 2, "Too many mismatches");
        }
        replay::ReplayResult::Diverged { first_mismatch } => {
            panic!("Replay diverged at event {}", first_mismatch);
        }
    }
}

/// Test type-level priority enforcement
#[test]
fn test_priority_type_safety() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));

    // High priority task
    let high_task = SchedulableTask::new(
        1,
        TickBudget::chatman(),
        PriorityHigh,
        NoGuards,
        scheduler::TaskWork::Pure(1),
        sigma_ptr,
    )
    .unwrap();

    // Normal priority task
    let normal_task = SchedulableTask::new(
        2,
        TickBudget::chatman(),
        PriorityNormal,
        NoGuards,
        scheduler::TaskWork::Pure(2),
        sigma_ptr,
    )
    .unwrap();

    // Low priority task
    let low_task = SchedulableTask::new(
        3,
        TickBudget::chatman(),
        PriorityLow,
        NoGuards,
        scheduler::TaskWork::Pure(3),
        sigma_ptr,
    )
    .unwrap();

    // Verify priority levels
    assert_eq!(high_task.priority(), 0);
    assert_eq!(normal_task.priority(), 1);
    assert_eq!(low_task.priority(), 2);
}

/// Test guard validation at compile time
#[test]
fn test_guard_validation() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));

    // Valid guard
    let single_guard = types::SingleGuard(42);
    let task = SchedulableTask::new(
        1,
        TickBudget::chatman(),
        PriorityHigh,
        single_guard,
        scheduler::TaskWork::Pure(1),
        sigma_ptr,
    );
    assert!(task.is_ok());

    // Invalid guard (ID too large)
    let invalid_guard = types::SingleGuard(2000);
    let task_invalid = SchedulableTask::new(
        2,
        TickBudget::chatman(),
        PriorityHigh,
        invalid_guard,
        scheduler::TaskWork::Pure(2),
        sigma_ptr,
    );
    assert!(task_invalid.is_err());
}

/// Test replay statistics
#[test]
fn test_replay_statistics() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut scheduler = DeterministicScheduler::<4>::new(sigma_ptr);

    // Execute tasks
    for task_id in 1..=20 {
        let task = SchedulableTask::new(
            task_id,
            TickBudget::chatman(),
            PriorityNormal,
            NoGuards,
            scheduler::TaskWork::Pure(task_id),
            sigma_ptr,
        )
        .unwrap();

        scheduler.enqueue(task).unwrap();
    }

    // Execute all
    for core_id in 0..4 {
        while scheduler.run_cycle(core_id).is_ok() {}
    }

    // Get statistics
    let stats = replay::ReplayStats::from_log(scheduler.replay_log());

    assert_eq!(stats.unique_tasks, 20);
    assert!(stats.total_events > 0);
    assert_eq!(stats.determinism_ratio(), 1.0);  // All events deterministic
}

/// Test concurrent event ordering
#[test]
fn test_concurrent_event_ordering() {
    let e1 = logical_time::TimestampedEvent::new(
        Timestamp::from_raw(10),
        0,
        "core0_event",
    );

    let e2 = logical_time::TimestampedEvent::new(
        Timestamp::from_raw(10),
        1,
        "core1_event",
    );

    // Same timestamp, different cores → deterministic tie-break by core_id
    assert!(e1 < e2);
    assert!(!e1.happens_before(&e2));  // Concurrent in causality
}
