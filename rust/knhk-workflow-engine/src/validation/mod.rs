//! Workflow validation module
//!
//! Provides validation capabilities including deadlock detection,
//! workflow structure validation, and soundness validation.
//!
//! - `deadlock`: Deadlock detection for workflow instances
//! - `sparql`: SPARQL-based validation rules (VR-N001, VR-DF001)
//! - `shacl`: SHACL-based soundness validation (VR-S001 through VR-S012)

pub mod deadlock;
pub mod shacl;
pub mod sparql;

pub use deadlock::{DeadlockDetectionResult, DeadlockDetector};
pub use shacl::{ShaclValidationReport, ShaclValidator, ShaclViolation, ValidationSeverity};
pub use sparql::{SparqlValidationResult, SparqlValidator, ValidationViolation};
