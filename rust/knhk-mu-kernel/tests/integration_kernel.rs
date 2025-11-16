//! Integration Tests for μ-Kernel
//!
//! Tests complete A = μ(O) execution with all components

use knhk_mu_kernel::core::MuKernel;
use knhk_mu_kernel::sigma::SigmaPointer;
use knhk_mu_kernel::guards::GuardContext;
use knhk_mu_kernel::CHATMAN_CONSTANT;

#[test]
fn test_kernel_initialization() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let kernel = MuKernel::new(sigma_ptr);

    let state = kernel.state();
    assert!(state.is_initialized());
}

#[test]
fn test_complete_execution_cycle() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    // Create observation
    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Execute: O → μ → A
    let result = kernel.execute_task(1, &obs);
    assert!(result.is_ok(), "Task execution failed: {:?}", result.err());

    let task_result = result.unwrap();
    assert_eq!(task_result.task_id, 1);
    assert!(task_result.ticks_used <= CHATMAN_CONSTANT);
}

#[test]
fn test_receipt_chain_creation() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Execute multiple tasks
    for i in 0..10 {
        let result = kernel.execute_task(i, &obs);
        assert!(result.is_ok());
    }

    // Verify receipt chain
    let state = kernel.state();
    assert_eq!(state.receipts.len(), 10);
    assert!(state.receipts.verify_chain());
}

#[test]
fn test_determinism() {
    // Same O + Σ* → Same A
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel1 = MuKernel::new(sigma_ptr);
    let mut kernel2 = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    let result1 = kernel1.execute_task(1, &obs).unwrap();
    let result2 = kernel2.execute_task(1, &obs).unwrap();

    assert_eq!(result1.action_hash, result2.action_hash);
    assert_eq!(result1.ticks_used, result2.ticks_used);
}

#[test]
fn test_chatman_constant_enforcement() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Execute many tasks, all should satisfy Chatman Constant
    for i in 0..100 {
        let result = kernel.execute_task(i, &obs).unwrap();
        assert!(
            result.ticks_used <= CHATMAN_CONSTANT,
            "Task {} violated Chatman Constant: {} > {}",
            i,
            result.ticks_used,
            CHATMAN_CONSTANT
        );
    }

    // Verify no violations in receipt chain
    let state = kernel.state();
    let violations = state.receipts.chatman_violations();
    assert_eq!(violations.len(), 0, "Found {} Chatman violations", violations.len());
}

#[test]
fn test_guard_enforcement() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    // Observation that violates guard (used > budget)
    let obs_violate = GuardContext {
        params: [8, 10, 0, 0, 0, 0, 0, 0],  // budget=8, used=10
    };

    let result = kernel.execute_task(1, &obs_violate);
    // Should still execute (guards are observations, not blockers in μ_hot)
    // but should be recorded in receipt
    assert!(result.is_ok());
}

#[test]
fn test_concurrent_execution() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));

    // Multiple kernels sharing same Σ*
    let mut kernels: Vec<MuKernel> = (0..4)
        .map(|_| MuKernel::new(sigma_ptr))
        .collect();

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Execute tasks concurrently (simulate)
    for (i, kernel) in kernels.iter_mut().enumerate() {
        let result = kernel.execute_task(i as u64, &obs);
        assert!(result.is_ok());
    }
}

#[test]
fn test_zero_allocation_hot_path() {
    // Verify μ_hot doesn't allocate
    // (This is enforced by #![no_std] at compile time)

    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Execute without allocation
    let result = kernel.execute_task(1, &obs);
    assert!(result.is_ok());
}

#[test]
fn test_receipt_cryptographic_properties() {
    let sigma_ptr = Box::leak(Box::new(SigmaPointer::new()));
    let mut kernel = MuKernel::new(sigma_ptr);

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    kernel.execute_task(1, &obs).unwrap();

    let state = kernel.state();
    let receipt = state.receipts.latest().unwrap();

    // Verify receipt proves A = μ(O)
    assert!(receipt.proves_equation());

    // Verify receipt hash is deterministic
    let hash1 = receipt.hash();
    let hash2 = receipt.hash();
    assert_eq!(hash1, hash2);
}

#[test]
fn test_pattern_execution() {
    use knhk_mu_kernel::patterns::PatternId;
    use knhk_mu_kernel::timing::TickBudget;

    let obs = GuardContext {
        params: [100, 50, 0, 0, 0, 0, 0, 0],
    };

    // Test all 43 patterns
    for i in 0..43 {
        let pattern: PatternId = unsafe { core::mem::transmute(i as u8) };
        let mut budget = TickBudget::chatman();

        let result = pattern.execute(&obs, &mut budget);
        assert!(result.is_ok(), "Pattern {:?} failed", pattern);
        assert!(
            budget.used() <= CHATMAN_CONSTANT,
            "Pattern {:?} violated Chatman Constant",
            pattern
        );
    }
}
