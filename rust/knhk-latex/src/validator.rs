//! LaTeX Validator Module
//!
//! Validates LaTeX syntax and document structure.

use std::path::Path;
use std::process::Command;

pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub struct StructureValidationResult {
    pub issues: Vec<String>,
}

pub fn check_latex_syntax(tex_path: &Path) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    // Use chktex or lacheck if available, otherwise use pdflatex in check mode
    if let Ok(chktex) = which::which("chktex") {
        return check_with_chktex(tex_path, chktex);
    }

    // Fallback: use pdflatex in draft mode
    check_with_pdflatex(tex_path)
}

pub fn validate_latex_structure(
    tex_path: &Path,
) -> Result<StructureValidationResult, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(tex_path)?;
    let mut issues = Vec::new();

    // Check for required sections
    if !content.contains("\\documentclass") {
        issues.push("Missing \\documentclass".to_string());
    }

    if !content.contains("\\begin{document}") {
        issues.push("Missing \\begin{document}".to_string());
    }

    if !content.contains("\\end{document}") {
        issues.push("Missing \\end{document}".to_string());
    }

    // Check for common issues
    if content.matches("\\begin{").count() != content.matches("\\end{").count() {
        issues.push("Mismatched begin/end environments".to_string());
    }

    // Check for undefined references (basic check)
    if content.contains("\\ref{") && !content.contains("\\label{") {
        issues.push("Document contains \\ref{} but no \\label{} found".to_string());
    }

    Ok(StructureValidationResult { issues })
}

fn check_with_chktex(
    tex_path: &Path,
    chktex_cmd: std::path::PathBuf,
) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    let output = Command::new(chktex_cmd)
        .arg("-q")
        .arg("-n")
        .arg(tex_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for line in stdout.lines() {
        if line.contains("Error") {
            errors.push(line.to_string());
        } else {
            warnings.push(line.to_string());
        }
    }

    Ok(ValidationResult { errors, warnings })
}

fn check_with_pdflatex(tex_path: &Path) -> Result<ValidationResult, Box<dyn std::error::Error>> {
    // Use pdflatex in draft mode for syntax checking
    let cmd = crate::compiler::find_command("pdflatex").ok_or("pdflatex not found")?;

    let output = Command::new(cmd)
        .arg("-interaction=nonstopmode")
        .arg("-draftmode")
        .arg(tex_path)
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Parse LaTeX output for errors and warnings
    for line in stderr.lines().chain(stdout.lines()) {
        if line.contains("Error") || line.contains("!") {
            errors.push(line.to_string());
        } else if line.contains("Warning") || line.contains("LaTeX Warning") {
            warnings.push(line.to_string());
        }
    }

    Ok(ValidationResult { errors, warnings })
}
