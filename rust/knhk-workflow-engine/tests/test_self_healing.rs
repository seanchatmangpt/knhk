//! Integration tests for self-healing code generation system
//!
//! Tests the complete self-healing workflow including:
//! - Error detection from compiler output
//! - Fix suggestion generation
//! - Automatic code repair
//! - Health metrics tracking

use knhk_workflow_engine::ggen::neural_patterns::TargetLanguage;
use knhk_workflow_engine::ggen::self_healing::{
    CodeError, ErrorType, Fix, HealthMetrics, SelfHealingGenerator, ValidationResult,
};
use std::time::Duration;

#[test]
fn test_generator_creation() {
    // Valid creation
    let generator = SelfHealingGenerator::new(3);
    assert!(
        generator.is_ok(),
        "Generator should be created successfully"
    );

    // Invalid creation (zero retries)
    let generator = SelfHealingGenerator::new(0);
    assert!(generator.is_err(), "Generator with 0 retries should fail");
}

#[test]
fn test_error_detection_rust() {
    // Test Rust compiler error detection
    let output = r#"
error[E0425]: cannot find value `x` in this scope
 --> main.rs:2:5
  |
2 |     x
  |     ^ not found in this scope
"#;

    let errors = SelfHealingGenerator::detect_errors(output).unwrap();
    assert!(!errors.is_empty(), "Should detect errors");
    assert_eq!(errors[0].error_type, ErrorType::UndefinedVariable);
}

#[test]
fn test_error_detection_type_mismatch() {
    let output = r#"
error[E0308]: mismatched types
 --> main.rs:3:10
  |
3 |     let x: String = "hello";
  |            ------   ^^^^^^^ expected `String`, found `&str`
"#;

    let errors = SelfHealingGenerator::detect_errors(output).unwrap();
    assert!(!errors.is_empty(), "Should detect type mismatch");
    assert_eq!(errors[0].error_type, ErrorType::TypeMismatch);
}

#[test]
fn test_error_detection_missing_import() {
    let output = r#"
error[E0433]: failed to resolve: use of undeclared crate or module `HashMap`
 --> main.rs:1:5
  |
1 | use HashMap;
  |     ^^^^^^^ use of undeclared crate or module `HashMap`
"#;

    let errors = SelfHealingGenerator::detect_errors(output).unwrap();
    assert!(!errors.is_empty(), "Should detect missing import");
    assert_eq!(errors[0].error_type, ErrorType::MissingImport);
}

#[test]
fn test_error_detection_python() {
    let output = "SyntaxError: invalid syntax";
    let errors = SelfHealingGenerator::detect_errors(output).unwrap();
    assert!(!errors.is_empty(), "Should detect Python syntax error");
    assert_eq!(errors[0].error_type, ErrorType::SyntaxError);
}

#[tokio::test]
async fn test_suggest_fixes_missing_import() {
    let generator = SelfHealingGenerator::new(3).unwrap();

    let error = CodeError::new(
        ErrorType::MissingImport,
        "cannot find HashMap".to_string(),
        None,
        "use HashMap;".to_string(),
    );

    let fixes = generator.suggest_fixes(&error).await.unwrap();
    assert!(!fixes.is_empty(), "Should suggest fixes for missing import");

    // Check fix confidence
    assert!(fixes[0].confidence > 0.8, "Fix should have high confidence");
}

#[tokio::test]
async fn test_suggest_fixes_type_mismatch() {
    let generator = SelfHealingGenerator::new(3).unwrap();

    let error = CodeError::new(
        ErrorType::TypeMismatch,
        "expected `String`, found `&str`".to_string(),
        None,
        "let x: String = \"hello\";".to_string(),
    );

    let fixes = generator.suggest_fixes(&error).await.unwrap();
    assert!(!fixes.is_empty(), "Should suggest fixes for type mismatch");
}

#[tokio::test]
async fn test_apply_fix_missing_import() {
    let generator = SelfHealingGenerator::new(3).unwrap();

    let code = "fn main() {\n    let map = HashMap::new();\n}";
    let fix = Fix::new(
        ErrorType::MissingImport,
        "Add HashMap import".to_string(),
        "use std::collections::HashMap;".to_string(),
        0.95,
    );

    let fixed_code = generator.apply_fix(code, &fix).await.unwrap();
    assert!(
        fixed_code.contains("use std::collections::HashMap;"),
        "Should add import"
    );
    assert!(
        fixed_code.contains("fn main()"),
        "Should preserve original code"
    );
}

#[tokio::test]
async fn test_health_metrics_initial() {
    let generator = SelfHealingGenerator::new(3).unwrap();
    let metrics = generator.get_health_metrics().await;

    assert_eq!(metrics.total_generations, 0);
    assert_eq!(metrics.successful_generations, 0);
    assert_eq!(metrics.total_repairs, 0);
    assert_eq!(metrics.generation_success_rate, 0.0);
    assert_eq!(metrics.average_repairs_per_generation, 0.0);
}

#[tokio::test]
async fn test_validation_result_success() {
    let result = ValidationResult::success(
        Duration::from_millis(100),
        "Compilation successful".to_string(),
    );

    assert!(result.passed, "Validation should pass");
    assert!(result.errors.is_empty(), "Should have no errors");
    assert_eq!(result.duration, Duration::from_millis(100));
}

#[tokio::test]
async fn test_validation_result_failure() {
    let errors = vec![CodeError::new(
        ErrorType::SyntaxError,
        "syntax error".to_string(),
        None,
        "test".to_string(),
    )];

    let result = ValidationResult::failure(
        errors.clone(),
        Duration::from_millis(50),
        "Compilation failed".to_string(),
    );

    assert!(!result.passed, "Validation should fail");
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_code_error_creation() {
    let error = CodeError::new(
        ErrorType::SyntaxError,
        "unexpected token".to_string(),
        None,
        "fn main() { }".to_string(),
    );

    assert_eq!(error.error_type, ErrorType::SyntaxError);
    assert_eq!(error.message, "unexpected token");
    assert!(error.location.is_none());
}

#[test]
fn test_fix_creation() {
    let fix = Fix::new(
        ErrorType::MissingImport,
        "Add import".to_string(),
        "use std::collections::HashMap;".to_string(),
        0.95,
    );

    assert_eq!(fix.error_type, ErrorType::MissingImport);
    assert_eq!(fix.confidence, 0.95);
    assert!(fix.code_replacement.contains("HashMap"));
}

#[test]
fn test_health_metrics_fields() {
    let metrics = HealthMetrics {
        generation_success_rate: 0.95,
        average_repairs_per_generation: 1.5,
        average_heal_time_ms: 250.0,
        confidence_score: 0.85,
        total_generations: 100,
        successful_generations: 95,
        total_repairs: 150,
    };

    assert_eq!(metrics.generation_success_rate, 0.95);
    assert_eq!(metrics.total_generations, 100);
    assert_eq!(metrics.successful_generations, 95);
    assert!(metrics.confidence_score > 0.8);
}

#[tokio::test]
async fn test_multiple_error_detection() {
    let output = r#"
error[E0425]: cannot find value `x` in this scope
error[E0308]: mismatched types
error[E0433]: unresolved import
"#;

    let errors = SelfHealingGenerator::detect_errors(output).unwrap();
    assert_eq!(errors.len(), 3, "Should detect all three errors");
}

#[tokio::test]
async fn test_fix_sorting_by_confidence() {
    let generator = SelfHealingGenerator::new(3).unwrap();

    let errors = vec![
        CodeError::new(
            ErrorType::MissingImport,
            "cannot find HashMap".to_string(),
            None,
            "".to_string(),
        ),
        CodeError::new(
            ErrorType::TypeMismatch,
            "expected String, found &str".to_string(),
            None,
            "".to_string(),
        ),
    ];

    let fixes = generator.suggest_fixes_for_errors(&errors).await.unwrap();

    // Fixes should be sorted by confidence (highest first)
    for i in 0..fixes.len().saturating_sub(1) {
        assert!(
            fixes[i].confidence >= fixes[i + 1].confidence,
            "Fixes should be sorted by confidence"
        );
    }
}

#[test]
fn test_error_type_equality() {
    assert_eq!(ErrorType::SyntaxError, ErrorType::SyntaxError);
    assert_ne!(ErrorType::SyntaxError, ErrorType::TypeMismatch);

    let compiler_error1 = ErrorType::CompilerError("E0425".to_string());
    let compiler_error2 = ErrorType::CompilerError("E0425".to_string());
    assert_eq!(compiler_error1, compiler_error2);
}
