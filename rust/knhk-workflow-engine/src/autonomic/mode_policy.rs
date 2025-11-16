// rust/knhk-workflow-engine/src/autonomic/mode_policy.rs
//! Mode-Aware Policy Enforcement
//!
//! Integrates autonomic modes with the policy lattice to gate actions
//! based on current system health and operating mode.
//!
//! **Design Principles**:
//! 1. Action types are statically annotated with minimum required mode
//! 2. Mode filtering is fail-safe: when in doubt, reject
//! 3. Mode changes propagate through policy lattice
//! 4. Rejected actions are logged and receipted
//! 5. Policy violations are observable via telemetry

use super::failure_modes::AutonomicMode;
use super::plan::{Action, ActionType};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Minimum mode required for an action type
///
/// This maps action types to the least restrictive mode that allows them.
/// Actions require Normal mode by default unless explicitly marked as safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinimumMode {
    /// Only allowed in Normal mode
    Normal,
    /// Allowed in Conservative or better
    Conservative,
    /// Allowed in any mode (Frozen and above)
    Frozen,
}

impl MinimumMode {
    /// Check if this minimum is satisfied by a given mode
    pub fn satisfied_by(&self, mode: AutonomicMode) -> bool {
        use AutonomicMode::*;
        match (self, mode) {
            (MinimumMode::Frozen, _) => true,
            (MinimumMode::Conservative, Normal) | (MinimumMode::Conservative, Conservative) => {
                true
            }
            (MinimumMode::Normal, Normal) => true,
            _ => false,
        }
    }
}

/// Action annotation with mode requirements
///
/// Statically declares what mode is required for each action type.
/// This is the type-level enforcement mechanism for mode-aware policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionAnnotation {
    /// Action type pattern
    pub action_pattern: ActionPattern,
    /// Minimum mode required
    pub minimum_mode: MinimumMode,
    /// Human-readable rationale
    pub rationale: String,
}

/// Pattern for matching action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionPattern {
    /// Exact action type match
    ScaleInstances,
    AdjustResources,
    Cancel,
    Compensate,
    MigrateRuntime,
    OptimizePattern,
    Custom { name: String },
    /// Wildcard - matches all
    Any,
}

impl ActionPattern {
    /// Check if this pattern matches an action type
    pub fn matches(&self, action: &ActionType) -> bool {
        use ActionPattern::*;
        match (self, action) {
            (Any, _) => true,
            (ScaleInstances, ActionType::ScaleInstances { .. }) => true,
            (AdjustResources, ActionType::AdjustResources { .. }) => true,
            (Cancel, ActionType::Cancel { .. }) => true,
            (Compensate, ActionType::Compensate { .. }) => true,
            (MigrateRuntime, ActionType::MigrateRuntime { .. }) => true,
            (OptimizePattern, ActionType::OptimizePattern { .. }) => true,
            (Custom { name: pattern_name }, ActionType::Custom { name, .. }) => {
                pattern_name == name
            }
            _ => false,
        }
    }
}

/// Default action annotations
///
/// These define the safe ceiling for each action type.
/// More risky actions require better system health.
pub fn default_action_annotations() -> Vec<ActionAnnotation> {
    vec![
        // Structural changes require Normal mode
        ActionAnnotation {
            action_pattern: ActionPattern::ScaleInstances,
            minimum_mode: MinimumMode::Normal,
            rationale: "Scaling changes system topology and requires full confidence".to_string(),
        },
        ActionAnnotation {
            action_pattern: ActionPattern::MigrateRuntime,
            minimum_mode: MinimumMode::Normal,
            rationale: "Runtime migration is complex and requires full health".to_string(),
        },
        // Resource adjustments allowed in Conservative mode
        ActionAnnotation {
            action_pattern: ActionPattern::AdjustResources,
            minimum_mode: MinimumMode::Conservative,
            rationale: "Resource tuning is low-risk and reversible".to_string(),
        },
        ActionAnnotation {
            action_pattern: ActionPattern::OptimizePattern,
            minimum_mode: MinimumMode::Conservative,
            rationale: "Pattern optimization is safe runtime tuning".to_string(),
        },
        // Reactive actions allowed in Conservative mode
        ActionAnnotation {
            action_pattern: ActionPattern::Cancel,
            minimum_mode: MinimumMode::Conservative,
            rationale: "Cancellation is defensive and prevents resource waste".to_string(),
        },
        ActionAnnotation {
            action_pattern: ActionPattern::Compensate,
            minimum_mode: MinimumMode::Conservative,
            rationale: "Compensation is corrective and maintains consistency".to_string(),
        },
        // Custom actions default to Normal mode (fail-safe)
        ActionAnnotation {
            action_pattern: ActionPattern::Custom {
                name: "*".to_string(),
            },
            minimum_mode: MinimumMode::Normal,
            rationale: "Unknown custom actions require full system health".to_string(),
        },
    ]
}

/// Mode-aware policy filter
///
/// Filters actions based on current autonomic mode and policy annotations.
pub struct ModePolicyFilter {
    /// Action annotations
    annotations: HashMap<String, ActionAnnotation>,
}

impl ModePolicyFilter {
    /// Create new filter with default annotations
    pub fn new() -> Self {
        let mut annotations = HashMap::new();
        for annotation in default_action_annotations() {
            let key = Self::pattern_key(&annotation.action_pattern);
            annotations.insert(key, annotation);
        }
        Self { annotations }
    }

    /// Add custom annotation
    pub fn add_annotation(&mut self, annotation: ActionAnnotation) {
        let key = Self::pattern_key(&annotation.action_pattern);
        self.annotations.insert(key, annotation);
    }

    /// Get pattern key for lookup
    fn pattern_key(pattern: &ActionPattern) -> String {
        use ActionPattern::*;
        match pattern {
            Any => "any".to_string(),
            ScaleInstances => "scale_instances".to_string(),
            AdjustResources => "adjust_resources".to_string(),
            Cancel => "cancel".to_string(),
            Compensate => "compensate".to_string(),
            MigrateRuntime => "migrate_runtime".to_string(),
            OptimizePattern => "optimize_pattern".to_string(),
            Custom { name } => format!("custom:{}", name),
        }
    }

    /// Get action type pattern
    fn action_type_pattern(action: &ActionType) -> ActionPattern {
        use ActionPattern::*;
        match action {
            ActionType::ScaleInstances { .. } => ScaleInstances,
            ActionType::AdjustResources { .. } => AdjustResources,
            ActionType::Cancel { .. } => Cancel,
            ActionType::Compensate { .. } => Compensate,
            ActionType::MigrateRuntime { .. } => MigrateRuntime,
            ActionType::OptimizePattern { .. } => OptimizePattern,
            ActionType::Custom { name, .. } => Custom { name: name.clone() },
        }
    }

    /// Check if action is allowed in current mode
    pub fn is_allowed(&self, action: &ActionType, mode: AutonomicMode) -> bool {
        // Get pattern for this action
        let pattern = Self::action_type_pattern(action);
        let key = Self::pattern_key(&pattern);

        // Look up annotation
        let annotation = self.annotations.get(&key);

        // If no specific annotation, check for wildcard
        let annotation = annotation.or_else(|| self.annotations.get("any"));

        // If still no annotation, fail-safe to Normal mode requirement
        let minimum_mode = annotation
            .map(|a| a.minimum_mode)
            .unwrap_or(MinimumMode::Normal);

        let allowed = minimum_mode.satisfied_by(mode);

        if !allowed {
            debug!(
                action = ?action,
                current_mode = ?mode,
                required_mode = ?minimum_mode,
                "Action rejected by mode policy"
            );
        }

        allowed
    }

    /// Filter actions based on mode
    pub fn filter_actions(&self, actions: &[Action], mode: AutonomicMode) -> Vec<Action> {
        actions
            .iter()
            .filter(|action| self.is_allowed(&action.action_type, mode))
            .cloned()
            .collect()
    }

    /// Filter actions and return rejected ones separately
    pub fn filter_with_rejected(
        &self,
        actions: &[Action],
        mode: AutonomicMode,
    ) -> (Vec<Action>, Vec<RejectedAction>) {
        let mut allowed = Vec::new();
        let mut rejected = Vec::new();

        for action in actions {
            if self.is_allowed(&action.action_type, mode) {
                allowed.push(action.clone());
            } else {
                let pattern = Self::action_type_pattern(&action.action_type);
                let key = Self::pattern_key(&pattern);
                let annotation = self.annotations.get(&key);

                let reason = annotation
                    .map(|a| a.rationale.clone())
                    .unwrap_or_else(|| "Action not allowed in current mode".to_string());

                let required_mode = annotation
                    .map(|a| a.minimum_mode)
                    .unwrap_or(MinimumMode::Normal);

                rejected.push(RejectedAction {
                    action: action.clone(),
                    current_mode: mode,
                    required_mode,
                    reason,
                });

                warn!(
                    action_id = ?action.id,
                    action_type = ?action.action_type,
                    current_mode = ?mode,
                    required_mode = ?required_mode,
                    reason = %reason,
                    "Action rejected by mode policy"
                );
            }
        }

        (allowed, rejected)
    }

    /// Get all annotations
    pub fn get_annotations(&self) -> Vec<&ActionAnnotation> {
        self.annotations.values().collect()
    }
}

impl Default for ModePolicyFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rejected action with reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedAction {
    /// The rejected action
    pub action: Action,
    /// Current mode that rejected it
    pub current_mode: AutonomicMode,
    /// Mode required for this action
    pub required_mode: MinimumMode,
    /// Reason for rejection
    pub reason: String,
}

/// Mode-aware adaptation plan
///
/// Wraps a standard adaptation plan with mode filtering.
pub struct ModeAwareAdaptationPlan {
    /// Allowed actions (passed mode filter)
    pub allowed_actions: Vec<Action>,
    /// Rejected actions (failed mode filter)
    pub rejected_actions: Vec<RejectedAction>,
    /// Mode at time of filtering
    pub mode: AutonomicMode,
}

impl ModeAwareAdaptationPlan {
    /// Create from plan and mode
    pub fn from_plan(
        plan: &super::plan::AdaptationPlan,
        mode: AutonomicMode,
        filter: &ModePolicyFilter,
    ) -> Self {
        let (allowed, rejected) = filter.filter_with_rejected(&plan.actions, mode);

        Self {
            allowed_actions: allowed,
            rejected_actions: rejected,
            mode,
        }
    }

    /// Check if any actions were allowed
    pub fn has_allowed_actions(&self) -> bool {
        !self.allowed_actions.is_empty()
    }

    /// Check if any actions were rejected
    pub fn has_rejected_actions(&self) -> bool {
        !self.rejected_actions.is_empty()
    }

    /// Get total action count (allowed + rejected)
    pub fn total_actions(&self) -> usize {
        self.allowed_actions.len() + self.rejected_actions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_mode_satisfaction() {
        // Frozen is satisfied by any mode
        assert!(MinimumMode::Frozen.satisfied_by(AutonomicMode::Frozen));
        assert!(MinimumMode::Frozen.satisfied_by(AutonomicMode::Conservative));
        assert!(MinimumMode::Frozen.satisfied_by(AutonomicMode::Normal));

        // Conservative requires Conservative or Normal
        assert!(!MinimumMode::Conservative.satisfied_by(AutonomicMode::Frozen));
        assert!(MinimumMode::Conservative.satisfied_by(AutonomicMode::Conservative));
        assert!(MinimumMode::Conservative.satisfied_by(AutonomicMode::Normal));

        // Normal requires Normal
        assert!(!MinimumMode::Normal.satisfied_by(AutonomicMode::Frozen));
        assert!(!MinimumMode::Normal.satisfied_by(AutonomicMode::Conservative));
        assert!(MinimumMode::Normal.satisfied_by(AutonomicMode::Normal));
    }

    #[test]
    fn test_action_pattern_matching() {
        let scale_pattern = ActionPattern::ScaleInstances;
        assert!(scale_pattern.matches(&ActionType::ScaleInstances { delta: 2 }));
        assert!(!scale_pattern.matches(&ActionType::Cancel {
            target: "test".to_string()
        }));

        let any_pattern = ActionPattern::Any;
        assert!(any_pattern.matches(&ActionType::ScaleInstances { delta: 2 }));
        assert!(any_pattern.matches(&ActionType::Cancel {
            target: "test".to_string()
        }));
    }

    #[test]
    fn test_mode_policy_filter_normal() {
        let filter = ModePolicyFilter::new();

        // All actions allowed in Normal mode
        assert!(filter.is_allowed(&ActionType::ScaleInstances { delta: 2 }, AutonomicMode::Normal));
        assert!(filter.is_allowed(
            &ActionType::MigrateRuntime {
                from: "W1".to_string(),
                to: "R1".to_string()
            },
            AutonomicMode::Normal
        ));
        assert!(filter.is_allowed(
            &ActionType::AdjustResources {
                resource: "cpu".to_string(),
                amount: 0.1
            },
            AutonomicMode::Normal
        ));
    }

    #[test]
    fn test_mode_policy_filter_conservative() {
        let filter = ModePolicyFilter::new();

        // Structural changes not allowed in Conservative
        assert!(!filter.is_allowed(
            &ActionType::ScaleInstances { delta: 2 },
            AutonomicMode::Conservative
        ));
        assert!(!filter.is_allowed(
            &ActionType::MigrateRuntime {
                from: "W1".to_string(),
                to: "R1".to_string()
            },
            AutonomicMode::Conservative
        ));

        // Safe actions allowed in Conservative
        assert!(filter.is_allowed(
            &ActionType::AdjustResources {
                resource: "cpu".to_string(),
                amount: 0.1
            },
            AutonomicMode::Conservative
        ));
        assert!(filter.is_allowed(
            &ActionType::OptimizePattern { pattern_id: 12 },
            AutonomicMode::Conservative
        ));
        assert!(filter.is_allowed(
            &ActionType::Cancel {
                target: "task1".to_string()
            },
            AutonomicMode::Conservative
        ));
    }

    #[test]
    fn test_mode_policy_filter_frozen() {
        let filter = ModePolicyFilter::new();

        // No actions allowed in Frozen mode
        assert!(!filter.is_allowed(
            &ActionType::ScaleInstances { delta: 2 },
            AutonomicMode::Frozen
        ));
        assert!(!filter.is_allowed(
            &ActionType::AdjustResources {
                resource: "cpu".to_string(),
                amount: 0.1
            },
            AutonomicMode::Frozen
        ));
        assert!(!filter.is_allowed(
            &ActionType::Cancel {
                target: "task1".to_string()
            },
            AutonomicMode::Frozen
        ));
    }

    #[test]
    fn test_filter_actions() {
        let filter = ModePolicyFilter::new();

        let actions = vec![
            Action::new(ActionType::ScaleInstances { delta: 2 }),
            Action::new(ActionType::AdjustResources {
                resource: "cpu".to_string(),
                amount: 0.1,
            }),
            Action::new(ActionType::OptimizePattern { pattern_id: 12 }),
        ];

        // In Normal mode, all actions pass
        let filtered = filter.filter_actions(&actions, AutonomicMode::Normal);
        assert_eq!(filtered.len(), 3);

        // In Conservative mode, only safe actions pass
        let filtered = filter.filter_actions(&actions, AutonomicMode::Conservative);
        assert_eq!(filtered.len(), 2); // AdjustResources and OptimizePattern

        // In Frozen mode, no actions pass
        let filtered = filter.filter_actions(&actions, AutonomicMode::Frozen);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_with_rejected() {
        let filter = ModePolicyFilter::new();

        let actions = vec![
            Action::new(ActionType::ScaleInstances { delta: 2 }),
            Action::new(ActionType::AdjustResources {
                resource: "cpu".to_string(),
                amount: 0.1,
            }),
        ];

        let (allowed, rejected) = filter.filter_with_rejected(&actions, AutonomicMode::Conservative);

        assert_eq!(allowed.len(), 1); // Only AdjustResources
        assert_eq!(rejected.len(), 1); // ScaleInstances rejected

        assert_eq!(rejected[0].current_mode, AutonomicMode::Conservative);
        assert_eq!(rejected[0].required_mode, MinimumMode::Normal);
    }

    #[test]
    fn test_custom_annotation() {
        let mut filter = ModePolicyFilter::new();

        // Add custom annotation for a specific custom action
        filter.add_annotation(ActionAnnotation {
            action_pattern: ActionPattern::Custom {
                name: "safe_custom".to_string(),
            },
            minimum_mode: MinimumMode::Conservative,
            rationale: "This custom action is safe".to_string(),
        });

        // Safe custom action allowed in Conservative
        assert!(filter.is_allowed(
            &ActionType::Custom {
                name: "safe_custom".to_string(),
                params: "{}".to_string()
            },
            AutonomicMode::Conservative
        ));

        // Unknown custom action requires Normal (fail-safe default)
        assert!(!filter.is_allowed(
            &ActionType::Custom {
                name: "unknown_custom".to_string(),
                params: "{}".to_string()
            },
            AutonomicMode::Conservative
        ));
    }
}
