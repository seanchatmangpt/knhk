//! Mermaid Diagram Pre-processing Module
//!
//! Handles conversion of Mermaid diagrams to SVG/PNG for LaTeX inclusion.
//! Supports both inline Mermaid blocks and standalone .mmd/.mermaid files.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use which;

/// Pre-process LaTeX file to convert Mermaid diagrams to images
pub fn preprocess_mermaid(
    tex_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Read the LaTeX file
    let content = fs::read_to_string(tex_path)?;

    // Check if Mermaid CLI is available
    let has_mermaid_cli = which::which("mmdc").is_ok() || which::which("npx").is_ok();

    if !has_mermaid_cli {
        eprintln!("Warning: Mermaid CLI not found. Mermaid diagrams will not be converted. Install with: npm install -g @mermaid-js/mermaid-cli");
        return Ok(tex_path.to_path_buf());
    }

    // Create output directory for Mermaid images
    // Images should be in the same directory as the compiled PDF for LaTeX to find them
    let mermaid_dir = output_dir.join("mermaid");
    fs::create_dir_all(&mermaid_dir)?;

    // Get source directory to scan for standalone .mmd/.mermaid files
    let source_dir = tex_path
        .parent()
        .ok_or("LaTeX file has no parent directory")?;

    // Process standalone Mermaid files in source directory
    process_standalone_mermaid_files(source_dir, &mermaid_dir)?;

    // Check if file contains Mermaid diagrams (inline or references)
    let has_inline_mermaid = content.contains("\\begin{mermaid}") || content.contains("\\mermaid{");
    let has_mermaid_refs = content.contains("\\includemermaid{")
        || content.contains("\\inputmermaid{")
        || content.contains("\\mermaidfile{");

    if !has_inline_mermaid && !has_mermaid_refs {
        // No Mermaid diagrams, return original path
        return Ok(tex_path.to_path_buf());
    }

    // Process Mermaid diagrams in LaTeX content
    // Use output_dir as the base for relative paths in LaTeX
    match process_mermaid_content(&content, &mermaid_dir, source_dir) {
        Ok(processed_content) => {
            // Write processed content to output directory (where PDF will be compiled)
            let file_name = tex_path.file_name().ok_or("LaTeX file has no filename")?;
            let output_tex = output_dir.join(file_name);
            fs::write(&output_tex, processed_content)?;
            Ok(output_tex)
        }
        Err(e) => {
            // Mermaid processing failed, return original path
            eprintln!(
                "Warning: Mermaid diagram conversion failed: {}. Using original LaTeX file.",
                e
            );
            Ok(tex_path.to_path_buf())
        }
    }
}

/// Process standalone .mmd and .mermaid files in source directory
fn process_standalone_mermaid_files(
    source_dir: &Path,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Scan for .mmd and .mermaid files
    let entries = fs::read_dir(source_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext == "mmd" || ext == "mermaid" {
                // Convert standalone Mermaid file
                let file_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Mermaid file has no valid stem")?;

                let content = fs::read_to_string(&path)?;
                let _ = convert_mermaid_to_image(&content, file_stem, output_dir);
                // Note: We don't fail if conversion fails - it will be handled when referenced
            }
        }
    }

    Ok(())
}

/// Process Mermaid diagram blocks and references in LaTeX content
fn process_mermaid_content(
    content: &str,
    mermaid_dir: &Path,
    source_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut processed = String::new();
    let mut in_mermaid = false;
    let mut mermaid_content = String::new();
    let mut diagram_index = 0;

    for line in content.lines() {
        // Handle inline Mermaid blocks: \begin{mermaid}...\end{mermaid}
        if line.trim().starts_with("\\begin{mermaid}") {
            in_mermaid = true;
            mermaid_content.clear();
            continue;
        }

        if in_mermaid {
            if line.trim().starts_with("\\end{mermaid}") {
                // Convert inline Mermaid to image
                let diagram_name = format!("mermaid_diagram_{}", diagram_index);
                match convert_mermaid_to_image(&mermaid_content, &diagram_name, mermaid_dir) {
                    Ok(image_path) => {
                        let image_include = format_mermaid_figure(&image_path, diagram_index);
                        processed.push_str(&image_include);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to convert inline Mermaid diagram {}: {}",
                            diagram_index, e
                        );
                        // Keep original block on error
                        processed.push_str("\\begin{mermaid}\n");
                        processed.push_str(&mermaid_content);
                        processed.push_str("\\end{mermaid}\n");
                    }
                }

                in_mermaid = false;
                mermaid_content.clear();
                diagram_index += 1;
                continue;
            }

            mermaid_content.push_str(line);
            mermaid_content.push('\n');
            continue;
        }

        // Handle \mermaid{...} inline syntax
        if line.contains("\\mermaid{") {
            let processed_line =
                process_mermaid_inline_command(line, &mut diagram_index, mermaid_dir)?;
            processed.push_str(&processed_line);
            processed.push('\n');
            continue;
        }

        // Handle external Mermaid file references: \includemermaid{file.mmd}
        if line.contains("\\includemermaid{")
            || line.contains("\\inputmermaid{")
            || line.contains("\\mermaidfile{")
        {
            let processed_line = process_mermaid_file_reference(line, source_dir, mermaid_dir)?;
            processed.push_str(&processed_line);
            processed.push('\n');
            continue;
        }

        processed.push_str(line);
        processed.push('\n');
    }

    Ok(processed)
}

/// Process \mermaid{...} inline command
fn process_mermaid_inline_command(
    line: &str,
    diagram_index: &mut usize,
    mermaid_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    // Extract content between \mermaid{ and }
    if let Some(start) = line.find("\\mermaid{") {
        if let Some(end) = line[start + 9..].find('}') {
            let mermaid_content = &line[start + 9..start + 9 + end];
            let diagram_name = format!("mermaid_inline_{}", diagram_index);

            match convert_mermaid_to_image(mermaid_content, &diagram_name, mermaid_dir) {
                Ok(image_path) => {
                    *diagram_index += 1;
                    let image_include = format_mermaid_figure(&image_path, *diagram_index - 1);
                    // Replace \mermaid{...} with figure
                    let mut result = line[..start].to_string();
                    result.push_str(&image_include);
                    if let Some(rest) = line.get(start + 9 + end + 1..) {
                        result.push_str(rest);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to convert inline Mermaid command: {}", e);
                    return Ok(line.to_string());
                }
            }
        }
    }

    Ok(line.to_string())
}

/// Process external Mermaid file reference: \includemermaid{file.mmd}
fn process_mermaid_file_reference(
    line: &str,
    source_dir: &Path,
    mermaid_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    // Extract filename from \includemermaid{file.mmd}, \inputmermaid{file.mmd}, or \mermaidfile{file.mmd}
    let commands = ["\\includemermaid{", "\\inputmermaid{", "\\mermaidfile{"];

    for cmd in &commands {
        if let Some(start) = line.find(cmd) {
            if let Some(end) = line[start + cmd.len()..].find('}') {
                let file_name = &line[start + cmd.len()..start + cmd.len() + end];
                let mermaid_file = source_dir.join(file_name);

                if !mermaid_file.exists() {
                    eprintln!(
                        "Warning: Mermaid file not found: {}",
                        mermaid_file.display()
                    );
                    return Ok(line.to_string());
                }

                // Read and convert Mermaid file
                let content = fs::read_to_string(&mermaid_file)?;
                let file_stem = mermaid_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("Mermaid file has no valid stem")?;

                match convert_mermaid_to_image(&content, file_stem, mermaid_dir) {
                    Ok(image_path) => {
                        let image_include = format_mermaid_figure(&image_path, 0);
                        // Replace command with figure
                        let mut result = line[..start].to_string();
                        result.push_str(&image_include);
                        if let Some(rest) = line.get(start + cmd.len() + end + 1..) {
                            result.push_str(rest);
                        }
                        return Ok(result);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to convert Mermaid file {}: {}",
                            file_name, e
                        );
                        return Ok(line.to_string());
                    }
                }
            }
        }
    }

    Ok(line.to_string())
}

/// Format LaTeX figure environment for Mermaid diagram
fn format_mermaid_figure(image_path: &Path, diagram_index: usize) -> String {
    let image_path_str = image_path.display().to_string();

    if image_path.extension().and_then(|s| s.to_str()) == Some("svg") {
        format!(
            "\\begin{{figure}}[h]\n\\centering\n\\includesvg[width=\\textwidth]{{{}}}\n\\caption{{Mermaid Diagram {}}}\n\\label{{fig:mermaid_{}}}\n\\end{{figure}}",
            image_path_str, diagram_index, diagram_index
        )
    } else {
        format!(
            "\\begin{{figure}}[h]\n\\centering\n\\includegraphics[width=\\textwidth]{{{}}}\n\\caption{{Mermaid Diagram {}}}\n\\label{{fig:mermaid_{}}}\n\\end{{figure}}",
            image_path_str, diagram_index, diagram_index
        )
    }
}

/// Convert Mermaid diagram to SVG/PNG image
fn convert_mermaid_to_image(
    mermaid_content: &str,
    diagram_name: &str,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Create temporary Mermaid file
    let mermaid_file = output_dir.join(format!("{}.mmd", diagram_name));
    fs::write(&mermaid_file, mermaid_content)?;

    // Try to find mmdc or use npx
    if let Ok(mmdc) = which::which("mmdc") {
        // Try SVG first (better quality, scalable)
        let svg_output = output_dir.join(format!("{}.svg", diagram_name));

        let mut cmd = Command::new(&mmdc);
        cmd.arg("-i")
            .arg(&mermaid_file)
            .arg("-o")
            .arg(&svg_output)
            .arg("-e")
            .arg("svg")
            .arg("-w")
            .arg("1200")
            .arg("-H")
            .arg("800");

        let output = cmd.output()?;

        if output.status.success() && svg_output.exists() {
            return Ok(PathBuf::from(format!("mermaid/{}.svg", diagram_name)));
        }

        // Try PNG as fallback
        let png_output = output_dir.join(format!("{}.png", diagram_name));
        let mut cmd = Command::new(&mmdc);
        cmd.arg("-i")
            .arg(&mermaid_file)
            .arg("-o")
            .arg(&png_output)
            .arg("-e")
            .arg("png")
            .arg("-w")
            .arg("1200")
            .arg("-H")
            .arg("800");

        let output = cmd.output()?;
        if output.status.success() && png_output.exists() {
            return Ok(PathBuf::from(format!("mermaid/{}.png", diagram_name)));
        }

        // If both failed, return error with details
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Mermaid conversion with mmdc failed. stderr: {}, stdout: {}",
            stderr, stdout
        )
        .into());
    }

    // Fallback to npx
    if which::which("npx").is_ok() {
        return convert_with_npx(mermaid_content, diagram_name, output_dir);
    }

    Err("Mermaid CLI (mmdc) not found. Install with: npm install -g @mermaid-js/mermaid-cli or ensure npx is available".into())
}

/// Convert Mermaid diagram using npx (fallback)
fn convert_with_npx(
    mermaid_content: &str,
    diagram_name: &str,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mermaid_file = output_dir.join(format!("{}.mmd", diagram_name));
    fs::write(&mermaid_file, mermaid_content)?;

    // Try SVG first (more reliable than PDF)
    let svg_output = output_dir.join(format!("{}.svg", diagram_name));

    let mut cmd = Command::new("npx");
    cmd.arg("-y")
        .arg("@mermaid-js/mermaid-cli")
        .arg("-i")
        .arg(&mermaid_file)
        .arg("-o")
        .arg(&svg_output)
        .arg("-e")
        .arg("svg")
        .arg("-w")
        .arg("1200")
        .arg("-H")
        .arg("800");

    let output = cmd.output()?;

    if output.status.success() && svg_output.exists() {
        return Ok(PathBuf::from(format!("mermaid/{}.svg", diagram_name)));
    }

    // Try PNG as fallback
    let png_output = output_dir.join(format!("{}.png", diagram_name));

    let mut cmd = Command::new("npx");
    cmd.arg("-y")
        .arg("@mermaid-js/mermaid-cli")
        .arg("-i")
        .arg(&mermaid_file)
        .arg("-o")
        .arg(&png_output)
        .arg("-e")
        .arg("png")
        .arg("-w")
        .arg("1200")
        .arg("-H")
        .arg("800");

    let output = cmd.output()?;

    if !output.status.success() || !png_output.exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Mermaid conversion with npx failed. stderr: {}, stdout: {}",
            stderr, stdout
        )
        .into());
    }

    Ok(PathBuf::from(format!("mermaid/{}.png", diagram_name)))
}
