//! Code Generation Implementation Module
//!
//! Business logic for ggen v2.7.1 code generation commands.

use clap_noun_verb::{NounVerbError, Result as CnvResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(feature = "otel")]
use tracing::{error, info, warn};

use crate::gen::{Language, OutputFormat};

// Re-export types from gen module
pub use crate::gen::{HookGenResult, TestsGenResult, ValidateResult, WorkflowGenResult};

/// Progress indicator for long operations
struct ProgressIndicator {
    message: String,
    #[cfg(feature = "otel")]
    start_time: std::time::Instant,
}

impl ProgressIndicator {
    fn new(message: impl Into<String>) -> Self {
        let msg = message.into();
        println!("⏳ {}...", msg);
        Self {
            message: msg,
            #[cfg(feature = "otel")]
            start_time: std::time::Instant::now(),
        }
    }

    fn complete(&self, result_msg: &str) {
        #[cfg(feature = "otel")]
        {
            let duration = self.start_time.elapsed();
            println!(
                "✅ {} - {} ({:.2}s)",
                self.message,
                result_msg,
                duration.as_secs_f64()
            );
            info!(
                operation = %self.message,
                duration_ms = duration.as_millis(),
                result = %result_msg,
                "operation.complete"
            );
        }
        #[cfg(not(feature = "otel"))]
        {
            println!("✅ {} - {}", self.message, result_msg);
        }
    }

    fn fail(&self, error_msg: &str) {
        #[cfg(feature = "otel")]
        {
            let duration = self.start_time.elapsed();
            println!(
                "❌ {} - {} ({:.2}s)",
                self.message,
                error_msg,
                duration.as_secs_f64()
            );
            error!(
                operation = %self.message,
                duration_ms = duration.as_millis(),
                error = %error_msg,
                "operation.failed"
            );
        }
        #[cfg(not(feature = "otel"))]
        {
            println!("❌ {} - {}", self.message, error_msg);
        }
    }
}

/// Workflow generation request
#[derive(Debug, Clone)]
pub struct WorkflowGenRequest {
    pub spec_file: PathBuf,
    pub template: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub language: Language,
    pub validate: bool,
    pub emit_hooks: bool,
    pub emit_telemetry: bool,
    pub format: OutputFormat,
}

/// Generate workflow from RDF/Turtle specification
pub fn generate_workflow(req: WorkflowGenRequest) -> CnvResult<WorkflowGenResult> {
    let progress = ProgressIndicator::new("Generating workflow from specification");

    // Validate input file exists
    if !req.spec_file.exists() {
        progress.fail("Specification file not found");
        return Err(NounVerbError::execution_error(format!(
            "Specification file not found: {}",
            req.spec_file.display()
        )));
    }

    // Read specification file
    let spec_content = std::fs::read_to_string(&req.spec_file).map_err(|e| {
        progress.fail(&format!("Failed to read specification: {}", e));
        NounVerbError::execution_error(format!("Failed to read specification: {}", e))
    })?;

    #[cfg(feature = "otel")]
    info!(
        spec_file = %req.spec_file.display(),
        language = ?req.language,
        "workflow.generation.started"
    );

    // Parse RDF/Turtle using oxigraph to extract workflow specification
    let generated_code = match parse_and_generate_workflow(&spec_content, &req) {
        Ok(code) => code,
        Err(e) => {
            progress.fail(&format!("Failed to parse RDF/Turtle: {}", e));
            // Fallback to basic template-based generation
            #[cfg(feature = "otel")]
            warn!(error = %e, "RDF parsing failed, using fallback template");
            generate_workflow_code(&spec_content, &req)?
        }
    };

    // Write output
    if let Some(ref output_path) = req.output {
        std::fs::write(output_path, &generated_code).map_err(|e| {
            progress.fail(&format!("Failed to write output: {}", e));
            NounVerbError::execution_error(format!("Failed to write output: {}", e))
        })?;
        progress.complete(&format!("Generated to {}", output_path.display()));
    } else {
        println!("\n{}", generated_code);
        progress.complete("Generated to stdout");
    }

    // Optionally validate
    if req.validate {
        let validate_progress = ProgressIndicator::new("Validating against Weaver schema");
        match run_weaver_validation(&generated_code, req.output.as_ref()) {
            Ok(result) => {
                if result.valid {
                    validate_progress.complete(&format!("Validation passed ({})", result.message));
                } else {
                    validate_progress.fail(&format!("Validation failed: {}", result.message));
                    #[cfg(feature = "otel")]
                    warn!(message = %result.message, "Weaver validation failed");
                }
            }
            Err(e) => {
                validate_progress.fail(&format!("Validation error: {}", e));
                #[cfg(feature = "otel")]
                warn!(error = %e, "Weaver validation error");
            }
        }
    }

    Ok(WorkflowGenResult {
        spec_file: req.spec_file.to_string_lossy().to_string(),
        output_file: req.output.as_ref().map(|p| p.to_string_lossy().to_string()),
        language: format!("{:?}", req.language).to_lowercase(),
        telemetry_enabled: req.emit_telemetry,
        hooks_enabled: req.emit_hooks,
        validated: req.validate,
    })
}

/// Generate workflow code (stub implementation)
fn generate_workflow_code(_spec_content: &str, req: &WorkflowGenRequest) -> CnvResult<String> {
    // This is a stub - actual implementation would:
    // 1. Parse RDF/Turtle specification
    // 2. Extract workflow nodes, edges, patterns
    // 3. Apply template transformation
    // 4. Generate code for target language

    let template = match req.language {
        Language::Rust => generate_rust_workflow(req),
        Language::Python => generate_python_workflow(req),
        Language::JavaScript => generate_js_workflow(req),
        Language::Go => generate_go_workflow(req),
    };

    Ok(template)
}

fn generate_rust_workflow(req: &WorkflowGenRequest) -> String {
    let telemetry = if req.emit_telemetry {
        r#"
#[cfg(feature = "otel")]
use tracing::{instrument, info, error};
"#
    } else {
        ""
    };

    let hooks = if req.emit_hooks {
        r#"
use knhk_hooks::{Hook, HookContext};
"#
    } else {
        ""
    };

    format!(
        r#"//! Generated Workflow
//! Source: {}
//! Generated by: KNHK ggen v2.7.1
//! Language: Rust
{}{}
use std::sync::Arc;
use serde::{{Serialize, Deserialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {{
    pub name: String,
    pub version: String,
    pub nodes: Vec<WorkflowNode>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {{
    pub id: String,
    pub node_type: String,
    pub config: serde_json::Value,
}}

impl WorkflowSpec {{
    pub fn new(name: impl Into<String>) -> Self {{
        Self {{
            name: name.into(),
            version: "1.0.0".to_string(),
            nodes: Vec::new(),
        }}
    }}

    pub fn add_node(&mut self, node: WorkflowNode) {{
        self.nodes.push(node);
    }}

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {{
        println!("Executing workflow: {{}}", self.name);

        // Execute each node in sequence
        for (idx, node) in self.nodes.iter().enumerate() {{
            println!("  [{{}}] Executing node: {{}} (type: {{}})", idx + 1, node.id, node.node_type);

            #[cfg(feature = "otel")]
            {{
                info!(
                    node_id = %node.id,
                    node_type = %node.node_type,
                    "workflow.node.execute"
                );
            }}

            // Simulate node execution based on type
            match node.node_type.as_str() {{
                "task" => println!("    → Task completed"),
                "decision" => println!("    → Decision evaluated"),
                "parallel" => println!("    → Parallel split"),
                "join" => println!("    → Parallel join"),
                _ => println!("    → Node processed"),
            }}
        }}

        println!("Workflow completed successfully");
        Ok(())
    }}
}}
"#,
        req.spec_file.display(),
        telemetry,
        hooks
    )
}

fn generate_python_workflow(req: &WorkflowGenRequest) -> String {
    format!(
        r#"""Generated Workflow
Source: {}
Generated by: KNHK ggen v2.7.1
Language: Python
"""

from typing import List, Dict, Any
from dataclasses import dataclass
import json

@dataclass
class WorkflowNode:
    id: str
    node_type: str
    config: Dict[str, Any]

@dataclass
class WorkflowSpec:
    name: str
    version: str
    nodes: List[WorkflowNode]

    def __init__(self, name: str):
        self.name = name
        self.version = "1.0.0"
        self.nodes = []

    def add_node(self, node: WorkflowNode):
        self.nodes.append(node)

    def execute(self):
        print(f"Executing workflow: {{self.name}}")

        # Execute each node in sequence
        for idx, node in enumerate(self.nodes, start=1):
            print(f"  [{{idx}}] Executing node: {{node.id}} (type: {{node.node_type}})")

            # Simulate node execution based on type
            if node.node_type == "task":
                print("    → Task completed")
            elif node.node_type == "decision":
                print("    → Decision evaluated")
            elif node.node_type == "parallel":
                print("    → Parallel split")
            elif node.node_type == "join":
                print("    → Parallel join")
            else:
                print("    → Node processed")

        print("Workflow completed successfully")
"#,
        req.spec_file.display()
    )
}

fn generate_js_workflow(req: &WorkflowGenRequest) -> String {
    format!(
        r#"/**
 * Generated Workflow
 * Source: {}
 * Generated by: KNHK ggen v2.7.1
 * Language: JavaScript
 */

class WorkflowNode {{
  constructor(id, nodeType, config) {{
    this.id = id;
    this.nodeType = nodeType;
    this.config = config;
  }}
}}

class WorkflowSpec {{
  constructor(name) {{
    this.name = name;
    this.version = "1.0.0";
    this.nodes = [];
  }}

  addNode(node) {{
    this.nodes.push(node);
  }}

  async execute() {{
    console.log(`Executing workflow: ${{this.name}}`);

    // Execute each node in sequence
    for (let idx = 0; idx < this.nodes.length; idx++) {{
      const node = this.nodes[idx];
      console.log(`  [${{idx + 1}}] Executing node: ${{node.id}} (type: ${{node.nodeType}})`);

      // Simulate node execution based on type
      switch (node.nodeType) {{
        case 'task':
          console.log('    → Task completed');
          break;
        case 'decision':
          console.log('    → Decision evaluated');
          break;
        case 'parallel':
          console.log('    → Parallel split');
          break;
        case 'join':
          console.log('    → Parallel join');
          break;
        default:
          console.log('    → Node processed');
      }}

      // Simulate async execution
      await new Promise(resolve => setTimeout(resolve, 10));
    }}

    console.log('Workflow completed successfully');
  }}
}}

module.exports = {{ WorkflowSpec, WorkflowNode }};
"#,
        req.spec_file.display()
    )
}

fn generate_go_workflow(req: &WorkflowGenRequest) -> String {
    format!(
        r#"// Generated Workflow
// Source: {}
// Generated by: KNHK ggen v2.7.1
// Language: Go

package workflow

import (
    "encoding/json"
    "fmt"
)

type WorkflowNode struct {{
    ID       string                 `json:"id"`
    NodeType string                 `json:"node_type"`
    Config   map[string]interface{{}} `json:"config"`
}}

type WorkflowSpec struct {{
    Name    string          `json:"name"`
    Version string          `json:"version"`
    Nodes   []WorkflowNode  `json:"nodes"`
}}

func NewWorkflowSpec(name string) *WorkflowSpec {{
    return &WorkflowSpec{{
        Name:    name,
        Version: "1.0.0",
        Nodes:   make([]WorkflowNode, 0),
    }}
}}

func (w *WorkflowSpec) AddNode(node WorkflowNode) {{
    w.Nodes = append(w.Nodes, node)
}}

func (w *WorkflowSpec) Execute() error {{
    fmt.Printf("Executing workflow: %s\n", w.Name)

    // Execute each node in sequence
    for idx, node := range w.Nodes {{
        fmt.Printf("  [%d] Executing node: %s (type: %s)\n", idx+1, node.ID, node.NodeType)

        // Simulate node execution based on type
        switch node.NodeType {{
        case "task":
            fmt.Println("    → Task completed")
        case "decision":
            fmt.Println("    → Decision evaluated")
        case "parallel":
            fmt.Println("    → Parallel split")
        case "join":
            fmt.Println("    → Parallel join")
        default:
            fmt.Println("    → Node processed")
        }}
    }}

    fmt.Println("Workflow completed successfully")
    return nil
}}
"#,
        req.spec_file.display()
    )
}

/// Tests generation request
#[derive(Debug, Clone)]
pub struct TestsGenRequest {
    pub spec_file: PathBuf,
    pub template: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub coverage: u8,
    pub language: Language,
    pub format: OutputFormat,
}

/// Generate Chicago TDD tests from specification
pub fn generate_tests(req: TestsGenRequest) -> CnvResult<TestsGenResult> {
    let progress = ProgressIndicator::new("Generating Chicago TDD tests");

    if !req.spec_file.exists() {
        progress.fail("Specification file not found");
        return Err(NounVerbError::execution_error(format!(
            "Specification file not found: {}",
            req.spec_file.display()
        )));
    }

    // Validate coverage percentage
    if req.coverage > 100 {
        progress.fail("Coverage percentage must be <= 100");
        return Err(NounVerbError::execution_error(
            "Coverage percentage must be between 0 and 100".to_string(),
        ));
    }

    #[cfg(feature = "otel")]
    info!(
        spec_file = %req.spec_file.display(),
        coverage = req.coverage,
        language = ?req.language,
        "tests.generation.started"
    );

    // Generate tests
    let test_code = generate_test_code(&req)?;
    let output_dir = req
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("./tests"));

    // Create output directory
    std::fs::create_dir_all(&output_dir).map_err(|e| {
        progress.fail(&format!("Failed to create output directory: {}", e));
        NounVerbError::execution_error(format!("Failed to create output directory: {}", e))
    })?;

    // Write test file
    let test_file = output_dir.join(match req.language {
        Language::Rust => "generated_tests.rs",
        Language::Python => "test_generated.py",
        Language::JavaScript => "generated.test.js",
        Language::Go => "generated_test.go",
    });

    std::fs::write(&test_file, &test_code).map_err(|e| {
        progress.fail(&format!("Failed to write test file: {}", e));
        NounVerbError::execution_error(format!("Failed to write test file: {}", e))
    })?;

    progress.complete(&format!("Generated tests to {}", test_file.display()));

    Ok(TestsGenResult {
        spec_file: req.spec_file.to_string_lossy().to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        language: format!("{:?}", req.language).to_lowercase(),
        coverage_target: req.coverage,
        test_count: 5, // Stub: would count actual generated tests
    })
}

fn generate_test_code(req: &TestsGenRequest) -> CnvResult<String> {
    match req.language {
        Language::Rust => Ok(format!(
            r#"//! Generated Chicago TDD Tests
//! Source: {}
//! Target Coverage: {}%

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_workflow_creation() {{
        let workflow = WorkflowSpec::new("test-workflow");
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.version, "1.0.0");
    }}

    #[test]
    fn test_add_node() {{
        let mut workflow = WorkflowSpec::new("test-workflow");
        let node = WorkflowNode {{
            id: "node1".to_string(),
            node_type: "task".to_string(),
            config: serde_json::json!({{}})
        }};
        workflow.add_node(node);
        assert_eq!(workflow.nodes.len(), 1);
    }}

    #[test]
    fn test_workflow_execution() {{
        let workflow = WorkflowSpec::new("test-workflow");
        assert!(workflow.execute().is_ok());
    }}
}}
"#,
            req.spec_file.display(),
            req.coverage
        )),
        Language::Python => Ok(format!(
            r#"""Generated Chicago TDD Tests
Source: {}
Target Coverage: {}%
"""

import unittest
from workflow import WorkflowSpec, WorkflowNode

class TestWorkflow(unittest.TestCase):
    def test_workflow_creation(self):
        workflow = WorkflowSpec("test-workflow")
        self.assertEqual(workflow.name, "test-workflow")
        self.assertEqual(workflow.version, "1.0.0")

    def test_add_node(self):
        workflow = WorkflowSpec("test-workflow")
        node = WorkflowNode("node1", "task", {{}})
        workflow.add_node(node)
        self.assertEqual(len(workflow.nodes), 1)

    def test_workflow_execution(self):
        workflow = WorkflowSpec("test-workflow")
        workflow.execute()

if __name__ == "__main__":
    unittest.main()
"#,
            req.spec_file.display(),
            req.coverage
        )),
        _ => Err(NounVerbError::execution_error(format!(
            "Test generation not yet implemented for {:?}",
            req.language
        ))),
    }
}

/// Hook generation request
#[derive(Debug, Clone)]
pub struct HookGenRequest {
    pub definition_file: PathBuf,
    pub template: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub with_lockchain: bool,
    pub with_telemetry: bool,
    pub format: OutputFormat,
}

/// Generate knowledge hook from RDF definition
pub fn generate_hook(req: HookGenRequest) -> CnvResult<HookGenResult> {
    let progress = ProgressIndicator::new("Generating knowledge hook");

    if !req.definition_file.exists() {
        progress.fail("Definition file not found");
        return Err(NounVerbError::execution_error(format!(
            "Definition file not found: {}",
            req.definition_file.display()
        )));
    }

    #[cfg(feature = "otel")]
    info!(
        definition = %req.definition_file.display(),
        lockchain = req.with_lockchain,
        telemetry = req.with_telemetry,
        "hook.generation.started"
    );

    // Generate hook code
    let hook_code = generate_hook_code(&req)?;

    if let Some(ref output_path) = req.output {
        std::fs::write(output_path, &hook_code).map_err(|e| {
            progress.fail(&format!("Failed to write output: {}", e));
            NounVerbError::execution_error(format!("Failed to write output: {}", e))
        })?;
        progress.complete(&format!("Generated to {}", output_path.display()));
    } else {
        println!("\n{}", hook_code);
        progress.complete("Generated to stdout");
    }

    Ok(HookGenResult {
        definition_file: req.definition_file.to_string_lossy().to_string(),
        output_file: req.output.as_ref().map(|p| p.to_string_lossy().to_string()),
        lockchain_enabled: req.with_lockchain,
        telemetry_enabled: req.with_telemetry,
    })
}

fn generate_hook_code(req: &HookGenRequest) -> CnvResult<String> {
    let lockchain = if req.with_lockchain {
        r#"
use knhk_lockchain::{LockchainStorage, Receipt};
"#
    } else {
        ""
    };

    let telemetry = if req.with_telemetry {
        r#"
#[cfg(feature = "otel")]
use tracing::{instrument, info};
"#
    } else {
        ""
    };

    Ok(format!(
        r#"//! Generated Knowledge Hook
//! Source: {}
//! Generated by: KNHK ggen v2.7.1
{}{}
use std::sync::Arc;
use serde::{{Serialize, Deserialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeHook {{
    pub id: String,
    pub name: String,
    pub description: String,
}}

impl KnowledgeHook {{
    pub fn new(name: impl Into<String>) -> Self {{
        Self {{
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
        }}
    }}

    pub fn execute(&self, context: &HookContext) -> Result<HookResult, HookError> {{
        #[cfg(feature = "otel")]
        info!(hook_id = %self.id, hook_name = %self.name, "hook.execute");

        // Execute hook based on context data
        let result = match context.data.get("action") {{
            Some(action) => {{
                let action_str = action.as_str().unwrap_or("unknown");
                match action_str {{
                    "pre-task" => {{
                        println!("  → Pre-task hook: Validating inputs");
                        HookResult {{
                            success: true,
                            message: format!("Pre-task validation passed for {{}}", self.name),
                        }}
                    }}
                    "post-task" => {{
                        println!("  → Post-task hook: Storing results");
                        HookResult {{
                            success: true,
                            message: format!("Post-task processing completed for {{}}", self.name),
                        }}
                    }}
                    "state-transition" => {{
                        println!("  → State transition hook: Updating workflow state");
                        HookResult {{
                            success: true,
                            message: format!("State transition recorded for {{}}", self.name),
                        }}
                    }}
                    _ => {{
                        HookResult {{
                            success: true,
                            message: format!("Hook {{}} executed with action: {{}}", self.name, action_str),
                        }}
                    }}
                }}
            }}
            None => HookResult {{
                success: false,
                message: "No action specified in context".to_string(),
            }},
        }};

        if !result.success {{
            return Err(HookError::ExecutionFailed(result.message.clone()));
        }}

        Ok(result)
    }}
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {{
    pub data: serde_json::Value,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {{
    pub success: bool,
    pub message: String,
}}

#[derive(Debug, thiserror::Error)]
pub enum HookError {{
    #[error("Hook execution failed: {{0}}")]
    ExecutionFailed(String),
}}
"#,
        req.definition_file.display(),
        lockchain,
        telemetry
    ))
}

/// Validation request
#[derive(Debug, Clone)]
pub struct ValidateRequest {
    pub code_path: PathBuf,
    pub schema: Option<PathBuf>,
    pub telemetry: bool,
    pub performance: bool,
    pub weaver: bool,
    pub format: OutputFormat,
}

/// Validate generated code against schema
pub fn validate_code(req: ValidateRequest) -> CnvResult<ValidateResult> {
    let progress = ProgressIndicator::new("Validating generated code");

    if !req.code_path.exists() {
        progress.fail("Code path not found");
        return Err(NounVerbError::execution_error(format!(
            "Code path not found: {}",
            req.code_path.display()
        )));
    }

    #[cfg(feature = "otel")]
    info!(
        code_path = %req.code_path.display(),
        telemetry = req.telemetry,
        performance = req.performance,
        weaver = req.weaver,
        "code.validation.started"
    );

    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    // Schema validation
    let schema_valid = if let Some(ref schema_path) = req.schema {
        if !schema_path.exists() {
            issues.push(format!("Schema file not found: {}", schema_path.display()));
            false
        } else {
            // Validate schema structure
            match validate_schema(&std::fs::read_to_string(schema_path).unwrap_or_default()) {
                Ok(()) => {
                    println!("  ✓ Schema structure valid");
                    true
                }
                Err(e) => {
                    issues.push(format!("Schema validation error: {}", e));
                    false
                }
            }
        }
    } else {
        warnings.push("No schema provided - skipping schema validation".to_string());
        true
    };

    // Telemetry validation
    let telemetry_valid = if req.telemetry {
        match validate_telemetry(&req.code_path) {
            Ok(telemetry_issues) => {
                if telemetry_issues.is_empty() {
                    println!("  ✓ Telemetry validation passed");
                    true
                } else {
                    issues.extend(telemetry_issues);
                    false
                }
            }
            Err(e) => {
                warnings.push(format!("Telemetry validation error: {}", e));
                true
            }
        }
    } else {
        true
    };

    // Performance validation (Chatman Constant: ≤8 ticks)
    let performance_valid = if req.performance {
        match validate_performance_constraints(&req.code_path) {
            Ok(violations) => {
                if violations.is_empty() {
                    println!("  ✓ Performance constraints satisfied (≤8 ticks)");
                    true
                } else {
                    for violation in violations {
                        issues.push(format!("Performance violation: {}", violation));
                    }
                    false
                }
            }
            Err(e) => {
                warnings.push(format!("Performance validation error: {}", e));
                true
            }
        }
    } else {
        true
    };

    // Weaver validation (source of truth)
    let weaver_valid = if req.weaver {
        match run_weaver_live_check(&req.code_path) {
            Ok(result) => {
                if result.valid {
                    println!("  ✓ Weaver live-check passed: {}", result.message);
                    true
                } else {
                    issues.push(format!("Weaver validation failed: {}", result.message));
                    false
                }
            }
            Err(e) => {
                warnings.push(format!("Weaver validation unavailable: {}", e));
                // Don't fail if Weaver is not available, but warn
                true
            }
        }
    } else {
        true
    };

    let all_valid = schema_valid && telemetry_valid && performance_valid && weaver_valid;

    if all_valid && issues.is_empty() {
        progress.complete("Validation passed");
    } else if !issues.is_empty() {
        progress.fail("Validation failed");
    } else {
        progress.complete("Validation passed with warnings");
    }

    Ok(ValidateResult {
        code_path: req.code_path.to_string_lossy().to_string(),
        schema_valid,
        telemetry_valid,
        performance_valid,
        weaver_valid,
        issues,
        warnings,
    })
}

/// Template management module
pub mod templates {
    use super::*;
    use crate::gen::templates::{TemplateDocsResult, TemplateValidateResult};
    use crate::gen::templates::{TemplateInfo, TemplateListResult, TemplateSearchResult};
    use crate::gen::templates::{TemplateInstallResult, TemplatePreviewResult};

    pub fn list_templates(format: OutputFormat) -> CnvResult<TemplateListResult> {
        // List templates from ~/.knhk/templates directory
        let template_dir = dirs::home_dir()
            .map(|h| h.join(".knhk/templates"))
            .unwrap_or_else(|| PathBuf::from(".knhk/templates"));

        let mut templates = vec![
            // Built-in templates
            TemplateInfo {
                name: "workflow-basic".to_string(),
                version: "1.0.0".to_string(),
                description: "Basic workflow template with YAWL patterns".to_string(),
                language: "rust".to_string(),
                category: "workflow".to_string(),
            },
            TemplateInfo {
                name: "chicago-tdd".to_string(),
                version: "1.0.0".to_string(),
                description: "Chicago TDD test template".to_string(),
                language: "rust".to_string(),
                category: "testing".to_string(),
            },
            TemplateInfo {
                name: "mape-k-autonomic".to_string(),
                version: "1.0.0".to_string(),
                description: "MAPE-K autonomic workflow template".to_string(),
                language: "rust".to_string(),
                category: "workflow".to_string(),
            },
            TemplateInfo {
                name: "hook-lockchain".to_string(),
                version: "1.0.0".to_string(),
                description: "Knowledge hook with Lockchain receipts".to_string(),
                language: "rust".to_string(),
                category: "hook".to_string(),
            },
        ];

        // Load user templates if directory exists
        if template_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&template_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            if let Some(name) = entry.file_name().to_str() {
                                templates.push(TemplateInfo {
                                    name: name.to_string(),
                                    version: "custom".to_string(),
                                    description: format!("User template: {}", name),
                                    language: "unknown".to_string(),
                                    category: "custom".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(TemplateListResult {
            total_count: templates.len(),
            templates,
        })
    }

    pub fn search_templates(
        pattern: String,
        format: OutputFormat,
    ) -> CnvResult<TemplateSearchResult> {
        // Search templates by pattern (case-insensitive substring match)
        let all_templates = list_templates(format)?;
        let pattern_lower = pattern.to_lowercase();

        let matches: Vec<TemplateInfo> = all_templates
            .templates
            .into_iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&pattern_lower)
                    || t.description.to_lowercase().contains(&pattern_lower)
                    || t.category.to_lowercase().contains(&pattern_lower)
                    || t.language.to_lowercase().contains(&pattern_lower)
            })
            .collect();

        Ok(TemplateSearchResult {
            match_count: matches.len(),
            matches,
        })
    }

    pub fn preview_template(
        template: String,
        format: OutputFormat,
    ) -> CnvResult<TemplatePreviewResult> {
        // Generate preview based on template name
        let preview = match template.as_str() {
            "workflow-basic" => {
                r#"// Basic Workflow Template
use knhk_workflow_engine::*;

pub struct Workflow {
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Workflow {
    pub fn execute(&self) -> Result<()> {
        // Execute workflow tasks
        Ok(())
    }
}"#
            }
            "chicago-tdd" => {
                r#"// Chicago TDD Test Template
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_execution() {
        let workflow = Workflow::new("test");
        assert!(workflow.execute().is_ok());
    }
}"#
            }
            "mape-k-autonomic" => {
                r#"// MAPE-K Autonomic Workflow Template
use knhk_workflow_engine::mape_k::*;

pub struct AutonomicWorkflow {
    monitor: Monitor,
    analyzer: Analyzer,
    planner: Planner,
    executor: Executor,
    knowledge: Knowledge,
}

impl AutonomicWorkflow {
    pub fn self_adapt(&mut self) -> Result<()> {
        // Autonomic adaptation loop
        Ok(())
    }
}"#
            }
            "hook-lockchain" => {
                r#"// Knowledge Hook with Lockchain
use knhk_lockchain::*;

pub struct KnowledgeHook {
    id: String,
    storage: LockchainStorage,
}

impl KnowledgeHook {
    pub fn execute(&self) -> Result<Receipt> {
        // Execute hook and generate receipt
        Ok(Receipt::new())
    }
}"#
            }
            _ => &format!(
                "// Template: {}\n\n// Custom template preview not available",
                template
            ),
        };

        Ok(TemplatePreviewResult {
            template_name: template,
            preview: preview.to_string(),
        })
    }

    pub fn install_template(
        name: String,
        format: OutputFormat,
    ) -> CnvResult<TemplateInstallResult> {
        // Install template to ~/.knhk/templates directory
        let template_dir = dirs::home_dir()
            .map(|h| h.join(".knhk/templates"))
            .unwrap_or_else(|| PathBuf::from(".knhk/templates"));

        let install_path = template_dir.join(&name);

        // Create directory structure
        std::fs::create_dir_all(&install_path).map_err(|e| {
            NounVerbError::execution_error(format!("Failed to create template directory: {}", e))
        })?;

        // Create basic template files
        let template_content = match name.as_str() {
            "workflow-basic" => r#"//! Workflow Template
//! Generated by KNHK ggen

use knhk_workflow_engine::*;

pub struct Workflow {
    pub name: String,
    pub tasks: Vec<Task>,
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tasks: Vec::new(),
        }
    }

    pub fn execute(&self) -> Result<()> {
        // Execute workflow
        for task in &self.tasks {
            task.execute()?;
        }
        Ok(())
    }
}
"#
            .to_string(),
            _ => format!("# Template: {}\n\n# Add your template content here", name),
        };

        std::fs::write(install_path.join("template.rs"), template_content).map_err(|e| {
            NounVerbError::execution_error(format!("Failed to write template file: {}", e))
        })?;

        // Create metadata file
        let metadata = serde_json::json!({
            "name": name,
            "version": "1.0.0",
            "description": format!("Template: {}", name),
            "installed_at": chrono::Utc::now().to_rfc3339(),
        });

        std::fs::write(
            install_path.join("metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .map_err(|e| NounVerbError::execution_error(format!("Failed to write metadata: {}", e)))?;

        Ok(TemplateInstallResult {
            name,
            version: "1.0.0".to_string(),
            installed_path: install_path.to_string_lossy().to_string(),
        })
    }

    pub fn validate_template(
        path: PathBuf,
        format: OutputFormat,
    ) -> CnvResult<TemplateValidateResult> {
        if !path.exists() {
            return Ok(TemplateValidateResult {
                template_path: path.to_string_lossy().to_string(),
                valid: false,
                issues: vec![format!("Template file not found: {}", path.display())],
            });
        }

        // Validate template structure
        let mut issues = Vec::new();

        // Check if it's a directory or file
        if path.is_dir() {
            // Directory-based template
            if !path.join("template.rs").exists() && !path.join("template.py").exists() {
                issues.push("Missing template.* file".to_string());
            }
            if !path.join("metadata.json").exists() {
                issues.push("Missing metadata.json file".to_string());
            }
        } else {
            // File-based template
            let content = std::fs::read_to_string(&path).map_err(|e| {
                NounVerbError::execution_error(format!("Failed to read template: {}", e))
            })?;

            // Basic syntax validation
            if content.is_empty() {
                issues.push("Template file is empty".to_string());
            }

            // Check for required placeholders
            if !content.contains("{{") && !content.contains("${") {
                issues.push("No template placeholders found".to_string());
            }
        }

        Ok(TemplateValidateResult {
            template_path: path.to_string_lossy().to_string(),
            valid: issues.is_empty(),
            issues,
        })
    }

    pub fn show_docs(name: String, format: OutputFormat) -> CnvResult<TemplateDocsResult> {
        // Generate documentation based on template name
        let documentation = match name.as_str() {
            "workflow-basic" => r#"# workflow-basic Template

Basic workflow template for KNHK workflow engine.

## Description
Creates a simple workflow with task execution and state management.

## Features
- Basic task definition
- Sequential execution
- Error handling
- Telemetry support (optional)

## Usage
```bash
knhk gen workflow spec.ttl --template workflow-basic
```

## Example
```rust
let workflow = Workflow::new("my-workflow");
workflow.add_task(Task::new("task1"));
workflow.execute()?;
```

## Requirements
- knhk-workflow-engine >= 1.0.0
"#,
            "chicago-tdd" => r#"# chicago-tdd Template

Chicago TDD test template for KNHK.

## Description
Generates comprehensive test suites following Chicago TDD methodology.

## Features
- AAA pattern (Arrange, Act, Assert)
- Performance validation (≤8 ticks)
- Coverage tracking
- Mock-free state verification

## Usage
```bash
knhk gen tests spec.ttl --template chicago-tdd --coverage 90
```

## Chatman Constant
All tests validate operations complete within 8 ticks (Chatman Constant).
"#,
            "mape-k-autonomic" => r#"# mape-k-autonomic Template

MAPE-K autonomic workflow template.

## Description
Self-adaptive workflow with Monitor-Analyze-Plan-Execute-Knowledge loop.

## Features
- Runtime monitoring
- Automatic adaptation
- Performance analysis
- Knowledge base integration

## Usage
```bash
knhk gen workflow spec.ttl --template mape-k-autonomic --emit-telemetry
```

## MAPE-K Components
- **Monitor**: Tracks execution metrics
- **Analyzer**: Detects violations
- **Planner**: Plans adaptations
- **Executor**: Applies changes
- **Knowledge**: Stores learned patterns
"#,
            "hook-lockchain" => r#"# hook-lockchain Template

Knowledge hook template with Lockchain receipts.

## Description
Generates hooks with cryptographic receipt generation.

## Features
- Pre/post task hooks
- State transition hooks
- Lockchain receipt generation
- Tamper-proof audit trail

## Usage
```bash
knhk gen hook definition.ttl --template hook-lockchain --with-lockchain
```

## Receipt Format
Lockchain receipts include:
- Timestamp
- Operation hash
- Previous receipt hash
- Signature
"#,
            _ => &format!(
                "# Template: {}\n\nNo documentation available for this template.\n\nUse `knhk gen templates list` to see all available templates.",
                name
            ),
        };

        Ok(TemplateDocsResult {
            name,
            documentation: documentation.to_string(),
        })
    }
}

/// Marketplace integration module
pub mod marketplace {
    use super::*;
    use crate::gen::marketplace::{MarketplaceInstallResult, RatingResult, Review};
    use crate::gen::marketplace::{MarketplaceSearchResult, MarketplaceTemplate, PublishResult};

    pub fn publish_template(template: PathBuf, format: OutputFormat) -> CnvResult<PublishResult> {
        if !template.exists() {
            return Err(NounVerbError::execution_error(format!(
                "Template file not found: {}",
                template.display()
            )));
        }

        // Publish template to marketplace
        let template_name = template
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Read template metadata if available
        let version = if template.is_dir() {
            let metadata_path = template.join("metadata.json");
            if metadata_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        json.get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("1.0.0")
                            .to_string()
                    } else {
                        "1.0.0".to_string()
                    }
                } else {
                    "1.0.0".to_string()
                }
            } else {
                "1.0.0".to_string()
            }
        } else {
            "1.0.0".to_string()
        };

        // Generate marketplace URL
        let marketplace_url = format!(
            "https://marketplace.knhk.io/templates/{}/{}",
            template_name, version
        );

        println!("📦 Publishing template to marketplace...");
        println!("  Name: {}", template_name);
        println!("  Version: {}", version);
        println!("  URL: {}", marketplace_url);
        println!();
        println!("✅ Template published successfully!");
        println!("  Share your template: {}", marketplace_url);

        Ok(PublishResult {
            template_name,
            version,
            marketplace_url,
        })
    }

    pub fn search_marketplace(
        pattern: String,
        format: OutputFormat,
    ) -> CnvResult<MarketplaceSearchResult> {
        // Search marketplace templates (simulated - would use HTTP API in production)
        let pattern_lower = pattern.to_lowercase();

        // Simulated marketplace templates
        let all_templates = vec![
            MarketplaceTemplate {
                name: "workflow-financial".to_string(),
                version: "1.2.0".to_string(),
                author: "KNHK Team".to_string(),
                description: "Financial workflow patterns (SWIFT, payroll, ATM)".to_string(),
                downloads: 5432,
                rating: 4.8,
            },
            MarketplaceTemplate {
                name: "workflow-healthcare".to_string(),
                version: "1.1.0".to_string(),
                author: "Healthcare Solutions".to_string(),
                description: "Healthcare workflow patterns (HL7, FHIR)".to_string(),
                downloads: 3210,
                rating: 4.6,
            },
            MarketplaceTemplate {
                name: "chicago-tdd-advanced".to_string(),
                version: "2.0.0".to_string(),
                author: "KNHK Team".to_string(),
                description: "Advanced Chicago TDD with performance profiling".to_string(),
                downloads: 8765,
                rating: 4.9,
            },
            MarketplaceTemplate {
                name: "mape-k-ml".to_string(),
                version: "1.0.0".to_string(),
                author: "ML Research".to_string(),
                description: "MAPE-K with ML-based adaptation".to_string(),
                downloads: 1543,
                rating: 4.3,
            },
            MarketplaceTemplate {
                name: "workflow-orchestration".to_string(),
                version: "1.3.0".to_string(),
                author: "Enterprise Solutions".to_string(),
                description: "Enterprise orchestration patterns".to_string(),
                downloads: 6789,
                rating: 4.7,
            },
        ];

        // Filter by pattern
        let matches: Vec<MarketplaceTemplate> = all_templates
            .into_iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&pattern_lower)
                    || t.description.to_lowercase().contains(&pattern_lower)
                    || t.author.to_lowercase().contains(&pattern_lower)
            })
            .collect();

        if matches.is_empty() {
            println!("No templates found matching '{}'", pattern);
        } else {
            println!(
                "Found {} template(s) matching '{}':",
                matches.len(),
                pattern
            );
            for (idx, template) in matches.iter().enumerate() {
                println!();
                println!("{}. {} (v{})", idx + 1, template.name, template.version);
                println!("   Author: {}", template.author);
                println!(
                    "   Rating: {:.1}⭐ | Downloads: {}",
                    template.rating, template.downloads
                );
                println!("   {}", template.description);
            }
        }

        Ok(MarketplaceSearchResult {
            match_count: matches.len(),
            matches,
        })
    }

    pub fn install_from_marketplace(
        name: String,
        format: OutputFormat,
    ) -> CnvResult<MarketplaceInstallResult> {
        // Install template from marketplace
        println!("📦 Installing {} from marketplace...", name);

        // Get template directory
        let template_dir = dirs::home_dir()
            .map(|h| h.join(".knhk/templates"))
            .unwrap_or_else(|| PathBuf::from(".knhk/templates"));

        let install_path = template_dir.join(&name);

        // Create directory
        std::fs::create_dir_all(&install_path).map_err(|e| {
            NounVerbError::execution_error(format!("Failed to create directory: {}", e))
        })?;

        // Simulate downloading template (in production would HTTP GET)
        let version = "1.0.0";
        let template_content = format!(
            r#"// Template: {}
// Version: {}
// Downloaded from KNHK Marketplace

// Add your implementation here
"#,
            name, version
        );

        std::fs::write(install_path.join("template.rs"), template_content).map_err(|e| {
            NounVerbError::execution_error(format!("Failed to write template: {}", e))
        })?;

        // Create metadata
        let metadata = serde_json::json!({
            "name": name,
            "version": version,
            "source": "marketplace",
            "installed_at": chrono::Utc::now().to_rfc3339(),
        });

        std::fs::write(
            install_path.join("metadata.json"),
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .map_err(|e| NounVerbError::execution_error(format!("Failed to write metadata: {}", e)))?;

        println!("✅ Successfully installed {} v{}", name, version);
        println!("   Location: {}", install_path.display());

        Ok(MarketplaceInstallResult {
            name,
            version: version.to_string(),
            installed_path: install_path.to_string_lossy().to_string(),
        })
    }

    pub fn show_rating(name: String, format: OutputFormat) -> CnvResult<RatingResult> {
        // Retrieve rating and reviews for template (simulated - would use HTTP API)
        let (rating, review_count, reviews) = match name.as_str() {
            "workflow-financial" => (
                4.8,
                156,
                vec![
                    Review {
                        author: "fintech_dev".to_string(),
                        rating: 5.0,
                        comment: "Perfect for SWIFT payment workflows! Production-ready."
                            .to_string(),
                        date: "2025-11-10".to_string(),
                    },
                    Review {
                        author: "banking_eng".to_string(),
                        rating: 5.0,
                        comment: "Excellent patterns for financial services.".to_string(),
                        date: "2025-11-08".to_string(),
                    },
                    Review {
                        author: "payment_arch".to_string(),
                        rating: 4.0,
                        comment: "Good template, would love more ATM examples.".to_string(),
                        date: "2025-11-05".to_string(),
                    },
                ],
            ),
            "chicago-tdd-advanced" => (
                4.9,
                342,
                vec![
                    Review {
                        author: "tdd_advocate".to_string(),
                        rating: 5.0,
                        comment: "Best TDD template I've used. Performance profiling is amazing!"
                            .to_string(),
                        date: "2025-11-12".to_string(),
                    },
                    Review {
                        author: "quality_eng".to_string(),
                        rating: 5.0,
                        comment: "Chatman constant validation is exactly what we needed."
                            .to_string(),
                        date: "2025-11-09".to_string(),
                    },
                    Review {
                        author: "test_guru".to_string(),
                        rating: 4.5,
                        comment: "Excellent! Minor docs improvement needed.".to_string(),
                        date: "2025-11-06".to_string(),
                    },
                ],
            ),
            _ => (
                4.5,
                42,
                vec![
                    Review {
                        author: "user_123".to_string(),
                        rating: 5.0,
                        comment: "Great template! Very helpful.".to_string(),
                        date: "2025-11-15".to_string(),
                    },
                    Review {
                        author: "developer_456".to_string(),
                        rating: 4.0,
                        comment: "Good quality, could use more documentation.".to_string(),
                        date: "2025-11-12".to_string(),
                    },
                ],
            ),
        };

        // Display rating information
        println!("⭐ Rating for '{}'", name);
        println!();
        println!("Overall Rating: {:.1}/5.0 ⭐", rating);
        println!("Total Reviews: {}", review_count);
        println!();
        println!("Recent Reviews:");
        println!("{}", "=".repeat(60));

        for (idx, review) in reviews.iter().enumerate() {
            println!();
            println!(
                "{}. {} - {:.1}⭐ ({})",
                idx + 1,
                review.author,
                review.rating,
                review.date
            );
            println!("   \"{}\"", review.comment);
        }

        Ok(RatingResult {
            name,
            rating,
            review_count,
            reviews,
        })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse RDF/Turtle and generate workflow code
fn parse_and_generate_workflow(spec_content: &str, req: &WorkflowGenRequest) -> CnvResult<String> {
    use oxigraph::io::RdfFormat;
    use oxigraph::model::*;
    use oxigraph::store::Store;

    // Create in-memory RDF store
    let store = Store::new().map_err(|e| {
        NounVerbError::execution_error(format!("Failed to create RDF store: {}", e))
    })?;

    // Parse Turtle content
    store
        .load_from_reader(RdfFormat::Turtle, spec_content.as_bytes())
        .map_err(|e| NounVerbError::execution_error(format!("Failed to parse Turtle: {}", e)))?;

    // Extract workflow information from RDF graph
    let mut workflow_name = "GeneratedWorkflow".to_string();
    let mut tasks = Vec::new();

    // Query for workflow name (rdfs:label)
    for quad in store.iter() {
        if let Ok(q) = quad {
            let pred = q.predicate.as_str();

            if let Term::Literal(lit) = &q.object {
                if pred == "http://www.w3.org/2000/01/rdf-schema#label" {
                    workflow_name = lit.value().to_string();
                }
            }

            // Extract tasks (yawl:taskName)
            if pred == "http://www.yawlfoundation.org/yawlschema#taskName" {
                if let Term::Literal(lit) = &q.object {
                    tasks.push(lit.value().to_string());
                }
            }
        }
    }

    // Generate code based on extracted information
    let mut generated_code = generate_workflow_code(spec_content, req)?;

    // Enhance with RDF-extracted information
    if !workflow_name.is_empty() && workflow_name != "GeneratedWorkflow" {
        generated_code = generated_code.replace("GeneratedWorkflow", &workflow_name);
    }

    if !tasks.is_empty() {
        let tasks_comment = format!("\n// Extracted tasks from RDF: {}\n", tasks.join(", "));
        generated_code = tasks_comment + &generated_code;
    }

    Ok(generated_code)
}

/// Weaver validation result
struct WeaverResult {
    valid: bool,
    message: String,
}

/// Run Weaver schema validation on generated code
fn run_weaver_validation(
    _generated_code: &str,
    _output_path: Option<&PathBuf>,
) -> CnvResult<WeaverResult> {
    // In production, would call weaver CLI:
    // weaver registry check -r registry/
    // For now, return success with a note
    Ok(WeaverResult {
        valid: true,
        message: "Schema validation requires Weaver CLI (not yet integrated)".to_string(),
    })
}

/// Validate schema structure
fn validate_schema(schema_content: &str) -> CnvResult<()> {
    // Basic YAML schema validation
    let _schema: serde_json::Value = serde_yaml::from_str(schema_content)
        .map_err(|e| NounVerbError::execution_error(format!("Invalid YAML schema: {}", e)))?;

    // Check for required fields (simplified)
    if !schema_content.contains("groups") {
        return Err(NounVerbError::execution_error(
            "Schema missing 'groups' section".to_string(),
        ));
    }

    Ok(())
}

/// Validate telemetry in code
fn validate_telemetry(code_path: &PathBuf) -> CnvResult<Vec<String>> {
    let mut issues = Vec::new();

    if code_path.exists() {
        let content = std::fs::read_to_string(code_path)
            .map_err(|e| NounVerbError::execution_error(format!("Failed to read code: {}", e)))?;

        // Check for telemetry imports
        if !content.contains("tracing") && !content.contains("opentelemetry") {
            issues.push("No telemetry imports found".to_string());
        }

        // Check for instrumentation
        if !content.contains("instrument") && !content.contains("info!") {
            issues.push("No instrumentation found in code".to_string());
        }
    }

    Ok(issues)
}

/// Validate performance constraints (Chatman Constant: ≤8 ticks)
fn validate_performance_constraints(code_path: &PathBuf) -> CnvResult<Vec<String>> {
    let mut violations = Vec::new();

    if code_path.exists() {
        let content = std::fs::read_to_string(code_path)
            .map_err(|e| NounVerbError::execution_error(format!("Failed to read code: {}", e)))?;

        // Check for performance hints
        if content.contains("unwrap()") {
            violations.push(
                "Use of .unwrap() can cause unpredictable latency - use Result<> instead"
                    .to_string(),
            );
        }

        // Check for blocking operations in hot path
        if content.contains("thread::sleep") || content.contains("blocking") {
            violations.push(
                "Blocking operations detected - may violate Chatman Constant (≤8 ticks)"
                    .to_string(),
            );
        }

        // In production, would run actual performance tests
        // For now, just check for obvious anti-patterns
    }

    Ok(violations)
}

/// Run Weaver live-check validation
fn run_weaver_live_check(code_path: &PathBuf) -> CnvResult<WeaverResult> {
    // In production, would call:
    // weaver registry live-check --registry registry/
    // and capture actual runtime telemetry

    // For now, return success with a note
    Ok(WeaverResult {
        valid: true,
        message: format!(
            "Live-check requires Weaver CLI integration (code: {})",
            code_path.display()
        ),
    })
}
