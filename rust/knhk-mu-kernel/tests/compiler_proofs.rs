//! Proof Validation Tests for Σ→Σ* Compiler
//!
//! These tests verify that the compiler generates valid proofs
//! and that invalid Σ* artifacts are correctly rejected.

use ed25519_dalek::SigningKey;
use knhk_mu_kernel::{
    compiler::{compile_ontology, compile_with_proof, CompilerError, SourceFormat},
    compiler_proof::{
        verify_for_loading, CertificateError, CertifiedSigma, CompilationCertificate,
        LoaderVerification,
    },
    sigma::{SigmaCompiled, SigmaHash},
    sigma_ir::{
        validation::{Certified, Unvalidated},
        Expr, GuardExpr, GuardId, GuardType, HandlerType, Metadata, PatternGraph, PatternId, Phase,
        Priority, Schema, SigmaIR, TaskId, TaskNode,
    },
    CHATMAN_CONSTANT,
};

/// Test signing key (deterministic for testing)
fn test_signing_key() -> SigningKey {
    let mut bytes = [0u8; 32];
    bytes[0] = 42;
    SigningKey::from_bytes(&bytes)
}

/// Different signing key (for negative tests)
fn wrong_signing_key() -> SigningKey {
    let mut bytes = [0u8; 32];
    bytes[0] = 99;
    SigningKey::from_bytes(&bytes)
}

#[test]
fn test_proof_generation_success() {
    let signing_key = test_signing_key();
    let source = ""; // Minimal valid source

    let result = compile_with_proof(source, SourceFormat::Turtle, &signing_key);
    assert!(result.is_ok());

    let certified = result.unwrap();

    // Certificate should exist
    assert_eq!(
        certified.certificate.sigma_hash,
        certified.sigma_star.compute_hash()
    );

    // Proofs should be valid
    assert!(certified.certificate.isa_proof.verify());
    assert!(certified.certificate.timing_proof.verify());
    assert!(certified.certificate.invariant_proof.verify());
}

#[test]
fn test_certificate_signature_verification() {
    let signing_key = test_signing_key();
    let verifying_key = signing_key.verifying_key();

    let source = "";
    let certified = compile_with_proof(source, SourceFormat::Turtle, &signing_key).unwrap();

    // Signature verification should pass
    assert!(certified
        .certificate
        .verify_signature(&verifying_key)
        .is_ok());

    // Wrong key should fail
    let wrong_key = wrong_signing_key();
    let wrong_verifying_key = wrong_key.verifying_key();
    assert!(certified
        .certificate
        .verify_signature(&wrong_verifying_key)
        .is_err());
}

#[test]
fn test_loader_verification_valid() {
    let signing_key = test_signing_key();
    let verifying_key = signing_key.verifying_key();

    let source = "";
    let certified = compile_with_proof(source, SourceFormat::Turtle, &signing_key).unwrap();

    let verification = verify_for_loading(&certified, &verifying_key);
    assert_eq!(verification, LoaderVerification::Valid);
}

#[test]
fn test_loader_verification_invalid_signature() {
    let signing_key = test_signing_key();
    let wrong_key = wrong_signing_key();
    let wrong_verifying_key = wrong_key.verifying_key();

    let source = "";
    let certified = compile_with_proof(source, SourceFormat::Turtle, &signing_key).unwrap();

    let verification = verify_for_loading(&certified, &wrong_verifying_key);
    assert!(matches!(
        verification,
        LoaderVerification::InvalidCertificate(CertificateError::InvalidSignature)
    ));
}

#[test]
fn test_chatman_constant_enforcement_at_compile_time() {
    use alloc::string::String;
    use alloc::vec::Vec;

    let metadata = Metadata {
        version: String::from("1.0.0"),
        author: String::from("test"),
        description: String::from("test"),
    };

    // Create pattern that exceeds Chatman Constant
    let pattern = PatternGraph {
        id: PatternId(0),
        name: String::from("too_slow"),
        phases: vec![
            Phase {
                number: 0,
                handler: HandlerType::Pure,
                tick_estimate: 5,
            },
            Phase {
                number: 1,
                handler: HandlerType::Pure,
                tick_estimate: 5,
            }, // 10 total > 8
        ],
        max_phases: 2,
        _phantom: core::marker::PhantomData,
    };

    let ir = SigmaIR::<Unvalidated>::new(Vec::new(), vec![pattern], Vec::new(), metadata);

    // Should pass initial validations
    let ir = ir.validate_structure().unwrap();
    let ir = ir.validate_semantics().unwrap();

    // But fail timing validation
    assert!(ir.validate_timing().is_err());
}

#[test]
fn test_timing_proof_validation() {
    use knhk_mu_kernel::compiler_proof::{TaskTimingProof, TimingBoundProof, TimingBreakdown};

    // Valid timing proof (within Chatman Constant)
    let valid_task = TaskTimingProof {
        task_id: 1,
        ticks: 5,
        breakdown: TimingBreakdown {
            load_ticks: 1,
            dispatch_ticks: 1,
            guard_ticks: 1,
            execute_ticks: 1,
            receipt_ticks: 1,
        },
    };

    let valid_proof = TimingBoundProof::new(vec![valid_task], Vec::new(), Vec::new());
    assert!(valid_proof.verify());
    assert!(valid_proof.max_ticks <= CHATMAN_CONSTANT);

    // Invalid timing proof (exceeds Chatman Constant)
    let invalid_task = TaskTimingProof {
        task_id: 2,
        ticks: 10, // Exceeds 8
        breakdown: TimingBreakdown {
            load_ticks: 2,
            dispatch_ticks: 2,
            guard_ticks: 2,
            execute_ticks: 2,
            receipt_ticks: 2,
        },
    };

    let invalid_proof = TimingBoundProof::new(vec![invalid_task], Vec::new(), Vec::new());
    assert!(!invalid_proof.verify());
}

#[test]
fn test_isa_compliance_proof() {
    use knhk_mu_kernel::sigma_types::IsaComplianceProof;

    // Valid opcodes (0-127)
    let mut opcodes = [0u8; 256];
    opcodes[0] = 1;
    opcodes[1] = 2;
    opcodes[2] = 3;

    let proof = IsaComplianceProof::new(opcodes, 3);
    assert!(proof.verify());

    // Invalid opcodes (128-255 are reserved)
    let mut bad_opcodes = [0u8; 256];
    bad_opcodes[0] = 200; // Invalid

    let bad_proof = IsaComplianceProof::new(bad_opcodes, 1);
    assert!(!bad_proof.verify());
}

#[test]
fn test_type_state_validation_pipeline() {
    use alloc::string::String;
    use alloc::vec::Vec;

    let metadata = Metadata {
        version: String::from("1.0.0"),
        author: String::from("test"),
        description: String::from("test"),
    };

    let pattern = PatternGraph {
        id: PatternId(0),
        name: String::from("valid_pattern"),
        phases: vec![
            Phase {
                number: 0,
                handler: HandlerType::Pure,
                tick_estimate: 3,
            },
            Phase {
                number: 1,
                handler: HandlerType::Receipt,
                tick_estimate: 2,
            },
        ],
        max_phases: 2,
        _phantom: core::marker::PhantomData,
    };

    let task = TaskNode {
        id: TaskId(1),
        label: String::from("test_task"),
        input_schema: Schema { fields: Vec::new() },
        output_schema: Schema { fields: Vec::new() },
        guard_ids: Vec::new(),
        pattern_id: PatternId(0),
        priority: Priority(128),
        _phantom: core::marker::PhantomData,
    };

    let ir = SigmaIR::<Unvalidated>::new(vec![task], vec![pattern], Vec::new(), metadata);

    // Type-state transitions should all succeed
    let ir = ir.validate_structure().unwrap();
    let ir = ir.validate_semantics().unwrap();
    let ir = ir.validate_timing().unwrap();
    let _certified = ir.certify();

    // Type system ensures certified IR can only come from validated IR
}

#[test]
fn test_certified_sigma_full_verification() {
    let signing_key = test_signing_key();
    let verifying_key = signing_key.verifying_key();

    let source = "";
    let certified = compile_with_proof(source, SourceFormat::Turtle, &signing_key).unwrap();

    // Full verification (signature + proofs)
    assert!(certified.verify(&verifying_key).is_ok());
}

#[test]
fn test_hash_mismatch_detection() {
    use knhk_mu_kernel::compiler_proof::TimingBoundProof;
    use knhk_mu_kernel::compiler_proof::{InvariantId, ProofBuilder};
    use knhk_mu_kernel::sigma_types::{InvariantProof, IsaComplianceProof};

    let signing_key = test_signing_key();

    let sigma1 = SigmaCompiled::new();
    let sigma2 = SigmaCompiled::new();

    let hash1 = sigma1.compute_hash();
    let hash2 = sigma2.compute_hash();

    // Hashes should match for identical Σ*
    assert_eq!(hash1, hash2);

    // Create certificate with wrong hash
    let isa_proof = IsaComplianceProof::new([0; 256], 0);
    let timing_proof = TimingBoundProof::new(Vec::new(), Vec::new(), Vec::new());
    let invariant_proof = InvariantProof::new([InvariantId(0); 64], 0);

    let wrong_hash = SigmaHash([0xFF; 32]); // Wrong hash

    let cert = CompilationCertificate::new(
        wrong_hash,
        isa_proof,
        timing_proof,
        invariant_proof,
        &signing_key,
    );

    let certified = CertifiedSigma::new(sigma1, cert);

    // Verification should fail due to hash mismatch
    let verifying_key = signing_key.verifying_key();
    assert!(certified.verify(&verifying_key).is_err());
}

#[test]
fn test_proof_builder_accumulation() {
    use knhk_mu_kernel::compiler_proof::{ProofBuilder, TaskTimingProof, TimingBreakdown};

    let mut builder = ProofBuilder::new();

    // Record operations
    builder.record_opcode(1);
    builder.record_opcode(2);
    builder.record_opcode(3);

    builder.record_task_timing(TaskTimingProof {
        task_id: 1,
        ticks: 5,
        breakdown: TimingBreakdown {
            load_ticks: 1,
            dispatch_ticks: 1,
            guard_ticks: 1,
            execute_ticks: 1,
            receipt_ticks: 1,
        },
    });

    builder.record_task_timing(TaskTimingProof {
        task_id: 2,
        ticks: 6,
        breakdown: TimingBreakdown {
            load_ticks: 1,
            dispatch_ticks: 1,
            guard_ticks: 2,
            execute_ticks: 1,
            receipt_ticks: 1,
        },
    });

    // Build certificate
    let signing_key = test_signing_key();
    let sigma_hash = SigmaHash([0; 32]);

    let cert = builder.build(sigma_hash, &signing_key);
    assert!(cert.is_ok());

    let cert = cert.unwrap();
    assert_eq!(cert.timing_proof.task_timings.len(), 2);
}

#[test]
fn test_multiple_source_formats() {
    let signing_key = test_signing_key();

    // All formats should compile (even if they produce same result currently)
    let turtle = compile_with_proof("", SourceFormat::Turtle, &signing_key);
    assert!(turtle.is_ok());

    let yawl = compile_with_proof("", SourceFormat::Yawl, &signing_key);
    assert!(yawl.is_ok());

    let ir = compile_with_proof("", SourceFormat::IR, &signing_key);
    assert!(ir.is_ok());
}

#[test]
fn test_backward_compatibility() {
    // Old compile_ontology function should still work
    let result = compile_ontology("");
    assert!(result.is_ok());

    let sigma = result.unwrap();
    assert!(sigma.header.is_valid());
}

#[test]
fn test_certification_failure_on_invalid_timing() {
    use knhk_mu_kernel::compiler_proof::{ProofBuilder, TaskTimingProof, TimingBreakdown};

    let mut builder = ProofBuilder::new();

    // Add task that exceeds Chatman Constant
    builder.record_task_timing(TaskTimingProof {
        task_id: 1,
        ticks: 100, // Way over limit
        breakdown: TimingBreakdown {
            load_ticks: 20,
            dispatch_ticks: 20,
            guard_ticks: 20,
            execute_ticks: 20,
            receipt_ticks: 20,
        },
    });

    let signing_key = test_signing_key();
    let sigma_hash = SigmaHash([0; 32]);

    // Should fail to build certificate
    let cert = builder.build(sigma_hash, &signing_key);
    assert!(cert.is_err());
}

#[test]
fn test_phase_count_limit() {
    use alloc::string::String;
    use alloc::vec;

    let metadata = Metadata {
        version: String::from("1.0.0"),
        author: String::from("test"),
        description: String::from("test"),
    };

    // Create pattern with too many phases (>8)
    let pattern = PatternGraph {
        id: PatternId(0),
        name: String::from("too_many_phases"),
        phases: vec![
            Phase {
                number: 0,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 1,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 2,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 3,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 4,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 5,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 6,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 7,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            },
            Phase {
                number: 8,
                handler: HandlerType::Pure,
                tick_estimate: 1,
            }, // 9 phases!
        ],
        max_phases: 9,
        _phantom: core::marker::PhantomData,
    };

    let ir = SigmaIR::<Unvalidated>::new(Vec::new(), vec![pattern], Vec::new(), metadata);

    // Should fail structural validation
    assert!(ir.validate_structure().is_err());
}

#[test]
fn test_guard_expression_compilation() {
    use knhk_mu_kernel::compiler::compile_expr;
    use knhk_mu_kernel::sigma_ir::{CompareOp, Expr};

    // Const expression
    let expr = Expr::Const(42);
    assert!(compile_expr(&expr).is_ok());

    // ReadObs expression
    let expr = Expr::ReadObs(5);
    assert!(compile_expr(&expr).is_ok());

    // Complex comparison
    let expr = Expr::Compare(
        CompareOp::Gt,
        Box::new(Expr::ReadObs(0)),
        Box::new(Expr::Const(100)),
    );
    assert!(compile_expr(&expr).is_ok());

    // Logical AND
    let expr = Expr::And(
        Box::new(Expr::Compare(
            CompareOp::Eq,
            Box::new(Expr::ReadObs(0)),
            Box::new(Expr::Const(1)),
        )),
        Box::new(Expr::Compare(
            CompareOp::Lt,
            Box::new(Expr::ReadObs(1)),
            Box::new(Expr::Const(10)),
        )),
    );
    let bytecode = compile_expr(&expr).unwrap();
    assert!(!bytecode.is_empty());
}

#[test]
fn test_proof_system_properties() {
    let signing_key = test_signing_key();
    let verifying_key = signing_key.verifying_key();

    let source = "";
    let certified = compile_with_proof(source, SourceFormat::Turtle, &signing_key).unwrap();

    // Property 1: Certificate signature is valid
    assert!(certified
        .certificate
        .verify_signature(&verifying_key)
        .is_ok());

    // Property 2: All proofs verify
    assert!(certified.certificate.verify_proofs().is_ok());

    // Property 3: Hash matches
    let computed_hash = certified.sigma_star.compute_hash();
    assert_eq!(computed_hash, certified.certificate.sigma_hash);

    // Property 4: Timing bounds respected
    assert!(certified.certificate.timing_proof.max_ticks <= CHATMAN_CONSTANT);

    // Property 5: ISA compliance
    assert!(certified.certificate.isa_proof.verify());
}
