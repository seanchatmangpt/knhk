// Guard Governance: Multi-party approval for critical policy changes
// Implements cryptographically-signed quorum-based approval workflows

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GovernanceError {
    #[error("Guard not found: {0}")]
    GuardNotFound(String),

    #[error("Guard cannot be relaxed: {0}")]
    GuardImmutable(String),

    #[error("Request not found: {0}")]
    RequestNotFound(String),

    #[error("Request not approved")]
    RequestNotApproved,

    #[error("Relaxation window not found: {0}")]
    RelaxationNotFound(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Insufficient approvals: need {needed}, have {current}")]
    InsufficientApprovals { needed: usize, current: usize },

    #[error("Request expired")]
    RequestExpired,

    #[error("Approval window not yet open")]
    ApprovalWindowNotOpen,

    #[error("Hex decoding error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("Signature error: {0}")]
    SignatureError(#[from] ed25519_dalek::SignatureError),
}

pub type Result<T> = std::result::Result<T, GovernanceError>;

/// A guard that protects system integrity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Guard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub guard_type: GuardType,
    pub criticality: Criticality,
    pub is_mutable: bool,
    pub relaxation_policy: RelaxationPolicy,
    #[serde(skip)]
    pub enforced: Arc<RwLock<bool>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GuardType {
    ApprovalChain { required_signers: Vec<String> },
    SafetyInterlock { condition: String },
    DataResidency { allowed_regions: Vec<String> },
    PerformanceBound { max_ticks: u32 },
    ComplianceRule { regulations: Vec<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Criticality {
    Critical, // Cannot be relaxed without board approval
    High,     // Requires C-level approval
    Medium,   // Requires department head approval
    Low,      // Can be relaxed by team lead
}

/// Policy for how a guard can be relaxed (changed)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelaxationPolicy {
    pub requires_multi_party: bool,
    pub required_approvers: Vec<String>, // Role IDs or person IDs
    pub approval_quorum: usize,          // How many must approve
    pub min_approval_duration_ms: u64,   // Minimum review period (ms)
    pub can_be_revoked: bool,
    pub revocation_requires_same_quorum: bool,
}

impl Guard {
    pub fn new(id: String, name: String, guard_type: GuardType, criticality: Criticality) -> Self {
        // Derive relaxation policy from criticality
        let (requires_multi, quorum, min_duration) = match criticality {
            Criticality::Critical => (true, 3, 86400000), // Board + Legal + CTO, 24h
            Criticality::High => (true, 2, 43200000),     // C-level + VP, 12h
            Criticality::Medium => (false, 1, 3600000),   // Single approver, 1h
            Criticality::Low => (false, 1, 0),            // No special approval, instant
        };

        let is_mutable = matches!(criticality, Criticality::Medium | Criticality::Low);

        Guard {
            id,
            name,
            description: String::new(),
            guard_type,
            criticality,
            is_mutable,
            relaxation_policy: RelaxationPolicy {
                requires_multi_party: requires_multi,
                required_approvers: vec![],
                approval_quorum: quorum,
                min_approval_duration_ms: min_duration,
                can_be_revoked: true,
                revocation_requires_same_quorum: requires_multi,
            },
            enforced: Arc::new(RwLock::new(true)),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_required_approvers(mut self, approvers: Vec<String>) -> Self {
        self.relaxation_policy.required_approvers = approvers;
        self
    }

    pub fn is_enforced(&self) -> bool {
        *self.enforced.read()
    }

    pub fn temporarily_disable(&self) {
        *self.enforced.write() = false;
    }

    pub fn re_enable(&self) {
        *self.enforced.write() = true;
    }
}

/// A request to relax or change a guard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuardRelaxationRequest {
    pub id: String,
    pub guard_id: String,
    pub requested_by: String,
    pub reason: String,
    pub proposed_change: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub required_approvals: Vec<Approval>,
    pub approval_signatures: Vec<GuardApproval>,
    pub state: RequestState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RequestState {
    Submitted,
    UnderReview,
    ApprovalsPending { pending_count: usize },
    Approved,
    Rejected { reason: String },
    Revoked { reason: String },
    Expired,
}

/// Single approval signature
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuardApproval {
    pub approver_id: String,
    pub signed_at: DateTime<Utc>,
    pub signature: String,     // hex-encoded ed25519 signature
    pub verifying_key: String, // hex-encoded VerifyingKey
    pub metadata: HashMap<String, serde_json::Value>,
}

impl GuardApproval {
    pub fn verify(&self, message: &str) -> Result<()> {
        let sig_bytes = hex::decode(&self.signature)?;
        let key_bytes = hex::decode(&self.verifying_key)?;

        let signature = Signature::from_bytes(
            &sig_bytes
                .try_into()
                .map_err(|_| GovernanceError::InvalidSignature)?,
        );
        let verifying_key = VerifyingKey::from_bytes(
            &key_bytes
                .try_into()
                .map_err(|_| GovernanceError::InvalidSignature)?,
        )?;

        verifying_key.verify(message.as_bytes(), &signature)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Approval {
    pub approver_role: String,
    pub approver_name: Option<String>,
    pub approved: bool,
    pub signature: Option<GuardApproval>,
}

/// Guard Governance Engine - manages approvals and state
pub struct GovernanceEngine {
    guards: DashMap<String, Arc<Guard>>,
    relaxation_requests: DashMap<String, Arc<RwLock<GuardRelaxationRequest>>>,
    approval_history: Arc<RwLock<Vec<GuardRelaxationRequest>>>,
    active_relaxations: DashMap<String, RelaxationWindow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelaxationWindow {
    pub request_id: String,
    pub guard_id: String,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reason: String,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        GovernanceEngine {
            guards: DashMap::new(),
            relaxation_requests: DashMap::new(),
            approval_history: Arc::new(RwLock::new(Vec::new())),
            active_relaxations: DashMap::new(),
        }
    }

    /// Register a guard in the governance system
    pub fn register_guard(&self, guard: Guard) -> Result<()> {
        self.guards.insert(guard.id.clone(), Arc::new(guard));
        Ok(())
    }

    /// Get a guard by ID
    pub fn get_guard(&self, guard_id: &str) -> Result<Arc<Guard>> {
        self.guards
            .get(guard_id)
            .map(|g| g.clone())
            .ok_or_else(|| GovernanceError::GuardNotFound(guard_id.to_string()))
    }

    /// Request to relax a guard (change its policy)
    pub fn request_relaxation(
        &self,
        guard_id: String,
        requested_by: String,
        reason: String,
        proposed_change: String,
    ) -> Result<String> {
        let guard = self
            .guards
            .get(&guard_id)
            .ok_or_else(|| GovernanceError::GuardNotFound(guard_id.clone()))?;

        if !guard.is_mutable {
            return Err(GovernanceError::GuardImmutable(guard_id));
        }

        // Create approval requirement list from relaxation policy
        let required_approvals = guard
            .relaxation_policy
            .required_approvers
            .iter()
            .map(|role| Approval {
                approver_role: role.clone(),
                approver_name: None,
                approved: false,
                signature: None,
            })
            .collect();

        let request_id = format!("req-{}", uuid::Uuid::new_v4());

        let request = GuardRelaxationRequest {
            id: request_id.clone(),
            guard_id,
            requested_by,
            reason,
            proposed_change,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(7),
            required_approvals,
            approval_signatures: vec![],
            state: RequestState::Submitted,
        };

        self.relaxation_requests
            .insert(request_id.clone(), Arc::new(RwLock::new(request)));

        Ok(request_id)
    }

    /// Approve a relaxation request with cryptographic signature
    pub fn approve_relaxation(
        &self,
        request_id: &str,
        approver_id: String,
        signing_key: &SigningKey,
    ) -> Result<()> {
        let request_ref = self
            .relaxation_requests
            .get(request_id)
            .ok_or_else(|| GovernanceError::RequestNotFound(request_id.to_string()))?;

        let mut request = request_ref.write();

        // Check if request is expired
        if Utc::now() > request.expires_at {
            request.state = RequestState::Expired;
            return Err(GovernanceError::RequestExpired);
        }

        // Check if already approved by this person
        if request
            .approval_signatures
            .iter()
            .any(|a| a.approver_id == approver_id)
        {
            return Ok(()); // Already approved, idempotent
        }

        // Sign the approval with ed25519
        let message = format!("{}-{}-{}", request.id, request.guard_id, approver_id);
        let signature = signing_key.sign(message.as_bytes());
        let verifying_key = signing_key.verifying_key();

        let approval = GuardApproval {
            approver_id: approver_id.clone(),
            signed_at: Utc::now(),
            signature: hex::encode(signature.to_bytes()),
            verifying_key: hex::encode(verifying_key.to_bytes()),
            metadata: HashMap::new(),
        };

        // Verify signature immediately
        approval.verify(&message)?;

        // Add approval to request
        request.approval_signatures.push(approval);

        // Check if we have quorum
        let guard = self
            .guards
            .get(&request.guard_id)
            .ok_or_else(|| GovernanceError::GuardNotFound(request.guard_id.clone()))?;

        if request.approval_signatures.len() >= guard.relaxation_policy.approval_quorum {
            // Check if minimum approval duration has passed
            let elapsed = Utc::now().signed_duration_since(request.created_at);
            let elapsed_ms = elapsed.num_milliseconds().max(0) as u64;
            if elapsed_ms >= guard.relaxation_policy.min_approval_duration_ms {
                request.state = RequestState::Approved;
            } else {
                request.state = RequestState::UnderReview;
            }
        } else {
            let pending =
                guard.relaxation_policy.approval_quorum - request.approval_signatures.len();
            request.state = RequestState::ApprovalsPending {
                pending_count: pending,
            };
        }

        Ok(())
    }

    /// Reject a relaxation request
    pub fn reject_relaxation(&self, request_id: &str, reason: String) -> Result<()> {
        let request_ref = self
            .relaxation_requests
            .get(request_id)
            .ok_or_else(|| GovernanceError::RequestNotFound(request_id.to_string()))?;

        let mut request = request_ref.write();
        request.state = RequestState::Rejected { reason };

        Ok(())
    }

    /// Check if relaxation is currently approved
    pub fn is_relaxation_approved(&self, request_id: &str) -> bool {
        self.relaxation_requests
            .get(request_id)
            .map(|req| matches!(req.read().state, RequestState::Approved))
            .unwrap_or(false)
    }

    /// Finalize approval after minimum duration has passed
    pub fn finalize_approval(&self, request_id: &str) -> Result<()> {
        let request_ref = self
            .relaxation_requests
            .get(request_id)
            .ok_or_else(|| GovernanceError::RequestNotFound(request_id.to_string()))?;

        let mut request = request_ref.write();

        if !matches!(request.state, RequestState::UnderReview) {
            return Ok(()); // Already in final state
        }

        let guard = self
            .guards
            .get(&request.guard_id)
            .ok_or_else(|| GovernanceError::GuardNotFound(request.guard_id.clone()))?;

        // Check quorum
        if request.approval_signatures.len() < guard.relaxation_policy.approval_quorum {
            return Err(GovernanceError::InsufficientApprovals {
                needed: guard.relaxation_policy.approval_quorum,
                current: request.approval_signatures.len(),
            });
        }

        // Check minimum duration
        let elapsed = Utc::now().signed_duration_since(request.created_at);
        let elapsed_ms = elapsed.num_milliseconds().max(0) as u64;
        if elapsed_ms < guard.relaxation_policy.min_approval_duration_ms {
            return Err(GovernanceError::ApprovalWindowNotOpen);
        }

        request.state = RequestState::Approved;
        Ok(())
    }

    /// Activate a relaxation (temporarily disable guard)
    pub fn activate_relaxation(
        &self,
        request_id: String,
        duration_ms: u64,
    ) -> Result<RelaxationWindow> {
        // Verify request is approved
        if !self.is_relaxation_approved(&request_id) {
            return Err(GovernanceError::RequestNotApproved);
        }

        // Get request to extract guard_id and reason
        let request_ref = self
            .relaxation_requests
            .get(&request_id)
            .ok_or_else(|| GovernanceError::RequestNotFound(request_id.clone()))?;

        let request = request_ref.read();
        let guard_id = request.guard_id.clone();
        let reason = request.reason.clone();

        // Get guard and verify it exists
        let guard = self
            .guards
            .get(&guard_id)
            .ok_or_else(|| GovernanceError::GuardNotFound(guard_id.clone()))?;

        // Create relaxation window
        let window = RelaxationWindow {
            request_id: request_id.clone(),
            guard_id: guard_id.clone(),
            started_at: Utc::now(),
            expires_at: Utc::now() + Duration::milliseconds(duration_ms as i64),
            reason,
        };

        self.active_relaxations
            .insert(request_id.clone(), window.clone());

        // Temporarily disable guard
        guard.temporarily_disable();

        // Archive request to history
        self.approval_history.write().push(request.clone());

        Ok(window)
    }

    /// Revoke a relaxation before expiration
    pub fn revoke_relaxation(&self, request_id: &str) -> Result<()> {
        let (_, window) = self
            .active_relaxations
            .remove(request_id)
            .ok_or_else(|| GovernanceError::RelaxationNotFound(request_id.to_string()))?;

        // Re-enable guard
        if let Some(guard) = self.guards.get(&window.guard_id) {
            guard.re_enable();
        }

        // Update request state
        if let Some(req_ref) = self.relaxation_requests.get(request_id) {
            let mut req = req_ref.write();
            req.state = RequestState::Revoked {
                reason: "Manual revocation".to_string(),
            };
        }

        Ok(())
    }

    /// Check and expire relaxations that have passed their window
    pub fn expire_relaxations(&self) -> Vec<String> {
        let now = Utc::now();
        let mut expired = Vec::new();

        self.active_relaxations.retain(|request_id, window| {
            if now > window.expires_at {
                expired.push(request_id.clone());

                // Re-enable guard
                if let Some(guard) = self.guards.get(&window.guard_id) {
                    guard.re_enable();
                }

                false // Remove from active relaxations
            } else {
                true // Keep active
            }
        });

        expired
    }

    /// Get guard status
    pub fn guard_status(&self, guard_id: &str) -> Result<GuardStatus> {
        let guard = self
            .guards
            .get(guard_id)
            .ok_or_else(|| GovernanceError::GuardNotFound(guard_id.to_string()))?;

        let active_relaxation = self
            .active_relaxations
            .iter()
            .find(|entry| entry.value().guard_id == guard_id)
            .map(|entry| entry.value().clone());

        Ok(GuardStatus {
            id: guard.id.clone(),
            name: guard.name.clone(),
            enforced: guard.is_enforced(),
            active_relaxation,
            criticality: format!("{:?}", guard.criticality),
        })
    }

    /// Get request status
    pub fn request_status(&self, request_id: &str) -> Result<RequestStatus> {
        let request_ref = self
            .relaxation_requests
            .get(request_id)
            .ok_or_else(|| GovernanceError::RequestNotFound(request_id.to_string()))?;

        let request = request_ref.read();

        Ok(RequestStatus {
            id: request.id.clone(),
            guard_id: request.guard_id.clone(),
            state: request.state.clone(),
            approvals_received: request.approval_signatures.len(),
            approvals_needed: self
                .guards
                .get(&request.guard_id)
                .map(|g| g.relaxation_policy.approval_quorum)
                .unwrap_or(0),
            created_at: request.created_at,
            expires_at: request.expires_at,
        })
    }

    /// List all guards
    pub fn list_guards(&self) -> Vec<String> {
        self.guards
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// List all active relaxations
    pub fn list_active_relaxations(&self) -> Vec<RelaxationWindow> {
        self.active_relaxations
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}

impl Default for GovernanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
pub struct GuardStatus {
    pub id: String,
    pub name: String,
    pub enforced: bool,
    pub active_relaxation: Option<RelaxationWindow>,
    pub criticality: String,
}

#[derive(Debug, Serialize)]
pub struct RequestStatus {
    pub id: String,
    pub guard_id: String,
    pub state: RequestState,
    pub approvals_received: usize,
    pub approvals_needed: usize,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn create_test_guard(id: &str, criticality: Criticality) -> Guard {
        Guard::new(
            id.to_string(),
            format!("Test Guard {}", id),
            GuardType::ApprovalChain {
                required_signers: vec!["CFO".to_string(), "Legal".to_string()],
            },
            criticality,
        )
        .with_required_approvers(vec![
            "CFO".to_string(),
            "Legal".to_string(),
            "CTO".to_string(),
        ])
    }

    #[test]
    fn test_critical_guard_requires_quorum() {
        let guard = create_test_guard("critical-1", Criticality::Critical);

        // Critical guards should require 3 approvals
        assert_eq!(guard.relaxation_policy.approval_quorum, 3);
        assert!(guard.relaxation_policy.requires_multi_party);
        assert_eq!(guard.relaxation_policy.min_approval_duration_ms, 86400000); // 24 hours
    }

    #[test]
    fn test_high_criticality_guard() {
        let guard = create_test_guard("high-1", Criticality::High);

        // High criticality should require 2 approvals
        assert_eq!(guard.relaxation_policy.approval_quorum, 2);
        assert!(guard.relaxation_policy.requires_multi_party);
    }

    #[test]
    fn test_guard_registration() {
        let engine = GovernanceEngine::new();
        let guard = create_test_guard("test-1", Criticality::Medium);

        engine.register_guard(guard).unwrap();

        let retrieved = engine.get_guard("test-1").unwrap();
        assert_eq!(retrieved.id, "test-1");
    }

    #[test]
    fn test_relaxation_request_creation() {
        let engine = GovernanceEngine::new();
        let guard = create_test_guard("test-2", Criticality::Medium);

        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-2".to_string(),
                "alice".to_string(),
                "Need to bypass for emergency".to_string(),
                "Disable for 1 hour".to_string(),
            )
            .unwrap();

        assert!(request_id.starts_with("req-"));
    }

    #[test]
    fn test_signature_verification() {
        let engine = GovernanceEngine::new();
        let guard = create_test_guard("test-3", Criticality::Low);

        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-3".to_string(),
                "bob".to_string(),
                "Testing signatures".to_string(),
                "Temporary relaxation".to_string(),
            )
            .unwrap();

        // Generate signing key
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        // Approve with signature
        engine
            .approve_relaxation(&request_id, "CFO".to_string(), &signing_key)
            .unwrap();

        // Verify signature was stored
        let request_ref = engine.relaxation_requests.get(&request_id).unwrap();
        let request = request_ref.read();

        assert_eq!(request.approval_signatures.len(), 1);
        assert_eq!(request.approval_signatures[0].approver_id, "CFO");

        // Verify the signature is valid
        let message = format!("{}-{}-{}", request.id, request.guard_id, "CFO");
        request.approval_signatures[0].verify(&message).unwrap();
    }

    #[test]
    fn test_quorum_approval() {
        let engine = GovernanceEngine::new();
        let guard =
            create_test_guard("test-4", Criticality::Critical).with_required_approvers(vec![
                "CFO".to_string(),
                "Legal".to_string(),
                "CTO".to_string(),
            ]);

        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-4".to_string(),
                "carol".to_string(),
                "Critical change needed".to_string(),
                "Relax compliance rule".to_string(),
            )
            .unwrap();

        let mut csprng = OsRng;

        // First approval
        let key1 = SigningKey::generate(&mut csprng);
        engine
            .approve_relaxation(&request_id, "CFO".to_string(), &key1)
            .unwrap();

        // Not yet approved (need 3)
        assert!(!engine.is_relaxation_approved(&request_id));

        // Second approval
        let key2 = SigningKey::generate(&mut csprng);
        engine
            .approve_relaxation(&request_id, "Legal".to_string(), &key2)
            .unwrap();

        // Still not approved (need 3)
        assert!(!engine.is_relaxation_approved(&request_id));

        // Third approval
        let key3 = SigningKey::generate(&mut csprng);
        engine
            .approve_relaxation(&request_id, "CTO".to_string(), &key3)
            .unwrap();

        // Need to wait for minimum duration, so state is UnderReview
        let status = engine.request_status(&request_id).unwrap();
        assert_eq!(status.approvals_received, 3);
        assert!(matches!(status.state, RequestState::UnderReview));
    }

    #[test]
    fn test_relaxation_window_activation() {
        let engine = GovernanceEngine::new();

        // Use low criticality for instant approval (no time delay)
        let guard = create_test_guard("test-5", Criticality::Low);
        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-5".to_string(),
                "dave".to_string(),
                "Quick test".to_string(),
                "Temporary relaxation".to_string(),
            )
            .unwrap();

        // Approve with required quorum (1 for low criticality)
        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        engine
            .approve_relaxation(&request_id, "TeamLead".to_string(), &key)
            .unwrap();

        // Should be approved immediately (low criticality has 0ms min duration)
        assert!(engine.is_relaxation_approved(&request_id));

        // Activate relaxation
        let window = engine
            .activate_relaxation(request_id.clone(), 1000)
            .unwrap();

        assert_eq!(window.guard_id, "test-5");

        // Guard should be disabled
        let guard = engine.get_guard("test-5").unwrap();
        assert!(!guard.is_enforced());
    }

    #[test]
    fn test_relaxation_window_expiration() {
        let engine = GovernanceEngine::new();

        let guard = create_test_guard("test-6", Criticality::Low);
        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-6".to_string(),
                "eve".to_string(),
                "Expiration test".to_string(),
                "Will expire".to_string(),
            )
            .unwrap();

        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        engine
            .approve_relaxation(&request_id, "Approver".to_string(), &key)
            .unwrap();

        // Activate with very short duration
        let window = engine.activate_relaxation(request_id.clone(), 1).unwrap();

        // Guard is disabled
        let guard = engine.get_guard("test-6").unwrap();
        assert!(!guard.is_enforced());

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_millis(2));

        // Manually expire (in production this would be a background task)
        let expired = engine.expire_relaxations();
        assert_eq!(expired.len(), 1);

        // Guard should be re-enabled
        assert!(guard.is_enforced());
    }

    #[test]
    fn test_relaxation_revocation() {
        let engine = GovernanceEngine::new();

        let guard = create_test_guard("test-7", Criticality::Low);
        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-7".to_string(),
                "frank".to_string(),
                "Revocation test".to_string(),
                "Will be revoked".to_string(),
            )
            .unwrap();

        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);
        engine
            .approve_relaxation(&request_id, "Approver".to_string(), &key)
            .unwrap();

        // Activate relaxation
        engine
            .activate_relaxation(request_id.clone(), 60000)
            .unwrap();

        // Guard is disabled
        let guard = engine.get_guard("test-7").unwrap();
        assert!(!guard.is_enforced());

        // Revoke relaxation
        engine.revoke_relaxation(&request_id).unwrap();

        // Guard should be re-enabled
        assert!(guard.is_enforced());

        // Request should be marked as revoked
        let status = engine.request_status(&request_id).unwrap();
        assert!(matches!(status.state, RequestState::Revoked { .. }));
    }

    #[test]
    fn test_immutable_guard_rejection() {
        let engine = GovernanceEngine::new();

        // Critical guards are not mutable by default
        let mut guard = create_test_guard("test-8", Criticality::Critical);
        guard.is_mutable = false;

        engine.register_guard(guard).unwrap();

        let result = engine.request_relaxation(
            "test-8".to_string(),
            "hacker".to_string(),
            "Try to bypass".to_string(),
            "Should fail".to_string(),
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GovernanceError::GuardImmutable(_)
        ));
    }

    #[test]
    fn test_guard_status() {
        let engine = GovernanceEngine::new();

        let guard = create_test_guard("test-9", Criticality::High);
        engine.register_guard(guard).unwrap();

        let status = engine.guard_status("test-9").unwrap();

        assert_eq!(status.id, "test-9");
        assert!(status.enforced);
        assert!(status.active_relaxation.is_none());
        assert_eq!(status.criticality, "High");
    }

    #[test]
    fn test_idempotent_approval() {
        let engine = GovernanceEngine::new();
        let guard = create_test_guard("test-10", Criticality::Medium);

        engine.register_guard(guard).unwrap();

        let request_id = engine
            .request_relaxation(
                "test-10".to_string(),
                "grace".to_string(),
                "Idempotency test".to_string(),
                "Double approval".to_string(),
            )
            .unwrap();

        let mut csprng = OsRng;
        let key = SigningKey::generate(&mut csprng);

        // Approve once
        engine
            .approve_relaxation(&request_id, "Approver1".to_string(), &key)
            .unwrap();

        // Approve again with same approver - should be idempotent
        engine
            .approve_relaxation(&request_id, "Approver1".to_string(), &key)
            .unwrap();

        // Should only have 1 approval
        let status = engine.request_status(&request_id).unwrap();
        assert_eq!(status.approvals_received, 1);
    }
}

// ============================================================================
// PROPERTY-BASED TESTS
// ============================================================================

#[cfg(test)]
mod prop_tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::OsRng;

    proptest! {
        #[test]
        fn prop_any_criticality_has_valid_quorum(
            criticality_level in 0usize..=3
        ) {
            let criticality = match criticality_level {
                0 => Criticality::Low,
                1 => Criticality::Medium,
                2 => Criticality::High,
                _ => Criticality::Critical,
            };

            let guard = Guard::new(
                "test-guard".to_string(),
                "Test".to_string(),
                GuardType::SafetyInterlock { condition: "test".to_string() },
                criticality.clone(),
            );

            // Property: Quorum requirements increase with criticality
            let quorum = guard.relaxation_policy.approval_quorum;
            match criticality {
                Criticality::Low => prop_assert_eq!(quorum, 1),
                Criticality::Medium => prop_assert_eq!(quorum, 1),
                Criticality::High => prop_assert_eq!(quorum, 2),
                Criticality::Critical => prop_assert_eq!(quorum, 3),
            }
        }

        #[test]
        fn prop_signature_verification_deterministic(
            approver_id in "[a-z]{3,10}"
        ) {
            let engine = GovernanceEngine::new();
            let guard = create_test_guard("test", Criticality::Low);
            engine.register_guard(guard).unwrap();

            let request_id = engine.request_relaxation(
                "test".to_string(),
                "requester".to_string(),
                "reason".to_string(),
                "change".to_string(),
            ).unwrap();

            let mut csprng = OsRng;
            let signing_key = SigningKey::generate(&mut csprng);

            // Approve twice with same key and approver
            engine.approve_relaxation(&request_id, approver_id.clone(), &signing_key).unwrap();
            engine.approve_relaxation(&request_id, approver_id.clone(), &signing_key).unwrap();

            let status = engine.request_status(&request_id).unwrap();

            // Property: Idempotent approval (should only count once)
            prop_assert_eq!(status.approvals_received, 1);
        }

        #[test]
        fn prop_relaxation_window_expires_correctly(
            duration_ms in 1u64..=1000
        ) {
            let engine = GovernanceEngine::new();
            let guard = create_test_guard("test", Criticality::Low);
            engine.register_guard(guard).unwrap();

            let request_id = engine.request_relaxation(
                "test".to_string(),
                "user".to_string(),
                "test".to_string(),
                "test".to_string(),
            ).unwrap();

            let mut csprng = OsRng;
            let signing_key = SigningKey::generate(&mut csprng);
            engine.approve_relaxation(&request_id, "approver".to_string(), &signing_key).unwrap();

            let window = engine.activate_relaxation(request_id.clone(), duration_ms).unwrap();

            // Property: Window has correct expiration time
            let expected_expiry = window.started_at.checked_add_signed(Duration::milliseconds(duration_ms as i64)).unwrap();
            prop_assert_eq!(window.expires_at, expected_expiry);
        }

        #[test]
        fn prop_guard_enforcement_atomic(
            enforce in any::<bool>()
        ) {
            let guard = create_test_guard("test", Criticality::Medium);

            if enforce {
                guard.re_enable();
            } else {
                guard.temporarily_disable();
            }

            // Property: Guard state matches command
            prop_assert_eq!(guard.is_enforced(), enforce);
        }

        #[test]
        fn prop_multiple_approvals_accumulate(
            approval_count in 1usize..=5
        ) {
            let engine = GovernanceEngine::new();
            let guard = create_test_guard("test", Criticality::Critical);
            engine.register_guard(guard).unwrap();

            let request_id = engine.request_relaxation(
                "test".to_string(),
                "user".to_string(),
                "test".to_string(),
                "test".to_string(),
            ).unwrap();

            let mut csprng = OsRng;

            for i in 0..approval_count {
                let key = SigningKey::generate(&mut csprng);
                engine.approve_relaxation(&request_id, format!("approver-{}", i), &key).unwrap();
            }

            let status = engine.request_status(&request_id).unwrap();

            // Property: All approvals accumulated
            prop_assert_eq!(status.approvals_received, approval_count);
        }

        #[test]
        fn prop_guard_type_serialization_stable(
            guard_type_variant in 0usize..=4
        ) {
            let guard_type = match guard_type_variant {
                0 => GuardType::ApprovalChain { required_signers: vec!["CFO".to_string()] },
                1 => GuardType::SafetyInterlock { condition: "test".to_string() },
                2 => GuardType::DataResidency { allowed_regions: vec!["US".to_string()] },
                3 => GuardType::PerformanceBound { max_ticks: 8 },
                _ => GuardType::ComplianceRule { regulations: vec!["HIPAA".to_string()] },
            };

            let guard1 = Guard::new(
                "test".to_string(),
                "Test".to_string(),
                guard_type.clone(),
                Criticality::Medium,
            );

            let guard2 = Guard::new(
                "test".to_string(),
                "Test".to_string(),
                guard_type,
                Criticality::Medium,
            );

            // Property: Same guard type produces consistent structure
            let serialized1 = serde_json::to_string(&guard1).unwrap();
            let serialized2 = serde_json::to_string(&guard2).unwrap();
            prop_assert_eq!(serialized1, serialized2);
        }
    }
}
