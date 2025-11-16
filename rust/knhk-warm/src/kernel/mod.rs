// kernel/mod.rs - Module declarations for kernel components
// Phase 3: Warm Path & Descriptor Management

pub mod warm_path;
pub mod descriptor_manager;
pub mod versioning;
pub mod telemetry_pipeline;
pub mod coordination;
pub mod degradation;
pub mod knowledge_integration;

// Re-exports for convenience
pub use warm_path::{WarmPathExecutor, WarmPathResult, WorkItem};
pub use descriptor_manager::{DescriptorManager, Descriptor, DescriptorContent, DescriptorVersion};
pub use versioning::{VersionGraph, VersionSigner, RollbackManager, TimeTravelExecutor};
pub use telemetry_pipeline::{TelemetryPipeline, TelemetryReceipt, TraceContext};
pub use coordination::{ChannelManager, CoordinationMessage, BackpressureController, ShutdownCoordinator};
pub use degradation::{DegradationManager, DegradationStrategy, CircuitBreaker};
pub use knowledge_integration::{KnowledgeBase, MAPEKIntegration, MAPEKPhase};