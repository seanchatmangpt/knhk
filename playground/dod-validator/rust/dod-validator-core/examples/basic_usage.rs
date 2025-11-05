// Example usage of KNHK DoD Validator

use dod_validator_core::{ValidationEngine, ValidationCategory};

fn main() -> Result<(), String> {
    // Create validation engine
    let mut engine = ValidationEngine::new()?;

    // Validate a code file
    let code_path = std::path::PathBuf::from("example.rs");
    let report = engine.validate_all(&code_path)?;

    // Check results
    if report.is_success() {
        println!("✓ All validations passed!");
    } else {
        println!("✗ Some validations failed:");
        for result in &report.results {
            if !result.passed {
                println!("  - {}", result.message);
            }
        }
    }

    // Check specific category
    if let Some(code_quality_results) = report.category_results.get(&ValidationCategory::CodeQuality) {
        println!("\nCode Quality Results:");
        for result in code_quality_results {
            println!("  {}: {}", 
                if result.passed { "✓" } else { "✗" },
                result.message
            );
        }
    }

    Ok(())
}

