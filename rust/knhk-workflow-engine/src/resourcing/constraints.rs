//! YAWL Resource Constraints
//!
//! Implements YAWL resource constraints with TRIZ Principle 35: Parameter Changes
//! - Dynamic constraint parameters based on context
//!
//! Based on: org.yawlfoundation.yawl.resourcing.constraints

use crate::error::{WorkflowError, WorkflowResult};
use crate::resource::allocation::types::ResourceId;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Constraint result
#[derive(Debug, Clone)]
pub struct ConstraintResult {
    /// Whether constraint is satisfied
    pub satisfied: bool,
    /// Constraint reason
    pub reason: Option<String>,
}

/// Constraint trait
pub trait Constraint: Send + Sync {
    /// Check if constraint is satisfied
    fn check(
        &self,
        resource_id: &ResourceId,
        context: &ConstraintContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<ConstraintResult>> + Send>>;
}

/// Constraint context
#[derive(Debug, Clone)]
pub struct ConstraintContext {
    /// Task ID
    pub task_id: String,
    /// Case ID
    pub case_id: String,
    /// Previous task assignments (for history-based constraints)
    pub previous_assignments: HashMap<String, ResourceId>,
    /// Current assignments
    pub current_assignments: HashMap<String, ResourceId>,
}

/// Constraint type (TRIZ Principle 32: Color Changes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintType {
    /// Separation of duties - Different users for tasks
    SeparationOfDuties,
    /// Retain familiar - Same user for related tasks
    RetainFamiliar,
    /// Case completion - Same user for all case tasks
    CaseCompletion,
    /// Simultaneous execution - Concurrent task allocation
    SimultaneousExecution,
    /// 4-eyes principle - Dual authorization
    FourEyesPrinciple,
    /// History constraint - Previous task completion affects eligibility
    HistoryConstraint,
    /// Data-based constraint - Task data determines eligible users
    DataBasedConstraint,
    /// Custom constraint - User-defined logic
    Custom,
}

/// Separation of duties constraint
///
/// Ensures different users are assigned to different tasks
pub struct SeparationOfDuties {
    /// Task pairs that require separation
    task_pairs: Vec<(String, String)>,
    /// Assignment history
    assignment_history: Arc<DashMap<String, ResourceId>>,
}

impl SeparationOfDuties {
    pub fn new(task_pairs: Vec<(String, String)>) -> Self {
        Self {
            task_pairs,
            assignment_history: Arc::new(DashMap::new()),
        }
    }
}

impl Constraint for SeparationOfDuties {
    fn check(
        &self,
        resource_id: &ResourceId,
        context: &ConstraintContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<ConstraintResult>> + Send>>
    {
        let task_pairs = self.task_pairs.clone();
        let assignment_history = self.assignment_history.clone();
        let task_id = context.task_id.clone();
        let resource_id = resource_id.clone();

        Box::pin(async move {
            // Check if this task is part of a separation pair
            for (task1, task2) in &task_pairs {
                if task_id == *task1 {
                    // Check if task2 was assigned to the same resource
                    if let Some(assigned_resource) = assignment_history.get(task2) {
                        if assigned_resource.value() == &resource_id {
                            return Ok(ConstraintResult {
                                satisfied: false,
                                reason: Some(format!(
                                    "Separation of duties violated: {} and {} assigned to same resource",
                                    task1, task2
                                )),
                            });
                        }
                    }
                } else if task_id == *task2 {
                    // Check if task1 was assigned to the same resource
                    if let Some(assigned_resource) = assignment_history.get(task1) {
                        if assigned_resource.value() == &resource_id {
                            return Ok(ConstraintResult {
                                satisfied: false,
                                reason: Some(format!(
                                    "Separation of duties violated: {} and {} assigned to same resource",
                                    task1, task2
                                )),
                            });
                        }
                    }
                }
            }

            Ok(ConstraintResult {
                satisfied: true,
                reason: None,
            })
        })
    }
}

/// 4-eyes principle constraint
///
/// Requires two different users to approve/execute
pub struct FourEyesPrinciple {
    /// Required approver role
    approver_role: String,
    /// Assignment history
    assignment_history: Arc<DashMap<String, ResourceId>>,
}

impl FourEyesPrinciple {
    pub fn new(approver_role: String) -> Self {
        Self {
            approver_role,
            assignment_history: Arc::new(DashMap::new()),
        }
    }
}

impl Constraint for FourEyesPrinciple {
    fn check(
        &self,
        resource_id: &ResourceId,
        context: &ConstraintContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<ConstraintResult>> + Send>>
    {
        let assignment_history = self.assignment_history.clone();
        let case_id = context.case_id.clone();
        let resource_id = resource_id.clone();

        Box::pin(async move {
            // Check if another resource has already been assigned to this case
            let mut other_assignments = 0;
            for entry in assignment_history.iter() {
                let key = entry.key();
                let assigned_resource = entry.value();
                if key.contains(&case_id) && assigned_resource != &resource_id {
                    other_assignments += 1;
                }
            }

            // 4-eyes requires at least one other assignment
            if other_assignments == 0 {
                Ok(ConstraintResult {
                    satisfied: false,
                    reason: Some(
                        "4-eyes principle requires at least one other approver".to_string(),
                    ),
                })
            } else {
                Ok(ConstraintResult {
                    satisfied: true,
                    reason: None,
                })
            }
        })
    }
}

/// Composite constraint (TRIZ Principle 40: Composite Materials)
///
/// Combines multiple constraints
pub struct CompositeConstraint {
    constraints: Vec<Arc<dyn Constraint>>,
}

impl CompositeConstraint {
    pub fn new(constraints: Vec<Arc<dyn Constraint>>) -> Self {
        Self { constraints }
    }
}

impl Constraint for CompositeConstraint {
    fn check(
        &self,
        resource_id: &ResourceId,
        context: &ConstraintContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WorkflowResult<ConstraintResult>> + Send>>
    {
        let constraints = self.constraints.clone();
        let resource_id = resource_id.clone();
        let context = context.clone();

        Box::pin(async move {
            for constraint in constraints {
                let result = constraint.check(&resource_id, &context).await?;
                if !result.satisfied {
                    return Ok(result);
                }
            }

            Ok(ConstraintResult {
                satisfied: true,
                reason: None,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_separation_of_duties() {
        let constraint = SeparationOfDuties::new(vec![("task1".to_string(), "task2".to_string())]);

        let context = ConstraintContext {
            task_id: "task1".to_string(),
            case_id: "case1".to_string(),
            previous_assignments: HashMap::new(),
            current_assignments: HashMap::new(),
        };

        // Use UUID-based ResourceId for testing
        let resource_id = ResourceId::new();
        let result = constraint.check(&resource_id, &context).await.unwrap();
        assert!(result.satisfied);
    }
}
