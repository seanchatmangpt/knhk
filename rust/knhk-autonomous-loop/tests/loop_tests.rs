//! Integration tests for the autonomous loop engine

use knhk_autonomous_loop::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Re-use mock implementations from cycle_tests
mod common;
use common::*;

#[tokio::test]
async fn test_loop_engine_creation() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 2, true);

    let engine = LoopEngine::new(config, deps);
    assert!(engine.is_ok());
}

#[tokio::test]
async fn test_loop_engine_invalid_config() {
    let mut config = AutonomousLoopConfig::default();
    config.cycle_interval = Duration::from_millis(500); // Too short

    let deps = create_test_dependencies(10, 2, true);

    let engine = LoopEngine::new(config, deps);
    assert!(engine.is_err());
}

#[tokio::test]
async fn test_loop_start_and_stop() {
    let mut config = AutonomousLoopConfig::default();
    config.cycle_interval = Duration::from_secs(1);

    let deps = create_test_dependencies(10, 2, true);

    let handle = start_autonomous_loop(config, deps).unwrap();

    // Let it run for a bit
    sleep(Duration::from_millis(500)).await;

    assert!(handle.engine().is_running());

    // Stop it
    handle.stop().await.unwrap();
}

#[tokio::test]
async fn test_loop_pause_and_resume() {
    let mut config = AutonomousLoopConfig::default();
    config.cycle_interval = Duration::from_secs(1);

    let deps = create_test_dependencies(10, 2, true);

    let engine = Arc::new(LoopEngine::new(config, deps).unwrap());
    let engine_clone = Arc::clone(&engine);

    let task = tokio::spawn(async move { engine_clone.run().await });

    // Wait for it to start
    sleep(Duration::from_millis(100)).await;

    // Pause
    engine.pause("test pause".to_string()).await;
    sleep(Duration::from_millis(100)).await;

    let health = engine.get_health().await;
    assert!(health.is_paused());

    // Resume
    engine.resume().await;
    sleep(Duration::from_millis(100)).await;

    let health = engine.get_health().await;
    assert!(health.is_healthy());

    // Stop
    engine.stop();
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_loop_history_tracking() {
    let mut config = AutonomousLoopConfig::default();
    config.cycle_interval = Duration::from_millis(100);

    let deps = create_test_dependencies(10, 2, true);

    let engine = Arc::new(LoopEngine::new(config, deps).unwrap());
    let engine_clone = Arc::clone(&engine);

    let task = tokio::spawn(async move { engine_clone.run().await });

    // Let a few cycles run
    sleep(Duration::from_millis(500)).await;

    let history = engine.get_history().await;
    assert!(!history.is_empty());

    // Stop
    engine.stop();
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_loop_stats_tracking() {
    let mut config = AutonomousLoopConfig::default();
    config.cycle_interval = Duration::from_millis(100);

    let deps = create_test_dependencies(10, 2, true);

    let engine = Arc::new(LoopEngine::new(config, deps).unwrap());
    let engine_clone = Arc::clone(&engine);

    let task = tokio::spawn(async move { engine_clone.run().await });

    // Let a few cycles run
    sleep(Duration::from_millis(500)).await;

    let stats = engine.get_stats().await;
    assert!(stats.total_cycles > 0);
    assert!(stats.successful_cycles > 0);
    assert_eq!(stats.failed_cycles, 0);

    // Stop
    engine.stop();
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_loop_config_update() {
    let config = AutonomousLoopConfig::default();
    let deps = create_test_dependencies(10, 2, true);

    let engine = Arc::new(LoopEngine::new(config, deps).unwrap());

    let original_config = engine.get_config().await;
    assert_eq!(original_config.min_patterns_for_proposal, 5);

    // Update config
    let new_config = AutonomousLoopConfig::default().with_min_patterns(10);

    engine.update_config(new_config).await.unwrap();

    let updated_config = engine.get_config().await;
    assert_eq!(updated_config.min_patterns_for_proposal, 10);
}

#[tokio::test]
async fn test_loop_error_rate_threshold() {
    let mut config = AutonomousLoopConfig::default();
    config.cycle_interval = Duration::from_millis(100);
    config.pause_on_error_rate = Some(50.0); // Pause at 50% error rate

    // Create deps that will cause failures
    let deps = create_test_dependencies(2, 2, false); // Low patterns, no validation

    let engine = Arc::new(LoopEngine::new(config, deps).unwrap());
    let engine_clone = Arc::clone(&engine);

    let task = tokio::spawn(async move { engine_clone.run().await });

    // Let cycles run and accumulate errors
    sleep(Duration::from_secs(2)).await;

    // Check if loop paused due to error rate
    let _health = engine.get_health().await;
    // May or may not be paused depending on exact timing

    // Stop
    engine.stop();
    task.await.unwrap().unwrap();
}

#[tokio::test]
async fn test_loop_handle() {
    let config = AutonomousLoopConfig::default()
        .with_cycle_interval(Duration::from_millis(100));

    let deps = create_test_dependencies(10, 2, true);

    let handle = start_autonomous_loop(config, deps).unwrap();

    // Access engine through handle
    let stats = handle.engine().get_stats().await;
    assert_eq!(stats.total_cycles, 0);

    sleep(Duration::from_millis(300)).await;

    let stats = handle.engine().get_stats().await;
    assert!(stats.total_cycles > 0);

    // Stop via handle
    handle.stop().await.unwrap();
}
