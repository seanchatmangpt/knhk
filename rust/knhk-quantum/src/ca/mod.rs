//! Quantum-Safe Certificate Authority Module
//!
//! Provides a certificate authority implementation using ML-DSA (Dilithium)
//! post-quantum signatures for issuing, revoking, and verifying certificates.
//!
//! # DOCTRINE ALIGNMENT
//! - Principle: Q (Hard Invariants) - Trust chain is quantum-safe
//! - Covenant: 2 (Invariants Are Law) - All certificates must be verifiable
//! - Validation: Signature verification + CRL checking

use serde::{Deserialize, Serialize};
use thiserror::Error;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use crate::sig::{QuantumSig, DilithiumSig, SigError};

/// Certificate authority errors
#[derive(Error, Debug)]
pub enum CAError {
    #[error("Signature error: {0}")]
    SignatureError(#[from] SigError),
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
    #[error("Certificate revoked: {0}")]
    CertificateRevoked(String),
    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),
    #[error("Invalid CSR: {0}")]
    InvalidCSR(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CAError>;

/// Certificate Signing Request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CertificateSigningRequest {
    pub subject_name: String,
    pub subject_public_key: Vec<u8>,
    pub usage: Vec<String>, // e.g., ["digital_signature", "key_encipherment"]
    pub metadata: HashMap<String, String>,
    pub requested_at: DateTime<Utc>,
}

impl CertificateSigningRequest {
    /// Create a new CSR
    pub fn new(
        subject_name: String,
        subject_public_key: Vec<u8>,
        usage: Vec<String>,
    ) -> Self {
        Self {
            subject_name,
            subject_public_key,
            usage,
            metadata: HashMap::new(),
            requested_at: Utc::now(),
        }
    }

    /// Compute hash for validation
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.subject_name);
        hasher.update(&self.subject_public_key);
        for u in &self.usage {
            hasher.update(u);
        }
        hasher.finalize().to_vec()
    }
}

/// X.509-style certificate (quantum-safe)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub serial_number: u64,
    pub issuer: String,
    pub subject: String,
    pub subject_public_key: Vec<u8>,
    pub usage: Vec<String>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub signature: Vec<u8>,
    pub algorithm: String, // "Dilithium3"
    pub issued_at: DateTime<Utc>,
}

impl Certificate {
    /// Compute hash of certificate (without signature)
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
        hasher.update(self.serial_number.to_le_bytes());
        hasher.update(&self.issuer);
        hasher.update(&self.subject);
        hasher.update(&self.subject_public_key);
        for u in &self.usage {
            hasher.update(u);
        }
        hasher.update(self.valid_from.to_rfc3339().as_bytes());
        hasher.update(self.valid_until.to_rfc3339().as_bytes());
        hasher.finalize().to_vec()
    }

    /// Verify certificate signature
    pub fn verify(&self, issuer_public_key: &[u8]) -> Result<bool> {
        let sig_engine = DilithiumSig::new();
        let cert_hash = self.hash();
        Ok(sig_engine.verify(issuer_public_key, &cert_hash, &self.signature)?)
    }

    /// Check if certificate is currently valid (time-wise)
    pub fn is_time_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.valid_from && now <= self.valid_until
    }
}

/// Certificate Revocation List entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CRLEntry {
    pub certificate_id: String,
    pub serial_number: u64,
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
}

/// Quantum-Safe Certificate Authority
pub struct QuantumSafeCA {
    root_private_key: Vec<u8>,
    root_public_key: Vec<u8>,
    root_certificate: Certificate,
    issued_certificates: HashMap<String, Certificate>,
    revocation_list: Vec<CRLEntry>,
    revoked_serials: HashSet<u64>,
    next_serial: u64,
    sig_engine: DilithiumSig,
}

impl QuantumSafeCA {
    /// Create a new CA with self-signed root certificate
    pub fn new(ca_name: String, validity_years: i64) -> Result<Self> {
        let sig_engine = DilithiumSig::new();
        let (root_public_key, root_private_key) = sig_engine.keygen()?;

        let now = Utc::now();
        let valid_until = now + chrono::Duration::days(validity_years * 365);

        let mut root_cert = Certificate {
            id: format!("root_{}", hex::encode(&root_public_key[..8])),
            serial_number: 1,
            issuer: ca_name.clone(),
            subject: ca_name,
            subject_public_key: root_public_key.clone(),
            usage: vec!["certificate_signing".to_string()],
            valid_from: now,
            valid_until,
            signature: Vec::new(),
            algorithm: "Dilithium3".to_string(),
            issued_at: now,
        };

        // Self-sign root certificate
        let root_hash = root_cert.hash();
        root_cert.signature = sig_engine.sign(&root_private_key, &root_hash)?;

        Ok(Self {
            root_private_key,
            root_public_key,
            root_certificate: root_cert,
            issued_certificates: HashMap::new(),
            revocation_list: Vec::new(),
            revoked_serials: HashSet::new(),
            next_serial: 2,
            sig_engine,
        })
    }

    /// Issue a certificate from a CSR
    pub fn issue_certificate(
        &mut self,
        csr: &CertificateSigningRequest,
        validity_years: i64,
    ) -> Result<Certificate> {
        let now = Utc::now();
        let valid_until = now + chrono::Duration::days(validity_years * 365);
        let serial = self.next_serial;
        self.next_serial += 1;

        let mut cert = Certificate {
            id: format!("cert_{:016x}", serial),
            serial_number: serial,
            issuer: self.root_certificate.subject.clone(),
            subject: csr.subject_name.clone(),
            subject_public_key: csr.subject_public_key.clone(),
            usage: csr.usage.clone(),
            valid_from: now,
            valid_until,
            signature: Vec::new(),
            algorithm: "Dilithium3".to_string(),
            issued_at: now,
        };

        // Sign certificate with root key
        let cert_hash = cert.hash();
        cert.signature = self.sig_engine.sign(&self.root_private_key, &cert_hash)?;

        self.issued_certificates.insert(cert.id.clone(), cert.clone());
        Ok(cert)
    }

    /// Revoke a certificate
    pub fn revoke_certificate(&mut self, cert_id: &str, reason: String) -> Result<()> {
        let cert = self.issued_certificates
            .get(cert_id)
            .ok_or_else(|| CAError::CertificateNotFound(cert_id.to_string()))?;

        let entry = CRLEntry {
            certificate_id: cert_id.to_string(),
            serial_number: cert.serial_number,
            revoked_at: Utc::now(),
            reason,
        };

        self.revocation_list.push(entry);
        self.revoked_serials.insert(cert.serial_number);
        Ok(())
    }

    /// Check if a certificate is revoked
    pub fn is_revoked(&self, serial_number: u64) -> bool {
        self.revoked_serials.contains(&serial_number)
    }

    /// Verify a certificate chain to the root
    pub fn verify_certificate_chain(&self, cert: &Certificate) -> Result<()> {
        // Check if revoked
        if self.is_revoked(cert.serial_number) {
            return Err(CAError::CertificateRevoked(cert.id.clone()));
        }

        // Verify signature with root key
        if !cert.verify(&self.root_public_key)? {
            return Err(CAError::InvalidCertificate(
                "Signature verification failed".to_string()
            ));
        }

        // Check time validity
        if !cert.is_time_valid() {
            return Err(CAError::InvalidCertificate(
                "Certificate has expired".to_string()
            ));
        }

        Ok(())
    }

    /// Get the root certificate
    pub fn root_certificate(&self) -> &Certificate {
        &self.root_certificate
    }

    /// Get all issued certificates
    pub fn issued_certificates(&self) -> &HashMap<String, Certificate> {
        &self.issued_certificates
    }

    /// Get the certificate revocation list
    pub fn revocation_list(&self) -> &[CRLEntry] {
        &self.revocation_list
    }

    /// Get root public key
    pub fn root_public_key(&self) -> &[u8] {
        &self.root_public_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_creation() {
        let ca = QuantumSafeCA::new("Test CA".to_string(), 10)
            .expect("Failed to create CA");
        assert_eq!(ca.root_certificate().subject, "Test CA");
        assert!(ca.issued_certificates().is_empty());
    }

    #[test]
    fn test_root_certificate_self_signed() {
        let ca = QuantumSafeCA::new("Test CA".to_string(), 10)
            .expect("Failed to create CA");
        let root = ca.root_certificate();
        assert!(root.verify(ca.root_public_key()).expect("Verification failed"));
    }

    #[test]
    fn test_issue_certificate() {
        let mut ca = QuantumSafeCA::new("Test CA".to_string(), 10)
            .expect("Failed to create CA");

        let sig_engine = DilithiumSig::new();
        let (subject_pk, _subject_sk) = sig_engine.keygen().expect("keygen failed");

        let csr = CertificateSigningRequest::new(
            "Test Subject".to_string(),
            subject_pk,
            vec!["digital_signature".to_string()],
        );

        let cert = ca.issue_certificate(&csr, 1)
            .expect("Failed to issue certificate");

        assert_eq!(cert.subject, "Test Subject");
        assert_eq!(cert.issuer, "Test CA");
    }

    #[test]
    fn test_verify_certificate_chain() {
        let mut ca = QuantumSafeCA::new("Test CA".to_string(), 10)
            .expect("Failed to create CA");

        let sig_engine = DilithiumSig::new();
        let (subject_pk, _subject_sk) = sig_engine.keygen().expect("keygen failed");

        let csr = CertificateSigningRequest::new(
            "Test Subject".to_string(),
            subject_pk,
            vec!["digital_signature".to_string()],
        );

        let cert = ca.issue_certificate(&csr, 1)
            .expect("Failed to issue certificate");

        ca.verify_certificate_chain(&cert)
            .expect("Certificate chain verification failed");
    }

    #[test]
    fn test_revoke_certificate() {
        let mut ca = QuantumSafeCA::new("Test CA".to_string(), 10)
            .expect("Failed to create CA");

        let sig_engine = DilithiumSig::new();
        let (subject_pk, _subject_sk) = sig_engine.keygen().expect("keygen failed");

        let csr = CertificateSigningRequest::new(
            "Test Subject".to_string(),
            subject_pk,
            vec!["digital_signature".to_string()],
        );

        let cert = ca.issue_certificate(&csr, 1)
            .expect("Failed to issue certificate");

        ca.revoke_certificate(&cert.id, "Test revocation".to_string())
            .expect("Revocation failed");

        assert!(ca.is_revoked(cert.serial_number));
        assert!(ca.verify_certificate_chain(&cert).is_err());
    }

    #[test]
    fn test_crl_tracking() {
        let mut ca = QuantumSafeCA::new("Test CA".to_string(), 10)
            .expect("Failed to create CA");

        let sig_engine = DilithiumSig::new();
        let (subject_pk, _subject_sk) = sig_engine.keygen().expect("keygen failed");

        let csr = CertificateSigningRequest::new(
            "Test Subject".to_string(),
            subject_pk,
            vec!["digital_signature".to_string()],
        );

        let cert = ca.issue_certificate(&csr, 1)
            .expect("Failed to issue certificate");

        ca.revoke_certificate(&cert.id, "Compromised".to_string())
            .expect("Revocation failed");

        assert_eq!(ca.revocation_list().len(), 1);
        assert_eq!(ca.revocation_list()[0].reason, "Compromised");
    }
}
