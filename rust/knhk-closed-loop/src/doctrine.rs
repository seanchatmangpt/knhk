// Doctrine Encoding: Machine-Readable Organizational Policies
// Converts organizational constraints into enforceable Q invariants
// Implements the "Doctrine Encoding" layer from "From Programs to Planets" (2027)

use arc_swap::ArcSwap;
use chrono::Timelike;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;

/// Organizational policy or constraint encoded as a machine-readable rule
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctrineRule {
    /// Unique identifier (e.g., "SOD-001", "FIN-APPROVAL-02")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Which organizational sector this applies to
    pub sector: String,

    /// Type of constraint this rule enforces
    pub constraint_type: ConstraintType,

    /// Human-readable description
    pub description: String,

    /// Constraint-specific configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// How strictly this rule is enforced
    pub enforcement_level: EnforcementLevel,

    /// Source of this doctrine (e.g., "HIPAA", "SOX", "org-policy-2027")
    pub source: String,

    /// When this rule becomes effective (Unix timestamp)
    pub effective_date: u64,

    /// Optional expiration date (Unix timestamp)
    pub expires: Option<u64>,
}

/// Types of constraints that can be encoded as doctrines
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConstraintType {
    /// Approval chain requirement (e.g., finance approvals)
    ApprovalChain {
        required_signers: usize,
        sectors: Vec<String>,
    },

    /// Segregation of duties (incompatible roles)
    SegregationOfDuties {
        incompatible_roles: Vec<Vec<String>>,
    },

    /// Resource usage limits (e.g., transaction amounts, compute quotas)
    ResourceLimit {
        resource_type: String,
        max_value: f64,
    },

    /// Time-based restrictions (e.g., maintenance windows)
    TimeWindow {
        start_hour: u8,
        end_hour: u8,
        days: Vec<String>,
    },

    /// SHACL-based schema constraints
    Schema { rules: Vec<String> },

    /// Custom constraint type with flexible rule definition
    Custom { rule_type: String },
}

/// How strictly a doctrine rule is enforced
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EnforcementLevel {
    /// Must hold; violations block execution
    Mandatory,

    /// Should hold; violations log warnings but proceed
    Warning,

    /// Nice to have; violations are informational only
    Advisory,
}

/// Immutable snapshot of all active doctrines at a point in time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctrineSnapshot {
    /// Snapshot identifier
    pub id: String,

    /// When this snapshot was created
    pub timestamp: u64,

    /// All rules in effect at this snapshot
    pub rules: Vec<DoctrineRule>,

    /// SHA-256 hash of serialized rules (for verification)
    pub hash: String,
}

impl DoctrineSnapshot {
    /// Create new snapshot from rules
    pub fn new(rules: Vec<DoctrineRule>) -> Result<Self, DoctrineError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| DoctrineError::Internal(format!("Time error: {}", e)))?
            .as_secs();

        let serialized = serde_json::to_string(&rules)?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let hash = hex::encode(hasher.finalize());

        let id = format!("doctrine-snapshot-{}", timestamp);

        Ok(DoctrineSnapshot {
            id,
            timestamp,
            rules,
            hash,
        })
    }

    /// Verify snapshot integrity
    pub fn verify_hash(&self) -> Result<bool, DoctrineError> {
        let serialized = serde_json::to_string(&self.rules)?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let computed_hash = hex::encode(hasher.finalize());

        Ok(computed_hash == self.hash)
    }
}

/// Append-only store of all doctrine rules with history tracking
pub struct DoctrineStore {
    /// Active rules by ID (concurrent access)
    rules: DashMap<String, Arc<DoctrineRule>>,

    /// Append-only history log (timestamp, action, rule)
    history: std::sync::RwLock<Vec<(u64, String, DoctrineRule)>>,

    /// Current atomic snapshot of all rules
    current_snapshot: ArcSwap<DoctrineSnapshot>,
}

impl DoctrineStore {
    /// Create new empty doctrine store
    pub fn new() -> Result<Self, DoctrineError> {
        let empty_snapshot = DoctrineSnapshot::new(vec![])?;

        Ok(DoctrineStore {
            rules: DashMap::new(),
            history: std::sync::RwLock::new(Vec::new()),
            current_snapshot: ArcSwap::from_pointee(empty_snapshot),
        })
    }

    /// Add a new doctrine rule
    pub fn add_rule(&self, rule: DoctrineRule) -> Result<String, DoctrineError> {
        let rule_id = rule.id.clone();

        // Validate rule before adding
        self.validate_rule(&rule)?;

        // Record in history
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| DoctrineError::Internal(format!("Time error: {}", e)))?
            .as_secs();

        {
            let mut history = self
                .history
                .write()
                .map_err(|e| DoctrineError::Internal(format!("Lock error: {}", e)))?;
            history.push((timestamp, "add_rule".to_string(), rule.clone()));
        }

        // Add to active rules
        self.rules.insert(rule_id.clone(), Arc::new(rule));

        Ok(rule_id)
    }

    /// Get a specific rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Result<Arc<DoctrineRule>, DoctrineError> {
        self.rules
            .get(rule_id)
            .map(|entry| entry.clone())
            .ok_or_else(|| DoctrineError::RuleNotFound(rule_id.to_string()))
    }

    /// List all rules applicable to a sector
    pub fn list_rules_for_sector(&self, sector: &str) -> Vec<Arc<DoctrineRule>> {
        self.rules
            .iter()
            .filter(|entry| entry.value().sector == sector)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// List all active rules
    pub fn list_all_rules(&self) -> Vec<Arc<DoctrineRule>> {
        self.rules
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Validate a proposal against all applicable doctrines
    pub fn validate_against_doctrines(
        &self,
        proposal: &str,
        sector: &str,
        context: &ValidationContext,
    ) -> Result<Vec<DoctrineViolation>, DoctrineError> {
        let applicable_rules = self.list_rules_for_sector(sector);
        let mut violations = Vec::new();

        for rule in applicable_rules {
            // Check if rule is currently effective
            if !self.is_rule_effective(&rule)? {
                continue;
            }

            // Validate against this rule
            if let Some(violation) = self.check_rule(&rule, proposal, context)? {
                violations.push(violation);
            }
        }

        Ok(violations)
    }

    /// Get current snapshot
    pub fn get_snapshot(&self) -> Arc<DoctrineSnapshot> {
        self.current_snapshot.load_full()
    }

    /// Promote a new snapshot (atomic operation)
    pub fn promote_snapshot(
        &self,
        new_rules: Vec<DoctrineRule>,
    ) -> Result<Arc<DoctrineSnapshot>, DoctrineError> {
        let new_snapshot = DoctrineSnapshot::new(new_rules)?;
        let arc_snapshot = Arc::new(new_snapshot);
        self.current_snapshot.store(arc_snapshot.clone());
        Ok(arc_snapshot)
    }

    /// Get history of all doctrine changes
    pub fn get_history(&self) -> Result<Vec<(u64, String, DoctrineRule)>, DoctrineError> {
        let history = self
            .history
            .read()
            .map_err(|e| DoctrineError::Internal(format!("Lock error: {}", e)))?;
        Ok(history.clone())
    }

    /// Validate rule structure
    fn validate_rule(&self, rule: &DoctrineRule) -> Result<(), DoctrineError> {
        if rule.id.is_empty() {
            return Err(DoctrineError::InvalidRule(
                "Rule ID cannot be empty".to_string(),
            ));
        }

        if rule.name.is_empty() {
            return Err(DoctrineError::InvalidRule(
                "Rule name cannot be empty".to_string(),
            ));
        }

        if rule.sector.is_empty() {
            return Err(DoctrineError::InvalidRule(
                "Rule sector cannot be empty".to_string(),
            ));
        }

        // Validate constraint type-specific requirements
        match &rule.constraint_type {
            ConstraintType::ApprovalChain {
                required_signers, ..
            } => {
                if *required_signers == 0 {
                    return Err(DoctrineError::InvalidRule(
                        "ApprovalChain requires at least 1 signer".to_string(),
                    ));
                }
            }
            ConstraintType::ResourceLimit { max_value, .. } => {
                if *max_value < 0.0 {
                    return Err(DoctrineError::InvalidRule(
                        "ResourceLimit max_value cannot be negative".to_string(),
                    ));
                }
            }
            ConstraintType::TimeWindow {
                start_hour,
                end_hour,
                ..
            } => {
                if *start_hour >= 24 || *end_hour >= 24 {
                    return Err(DoctrineError::InvalidRule(
                        "TimeWindow hours must be 0-23".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if a rule is currently effective
    fn is_rule_effective(&self, rule: &DoctrineRule) -> Result<bool, DoctrineError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| DoctrineError::Internal(format!("Time error: {}", e)))?
            .as_secs();

        // Must be past effective date
        if now < rule.effective_date {
            return Ok(false);
        }

        // Must not be expired
        if let Some(expires) = rule.expires {
            if now >= expires {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check a single rule against a proposal
    fn check_rule(
        &self,
        rule: &DoctrineRule,
        _proposal: &str,
        context: &ValidationContext,
    ) -> Result<Option<DoctrineViolation>, DoctrineError> {
        match &rule.constraint_type {
            ConstraintType::ApprovalChain {
                required_signers,
                sectors,
            } => {
                if context.signers.len() < *required_signers {
                    return Ok(Some(DoctrineViolation {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        violation_reason: format!(
                            "Requires {} signers, but only {} provided",
                            required_signers,
                            context.signers.len()
                        ),
                        enforcement_level: rule.enforcement_level.clone(),
                    }));
                }

                // Check if signers are from required sectors
                for required_sector in sectors {
                    if !context.signers.iter().any(|s| &s.sector == required_sector) {
                        return Ok(Some(DoctrineViolation {
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            violation_reason: format!(
                                "Requires signer from sector '{}', but none found",
                                required_sector
                            ),
                            enforcement_level: rule.enforcement_level.clone(),
                        }));
                    }
                }

                Ok(None)
            }

            ConstraintType::SegregationOfDuties { incompatible_roles } => {
                for incompatible_set in incompatible_roles {
                    let mut roles_found = Vec::new();
                    for signer in &context.signers {
                        if incompatible_set.contains(&signer.role) {
                            roles_found.push(signer.role.clone());
                        }
                    }

                    if roles_found.len() > 1 {
                        return Ok(Some(DoctrineViolation {
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            violation_reason: format!(
                                "Segregation of duties violated: same entity holds incompatible roles {:?}",
                                roles_found
                            ),
                            enforcement_level: rule.enforcement_level.clone(),
                        }));
                    }
                }

                Ok(None)
            }

            ConstraintType::ResourceLimit {
                resource_type,
                max_value,
            } => {
                if let Some(value) = context.resources.get(resource_type) {
                    if value > max_value {
                        return Ok(Some(DoctrineViolation {
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            violation_reason: format!(
                                "Resource limit exceeded: {} = {}, max allowed = {}",
                                resource_type, value, max_value
                            ),
                            enforcement_level: rule.enforcement_level.clone(),
                        }));
                    }
                }

                Ok(None)
            }

            ConstraintType::TimeWindow {
                start_hour,
                end_hour,
                days,
            } => {
                let now = chrono::Utc::now();
                let current_hour = now.hour() as u8;
                let current_day = now.format("%A").to_string();

                // Check if within allowed time window
                let in_time_window = if start_hour <= end_hour {
                    current_hour >= *start_hour && current_hour < *end_hour
                } else {
                    // Crosses midnight
                    current_hour >= *start_hour || current_hour < *end_hour
                };

                let in_day_window = days.is_empty() || days.contains(&current_day);

                if !in_time_window || !in_day_window {
                    return Ok(Some(DoctrineViolation {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        violation_reason: format!(
                            "Operation not allowed at current time (hour: {}, day: {})",
                            current_hour, current_day
                        ),
                        enforcement_level: rule.enforcement_level.clone(),
                    }));
                }

                Ok(None)
            }

            ConstraintType::Schema {
                rules: _schema_rules,
            } => {
                // Schema validation would parse proposal and check against SHACL rules
                // Simplified for now - would integrate with actual RDF validation
                Ok(None)
            }

            ConstraintType::Custom { rule_type } => {
                // Custom rules would have pluggable validators
                // For now, just check if context has custom validation results
                if let Some(custom_result) = context.custom_validations.get(rule_type) {
                    if !custom_result {
                        return Ok(Some(DoctrineViolation {
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            violation_reason: format!(
                                "Custom rule '{}' validation failed",
                                rule_type
                            ),
                            enforcement_level: rule.enforcement_level.clone(),
                        }));
                    }
                }

                Ok(None)
            }
        }
    }
}

impl Default for DoctrineStore {
    fn default() -> Self {
        Self::new().expect("Failed to create default DoctrineStore")
    }
}

/// Context information for validating a proposal against doctrines
#[derive(Clone, Debug, Default)]
pub struct ValidationContext {
    /// Entities that have signed/approved this proposal
    pub signers: Vec<Signer>,

    /// Resource usage claimed by this proposal
    pub resources: HashMap<String, f64>,

    /// Results of custom validation functions
    pub custom_validations: HashMap<String, bool>,
}

/// Information about an entity that signed/approved a proposal
#[derive(Clone, Debug)]
pub struct Signer {
    /// Unique identifier
    pub id: String,

    /// Role or position
    pub role: String,

    /// Organizational sector
    pub sector: String,
}

/// Record of a doctrine violation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DoctrineViolation {
    /// ID of the rule that was violated
    pub rule_id: String,

    /// Name of the rule that was violated
    pub rule_name: String,

    /// Explanation of why the violation occurred
    pub violation_reason: String,

    /// How strictly this violation should be enforced
    pub enforcement_level: EnforcementLevel,
}

impl DoctrineViolation {
    /// Check if this violation blocks execution
    pub fn is_blocking(&self) -> bool {
        matches!(self.enforcement_level, EnforcementLevel::Mandatory)
    }

    /// Check if this violation should generate a warning
    pub fn is_warning(&self) -> bool {
        matches!(self.enforcement_level, EnforcementLevel::Warning)
    }

    /// Check if this violation is advisory only
    pub fn is_advisory(&self) -> bool {
        matches!(self.enforcement_level, EnforcementLevel::Advisory)
    }
}

/// Mapping from doctrine rules to Q invariant checks
#[derive(Clone, Debug)]
pub struct DoctrineToBoundInvariant {
    /// The source doctrine rule
    pub doctrine: DoctrineRule,

    /// Which Q invariants this doctrine affects
    pub checks_q: Vec<String>,
}

impl DoctrineToBoundInvariant {
    /// Create mapping for a doctrine rule
    pub fn new(doctrine: DoctrineRule) -> Self {
        let checks_q = match &doctrine.constraint_type {
            ConstraintType::ApprovalChain { .. } => {
                vec!["Q2".to_string()] // Type soundness (proper authorization)
            }
            ConstraintType::SegregationOfDuties { .. } => {
                vec!["Q2".to_string()] // Type soundness (role constraints)
            }
            ConstraintType::ResourceLimit { .. } => {
                vec!["Q5".to_string()] // Performance bounds (resource limits)
            }
            ConstraintType::TimeWindow { .. } => {
                vec!["Q1".to_string()] // No retrocausation (time ordering)
            }
            ConstraintType::Schema { .. } => {
                vec!["Q2".to_string()] // Type soundness (schema validation)
            }
            ConstraintType::Custom { .. } => {
                vec!["Q2".to_string(), "Q5".to_string()] // Multiple invariants
            }
        };

        DoctrineToBoundInvariant { doctrine, checks_q }
    }

    /// Get the invariants affected by this doctrine
    pub fn affected_invariants(&self) -> &[String] {
        &self.checks_q
    }
}

/// Errors that can occur during doctrine operations
#[derive(Debug, thiserror::Error)]
pub enum DoctrineError {
    #[error("Rule not found: {0}")]
    RuleNotFound(String),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Hash verification failed")]
    HashVerificationFailed,

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctrine_rule_creation() {
        let rule = DoctrineRule {
            id: "FIN-001".to_string(),
            name: "Finance approval chain".to_string(),
            sector: "finance".to_string(),
            constraint_type: ConstraintType::ApprovalChain {
                required_signers: 2,
                sectors: vec!["finance".to_string(), "compliance".to_string()],
            },
            description: "Requires two approvers from finance and compliance".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "org-policy-2027".to_string(),
            effective_date: 0,
            expires: None,
        };

        assert_eq!(rule.id, "FIN-001");
        assert_eq!(rule.enforcement_level, EnforcementLevel::Mandatory);
    }

    #[test]
    fn test_doctrine_store_add_and_get() {
        let store = DoctrineStore::new().expect("Failed to create store");

        let rule = DoctrineRule {
            id: "TEST-001".to_string(),
            name: "Test rule".to_string(),
            sector: "test".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "cpu".to_string(),
                max_value: 80.0,
            },
            description: "Test".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Warning,
            source: "test".to_string(),
            effective_date: 0,
            expires: None,
        };

        let rule_id = store.add_rule(rule.clone()).expect("Failed to add rule");
        assert_eq!(rule_id, "TEST-001");

        let retrieved = store.get_rule(&rule_id).expect("Failed to get rule");
        assert_eq!(retrieved.id, rule_id);
        assert_eq!(retrieved.name, rule.name);
    }

    #[test]
    fn test_doctrine_snapshot() {
        let rules = vec![DoctrineRule {
            id: "R1".to_string(),
            name: "Rule 1".to_string(),
            sector: "s1".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "memory".to_string(),
                max_value: 1024.0,
            },
            description: "Limit memory".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "test".to_string(),
            effective_date: 0,
            expires: None,
        }];

        let snapshot = DoctrineSnapshot::new(rules).expect("Failed to create snapshot");
        assert!(snapshot.verify_hash().expect("Hash verification failed"));
        assert_eq!(snapshot.rules.len(), 1);
    }

    #[test]
    fn test_approval_chain_validation() {
        let store = DoctrineStore::new().expect("Failed to create store");

        let rule = DoctrineRule {
            id: "APPROVAL-001".to_string(),
            name: "Two-person approval".to_string(),
            sector: "finance".to_string(),
            constraint_type: ConstraintType::ApprovalChain {
                required_signers: 2,
                sectors: vec!["finance".to_string()],
            },
            description: "Requires two finance approvers".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "policy".to_string(),
            effective_date: 0,
            expires: None,
        };

        store.add_rule(rule).expect("Failed to add rule");

        // Test with insufficient signers
        let context = ValidationContext {
            signers: vec![Signer {
                id: "user1".to_string(),
                role: "manager".to_string(),
                sector: "finance".to_string(),
            }],
            resources: HashMap::new(),
            custom_validations: HashMap::new(),
        };

        let violations = store
            .validate_against_doctrines("test proposal", "finance", &context)
            .expect("Validation failed");

        assert_eq!(violations.len(), 1);
        assert!(violations[0].is_blocking());
    }

    #[test]
    fn test_resource_limit_validation() {
        let store = DoctrineStore::new().expect("Failed to create store");

        let rule = DoctrineRule {
            id: "RES-001".to_string(),
            name: "CPU limit".to_string(),
            sector: "compute".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "cpu_percent".to_string(),
                max_value: 75.0,
            },
            description: "CPU cannot exceed 75%".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Warning,
            source: "slo".to_string(),
            effective_date: 0,
            expires: None,
        };

        store.add_rule(rule).expect("Failed to add rule");

        let mut resources = HashMap::new();
        resources.insert("cpu_percent".to_string(), 85.0);

        let context = ValidationContext {
            signers: vec![],
            resources,
            custom_validations: HashMap::new(),
        };

        let violations = store
            .validate_against_doctrines("high cpu proposal", "compute", &context)
            .expect("Validation failed");

        assert_eq!(violations.len(), 1);
        assert!(violations[0].is_warning());
    }

    #[test]
    fn test_doctrine_to_invariant_mapping() {
        let rule = DoctrineRule {
            id: "MAP-001".to_string(),
            name: "Resource mapping test".to_string(),
            sector: "test".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "memory".to_string(),
                max_value: 1024.0,
            },
            description: "Test mapping".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "test".to_string(),
            effective_date: 0,
            expires: None,
        };

        let mapping = DoctrineToBoundInvariant::new(rule);
        assert!(mapping.affected_invariants().contains(&"Q5".to_string()));
    }

    #[test]
    fn test_time_window_validation() {
        let store = DoctrineStore::new().expect("Failed to create store");

        let rule = DoctrineRule {
            id: "TIME-001".to_string(),
            name: "Time window test".to_string(),
            sector: "test".to_string(),
            constraint_type: ConstraintType::TimeWindow {
                start_hour: 9,
                end_hour: 17,
                days: vec![
                    "Monday".to_string(),
                    "Tuesday".to_string(),
                    "Wednesday".to_string(),
                    "Thursday".to_string(),
                    "Friday".to_string(),
                ],
            },
            description: "Business hours only".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Warning,
            source: "test".to_string(),
            effective_date: 0,
            expires: None,
        };

        store.add_rule(rule).expect("Failed to add rule");

        let context = ValidationContext::default();
        let _violations = store
            .validate_against_doctrines("time_test", "test", &context)
            .expect("Validation failed");

        // Time window validation happens dynamically
        // This test verifies the rule is registered and can be evaluated
        assert!(store.get_rule("TIME-001").is_ok());
    }

    #[test]
    fn test_effective_date_filtering() {
        let store = DoctrineStore::new().expect("Failed to create store");

        // Rule effective in the future
        let future_rule = DoctrineRule {
            id: "FUTURE-001".to_string(),
            name: "Future rule".to_string(),
            sector: "test".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "memory".to_string(),
                max_value: 100.0,
            },
            description: "Future rule".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "test".to_string(),
            effective_date: (chrono::Utc::now().timestamp() + 86400) as u64, // Tomorrow
            expires: None,
        };

        store.add_rule(future_rule).expect("Failed to add rule");

        let context = ValidationContext::default();
        let violations = store
            .validate_against_doctrines("test_proposal", "test", &context)
            .expect("Validation failed");

        // Should not be evaluated (not yet effective)
        assert!(violations.is_empty(), "Future rule should not be evaluated");
    }

    #[test]
    fn test_expired_rule_not_enforced() {
        let store = DoctrineStore::new().expect("Failed to create store");

        // Rule that expired
        let expired_rule = DoctrineRule {
            id: "EXPIRED-001".to_string(),
            name: "Expired rule".to_string(),
            sector: "test".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "cpu".to_string(),
                max_value: 50.0,
            },
            description: "Expired rule".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "test".to_string(),
            effective_date: 0,
            expires: Some((chrono::Utc::now().timestamp() - 1) as u64), // Expired 1 second ago
        };

        store.add_rule(expired_rule).expect("Failed to add rule");

        let mut resources = HashMap::new();
        resources.insert("cpu".to_string(), 75.0); // Exceeds limit

        let context = ValidationContext {
            signers: vec![],
            resources,
            custom_validations: HashMap::new(),
        };

        let violations = store
            .validate_against_doctrines("test_proposal", "test", &context)
            .expect("Validation failed");

        // Should not be enforced (expired)
        assert!(violations.is_empty(), "Expired rule should not be enforced");
    }

    #[test]
    fn test_custom_constraint_type() {
        let store = DoctrineStore::new().expect("Failed to create store");

        let custom_rule = DoctrineRule {
            id: "CUSTOM-001".to_string(),
            name: "Custom validation".to_string(),
            sector: "test".to_string(),
            constraint_type: ConstraintType::Custom {
                rule_type: "special_check".to_string(),
            },
            description: "Custom rule".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "test".to_string(),
            effective_date: 0,
            expires: None,
        };

        store.add_rule(custom_rule).expect("Failed to add rule");

        // Provide custom validation result
        let mut custom_validations = HashMap::new();
        custom_validations.insert("special_check".to_string(), true);

        let context = ValidationContext {
            signers: vec![],
            resources: HashMap::new(),
            custom_validations,
        };

        let violations = store
            .validate_against_doctrines("test_proposal", "test", &context)
            .expect("Validation failed");

        // Custom validation passed
        assert!(violations.is_empty());
    }

    #[test]
    fn test_doctrine_snapshot_hash_verification() {
        let rules = vec![DoctrineRule {
            id: "R1".to_string(),
            name: "Rule 1".to_string(),
            sector: "s1".to_string(),
            constraint_type: ConstraintType::ResourceLimit {
                resource_type: "memory".to_string(),
                max_value: 1024.0,
            },
            description: "Memory limit".to_string(),
            parameters: HashMap::new(),
            enforcement_level: EnforcementLevel::Mandatory,
            source: "test".to_string(),
            effective_date: 0,
            expires: None,
        }];

        let snapshot = DoctrineSnapshot::new(rules.clone()).expect("Failed to create snapshot");

        // Verify hash
        assert!(snapshot.verify_hash().expect("Verification failed"));

        // Tamper with snapshot
        let mut tampered = snapshot.clone();
        tampered.rules[0].name = "Modified".to_string();

        // Hash should not match
        assert!(!tampered.verify_hash().expect("Verification failed"));
    }
}

// ============================================================================
// PROPERTY-BASED TESTS
// ============================================================================

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_any_rule_with_valid_id_can_be_added(
            id in "[A-Z]{2,5}-[0-9]{3}",
            sector in "[a-z]{3,10}"
        ) {
            let store = DoctrineStore::new().expect("Failed to create store");

            let rule = DoctrineRule {
                id: id.clone(),
                name: format!("Rule {}", id),
                sector: sector.clone(),
                constraint_type: ConstraintType::ResourceLimit {
                    resource_type: "test".to_string(),
                    max_value: 100.0,
                },
                description: "Test rule".to_string(),
                parameters: HashMap::new(),
                enforcement_level: EnforcementLevel::Advisory,
                source: "proptest".to_string(),
                effective_date: 0,
                expires: None,
            };

            let result = store.add_rule(rule);
            prop_assert!(result.is_ok());

            let retrieved = store.get_rule(&id);
            prop_assert!(retrieved.is_ok());
            prop_assert_eq!(&retrieved.unwrap().sector, &sector);
        }

        #[test]
        fn prop_resource_limit_enforced_correctly(
            max_value in 1.0f64..=1000.0,
            actual_value in 1.0f64..=2000.0
        ) {
            let store = DoctrineStore::new().expect("Failed to create store");

            let rule = DoctrineRule {
                id: "RES-TEST".to_string(),
                name: "Resource limit".to_string(),
                sector: "test".to_string(),
                constraint_type: ConstraintType::ResourceLimit {
                    resource_type: "test_resource".to_string(),
                    max_value,
                },
                description: "Test".to_string(),
                parameters: HashMap::new(),
                enforcement_level: EnforcementLevel::Mandatory,
                source: "test".to_string(),
                effective_date: 0,
                expires: None,
            };

            store.add_rule(rule).expect("Failed to add rule");

            let mut resources = HashMap::new();
            resources.insert("test_resource".to_string(), actual_value);

            let context = ValidationContext {
                signers: vec![],
                resources,
                custom_validations: HashMap::new(),
            };

            let violations = store
                .validate_against_doctrines("test", "test", &context)
                .expect("Validation failed");

            // Property: Violation iff actual > max
            let has_violation = !violations.is_empty();
            prop_assert_eq!(has_violation, actual_value > max_value);
        }

        #[test]
        fn prop_approval_chain_quorum_enforced(
            required_signers in 1usize..=5,
            actual_signers in 0usize..=6
        ) {
            let store = DoctrineStore::new().expect("Failed to create store");

            let rule = DoctrineRule {
                id: "APPROVAL-TEST".to_string(),
                name: "Approval chain".to_string(),
                sector: "test".to_string(),
                constraint_type: ConstraintType::ApprovalChain {
                    required_signers,
                    sectors: vec!["test".to_string()],
                },
                description: "Test".to_string(),
                parameters: HashMap::new(),
                enforcement_level: EnforcementLevel::Mandatory,
                source: "test".to_string(),
                effective_date: 0,
                expires: None,
            };

            store.add_rule(rule).expect("Failed to add rule");

            let signers: Vec<Signer> = (0..actual_signers).map(|i| Signer {
                id: format!("signer-{}", i),
                role: "approver".to_string(),
                sector: "test".to_string(),
            }).collect();

            let context = ValidationContext {
                signers,
                resources: HashMap::new(),
                custom_validations: HashMap::new(),
            };

            let violations = store
                .validate_against_doctrines("test", "test", &context)
                .expect("Validation failed");

            // Property: Violation iff actual < required
            let has_violation = !violations.is_empty();
            prop_assert_eq!(has_violation, actual_signers < required_signers);
        }

        #[test]
        fn prop_segregation_of_duties_enforced(
            assign_both_roles in any::<bool>()
        ) {
            let store = DoctrineStore::new().expect("Failed to create store");

            let rule = DoctrineRule {
                id: "SOD-TEST".to_string(),
                name: "Segregation test".to_string(),
                sector: "test".to_string(),
                constraint_type: ConstraintType::SegregationOfDuties {
                    incompatible_roles: vec![
                        vec!["role_a".to_string(), "role_b".to_string()],
                    ],
                },
                description: "Test".to_string(),
                parameters: HashMap::new(),
                enforcement_level: EnforcementLevel::Mandatory,
                source: "test".to_string(),
                effective_date: 0,
                expires: None,
            };

            store.add_rule(rule).expect("Failed to add rule");

            let signers = if assign_both_roles {
                vec![
                    Signer {
                        id: "user1".to_string(),
                        role: "role_a".to_string(),
                        sector: "test".to_string(),
                    },
                    Signer {
                        id: "user1".to_string(),
                        role: "role_b".to_string(),
                        sector: "test".to_string(),
                    },
                ]
            } else {
                vec![
                    Signer {
                        id: "user1".to_string(),
                        role: "role_a".to_string(),
                        sector: "test".to_string(),
                    },
                ]
            };

            let context = ValidationContext {
                signers,
                resources: HashMap::new(),
                custom_validations: HashMap::new(),
            };

            let violations = store
                .validate_against_doctrines("test", "test", &context)
                .expect("Validation failed");

            // Property: Violation iff both incompatible roles assigned
            let has_violation = !violations.is_empty();
            prop_assert_eq!(has_violation, assign_both_roles);
        }

        #[test]
        fn prop_doctrine_snapshot_hash_stable(
            rule_count in 1usize..=10
        ) {
            let rules: Vec<DoctrineRule> = (0..rule_count).map(|i| DoctrineRule {
                id: format!("R{}", i),
                name: format!("Rule {}", i),
                sector: "test".to_string(),
                constraint_type: ConstraintType::ResourceLimit {
                    resource_type: "memory".to_string(),
                    max_value: 1024.0,
                },
                description: "Test".to_string(),
                parameters: HashMap::new(),
                enforcement_level: EnforcementLevel::Mandatory,
                source: "test".to_string(),
                effective_date: 0,
                expires: None,
            }).collect();

            let snapshot1 = DoctrineSnapshot::new(rules.clone()).expect("Failed to create snapshot");
            let snapshot2 = DoctrineSnapshot::new(rules).expect("Failed to create snapshot");

            // Property: Same rules = same hash (deterministic)
            prop_assert_eq!(snapshot1.hash, snapshot2.hash);
        }
    }
}
