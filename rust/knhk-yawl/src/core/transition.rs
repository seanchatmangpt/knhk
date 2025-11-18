//! Transition and control flow structures
//!
//! Covenant 4: All patterns are expressible via permutations

use serde::{Deserialize, Serialize};
use std::fmt;

/// Split type for control flow (from YAWL ontology)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SplitType {
    /// AND-split: all outgoing branches execute in parallel
    AND,
    /// XOR-split: exactly one outgoing branch executes
    XOR,
    /// OR-split: one or more outgoing branches execute
    OR,
}

impl fmt::Display for SplitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AND => write!(f, "AND"),
            Self::XOR => write!(f, "XOR"),
            Self::OR => write!(f, "OR"),
        }
    }
}

/// Join type for control flow (from YAWL ontology)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JoinType {
    /// AND-join: wait for all incoming branches
    AND,
    /// XOR-join: proceed when first incoming branch arrives
    XOR,
    /// OR-join: wait for all active incoming branches
    OR,
}

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AND => write!(f, "AND"),
            Self::XOR => write!(f, "XOR"),
            Self::OR => write!(f, "OR"),
        }
    }
}

/// A transition between tasks in a workflow
///
/// Represents control flow from one task to another.
/// Aligned with `yawl:Transition` in the YAWL ontology.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transition {
    /// Source task ID
    pub source: String,

    /// Target task ID
    pub target: String,

    /// Condition for this transition (optional)
    pub condition: Option<String>,

    /// Join type (if target has multiple incoming transitions)
    pub join_type: Option<JoinType>,

    /// Split type (if source has multiple outgoing transitions)
    pub split_type: Option<SplitType>,

    /// Flow label (for identification)
    pub label: Option<String>,
}

impl Transition {
    /// Create a new transition builder
    #[must_use]
    pub fn builder() -> TransitionBuilder {
        TransitionBuilder::default()
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transition({} -> {})", self.source, self.target)?;
        if let Some(ref condition) = self.condition {
            write!(f, " [condition: {}]", condition)?;
        }
        Ok(())
    }
}

/// Builder for constructing Transitions
#[derive(Default)]
pub struct TransitionBuilder {
    source: Option<String>,
    target: Option<String>,
    condition: Option<String>,
    join_type: Option<JoinType>,
    split_type: Option<SplitType>,
    label: Option<String>,
}

impl TransitionBuilder {
    /// Set source task
    #[must_use]
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set target task
    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set condition
    #[must_use]
    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Set join type
    #[must_use]
    pub fn join_type(mut self, join_type: JoinType) -> Self {
        self.join_type = Some(join_type);
        self
    }

    /// Set split type
    #[must_use]
    pub fn split_type(mut self, split_type: SplitType) -> Self {
        self.split_type = Some(split_type);
        self
    }

    /// Set label
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Build the transition
    ///
    /// # Panics
    /// Panics if required fields are not set
    #[must_use]
    pub fn build(self) -> Transition {
        Transition {
            source: self.source.expect("Source task is required"),
            target: self.target.expect("Target task is required"),
            condition: self.condition,
            join_type: self.join_type,
            split_type: self.split_type,
            label: self.label,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_builder() {
        let transition = Transition::builder()
            .source("task1")
            .target("task2")
            .condition("x > 10")
            .split_type(SplitType::XOR)
            .build();

        assert_eq!(transition.source, "task1");
        assert_eq!(transition.target, "task2");
        assert_eq!(transition.condition, Some("x > 10".to_string()));
        assert_eq!(transition.split_type, Some(SplitType::XOR));
    }

    #[test]
    fn test_split_join_types() {
        assert_eq!(format!("{}", SplitType::AND), "AND");
        assert_eq!(format!("{}", JoinType::XOR), "XOR");
    }
}
