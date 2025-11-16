// rust/knhk-workflow-engine/src/autonomic/knowledge.rs
//! Knowledge Base for MAPE-K Framework
//!
//! Central repository for goals, rules, facts, and policies shared across
//! all MAPE components (Monitor, Analyze, Plan, Execute).
//!
//! **Architecture**:
//! - Goals: High-level objectives (e.g., "maintain 95% SLO")
//! - Rules: If-then adaptation rules
//! - Facts: Runtime observations and measurements
//! - Policies: Constraints on adaptations

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Knowledge identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KnowledgeId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl KnowledgeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for KnowledgeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Goal type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalType {
    /// Performance goal (e.g., latency < 100ms)
    Performance,
    /// Reliability goal (e.g., 99.9% uptime)
    Reliability,
    /// Resource utilization goal (e.g., CPU < 80%)
    Resource,
    /// Business goal (e.g., cost < $1000/month)
    Business,
}

/// Goal with measurable objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    /// Goal identifier
    pub id: KnowledgeId,
    /// Goal name
    pub name: String,
    /// Goal type
    pub goal_type: GoalType,
    /// Metric name to measure
    pub metric: String,
    /// Target value
    pub target: f64,
    /// Acceptable range (± tolerance)
    pub tolerance: f64,
    /// Priority (0-100, higher = more important)
    pub priority: u8,
    /// Whether goal is currently satisfied
    pub satisfied: bool,
}

impl Goal {
    pub fn new(name: String, goal_type: GoalType, metric: String, target: f64) -> Self {
        Self {
            id: KnowledgeId::new(),
            name,
            goal_type,
            metric,
            target,
            tolerance: target * 0.1, // Default 10% tolerance
            priority: 50,
            satisfied: false,
        }
    }

    /// Check if goal is satisfied by given value
    pub fn is_satisfied(&self, value: f64) -> bool {
        (value - self.target).abs() <= self.tolerance
    }

    /// Calculate distance from target (normalized)
    pub fn distance(&self, value: f64) -> f64 {
        ((value - self.target) / self.tolerance).abs()
    }
}

/// Adaptation rule (if-then)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Rule identifier
    pub id: KnowledgeId,
    /// Rule name
    pub name: String,
    /// Condition (metric name and threshold)
    pub condition: String,
    /// Action to take when condition is true
    pub action: String,
    /// Rule priority (higher = evaluated first)
    pub priority: u8,
    /// Whether rule is enabled
    pub enabled: bool,
}

impl Rule {
    pub fn new(name: String, condition: String, action: String) -> Self {
        Self {
            id: KnowledgeId::new(),
            name,
            condition,
            action,
            priority: 50,
            enabled: true,
        }
    }

    /// Evaluate rule condition against facts
    pub fn evaluate(&self, facts: &HashMap<String, Fact>) -> WorkflowResult<bool> {
        // Parse condition: "metric_name operator value"
        // Example: "cpu_usage > 0.8"
        let parts: Vec<&str> = self.condition.split_whitespace().collect();
        if parts.len() != 3 {
            return Ok(false);
        }

        let metric_name = parts[0];
        let operator = parts[1];
        let threshold: f64 = parts[2].parse().map_err(|e| {
            WorkflowError::Internal(format!("Cannot parse threshold: {}", e))
        })?;

        // Get fact value
        if let Some(fact) = facts.get(metric_name) {
            match operator {
                ">" => Ok(fact.value > threshold),
                "<" => Ok(fact.value < threshold),
                "==" => Ok((fact.value - threshold).abs() < 0.001),
                ">=" => Ok(fact.value >= threshold),
                "<=" => Ok(fact.value <= threshold),
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }
}

/// Runtime fact (observation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Fact identifier
    pub id: KnowledgeId,
    /// Metric name
    pub metric: String,
    /// Metric value
    pub value: f64,
    /// Timestamp (ms since epoch)
    pub timestamp_ms: u64,
    /// Source (where fact came from)
    pub source: String,
}

impl Fact {
    pub fn new(metric: String, value: f64, source: String) -> Self {
        Self {
            id: KnowledgeId::new(),
            metric,
            value,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            source,
        }
    }
}

/// Policy constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy identifier
    pub id: KnowledgeId,
    /// Policy name
    pub name: String,
    /// Constraint expression
    pub constraint: String,
    /// Whether policy is enforced
    pub enforced: bool,
}

impl Policy {
    pub fn new(name: String, constraint: String) -> Self {
        Self {
            id: KnowledgeId::new(),
            name,
            constraint,
            enforced: true,
        }
    }
}

/// Knowledge base (shared across MAPE components)
pub struct KnowledgeBase {
    /// Goals
    goals: Arc<RwLock<HashMap<KnowledgeId, Goal>>>,
    /// Rules
    rules: Arc<RwLock<HashMap<KnowledgeId, Rule>>>,
    /// Facts (current observations)
    facts: Arc<RwLock<HashMap<String, Fact>>>,
    /// Policies
    policies: Arc<RwLock<HashMap<KnowledgeId, Policy>>>,
    /// Historical facts (for trend analysis)
    history: Arc<RwLock<Vec<Fact>>>,
    /// Maximum history size
    max_history: usize,
}

impl KnowledgeBase {
    /// Create new knowledge base
    pub fn new() -> Self {
        Self {
            goals: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(HashMap::new())),
            facts: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 10000,
        }
    }

    /// Add goal
    pub async fn add_goal(&self, goal: Goal) -> WorkflowResult<KnowledgeId> {
        let id = goal.id;
        let mut goals = self.goals.write().await;
        goals.insert(id, goal);
        Ok(id)
    }

    /// Add rule
    pub async fn add_rule(&self, rule: Rule) -> WorkflowResult<KnowledgeId> {
        let id = rule.id;
        let mut rules = self.rules.write().await;
        rules.insert(id, rule);
        Ok(id)
    }

    /// Add fact (current observation)
    pub async fn add_fact(&self, fact: Fact) -> WorkflowResult<()> {
        let metric = fact.metric.clone();

        // Update current facts
        let mut facts = self.facts.write().await;
        facts.insert(metric, fact.clone());

        // Add to history
        let mut history = self.history.write().await;
        history.push(fact);

        // Trim history if needed
        if history.len() > self.max_history {
            history.drain(0..history.len() - self.max_history);
        }

        Ok(())
    }

    /// Add policy
    pub async fn add_policy(&self, policy: Policy) -> WorkflowResult<KnowledgeId> {
        let id = policy.id;
        let mut policies = self.policies.write().await;
        policies.insert(id, policy);
        Ok(id)
    }

    /// Get all goals
    pub async fn get_goals(&self) -> Vec<Goal> {
        let goals = self.goals.read().await;
        goals.values().cloned().collect()
    }

    /// Get active rules (sorted by priority)
    pub async fn get_active_rules(&self) -> Vec<Rule> {
        let rules = self.rules.read().await;
        let mut active: Vec<Rule> = rules
            .values()
            .filter(|r| r.enabled)
            .cloned()
            .collect();
        active.sort_by_key(|r| std::cmp::Reverse(r.priority));
        active
    }

    /// Get current facts
    pub async fn get_facts(&self) -> HashMap<String, Fact> {
        let facts = self.facts.read().await;
        facts.clone()
    }

    /// Get fact by metric name
    pub async fn get_fact(&self, metric: &str) -> Option<Fact> {
        let facts = self.facts.read().await;
        facts.get(metric).cloned()
    }

    /// Get fact history for metric
    pub async fn get_history(&self, metric: &str, limit: usize) -> Vec<Fact> {
        let history = self.history.read().await;
        history
            .iter()
            .filter(|f| f.metric == metric)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Evaluate all goals
    pub async fn evaluate_goals(&self) -> HashMap<KnowledgeId, bool> {
        let goals = self.goals.read().await;
        let facts = self.facts.read().await;

        let mut results = HashMap::new();

        for (id, goal) in goals.iter() {
            if let Some(fact) = facts.get(&goal.metric) {
                let satisfied = goal.is_satisfied(fact.value);
                results.insert(*id, satisfied);
            }
        }

        results
    }

    /// Find violated goals
    pub async fn find_violated_goals(&self) -> Vec<Goal> {
        let goals = self.goals.read().await;
        let facts = self.facts.read().await;

        goals
            .values()
            .filter_map(|goal| {
                if let Some(fact) = facts.get(&goal.metric) {
                    if !goal.is_satisfied(fact.value) {
                        return Some(goal.clone());
                    }
                }
                None
            })
            .collect()
    }

    /// Find matching rules
    pub async fn find_matching_rules(&self) -> WorkflowResult<Vec<Rule>> {
        let rules = self.get_active_rules().await;
        let facts = self.get_facts().await;

        let mut matching = Vec::new();

        for rule in rules {
            if rule.evaluate(&facts)? {
                matching.push(rule);
            }
        }

        Ok(matching)
    }

    /// Get policies
    pub async fn get_policies(&self) -> Vec<Policy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    /// Clear all knowledge
    pub async fn clear(&self) {
        let mut goals = self.goals.write().await;
        goals.clear();
        let mut rules = self.rules.write().await;
        rules.clear();
        let mut facts = self.facts.write().await;
        facts.clear();
        let mut policies = self.policies.write().await;
        policies.clear();
        let mut history = self.history.write().await;
        history.clear();
    }

    /// Get knowledge statistics
    pub async fn get_stats(&self) -> KnowledgeStats {
        let goals = self.goals.read().await;
        let rules = self.rules.read().await;
        let facts = self.facts.read().await;
        let policies = self.policies.read().await;
        let history = self.history.read().await;

        KnowledgeStats {
            total_goals: goals.len(),
            total_rules: rules.len(),
            active_rules: rules.values().filter(|r| r.enabled).count(),
            total_facts: facts.len(),
            history_size: history.len(),
            total_policies: policies.len(),
            enforced_policies: policies.values().filter(|p| p.enforced).count(),
        }
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

/// Knowledge base statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStats {
    pub total_goals: usize,
    pub total_rules: usize,
    pub active_rules: usize,
    pub total_facts: usize,
    pub history_size: usize,
    pub total_policies: usize,
    pub enforced_policies: usize,
}

/// Knowledge trait (for custom knowledge stores)
pub trait Knowledge: Send + Sync {
    /// Query knowledge base
    fn query(&self, query: &str) -> WorkflowResult<Vec<Fact>>;

    /// Update knowledge
    fn update(&mut self, fact: Fact) -> WorkflowResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_goal_satisfaction() {
        let goal = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );

        assert!(goal.is_satisfied(100.0));
        assert!(goal.is_satisfied(105.0)); // Within 10% tolerance
        assert!(!goal.is_satisfied(150.0)); // Outside tolerance
    }

    #[tokio::test]
    async fn test_rule_evaluation() {
        let rule = Rule::new(
            "scale_up".to_string(),
            "cpu_usage > 0.8".to_string(),
            "add_instance".to_string(),
        );

        let mut facts = HashMap::new();
        facts.insert(
            "cpu_usage".to_string(),
            Fact::new("cpu_usage".to_string(), 0.9, "monitor".to_string()),
        );

        assert!(rule.evaluate(&facts).unwrap());

        facts.insert(
            "cpu_usage".to_string(),
            Fact::new("cpu_usage".to_string(), 0.5, "monitor".to_string()),
        );

        assert!(!rule.evaluate(&facts).unwrap());
    }

    #[tokio::test]
    async fn test_knowledge_base() {
        let kb = KnowledgeBase::new();

        // Add goal
        let goal = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );
        kb.add_goal(goal).await.unwrap();

        // Add fact
        let fact = Fact::new("avg_latency_ms".to_string(), 95.0, "monitor".to_string());
        kb.add_fact(fact).await.unwrap();

        // Evaluate goals
        let results = kb.evaluate_goals().await;
        assert_eq!(results.len(), 1);
        assert!(results.values().next().unwrap());
    }

    #[tokio::test]
    async fn test_violated_goals() {
        let kb = KnowledgeBase::new();

        // Add goals
        let goal1 = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );
        kb.add_goal(goal1).await.unwrap();

        let goal2 = Goal::new(
            "cpu".to_string(),
            GoalType::Resource,
            "cpu_usage".to_string(),
            0.7,
        );
        kb.add_goal(goal2).await.unwrap();

        // Add facts (one violates goal)
        kb.add_fact(Fact::new(
            "avg_latency_ms".to_string(),
            95.0,
            "monitor".to_string(),
        ))
        .await
        .unwrap();

        kb.add_fact(Fact::new(
            "cpu_usage".to_string(),
            0.9,
            "monitor".to_string(),
        ))
        .await
        .unwrap();

        let violated = kb.find_violated_goals().await;
        assert_eq!(violated.len(), 1);
        assert_eq!(violated[0].metric, "cpu_usage");
    }
}
