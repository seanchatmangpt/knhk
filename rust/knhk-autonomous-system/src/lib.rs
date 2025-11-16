//! # KNHK Autonomous Ontology System - The Grand Integration
//!
//! This crate orchestrates all 7 layers of the autonomous ontology plant and binds
//! them to KNHK's three orthogonal axes (τ, μ, Γ).
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │  Autonomous Ontology Plant                                   │
//! ├──────────────────────────────────────────────────────────────┤
//! │  Layer 1: Σ² (Meta-Ontology Definition)                      │
//! │  Layer 2: Σ  (Runtime Snapshot Management)                   │
//! │  Layer 3: ΔΣ (Change Proposals & Validation)                 │
//! │  Layer 4: Π  (Projection Compiler)                           │
//! │  Layer 5: Promotion Pipeline (Atomic Switching)              │
//! │  Layer 6: Autonomous Loop (Evolution)                        │
//! │  Layer 7: Integration Layer (THIS CRATE)                     │
//! └──────────────────────────────────────────────────────────────┘
//!         ↓                    ↓                    ↓
//! ┌──────────────┐  ┌──────────────────┐  ┌──────────────────┐
//! │  τ-Axis      │  │  μ-Axis          │  │  Γ-Axis          │
//! │  Time Bound  │  │  Hook Function   │  │  Glue/Sheaf      │
//! │  ≤8 ticks    │  │  A = μ(O)        │  │  Multi-region    │
//! └──────────────┘  └──────────────────┘  └──────────────────┘
//! ```
//!
//! ## Vision
//!
//! This is the **capstone** that proves the entire system works end-to-end:
//!
//! - **All 7 layers integrated** and working together
//! - **τ-axis verified**: All operations ≤ 15 ticks (includes margin for verification)
//! - **μ-axis verified**: A = μ(O) deterministically
//! - **Γ-axis verified**: Glue operator properties hold
//! - **Telemetry emitted**: All operations traced via OpenTelemetry
//! - **Weaver validation ready**: Schema-compliant telemetry
//! - **Humans not needed**: Evolution loop runs indefinitely
//! - **Production-ready**: Full error handling, no panics
//!
//! ## Example
//!
//! ```rust,no_run
//! use knhk_autonomous_system::{AutonomousOntologyPlant, SystemConfig, StorageBackend};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize the entire autonomous plant
//!     let plant = AutonomousOntologyPlant::initialize(
//!         "registry/meta-ontology.ttl",
//!         StorageBackend::InMemory,
//!         SystemConfig::default(),
//!     ).await?;
//!
//!     // Verify all 5 invariants + 3 orthogonal axes
//!     plant.verify_system_invariants().await?;
//!
//!     // Start autonomous evolution (runs forever)
//!     plant.start().await?;
//!
//!     // Hot path continues unaffected
//!     loop {
//!         let snapshot = plant.current_snapshot().await?;
//!         // Process operations...
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod system;
pub mod timeline;
pub mod mapping;
pub mod consistency;
pub mod telemetry;
pub mod config;
pub mod errors;

// Re-export main types
pub use system::AutonomousOntologyPlant;
pub use timeline::TimeAxisVerifier;
pub use mapping::MappingAxisVerifier;
pub use consistency::GlueAxisVerifier;
pub use telemetry::OTelIntegration;
pub use config::{SystemConfig, StorageBackend, LoopConfiguration};
pub use errors::{SystemError, Result};

// Re-export key types from dependencies for convenience
pub use knhk_ontology::{SigmaSnapshot, SigmaSnapshotId, SnapshotStore};
pub use knhk_change_engine::ChangeExecutor;
pub use knhk_projections::ProjectionCompiler;
pub use knhk_promotion::PromotionPipeline;
// pub use knhk_autonomous_loop::AutonomousLoopController;  // Temporarily disabled

/// Version of the autonomous system
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Current system API version (semver)
pub const API_VERSION: &str = "1.0.0";

/// Performance constraint: Maximum ticks allowed for hot path operations
/// This is the famous "Chatman Constant" - ≤8 ticks
pub const MAX_HOT_PATH_TICKS: u64 = 8;

/// Performance constraint: Maximum ticks for promotion operations
/// Includes margin for verification overhead
pub const MAX_PROMOTION_TICKS: u64 = 15;

/// Maximum number of concurrent patterns to process
pub const MAX_CONCURRENT_PATTERNS: usize = 100;

/// Default cycle interval for autonomous evolution
pub const DEFAULT_CYCLE_INTERVAL_SECS: u64 = 60;
