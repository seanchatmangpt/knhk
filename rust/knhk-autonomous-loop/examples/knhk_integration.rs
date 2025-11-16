//! KNHK Integration Example
//!
//! This example shows how to integrate the Autonomous Evolution Loop
//! into a KNHK application for continuous ontology evolution.

use knhk_autonomous_loop::*;
use std::time::Duration;

/// Example KNHK application with autonomous evolution
///
/// In a real KNHK application, this would be integrated into the main
/// initialization code, typically in `knhk/src/main.rs` or a dedicated
/// evolution module.
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("knhk_autonomous_loop=info,knhk=info")
        .init();

    println!("🌟 KNHK with Autonomous Ontology Evolution\n");

    // ========================================================================
    // Step 1: Initialize KNHK Components
    // ========================================================================
    println!("📦 Initializing KNHK components...");

    // In production, these would be real KNHK components:
    // - Snapshot store from knhk-snapshot
    // - Pattern miner from knhk-pattern-mining
    // - Validator from knhk-delta-sigma
    // - etc.

    // For this example, we use a simplified setup:
    // let snapshot_store = initialize_knhk_snapshot_store().await?;
    // let pattern_miner = initialize_knhk_pattern_miner()?;
    // let validator = initialize_knhk_validator()?;
    // etc.

    println!("   ✅ Snapshot store initialized");
    println!("   ✅ Pattern miner initialized");
    println!("   ✅ Delta-Sigma validator initialized");
    println!("   ✅ Promotion pipeline initialized");
    println!();

    // ========================================================================
    // Step 2: Configure Autonomous Evolution
    // ========================================================================
    println!("⚙️  Configuring autonomous evolution...");

    let config = AutonomousLoopConfig::new()
        // Run evolution every 60 seconds
        .with_cycle_interval(Duration::from_secs(60))
        // Require at least 10 patterns before proposing changes
        .with_min_patterns(10)
        // Automatically promote production-ready snapshots
        .with_auto_promote(true)
        // Alert on major changes
        .with_alert_email("ontology-team@example.com".to_string())
        // Pause if error rate exceeds 5%
        .with_error_threshold(5.0);

    config.validate()?;

    println!("   ✅ Configuration validated");
    println!();

    // ========================================================================
    // Step 3: Create Dependencies
    // ========================================================================
    println!("🔗 Creating dependency injection container...");

    // In production:
    // let dependencies = LoopDependencies::new(
    //     Arc::new(snapshot_store),
    //     Arc::new(pattern_miner),
    //     Arc::new(proposer),
    //     Arc::new(validator),
    //     Arc::new(promotion_pipeline),
    //     Arc::new(change_executor),
    //     Arc::new(receipt_log),
    // );

    println!("   ✅ Dependencies assembled");
    println!();

    // ========================================================================
    // Step 4: Start Autonomous Loop
    // ========================================================================
    println!("🚀 Starting autonomous evolution loop...");

    // Uncomment in production:
    // let handle = start_autonomous_loop(config, dependencies)?;

    println!("   ✅ Loop started successfully");
    println!();

    // ========================================================================
    // Step 5: Run Main KNHK Application
    // ========================================================================
    println!("🏃 Running KNHK application...");
    println!();
    println!("The autonomous loop now runs in the background, continuously:");
    println!("  1. Observing telemetry data");
    println!("  2. Detecting patterns");
    println!("  3. Proposing ontology changes");
    println!("  4. Validating against invariants");
    println!("  5. Compiling new snapshots");
    println!("  6. Promoting to production when ready");
    println!();

    // Main application logic here
    // knhk_main_loop().await?;

    // ========================================================================
    // Step 6: Graceful Shutdown
    // ========================================================================
    println!("🛑 Shutting down...");

    // Stop evolution loop
    // handle.stop().await?;

    println!("   ✅ Evolution loop stopped");
    println!("   ✅ KNHK shutdown complete");

    Ok(())
}

// ============================================================================
// Example Integration Functions
// ============================================================================

/// Example: Initialize KNHK with autonomous evolution
///
/// This would be called from `main()` in a real KNHK application.
#[allow(dead_code)]
async fn initialize_knhk_with_evolution() -> Result<LoopHandle> {
    // Load configuration
    let config = AutonomousLoopConfig::default();

    // Create dependencies from KNHK components
    // let deps = create_knhk_dependencies().await?;

    // Start loop
    // start_autonomous_loop(config, deps)

    unimplemented!("Wire up real KNHK components")
}

/// Example: Monitor evolution loop health
///
/// This could be exposed via a /health endpoint or monitoring dashboard.
#[allow(dead_code)]
async fn monitor_evolution_health(handle: &LoopHandle) -> String {
    let health = handle.engine().get_health().await;
    let stats = handle.engine().get_stats().await;

    format!(
        "Evolution Loop Health:\n\
         Status: {}\n\
         Total Cycles: {}\n\
         Success Rate: {:.1}%\n\
         Error Rate: {:.1}%\n\
         Avg Duration: {}ms",
        health.status(),
        stats.total_cycles,
        stats.success_rate(),
        stats.error_rate,
        stats.avg_cycle_duration_ms
    )
}

/// Example: Pause evolution for maintenance
#[allow(dead_code)]
async fn pause_for_maintenance(handle: &LoopHandle) {
    handle
        .engine()
        .pause("Scheduled maintenance window".to_string())
        .await;

    println!("Evolution paused for maintenance");

    // Perform maintenance...

    handle.engine().resume().await;

    println!("Evolution resumed");
}
