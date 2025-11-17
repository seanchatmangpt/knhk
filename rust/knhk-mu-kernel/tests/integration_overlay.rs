//! Integration Tests for ΔΣ Overlay Algebra
//!
//! Tests proof-carrying overlays and Σ* evolution

use knhk_mu_kernel::guards::GuardResult;
use knhk_mu_kernel::overlay::{
    DeltaSigma, InvariantCheck, OverlayAlgebra, PerformanceEstimate, ProofSketch, SigmaChange,
};
use knhk_mu_kernel::sigma::{SigmaCompiled, SigmaHash, SigmaPointer, TaskDescriptor};

#[test]
fn test_overlay_creation() {
    let delta = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.2,
                confidence: 0.8,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    assert_eq!(delta.id, 1);
    assert_eq!(delta.priority, 10);
}

#[test]
fn test_overlay_validation() {
    let delta = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![InvariantCheck {
                name: "tick_budget".to_string(),
                result: GuardResult::Pass,
                evidence: "all tasks ≤8 ticks".to_string(),
            }],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.3,
                confidence: 0.9,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    let result = delta.validate();
    assert!(result.is_ok(), "Valid overlay should pass validation");
}

#[test]
fn test_overlay_composition() {
    let delta1 = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![SigmaChange::AddTask(TaskDescriptor::default())],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.2,
                confidence: 0.8,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    let delta2 = DeltaSigma {
        id: 2,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![SigmaChange::RemoveTask(100)],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 5,
                expected_improvement: 0.1,
                confidence: 0.7,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 5,
    };

    // Compose overlays
    let result = delta1.compose(&delta2);
    assert!(result.is_ok(), "Compatible overlays should compose");

    let composed = result.unwrap();
    assert_eq!(composed.changes.len(), 2);
    assert_eq!(composed.priority, 10); // max priority
}

#[test]
fn test_overlay_incompatible_base() {
    let delta1 = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.2,
                confidence: 0.8,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    let delta2 = DeltaSigma {
        id: 2,
        base_sigma: SigmaHash([2; 32]), // Different base
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 5,
                expected_improvement: 0.1,
                confidence: 0.7,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 5,
    };

    let result = delta1.compose(&delta2);
    assert!(result.is_err(), "Incompatible bases should not compose");
}

#[test]
fn test_overlay_conflict_detection() {
    let delta1 = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![SigmaChange::ModifyTask {
            task_id: 100,
            new_descriptor: TaskDescriptor::default(),
        }],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.2,
                confidence: 0.8,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    let delta2 = DeltaSigma {
        id: 2,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![SigmaChange::ModifyTask {
            task_id: 100, // Same task
            new_descriptor: TaskDescriptor::default(),
        }],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 5,
                expected_improvement: 0.1,
                confidence: 0.7,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 5,
    };

    assert!(
        delta1.conflicts_with(&delta2),
        "Should detect conflicting changes"
    );
}

#[test]
fn test_sigma_atomic_update() {
    let sigma_ptr = SigmaPointer::new();

    let sigma1 = SigmaCompiled::default();
    let mut sigma2 = sigma1;
    sigma2.header.hash = SigmaHash([2; 32]);

    // Store initial Σ*
    sigma_ptr.store(&sigma1);

    // Atomic update
    sigma_ptr.store(&sigma2);

    // Verify update
    let loaded = sigma_ptr.load().unwrap();
    assert_eq!(loaded.header.hash, sigma2.header.hash);
}

#[test]
fn test_overlay_shadow_deployment() {
    // Shadow deployment workflow:
    // 1. Apply ΔΣ to shadow Σ*
    // 2. Test in shadow environment
    // 3. If tests pass, atomic swap to make active

    let sigma_ptr = SigmaPointer::new();
    let sigma_original = SigmaCompiled::default();
    sigma_ptr.store(&sigma_original);

    let delta = DeltaSigma {
        id: 1,
        base_sigma: sigma_original.header.hash,
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.2,
                confidence: 0.8,
            },
            mape_evidence: vec![],
            signature: [0; 64],
        },
        priority: 10,
    };

    // Apply to shadow
    let sigma_new = delta.apply_to(&sigma_original);
    assert!(sigma_new.is_ok(), "ΔΣ application should succeed");

    // Atomic promotion
    let sigma_promoted = sigma_new.unwrap();
    sigma_ptr.store(&sigma_promoted);

    // Verify new Σ* is active
    let active = sigma_ptr.load().unwrap();
    assert_eq!(active.header.hash, sigma_promoted.header.hash);
}

#[test]
fn test_proof_sketch_evidence() {
    use knhk_mu_kernel::overlay::MapeEvidence;

    let delta = DeltaSigma {
        id: 1,
        base_sigma: SigmaHash([1; 32]),
        changes: vec![],
        proof: ProofSketch {
            invariants_checked: vec![InvariantCheck {
                name: "chatman_constant".to_string(),
                result: GuardResult::Pass,
                evidence: "verified via static analysis".to_string(),
            }],
            perf_estimate: PerformanceEstimate {
                max_ticks: 6,
                expected_improvement: 0.25,
                confidence: 0.9,
            },
            mape_evidence: vec![
                MapeEvidence {
                    evidence_type: knhk_mu_kernel::overlay::MapePhase::Monitor,
                    metric: "avg_tau".to_string(),
                    value: 7.5,
                },
                MapeEvidence {
                    evidence_type: knhk_mu_kernel::overlay::MapePhase::Analyze,
                    metric: "violation_rate".to_string(),
                    value: 0.15,
                },
            ],
            signature: [0; 64],
        },
        priority: 10,
    };

    assert_eq!(delta.proof.mape_evidence.len(), 2);
    assert_eq!(delta.proof.invariants_checked.len(), 1);
}
