// rust/knhk-workflow-engine/src/autonomic/analyze.rs
//! Analyze Component for MAPE-K Framework
//!
//! Analyzes collected metrics to detect anomalies, evaluate goals,
//! and identify adaptation opportunities.

use super::knowledge::{Goal, KnowledgeBase};
use crate::error::WorkflowResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Anomaly detected by analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Metric involved
    pub metric: String,
    /// Current value
    pub current_value: f64,
    /// Expected value
    pub expected_value: f64,
    /// Severity (0.0-1.0)
    pub severity: f64,
    /// Timestamp when detected
    pub timestamp_ms: u64,
}

/// Anomaly type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Value above threshold
    AboveThreshold,
    /// Value below threshold
    BelowThreshold,
    /// Sudden spike in value
    Spike,
    /// Sudden drop in value
    Drop,
    /// Trending upward
    TrendUp,
    /// Trending downward
    TrendDown,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All goals satisfied
    Healthy,
    /// Minor goal violations
    Degraded,
    /// Major goal violations
    Unhealthy,
    /// Critical failures
    Critical,
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    /// Health status
    pub health: HealthStatus,
    /// Violated goals
    pub violated_goals: Vec<Goal>,
    /// Detected anomalies
    pub anomalies: Vec<Anomaly>,
    /// Adaptation needed
    pub adaptation_needed: bool,
    /// Analysis timestamp
    pub timestamp_ms: u64,
}

impl Analysis {
    pub fn new() -> Self {
        Self {
            health: HealthStatus::Healthy,
            violated_goals: Vec::new(),
            anomalies: Vec::new(),
            adaptation_needed: false,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

impl Default for Analysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzer component
pub struct Analyzer {
    /// Knowledge base
    knowledge: Arc<KnowledgeBase>,
    /// Anomaly detection threshold
    anomaly_threshold: f64,
}

impl Analyzer {
    /// Create new analyzer
    pub fn new(knowledge: Arc<KnowledgeBase>) -> Self {
        Self {
            knowledge,
            anomaly_threshold: 0.3, // 30% deviation triggers anomaly
        }
    }

    /// Analyze current state
    pub async fn analyze(&self) -> WorkflowResult<Analysis> {
        let mut analysis = Analysis::new();

        // Find violated goals
        analysis.violated_goals = self.knowledge.find_violated_goals().await;

        // Detect anomalies
        analysis.anomalies = self.detect_anomalies().await?;

        // Determine health status
        analysis.health = self.calculate_health(&analysis.violated_goals, &analysis.anomalies);

        // Check if adaptation is needed
        analysis.adaptation_needed = analysis.health != HealthStatus::Healthy
            || !analysis.anomalies.is_empty();

        Ok(analysis)
    }

    /// Detect anomalies in metrics
    async fn detect_anomalies(&self) -> WorkflowResult<Vec<Anomaly>> {
        let mut anomalies = Vec::new();
        let goals = self.knowledge.get_goals().await;
        let facts = self.knowledge.get_facts().await;

        for goal in goals {
            if let Some(fact) = facts.get(&goal.metric) {
                let distance = goal.distance(fact.value);

                // Check for threshold violations
                if fact.value > goal.target * (1.0 + self.anomaly_threshold) {
                    anomalies.push(Anomaly {
                        anomaly_type: AnomalyType::AboveThreshold,
                        metric: goal.metric.clone(),
                        current_value: fact.value,
                        expected_value: goal.target,
                        severity: distance.min(1.0),
                        timestamp_ms: fact.timestamp_ms,
                    });
                } else if fact.value < goal.target * (1.0 - self.anomaly_threshold) {
                    anomalies.push(Anomaly {
                        anomaly_type: AnomalyType::BelowThreshold,
                        metric: goal.metric.clone(),
                        current_value: fact.value,
                        expected_value: goal.target,
                        severity: distance.min(1.0),
                        timestamp_ms: fact.timestamp_ms,
                    });
                }

                // Check for trends
                let history = self.knowledge.get_history(&goal.metric, 10).await;
                if history.len() >= 3 {
                    if self.is_trending_up(&history) {
                        anomalies.push(Anomaly {
                            anomaly_type: AnomalyType::TrendUp,
                            metric: goal.metric.clone(),
                            current_value: fact.value,
                            expected_value: goal.target,
                            severity: 0.5,
                            timestamp_ms: fact.timestamp_ms,
                        });
                    } else if self.is_trending_down(&history) {
                        anomalies.push(Anomaly {
                            anomaly_type: AnomalyType::TrendDown,
                            metric: goal.metric.clone(),
                            current_value: fact.value,
                            expected_value: goal.target,
                            severity: 0.5,
                            timestamp_ms: fact.timestamp_ms,
                        });
                    }
                }
            }
        }

        Ok(anomalies)
    }

    /// Calculate overall health status
    fn calculate_health(&self, violated_goals: &[Goal], anomalies: &[Anomaly]) -> HealthStatus {
        if violated_goals.is_empty() && anomalies.is_empty() {
            return HealthStatus::Healthy;
        }

        // Calculate severity score
        let goal_score: f64 = violated_goals.iter().map(|g| g.priority as f64 / 100.0).sum();
        let anomaly_score: f64 = anomalies.iter().map(|a| a.severity).sum();

        let total_score = goal_score + anomaly_score;

        if total_score > 2.0 {
            HealthStatus::Critical
        } else if total_score > 1.0 {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Degraded
        }
    }

    /// Check if metric is trending up
    fn is_trending_up(&self, history: &[super::knowledge::Fact]) -> bool {
        if history.len() < 3 {
            return false;
        }

        let mut increases = 0;
        for i in 1..history.len() {
            if history[i].value > history[i - 1].value {
                increases += 1;
            }
        }

        increases >= history.len() * 2 / 3
    }

    /// Check if metric is trending down
    fn is_trending_down(&self, history: &[super::knowledge::Fact]) -> bool {
        if history.len() < 3 {
            return false;
        }

        let mut decreases = 0;
        for i in 1..history.len() {
            if history[i].value < history[i - 1].value {
                decreases += 1;
            }
        }

        decreases >= history.len() * 2 / 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autonomic::knowledge::{Fact, Goal, GoalType};

    #[tokio::test]
    async fn test_analyzer() {
        let kb = Arc::new(KnowledgeBase::new());
        let analyzer = Analyzer::new(kb.clone());

        // Add goal
        let goal = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );
        kb.add_goal(goal).await.unwrap();

        // Add fact (violates goal)
        let fact = Fact::new("avg_latency_ms".to_string(), 200.0, "monitor".to_string());
        kb.add_fact(fact).await.unwrap();

        // Analyze
        let analysis = analyzer.analyze().await.unwrap();

        assert_eq!(analysis.violated_goals.len(), 1);
        assert!(!analysis.anomalies.is_empty());
        assert_ne!(analysis.health, HealthStatus::Healthy);
        assert!(analysis.adaptation_needed);
    }

    #[tokio::test]
    async fn test_health_calculation() {
        let kb = Arc::new(KnowledgeBase::new());
        let analyzer = Analyzer::new(kb.clone());

        // Add goal
        let goal = Goal::new(
            "latency".to_string(),
            GoalType::Performance,
            "avg_latency_ms".to_string(),
            100.0,
        );
        kb.add_goal(goal).await.unwrap();

        // Add fact (healthy)
        let fact = Fact::new("avg_latency_ms".to_string(), 95.0, "monitor".to_string());
        kb.add_fact(fact).await.unwrap();

        let analysis = analyzer.analyze().await.unwrap();
        assert_eq!(analysis.health, HealthStatus::Healthy);
    }
}
