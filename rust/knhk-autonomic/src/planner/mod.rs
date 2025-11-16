//! # Plan Component - Decision Making & Action Planning
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Plan component decides what actions to take based on analysis results.
//! It evaluates policies (SPARQL queries), selects actions from the knowledge base,
//! and creates execution plans.
//!
//! ## Responsibilities
//!
//! - Evaluate autonomic policies against analysis results
//! - Select actions based on historical success rates
//! - Sequence actions logically (dependencies, priorities)
//! - Assess risk of actions
//! - Generate execution plans
//! - Run at ≤8 ticks for hot path policy evaluation
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::planner::PlanningComponent;
//! use knhk_autonomic::types::{Policy, RuleType};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut planner = PlanningComponent::new();
//!
//! // Register policy
//! let action_ids = vec![/* action UUIDs */];
//! planner.register_policy(
//!     "Retry on Failure",
//!     "?problem mape:ruleType mape:HighErrorRate",
//!     action_ids,
//!     100, // priority
//! ).await?;
//!
//! // Create plan from analysis
//! let analysis = /* ... */;
//! let plan = planner.create_plan(&analysis).await?;
//! # Ok(())
//! # }
//! ```

use crate::types::{Analysis, Policy, Plan, Action, RiskLevel};
use crate::error::{AutonomicError, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, debug};
use uuid::Uuid;

/// Planning component for deciding actions
#[derive(Debug, Clone)]
pub struct PlanningComponent {
    /// Registered policies
    policies: Arc<RwLock<HashMap<String, Policy>>>,

    /// Available actions
    actions: Arc<RwLock<HashMap<Uuid, Action>>>,

    /// Policy evaluation cache
    plan_cache: Arc<RwLock<HashMap<Uuid, Plan>>>,
}

impl Default for PlanningComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanningComponent {
    /// Create a new planning component
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            actions: Arc::new(RwLock::new(HashMap::new())),
            plan_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an autonomic policy
    #[instrument(skip(self))]
    pub async fn register_policy(
        &mut self,
        name: impl Into<String>,
        trigger: impl Into<String>,
        actions: Vec<Uuid>,
        priority: i32,
    ) -> Result<Uuid> {
        let name = name.into();
        let policy = Policy {
            id: Uuid::new_v4(),
            name: name.clone(),
            trigger: trigger.into(),
            actions,
            priority,
            condition: None,
        };

        let id = policy.id;
        let mut policies = self.policies.write().await;
        policies.insert(name, policy);

        debug!("Registered policy: {}", id);
        Ok(id)
    }

    /// Register an action
    #[instrument(skip(self))]
    pub async fn register_action(&mut self, action: Action) -> Result<Uuid> {
        let id = action.id;
        let mut actions = self.actions.write().await;
        actions.insert(id, action);

        debug!("Registered action: {}", id);
        Ok(id)
    }

    /// Create execution plan from analysis
    #[instrument(skip(self, analysis))]
    pub async fn create_plan(
        &self,
        analysis: &Analysis,
        success_rates: &HashMap<Uuid, f64>,
    ) -> Result<Option<Plan>> {
        // Find policies that match this analysis
        let matching_policies = self.find_matching_policies(analysis).await?;

        if matching_policies.is_empty() {
            debug!("No matching policies for analysis {}", analysis.id);
            return Ok(None);
        }

        // Select best actions from matching policies
        let selected_actions = self.select_actions(&matching_policies, success_rates).await?;

        if selected_actions.is_empty() {
            debug!("No suitable actions found for analysis {}", analysis.id);
            return Ok(None);
        }

        // Create plan
        let plan = Plan {
            id: Uuid::new_v4(),
            actions: selected_actions,
            rationale: format!(
                "Responding to {} with {} policy",
                analysis.problem,
                matching_policies[0].name
            ),
            expected_outcome: "Problem resolution and metric normalization".to_string(),
            created_at: Utc::now(),
        };

        // Cache plan
        let mut cache = self.plan_cache.write().await;
        cache.insert(plan.id, plan.clone());

        debug!("Created plan {} with {} actions", plan.id, plan.actions.len());
        Ok(Some(plan))
    }

    /// Find policies that match the analysis
    async fn find_matching_policies(&self, analysis: &Analysis) -> Result<Vec<Policy>> {
        let policies = self.policies.read().await;

        // Sort by priority (highest first)
        let mut matching: Vec<_> = policies
            .values()
            .filter(|p| self.policy_matches(p, analysis))
            .cloned()
            .collect();

        matching.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(matching)
    }

    /// Check if policy matches analysis
    fn policy_matches(&self, policy: &Policy, analysis: &Analysis) -> bool {
        // Simplified matching (in production, this would use SPARQL)
        // Match based on rule type mentioned in policy trigger

        let trigger_lower = policy.trigger.to_lowercase();
        let rule_type_str = format!("{:?}", analysis.rule_type).to_lowercase();

        trigger_lower.contains(&rule_type_str)
            || trigger_lower.contains("higherrorrate") && matches!(analysis.rule_type, crate::types::RuleType::HighErrorRate)
            || trigger_lower.contains("performancedegradation") && matches!(analysis.rule_type, crate::types::RuleType::PerformanceDegradation)
    }

    /// Select actions from policies based on success rates
    async fn select_actions(
        &self,
        policies: &[Policy],
        success_rates: &HashMap<Uuid, f64>,
    ) -> Result<Vec<Uuid>> {
        let actions_store = self.actions.read().await;
        let mut selected = Vec::new();

        for policy in policies {
            for action_id in &policy.actions {
                if let Some(action) = actions_store.get(action_id) {
                    // Check risk level and success rate
                    let success_rate = success_rates.get(action_id).copied().unwrap_or(0.5);

                    // Only select actions with good success rates or low risk
                    if success_rate > 0.7 || action.risk_level == RiskLevel::LowRisk {
                        selected.push(*action_id);
                    }
                }
            }

            // Limit to top policy's actions for now
            if !selected.is_empty() {
                break;
            }
        }

        Ok(selected)
    }

    /// Get action details
    pub async fn get_action(&self, action_id: &Uuid) -> Result<Option<Action>> {
        let actions = self.actions.read().await;
        Ok(actions.get(action_id).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ActionType, RuleType};

    #[tokio::test]
    async fn test_register_policy_and_action() {
        let mut planner = PlanningComponent::new();

        // Register action
        let action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Heal,
            description: "Retry operation".to_string(),
            target: "payment_task".to_string(),
            implementation: "retry_handler".to_string(),
            estimated_impact: "70% recovery".to_string(),
            risk_level: RiskLevel::LowRisk,
        };

        let action_id = action.id;
        planner.register_action(action).await.unwrap();

        // Register policy
        let policy_id = planner.register_policy(
            "Retry on Failure",
            "HighErrorRate",
            vec![action_id],
            100,
        ).await.unwrap();

        assert!(policy_id.as_u128() > 0);
    }

    #[tokio::test]
    async fn test_create_plan() {
        let mut planner = PlanningComponent::new();

        // Setup action and policy
        let action = Action {
            id: Uuid::new_v4(),
            action_type: ActionType::Heal,
            description: "Retry operation".to_string(),
            target: "payment_task".to_string(),
            implementation: "retry_handler".to_string(),
            estimated_impact: "70% recovery".to_string(),
            risk_level: RiskLevel::LowRisk,
        };

        let action_id = action.id;
        planner.register_action(action).await.unwrap();

        planner.register_policy(
            "Retry on Failure",
            "HighErrorRate",
            vec![action_id],
            100,
        ).await.unwrap();

        // Create analysis
        let analysis = Analysis {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            problem: "High error rate".to_string(),
            root_cause: "Transient failures".to_string(),
            affected_elements: vec!["payment_task".to_string()],
            recommended_actions: vec![],
            confidence: 0.9,
            rule_type: RuleType::HighErrorRate,
        };

        // Create plan
        let mut success_rates = HashMap::new();
        success_rates.insert(action_id, 0.85);

        let plan = planner.create_plan(&analysis, &success_rates).await.unwrap();

        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.actions.len(), 1);
        assert_eq!(plan.actions[0], action_id);
    }
}
