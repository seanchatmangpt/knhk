//! Autonomous Validation Example
//!
//! Demonstrates autonomous self-healing validation:
//! - Observe → Reflect → Act → Verify cycle
//! - Automatic fix generation
//! - Receipt validation

use dod_validator_autonomous::AutonomousValidator;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    println!("DoD Validator - Autonomous Example");
    println!("==================================\n");

    let test_dir = std::env::temp_dir().join("dod_validator_autonomous");
    std::fs::create_dir_all(&test_dir)
        .map_err(|e| format!("Failed to create test directory: {}", e))?;

    let test_file = test_dir.join("autonomous.rs");
    let test_code = r#"
fn main() {
    let x: Option<i32> = Some(42);
    let value = x.unwrap(); // Violation: unwrap()
    println!("{}", value);
}
"#;

    std::fs::write(&test_file, test_code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;

    println!("Creating autonomous validator...");
    let mut validator = AutonomousValidator::new(test_dir.clone())
        .map_err(|e| format!("Failed to create autonomous validator: {}", e))?;

    println!("Observing violations...");
    let observation = validator.observe_path(&test_file)?;
    
    println!("  Found {} violation(s)", observation.violations.len());
    for violation in &observation.violations {
        println!("    - {:?} at {}:{}", 
            violation.pattern,
            violation.file.display(),
            violation.line
        );
    }

    if !observation.violations.is_empty() {
        println!("\nReflecting on fixes...");
        let action = validator.reflect(&observation)?;
        
        println!("  Generated {} fix(es)", action.fixes.len());
        for fix in &action.fixes {
            println!("    Fix for {:?}:", fix.violation.pattern);
            println!("      Before: {}", fix.code_before);
            println!("      After: {}", fix.code_after);
        }

        println!("\nApplying fixes...");
        let receipts = validator.act(&action)?;
        
        println!("  Applied {} fix(es)", receipts.len());
        for receipt in &receipts {
            println!("    Receipt: span_id=0x{:x}, hash=0x{:x}", 
                receipt.span_id,
                receipt.action_hash
            );
        }

        println!("\nVerifying fixes...");
        let verify_result = validator.verify(&receipts, &test_file);
        match verify_result {
            Ok(_) => println!("  ✅ All fixes verified successfully"),
            Err(e) => println!("  ⚠️  Verification warning: {}", e),
        }

        // Show fixed code
        let fixed_code = std::fs::read_to_string(&test_file)
            .map_err(|e| format!("Failed to read fixed file: {}", e))?;
        println!("\nFixed code:");
        println!("{}", fixed_code);
    }

    std::fs::remove_file(&test_file).ok();
    std::fs::remove_dir_all(&test_dir).ok();

    println!("\n✅ Autonomous validation complete!");
    Ok(())
}

