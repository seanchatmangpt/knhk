//! Descriptor Signer
//!
//! Cryptographic signing and verification of compiled descriptors.
//! Uses Ed25519 for signatures.

use crate::compiler::linker::LinkedDescriptor;
use crate::error::{WorkflowError, WorkflowResult};
use ring::{rand, signature};
use std::fs;
use std::path::Path;
use tracing::{debug, info, instrument};

/// Descriptor signer
pub struct DescriptorSigner {
    enabled: bool,
    key_pair: Option<signature::Ed25519KeyPair>,
}

impl DescriptorSigner {
    /// Create new signer
    pub fn new(enabled: bool) -> Self {
        let key_pair = if enabled {
            Some(Self::generate_key_pair())
        } else {
            None
        };

        Self { enabled, key_pair }
    }

    /// Generate new key pair
    fn generate_key_pair() -> signature::Ed25519KeyPair {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Failed to generate key pair");

        signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .expect("Failed to parse key pair")
    }

    /// Load key pair from file
    pub fn load_key_pair<P: AsRef<Path>>(path: P) -> WorkflowResult<signature::Ed25519KeyPair> {
        let pkcs8_bytes = fs::read(path)
            .map_err(|e| WorkflowError::Io(format!("Failed to read key file: {}", e)))?;

        signature::Ed25519KeyPair::from_pkcs8(&pkcs8_bytes)
            .map_err(|e| WorkflowError::Crypto(format!("Invalid key format: {:?}", e)))
    }

    /// Save key pair to file
    pub fn save_key_pair<P: AsRef<Path>>(
        key_pair: &signature::Ed25519KeyPair,
        path: P,
    ) -> WorkflowResult<()> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| WorkflowError::Crypto(format!("Failed to encode key: {:?}", e)))?;

        fs::write(path, pkcs8_bytes.as_ref())
            .map_err(|e| WorkflowError::Io(format!("Failed to save key: {}", e)))?;

        Ok(())
    }

    /// Sign descriptor
    #[instrument(skip(self, descriptor))]
    pub async fn sign(&self, descriptor: &LinkedDescriptor) -> WorkflowResult<Vec<u8>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let key_pair = self.key_pair.as_ref()
            .ok_or_else(|| WorkflowError::Crypto("No key pair loaded".to_string()))?;

        info!("Signing descriptor");

        // Serialize descriptor for signing
        let message = self.serialize_for_signing(descriptor)?;

        // Sign the message
        let signature = key_pair.sign(&message);

        debug!("Generated signature: {} bytes", signature.as_ref().len());

        Ok(signature.as_ref().to_vec())
    }

    /// Verify signature
    pub fn verify(&self, descriptor: &[u8], signature: &[u8]) -> WorkflowResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let key_pair = self.key_pair.as_ref()
            .ok_or_else(|| WorkflowError::Crypto("No key pair loaded".to_string()))?;

        let public_key = key_pair.public_key();
        let peer_public_key = signature::UnparsedPublicKey::new(
            &signature::ED25519,
            public_key.as_ref(),
        );

        peer_public_key.verify(descriptor, signature)
            .map_err(|e| WorkflowError::Crypto(format!("Signature verification failed: {:?}", e)))?;

        info!("Signature verified successfully");
        Ok(())
    }

    /// Serialize descriptor for signing
    fn serialize_for_signing(&self, descriptor: &LinkedDescriptor) -> WorkflowResult<Vec<u8>> {
        let mut message = Vec::new();

        // Include all critical fields
        message.extend_from_slice(&(descriptor.pattern_count as u32).to_le_bytes());
        message.extend_from_slice(&descriptor.code_segment.bytecode);
        message.extend_from_slice(&descriptor.data_segment.data);
        message.extend_from_slice(&descriptor.data_segment.constants);
        message.extend_from_slice(&descriptor.data_segment.strings);
        message.extend_from_slice(&descriptor.metadata.timestamp.to_le_bytes());
        message.extend_from_slice(&descriptor.metadata.checksum.to_le_bytes());

        Ok(message)
    }
}

impl Default for DescriptorSigner {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::linker::{
        LinkedDescriptor, CodeSegment, DataSegment, LinkedSymbolTable, LinkMetadata,
    };
    use std::collections::HashMap;

    fn create_test_descriptor() -> LinkedDescriptor {
        LinkedDescriptor {
            pattern_count: 1,
            code_segment: CodeSegment {
                bytecode: vec![0x01, 0x02, 0x03],
                size: 3,
                alignment: 64,
            },
            data_segment: DataSegment {
                data: vec![0x04, 0x05],
                constants: vec![0x06],
                strings: vec![0x07],
                size: 4,
            },
            symbol_table: LinkedSymbolTable {
                symbols: Vec::new(),
                name_map: HashMap::new(),
            },
            relocations: Vec::new(),
            entry_points: HashMap::new(),
            metadata: LinkMetadata {
                timestamp: 0,
                linker_version: "1.0.0".to_string(),
                total_size: 7,
                checksum: 0,
            },
        }
    }

    #[tokio::test]
    async fn test_signer_creation() {
        let signer = DescriptorSigner::new(true);
        assert!(signer.enabled);
        assert!(signer.key_pair.is_some());
    }

    #[tokio::test]
    async fn test_signing() {
        let signer = DescriptorSigner::new(true);
        let descriptor = create_test_descriptor();

        let signature = signer.sign(&descriptor).await.unwrap();
        assert_eq!(signature.len(), 64); // Ed25519 signature size
    }

    #[tokio::test]
    async fn test_verification() {
        let signer = DescriptorSigner::new(true);
        let descriptor = create_test_descriptor();

        let message = signer.serialize_for_signing(&descriptor).unwrap();
        let signature = signer.sign(&descriptor).await.unwrap();

        signer.verify(&message, &signature).unwrap();
    }

    #[tokio::test]
    async fn test_disabled_signer() {
        let signer = DescriptorSigner::new(false);
        let descriptor = create_test_descriptor();

        let signature = signer.sign(&descriptor).await.unwrap();
        assert!(signature.is_empty());
    }
}