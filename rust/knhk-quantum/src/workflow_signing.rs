//! Quantum-Safe Workflow Signing Module
//!
//! Provides cryptographic signing and verification for YAWL workflow specifications
//! using ML-DSA (Dilithium) post-quantum signatures. Supports delegation certificates
//! for multi-party signing.
//!
//! # DOCTRINE ALIGNMENT
//! - Principle: Q (Hard Invariants) - Workflow authenticity is mandatory
//! - Covenant: 2 (Invariants Are Law) - No unsigned workflows in production
//! - Validation: Signature verification + Weaver schema

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use crate::sig::{QuantumSig, DilithiumSig, SigError};

/// Workflow signing errors
#[derive(Error, Debug)]
pub enum WorkflowSigningError {
    #[error("Signature error: {0}")]
    SignatureError(#[from] SigError),
    #[error("Invalid workflow specification: {0}")]
    InvalidSpecification(String),
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Invalid delegation certificate: {0}")]
    InvalidDelegation(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, WorkflowSigningError>;

/// YAWL Specification (simplified for signing)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YAWLSpecification {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub turtle_content: String, // RDF/Turtle representation
    pub created_at: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl YAWLSpecification {
    /// Create a new YAWL specification
    pub fn new(
        id: String,
        name: String,
        version: String,
        description: String,
        turtle_content: String,
    ) -> Self {
        Self {
            id,
            name,
            version,
            description,
            turtle_content,
            created_at: Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Compute hash of specification for signing
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(&self.name);
        hasher.update(&self.version);
        hasher.update(&self.turtle_content);
        hasher.finalize().to_vec()
    }
}

/// Signed YAWL specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedSpecification {
    pub specification: YAWLSpecification,
    pub signature: Vec<u8>,
    pub signer_public_key_id: String,
    pub algorithm: String, // "Dilithium3"
    pub signed_at: DateTime<Utc>,
}

impl SignedSpecification {
    /// Verify the signature on this specification
    pub fn verify(&self, public_key: &[u8]) -> Result<bool> {
        let sig_engine = DilithiumSig::new();
        let spec_hash = self.specification.hash();
        Ok(sig_engine.verify(public_key, &spec_hash, &self.signature)?)
    }
}

/// Delegation certificate for multi-party signing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelegationCertificate {
    pub id: String,
    pub issuer_public_key_id: String,
    pub delegate_public_key: Vec<u8>,
    pub delegate_name: String,
    pub permissions: Vec<String>, // e.g., ["sign_workflows", "create_tasks"]
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub signature: Vec<u8>,
    pub algorithm: String,
}

impl DelegationCertificate {
    /// Compute hash of certificate for signing
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(&self.issuer_public_key_id);
        hasher.update(&self.delegate_public_key);
        hasher.update(&self.delegate_name);
        for permission in &self.permissions {
            hasher.update(permission);
        }
        hasher.update(self.valid_from.to_rfc3339().as_bytes());
        hasher.update(self.valid_until.to_rfc3339().as_bytes());
        hasher.finalize().to_vec()
    }

    /// Verify the certificate signature
    pub fn verify(&self, issuer_public_key: &[u8]) -> Result<bool> {
        let sig_engine = DilithiumSig::new();
        let cert_hash = self.hash();
        Ok(sig_engine.verify(issuer_public_key, &cert_hash, &self.signature)?)
    }

    /// Check if certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.valid_from && now <= self.valid_until
    }
}

/// Workflow signer with key management
pub struct WorkflowSigner {
    private_key: Vec<u8>,
    public_key: Vec<u8>,
    public_key_id: String,
    sig_engine: DilithiumSig,
}

impl WorkflowSigner {
    /// Create a new workflow signer with generated keys
    pub fn new() -> Result<Self> {
        let sig_engine = DilithiumSig::new();
        let (public_key, private_key) = sig_engine.keygen()?;
        let public_key_id = format!("pk_{}", hex::encode(&public_key[..16]));

        Ok(Self {
            private_key,
            public_key,
            public_key_id,
            sig_engine,
        })
    }

    /// Create signer with existing keys
    pub fn with_keys(
        private_key: Vec<u8>,
        public_key: Vec<u8>,
        public_key_id: String,
    ) -> Self {
        Self {
            private_key,
            public_key,
            public_key_id,
            sig_engine: DilithiumSig::new(),
        }
    }

    /// Sign a YAWL specification
    pub fn sign_specification(&self, spec: &YAWLSpecification) -> Result<SignedSpecification> {
        let spec_hash = spec.hash();
        let signature = self.sig_engine.sign(&self.private_key, &spec_hash)?;

        Ok(SignedSpecification {
            specification: spec.clone(),
            signature,
            signer_public_key_id: self.public_key_id.clone(),
            algorithm: "Dilithium3".to_string(),
            signed_at: Utc::now(),
        })
    }

    /// Verify a signed specification
    pub fn verify_specification(&self, signed_spec: &SignedSpecification) -> Result<()> {
        if !signed_spec.verify(&self.public_key)? {
            return Err(WorkflowSigningError::VerificationFailed);
        }
        Ok(())
    }

    /// Create a delegation certificate for another signer
    pub fn create_delegation_cert(
        &self,
        delegate_public_key: &[u8],
        delegate_name: String,
        permissions: Vec<String>,
        valid_days: i64,
    ) -> Result<DelegationCertificate> {
        let now = Utc::now();
        let valid_until = now + chrono::Duration::days(valid_days);

        let mut cert = DelegationCertificate {
            id: format!("cert_{}", hex::encode(&delegate_public_key[..8])),
            issuer_public_key_id: self.public_key_id.clone(),
            delegate_public_key: delegate_public_key.to_vec(),
            delegate_name,
            permissions,
            valid_from: now,
            valid_until,
            signature: Vec::new(),
            algorithm: "Dilithium3".to_string(),
        };

        let cert_hash = cert.hash();
        cert.signature = self.sig_engine.sign(&self.private_key, &cert_hash)?;

        Ok(cert)
    }

    /// Verify a delegation certificate
    pub fn verify_delegation_cert(&self, cert: &DelegationCertificate) -> Result<()> {
        if !cert.verify(&self.public_key)? {
            return Err(WorkflowSigningError::InvalidDelegation(
                "Signature verification failed".to_string()
            ));
        }
        if !cert.is_valid() {
            return Err(WorkflowSigningError::InvalidDelegation(
                "Certificate has expired".to_string()
            ));
        }
        Ok(())
    }

    /// Get public key
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Get public key ID
    pub fn public_key_id(&self) -> &str {
        &self.public_key_id
    }
}

impl Default for WorkflowSigner {
    fn default() -> Self {
        Self::new().expect("Failed to create default workflow signer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yawl_specification() {
        let spec = YAWLSpecification::new(
            "spec_001".to_string(),
            "Test Workflow".to_string(),
            "1.0.0".to_string(),
            "A test workflow".to_string(),
            "@prefix yawl: <http://example.org/yawl#> .".to_string(),
        );
        assert_eq!(spec.id, "spec_001");
        assert_eq!(spec.name, "Test Workflow");
    }

    #[test]
    fn test_specification_hash() {
        let spec = YAWLSpecification::new(
            "spec_001".to_string(),
            "Test Workflow".to_string(),
            "1.0.0".to_string(),
            "A test workflow".to_string(),
            "@prefix yawl: <http://example.org/yawl#> .".to_string(),
        );
        let hash1 = spec.hash();
        let hash2 = spec.hash();
        assert_eq!(hash1, hash2, "Hash should be deterministic");
    }

    #[test]
    fn test_workflow_signer_creation() {
        let signer = WorkflowSigner::new().expect("Failed to create signer");
        assert!(!signer.public_key_id().is_empty());
    }

    #[test]
    fn test_sign_and_verify_specification() {
        let signer = WorkflowSigner::new().expect("Failed to create signer");
        let spec = YAWLSpecification::new(
            "spec_001".to_string(),
            "Test Workflow".to_string(),
            "1.0.0".to_string(),
            "A test workflow".to_string(),
            "@prefix yawl: <http://example.org/yawl#> .".to_string(),
        );

        let signed = signer.sign_specification(&spec)
            .expect("Failed to sign specification");

        signer.verify_specification(&signed)
            .expect("Verification failed");
    }

    #[test]
    fn test_delegation_certificate() {
        let issuer = WorkflowSigner::new().expect("Failed to create issuer");
        let delegate = WorkflowSigner::new().expect("Failed to create delegate");

        let cert = issuer.create_delegation_cert(
            delegate.public_key(),
            "Delegate User".to_string(),
            vec!["sign_workflows".to_string()],
            365,
        ).expect("Failed to create certificate");

        issuer.verify_delegation_cert(&cert)
            .expect("Certificate verification failed");
        assert!(cert.is_valid());
    }

    #[test]
    fn test_expired_certificate() {
        let issuer = WorkflowSigner::new().expect("Failed to create issuer");
        let delegate = WorkflowSigner::new().expect("Failed to create delegate");

        let mut cert = issuer.create_delegation_cert(
            delegate.public_key(),
            "Delegate User".to_string(),
            vec!["sign_workflows".to_string()],
            -1, // Already expired
        ).expect("Failed to create certificate");

        // Force expiration
        cert.valid_until = Utc::now() - chrono::Duration::days(1);

        assert!(!cert.is_valid());
    }
}
