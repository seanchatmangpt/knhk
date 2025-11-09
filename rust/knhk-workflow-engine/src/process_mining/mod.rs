//! Process Mining - XES export for ProM compatibility
//!
//! Enables process mining analysis by exporting workflow execution logs
//! in IEEE XES (eXtensible Event Stream) format.
//!
//! **80/20 Focus:**
//! - Case ID (trace identifier)
//! - Activity name
//! - Timestamp (event ordering)
//! - Lifecycle (start/complete/cancel)
//!
//! **Usage with ProM:**
//! ```bash
//! # Export workflow execution to XES
//! knhk-workflow export-xes case-abc123 --output case-abc123.xes
//!
//! # Import into ProM for analysis
//! prom --import case-abc123.xes
//! prom --discover-model case-abc123.xes --output discovered_model.pnml
//! prom --check-conformance workflow1.pnml case-abc123.xes
//! ```

pub mod xes_export;

pub use xes_export::{WorkflowEvent, XesExporter};
