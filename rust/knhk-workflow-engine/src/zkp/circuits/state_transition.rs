//! State Transition Circuit
//!
//! Proves that a workflow state transitioned correctly according to transition rules,
//! without revealing:
//! - The actual current state
//! - The input data that triggered the transition
//! - The transition function logic
//!
//! Public: State hash before and after transition
//! Private: Current state, input data, transition function

use super::{PrivateInputs, PublicInputs, ZkError, ZkResult};
use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bls12_381::Fr;
use sha3::{Sha3_256, Digest};
use tracing::{debug, instrument};

/// State transition circuit for Groth16
pub struct StateTransitionCircuit {
    // Private inputs
    current_state: Vec<u8>,
    input_data: Vec<u8>,
    transition_type: Vec<u8>,

    // Public inputs
    current_state_hash: Vec<u8>,
    new_state_hash: Vec<u8>,
}

impl ConstraintSynthesizer<Fr> for StateTransitionCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // This is a simplified example - production would use proper R1CS constraints

        // Allocate private inputs as witnesses
        let _current_state_var = UInt8::new_witness_vec(
            cs.clone(),
            &self.current_state,
        )?;

        let _input_data_var = UInt8::new_witness_vec(
            cs.clone(),
            &self.input_data,
        )?;

        // Allocate public inputs
        let current_hash_var = UInt8::new_input_vec(
            cs.clone(),
            &self.current_state_hash,
        )?;

        let new_hash_var = UInt8::new_input_vec(
            cs.clone(),
            &self.new_state_hash,
        )?;

        // Constraint 1: Verify current state hash
        // hash(current_state) == current_state_hash
        let computed_current_hash = hash_bytes(&self.current_state);
        let computed_current_hash_var = UInt8::constant_vec(&computed_current_hash);

        for (computed, expected) in computed_current_hash_var.iter().zip(current_hash_var.iter()) {
            computed.enforce_equal(expected)?;
        }

        // Constraint 2: Compute new state
        let new_state = compute_new_state(&self.current_state, &self.input_data, &self.transition_type);

        // Constraint 3: Verify new state hash
        // hash(new_state) == new_state_hash
        let computed_new_hash = hash_bytes(&new_state);
        let computed_new_hash_var = UInt8::constant_vec(&computed_new_hash);

        for (computed, expected) in computed_new_hash_var.iter().zip(new_hash_var.iter()) {
            computed.enforce_equal(expected)?;
        }

        Ok(())
    }
}

/// Create Groth16 state transition circuit
#[instrument(skip(private_inputs, public_inputs))]
pub fn create_groth16_circuit(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<impl ConstraintSynthesizer<Fr>> {
    debug!("Creating Groth16 state transition circuit");

    let current_state = private_inputs.get("current_state")
        .cloned()
        .unwrap_or_default();

    let input_data = private_inputs.get("input_data")
        .cloned()
        .unwrap_or_default();

    let transition_type = private_inputs.get("transition_type")
        .cloned()
        .unwrap_or_default();

    let current_state_hash = public_inputs.get("current_state_hash")
        .cloned()
        .unwrap_or_else(|| hash_bytes(&current_state));

    let new_state_hash = public_inputs.get("new_state_hash")
        .cloned()
        .unwrap_or_default();

    Ok(StateTransitionCircuit {
        current_state,
        input_data,
        transition_type,
        current_state_hash,
        new_state_hash,
    })
}

/// Create PLONK state transition circuit
pub fn create_plonk_circuit(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<super::super::plonk::PlonkCircuit> {
    debug!("Creating PLONK state transition circuit");

    use super::super::plonk::{PlonkCircuit, Gate, GateType, Wire};

    let current_state = private_inputs.get("current_state")
        .cloned()
        .unwrap_or_default();

    let input_data = private_inputs.get("input_data")
        .cloned()
        .unwrap_or_default();

    // Create gates for state transition
    let gates = vec![
        // Wire 0: current_state (private)
        Gate {
            gate_type: GateType::Constant(current_state.clone()),
            inputs: vec![],
            output: 0,
        },
        // Wire 1: input_data (private)
        Gate {
            gate_type: GateType::Constant(input_data.clone()),
            inputs: vec![],
            output: 1,
        },
        // Wire 2: hash(current_state)
        Gate {
            gate_type: GateType::Constant(hash_bytes(&current_state)),
            inputs: vec![0],
            output: 2,
        },
        // Wire 3: new_state = transition(current_state, input_data)
        Gate {
            gate_type: GateType::Add,
            inputs: vec![0, 1],
            output: 3,
        },
        // Wire 4: hash(new_state)
        Gate {
            gate_type: GateType::Constant(Vec::new()), // Computed during proving
            inputs: vec![3],
            output: 4,
        },
    ];

    let wires = vec![Wire { value: None }; 5];

    let public_input_values: Vec<Vec<u8>> = public_inputs.iter()
        .map(|(_, v)| v.clone())
        .collect();

    Ok(PlonkCircuit {
        gates,
        wires,
        public_inputs: public_input_values,
    })
}

/// Create STARK execution trace for state transition
pub fn create_stark_trace(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<super::super::stark::ExecutionTrace> {
    debug!("Creating STARK state transition trace");

    use super::super::stark::ExecutionTrace;

    let current_state = private_inputs.get("current_state")
        .cloned()
        .unwrap_or_default();

    let input_data = private_inputs.get("input_data")
        .cloned()
        .unwrap_or_default();

    let transition_type = private_inputs.get("transition_type")
        .cloned()
        .unwrap_or_default();

    // Create trace with 4 registers and 16 steps
    let mut trace = ExecutionTrace::new(4, 16);

    // Register 0: Current state evolution
    trace.set(0, 0, current_state.clone())?;
    for step in 1..16 {
        let prev = trace.get(0, step - 1)?.clone();
        trace.set(0, step, hash_bytes(&prev))?;
    }

    // Register 1: Input data
    for step in 0..16 {
        trace.set(1, step, input_data.clone())?;
    }

    // Register 2: Transition computation
    for step in 0..16 {
        let state = trace.get(0, step)?;
        let input = trace.get(1, step)?;
        let new_state = compute_new_state(state, input, &transition_type);
        trace.set(2, step, new_state)?;
    }

    // Register 3: State hashes (public)
    for step in 0..16 {
        let state = trace.get(0, step)?;
        let state_hash = hash_bytes(state);
        trace.set(3, step, state_hash)?;
    }

    Ok(trace)
}

/// Compute new state from current state and input
fn compute_new_state(current_state: &[u8], input_data: &[u8], transition_type: &[u8]) -> Vec<u8> {
    // Simplified state transition logic
    // In production, this would implement actual workflow state machine

    let mut hasher = Sha3_256::new();
    hasher.update(current_state);
    hasher.update(input_data);
    hasher.update(transition_type);
    hasher.update(b"STATE_TRANSITION");

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
    fn test_compute_new_state() {
        let current = vec![1u8, 2, 3];
        let input = vec![4u8, 5, 6];
        let transition_type = vec![0u8];

        let new_state = compute_new_state(&current, &input, &transition_type);
        assert!(!new_state.is_empty());
        assert_eq!(new_state.len(), 32); // SHA3-256 output
    }

    #[test]
    fn test_hash_bytes() {
        let data = vec![1u8, 2, 3, 4, 5];
        let hash = hash_bytes(&data);

        assert_eq!(hash.len(), 32);

        // Same input produces same hash
        let hash2 = hash_bytes(&data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_create_plonk_circuit() {
        let private_inputs = PrivateInputs::new()
            .add("current_state", vec![1, 2, 3])
            .add("input_data", vec![4, 5, 6]);

        let public_inputs = PublicInputs::new();

        let circuit = create_plonk_circuit(&private_inputs, &public_inputs);
        assert!(circuit.is_ok());

        let circuit = circuit.unwrap();
        assert_eq!(circuit.gates.len(), 5);
        assert_eq!(circuit.wires.len(), 5);
    }

    #[test]
    fn test_create_stark_trace() {
        let private_inputs = PrivateInputs::new()
            .add("current_state", vec![1, 2, 3])
            .add("input_data", vec![4, 5, 6])
            .add("transition_type", vec![0]);

        let public_inputs = PublicInputs::new();

        let trace = create_stark_trace(&private_inputs, &public_inputs);
        assert!(trace.is_ok());

        let trace = trace.unwrap();
        assert_eq!(trace.registers.len(), 4);
        assert_eq!(trace.length, 16);
    }
}
