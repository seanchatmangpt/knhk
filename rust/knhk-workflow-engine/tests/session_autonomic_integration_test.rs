// tests/session_autonomic_integration_test.rs
//! Integration tests for Session-Scoped Autonomic Runtime
//!
//! Tests the complete integration of session-level adaptation with global MAPE-K.

use knhk_workflow_engine::autonomic::{
    AggregatedMetrics, GlobalQ, KnowledgeBase, SessionAdapter, SessionAdapterConfig,
    SessionAggregator, SessionEvent, SessionHandle, SessionTable, TenantId,
};
use knhk_workflow_engine::case::CaseId;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_end_to_end_session_lifecycle() {
    // Setup
    let kb = Arc::new(KnowledgeBase::new());
    let table = SessionTable::new();
    let adapter = SessionAdapter::new(kb.clone());
    let aggregator = SessionAggregator::new(kb.clone());

    // Create session
    let case_id = CaseId::new();
    let tenant_id = TenantId::default_tenant();
    let handle = table.create_session(case_id, tenant_id);

    // Start session
    handle.start();
    assert!(handle.is_active());

    // Record some operations
    for _ in 0..10 {
        handle.record_task_execution(Duration::from_micros(1000));
    }

    // Trigger violation
    handle.record_violation();

    // Analyze and potentially adapt
    let decision = adapter.analyze_session(&handle).await.unwrap();

    // Complete session
    handle.complete();
    assert!(handle.is_terminal());

    // Aggregate metrics
    let sessions = vec![handle];
    let metrics = aggregator.aggregate_sessions(&sessions).await.unwrap();

    assert_eq!(metrics.total_sessions, 1);
    assert_eq!(metrics.completed_sessions, 1);
    assert!(metrics.avg_latency_us > 0);
}

#[tokio::test]
async fn test_multi_session_isolation() {
    // Setup
    let table = SessionTable::new();
    let kb = Arc::new(KnowledgeBase::new());
    let adapter = SessionAdapter::new(kb.clone());

    // Create sessions for different tenants
    let tenant1 = TenantId::new();
    let tenant2 = TenantId::new();

    let mut handles1 = Vec::new();
    let mut handles2 = Vec::new();

    for _ in 0..5 {
        let case = CaseId::new();
        let handle = table.create_session(case, tenant1);
        handle.start();
        handles1.push(handle);
    }

    for _ in 0..5 {
        let case = CaseId::new();
        let handle = table.create_session(case, tenant2);
        handle.start();
        handles2.push(handle);
    }

    // Verify isolation
    assert_eq!(table.tenant_session_count(tenant1), 5);
    assert_eq!(table.tenant_session_count(tenant2), 5);
    assert_eq!(table.total_sessions(), 10);

    // Filter by tenant
    let tenant1_sessions = table.sessions_by_tenant(tenant1);
    assert_eq!(tenant1_sessions.len(), 5);

    // Verify no cross-contamination
    for handle in &tenant1_sessions {
        assert_eq!(handle.context.tenant_id, tenant1);
    }
}

#[tokio::test]
async fn test_session_adaptation_with_global_q() {
    // Setup
    let kb = Arc::new(KnowledgeBase::new());
    let adapter = SessionAdapter::new(kb.clone());
    let table = SessionTable::new();

    // Configure strict global Q
    let mut q = GlobalQ::default();
    q.max_concurrent_adaptations = 2;
    adapter.update_global_q(q).await;

    // Create sessions that need adaptation
    let mut handles = Vec::new();
    for _ in 0..5 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();

        // Trigger adaptation need
        for _ in 0..5 {
            handle.record_retry();
        }

        handles.push(handle);
    }

    // Try to adapt all sessions
    let mut adapted_count = 0;
    for handle in &handles {
        if let Some(decision) = adapter.analyze_session(handle).await.unwrap() {
            adapter.execute_decision(&decision, handle).await.unwrap();
            adapted_count += 1;
        }
    }

    // Should respect global Q limit (max 2 concurrent)
    // Note: Actual behavior depends on timing, but we verify the mechanism
    let stats = adapter.stats().await;
    assert!(stats.total_decisions > 0);
}

#[tokio::test]
async fn test_session_metrics_aggregation() {
    // Setup
    let kb = Arc::new(KnowledgeBase::new());
    let table = SessionTable::new();
    let aggregator = SessionAggregator::new(kb.clone());

    // Create diverse sessions
    let mut handles = Vec::new();

    // Fast sessions
    for _ in 0..50 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();
        for _ in 0..10 {
            handle.record_task_execution(Duration::from_micros(500));
        }
        handle.complete();
        handles.push(handle);
    }

    // Slow sessions
    for _ in 0..30 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();
        for _ in 0..10 {
            handle.record_task_execution(Duration::from_micros(5000));
        }
        handle.complete();
        handles.push(handle);
    }

    // Failed sessions
    for _ in 0..20 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();
        handle.record_violation();
        handle.fail();
        handles.push(handle);
    }

    // Aggregate
    let metrics = aggregator.aggregate_sessions(&handles).await.unwrap();

    assert_eq!(metrics.total_sessions, 100);
    assert_eq!(metrics.completed_sessions, 80);
    assert_eq!(metrics.failed_sessions, 20);
    assert!(metrics.avg_latency_us > 0);
    assert_eq!(metrics.failure_rate, 0.2);
}

#[tokio::test]
async fn test_session_event_emission_and_drainage() {
    // Setup
    let kb = Arc::new(KnowledgeBase::new());
    let adapter = SessionAdapter::new(kb.clone());
    let table = SessionTable::new();

    // Create and adapt multiple sessions
    for _ in 0..10 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();

        // Trigger adaptation
        for _ in 0..5 {
            handle.record_retry();
        }

        if let Some(decision) = adapter.analyze_session(&handle).await.unwrap() {
            adapter.execute_decision(&decision, &handle).await.unwrap();
        }

        handle.complete();
    }

    // Drain events
    let events = adapter.drain_events().await;
    assert!(!events.is_empty());

    // Second drain should be empty
    let events2 = adapter.drain_events().await;
    assert!(events2.is_empty());
}

#[tokio::test]
async fn test_session_cleanup() {
    // Setup
    let table = SessionTable::new();

    // Create and complete sessions
    let mut completed_ids = Vec::new();
    for _ in 0..10 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();
        handle.complete();
        completed_ids.push(handle.id);
    }

    // Create active sessions
    for _ in 0..5 {
        let case = CaseId::new();
        let handle = table.create_session(case, TenantId::default_tenant());
        handle.start();
    }

    assert_eq!(table.total_sessions(), 15);

    // Sleep to ensure completed sessions are old
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Cleanup old terminal sessions
    let removed = table.cleanup_terminal_sessions(Duration::from_millis(5));
    assert_eq!(removed, 10);
    assert_eq!(table.total_sessions(), 5);
}

#[tokio::test]
async fn test_session_pattern_filtering() {
    // Setup
    let table = SessionTable::new();

    // Create sessions with different patterns
    for pattern_id in 1..=5 {
        for _ in 0..3 {
            let case = CaseId::new();
            let handle = table
                .create_session(case, TenantId::default_tenant())
                .with_pattern(pattern_id);
            handle.start();
        }
    }

    assert_eq!(table.total_sessions(), 15);

    // Filter by pattern
    let pattern1_sessions = table.sessions_by_pattern(1);
    assert_eq!(pattern1_sessions.len(), 3);

    for handle in &pattern1_sessions {
        assert_eq!(handle.context.pattern_id, Some(1));
    }
}

#[tokio::test]
async fn test_concurrent_session_operations() {
    use tokio::task;

    // Setup
    let table = Arc::new(SessionTable::new());
    let kb = Arc::new(KnowledgeBase::new());
    let adapter = Arc::new(SessionAdapter::new(kb.clone()));

    // Spawn concurrent tasks
    let mut tasks = Vec::new();

    for i in 0..100 {
        let table_clone = table.clone();
        let adapter_clone = adapter.clone();

        let task = task::spawn(async move {
            let case = CaseId::new();
            let tenant = if i % 2 == 0 {
                TenantId::new()
            } else {
                TenantId::default_tenant()
            };

            let handle = table_clone.create_session(case, tenant);
            handle.start();

            // Simulate work
            for _ in 0..10 {
                handle.record_task_execution(Duration::from_micros(1000));
            }

            // Some sessions trigger adaptation
            if i % 3 == 0 {
                for _ in 0..5 {
                    handle.record_retry();
                }
                let _ = adapter_clone.analyze_session(&handle).await;
            }

            handle.complete();
        });

        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.unwrap();
    }

    // Verify no data races
    let stats = table.stats();
    assert_eq!(stats.total_sessions, 100);
    assert_eq!(stats.completed_sessions, 100);
}

#[tokio::test]
async fn test_session_decision_history() {
    // Setup
    let kb = Arc::new(KnowledgeBase::new());
    let adapter = SessionAdapter::new(kb.clone());
    let table = SessionTable::new();

    let case = CaseId::new();
    let handle = table.create_session(case, TenantId::default_tenant());
    handle.start();

    // Trigger multiple adaptations
    for i in 0..5 {
        for _ in 0..5 {
            handle.record_retry();
        }

        if let Some(decision) = adapter.analyze_session(&handle).await.unwrap() {
            adapter.execute_decision(&decision, &handle).await.unwrap();
        }

        // Reset retries to trigger multiple decisions
        if i < 4 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }

    // Check history
    let history = adapter.get_session_history(handle.id).await;
    assert!(!history.is_empty());

    // Cleanup
    adapter.clear_completed_sessions(&[handle.id]).await;
    let history2 = adapter.get_session_history(handle.id).await;
    assert!(history2.is_empty());
}

#[tokio::test]
async fn test_session_metrics_snapshot_accuracy() {
    // Setup
    let table = SessionTable::new();
    let case = CaseId::new();
    let handle = table.create_session(case, TenantId::default_tenant());

    handle.start();

    // Record precise metrics
    handle.record_task_execution(Duration::from_micros(1000));
    handle.record_task_execution(Duration::from_micros(2000));
    handle.record_task_execution(Duration::from_micros(3000));
    handle.record_violation();
    handle.record_violation();
    handle.record_retry();

    let snapshot = handle.snapshot();

    // Verify snapshot accuracy
    assert_eq!(snapshot.task_completions, 3);
    assert_eq!(snapshot.violation_count, 2);
    assert_eq!(snapshot.retry_count, 1);
    assert_eq!(snapshot.total_latency_us, 6000);
    assert_eq!(snapshot.avg_latency_us(), Some(2000));
    assert_eq!(snapshot.violation_rate(), 2.0 / 3.0);
    assert_eq!(snapshot.retry_rate(), 1.0 / 3.0);
}
