//! OTEL Telemetry Generator - Auto-generate OpenTelemetry instrumentation
//!
//! Generates OTEL instrumentation from specifications:
//! - Span definitions with attributes
//! - Metric collectors (counters, histograms, gauges)
//! - Log event structures
//! - Weaver schema definitions
//! - Schema validation integration
//!
//! # Features
//!
//! - Schema-first approach (Weaver integration)
//! - Type-safe span/metric definitions
//! - Automatic schema generation
//! - Validation against schema
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::ggen::telemetry_generator::TelemetryGenerator;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let generator = TelemetryGenerator::new("templates/telemetry")?;
//! let spans = generator.generate_span_definitions()?;
//! let schema = generator.generate_weaver_schema()?;
//! # Ok(())
//! # }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tera::{Context, Tera};
use tracing::{debug, info, instrument};

/// Span definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanDefinition {
    /// Span name
    pub name: String,
    /// Span description
    pub description: String,
    /// Span attributes
    pub attributes: Vec<AttributeDefinition>,
    /// Span events
    pub events: Vec<String>,
}

/// Attribute definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    /// Attribute name
    pub name: String,
    /// Attribute type (string, int, bool, float)
    pub attr_type: String,
    /// Whether attribute is required
    pub required: bool,
    /// Attribute description
    pub description: String,
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// Metric name
    pub name: String,
    /// Metric type (counter, histogram, gauge)
    pub metric_type: String,
    /// Metric description
    pub description: String,
    /// Metric unit
    pub unit: String,
    /// Metric attributes
    pub attributes: Vec<AttributeDefinition>,
}

/// Log event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEventDefinition {
    /// Event name
    pub name: String,
    /// Event severity (debug, info, warn, error)
    pub severity: String,
    /// Event description
    pub description: String,
    /// Event attributes
    pub attributes: Vec<AttributeDefinition>,
}

/// Telemetry generator
pub struct TelemetryGenerator {
    /// Tera template engine
    tera: Tera,
    /// Span definitions
    spans: Vec<SpanDefinition>,
    /// Metric definitions
    metrics: Vec<MetricDefinition>,
    /// Log event definitions
    log_events: Vec<LogEventDefinition>,
}

impl TelemetryGenerator {
    /// Create new telemetry generator
    ///
    /// # Arguments
    ///
    /// * `template_dir` - Directory containing telemetry templates
    ///
    /// # Errors
    ///
    /// Returns error if template directory is invalid or Tera initialization fails.
    #[instrument(skip(template_dir))]
    pub fn new(template_dir: impl AsRef<Path>) -> WorkflowResult<Self> {
        let template_dir = template_dir.as_ref();

        // Initialize Tera with telemetry templates
        let template_pattern = template_dir
            .join("**/*.tera")
            .to_str()
            .ok_or_else(|| WorkflowError::Internal("Invalid template path".to_string()))?
            .to_string();

        let tera = Tera::new(&template_pattern)
            .map_err(|e| WorkflowError::Internal(format!("Failed to initialize Tera: {}", e)))?;

        info!("Created telemetry generator");

        Ok(Self {
            tera,
            spans: Vec::new(),
            metrics: Vec::new(),
            log_events: Vec::new(),
        })
    }

    /// Add span definition
    pub fn add_span(&mut self, span: SpanDefinition) {
        debug!("Added span definition: {}", span.name);
        self.spans.push(span);
    }

    /// Add metric definition
    pub fn add_metric(&mut self, metric: MetricDefinition) {
        debug!("Added metric definition: {}", metric.name);
        self.metrics.push(metric);
    }

    /// Add log event definition
    pub fn add_log_event(&mut self, log_event: LogEventDefinition) {
        debug!("Added log event definition: {}", log_event.name);
        self.log_events.push(log_event);
    }

    /// Generate span definition code
    ///
    /// Generates Rust code for OTEL span creation with attributes.
    ///
    /// # Errors
    ///
    /// Returns error if code generation fails.
    #[instrument(skip(self))]
    pub fn generate_span_definitions(&self) -> WorkflowResult<String> {
        let mut code = String::from(
            r#"//! Generated OTEL Span Definitions
//!
//! Auto-generated from telemetry specifications.

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use tracing::{instrument, span, Level};

"#,
        );

        for span_def in &self.spans {
            code.push_str(&self.generate_span_function(span_def)?);
            code.push('\n');
        }

        Ok(code)
    }

    /// Generate individual span function
    fn generate_span_function(&self, span_def: &SpanDefinition) -> WorkflowResult<String> {
        let span_name = &span_def.name;
        let description = &span_def.description;

        // Generate attribute parameters
        let mut params = Vec::new();
        let mut attributes = Vec::new();

        for attr in &span_def.attributes {
            if attr.required {
                let param_type = match attr.attr_type.as_str() {
                    "string" => "String",
                    "int" => "i64",
                    "bool" => "bool",
                    "float" => "f64",
                    _ => "String",
                };
                params.push(format!("{}: {}", attr.name, param_type));

                let attr_value = match attr.attr_type.as_str() {
                    "string" => format!("{}.into()", attr.name),
                    "int" => format!("{}  as i64", attr.name),
                    "bool" => format!("{} as i64", attr.name),
                    "float" => format!("{}", attr.name),
                    _ => format!("{}.into()", attr.name),
                };

                attributes.push(format!(
                    r#"        KeyValue::new("{}", {})"#,
                    attr.name, attr_value
                ));
            }
        }

        let params_str = if params.is_empty() {
            String::new()
        } else {
            format!(", {}", params.join(", "))
        };

        let attributes_str = if attributes.is_empty() {
            String::new()
        } else {
            format!("\n{}", attributes.join(",\n"))
        };

        let code = format!(
            r#"/// {}
#[instrument(skip(tracer))]
pub fn create_span_{}(tracer: &impl Tracer{}) -> impl Span {{
    tracer
        .span_builder("{}")
        .with_attributes(vec![{}
        ])
        .start(tracer)
}}
"#,
            description,
            span_name.replace('-', "_"),
            params_str,
            span_name,
            attributes_str
        );

        Ok(code)
    }

    /// Generate metric collector code
    ///
    /// Generates Rust code for OTEL metric collectors.
    ///
    /// # Errors
    ///
    /// Returns error if code generation fails.
    #[instrument(skip(self))]
    pub fn generate_metric_collectors(&self) -> WorkflowResult<String> {
        let mut code = String::from(
            r#"//! Generated OTEL Metric Collectors
//!
//! Auto-generated from telemetry specifications.

use opentelemetry::metrics::{Counter, Histogram, Gauge, Meter};
use opentelemetry::KeyValue;

"#,
        );

        for metric_def in &self.metrics {
            code.push_str(&self.generate_metric_collector(metric_def)?);
            code.push('\n');
        }

        Ok(code)
    }

    /// Generate individual metric collector
    fn generate_metric_collector(&self, metric_def: &MetricDefinition) -> WorkflowResult<String> {
        let metric_name = &metric_def.name;
        let description = &metric_def.description;
        let unit = &metric_def.unit;

        let collector_type = match metric_def.metric_type.as_str() {
            "counter" => "Counter<u64>",
            "histogram" => "Histogram<f64>",
            "gauge" => "Gauge<f64>",
            _ => "Counter<u64>",
        };

        let builder_method = match metric_def.metric_type.as_str() {
            "counter" => "u64_counter",
            "histogram" => "f64_histogram",
            "gauge" => "f64_gauge",
            _ => "u64_counter",
        };

        let code = format!(
            r#"/// {}
pub fn create_metric_{}(meter: &Meter) -> {} {{
    meter
        .{}("{}")
        .with_description("{}")
        .with_unit("{}")
        .init()
}}
"#,
            description,
            metric_name.replace('-', "_"),
            collector_type,
            builder_method,
            metric_name,
            description,
            unit
        );

        Ok(code)
    }

    /// Generate log event structures
    ///
    /// Generates Rust structs for structured logging.
    ///
    /// # Errors
    ///
    /// Returns error if code generation fails.
    #[instrument(skip(self))]
    pub fn generate_log_events(&self) -> WorkflowResult<String> {
        let mut code = String::from(
            r#"//! Generated Log Event Structures
//!
//! Auto-generated from telemetry specifications.

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

"#,
        );

        for log_def in &self.log_events {
            code.push_str(&self.generate_log_event_struct(log_def)?);
            code.push('\n');
        }

        Ok(code)
    }

    /// Generate individual log event struct
    fn generate_log_event_struct(&self, log_def: &LogEventDefinition) -> WorkflowResult<String> {
        let event_name = &log_def.name;
        let description = &log_def.description;

        let mut fields = Vec::new();
        for attr in &log_def.attributes {
            let field_type = match attr.attr_type.as_str() {
                "string" => "String",
                "int" => "i64",
                "bool" => "bool",
                "float" => "f64",
                _ => "String",
            };
            fields.push(format!("    pub {}: {}", attr.name, field_type));
        }

        let fields_str = fields.join(",\n");

        let log_macro = match log_def.severity.as_str() {
            "debug" => "debug",
            "info" => "info",
            "warn" => "warn",
            "error" => "error",
            _ => "info",
        };

        let code = format!(
            r#"/// {}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
{}
}}

impl {} {{
    /// Log this event
    pub fn log(&self) {{
        {}!("{}: {{:?}}", self);
    }}
}}
"#,
            description, event_name, fields_str, event_name, log_macro, event_name
        );

        Ok(code)
    }

    /// Generate Weaver schema definition
    ///
    /// Generates YAML schema for OpenTelemetry Weaver validation.
    ///
    /// # Errors
    ///
    /// Returns error if schema generation fails.
    #[instrument(skip(self))]
    pub fn generate_weaver_schema(&self) -> WorkflowResult<String> {
        let mut schema = String::from(
            r#"# Generated OpenTelemetry Weaver Schema
# Auto-generated from telemetry specifications

schema_version: 1.0.0
name: knhk-workflow-engine
version: 1.0.0

spans:
"#,
        );

        // Generate span schema definitions
        for span_def in &self.spans {
            schema.push_str(&format!("  - name: {}\n", span_def.name));
            schema.push_str(&format!("    description: {}\n", span_def.description));
            schema.push_str("    attributes:\n");

            for attr in &span_def.attributes {
                schema.push_str(&format!("      - name: {}\n", attr.name));
                schema.push_str(&format!("        type: {}\n", attr.attr_type));
                schema.push_str(&format!("        required: {}\n", attr.required));
                schema.push_str(&format!("        description: {}\n", attr.description));
            }
        }

        schema.push_str("\nmetrics:\n");

        // Generate metric schema definitions
        for metric_def in &self.metrics {
            schema.push_str(&format!("  - name: {}\n", metric_def.name));
            schema.push_str(&format!("    type: {}\n", metric_def.metric_type));
            schema.push_str(&format!("    unit: {}\n", metric_def.unit));
            schema.push_str(&format!("    description: {}\n", metric_def.description));
        }

        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_telemetry_generator_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

        let generator = TelemetryGenerator::new(&template_dir);
        assert!(
            generator.is_ok(),
            "Generator should be created successfully"
        );
    }

    #[test]
    fn test_add_span_definition() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

        let mut generator =
            TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

        let span = SpanDefinition {
            name: "workflow.execute".to_string(),
            description: "Workflow execution span".to_string(),
            attributes: vec![AttributeDefinition {
                name: "workflow.id".to_string(),
                attr_type: "string".to_string(),
                required: true,
                description: "Workflow identifier".to_string(),
            }],
            events: vec![],
        };

        generator.add_span(span);

        let code = generator.generate_span_definitions();
        assert!(code.is_ok(), "Span code generation should succeed");
    }

    #[test]
    fn test_generate_weaver_schema() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let template_dir = temp_dir.path().join("templates");
        std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

        let mut generator =
            TelemetryGenerator::new(&template_dir).expect("Failed to create generator");

        let metric = MetricDefinition {
            name: "workflow.duration".to_string(),
            metric_type: "histogram".to_string(),
            description: "Workflow execution duration".to_string(),
            unit: "ms".to_string(),
            attributes: vec![],
        };

        generator.add_metric(metric);

        let schema = generator.generate_weaver_schema();
        assert!(schema.is_ok(), "Schema generation should succeed");

        let schema_content = schema.expect("Schema generation failed");
        assert!(schema_content.contains("workflow.duration"));
    }
}
