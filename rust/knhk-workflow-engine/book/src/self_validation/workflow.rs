//! Self-validation workflow definition
//!
//! Defines the workflow specification used for self-validation.

/// Self-validation workflow in Turtle format
pub const SELF_VALIDATION_WORKFLOW: &str = r#"
@prefix yawl: <http://www.yawlfoundation.org/xsd/yawl_20#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://knhk.org/workflow/self-validation>
    rdf:type yawl:WorkflowSpecification ;
    rdfs:label "Self-Validation Workflow" ;
    rdfs:comment "Workflow that uses the engine to validate itself" ;
    yawl:hasStartCondition <http://knhk.org/workflow/self-validation/start> ;
    yawl:hasEndCondition <http://knhk.org/workflow/self-validation/end> .

<http://knhk.org/workflow/self-validation/start>
    rdf:type yawl:Condition ;
    rdfs:label "Start Validation" .

<http://knhk.org/workflow/self-validation/validate-patterns>
    rdf:type yawl:Task ;
    rdf:type yawl:AtomicTask ;
    rdfs:label "Validate All 43 Patterns" ;
    rdfs:comment "Validates that all 43 workflow patterns are registered and functional" ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" ;
    yawl:maxTicks 8 ;
    yawl:useSimd true .

<http://knhk.org/workflow/self-validation/validate-capabilities>
    rdf:type yawl:Task ;
    rdf:type yawl:AtomicTask ;
    rdfs:label "Validate Capabilities" ;
    rdfs:comment "Validates that all required capabilities are available" ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" ;
    yawl:maxTicks 8 ;
    yawl:useSimd true .

<http://knhk.org/workflow/self-validation/validate-engine>
    rdf:type yawl:Task ;
    rdf:type yawl:AtomicTask ;
    rdfs:label "Validate Engine" ;
    rdfs:comment "Validates that the engine can create and execute cases" ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" ;
    yawl:maxTicks 8 ;
    yawl:useSimd true .

<http://knhk.org/workflow/self-validation/end>
    rdf:type yawl:Condition ;
    rdfs:label "Validation Complete" .

<http://knhk.org/workflow/self-validation/flow1>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/start> ;
    yawl:target <http://knhk.org/workflow/self-validation/validate-patterns> .

<http://knhk.org/workflow/self-validation/flow2>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/validate-patterns> ;
    yawl:target <http://knhk.org/workflow/self-validation/validate-capabilities> .

<http://knhk.org/workflow/self-validation/flow3>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/validate-capabilities> ;
    yawl:target <http://knhk.org/workflow/self-validation/validate-engine> .

<http://knhk.org/workflow/self-validation/flow4>
    rdf:type yawl:Flow ;
    yawl:source <http://knhk.org/workflow/self-validation/validate-engine> ;
    yawl:target <http://knhk.org/workflow/self-validation/end> .
"#;
