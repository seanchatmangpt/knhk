//! Validator Set Management
//!
//! Byzantine node identification, reputation tracking, and automatic rotation
//! Maintains dynamic validator set with reputation-based selection

use crate::{ConsensusError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Validator reputation metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ValidatorMetrics {
    /// Messages successfully validated
    pub valid_messages: u64,
    /// Messages with invalid signatures
    pub invalid_signatures: u64,
    /// Byzantine behaviors detected
    pub byzantine_behaviors: u64,
    /// Uptime percentage (0-100)
    pub uptime: u8,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
}

impl ValidatorMetrics {
    /// Compute reputation score (0.0-1.0)
    pub fn reputation_score(&self) -> f64 {
        let total = self.valid_messages + self.invalid_signatures + self.byzantine_behaviors;
        if total == 0 {
            0.5 // Neutral score for new validators
        } else {
            let valid_ratio = self.valid_messages as f64 / total as f64;
            let uptime_factor = (self.uptime as f64) / 100.0;
            (valid_ratio * 0.7 + uptime_factor * 0.3).clamp(0.0, 1.0)
        }
    }

    /// Is validator healthy
    pub fn is_healthy(&self) -> bool {
        self.reputation_score() >= 0.7
            && self.uptime >= 80
            && self.byzantine_behaviors == 0
    }
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Node ID
    pub node_id: String,
    /// Public key for signatures
    pub public_key: Vec<u8>,
    /// Performance metrics
    pub metrics: ValidatorMetrics,
    /// Is currently active
    pub is_active: bool,
    /// Joined at timestamp
    pub joined_timestamp_ms: u64,
    /// Last activity timestamp
    pub last_activity_ms: u64,
}

impl ValidatorInfo {
    /// Update activity timestamp
    pub fn touch(&mut self) {
        self.last_activity_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
}

/// Validator set manager
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    /// Active validators
    validators: Arc<DashMap<String, ValidatorInfo>>,
    /// Maximum validators
    max_validators: usize,
    /// Minimum required validators
    min_validators: usize,
    /// Validator inactivity timeout (ms)
    inactivity_timeout_ms: u64,
}

impl ValidatorSet {
    /// Create new validator set
    pub fn new(max_validators: usize, min_validators: usize) -> Result<Self> {
        if min_validators > max_validators || min_validators < 3 {
            return Err(ConsensusError::InvalidValidatorSet(
                "Invalid validator set parameters".to_string(),
            ));
        }

        Ok(ValidatorSet {
            validators: Arc::new(DashMap::new()),
            max_validators,
            min_validators,
            inactivity_timeout_ms: 300000, // 5 minutes
        })
    }

    /// Add validator
    pub fn add_validator(&self, validator: ValidatorInfo) -> Result<()> {
        if self.validators.len() >= self.max_validators {
            return Err(ConsensusError::InvalidValidatorSet(
                "Validator set is full".to_string(),
            ));
        }

        if validator.node_id.is_empty() {
            return Err(ConsensusError::InvalidValidatorSet(
                "Validator node_id cannot be empty".to_string(),
            ));
        }

        if validator.public_key.is_empty() {
            return Err(ConsensusError::InvalidValidatorSet(
                "Validator public_key cannot be empty".to_string(),
            ));
        }

        self.validators
            .insert(validator.node_id.clone(), validator.clone());

        info!(
            validator_id = %validator.node_id,
            total = self.validators.len(),
            "Validator added"
        );

        Ok(())
    }

    /// Remove validator
    pub fn remove_validator(&self, node_id: &str) -> Result<()> {
        if self.validators.len() <= self.min_validators {
            return Err(ConsensusError::InvalidValidatorSet(
                "Cannot remove validator below minimum".to_string(),
            ));
        }

        self.validators.remove(node_id);

        warn!(
            validator_id = %node_id,
            remaining = self.validators.len(),
            "Validator removed"
        );

        Ok(())
    }

    /// Get validator
    pub fn get_validator(&self, node_id: &str) -> Option<ValidatorInfo> {
        self.validators.get(node_id).map(|v| v.clone())
    }

    /// Get all validators
    pub fn get_all_validators(&self) -> Vec<ValidatorInfo> {
        self.validators
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get active validators
    pub fn get_active_validators(&self) -> Vec<ValidatorInfo> {
        self.validators
            .iter()
            .filter(|entry| entry.value().is_active)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Mark validator as Byzantine
    pub fn mark_byzantine(&self, node_id: &str) -> Result<()> {
        if let Some(mut validator) = self.validators.get_mut(node_id) {
            validator.metrics.byzantine_behaviors += 1;
            validator.is_active = false;

            warn!(
                validator_id = %node_id,
                byzantine_count = validator.metrics.byzantine_behaviors,
                "Validator marked as Byzantine"
            );

            Ok(())
        } else {
            Err(ConsensusError::InvalidValidatorSet(format!(
                "Validator {} not found",
                node_id
            )))
        }
    }

    /// Update validator metrics
    pub fn update_metrics(&self, node_id: &str, metrics: ValidatorMetrics) -> Result<()> {
        if let Some(mut validator) = self.validators.get_mut(node_id) {
            validator.metrics = metrics;
            validator.touch();

            if !metrics.is_healthy() && validator.is_active {
                validator.is_active = false;
                warn!(
                    validator_id = %node_id,
                    reputation = metrics.reputation_score(),
                    "Validator deactivated due to low reputation"
                );
            }

            Ok(())
        } else {
            Err(ConsensusError::InvalidValidatorSet(format!(
                "Validator {} not found",
                node_id
            )))
        }
    }

    /// Rotate validators: remove inactive, add new
    pub fn rotate_validators(&self, new_validators: Vec<ValidatorInfo>) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Remove inactive validators
        let to_remove: Vec<_> = self
            .validators
            .iter()
            .filter(|entry| {
                now - entry.value().last_activity_ms > self.inactivity_timeout_ms
            })
            .map(|entry| entry.key().clone())
            .collect();

        for node_id in to_remove {
            if self.validators.len() > self.min_validators {
                self.validators.remove(&node_id);
                info!(validator_id = %node_id, "Inactive validator removed");
            }
        }

        // Add new validators if space available
        for validator in new_validators {
            if self.validators.len() < self.max_validators {
                self.add_validator(validator)?;
            }
        }

        debug!(
            active = self.validators.len(),
            max = self.max_validators,
            "Validator rotation complete"
        );

        Ok(())
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }

    /// Get active validator count
    pub fn active_validator_count(&self) -> usize {
        self.validators
            .iter()
            .filter(|entry| entry.value().is_active)
            .count()
    }

    /// Check if validator set is healthy
    pub fn is_healthy(&self) -> bool {
        self.active_validator_count() >= self.min_validators
    }

    /// Identify Byzantine nodes
    pub fn identify_byzantine(&self) -> Vec<String> {
        self.validators
            .iter()
            .filter(|entry| entry.value().metrics.byzantine_behaviors > 0)
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get health status
    pub fn get_health_status(&self) -> ValidatorSetHealth {
        ValidatorSetHealth {
            total_validators: self.validators.len(),
            active_validators: self.active_validator_count(),
            byzantine_validators: self.identify_byzantine().len(),
            is_healthy: self.is_healthy(),
        }
    }
}

/// Validator set health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSetHealth {
    /// Total validators
    pub total_validators: usize,
    /// Active validators
    pub active_validators: usize,
    /// Byzantine validators
    pub byzantine_validators: usize,
    /// Is healthy
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_metrics_reputation() {
        let metrics = ValidatorMetrics {
            valid_messages: 100,
            invalid_signatures: 10,
            byzantine_behaviors: 0,
            uptime: 95,
            avg_response_time_ms: 100,
        };
        assert!(metrics.reputation_score() > 0.7);
        assert!(metrics.is_healthy());
    }

    #[test]
    fn test_validator_set_creation() {
        let set = ValidatorSet::new(10, 3).unwrap();
        assert_eq!(set.validator_count(), 0);
    }

    #[test]
    fn test_add_validator() {
        let set = ValidatorSet::new(10, 3).unwrap();
        let validator = ValidatorInfo {
            node_id: "node1".to_string(),
            public_key: vec![1; 32],
            metrics: ValidatorMetrics {
                valid_messages: 0,
                invalid_signatures: 0,
                byzantine_behaviors: 0,
                uptime: 100,
                avg_response_time_ms: 0,
            },
            is_active: true,
            joined_timestamp_ms: 0,
            last_activity_ms: 0,
        };
        set.add_validator(validator).unwrap();
        assert_eq!(set.validator_count(), 1);
    }

    #[test]
    fn test_byzantine_marking() {
        let set = ValidatorSet::new(10, 3).unwrap();
        let validator = ValidatorInfo {
            node_id: "node1".to_string(),
            public_key: vec![1; 32],
            metrics: ValidatorMetrics {
                valid_messages: 0,
                invalid_signatures: 0,
                byzantine_behaviors: 0,
                uptime: 100,
                avg_response_time_ms: 0,
            },
            is_active: true,
            joined_timestamp_ms: 0,
            last_activity_ms: 0,
        };
        set.add_validator(validator).unwrap();
        set.mark_byzantine("node1").unwrap();
        assert_eq!(set.identify_byzantine().len(), 1);
    }
}
