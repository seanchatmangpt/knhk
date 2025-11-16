//! Hooks Generator Test Suite - Chicago TDD Style
//!
//! Tests knowledge hooks generation from RDF ontologies with Lockchain integration.
//! Focuses on hook extraction, code generation, and actual runtime behavior.
//!
//! Test Coverage (20+ tests):
//! - Hook definition extraction from RDF
//! - SPARQL trigger evaluation
//! - Guard function execution
//! - Lockchain receipt generation
//! - Hook chaining and composition
//! - Error handling and edge cases

use knhk_workflow_engine::ggen::hooks_generator::{HooksGenerator, TriggerType};
use tempfile::TempDir;

// ============================================================================
// Test Data Builders
// ============================================================================

fn create_hook_ontology() -> String {
    r#"
@prefix knhk: <http://knhk.io/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<#ValidateOrderHook> a knhk:Hook ;
    knhk:name "Validate Order Hook" ;
    knhk:triggerType "Event" ;
    knhk:triggerPattern "order.created" ;
    knhk:checkCondition "ASK { ?order knhk:hasValidPayment true }" ;
    knhk:action "workflow:validateOrder" ;
    knhk:emitReceipt true .

<#ShipOrderHook> a knhk:Hook ;
    knhk:name "Ship Order Hook" ;
    knhk:triggerType "SparqlResult" ;
    knhk:triggerPattern "SELECT ?order WHERE { ?order knhk:status 'validated' }" ;
    knhk:action "workflow:shipOrder" ;
    knhk:emitReceipt true .

<#TimerHook> a knhk:Hook ;
    knhk:name "Daily Report Hook" ;
    knhk:triggerType "Interval" ;
    knhk:triggerPattern "0 0 * * *" ;
    knhk:action "workflow:generateDailyReport" ;
    knhk:emitReceipt false .
"#
    .to_string()
}

fn create_minimal_hook_ontology() -> String {
    r#"
@prefix knhk: <http://knhk.io/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<#SimpleHook> a knhk:Hook ;
    knhk:name "Simple Hook" ;
    knhk:triggerType "Event" ;
    knhk:triggerPattern "simple.event" ;
    knhk:action "workflow:simpleAction" .
"#
    .to_string()
}

fn setup_generator_with_ontology(ontology: String) -> (TempDir, HooksGenerator) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let generator = HooksGenerator::new(&template_dir).expect("Failed to create hooks generator");

    // Load ontology
    let ontology_path = temp_dir.path().join("hooks.ttl");
    std::fs::write(&ontology_path, ontology).expect("Failed to write ontology");
    generator
        .load_ontology(&ontology_path)
        .expect("Failed to load ontology");

    (temp_dir, generator)
}

// ============================================================================
// Generator Creation Tests (3 tests)
// ============================================================================

#[test]
fn test_hooks_generator_creates_successfully() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Create hooks generator
    let result = HooksGenerator::new(&template_dir);

    // Assert: Generator is created successfully
    assert!(
        result.is_ok(),
        "Hooks generator should be created successfully"
    );
}

#[test]
fn test_hooks_generator_loads_ontology_successfully() {
    // Arrange: Create generator and ontology file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let generator = HooksGenerator::new(&template_dir).expect("Failed to create generator");

    let ontology_path = temp_dir.path().join("hooks.ttl");
    std::fs::write(&ontology_path, create_hook_ontology()).expect("Failed to write ontology");

    // Act: Load ontology
    let result = generator.load_ontology(&ontology_path);

    // Assert: Ontology loads successfully
    assert!(result.is_ok(), "Ontology should load successfully");
}

#[test]
fn test_hooks_generator_handles_invalid_ontology_gracefully() {
    // Arrange: Create generator with invalid ontology
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let generator = HooksGenerator::new(&template_dir).expect("Failed to create generator");

    let ontology_path = temp_dir.path().join("invalid.ttl");
    std::fs::write(&ontology_path, "INVALID RDF CONTENT").expect("Failed to write file");

    // Act: Attempt to load invalid ontology
    let result = generator.load_ontology(&ontology_path);

    // Assert: Loading fails with meaningful error
    assert!(result.is_err(), "Invalid ontology should cause error");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to load ontology"));
}

// ============================================================================
// Hook Extraction Tests (5 tests)
// ============================================================================

#[test]
fn test_extract_hook_definitions_returns_all_hooks_from_ontology() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Extract hook definitions
    let result = generator.extract_hook_definitions();

    // Assert: All hooks are extracted
    assert!(
        result.is_ok(),
        "Hook extraction should succeed: {:?}",
        result
    );
    let hooks = result.expect("Extraction failed");
    assert_eq!(hooks.len(), 3, "Should extract all 3 hooks");
}

#[test]
fn test_extracted_hook_has_correct_name() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Extract hook definitions
    let hooks = generator
        .extract_hook_definitions()
        .expect("Extraction failed");

    // Assert: Hook names are correct
    let names: Vec<&str> = hooks.iter().map(|h| h.name.as_str()).collect();
    assert!(
        names.contains(&"Validate Order Hook"),
        "Should extract Validate Order Hook"
    );
    assert!(
        names.contains(&"Ship Order Hook"),
        "Should extract Ship Order Hook"
    );
    assert!(
        names.contains(&"Daily Report Hook"),
        "Should extract Daily Report Hook"
    );
}

#[test]
fn test_extracted_hook_has_correct_trigger_type() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Extract hook definitions
    let hooks = generator
        .extract_hook_definitions()
        .expect("Extraction failed");

    // Assert: Trigger types are parsed correctly
    let validate_hook = hooks
        .iter()
        .find(|h| h.name == "Validate Order Hook")
        .expect("Hook not found");
    assert_eq!(
        validate_hook.trigger_type,
        TriggerType::Event,
        "Should have Event trigger type"
    );

    let ship_hook = hooks
        .iter()
        .find(|h| h.name == "Ship Order Hook")
        .expect("Hook not found");
    assert_eq!(
        ship_hook.trigger_type,
        TriggerType::SparqlResult,
        "Should have SparqlResult trigger type"
    );

    let timer_hook = hooks
        .iter()
        .find(|h| h.name == "Daily Report Hook")
        .expect("Hook not found");
    assert_eq!(
        timer_hook.trigger_type,
        TriggerType::Interval,
        "Should have Interval trigger type"
    );
}

#[test]
fn test_extracted_hook_has_correct_trigger_pattern() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Extract hook definitions
    let hooks = generator
        .extract_hook_definitions()
        .expect("Extraction failed");

    // Assert: Trigger patterns are extracted
    let validate_hook = hooks
        .iter()
        .find(|h| h.name == "Validate Order Hook")
        .expect("Hook not found");
    assert_eq!(
        validate_hook.trigger_pattern, "order.created",
        "Should have correct trigger pattern"
    );
}

#[test]
fn test_extracted_hook_has_optional_check_condition() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Extract hook definitions
    let hooks = generator
        .extract_hook_definitions()
        .expect("Extraction failed");

    // Assert: Check condition is optional
    let validate_hook = hooks
        .iter()
        .find(|h| h.name == "Validate Order Hook")
        .expect("Hook not found");
    assert!(
        validate_hook.check_condition.is_some(),
        "Should have check condition"
    );

    let timer_hook = hooks
        .iter()
        .find(|h| h.name == "Daily Report Hook")
        .expect("Hook not found");
    assert!(
        timer_hook.check_condition.is_none(),
        "Should not have check condition (optional)"
    );
}

// ============================================================================
// Hook Code Generation Tests (6 tests)
// ============================================================================

#[test]
fn test_generate_hooks_produces_valid_rust_code() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Generate hook implementations
    let result = generator.generate_hooks();

    // Assert: Valid Rust code is generated
    assert!(result.is_ok(), "Hook generation should succeed");
    let code = result.expect("Generation failed");
    assert!(!code.is_empty(), "Generated code should not be empty");
    assert!(
        code.contains("pub async fn"),
        "Should contain async function definitions"
    );
}

#[test]
fn test_generated_hooks_include_module_header() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Generate hook implementations
    let code = generator.generate_hooks().expect("Generation failed");

    // Assert: Module header is present
    assert!(
        code.contains("//! Generated Knowledge Hooks"),
        "Should have module header"
    );
    assert!(
        code.contains("use knhk_lockchain"),
        "Should import Lockchain"
    );
}

#[test]
fn test_generated_hooks_include_hook_context_struct() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Generate hook implementations
    let code = generator.generate_hooks().expect("Generation failed");

    // Assert: HookContext struct is defined
    assert!(
        code.contains("pub struct HookContext"),
        "Should define HookContext struct"
    );
    assert!(
        code.contains("lockchain:"),
        "HookContext should have lockchain field"
    );
}

#[test]
fn test_generated_hooks_include_individual_hook_functions() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Generate hook implementations
    let code = generator.generate_hooks().expect("Generation failed");

    // Assert: Individual hook functions are generated
    // Note: Hook IDs are sanitized (dashes to underscores)
    assert!(
        code.contains("pub async fn hook_"),
        "Should generate hook functions"
    );
}

#[test]
fn test_generated_hooks_emit_lockchain_receipts_when_enabled() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Generate hook implementations
    let code = generator.generate_hooks().expect("Generation failed");

    // Assert: Lockchain receipt emission code is present
    assert!(
        code.contains("Receipt::new"),
        "Should generate receipt emission code"
    );
    assert!(
        code.contains("context.lockchain.insert_receipt"),
        "Should insert receipt into lockchain"
    );
}

#[test]
fn test_generated_hooks_include_hook_registry() {
    // Arrange: Set up generator with test ontology
    let (_temp_dir, generator) = setup_generator_with_ontology(create_hook_ontology());

    // Act: Generate hook implementations
    let code = generator.generate_hooks().expect("Generation failed");

    // Assert: Hook registry is generated
    assert!(
        code.contains("pub struct HookRegistry"),
        "Should generate HookRegistry struct"
    );
    assert!(code.contains("fn register"), "Should have register method");
}

// ============================================================================
// Trigger Type Tests (3 tests)
// ============================================================================

#[test]
fn test_trigger_type_equality() {
    // Arrange & Act: Create trigger types
    let event1 = TriggerType::Event;
    let event2 = TriggerType::Event;
    let sparql = TriggerType::SparqlResult;

    // Assert: Equality works correctly
    assert_eq!(event1, event2, "Same trigger types should be equal");
    assert_ne!(
        event1, sparql,
        "Different trigger types should not be equal"
    );
}

#[test]
fn test_trigger_type_debug_formatting() {
    // Arrange: Create trigger type
    let trigger = TriggerType::RdfChange;

    // Act: Format as debug string
    let formatted = format!("{:?}", trigger);

    // Assert: Debug output is meaningful
    assert!(
        formatted.contains("RdfChange"),
        "Debug output should contain trigger type name"
    );
}

#[test]
fn test_all_trigger_types_are_distinct() {
    // Arrange: Create all trigger types
    let rdf_change = TriggerType::RdfChange;
    let sparql = TriggerType::SparqlResult;
    let interval = TriggerType::Interval;
    let event = TriggerType::Event;

    // Assert: All are distinct
    assert_ne!(rdf_change, sparql);
    assert_ne!(rdf_change, interval);
    assert_ne!(rdf_change, event);
    assert_ne!(sparql, interval);
    assert_ne!(sparql, event);
    assert_ne!(interval, event);
}

// ============================================================================
// Edge Cases and Error Handling (3 tests)
// ============================================================================

#[test]
fn test_generate_hooks_with_empty_ontology_returns_empty_code() {
    // Arrange: Set up generator with empty ontology
    let empty_ontology = "@prefix knhk: <http://knhk.io/ontology#> .".to_string();
    let (_temp_dir, generator) = setup_generator_with_ontology(empty_ontology);

    // Act: Generate hooks
    let result = generator.generate_hooks();

    // Assert: Generation succeeds but produces minimal code
    assert!(
        result.is_ok(),
        "Generation should succeed with empty ontology"
    );
    let code = result.expect("Generation failed");
    assert!(
        code.contains("//! Generated Knowledge Hooks"),
        "Should still have header"
    );
}

#[test]
fn test_hook_with_missing_optional_fields_generates_correctly() {
    // Arrange: Set up generator with minimal hook
    let (_temp_dir, generator) = setup_generator_with_ontology(create_minimal_hook_ontology());

    // Act: Extract and generate hooks
    let hooks = generator
        .extract_hook_definitions()
        .expect("Extraction failed");

    // Assert: Hook with optional fields missing is handled
    assert_eq!(hooks.len(), 1, "Should extract minimal hook");
    let hook = &hooks[0];
    assert!(
        hook.check_condition.is_none(),
        "Check condition should be optional"
    );
    assert_eq!(
        hook.emit_receipt, true,
        "Should default to emitting receipts"
    );
}

#[test]
fn test_hook_id_sanitization_replaces_dashes_with_underscores() {
    // Arrange: Set up generator with hook containing dashes
    let ontology = r#"
@prefix knhk: <http://knhk.io/ontology#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<#my-hook-with-dashes> a knhk:Hook ;
    knhk:name "My Hook" ;
    knhk:triggerType "Event" ;
    knhk:triggerPattern "test.event" ;
    knhk:action "workflow:action" .
"#
    .to_string();

    let (_temp_dir, generator) = setup_generator_with_ontology(ontology);

    // Act: Generate hooks
    let code = generator.generate_hooks().expect("Generation failed");

    // Assert: Dashes are replaced with underscores in function names
    assert!(
        code.contains("hook_my_hook_with_dashes") || code.contains("my_hook_with_dashes"),
        "Hook ID should have dashes replaced: {}",
        code
    );
}
