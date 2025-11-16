//! Comprehensive BFT consensus tests

use knhk_workflow_engine::consensus::*;
use knhk_workflow_engine::consensus::bft::*;

#[tokio::test]
async fn test_bft_cluster_creation() {
    let crypto = CryptoProvider::new();
    let cluster = BftCluster::new_hotstuff(
        NodeId::new(1),
        vec![],
        crypto,
    ).unwrap();

    let metrics = cluster.metrics().await;
    assert_eq!(metrics.proposals_submitted, 0);
}

#[tokio::test]
async fn test_bft_cluster_pbft() {
    let crypto = CryptoProvider::new();
    let cluster = BftCluster::new_pbft(
        NodeId::new(1),
        vec![],
        crypto,
    ).unwrap();

    let metrics = cluster.metrics().await;
    assert_eq!(metrics.byzantine_failures, 0);
}

#[test]
fn test_crypto_provider() {
    let crypto = CryptoProvider::new();
    let message = b"Test message for BFT";

    let signature = crypto.sign(message);

    // Signature should have 64 bytes
    assert_eq!(signature.bytes.len(), 64);
}

#[test]
fn test_keypair_generation() {
    let keypair1 = KeyPair::generate();
    let keypair2 = KeyPair::generate();

    // Different keypairs should have different public keys
    let pk1 = keypair1.public_key_bytes();
    let pk2 = keypair2.public_key_bytes();

    assert_ne!(pk1, pk2);
}

#[test]
fn test_signature_verification() {
    let keypair = KeyPair::generate();
    let message = b"Byzantine consensus test";

    let signature = keypair.sign(message);
    let public_key = keypair.public_key();

    // Signature should verify with correct public key
    assert!(signature.verify(message, &public_key));

    // Signature should fail with different message
    let wrong_message = b"Wrong message";
    assert!(!signature.verify(wrong_message, &public_key));
}

#[test]
fn test_signature_verifier() {
    let mut verifier = SignatureVerifier::new();

    let kp1 = KeyPair::generate();
    let kp2 = KeyPair::generate();

    verifier.add_public_key(NodeId::new(1), kp1.public_key());
    verifier.add_public_key(NodeId::new(2), kp2.public_key());

    let message = b"BFT message";
    let sig1 = kp1.sign(message);
    let sig2 = kp2.sign(message);

    // Verify signatures from nodes
    assert!(verifier.verify(NodeId::new(1), message, &sig1));
    assert!(verifier.verify(NodeId::new(2), message, &sig2));

    // Wrong node ID should fail
    assert!(!verifier.verify(NodeId::new(2), message, &sig1));

    // Unknown node should fail
    assert!(!verifier.verify(NodeId::new(3), message, &sig1));
}

#[test]
fn test_quorum_verification() {
    let crypto = CryptoProvider::new();

    let kp2 = KeyPair::generate();
    let kp3 = KeyPair::generate();
    let kp4 = KeyPair::generate();

    // Create a new crypto provider with peer keys
    let mut crypto2 = CryptoProvider::new();
    crypto2.add_peer_key(NodeId::new(2), kp2.public_key());
    crypto2.add_peer_key(NodeId::new(3), kp3.public_key());
    crypto2.add_peer_key(NodeId::new(4), kp4.public_key());

    let message = b"Quorum test message";

    let mut signatures = std::collections::HashMap::new();
    signatures.insert(NodeId::new(2), kp2.sign(message));
    signatures.insert(NodeId::new(3), kp3.sign(message));
    signatures.insert(NodeId::new(4), kp4.sign(message));

    // Quorum of 3 should succeed
    assert!(crypto2.verify_quorum(message, &signatures, 3));

    // Quorum of 2 should succeed
    assert!(crypto2.verify_quorum(message, &signatures, 2));

    // Quorum of 4 should fail (only have 3 signatures)
    assert!(!crypto2.verify_quorum(message, &signatures, 4));
}
