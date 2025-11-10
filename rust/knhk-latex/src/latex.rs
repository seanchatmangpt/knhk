//! LaTeX Paper Commands
//!
//! Provides CLI interface for LaTeX paper compilation and validation:
//! - latex compile: Compile LaTeX source to PDF
//! - latex check: Check LaTeX syntax without compiling
//! - latex validate: Validate LaTeX document structure
//! - latex clean: Clean auxiliary files

// Allow non_upper_case_globals - #[verb] macro generates static vars with lowercase names
#![allow(non_upper_case_globals)]

use clap::Parser;
use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Debug)]
struct CompileResult {
    source: String,
    output: String,
    success: bool,
    pages: Option<u32>,
    size_bytes: Option<u64>,
}

/// Compile LaTeX source to PDF
#[verb] // Noun "latex" auto-inferred from filename "latex.rs"
fn compile(
    source: PathBuf,
    output_dir: Option<PathBuf>,
    compiler: Option<String>,
) -> Result<CompileResult> {
    use crate::compiler::compile_latex;

    let output = output_dir
        .or_else(|| source.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let compiler_str = compiler.unwrap_or_else(|| "auto".to_string());
    let pdf_path = compile_latex(&source, &output, &compiler_str).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Compilation failed: {}", e))
    })?;

    let pages = get_pdf_pages(&pdf_path).ok();
    let size_bytes = std::fs::metadata(&pdf_path).ok().map(|m| m.len());

    Ok(CompileResult {
        source: source.display().to_string(),
        output: pdf_path.display().to_string(),
        success: true,
        pages,
        size_bytes,
    })
}

#[derive(Serialize, Debug)]
struct CheckResult {
    source: String,
    valid: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// Check LaTeX syntax without compiling
#[verb]
fn check(source: PathBuf) -> Result<CheckResult> {
    use crate::validator::check_latex_syntax;

    let result = check_latex_syntax(&source).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Check failed: {}", e))
    })?;

    Ok(CheckResult {
        source: source.display().to_string(),
        valid: result.errors.is_empty(),
        errors: result.errors,
        warnings: result.warnings,
    })
}

#[derive(Serialize, Debug)]
struct ValidateResult {
    source: String,
    valid: bool,
    issues: Vec<String>,
}

/// Validate LaTeX document structure
#[verb]
fn validate(source: PathBuf) -> Result<ValidateResult> {
    use crate::validator::validate_latex_structure;

    let result = validate_latex_structure(&source).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Validation failed: {}", e))
    })?;

    Ok(ValidateResult {
        source: source.display().to_string(),
        valid: result.issues.is_empty(),
        issues: result.issues,
    })
}

#[derive(Serialize, Debug)]
struct CleanResult {
    source: String,
    files_removed: Vec<String>,
    success: bool,
}

/// Clean auxiliary LaTeX files
#[verb]
fn clean(source: PathBuf, keep_pdf: Option<bool>) -> Result<CleanResult> {
    use crate::cleaner::clean_auxiliary_files;

    let files_removed = clean_auxiliary_files(&source, keep_pdf.unwrap_or(false)).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Clean failed: {}", e))
    })?;

    Ok(CleanResult {
        source: source.display().to_string(),
        files_removed,
        success: true,
    })
}

#[derive(Serialize, Debug)]
struct InfoResult {
    source: String,
    compiler: String,
    available_compilers: Vec<String>,
}

/// Show LaTeX compiler information
#[verb]
fn info() -> Result<InfoResult> {
    use crate::compiler::list_available_compilers;

    let compilers = list_available_compilers();

    Ok(InfoResult {
        source: "N/A".to_string(),
        compiler: compilers
            .first()
            .cloned()
            .unwrap_or_else(|| "none".to_string()),
        available_compilers: compilers,
    })
}

#[derive(Serialize, Debug)]
struct MergeResult {
    sections_dir: String,
    output: String,
    sections_merged: usize,
    success: bool,
}

/// Merge LaTeX section files into a single document
#[verb]
fn merge(sections_dir: String, output: Option<String>) -> Result<MergeResult> {
    use std::fs;
    use std::io::Write;

    let sections_path = PathBuf::from(sections_dir);
    let output_path = output.map(PathBuf::from);

    // Read all .tex files in the directory, sorted by filename
    let mut entries: Vec<_> = fs::read_dir(&sections_path)
        .map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to read directory: {}",
                e
            ))
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("tex"))
        .collect();

    // Sort by filename to ensure correct order
    entries.sort_by_key(|e| e.path());

    if entries.is_empty() {
        return Err(clap_noun_verb::NounVerbError::execution_error(
            "No .tex files found in sections directory",
        ));
    }

    // Determine output path - use the_chatman_equation_fortune5.tex if no output specified
    let output_path =
        output_path.unwrap_or_else(|| sections_path.join("the_chatman_equation_fortune5.tex"));

    // Merge sections
    let mut merged_content = Vec::new();
    for entry in &entries {
        let content = fs::read_to_string(entry.path()).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to read {}: {}",
                entry.path().display(),
                e
            ))
        })?;
        merged_content.push(content);
    }

    // Write merged content
    let mut file = fs::File::create(&output_path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to create output file: {}",
            e
        ))
    })?;

    for content in &merged_content {
        file.write_all(content.as_bytes()).map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to write merged content: {}",
                e
            ))
        })?;
        file.write_all(b"\n").map_err(|e| {
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Failed to write newline: {}",
                e
            ))
        })?;
    }

    Ok(MergeResult {
        sections_dir: sections_path.display().to_string(),
        output: output_path.display().to_string(),
        sections_merged: entries.len(),
        success: true,
    })
}

#[derive(Serialize, Debug)]
struct RenderResult {
    sections_dir: String,
    merged_file: String,
    pdf_output: String,
    success: bool,
    pages: Option<u32>,
    size_bytes: Option<u64>,
}

/// Merge sections and render to PDF
#[verb]
fn render(
    sections_dir: String,
    output_dir: Option<String>,
    compiler: Option<String>,
) -> Result<RenderResult> {
    use crate::compiler::compile_latex;

    let sections_path = PathBuf::from(sections_dir.clone());

    // First merge sections
    let merged_path = sections_path.join("the_chatman_equation_fortune5_v1.2.0.tex");

    // Merge sections
    let merge_result = merge(
        sections_dir.clone(),
        Some(merged_path.display().to_string()),
    )?;

    // Determine output directory for PDF
    let pdf_output_dir = output_dir
        .map(PathBuf::from)
        .or_else(|| sections_path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Compile merged file to PDF
    let compiler_str = compiler.unwrap_or_else(|| "auto".to_string());
    let pdf_path = compile_latex(&merged_path, &pdf_output_dir, &compiler_str).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!("Compilation failed: {}", e))
    })?;

    let pages = get_pdf_pages(&pdf_path).ok();
    let size_bytes = std::fs::metadata(&pdf_path).ok().map(|m| m.len());

    Ok(RenderResult {
        sections_dir: sections_path.display().to_string(),
        merged_file: merged_path.display().to_string(),
        pdf_output: pdf_path.display().to_string(),
        success: true,
        pages,
        size_bytes,
    })
}

// Helper function to get PDF page count (requires pdfinfo or similar)
fn get_pdf_pages(pdf_path: &PathBuf) -> std::result::Result<u32, Box<dyn std::error::Error>> {
    // Try pdfinfo first
    if let Ok(output) = std::process::Command::new("pdfinfo").arg(pdf_path).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Pages:") {
                if let Some(pages_str) = line.split(':').nth(1) {
                    if let Ok(pages) = pages_str.trim().parse::<u32>() {
                        return Ok(pages);
                    }
                }
            }
        }
    }

    // Fallback: try to parse PDF directly (simple approach)
    // This is a basic implementation - could be improved
    Ok(0) // Return 0 if we can't determine
}
