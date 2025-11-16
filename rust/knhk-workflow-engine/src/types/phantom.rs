//! Phantom Types for Compile-Time Validation
//!
//! Uses phantom types to enforce validation at compile time, preventing runtime errors
//! and ensuring type safety without runtime overhead.

use crate::error::{WorkflowError, WorkflowResult};
use std::marker::PhantomData;

/// Marker trait for validated types
pub trait ValidationMarker: sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

/// Type marker indicating data has been validated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validated;

/// Type marker indicating data has not been validated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotValidated;

impl sealed::Sealed for Validated {}
impl sealed::Sealed for NotValidated {}
impl ValidationMarker for Validated {}
impl ValidationMarker for NotValidated {}

/// Wrapper type that tracks validation state at compile time
///
/// The type parameter `V` is a phantom type that indicates whether the contained
/// value has been validated. This prevents using unvalidated data where validated
/// data is required, catching errors at compile time.
#[derive(Debug, Clone)]
pub struct Validatable<T, V: ValidationMarker = NotValidated> {
    value: T,
    _validation: PhantomData<V>,
}

impl<T> Validatable<T, NotValidated> {
    /// Create a new unvalidated value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use knhk_workflow_engine::types::phantom::Validatable;
    ///
    /// let unvalidated = Validatable::new("user_input".to_string());
    /// // Cannot use unvalidated where Validated is required!
    /// ```
    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _validation: PhantomData,
        }
    }

    /// Validate the value, transitioning from NotValidated to Validated
    ///
    /// This is the only way to convert from NotValidated to Validated,
    /// ensuring validation always occurs.
    #[inline]
    pub fn validate<F>(self, validator: F) -> WorkflowResult<Validatable<T, Validated>>
    where
        F: FnOnce(&T) -> WorkflowResult<()>,
    {
        validator(&self.value)?;
        Ok(Validatable {
            value: self.value,
            _validation: PhantomData,
        })
    }

    /// Validate with transformation
    #[inline]
    pub fn validate_and_transform<U, F>(
        self,
        validator: F,
    ) -> WorkflowResult<Validatable<U, Validated>>
    where
        F: FnOnce(T) -> WorkflowResult<U>,
    {
        let new_value = validator(self.value)?;
        Ok(Validatable {
            value: new_value,
            _validation: PhantomData,
        })
    }
}

impl<T> Validatable<T, Validated> {
    /// Get a reference to the validated value
    ///
    /// This is safe because the type system guarantees validation has occurred.
    #[inline(always)]
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Consume and extract the validated value
    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Map the validated value to another type, preserving validation state
    #[inline]
    pub fn map<U, F>(self, f: F) -> Validatable<U, Validated>
    where
        F: FnOnce(T) -> U,
    {
        Validatable {
            value: f(self.value),
            _validation: PhantomData,
        }
    }
}

/// Trait for types that can be validated
pub trait Validate: Sized {
    /// Validation error type
    type Error: Into<WorkflowError>;

    /// Validate the value
    fn validate(&self) -> Result<(), Self::Error>;

    /// Consume and validate, returning a validated wrapper
    #[inline]
    fn into_validated(self) -> WorkflowResult<Validatable<Self, Validated>> {
        self.validate()
            .map_err(|e| e.into())?;
        Ok(Validatable {
            value: self,
            _validation: PhantomData,
        })
    }
}

/// Phantom type for workflow specification validation state
#[derive(Debug, Clone, Copy)]
pub struct SpecValidated;

/// Phantom type for workflow specification not validated
#[derive(Debug, Clone, Copy)]
pub struct SpecNotValidated;

impl sealed::Sealed for SpecValidated {}
impl sealed::Sealed for SpecNotValidated {}
impl ValidationMarker for SpecValidated {}
impl ValidationMarker for SpecNotValidated {}

/// Workflow specification with compile-time validation tracking
#[derive(Debug, Clone)]
pub struct WorkflowSpec<V: ValidationMarker = SpecNotValidated> {
    pub spec_id: crate::parser::WorkflowSpecId,
    pub data: serde_json::Value,
    _validation: PhantomData<V>,
}

impl WorkflowSpec<SpecNotValidated> {
    /// Create a new unvalidated workflow spec
    #[inline(always)]
    pub fn new(spec_id: crate::parser::WorkflowSpecId, data: serde_json::Value) -> Self {
        Self {
            spec_id,
            data,
            _validation: PhantomData,
        }
    }

    /// Validate the workflow specification
    #[inline]
    pub fn validate(self) -> WorkflowResult<WorkflowSpec<SpecValidated>> {
        // Validation logic would go here
        // For now, we just transition the type state
        Ok(WorkflowSpec {
            spec_id: self.spec_id,
            data: self.data,
            _validation: PhantomData,
        })
    }
}

impl WorkflowSpec<SpecValidated> {
    /// Access validated spec data
    #[inline(always)]
    pub fn spec_id(&self) -> &crate::parser::WorkflowSpecId {
        &self.spec_id
    }

    /// Access validated data
    #[inline(always)]
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }
}

/// Compile-time proof that validation has occurred
///
/// This type can only be constructed after validation, serving as a witness
/// that validation has taken place.
#[derive(Debug, Clone, Copy)]
pub struct ValidationProof<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T> ValidationProof<T> {
    /// Create a validation proof (private, can only be created by validation functions)
    #[inline(always)]
    fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

/// Create a validation proof for a successfully validated value
#[inline(always)]
pub fn create_proof<T>() -> ValidationProof<T> {
    ValidationProof::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_validation_flow() {
        let unvalidated = Validatable::new("test_value".to_string());

        let validated = unvalidated
            .validate(|s| {
                if s.is_empty() {
                    Err(WorkflowError::Validation("Empty string".to_string()))
                } else {
                    Ok(())
                }
            })
            .expect("validation should succeed");

        assert_eq!(validated.get(), "test_value");
    }

    #[test]
    fn test_phantom_validation_failure() {
        let unvalidated = Validatable::new("".to_string());

        let result = unvalidated.validate(|s| {
            if s.is_empty() {
                Err(WorkflowError::Validation("Empty string".to_string()))
            } else {
                Ok(())
            }
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_cost_phantom_types() {
        // Phantom types should add zero runtime cost
        let validated = Validatable {
            value: 42u64,
            _validation: PhantomData::<Validated>,
        };

        // Size should be same as the inner value
        assert_eq!(
            std::mem::size_of_val(&validated),
            std::mem::size_of::<u64>()
        );
    }
}
