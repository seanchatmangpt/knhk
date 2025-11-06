//! Advanced Pattern Detection Example
//!
//! Demonstrates advanced pattern detection:
//! - Closures with unwrap (`|x| x.unwrap()`)
//! - Macro definitions (`macro_rules!`)
//! - Async/await patterns (`.await.unwrap()`)

use dod_validator_core::ValidationEngine;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    println!("DoD Validator - Advanced Pattern Detection Example");
    println!("==================================================\n");

    let mut engine = ValidationEngine::new()
        .map_err(|e| format!("Failed to create validation engine: {}", e))?;

    let test_dir = std::env::temp_dir().join("dod_validator_advanced");
    std::fs::create_dir_all(&test_dir)
        .map_err(|e| format!("Failed to create test directory: {}", e))?;

    let test_file = test_dir.join("advanced.rs");
    let test_code = r#"
// Advanced patterns example

// Closure with unwrap
fn process_items(items: Vec<Option<i32>>) {
    let values: Vec<i32> = items
        .into_iter()
        .map(|x| x.unwrap()) // Violation: unwrap() in closure
        .collect();
}

// Macro definition
macro_rules! my_macro {
    // Violation: macro definition (placeholder)
}

// Async/await with unwrap
async fn fetch_data() -> i32 {
    let result = tokio::task::spawn(async {
        Ok(42)
    }).await.unwrap().unwrap(); // Violation: .await.unwrap()
    result
}
"#;

    std::fs::write(&test_file, test_code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;

    println!("Validating advanced patterns in: {}", test_file.display());
    println!();

    let report = engine.validate_all(&test_file)?;

    println!("Advanced Pattern Detection Results:");
    println!("  Total checks: {}", report.total);
    println!("  Violations found: {}", report.failed);
    println!();

    // Show advanced pattern violations
    for result in &report.results {
        if !result.passed {
            println!("  ✗ {}", result.message);
            if let Some(line) = result.line {
                println!("    Line: {}", line);
            }
            if let Some(ref snippet) = result.code_snippet {
                println!("    Code: {}", snippet);
            }
            if let Some(ref context) = result.context_lines {
                if !context.is_empty() {
                    println!("    Context:");
                    for (idx, ctx_line) in context.iter().enumerate() {
                        let line_num = result.line.unwrap_or(0).saturating_sub(3) + idx as u32;
                        println!("      {:4}| {}", line_num, ctx_line);
                    }
                }
            }
            println!();
        }
    }

    std::fs::remove_file(&test_file).ok();
    std::fs::remove_dir_all(&test_dir).ok();

    Ok(())
}

