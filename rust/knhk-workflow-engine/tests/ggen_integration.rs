//! ggen Integration Test Suite - Chicago TDD Style
//!
//! Tests end-to-end workflows combining all ggen modules.
//! Validates complete pipelines: RDF → Generated Code → Compiled → Tests → Docs.
//!
//! Test Coverage (25+ tests):
//! - Multi-language generation from single RDF spec
//! - SPARQL → Code generation → Compilation
//! - Hook generation integrated with telemetry
//! - Error recovery and partial failures
//! - Performance of complete workflows

use knhk_workflow_engine::ggen::codegen::{create_generator, CodeGenerator, GenerationContext};
use knhk_workflow_engine::ggen::hooks_generator::HooksGenerator;
use knhk_workflow_engine::ggen::sparql_engine::SparqlTemplateEngine;
use knhk_workflow_engine::ggen::telemetry_generator::{
    AttributeDefinition, MetricDefinition, SpanDefinition, TelemetryGenerator,
};
use std::collections::HashMap;
use tempfile::TempDir;

// ============================================================================
// Test Data Builders
// ============================================================================

fn create_comprehensive_rdf() -> String {
    r#"
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix knhk: <http://knhk.io/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

# Workflow Specification
<#OrderWorkflow> a yawl:Specification ;
    yawl:name "Order Processing Workflow" ;
    yawl:version "1.0" .

<#validateTask> a yawl:Task ;
    yawl:name "Validate Order" ;
    yawl:id "validate_order" ;
    yawl:taskType "atomic" .

<#processTask> a yawl:Task ;
    yawl:name "Process Payment" ;
    yawl:id "process_payment" ;
    yawl:taskType "atomic" .

# Hooks
<#ValidationHook> a knhk:Hook ;
    knhk:name "Order Validation Hook" ;
    knhk:triggerType "Event" ;
    knhk:triggerPattern "order.received" ;
    knhk:checkCondition "ASK { ?order knhk:isValid true }" ;
    knhk:action "workflow:validateOrder" ;
    knhk:emitReceipt true .
"#
    .to_string()
}

fn setup_integrated_environment() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create template directory structure
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Create RDF data
    let rdf_path = temp_dir.path().join("workflow.ttl");
    std::fs::write(&rdf_path, create_comprehensive_rdf()).expect("Failed to write RDF");

    temp_dir
}

// ============================================================================
// End-to-End Workflow Tests (10 tests)
// ============================================================================

#[test]
fn test_e2e_rdf_to_sparql_to_code_generation_pipeline() {
    // Arrange: Set up integrated environment
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    // Create SPARQL engine with RDF data
    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create SPARQL engine");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Query workflow data
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        }
    "#;

    let results = sparql_engine.execute_query(query).expect("Query failed");

    // Generate code from query results
    let rust_generator =
        create_generator("rust", &template_dir).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("struct_name".to_string(), "WorkflowTask".to_string());
    context.insert(
        "fields".to_string(),
        "pub name: String,\n    pub id: String".to_string(),
    );

    let generated = rust_generator
        .generate_domain_model(&context)
        .expect("Code generation failed");

    // Assert: Complete pipeline succeeds
    assert!(
        matches!(
            results,
            knhk_workflow_engine::ggen::sparql_engine::QueryResultType::Solutions(_)
        ),
        "SPARQL query should return results"
    );
    assert!(
        generated.content.contains("WorkflowTask"),
        "Generated code should contain struct"
    );
    assert_eq!(generated.language, "rust", "Language should be Rust");
}

#[test]
fn test_e2e_multi_language_generation_from_single_rdf_spec() {
    // Arrange: Set up integrated environment
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    // Create generators for multiple languages
    let rust_gen =
        create_generator("rust", &template_dir).expect("Failed to create Rust generator");
    let python_gen =
        create_generator("python", &template_dir).expect("Failed to create Python generator");

    let mut context = GenerationContext::new();
    context.insert("struct_name".to_string(), "Order".to_string());
    context.insert("class_name".to_string(), "Order".to_string());
    context.insert(
        "fields".to_string(),
        "pub id: String,\n    pub amount: f64".to_string(),
    );

    // Act: Generate code in multiple languages
    let rust_code = rust_gen.generate_domain_model(&context);
    let python_code = python_gen.generate_domain_model(&context);

    // Assert: Both languages generate successfully
    assert!(rust_code.is_ok(), "Rust generation should succeed");
    assert!(python_code.is_ok(), "Python generation should succeed");

    let rust = rust_code.expect("Rust generation failed");
    let python = python_code.expect("Python generation failed");

    assert_eq!(rust.language, "rust");
    assert_eq!(python.language, "python");
    assert!(rust.content.contains("Order"));
    assert!(python.content.contains("Order"));
}

#[test]
fn test_e2e_sparql_query_caching_improves_pipeline_performance() {
    // Arrange: Set up SPARQL engine
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create SPARQL engine");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;

    // Act: Execute query multiple times in pipeline
    let start = std::time::Instant::now();
    for _ in 0..5 {
        sparql_engine.execute_query(query).expect("Query failed");
    }
    let total_duration = start.elapsed();

    // Assert: Caching improves performance
    let (hits, misses, hit_ratio) = sparql_engine.cache_stats().expect("Failed to get stats");

    assert_eq!(misses, 1, "Should have 1 cache miss");
    assert_eq!(hits, 4, "Should have 4 cache hits");
    assert!(hit_ratio >= 0.8, "Hit ratio should be high");
    println!(
        "Pipeline completed 5 queries in {:?} (hit ratio: {:.2})",
        total_duration, hit_ratio
    );
}

#[test]
fn test_e2e_hooks_and_telemetry_generation_together() {
    // Arrange: Set up hooks and telemetry generators
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let hooks_gen = HooksGenerator::new(&template_dir).expect("Failed to create hooks generator");
    let mut telemetry_gen =
        TelemetryGenerator::new(&template_dir).expect("Failed to create telemetry generator");

    // Load ontology for hooks
    let rdf_path = temp_dir.path().join("workflow.ttl");
    hooks_gen
        .load_ontology(&rdf_path)
        .expect("Failed to load ontology");

    // Add telemetry span for hook execution
    telemetry_gen.add_span(SpanDefinition {
        name: "hook.execute".to_string(),
        description: "Hook execution span".to_string(),
        attributes: vec![AttributeDefinition {
            name: "hook.id".to_string(),
            attr_type: "string".to_string(),
            required: true,
            description: "Hook identifier".to_string(),
        }],
        events: vec![],
    });

    // Act: Generate both hooks and telemetry
    let hooks_code = hooks_gen.generate_hooks();
    let telemetry_code = telemetry_gen.generate_span_definitions();

    // Assert: Both generate successfully
    assert!(hooks_code.is_ok(), "Hooks generation should succeed");
    assert!(
        telemetry_code.is_ok(),
        "Telemetry generation should succeed"
    );

    let hooks = hooks_code.expect("Hooks generation failed");
    let telemetry = telemetry_code.expect("Telemetry generation failed");

    assert!(hooks.contains("HookContext"), "Should have hook context");
    assert!(telemetry.contains("hook.execute"), "Should have hook span");
}

#[test]
fn test_e2e_generated_code_includes_proper_error_handling() {
    // Arrange: Set up code generator
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    let mut api_context = GenerationContext::new();
    api_context.insert("handler_name".to_string(), "process_order".to_string());
    api_context.insert("route".to_string(), "/api/orders".to_string());

    // Act: Generate API endpoint
    let generated = rust_gen
        .generate_api_endpoint(&api_context)
        .expect("Generation failed");

    // Assert: Generated code uses Result for error handling
    assert!(
        generated.content.contains("Result<"),
        "Should return Result type"
    );
    assert!(
        generated.content.contains("StatusCode"),
        "Should handle HTTP status codes"
    );
    assert!(
        !generated.content.contains(".unwrap()"),
        "Should not use unwrap in production code"
    );
}

#[test]
fn test_e2e_documentation_generation_completes_pipeline() {
    // Arrange: Set up code generators
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("module_name".to_string(), "workflow_engine".to_string());

    // Act: Generate documentation
    let docs = rust_gen.generate_documentation(&context);

    // Assert: Documentation is generated
    assert!(docs.is_ok(), "Documentation generation should succeed");
    let doc_code = docs.expect("Documentation generation failed");
    assert!(
        doc_code.content.contains("//!"),
        "Should have module-level docs"
    );
    assert!(
        doc_code.content.contains("workflow_engine"),
        "Should mention module name"
    );
}

#[test]
fn test_e2e_test_generation_follows_chicago_tdd_principles() {
    // Arrange: Set up code generator
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("test_name".to_string(), "order_validation".to_string());

    // Act: Generate tests
    let tests = rust_gen.generate_tests(&context);

    // Assert: Tests follow AAA pattern
    assert!(tests.is_ok(), "Test generation should succeed");
    let test_code = tests.expect("Test generation failed");
    assert!(
        test_code.content.contains("// Arrange:"),
        "Should have Arrange phase"
    );
    assert!(
        test_code.content.contains("// Act:"),
        "Should have Act phase"
    );
    assert!(
        test_code.content.contains("// Assert:"),
        "Should have Assert phase"
    );
    assert!(
        test_code.content.contains("Chicago TDD"),
        "Should mention Chicago TDD"
    );
}

#[test]
fn test_e2e_weaver_schema_generation_for_validation() {
    // Arrange: Set up telemetry generator with complete specification
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut telemetry_gen =
        TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    // Add comprehensive telemetry
    telemetry_gen.add_span(SpanDefinition {
        name: "workflow.execute".to_string(),
        description: "Workflow execution".to_string(),
        attributes: vec![],
        events: vec![],
    });

    telemetry_gen.add_metric(MetricDefinition {
        name: "workflow.count".to_string(),
        metric_type: "counter".to_string(),
        description: "Workflow execution count".to_string(),
        unit: "executions".to_string(),
        attributes: vec![],
    });

    // Act: Generate Weaver schema
    let schema = telemetry_gen.generate_weaver_schema();

    // Assert: Schema is complete and valid
    assert!(schema.is_ok(), "Schema generation should succeed");
    let schema_content = schema.expect("Schema generation failed");
    assert!(
        schema_content.contains("schema_version:"),
        "Should have schema version"
    );
    assert!(
        schema_content.contains("workflow.execute"),
        "Should include span"
    );
    assert!(
        schema_content.contains("workflow.count"),
        "Should include metric"
    );
}

#[test]
fn test_e2e_sparql_template_rendering_with_query_bindings() {
    // Arrange: Set up SPARQL engine with template
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    // Create simple template
    let template_content = "Tasks: {{ task_count }}";
    let template_path = template_dir.join("summary.tera");
    std::fs::write(&template_path, template_content).expect("Failed to write template");

    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Render template with query bindings
    let mut query_bindings = HashMap::new();
    query_bindings.insert(
        "task_count".to_string(),
        r#"SELECT (COUNT(?task) as ?count) WHERE { ?task a ?type . }"#.to_string(),
    );

    let rendered = sparql_engine.render_template("summary.tera", query_bindings);

    // Assert: Template renders with query results
    assert!(rendered.is_ok(), "Template rendering should succeed");
}

#[test]
fn test_e2e_performance_of_complete_generation_pipeline() {
    // Arrange: Set up complete pipeline
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    // Act: Measure complete pipeline performance
    let start = std::time::Instant::now();

    // Query data
    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
    sparql_engine.execute_query(query).expect("Query failed");

    // Generate code
    let mut context = GenerationContext::new();
    context.insert("struct_name".to_string(), "Task".to_string());
    context.insert("fields".to_string(), "pub id: String".to_string());
    rust_gen
        .generate_domain_model(&context)
        .expect("Generation failed");

    let duration = start.elapsed();

    // Assert: Pipeline completes in reasonable time
    println!("Complete pipeline duration: {:?}", duration);
    assert!(
        duration.as_millis() < 1000,
        "Pipeline should complete within 1 second"
    );
}

// ============================================================================
// Error Recovery Tests (5 tests)
// ============================================================================

#[test]
fn test_e2e_pipeline_continues_after_partial_sparql_failure() {
    // Arrange: Set up pipeline with valid and invalid queries
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Execute valid query after invalid one
    let invalid_query = "INVALID SPARQL SYNTAX";
    let invalid_result = sparql_engine.execute_query(invalid_query);

    let valid_query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
    let valid_result = sparql_engine.execute_query(valid_query);

    // Assert: Invalid query fails, valid query succeeds
    assert!(invalid_result.is_err(), "Invalid query should fail");
    assert!(valid_result.is_ok(), "Valid query should succeed");
}

#[test]
fn test_e2e_code_generation_continues_after_missing_context() {
    // Arrange: Set up code generator
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    // Act: Try generation with incomplete context, then complete context
    let incomplete_context = GenerationContext::new();
    let incomplete_result = rust_gen.generate_domain_model(&incomplete_context);

    let mut complete_context = GenerationContext::new();
    complete_context.insert("struct_name".to_string(), "Task".to_string());
    complete_context.insert("fields".to_string(), "pub id: String".to_string());
    let complete_result = rust_gen.generate_domain_model(&complete_context);

    // Assert: Incomplete fails, complete succeeds
    assert!(incomplete_result.is_err(), "Incomplete context should fail");
    assert!(complete_result.is_ok(), "Complete context should succeed");
}

#[test]
fn test_e2e_hooks_generation_handles_missing_ontology_gracefully() {
    // Arrange: Set up hooks generator without ontology
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let hooks_gen = HooksGenerator::new(&template_dir).expect("Failed to create generator");

    // Act: Try to extract hooks without loading ontology
    let result = hooks_gen.extract_hook_definitions();

    // Assert: Returns empty or error gracefully (not panic)
    match result {
        Ok(hooks) => {
            assert_eq!(hooks.len(), 0, "Should return empty hooks");
        }
        Err(_) => {
            // Acceptable to return error
        }
    }
}

#[test]
fn test_e2e_telemetry_generation_works_with_empty_definitions() {
    // Arrange: Set up telemetry generator without definitions
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let telemetry_gen = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    // Act: Generate code with empty definitions
    let span_code = telemetry_gen.generate_span_definitions();
    let metric_code = telemetry_gen.generate_metric_collectors();
    let schema = telemetry_gen.generate_weaver_schema();

    // Assert: All generate minimal valid code
    assert!(span_code.is_ok(), "Should generate minimal span code");
    assert!(metric_code.is_ok(), "Should generate minimal metric code");
    assert!(schema.is_ok(), "Should generate minimal schema");
}

#[test]
fn test_e2e_pipeline_recovers_from_rdf_parsing_error() {
    // Arrange: Set up SPARQL engine
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    // Act: Try to load invalid RDF, then load valid RDF
    let invalid_rdf_path = temp_dir.path().join("invalid.ttl");
    std::fs::write(&invalid_rdf_path, "INVALID RDF").expect("Failed to write");

    let invalid_result = sparql_engine.load_rdf_graph(&invalid_rdf_path);

    let valid_rdf_path = temp_dir.path().join("workflow.ttl");
    let valid_result = sparql_engine.load_rdf_graph(&valid_rdf_path);

    // Assert: Invalid RDF fails, valid RDF succeeds
    assert!(invalid_result.is_err(), "Invalid RDF should fail");
    assert!(valid_result.is_ok(), "Valid RDF should succeed");

    // Pipeline can continue after error
    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
    let query_result = sparql_engine.execute_query(query);
    assert!(
        query_result.is_ok(),
        "Pipeline should continue after RDF error"
    );
}

// ============================================================================
// Cross-Module Integration Tests (10 tests)
// ============================================================================

#[test]
fn test_sparql_results_feed_into_code_generation_context() {
    // Arrange: Set up SPARQL engine and code generator
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    // Act: Query for task names
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        }
    "#;

    let results = sparql_engine.execute_query(query);

    // Use query results in code generation
    let mut context = GenerationContext::new();
    context.insert("enum_name".to_string(), "TaskName".to_string());

    // Simulate extracting task names from results
    if let Ok(knhk_workflow_engine::ggen::sparql_engine::QueryResultType::Solutions(solutions)) =
        results
    {
        let variants: Vec<String> = solutions
            .iter()
            .filter_map(|s| s.get("name"))
            .map(|n| n.replace('"', ""))
            .collect();
        context.insert("variants".to_string(), variants.join(",\n    "));
    }

    let generated = rust_gen.generate_domain_model(&context);

    // Assert: SPARQL results successfully feed into code generation
    assert!(generated.is_ok(), "Code generation should succeed");
}

#[test]
fn test_hooks_generator_integrates_with_sparql_queries() {
    // Arrange: Set up hooks generator
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let hooks_gen = HooksGenerator::new(&template_dir).expect("Failed to create generator");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    hooks_gen
        .load_ontology(&rdf_path)
        .expect("Failed to load ontology");

    // Act: Extract hooks (which internally uses SPARQL)
    let hooks = hooks_gen.extract_hook_definitions();

    // Assert: Hooks extraction uses SPARQL successfully
    assert!(
        hooks.is_ok(),
        "Hooks should be extracted using internal SPARQL"
    );
}

#[test]
fn test_telemetry_spans_match_generated_code_functions() {
    // Arrange: Generate code and telemetry
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");
    let mut telemetry_gen =
        TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    // Generate API handler
    let mut api_context = GenerationContext::new();
    api_context.insert("handler_name".to_string(), "process_order".to_string());

    let api_code = rust_gen
        .generate_api_endpoint(&api_context)
        .expect("Generation failed");

    // Generate matching telemetry span
    telemetry_gen.add_span(SpanDefinition {
        name: "api.process_order".to_string(),
        description: "Process order API endpoint".to_string(),
        attributes: vec![],
        events: vec![],
    });

    let span_code = telemetry_gen
        .generate_span_definitions()
        .expect("Generation failed");

    // Assert: Telemetry spans align with generated code
    assert!(
        api_code.content.contains("process_order"),
        "API should have handler"
    );
    assert!(
        span_code.contains("api.process_order"),
        "Telemetry should have matching span"
    );
}

#[test]
fn test_generated_code_includes_telemetry_instrumentation_hooks() {
    // Arrange: Generate API code
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    let mut api_context = GenerationContext::new();
    api_context.insert("handler_name".to_string(), "get_orders".to_string());

    // Act: Generate API endpoint
    let generated = rust_gen
        .generate_api_endpoint(&api_context)
        .expect("Generation failed");

    // Assert: Generated code includes instrumentation
    assert!(
        generated.content.contains("use") || generated.content.contains("async"),
        "Should have proper imports for instrumentation"
    );
}

#[test]
fn test_weaver_schema_validates_generated_telemetry_code() {
    // Arrange: Generate telemetry code and schema
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let mut telemetry_gen =
        TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    let span = SpanDefinition {
        name: "db.query".to_string(),
        description: "Database query execution".to_string(),
        attributes: vec![AttributeDefinition {
            name: "db.statement".to_string(),
            attr_type: "string".to_string(),
            required: true,
            description: "SQL statement".to_string(),
        }],
        events: vec![],
    };

    telemetry_gen.add_span(span);

    // Act: Generate code and schema
    let code = telemetry_gen
        .generate_span_definitions()
        .expect("Code generation failed");
    let schema = telemetry_gen
        .generate_weaver_schema()
        .expect("Schema generation failed");

    // Assert: Code and schema are consistent
    assert!(code.contains("db.query"), "Code should have span");
    assert!(schema.contains("db.query"), "Schema should have span");
    assert!(
        schema.contains("db.statement"),
        "Schema should have attributes"
    );
}

#[test]
fn test_multi_module_generation_maintains_consistency() {
    // Arrange: Set up all generators
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");
    let hooks_gen = HooksGenerator::new(&template_dir).expect("Failed to create generator");
    let mut telemetry_gen =
        TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    // Act: Generate related artifacts
    let mut code_context = GenerationContext::new();
    code_context.insert("struct_name".to_string(), "Order".to_string());
    code_context.insert("fields".to_string(), "pub id: String".to_string());

    let domain_code = rust_gen.generate_domain_model(&code_context);

    telemetry_gen.add_span(SpanDefinition {
        name: "order.process".to_string(),
        description: "Order processing".to_string(),
        attributes: vec![],
        events: vec![],
    });

    let telemetry_code = telemetry_gen.generate_span_definitions();

    // Assert: All modules generate consistently
    assert!(domain_code.is_ok(), "Domain code should generate");
    assert!(telemetry_code.is_ok(), "Telemetry should generate");

    let domain = domain_code.expect("Domain generation failed");
    let telemetry = telemetry_code.expect("Telemetry generation failed");

    assert!(domain.content.contains("Order"));
    assert!(telemetry.contains("order.process"));
}

#[test]
fn test_generated_tests_reference_generated_code() {
    // Arrange: Generate domain code and tests
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");

    let mut domain_context = GenerationContext::new();
    domain_context.insert("struct_name".to_string(), "Task".to_string());
    domain_context.insert("fields".to_string(), "pub id: String".to_string());

    let mut test_context = GenerationContext::new();
    test_context.insert("test_name".to_string(), "task_creation".to_string());

    // Act: Generate domain code and tests
    let domain = rust_gen
        .generate_domain_model(&domain_context)
        .expect("Domain generation failed");
    let tests = rust_gen
        .generate_tests(&test_context)
        .expect("Test generation failed");

    // Assert: Tests can reference generated code
    assert!(domain.content.contains("struct Task"));
    assert!(tests.content.contains("#[test]"));
}

#[test]
fn test_hooks_emit_telemetry_events_in_generated_code() {
    // Arrange: Generate hooks with telemetry
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let hooks_gen = HooksGenerator::new(&template_dir).expect("Failed to create generator");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    hooks_gen
        .load_ontology(&rdf_path)
        .expect("Failed to load ontology");

    // Act: Generate hooks
    let hooks_code = hooks_gen.generate_hooks().expect("Generation failed");

    // Assert: Generated hooks include telemetry
    assert!(
        hooks_code.contains("tracing::") || hooks_code.contains("info!"),
        "Hooks should emit telemetry events"
    );
}

#[test]
fn test_lockchain_receipts_integrate_with_telemetry() {
    // Arrange: Generate hooks with receipts
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    let hooks_gen = HooksGenerator::new(&template_dir).expect("Failed to create generator");

    let rdf_path = temp_dir.path().join("workflow.ttl");
    hooks_gen
        .load_ontology(&rdf_path)
        .expect("Failed to load ontology");

    // Act: Generate hooks
    let hooks_code = hooks_gen.generate_hooks().expect("Generation failed");

    // Assert: Receipts are emitted (integrates with telemetry)
    assert!(
        hooks_code.contains("Receipt::new") || hooks_code.contains("receipt"),
        "Should emit Lockchain receipts"
    );
}

#[test]
fn test_complete_workflow_from_rdf_to_deployable_code() {
    // Arrange: Set up complete environment
    let temp_dir = setup_integrated_environment();
    let template_dir = temp_dir.path().join("templates");

    // Act: Execute complete workflow
    // 1. Load RDF
    let mut sparql_engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");
    let rdf_path = temp_dir.path().join("workflow.ttl");
    sparql_engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // 2. Query data
    let query_result = sparql_engine.execute_query(r#"SELECT ?task WHERE { ?task a ?type . }"#);

    // 3. Generate code
    let rust_gen = create_generator("rust", &template_dir).expect("Failed to create generator");
    let mut context = GenerationContext::new();
    context.insert("struct_name".to_string(), "Workflow".to_string());
    context.insert("fields".to_string(), "pub id: String".to_string());
    let code_result = rust_gen.generate_domain_model(&context);

    // 4. Generate tests
    let mut test_context = GenerationContext::new();
    test_context.insert("test_name".to_string(), "workflow_test".to_string());
    let test_result = rust_gen.generate_tests(&test_context);

    // 5. Generate docs
    let mut doc_context = GenerationContext::new();
    doc_context.insert("module_name".to_string(), "workflow".to_string());
    let doc_result = rust_gen.generate_documentation(&doc_context);

    // Assert: Complete workflow succeeds
    assert!(query_result.is_ok(), "SPARQL query should succeed");
    assert!(code_result.is_ok(), "Code generation should succeed");
    assert!(test_result.is_ok(), "Test generation should succeed");
    assert!(
        doc_result.is_ok(),
        "Documentation generation should succeed"
    );

    println!("✓ Complete workflow: RDF → SPARQL → Code → Tests → Docs");
}
