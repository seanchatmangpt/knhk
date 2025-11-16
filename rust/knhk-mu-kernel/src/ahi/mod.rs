//! AHI - Anticipatory Hybrid Intelligence Layer
//!
//! This module implements the AHI layer as a constrained user space
//! that operates on top of the μ-kernel. AHI components must request
//! resources from the kernel and prove their tick budgets.
//!
//! # Architecture
//!
//! ```text
//! AHI Layer (User Space):
//!   - MAPE-K control loops
//!   - Doctrine enforcement
//!   - Marketplace mechanisms
//!   - Learning systems
//!
//! Constitutional Boundary:
//!   - Resource quotas
//!   - Tick budgets
//!   - Proof obligations
//!   - Type-safe separation
//!
//! μ-Kernel (Privileged):
//!   - ISA operations
//!   - Σ* management
//!   - Receipt generation
//!   - Guard enforcement
//! ```
//!
//! # Key Principles
//!
//! 1. **Typed Constitution**: AHI cannot construct kernel artifacts
//!    except through sanctioned interfaces
//! 2. **Resource Accounting**: All tick consumption tracked and budgeted
//! 3. **Proof-Carrying**: ΔΣ requires recognized proof objects
//! 4. **Timescale Separation**: Hot/warm/cold enforced by type system

pub mod decision;
pub mod userspace;
pub mod timescales;

pub use decision::{Decision, ObservationSlice, InvariantId, RiskClass};
pub use userspace::{AhiContext, AhiError, AhiProvenOverlay, AhiOverlayProof, SubmitToken};
pub use timescales::{Hot, Warm, Cold, HotError, WarmError, ColdError, TimescaleClass};

/// AHI version
pub const AHI_VERSION: (u8, u8, u8) = (1, 0, 0);

/// Maximum tick quota for AHI operations
pub const AHI_TICK_QUOTA: u64 = 1_000_000;

/// Default tick quota for single AHI operation
pub const AHI_DEFAULT_QUOTA: u64 = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ahi_module_structure() {
        // Verify module is properly structured
        assert_eq!(AHI_VERSION, (1, 0, 0));
        assert_eq!(AHI_TICK_QUOTA, 1_000_000);
    }
}
