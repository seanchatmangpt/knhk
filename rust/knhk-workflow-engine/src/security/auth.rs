//! Authentication and authorization for workflow engine

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Principal (user/service) identity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Principal {
    /// Principal ID (SPIFFE ID, user ID, etc.)
    pub id: String,
    /// Principal type
    pub principal_type: PrincipalType,
    /// Principal attributes (roles, groups, etc.)
    #[serde(skip)]
    pub attributes: HashMap<String, String>,
}

/// Principal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrincipalType {
    /// Human user
    User,
    /// Service/application
    Service,
    /// System principal
    System,
}

/// Authorization policy
#[derive(Debug, Clone)]
pub struct AuthPolicy {
    /// Policy name
    pub name: String,
    /// Allowed principals
    pub allowed_principals: Vec<String>,
    /// Allowed roles
    pub allowed_roles: Vec<String>,
    /// Resource patterns (workflow IDs, case IDs, etc.)
    pub resource_patterns: Vec<String>,
    /// Actions allowed
    pub actions: Vec<Action>,
}

/// Action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Create workflow
    CreateWorkflow,
    /// Read workflow
    ReadWorkflow,
    /// Update workflow
    UpdateWorkflow,
    /// Delete workflow
    DeleteWorkflow,
    /// Create case
    CreateCase,
    /// Read case
    ReadCase,
    /// Update case
    UpdateCase,
    /// Cancel case
    CancelCase,
    /// Execute pattern
    ExecutePattern,
}

/// Authentication and authorization manager
pub struct AuthManager {
    /// Active principals
    principals: HashMap<String, Principal>,
    /// Authorization policies
    policies: Vec<AuthPolicy>,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Self {
        Self {
            principals: HashMap::new(),
            policies: Vec::new(),
        }
    }

    /// Authenticate a principal
    ///
    /// Supports SPIFFE/SPIRE authentication by validating SPIFFE ID format
    /// and extracting principal information from the SPIFFE ID.
    ///
    /// The token parameter can be:
    /// - A SPIFFE ID (spiffe://trust-domain/path) - validated and used directly
    /// - A certificate or certificate path - would require SPIFFE Workload API client
    ///
    /// For now, implements SPIFFE ID validation and extraction.
    /// Full SPIFFE/SPIRE integration with Workload API would require additional dependencies.
    pub fn authenticate(&self, token: &str) -> WorkflowResult<Principal> {
        // Check if token is a SPIFFE ID
        if token.starts_with("spiffe://") {
            return self.authenticate_spiffe_id(token);
        }

        // Check if token is a registered principal ID
        if let Some(principal) = self.principals.get(token) {
            return Ok(principal.clone());
        }

        // Token format not recognized
        Err(WorkflowError::Validation(format!(
            "Invalid authentication token format. Expected SPIFFE ID (spiffe://trust-domain/path) or registered principal ID, got: '{}'",
            token
        )))
    }

    /// Authenticate using SPIFFE ID
    ///
    /// Validates SPIFFE ID format and extracts principal information.
    fn authenticate_spiffe_id(&self, spiffe_id: &str) -> WorkflowResult<Principal> {
        // Validate SPIFFE ID format: spiffe://trust-domain/path
        if !spiffe_id.starts_with("spiffe://") || spiffe_id.len() <= 10 {
            return Err(WorkflowError::Validation(format!(
                "Invalid SPIFFE ID format: '{}'. Expected format: spiffe://trust-domain/path",
                spiffe_id
            )));
        }

        // Extract trust domain and path
        let without_prefix = &spiffe_id[9..]; // Remove "spiffe://"
        let (trust_domain, path) = if let Some(slash_pos) = without_prefix.find('/') {
            (
                &without_prefix[..slash_pos],
                &without_prefix[slash_pos + 1..],
            )
        } else {
            (without_prefix, "")
        };

        if trust_domain.is_empty() {
            return Err(WorkflowError::Validation(format!(
                "Invalid SPIFFE ID: trust domain cannot be empty in '{}'",
                spiffe_id
            )));
        }

        // Extract attributes from SPIFFE ID path
        let mut attributes = HashMap::new();
        attributes.insert("trust_domain".to_string(), trust_domain.to_string());
        if !path.is_empty() {
            attributes.insert("spiffe_path".to_string(), path.to_string());
            // Extract service name from path (last component)
            if let Some(service_name) = path.split('/').last() {
                if !service_name.is_empty() {
                    attributes.insert("service".to_string(), service_name.to_string());
                }
            }
        }

        // Determine principal type from SPIFFE ID path
        let principal_type = if path.starts_with("user/") || path.contains("/user/") {
            PrincipalType::User
        } else if path.starts_with("system/") || path.contains("/system/") {
            PrincipalType::System
        } else {
            PrincipalType::Service // Default for SPIFFE IDs
        };

        Ok(Principal {
            id: spiffe_id.to_string(),
            principal_type,
            attributes,
        })
    }

    /// Authorize an action
    pub fn authorize(
        &self,
        principal: &Principal,
        action: Action,
        resource: &str,
    ) -> WorkflowResult<()> {
        // Check policies
        for policy in &self.policies {
            if policy.actions.contains(&action) {
                // Check if principal matches
                if policy.allowed_principals.contains(&principal.id)
                    || principal
                        .attributes
                        .get("role")
                        .map(|role| policy.allowed_roles.contains(role))
                        .unwrap_or(false)
                {
                    // Check resource pattern
                    if policy.resource_patterns.is_empty()
                        || policy
                            .resource_patterns
                            .iter()
                            .any(|pattern| resource.contains(pattern.as_str()))
                    {
                        return Ok(());
                    }
                }
            }
        }

        Err(WorkflowError::Validation(format!(
            "Principal {} not authorized for action {:?} on resource {}",
            principal.id, action, resource
        )))
    }

    /// Add a policy
    pub fn add_policy(&mut self, policy: AuthPolicy) {
        self.policies.push(policy);
    }

    /// Register a principal
    pub fn register_principal(&mut self, principal: Principal) {
        self.principals.insert(principal.id.clone(), principal);
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager() {
        let mut manager = AuthManager::new();
        let principal = Principal {
            id: "test-user".to_string(),
            principal_type: PrincipalType::User,
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("role".to_string(), "admin".to_string());
                attrs
            },
        };

        manager.register_principal(principal.clone());

        let policy = AuthPolicy {
            name: "admin-policy".to_string(),
            allowed_principals: vec![],
            allowed_roles: vec!["admin".to_string()],
            resource_patterns: vec!["workflow".to_string()], // Pattern that matches "workflow-1"
            actions: vec![Action::CreateWorkflow, Action::ReadWorkflow],
        };

        manager.add_policy(policy);

        assert!(manager
            .authorize(&principal, Action::CreateWorkflow, "workflow-1")
            .is_ok());
        assert!(manager
            .authorize(&principal, Action::DeleteWorkflow, "workflow-1")
            .is_err());
    }

    #[test]
    fn test_authenticate_spiffe_id() {
        let manager = AuthManager::new();

        // Valid SPIFFE ID
        let principal = manager
            .authenticate("spiffe://example.com/workflow-engine")
            .expect("should authenticate SPIFFE ID");
        assert_eq!(principal.id, "spiffe://example.com/workflow-engine");
        assert_eq!(principal.principal_type, PrincipalType::Service);
        assert_eq!(
            principal.attributes.get("trust_domain"),
            Some(&"example.com".to_string())
        );
        assert_eq!(
            principal.attributes.get("service"),
            Some(&"workflow-engine".to_string())
        );

        // SPIFFE ID with user path
        let principal = manager
            .authenticate("spiffe://example.com/user/admin")
            .expect("should authenticate SPIFFE ID");
        assert_eq!(principal.principal_type, PrincipalType::User);

        // SPIFFE ID with system path
        let principal = manager
            .authenticate("spiffe://example.com/system/knhk")
            .expect("should authenticate SPIFFE ID");
        assert_eq!(principal.principal_type, PrincipalType::System);

        // Invalid SPIFFE ID
        assert!(manager.authenticate("invalid").is_err());
        assert!(manager.authenticate("spiffe://").is_err());
        assert!(manager.authenticate("spiffe://example.com").is_ok()); // Valid, empty path
    }

    #[test]
    fn test_authenticate_registered_principal() {
        let mut manager = AuthManager::new();
        let principal = Principal {
            id: "test-service".to_string(),
            principal_type: PrincipalType::Service,
            attributes: HashMap::new(),
        };

        manager.register_principal(principal.clone());

        let authenticated = manager
            .authenticate("test-service")
            .expect("should authenticate registered principal");
        assert_eq!(authenticated.id, "test-service");
    }
}
