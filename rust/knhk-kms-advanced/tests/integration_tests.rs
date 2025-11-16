use knhk_kms_advanced::{
    batch_signer::BatchSigner,
    config::KmsConfig,
    provider_dispatch::{AwsProvider, AzureProvider, KmsManager, VaultProvider},
    simd_ops::{SimdHasher, SimdSigner},
    type_safety::{TypedKey, Unsigned},
};

#[test]
fn test_end_to_end_aws_workflow() {
    // Create AWS KMS manager
    let manager = KmsManager::<AwsProvider>::new("test-key-id");

    // Sign a message
    let message = b"important message";
    let signature = manager.sign(message).unwrap();

    // Verify the signature
    assert!(manager.verify(message, &signature).unwrap());

    // Verify fails with wrong message
    assert!(!manager.verify(b"wrong message", &signature).unwrap());
}

#[test]
fn test_multi_provider_interop() {
    let message = b"cross-provider test";

    // Sign with different providers
    let aws_manager = KmsManager::<AwsProvider>::new("key");
    let azure_manager = KmsManager::<AzureProvider>::new("key");
    let vault_manager = KmsManager::<VaultProvider>::new("key");

    let aws_sig = aws_manager.sign(message).unwrap();
    let azure_sig = azure_manager.sign(message).unwrap();
    let vault_sig = vault_manager.sign(message).unwrap();

    // Each provider produces different signatures
    assert_ne!(aws_sig, azure_sig);
    assert_ne!(azure_sig, vault_sig);
    assert_ne!(aws_sig, vault_sig);

    // Each provider can verify its own signatures
    assert!(aws_manager.verify(message, &aws_sig).unwrap());
    assert!(azure_manager.verify(message, &azure_sig).unwrap());
    assert!(vault_manager.verify(message, &vault_sig).unwrap());

    // Cross-verification fails
    assert!(!aws_manager.verify(message, &azure_sig).unwrap());
    assert!(!azure_manager.verify(message, &vault_sig).unwrap());
}

#[test]
fn test_batch_signer_integration() {
    let key = [42u8; 32];
    let batch_signer = BatchSigner::new(key, 64).unwrap();

    // Generate test messages
    let messages: Vec<[u8; 32]> = (0..100)
        .map(|i| {
            let mut msg = [0u8; 32];
            msg[0] = i as u8;
            msg
        })
        .collect();

    // Batch sign
    let result = batch_signer.batch_sign(&messages).unwrap();
    assert!(result.all_succeeded());
    assert_eq!(result.total, 100);
    assert_eq!(result.signatures.len(), 100);

    // Batch verify
    let verifications = batch_signer
        .batch_verify(&messages, &result.signatures)
        .unwrap();

    assert_eq!(verifications.len(), 100);
    assert!(verifications.iter().all(|&v| v));
}

#[test]
fn test_simd_hasher_integration() {
    let hasher = SimdHasher::new();

    // Hash multiple inputs
    let inputs: Vec<[u8; 32]> = (0..128).map(|i| [i as u8; 32]).collect();
    let hashes = hasher.batch_hash(&inputs);

    assert_eq!(hashes.len(), 128);

    // Verify determinism
    let hashes2 = hasher.batch_hash(&inputs);
    assert_eq!(hashes, hashes2);

    // Different inputs produce different hashes
    assert_ne!(hashes[0], hashes[1]);
}

#[test]
fn test_simd_signer_integration() {
    let key = [99u8; 32];
    let signer = SimdSigner::<64>::new(key);

    // Sign multiple messages
    let messages: Vec<[u8; 32]> = (0..200)
        .map(|i| {
            let mut msg = [0u8; 32];
            msg[0] = (i % 256) as u8;
            msg[1] = (i / 256) as u8;
            msg
        })
        .collect();

    let signatures = signer.batch_sign(&messages).unwrap();
    assert_eq!(signatures.len(), 200);

    // Verify all signatures
    let verifications = signer.batch_verify(&messages, &signatures).unwrap();
    assert!(verifications.iter().all(|&v| v));
}

#[test]
fn test_typed_key_state_machine() {
    let key_material = [77u8; 32];

    // Create unsigned key
    let unsigned_key = TypedKey::<Unsigned>::new(key_material);

    // Sign to get signed key
    let message = b"state machine test";
    let signed_key = unsigned_key.sign(message).unwrap();

    // Verify to get verified key
    let verified_key = signed_key.verify(message).unwrap();

    // Use verified key
    let extracted = verified_key.key_material();
    assert_eq!(extracted, &key_material);
}

#[test]
fn test_typed_key_seal_unseal() {
    let key_material = [55u8; 32];
    let encryption_key = [123u8; 32];

    // Create and seal key
    let unsigned_key = TypedKey::<Unsigned>::new(key_material);
    let sealed_key = unsigned_key.seal(&encryption_key).unwrap();

    // Sealed material should be different
    assert_ne!(sealed_key.sealed_material(), &key_material);

    // Unseal and verify
    let unsealed_key = sealed_key.unseal(&encryption_key).unwrap();
    let signed_key = unsealed_key.sign(b"test").unwrap();
    let verified_key = signed_key.verify(b"test").unwrap();

    assert_eq!(verified_key.key_material(), &key_material);
}

#[test]
fn test_config_validation() {
    let mut config = KmsConfig::default();
    assert!(config.validate().is_ok());

    // Invalid batch size (not power of 2)
    config.batch_size = 63;
    assert!(config.validate().is_err());

    // Valid batch size
    config.batch_size = 128;
    assert!(config.validate().is_ok());

    // Too large
    config.batch_size = 2048;
    assert!(config.validate().is_err());
}

#[test]
fn test_large_scale_batch_processing() {
    let key = [200u8; 32];
    let batch_signer = BatchSigner::new(key, 64).unwrap();

    // Process 10,000 signatures
    let messages: Vec<[u8; 32]> = (0..10_000)
        .map(|i| {
            let mut msg = [0u8; 32];
            msg[0] = (i % 256) as u8;
            msg[1] = ((i / 256) % 256) as u8;
            msg[2] = (i / 65536) as u8;
            msg
        })
        .collect();

    let result = batch_signer.batch_sign_chunked(&messages).unwrap();
    assert_eq!(result.total, 10_000);
    assert!(result.all_succeeded());
    assert_eq!(result.success_rate(), 100.0);
}

#[test]
fn test_provider_encryption_roundtrip() {
    let manager = KmsManager::<AwsProvider>::new("encryption-key");

    let plaintext = b"super secret data that needs encryption";
    let ciphertext = manager.encrypt(plaintext).unwrap();

    // Ciphertext should be different from plaintext
    assert_ne!(ciphertext.as_slice(), plaintext);

    // Decrypt and verify
    let decrypted = manager.decrypt(&ciphertext).unwrap();
    assert_eq!(decrypted.as_slice(), plaintext);
}

#[test]
fn test_batch_operations_with_provider() {
    let manager = KmsManager::<AzureProvider>::new("batch-key");

    let messages: Vec<&[u8]> = vec![
        b"message 1",
        b"message 2",
        b"message 3",
        b"message 4",
        b"message 5",
    ];

    let signatures = manager.batch_sign(&messages).unwrap();
    assert_eq!(signatures.len(), 5);

    // Verify each signature individually
    for (msg, sig) in messages.iter().zip(signatures.iter()) {
        assert!(manager.verify(msg, sig).unwrap());
    }
}

#[test]
fn test_concurrent_safety() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(KmsManager::<AwsProvider>::new("concurrent-key"));
    let mut handles = vec![];

    // Spawn 10 threads that sign messages concurrently
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            let message = format!("thread {} message", i);
            let signature = manager_clone.sign(message.as_bytes()).unwrap();
            manager_clone
                .verify(message.as_bytes(), &signature)
                .unwrap()
        });
        handles.push(handle);
    }

    // All threads should succeed
    for handle in handles {
        assert!(handle.join().unwrap());
    }
}
