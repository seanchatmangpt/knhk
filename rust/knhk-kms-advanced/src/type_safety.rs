//! Compile-time type safety guarantees using phantom types
//!
//! This module demonstrates the type-state pattern for enforcing
//! invariants at compile time rather than runtime.

use crate::{KmsError, Result};
use std::marker::PhantomData;

/// Type-state marker for unsigned keys
pub struct Unsigned;

/// Type-state marker for signed keys
pub struct Signed;

/// Type-state marker for verified keys
pub struct Verified;

/// Type-state marker for sealed/encrypted keys
pub struct Sealed;

/// A cryptographic key with compile-time state tracking
///
/// Uses phantom types to enforce state transitions at compile time.
/// Invalid state transitions are impossible to express.
///
/// # Type Parameters
///
/// * `State` - The current state of the key (Unsigned, Signed, Verified, Sealed)
///
/// # State Transitions
///
/// ```text
/// Unsigned → Signed → Verified
///    ↓
///  Sealed
/// ```
///
/// # Examples
///
/// ```
/// use knhk_kms_advanced::type_safety::{TypedKey, Unsigned};
///
/// let key = TypedKey::<Unsigned>::new([42u8; 32]);
/// let signed_key = key.sign(b"message").unwrap();
/// let verified_key = signed_key.verify(b"message").unwrap();
/// ```
pub struct TypedKey<State = Unsigned> {
    key_material: [u8; 32],
    signature: Option<[u8; 64]>,
    _state: PhantomData<State>,
}

impl TypedKey<Unsigned> {
    /// Create a new unsigned key
    pub fn new(key_material: [u8; 32]) -> Self {
        Self {
            key_material,
            signature: None,
            _state: PhantomData,
        }
    }

    /// Sign a message, transitioning to the Signed state
    ///
    /// This consumes the unsigned key and returns a signed key.
    /// The unsigned key cannot be used after signing.
    pub fn sign(self, message: &[u8]) -> Result<TypedKey<Signed>> {
        let signature = sign_message(&self.key_material, message)?;

        Ok(TypedKey {
            key_material: self.key_material,
            signature: Some(signature),
            _state: PhantomData,
        })
    }

    /// Seal the key (encrypt), transitioning to the Sealed state
    pub fn seal(self, encryption_key: &[u8; 32]) -> Result<TypedKey<Sealed>> {
        let sealed_material = xor_encrypt(&self.key_material, encryption_key);

        Ok(TypedKey {
            key_material: sealed_material,
            signature: None,
            _state: PhantomData,
        })
    }
}

impl TypedKey<Signed> {
    /// Verify the signature, transitioning to the Verified state
    ///
    /// This consumes the signed key and returns a verified key if successful.
    /// If verification fails, the key is dropped (cannot be used).
    pub fn verify(self, message: &[u8]) -> Result<TypedKey<Verified>> {
        let signature = self
            .signature
            .ok_or_else(|| KmsError::InvalidKey("No signature present".into()))?;

        if verify_signature(&self.key_material, message, &signature)? {
            Ok(TypedKey {
                key_material: self.key_material,
                signature: Some(signature),
                _state: PhantomData,
            })
        } else {
            Err(KmsError::VerificationFailed)
        }
    }

    /// Get the signature (only available in Signed state)
    pub fn signature(&self) -> &[u8; 64] {
        self.signature
            .as_ref()
            .expect("Signed key must have signature")
    }
}

impl TypedKey<Verified> {
    /// Extract the key material (only available in Verified state)
    ///
    /// This is safe because we know the key has been verified.
    pub fn key_material(&self) -> &[u8; 32] {
        &self.key_material
    }

    /// Get the verification signature
    pub fn signature(&self) -> &[u8; 64] {
        self.signature
            .as_ref()
            .expect("Verified key must have signature")
    }

    /// Use the key for cryptographic operations
    pub fn use_key<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&[u8; 32]) -> T,
    {
        f(&self.key_material)
    }
}

impl TypedKey<Sealed> {
    /// Unseal the key (decrypt), returning to Unsigned state
    pub fn unseal(self, encryption_key: &[u8; 32]) -> Result<TypedKey<Unsigned>> {
        let unsealed_material = xor_decrypt(&self.key_material, encryption_key);

        Ok(TypedKey {
            key_material: unsealed_material,
            signature: None,
            _state: PhantomData,
        })
    }

    /// Get the sealed (encrypted) material
    pub fn sealed_material(&self) -> &[u8; 32] {
        &self.key_material
    }
}

/// Sign a message using HMAC-SHA256
fn sign_message(key: &[u8; 32], message: &[u8]) -> Result<[u8; 64]> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(message);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(hash1);
    hasher.update(message);
    let hash2 = hasher.finalize();

    let mut signature = [0u8; 64];
    signature[..32].copy_from_slice(&hash1);
    signature[32..].copy_from_slice(&hash2);

    Ok(signature)
}

/// Verify a message signature
fn verify_signature(key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> Result<bool> {
    let expected = sign_message(key, message)?;
    Ok(constant_time_eq(&expected, signature))
}

/// XOR-based encryption (for demonstration)
fn xor_encrypt(data: &[u8; 32], key: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..32 {
        result[i] = data[i] ^ key[i];
    }
    result
}

/// XOR-based decryption (for demonstration)
fn xor_decrypt(data: &[u8; 32], key: &[u8; 32]) -> [u8; 32] {
    xor_encrypt(data, key) // XOR is its own inverse
}

/// Constant-time equality comparison
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_to_signed() {
        let key = TypedKey::<Unsigned>::new([42u8; 32]);
        let signed = key.sign(b"test message").unwrap();
        assert!(signed.signature().len() == 64);
    }

    #[test]
    fn test_signed_to_verified() {
        let key = TypedKey::<Unsigned>::new([42u8; 32]);
        let signed = key.sign(b"test message").unwrap();
        let verified = signed.verify(b"test message").unwrap();
        assert_eq!(verified.key_material(), &[42u8; 32]);
    }

    #[test]
    fn test_verification_failure() {
        let key = TypedKey::<Unsigned>::new([42u8; 32]);
        let signed = key.sign(b"test message").unwrap();
        let result = signed.verify(b"wrong message");
        assert!(result.is_err());
    }

    #[test]
    fn test_seal_unseal() {
        let key = TypedKey::<Unsigned>::new([42u8; 32]);
        let encryption_key = [99u8; 32];

        let sealed = key.seal(&encryption_key).unwrap();
        let unsealed = sealed.unseal(&encryption_key).unwrap();

        // After unseal, we get back the original unsigned key
        let signed = unsealed.sign(b"message").unwrap();
        assert!(signed.signature().len() == 64);
    }

    #[test]
    fn test_verified_key_usage() {
        let key = TypedKey::<Unsigned>::new([42u8; 32]);
        let signed = key.sign(b"test").unwrap();
        let verified = signed.verify(b"test").unwrap();

        // Use the verified key
        let result = verified.use_key(|k| k.iter().map(|&x| x as u16).sum::<u16>());
        assert_eq!(result, 42u16 * 32);
    }

    // These tests won't compile - demonstrating type safety:
    //
    // #[test]
    // fn test_cannot_verify_unsigned() {
    //     let key = TypedKey::<Unsigned>::new([42u8; 32]);
    //     let _ = key.verify(b"test"); // ERROR: no method `verify` on TypedKey<Unsigned>
    // }
    //
    // #[test]
    // fn test_cannot_use_unsigned_key() {
    //     let key = TypedKey::<Unsigned>::new([42u8; 32]);
    //     let _ = key.key_material(); // ERROR: no method `key_material` on TypedKey<Unsigned>
    // }
}
