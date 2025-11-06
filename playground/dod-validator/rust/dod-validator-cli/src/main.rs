//! dod-validator-cli: CLI tool for DoD validation
//! 
//! Provides command-line interface for validating code against
//! Definition of Done criteria using KNHK's 2ns capabilities.

use clap::{Parser, Subcommand};
use dod_validator_core::{ValidationEngine, ValidationCategory, ValidationReport, reporting::generate_html_report};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "dod-validator")]
#[command(about = "KNHK Definition of Done Validator using 2ns pattern matching")]
#[command(long_about = r#"
KNHK DoD Validator - Ultra-fast code quality validation

The DoD Validator leverages KNHK's ≤2ns hot path capabilities to validate
code against Definition of Done criteria at unprecedented speeds.

Examples:
  # Validate a single file
  dod-validator validate src/main.rs

  # Validate a directory with JSON output
  dod-validator validate src/ --format json

  # Validate specific category
  dod-validator category code-quality src/

  # View saved report
  dod-validator report validation_report.json

Categories:
  code-quality      - Pattern matching (unwrap, expect, TODO, etc.)
  performance       - Hot path timing validation
  testing           - Test coverage analysis
  documentation     - Documentation completeness checks
  integration       - FFI, ETL, lockchain integration validation
  error-handling    - Result<T, E> pattern validation
  guard-constraints - max_run_len ≤ 8 enforcement
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a single file or directory
    ///
    /// Validates code against all Definition of Done criteria.
    ///
    /// Examples:
    ///   # Validate a file
    ///   dod-validator validate src/main.rs
    ///
    ///   # Validate directory with JSON output
    ///   dod-validator validate src/ --format json > report.json
    ///
    /// Output formats:
    ///   - text: Human-readable text output (default)
    ///   - json: Machine-readable JSON output
    ///   - html: Styled HTML report with syntax highlighting
    Validate {
        /// Path to file or directory to validate
        path: PathBuf,
        
        /// Output format (json, text, html)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Validate against specific DoD category
    ///
    /// Validates code against a single category.
    ///
    /// Examples:
    ///   # Validate code quality only
    ///   dod-validator category code-quality src/
    ///
    ///   # Validate guard constraints
    ///   dod-validator category guard-constraints src/
    ///
    /// Available categories:
    ///   - code-quality: Pattern matching (unwrap, expect, TODO, etc.)
    ///   - performance: Hot path timing validation
    ///   - testing: Test coverage analysis
    ///   - documentation: Documentation completeness checks
    ///   - integration: FFI, ETL, lockchain integration validation
    ///   - error-handling: Result<T, E> pattern validation
    ///   - guard-constraints: max_run_len ≤ 8 enforcement
    Category {
        /// Category to validate
        category: String,
        
        /// Path to file or directory
        path: PathBuf,
    },
    
    /// Show validation report from saved JSON file
    ///
    /// Displays a previously saved validation report in text format.
    ///
    /// Example:
    ///   dod-validator report validation_report.json
    Report {
        /// Path to validation report file (JSON format)
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
                "html" => {
                    let html = generate_html_report(&report);
                    println!("{}", html);
                }
                _ => {
                    return Err(format!("Unknown format: {}. Supported formats: json, text, html", format));
                }
            }

            if !report.is_success() {
                // Exit with code 1 if validation failed
                std::process::exit(1);
            }
            // Exit with code 0 if validation passed
            std::process::exit(0);
        }
        Commands::Category { category, path } => {
            let mut engine = ValidationEngine::new()
                .map_err(|e| format!("Failed to initialize validation engine: {}", e))?;
            let report = engine.validate_all(&path)
                .map_err(|e| format!("Validation failed: {}", e))?;

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
                // Exit with code 1 if any violations in category
                let has_violations = results.iter().any(|r| !r.passed);
                std::process::exit(if has_violations { 1 } else { 0 });
            } else {
                println!("No results for category: {}", category);
                std::process::exit(0);
            }
        }
        Commands::Report { report_path } => {
            let content = std::fs::read_to_string(&report_path)
                .map_err(|e| format!("Failed to read report file {}: {}. Check that the file exists and you have read permissions.", 
                    report_path.display(), e))?;
            let report: ValidationReport = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse report file {}: {}. Ensure the file is valid JSON.", 
                    report_path.display(), e))?;
            print_report_text(&report);
            // Exit with code 1 if report shows failures
            std::process::exit(if report.is_success() { 0 } else { 1 });
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
        if results.is_empty() {
            continue;
        }
        
        println!("Category: {:?}", category);
        for result in results {
            let status = if result.passed { "✓" } else { "✗" };
            println!("  {}: {}", status, result.message);
            
            // Display file path
            if let Some(ref file) = result.file {
                println!("    File: {}", file.display());
            }
            
            // Display line and column
            if let Some(line) = result.line {
                if let Some(col) = result.column {
                    println!("    Location: Line {}, Column {}", line, col);
                } else {
                    println!("    Line: {}", line);
                }
            }
            
            // Display code snippet
            if let Some(ref snippet) = result.code_snippet {
                println!("    Code:");
                println!("      {}", snippet);
            }
            
            // Display context lines
            if let Some(ref context) = result.context_lines {
                if !context.is_empty() {
                    println!("    Context:");
                    for (idx, context_line) in context.iter().enumerate() {
                        // Find the violation line (usually middle of context)
                        let line_num = if let Some(line) = result.line {
                            let context_start = line.saturating_sub(3);
                            context_start + idx as u32
                        } else {
                            idx as u32 + 1
                        };
                        
                        // Mark violation line with arrow
                        if let Some(ref snippet) = result.code_snippet {
                            if context_line.trim() == snippet.trim() {
                                println!("     {:4}| {}", line_num, context_line);
                                println!("         ^ Violation here");
                            } else {
                                println!("     {:4}| {}", line_num, context_line);
                            }
                        } else {
                            println!("     {:4}| {}", line_num, context_line);
                        }
                    }
                }
            }
            
            // Display span ID and duration
            if let Some(span_id) = result.span_id {
                println!("    Span ID: 0x{:x}", span_id);
            }
            if let Some(duration) = result.duration_ns {
                println!("    Duration: {}ns", duration);
            }
            
            println!();
        }
    }
}

