//! KNHK Promotion Pipeline - Type-Safe Snapshot Promotion with ≤10 Tick Guarantees
//!
//! ## Vision: 2027-Grade Rust
//!
//! This crate demonstrates cutting-edge Rust patterns:
//! - **Type-level state machines** (compile-time proof of promotion safety)
//! - **Const generics** (snapshot IDs verified at compile-time)
//! - **GATs** (Generic Associated Types) for flexible snapshot binding
//! - **Procedural macros** for automatic invariant checking
//! - **Lock-free atomics** (≤10 ticks)
//! - **Zero-cost abstractions** (no allocation on hot path)
//!
//! ## Architecture
//!
//! ```text
//! Preparing → compile artifacts → Ready → atomic swap → Promoted
//!    ↑           (async, slow)       ↑      (≤10 ticks)     ↑
//!    │                               │                       │
//!    └─────── Type System Guards ────┴───────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_promotion::*;
//! use knhk_ontology::*;
//! use knhk_projections::*;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize hot path (once at startup)
//! init_hot_path();
//!
//! // Create snapshot store
//! let store = SnapshotStore::new();
//! let snapshot_id = [1u8; 32]; // Your snapshot ID
//!
//! // Create promotion guard (Preparing state)
//! let guard = PromotionGuard::new(
//!     snapshot_id,
//!     Arc::new(SigmaReceipt::new(/* ... */)),
//!     Arc::new(CompiledProjections { /* ... */ }),
//! )?;
//!
//! // Transition to Ready (after compilation completes)
//! let guard = guard.ready().await?;
//!
//! // Promote atomically (≤10 ticks)
//! let promoted = guard.promote()?;
//!
//! println!("Promotion successful!");
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod state_machine;
pub mod descriptor;
pub mod validation;
pub mod promotion;
pub mod hot_path;
pub mod errors;
pub mod telemetry;

// Re-export main types
pub use state_machine::{PromotionGuard, Preparing, Ready, Promoted};
pub use descriptor::{SnapshotDescriptor, HotPathAccessor};
pub use validation::{PromotionSafe, InvariantsPreserved};
pub use promotion::{PromotionPipeline, PromotionResult};
pub use hot_path::{init_hot_path, get_current_snapshot, HotPathBinder};
pub use errors::{PromotionError, Result};
pub use telemetry::PromotionTelemetry;

// Re-export from knhk-promotion-macros
pub use knhk_promotion_macros::{PromotionSafe as DerivePromotionSafe, atomic_operation, PhantomState};

// Type aliases for convenience
pub use knhk_ontology::{SigmaSnapshot, SigmaSnapshotId, SigmaReceipt, SnapshotStore};
pub use knhk_projections::{ProjectionCompiler, CompiledProjections};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use knhk_ontology::*;
    use std::sync::Arc;

    #[test]
    fn test_hot_path_init() {
        init_hot_path();
        let snapshot_id = get_current_snapshot();
        assert_eq!(snapshot_id, [0u8; 32], "Initial snapshot should be zeros");
    }

    #[test]
    fn test_type_level_state_machine() {
        // This test demonstrates that the type system enforces state transitions
        init_hot_path();

        let snapshot_id = [1u8; 32];
        let results = ValidationResults {
            static_checks_passed: true,
            dynamic_checks_passed: true,
            performance_checks_passed: true,
            invariants_q_preserved: true,
            errors: vec![],
            warnings: vec![],
        };

        let receipt = Arc::new(SigmaReceipt::new(
            snapshot_id,
            None,
            "Test".to_string(),
            results,
            100,
        ));

        // Create compiled projections (minimal for test)
        let compiled = Arc::new(CompiledProjections {
            snapshot_id,
            snapshot_hash: [0; 32],
            rust_models: knhk_projections::generators::RustModelsOutput {
                models_code: String::new(),
                hash: [0; 32],
            },
            openapi_spec: knhk_projections::generators::OpenApiOutput {
                openapi_spec: String::new(),
                paths: Vec::new(),
                schemas: Vec::new(),
                hash: [0; 32],
            },
            hooks_config: knhk_projections::generators::HooksOutput {
                hooks_config: String::new(),
                guards: Vec::new(),
                operators: Vec::new(),
                hash: [0; 32],
            },
            markdown_docs: knhk_projections::generators::MarkdownOutput {
                markdown: String::new(),
                sections: Vec::new(),
                hash: [0; 32],
            },
            otel_schema: knhk_projections::generators::OtelOutput {
                otel_schema: String::new(),
                spans: Vec::new(),
                metrics: Vec::new(),
                hash: [0; 32],
            },
            compiled_at: std::time::SystemTime::now(),
        });

        // Create guard in Preparing state
        let guard = PromotionGuard::new(snapshot_id, receipt, compiled);
        assert!(guard.is_ok(), "Should create guard in Preparing state");

        // Note: Cannot call promote() here - compiler prevents it!
        // guard.promote(); // ← Compile error: method not available in Preparing state
    }
}
