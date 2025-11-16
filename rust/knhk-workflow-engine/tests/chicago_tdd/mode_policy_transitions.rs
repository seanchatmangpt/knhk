//! Mode-Aware Policy Transition Tests
//!
//! Tests mode-based policy enforcement and state transitions:
//! - Mode filtering gates actions correctly
//! - Mode transitions are safe and observable
//! - Policy violations are logged
//! - Action annotations enforce mode requirements
//!
//! **Chicago TDD Approach**: Tests real policy engine, not mocks

use knhk_workflow_engine::autonomic::failure_modes::AutonomicMode;
use knhk_workflow_engine::autonomic::mode_policy::{
    ActionAnnotation, ActionPattern, MinimumMode, ModePolicyFilter, default_action_annotations,
};
use knhk_workflow_engine::autonomic::plan::{Action, ActionType};
use uuid::Uuid;

// ============================================================================
// Mode Satisfaction Tests
// ============================================================================

#[test]
fn test_minimum_mode_frozen_satisfied_by_all() {
    // Arrange
    let min_mode = MinimumMode::Frozen;

    // Act & Assert: Frozen actions allowed in all modes
    assert!(min_mode.satisfied_by(AutonomicMode::Frozen));
    assert!(min_mode.satisfied_by(AutonomicMode::Conservative));
    assert!(min_mode.satisfied_by(AutonomicMode::Normal));
}

#[test]
fn test_minimum_mode_conservative_requirements() {
    // Arrange
    let min_mode = MinimumMode::Conservative;

    // Act & Assert: Conservative requires Conservative or Normal
    assert!(!min_mode.satisfied_by(AutonomicMode::Frozen));
    assert!(min_mode.satisfied_by(AutonomicMode::Conservative));
    assert!(min_mode.satisfied_by(AutonomicMode::Normal));
}

#[test]
fn test_minimum_mode_normal_most_restrictive() {
    // Arrange
    let min_mode = MinimumMode::Normal;

    // Act & Assert: Normal only satisfied by Normal
    assert!(!min_mode.satisfied_by(AutonomicMode::Frozen));
    assert!(!min_mode.satisfied_by(AutonomicMode::Conservative));
    assert!(min_mode.satisfied_by(AutonomicMode::Normal));
}

// ============================================================================
// Action Pattern Matching Tests
// ============================================================================

#[test]
fn test_action_pattern_scale_instances_matches() {
    // Arrange
    let pattern = ActionPattern::ScaleInstances;
    let action = ActionType::ScaleInstances {
        service: "web".to_string(),
        delta: 2,
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_adjust_resources_matches() {
    // Arrange
    let pattern = ActionPattern::AdjustResources;
    let action = ActionType::AdjustResources {
        resource_type: "cpu".to_string(),
        delta: 0.5,
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_cancel_matches() {
    // Arrange
    let pattern = ActionPattern::Cancel;
    let action = ActionType::Cancel {
        case_id: uuid::Uuid::new_v4(),
        reason: "timeout".to_string(),
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_compensate_matches() {
    // Arrange
    let pattern = ActionPattern::Compensate;
    let action = ActionType::Compensate {
        case_id: uuid::Uuid::new_v4(),
        compensation_type: "rollback".to_string(),
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_migrate_runtime_matches() {
    // Arrange
    let pattern = ActionPattern::MigrateRuntime;
    let action = ActionType::MigrateRuntime {
        from_host: "host1".to_string(),
        to_host: "host2".to_string(),
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_optimize_pattern_matches() {
    // Arrange
    let pattern = ActionPattern::OptimizePattern;
    let action = ActionType::OptimizePattern {
        pattern_id: 42,
        optimization: "cache".to_string(),
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_custom_matches() {
    // Arrange
    let pattern = ActionPattern::Custom {
        name: "custom_action".to_string(),
    };
    let action = ActionType::Custom {
        name: "custom_action".to_string(),
        parameters: serde_json::json!({}),
    };

    // Act & Assert
    assert!(pattern.matches(&action));
}

#[test]
fn test_action_pattern_any_matches_all() {
    // Arrange
    let pattern = ActionPattern::Any;

    // Act & Assert: Any should match all action types
    assert!(pattern.matches(&ActionType::ScaleInstances {
        service: "test".to_string(),
        delta: 1,
    }));
    assert!(pattern.matches(&ActionType::AdjustResources {
        resource_type: "mem".to_string(),
        delta: 0.1,
    }));
    assert!(pattern.matches(&ActionType::Cancel {
        case_id: Uuid::new_v4(),
        reason: "test".to_string(),
    }));
}

// ============================================================================
// Default Annotations Tests
// ============================================================================

#[test]
fn test_default_annotations_structure() {
    // Arrange & Act
    let annotations = default_action_annotations();

    // Assert: Should have annotations for key action types
    assert!(!annotations.is_empty());

    // Find specific annotations
    let scale_annotation = annotations.iter()
        .find(|a| matches!(a.action_pattern, ActionPattern::ScaleInstances));
    assert!(scale_annotation.is_some());

    let adjust_annotation = annotations.iter()
        .find(|a| matches!(a.action_pattern, ActionPattern::AdjustResources));
    assert!(adjust_annotation.is_some());
}

#[test]
fn test_scale_instances_requires_normal_mode() {
    // Arrange
    let annotations = default_action_annotations();
    let scale_annotation = annotations.iter()
        .find(|a| matches!(a.action_pattern, ActionPattern::ScaleInstances))
        .expect("Scale annotation should exist");

    // Act & Assert: Should require Normal mode
    assert_eq!(scale_annotation.minimum_mode, MinimumMode::Normal);
}

#[test]
fn test_adjust_resources_allows_conservative() {
    // Arrange
    let annotations = default_action_annotations();
    let adjust_annotation = annotations.iter()
        .find(|a| matches!(a.action_pattern, ActionPattern::AdjustResources))
        .expect("Adjust annotation should exist");

    // Act & Assert: Should allow Conservative mode
    assert_eq!(adjust_annotation.minimum_mode, MinimumMode::Conservative);
}

#[test]
fn test_cancel_allows_conservative() {
    // Arrange
    let annotations = default_action_annotations();
    let cancel_annotation = annotations.iter()
        .find(|a| matches!(a.action_pattern, ActionPattern::Cancel))
        .expect("Cancel annotation should exist");

    // Act & Assert
    assert_eq!(cancel_annotation.minimum_mode, MinimumMode::Conservative);
}

#[test]
fn test_compensate_allows_conservative() {
    // Arrange
    let annotations = default_action_annotations();
    let compensate_annotation = annotations.iter()
        .find(|a| matches!(a.action_pattern, ActionPattern::Compensate))
        .expect("Compensate annotation should exist");

    // Act & Assert
    assert_eq!(compensate_annotation.minimum_mode, MinimumMode::Conservative);
}

// ============================================================================
// Mode Policy Filter Tests
// ============================================================================

#[test]
fn test_filter_allows_action_in_normal_mode() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "web".to_string(),
            delta: 2,
        },
        rationale: "Scale up".to_string(),
        policy_element: None,
    };

    // Act
    let result = filter.filter_action(&action, AutonomicMode::Normal);

    // Assert: Should be allowed in Normal mode
    assert!(result.is_ok());
}

#[test]
fn test_filter_blocks_scale_in_frozen_mode() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "web".to_string(),
            delta: 2,
        },
        rationale: "Scale up".to_string(),
        policy_element: None,
    };

    // Act
    let result = filter.filter_action(&action, AutonomicMode::Frozen);

    // Assert: Should be blocked in Frozen mode
    assert!(result.is_err());
}

#[test]
fn test_filter_blocks_scale_in_conservative_mode() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "web".to_string(),
            delta: 2,
        },
        rationale: "Scale up".to_string(),
        policy_element: None,
    };

    // Act
    let result = filter.filter_action(&action, AutonomicMode::Conservative);

    // Assert: Should be blocked in Conservative mode
    assert!(result.is_err());
}

#[test]
fn test_filter_allows_adjust_resources_in_conservative() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::AdjustResources {
            resource_type: "cpu".to_string(),
            delta: 0.1,
        },
        rationale: "Tune CPU".to_string(),
        policy_element: None,
    };

    // Act
    let result = filter.filter_action(&action, AutonomicMode::Conservative);

    // Assert: Should be allowed
    assert!(result.is_ok());
}

#[test]
fn test_filter_allows_cancel_in_conservative() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::Cancel {
            case_id: Uuid::new_v4(),
            reason: "timeout".to_string(),
        },
        rationale: "Cancel slow workflow".to_string(),
        policy_element: None,
    };

    // Act
    let result = filter.filter_action(&action, AutonomicMode::Conservative);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_filter_blocks_migrate_in_conservative() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::MigrateRuntime {
            from_host: "host1".to_string(),
            to_host: "host2".to_string(),
        },
        rationale: "Migrate to faster host".to_string(),
        policy_element: None,
    };

    // Act
    let result = filter.filter_action(&action, AutonomicMode::Conservative);

    // Assert: Migration requires Normal mode
    assert!(result.is_err());
}

// ============================================================================
// Mode Transition Tests
// ============================================================================

#[test]
fn test_mode_transition_normal_to_conservative() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "web".to_string(),
            delta: 1,
        },
        rationale: "Scale".to_string(),
        policy_element: None,
    };

    // Act: Allow in Normal, then transition to Conservative
    assert!(filter.filter_action(&action, AutonomicMode::Normal).is_ok());
    assert!(filter.filter_action(&action, AutonomicMode::Conservative).is_err());
}

#[test]
fn test_mode_transition_conservative_to_frozen() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::AdjustResources {
            resource_type: "mem".to_string(),
            delta: 0.2,
        },
        rationale: "Adjust memory".to_string(),
        policy_element: None,
    };

    // Act: Allow in Conservative, then block in Frozen
    assert!(filter.filter_action(&action, AutonomicMode::Conservative).is_ok());
    assert!(filter.filter_action(&action, AutonomicMode::Frozen).is_err());
}

#[test]
fn test_mode_recovery_frozen_to_normal() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::ScaleInstances {
            service: "api".to_string(),
            delta: 2,
        },
        rationale: "Scale up".to_string(),
        policy_element: None,
    };

    // Act: Blocked in Frozen, allowed after recovery to Normal
    assert!(filter.filter_action(&action, AutonomicMode::Frozen).is_err());
    assert!(filter.filter_action(&action, AutonomicMode::Normal).is_ok());
}

// ============================================================================
// Batch Action Filtering Tests
// ============================================================================

#[test]
fn test_filter_multiple_actions_in_normal_mode() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let actions = vec![
        Action {
            action_id: Uuid::new_v4(),
            action_type: ActionType::ScaleInstances {
                service: "web".to_string(),
                delta: 2,
            },
            rationale: "Scale".to_string(),
            policy_element: None,
        },
        Action {
            action_id: Uuid::new_v4(),
            action_type: ActionType::AdjustResources {
                resource_type: "cpu".to_string(),
                delta: 0.5,
            },
            rationale: "Tune".to_string(),
            policy_element: None,
        },
    ];

    // Act
    let results: Vec<_> = actions.iter()
        .map(|a| filter.filter_action(a, AutonomicMode::Normal))
        .collect();

    // Assert: All should be allowed in Normal
    assert!(results.iter().all(|r| r.is_ok()));
}

#[test]
fn test_filter_mixed_actions_in_conservative_mode() {
    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let actions = vec![
        Action {
            action_id: Uuid::new_v4(),
            action_type: ActionType::ScaleInstances {
                service: "web".to_string(),
                delta: 2,
            },
            rationale: "Scale".to_string(),
            policy_element: None,
        },
        Action {
            action_id: Uuid::new_v4(),
            action_type: ActionType::AdjustResources {
                resource_type: "cpu".to_string(),
                delta: 0.1,
            },
            rationale: "Tune".to_string(),
            policy_element: None,
        },
        Action {
            action_id: Uuid::new_v4(),
            action_type: ActionType::Cancel {
                case_id: Uuid::new_v4(),
                reason: "timeout".to_string(),
            },
            rationale: "Cancel".to_string(),
            policy_element: None,
        },
    ];

    // Act
    let results: Vec<_> = actions.iter()
        .map(|a| filter.filter_action(a, AutonomicMode::Conservative))
        .collect();

    // Assert: Scale should fail, others should succeed
    assert!(results[0].is_err()); // ScaleInstances blocked
    assert!(results[1].is_ok());  // AdjustResources allowed
    assert!(results[2].is_ok());  // Cancel allowed
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_action_filtering_performance() {
    use std::time::Instant;

    // Arrange
    let filter = ModePolicyFilter::new(default_action_annotations());
    let action = Action {
        action_id: Uuid::new_v4(),
        action_type: ActionType::AdjustResources {
            resource_type: "cpu".to_string(),
            delta: 0.1,
        },
        rationale: "Tune".to_string(),
        policy_element: None,
    };

    // Act: Filter 100,000 times
    let start = Instant::now();
    for _ in 0..100_000 {
        let _ = filter.filter_action(&action, AutonomicMode::Normal);
    }
    let elapsed = start.elapsed();

    // Assert: Should be very fast (< 100ms)
    assert!(
        elapsed.as_millis() < 100,
        "Action filtering should be fast: took {}ms",
        elapsed.as_millis()
    );
}
