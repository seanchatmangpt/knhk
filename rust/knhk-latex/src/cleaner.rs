//! LaTeX Cleaner Module
//!
//! Cleans auxiliary files generated during LaTeX compilation.

use std::fs;
use std::path::Path;

pub fn clean_auxiliary_files(
    tex_path: &Path,
    keep_pdf: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let base_name = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?;

    let dir = tex_path.parent().unwrap_or_else(|| Path::new("."));

    let mut files_removed = Vec::new();

    // List of auxiliary file extensions
    let aux_extensions = vec![
        "aux",
        "log",
        "out",
        "toc",
        "lof",
        "lot",
        "fls",
        "fdb_latexmk",
        "synctex.gz",
        "bbl",
        "blg",
        "idx",
        "ilg",
        "ind",
        "nav",
        "snm",
        "vrb",
        "snm",
    ];

    for ext in aux_extensions {
        let aux_file = dir.join(format!("{}.{}", base_name, ext));
        if aux_file.exists() {
            fs::remove_file(&aux_file)?;
            files_removed.push(aux_file.display().to_string());
        }
    }

    // Remove PDF if requested
    if !keep_pdf {
        let pdf_file = dir.join(format!("{}.pdf", base_name));
        if pdf_file.exists() {
            fs::remove_file(&pdf_file)?;
            files_removed.push(pdf_file.display().to_string());
        }
    }

    Ok(files_removed)
}
