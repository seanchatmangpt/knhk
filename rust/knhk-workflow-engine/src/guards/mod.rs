//! Guard Enforcement System
//!
//! Runtime enforcement of guards Q including:
//! - Invariant checking
//! - SHACL validation
//! - Precondition/postcondition enforcement

pub mod invariant_checker;
pub mod shacl_validator;

pub use invariant_checker::{
    Invariant, InvariantCheckResult, InvariantChecker, InvariantPredicate, InvariantType,
};
pub use shacl_validator::{
    NodeKind, PropertyConstraint, ShaclSeverity, ShaclShape, ShaclValidationResult,
    ShaclValidator, ShaclViolation,
};
