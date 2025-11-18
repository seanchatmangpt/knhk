//! Core YAWL Types
//!
//! Concrete types for workflow elements mapped from RDF ontology.

use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// YAWL Task - the fundamental unit of work
///
/// Maps to `yawl:Task` in the ontology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub split_type: Option<SplitType>,
    pub join_type: Option<JoinType>,
    pub preconditions: Vec<Predicate>,
    pub postconditions: Vec<Predicate>,
    pub outgoing_arcs: Vec<ArcId>,
    pub incoming_arcs: Vec<ArcId>,
    pub data_inputs: Vec<DataBinding>,
    pub data_outputs: Vec<DataBinding>,
    pub cancellation_region: Option<CancellationRegion>,
}

/// Arc - flow between tasks
///
/// Maps to `yawl:Arc` in the ontology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    pub id: ArcId,
    pub source: TaskId,
    pub target: TaskId,
    pub arc_type: ArcType,
    pub predicate: Option<Predicate>,
    pub priority: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcType {
    ControlFlow,
    DataFlow,
}

/// Predicate - conditional expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub expression: String,
    pub variables: Vec<String>,
}

/// Data binding - maps workflow data to task parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBinding {
    pub name: String,
    pub path: String,
    pub binding_type: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    Json,
    Custom(String),
}

/// Cancellation region - defines scope of cancellation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationRegion {
    pub tasks: Vec<TaskId>,
    pub on_cancel: Option<TaskId>, // Compensation task
}

/// Complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: WorkflowId,
    pub name: String,
    pub version: String,
    pub tasks: HashMap<TaskId, Task>,
    pub arcs: HashMap<ArcId, Arc>,
    pub start_task: TaskId,
    pub end_tasks: Vec<TaskId>,
    pub global_data: HashMap<String, DataType>,
}

impl Workflow {
    /// Validate workflow structure against Q invariants
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Q1: Check for unreachable tasks
        self.check_reachability()?;

        // Q2: Validate split-join combinations against permutation matrix
        self.validate_split_join_combinations()?;

        // Q3: Check for invalid cycles
        self.check_cycles()?;

        Ok(())
    }

    fn check_reachability(&self) -> Result<(), ValidationError> {
        // TODO: Implement reachability analysis
        Ok(())
    }

    fn validate_split_join_combinations(&self) -> Result<(), ValidationError> {
        for task in self.tasks.values() {
            if let (Some(split), Some(join)) = (&task.split_type, &task.join_type) {
                // Validate against yawl-pattern-permutations.ttl
                if !is_valid_combination(*split, *join) {
                    return Err(ValidationError::InvalidSplitJoinCombination {
                        split: *split,
                        join: *join,
                    });
                }
            }
        }
        Ok(())
    }

    fn check_cycles(&self) -> Result<(), ValidationError> {
        // TODO: Implement cycle detection (allowed for loops, not for invalid recursion)
        Ok(())
    }
}

/// Check if split-join combination is valid per permutation matrix
fn is_valid_combination(split: SplitType, join: JoinType) -> bool {
    use JoinType::*;
    use SplitType::*;

    match (split, join) {
        // Valid combinations from yawl-pattern-permutations.ttl
        (XOR, XOR) => true,
        (AND, AND) => true,
        (AND, XOR) => true,
        (AND, OR) => true,
        (OR, OR) => true,
        (OR, XOR) => true,
        (XOR, Discriminator { .. }) => true,
        (AND, Discriminator { .. }) => true,
        // Invalid combinations
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_combinations() {
        assert!(is_valid_combination(SplitType::XOR, JoinType::XOR));
        assert!(is_valid_combination(SplitType::AND, JoinType::AND));
        assert!(is_valid_combination(SplitType::AND, JoinType::XOR));
    }

    #[test]
    fn test_invalid_combinations() {
        // OR split with AND join is generally invalid
        // (OR join must be used to handle variable activations)
        assert!(!is_valid_combination(SplitType::OR, JoinType::AND));
    }
}
