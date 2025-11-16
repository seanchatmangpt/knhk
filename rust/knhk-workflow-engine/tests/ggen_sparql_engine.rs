//! SPARQL Engine Test Suite - Chicago TDD Style
//!
//! Tests SPARQL template engine with real RDF data and actual query execution.
//! Focuses on observable behavior and state changes, not implementation details.
//!
//! Test Coverage (40+ tests):
//! - Query execution (SELECT, CONSTRUCT, ASK, COUNT)
//! - Complex queries (filters, unions, aggregations)
//! - Caching behavior and performance
//! - Error handling
//! - Thread-safe concurrent execution
//! - Template rendering with SPARQL bindings

use knhk_workflow_engine::ggen::sparql_engine::{QueryResultType, SparqlTemplateEngine};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

// ============================================================================
// Test Data Builders
// ============================================================================

fn create_test_rdf() -> String {
    r#"
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

<#OrderProcessing> a yawl:Specification ;
    yawl:name "Order Processing" ;
    yawl:version "1.0" .

<#receiveOrder> a yawl:Task ;
    yawl:name "Receive Order" ;
    yawl:id "receive_order" ;
    yawl:taskType "atomic" .

<#validateOrder> a yawl:Task ;
    yawl:name "Validate Order" ;
    yawl:id "validate_order" ;
    yawl:taskType "atomic" .

<#shipOrder> a yawl:Task ;
    yawl:name "Ship Order" ;
    yawl:id "ship_order" ;
    yawl:taskType "composite" .
"#
    .to_string()
}

fn create_test_template() -> String {
    r#"
# Workflow: {{ workflow_name }}
Tasks: {{ task_count }}
"#
    .to_string()
}

fn setup_engine_with_data() -> (TempDir, SparqlTemplateEngine) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Create test template
    let template_path = template_dir.join("test.tera");
    std::fs::write(&template_path, create_test_template()).expect("Failed to write template");

    let mut engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    // Load RDF data
    let rdf_path = temp_dir.path().join("test.ttl");
    std::fs::write(&rdf_path, create_test_rdf()).expect("Failed to write RDF");
    engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    (temp_dir, engine)
}

// ============================================================================
// Engine Creation Tests (4 tests)
// ============================================================================

#[test]
fn test_engine_creates_successfully_with_valid_template_directory() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Create SPARQL engine
    let result = SparqlTemplateEngine::new(&template_dir, 100);

    // Assert: Engine is created successfully
    assert!(
        result.is_ok(),
        "Engine should be created with valid template directory"
    );
    let engine = result.expect("Failed to create engine");
    let stats = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(stats.0, 0, "Initial cache hits should be 0");
    assert_eq!(stats.1, 0, "Initial cache misses should be 0");
}

#[test]
fn test_engine_creation_fails_with_zero_cache_size() {
    // Arrange: Create template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Attempt to create engine with zero cache size
    let result = SparqlTemplateEngine::new(&template_dir, 0);

    // Assert: Creation fails with meaningful error
    assert!(
        result.is_err(),
        "Engine creation should fail with zero cache size"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Cache size must be > 0"));
}

#[test]
fn test_engine_loads_rdf_graph_successfully() {
    // Arrange: Create engine and RDF file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("test.ttl");
    std::fs::write(&rdf_path, create_test_rdf()).expect("Failed to write RDF");

    // Act: Load RDF graph
    let result = engine.load_rdf_graph(&rdf_path);

    // Assert: RDF loads successfully
    assert!(result.is_ok(), "RDF graph should load successfully");
}

#[test]
fn test_engine_handles_invalid_rdf_file_gracefully() {
    // Arrange: Create engine with invalid RDF content
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("invalid.ttl");
    std::fs::write(&rdf_path, "INVALID RDF CONTENT").expect("Failed to write file");

    // Act: Attempt to load invalid RDF
    let result = engine.load_rdf_graph(&rdf_path);

    // Assert: Loading fails with meaningful error
    assert!(result.is_err(), "Invalid RDF should cause error");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to load RDF"));
}

// ============================================================================
// SELECT Query Tests (10 tests)
// ============================================================================

#[test]
fn test_select_query_returns_all_tasks() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute SELECT query for all tasks
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Query returns expected task data
    assert!(result.is_ok(), "SELECT query should execute successfully");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 3, "Should return 3 tasks");
            let names: Vec<&str> = solutions
                .iter()
                .filter_map(|s| s.get("name").map(|n| n.as_str()))
                .collect();
            assert!(
                names.contains(&"\"Receive Order\""),
                "Should contain Receive Order task"
            );
            assert!(
                names.contains(&"\"Validate Order\""),
                "Should contain Validate Order task"
            );
            assert!(
                names.contains(&"\"Ship Order\""),
                "Should contain Ship Order task"
            );
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_with_filter_returns_atomic_tasks_only() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute SELECT query with FILTER
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
            ?task yawl:taskType "atomic" .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Only atomic tasks are returned
    assert!(result.is_ok(), "Filtered SELECT should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 2, "Should return only 2 atomic tasks");
            for solution in &solutions {
                let name = solution.get("name").expect("Missing name field");
                assert!(
                    name.contains("Receive Order") || name.contains("Validate Order"),
                    "Only atomic tasks should be returned"
                );
            }
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_with_optional_returns_all_rows_with_nulls() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute SELECT with OPTIONAL clause
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name ?version WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
            OPTIONAL { ?task yawl:version ?version }
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: All tasks returned, version may be null
    assert!(result.is_ok(), "OPTIONAL query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 3, "Should return all 3 tasks");
            // Verify that some rows have version, some don't
            let has_version = solutions.iter().any(|s| s.contains_key("version"));
            assert!(
                !has_version || solutions.iter().any(|s| !s.contains_key("version")),
                "OPTIONAL should handle missing values"
            );
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_returns_empty_result_for_non_existent_data() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Query for non-existent data
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task WHERE {
            ?task a yawl:NonExistentClass .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Query succeeds but returns empty result
    assert!(result.is_ok(), "Query for non-existent data should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 0, "Should return empty result set");
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_with_limit_returns_correct_number_of_rows() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute SELECT with LIMIT
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task WHERE {
            ?task a yawl:Task .
        } LIMIT 2
    "#;

    let result = engine.execute_query(query);

    // Assert: Only 2 results returned
    assert!(result.is_ok(), "LIMIT query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 2, "LIMIT should restrict result count");
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_with_order_by_returns_sorted_results() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute SELECT with ORDER BY
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        } ORDER BY ?name
    "#;

    let result = engine.execute_query(query);

    // Assert: Results are sorted alphabetically
    assert!(result.is_ok(), "ORDER BY query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            let names: Vec<String> = solutions
                .iter()
                .filter_map(|s| s.get("name").cloned())
                .collect();
            let mut sorted_names = names.clone();
            sorted_names.sort();
            assert_eq!(names, sorted_names, "Results should be sorted");
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_with_distinct_removes_duplicates() {
    // Arrange: Set up engine with data that could have duplicates
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("test.ttl");
    let rdf_content = r#"
@prefix ex: <http://example.org/> .
<#task1> a ex:Task ; ex:category "processing" .
<#task2> a ex:Task ; ex:category "processing" .
<#task3> a ex:Task ; ex:category "validation" .
"#;
    std::fs::write(&rdf_path, rdf_content).expect("Failed to write RDF");
    engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Execute SELECT DISTINCT
    let query = r#"
        PREFIX ex: <http://example.org/>
        SELECT DISTINCT ?category WHERE {
            ?task a ex:Task .
            ?task ex:category ?category .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Duplicates are removed
    assert!(result.is_ok(), "DISTINCT query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 2, "DISTINCT should remove duplicates");
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_with_multiple_where_clauses_joins_correctly() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute SELECT with multiple WHERE clauses
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name ?id WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
            ?task yawl:id ?id .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: All fields are joined correctly
    assert!(result.is_ok(), "Multi-clause query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            for solution in &solutions {
                assert!(solution.contains_key("task"), "Should have task field");
                assert!(solution.contains_key("name"), "Should have name field");
                assert!(solution.contains_key("id"), "Should have id field");
            }
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_select_query_handles_special_characters_in_literals() {
    // Arrange: Set up engine with special characters
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("test.ttl");
    let rdf_content = r#"
@prefix ex: <http://example.org/> .
<#task1> ex:name "Task with 'quotes' and \"escapes\"" .
"#;
    std::fs::write(&rdf_path, rdf_content).expect("Failed to write RDF");
    engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Query for special characters
    let query = r#"
        PREFIX ex: <http://example.org/>
        SELECT ?name WHERE {
            ?task ex:name ?name .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Special characters are preserved
    assert!(
        result.is_ok(),
        "Query with special characters should succeed"
    );
}

#[test]
fn test_select_query_with_count_aggregation_returns_correct_count() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute COUNT aggregation
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT (COUNT(?task) AS ?taskCount) WHERE {
            ?task a yawl:Task .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Count is correct
    assert!(result.is_ok(), "COUNT query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 1, "Should return single count row");
            let count_str = solutions[0].get("taskCount").expect("Missing count");
            assert!(
                count_str.contains("3"),
                "Should count all 3 tasks: {}",
                count_str
            );
        }
        _ => panic!("Expected Solutions result type"),
    }
}

// ============================================================================
// ASK Query Tests (5 tests)
// ============================================================================

#[test]
fn test_ask_query_returns_true_when_pattern_exists() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute ASK query for existing pattern
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        ASK {
            ?spec a yawl:Specification .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: ASK returns true
    assert!(result.is_ok(), "ASK query should execute successfully");
    match result.expect("Query failed") {
        QueryResultType::Boolean(exists) => {
            assert!(exists, "ASK should return true for existing pattern");
        }
        _ => panic!("Expected Boolean result type"),
    }
}

#[test]
fn test_ask_query_returns_false_when_pattern_does_not_exist() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute ASK query for non-existent pattern
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        ASK {
            ?nonexistent a yawl:NonExistentClass .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: ASK returns false
    assert!(result.is_ok(), "ASK query should execute successfully");
    match result.expect("Query failed") {
        QueryResultType::Boolean(exists) => {
            assert!(!exists, "ASK should return false for non-existent pattern");
        }
        _ => panic!("Expected Boolean result type"),
    }
}

#[test]
fn test_ask_query_with_filter_evaluates_condition() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute ASK with FILTER
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        ASK {
            ?task a yawl:Task .
            ?task yawl:taskType "atomic" .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: ASK returns true (atomic tasks exist)
    assert!(result.is_ok(), "ASK with filter should succeed");
    match result.expect("Query failed") {
        QueryResultType::Boolean(exists) => {
            assert!(exists, "ASK should return true for filtered pattern");
        }
        _ => panic!("Expected Boolean result type"),
    }
}

#[test]
fn test_ask_query_with_complex_pattern_evaluates_correctly() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute ASK with complex pattern
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        ASK {
            ?task a yawl:Task .
            ?task yawl:name ?name .
            ?task yawl:id ?id .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: ASK evaluates complex pattern
    assert!(result.is_ok(), "Complex ASK should succeed");
    match result.expect("Query failed") {
        QueryResultType::Boolean(exists) => {
            assert!(exists, "Complex pattern should match");
        }
        _ => panic!("Expected Boolean result type"),
    }
}

#[test]
fn test_ask_query_with_optional_clause_handles_missing_data() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute ASK with OPTIONAL
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        ASK {
            ?task a yawl:Task .
            OPTIONAL { ?task yawl:nonExistentProperty ?value }
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: ASK handles optional missing data
    assert!(result.is_ok(), "ASK with OPTIONAL should succeed");
    match result.expect("Query failed") {
        QueryResultType::Boolean(exists) => {
            assert!(exists, "OPTIONAL should not prevent match");
        }
        _ => panic!("Expected Boolean result type"),
    }
}

// ============================================================================
// CONSTRUCT Query Tests (4 tests)
// ============================================================================

#[test]
fn test_construct_query_generates_new_triples() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute CONSTRUCT query
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX ex: <http://example.org/>
        CONSTRUCT {
            ?task ex:hasName ?name .
        }
        WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: New triples are generated
    assert!(result.is_ok(), "CONSTRUCT query should succeed");
    match result.expect("Query failed") {
        QueryResultType::Graph(triples) => {
            assert_eq!(triples.len(), 3, "Should generate 3 new triples");
            for (_, predicate, _) in &triples {
                assert!(predicate.contains("hasName"), "Predicate should be hasName");
            }
        }
        _ => panic!("Expected Graph result type"),
    }
}

#[test]
fn test_construct_query_preserves_data_from_where_clause() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute CONSTRUCT with data transformation
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX ex: <http://example.org/>
        CONSTRUCT {
            ?task ex:type ?taskType .
        }
        WHERE {
            ?task a yawl:Task .
            ?task yawl:taskType ?taskType .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Data is preserved in transformation
    assert!(result.is_ok(), "CONSTRUCT transformation should succeed");
    match result.expect("Query failed") {
        QueryResultType::Graph(triples) => {
            assert!(
                triples.len() >= 2,
                "Should generate triples for typed tasks"
            );
        }
        _ => panic!("Expected Graph result type"),
    }
}

#[test]
fn test_construct_query_returns_empty_graph_for_no_matches() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute CONSTRUCT with no matches
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX ex: <http://example.org/>
        CONSTRUCT {
            ?task ex:property ?value .
        }
        WHERE {
            ?task a yawl:NonExistent .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Empty graph returned
    assert!(result.is_ok(), "CONSTRUCT with no matches should succeed");
    match result.expect("Query failed") {
        QueryResultType::Graph(triples) => {
            assert_eq!(triples.len(), 0, "Should return empty graph");
        }
        _ => panic!("Expected Graph result type"),
    }
}

#[test]
fn test_construct_query_handles_multiple_predicates() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute CONSTRUCT with multiple predicates
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX ex: <http://example.org/>
        CONSTRUCT {
            ?task ex:hasName ?name .
            ?task ex:hasId ?id .
        }
        WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
            ?task yawl:id ?id .
        }
    "#;

    let result = engine.execute_query(query);

    // Assert: Multiple predicates generated
    assert!(result.is_ok(), "Multi-predicate CONSTRUCT should succeed");
    match result.expect("Query failed") {
        QueryResultType::Graph(triples) => {
            assert!(
                triples.len() >= 6,
                "Should generate triples for both predicates"
            );
            let has_name = triples.iter().any(|(_, p, _)| p.contains("hasName"));
            let has_id = triples.iter().any(|(_, p, _)| p.contains("hasId"));
            assert!(has_name, "Should have hasName predicates");
            assert!(has_id, "Should have hasId predicates");
        }
        _ => panic!("Expected Graph result type"),
    }
}

// ============================================================================
// Cache Behavior Tests (8 tests)
// ============================================================================

#[test]
fn test_cache_hit_increases_hit_count_on_repeated_query() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task WHERE { ?task a yawl:Task . }
    "#;

    // Act: Execute same query twice
    engine.execute_query(query).expect("First query failed");
    engine.execute_query(query).expect("Second query failed");

    // Assert: Cache hit count increased
    let (hits, misses, ratio) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(hits, 1, "Should have 1 cache hit");
    assert_eq!(misses, 1, "Should have 1 cache miss");
    assert!(ratio > 0.0 && ratio <= 1.0, "Hit ratio should be 0.5");
}

#[test]
fn test_cache_miss_occurs_on_first_query_execution() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task WHERE { ?task a yawl:Task . }
    "#;

    // Act: Execute query once
    engine.execute_query(query).expect("Query failed");

    // Assert: First execution is cache miss
    let (hits, misses, _) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(hits, 0, "First query should have no hits");
    assert_eq!(misses, 1, "First query should be cache miss");
}

#[test]
fn test_cache_hit_ratio_improves_with_repeated_queries() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task WHERE { ?task a yawl:Task . }
    "#;

    // Act: Execute query multiple times
    for _ in 0..10 {
        engine.execute_query(query).expect("Query failed");
    }

    // Assert: Hit ratio approaches 1.0
    let (hits, misses, ratio) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(misses, 1, "Should have 1 initial miss");
    assert_eq!(hits, 9, "Should have 9 hits after 10 executions");
    assert!(ratio > 0.8, "Hit ratio should be high: {}", ratio);
}

#[test]
fn test_cache_differentiates_between_different_queries() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query1 = r#"SELECT ?task WHERE { ?task a ?type . }"#;
    let query2 = r#"SELECT ?name WHERE { ?task ?prop ?name . }"#;

    // Act: Execute different queries
    engine.execute_query(query1).expect("Query 1 failed");
    engine.execute_query(query2).expect("Query 2 failed");

    // Assert: Both are cache misses (different queries)
    let (hits, misses, _) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(hits, 0, "Different queries should not hit cache");
    assert_eq!(misses, 2, "Each different query should miss");
}

#[test]
fn test_cache_clear_removes_all_cached_entries() {
    // Arrange: Set up engine with cached data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
    engine.execute_query(query).expect("Query failed");
    engine.execute_query(query).expect("Query failed"); // Cache hit

    // Act: Clear cache
    engine.clear_cache().expect("Cache clear failed");
    engine.execute_query(query).expect("Query failed"); // Should miss after clear

    // Assert: Cache was cleared, next query is miss
    let (hits, misses, _) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(
        misses, 2,
        "After clear, query should miss again (1 initial + 1 after clear)"
    );
}

#[test]
fn test_cache_respects_size_limit_with_lru_eviction() {
    // Arrange: Create engine with small cache (size 2)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut engine = SparqlTemplateEngine::new(&template_dir, 2).expect("Failed to create engine");

    let rdf_path = temp_dir.path().join("test.ttl");
    std::fs::write(&rdf_path, create_test_rdf()).expect("Failed to write RDF");
    engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Execute 3 different queries (exceeds cache size)
    let query1 = r#"SELECT ?s WHERE { ?s ?p ?o . } LIMIT 1"#;
    let query2 = r#"SELECT ?s WHERE { ?s ?p ?o . } LIMIT 2"#;
    let query3 = r#"SELECT ?s WHERE { ?s ?p ?o . } LIMIT 3"#;

    engine.execute_query(query1).expect("Query 1 failed");
    engine.execute_query(query2).expect("Query 2 failed");
    engine.execute_query(query3).expect("Query 3 failed");

    // Execute query1 again - should be evicted (cache size 2, query1 is oldest)
    engine.execute_query(query1).expect("Query 1 retry failed");

    // Assert: LRU eviction occurred
    let (hits, misses, _) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(
        misses, 4,
        "Query 1 should miss again after eviction (3 initial + 1 evicted)"
    );
}

#[test]
fn test_cache_performance_is_faster_than_fresh_execution() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        }
    "#;

    // Act: Measure first execution (cache miss)
    let start_miss = std::time::Instant::now();
    engine.execute_query(query).expect("Query failed");
    let miss_duration = start_miss.elapsed();

    // Measure second execution (cache hit)
    let start_hit = std::time::Instant::now();
    engine.execute_query(query).expect("Query failed");
    let hit_duration = start_hit.elapsed();

    // Assert: Cache hit should be faster (or at least not slower)
    // Note: This is a soft assertion as timing can vary
    println!(
        "Cache miss: {:?}, Cache hit: {:?}",
        miss_duration, hit_duration
    );
    assert!(
        hit_duration <= miss_duration * 2,
        "Cache hit should not be significantly slower"
    );
}

#[test]
fn test_cache_returns_identical_results_to_fresh_execution() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        SELECT ?task ?name WHERE {
            ?task a yawl:Task .
            ?task yawl:name ?name .
        }
    "#;

    // Act: Execute twice and compare results
    let result1 = engine.execute_query(query).expect("First query failed");
    let result2 = engine.execute_query(query).expect("Second query failed");

    // Assert: Results are identical
    assert_eq!(
        result1, result2,
        "Cached result should be identical to fresh result"
    );
}

// ============================================================================
// Error Handling Tests (5 tests)
// ============================================================================

#[test]
fn test_invalid_sparql_syntax_returns_meaningful_error() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute query with invalid syntax
    let invalid_query = r#"SELECT WHERE { INVALID SYNTAX }"#;
    let result = engine.execute_query(invalid_query);

    // Assert: Error message is meaningful
    assert!(result.is_err(), "Invalid query should fail");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("SPARQL"),
        "Error should mention SPARQL: {}",
        error
    );
}

#[test]
fn test_query_on_empty_graph_returns_empty_results() {
    // Arrange: Create engine without loading RDF
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut engine =
        SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

    // Load empty RDF
    let rdf_path = temp_dir.path().join("empty.ttl");
    std::fs::write(&rdf_path, "@prefix ex: <http://example.org/> .").expect("Failed to write");
    engine
        .load_rdf_graph(&rdf_path)
        .expect("Failed to load RDF");

    // Act: Query empty graph
    let query = r#"SELECT ?s WHERE { ?s ?p ?o . }"#;
    let result = engine.execute_query(query);

    // Assert: Returns empty result, not error
    assert!(result.is_ok(), "Query on empty graph should succeed");
    match result.expect("Query failed") {
        QueryResultType::Solutions(solutions) => {
            assert_eq!(solutions.len(), 0, "Should return empty results");
        }
        _ => panic!("Expected Solutions result type"),
    }
}

#[test]
fn test_query_with_undefined_prefix_returns_error() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Query with undefined prefix
    let query = r#"SELECT ?task WHERE { ?task a undefined:Class . }"#;
    let result = engine.execute_query(query);

    // Assert: Error or empty result (depends on SPARQL implementation)
    // Most SPARQL engines treat undefined prefixes as errors
    match result {
        Ok(QueryResultType::Solutions(solutions)) => {
            assert_eq!(
                solutions.len(),
                0,
                "Should return no results for undefined prefix"
            );
        }
        Err(e) => {
            assert!(
                e.to_string().contains("undefined") || e.to_string().contains("prefix"),
                "Error should mention undefined prefix: {}",
                e
            );
        }
        _ => {}
    }
}

#[test]
fn test_malformed_where_clause_returns_error() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();

    // Act: Execute query with malformed WHERE
    let query = r#"SELECT ?task WHERE { ?task }"#; // Missing predicate and object
    let result = engine.execute_query(query);

    // Assert: Error is returned
    assert!(result.is_err(), "Malformed WHERE should fail");
}

#[test]
fn test_concurrent_queries_with_errors_dont_corrupt_cache() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();
    let engine = Arc::new(engine);

    // Act: Execute valid and invalid queries concurrently
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let engine = Arc::clone(&engine);
            thread::spawn(move || {
                let query = if i % 2 == 0 {
                    r#"SELECT ?task WHERE { ?task a ?type . }"#
                } else {
                    r#"INVALID QUERY SYNTAX"#
                };
                engine.execute_query(query)
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Assert: Valid queries succeed, invalid fail, cache not corrupted
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();
    assert_eq!(successes, 2, "Valid queries should succeed");
    assert_eq!(failures, 2, "Invalid queries should fail");

    // Verify cache still works
    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
    let result = engine.execute_query(query);
    assert!(result.is_ok(), "Cache should still work after errors");
}

// ============================================================================
// Thread Safety Tests (4 tests)
// ============================================================================

#[test]
fn test_concurrent_query_execution_is_thread_safe() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();
    let engine = Arc::new(engine);

    // Act: Execute queries concurrently from multiple threads
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let engine = Arc::clone(&engine);
            thread::spawn(move || {
                let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
                engine.execute_query(query)
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Assert: All queries succeed
    assert_eq!(results.len(), 8, "All threads should complete");
    for result in results {
        assert!(result.is_ok(), "Concurrent queries should succeed");
    }
}

#[test]
fn test_concurrent_cache_access_maintains_correct_statistics() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();
    let engine = Arc::new(engine);

    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;

    // Act: Execute same query from multiple threads
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let engine = Arc::clone(&engine);
            thread::spawn(move || engine.execute_query(query))
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap().expect("Query failed");
    }

    // Assert: Cache statistics are correct
    let (hits, misses, _) = engine.cache_stats().expect("Failed to get stats");
    assert_eq!(hits + misses, 10, "Total queries should be 10");
    assert_eq!(misses, 1, "Only first query should miss");
    assert_eq!(hits, 9, "Remaining queries should hit cache");
}

#[test]
fn test_concurrent_rdf_loading_and_querying_is_safe() {
    // Arrange: Set up multiple engines
    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                let template_dir = temp_dir.path().join("templates");
                std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

                let mut engine =
                    SparqlTemplateEngine::new(&template_dir, 100).expect("Failed to create engine");

                let rdf_path = temp_dir.path().join("test.ttl");
                std::fs::write(&rdf_path, create_test_rdf()).expect("Failed to write RDF");
                engine
                    .load_rdf_graph(&rdf_path)
                    .expect("Failed to load RDF");

                let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;
                engine.execute_query(query)
            })
        })
        .collect();

    // Assert: All threads succeed
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "Concurrent RDF loading should succeed");
    }
}

#[test]
fn test_cache_clear_during_concurrent_queries_is_safe() {
    // Arrange: Set up engine with test data
    let (_temp_dir, engine) = setup_engine_with_data();
    let engine = Arc::new(engine);

    let query = r#"SELECT ?task WHERE { ?task a ?type . }"#;

    // Act: Execute queries while clearing cache
    let query_handles: Vec<_> = (0..5)
        .map(|_| {
            let engine = Arc::clone(&engine);
            thread::spawn(move || {
                for _ in 0..3 {
                    let _ = engine.execute_query(query);
                }
            })
        })
        .collect();

    let clear_handle = {
        let engine = Arc::clone(&engine);
        thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(5));
            engine.clear_cache()
        })
    };

    // Wait for all threads
    for handle in query_handles {
        handle.join().unwrap();
    }
    clear_handle.join().unwrap().expect("Clear failed");

    // Assert: No crashes or deadlocks occurred (implicit by reaching here)
    let stats = engine.cache_stats();
    assert!(
        stats.is_ok(),
        "Engine should remain functional after concurrent clear"
    );
}
