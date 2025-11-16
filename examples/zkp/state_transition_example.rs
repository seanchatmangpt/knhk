//! Example: Zero-Knowledge State Transition Proof
//!
//! This example demonstrates how to prove that a workflow state transitioned correctly
//! without revealing the actual state, input data, or transition logic.

#[cfg(feature = "zkp")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use knhk_workflow_engine::zkp::*;

    println!("=== Zero-Knowledge State Transition Proof Example ===\n");

    // Scenario: E-commerce order state transition
    // Current state: "pending_payment" -> New state: "payment_confirmed"
    // Input: payment_transaction_id
    // Requirement: Prove state transitioned correctly without revealing order details

    println!("1. Setup: Creating workflow state transition");
    let current_state = b"pending_payment_order_12345";
    let payment_input = b"transaction_id_abc123";
    let transition_type = vec![0u8]; // 0 = payment transition

    // Private inputs (hidden from verifier)
    let private_inputs = PrivateInputs::new()
        .add("current_state", current_state.to_vec())
        .add("input_data", payment_input.to_vec())
        .add("transition_type", transition_type);

    println!("   - Current state: {} bytes (hidden)", current_state.len());
    println!("   - Payment input: {} bytes (hidden)", payment_input.len());

    // Public inputs (visible to verifier)
    use sha3::{Sha3_256, Digest};
    let mut hasher = Sha3_256::new();
    hasher.update(current_state);
    let current_state_hash = hasher.finalize().to_vec();

    let public_inputs = PublicInputs::new()
        .add("workflow_id", b"ecommerce_order".to_vec())
        .add("current_state_hash", current_state_hash.clone());

    println!("   - Current state hash: {} (public)", hex::encode(&current_state_hash));
    println!();

    // Choose proof system
    println!("2. Proof System Selection:");
    println!("   [1] Groth16: Fast verification (2ms), requires trusted setup");
    println!("   [2] PLONK: Universal setup, flexible (5ms verification)");
    println!("   [3] STARK: No trusted setup, quantum-resistant (50ms verification)");
    println!();

    println!("   Selected: Groth16 (for fastest verification)");
    let proof_system = ProofSystem::Groth16;

    println!();
    println!("3. Proof Generation (simulated):");
    println!("   Note: Actual proof generation requires circuit implementation");
    println!("   This example demonstrates the API structure.");
    println!();

    // Create prover
    let config = ProverConfig {
        security_level: 128,
        enable_telemetry: true,
        parallel_proving: true,
    };

    match ZkProver::new(proof_system)
        .with_circuit("state_transition")
        .with_config(config)
        .build()
    {
        Ok(prover) => {
            println!("   ✓ Prover initialized successfully");
            println!("   ✓ Circuit: state_transition");
            println!("   ✓ Security level: 128-bit");
            println!();

            println!("   Expected proof properties:");
            println!("   - Proof size: ~200 bytes (Groth16)");
            println!("   - Generation time: ~500ms");
            println!("   - Verification time: ~2ms");
            println!();

            // In production, this would generate an actual proof:
            // let proof = prover.prove(&private_inputs, &public_inputs).await?;
        }
        Err(e) => {
            println!("   ⚠ Prover initialization: {}", e);
            println!("   (Expected - circuits not yet implemented)");
            println!();
        }
    }

    println!("4. Privacy Guarantees:");
    println!("   ✓ Verifier cannot see:");
    println!("     - Actual current state (order details)");
    println!("     - Payment transaction ID");
    println!("     - Transition function logic");
    println!();
    println!("   ✓ Verifier can verify:");
    println!("     - State hash matches declared hash");
    println!("     - State transition was valid");
    println!("     - New state hash is correct");
    println!();

    println!("5. Use Cases:");
    println!("   - Privacy-preserving audit trails");
    println!("   - Confidential workflow execution");
    println!("   - Regulatory compliance without data exposure");
    println!("   - Multi-party workflows with privacy");
    println!();

    Ok(())
}

#[cfg(not(feature = "zkp"))]
fn main() {
    println!("This example requires the 'zkp' feature.");
    println!("Run with: cargo run --example state_transition_example --features zkp");
}
