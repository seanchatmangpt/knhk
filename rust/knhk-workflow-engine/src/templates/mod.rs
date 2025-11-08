//! Workflow template library
//!
//! Pre-built workflow templates for common business patterns.

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpec;
use serde_json::Value;
use std::collections::HashMap;

/// Template library
pub struct TemplateLibrary {
    /// Registered templates
    templates: HashMap<String, WorkflowTemplate>,
}

/// Workflow template
pub struct WorkflowTemplate {
    /// Template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Template category
    pub category: String,
    /// Template description
    pub description: String,
    /// Template generator function
    pub generator: Box<dyn Fn(&Value) -> WorkflowResult<WorkflowSpec> + Send + Sync>,
}

impl TemplateLibrary {
    /// Create a new template library with built-in templates
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Register built-in templates
        templates.insert(
            "two-stage-approval".to_string(),
            WorkflowTemplate {
                id: "two-stage-approval".to_string(),
                name: "Two-Stage Approval".to_string(),
                category: "approval".to_string(),
                description: "Requires approval from two different roles".to_string(),
                generator: Box::new(Self::generate_two_stage_approval),
            },
        );

        templates.insert(
            "sequential-processing".to_string(),
            WorkflowTemplate {
                id: "sequential-processing".to_string(),
                name: "Sequential Processing".to_string(),
                category: "processing".to_string(),
                description: "Tasks execute in sequence".to_string(),
                generator: Box::new(Self::generate_sequential_processing),
            },
        );

        Self { templates }
    }

    /// Get template by ID
    pub fn get_template(&self, template_id: &str) -> WorkflowResult<&WorkflowTemplate> {
        self.templates.get(template_id).ok_or_else(|| {
            WorkflowError::InvalidSpecification(format!("Template {} not found", template_id))
        })
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }

    /// List templates by category
    pub fn list_by_category(&self, category: &str) -> Vec<&WorkflowTemplate> {
        self.templates
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Instantiate a template with parameters
    pub fn instantiate(&self, template_id: &str, params: Value) -> WorkflowResult<WorkflowSpec> {
        let template = self.get_template(template_id)?;
        (template.generator)(&params)
    }

    /// Generate two-stage approval workflow
    fn generate_two_stage_approval(params: &Value) -> WorkflowResult<WorkflowSpec> {
        // In production, would generate full workflow spec
        // For now, return error indicating implementation needed
        Err(WorkflowError::ExecutionFailed(
            "Template instantiation not yet implemented".to_string(),
        ))
    }

    /// Generate sequential processing workflow
    fn generate_sequential_processing(_params: &Value) -> WorkflowResult<WorkflowSpec> {
        // In production, would generate full workflow spec
        Err(WorkflowError::ExecutionFailed(
            "Template instantiation not yet implemented".to_string(),
        ))
    }
}

impl Default for TemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates() {
        let library = TemplateLibrary::new();
        let templates = library.list_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_get_template() {
        let library = TemplateLibrary::new();
        let template = library.get_template("two-stage-approval");
        assert!(template.is_ok());
    }
}
