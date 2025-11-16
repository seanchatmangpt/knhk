//! Governance Layer Integration for Zero-Knowledge Proofs
//!
//! This module integrates ZK proofs with KNHK's governance layer:
//! - ΔΣ overlay safety proofs (without revealing overlay contents)
//! - Policy lattice compliance proofs
//! - Session isolation proofs
//! - Verifiable counterfactual analysis

use super::{
    ZkProver, ZkVerifier, PrivateInputs, PublicInputs, ProofSystem,
    ZkProof, ZkError, ZkResult, ProverConfig,
};
use sha3::{Sha3_256, Digest};
use std::collections::HashMap;
use tracing::{info, debug, instrument};

/// Governance proof types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceProofType {
    /// Prove ΔΣ overlay is safe without revealing overlay
    DeltaSigmaOverlaySafety,
    /// Prove policy lattice compliance
    PolicyLatticeCompliance,
    /// Prove session isolation
    SessionIsolation,
    /// Prove counterfactual query safety
    CounterfactualSafety,
}

/// Governance proof request
#[derive(Debug, Clone)]
pub struct GovernanceProofRequest {
    pub proof_type: GovernanceProofType,
    pub workflow_id: String,
    pub session_id: String,
    pub private_data: HashMap<String, Vec<u8>>,
    pub public_data: HashMap<String, Vec<u8>>,
}

/// Governance proof verifier
pub struct GovernanceVerifier {
    proof_system: ProofSystem,
    verifier: ZkVerifier,
}

impl GovernanceVerifier {
    /// Create new governance verifier
    pub fn new(proof_system: ProofSystem) -> Self {
        Self {
            proof_system,
            verifier: ZkVerifier::new(proof_system),
        }
    }

    /// Verify governance proof
    #[instrument(skip(self, proof), fields(proof_type = ?proof.circuit_id))]
    pub fn verify_governance_proof(&self, proof: &ZkProof) -> ZkResult<bool> {
        info!("Verifying governance proof: {}", proof.circuit_id);

        // Verify proof using standard ZK verification
        let valid = self.verifier.verify(proof, &proof.public_inputs)?;

        if !valid {
            return Ok(false);
        }

        // Additional governance-specific checks
        match self.extract_proof_type(&proof.circuit_id)? {
            GovernanceProofType::DeltaSigmaOverlaySafety => {
                self.verify_overlay_safety_proof(proof)
            }
            GovernanceProofType::PolicyLatticeCompliance => {
                self.verify_policy_lattice_proof(proof)
            }
            GovernanceProofType::SessionIsolation => {
                self.verify_session_isolation_proof(proof)
            }
            GovernanceProofType::CounterfactualSafety => {
                self.verify_counterfactual_safety_proof(proof)
            }
        }
    }

    /// Extract proof type from circuit ID
    fn extract_proof_type(&self, circuit_id: &str) -> ZkResult<GovernanceProofType> {
        match circuit_id {
            id if id.contains("delta_sigma_overlay") => {
                Ok(GovernanceProofType::DeltaSigmaOverlaySafety)
            }
            id if id.contains("policy_lattice") => {
                Ok(GovernanceProofType::PolicyLatticeCompliance)
            }
            id if id.contains("session_isolation") => {
                Ok(GovernanceProofType::SessionIsolation)
            }
            id if id.contains("counterfactual") => {
                Ok(GovernanceProofType::CounterfactualSafety)
            }
            _ => Err(ZkError::InvalidInputs(format!("Unknown proof type: {}", circuit_id))),
        }
    }

    /// Verify ΔΣ overlay safety proof
    fn verify_overlay_safety_proof(&self, proof: &ZkProof) -> ZkResult<bool> {
        // Check that overlay hash is present in public inputs
        let overlay_hash = proof.public_inputs.get("overlay_hash")
            .ok_or_else(|| ZkError::VerificationFailed("Missing overlay_hash".into()))?;

        if overlay_hash.is_empty() {
            return Ok(false);
        }

        // Check safety constraints are met
        let safety_check = proof.public_inputs.get("safety_check")
            .and_then(|v| v.first().copied())
            .unwrap_or(0);

        Ok(safety_check == 1)
    }

    /// Verify policy lattice compliance proof
    fn verify_policy_lattice_proof(&self, proof: &ZkProof) -> ZkResult<bool> {
        // Check lattice level is valid
        let lattice_level = proof.public_inputs.get("lattice_level")
            .and_then(|v| v.first().copied())
            .unwrap_or(0);

        // Lattice levels: 0=system, 1=domain, 2=workflow, 3=task
        if lattice_level > 3 {
            return Ok(false);
        }

        // Check compliance result
        let compliant = proof.public_inputs.get("compliant")
            .and_then(|v| v.first().copied())
            .unwrap_or(0);

        Ok(compliant == 1)
    }

    /// Verify session isolation proof
    fn verify_session_isolation_proof(&self, proof: &ZkProof) -> ZkResult<bool> {
        // Check session ID is present
        let session_id = proof.public_inputs.get("session_id")
            .ok_or_else(|| ZkError::VerificationFailed("Missing session_id".into()))?;

        if session_id.is_empty() {
            return Ok(false);
        }

        // Check isolation guarantee
        let isolated = proof.public_inputs.get("isolated")
            .and_then(|v| v.first().copied())
            .unwrap_or(0);

        Ok(isolated == 1)
    }

    /// Verify counterfactual safety proof
    fn verify_counterfactual_safety_proof(&self, proof: &ZkProof) -> ZkResult<bool> {
        // Check counterfactual query hash
        let query_hash = proof.public_inputs.get("query_hash")
            .ok_or_else(|| ZkError::VerificationFailed("Missing query_hash".into()))?;

        if query_hash.is_empty() {
            return Ok(false);
        }

        // Check safety result
        let safe = proof.public_inputs.get("safe")
            .and_then(|v| v.first().copied())
            .unwrap_or(0);

        Ok(safe == 1)
    }
}

/// Generate ΔΣ overlay safety proof
#[instrument(skip(overlay_delta, sigma_base), fields(workflow_id = %workflow_id))]
pub async fn prove_overlay_safety(
    workflow_id: &str,
    overlay_delta: &[u8],
    sigma_base: &[u8],
    proof_system: ProofSystem,
) -> ZkResult<ZkProof> {
    info!("Generating ΔΣ overlay safety proof for workflow: {}", workflow_id);

    // Private inputs: overlay delta and base sigma (hidden)
    let private_inputs = PrivateInputs::new()
        .add("overlay_delta", overlay_delta.to_vec())
        .add("sigma_base", sigma_base.to_vec());

    // Public inputs: overlay hash and safety result
    let overlay_hash = hash_overlay(overlay_delta, sigma_base);
    let safety_check = check_overlay_safety(overlay_delta, sigma_base);

    let public_inputs = PublicInputs::new()
        .add("workflow_id", workflow_id.as_bytes().to_vec())
        .add("overlay_hash", overlay_hash)
        .add("safety_check", vec![safety_check as u8]);

    // Generate proof
    let prover = ZkProver::new(proof_system)
        .with_circuit("delta_sigma_overlay_safety")
        .build()?;

    prover.prove(&private_inputs, &public_inputs).await
}

/// Generate policy lattice compliance proof
#[instrument(skip(policy_rules, workflow_state), fields(workflow_id = %workflow_id))]
pub async fn prove_policy_compliance(
    workflow_id: &str,
    lattice_level: u8,
    policy_rules: &[u8],
    workflow_state: &[u8],
    proof_system: ProofSystem,
) -> ZkResult<ZkProof> {
    info!("Generating policy lattice compliance proof for workflow: {}", workflow_id);

    // Private inputs: policy rules and workflow state
    let private_inputs = PrivateInputs::new()
        .add("policy_rules", policy_rules.to_vec())
        .add("workflow_state", workflow_state.to_vec());

    // Public inputs: lattice level and compliance result
    let compliant = check_policy_compliance(policy_rules, workflow_state, lattice_level);

    let public_inputs = PublicInputs::new()
        .add("workflow_id", workflow_id.as_bytes().to_vec())
        .add("lattice_level", vec![lattice_level])
        .add("compliant", vec![compliant as u8]);

    // Generate proof
    let prover = ZkProver::new(proof_system)
        .with_circuit("policy_lattice_compliance")
        .build()?;

    prover.prove(&private_inputs, &public_inputs).await
}

/// Generate session isolation proof
#[instrument(skip(session_state, other_sessions), fields(session_id = %session_id))]
pub async fn prove_session_isolation(
    session_id: &str,
    session_state: &[u8],
    other_sessions: &[Vec<u8>],
    proof_system: ProofSystem,
) -> ZkResult<ZkProof> {
    info!("Generating session isolation proof for session: {}", session_id);

    // Private inputs: session state and other sessions
    let mut private_inputs = PrivateInputs::new()
        .add("session_state", session_state.to_vec());

    for (i, other) in other_sessions.iter().enumerate() {
        private_inputs = private_inputs.add(&format!("other_session_{}", i), other.clone());
    }

    // Public inputs: session ID and isolation result
    let isolated = check_session_isolation(session_state, other_sessions);

    let public_inputs = PublicInputs::new()
        .add("session_id", session_id.as_bytes().to_vec())
        .add("isolated", vec![isolated as u8]);

    // Generate proof
    let prover = ZkProver::new(proof_system)
        .with_circuit("session_isolation")
        .build()?;

    prover.prove(&private_inputs, &public_inputs).await
}

/// Generate counterfactual safety proof
#[instrument(skip(sigma_state, counterfactual_query), fields(workflow_id = %workflow_id))]
pub async fn prove_counterfactual_safety(
    workflow_id: &str,
    sigma_state: &[u8],
    counterfactual_query: &[u8],
    proof_system: ProofSystem,
) -> ZkResult<ZkProof> {
    info!("Generating counterfactual safety proof for workflow: {}", workflow_id);

    // Private inputs: Σ state and counterfactual query (hidden)
    let private_inputs = PrivateInputs::new()
        .add("sigma_state", sigma_state.to_vec())
        .add("counterfactual_query", counterfactual_query.to_vec());

    // Public inputs: query hash and safety result
    let query_hash = hash_counterfactual_query(counterfactual_query);
    let safe = check_counterfactual_safety(sigma_state, counterfactual_query);

    let public_inputs = PublicInputs::new()
        .add("workflow_id", workflow_id.as_bytes().to_vec())
        .add("query_hash", query_hash)
        .add("safe", vec![safe as u8]);

    // Generate proof
    let prover = ZkProver::new(proof_system)
        .with_circuit("counterfactual_safety")
        .build()?;

    prover.prove(&private_inputs, &public_inputs).await
}

/// Hash overlay delta and base sigma
fn hash_overlay(overlay_delta: &[u8], sigma_base: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(overlay_delta);
    hasher.update(sigma_base);
    hasher.update(b"OVERLAY_HASH");
    hasher.finalize().to_vec()
}

/// Check overlay safety
fn check_overlay_safety(overlay_delta: &[u8], sigma_base: &[u8]) -> bool {
    // Simplified safety check
    // In production: check that overlay doesn't violate invariants

    // Example: overlay size must be reasonable
    if overlay_delta.len() > 1024 * 1024 {
        return false;
    }

    // Example: overlay must not completely override base
    if overlay_delta.len() > sigma_base.len() * 2 {
        return false;
    }

    true
}

/// Check policy compliance
fn check_policy_compliance(policy_rules: &[u8], workflow_state: &[u8], lattice_level: u8) -> bool {
    // Simplified compliance check
    // In production: evaluate policy rules against state

    if policy_rules.is_empty() || workflow_state.is_empty() {
        return false;
    }

    // Check lattice level is valid
    if lattice_level > 3 {
        return false;
    }

    true
}

/// Check session isolation
fn check_session_isolation(session_state: &[u8], other_sessions: &[Vec<u8>]) -> bool {
    // Simplified isolation check
    // In production: verify no data leakage between sessions

    // Check session state doesn't match any other session
    for other in other_sessions {
        if session_state == other {
            return false;
        }
    }

    true
}

/// Hash counterfactual query
fn hash_counterfactual_query(query: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(query);
    hasher.update(b"COUNTERFACTUAL_QUERY");
    hasher.finalize().to_vec()
}

/// Check counterfactual safety
fn check_counterfactual_safety(sigma_state: &[u8], counterfactual_query: &[u8]) -> bool {
    // Simplified safety check
    // In production: verify query doesn't violate time-travel constraints

    if sigma_state.is_empty() || counterfactual_query.is_empty() {
        return false;
    }

    // Example: query must not be too large
    if counterfactual_query.len() > 1024 {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_overlay() {
        let overlay = b"overlay_delta";
        let sigma = b"sigma_base";

        let hash = hash_overlay(overlay, sigma);
        assert_eq!(hash.len(), 32);

        // Same inputs produce same hash
        let hash2 = hash_overlay(overlay, sigma);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_check_overlay_safety() {
        let overlay = vec![1u8; 100];
        let sigma = vec![2u8; 200];

        assert!(check_overlay_safety(&overlay, &sigma));

        // Too large overlay
        let large_overlay = vec![1u8; 2_000_000];
        assert!(!check_overlay_safety(&large_overlay, &sigma));
    }

    #[test]
    fn test_check_policy_compliance() {
        let rules = b"policy_rules";
        let state = b"workflow_state";

        assert!(check_policy_compliance(rules, state, 0));
        assert!(check_policy_compliance(rules, state, 3));
        assert!(!check_policy_compliance(rules, state, 4));

        // Empty inputs
        assert!(!check_policy_compliance(b"", state, 0));
        assert!(!check_policy_compliance(rules, b"", 0));
    }

    #[test]
    fn test_check_session_isolation() {
        let session = vec![1, 2, 3];
        let other_sessions = vec![
            vec![4, 5, 6],
            vec![7, 8, 9],
        ];

        assert!(check_session_isolation(&session, &other_sessions));

        // Duplicate session
        let other_with_dup = vec![
            vec![4, 5, 6],
            vec![1, 2, 3], // Same as session
        ];

        assert!(!check_session_isolation(&session, &other_with_dup));
    }

    #[test]
    fn test_hash_counterfactual_query() {
        let query = b"SELECT * FROM past WHERE time < now()";

        let hash = hash_counterfactual_query(query);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_check_counterfactual_safety() {
        let sigma = b"sigma_state";
        let query = b"counterfactual_query";

        assert!(check_counterfactual_safety(sigma, query));

        // Too large query
        let large_query = vec![1u8; 2000];
        assert!(!check_counterfactual_safety(sigma, &large_query));

        // Empty inputs
        assert!(!check_counterfactual_safety(b"", query));
        assert!(!check_counterfactual_safety(sigma, b""));
    }

    #[test]
    fn test_governance_verifier() {
        let verifier = GovernanceVerifier::new(ProofSystem::Groth16);

        // Test proof type extraction
        assert_eq!(
            verifier.extract_proof_type("delta_sigma_overlay_safety").unwrap(),
            GovernanceProofType::DeltaSigmaOverlaySafety
        );

        assert_eq!(
            verifier.extract_proof_type("policy_lattice_compliance").unwrap(),
            GovernanceProofType::PolicyLatticeCompliance
        );
    }
}
