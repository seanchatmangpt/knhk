//! RDF serialization functions

use super::metadata::PatternMetadata;
use super::utils::escape_string;
use super::{WORKFLOW_PATTERN_NS, YAWL_NS};
use crate::error::WorkflowResult;
use crate::patterns::{PatternExecutionContext, PatternExecutionResult, PatternId};

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

/// Serialize pattern metadata to RDF/Turtle
pub fn serialize_metadata_to_rdf(metadata: &PatternMetadata) -> WorkflowResult<String> {
    let pattern_ns = WORKFLOW_PATTERN_NS;
    let pattern_iri = format!("{}pattern:{}", pattern_ns, metadata.pattern_id);

    let mut turtle = format!(
        "@prefix pattern: <{}> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n",
        pattern_ns
    );

    turtle.push_str(&format!(
        "<{}> rdf:type pattern:WorkflowPattern ;\n",
        pattern_iri
    ));
    turtle.push_str(&format!(
        "    pattern:patternId {} ;\n",
        metadata.pattern_id
    ));
    turtle.push_str(&format!(
        "    rdfs:label \"{}\" ;\n",
        escape_string(&metadata.name)
    ));
    turtle.push_str(&format!(
        "    rdfs:description \"{}\" ;\n",
        escape_string(&metadata.description)
    ));
    turtle.push_str(&format!(
        "    pattern:category \"{}\" ;\n",
        escape_string(&metadata.category)
    ));
    turtle.push_str(&format!(
        "    pattern:complexity \"{}\" ;\n",
        escape_string(&metadata.complexity)
    ));

    if !metadata.dependencies.is_empty() {
        turtle.push_str("    pattern:dependsOn ");
        for (i, dep_id) in metadata.dependencies.iter().enumerate() {
            if i > 0 {
                turtle.push_str(", ");
            }
            turtle.push_str(&format!("pattern:pattern:{}", dep_id));
        }
        turtle.push_str(" ;\n");
    }

    turtle.push_str("    .\n");

    Ok(turtle)
}
