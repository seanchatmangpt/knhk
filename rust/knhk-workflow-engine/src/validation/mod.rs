//! Workflow validation module
//!
//! Provides validation capabilities including deadlock detection,
//! workflow structure validation, and soundness validation.
//!
//! - `deadlock`: Deadlock detection for workflow instances
//! - `sparql`: SPARQL-based validation rules (VR-N001, VR-DF001)
//! - `shacl`: SHACL-based soundness validation (VR-S001 through VR-S012)
//! - `framework`: Van der Aalst end-to-end validation framework
//! - `fitness`: Fitness validation (can the process execute?)
//! - `precision`: Precision validation (does it match specification?)
//! - `generalization`: Generalization validation (works beyond examples?)
//! - `process_mining`: Process mining analysis (XES, conformance)
//! - `formal`: Formal verification (state transitions, deadlock freedom)
//! - `report`: Validation report generation

pub mod deadlock;
pub mod shacl;
pub mod sparql;

// Van der Aalst validation framework
pub mod fitness;
pub mod formal;
pub mod framework;
pub mod generalization;
pub mod precision;
pub mod process_mining;
pub mod report;

pub use deadlock::{DeadlockDetectionResult, DeadlockDetector};
pub use shacl::{ShaclValidationReport, ShaclValidator, ShaclViolation, ValidationSeverity};
pub use sparql::{SparqlValidationResult, SparqlValidator, ValidationViolation};

// Van der Aalst framework exports
pub use fitness::FitnessValidator;
pub use formal::FormalVerifier;
pub use framework::ValidationFramework;
pub use generalization::GeneralizationValidator;
pub use precision::PrecisionValidator;
pub use process_mining::ProcessMiningAnalyzer;
pub use report::{ValidationDetail, ValidationReport, ValidationResult, ValidationStatus};
