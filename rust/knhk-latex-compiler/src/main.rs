use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get workspace root (parent of rust directory)
    let workspace_root = std::env::current_dir()?
        .parent()
        .ok_or("Could not find workspace root")?
        .to_path_buf();

    let paper_path = workspace_root.join("docs/papers/kgc-manifestation-fortune5.tex");
    let output_dir = workspace_root.join("docs/papers");

    println!("Compiling LaTeX paper to PDF...");
    println!("Source: {}", paper_path.display());
    println!("Output directory: {}", output_dir.display());

    // Check if source file exists
    if !paper_path.exists() {
        return Err(format!("LaTeX source file not found: {}", paper_path.display()).into());
    }

    // Ensure output directory exists
    std::fs::create_dir_all(&output_dir)?;

    // Try tectonic first (Rust-based LaTeX engine)
    println!("\nTrying Tectonic...");
    match compile_with_tectonic(&paper_path, &output_dir) {
        Ok(output) => {
            println!("✓ Successfully compiled with Tectonic");
            println!("PDF output: {}", output.display());
            return Ok(());
        }
        Err(e) => println!("  Tectonic failed: {}", e),
    }

    // Fallback to pdflatex
    println!("\nTrying pdflatex...");
    match compile_with_pdflatex(&paper_path, &output_dir) {
        Ok(output) => {
            println!("✓ Successfully compiled with pdflatex");
            println!("PDF output: {}", output.display());
            return Ok(());
        }
        Err(e) => println!("  pdflatex failed: {}", e),
    }

    // Fallback to xelatex
    println!("\nTrying xelatex...");
    match compile_with_xelatex(&paper_path, &output_dir) {
        Ok(output) => {
            println!("✓ Successfully compiled with xelatex");
            println!("PDF output: {}", output.display());
            return Ok(());
        }
        Err(e) => println!("  xelatex failed: {}", e),
    }

    println!("\n❌ All LaTeX compilers failed.");
    println!("\nPlease install one of:");
    println!("  - Tectonic: cargo install tectonic");
    println!("  - pdflatex: Install TeX Live or MacTeX");
    println!("  - xelatex: Install TeX Live or MacTeX");

    Err("Failed to compile LaTeX. No LaTeX compiler found.".into())
}

fn compile_with_tectonic(
    tex_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut cmd = Command::new("tectonic");
    cmd.arg("--outdir").arg(output_dir).arg(tex_path);

    let output = cmd.output()?;

    if !output.status.success() {
        io::stderr().write_all(&output.stderr)?;
        return Err("Tectonic compilation failed".into());
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
    // Try common pdflatex locations
    let pdflatex_cmd =
        find_command("pdflatex").ok_or("pdflatex not found in PATH or /Library/TeX/texbin")?;

    // Change to output directory for compilation
    let original_dir = std::env::current_dir()?;

    // Copy tex file to output directory if needed
    let tex_in_output = output_dir.join(
        tex_path
            .file_name()
            .ok_or("Invalid tex file path: no file name")?,
    );
    if tex_path != tex_in_output {
        std::fs::copy(tex_path, &tex_in_output)?;
    }

    std::env::set_current_dir(output_dir)?;

    let tex_file = tex_in_output
        .file_name()
        .ok_or("Invalid tex file path: no file name")?;
    let pdf_name = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}.pdf", s))
        .ok_or("Invalid file name")?;
    let pdf_path = output_dir.join(&pdf_name);

    // Run pdflatex twice for references (ignore exit code, check for PDF file)
    for _ in 0..2 {
        let mut cmd = Command::new(&pdflatex_cmd);
        cmd.arg("-interaction=nonstopmode")
            .arg("-output-directory")
            .arg(".")
            .arg(tex_file);

        let _output = cmd.output()?;
        // Don't check exit code - pdflatex can return non-zero for warnings
        // but still produce valid PDF
    }

    std::env::set_current_dir(original_dir)?;

    // Check if PDF was actually created
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
    // Try common xelatex locations
    let xelatex_cmd =
        find_command("xelatex").ok_or("xelatex not found in PATH or /Library/TeX/texbin")?;

    // Change to output directory for compilation
    let original_dir = std::env::current_dir()?;

    // Copy tex file to output directory if needed
    let tex_in_output = output_dir.join(
        tex_path
            .file_name()
            .ok_or("Invalid tex file path: no file name")?,
    );
    if tex_path != tex_in_output {
        std::fs::copy(tex_path, &tex_in_output)?;
    }

    std::env::set_current_dir(output_dir)?;

    let tex_file = tex_in_output
        .file_name()
        .ok_or("Invalid tex file path: no file name")?;
    let pdf_name = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| format!("{}.pdf", s))
        .ok_or("Invalid file name")?;
    let pdf_path = output_dir.join(&pdf_name);

    // Run xelatex twice for references (ignore exit code, check for PDF file)
    for _ in 0..2 {
        let mut cmd = Command::new(&xelatex_cmd);
        cmd.arg("-interaction=nonstopmode")
            .arg("-output-directory")
            .arg(".")
            .arg(tex_file);

        let _output = cmd.output()?;
        // Don't check exit code - xelatex can return non-zero for warnings
        // but still produce valid PDF
    }

    std::env::set_current_dir(original_dir)?;

    // Check if PDF was actually created
    if pdf_path.exists() {
        Ok(pdf_path)
    } else {
        Err("PDF file was not created".into())
    }
}

fn find_command(cmd: &str) -> Option<PathBuf> {
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
