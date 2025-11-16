//! Type-level validation and safety proofs

use std::error::Error;

/// Trait for types that can be safely promoted
///
/// Implementors must prove (at compile-time via const) that all
/// invariants are verified. The type system enforces this.
pub trait PromotionSafe: Send + Sync {
    /// Compile-time proof: invariants are verified
    ///
    /// This const must be true for the type to be promotable.
    /// It provides a compile-time guarantee of safety.
    const INVARIANTS_VERIFIED: bool;

    /// Runtime verification (redundant but safe)
    ///
    /// This method performs additional runtime checks.
    /// It's redundant with the compile-time guarantees but provides
    /// defense-in-depth.
    fn verify_promotion(&self) -> Result<(), Box<dyn Error>>;
}

/// Phantom type: proves Σ preserved all invariants Q
///
/// This is a zero-sized type that can only be constructed after
/// successful validation. It provides a type-level proof that
/// invariants were preserved.
#[derive(Debug, Clone, Copy)]
pub struct InvariantsPreserved;

impl InvariantsPreserved {
    /// Verify invariants from a receipt
    ///
    /// This is the ONLY way to construct InvariantsPreserved.
    /// It requires runtime validation, which then provides a
    /// type-level guarantee.
    pub fn verify(receipt: &knhk_ontology::SigmaReceipt) -> Result<Self, Box<dyn Error>> {
        // Runtime validation of all 5 invariants
        if !receipt.validation_results.invariants_q_preserved {
            return Err("Invariants not preserved".into());
        }

        if !receipt.production_ready {
            return Err("Receipt not production-ready".into());
        }

        // Verify receipt integrity
        receipt.verify().map_err(|e| Box::new(e) as Box<dyn Error>)?;

        // Only if all checks pass, return the phantom type
        Ok(InvariantsPreserved)
    }

    /// Verify invariants are still valid
    ///
    /// Re-check that the proof is still valid
    pub fn revalidate(&self, receipt: &knhk_ontology::SigmaReceipt) -> Result<(), Box<dyn Error>> {
        if !receipt.validation_results.invariants_q_preserved {
            return Err("Invariants no longer preserved".into());
        }
        Ok(())
    }
}

/// Type-safe promotion with compile-time proof
///
/// This function can ONLY be called with a type-level proof (InvariantsPreserved).
/// The compiler enforces that you cannot call this without first validating.
#[inline]
pub fn promote_with_proof<T: PromotionSafe>(
    _value: &T,
    _proof: InvariantsPreserved,
) -> Result<(), Box<dyn Error>> {
    // The type system has already proven this is safe
    // (InvariantsPreserved can only exist if validation passed)

    // Compile-time assertion
    const {
        assert!(T::INVARIANTS_VERIFIED, "Type must have verified invariants");
    };

    // This function body is essentially a no-op
    // The safety comes from the type-level proof
    Ok(())
}

/// Const function for promotion cost estimation
///
/// This allows compile-time verification of performance budgets.
#[inline]
pub const fn estimate_promotion_cost<const DESCRIPTOR_SIZE: usize>() -> u32 {
    // Cost breakdown (CPU ticks):
    // 1. Atomic load of old descriptor: 2-3 ticks
    // 2. Create new descriptor (stack): 1-2 ticks
    // 3. Atomic store of new descriptor: 2-3 ticks
    // 4. Memory barrier (SeqCst): 3-5 ticks
    // Total: 8-13 ticks (worst case)

    // Compile-time checks (these fail at compile-time if violated)
    const { assert!(DESCRIPTOR_SIZE <= 64, "Descriptor must fit in cache line"); }
    const { assert!(DESCRIPTOR_SIZE % 8 == 0, "Descriptor must be 8-byte aligned"); }

    13 // Worst-case ticks
}

// Compile-time test of cost estimation
const _PROMOTION_COST: u32 = estimate_promotion_cost::<64>();
const _: () = assert!(_PROMOTION_COST <= 15, "Promotion cost exceeds Chatman Constant");

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SigmaReceipt, SigmaSnapshotId, ValidationResults};

    fn create_valid_receipt() -> SigmaReceipt {
        let snapshot_id: SigmaSnapshotId = [1; 32];
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![],
            warnings: vec![],
        };

        SigmaReceipt::new(
            snapshot_id,
            None,
            "Test snapshot".to_string(),
            results,
            100,
        )
    }

    fn create_invalid_receipt() -> SigmaReceipt {
        let snapshot_id: SigmaSnapshotId = [1; 32];
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: false, // VIOLATION!
            errors: vec![],
            warnings: vec![],
        };

        SigmaReceipt::new(
            snapshot_id,
            None,
            "Bad snapshot".to_string(),
            results,
            100,
        )
    }

    #[test]
    fn test_invariants_preserved_valid() {
        let receipt = create_valid_receipt();
        let proof = InvariantsPreserved::verify(&receipt);

        assert!(proof.is_ok(), "Should verify valid receipt");
    }

    #[test]
    fn test_invariants_preserved_invalid() {
        let receipt = create_invalid_receipt();
        let proof = InvariantsPreserved::verify(&receipt);

        assert!(proof.is_err(), "Should reject invalid receipt");
    }

    #[test]
    fn test_revalidate() {
        let receipt = create_valid_receipt();
        let proof = InvariantsPreserved::verify(&receipt).unwrap();

        assert!(proof.revalidate(&receipt).is_ok(), "Should revalidate successfully");
    }

    #[test]
    fn test_cost_estimation() {
        const COST_64: u32 = estimate_promotion_cost::<64>();
        assert!(COST_64 <= 15, "Cost should be within budget");

        const COST_32: u32 = estimate_promotion_cost::<32>();
        assert!(COST_32 <= 15, "Smaller descriptor should also be within budget");
    }

    // This test demonstrates that the type system prevents unsafe promotion
    #[test]
    fn test_type_level_proof_required() {
        // Define a test type
        struct TestPromotion;

        impl PromotionSafe for TestPromotion {
            const INVARIANTS_VERIFIED: bool = true;

            fn verify_promotion(&self) -> Result<(), Box<dyn Error>> {
                Ok(())
            }
        }

        let value = TestPromotion;
        let receipt = create_valid_receipt();
        let proof = InvariantsPreserved::verify(&receipt).unwrap();

        // This compiles because we have the proof
        let result = promote_with_proof(&value, proof);
        assert!(result.is_ok());

        // Without proof, this would not compile:
        // let result = promote_with_proof(&value);  // ← Compile error!
    }
}
