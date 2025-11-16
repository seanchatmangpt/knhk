//! LaTeX Compiler Module
//!
//! Handles compilation of LaTeX documents to PDF using various engines.

use std::path::{Path, PathBuf};
use std::process::Command;

pub fn compile_latex(
    tex_path: &Path,
    output_dir: &Path,
    compiler: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Pre-process Mermaid diagrams if present
    let processed_tex = crate::mermaid::preprocess_mermaid(tex_path, output_dir)?;

    // Determine which compiler to use
    let compiler_type = if compiler == "auto" {
        detect_best_compiler()
    } else {
        compiler.to_string()
    };

    match compiler_type.as_str() {
        "tectonic" => compile_with_tectonic(&processed_tex, output_dir),
        "pdflatex" => compile_with_pdflatex(&processed_tex, output_dir),
        "xelatex" => compile_with_xelatex(&processed_tex, output_dir),
        _ => Err(format!("Unknown compiler: {}", compiler_type).into()),
    }
}

pub fn find_output_dir(source: &Path) -> PathBuf {
    source
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn list_available_compilers() -> Vec<String> {
    let mut compilers = Vec::new();

    if find_command("tectonic").is_some() {
        compilers.push("tectonic".to_string());
    }
    if find_command("pdflatex").is_some() {
        compilers.push("pdflatex".to_string());
    }
    if find_command("xelatex").is_some() {
        compilers.push("xelatex".to_string());
    }

    compilers
}

fn detect_best_compiler() -> String {
    // Prefer tectonic (Rust-based), then pdflatex, then xelatex
    if find_command("tectonic").is_some() {
        "tectonic".to_string()
    } else if find_command("pdflatex").is_some() {
        "pdflatex".to_string()
    } else if find_command("xelatex").is_some() {
        "xelatex".to_string()
    } else {
        "none".to_string()
    }
}

fn compile_with_tectonic(
    tex_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cmd = find_command("tectonic").ok_or("tectonic not found")?;

    let mut process = Command::new(cmd);
    process.arg("--outdir").arg(output_dir).arg(tex_path);

    let output = process.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Tectonic compilation failed: {}", stderr).into());
    }

    let pdf_name = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}.pdf", s))
        .ok_or("Invalid file name")?;

    Ok(output_dir.join(pdf_name))
}

fn compile_with_pdflatex(
    tex_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cmd = find_command("pdflatex").ok_or("pdflatex not found")?;

    let original_dir = std::env::current_dir()?;
    let tex_in_output = output_dir.join(tex_path.file_name().ok_or("LaTeX file has no filename")?);

    if tex_path != tex_in_output {
        std::fs::copy(tex_path, &tex_in_output)?;
    }

    std::env::set_current_dir(output_dir)?;

    let tex_file = tex_in_output
        .file_name()
        .ok_or("Output file has no filename")?;
    let pdf_name = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}.pdf", s))
        .ok_or("Invalid file name")?;
    let pdf_path = output_dir.join(&pdf_name);

    // Run pdflatex twice for references
    for _ in 0..2 {
        let mut process = Command::new(&cmd);
        process
            .arg("-interaction=nonstopmode")
            .arg("-output-directory")
            .arg(".")
            .arg(tex_file);

        let _output = process.output()?;
        // Don't check exit code - pdflatex can return non-zero for warnings
    }

    std::env::set_current_dir(original_dir)?;

    if pdf_path.exists() {
        Ok(pdf_path)
    } else {
        Err("PDF file was not created".into())
    }
}

fn compile_with_xelatex(
    tex_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cmd = find_command("xelatex").ok_or("xelatex not found")?;

    let original_dir = std::env::current_dir()?;
    let tex_in_output = output_dir.join(tex_path.file_name().ok_or("LaTeX file has no filename")?);

    if tex_path != tex_in_output {
        std::fs::copy(tex_path, &tex_in_output)?;
    }

    std::env::set_current_dir(output_dir)?;

    let tex_file = tex_in_output
        .file_name()
        .ok_or("Output file has no filename")?;
    let pdf_name = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}.pdf", s))
        .ok_or("Invalid file name")?;
    let pdf_path = output_dir.join(&pdf_name);

    // Run xelatex twice for references
    for _ in 0..2 {
        let mut process = Command::new(&cmd);
        process
            .arg("-interaction=nonstopmode")
            .arg("-output-directory")
            .arg(".")
            .arg(tex_file);

        let _output = process.output()?;
        // Don't check exit code - xelatex can return non-zero for warnings
    }

    std::env::set_current_dir(original_dir)?;

    if pdf_path.exists() {
        Ok(pdf_path)
    } else {
        Err("PDF file was not created".into())
    }
}

pub fn find_command(cmd: &str) -> Option<PathBuf> {
    // First try PATH
    if let Ok(path) = which::which(cmd) {
        return Some(path);
    }

    // Fallback to common macOS TeX location
    let tex_path = PathBuf::from("/Library/TeX/texbin").join(cmd);
    if tex_path.exists() {
        return Some(tex_path);
    }

    None
}
