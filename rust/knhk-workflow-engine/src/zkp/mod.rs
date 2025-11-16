//! Zero-Knowledge Proof Systems for Privacy-Preserving Workflow Verification
//!
//! This module provides three ZK proof systems:
//! - **Groth16**: Fast verification (constant-time), requires trusted setup
//! - **PLONK**: Universal setup, more flexible
//! - **STARK**: No trusted setup, quantum-resistant, larger proofs
//!
//! # Example
//!
//! ```no_run
//! use knhk_workflow_engine::zkp::*;
//!
//! # async fn example() -> Result<(), ZkError> {
//! // Initialize ZK prover
//! let prover = ZkProver::new(ProofSystem::Groth16)
//!     .with_circuit("state_transition")
//!     .build()?;
//!
//! // Create inputs
//! let private_inputs = PrivateInputs::new()
//!     .add("current_state", vec![42u8])
//!     .add("user_data", vec![1, 2, 3]);
//!
//! let public_inputs = PublicInputs::new()
//!     .add("workflow_id", vec![0x01, 0x02, 0x03, 0x04]);
//!
//! // Generate proof
//! let proof = prover.prove(&private_inputs, &public_inputs).await?;
//!
//! // Verify proof
//! let verifier = ZkVerifier::new(ProofSystem::Groth16);
//! assert!(verifier.verify(&proof, &public_inputs)?);
//! # Ok(())
//! # }
//! ```

pub mod groth16;
pub mod plonk;
pub mod stark;
pub mod privacy;
pub mod circuits;
pub mod governance;

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tracing::{info, warn, error, debug, instrument};

/// Zero-knowledge proof system variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofSystem {
    /// Groth16: Fast verification, requires trusted setup
    Groth16,
    /// PLONK: Universal setup, flexible
    Plonk,
    /// STARK: No trusted setup, quantum-resistant
    Stark,
}

/// Private inputs (hidden from verifier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInputs {
    inputs: HashMap<String, Vec<u8>>,
}

impl PrivateInputs {
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
        }
    }

    pub fn add(mut self, key: impl Into<String>, value: Vec<u8>) -> Self {
        self.inputs.insert(key.into(), value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.inputs.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.inputs.iter()
    }
}

impl Default for PrivateInputs {
    fn default() -> Self {
        Self::new()
    }
}

/// Public inputs (visible to verifier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    inputs: HashMap<String, Vec<u8>>,
}

impl PublicInputs {
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
        }
    }

    pub fn add(mut self, key: impl Into<String>, value: Vec<u8>) -> Self {
        self.inputs.insert(key.into(), value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.inputs.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<u8>)> {
        self.inputs.iter()
    }
}

impl Default for PublicInputs {
    fn default() -> Self {
        Self::new()
    }
}

/// Zero-knowledge proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    pub system: ProofSystem,
    pub circuit_id: String,
    pub proof_data: Vec<u8>,
    pub public_inputs: PublicInputs,
    pub metadata: ProofMetadata,
}

/// Proof metadata for tracking and auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub created_at: u64,
    pub proof_size_bytes: usize,
    pub generation_time_ms: u64,
    pub security_level: u32,
}

/// ZK proof errors
#[derive(Error, Debug)]
pub enum ZkError {
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Proof verification failed: {0}")]
    VerificationFailed(String),

    #[error("Circuit error: {0}")]
    CircuitError(String),

    #[error("Invalid inputs: {0}")]
    InvalidInputs(String),

    #[error("Trusted setup error: {0}")]
    TrustedSetupError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Cryptographic error: {0}")]
    CryptographicError(String),

    #[error("Invalid proof system: {0:?}")]
    InvalidProofSystem(ProofSystem),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type ZkResult<T> = Result<T, ZkError>;

/// Zero-knowledge prover
pub struct ZkProver {
    system: ProofSystem,
    circuit_id: Option<String>,
    config: ProverConfig,
}

/// Prover configuration
#[derive(Debug, Clone)]
pub struct ProverConfig {
    pub security_level: u32,
    pub enable_telemetry: bool,
    pub parallel_proving: bool,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            security_level: 128,
            enable_telemetry: true,
            parallel_proving: true,
        }
    }
}

impl ZkProver {
    /// Create a new ZK prover
    pub fn new(system: ProofSystem) -> Self {
        info!("Initializing ZK prover with system: {:?}", system);
        Self {
            system,
            circuit_id: None,
            config: ProverConfig::default(),
        }
    }

    /// Set the circuit ID
    pub fn with_circuit(mut self, circuit_id: impl Into<String>) -> Self {
        self.circuit_id = Some(circuit_id.into());
        self
    }

    /// Set prover configuration
    pub fn with_config(mut self, config: ProverConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the prover
    pub fn build(self) -> ZkResult<Self> {
        if self.circuit_id.is_none() {
            return Err(ZkError::InvalidInputs("Circuit ID not set".into()));
        }
        Ok(self)
    }

    /// Generate a zero-knowledge proof
    #[instrument(skip(self, private_inputs, public_inputs), fields(circuit_id = ?self.circuit_id))]
    pub async fn prove(
        &self,
        private_inputs: &PrivateInputs,
        public_inputs: &PublicInputs,
    ) -> ZkResult<ZkProof> {
        let start = std::time::Instant::now();

        info!(
            "Generating proof with system {:?} for circuit {:?}",
            self.system, self.circuit_id
        );

        let circuit_id = self.circuit_id.as_ref()
            .ok_or_else(|| ZkError::InvalidInputs("Circuit ID not set".into()))?;

        let proof_data = match self.system {
            ProofSystem::Groth16 => {
                groth16::generate_proof(circuit_id, private_inputs, public_inputs, &self.config).await?
            }
            ProofSystem::Plonk => {
                plonk::generate_proof(circuit_id, private_inputs, public_inputs, &self.config).await?
            }
            ProofSystem::Stark => {
                stark::generate_proof(circuit_id, private_inputs, public_inputs, &self.config).await?
            }
        };

        let generation_time_ms = start.elapsed().as_millis() as u64;

        info!(
            "Proof generated in {}ms, size: {} bytes",
            generation_time_ms,
            proof_data.len()
        );

        Ok(ZkProof {
            system: self.system,
            circuit_id: circuit_id.clone(),
            proof_data,
            public_inputs: public_inputs.clone(),
            metadata: ProofMetadata {
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                proof_size_bytes: proof_data.len(),
                generation_time_ms,
                security_level: self.config.security_level,
            },
        })
    }
}

/// Zero-knowledge verifier
pub struct ZkVerifier {
    system: ProofSystem,
    config: VerifierConfig,
}

/// Verifier configuration
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    pub enable_telemetry: bool,
    pub strict_validation: bool,
}

impl Default for VerifierConfig {
    fn default() -> Self {
        Self {
            enable_telemetry: true,
            strict_validation: true,
        }
    }
}

impl ZkVerifier {
    /// Create a new ZK verifier
    pub fn new(system: ProofSystem) -> Self {
        info!("Initializing ZK verifier with system: {:?}", system);
        Self {
            system,
            config: VerifierConfig::default(),
        }
    }

    /// Set verifier configuration
    pub fn with_config(mut self, config: VerifierConfig) -> Self {
        self.config = config;
        self
    }

    /// Verify a zero-knowledge proof
    #[instrument(skip(self, proof, public_inputs), fields(circuit_id = ?proof.circuit_id))]
    pub fn verify(
        &self,
        proof: &ZkProof,
        public_inputs: &PublicInputs,
    ) -> ZkResult<bool> {
        let start = std::time::Instant::now();

        if proof.system != self.system {
            return Err(ZkError::VerificationFailed(
                format!("Proof system mismatch: expected {:?}, got {:?}",
                    self.system, proof.system)
            ));
        }

        info!(
            "Verifying proof with system {:?} for circuit {}",
            self.system, proof.circuit_id
        );

        let result = match self.system {
            ProofSystem::Groth16 => {
                groth16::verify_proof(&proof.circuit_id, &proof.proof_data, public_inputs, &self.config)?
            }
            ProofSystem::Plonk => {
                plonk::verify_proof(&proof.circuit_id, &proof.proof_data, public_inputs, &self.config)?
            }
            ProofSystem::Stark => {
                stark::verify_proof(&proof.circuit_id, &proof.proof_data, public_inputs, &self.config)?
            }
        };

        let verification_time_ms = start.elapsed().as_millis();

        info!(
            "Proof verification completed in {}ms: {}",
            verification_time_ms,
            if result { "VALID" } else { "INVALID" }
        );

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_inputs() {
        let inputs = PrivateInputs::new()
            .add("key1", vec![1, 2, 3])
            .add("key2", vec![4, 5, 6]);

        assert_eq!(inputs.get("key1"), Some(&vec![1, 2, 3]));
        assert_eq!(inputs.get("key2"), Some(&vec![4, 5, 6]));
        assert_eq!(inputs.get("key3"), None);
    }

    #[test]
    fn test_public_inputs() {
        let inputs = PublicInputs::new()
            .add("public_key", vec![0xAB, 0xCD]);

        assert_eq!(inputs.get("public_key"), Some(&vec![0xAB, 0xCD]));
    }

    #[test]
    fn test_prover_builder() {
        let prover = ZkProver::new(ProofSystem::Groth16)
            .with_circuit("test_circuit")
            .build();

        assert!(prover.is_ok());
    }

    #[test]
    fn test_prover_builder_without_circuit() {
        let prover = ZkProver::new(ProofSystem::Groth16)
            .build();

        assert!(prover.is_err());
    }
}
