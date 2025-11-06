//! Basic DoD Validation Example
//!
//! Demonstrates basic pattern detection for common violations:
//! - `.unwrap()` patterns
//! - `.expect()` patterns
//! - TODO comments
//! - Placeholder text

use dod_validator_core::ValidationEngine;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    println!("DoD Validator - Basic Example");
    println!("=============================\n");

    // Create validation engine
    let mut engine = ValidationEngine::new()
        .map_err(|e| format!("Failed to create validation engine: {}", e))?;

    // Create a temporary test file with violations
    let test_dir = std::env::temp_dir().join("dod_validator_example");
    std::fs::create_dir_all(&test_dir)
        .map_err(|e| format!("Failed to create test directory: {}", e))?;

    let test_file = test_dir.join("example.rs");
    let test_code = r#"
fn main() {
    let x: Option<i32> = Some(42);
    let value = x.unwrap(); // Violation: unwrap()
    println!("{}", value);
    
    let y: Result<i32, String> = Ok(42);
    let result = y.expect("error"); // Violation: expect()
    
    // TODO: Add error handling // Violation: TODO comment
    
    let placeholder = "placeholder"; // Violation: placeholder text
}
"#;

    std::fs::write(&test_file, test_code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;

    println!("Validating test file: {}", test_file.display());
    println!();

    // Validate the file
    let report = engine.validate_all(&test_file)?;

    // Display results
    println!("Validation Results:");
    println!("  Total checks: {}", report.total);
    println!("  Passed: {}", report.passed);
    println!("  Failed: {}", report.failed);
    println!();

    // Show violations
    if !report.results.is_empty() {
        println!("Violations:");
        for result in &report.results {
            if !result.passed {
                println!("\n  ✗ {}", result.message);
                if let Some(ref file) = result.file {
                    println!("    File: {}", file.display());
                }
                if let Some(line) = result.line {
                    if let Some(col) = result.column {
                        println!("    Location: Line {}, Column {}", line, col);
                    } else {
                        println!("    Line: {}", line);
                    }
                }
                if let Some(ref snippet) = result.code_snippet {
                    println!("    Code: {}", snippet);
                }
            }
        }
    }

    // Cleanup
    std::fs::remove_file(&test_file).ok();
    std::fs::remove_dir_all(&test_dir).ok();

    if report.is_success() {
        println!("\n✅ All checks passed!");
        Ok(())
    } else {
        println!("\n❌ Validation failed with {} violation(s)", report.failed);
        Err("Validation failed".to_string())
    }
}

