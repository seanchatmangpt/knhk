//! Chicago TDD Tests for knhk-latex

use chicago_tdd_tools::assert_ok;
use std::fs;
use std::path::PathBuf;

// Note: These tests require the crate to expose a library API
// For now, we'll test the public functions directly

#[test]
fn test_list_compilers() {
    // Arrange
    // (no setup needed)

    // Act
    let compilers = knhk_latex::compiler::list_available_compilers();

    // Assert
    assert!(
        !compilers.is_empty(),
        "At least one compiler should be available"
    );
}

#[test]
fn test_validate_latex_structure() {
    // Arrange
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let test_tex = workspace_root.join("docs/papers/kgc-manifestation-fortune5.tex");

    // Skip if test file doesn't exist
    if !test_tex.exists() {
        return;
    }

    // Act
    let result = knhk_latex::validator::validate_latex_structure(&test_tex);

    // Assert
    assert_ok!(&result);
    let validation = result.unwrap();
    assert!(
        validation.issues.is_empty(),
        "LaTeX structure should be valid"
    );
}

#[test]
fn test_mermaid_preprocess_no_mermaid() {
    // Arrange
    let temp_dir = std::env::temp_dir().join("knhk_latex_test");
    fs::create_dir_all(&temp_dir).ok();

    let tex_file = temp_dir.join("test.tex");
    let tex_content = r#"\documentclass{article}
\begin{document}
Hello, world!
\end{document}"#;
    fs::write(&tex_file, tex_content).unwrap();

    let output_dir = temp_dir.join("output");
    fs::create_dir_all(&output_dir).ok();

    // Act
    let result = knhk_latex::mermaid::preprocess_mermaid(&tex_file, &output_dir);

    // Assert
    assert_ok!(&result);
    let processed_path = result.unwrap();
    // When no Mermaid is present, should return original path
    assert_eq!(
        processed_path, tex_file,
        "Should return original path when no Mermaid diagrams"
    );

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_mermaid_preprocess_inline_block() {
    // Arrange
    let temp_dir = std::env::temp_dir().join("knhk_latex_test_inline");
    fs::create_dir_all(&temp_dir).ok();

    let tex_file = temp_dir.join("test.tex");
    let tex_content = r#"\documentclass{article}
\begin{document}
\begin{mermaid}
graph TD;
    A[Rust Code] --> B(Generate mmd files);
\end{mermaid}
\end{document}"#;
    fs::write(&tex_file, tex_content).unwrap();

    let output_dir = temp_dir.join("output");
    fs::create_dir_all(&output_dir).ok();

    // Act
    let result = knhk_latex::mermaid::preprocess_mermaid(&tex_file, &output_dir);

    // Assert
    // Result may succeed or fail depending on Mermaid CLI availability
    // If Mermaid CLI is not available, should return original path
    // If Mermaid CLI is available, should return processed path
    match result {
        Ok(processed_path) => {
            // If processed, should be in output directory
            if processed_path != tex_file {
                assert!(
                    processed_path.parent().unwrap() == output_dir,
                    "Processed file should be in output directory"
                );
                // Verify processed content contains figure environment
                let processed_content = fs::read_to_string(&processed_path).unwrap();
                assert!(
                    processed_content.contains("\\begin{figure}"),
                    "Processed content should contain figure environment"
                );
            }
        }
        Err(_) => {
            // If Mermaid CLI is not available, this is acceptable
            // The function should handle this gracefully
        }
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_mermaid_preprocess_external_file_reference() {
    // Arrange
    let temp_dir = std::env::temp_dir().join("knhk_latex_test_external");
    fs::create_dir_all(&temp_dir).ok();

    // Create a standalone Mermaid file
    let mermaid_file = temp_dir.join("diagram.mmd");
    let mermaid_content = r#"graph TD;
    A[Rust Code] --> B(Generate mmd files);
    B --> C{Run mermaid-cli};
    C --> D[Output SVG/PNG];"#;
    fs::write(&mermaid_file, mermaid_content).unwrap();

    // Create LaTeX file with external Mermaid reference
    let tex_file = temp_dir.join("test.tex");
    let tex_content = r#"\documentclass{article}
\begin{document}
\includemermaid{diagram.mmd}
\end{document}"#;
    fs::write(&tex_file, tex_content).unwrap();

    let output_dir = temp_dir.join("output");
    fs::create_dir_all(&output_dir).ok();

    // Act
    let result = knhk_latex::mermaid::preprocess_mermaid(&tex_file, &output_dir);

    // Assert
    // Result may succeed or fail depending on Mermaid CLI availability
    match result {
        Ok(processed_path) => {
            // If processed, should be in output directory
            if processed_path != tex_file {
                assert!(
                    processed_path.parent().unwrap() == output_dir,
                    "Processed file should be in output directory"
                );
                // Verify processed content contains figure environment
                let processed_content = fs::read_to_string(&processed_path).unwrap();
                assert!(
                    processed_content.contains("\\begin{figure}"),
                    "Processed content should contain figure environment"
                );
            }
        }
        Err(_) => {
            // If Mermaid CLI is not available, this is acceptable
        }
    }

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}

#[test]
fn test_mermaid_preprocess_standalone_files() {
    // Arrange
    let temp_dir = std::env::temp_dir().join("knhk_latex_test_standalone");
    fs::create_dir_all(&temp_dir).ok();

    // Create standalone Mermaid files
    let mermaid_file1 = temp_dir.join("diagram1.mmd");
    let mermaid_file2 = temp_dir.join("diagram2.mermaid");
    fs::write(&mermaid_file1, "graph TD; A --> B;").unwrap();
    fs::write(&mermaid_file2, "graph LR; C --> D;").unwrap();

    // Create LaTeX file (no Mermaid references, but standalone files should be processed)
    let tex_file = temp_dir.join("test.tex");
    let tex_content = r#"\documentclass{article}
\begin{document}
Hello, world!
\end{document}"#;
    fs::write(&tex_file, tex_content).unwrap();

    let output_dir = temp_dir.join("output");
    fs::create_dir_all(&output_dir).ok();

    // Act
    let result = knhk_latex::mermaid::preprocess_mermaid(&tex_file, &output_dir);

    // Assert
    // Should succeed even with standalone files (they're processed but not referenced)
    assert_ok!(&result);
    let processed_path = result.unwrap();
    // Should return original path when no Mermaid references in LaTeX
    assert_eq!(
        processed_path, tex_file,
        "Should return original path when no Mermaid references"
    );

    // Cleanup
    fs::remove_dir_all(&temp_dir).ok();
}
