// examples/session_autonomic_example.rs
//! Complete example of Session-Scoped Autonomic Runtime
//!
//! Demonstrates:
//! - Creating and managing sessions
//! - Per-session adaptation with global Q compliance
//! - Aggregating session metrics to global MAPE-K
//! - Multi-tenant isolation
//! - Session lifecycle management
//!
//! Run with: cargo run --example session_autonomic_example

use knhk_workflow_engine::autonomic::{
    GlobalQ, KnowledgeBase, SessionAdapter, SessionAdapterConfig, SessionAggregator, SessionTable,
    TenantId,
};
use knhk_workflow_engine::case::CaseId;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Session-Scoped Autonomic Runtime Example\n");

    // Step 1: Setup infrastructure
    println!("📋 Setting up autonomic infrastructure...");
    let kb = Arc::new(KnowledgeBase::new());
    let session_table = Arc::new(SessionTable::new());
    let adapter = Arc::new(SessionAdapter::new(kb.clone()));
    let aggregator = Arc::new(SessionAggregator::new(kb.clone()));

    // Configure global Q (doctrine)
    let global_q = GlobalQ {
        max_total_resources: 0.9,
        max_concurrent_adaptations: 5,
        min_slo_compliance: 0.95,
        max_failure_rate: 0.05,
    };
    adapter.update_global_q(global_q).await;
    println!("✅ Global Q configured: max_concurrent_adaptations = 5\n");

    // Step 2: Simulate multi-tenant workload
    println!("👥 Creating multi-tenant workload...");
    let tenant_a = TenantId::new();
    let tenant_b = TenantId::new();

    // Tenant A: E-commerce workflows
    println!("  🏪 Tenant A (E-commerce): Creating order processing sessions");
    let mut tenant_a_sessions = Vec::new();
    for i in 0..20 {
        let case = CaseId::new();
        let handle = session_table
            .create_session(case, tenant_a)
            .with_pattern(1) // Order processing pattern
            .with_tag(format!("order-{}", i));
        handle.start();
        tenant_a_sessions.push(handle);
    }

    // Tenant B: Data processing workflows
    println!("  📊 Tenant B (Analytics): Creating data pipeline sessions");
    let mut tenant_b_sessions = Vec::new();
    for i in 0..15 {
        let case = CaseId::new();
        let handle = session_table
            .create_session(case, tenant_b)
            .with_pattern(5) // Data pipeline pattern
            .with_tag(format!("pipeline-{}", i));
        handle.start();
        tenant_b_sessions.push(handle);
    }

    println!("✅ Created {} sessions across 2 tenants\n", session_table.total_sessions());

    // Step 3: Simulate workflow execution with varying performance
    println!("⚙️  Simulating workflow execution...");

    // Fast sessions (good performance)
    for handle in &tenant_a_sessions[0..10] {
        for _ in 0..5 {
            handle.record_task_execution(Duration::from_micros(500)); // Fast
        }
    }
    println!("  ✅ Tenant A: 10 sessions with good performance");

    // Slow sessions (degraded performance)
    for handle in &tenant_a_sessions[10..15] {
        for _ in 0..5 {
            handle.record_task_execution(Duration::from_micros(5000)); // Slow
            handle.record_violation();
        }
    }
    println!("  ⚠️  Tenant A: 5 sessions with degraded performance");

    // Failed sessions (need compensation)
    for handle in &tenant_a_sessions[15..] {
        handle.record_task_execution(Duration::from_micros(1000));
        for _ in 0..5 {
            handle.record_retry();
        }
        handle.fail();
    }
    println!("  ❌ Tenant A: 5 sessions failed");

    // Tenant B: Normal operation
    for handle in &tenant_b_sessions {
        for _ in 0..10 {
            handle.record_task_execution(Duration::from_micros(2000));
        }
        handle.complete();
    }
    println!("  ✅ Tenant B: 15 sessions completed successfully\n");

    // Step 4: Session-level adaptation
    println!("🔧 Analyzing sessions for local adaptation...");
    let mut adapted_count = 0;
    let mut rejected_count = 0;

    for handle in &tenant_a_sessions {
        if !handle.is_terminal() {
            match adapter.analyze_session(handle).await? {
                Some(decision) => {
                    println!(
                        "  📍 Session {}: {} - {}",
                        &handle.id.to_string()[..8],
                        match &decision.action {
                            knhk_workflow_engine::autonomic::SessionAction::RetryTask { .. } =>
                                "Retrying task",
                            knhk_workflow_engine::autonomic::SessionAction::DegradePerformance { .. } =>
                                "Degrading performance",
                            knhk_workflow_engine::autonomic::SessionAction::RequestResources { .. } =>
                                "Requesting resources",
                            knhk_workflow_engine::autonomic::SessionAction::CancelOptionalTasks =>
                                "Cancelling optional tasks",
                            knhk_workflow_engine::autonomic::SessionAction::TriggerCompensation { .. } =>
                                "Triggering compensation",
                            knhk_workflow_engine::autonomic::SessionAction::LogAndContinue { .. } =>
                                "Logging warning",
                        },
                        decision.reason
                    );
                    adapter.execute_decision(&decision, handle).await?;
                    adapted_count += 1;
                }
                None => {
                    rejected_count += 1;
                }
            }
        }
    }

    println!("\n  ✅ Adapted: {}", adapted_count);
    println!("  ⏭️  Rejected by global Q: {}", rejected_count);

    // Sleep briefly to let operations complete
    sleep(Duration::from_millis(100)).await;

    // Step 5: Aggregate session metrics to global MAPE-K
    println!("\n📊 Aggregating session metrics to global knowledge base...");

    let all_sessions: Vec<_> = session_table.active_sessions();
    let metrics = aggregator.aggregate_sessions(&all_sessions).await?;

    println!("\n📈 Aggregated Metrics:");
    println!("  Total sessions: {}", metrics.total_sessions);
    println!("  Active sessions: {}", metrics.active_sessions);
    println!("  Completed sessions: {}", metrics.completed_sessions);
    println!("  Failed sessions: {}", metrics.failed_sessions);
    println!("  Average latency: {}μs", metrics.avg_latency_us);
    println!("  Violation rate: {:.2}%", metrics.violation_rate * 100.0);
    println!("  Retry rate: {:.2}%", metrics.retry_rate * 100.0);
    println!("  Failure rate: {:.2}%", metrics.failure_rate * 100.0);

    // Step 6: Tenant isolation verification
    println!("\n🔒 Verifying tenant isolation...");
    let tenant_a_count = session_table.tenant_session_count(tenant_a);
    let tenant_b_count = session_table.tenant_session_count(tenant_b);

    println!("  Tenant A sessions: {}", tenant_a_count);
    println!("  Tenant B sessions: {}", tenant_b_count);

    let tenant_a_filtered = session_table.sessions_by_tenant(tenant_a);
    assert_eq!(tenant_a_filtered.len(), tenant_a_count as usize);
    println!("  ✅ Tenant isolation verified");

    // Step 7: Session events
    println!("\n📤 Draining session events...");
    let events = adapter.drain_events().await;
    println!("  Emitted {} events for global aggregation", events.len());

    for (i, event) in events.iter().take(5).enumerate() {
        match event {
            knhk_workflow_engine::autonomic::SessionEvent::Started { session_id, pattern_id } => {
                println!(
                    "    {}: Session started (pattern: {:?})",
                    &session_id.to_string()[..8],
                    pattern_id
                );
            }
            knhk_workflow_engine::autonomic::SessionEvent::Adapted {
                session_id,
                action,
                reason,
            } => {
                println!(
                    "    {}: Adapted - {}",
                    &session_id.to_string()[..8],
                    reason
                );
            }
            knhk_workflow_engine::autonomic::SessionEvent::Completed {
                session_id,
                duration_ms,
                ..
            } => {
                println!(
                    "    {}: Completed in {}ms",
                    &session_id.to_string()[..8],
                    duration_ms
                );
            }
            _ => {}
        }

        if i == 4 && events.len() > 5 {
            println!("    ... and {} more events", events.len() - 5);
        }
    }

    // Step 8: Session cleanup
    println!("\n🧹 Cleaning up terminal sessions...");
    let stats_before = session_table.stats();
    println!("  Before cleanup: {} total sessions", stats_before.total_sessions);

    sleep(Duration::from_millis(50)).await;
    let removed = session_table.cleanup_terminal_sessions(Duration::from_millis(10));

    let stats_after = session_table.stats();
    println!("  Removed {} terminal sessions", removed);
    println!("  After cleanup: {} total sessions", stats_after.total_sessions);

    // Step 9: Adapter statistics
    println!("\n📊 Session Adapter Statistics:");
    let adapter_stats = adapter.stats().await;
    println!("  Active adaptations: {}", adapter_stats.active_adaptations);
    println!("  Total decisions: {}", adapter_stats.total_decisions);
    println!("  Tracked sessions: {}", adapter_stats.tracked_sessions);
    println!("  Pending events: {}", adapter_stats.pending_events);

    // Step 10: Pattern-based filtering
    println!("\n🔍 Pattern-based session filtering:");
    let pattern1_sessions = session_table.sessions_by_pattern(1);
    let pattern5_sessions = session_table.sessions_by_pattern(5);

    println!("  Pattern 1 (Order processing): {} sessions", pattern1_sessions.len());
    println!("  Pattern 5 (Data pipeline): {} sessions", pattern5_sessions.len());

    println!("\n✅ Example complete!\n");
    println!("💡 Key Takeaways:");
    println!("  • Sessions provide per-workflow isolation");
    println!("  • Lock-free atomic metrics enable efficient hot-path updates");
    println!("  • Local adaptations respect global Q (doctrine)");
    println!("  • Session metrics aggregate to global MAPE-K knowledge base");
    println!("  • Multi-tenant isolation is enforced");
    println!("  • Millions of concurrent sessions can be supported");

    Ok(())
}
