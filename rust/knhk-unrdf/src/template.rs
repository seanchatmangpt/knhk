// knhk-unrdf: Template engine for JavaScript templates
// Uses Tera template engine for safe, compile-time template rendering

// ACCEPTABLE: Singleton initialization .expect() is allowed (unrecoverable deployment error)
#![allow(clippy::expect_used)]

use crate::error::UnrdfError;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tera::{Context, Result as TeraResult, Tera, Value};

// Singleton template engine instance
static TEMPLATE_ENGINE: OnceLock<Mutex<TemplateEngine>> = OnceLock::new();

/// Template engine wrapper for Tera
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Initialize template engine with all templates loaded at compile time
    fn new() -> Result<Self, UnrdfError> {
        let mut tera = Tera::default();

        // Register custom filter for JavaScript string escaping
        tera.register_filter("escape_js", Self::escape_js_filter);

        // Load all templates at compile time
        tera.add_raw_template(
            "query-with-data",
            include_str!("../templates/query-with-data.tera"),
        )
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to load query-with-data template: {}", e))
        })?;

        tera.add_raw_template("query-only", include_str!("../templates/query-only.tera"))
            .map_err(|e| {
                UnrdfError::InvalidInput(format!("Failed to load query-only template: {}", e))
            })?;

        tera.add_raw_template(
            "hook-execute",
            include_str!("../templates/hook-execute.tera"),
        )
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to load hook-execute template: {}", e))
        })?;

        tera.add_raw_template(
            "hook-execute-with-data",
            include_str!("../templates/hook-execute-with-data.tera"),
        )
        .map_err(|e| {
            UnrdfError::InvalidInput(format!(
                "Failed to load hook-execute-with-data template: {}",
                e
            ))
        })?;

        tera.add_raw_template(
            "hook-register",
            include_str!("../templates/hook-register.tera"),
        )
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to load hook-register template: {}", e))
        })?;

        tera.add_raw_template("store", include_str!("../templates/store.tera"))
            .map_err(|e| {
                UnrdfError::InvalidInput(format!("Failed to load store template: {}", e))
            })?;

        tera.add_raw_template(
            "transaction-commit",
            include_str!("../templates/transaction-commit.tera"),
        )
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to load transaction-commit template: {}", e))
        })?;

        tera.add_raw_template(
            "shacl-validate",
            include_str!("../templates/shacl-validate.tera"),
        )
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to load shacl-validate template: {}", e))
        })?;

        tera.add_raw_template("serialize", include_str!("../templates/serialize.tera"))
            .map_err(|e| {
                UnrdfError::InvalidInput(format!("Failed to load serialize template: {}", e))
            })?;

        Ok(Self { tera })
    }

    /// Get singleton template engine instance
    pub fn get() -> Result<&'static Mutex<TemplateEngine>, UnrdfError> {
        // ACCEPTABLE: Singleton initialization failure is rare and unrecoverable.
        // Template engine includes hardcoded templates, so failure indicates broken deployment.
        // Alternative: Use once_cell crate's get_or_try_init (requires adding dependency).
        Ok(TEMPLATE_ENGINE.get_or_init(|| {
            Mutex::new(Self::new().expect(
                "FATAL: Failed to initialize template engine - templates are corrupted or missing",
            ))
        }))
    }

    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: &Context) -> TeraResult<String> {
        self.tera.render(template_name, context)
    }

    /// Custom Tera filter for JavaScript string escaping
    /// Escapes backticks, backslashes, and dollar signs for JavaScript template literals
    fn escape_js_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
        let s = value
            .as_str()
            .ok_or_else(|| tera::Error::msg("escape_js filter requires a string value"))?;
        Ok(Value::String(
            s.replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace('$', "\\$"),
        ))
    }

    /// Helper function for JavaScript string escaping (used outside templates)
    pub fn escape_js_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace('$', "\\$")
    }
}
