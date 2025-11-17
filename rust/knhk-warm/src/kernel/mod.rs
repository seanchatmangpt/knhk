// kernel/mod.rs - Module declarations for kernel components
// Phase 3: Warm Path & Descriptor Management

pub mod coordination;
pub mod degradation;
pub mod descriptor_manager;
pub mod knowledge_integration;
pub mod telemetry_pipeline;
pub mod versioning;
pub mod warm_path;

// Re-exports for convenience
pub use coordination::{
    BackpressureController, ChannelManager, CoordinationMessage, ShutdownCoordinator,
};
pub use degradation::{CircuitBreaker, DegradationManager, DegradationStrategy};
pub use descriptor_manager::{Descriptor, DescriptorContent, DescriptorManager, DescriptorVersion};
pub use knowledge_integration::{KnowledgeBase, MAPEKIntegration, MAPEKPhase};
pub use telemetry_pipeline::{TelemetryPipeline, TelemetryReceipt, TraceContext};
pub use versioning::{RollbackManager, TimeTravelExecutor, VersionGraph, VersionSigner};
pub use warm_path::{WarmPathExecutor, WarmPathResult, WorkItem};
