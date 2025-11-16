//! Concurrency Tests for Session Management
//!
//! Tests lock-free atomic operations and concurrent session access:
//! - Atomic counter operations are thread-safe
//! - Session table handles concurrent access
//! - No data races or deadlocks
//! - Session isolation guarantees hold under concurrency
//!
//! **Chicago TDD Approach**: Tests real concurrent execution, not mocks

use knhk_workflow_engine::autonomic::session::{
    SessionId, SessionMetrics, SessionState, SessionTable, TenantId,
};
use knhk_workflow_engine::case::CaseId;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// ============================================================================
// Atomic Counter Tests
// ============================================================================

#[test]
fn test_session_metrics_concurrent_retry_increments() {
    // Arrange: Create shared metrics
    let metrics = Arc::new(SessionMetrics::new());
    let mut handles = vec![];

    // Act: Spawn 10 threads, each incrementing 1000 times
    for _ in 0..10 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                metrics_clone.increment_retries();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Total should be exactly 10,000
    assert_eq!(
        metrics.get_retry_count(),
        10_000,
        "Atomic increments should be thread-safe"
    );
}

#[test]
fn test_session_metrics_concurrent_latency_updates() {
    // Arrange
    let metrics = Arc::new(SessionMetrics::new());
    let mut handles = vec![];

    // Act: Concurrent latency updates
    for i in 0..10 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                metrics_clone.add_latency_us(100 + i);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Total latency should be correct
    let total = metrics.get_total_latency_us();
    assert!(
        total > 0,
        "Concurrent latency updates should accumulate"
    );
    assert!(
        total >= 100_000, // At least 1000 * 100
        "Latency should be at least base value"
    );
}

#[test]
fn test_session_metrics_concurrent_task_completions() {
    // Arrange
    let metrics = Arc::new(SessionMetrics::new());
    let mut handles = vec![];

    // Act: Concurrent task completions
    for _ in 0..10 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                metrics_clone.increment_task_completions();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    assert_eq!(metrics.get_task_completions(), 10_000);
}

#[test]
fn test_session_metrics_concurrent_violations() {
    // Arrange
    let metrics = Arc::new(SessionMetrics::new());
    let mut handles = vec![];

    // Act
    for _ in 0..5 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..200 {
                metrics_clone.increment_violations();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    assert_eq!(metrics.get_violation_count(), 1_000);
}

#[test]
fn test_session_metrics_concurrent_adaptations() {
    // Arrange
    let metrics = Arc::new(SessionMetrics::new());
    let mut handles = vec![];

    // Act
    for _ in 0..8 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..125 {
                metrics_clone.increment_adaptations();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    assert_eq!(metrics.get_adaptation_count(), 1_000);
}

// ============================================================================
// State Transition Tests
// ============================================================================

#[test]
fn test_session_state_concurrent_transitions() {
    // Arrange
    let metrics = Arc::new(SessionMetrics::new());
    let mut handles = vec![];

    // Act: Try to transition state from multiple threads
    // Only one should succeed
    for _ in 0..10 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_micros(10));
            metrics_clone.set_state(SessionState::Active);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: State should be Active
    assert_eq!(metrics.get_state(), SessionState::Active);
}

#[test]
fn test_session_state_progression() {
    // Arrange
    let metrics = SessionMetrics::new();

    // Act & Assert: Verify state progression
    assert_eq!(metrics.get_state(), SessionState::Created);

    metrics.set_state(SessionState::Active);
    assert_eq!(metrics.get_state(), SessionState::Active);

    metrics.set_state(SessionState::Adapted);
    assert_eq!(metrics.get_state(), SessionState::Adapted);

    metrics.set_state(SessionState::Completed);
    assert_eq!(metrics.get_state(), SessionState::Completed);
}

// ============================================================================
// Session Table Concurrency Tests
// ============================================================================

#[test]
fn test_session_table_concurrent_inserts() {
    // Arrange
    let table = Arc::new(SessionTable::new());
    let mut handles = vec![];

    // Act: Concurrent session creation
    for _ in 0..10 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let session_id = SessionId::new();
                let case_id = CaseId::new();
                let tenant_id = TenantId::default_tenant();

                table_clone.create_session(session_id, case_id, tenant_id);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Should have 1000 sessions
    assert_eq!(
        table.active_session_count(),
        1000,
        "Concurrent inserts should all succeed"
    );
}

#[test]
fn test_session_table_concurrent_lookups() {
    // Arrange: Create sessions first
    let table = Arc::new(SessionTable::new());
    let mut session_ids = vec![];

    for _ in 0..100 {
        let session_id = SessionId::new();
        let case_id = CaseId::new();
        let tenant_id = TenantId::default_tenant();

        table.create_session(session_id, case_id, tenant_id);
        session_ids.push(session_id);
    }

    // Act: Concurrent lookups
    let mut handles = vec![];
    for i in 0..10 {
        let table_clone = Arc::clone(&table);
        let ids_clone = session_ids.clone();

        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                let session_id = ids_clone[i % ids_clone.len()];
                let _metrics = table_clone.get_session_metrics(session_id);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: All lookups should succeed without panic
    assert_eq!(table.active_session_count(), 100);
}

#[test]
fn test_session_table_concurrent_updates() {
    // Arrange
    let table = Arc::new(SessionTable::new());
    let session_id = SessionId::new();
    let case_id = CaseId::new();
    let tenant_id = TenantId::default_tenant();

    table.create_session(session_id, case_id, tenant_id);

    // Act: Concurrent updates to same session
    let mut handles = vec![];
    for _ in 0..10 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            if let Some(metrics) = table_clone.get_session_metrics(session_id) {
                for _ in 0..1000 {
                    metrics.increment_retries();
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Total retries should be 10,000
    if let Some(metrics) = table.get_session_metrics(session_id) {
        assert_eq!(metrics.get_retry_count(), 10_000);
    } else {
        panic!("Session should exist");
    }
}

#[test]
fn test_session_table_concurrent_removals() {
    // Arrange: Create sessions
    let table = Arc::new(SessionTable::new());
    let mut session_ids = vec![];

    for _ in 0..100 {
        let session_id = SessionId::new();
        let case_id = CaseId::new();
        let tenant_id = TenantId::default_tenant();

        table.create_session(session_id, case_id, tenant_id);
        session_ids.push(session_id);
    }

    // Act: Concurrent removals
    let mut handles = vec![];
    for chunk in session_ids.chunks(10) {
        let table_clone = Arc::clone(&table);
        let ids = chunk.to_vec();

        let handle = thread::spawn(move || {
            for session_id in ids {
                table_clone.remove_session(session_id);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: All sessions should be removed
    assert_eq!(table.active_session_count(), 0);
}

// ============================================================================
// Isolation Tests
// ============================================================================

#[test]
fn test_session_isolation_across_tenants() {
    // Arrange: Create sessions for different tenants
    let table = Arc::new(SessionTable::new());

    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    let session1 = SessionId::new();
    let session2 = SessionId::new();

    table.create_session(session1, CaseId::new(), tenant1);
    table.create_session(session2, CaseId::new(), tenant2);

    // Act: Update metrics for each tenant
    let mut handles = vec![];

    let table1 = Arc::clone(&table);
    let h1 = thread::spawn(move || {
        if let Some(metrics) = table1.get_session_metrics(session1) {
            for _ in 0..1000 {
                metrics.increment_retries();
            }
        }
    });

    let table2 = Arc::clone(&table);
    let h2 = thread::spawn(move || {
        if let Some(metrics) = table2.get_session_metrics(session2) {
            for _ in 0..500 {
                metrics.increment_retries();
            }
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();

    // Assert: Metrics should be isolated
    if let Some(metrics1) = table.get_session_metrics(session1) {
        assert_eq!(metrics1.get_retry_count(), 1000);
    }

    if let Some(metrics2) = table.get_session_metrics(session2) {
        assert_eq!(metrics2.get_retry_count(), 500);
    }
}

#[test]
fn test_no_cross_session_contamination() {
    // Arrange: Create multiple sessions
    let table = Arc::new(SessionTable::new());
    let sessions: Vec<_> = (0..10)
        .map(|_| {
            let session_id = SessionId::new();
            let case_id = CaseId::new();
            let tenant_id = TenantId::default_tenant();
            table.create_session(session_id, case_id, tenant_id);
            session_id
        })
        .collect();

    // Act: Update each session with unique value
    let mut handles = vec![];
    for (i, &session_id) in sessions.iter().enumerate() {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            if let Some(metrics) = table_clone.get_session_metrics(session_id) {
                for _ in 0..(i + 1) * 100 {
                    metrics.increment_retries();
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert: Each session should have unique count
    for (i, &session_id) in sessions.iter().enumerate() {
        if let Some(metrics) = table.get_session_metrics(session_id) {
            assert_eq!(
                metrics.get_retry_count(),
                ((i + 1) * 100) as u64,
                "Session {} should have isolated count",
                i
            );
        }
    }
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_atomic_operations_performance() {
    use std::time::Instant;

    // Arrange
    let metrics = SessionMetrics::new();

    // Act: Measure 1 million atomic increments
    let start = Instant::now();
    for _ in 0..1_000_000 {
        metrics.increment_retries();
    }
    let elapsed = start.elapsed();

    // Assert: Should be very fast (< 100ms)
    assert!(
        elapsed.as_millis() < 100,
        "Atomic operations should be fast: took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_session_table_lookup_performance() {
    use std::time::Instant;

    // Arrange: Create many sessions
    let table = SessionTable::new();
    let mut session_ids = vec![];

    for _ in 0..1000 {
        let session_id = SessionId::new();
        table.create_session(session_id, CaseId::new(), TenantId::default_tenant());
        session_ids.push(session_id);
    }

    // Act: Measure lookup time
    let start = Instant::now();
    for _ in 0..10_000 {
        let session_id = session_ids[fastrand::usize(0..session_ids.len())];
        let _ = table.get_session_metrics(session_id);
    }
    let elapsed = start.elapsed();

    // Assert: Lookups should be fast (< 50ms for 10k lookups)
    assert!(
        elapsed.as_millis() < 50,
        "Session lookups should be O(1): took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_concurrent_session_creation_scalability() {
    use std::time::Instant;

    // Arrange
    let table = Arc::new(SessionTable::new());

    // Act: Create 10,000 sessions concurrently
    let start = Instant::now();
    let mut handles = vec![];

    for _ in 0..100 {
        let table_clone = Arc::clone(&table);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let session_id = SessionId::new();
                table_clone.create_session(
                    session_id,
                    CaseId::new(),
                    TenantId::default_tenant()
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start.elapsed();

    // Assert: Should scale well (< 500ms for 10k sessions)
    assert!(
        elapsed.as_millis() < 500,
        "Session creation should scale: took {}ms for 10k sessions",
        elapsed.as_millis()
    );
    assert_eq!(table.active_session_count(), 10_000);
}
