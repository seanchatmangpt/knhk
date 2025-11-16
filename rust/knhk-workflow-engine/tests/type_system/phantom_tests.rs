//! Phantom type validation integration tests

use knhk_workflow_engine::error::WorkflowError;
use knhk_workflow_engine::parser::WorkflowSpecId;
use knhk_workflow_engine::types::phantom::{Validatable, Validate, WorkflowSpec};

#[test]
fn test_phantom_validation_success() {
    let unvalidated = Validatable::new("valid_input".to_string());

    let validated = unvalidated
        .validate(|s| {
            if s.len() > 5 {
                Ok(())
            } else {
                Err(WorkflowError::Validation("Too short".to_string()))
            }
        })
        .expect("validation should succeed");

    assert_eq!(validated.get(), "valid_input");
}

#[test]
fn test_phantom_validation_failure() {
    let unvalidated = Validatable::new("abc".to_string());

    let result = unvalidated.validate(|s| {
        if s.len() > 5 {
            Ok(())
        } else {
            Err(WorkflowError::Validation("Too short".to_string()))
        }
    });

    assert!(result.is_err());
}

#[test]
fn test_phantom_validation_with_transformation() {
    let unvalidated = Validatable::new("123".to_string());

    let validated = unvalidated
        .validate_and_transform(|s| {
            s.parse::<u32>()
                .map_err(|_| WorkflowError::Validation("Not a number".to_string()))
        })
        .expect("transformation should succeed");

    assert_eq!(validated.get(), &123u32);
}

#[test]
fn test_phantom_validation_map() {
    let unvalidated = Validatable::new(10u32);

    let validated = unvalidated
        .validate(|n| {
            if *n > 0 {
                Ok(())
            } else {
                Err(WorkflowError::Validation("Must be positive".to_string()))
            }
        })
        .expect("validation should succeed");

    let doubled = validated.map(|n| n * 2);
    assert_eq!(doubled.get(), &20u32);
}

#[test]
fn test_phantom_workflow_spec_validation() {
    let spec_id = WorkflowSpecId::new("test-spec".to_string());
    let data = serde_json::json!({"key": "value"});

    let unvalidated = WorkflowSpec::new(spec_id.clone(), data.clone());
    let validated = unvalidated.validate().expect("validation should succeed");

    assert_eq!(validated.spec_id(), &spec_id);
    assert_eq!(validated.data(), &data);
}

// This test demonstrates compile-time safety
// Uncommenting the following would cause a compile error:
/*
#[test]
fn test_phantom_cannot_use_unvalidated() {
    fn requires_validated(v: &Validatable<String, Validated>) {
        println!("{}", v.get());
    }

    let unvalidated = Validatable::new("test".to_string());

    // ERROR: expected `Validated`, found `NotValidated`
    requires_validated(&unvalidated);  // ← Won't compile!
}
*/

#[test]
fn test_phantom_type_zero_cost() {
    // Phantom types should add no runtime overhead
    let validated = Validatable::new(42u64)
        .validate(|_| Ok(()))
        .unwrap();

    assert_eq!(
        std::mem::size_of_val(&validated),
        std::mem::size_of::<u64>()
    );
}

#[test]
fn test_phantom_validation_chain() {
    let result = Validatable::new("100".to_string())
        .validate(|s| {
            if s.is_empty() {
                Err(WorkflowError::Validation("Empty".to_string()))
            } else {
                Ok(())
            }
        })
        .and_then(|v| {
            v.validate_and_transform(|s| {
                s.parse::<u32>()
                    .map_err(|_| WorkflowError::Validation("Not a number".to_string()))
            })
        })
        .map(|v| v.map(|n| n * 2));

    assert!(result.is_ok());
    assert_eq!(result.unwrap().get(), &200u32);
}

#[test]
fn test_phantom_into_inner() {
    let validated = Validatable::new("test".to_string())
        .validate(|_| Ok(()))
        .unwrap();

    let inner = validated.into_inner();
    assert_eq!(inner, "test");
}

struct CustomType {
    value: u32,
}

impl Validate for CustomType {
    type Error = WorkflowError;

    fn validate(&self) -> Result<(), Self::Error> {
        if self.value > 0 && self.value <= 100 {
            Ok(())
        } else {
            Err(WorkflowError::Validation(
                "Value must be between 1 and 100".to_string(),
            ))
        }
    }
}

#[test]
fn test_phantom_validate_trait() {
    let valid = CustomType { value: 50 };
    let validated = valid.into_validated();
    assert!(validated.is_ok());

    let invalid = CustomType { value: 0 };
    let validated = invalid.into_validated();
    assert!(validated.is_err());
}
