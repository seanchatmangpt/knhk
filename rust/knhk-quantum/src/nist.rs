//! NIST PQC Compliance Validation
//!
//! Validates that quantum-safe algorithms conform to NIST Post-Quantum Cryptography standards.
//! All selected algorithms are from NIST Round 3 standardization process.

use thiserror::Error;
use serde::{Deserialize, Serialize};

/// NIST compliance errors
#[derive(Error, Debug)]
pub enum NISTError {
    #[error("Algorithm not NIST-approved: {0}")]
    NotNISTApproved(String),
    #[error("Insufficient security level: {0:?} < {1:?}")]
    InsufficientLevel(NISTLevel, NISTLevel),
    #[error("Parameter validation failed: {0}")]
    ParameterValidation(String),
    #[error("Invalid security claim")]
    InvalidSecurityClaim,
}

pub type Result<T> = std::result::Result<T, NISTError>;

/// NIST Security Levels for PQC
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum NISTLevel {
    /// Level 1: ≥ 128-bit classical security
    Level1 = 1,
    /// Level 2: ≥ 192-bit classical security
    Level2 = 2,
    /// Level 3: ≥ 256-bit classical security
    Level3 = 3,
    /// Level 4: ≥ 192-bit post-quantum security
    Level4 = 4,
    /// Level 5: ≥ 256-bit post-quantum security
    Level5 = 5,
}

impl std::fmt::Display for NISTLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NISTLevel::Level1 => write!(f, "Level 1 (128-bit)"),
            NISTLevel::Level2 => write!(f, "Level 2 (192-bit)"),
            NISTLevel::Level3 => write!(f, "Level 3 (256-bit)"),
            NISTLevel::Level4 => write!(f, "Level 4 (192-bit PQC)"),
            NISTLevel::Level5 => write!(f, "Level 5 (256-bit PQC)"),
        }
    }
}

/// NIST PQC Algorithm information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NISTAlgorithm {
    pub name: String,
    pub family: AlgorithmFamily,
    pub category: AlgorithmCategory,
    pub nist_level: NISTLevel,
    pub key_encapsulation: bool,
    pub digital_signature: bool,
    pub reference: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgorithmFamily {
    Lattice,
    Multivariate,
    HashBased,
    CodeBased,
    IsogenyBased,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgorithmCategory {
    Primary,
    Alternative,
}

impl NISTAlgorithm {
    /// Create Kyber768 algorithm descriptor (Primary KEM, NIST Level 1)
    pub fn kyber768() -> Self {
        Self {
            name: "Kyber768".to_string(),
            family: AlgorithmFamily::Lattice,
            category: AlgorithmCategory::Primary,
            nist_level: NISTLevel::Level1,
            key_encapsulation: true,
            digital_signature: false,
            reference: "NIST PQC Round 3 - ML-KEM (Kyber)".to_string(),
        }
    }

    /// Create Kyber1024 algorithm descriptor (Primary KEM, NIST Level 5)
    pub fn kyber1024() -> Self {
        Self {
            name: "Kyber1024".to_string(),
            family: AlgorithmFamily::Lattice,
            category: AlgorithmCategory::Primary,
            nist_level: NISTLevel::Level5,
            key_encapsulation: true,
            digital_signature: false,
            reference: "NIST PQC Round 3 - ML-KEM (Kyber)".to_string(),
        }
    }

    /// Create Dilithium3 algorithm descriptor (Primary Signature, NIST Level 2)
    pub fn dilithium3() -> Self {
        Self {
            name: "Dilithium3".to_string(),
            family: AlgorithmFamily::Lattice,
            category: AlgorithmCategory::Primary,
            nist_level: NISTLevel::Level2,
            key_encapsulation: false,
            digital_signature: true,
            reference: "NIST PQC Round 3 - ML-DSA (Dilithium)".to_string(),
        }
    }

    /// Create Dilithium5 algorithm descriptor (Primary Signature, NIST Level 5)
    pub fn dilithium5() -> Self {
        Self {
            name: "Dilithium5".to_string(),
            family: AlgorithmFamily::Lattice,
            category: AlgorithmCategory::Primary,
            nist_level: NISTLevel::Level5,
            key_encapsulation: false,
            digital_signature: true,
            reference: "NIST PQC Round 3 - ML-DSA (Dilithium)".to_string(),
        }
    }

    /// Validate algorithm against security requirements
    pub fn validate(&self, min_level: NISTLevel) -> Result<()> {
        if self.nist_level < min_level {
            return Err(NISTError::InsufficientLevel(self.nist_level, min_level));
        }
        Ok(())
    }

    /// Check if algorithm is suitable for key encapsulation
    pub fn supports_kem(&self) -> bool {
        self.key_encapsulation
    }

    /// Check if algorithm is suitable for digital signatures
    pub fn supports_sig(&self) -> bool {
        self.digital_signature
    }
}

/// NIST PQC validation and compliance checking
pub struct NISTCompliance;

impl NISTCompliance {
    /// Validate that a set of algorithms meets NIST requirements
    pub fn validate_suite(algorithms: &[NISTAlgorithm], min_level: NISTLevel) -> Result<()> {
        let mut has_kem = false;
        let mut has_sig = false;

        for algo in algorithms {
            algo.validate(min_level)?;
            if algo.supports_kem() {
                has_kem = true;
            }
            if algo.supports_sig() {
                has_sig = true;
            }
        }

        if !has_kem {
            return Err(NISTError::NotNISTApproved(
                "No approved KEM algorithm found".to_string(),
            ));
        }

        if !has_sig {
            return Err(NISTError::NotNISTApproved(
                "No approved signature algorithm found".to_string(),
            ));
        }

        Ok(())
    }

    /// Get recommended algorithm suite for a target NIST level
    pub fn recommended_suite(level: NISTLevel) -> Vec<NISTAlgorithm> {
        match level {
            NISTLevel::Level1 | NISTLevel::Level2 | NISTLevel::Level3 => {
                vec![NISTAlgorithm::kyber768(), NISTAlgorithm::dilithium3()]
            }
            NISTLevel::Level4 | NISTLevel::Level5 => {
                vec![NISTAlgorithm::kyber1024(), NISTAlgorithm::dilithium5()]
            }
        }
    }

    /// Verify that claimed security level is achieved by algorithm suite
    pub fn verify_security_claim(suite: &[NISTAlgorithm]) -> Result<NISTLevel> {
        if suite.is_empty() {
            return Err(NISTError::InvalidSecurityClaim);
        }

        // Security level is determined by the weakest algorithm
        let min_level = suite
            .iter()
            .map(|a| a.nist_level)
            .min()
            .ok_or(NISTError::InvalidSecurityClaim)?;

        Ok(min_level)
    }
}

/// Validate NIST PQC compliance for the entire system
pub fn validate_nist_compliance(min_level: NISTLevel) -> Result<NISTLevel> {
    let suite = NISTCompliance::recommended_suite(min_level);
    NISTCompliance::validate_suite(&suite, min_level)?;
    NISTCompliance::verify_security_claim(&suite)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_levels_ordering() {
        assert!(NISTLevel::Level1 < NISTLevel::Level5);
        assert!(NISTLevel::Level3 < NISTLevel::Level5);
    }

    #[test]
    fn test_kyber768_properties() {
        let algo = NISTAlgorithm::kyber768();
        assert!(algo.supports_kem());
        assert!(!algo.supports_sig());
        assert_eq!(algo.nist_level, NISTLevel::Level1);
    }

    #[test]
    fn test_dilithium3_properties() {
        let algo = NISTAlgorithm::dilithium3();
        assert!(!algo.supports_kem());
        assert!(algo.supports_sig());
        assert_eq!(algo.nist_level, NISTLevel::Level2);
    }

    #[test]
    fn test_nist_suite_validation() {
        let suite = vec![NISTAlgorithm::kyber768(), NISTAlgorithm::dilithium3()];
        NISTCompliance::validate_suite(&suite, NISTLevel::Level1)
            .expect("Suite validation failed");
    }

    #[test]
    fn test_nist_suite_insufficient_level() {
        let suite = vec![NISTAlgorithm::kyber768()];
        let result = NISTCompliance::validate_suite(&suite, NISTLevel::Level1);
        assert!(result.is_err());
    }

    #[test]
    fn test_recommended_suite_level1() {
        let suite = NISTCompliance::recommended_suite(NISTLevel::Level1);
        assert_eq!(suite.len(), 2);
        assert!(suite.iter().any(|a| a.name == "Kyber768"));
        assert!(suite.iter().any(|a| a.name == "Dilithium3"));
    }

    #[test]
    fn test_verify_security_claim() {
        let suite = vec![NISTAlgorithm::kyber768(), NISTAlgorithm::dilithium3()];
        let level = NISTCompliance::verify_security_claim(&suite)
            .expect("Security claim verification failed");
        assert_eq!(level, NISTLevel::Level1);
    }

    #[test]
    fn test_validate_nist_compliance() {
        let level = validate_nist_compliance(NISTLevel::Level1)
            .expect("NIST compliance validation failed");
        assert!(level >= NISTLevel::Level1);
    }
}
