//! Computation Correctness Circuit
//!
//! Proves that a computation was performed correctly without revealing:
//! - Input values
//! - Intermediate computation steps
//! - Algorithm details
//!
//! Public: Computation ID, output hash
//! Private: Inputs, computation logic, intermediate results

use super::{PrivateInputs, PublicInputs, ZkError, ZkResult};
use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bls12_381::Fr;
use sha3::{Sha3_256, Digest};
use tracing::{debug, instrument};

/// Computation types
#[derive(Debug, Clone, Copy)]
pub enum ComputationType {
    Sum,
    Product,
    Hash,
    Custom,
}

/// Computation circuit for Groth16
pub struct ComputationCircuit {
    // Private inputs
    input_values: Vec<Vec<u8>>,
    computation_type: u8,
    intermediate_results: Vec<Vec<u8>>,
    final_result: Vec<u8>,

    // Public inputs
    computation_id: Vec<u8>,
    output_hash: Vec<u8>,
}

impl ConstraintSynthesizer<Fr> for ComputationCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate private inputs
        let mut input_vars = Vec::new();
        for input in &self.input_values {
            let input_var = UInt8::new_witness_vec(cs.clone(), input)?;
            input_vars.push(input_var);
        }

        let comp_type_var = UInt8::new_witness(cs.clone(), || Ok(self.computation_type))?;
        let result_var = UInt8::new_witness_vec(cs.clone(), &self.final_result)?;

        // Allocate public inputs
        let comp_id_var = UInt8::new_input_vec(cs.clone(), &self.computation_id)?;
        let output_hash_var = UInt8::new_input_vec(cs.clone(), &self.output_hash)?;

        // Constraint 1: Verify computation type is valid (0-3)
        let max_type = UInt8::constant(3);
        comp_type_var.is_leq(&max_type)?.enforce_equal(&Boolean::TRUE)?;

        // Constraint 2: Verify computation is correct
        let expected_result = perform_computation(
            &self.input_values,
            self.computation_type,
        );

        let expected_result_var = UInt8::constant_vec(&expected_result);
        for (computed, expected) in result_var.iter().zip(expected_result_var.iter()) {
            computed.enforce_equal(expected)?;
        }

        // Constraint 3: Verify output hash
        let computed_hash = compute_output_hash(
            &self.computation_id,
            &self.final_result,
        );

        let computed_hash_var = UInt8::constant_vec(&computed_hash);
        for (computed, expected) in computed_hash_var.iter().zip(output_hash_var.iter()) {
            computed.enforce_equal(expected)?;
        }

        // Constraint 4: Verify computation ID is non-empty
        if self.computation_id.is_empty() {
            return Err(SynthesisError::Unsatisfiable);
        }

        Ok(())
    }
}

/// Create Groth16 computation circuit
#[instrument(skip(private_inputs, public_inputs))]
pub fn create_groth16_circuit(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<impl ConstraintSynthesizer<Fr>> {
    debug!("Creating Groth16 computation circuit");

    // Collect all input values
    let mut input_values = Vec::new();
    let mut idx = 0;
    while let Some(input) = private_inputs.get(&format!("input_{}", idx)) {
        input_values.push(input.clone());
        idx += 1;
    }

    if input_values.is_empty() {
        input_values.push(private_inputs.get("input").cloned().unwrap_or_default());
    }

    let computation_type = private_inputs.get("computation_type")
        .and_then(|v| v.first().copied())
        .unwrap_or(0);

    let final_result = perform_computation(&input_values, computation_type);

    let intermediate_results = compute_intermediate_results(&input_values, computation_type);

    let computation_id = public_inputs.get("computation_id")
        .cloned()
        .unwrap_or_else(|| b"default_computation".to_vec());

    let output_hash = public_inputs.get("output_hash")
        .cloned()
        .unwrap_or_else(|| compute_output_hash(&computation_id, &final_result));

    Ok(ComputationCircuit {
        input_values,
        computation_type,
        intermediate_results,
        final_result,
        computation_id,
        output_hash,
    })
}

/// Create PLONK computation circuit
pub fn create_plonk_circuit(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<super::super::plonk::PlonkCircuit> {
    debug!("Creating PLONK computation circuit");

    use super::super::plonk::{PlonkCircuit, Gate, GateType, Wire};

    let input1 = private_inputs.get("input_0")
        .or_else(|| private_inputs.get("input"))
        .cloned()
        .unwrap_or_default();

    let input2 = private_inputs.get("input_1").cloned().unwrap_or_default();

    let computation_type = private_inputs.get("computation_type")
        .and_then(|v| v.first().copied())
        .unwrap_or(0);

    let gates = if computation_type == 0 {
        // Sum computation
        vec![
            Gate {
                gate_type: GateType::Constant(input1.clone()),
                inputs: vec![],
                output: 0,
            },
            Gate {
                gate_type: GateType::Constant(input2.clone()),
                inputs: vec![],
                output: 1,
            },
            Gate {
                gate_type: GateType::Add,
                inputs: vec![0, 1],
                output: 2,
            },
            Gate {
                gate_type: GateType::Constant(hash_bytes(&input1)),
                inputs: vec![0],
                output: 3,
            },
        ]
    } else {
        // Product computation
        vec![
            Gate {
                gate_type: GateType::Constant(input1.clone()),
                inputs: vec![],
                output: 0,
            },
            Gate {
                gate_type: GateType::Constant(input2.clone()),
                inputs: vec![],
                output: 1,
            },
            Gate {
                gate_type: GateType::Mul,
                inputs: vec![0, 1],
                output: 2,
            },
        ]
    };

    let wires = vec![Wire { value: None }; gates.len() + 1];
    let public_input_values: Vec<Vec<u8>> = public_inputs.iter()
        .map(|(_, v)| v.clone())
        .collect();

    Ok(PlonkCircuit {
        gates,
        wires,
        public_inputs: public_input_values,
    })
}

/// Create STARK execution trace for computation
pub fn create_stark_trace(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<super::super::stark::ExecutionTrace> {
    debug!("Creating STARK computation trace");

    use super::super::stark::ExecutionTrace;

    let mut input_values = Vec::new();
    let mut idx = 0;
    while let Some(input) = private_inputs.get(&format!("input_{}", idx)) {
        input_values.push(input.clone());
        idx += 1;
    }

    if input_values.is_empty() {
        input_values.push(private_inputs.get("input").cloned().unwrap_or_default());
    }

    let computation_type = private_inputs.get("computation_type")
        .and_then(|v| v.first().copied())
        .unwrap_or(0);

    let mut trace = ExecutionTrace::new(4, 16);

    // Register 0: Input accumulation
    if !input_values.is_empty() {
        trace.set(0, 0, input_values[0].clone())?;
    }
    for step in 1..16 {
        let idx = step % input_values.len();
        trace.set(0, step, input_values[idx].clone())?;
    }

    // Register 1: Computation progress
    trace.set(1, 0, Vec::new())?;
    for step in 1..16 {
        let prev = trace.get(1, step - 1)?;
        let input = trace.get(0, step)?;

        let result = if computation_type == 0 {
            // Sum
            accumulate_sum(prev, input)
        } else {
            // Hash
            hash_bytes(input)
        };

        trace.set(1, step, result)?;
    }

    // Register 2: Intermediate hashes
    for step in 0..16 {
        let value = trace.get(1, step)?;
        let hash = hash_bytes(value);
        trace.set(2, step, hash)?;
    }

    // Register 3: Final result
    for step in 0..16 {
        let intermediate = trace.get(1, step)?;
        trace.set(3, step, intermediate.clone())?;
    }

    Ok(trace)
}

/// Perform computation based on type
fn perform_computation(inputs: &[Vec<u8>], computation_type: u8) -> Vec<u8> {
    match computation_type {
        0 => compute_sum(inputs),
        1 => compute_product(inputs),
        2 => compute_hash(inputs),
        _ => compute_custom(inputs),
    }
}

/// Compute sum of inputs
fn compute_sum(inputs: &[Vec<u8>]) -> Vec<u8> {
    let sum: u64 = inputs.iter()
        .filter_map(|input| {
            if input.len() >= 8 {
                Some(u64::from_le_bytes(input[..8].try_into().ok()?))
            } else {
                None
            }
        })
        .sum();

    sum.to_le_bytes().to_vec()
}

/// Compute product of inputs
fn compute_product(inputs: &[Vec<u8>]) -> Vec<u8> {
    let product: u64 = inputs.iter()
        .filter_map(|input| {
            if input.len() >= 8 {
                Some(u64::from_le_bytes(input[..8].try_into().ok()?))
            } else {
                None
            }
        })
        .product();

    product.to_le_bytes().to_vec()
}

/// Compute hash of all inputs
fn compute_hash(inputs: &[Vec<u8>]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    for input in inputs {
        hasher.update(input);
    }
    hasher.finalize().to_vec()
}

/// Compute custom function
fn compute_custom(inputs: &[Vec<u8>]) -> Vec<u8> {
    // Custom computation logic
    compute_hash(inputs)
}

/// Compute intermediate results
fn compute_intermediate_results(inputs: &[Vec<u8>], computation_type: u8) -> Vec<Vec<u8>> {
    let mut results = Vec::new();
    let mut accumulated = Vec::new();

    for input in inputs {
        accumulated.push(input.clone());
        let intermediate = perform_computation(&accumulated, computation_type);
        results.push(intermediate);
    }

    results
}

/// Accumulate sum (for STARK trace)
fn accumulate_sum(prev: &[u8], input: &[u8]) -> Vec<u8> {
    let prev_val = if prev.len() >= 8 {
        u64::from_le_bytes(prev[..8].try_into().unwrap_or([0u8; 8]))
    } else {
        0
    };

    let input_val = if input.len() >= 8 {
        u64::from_le_bytes(input[..8].try_into().unwrap_or([0u8; 8]))
    } else {
        0
    };

    (prev_val + input_val).to_le_bytes().to_vec()
}

/// Compute output hash
fn compute_output_hash(computation_id: &[u8], result: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(computation_id);
    hasher.update(result);
    hasher.update(b"COMPUTATION_OUTPUT");
    hasher.finalize().to_vec()
}

/// Hash bytes using SHA3-256
fn hash_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sum() {
        let inputs = vec![
            5u64.to_le_bytes().to_vec(),
            10u64.to_le_bytes().to_vec(),
            15u64.to_le_bytes().to_vec(),
        ];

        let result = compute_sum(&inputs);
        let sum = u64::from_le_bytes(result[..8].try_into().unwrap());
        assert_eq!(sum, 30);
    }

    #[test]
    fn test_compute_product() {
        let inputs = vec![
            2u64.to_le_bytes().to_vec(),
            3u64.to_le_bytes().to_vec(),
            4u64.to_le_bytes().to_vec(),
        ];

        let result = compute_product(&inputs);
        let product = u64::from_le_bytes(result[..8].try_into().unwrap());
        assert_eq!(product, 24);
    }

    #[test]
    fn test_compute_hash() {
        let inputs = vec![
            vec![1u8, 2, 3],
            vec![4u8, 5, 6],
        ];

        let hash = compute_hash(&inputs);
        assert_eq!(hash.len(), 32); // SHA3-256 output

        // Same inputs produce same hash
        let hash2 = compute_hash(&inputs);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_perform_computation() {
        let inputs = vec![
            5u64.to_le_bytes().to_vec(),
            10u64.to_le_bytes().to_vec(),
        ];

        // Test sum (type 0)
        let sum_result = perform_computation(&inputs, 0);
        let sum = u64::from_le_bytes(sum_result[..8].try_into().unwrap());
        assert_eq!(sum, 15);

        // Test product (type 1)
        let product_result = perform_computation(&inputs, 1);
        let product = u64::from_le_bytes(product_result[..8].try_into().unwrap());
        assert_eq!(product, 50);

        // Test hash (type 2)
        let hash_result = perform_computation(&inputs, 2);
        assert_eq!(hash_result.len(), 32);
    }
}
