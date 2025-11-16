//! Formal Verification Example
//!
//! Demonstrates how to use KNHK's formal verification system to prove
//! that governance adaptations are mathematically sound.
//!
//! Run with:
//! ```bash
//! cargo run --example formal_verification
//! ```

use knhk_workflow_engine::{
    // Autonomic computing
    autonomic::{
        delta_sigma::{DeltaSigma, OverlayChange, OverlayScope},
        doctrine::Doctrine,
        policy_lattice::{LatencyBound, PolicyElement, Strictness},
    },
    // Patterns
    patterns::PatternId,
    // Verification
    verification::{
        InvariantChecker, InvariantContext, PolicyConsistencyInvariant, PolicyVerifier,
        ProofCache, ProofSubject,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔬 KNHK Formal Verification Example");
    println!("====================================\n");

    // Example 1: Policy Verification
    example_policy_verification().await?;

    // Example 2: Overlay Verification
    example_overlay_verification().await?;

    // Example 3: Doctrine Projection
    example_doctrine_projection().await?;

    // Example 4: Invariant Checking
    example_invariant_checking().await?;

    // Example 5: Proof Caching
    example_proof_caching().await?;

    println!("\n✅ All verification examples completed successfully!");

    Ok(())
}

async fn example_policy_verification() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 1: Policy Verification");
    println!("----------------------------------");

    // Create policy verifier
    let verifier = PolicyVerifier::new()?;

    // Create policy: Latency ≤ 100ms
    let latency = LatencyBound::new(100.0, Strictness::Hard)?;
    let policy = PolicyElement::Latency(latency);

    println!("Policy: {}", policy);

    // Verify policy
    let proof = verifier.verify_policy(&policy)?;

    if proof.is_valid() {
        println!("✅ Policy is valid (SAT)");
        println!("   Verification time: {}ms", proof.result.duration_ms);
    } else {
        println!("❌ Policy is invalid (UNSAT)");
        if let Some(counterexample) = proof.counterexample() {
            println!("   Counterexample: {}", counterexample);
        }
    }

    println!();
    Ok(())
}

async fn example_overlay_verification() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Example 2: Overlay Verification");
    println!("-----------------------------------");

    // Create overlay proposal
    let scope = OverlayScope::new().with_pattern(PatternId::new(12)?); // Multi-instance pattern

    let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];

    println!("Overlay scope: {} patterns", scope.patterns.len());
    println!("Changes: {:?}", changes[0].description());

    let unproven = DeltaSigma::new(scope, changes);

    // Generate proof obligations
    let proof_pending = unproven.generate_proof_obligations()?;
    let obligations = proof_pending.proof_obligations();

    println!("Proof obligations generated: {}", obligations.len());
    for (i, obligation) in obligations.iter().enumerate() {
        println!("  {}. {}", i + 1, obligation.description());
    }

    // Verify overlay
    let verifier = PolicyVerifier::new()?;
    let proof = verifier.verify_overlay(&proof_pending).await?;

    if proof.is_valid() {
        println!("✅ Overlay is safe to apply");
    } else {
        println!("❌ Overlay verification failed");
    }

    println!();
    Ok(())
}

async fn example_doctrine_projection() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚖️  Example 3: Doctrine Projection (Q ∧ policy → policy')");
    println!("--------------------------------------------------------");

    let doctrine = Doctrine::new();
    println!("Doctrine μ-kernel constraints:");
    println!("  - max_exec_ticks: {} (Chatman Constant)", doctrine.max_exec_ticks);
    println!("  - max_run_len: {}", doctrine.max_run_len);
    println!("  - max_hot_path_latency_ms: {}ms", doctrine.max_hot_path_latency_ms);

    // Policy within bounds
    let valid_policy = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Soft)?);

    match doctrine.project(&valid_policy)? {
        Some(policy_prime) => {
            println!("\n✅ Policy satisfies doctrine");
            println!("   Original: {}", valid_policy);
            println!("   Projected: {}", policy_prime);
        }
        None => {
            println!("\n❌ Policy violates doctrine (projected to ⊥)");
        }
    }

    // Policy exceeds bounds
    let excessive_policy = PolicyElement::Latency(LatencyBound::new(200.0, Strictness::Hard)?);

    match doctrine.project(&excessive_policy)? {
        Some(policy_prime) => {
            println!("\n⚠️  Excessive policy clamped to doctrine bounds");
            println!("   Original: {}", excessive_policy);
            println!("   Projected: {}", policy_prime);
        }
        None => {
            println!("\n❌ Excessive policy violates doctrine");
        }
    }

    println!();
    Ok(())
}

async fn example_invariant_checking() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Example 4: Runtime Invariant Checking");
    println!("----------------------------------------");

    // Create invariant checker
    let mut checker = InvariantChecker::new();

    // Register invariants
    checker.register(PolicyConsistencyInvariant);

    println!("Registered invariants: {}", checker.invariant_count());

    // Create context with policy
    let policy = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Hard)?);
    let context = InvariantContext::new().with_policy(policy);

    // Check invariants
    match checker.check_all(&context) {
        Ok(()) => {
            println!("✅ All invariants satisfied");
            println!("   Violations: {}", checker.violation_count());
        }
        Err(e) => {
            println!("❌ Invariant violations detected");
            println!("   Error: {}", e);
            println!("   Recent violations:");
            for violation in checker.get_violations(5) {
                println!("     - {}: {}", violation.invariant, violation.message);
            }
        }
    }

    println!();
    Ok(())
}

async fn example_proof_caching() -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Example 5: Proof Caching");
    println!("---------------------------");

    // Create proof cache (100 proofs, 60 second TTL)
    let cache = ProofCache::new(100, 60_000);

    println!("Cache configuration:");
    println!("  - Max proofs: 100");
    println!("  - TTL: 60 seconds");

    // Create and cache proof
    let overlay_id = knhk_workflow_engine::autonomic::delta_sigma::OverlayId::new();
    let subject = ProofSubject::Overlay(overlay_id);

    let proof = knhk_workflow_engine::verification::ProofCertificate::new(
        subject.clone(),
        knhk_workflow_engine::verification::ProofStatus::Valid,
    );

    cache.put(proof.clone()).await?;

    println!("\n✅ Proof cached");
    println!("   Proof ID: {}", proof.proof_id);

    // Retrieve from cache
    if let Some(cached_proof) = cache.get(&subject).await {
        println!("\n✅ Proof retrieved from cache");
        println!("   Cache hit! Verification time: < 1ms");
        println!("   Proof valid: {}", cached_proof.is_valid());
    } else {
        println!("\n❌ Cache miss");
    }

    // Get cache statistics
    let stats = cache.stats().await;
    println!("\nCache statistics:");
    println!("  - Total proofs: {}", stats.total_proofs);
    println!("  - TTL: {}ms", stats.default_ttl_ms);

    println!();
    Ok(())
}
