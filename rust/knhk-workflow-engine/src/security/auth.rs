#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
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

    /// Authenticate a principal (placeholder - would integrate with SPIFFE/SPIRE)
    pub fn authenticate(&self, token: &str) -> WorkflowResult<Principal> {
        // FUTURE: Integrate with SPIFFE/SPIRE for mTLS authentication
        // For now, return a placeholder principal
        Ok(Principal {
            id: token.to_string(),
            principal_type: PrincipalType::Service,
            attributes: HashMap::new(),
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
            resource_patterns: vec!["*".to_string()],
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
}
