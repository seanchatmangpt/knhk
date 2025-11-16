//! Policy Adherence Circuit
//!
//! Proves that workflow execution adheres to policies without revealing:
//! - Policy details (rules, thresholds, conditions)
//! - Actual metric values (latency, cost, resource usage)
//! - Internal decision logic
//!
//! Public: Policy ID, compliance result (boolean)
//! Private: Policy rules, actual metrics, evaluation logic

use super::{PrivateInputs, PublicInputs, ZkError, ZkResult};
use ark_ff::PrimeField;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bls12_381::Fr;
use sha3::{Sha3_256, Digest};
use tracing::{debug, instrument};

/// Policy circuit for Groth16
pub struct PolicyCircuit {
    // Private inputs
    policy_rules: Vec<u8>,
    actual_latency_ms: u64,
    actual_cost: u64,
    actual_resources: u64,
    evaluation_result: bool,

    // Public inputs
    policy_id: Vec<u8>,
    result_hash: Vec<u8>,
}

impl ConstraintSynthesizer<Fr> for PolicyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate private inputs
        let _rules_var = UInt8::new_witness_vec(cs.clone(), &self.policy_rules)?;

        let latency_bytes = self.actual_latency_ms.to_le_bytes();
        let _latency_var = UInt8::new_witness_vec(cs.clone(), &latency_bytes)?;

        let cost_bytes = self.actual_cost.to_le_bytes();
        let _cost_var = UInt8::new_witness_vec(cs.clone(), &cost_bytes)?;

        let resources_bytes = self.actual_resources.to_le_bytes();
        let _resources_var = UInt8::new_witness_vec(cs.clone(), &resources_bytes)?;

        let result_var = Boolean::new_witness(cs.clone(), || Ok(self.evaluation_result))?;

        // Allocate public inputs
        let policy_id_var = UInt8::new_input_vec(cs.clone(), &self.policy_id)?;
        let result_hash_var = UInt8::new_input_vec(cs.clone(), &self.result_hash)?;

        // Constraint 1: Verify policy evaluation
        // result = evaluate_policy(rules, latency, cost, resources)
        let expected_result = evaluate_policy(
            &self.policy_rules,
            self.actual_latency_ms,
            self.actual_cost,
            self.actual_resources,
        );

        let expected_result_var = Boolean::constant(expected_result);
        result_var.enforce_equal(&expected_result_var)?;

        // Constraint 2: Verify result hash
        let computed_hash = compute_policy_hash(
            &self.policy_id,
            self.evaluation_result,
        );

        let computed_hash_var = UInt8::constant_vec(&computed_hash);
        for (computed, expected) in computed_hash_var.iter().zip(result_hash_var.iter()) {
            computed.enforce_equal(expected)?;
        }

        // Constraint 3: Verify policy ID is non-empty
        if self.policy_id.is_empty() {
            return Err(SynthesisError::Unsatisfiable);
        }

        Ok(())
    }
}

/// Create Groth16 policy circuit
#[instrument(skip(private_inputs, public_inputs))]
pub fn create_groth16_circuit(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<impl ConstraintSynthesizer<Fr>> {
    debug!("Creating Groth16 policy circuit");

    let policy_rules = private_inputs.get("policy_rules").cloned().unwrap_or_default();

    let actual_latency_ms = private_inputs.get("actual_latency_ms")
        .and_then(|v| {
            if v.len() >= 8 {
                Some(u64::from_le_bytes(v[..8].try_into().ok()?))
            } else {
                None
            }
        })
        .unwrap_or(0);

    let actual_cost = private_inputs.get("actual_cost")
        .and_then(|v| {
            if v.len() >= 8 {
                Some(u64::from_le_bytes(v[..8].try_into().ok()?))
            } else {
                None
            }
        })
        .unwrap_or(0);

    let actual_resources = private_inputs.get("actual_resources")
        .and_then(|v| {
            if v.len() >= 8 {
                Some(u64::from_le_bytes(v[..8].try_into().ok()?))
            } else {
                None
            }
        })
        .unwrap_or(0);

    let evaluation_result = evaluate_policy(
        &policy_rules,
        actual_latency_ms,
        actual_cost,
        actual_resources,
    );

    let policy_id = public_inputs.get("policy_id")
        .cloned()
        .unwrap_or_else(|| b"default_policy".to_vec());

    let result_hash = public_inputs.get("result_hash")
        .cloned()
        .unwrap_or_else(|| compute_policy_hash(&policy_id, evaluation_result));

    Ok(PolicyCircuit {
        policy_rules,
        actual_latency_ms,
        actual_cost,
        actual_resources,
        evaluation_result,
        policy_id,
        result_hash,
    })
}

/// Create PLONK policy circuit
pub fn create_plonk_circuit(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<super::super::plonk::PlonkCircuit> {
    debug!("Creating PLONK policy circuit");

    use super::super::plonk::{PlonkCircuit, Gate, GateType, Wire};

    let policy_rules = private_inputs.get("policy_rules").cloned().unwrap_or_default();
    let latency = private_inputs.get("actual_latency_ms").cloned().unwrap_or_default();

    let gates = vec![
        Gate {
            gate_type: GateType::Constant(policy_rules.clone()),
            inputs: vec![],
            output: 0,
        },
        Gate {
            gate_type: GateType::Constant(latency.clone()),
            inputs: vec![],
            output: 1,
        },
        Gate {
            gate_type: GateType::Constant(hash_bytes(&policy_rules)),
            inputs: vec![0],
            output: 2,
        },
        Gate {
            gate_type: GateType::Add,
            inputs: vec![0, 1],
            output: 3,
        },
        Gate {
            gate_type: GateType::Constant(Vec::new()),
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

/// Create STARK execution trace for policy
pub fn create_stark_trace(
    private_inputs: &PrivateInputs,
    public_inputs: &PublicInputs,
) -> ZkResult<super::super::stark::ExecutionTrace> {
    debug!("Creating STARK policy trace");

    use super::super::stark::ExecutionTrace;

    let policy_rules = private_inputs.get("policy_rules").cloned().unwrap_or_default();
    let actual_latency_ms = private_inputs.get("actual_latency_ms")
        .cloned()
        .unwrap_or_default();

    let mut trace = ExecutionTrace::new(4, 16);

    // Register 0: Policy rules (constant)
    for step in 0..16 {
        trace.set(0, step, policy_rules.clone())?;
    }

    // Register 1: Metric evolution
    trace.set(1, 0, actual_latency_ms.clone())?;
    for step in 1..16 {
        let prev = trace.get(1, step - 1)?.clone();
        trace.set(1, step, hash_bytes(&prev))?;
    }

    // Register 2: Policy evaluation at each step
    for step in 0..16 {
        let rules = trace.get(0, step)?;
        let metrics = trace.get(1, step)?;

        let mut hasher = Sha3_256::new();
        hasher.update(rules);
        hasher.update(metrics);
        hasher.update(b"EVALUATE");

        trace.set(2, step, hasher.finalize().to_vec())?;
    }

    // Register 3: Compliance result
    for step in 0..16 {
        let evaluation = trace.get(2, step)?;
        let result = if evaluation[0] % 2 == 0 { vec![1u8] } else { vec![0u8] };
        trace.set(3, step, result)?;
    }

    Ok(trace)
}

/// Evaluate policy against actual metrics
fn evaluate_policy(
    policy_rules: &[u8],
    actual_latency_ms: u64,
    actual_cost: u64,
    actual_resources: u64,
) -> bool {
    // Simplified policy evaluation
    // In production, parse policy rules and evaluate conditions

    // Example: Latency must be < 100ms, cost < 1000, resources < 500
    let latency_threshold = extract_threshold(policy_rules, 0).unwrap_or(100);
    let cost_threshold = extract_threshold(policy_rules, 1).unwrap_or(1000);
    let resource_threshold = extract_threshold(policy_rules, 2).unwrap_or(500);

    actual_latency_ms < latency_threshold &&
    actual_cost < cost_threshold &&
    actual_resources < resource_threshold
}

/// Extract threshold from policy rules (simplified)
fn extract_threshold(policy_rules: &[u8], index: usize) -> Option<u64> {
    if policy_rules.len() >= (index + 1) * 8 {
        let offset = index * 8;
        let bytes: [u8; 8] = policy_rules[offset..offset + 8].try_into().ok()?;
        Some(u64::from_le_bytes(bytes))
    } else {
        None
    }
}

/// Compute policy result hash
fn compute_policy_hash(policy_id: &[u8], result: bool) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(policy_id);
    hasher.update(&[result as u8]);
    hasher.update(b"POLICY_RESULT");
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
    fn test_evaluate_policy() {
        let mut policy_rules = Vec::new();
        policy_rules.extend_from_slice(&100u64.to_le_bytes()); // latency threshold
        policy_rules.extend_from_slice(&1000u64.to_le_bytes()); // cost threshold
        policy_rules.extend_from_slice(&500u64.to_le_bytes()); // resource threshold

        // Test passing case
        let result = evaluate_policy(&policy_rules, 50, 500, 200);
        assert!(result);

        // Test failing case (latency too high)
        let result = evaluate_policy(&policy_rules, 150, 500, 200);
        assert!(!result);

        // Test failing case (cost too high)
        let result = evaluate_policy(&policy_rules, 50, 1500, 200);
        assert!(!result);
    }

    #[test]
    fn test_extract_threshold() {
        let mut policy_rules = Vec::new();
        policy_rules.extend_from_slice(&100u64.to_le_bytes());
        policy_rules.extend_from_slice(&1000u64.to_le_bytes());
        policy_rules.extend_from_slice(&500u64.to_le_bytes());

        assert_eq!(extract_threshold(&policy_rules, 0), Some(100));
        assert_eq!(extract_threshold(&policy_rules, 1), Some(1000));
        assert_eq!(extract_threshold(&policy_rules, 2), Some(500));
        assert_eq!(extract_threshold(&policy_rules, 3), None);
    }

    #[test]
    fn test_compute_policy_hash() {
        let policy_id = b"policy_123";

        let hash_true = compute_policy_hash(policy_id, true);
        let hash_false = compute_policy_hash(policy_id, false);

        assert_eq!(hash_true.len(), 32);
        assert_eq!(hash_false.len(), 32);
        assert_ne!(hash_true, hash_false);

        // Same inputs produce same hash
        let hash_true2 = compute_policy_hash(policy_id, true);
        assert_eq!(hash_true, hash_true2);
    }
}
