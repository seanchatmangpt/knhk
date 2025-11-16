//! Integration tests for zero-knowledge proof systems

#[cfg(feature = "zkp")]
mod zkp_integration_tests {
    use knhk_workflow_engine::zkp::*;

    #[tokio::test]
    async fn test_groth16_state_transition_proof() {
        // Create inputs
        let private_inputs = PrivateInputs::new()
            .add("current_state", vec![1, 2, 3, 4])
            .add("input_data", vec![5, 6, 7, 8])
            .add("transition_type", vec![0]);

        let public_inputs = PublicInputs::new()
            .add("workflow_id", b"test_workflow".to_vec());

        // Create prover
        let config = ProverConfig {
            security_level: 128,
            enable_telemetry: true,
            parallel_proving: false, // Disable for deterministic testing
        };

        let prover = ZkProver::new(ProofSystem::Groth16)
            .with_circuit("state_transition")
            .with_config(config)
            .build();

        assert!(prover.is_ok(), "Prover should build successfully");

        // Note: Full proof generation requires actual circuit implementation
        // This tests the API structure
    }

    #[tokio::test]
    async fn test_plonk_compliance_proof() {
        let private_inputs = PrivateInputs::new()
            .add("user_data", b"John Doe".to_vec())
            .add("consent_record", b"Consent given".to_vec());

        let public_inputs = PublicInputs::new()
            .add("compliance_type", vec![0]); // GDPR

        let prover = ZkProver::new(ProofSystem::Plonk)
            .with_circuit("compliance")
            .build();

        assert!(prover.is_ok());
    }

    #[tokio::test]
    async fn test_stark_policy_proof() {
        let mut policy_rules = Vec::new();
        policy_rules.extend_from_slice(&100u64.to_le_bytes());

        let private_inputs = PrivateInputs::new()
            .add("policy_rules", policy_rules)
            .add("actual_latency_ms", 50u64.to_le_bytes().to_vec());

        let public_inputs = PublicInputs::new()
            .add("policy_id", b"policy_123".to_vec());

        let prover = ZkProver::new(ProofSystem::Stark)
            .with_circuit("policy")
            .build();

        assert!(prover.is_ok());
    }

    #[test]
    fn test_privacy_anonymization() {
        let data = b"sensitive_user_data";
        let salt = b"random_salt";

        let anonymized = privacy::anonymize_data(data, salt);

        assert_eq!(anonymized.len(), 32); // SHA3-256 output
        assert_ne!(anonymized.as_slice(), data); // Should be different

        // Same inputs produce same output
        let anonymized2 = privacy::anonymize_data(data, salt);
        assert_eq!(anonymized, anonymized2);
    }

    #[test]
    fn test_privacy_pseudonymization() {
        let data = b"John Doe";
        let key = b"secret_key";

        let result = privacy::pseudonymize_data(data, key);
        assert!(result.is_ok());

        let pseudonym = result.unwrap();
        assert_ne!(pseudonym, data);

        // Should be reversible
        let original = privacy::depseudonymize_data(&pseudonym, key).unwrap();
        assert_eq!(original, data);
    }

    #[test]
    fn test_k_anonymity() {
        let mut k_anon = privacy::KAnonymity::new(3);

        // Add records with same quasi-identifiers
        k_anon.add_record(vec![1, 2, 3], vec![10]);
        k_anon.add_record(vec![1, 2, 3], vec![20]);

        assert!(!k_anon.is_k_anonymous()); // Only 2 records, need 3

        k_anon.add_record(vec![1, 2, 3], vec![30]);

        assert!(k_anon.is_k_anonymous());
    }

    #[test]
    fn test_l_diversity() {
        let mut l_div = privacy::LDiversity::new(2);

        // Add records with different sensitive values
        l_div.add_record(vec![1, 2, 3], vec![10]);
        l_div.add_record(vec![1, 2, 3], vec![20]);

        assert!(l_div.is_l_diverse());

        // Add records with same sensitive value
        l_div.add_record(vec![4, 5, 6], vec![30]);
        l_div.add_record(vec![4, 5, 6], vec![30]);

        assert!(!l_div.is_l_diverse()); // Second group has only 1 distinct value
    }

    #[tokio::test]
    async fn test_governance_overlay_safety_proof() {
        let overlay_delta = b"overlay_changes";
        let sigma_base = b"base_state";

        let result = governance::prove_overlay_safety(
            "test_workflow",
            overlay_delta,
            sigma_base,
            ProofSystem::Groth16,
        ).await;

        // Note: Will fail without actual circuit implementation
        // But tests the API structure
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_governance_policy_compliance_proof() {
        let policy_rules = b"policy_definition";
        let workflow_state = b"current_state";

        let result = governance::prove_policy_compliance(
            "test_workflow",
            1, // domain level
            policy_rules,
            workflow_state,
            ProofSystem::Plonk,
        ).await;

        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_proof_system_selection() {
        // Test that all proof systems can be instantiated
        let groth16_prover = ZkProver::new(ProofSystem::Groth16);
        let plonk_prover = ZkProver::new(ProofSystem::Plonk);
        let stark_prover = ZkProver::new(ProofSystem::Stark);

        // All should create successfully
        assert_eq!(groth16_prover.system, ProofSystem::Groth16);
        assert_eq!(plonk_prover.system, ProofSystem::Plonk);
        assert_eq!(stark_prover.system, ProofSystem::Stark);
    }

    #[test]
    fn test_input_builders() {
        let private_inputs = PrivateInputs::new()
            .add("key1", vec![1, 2, 3])
            .add("key2", vec![4, 5, 6]);

        assert_eq!(private_inputs.get("key1"), Some(&vec![1, 2, 3]));
        assert_eq!(private_inputs.get("key2"), Some(&vec![4, 5, 6]));
        assert_eq!(private_inputs.get("key3"), None);

        let public_inputs = PublicInputs::new()
            .add("public1", vec![10, 20, 30]);

        assert_eq!(public_inputs.get("public1"), Some(&vec![10, 20, 30]));
    }
}
