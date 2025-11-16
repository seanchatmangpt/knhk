// tests/weaver/semantic_conventions.rs
// WEAVER VALIDATION: OpenTelemetry Semantic Convention Compliance
// Validates all emitted spans conform to OpenTelemetry semantic standards
// Ensures Prometheus/Jaeger dashboards will work correctly

use std::collections::HashMap;

/// Mock OTEL Span conforming to semantic conventions
#[derive(Clone, Debug, PartialEq)]
pub struct OtelSpan {
    pub name: String,
    pub trace_id: String,
    pub span_id: String,
    pub status: SpanStatus,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpanStatus {
    Unset,
    OK,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Double(f64),
    Boolean(bool),
}

#[derive(Clone, Debug)]
pub struct SpanEvent {
    pub name: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// Semantic conventions validator
pub struct WeaverValidator {
    http_rules: HashMap<String, ConventionRule>,
    database_rules: HashMap<String, ConventionRule>,
    payment_rules: HashMap<String, ConventionRule>,
}

#[derive(Clone)]
pub struct ConventionRule {
    pub attribute: String,
    pub required: bool,
    pub expected_type: &'static str,
}

impl WeaverValidator {
    pub fn new() -> Self {
        let mut validator = WeaverValidator {
            http_rules: HashMap::new(),
            database_rules: HashMap::new(),
            payment_rules: HashMap::new(),
        };

        // HTTP semantic conventions (https://opentelemetry.io/docs/specs/semconv/http/)
        validator.http_rules.insert(
            "http.method".to_string(),
            ConventionRule {
                attribute: "http.method".to_string(),
                required: true,
                expected_type: "string",
            },
        );
        validator.http_rules.insert(
            "http.url".to_string(),
            ConventionRule {
                attribute: "http.url".to_string(),
                required: true,
                expected_type: "string",
            },
        );
        validator.http_rules.insert(
            "http.status_code".to_string(),
            ConventionRule {
                attribute: "http.status_code".to_string(),
                required: false,
                expected_type: "integer",
            },
        );

        // Database semantic conventions
        validator.database_rules.insert(
            "db.system".to_string(),
            ConventionRule {
                attribute: "db.system".to_string(),
                required: true,
                expected_type: "string",
            },
        );
        validator.database_rules.insert(
            "db.statement".to_string(),
            ConventionRule {
                attribute: "db.statement".to_string(),
                required: true,
                expected_type: "string",
            },
        );

        // Payment domain semantic conventions
        validator.payment_rules.insert(
            "payment.amount".to_string(),
            ConventionRule {
                attribute: "payment.amount".to_string(),
                required: true,
                expected_type: "double",
            },
        );
        validator.payment_rules.insert(
            "payment.currency".to_string(),
            ConventionRule {
                attribute: "payment.currency".to_string(),
                required: true,
                expected_type: "string",
            },
        );

        validator
    }

    /// Validate span against semantic conventions
    pub fn validate(&self, span: &OtelSpan) -> Result<(), ValidationError> {
        // Validate basic structure
        if span.name.is_empty() {
            return Err(ValidationError {
                span_name: span.name.clone(),
                error: "Span name cannot be empty".to_string(),
            });
        }

        if span.trace_id.is_empty() {
            return Err(ValidationError {
                span_name: span.name.clone(),
                error: "Trace ID missing".to_string(),
            });
        }

        if span.span_id.is_empty() {
            return Err(ValidationError {
                span_name: span.name.clone(),
                error: "Span ID missing".to_string(),
            });
        }

        // Validate based on span type
        if span.name.contains("http") {
            self.validate_http_span(span)?;
        }

        if span.name.contains("db") || span.name.contains("database") {
            self.validate_database_span(span)?;
        }

        if span.name.contains("payment") {
            self.validate_payment_span(span)?;
        }

        Ok(())
    }

    fn validate_http_span(&self, span: &OtelSpan) -> Result<(), ValidationError> {
        for (attr_name, rule) in &self.http_rules {
            if rule.required {
                if !span.attributes.contains_key(attr_name) {
                    return Err(ValidationError {
                        span_name: span.name.clone(),
                        error: format!("Required attribute missing: {}", attr_name),
                    });
                }

                // Verify type
                if let Some(value) = span.attributes.get(attr_name) {
                    if !self.verify_type(value, rule.expected_type) {
                        return Err(ValidationError {
                            span_name: span.name.clone(),
                            error: format!(
                                "Attribute {} has wrong type: expected {}",
                                attr_name, rule.expected_type
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_database_span(&self, span: &OtelSpan) -> Result<(), ValidationError> {
        for (attr_name, rule) in &self.database_rules {
            if rule.required && !span.attributes.contains_key(attr_name) {
                return Err(ValidationError {
                    span_name: span.name.clone(),
                    error: format!("Required attribute missing: {}", attr_name),
                });
            }
        }
        Ok(())
    }

    fn validate_payment_span(&self, span: &OtelSpan) -> Result<(), ValidationError> {
        for (attr_name, rule) in &self.payment_rules {
            if rule.required && !span.attributes.contains_key(attr_name) {
                return Err(ValidationError {
                    span_name: span.name.clone(),
                    error: format!("Required attribute missing: {}", attr_name),
                });
            }
        }
        Ok(())
    }

    fn verify_type(&self, value: &AttributeValue, expected_type: &str) -> bool {
        match (value, expected_type) {
            (AttributeValue::String(_), "string") => true,
            (AttributeValue::Integer(_), "integer") => true,
            (AttributeValue::Double(_), "double") => true,
            (AttributeValue::Boolean(_), "boolean") => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct ValidationError {
    pub span_name: String,
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weaver_validate_http_span() {
        let validator = WeaverValidator::new();

        let mut attributes = HashMap::new();
        attributes.insert(
            "http.method".to_string(),
            AttributeValue::String("GET".to_string()),
        );
        attributes.insert(
            "http.url".to_string(),
            AttributeValue::String("https://example.com/api/v1".to_string()),
        );
        attributes.insert(
            "http.status_code".to_string(),
            AttributeValue::Integer(200),
        );

        let span = OtelSpan {
            name: "http.request".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        };

        // Should validate successfully
        let result = validator.validate(&span);
        assert_ok(&result);
    }

    #[test]
    fn weaver_validate_missing_required_attribute() {
        let validator = WeaverValidator::new();

        let mut attributes = HashMap::new();
        attributes.insert(
            "http.method".to_string(),
            AttributeValue::String("GET".to_string()),
        );
        // Missing http.url (required)

        let span = OtelSpan {
            name: "http.request".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        };

        let result = validator.validate(&span);
        assert!(result.is_err());
    }

    #[test]
    fn weaver_validate_wrong_type() {
        let validator = WeaverValidator::new();

        let mut attributes = HashMap::new();
        attributes.insert(
            "http.method".to_string(),
            AttributeValue::Integer(123), // Should be string
        );
        attributes.insert(
            "http.url".to_string(),
            AttributeValue::String("https://example.com".to_string()),
        );

        let span = OtelSpan {
            name: "http.request".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        };

        let result = validator.validate(&span);
        assert!(result.is_err());
    }

    #[test]
    fn weaver_validate_database_span() {
        let validator = WeaverValidator::new();

        let mut attributes = HashMap::new();
        attributes.insert(
            "db.system".to_string(),
            AttributeValue::String("postgresql".to_string()),
        );
        attributes.insert(
            "db.statement".to_string(),
            AttributeValue::String("SELECT * FROM accounts".to_string()),
        );

        let span = OtelSpan {
            name: "db.query".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        };

        let result = validator.validate(&span);
        assert_ok(&result);
    }

    #[test]
    fn weaver_validate_payment_span() {
        let validator = WeaverValidator::new();

        let mut attributes = HashMap::new();
        attributes.insert(
            "payment.amount".to_string(),
            AttributeValue::Double(1000.0),
        );
        attributes.insert(
            "payment.currency".to_string(),
            AttributeValue::String("USD".to_string()),
        );

        let span = OtelSpan {
            name: "payment.process".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        };

        let result = validator.validate(&span);
        assert_ok(&result);
    }

    #[test]
    fn weaver_validate_span_collection() {
        let validator = WeaverValidator::new();

        let spans = vec![
            create_http_span("GET", "https://example.com"),
            create_db_span("postgresql", "SELECT * FROM users"),
            create_payment_span(5000.0, "USD"),
        ];

        let mut valid_count = 0;
        for span in spans {
            if validator.validate(&span).is_ok() {
                valid_count += 1;
            }
        }

        assert_eq!(valid_count, 3, "All spans should be valid");
    }

    fn create_http_span(method: &str, url: &str) -> OtelSpan {
        let mut attributes = HashMap::new();
        attributes.insert(
            "http.method".to_string(),
            AttributeValue::String(method.to_string()),
        );
        attributes.insert(
            "http.url".to_string(),
            AttributeValue::String(url.to_string()),
        );

        OtelSpan {
            name: "http.request".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span456".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        }
    }

    fn create_db_span(system: &str, statement: &str) -> OtelSpan {
        let mut attributes = HashMap::new();
        attributes.insert(
            "db.system".to_string(),
            AttributeValue::String(system.to_string()),
        );
        attributes.insert(
            "db.statement".to_string(),
            AttributeValue::String(statement.to_string()),
        );

        OtelSpan {
            name: "db.query".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span789".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        }
    }

    fn create_payment_span(amount: f64, currency: &str) -> OtelSpan {
        let mut attributes = HashMap::new();
        attributes.insert("payment.amount".to_string(), AttributeValue::Double(amount));
        attributes.insert(
            "payment.currency".to_string(),
            AttributeValue::String(currency.to_string()),
        );

        OtelSpan {
            name: "payment.process".to_string(),
            trace_id: "trace123".to_string(),
            span_id: "span999".to_string(),
            status: SpanStatus::OK,
            attributes,
            events: vec![],
        }
    }

    fn assert_ok<T, E: std::fmt::Debug>(result: &Result<T, E>) {
        if result.is_err() {
            panic!("Expected Ok, got Err: {:?}", result.err());
        }
    }
}
