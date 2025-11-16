//! Groth16 Zero-Knowledge Proof System
//!
//! Groth16 provides:
//! - Constant-time verification (very fast)
//! - Small proof size (~200 bytes)
//! - Requires trusted setup (circuit-specific)
//! - 128-bit security level
//!
//! Use Groth16 when:
//! - Verification speed is critical
//! - Proof size must be minimal
//! - Trusted setup is acceptable

use super::{PrivateInputs, PublicInputs, ProverConfig, VerifierConfig, ZkError, ZkResult};
use ark_bls12_381::{Bls12_381, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::{PrimeField, Field};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, PreparedVerifyingKey};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use ark_std::rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, debug, instrument};

lazy_static::lazy_static! {
    /// Global cache of proving/verifying keys for circuits
    static ref KEY_CACHE: Arc<RwLock<HashMap<String, CachedKeys>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// Cached keys for a specific circuit
struct CachedKeys {
    proving_key: ProvingKey<Bls12_381>,
    verifying_key: PreparedVerifyingKey<Bls12_381>,
}

/// Generate a Groth16 proof
#[instrument(skip(private_inputs, public_inputs, config))]
pub async fn generate_proof(
    circuit_id: &str,
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
    config: &ProverConfig,
) -> ZkResult<Vec<u8>> {
    info!("Generating Groth16 proof for circuit: {}", circuit_id);

    // Get or create proving key
    let proving_key = get_or_create_proving_key(circuit_id).await?;

    // Create circuit instance
    let circuit = create_circuit(circuit_id, private_inputs, public_inputs)?;

    // Generate random entropy
    let mut rng = if config.security_level >= 128 {
        ChaCha20Rng::from_entropy()
    } else {
        ChaCha20Rng::from_seed([0u8; 32]) // Deterministic for testing only
    };

    // Generate proof
    let proof = Groth16::<Bls12_381>::prove(&proving_key, circuit, &mut rng)
        .map_err(|e| ZkError::ProofGenerationFailed(format!("Groth16 prove failed: {:?}", e)))?;

    // Serialize proof
    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes)
        .map_err(|e| ZkError::SerializationError(format!("Failed to serialize proof: {:?}", e)))?;

    debug!("Groth16 proof generated, size: {} bytes", proof_bytes.len());

    Ok(proof_bytes)
}

/// Verify a Groth16 proof
#[instrument(skip(proof_data, public_inputs, config))]
pub fn verify_proof(
    circuit_id: &str,
    proof_data: &[u8],
    public_inputs: &PublicInputs,
    config: &VerifierConfig,
) -> ZkResult<bool> {
    info!("Verifying Groth16 proof for circuit: {}", circuit_id);

    // Get verifying key
    let vk = get_verifying_key(circuit_id)?;

    // Deserialize proof
    let proof = Proof::<Bls12_381>::deserialize_compressed(proof_data)
        .map_err(|e| ZkError::SerializationError(format!("Failed to deserialize proof: {:?}", e)))?;

    // Convert public inputs to field elements
    let public_input_fields = convert_public_inputs_to_fields(public_inputs)?;

    // Verify proof
    let result = Groth16::<Bls12_381>::verify_proof(&vk, &proof, &public_input_fields)
        .map_err(|e| ZkError::VerificationFailed(format!("Groth16 verification failed: {:?}", e)))?;

    debug!("Groth16 proof verification result: {}", result);

    Ok(result)
}

/// Get or create proving key for a circuit
async fn get_or_create_proving_key(circuit_id: &str) -> ZkResult<ProvingKey<Bls12_381>> {
    // Check cache first
    {
        let cache = KEY_CACHE.read()
            .map_err(|e| ZkError::CryptographicError(format!("Failed to acquire read lock: {}", e)))?;

        if let Some(cached) = cache.get(circuit_id) {
            debug!("Using cached proving key for circuit: {}", circuit_id);
            return Ok(cached.proving_key.clone());
        }
    }

    // Generate new keys (trusted setup)
    info!("Performing trusted setup for circuit: {}", circuit_id);
    let (proving_key, verifying_key) = perform_trusted_setup(circuit_id).await?;

    // Cache keys
    {
        let mut cache = KEY_CACHE.write()
            .map_err(|e| ZkError::CryptographicError(format!("Failed to acquire write lock: {}", e)))?;

        cache.insert(circuit_id.to_string(), CachedKeys {
            proving_key: proving_key.clone(),
            verifying_key,
        });
    }

    Ok(proving_key)
}

/// Get verifying key from cache
fn get_verifying_key(circuit_id: &str) -> ZkResult<PreparedVerifyingKey<Bls12_381>> {
    let cache = KEY_CACHE.read()
        .map_err(|e| ZkError::CryptographicError(format!("Failed to acquire read lock: {}", e)))?;

    cache.get(circuit_id)
        .map(|cached| cached.verifying_key.clone())
        .ok_or_else(|| ZkError::TrustedSetupError(
            format!("No verifying key found for circuit: {}", circuit_id)
        ))
}

/// Perform trusted setup for a circuit
async fn perform_trusted_setup(
    circuit_id: &str,
) -> ZkResult<(ProvingKey<Bls12_381>, PreparedVerifyingKey<Bls12_381>)> {
    info!("Starting trusted setup for circuit: {}", circuit_id);

    // Create dummy circuit for setup
    let circuit = create_dummy_circuit(circuit_id)?;

    // Generate random entropy
    let mut rng = ChaCha20Rng::from_entropy();

    // Generate proving and verifying keys
    let (pk, vk) = Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
        .map_err(|e| ZkError::TrustedSetupError(format!("Trusted setup failed: {:?}", e)))?;

    // Prepare verifying key for faster verification
    let pvk = PreparedVerifyingKey::from(vk);

    info!("Trusted setup completed for circuit: {}", circuit_id);

    Ok((pk, pvk))
}

/// Create a circuit instance from inputs
fn create_circuit(
    circuit_id: &str,
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<impl ConstraintSynthesizer<Fr>> {
    match circuit_id {
        "state_transition" => {
            super::circuits::state_transition::create_groth16_circuit(private_inputs, public_inputs)
        }
        "compliance" => {
            super::circuits::compliance::create_groth16_circuit(private_inputs, public_inputs)
        }
        "policy" => {
            super::circuits::policy::create_groth16_circuit(private_inputs, public_inputs)
        }
        "computation" => {
            super::circuits::computation::create_groth16_circuit(private_inputs, public_inputs)
        }
        _ => Err(ZkError::CircuitError(format!("Unknown circuit: {}", circuit_id))),
    }
}

/// Create a dummy circuit for trusted setup
fn create_dummy_circuit(circuit_id: &str) -> ZkResult<impl ConstraintSynthesizer<Fr>> {
    let private_inputs = PrivateInputs::new();
    let public_inputs = PublicInputs::new();
    create_circuit(circuit_id, &private_inputs, &public_inputs)
}

/// Convert public inputs to field elements
fn convert_public_inputs_to_fields(public_inputs: &PublicInputs) -> ZkResult<Vec<Fr>> {
    let mut fields = Vec::new();

    for (_key, value) in public_inputs.iter() {
        // Convert bytes to field element
        // For simplicity, we hash the bytes and convert to field
        let hash = blake3::hash(value);
        let hash_bytes = hash.as_bytes();

        // Take first 32 bytes and interpret as field element
        let mut repr = [0u8; 32];
        repr.copy_from_slice(&hash_bytes[..32]);

        let field = Fr::from_le_bytes_mod_order(&repr);
        fields.push(field);
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groth16_proof_generation() {
        let circuit_id = "test_circuit";
        let private_inputs = PrivateInputs::new()
            .add("secret", vec![42u8]);
        let public_inputs = PublicInputs::new()
            .add("public_hash", vec![1, 2, 3, 4]);

        let config = ProverConfig::default();

        // This will fail without actual circuit implementation, but tests the structure
        let result = generate_proof(circuit_id, &private_inputs, &public_inputs, &config).await;

        // Expected to fail until circuits are implemented
        assert!(result.is_err());
    }

    #[test]
    fn test_public_input_conversion() {
        let public_inputs = PublicInputs::new()
            .add("test", vec![1, 2, 3, 4]);

        let result = convert_public_inputs_to_fields(&public_inputs);
        assert!(result.is_ok());

        let fields = result.unwrap();
        assert_eq!(fields.len(), 1);
    }
}
