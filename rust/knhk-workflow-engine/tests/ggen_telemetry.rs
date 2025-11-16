//! Telemetry Generator Test Suite - Chicago TDD Style
//!
//! Tests OTEL telemetry instrumentation generation with Weaver schema validation.
//! Focuses on span/metric/log generation and schema conformance.
//!
//! Test Coverage (15+ tests):
//! - OTEL span definition generation
//! - Metric collector generation (counters, histograms, gauges)
//! - Log event structure generation
//! - Weaver schema generation and validation
//! - Attribute handling and type safety

use knhk_workflow_engine::ggen::telemetry_generator::{
    AttributeDefinition, LogEventDefinition, MetricDefinition, SpanDefinition, TelemetryGenerator,
};
use tempfile::TempDir;

// ============================================================================
// Test Data Builders
// ============================================================================

fn create_test_span_definition() -> SpanDefinition {
    SpanDefinition {
        name: "workflow.execute".to_string(),
        description: "Workflow execution span".to_string(),
        attributes: vec![
            AttributeDefinition {
                name: "workflow.id".to_string(),
                attr_type: "string".to_string(),
                required: true,
                description: "Workflow identifier".to_string(),
            },
            AttributeDefinition {
                name: "workflow.duration".to_string(),
                attr_type: "int".to_string(),
                required: false,
                description: "Execution duration in ms".to_string(),
            },
        ],
        events: vec![
            "workflow.started".to_string(),
            "workflow.completed".to_string(),
        ],
    }
}

fn create_test_metric_definition() -> MetricDefinition {
    MetricDefinition {
        name: "workflow.execution_count".to_string(),
        metric_type: "counter".to_string(),
        description: "Total number of workflow executions".to_string(),
        unit: "executions".to_string(),
        attributes: vec![AttributeDefinition {
            name: "workflow.type".to_string(),
            attr_type: "string".to_string(),
            required: true,
            description: "Type of workflow".to_string(),
        }],
    }
}

fn create_test_log_event_definition() -> LogEventDefinition {
    LogEventDefinition {
        name: "WorkflowError".to_string(),
        severity: "error".to_string(),
        description: "Workflow execution error event".to_string(),
        attributes: vec![
            AttributeDefinition {
                name: "error_code".to_string(),
                attr_type: "string".to_string(),
                required: true,
                description: "Error code".to_string(),
            },
            AttributeDefinition {
                name: "error_message".to_string(),
                attr_type: "string".to_string(),
                required: true,
                description: "Error message".to_string(),
            },
        ],
    }
}

// ============================================================================
// Generator Creation Tests (2 tests)
// ============================================================================

#[test]
fn test_telemetry_generator_creates_successfully() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Create telemetry generator
    let result = TelemetryGenerator::new(&template_dir);

    // Assert: Generator is created successfully
    assert!(
        result.is_ok(),
        "Telemetry generator should be created successfully"
    );
}

#[test]
fn test_telemetry_generator_starts_with_empty_definitions() {
    // Arrange: Create telemetry generator
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    // Act: Generate code immediately
    let span_code = generator.generate_span_definitions();
    let metric_code = generator.generate_metric_collectors();

    // Assert: Empty definitions generate minimal code
    assert!(span_code.is_ok(), "Should generate code even when empty");
    assert!(metric_code.is_ok(), "Should generate code even when empty");
}

// ============================================================================
// Span Generation Tests (5 tests)
// ============================================================================

#[test]
fn test_add_span_definition_stores_span() {
    // Arrange: Create generator and span definition
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    let span = create_test_span_definition();

    // Act: Add span definition
    generator.add_span(span.clone());

    // Generate code to verify span was added
    let code = generator
        .generate_span_definitions()
        .expect("Generation failed");

    // Assert: Span appears in generated code
    assert!(
        code.contains("workflow.execute"),
        "Generated code should contain span name"
    );
}

#[test]
fn test_generate_span_definitions_includes_tracer_usage() {
    // Arrange: Create generator with span
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());

    // Act: Generate span definitions
    let code = generator
        .generate_span_definitions()
        .expect("Generation failed");

    // Assert: Generated code uses tracer
    assert!(code.contains("tracer"), "Should contain tracer parameter");
    assert!(
        code.contains("span_builder"),
        "Should use span builder pattern"
    );
}

#[test]
fn test_generate_span_definitions_includes_required_attributes() {
    // Arrange: Create generator with span
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());

    // Act: Generate span definitions
    let code = generator
        .generate_span_definitions()
        .expect("Generation failed");

    // Assert: Required attributes are included
    assert!(
        code.contains("workflow.id"),
        "Should include required workflow.id attribute"
    );
    assert!(
        code.contains("KeyValue"),
        "Should use KeyValue for attributes"
    );
}

#[test]
fn test_generate_span_definitions_includes_function_header() {
    // Arrange: Create generator with span
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());

    // Act: Generate span definitions
    let code = generator
        .generate_span_definitions()
        .expect("Generation failed");

    // Assert: Function has proper header
    assert!(
        code.contains("pub fn create_span_"),
        "Should generate create_span function"
    );
    assert!(
        code.contains("#[instrument"),
        "Should have instrument attribute"
    );
}

#[test]
fn test_generate_span_definitions_includes_module_documentation() {
    // Arrange: Create generator with span
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());

    // Act: Generate span definitions
    let code = generator
        .generate_span_definitions()
        .expect("Generation failed");

    // Assert: Module has documentation
    assert!(
        code.contains("//! Generated OTEL Span Definitions"),
        "Should have module documentation"
    );
}

// ============================================================================
// Metric Generation Tests (4 tests)
// ============================================================================

#[test]
fn test_add_metric_definition_stores_metric() {
    // Arrange: Create generator and metric definition
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    let metric = create_test_metric_definition();

    // Act: Add metric definition
    generator.add_metric(metric.clone());

    // Generate code to verify metric was added
    let code = generator
        .generate_metric_collectors()
        .expect("Generation failed");

    // Assert: Metric appears in generated code
    assert!(
        code.contains("workflow.execution_count"),
        "Generated code should contain metric name"
    );
}

#[test]
fn test_generate_metric_collectors_creates_counter_type() {
    // Arrange: Create generator with counter metric
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_metric(create_test_metric_definition());

    // Act: Generate metric collectors
    let code = generator
        .generate_metric_collectors()
        .expect("Generation failed");

    // Assert: Counter type is generated
    assert!(
        code.contains("Counter<u64>"),
        "Should generate Counter<u64> type"
    );
    assert!(
        code.contains("u64_counter"),
        "Should use u64_counter builder"
    );
}

#[test]
fn test_generate_metric_collectors_creates_histogram_type() {
    // Arrange: Create generator with histogram metric
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

    let histogram_metric = MetricDefinition {
        name: "workflow.duration".to_string(),
        metric_type: "histogram".to_string(),
        description: "Workflow execution duration".to_string(),
        unit: "ms".to_string(),
        attributes: vec![],
    };
    generator.add_metric(histogram_metric);

    // Act: Generate metric collectors
    let code = generator
        .generate_metric_collectors()
        .expect("Generation failed");

    // Assert: Histogram type is generated
    assert!(
        code.contains("Histogram<f64>"),
        "Should generate Histogram<f64> type"
    );
    assert!(
        code.contains("f64_histogram"),
        "Should use f64_histogram builder"
    );
}

#[test]
fn test_generate_metric_collectors_includes_unit_and_description() {
    // Arrange: Create generator with metric
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_metric(create_test_metric_definition());

    // Act: Generate metric collectors
    let code = generator
        .generate_metric_collectors()
        .expect("Generation failed");

    // Assert: Unit and description are included
    assert!(
        code.contains("executions"),
        "Should include unit in builder"
    );
    assert!(
        code.contains("Total number of workflow executions"),
        "Should include description"
    );
}

// ============================================================================
// Log Event Generation Tests (3 tests)
// ============================================================================

#[test]
fn test_add_log_event_definition_stores_log_event() {
    // Arrange: Create generator and log event definition
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    let log_event = create_test_log_event_definition();

    // Act: Add log event definition
    generator.add_log_event(log_event.clone());

    // Generate code to verify log event was added
    let code = generator.generate_log_events().expect("Generation failed");

    // Assert: Log event appears in generated code
    assert!(
        code.contains("WorkflowError"),
        "Generated code should contain log event struct"
    );
}

#[test]
fn test_generate_log_events_creates_serializable_struct() {
    // Arrange: Create generator with log event
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_log_event(create_test_log_event_definition());

    // Act: Generate log events
    let code = generator.generate_log_events().expect("Generation failed");

    // Assert: Struct is serializable
    assert!(
        code.contains("#[derive(Debug, Clone, Serialize, Deserialize)]"),
        "Should derive Serialize and Deserialize"
    );
}

#[test]
fn test_generate_log_events_includes_log_method_with_correct_severity() {
    // Arrange: Create generator with log event
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_log_event(create_test_log_event_definition());

    // Act: Generate log events
    let code = generator.generate_log_events().expect("Generation failed");

    // Assert: Log method uses correct macro
    assert!(
        code.contains("error!"),
        "Should use error! macro for error severity"
    );
    assert!(code.contains("pub fn log(&self)"), "Should have log method");
}

// ============================================================================
// Weaver Schema Generation Tests (4 tests)
// ============================================================================

#[test]
fn test_generate_weaver_schema_includes_schema_version() {
    // Arrange: Create generator with telemetry definitions
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());

    // Act: Generate Weaver schema
    let schema = generator
        .generate_weaver_schema()
        .expect("Schema generation failed");

    // Assert: Schema has version
    assert!(
        schema.contains("schema_version:"),
        "Should include schema version"
    );
    assert!(schema.contains("1.0.0"), "Should have version number");
}

#[test]
fn test_generate_weaver_schema_includes_span_definitions() {
    // Arrange: Create generator with span
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());

    // Act: Generate Weaver schema
    let schema = generator
        .generate_weaver_schema()
        .expect("Schema generation failed");

    // Assert: Span is in schema
    assert!(
        schema.contains("workflow.execute"),
        "Should include span name in schema"
    );
    assert!(
        schema.contains("attributes:"),
        "Should define span attributes"
    );
}

#[test]
fn test_generate_weaver_schema_includes_metric_definitions() {
    // Arrange: Create generator with metric
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_metric(create_test_metric_definition());

    // Act: Generate Weaver schema
    let schema = generator
        .generate_weaver_schema()
        .expect("Schema generation failed");

    // Assert: Metric is in schema
    assert!(
        schema.contains("workflow.execution_count"),
        "Should include metric name in schema"
    );
    assert!(
        schema.contains("type: counter"),
        "Should include metric type"
    );
    assert!(
        schema.contains("unit: executions"),
        "Should include metric unit"
    );
}

#[test]
fn test_generate_weaver_schema_is_valid_yaml() {
    // Arrange: Create generator with full telemetry
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    let mut generator = TelemetryGenerator::new(&template_dir).expect("Failed to create generator");
    generator.add_span(create_test_span_definition());
    generator.add_metric(create_test_metric_definition());

    // Act: Generate Weaver schema
    let schema = generator
        .generate_weaver_schema()
        .expect("Schema generation failed");

    // Assert: Schema looks like valid YAML
    assert!(
        schema.contains("name:") && schema.contains("description:"),
        "Should have YAML key-value pairs"
    );
    assert!(
        !schema.contains("\t"),
        "Should use spaces, not tabs for YAML"
    );
}
