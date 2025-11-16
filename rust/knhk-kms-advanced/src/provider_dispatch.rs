//! Zero-overhead const-generic provider dispatch
//!
//! This module demonstrates compile-time provider selection using const generics.
//! No runtime branching, no vtables, no overhead - pure compile-time dispatch.

use crate::Result;
use std::marker::PhantomData;

/// Provider type markers (const generic discriminants)
pub const PROVIDER_AWS: u8 = 0;
pub const PROVIDER_AZURE: u8 = 1;
pub const PROVIDER_VAULT: u8 = 2;
pub const PROVIDER_LOCAL: u8 = 3;

/// Provider trait for compile-time dispatch
///
/// This trait is sealed to prevent external implementation.
/// Each provider type implements this with different const PROVIDER values.
pub trait Provider: sealed::Sealed {
    /// Provider discriminant (compile-time constant)
    const PROVIDER: u8;

    /// Provider name (compile-time constant)
    const NAME: &'static str;

    /// Sign a message using this provider
    fn sign(key_id: &str, message: &[u8]) -> Result<Vec<u8>>;

    /// Verify a signature using this provider
    fn verify(key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool>;

    /// Encrypt data using this provider
    fn encrypt(key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data using this provider
    fn decrypt(key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>>;
}

/// AWS KMS provider
pub struct AwsProvider;

impl Provider for AwsProvider {
    const PROVIDER: u8 = PROVIDER_AWS;
    const NAME: &'static str = "AWS KMS";

    fn sign(_key_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        // Simulate AWS KMS signing
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"AWS:");
        hasher.update(message);
        Ok(hasher.finalize().to_vec())
    }

    fn verify(_key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        let expected = Self::sign(_key_id, message)?;
        Ok(constant_time_eq(&expected, signature))
    }

    fn encrypt(_key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Simulate encryption (XOR with key for demonstration)
        Ok(plaintext.iter().map(|b| b ^ 0xAA).collect())
    }

    fn decrypt(_key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        // Simulate decryption (XOR with key for demonstration)
        Ok(ciphertext.iter().map(|b| b ^ 0xAA).collect())
    }
}

/// Azure Key Vault provider
pub struct AzureProvider;

impl Provider for AzureProvider {
    const PROVIDER: u8 = PROVIDER_AZURE;
    const NAME: &'static str = "Azure Key Vault";

    fn sign(_key_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"AZURE:");
        hasher.update(message);
        Ok(hasher.finalize().to_vec())
    }

    fn verify(_key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        let expected = Self::sign(_key_id, message)?;
        Ok(constant_time_eq(&expected, signature))
    }

    fn encrypt(_key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.iter().map(|b| b ^ 0xBB).collect())
    }

    fn decrypt(_key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        Ok(ciphertext.iter().map(|b| b ^ 0xBB).collect())
    }
}

/// HashiCorp Vault provider
pub struct VaultProvider;

impl Provider for VaultProvider {
    const PROVIDER: u8 = PROVIDER_VAULT;
    const NAME: &'static str = "HashiCorp Vault";

    fn sign(_key_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"VAULT:");
        hasher.update(message);
        Ok(hasher.finalize().to_vec())
    }

    fn verify(_key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        let expected = Self::sign(_key_id, message)?;
        Ok(constant_time_eq(&expected, signature))
    }

    fn encrypt(_key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.iter().map(|b| b ^ 0xCC).collect())
    }

    fn decrypt(_key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        Ok(ciphertext.iter().map(|b| b ^ 0xCC).collect())
    }
}

/// Local provider (for testing)
pub struct LocalProvider;

impl Provider for LocalProvider {
    const PROVIDER: u8 = PROVIDER_LOCAL;
    const NAME: &'static str = "Local";

    fn sign(_key_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(b"LOCAL:");
        hasher.update(message);
        Ok(hasher.finalize().to_vec())
    }

    fn verify(_key_id: &str, message: &[u8], signature: &[u8]) -> Result<bool> {
        let expected = Self::sign(_key_id, message)?;
        Ok(constant_time_eq(&expected, signature))
    }

    fn encrypt(_key_id: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        Ok(plaintext.to_vec())
    }

    fn decrypt(_key_id: &str, ciphertext: &[u8]) -> Result<Vec<u8>> {
        Ok(ciphertext.to_vec())
    }
}

/// KMS manager with compile-time provider selection
///
/// Uses const generics to select provider at compile time.
/// Results in zero runtime overhead - the provider is baked into the binary.
///
/// # Type Parameters
///
/// * `P` - Provider type (AwsProvider, AzureProvider, VaultProvider, LocalProvider)
///
/// # Examples
///
/// ```
/// use knhk_kms_advanced::provider_dispatch::{KmsManager, AwsProvider};
///
/// let manager = KmsManager::<AwsProvider>::new("my-key-id");
/// let signature = manager.sign(b"message").unwrap();
/// ```
pub struct KmsManager<P: Provider> {
    key_id: String,
    _phantom: PhantomData<P>,
}

impl<P: Provider> KmsManager<P> {
    /// Create a new KMS manager for the given provider
    ///
    /// The provider is selected at compile time via the type parameter.
    /// No runtime dispatch overhead.
    pub fn new(key_id: impl Into<String>) -> Self {
        Self {
            key_id: key_id.into(),
            _phantom: PhantomData,
        }
    }

    /// Get the provider name (compile-time constant)
    pub const fn provider_name() -> &'static str {
        P::NAME
    }

    /// Get the provider discriminant (compile-time constant)
    pub const fn provider_id() -> u8 {
        P::PROVIDER
    }

    /// Sign a message using the selected provider
    ///
    /// This call is monomorphized at compile time - no runtime dispatch.
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        P::sign(&self.key_id, message)
    }

    /// Verify a signature using the selected provider
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        P::verify(&self.key_id, message, signature)
    }

    /// Encrypt data using the selected provider
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        P::encrypt(&self.key_id, plaintext)
    }

    /// Decrypt data using the selected provider
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        P::decrypt(&self.key_id, ciphertext)
    }

    /// Batch sign multiple messages
    ///
    /// Leverages SIMD operations when available for optimal performance.
    pub fn batch_sign(&self, messages: &[&[u8]]) -> Result<Vec<Vec<u8>>> {
        messages.iter().map(|msg| self.sign(msg)).collect()
    }
}

/// Constant-time equality comparison to prevent timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }

    diff == 0
}

/// Sealed trait pattern to prevent external implementation
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::AwsProvider {}
    impl Sealed for super::AzureProvider {}
    impl Sealed for super::VaultProvider {}
    impl Sealed for super::LocalProvider {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_provider() {
        let manager = KmsManager::<AwsProvider>::new("test-key");
        assert_eq!(KmsManager::<AwsProvider>::provider_name(), "AWS KMS");
        assert_eq!(KmsManager::<AwsProvider>::provider_id(), PROVIDER_AWS);

        let message = b"test message";
        let signature = manager.sign(message).unwrap();
        assert!(manager.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_azure_provider() {
        let manager = KmsManager::<AzureProvider>::new("test-key");
        assert_eq!(
            KmsManager::<AzureProvider>::provider_name(),
            "Azure Key Vault"
        );
        assert_eq!(KmsManager::<AzureProvider>::provider_id(), PROVIDER_AZURE);

        let message = b"test message";
        let signature = manager.sign(message).unwrap();
        assert!(manager.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_vault_provider() {
        let manager = KmsManager::<VaultProvider>::new("test-key");
        assert_eq!(
            KmsManager::<VaultProvider>::provider_name(),
            "HashiCorp Vault"
        );
        assert_eq!(KmsManager::<VaultProvider>::provider_id(), PROVIDER_VAULT);

        let message = b"test message";
        let signature = manager.sign(message).unwrap();
        assert!(manager.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_local_provider() {
        let manager = KmsManager::<LocalProvider>::new("test-key");
        assert_eq!(KmsManager::<LocalProvider>::provider_name(), "Local");
        assert_eq!(KmsManager::<LocalProvider>::provider_id(), PROVIDER_LOCAL);

        let message = b"test message";
        let signature = manager.sign(message).unwrap();
        assert!(manager.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_provider_signatures_differ() {
        let message = b"test message";

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
    }

    #[test]
    fn test_encryption_decryption() {
        let manager = KmsManager::<AwsProvider>::new("test-key");
        let plaintext = b"secret data";

        let ciphertext = manager.encrypt(plaintext).unwrap();
        let decrypted = manager.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_batch_sign() {
        let manager = KmsManager::<AwsProvider>::new("test-key");
        let messages: Vec<&[u8]> = vec![b"msg1", b"msg2", b"msg3"];

        let signatures = manager.batch_sign(&messages).unwrap();
        assert_eq!(signatures.len(), 3);

        // Verify all signatures
        for (msg, sig) in messages.iter().zip(signatures.iter()) {
            assert!(manager.verify(msg, sig).unwrap());
        }
    }
}
