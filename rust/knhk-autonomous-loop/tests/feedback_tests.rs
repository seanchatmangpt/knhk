//! Tests for the feedback system

use knhk_autonomous_loop::*;

#[tokio::test]
async fn test_feedback_system_creation() {
    let feedback = FeedbackSystem::new().await.unwrap();
    let metrics = feedback.get_metrics().await;

    assert_eq!(metrics.schema_mismatches, 0);
    assert_eq!(metrics.guard_violations, 0);
    assert!(!metrics.performance_regression_detected);
}

#[tokio::test]
async fn test_custom_thresholds() {
    let thresholds = FeedbackThresholds {
        schema_mismatch_count: 5,
        guard_violation_count: 3,
        performance_regression: false,
        new_patterns_detected: false,
        error_rate_threshold: 0.1,
    };

    let feedback = FeedbackSystem::with_thresholds(thresholds)
        .await
        .unwrap();

    // Should not trigger with low counts
    for _ in 0..2 {
        feedback.record_schema_mismatch().await.unwrap();
    }

    let reason = feedback.should_trigger_evolution().await.unwrap();
    assert!(matches!(reason, TriggerReason::None));
}

#[tokio::test]
async fn test_schema_drift_detection() {
    let feedback = FeedbackSystem::new().await.unwrap();

    // Record enough mismatches to trigger
    for _ in 0..15 {
        feedback.record_schema_mismatch().await.unwrap();
    }

    let reason = feedback.should_trigger_evolution().await.unwrap();
    assert!(matches!(reason, TriggerReason::SchemaDrift));
}

#[tokio::test]
async fn test_guard_violation_detection() {
    let feedback = FeedbackSystem::new().await.unwrap();

    for _ in 0..10 {
        feedback.record_guard_violation().await.unwrap();
    }

    let reason = feedback.should_trigger_evolution().await.unwrap();
    assert!(matches!(reason, TriggerReason::GuardViolations));
}

#[tokio::test]
async fn test_performance_regression_detection() {
    let feedback = FeedbackSystem::new().await.unwrap();

    feedback.record_performance_regression().await.unwrap();

    let reason = feedback.should_trigger_evolution().await.unwrap();
    assert!(matches!(reason, TriggerReason::PerformanceRegression));
}

#[tokio::test]
async fn test_high_error_rate_detection() {
    let feedback = FeedbackSystem::new().await.unwrap();

    feedback.update_error_rate(0.1).await.unwrap();

    let reason = feedback.should_trigger_evolution().await.unwrap();
    assert!(matches!(reason, TriggerReason::HighErrorRate));
}

#[tokio::test]
async fn test_multiple_triggers() {
    let feedback = FeedbackSystem::new().await.unwrap();

    // Trigger multiple conditions
    for _ in 0..15 {
        feedback.record_schema_mismatch().await.unwrap();
    }
    for _ in 0..10 {
        feedback.record_guard_violation().await.unwrap();
    }
    feedback.record_performance_regression().await.unwrap();

    let reason = feedback.should_trigger_evolution().await.unwrap();

    match reason {
        TriggerReason::Multiple(reasons) => {
            assert!(reasons.len() >= 2);
        }
        _ => panic!("Expected multiple triggers"),
    }
}

#[tokio::test]
async fn test_metrics_reset() {
    let feedback = FeedbackSystem::new().await.unwrap();

    for _ in 0..5 {
        feedback.record_schema_mismatch().await.unwrap();
    }
    feedback.record_performance_regression().await.unwrap();

    feedback.reset_metrics().await.unwrap();

    let metrics = feedback.get_metrics().await;
    assert_eq!(metrics.schema_mismatches, 0);
    assert!(!metrics.performance_regression_detected);
}

#[tokio::test]
async fn test_new_patterns_trigger() {
    let feedback = FeedbackSystem::new().await.unwrap();

    let patterns = DetectedPatterns {
        patterns: vec![DetectedPattern {
            pattern_id: "test-pattern".to_string(),
            confidence: 0.9,
            frequency: 10,
            first_seen: std::time::SystemTime::now(),
            last_seen: std::time::SystemTime::now(),
            metadata: serde_json::json!({}),
        }],
    };

    feedback.record_new_patterns(patterns).await.unwrap();

    let reason = feedback.should_trigger_evolution().await.unwrap();
    assert!(matches!(reason, TriggerReason::NewPatterns));
}

#[tokio::test]
async fn test_concurrent_metric_updates() {
    let feedback = FeedbackSystem::new().await.unwrap();

    // Spawn multiple tasks updating metrics concurrently
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let f = feedback.clone();
            tokio::spawn(async move {
                f.record_schema_mismatch().await.unwrap();
                f.record_guard_violation().await.unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap();
    }

    let metrics = feedback.get_metrics().await;
    assert_eq!(metrics.schema_mismatches, 10);
    assert_eq!(metrics.guard_violations, 10);
}
