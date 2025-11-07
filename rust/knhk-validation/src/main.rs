// knhk-validation
// Release validation binary for v0.4.0

use knhk_validation::*;

#[cfg(feature = "policy-engine")]
use knhk_validation::policy_engine::PolicyEngine;

#[cfg(feature = "diagnostics")]
use knhk_validation::diagnostics::{Diagnostics, DiagnosticMessage, Severity};

#[cfg(feature = "std")]
fn main() {
    let mut report = ValidationReport::new();

    println!("KNHKS v0.4.0 Validation");
    println!("========================");
    println!();

    // CLI validation
    println!("CLI Validation:");
    report.add_result(cli_validation::validate_cli_binary_exists());
    report.add_result(cli_validation::validate_cli_command("--help", &[]));
    report.add_result(cli_validation::validate_cli_command("hook", &["--help"]));
    report.add_result(cli_validation::validate_cli_command("connect", &["--help"]));
    report.add_result(cli_validation::validate_cli_command("receipt", &["--help"]));

    // Network validation
    #[cfg(feature = "test-deps")]
    {
        println!("Network Validation:");
        report.add_result(network_validation::validate_http_client_exists());
        report.add_result(network_validation::validate_otel_exporter_exists());
    }

    // Configuration validation
    println!("Configuration Validation:");
    report.add_result(configuration_validation::validate_config_file_parsing());

    // Property validation
    println!("Property Validation:");
    report.add_result(property_validation::validate_receipt_merging_properties());
    report.add_result(property_validation::validate_iri_hashing_properties());
    report.add_result(property_validation::validate_guard_constraints());
    
    #[cfg(feature = "policy-engine")]
    {
        // Use policy engine for guard constraint validation
        let policy_engine = PolicyEngine::new();
        let guard_result = guard_validation::validate_guard_constraint(8);
        report.add_result(guard_result);
        
        let guard_violation = guard_validation::validate_guard_constraint(9);
        report.add_result(guard_violation);
    }

    // Performance validation
    println!("Performance Validation:");
    // Note: performance_validation module pending implementation
    // #[cfg(feature = "test-deps")]
    // {
    //     report.add_result(performance_validation::validate_hot_path_performance());
    // }
    // report.add_result(performance_validation::validate_cli_latency());
    
    #[cfg(all(feature = "policy-engine", feature = "diagnostics"))]
    {
        // Use policy engine with diagnostics for performance validation
        let (perf_result, diag) = performance_validation::validate_hot_path_performance_with_diagnostics(8);
        report.add_result(perf_result);
        
        let (perf_violation, violation_diag) = performance_validation::validate_hot_path_performance_with_diagnostics(9);
        report.add_result(perf_violation);
        
        if let Some(diag) = violation_diag {
            println!("Performance violation diagnostic: {}", diag.format_ansi());
        }
    }

    // Print results
    println!();
    println!("Summary:");
    println!("  Total: {}", report.total);
    println!("  Passed: {}", report.passed);
    println!("  Failed: {}", report.failed);
    println!("  Warnings: {}", report.warnings);
    println!();
    
    #[cfg(feature = "diagnostics")]
    {
        // Collect diagnostics from validation results
        let mut diags = Diagnostics::new();
        for result in &report.results {
            if !result.passed {
                let msg = DiagnosticMessage::new("VALIDATION_FAILED", result.message.clone())
                    .with_severity(Severity::Error);
                diags.add(msg);
            }
        }

        if diags.has_errors() {
            println!("Diagnostics:");
            let formatted = knhk_validation::diagnostics::format_diagnostics(&diags);
            println!("{}", formatted);

            // Also output JSON for CI/CD
            if let Ok(json) = knhk_validation::diagnostics::format_diagnostics_json(&diags) {
                println!("\nJSON Output:");
                println!("{}", json);
            }
        }
    }

    for result in &report.results {
        if result.passed {
            println!("  ✓ {}", result.message);
        } else {
            println!("  ✗ {}", result.message);
        }
    }

    println!();

    if report.is_success() {
        println!("All validation checks passed!");
        std::process::exit(0);
    } else {
        println!("Some validation checks failed!");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "std"))]
fn main() {
    println!("Validation requires std feature");
}

