//! Data retention policies for compliance

// Retention manager implementation
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

/// Retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Policy name
    pub name: String,
    /// Resource type (Case, WorkflowSpec, etc.)
    pub resource_type: String,
    /// Retention period (days)
    pub retention_days: i64,
    /// Legal hold enabled
    pub legal_hold: bool,
    /// GDPR right-to-be-forgotten enabled
    pub gdpr_rtbf: bool,
}

/// Retention manager
pub struct RetentionManager {
    policies: HashMap<String, RetentionPolicy>,
    /// Legal hold resources (cannot be deleted)
    legal_holds: std::sync::Arc<std::sync::Mutex<HashMap<String, DateTime<Utc>>>>,
}

impl RetentionManager {
    /// Create a new retention manager
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            legal_holds: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Add a retention policy
    pub fn add_policy(&mut self, policy: RetentionPolicy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Check if resource should be retained
    pub fn should_retain(&self, resource_type: &str, created_at: DateTime<Utc>) -> bool {
        // Check legal hold
        if let Ok(legal_holds) = self.legal_holds.lock() {
            if legal_holds.contains_key(resource_type) {
                return true;
            }
        } else {
            warn!("Failed to acquire legal_holds lock in should_retain");
        }

        // Check retention policy
        if let Some(policy) = self
            .policies
            .values()
            .find(|p| p.resource_type == resource_type)
        {
            let retention_until = created_at + Duration::days(policy.retention_days);
            Utc::now() < retention_until
        } else {
            true // Default: retain if no policy
        }
    }

    /// Place resource under legal hold
    pub fn place_legal_hold(&self, resource_type: String) {
        if let Ok(mut legal_holds) = self.legal_holds.lock() {
            legal_holds.insert(resource_type, Utc::now());
        } else {
            warn!("Failed to acquire legal_holds lock in place_legal_hold");
        }
    }

    /// Release legal hold
    pub fn release_legal_hold(&self, resource_type: &str) {
        if let Ok(mut legal_holds) = self.legal_holds.lock() {
            legal_holds.remove(resource_type);
        } else {
            warn!("Failed to acquire legal_holds lock in release_legal_hold");
        }
    }

    /// Check if resource is under legal hold
    pub fn is_legal_hold(&self, resource_type: &str) -> bool {
        self.legal_holds
            .lock()
            .map(|legal_holds| legal_holds.contains_key(resource_type))
            .unwrap_or(false)
    }

    /// Get resources eligible for deletion
    pub fn get_eligible_for_deletion(&self, resources: &[(String, DateTime<Utc>)]) -> Vec<String> {
        resources
            .iter()
            .filter(|(resource_type, created_at)| !self.should_retain(resource_type, *created_at))
            .map(|(resource_type, _)| resource_type.clone())
            .collect()
    }
}

impl Default for RetentionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_manager() {
        let mut manager = RetentionManager::new();
        let policy = RetentionPolicy {
            name: "case-policy".to_string(),
            resource_type: "Case".to_string(),
            retention_days: 90,
            legal_hold: false,
            gdpr_rtbf: true,
        };
        manager.add_policy(policy);

        let created_at = Utc::now() - Duration::days(100);
        assert!(!manager.should_retain("Case", created_at));
    }
}
