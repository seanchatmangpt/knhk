// rust/knhk-workflow-engine/src/autonomic/plan.rs
//! Plan Component for MAPE-K Framework
//!
//! Generates adaptation plans based on analysis results.

use super::analyze::Analysis;
use super::knowledge::KnowledgeBase;
use super::policy_lattice::PolicyElement;
use crate::error::WorkflowResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Action identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActionId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl ActionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ActionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Action type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Scale multi-instance count
    ScaleInstances { delta: i32 },
    /// Adjust resource allocation
    AdjustResources { resource: String, amount: f64 },
    /// Cancel task/region
    Cancel { target: String },
    /// Trigger compensation
    Compensate { task_id: String },
    /// Migrate to different runtime class
    MigrateRuntime { from: String, to: String },
    /// Optimize pattern execution
    OptimizePattern { pattern_id: u8 },
    /// Custom action
    Custom { name: String, params: String },
}

/// Adaptation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action identifier
    pub id: ActionId,
    /// Action type
    pub action_type: ActionType,
    /// Priority (0-100, higher = more urgent)
    pub priority: u8,
    /// Expected impact (0.0-1.0)
    pub expected_impact: f64,
    /// Cost/risk (0.0-1.0)
    pub cost: f64,
    /// Policy element governing this action
    /// Actions without policy constraints use an empty Conjunction
    pub policy: Option<PolicyElement>,
}

impl Action {
    pub fn new(action_type: ActionType) -> Self {
        Self {
            id: ActionId::new(),
            action_type,
            priority: 50,
            expected_impact: 0.5,
            cost: 0.3,
            policy: None,
        }
    }

    /// Create action with policy element
    pub fn with_policy(action_type: ActionType, policy: PolicyElement) -> Self {
        Self {
            id: ActionId::new(),
            action_type,
            priority: 50,
            expected_impact: 0.5,
            cost: 0.3,
            policy: Some(policy),
        }
    }

    /// Set policy for this action
    pub fn set_policy(&mut self, policy: PolicyElement) {
        self.policy = Some(policy);
    }

    /// Get policy element
    pub fn get_policy(&self) -> Option<&PolicyElement> {
        self.policy.as_ref()
    }

    /// Check if action has policy constraints
    pub fn has_policy(&self) -> bool {
        self.policy.is_some()
    }
}

/// Adaptation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationPlan {
    /// Plan identifier
    pub id: Uuid,
    /// Actions to execute
    pub actions: Vec<Action>,
    /// Plan priority
    pub priority: u8,
    /// Expected benefit
    pub expected_benefit: f64,
    /// Plan timestamp
    pub timestamp_ms: u64,
}

impl AdaptationPlan {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            actions: Vec::new(),
            priority: 50,
            expected_benefit: 0.0,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Calculate total cost
    pub fn total_cost(&self) -> f64 {
        self.actions.iter().map(|a| a.cost).sum()
    }

    /// Calculate benefit/cost ratio
    pub fn benefit_cost_ratio(&self) -> f64 {
        let cost = self.total_cost();
        if cost > 0.0 {
            self.expected_benefit / cost
        } else {
            self.expected_benefit
        }
    }
}

impl Default for AdaptationPlan {
    fn default() -> Self {
        Self::new()
    }
}

/// Planner component
pub struct Planner {
    /// Knowledge base
    knowledge: Arc<KnowledgeBase>,
}

impl Planner {
    /// Create new planner
    pub fn new(knowledge: Arc<KnowledgeBase>) -> Self {
        Self { knowledge }
    }

    /// Generate adaptation plan
    pub async fn plan(&self, analysis: &Analysis) -> WorkflowResult<Option<AdaptationPlan>> {
        if !analysis.adaptation_needed {
            return Ok(None);
        }

        let mut plan = AdaptationPlan::new();

        // Generate actions based on violated goals
        for goal in &analysis.violated_goals {
            let actions = self.generate_actions_for_goal(goal).await?;
            plan.actions.extend(actions);
        }

        // Generate actions based on anomalies
        for anomaly in &analysis.anomalies {
            let actions = self.generate_actions_for_anomaly(anomaly).await?;
            plan.actions.extend(actions);
        }

        // Find matching rules in knowledge base
        let rules = self.knowledge.find_matching_rules().await?;
        for rule in rules {
            let action = Action::new(ActionType::Custom {
                name: rule.name.clone(),
                action: rule.action.clone(),
            });
            plan.actions.push(action);
        }

        // Sort actions by priority
        plan.actions.sort_by_key(|a| std::cmp::Reverse(a.priority));

        // Calculate expected benefit
        plan.expected_benefit = plan.actions.iter().map(|a| a.expected_impact).sum::<f64>()
            / plan.actions.len().max(1) as f64;

        if plan.actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(plan))
        }
    }

    /// Generate actions for violated goal
    async fn generate_actions_for_goal(
        &self,
        goal: &super::knowledge::Goal,
    ) -> WorkflowResult<Vec<Action>> {
        let mut actions = Vec::new();

        match goal.goal_type {
            super::knowledge::GoalType::Performance => {
                // For performance goals, consider scaling or optimization
                if goal.metric.contains("latency") || goal.metric.contains("throughput") {
                    actions.push(Action::new(ActionType::ScaleInstances { delta: 2 }));
                    actions.push(Action::new(ActionType::OptimizePattern { pattern_id: 12 }));
                }
            }
            super::knowledge::GoalType::Resource => {
                // For resource goals, adjust allocations
                actions.push(Action::new(ActionType::AdjustResources {
                    resource: goal.metric.clone(),
                    amount: goal.target * 0.1,
                }));
            }
            _ => {}
        }

        Ok(actions)
    }

    /// Generate actions for anomaly
    async fn generate_actions_for_anomaly(
        &self,
        anomaly: &super::analyze::Anomaly,
    ) -> WorkflowResult<Vec<Action>> {
        let mut actions = Vec::new();

        match anomaly.anomaly_type {
            super::analyze::AnomalyType::AboveThreshold => {
                actions.push(Action::new(ActionType::ScaleInstances { delta: 1 }));
            }
            super::analyze::AnomalyType::Spike => {
                actions.push(Action::new(ActionType::MigrateRuntime {
                    from: "W1".to_string(),
                    to: "R1".to_string(),
                }));
            }
            _ => {}
        }

        Ok(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::analyze::HealthStatus;
    use crate::autonomic::knowledge::{Goal, GoalType};

    #[tokio::test]
    async fn test_planner() {
        let kb = Arc::new(KnowledgeBase::new());
        let planner = Planner::new(kb.clone());

        let mut analysis = Analysis::new();
        analysis.adaptation_needed = true;

        let goal = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );
        analysis.violated_goals.push(goal);

        let plan = planner.plan(&analysis).await.unwrap();
        assert!(plan.is_some());

        let plan = plan.unwrap();
        assert!(!plan.actions.is_empty());
    }

    #[tokio::test]
    async fn test_no_plan_needed() {
        let kb = Arc::new(KnowledgeBase::new());
        let planner = Planner::new(kb);

        let analysis = Analysis::new(); // Healthy state

        let plan = planner.plan(&analysis).await.unwrap();
        assert!(plan.is_none());
    }
}
