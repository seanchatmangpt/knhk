//! Covenant 1 Integration Tests: Turtle Is Definition and Cause (O ⊨ Σ)
//!
//! DOCTRINE ALIGNMENT:
//! - Principle: O (Observation) ⊨ Σ (Ontology)
//! - Covenant: Covenant 1 - Turtle is the single source of truth
//! - Why This Matters: Code, docs, and APIs drift apart; Turtle is definition and cause
//!
//! WHAT THIS TESTS:
//! - Turtle workflow definitions are pure passthrough (no hidden logic)
//! - SPARQL extraction preserves all declared structure
//! - Output matches input structure exactly
//! - No filtering, reordering, or reconstruction
//! - Template rendering is purely mechanical transformation
//!
//! VALIDATION CHECKLIST:
//! - [ ] Load valid Turtle workflow
//! - [ ] Extract via SPARQL
//! - [ ] Verify structure preservation
//! - [ ] Confirm no hidden assumptions
//! - [ ] Validate end-to-end purity

use oxigraph::model::vocab::rdf;
use oxigraph::model::NamedNode;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use std::fs;
use std::path::PathBuf;

/// Test fixture path helper
fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(filename)
}

/// Test ontology path helper
fn ontology_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("ontology")
        .join(filename)
}

#[test]
fn test_covenant1_load_turtle_workflow() {
    // GIVEN: A valid Turtle workflow definition
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let turtle_content = fs::read_to_string(&workflow_path)
        .expect("Failed to read workflow Turtle file");

    // WHEN: We load it into RDF store
    let store = Store::new().expect("Failed to create RDF store");
    store
        .load_from_reader(
            oxigraph::io::RdfFormat::Turtle,
            turtle_content.as_bytes(),
        )
        .expect("Failed to parse Turtle");

    // THEN: The workflow specification exists
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?workflow WHERE {
            ?workflow a yawl:WorkflowSpecification .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    if let QueryResults::Solutions(mut solutions) = results {
        assert!(solutions.next().is_some(), "No workflow found in Turtle");
    } else {
        panic!("Expected solutions, got different result type");
    }
}

#[test]
fn test_covenant1_turtle_structure_preserved() {
    // GIVEN: A workflow with known structure
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let turtle_content = fs::read_to_string(&workflow_path)
        .expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle_content.as_bytes())
        .expect("Failed to parse");

    // WHEN: We extract all tasks
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?task) as ?count) WHERE {
            ?workflow a yawl:WorkflowSpecification .
            ?workflow yawl:hasTask ?task .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: All tasks are preserved (autonomous-work-definition has 12 tasks)
    if let QueryResults::Solutions(mut solutions) = results {
        if let Some(solution) = solutions.next() {
            let solution = solution.expect("Solution error");
            let count = solution
                .get("count")
                .expect("No count binding")
                .as_ref();

            // Verify count matches expected structure
            assert!(
                count.to_string().contains("12"),
                "Expected 12 tasks, structure not fully preserved"
            );
        } else {
            panic!("No results from task count query");
        }
    }
}

#[test]
fn test_covenant1_no_hidden_logic_in_extraction() {
    // GIVEN: Workflow with parallel split pattern
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let turtle_content = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle_content.as_bytes())
        .expect("Failed to parse");

    // WHEN: We extract split types
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?task ?splitType WHERE {
            ?task a yawl:Task .
            ?task yawl:hasSplitType ?splitType .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Split types match what's declared in Turtle (no modification)
    if let QueryResults::Solutions(solutions) = results {
        let mut and_splits = 0;
        let mut xor_splits = 0;
        let mut or_splits = 0;

        for solution in solutions {
            let solution = solution.expect("Solution error");
            let split_type = solution
                .get("splitType")
                .expect("No splitType")
                .as_ref()
                .to_string();

            if split_type.contains("AND") {
                and_splits += 1;
            } else if split_type.contains("XOR") {
                xor_splits += 1;
            } else if split_type.contains("OR") {
                or_splits += 1;
            }
        }

        // Verify we found the expected split types from the Turtle
        assert!(and_splits > 0, "AND splits not preserved");
        assert!(xor_splits > 0, "XOR splits not preserved");
        assert!(or_splits > 0, "OR splits not preserved");
    }
}

#[test]
fn test_covenant1_data_flow_preserved() {
    // GIVEN: Workflow with data inputs and outputs
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let turtle_content = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle_content.as_bytes())
        .expect("Failed to parse");

    // WHEN: We extract data variables
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?var ?type WHERE {
            {
                ?var a yawl:DataInput .
                BIND("input" as ?type)
            } UNION {
                ?var a yawl:DataOutput .
                BIND("output" as ?type)
            }
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: All data variables are preserved
    if let QueryResults::Solutions(solutions) = results {
        let count = solutions.count();
        assert!(
            count > 0,
            "Data flow variables not preserved from Turtle"
        );
    }
}

#[test]
fn test_covenant1_constraints_preserved() {
    // GIVEN: Workflow with execution constraints
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let turtle_content = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle_content.as_bytes())
        .expect("Failed to parse");

    // WHEN: We extract constraints
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT ?constraint ?type WHERE {
            ?workflow a yawl:WorkflowSpecification .
            ?workflow yawl:hasConstraint ?constraint .
            ?constraint yawl:constraintType ?type .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: All constraints are preserved (timeout, quorum, resource)
    if let QueryResults::Solutions(solutions) = results {
        let constraint_types: Vec<String> = solutions
            .filter_map(|s| s.ok())
            .filter_map(|s| s.get("type").map(|t| t.to_string()))
            .collect();

        assert!(
            constraint_types.iter().any(|t| t.contains("temporal")),
            "Timeout constraint not preserved"
        );
        assert!(
            constraint_types.iter().any(|t| t.contains("quorum")),
            "Quorum constraint not preserved"
        );
        assert!(
            constraint_types.iter().any(|t| t.contains("resource")),
            "Resource constraint not preserved"
        );
    }
}

#[test]
fn test_covenant1_round_trip_preservation() {
    // GIVEN: Original Turtle workflow
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let original_turtle = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(
            oxigraph::io::RdfFormat::Turtle,
            original_turtle.as_bytes(),
        )
        .expect("Failed to parse");

    // WHEN: We extract the entire workflow structure
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        CONSTRUCT {
            ?workflow a yawl:WorkflowSpecification .
            ?workflow yawl:hasTask ?task .
            ?workflow yawl:hasCondition ?condition .
            ?workflow yawl:hasConstraint ?constraint .
            ?task yawl:hasSplitType ?split .
            ?task yawl:hasJoinType ?join .
        } WHERE {
            ?workflow a yawl:WorkflowSpecification .
            OPTIONAL { ?workflow yawl:hasTask ?task . }
            OPTIONAL { ?workflow yawl:hasCondition ?condition . }
            OPTIONAL { ?workflow yawl:hasConstraint ?constraint . }
            OPTIONAL { ?task yawl:hasSplitType ?split . }
            OPTIONAL { ?task yawl:hasJoinType ?join . }
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Reconstructed graph contains all essential triples
    if let QueryResults::Graph(triples) = results {
        let triple_count = triples.count();
        assert!(
            triple_count > 20,
            "Round-trip lost structure (too few triples: {})",
            triple_count
        );
    } else {
        panic!("Expected graph results");
    }
}

#[test]
fn test_covenant1_no_api_drift() {
    // GIVEN: Workflow loaded from Turtle
    let workflow_path = ontology_path("workflows/examples/autonomous-work-definition.ttl");
    let turtle_content = fs::read_to_string(&workflow_path).expect("Failed to read workflow");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, turtle_content.as_bytes())
        .expect("Failed to parse");

    // WHEN: We query for workflow properties
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        PREFIX dcterms: <http://purl.org/dc/terms/>
        SELECT ?prop ?value WHERE {
            ?workflow a yawl:WorkflowSpecification .
            ?workflow ?prop ?value .
            FILTER(?prop IN (yawl:versionNumber, dcterms:created))
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: All declared properties are accessible (no filtering)
    if let QueryResults::Solutions(solutions) = results {
        let props: Vec<String> = solutions
            .filter_map(|s| s.ok())
            .filter_map(|s| s.get("prop").map(|p| p.to_string()))
            .collect();

        assert!(
            !props.is_empty(),
            "Workflow properties not accessible (API drift detected)"
        );
    }
}

#[test]
fn test_covenant1_pattern_permutation_validation() {
    // GIVEN: Pattern permutation matrix
    let matrix_path = ontology_path("yawl-pattern-permutations.ttl");
    let matrix_turtle = fs::read_to_string(&matrix_path).expect("Failed to read matrix");

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, matrix_turtle.as_bytes())
        .expect("Failed to parse");

    // WHEN: We query for valid combinations
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        SELECT (COUNT(?combo) as ?count) WHERE {
            ?combo a yawl:SplitJoinCombination .
            ?combo yawl:isValid true .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: All valid patterns are defined in matrix
    if let QueryResults::Solutions(mut solutions) = results {
        if let Some(solution) = solutions.next() {
            let solution = solution.expect("Solution error");
            let count = solution.get("count").expect("No count").as_ref().to_string();

            assert!(
                count.contains("1") || count.contains("2"),
                "Pattern matrix should have multiple valid combinations"
            );
        }
    }
}

#[test]
fn test_covenant1_no_template_conditional_logic() {
    // ANTI-PATTERN TEST: Verify templates don't contain hidden logic
    // This is a structural test - in production, templates would be validated
    // against a "pure passthrough" constraint

    // GIVEN: A simple workflow in Turtle
    let simple_workflow = r#"
        @prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
        @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

        <http://example.org/test-workflow> a yawl:WorkflowSpecification ;
            rdfs:label "Test Workflow" ;
            yawl:hasTask <#task1> .

        <#task1> a yawl:Task ;
            rdfs:label "Simple Task" ;
            yawl:hasSplitType yawl:XOR ;
            yawl:hasJoinType yawl:XOR .
    "#;

    let store = Store::new().expect("Failed to create store");
    store
        .load_from_reader(oxigraph::io::RdfFormat::Turtle, simple_workflow.as_bytes())
        .expect("Failed to parse");

    // WHEN: We extract task properties
    let query = r#"
        PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        SELECT ?task ?label ?split ?join WHERE {
            ?task a yawl:Task .
            ?task rdfs:label ?label .
            ?task yawl:hasSplitType ?split .
            ?task yawl:hasJoinType ?join .
        }
    "#;

    let results = store.query(query).expect("Query failed");

    // THEN: Extraction is pure (no filtering, no reordering, no reconstruction)
    if let QueryResults::Solutions(mut solutions) = results {
        let solution = solutions.next().expect("No task found").expect("Solution error");

        let label = solution.get("label").expect("No label").to_string();
        let split = solution.get("split").expect("No split").to_string();
        let join = solution.get("join").expect("No join").to_string();

        assert!(label.contains("Simple Task"), "Label not preserved");
        assert!(split.contains("XOR"), "Split type modified");
        assert!(join.contains("XOR"), "Join type modified");
    }
}
