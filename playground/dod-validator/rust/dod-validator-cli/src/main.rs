//! dod-validator-cli: CLI tool for DoD validation
//! 
//! Provides command-line interface for validating code against
//! Definition of Done criteria using KNHK's 2ns capabilities.

use clap::{Parser, Subcommand};
use dod_validator_core::{ValidationEngine, ValidationCategory, ValidationReport};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dod-validator")]
#[command(about = "KNHK Definition of Done Validator using 2ns pattern matching")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a single file or directory
    Validate {
        /// Path to file or directory to validate
        path: PathBuf,
        
        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Validate against specific DoD category
    Category {
        /// Category to validate
        category: String,
        
        /// Path to file or directory
        path: PathBuf,
    },
    
    /// Show validation report
    Report {
        /// Path to validation report file
        report_path: PathBuf,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path, format } => {
            let mut engine = ValidationEngine::new()?;
            let report = engine.validate_all(&path)?;

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&report)
                        .map_err(|e| format!("Failed to serialize report: {}", e))?;
                    println!("{}", json);
                }
                "text" => {
                    print_report_text(&report);
                }
                _ => {
                    return Err(format!("Unknown format: {}", format));
                }
            }

            if !report.is_success() {
                std::process::exit(1);
            }
        }
        Commands::Category { category, path } => {
            let mut engine = ValidationEngine::new()?;
            let report = engine.validate_all(&path)?;

            let category_enum = match category.as_str() {
                "code-quality" => ValidationCategory::CodeQuality,
                "performance" => ValidationCategory::Performance,
                "testing" => ValidationCategory::Testing,
                "documentation" => ValidationCategory::Documentation,
                "integration" => ValidationCategory::Integration,
                "error-handling" => ValidationCategory::ErrorHandling,
                "guard-constraints" => ValidationCategory::GuardConstraints,
                _ => {
                    return Err(format!("Unknown category: {}", category));
                }
            };

            if let Some(results) = report.category_results.get(&category_enum) {
                println!("Category: {:?}", category_enum);
                for result in results {
                    println!("  {}: {}", 
                        if result.passed { "✓" } else { "✗" },
                        result.message
                    );
                }
            } else {
                println!("No results for category: {}", category);
            }
        }
        Commands::Report { report_path } => {
            let content = std::fs::read_to_string(&report_path)
                .map_err(|e| format!("Failed to read report: {}", e))?;
            let report: ValidationReport = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse report: {}", e))?;
            print_report_text(&report);
        }
    }

    Ok(())
}

fn print_report_text(report: &ValidationReport) {
    println!("KNHK DoD Validator Report");
    println!("=========================");
    println!("Total checks: {}", report.total);
    println!("Passed: {}", report.passed);
    println!("Failed: {}", report.failed);
    println!("Warnings: {}", report.warnings);
    println!("Duration: {}ms", report.duration_ms);
    println!();

    for (category, results) in &report.category_results {
        println!("Category: {:?}", category);
        for result in results {
            println!("  {}: {}", 
                if result.passed { "✓" } else { "✗" },
                result.message
            );
            if let Some(line) = result.line {
                println!("    Line: {}", line);
            }
            if let Some(span_id) = result.span_id {
                println!("    Span ID: 0x{:x}", span_id);
            }
            if let Some(duration) = result.duration_ns {
                println!("    Duration: {}ns", duration);
            }
        }
        println!();
    }
}

