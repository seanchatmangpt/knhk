//! Privacy-Preserving Workflow Primitives
//!
//! This module provides utilities for privacy-preserving workflow operations:
//! - Data anonymization and pseudonymization
//! - Differential privacy mechanisms
//! - Secure multi-party computation helpers
//! - Homomorphic encryption wrappers

use super::{ZkError, ZkResult};
use sha3::{Sha3_256, Digest};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use tracing::{debug, instrument};

/// Privacy level for data protection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyLevel {
    /// No privacy protection
    None,
    /// Basic anonymization (hashing, pseudonyms)
    Basic,
    /// Differential privacy with epsilon budget
    Differential { epsilon: f64 },
    /// Zero-knowledge proofs required
    ZeroKnowledge,
    /// Full homomorphic encryption
    Homomorphic,
}

/// Anonymize sensitive data
#[instrument(skip(data))]
pub fn anonymize_data(data: &[u8], salt: &[u8]) -> Vec<u8> {
    debug!("Anonymizing data with {} bytes", data.len());

    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.update(salt);
    hasher.update(b"ANONYMIZE_V1");

    hasher.finalize().to_vec()
}

/// Pseudonymize data (reversible with key)
#[instrument(skip(data, key))]
pub fn pseudonymize_data(data: &[u8], key: &[u8]) -> ZkResult<Vec<u8>> {
    debug!("Pseudonymizing data");

    // Simplified pseudonymization using XOR with derived key
    let derived_key = derive_key(key, b"PSEUDONYM");

    let mut result = data.to_vec();
    for (i, byte) in result.iter_mut().enumerate() {
        *byte ^= derived_key[i % derived_key.len()];
    }

    Ok(result)
}

/// Reverse pseudonymization
#[instrument(skip(pseudonym, key))]
pub fn depseudonymize_data(pseudonym: &[u8], key: &[u8]) -> ZkResult<Vec<u8>> {
    debug!("Depseudonymizing data");

    // XOR is its own inverse
    pseudonymize_data(pseudonym, key)
}

/// Add differential privacy noise
#[instrument(skip(value))]
pub fn add_laplace_noise(value: f64, epsilon: f64, sensitivity: f64) -> f64 {
    debug!("Adding Laplace noise with epsilon={}, sensitivity={}", epsilon, sensitivity);

    let mut rng = ChaCha20Rng::from_entropy();

    // Laplace distribution: scale = sensitivity / epsilon
    let scale = sensitivity / epsilon;

    // Generate Laplace noise
    let u: f64 = rng.gen_range(-0.5..0.5);
    let noise = -scale * u.signum() * (1.0 - 2.0 * u.abs()).ln();

    value + noise
}

/// Add Gaussian noise for differential privacy
#[instrument(skip(value))]
pub fn add_gaussian_noise(value: f64, sigma: f64) -> f64 {
    debug!("Adding Gaussian noise with sigma={}", sigma);

    let mut rng = ChaCha20Rng::from_entropy();

    // Box-Muller transform for Gaussian distribution
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();

    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    let noise = sigma * z0;

    value + noise
}

/// K-anonymity grouping
#[derive(Debug)]
pub struct KAnonymity {
    k: usize,
    groups: HashMap<Vec<u8>, Vec<Vec<u8>>>,
}

impl KAnonymity {
    /// Create new k-anonymity grouper
    pub fn new(k: usize) -> Self {
        Self {
            k,
            groups: HashMap::new(),
        }
    }

    /// Add record to anonymity set
    pub fn add_record(&mut self, quasi_identifiers: Vec<u8>, sensitive_data: Vec<u8>) {
        self.groups
            .entry(quasi_identifiers)
            .or_insert_with(Vec::new)
            .push(sensitive_data);
    }

    /// Check if k-anonymity is satisfied
    pub fn is_k_anonymous(&self) -> bool {
        self.groups.values().all(|group| group.len() >= self.k)
    }

    /// Get anonymized groups
    pub fn get_anonymous_groups(&self) -> Vec<(Vec<u8>, Vec<Vec<u8>>)> {
        self.groups
            .iter()
            .filter(|(_, group)| group.len() >= self.k)
            .map(|(qi, group)| (qi.clone(), group.clone()))
            .collect()
    }
}

/// L-diversity checker
#[derive(Debug)]
pub struct LDiversity {
    l: usize,
    groups: HashMap<Vec<u8>, Vec<Vec<u8>>>,
}

impl LDiversity {
    /// Create new l-diversity checker
    pub fn new(l: usize) -> Self {
        Self {
            l,
            groups: HashMap::new(),
        }
    }

    /// Add record
    pub fn add_record(&mut self, quasi_identifiers: Vec<u8>, sensitive_value: Vec<u8>) {
        self.groups
            .entry(quasi_identifiers)
            .or_insert_with(Vec::new)
            .push(sensitive_value);
    }

    /// Check if l-diversity is satisfied
    pub fn is_l_diverse(&self) -> bool {
        self.groups.values().all(|group| {
            // Count distinct sensitive values
            let mut unique_values: Vec<_> = group.clone();
            unique_values.sort();
            unique_values.dedup();

            unique_values.len() >= self.l
        })
    }
}

/// Homomorphic encryption helpers (simplified)
pub struct HomomorphicEncryption {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

impl HomomorphicEncryption {
    /// Generate key pair
    #[instrument]
    pub fn generate_keys() -> ZkResult<Self> {
        debug!("Generating homomorphic encryption keys");

        let mut rng = ChaCha20Rng::from_entropy();

        let mut public_key = vec![0u8; 32];
        let mut private_key = vec![0u8; 32];

        rng.fill(&mut public_key[..]);
        rng.fill(&mut private_key[..]);

        Ok(Self {
            public_key,
            private_key,
        })
    }

    /// Encrypt value (simplified)
    #[instrument(skip(self, plaintext))]
    pub fn encrypt(&self, plaintext: u64) -> Vec<u8> {
        debug!("Encrypting value");

        let mut hasher = Sha3_256::new();
        hasher.update(&self.public_key);
        hasher.update(&plaintext.to_le_bytes());
        hasher.update(b"HE_ENCRYPT");

        hasher.finalize().to_vec()
    }

    /// Decrypt value (simplified)
    #[instrument(skip(self, ciphertext))]
    pub fn decrypt(&self, ciphertext: &[u8]) -> ZkResult<u64> {
        debug!("Decrypting value");

        // Simplified: In production, use actual HE scheme (BFV, CKKS, etc.)
        let mut hasher = Sha3_256::new();
        hasher.update(&self.private_key);
        hasher.update(ciphertext);
        hasher.update(b"HE_DECRYPT");

        let hash = hasher.finalize();
        let value = u64::from_le_bytes(hash[..8].try_into().unwrap_or([0u8; 8]));

        Ok(value)
    }

    /// Homomorphic addition (ciphertext + ciphertext)
    #[instrument(skip(c1, c2))]
    pub fn add(c1: &[u8], c2: &[u8]) -> Vec<u8> {
        debug!("Homomorphic addition");

        let mut hasher = Sha3_256::new();
        hasher.update(c1);
        hasher.update(c2);
        hasher.update(b"HE_ADD");

        hasher.finalize().to_vec()
    }

    /// Homomorphic multiplication (ciphertext * plaintext)
    #[instrument(skip(c, p))]
    pub fn multiply_plain(c: &[u8], p: u64) -> Vec<u8> {
        debug!("Homomorphic scalar multiplication");

        let mut hasher = Sha3_256::new();
        hasher.update(c);
        hasher.update(&p.to_le_bytes());
        hasher.update(b"HE_MUL_PLAIN");

        hasher.finalize().to_vec()
    }
}

/// Secure aggregation for privacy-preserving statistics
pub struct SecureAggregation {
    shares: Vec<Vec<u8>>,
    threshold: usize,
}

impl SecureAggregation {
    /// Create new secure aggregation
    pub fn new(threshold: usize) -> Self {
        Self {
            shares: Vec::new(),
            threshold,
        }
    }

    /// Add share from participant
    pub fn add_share(&mut self, share: Vec<u8>) {
        self.shares.push(share);
    }

    /// Compute aggregate if threshold is met
    pub fn aggregate(&self) -> ZkResult<Vec<u8>> {
        if self.shares.len() < self.threshold {
            return Err(ZkError::InvalidInputs(
                format!("Insufficient shares: {} < {}", self.shares.len(), self.threshold)
            ));
        }

        // XOR all shares together (simplified)
        let mut result = vec![0u8; 32];

        for share in &self.shares {
            for (i, &byte) in share.iter().enumerate() {
                if i < result.len() {
                    result[i] ^= byte;
                }
            }
        }

        Ok(result)
    }
}

/// Derive key from master key and context
fn derive_key(master_key: &[u8], context: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(master_key);
    hasher.update(context);
    hasher.update(b"KEY_DERIVE_V1");

    hasher.finalize().to_vec()
}

/// Compute privacy budget remaining
pub fn compute_privacy_budget(
    total_epsilon: f64,
    queries: &[(f64, f64)], // (epsilon, sensitivity) pairs
) -> f64 {
    let spent: f64 = queries.iter().map(|(eps, _)| eps).sum();
    total_epsilon - spent
}

/// Check if privacy budget is exhausted
pub fn is_privacy_budget_exhausted(
    total_epsilon: f64,
    queries: &[(f64, f64)],
) -> bool {
    compute_privacy_budget(total_epsilon, queries) <= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anonymize_data() {
        let data = b"sensitive data";
        let salt = b"random_salt";

        let anonymized = anonymize_data(data, salt);
        assert_eq!(anonymized.len(), 32); // SHA3-256 output

        // Same inputs produce same output
        let anonymized2 = anonymize_data(data, salt);
        assert_eq!(anonymized, anonymized2);

        // Different salt produces different output
        let anonymized3 = anonymize_data(data, b"different_salt");
        assert_ne!(anonymized, anonymized3);
    }

    #[test]
    fn test_pseudonymize_data() {
        let data = b"John Doe";
        let key = b"secret_key";

        let pseudonym = pseudonymize_data(data, key).unwrap();
        assert_ne!(pseudonym, data);

        // Reversible
        let original = depseudonymize_data(&pseudonym, key).unwrap();
        assert_eq!(original, data);
    }

    #[test]
    fn test_laplace_noise() {
        let value = 100.0;
        let epsilon = 1.0;
        let sensitivity = 1.0;

        let noisy_value = add_laplace_noise(value, epsilon, sensitivity);

        // Noise should be relatively small for epsilon=1.0
        assert!((noisy_value - value).abs() < 10.0);
    }

    #[test]
    fn test_k_anonymity() {
        let mut k_anon = KAnonymity::new(2);

        // Add records with same quasi-identifiers
        k_anon.add_record(vec![1, 2, 3], vec![10]);
        k_anon.add_record(vec![1, 2, 3], vec![20]);

        assert!(k_anon.is_k_anonymous());

        // Add record with different quasi-identifiers
        k_anon.add_record(vec![4, 5, 6], vec![30]);

        assert!(!k_anon.is_k_anonymous());
    }

    #[test]
    fn test_l_diversity() {
        let mut l_div = LDiversity::new(2);

        // Add records with same quasi-identifiers but different sensitive values
        l_div.add_record(vec![1, 2, 3], vec![10]);
        l_div.add_record(vec![1, 2, 3], vec![20]);

        assert!(l_div.is_l_diverse());

        // Add records with same sensitive value
        l_div.add_record(vec![4, 5, 6], vec![30]);
        l_div.add_record(vec![4, 5, 6], vec![30]);

        assert!(!l_div.is_l_diverse());
    }

    #[test]
    fn test_homomorphic_encryption() {
        let he = HomomorphicEncryption::generate_keys().unwrap();

        let plaintext = 42;
        let ciphertext = he.encrypt(plaintext);

        // Ciphertext should be different from plaintext
        assert_ne!(ciphertext, plaintext.to_le_bytes());

        // Homomorphic operations
        let c1 = he.encrypt(10);
        let c2 = he.encrypt(20);
        let _c_sum = HomomorphicEncryption::add(&c1, &c2);

        let _c_mul = HomomorphicEncryption::multiply_plain(&c1, 5);
    }

    #[test]
    fn test_secure_aggregation() {
        let mut agg = SecureAggregation::new(3);

        agg.add_share(vec![1, 2, 3, 4]);
        agg.add_share(vec![5, 6, 7, 8]);

        // Should fail (not enough shares)
        assert!(agg.aggregate().is_err());

        agg.add_share(vec![9, 10, 11, 12]);

        // Should succeed
        assert!(agg.aggregate().is_ok());
    }

    #[test]
    fn test_privacy_budget() {
        let total_epsilon = 1.0;
        let queries = vec![
            (0.3, 1.0),
            (0.2, 1.0),
        ];

        let remaining = compute_privacy_budget(total_epsilon, &queries);
        assert_eq!(remaining, 0.5);

        assert!(!is_privacy_budget_exhausted(total_epsilon, &queries));

        let more_queries = vec![
            (0.3, 1.0),
            (0.2, 1.0),
            (0.6, 1.0),
        ];

        assert!(is_privacy_budget_exhausted(total_epsilon, &more_queries));
    }
}
