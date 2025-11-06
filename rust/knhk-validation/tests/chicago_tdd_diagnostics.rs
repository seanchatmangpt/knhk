// rust/knhk-validation/tests/chicago_tdd_diagnostics.rs
// Chicago TDD tests for Diagnostic System
// Tests behaviors, not implementation details

#[cfg(feature = "diagnostics")]
mod tests {
    use knhk_validation::diagnostics::*;

    #[test]
    fn test_diagnostic_message_creation() {
        // Arrange & Act: Create diagnostic message
        let diagnostic = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error message".to_string(),
        );
        
        // Assert: Message created correctly
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.code, "E001");
        assert_eq!(diagnostic.message, "Test error message");
        assert!(diagnostic.location.is_none());
        assert!(diagnostic.context.is_empty());
        assert!(diagnostic.related.is_empty());
    }

    #[test]
    fn test_diagnostic_message_with_location() {
        // Arrange: Create diagnostic with location
        let location = DiagnosticLocation {
            file: "src/main.rs".to_string(),
            line: Some(42),
            column: Some(10),
        };
        
        // Act: Add location
        let diagnostic = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E002".to_string(),
            "Error with location".to_string(),
        ).with_location(location.clone());
        
        // Assert: Location set
        assert_eq!(diagnostic.location, Some(location));
    }

    #[test]
    fn test_diagnostic_message_with_context() {
        // Arrange & Act: Create diagnostic with context
        let diagnostic = DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W001".to_string(),
            "Warning message".to_string(),
        ).with_context("key1".to_string(), "value1".to_string())
         .with_context("key2".to_string(), "value2".to_string());
        
        // Assert: Context added
        assert_eq!(diagnostic.context.len(), 2);
        assert_eq!(diagnostic.context.get("key1"), Some(&"value1".to_string()));
        assert_eq!(diagnostic.context.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_diagnostic_message_with_related() {
        // Arrange: Create related diagnostic
        let related = DiagnosticMessage::new(
            DiagnosticSeverity::Info,
            "I001".to_string(),
            "Related info".to_string(),
        );
        
        // Act: Add related diagnostic
        let diagnostic = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E003".to_string(),
            "Main error".to_string(),
        ).with_related(related.clone());
        
        // Assert: Related diagnostic added
        assert_eq!(diagnostic.related.len(), 1);
        assert_eq!(diagnostic.related[0].code, "I001");
    }

    #[test]
    fn test_diagnostic_message_format_ansi() {
        // Arrange: Create diagnostic
        let diagnostic = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E004".to_string(),
            "Test error".to_string(),
        ).with_location(DiagnosticLocation {
            file: "test.rs".to_string(),
            line: Some(10),
            column: Some(5),
        });
        
        // Act: Format as ANSI
        let formatted = diagnostic.format_ansi();
        
        // Assert: Contains expected elements
        assert!(formatted.contains("✗"));
        assert!(formatted.contains("E004"));
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("test.rs"));
        assert!(formatted.contains("10"));
    }

    #[test]
    fn test_diagnostic_message_format_json() {
        // Arrange: Create diagnostic
        let diagnostic = DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W002".to_string(),
            "JSON test".to_string(),
        );
        
        // Act: Format as JSON
        let json_result = diagnostic.format_json();
        
        // Assert: Valid JSON
        assert!(json_result.is_ok());
        let json = json_result.unwrap();
        assert!(json.contains("W002"));
        assert!(json.contains("JSON test"));
        assert!(json.contains("warning"));
    }

    #[test]
    fn test_diagnostic_messages_collection() {
        // Arrange: Create collection
        let mut diagnostics = DiagnosticMessages::new();
        
        // Act: Add messages
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Info,
            "I001".to_string(),
            "Info message".to_string(),
        ));
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W001".to_string(),
            "Warning message".to_string(),
        ));
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Error message".to_string(),
        ));
        
        // Assert: Counts correct
        assert_eq!(diagnostics.counts.info, 1);
        assert_eq!(diagnostics.counts.warning, 1);
        assert_eq!(diagnostics.counts.error, 1);
        assert_eq!(diagnostics.counts.fatal, 0);
        assert_eq!(diagnostics.counts.total(), 3);
        assert_eq!(diagnostics.messages.len(), 3);
    }

    #[test]
    fn test_diagnostic_messages_has_errors() {
        // Arrange: Create collection with errors
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Error".to_string(),
        ));
        
        // Act: Check has errors
        let has_errors = diagnostics.has_errors();
        
        // Assert: Has errors
        assert!(has_errors);
    }

    #[test]
    fn test_diagnostic_messages_no_errors() {
        // Arrange: Create collection without errors
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Info,
            "I001".to_string(),
            "Info".to_string(),
        ));
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W001".to_string(),
            "Warning".to_string(),
        ));
        
        // Act: Check has errors
        let has_errors = diagnostics.has_errors();
        
        // Assert: No errors
        assert!(!has_errors);
    }

    #[test]
    fn test_diagnostic_messages_has_fatal_errors() {
        // Arrange: Create collection with fatal
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Fatal,
            "F001".to_string(),
            "Fatal error".to_string(),
        ));
        
        // Act: Check has errors
        let has_errors = diagnostics.has_errors();
        
        // Assert: Has errors (fatal counts as error)
        assert!(has_errors);
        assert_eq!(diagnostics.counts.fatal, 1);
    }

    #[test]
    fn test_diagnostic_messages_format_ansi() {
        // Arrange: Create collection
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test error".to_string(),
        ));
        
        // Act: Format as ANSI
        let formatted = diagnostics.format_ansi();
        
        // Assert: Contains expected elements
        assert!(formatted.contains("Diagnostic Report"));
        assert!(formatted.contains("E001"));
        assert!(formatted.contains("Total"));
    }

    #[test]
    fn test_diagnostic_messages_format_json() {
        // Arrange: Create collection
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W001".to_string(),
            "JSON warning".to_string(),
        ));
        
        // Act: Format as JSON
        let json_result = diagnostics.format_json();
        
        // Assert: Valid JSON
        assert!(json_result.is_ok());
        let json = json_result.unwrap();
        assert!(json.contains("W001"));
    }

    #[test]
    fn test_diagnostic_format_ansi() {
        // Arrange: Create diagnostics
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Format test".to_string(),
        ));
        
        // Act: Format using DiagnosticFormat
        let format = DiagnosticFormat::Ansi;
        let formatted = format.format(&diagnostics);
        
        // Assert: ANSI formatted
        assert!(formatted.is_ok());
        let output = formatted.unwrap();
        assert!(output.contains("✗"));
    }

    #[test]
    fn test_diagnostic_format_json() {
        // Arrange: Create diagnostics
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Info,
            "I001".to_string(),
            "JSON format test".to_string(),
        ));
        
        // Act: Format using DiagnosticFormat
        let format = DiagnosticFormat::Json;
        let formatted = format.format(&diagnostics);
        
        // Assert: JSON formatted
        assert!(formatted.is_ok());
        let output = formatted.unwrap();
        assert!(output.contains("\"code\""));
    }

    #[test]
    fn test_diagnostic_format_github_workflow() {
        // Arrange: Create diagnostics with location
        let mut diagnostics = DiagnosticMessages::new();
        diagnostics.add(DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "GitHub test".to_string(),
        ).with_location(DiagnosticLocation {
            file: "src/main.rs".to_string(),
            line: Some(42),
            column: Some(10),
        }));
        
        // Act: Format using GitHub workflow format
        let format = DiagnosticFormat::GitHubWorkflow;
        let formatted = format.format(&diagnostics);
        
        // Assert: GitHub workflow format
        assert!(formatted.is_ok());
        let output = formatted.unwrap();
        assert!(output.contains("::error"));
        assert!(output.contains("file=src/main.rs"));
        assert!(output.contains("line=42"));
    }

    #[test]
    fn test_diagnostic_severity_levels() {
        // Arrange & Act: Create diagnostics with all severity levels
        let info = DiagnosticMessage::new(
            DiagnosticSeverity::Info,
            "I001".to_string(),
            "Info".to_string(),
        );
        let warning = DiagnosticMessage::new(
            DiagnosticSeverity::Warning,
            "W001".to_string(),
            "Warning".to_string(),
        );
        let error = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Error".to_string(),
        );
        let fatal = DiagnosticMessage::new(
            DiagnosticSeverity::Fatal,
            "F001".to_string(),
            "Fatal".to_string(),
        );
        
        // Assert: All severity levels distinct
        assert_ne!(info.severity, warning.severity);
        assert_ne!(warning.severity, error.severity);
        assert_ne!(error.severity, fatal.severity);
    }

    #[test]
    fn test_diagnostic_location_optional_fields() {
        // Arrange: Create location with optional fields
        let location_with_all = DiagnosticLocation {
            file: "test.rs".to_string(),
            line: Some(10),
            column: Some(5),
        };
        
        let location_no_line = DiagnosticLocation {
            file: "test.rs".to_string(),
            line: None,
            column: Some(5),
        };
        
        // Act: Format with location
        let diagnostic1 = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E001".to_string(),
            "Test".to_string(),
        ).with_location(location_with_all);
        
        let diagnostic2 = DiagnosticMessage::new(
            DiagnosticSeverity::Error,
            "E002".to_string(),
            "Test".to_string(),
        ).with_location(location_no_line);
        
        // Assert: Both format correctly
        let formatted1 = diagnostic1.format_ansi();
        let formatted2 = diagnostic2.format_ansi();
        
        assert!(formatted1.contains("10"));
        assert!(formatted2.contains("?"));
    }
}

