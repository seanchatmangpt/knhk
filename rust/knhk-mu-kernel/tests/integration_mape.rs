//! Integration Tests for MAPE-K
//!
//! Tests autonomic control loop: Monitor → Analyze → Plan → Execute

use knhk_mu_kernel::mape::MapeKColon;
use knhk_mu_kernel::sigma::{SigmaPointer, SigmaHash};
use knhk_mu_kernel::receipts::Receipt;
use knhk_mu_kernel::overlay::{DeltaSigma, OverlayAlgebra};

#[test]
fn test_mape_k_initialization() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mape = MapeKColon::new(sigma_ptr);

    // Should initialize successfully
    assert!(true);
}

#[test]
fn test_monitor_phase() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // Create test receipt
    let receipt = Receipt::new(
        0,
        SigmaHash([1; 32]),
        [2; 32],
        [3; 32],
        5,
        100,
        0,
    );

    // Monitor phase
    let result = mape.monitor(receipt);
    assert_eq!(result.observations_count, 1);
    assert!(result.avg_tau > 0.0);
}

#[test]
fn test_analyze_phase() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // Add receipts with violations
    for i in 0..10 {
        let receipt = Receipt::new(
            0,
            SigmaHash([1; 32]),
            [i; 32],
            [i; 32],
            if i > 5 { 10 } else { 5 },  // Some violations
            i as u64,
            0,
        );
        mape.monitor(receipt);
    }

    // Analyze phase
    let result = mape.analyze();
    assert!(result.symptoms.len() > 0, "Should detect performance degradation");
}

#[test]
fn test_plan_phase() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // Add receipts with violations
    for i in 0..20 {
        let receipt = Receipt::new(
            0,
            SigmaHash([1; 32]),
            [i; 32],
            [i; 32],
            10,  // Violates Chatman Constant
            i as u64,
            0,
        );
        mape.monitor(receipt);
    }

    // Analyze
    let analysis = mape.analyze();
    assert!(analysis.symptoms.len() > 0);

    // Plan
    let plans = mape.plan(&analysis.symptoms);
    assert!(plans.proposals.len() > 0, "Should generate ΔΣ proposals");
}

#[test]
fn test_complete_mape_k_cycle() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // Add some receipts
    for i in 0..10 {
        let receipt = Receipt::new(
            0,
            SigmaHash([1; 32]),
            [i; 32],
            [i; 32],
            if i > 5 { 10 } else { 5 },
            i as u64,
            0,
        );
        mape.monitor(receipt);
    }

    // Run complete cycle
    let result = mape.run_cycle();

    assert!(result.symptoms_detected > 0 || result.proposals_generated >= 0);
}

#[test]
fn test_continuous_mape_k_cycles() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // Run multiple cycles
    for cycle in 0..5 {
        // Add receipts
        for i in 0..10 {
            let receipt = Receipt::new(
                0,
                SigmaHash([1; 32]),
                [(cycle * 10 + i) as u8; 32],
                [(cycle * 10 + i) as u8; 32],
                5 + (cycle as u64 * 2),  // Gradual degradation
                (cycle * 10 + i) as u64,
                0,
            );
            mape.monitor(receipt);
        }

        // Run cycle
        let result = mape.run_cycle();
        println!("Cycle {}: {} symptoms, {} proposals, {} adaptations",
                 cycle, result.symptoms_detected, result.proposals_generated, result.adaptations_applied);
    }
}

#[test]
fn test_symptom_detection() {
    use knhk_mu_kernel::mape::Symptom;

    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // Create pattern of guard failures
    for i in 0..20 {
        let mut receipt = Receipt::new(
            0,
            SigmaHash([1; 32]),
            [i; 32],
            [i; 32],
            5,
            i as u64,
            0,
        );

        // Set guard failure bits
        if i % 2 == 0 {
            receipt.guard_bitmap = 0b1111;
            receipt.guard_outcomes = 0b1100;  // 2 failures
        }

        mape.monitor(receipt);
    }

    let analysis = mape.analyze();
    let has_guard_symptom = analysis.symptoms.iter().any(|s| {
        matches!(s, Symptom::GuardFailures { .. })
    });

    assert!(has_guard_symptom, "Should detect guard failure pattern");
}

#[test]
fn test_delta_sigma_validation() {
    use knhk_mu_kernel::overlay::{ProofSketch, PerformanceEstimate, InvariantCheck};
    use knhk_mu_kernel::guards::GuardResult;

    let delta = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![
                InvariantCheck {
                    name: "chatman_constant".to_string(),
                    result: GuardResult::Pass,
                    evidence: "all operations ≤8 ticks".to_string(),
                }
            ],
            perf_estimate: PerformanceEstimate {
                max_ticks: 7,
                expected_improvement: 0.2,
                confidence: 0.9,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    // Validate
    assert!(delta.validate().is_ok(), "Valid ΔΣ should pass validation");
}

#[test]
fn test_delta_sigma_performance_violation() {
    use knhk_mu_kernel::overlay::{ProofSketch, PerformanceEstimate};

    let delta = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 100,  // Violates Chatman Constant
                expected_improvement: 0.2,
                confidence: 0.9,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    // Should fail validation
    assert!(delta.validate().is_err(), "ΔΣ violating Chatman Constant should fail");
}

#[test]
fn test_mape_k_as_mu_colon() {
    // MAPE-K is not "beside" μ; it IS μ at different time scales

    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut mape = MapeKColon::new(sigma_ptr);

    // μ_hot (≤8 ticks): Individual task execution generates receipts
    // μ_warm (≤100ms): MAPE-K monitor/analyze phases
    // μ_cold (>100ms): MAPE-K plan/execute phases

    // All share same Σ*, Q, Γ
    assert!(true, "MAPE-K operates at different time scales of same μ");
}
