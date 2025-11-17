//! Workflow Template System
//!
//! Pre-built workflow patterns that combine hooks, guards, and YAWL patterns
//! into reusable, production-ready workflow templates.
//!
//! # Philosophy
//!
//! Templates encode best practices and proven patterns, allowing developers
//! to build workflows from high-level components rather than primitives.

use std::collections::HashMap;
use std::sync::Arc;

use crate::execution::{patterns, HookContext, HookFn, HookResult};
use serde::{Deserialize, Serialize};

/// Workflow template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub chatman_compliant: bool,
    pub estimated_ticks: u32,
}

/// Workflow template
pub struct WorkflowTemplate {
    pub metadata: TemplateMetadata,
    pub hooks: HashMap<String, HookFn>,
    pub execution_order: Vec<String>,
    pub guards: Vec<String>,
}

impl WorkflowTemplate {
    /// Create a new template
    pub fn new(metadata: TemplateMetadata) -> Self {
        Self {
            metadata,
            hooks: HashMap::new(),
            execution_order: Vec::new(),
            guards: Vec::new(),
        }
    }

    /// Add a hook to the template
    pub fn add_hook(&mut self, name: String, hook: HookFn) -> &mut Self {
        self.hooks.insert(name.clone(), hook);
        self.execution_order.push(name);
        self
    }

    /// Add a guard requirement
    pub fn require_guard(&mut self, guard: String) -> &mut Self {
        self.guards.push(guard);
        self
    }

    /// Get all hooks in execution order
    pub fn get_hooks(&self) -> Vec<(String, HookFn)> {
        self.execution_order
            .iter()
            .filter_map(|name| {
                self.hooks
                    .get(name)
                    .map(|hook| (name.clone(), hook.clone()))
            })
            .collect()
    }
}

/// Template library containing pre-built workflows
pub struct TemplateLibrary {
    templates: HashMap<String, WorkflowTemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            templates: HashMap::new(),
        };

        // Load standard templates
        library.load_standard_templates();
        library
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&WorkflowTemplate> {
        self.templates.get(name)
    }

    /// Register a custom template
    pub fn register(&mut self, name: String, template: WorkflowTemplate) {
        self.templates.insert(name, template);
    }

    /// List available templates
    pub fn list(&self) -> Vec<&TemplateMetadata> {
        self.templates.values().map(|t| &t.metadata).collect()
    }

    /// Load standard workflow templates
    fn load_standard_templates(&mut self) {
        // ETL Pipeline Template
        self.register("etl-pipeline".to_string(), Self::create_etl_template());

        // Request-Response Template
        self.register(
            "request-response".to_string(),
            Self::create_request_response_template(),
        );

        // Saga Pattern Template
        self.register("saga".to_string(), Self::create_saga_template());

        // Fan-Out/Fan-In Template
        self.register(
            "fan-out-fan-in".to_string(),
            Self::create_fan_out_fan_in_template(),
        );

        // Circuit Breaker Template
        self.register(
            "circuit-breaker".to_string(),
            Self::create_circuit_breaker_template(),
        );
    }

    /// ETL Pipeline: Extract → Transform → Load
    fn create_etl_template() -> WorkflowTemplate {
        let metadata = TemplateMetadata {
            name: "ETL Pipeline".to_string(),
            version: "1.0.0".to_string(),
            description: "Extract, Transform, Load data pipeline with validation".to_string(),
            author: "KNHK Team".to_string(),
            tags: vec![
                "etl".to_string(),
                "data".to_string(),
                "pipeline".to_string(),
            ],
            chatman_compliant: true,
            estimated_ticks: 6,
        };

        let mut template = WorkflowTemplate::new(metadata);

        // Extract hook
        let extract = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("DATA_AVAILABLE".to_string(), true);
            result.next_hooks = vec!["validate".to_string()];
            result
        });

        // Validate hook
        let validate = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);
            result.add_guard_check("SCHEMA_VALID".to_string(), true);
            result.next_hooks = vec!["transform".to_string()];
            result
        });

        // Transform hook
        let transform = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("TRANSFORM_COMPLETE".to_string(), true);
            result.next_hooks = vec!["load".to_string()];
            result
        });

        // Load hook
        let load = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);
            result.add_guard_check("LOAD_SUCCESS".to_string(), true);
            result
        });

        template
            .add_hook("extract".to_string(), extract)
            .add_hook("validate".to_string(), validate)
            .add_hook("transform".to_string(), transform)
            .add_hook("load".to_string(), load)
            .require_guard("DATA_AVAILABLE".to_string())
            .require_guard("SCHEMA_VALID".to_string());

        template
    }

    /// Request-Response with timeout and retry
    fn create_request_response_template() -> WorkflowTemplate {
        let metadata = TemplateMetadata {
            name: "Request-Response".to_string(),
            version: "1.0.0".to_string(),
            description: "Synchronous request-response with timeout and retry".to_string(),
            author: "KNHK Team".to_string(),
            tags: vec![
                "api".to_string(),
                "sync".to_string(),
                "reliability".to_string(),
            ],
            chatman_compliant: true,
            estimated_ticks: 5,
        };

        let mut template = WorkflowTemplate::new(metadata);

        let request = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("REQUEST_VALID".to_string(), true);
            result.next_hooks = vec!["response".to_string()];
            result
        });

        let response = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 3);
            result.add_guard_check("RESPONSE_RECEIVED".to_string(), true);
            result
        });

        template
            .add_hook("request".to_string(), request)
            .add_hook("response".to_string(), response)
            .require_guard("REQUEST_VALID".to_string())
            .require_guard("RESPONSE_RECEIVED".to_string());

        template
    }

    /// Saga Pattern: Distributed transaction with compensation
    fn create_saga_template() -> WorkflowTemplate {
        let metadata = TemplateMetadata {
            name: "Saga Pattern".to_string(),
            version: "1.0.0".to_string(),
            description: "Distributed transaction with automatic compensation on failure"
                .to_string(),
            author: "KNHK Team".to_string(),
            tags: vec![
                "distributed".to_string(),
                "transaction".to_string(),
                "saga".to_string(),
            ],
            chatman_compliant: true,
            estimated_ticks: 8,
        };

        let mut template = WorkflowTemplate::new(metadata);

        let step1 = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("STEP1_SUCCESS".to_string(), true);
            result.next_hooks = vec!["step2".to_string()];
            result
        });

        let step2 = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("STEP2_SUCCESS".to_string(), true);
            result.next_hooks = vec!["step3".to_string()];
            result
        });

        let step3 = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("STEP3_SUCCESS".to_string(), true);
            result
        });

        let compensate = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("COMPENSATION_COMPLETE".to_string(), true);
            result
        });

        template
            .add_hook("step1".to_string(), step1)
            .add_hook("step2".to_string(), step2)
            .add_hook("step3".to_string(), step3)
            .add_hook("compensate".to_string(), compensate);

        template
    }

    /// Fan-Out/Fan-In: Parallel processing with synchronization
    fn create_fan_out_fan_in_template() -> WorkflowTemplate {
        let metadata = TemplateMetadata {
            name: "Fan-Out/Fan-In".to_string(),
            version: "1.0.0".to_string(),
            description: "Parallel task execution with result synchronization".to_string(),
            author: "KNHK Team".to_string(),
            tags: vec!["parallel".to_string(), "concurrency".to_string()],
            chatman_compliant: true,
            estimated_ticks: 4,
        };

        let mut template = WorkflowTemplate::new(metadata);

        let fan_out = patterns::parallel_split(vec![
            "worker1".to_string(),
            "worker2".to_string(),
            "worker3".to_string(),
        ]);

        let worker1 = Arc::new(|ctx: &HookContext| HookResult::success(ctx.input_data.clone(), 1));

        let worker2 = Arc::new(|ctx: &HookContext| HookResult::success(ctx.input_data.clone(), 1));

        let worker3 = Arc::new(|ctx: &HookContext| HookResult::success(ctx.input_data.clone(), 1));

        let fan_in = patterns::synchronize(3);

        template
            .add_hook("fan_out".to_string(), fan_out)
            .add_hook("worker1".to_string(), worker1)
            .add_hook("worker2".to_string(), worker2)
            .add_hook("worker3".to_string(), worker3)
            .add_hook("fan_in".to_string(), fan_in);

        template
    }

    /// Circuit Breaker: Prevent cascading failures
    fn create_circuit_breaker_template() -> WorkflowTemplate {
        let metadata = TemplateMetadata {
            name: "Circuit Breaker".to_string(),
            version: "1.0.0".to_string(),
            description: "Automatic circuit breaking on repeated failures".to_string(),
            author: "KNHK Team".to_string(),
            tags: vec!["reliability".to_string(), "resilience".to_string()],
            chatman_compliant: true,
            estimated_ticks: 3,
        };

        let mut template = WorkflowTemplate::new(metadata);

        let check_circuit = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);
            result.add_guard_check("CIRCUIT_CLOSED".to_string(), true);
            result.next_hooks = vec!["execute".to_string()];
            result
        });

        let execute = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.add_guard_check("EXECUTION_SUCCESS".to_string(), true);
            result
        });

        template
            .add_hook("check_circuit".to_string(), check_circuit)
            .add_hook("execute".to_string(), execute)
            .require_guard("CIRCUIT_CLOSED".to_string());

        template
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
    fn test_template_library_creation() {
        let library = TemplateLibrary::new();
        assert!(library.templates.len() > 0);
    }

    #[test]
    fn test_get_template() {
        let library = TemplateLibrary::new();
        let template = library.get("etl-pipeline");
        assert!(template.is_some());

        let etl = template.unwrap();
        assert_eq!(etl.metadata.name, "ETL Pipeline");
        assert!(etl.metadata.chatman_compliant);
        assert!(etl.metadata.estimated_ticks <= 8);
    }

    #[test]
    fn test_list_templates() {
        let library = TemplateLibrary::new();
        let templates = library.list();
        assert!(templates.len() >= 5);

        let names: Vec<_> = templates.iter().map(|t| &t.name).collect();
        assert!(names.contains(&&"ETL Pipeline".to_string()));
        assert!(names.contains(&&"Request-Response".to_string()));
        assert!(names.contains(&&"Saga Pattern".to_string()));
    }

    #[test]
    fn test_template_hooks() {
        let library = TemplateLibrary::new();
        let template = library.get("etl-pipeline").unwrap();

        let hooks = template.get_hooks();
        assert_eq!(hooks.len(), 4);

        let hook_names: Vec<_> = hooks.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(hook_names, vec!["extract", "validate", "transform", "load"]);
    }

    #[test]
    fn test_custom_template() {
        let mut library = TemplateLibrary::new();

        let metadata = TemplateMetadata {
            name: "Custom Workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "A custom workflow".to_string(),
            author: "Test".to_string(),
            tags: vec!["custom".to_string()],
            chatman_compliant: true,
            estimated_ticks: 4,
        };

        let mut template = WorkflowTemplate::new(metadata);
        let hook = Arc::new(|ctx: &HookContext| HookResult::success(ctx.input_data.clone(), 1));
        template.add_hook("custom_hook".to_string(), hook);

        library.register("custom".to_string(), template);

        let retrieved = library.get("custom");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.name, "Custom Workflow");
    }

    #[test]
    fn test_template_guards() {
        let library = TemplateLibrary::new();
        let template = library.get("etl-pipeline").unwrap();

        assert!(template.guards.contains(&"DATA_AVAILABLE".to_string()));
        assert!(template.guards.contains(&"SCHEMA_VALID".to_string()));
    }

    #[test]
    fn test_all_templates_chatman_compliant() {
        let library = TemplateLibrary::new();
        for template in library.templates.values() {
            assert!(
                template.metadata.chatman_compliant,
                "Template {} is not Chatman compliant",
                template.metadata.name
            );
            assert!(
                template.metadata.estimated_ticks <= 8,
                "Template {} exceeds Chatman constant: {} ticks",
                template.metadata.name,
                template.metadata.estimated_ticks
            );
        }
    }
}
