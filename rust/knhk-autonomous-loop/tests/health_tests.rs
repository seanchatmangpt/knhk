//! Tests for health monitoring

use knhk_autonomous_loop::cycle::{CycleResult, CycleStep};
use knhk_autonomous_loop::health::{HealthStats, LoopHealth};
use std::time::SystemTime;

#[test]
fn test_loop_health_running() {
    let health = LoopHealth::Running;
    assert!(health.is_healthy());
    assert!(!health.is_paused());
    assert!(!health.is_error());
    assert!(!health.is_stopped());
    assert_eq!(health.status(), "running");
}

#[test]
fn test_loop_health_paused() {
    let health = LoopHealth::Paused {
        reason: "manual pause".to_string(),
    };
    assert!(!health.is_healthy());
    assert!(health.is_paused());
    assert_eq!(health.status(), "paused");
    assert!(health.message().contains("manual pause"));
}

#[test]
fn test_loop_health_error() {
    let health = LoopHealth::Error {
        error: "test error".to_string(),
        retry_count: 2,
        last_error_time: Some(SystemTime::now()),
    };
    assert!(health.is_error());
    assert!(health.message().contains("test error"));
    assert!(health.message().contains("retry 2"));
}

#[test]
fn test_loop_health_stopped() {
    let health = LoopHealth::Stopped;
    assert!(health.is_stopped());
    assert_eq!(health.status(), "stopped");
}

#[test]
fn test_health_stats_default() {
    let stats = HealthStats::default();
    assert_eq!(stats.total_cycles, 0);
    assert_eq!(stats.successful_cycles, 0);
    assert_eq!(stats.failed_cycles, 0);
    assert_eq!(stats.error_rate, 0.0);
    assert_eq!(stats.success_rate(), 0.0);
}

#[test]
fn test_health_stats_record_success() {
    let mut stats = HealthStats::default();

    let result = CycleResult::Success {
        new_snapshot_id: [0u8; 32],
        duration_ms: 1000,
    };

    stats.record_cycle_result(&result, 1000);

    assert_eq!(stats.total_cycles, 1);
    assert_eq!(stats.successful_cycles, 1);
    assert_eq!(stats.failed_cycles, 0);
    assert_eq!(stats.error_rate, 0.0);
    assert_eq!(stats.success_rate(), 100.0);
    assert_eq!(stats.avg_cycle_duration_ms, 1000);
    assert!(stats.last_success_time.is_some());
}

#[test]
fn test_health_stats_record_failure() {
    let mut stats = HealthStats::default();

    let result = CycleResult::Failure {
        error: "test".to_string(),
        rollback_performed: false,
    };

    stats.record_cycle_result(&result, 500);

    assert_eq!(stats.total_cycles, 1);
    assert_eq!(stats.successful_cycles, 0);
    assert_eq!(stats.failed_cycles, 1);
    assert_eq!(stats.error_rate, 100.0);
    assert_eq!(stats.success_rate(), 0.0);
    assert!(stats.last_error_time.is_some());
}

#[test]
fn test_health_stats_mixed_results() {
    let mut stats = HealthStats::default();

    // 7 successes
    for _ in 0..7 {
        stats.record_cycle_result(
            &CycleResult::Success {
                new_snapshot_id: [0u8; 32],
                duration_ms: 1000,
            },
            1000,
        );
    }

    // 2 failures
    for _ in 0..2 {
        stats.record_cycle_result(
            &CycleResult::Failure {
                error: "test".to_string(),
                rollback_performed: false,
            },
            500,
        );
    }

    // 1 partial success
    stats.record_cycle_result(
        &CycleResult::PartialSuccess {
            patterns_detected: 5,
            proposals_rejected: 1,
            reason: "test".to_string(),
        },
        750,
    );

    assert_eq!(stats.total_cycles, 10);
    assert_eq!(stats.successful_cycles, 7);
    assert_eq!(stats.failed_cycles, 2);
    assert_eq!(stats.partial_cycles, 1);
    assert_eq!(stats.error_rate, 20.0);
    assert_eq!(stats.success_rate(), 70.0);
}

#[test]
fn test_health_stats_average_duration() {
    let mut stats = HealthStats::default();

    stats.record_cycle_result(
        &CycleResult::Success {
            new_snapshot_id: [0u8; 32],
            duration_ms: 1000,
        },
        1000,
    );

    stats.record_cycle_result(
        &CycleResult::Success {
            new_snapshot_id: [0u8; 32],
            duration_ms: 2000,
        },
        2000,
    );

    stats.record_cycle_result(
        &CycleResult::Success {
            new_snapshot_id: [0u8; 32],
            duration_ms: 3000,
        },
        3000,
    );

    assert_eq!(stats.avg_cycle_duration_ms, 2000);
}

#[test]
fn test_error_threshold_not_exceeded() {
    let mut stats = HealthStats::default();

    // 8 successes
    for _ in 0..8 {
        stats.record_cycle_result(
            &CycleResult::Success {
                new_snapshot_id: [0u8; 32],
                duration_ms: 1000,
            },
            1000,
        );
    }

    // 2 failures (20% error rate)
    for _ in 0..2 {
        stats.record_cycle_result(
            &CycleResult::Failure {
                error: "test".to_string(),
                rollback_performed: false,
            },
            500,
        );
    }

    assert_eq!(stats.error_rate, 20.0);
    assert!(!stats.exceeds_error_threshold(Some(25.0)));
    assert!(!stats.exceeds_error_threshold(Some(20.0))); // Equal, not exceeded
    assert!(!stats.exceeds_error_threshold(None)); // No threshold
}

#[test]
fn test_error_threshold_exceeded() {
    let mut stats = HealthStats::default();

    // 5 successes
    for _ in 0..5 {
        stats.record_cycle_result(
            &CycleResult::Success {
                new_snapshot_id: [0u8; 32],
                duration_ms: 1000,
            },
            1000,
        );
    }

    // 5 failures (50% error rate)
    for _ in 0..5 {
        stats.record_cycle_result(
            &CycleResult::Failure {
                error: "test".to_string(),
                rollback_performed: false,
            },
            500,
        );
    }

    assert_eq!(stats.error_rate, 50.0);
    assert!(stats.exceeds_error_threshold(Some(25.0)));
    assert!(stats.exceeds_error_threshold(Some(40.0)));
}

#[test]
fn test_error_threshold_requires_min_cycles() {
    let mut stats = HealthStats::default();

    // Only 5 cycles (below minimum of 10)
    for _ in 0..5 {
        stats.record_cycle_result(
            &CycleResult::Failure {
                error: "test".to_string(),
                rollback_performed: false,
            },
            500,
        );
    }

    assert_eq!(stats.error_rate, 100.0);
    // Should not trigger threshold because total_cycles < 10
    assert!(!stats.exceeds_error_threshold(Some(50.0)));

    // Add 5 more failures
    for _ in 0..5 {
        stats.record_cycle_result(
            &CycleResult::Failure {
                error: "test".to_string(),
                rollback_performed: false,
            },
            500,
        );
    }

    assert_eq!(stats.total_cycles, 10);
    // Now should trigger
    assert!(stats.exceeds_error_threshold(Some(50.0)));
}
