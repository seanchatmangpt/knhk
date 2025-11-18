//! Comprehensive Test Suite for Quantum-Safe Cryptography
//!
//! This test suite validates NIST PQC compliance, performance requirements,
//! and integration with the KNHK platform.
//!
//! # DOCTRINE ALIGNMENT
//! - Validation: Weaver schema + NIST test vectors + performance benchmarks
//! - Q Invariant: Signature verification <10ms (Covenant 2)

use knhk_quantum::*;
use std::time::Instant;

// ============================================================================
// KEM (Key Encapsulation Mechanism) Tests
// ============================================================================

#[test]
fn test_kem_keypair_generation() {
    let kem = KyberKEM::new();
    let (pk, sk) = kem.keygen().expect("KEM keygen failed");

    // Kyber768 sizes
    assert_eq!(pk.len(), 1184, "Public key size mismatch");
    assert_eq!(sk.len(), 2400, "Secret key size mismatch");
}

#[test]
fn test_kem_encapsulation_decapsulation() {
    let kem = KyberKEM::new();
    let (pk, sk) = kem.keygen().expect("KEM keygen failed");

    let (ss_encap, ct) = kem.encapsulate(&pk).expect("Encapsulation failed");
    assert_eq!(ss_encap.len(), 32, "Shared secret size should be 32 bytes");
    assert_eq!(ct.len(), 1088, "Ciphertext size mismatch");

    let ss_decap = kem.decapsulate(&sk, &ct).expect("Decapsulation failed");
    assert_eq!(ss_encap, ss_decap, "Shared secrets don't match");
}

#[test]
fn test_kem_multiple_encapsulations() {
    let kem = KyberKEM::new();
    let (pk, sk) = kem.keygen().expect("KEM keygen failed");

    // Multiple encapsulations should produce different ciphertexts
    let (ss1, ct1) = kem.encapsulate(&pk).expect("Encapsulation 1 failed");
    let (ss2, ct2) = kem.encapsulate(&pk).expect("Encapsulation 2 failed");

    assert_ne!(ct1, ct2, "Ciphertexts should be different");
    assert_ne!(ss1, ss2, "Shared secrets should be different");

    // Both should decapsulate correctly
    let ss1_decap = kem.decapsulate(&sk, &ct1).expect("Decapsulation 1 failed");
    let ss2_decap = kem.decapsulate(&sk, &ct2).expect("Decapsulation 2 failed");

    assert_eq!(ss1, ss1_decap);
    assert_eq!(ss2, ss2_decap);
}

// ============================================================================
// Digital Signature Tests
// ============================================================================

#[test]
fn test_signature_keypair_generation() {
    let sig = DilithiumSig::new();
    let (pk, sk) = sig.keygen().expect("Signature keygen failed");

    // Dilithium3 sizes
    assert_eq!(pk.len(), 1472, "Public key size mismatch");
    assert_eq!(sk.len(), 4000, "Secret key size mismatch");
}

#[test]
fn test_signature_sign_verify() {
    let sig = DilithiumSig::new();
    let (pk, sk) = sig.keygen().expect("Signature keygen failed");

    let message = b"Test message for quantum-safe signature";
    let signature = sig.sign(&sk, message).expect("Signing failed");

    assert_eq!(signature.len(), 2701, "Signature size mismatch");

    let valid = sig.verify(&pk, message, &signature).expect("Verification failed");
    assert!(valid, "Signature should be valid");
}

#[test]
fn test_signature_wrong_message() {
    let sig = DilithiumSig::new();
    let (pk, sk) = sig.keygen().expect("Signature keygen failed");

    let message = b"Original message";
    let signature = sig.sign(&sk, message).expect("Signing failed");

    let wrong_message = b"Tampered message";
    let valid = sig.verify(&pk, wrong_message, &signature).expect("Verification failed");
    assert!(!valid, "Tampered message should not verify");
}

#[test]
fn test_signature_wrong_key() {
    let sig = DilithiumSig::new();
    let (pk1, sk1) = sig.keygen().expect("Keygen 1 failed");
    let (pk2, _sk2) = sig.keygen().expect("Keygen 2 failed");

    let message = b"Test message";
    let signature = sig.sign(&sk1, message).expect("Signing failed");

    let valid = sig.verify(&pk2, message, &signature).expect("Verification failed");
    assert!(!valid, "Signature should not verify with different key");
}

// ============================================================================
// Audit Trail Tests
// ============================================================================

#[test]
fn test_audit_trail_creation() {
    let trail = QuantumSafeAuditTrail::new().expect("Failed to create audit trail");
    assert!(trail.receipts().is_empty());
    assert!(!trail.public_key_id().is_empty());
}

#[test]
fn test_audit_trail_record_receipt() {
    let mut trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");

    let receipt = Receipt::new(
        "receipt_001".to_string(),
        "workflow_001".to_string(),
        "execute_task".to_string(),
        b"Task execution data".to_vec(),
    );

    let signed = trail.record_receipt(receipt).expect("Failed to record receipt");
    assert_eq!(signed.algorithm, "Dilithium3");
    assert!(signed.verify(trail.public_key()).expect("Verification failed"));
}

#[test]
fn test_audit_trail_integrity() {
    let mut trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");

    // Record 10 receipts
    for i in 0..10 {
        let receipt = Receipt::new(
            format!("receipt_{:03}", i),
            "workflow_001".to_string(),
            "execute_task".to_string(),
            format!("Task {} data", i).into_bytes(),
        );
        trail.record_receipt(receipt).expect("Failed to record receipt");
    }

    assert_eq!(trail.receipts().len(), 10);
    trail.verify_trail_integrity().expect("Integrity check failed");
}

#[test]
fn test_audit_trail_merkle_proof() {
    let mut trail = QuantumSafeAuditTrail::new().expect("Failed to create trail");

    // Record 8 receipts (power of 2 for clean Merkle tree)
    for i in 0..8 {
        let receipt = Receipt::new(
            format!("receipt_{:03}", i),
            "workflow_001".to_string(),
            "execute_task".to_string(),
            format!("Task {} data", i).into_bytes(),
        );
        trail.record_receipt(receipt).expect("Failed to record receipt");
    }

    // Export Merkle proof for receipt_005
    let proof = trail.export_merkle_proof("receipt_005")
        .expect("Failed to export proof");
    assert!(proof.verify(), "Merkle proof verification failed");
}

// ============================================================================
// Workflow Signing Tests
// ============================================================================

#[test]
fn test_workflow_signer_creation() {
    let signer = WorkflowSigner::new().expect("Failed to create signer");
    assert!(!signer.public_key_id().is_empty());
}

#[test]
fn test_workflow_specification_signing() {
    let signer = WorkflowSigner::new().expect("Failed to create signer");

    let spec = YAWLSpecification::new(
        "spec_001".to_string(),
        "Revenue Operations Workflow".to_string(),
        "1.0.0".to_string(),
        "Complete RevOps workflow with MAPE-K loops".to_string(),
        r#"
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

yawl:spec_001 a yawl:Specification ;
    yawl:name "Revenue Operations Workflow" ;
    yawl:version "1.0.0" .
        "#.to_string(),
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
        "Delegate Workflow Engineer".to_string(),
        vec!["sign_workflows".to_string(), "create_tasks".to_string()],
        365, // 1 year validity
    ).expect("Failed to create delegation certificate");

    issuer.verify_delegation_cert(&cert)
        .expect("Certificate verification failed");
    assert!(cert.is_valid());
}

#[test]
fn test_multi_party_signing() {
    // Scenario: Issuer delegates authority to two engineers
    let issuer = WorkflowSigner::new().expect("Failed to create issuer");
    let engineer1 = WorkflowSigner::new().expect("Failed to create engineer1");
    let engineer2 = WorkflowSigner::new().expect("Failed to create engineer2");

    // Issue delegation certificates
    let cert1 = issuer.create_delegation_cert(
        engineer1.public_key(),
        "Engineer 1".to_string(),
        vec!["sign_workflows".to_string()],
        365,
    ).expect("Failed to create cert1");

    let cert2 = issuer.create_delegation_cert(
        engineer2.public_key(),
        "Engineer 2".to_string(),
        vec!["sign_workflows".to_string()],
        365,
    ).expect("Failed to create cert2");

    // Both certificates should be valid
    issuer.verify_delegation_cert(&cert1).expect("Cert1 verification failed");
    issuer.verify_delegation_cert(&cert2).expect("Cert2 verification failed");
}

// ============================================================================
// Certificate Authority Tests
// ============================================================================

#[test]
fn test_ca_creation() {
    let ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");
    assert_eq!(ca.root_certificate().subject, "KNHK Root CA");
}

#[test]
fn test_ca_issue_certificate() {
    let mut ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");

    let sig = DilithiumSig::new();
    let (subject_pk, _subject_sk) = sig.keygen().expect("keygen failed");

    let csr = CertificateSigningRequest::new(
        "workflow-engine-01.knhk.io".to_string(),
        subject_pk,
        vec!["digital_signature".to_string(), "workflow_execution".to_string()],
    );

    let cert = ca.issue_certificate(&csr, 1)
        .expect("Failed to issue certificate");

    assert_eq!(cert.subject, "workflow-engine-01.knhk.io");
    assert_eq!(cert.issuer, "KNHK Root CA");
    assert!(cert.is_time_valid());
}

#[test]
fn test_ca_certificate_chain_verification() {
    let mut ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");

    let sig = DilithiumSig::new();
    let (subject_pk, _subject_sk) = sig.keygen().expect("keygen failed");

    let csr = CertificateSigningRequest::new(
        "workflow-engine-01.knhk.io".to_string(),
        subject_pk,
        vec!["digital_signature".to_string()],
    );

    let cert = ca.issue_certificate(&csr, 1)
        .expect("Failed to issue certificate");

    ca.verify_certificate_chain(&cert)
        .expect("Certificate chain verification failed");
}

#[test]
fn test_ca_certificate_revocation() {
    let mut ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");

    let sig = DilithiumSig::new();
    let (subject_pk, _subject_sk) = sig.keygen().expect("keygen failed");

    let csr = CertificateSigningRequest::new(
        "compromised-engine.knhk.io".to_string(),
        subject_pk,
        vec!["digital_signature".to_string()],
    );

    let cert = ca.issue_certificate(&csr, 1)
        .expect("Failed to issue certificate");

    // Certificate should verify before revocation
    ca.verify_certificate_chain(&cert).expect("Initial verification failed");

    // Revoke certificate
    ca.revoke_certificate(&cert.id, "Key compromise".to_string())
        .expect("Revocation failed");

    // Certificate should not verify after revocation
    assert!(ca.verify_certificate_chain(&cert).is_err());
    assert!(ca.is_revoked(cert.serial_number));
}

#[test]
fn test_ca_crl_tracking() {
    let mut ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");

    let sig = DilithiumSig::new();

    // Issue and revoke 3 certificates
    for i in 0..3 {
        let (subject_pk, _subject_sk) = sig.keygen().expect("keygen failed");
        let csr = CertificateSigningRequest::new(
            format!("engine-{}.knhk.io", i),
            subject_pk,
            vec!["digital_signature".to_string()],
        );
        let cert = ca.issue_certificate(&csr, 1).expect("Failed to issue");
        ca.revoke_certificate(&cert.id, format!("Reason {}", i))
            .expect("Failed to revoke");
    }

    assert_eq!(ca.revocation_list().len(), 3);
}

// ============================================================================
// Performance Tests (Q Invariant: <10ms signature verification)
// ============================================================================

#[test]
fn test_signature_performance() {
    let sig = DilithiumSig::new();
    let (pk, sk) = sig.keygen().expect("keygen failed");
    let message = b"Performance test message";

    // Measure signing performance
    let start = Instant::now();
    let signature = sig.sign(&sk, message).expect("signing failed");
    let sign_duration = start.elapsed();

    // Measure verification performance
    let start = Instant::now();
    let valid = sig.verify(&pk, message, &signature).expect("verification failed");
    let verify_duration = start.elapsed();

    assert!(valid);
    println!("Signing time: {:?}", sign_duration);
    println!("Verification time: {:?}", verify_duration);

    // Q Invariant: Verification must be <10ms
    assert!(
        verify_duration.as_millis() < 10,
        "Signature verification took {}ms, exceeds 10ms requirement",
        verify_duration.as_millis()
    );
}

#[test]
fn test_batch_signature_verification() {
    let sig = DilithiumSig::new();
    let (pk, sk) = sig.keygen().expect("keygen failed");

    // Sign 100 messages
    let mut signatures = Vec::new();
    let mut messages = Vec::new();

    for i in 0..100 {
        let msg = format!("Message {}", i);
        messages.push(msg.clone());
        let signature = sig.sign(&sk, msg.as_bytes()).expect("signing failed");
        signatures.push(signature);
    }

    // Verify all 100 signatures
    let start = Instant::now();
    for (msg, signature) in messages.iter().zip(signatures.iter()) {
        let valid = sig.verify(&pk, msg.as_bytes(), signature).expect("verification failed");
        assert!(valid);
    }
    let total_duration = start.elapsed();

    println!("Total verification time for 100 signatures: {:?}", total_duration);
    println!("Average per signature: {:?}", total_duration / 100);

    // Average should be well under 10ms
    assert!(
        (total_duration / 100).as_millis() < 10,
        "Average verification time exceeds 10ms"
    );
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_end_to_end_workflow_signing_and_audit() {
    // Scenario: Sign a workflow spec and audit the signing event

    // 1. Create CA and issue certificate
    let mut ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");

    let signer = WorkflowSigner::new().expect("Failed to create signer");

    let csr = CertificateSigningRequest::new(
        "workflow-signer-01.knhk.io".to_string(),
        signer.public_key().to_vec(),
        vec!["digital_signature".to_string(), "workflow_signing".to_string()],
    );

    let cert = ca.issue_certificate(&csr, 1)
        .expect("Failed to issue certificate");

    ca.verify_certificate_chain(&cert).expect("Cert verification failed");

    // 2. Sign workflow specification
    let spec = YAWLSpecification::new(
        "spec_revops_001".to_string(),
        "Revenue Operations Workflow".to_string(),
        "1.0.0".to_string(),
        "Complete RevOps workflow".to_string(),
        "@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .".to_string(),
    );

    let signed_spec = signer.sign_specification(&spec)
        .expect("Failed to sign specification");

    signer.verify_specification(&signed_spec).expect("Spec verification failed");

    // 3. Audit the signing event
    let mut audit_trail = QuantumSafeAuditTrail::new()
        .expect("Failed to create audit trail");

    let receipt = Receipt::new(
        "receipt_workflow_signed_001".to_string(),
        "spec_revops_001".to_string(),
        "workflow_signed".to_string(),
        signed_spec.signature.clone(),
    );

    let signed_receipt = audit_trail.record_receipt(receipt)
        .expect("Failed to record receipt");

    assert!(signed_receipt.verify(audit_trail.public_key()).expect("Receipt verification failed"));

    // 4. Verify audit trail integrity
    audit_trail.verify_trail_integrity().expect("Audit trail integrity check failed");
}

#[test]
fn test_large_scale_certificate_issuance() {
    let mut ca = QuantumSafeCA::new("KNHK Root CA".to_string(), 10)
        .expect("Failed to create CA");

    let sig = DilithiumSig::new();

    // Issue 50 certificates
    for i in 0..50 {
        let (subject_pk, _subject_sk) = sig.keygen().expect("keygen failed");
        let csr = CertificateSigningRequest::new(
            format!("workflow-engine-{:02}.knhk.io", i),
            subject_pk,
            vec!["digital_signature".to_string()],
        );
        let cert = ca.issue_certificate(&csr, 1).expect("Failed to issue");
        ca.verify_certificate_chain(&cert).expect("Verification failed");
    }

    assert_eq!(ca.issued_certificates().len(), 50);
}
