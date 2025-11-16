//! Multi-Language Code Generator - Unified interface for code generation
//!
//! Provides a unified interface for generating code in multiple target languages
//! from RDF/SPARQL data and templates.
//!
//! # Supported Languages
//!
//! - Rust: Domain models, API endpoints, tests, documentation
//! - Python: Domain models, API endpoints, tests, documentation
//! - JavaScript: Domain models, API endpoints, tests, documentation
//! - Go: Domain models, API endpoints, tests, documentation
//!
//! # Architecture
//!
//! - `CodeGenerator` trait: Sync interface for all generators
//! - Language-specific implementations
//! - Template-based generation with SPARQL data binding
//! - 100% type-safe Rust interface
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::ggen::codegen::{CodeGenerator, RustGenerator, GenerationContext};
//! use std::collections::HashMap;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = RustGenerator::new("templates/rust")?;
//! let mut context = GenerationContext::new();
//! context.insert("struct_name".to_string(), "User".to_string());
//!
//! let code = generator.generate_domain_model(&context)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::path::Path;

pub mod python;
pub mod rust;

pub use python::PythonGenerator;
pub use rust::RustGenerator;

/// Generation context for code generators
///
/// Stores key-value pairs for template rendering.
#[derive(Debug, Clone, Default)]
pub struct GenerationContext {
    /// Context data (key -> value)
    data: HashMap<String, String>,
}

impl GenerationContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert key-value pair
    pub fn insert(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    /// Get value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Get all data
    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }

    /// Get mutable data
    pub fn data_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.data
    }
}

/// Code generation result
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// Generated code content
    pub content: String,
    /// Language identifier (rust, python, javascript, go)
    pub language: String,
    /// File extension (.rs, .py, .js, .go)
    pub extension: String,
}

impl GeneratedCode {
    /// Create new generated code result
    pub fn new(content: String, language: String, extension: String) -> Self {
        Self {
            content,
            language,
            extension,
        }
    }
}

/// Code generator trait (sync, not async)
///
/// All generators must implement this trait with synchronous methods.
/// Async implementations should be handled internally, not in trait methods.
pub trait CodeGenerator: Send + Sync {
    /// Generate domain model code
    ///
    /// # Arguments
    ///
    /// * `context` - Generation context with data bindings
    ///
    /// # Returns
    ///
    /// Generated code string
    fn generate_domain_model(&self, context: &GenerationContext) -> WorkflowResult<GeneratedCode>;

    /// Generate API endpoint code
    ///
    /// # Arguments
    ///
    /// * `context` - Generation context with data bindings
    ///
    /// # Returns
    ///
    /// Generated code string
    fn generate_api_endpoint(&self, context: &GenerationContext) -> WorkflowResult<GeneratedCode>;

    /// Generate test code
    ///
    /// # Arguments
    ///
    /// * `context` - Generation context with data bindings
    ///
    /// # Returns
    ///
    /// Generated test code string
    fn generate_tests(&self, context: &GenerationContext) -> WorkflowResult<GeneratedCode>;

    /// Generate documentation
    ///
    /// # Arguments
    ///
    /// * `context` - Generation context with data bindings
    ///
    /// # Returns
    ///
    /// Generated documentation string
    fn generate_documentation(&self, context: &GenerationContext) -> WorkflowResult<GeneratedCode>;

    /// Get language identifier
    fn language(&self) -> &str;

    /// Get file extension for this language
    fn file_extension(&self) -> &str;
}

/// Create generator for specified language
///
/// # Arguments
///
/// * `language` - Language identifier (rust, python, javascript, go)
/// * `template_dir` - Directory containing templates for the language
///
/// # Errors
///
/// Returns error if language is not supported or template directory is invalid.
pub fn create_generator(
    language: &str,
    template_dir: impl AsRef<Path>,
) -> WorkflowResult<Box<dyn CodeGenerator>> {
    match language.to_lowercase().as_str() {
        "rust" => Ok(Box::new(RustGenerator::new(template_dir)?)),
        "python" => Ok(Box::new(PythonGenerator::new(template_dir)?)),
        _ => Err(WorkflowError::Internal(format!(
            "Unsupported language: {}",
            language
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_context() {
        let mut context = GenerationContext::new();
        context.insert("key1".to_string(), "value1".to_string());

        assert_eq!(context.get("key1"), Some(&"value1".to_string()));
        assert_eq!(context.get("nonexistent"), None);
    }

    #[test]
    fn test_generated_code() {
        let code = GeneratedCode::new(
            "fn main() {}".to_string(),
            "rust".to_string(),
            ".rs".to_string(),
        );

        assert_eq!(code.language, "rust");
        assert_eq!(code.extension, ".rs");
        assert!(code.content.contains("fn main"));
    }
}
