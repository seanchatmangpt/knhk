//! RDF support for workflow patterns
//!
//! Provides RDF serialization and metadata for workflow patterns:
//! - Pattern metadata extraction
//! - Execution context serialization to RDF
//! - Execution result serialization to RDF
//! - Pattern ontology support

use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};
use std::collections::HashMap;

/// Workflow pattern namespace
pub const WORKFLOW_PATTERN_NS: &str = "http://bitflow.ai/ontology/workflow-pattern/v1#";

/// YAWL namespace
pub const YAWL_NS: &str = "http://bitflow.ai/ontology/yawl/v2#";

/// Pattern metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PatternMetadata {
    /// Pattern ID (1-43)
    pub pattern_id: u32,
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Pattern category
    pub category: String,
    /// Pattern complexity
    pub complexity: String,
    /// Pattern dependencies (other pattern IDs)
    pub dependencies: Vec<u32>,
}

impl PatternMetadata {
    /// Create pattern metadata
    pub fn new(
        pattern_id: u32,
        name: String,
        description: String,
        category: String,
        complexity: String,
        dependencies: Vec<u32>,
    ) -> Self {
        Self {
            pattern_id,
            name,
            description,
            category,
            complexity,
            dependencies,
        }
    }
}

/// Get all pattern metadata
pub fn get_all_pattern_metadata() -> Vec<PatternMetadata> {
    vec![
        // Basic Control Flow (1-5)
        PatternMetadata::new(
            1,
            "Sequence".to_string(),
            "Execute activities in strict sequential order".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            2,
            "Parallel Split".to_string(),
            "Split execution into multiple parallel branches".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            3,
            "Synchronization".to_string(),
            "Synchronize multiple parallel branches".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![2],
        ),
        PatternMetadata::new(
            4,
            "Exclusive Choice".to_string(),
            "Choose one branch based on condition".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            5,
            "Simple Merge".to_string(),
            "Merge multiple branches into one".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![4],
        ),
        // Advanced Branching (6-11)
        PatternMetadata::new(
            6,
            "Multi-Choice".to_string(),
            "Choose multiple branches based on conditions".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            7,
            "Structured Synchronizing Merge".to_string(),
            "Synchronize multiple branches with structured merge".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![6],
        ),
        PatternMetadata::new(
            8,
            "Multi-Merge".to_string(),
            "Merge multiple branches without synchronization".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            9,
            "Discriminator".to_string(),
            "Wait for first branch to complete, then continue".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            10,
            "Arbitrary Cycles".to_string(),
            "Support arbitrary cycles in workflow".to_string(),
            "Advanced Branching".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            11,
            "Implicit Termination".to_string(),
            "Terminate when all active branches complete".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        // Multiple Instance (12-15)
        PatternMetadata::new(
            12,
            "MI Without Sync".to_string(),
            "Multiple instances without synchronization".to_string(),
            "Multiple Instance".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            13,
            "MI With Design-Time Knowledge".to_string(),
            "Multiple instances with known count at design time".to_string(),
            "Multiple Instance".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            14,
            "MI With Runtime Knowledge".to_string(),
            "Multiple instances with count known at runtime".to_string(),
            "Multiple Instance".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            15,
            "MI Without Runtime Knowledge".to_string(),
            "Multiple instances with unknown count".to_string(),
            "Multiple Instance".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        // State-Based (16-18)
        PatternMetadata::new(
            16,
            "Deferred Choice".to_string(),
            "Choose branch based on external event".to_string(),
            "State-Based".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            17,
            "Interleaved Parallel Routing".to_string(),
            "Execute branches in interleaved order".to_string(),
            "State-Based".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            18,
            "Milestone".to_string(),
            "Enable activity when milestone is reached".to_string(),
            "State-Based".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        // Cancellation (19-25)
        PatternMetadata::new(
            19,
            "Cancel Activity".to_string(),
            "Cancel a specific activity".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            20,
            "Cancel Case".to_string(),
            "Cancel entire workflow case".to_string(),
            "Cancellation".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            21,
            "Cancel Region".to_string(),
            "Cancel a region of activities".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            22,
            "Cancel MI Activity".to_string(),
            "Cancel multiple instance activity".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![12, 13, 14, 15],
        ),
        PatternMetadata::new(
            23,
            "Complete MI Activity".to_string(),
            "Complete multiple instance activity".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![12, 13, 14, 15],
        ),
        PatternMetadata::new(
            24,
            "Blocking Discriminator".to_string(),
            "Wait for first branch, block others".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![9],
        ),
        PatternMetadata::new(
            25,
            "Cancelling Discriminator".to_string(),
            "Wait for first branch, cancel others".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![9],
        ),
        // Advanced Control (26-39) - Placeholder metadata
        PatternMetadata::new(
            26,
            "Pattern 26".to_string(),
            "Advanced control pattern 26".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            27,
            "Pattern 27".to_string(),
            "Advanced control pattern 27".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            28,
            "Pattern 28".to_string(),
            "Advanced control pattern 28".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            29,
            "Pattern 29".to_string(),
            "Advanced control pattern 29".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            30,
            "Pattern 30".to_string(),
            "Advanced control pattern 30".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            31,
            "Pattern 31".to_string(),
            "Advanced control pattern 31".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            32,
            "Pattern 32".to_string(),
            "Advanced control pattern 32".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            33,
            "Pattern 33".to_string(),
            "Advanced control pattern 33".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            34,
            "Pattern 34".to_string(),
            "Advanced control pattern 34".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            35,
            "Pattern 35".to_string(),
            "Advanced control pattern 35".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            36,
            "Pattern 36".to_string(),
            "Advanced control pattern 36".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            37,
            "Pattern 37".to_string(),
            "Advanced control pattern 37".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            38,
            "Pattern 38".to_string(),
            "Advanced control pattern 38".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            39,
            "Pattern 39".to_string(),
            "Advanced control pattern 39".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        // Trigger Patterns (40-43)
        PatternMetadata::new(
            40,
            "Pattern 40".to_string(),
            "Trigger pattern 40".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            41,
            "Pattern 41".to_string(),
            "Trigger pattern 41".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            42,
            "Pattern 42".to_string(),
            "Trigger pattern 42".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            43,
            "Pattern 43".to_string(),
            "Trigger pattern 43".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
    ]
}

/// Serialize pattern execution context to RDF/Turtle
pub fn serialize_context_to_rdf(
    pattern_id: &PatternId,
    context: &PatternExecutionContext,
) -> WorkflowResult<String> {
    let pattern_ns = WORKFLOW_PATTERN_NS;
    let yawl_ns = YAWL_NS;
    let pattern_iri = format!("{}pattern:{}", pattern_ns, pattern_id.0);
    let context_iri = format!(
        "{}execution:{}:{}",
        pattern_ns, context.case_id, context.workflow_id
    );

    let mut turtle = format!(
        "@prefix pattern: <{}> .\n\
         @prefix yawl: <{}> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n",
        pattern_ns, yawl_ns
    );

    turtle.push_str(&format!(
        "<{}> rdf:type pattern:PatternExecution ;\n",
        context_iri
    ));
    turtle.push_str(&format!(
        "    pattern:executesPattern <{}> ;\n",
        pattern_iri
    ));
    turtle.push_str(&format!(
        "    yawl:hasCase <{}> ;\n",
        format!("{}case:{}", yawl_ns, context.case_id)
    ));
    turtle.push_str(&format!(
        "    yawl:hasWorkflowSpec <{}> ;\n",
        format!("{}workflow:{}", yawl_ns, context.workflow_id)
    ));

    if !context.variables.is_empty() {
        turtle.push_str("    pattern:hasVariables [\n");
        for (key, value) in &context.variables {
            turtle.push_str(&format!(
                "        pattern:variable \"{}\" \"{}\" ;\n",
                escape_string(key),
                escape_string(value)
            ));
        }
        turtle.push_str("    ] ;\n");
    }

    turtle.push_str("    .\n\n");

    turtle.push_str(&format!("<{}> rdf:type pattern:Pattern ;\n", pattern_iri));
    turtle.push_str(&format!("    pattern:patternId {} ;\n", pattern_id.0));
    turtle.push_str("    .\n");

    Ok(turtle)
}

/// Serialize pattern execution result to RDF/Turtle
pub fn serialize_result_to_rdf(
    pattern_id: &PatternId,
    context: &PatternExecutionContext,
    result: &PatternExecutionResult,
) -> WorkflowResult<String> {
    let pattern_ns = WORKFLOW_PATTERN_NS;
    let yawl_ns = YAWL_NS;
    let result_iri = format!(
        "{}result:{}:{}:{}",
        pattern_ns, pattern_id.0, context.case_id, context.workflow_id
    );

    let mut turtle = format!(
        "@prefix pattern: <{}> .\n\
         @prefix yawl: <{}> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n",
        pattern_ns, yawl_ns
    );

    turtle.push_str(&format!(
        "<{}> rdf:type pattern:PatternExecutionResult ;\n",
        result_iri
    ));
    turtle.push_str(&format!(
        "    pattern:success {} ;\n",
        if result.success { "true" } else { "false" }
    ));

    if let Some(ref next_state) = result.next_state {
        turtle.push_str(&format!(
            "    pattern:nextState \"{}\" ;\n",
            escape_string(next_state)
        ));
    }

    if !result.variables.is_empty() {
        turtle.push_str("    pattern:hasOutputVariables [\n");
        for (key, value) in &result.variables {
            turtle.push_str(&format!(
                "        pattern:variable \"{}\" \"{}\" ;\n",
                escape_string(key),
                escape_string(value)
            ));
        }
        turtle.push_str("    ] ;\n");
    }

    turtle.push_str("    .\n");

    Ok(turtle)
}

/// Escape string for Turtle/RDF
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
