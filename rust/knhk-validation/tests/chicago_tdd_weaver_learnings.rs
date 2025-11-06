// rust/knhk-validation/tests/chicago_tdd_weaver_learnings.rs
// Chicago TDD tests for Weaver learnings implementations
// Tests verify behavior and state, not implementation details

#[cfg(test)]
mod tests {
    use knhk_validation::policy_engine::{PolicyEngine, PolicyViolation, ViolationLevel};
    #[cfg(feature = "diagnostics")]
    use knhk_validation::diagnostics::{
        DiagnosticMessage, DiagnosticMessages, DiagnosticSeverity, DiagnosticLocation,
        DiagnosticFormat,
    };
    #[cfg(feature = "schema-resolution")]
    use knhk_validation::resolved_schema::{
        ResolvedRdfSchema, SchemaVersion, SchemaCatalog,
    };

    // ============================================================================
    // Policy Engine Tests (Chicago TDD: State-based verification)
    // ============================================================================

    /// Test: Policy engine validates guard constraint correctly
    /// Chicago TDD: Verify state (validation result) not implementation
    #[test]
    fn test_policy_engine_guard_constraint_valid() {
        let engine = PolicyEngine::new();
        
        // Act: Validate valid run length
        let result = engine.validate_guard_constraint(8);
        
        // Assert: Verify state (validation passes)
        assert!(result.is_ok(), "Valid run length should pass validation");
    }

    /// Test: Policy engine rejects guard constraint violation
    /// Chicago TDD: Verify state (violation returned) not implementation
    #[test]
    fn test_policy_engine_guard_constraint_violation() {
        let engine = PolicyEngine::new();
        
        // Act: Validate invalid run length
        let result = engine.validate_guard_constraint(9);
        
        // Assert: Verify state (violation returned with correct values)
        assert!(result.is_err(), "Invalid run length should return violation");
        
        if let Err(PolicyViolation::GuardConstraintViolation {
            actual_run_len,
            max_run_len,
            ..
        }) = result {
            assert_eq!(actual_run_len, 9, "Actual run length should be 9");
            assert_eq!(max_run_len, 8, "Max run length should be 8");
        } else {
            panic!("Expected GuardConstraintViolation");
        }
    }

    /// Test: Policy engine validates performance budget correctly
    /// Chicago TDD: Verify state (validation result) not implementation
    #[test]
    fn test_policy_engine_performance_budget_valid() {
        let engine = PolicyEngine::new();
        
        // Act: Validate valid tick count
        let result = engine.validate_performance_budget(8);
        
        // Assert: Verify state (validation passes)
        assert!(result.is_ok(), "Valid tick count should pass validation");
    }

    /// Test: Policy engine rejects performance budget violation
    /// Chicago TDD: Verify state (violation returned) not implementation
    #[test]
    fn test_policy_engine_performance_budget_violation() {
        let engine = PolicyEngine::new();
        
        // Act: Validate invalid tick count
        let result = engine.validate_performance_budget(9);
        
        // Assert: Verify state (violation returned with correct values)
        assert!(result.is_err(), "Invalid tick count should return violation");
        
        if let Err(PolicyViolation::PerformanceBudgetViolation {
            actual_ticks,
            max_ticks,
            ..
        }) = result {
            assert_eq!(actual_ticks, 9, "Actual ticks should be 9");
            assert_eq!(max_ticks, 8, "Max ticks should be 8");
        } else {
            panic!("Expected PerformanceBudgetViolation");
        }
    }

    /// Test: Policy engine validates receipt correctly
    /// Chicago TDD: Verify state (validation result) not implementation
    #[test]
    fn test_policy_engine_receipt_validation_valid() {
        let engine = PolicyEngine::new();
        let hash = b"test_hash";
        
        // Act: Validate matching receipt hash
        let result = engine.validate_receipt("receipt-1", hash, hash);
        
        // Assert: Verify state (validation passes)
        assert!(result.is_ok(), "Matching receipt hash should pass validation");
    }

    /// Test: Policy engine rejects receipt validation violation
    /// Chicago TDD: Verify state (violation returned) not implementation
    #[test]
    fn test_policy_engine_receipt_validation_violation() {
        let engine = PolicyEngine::new();
        let hash1 = b"test_hash_1";
        let hash2 = b"test_hash_2";
        
        // Act: Validate mismatched receipt hash
        let result = engine.validate_receipt("receipt-1", hash1, hash2);
        
        // Assert: Verify state (violation returned)
        assert!(result.is_err(), "Mismatched receipt hash should return violation");
        
        if let Err(PolicyViolation::ReceiptValidationViolation { receipt_id, .. }) = result {
            assert_eq!(receipt_id, "receipt-1", "Receipt ID should match");
        } else {
            panic!("Expected ReceiptValidationViolation");
        }
    }

    /// Test: Policy engine check_all returns all violations
    /// Chicago TDD: Verify state (violation collection) not implementation
    #[test]
    fn test_policy_engine_check_all() {
        let engine = PolicyEngine::new();
        
        // Act: Check all policies with violations
        let violations = engine.check_all(Some(9), Some(10), None);
        
        // Assert: Verify state (both violations returned)
        assert_eq!(violations.len(), 2, "Should return 2 violations");
        assert!(violations.iter().any(|v| matches!(v, PolicyViolation::GuardConstraintViolation { .. })));
        assert!(violations.iter().any(|v| matches!(v, PolicyViolation::PerformanceBudgetViolation { .. })));
    }

    /// Test: Policy violation levels are correct
    /// Chicago TDD: Verify state (violation level) not implementation
    #[test]
    fn test_policy_violation_levels() {
        let engine = PolicyEngine::new();
        
        // Act: Get violations
        let guard_violation = engine.validate_guard_constraint(9).unwrap_err();
        let perf_violation = engine.validate_performance_budget(9).unwrap_err();
        let receipt_violation = engine.validate_receipt("r1", b"h1", b"h2").unwrap_err();
        
        // Assert: Verify state (all violations have correct level)
        assert_eq!(guard_violation.level(), ViolationLevel::Violation);
        assert_eq!(perf_violation.level(), ViolationLevel::Violation);
        assert_eq!(receipt_violation.level(), ViolationLevel::Violation);
    }

    // ============================================================================
    // Diagnostics Tests (Chicago TDD: State-based verification)
    // ============================================================================

    /// Test: Diagnostic message creation with correct severity
    /// Chicago TDD: Verify state (message structure) not implementation
    #[cfg(feature = "diagnostics")]
    #[test]
    fn test_diagnostic_message_creation() {
        // Act: Create diagnostic message
        let diag = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error message".to_string(),
        );
        
        // Assert: Verify state (message has correct fields)
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code, "E001");
        assert_eq!(diag.message, "Test error message");
        assert!(diag.context.is_empty());
        assert!(diag.related.is_empty());
    }

    /// Test: Diagnostic message with location
    /// Chicago TDD: Verify state (location set correctly) not implementation
    #[cfg(feature = "diagnostics")]
    #[test]
    fn test_diagnostic_message_with_location() {
        // Act: Create diagnostic with location
        let diag = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error".to_string(),
        ).with_location(DiagnosticLocation {
            file: "test.rs".to_string(),
            line: Some(42),
            column: Some(10),
        });
        
        // Assert: Verify state (location is set)
        assert!(diag.location.is_some());
        if let Some(ref location) = diag.location {
            assert_eq!(location.file, "test.rs");
            assert_eq!(location.line, Some(42));
            assert_eq!(location.column, Some(10));
        }
    }

    /// Test: Diagnostic message with context
    /// Chicago TDD: Verify state (context added) not implementation
    #[cfg(feature = "diagnostics")]
    #[test]
    fn test_diagnostic_message_with_context() {
        // Act: Create diagnostic with context
        let diag = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error".to_string(),
        ).with_context("key1".to_string(), "value1".to_string())
         .with_context("key2".to_string(), "value2".to_string());
        
        // Assert: Verify state (context contains both fields)
        assert_eq!(diag.context.len(), 2);
        assert_eq!(diag.context.get("key1"), Some(&"value1".to_string()));
        assert_eq!(diag.context.get("key2"), Some(&"value2".to_string()));
    }

    /// Test: Diagnostic messages collection counts correctly
    /// Chicago TDD: Verify state (counts are correct) not implementation
    #[cfg(feature = "diagnostics")]
    #[test]
    fn test_diagnostic_messages_collection() {
        let mut diags = DiagnosticMessages::new();
        
        // Act: Add various diagnostics
        diags.add(DiagnosticMessage::new(
            DiagnosticSeverity::Info,
            "I001".to_string(),
            "Info message".to_string(),
        ));
        diags.add(DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W001".to_string(),
            "Warning message".to_string(),
        ));
        diags.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Error message".to_string(),
        ));
        
        // Assert: Verify state (counts are correct)
        assert_eq!(diags.counts.info, 1);
        assert_eq!(diags.counts.warning, 1);
        assert_eq!(diags.counts.error, 1);
        assert_eq!(diags.counts.fatal, 0);
        assert_eq!(diags.counts.total(), 3);
        assert!(diags.has_errors());
    }

    /// Test: Diagnostic messages JSON serialization
    /// Chicago TDD: Verify output (JSON format) not implementation
    #[cfg(feature = "diagnostics")]
    #[test]
    fn test_diagnostic_messages_json() {
        let mut diags = DiagnosticMessages::new();
        diags.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error".to_string(),
        ));
        
        // Act: Format as JSON
        let json = diags.format_json().unwrap();
        
        // Assert: Verify output (JSON contains expected fields)
        assert!(json.contains("E001"));
        assert!(json.contains("Test error"));
        assert!(json.contains("error"));
    }

    /// Test: Diagnostic format options work correctly
    /// Chicago TDD: Verify output (format) not implementation
    #[cfg(feature = "diagnostics")]
    #[test]
    fn test_diagnostic_format_options() {
        let mut diags = DiagnosticMessages::new();
        diags.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error".to_string(),
        ).with_location(DiagnosticLocation {
            file: "test.rs".to_string(),
            line: Some(42),
            column: Some(10),
        }));
        
        // Act: Format in different formats
        let ansi = DiagnosticFormat::Ansi.format(&diags).unwrap();
        let json = DiagnosticFormat::Json.format(&diags).unwrap();
        let github = DiagnosticFormat::GitHubWorkflow.format(&diags).unwrap();
        
        // Assert: Verify output (each format produces valid output)
        assert!(ansi.contains("E001"));
        assert!(json.contains("E001"));
        assert!(github.contains("E001"));
        assert!(github.contains("test.rs"));
    }

    // ============================================================================
    // Schema Resolution Tests (Chicago TDD: State-based verification)
    // ============================================================================

    /// Test: Schema version parsing and formatting
    /// Chicago TDD: Verify state (version structure) not implementation
    #[cfg(feature = "schema-resolution")]
    #[test]
    fn test_schema_version() {
        // Act: Create and parse version
        let v1 = SchemaVersion::new(1, 2, 3);
        let v2 = SchemaVersion::parse("1.2.3").unwrap();
        
        // Assert: Verify state (versions match)
        assert_eq!(v1.major, 1);
        assert_eq!(v1.minor, 2);
        assert_eq!(v1.patch, 3);
        assert_eq!(v1.to_string(), "1.2.3");
        assert_eq!(v1, v2);
    }

    /// Test: Resolved schema creation and metadata
    /// Chicago TDD: Verify state (schema structure) not implementation
    #[cfg(feature = "schema-resolution")]
    #[test]
    fn test_resolved_schema_creation() {
        // Act: Create resolved schema
        let mut schema = ResolvedRdfSchema::new(
            "test-schema".to_string(),
            SchemaVersion::new(1, 0, 0),
            "Test Schema".to_string(),
            "https://example.com/schema".to_string(),
        );
        schema.add_metadata("author".to_string(), "Test Author".to_string());
        
        // Assert: Verify state (schema has correct fields)
        assert_eq!(schema.schema_id, "test-schema");
        assert_eq!(schema.version.major, 1);
        assert_eq!(schema.name, "Test Schema");
        assert_eq!(schema.metadata.get("author"), Some(&"Test Author".to_string()));
    }

    /// Test: Schema compatibility checking
    /// Chicago TDD: Verify state (compatibility result) not implementation
    #[cfg(feature = "schema-resolution")]
    #[test]
    fn test_schema_compatibility() {
        // Act: Create schema and check compatibility
        let schema = ResolvedRdfSchema::new(
            "test-schema".to_string(),
            SchemaVersion::new(1, 0, 0),
            "Test Schema".to_string(),
            "https://example.com/schema".to_string(),
        );
        
        // Assert: Verify state (compatibility checks work)
        assert!(schema.is_compatible_with(&SchemaVersion::new(1, 1, 0)));
        assert!(schema.is_compatible_with(&SchemaVersion::new(1, 0, 1)));
        assert!(!schema.is_compatible_with(&SchemaVersion::new(2, 0, 0)));
    }

    /// Test: Schema identifier generation
    /// Chicago TDD: Verify output (identifier format) not implementation
    #[cfg(feature = "schema-resolution")]
    #[test]
    fn test_schema_identifier() {
        // Act: Create schema and get identifier
        let schema = ResolvedRdfSchema::new(
            "test-schema".to_string(),
            SchemaVersion::new(1, 2, 3),
            "Test Schema".to_string(),
            "https://example.com/schema".to_string(),
        );
        
        // Assert: Verify output (identifier format is correct)
        assert_eq!(schema.identifier(), "test-schema:1.2.3");
    }

    /// Test: Schema catalog operations
    /// Chicago TDD: Verify state (catalog state) not implementation
    #[cfg(feature = "schema-resolution")]
    #[test]
    fn test_schema_catalog() {
        // Act: Create catalog and add entries
        let mut catalog = SchemaCatalog::new();
        catalog.add_entry(knhk_validation::resolved_schema::SchemaCatalogEntry {
            id: "pred1".to_string(),
            entry_type: "predicate".to_string(),
            definition: alloc::collections::BTreeMap::new(),
        });
        
        // Assert: Verify state (catalog contains entry)
        assert_eq!(catalog.entries.len(), 1);
        assert!(catalog.find_entry("pred1").is_some());
        assert_eq!(catalog.find_entry("pred1").unwrap().entry_type, "predicate");
    }

    /// Test: Schema resolution result success
    /// Chicago TDD: Verify state (resolution result) not implementation
    #[cfg(feature = "schema-resolution")]
    #[test]
    fn test_schema_resolution_success() {
        // Act: Create successful resolution result
        let schema = ResolvedRdfSchema::new(
            "test-schema".to_string(),
            SchemaVersion::new(1, 0, 0),
            "Test Schema".to_string(),
            "https://example.com/schema".to_string(),
        );
        let result = knhk_validation::resolved_schema::SchemaResolutionResult::success(
            schema,
            vec!["dep1".to_string(), "dep2".to_string()],
        );
        
        // Assert: Verify state (resolution is successful)
        assert!(result.is_success());
        assert_eq!(result.lineage.len(), 2);
        assert!(result.errors.is_empty());
    }

    // ============================================================================
    // Integration Tests (Chicago TDD: Verify behavior across modules)
    // ============================================================================

    /// Test: Policy violations produce diagnostics
    /// Chicago TDD: Verify behavior (integration) not implementation
    #[cfg(all(feature = "policy-engine", feature = "diagnostics"))]
    #[test]
    fn test_policy_violation_to_diagnostic() {
        let engine = PolicyEngine::new();
        
        // Act: Get violation and convert to diagnostic
        let violation = engine.validate_guard_constraint(9).unwrap_err();
        let diag = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "GUARD_CONSTRAINT_VIOLATION".to_string(),
            violation.message().to_string(),
        ).with_context("run_len".to_string(), "9".to_string())
         .with_context("max_run_len".to_string(), "8".to_string());
        
        // Assert: Verify behavior (diagnostic contains violation info)
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.code, "GUARD_CONSTRAINT_VIOLATION");
        assert!(diag.message.contains("Guard constraint violated"));
        assert_eq!(diag.context.get("run_len"), Some(&"9".to_string()));
    }

    /// Test: Complete validation workflow
    /// Chicago TDD: Verify behavior (end-to-end) not implementation
    #[cfg(all(feature = "policy-engine", feature = "diagnostics"))]
    #[test]
    fn test_complete_validation_workflow() {
        let engine = PolicyEngine::new();
        let mut diags = DiagnosticMessages::new();
        
        // Act: Validate multiple constraints and collect diagnostics
        let violations = engine.check_all(Some(9), Some(10), None);
        
        for violation in violations {
            let diag = DiagnosticMessage::new(
                DiagnosticSeverity::Error,
                violation.id().to_string(),
                violation.message().to_string(),
            );
            diags.add(diag);
        }
        
        // Assert: Verify behavior (all violations captured)
        assert_eq!(diags.counts.error, 2);
        assert!(diags.has_errors());
        
        // Verify JSON output works
        let json = diags.format_json().unwrap();
        assert!(json.contains("Guard constraint"));
        assert!(json.contains("Performance budget"));
    }
}

