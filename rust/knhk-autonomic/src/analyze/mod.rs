//! # Analyze Component - Pattern Recognition & Root Cause Analysis
//!
//! **Covenant 3**: Feedback loops run at machine speed
//!
//! The Analyze component examines metrics and observations to understand what's
//! happening. It uses analysis rules (SPARQL queries) to match patterns and
//! identify root causes.
//!
//! ## Responsibilities
//!
//! - Match observations to analysis rules
//! - Identify root causes using SPARQL pattern matching
//! - Assess problem severity and confidence
//! - Recommend actions based on learned patterns
//! - Run at ≤8 ticks for hot path rule matching
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomic::analyze::AnalysisComponent;
//! use knhk_autonomic::types::{Observation, RuleType};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut analyzer = AnalysisComponent::new();
//!
//! // Register analysis rule
//! analyzer.register_rule(
//!     "High Error Rate",
//!     RuleType::HighErrorRate,
//!     "?metric mape:metricName 'Error Count' ; mape:currentValue ?val . FILTER(?val > 5)"
//! ).await?;
//!
//! // Analyze observations
//! let observations = vec![/* ... */];
//! let analysis = analyzer.analyze(&observations).await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{AutonomicError, Result};
use crate::types::{Analysis, Metric, Observation, RuleType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};
use uuid::Uuid;

/// Analysis rule for pattern matching
#[derive(Debug, Clone)]
pub struct AnalysisRule {
    /// Rule identifier
    pub id: Uuid,

    /// Rule name
    pub name: String,

    /// Rule type
    pub rule_type: RuleType,

    /// SPARQL WHERE clause for pattern matching
    pub condition: String,

    /// Priority (higher = checked first)
    pub priority: i32,
}

/// Analysis component for understanding problems
#[derive(Debug, Clone)]
pub struct AnalysisComponent {
    /// Registered analysis rules
    rules: Arc<RwLock<HashMap<String, AnalysisRule>>>,

    /// Cache of recent analyses
    analysis_cache: Arc<RwLock<HashMap<Uuid, Analysis>>>,
}

impl Default for AnalysisComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisComponent {
    /// Create a new analysis component
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an analysis rule
    #[instrument(skip(self))]
    pub async fn register_rule(
        &mut self,
        name: impl Into<String>,
        rule_type: RuleType,
        condition: impl Into<String>,
    ) -> Result<Uuid> {
        let name = name.into();
        let rule = AnalysisRule {
            id: Uuid::new_v4(),
            name: name.clone(),
            rule_type,
            condition: condition.into(),
            priority: 100,
        };

        let id = rule.id;
        let mut rules = self.rules.write().await;
        rules.insert(name, rule);

        debug!("Registered analysis rule: {}", id);
        Ok(id)
    }

    /// Analyze observations to identify problems
    #[instrument(skip(self, observations))]
    pub async fn analyze(
        &self,
        observations: &[Observation],
        metrics: &[Metric],
    ) -> Result<Vec<Analysis>> {
        let mut analyses = Vec::new();

        if observations.is_empty() {
            return Ok(analyses);
        }

        let rules = self.rules.read().await;

        // Sort rules by priority
        let mut sorted_rules: Vec<_> = rules.values().collect();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in sorted_rules {
            if let Some(analysis) = self.match_rule(rule, observations, metrics).await? {
                analyses.push(analysis);
            }
        }

        // Cache analyses
        let mut cache = self.analysis_cache.write().await;
        for analysis in &analyses {
            cache.insert(analysis.id, analysis.clone());
        }

        debug!("Generated {} analyses", analyses.len());
        Ok(analyses)
    }

    /// Match a rule against observations and metrics
    #[instrument(skip(self, rule, observations, metrics))]
    async fn match_rule(
        &self,
        rule: &AnalysisRule,
        observations: &[Observation],
        metrics: &[Metric],
    ) -> Result<Option<Analysis>> {
        // Simplified rule matching (in production, this would use SPARQL)
        // For now, we use heuristic matching based on rule type

        let matched = match rule.rule_type {
            RuleType::HighErrorRate => metrics
                .iter()
                .any(|m| m.name.to_lowercase().contains("error") && m.is_anomalous),
            RuleType::PerformanceDegradation => metrics
                .iter()
                .any(|m| m.metric_type == crate::types::MetricType::Performance && m.is_anomalous),
            RuleType::ResourceStarvation => metrics
                .iter()
                .any(|m| m.metric_type == crate::types::MetricType::Resource && m.is_anomalous),
            _ => false,
        };

        if !matched {
            return Ok(None);
        }

        // Generate analysis
        let affected_elements: Vec<String> = observations
            .iter()
            .map(|o| o.observed_element.clone())
            .collect();

        let problem = format!("{:?} detected", rule.rule_type);
        let root_cause = self.identify_root_cause(rule.rule_type, metrics);
        let confidence = self.calculate_confidence(observations, metrics);

        Ok(Some(Analysis {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            problem,
            root_cause,
            affected_elements,
            recommended_actions: Vec::new(), // Filled in by planner
            confidence,
            rule_type: rule.rule_type,
        }))
    }

    /// Identify root cause based on rule type and metrics
    fn identify_root_cause(&self, rule_type: RuleType, metrics: &[Metric]) -> String {
        match rule_type {
            RuleType::HighErrorRate => {
                "Error rate exceeds threshold, likely due to transient failures or resource issues"
                    .to_string()
            }
            RuleType::PerformanceDegradation => {
                if metrics
                    .iter()
                    .any(|m| m.name.to_lowercase().contains("cpu") && m.is_anomalous)
                {
                    "High CPU usage causing performance degradation".to_string()
                } else if metrics
                    .iter()
                    .any(|m| m.name.to_lowercase().contains("memory") && m.is_anomalous)
                {
                    "Memory pressure causing performance degradation".to_string()
                } else {
                    "Performance degradation detected, analyzing root cause".to_string()
                }
            }
            RuleType::ResourceStarvation => {
                "Resource exhaustion detected, may need scaling or optimization".to_string()
            }
            _ => format!("{:?} root cause analysis", rule_type),
        }
    }

    /// Calculate confidence in analysis
    fn calculate_confidence(&self, observations: &[Observation], metrics: &[Metric]) -> f64 {
        let anomaly_count = metrics.iter().filter(|m| m.is_anomalous).count();
        let observation_severity: i32 = observations
            .iter()
            .map(|o| match o.severity {
                crate::types::Severity::Critical => 4,
                crate::types::Severity::High => 3,
                crate::types::Severity::Medium => 2,
                crate::types::Severity::Low => 1,
            })
            .sum();

        // Simple confidence calculation
        let base_confidence = 0.5;
        let anomaly_boost = (anomaly_count as f64 * 0.1).min(0.3);
        let severity_boost = (observation_severity as f64 * 0.05).min(0.2);

        (base_confidence + anomaly_boost + severity_boost).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventType, MetricType, Severity, TrendDirection};

    #[tokio::test]
    async fn test_register_rule() {
        let mut analyzer = AnalysisComponent::new();

        let id = analyzer
            .register_rule(
                "test_rule",
                RuleType::HighErrorRate,
                "?metric mape:currentValue ?val . FILTER(?val > 5)",
            )
            .await
            .unwrap();

        assert!(id.as_u128() > 0);
    }

    #[tokio::test]
    async fn test_analyze_high_error_rate() {
        let mut analyzer = AnalysisComponent::new();

        analyzer
            .register_rule("High Error Rate", RuleType::HighErrorRate, "")
            .await
            .unwrap();

        let observations = vec![];
        let metrics = vec![Metric {
            id: Uuid::new_v4(),
            name: "Error Count".to_string(),
            metric_type: MetricType::Reliability,
            current_value: 10.0,
            expected_value: 1.0,
            unit: "count".to_string(),
            anomaly_threshold: 5.0,
            is_anomalous: true,
            trend: TrendDirection::Degrading,
            timestamp: Utc::now(),
        }];

        let analyses = analyzer.analyze(&observations, &metrics).await.unwrap();

        assert_eq!(analyses.len(), 1);
        assert_eq!(analyses[0].rule_type, RuleType::HighErrorRate);
    }
}
