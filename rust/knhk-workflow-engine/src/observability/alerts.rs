#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Alerting for workflow engine

use crate::error::WorkflowResult;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Alert level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Info alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
}

/// Alert rule
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Alert level
    pub level: AlertLevel,
    /// Condition function name (for serialization)
    pub condition_name: String,
    /// Cooldown period (prevent alert spam)
    pub cooldown: Duration,
    /// Condition check function (not serialized)
    #[serde(skip_serializing, skip_deserializing)]
    pub condition: Box<dyn Fn() -> bool + Send + Sync>,
}

/// Alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: uuid::Uuid,
    /// Alert timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Alert level
    pub level: AlertLevel,
    /// Alert message
    pub message: String,
    /// Rule name that triggered
    pub rule_name: String,
}

/// Alert manager
pub struct AlertManager {
    rules: Arc<Mutex<Vec<AlertRule>>>,
    alerts: Arc<Mutex<Vec<Alert>>>,
    last_fire_time: Arc<Mutex<std::collections::HashMap<String, u64>>>, // Store as timestamp
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: Arc::new(Mutex::new(Vec::new())),
            alerts: Arc::new(Mutex::new(Vec::new())),
            last_fire_time: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// Add an alert rule
    pub fn add_rule(&self, rule: AlertRule) {
        let mut rules = self.rules.lock().unwrap();
        rules.push(rule);
    }

    /// Check all rules and fire alerts if needed
    pub fn check_rules(&self) -> WorkflowResult<Vec<Alert>> {
        let rules = self.rules.lock().unwrap();
        let mut fired_alerts = Vec::new();
        let mut last_fire = self.last_fire_time.lock().unwrap();
        let mut alerts = self.alerts.lock().unwrap();

        for rule in rules.iter() {
            // Check cooldown
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if let Some(last_time) = last_fire.get(&rule.name) {
                if now_ms.saturating_sub(*last_time) < rule.cooldown.as_secs() {
                    continue;
                }
            }

            // Check condition
            if (rule.condition)() {
                let alert = Alert {
                    id: uuid::Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    level: rule.level,
                    message: format!("Alert rule '{}' triggered", rule.name),
                    rule_name: rule.name.clone(),
                };

                fired_alerts.push(alert.clone());
                alerts.push(alert);
                last_fire.insert(rule.name.clone(), now_ms);

                // Keep only last 1000 alerts
                if alerts.len() > 1000 {
                    alerts.remove(0);
                }
            }
        }

        Ok(fired_alerts)
    }

    /// Get recent alerts
    pub fn get_alerts(&self, limit: Option<usize>) -> Vec<Alert> {
        let alerts = self.alerts.lock().unwrap();
        let mut result: Vec<Alert> = alerts.iter().cloned().collect();
        result.reverse(); // Most recent first

        if let Some(limit) = limit {
            result.truncate(limit);
        }

        result
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_manager() {
        let manager = AlertManager::new();

        let rule = AlertRule {
            name: "test_rule".to_string(),
            level: AlertLevel::Warning,
            condition_name: "always_true".to_string(),
            condition: Box::new(|| true),
            cooldown: Duration::from_secs(60),
        };

        manager.add_rule(rule);
        let alerts = manager.check_rules().unwrap();
        assert_eq!(alerts.len(), 1);
    }
}
