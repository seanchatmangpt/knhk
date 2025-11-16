//! Tests for the self-healing system

use knhk_autonomous_loop::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_healer_creation() {
    let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();
    assert!(healer.get_history().await.is_empty());
}

#[tokio::test]
async fn test_should_heal_low_failures() {
    let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

    let state = Arc::new(RwLock::new(LoopState {
        cycle_count: 10,
        failure_count: 1,
        ..Default::default()
    }));

    let should_heal = healer.should_heal(&state).await.unwrap();
    assert!(!should_heal);
}

#[tokio::test]
async fn test_should_heal_high_failures() {
    let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

    let state = Arc::new(RwLock::new(LoopState {
        cycle_count: 10,
        failure_count: 5,
        ..Default::default()
    }));

    let should_heal = healer.should_heal(&state).await.unwrap();
    assert!(should_heal);
}

#[tokio::test]
async fn test_should_heal_failure_rate() {
    let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

    // 40% failure rate (4 out of 10)
    let state = Arc::new(RwLock::new(LoopState {
        cycle_count: 10,
        failure_count: 4,
        ..Default::default()
    }));

    let should_heal = healer.should_heal(&state).await.unwrap();
    assert!(should_heal);
}

#[tokio::test]
async fn test_should_heal_insufficient_data() {
    let healer = SelfHealer::new(RecoveryStrategy::Rollback).await.unwrap();

    // Not enough cycles for rate calculation
    let state = Arc::new(RwLock::new(LoopState {
        cycle_count: 5,
        failure_count: 2,
        ..Default::default()
    }));

    let should_heal = healer.should_heal(&state).await.unwrap();
    // Should still heal because failure_count >= 3 threshold might not be met
    // But failure rate check won't trigger
    assert!(!should_heal);
}

#[tokio::test]
async fn test_recovery_strategy_continue() {
    let healer = SelfHealer::new(RecoveryStrategy::Continue).await.unwrap();

    // Continue strategy should always succeed
    let result = healer.heal().await;
    assert!(result.is_ok());

    let history = healer.get_history().await;
    assert_eq!(history.len(), 1);
    assert!(history[0].success);
}

#[tokio::test]
async fn test_recovery_strategy_strict_validation() {
    let healer = SelfHealer::new(RecoveryStrategy::StrictValidation)
        .await
        .unwrap();

    let result = healer.heal().await;
    assert!(result.is_ok());

    let history = healer.get_history().await;
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_recovery_strategy_pause() {
    let healer = SelfHealer::new(RecoveryStrategy::Pause).await.unwrap();

    let result = healer.heal().await;
    assert!(result.is_ok());

    let history = healer.get_history().await;
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn test_multiple_healing_actions() {
    let healer = SelfHealer::new(RecoveryStrategy::Continue).await.unwrap();

    for _ in 0..5 {
        healer.heal().await.unwrap();
    }

    let history = healer.get_history().await;
    assert_eq!(history.len(), 5);
}

#[tokio::test]
async fn test_healing_history_tracking() {
    let healer = SelfHealer::new(RecoveryStrategy::Continue).await.unwrap();

    healer.heal().await.unwrap();
    healer.heal().await.unwrap();

    let history = healer.get_history().await;
    assert_eq!(history.len(), 2);

    // All should be successful (Continue strategy)
    assert!(history.iter().all(|action| action.success));
}

#[tokio::test]
async fn test_concurrent_healing_checks() {
    let healer = SelfHealer::new(RecoveryStrategy::Continue).await.unwrap();

    let state = Arc::new(RwLock::new(LoopState {
        cycle_count: 10,
        failure_count: 5,
        ..Default::default()
    }));

    // Multiple concurrent checks
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let h = healer.clone();
            let s = state.clone();
            tokio::spawn(async move { h.should_heal(&s).await.unwrap() })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result);
    }
}
