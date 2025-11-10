//! RDF support for workflow patterns
//!
//! Provides RDF serialization and metadata for workflow patterns:
//! - Pattern metadata extraction
//! - Execution context serialization to RDF
//! - Execution result serialization to RDF
//! - Pattern ontology support

pub mod deserialize;
pub mod metadata;
pub mod serialize;
pub mod utils;

#[cfg(feature = "rdf")]
pub use deserialize::{
    deserialize_context_from_rdf,
    deserialize_metadata_from_rdf,
    deserialize_result_from_rdf,
};
pub use deserialize::load_all_metadata_from_rdf;
pub use metadata::{get_all_pattern_metadata, PatternMetadata};
pub use serialize::{serialize_context_to_rdf, serialize_metadata_to_rdf, serialize_result_to_rdf};

/// Workflow pattern namespace
pub const WORKFLOW_PATTERN_NS: &str = "http://bitflow.ai/ontology/workflow-pattern/v1#";

/// YAWL namespace
pub const YAWL_NS: &str = "http://bitflow.ai/ontology/yawl/v2#";
