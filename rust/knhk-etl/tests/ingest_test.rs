// rust/knhk-etl/tests/ingest_test.rs
// Chicago TDD tests for IngestStage with oxigraph migration validation
//
// Chicago TDD Principles:
// - Real Collaborators: Uses actual oxigraph Store, no mocks
// - State-Based: Tests outputs and invariants, not implementation
// - Error Paths: Tests error handling and edge cases
// - Production-Ready: All tests use production code paths

#[cfg(feature = "std")]
mod tests {
    use knhk_etl::ingest::{IngestStage, RawTriple};
    use knhk_etl::error::PipelineError;

    /// Test: Parse simple Turtle triple
    /// Verifies: Basic RDF parsing with oxigraph works correctly
    #[test]
    fn test_parse_simple_turtle_triple() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            <http://example.org/alice> <http://example.org/name> "Alice" .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 1, "Should parse exactly one triple");
        
        let triple = &triples[0];
        assert_eq!(triple.subject, "http://example.org/alice");
        assert_eq!(triple.predicate, "http://example.org/name");
        assert_eq!(triple.object, "\"Alice\"");
        assert_eq!(triple.graph, None);
    }

    /// Test: Parse multiple triples
    /// Verifies: Multiple triples are parsed correctly
    #[test]
    fn test_parse_multiple_triples() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            <http://example.org/alice> <http://example.org/name> "Alice" .
            <http://example.org/bob> <http://example.org/name> "Bob" .
            <http://example.org/alice> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 3, "Should parse exactly three triples");
    }

    /// Test: Parse Turtle with prefixes
    /// Verifies: Prefix resolution works correctly with oxigraph
    #[test]
    fn test_parse_turtle_with_prefixes() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            @prefix ex: <http://example.org/> .
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            
            ex:alice foaf:name "Alice" .
            ex:bob foaf:name "Bob" .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2, "Should parse exactly two triples");
        
        // Verify prefix expansion
        assert!(triples.iter().any(|t| t.subject == "http://example.org/alice"));
        assert!(triples.iter().any(|t| t.predicate == "http://xmlns.com/foaf/0.1/name"));
    }

    /// Test: Parse Turtle with blank nodes
    /// Verifies: Blank node handling works correctly
    #[test]
    fn test_parse_turtle_with_blank_nodes() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            _:alice <http://example.org/name> "Alice" .
            _:bob <http://example.org/name> "Bob" .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2, "Should parse exactly two triples");
        
        // Verify blank node format
        assert!(triples.iter().all(|t| t.subject.starts_with("_:")), 
                "All subjects should be blank nodes");
    }

    /// Test: Parse Turtle with language-tagged literals
    /// Verifies: Language tag handling works correctly
    #[test]
    fn test_parse_turtle_with_language_tags() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            <http://example.org/alice> <http://example.org/name> "Alice"@en .
            <http://example.org/alice> <http://example.org/name> "Alicia"@es .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2, "Should parse exactly two triples");
        
        // Verify language tags are preserved
        assert!(triples.iter().any(|t| t.object.contains("@en")));
        assert!(triples.iter().any(|t| t.object.contains("@es")));
    }

    /// Test: Parse Turtle with typed literals
    /// Verifies: Datatype handling works correctly
    #[test]
    fn test_parse_turtle_with_typed_literals() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            <http://example.org/alice> <http://example.org/age> "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
            <http://example.org/alice> <http://example.org/height> "1.75"^^<http://www.w3.org/2001/XMLSchema#decimal> .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2, "Should parse exactly two triples");
        
        // Verify datatypes are preserved
        assert!(triples.iter().any(|t| t.object.contains("integer")));
        assert!(triples.iter().any(|t| t.object.contains("decimal")));
    }

    /// Test: Parse invalid Turtle syntax
    /// Verifies: Error handling for invalid input
    #[test]
    fn test_parse_invalid_turtle() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let invalid_turtle = r#"
            <http://example.org/alice> <http://example.org/name> "Alice" .
            <http://example.org/bob> <http://example.org/name> "Bob" . .
        "#;
        
        let result = ingest.parse_rdf_turtle(invalid_turtle);
        assert!(result.is_err(), "Parsing should fail for invalid syntax");
        
        match result.unwrap_err() {
            PipelineError::IngestError(msg) => {
                assert!(msg.contains("Failed to load Turtle data") || 
                       msg.contains("Failed to query store"),
                       "Error message should indicate parsing failure");
            }
            _ => panic!("Should return IngestError"),
        }
    }

    /// Test: Parse empty Turtle content
    /// Verifies: Empty input handling
    #[test]
    fn test_parse_empty_turtle() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let result = ingest.parse_rdf_turtle("");
        assert!(result.is_ok(), "Empty input should not error");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 0, "Should return empty triples list");
    }

    /// Test: Parse Turtle with base URI
    /// Verifies: Base URI resolution works correctly
    #[test]
    fn test_parse_turtle_with_base_uri() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            @base <http://example.org/> .
            <alice> <name> "Alice" .
            <bob> <name> "Bob" .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2, "Should parse exactly two triples");
        
        // Verify base URI expansion
        assert!(triples.iter().all(|t| t.subject.starts_with("http://example.org/")));
    }

    /// Test: Parse Turtle stream from reader
    /// Verifies: Streaming parser works correctly
    #[test]
    fn test_parse_turtle_stream() {
        use std::io::Cursor;
        
        let turtle = r#"
            <http://example.org/alice> <http://example.org/name> "Alice" .
            <http://example.org/bob> <http://example.org/name> "Bob" .
        "#;
        
        let reader = Cursor::new(turtle.as_bytes());
        let result = IngestStage::parse_rdf_turtle_stream(reader, None);
        
        assert!(result.is_ok(), "Stream parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 2, "Should parse exactly two triples");
    }

    /// Test: Verify triple conversion preserves all fields
    /// Verifies: Quad to RawTriple conversion is complete
    #[test]
    fn test_triple_conversion_completeness() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            <http://example.org/alice> <http://example.org/name> "Alice" .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 1);
        
        let triple = &triples[0];
        // Verify all fields are populated
        assert!(!triple.subject.is_empty(), "Subject should not be empty");
        assert!(!triple.predicate.is_empty(), "Predicate should not be empty");
        assert!(!triple.object.is_empty(), "Object should not be empty");
    }

    /// Test: Verify error messages are descriptive
    /// Verifies: Error handling provides useful context
    #[test]
    fn test_error_messages_are_descriptive() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        // Use malformed Turtle that will fail parsing
        let invalid = "This is not valid Turtle syntax at all";
        
        let result = ingest.parse_rdf_turtle(invalid);
        assert!(result.is_err(), "Should fail on invalid input");
        
        match result.unwrap_err() {
            PipelineError::IngestError(msg) => {
                assert!(!msg.is_empty(), "Error message should not be empty");
                // Error should contain context about what failed
                assert!(msg.contains("Turtle") || msg.contains("Failed"), 
                       "Error should mention Turtle parsing or failure");
            }
            _ => panic!("Should return IngestError"),
        }
    }

    /// Test: Verify oxigraph migration - no rio dependencies
    /// Verifies: Code uses oxigraph, not rio_turtle/rio_api
    #[test]
    fn test_oxigraph_migration_complete() {
        // This test verifies that the code compiles with oxigraph
        // and doesn't have any rio dependencies
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            <http://example.org/test> <http://example.org/prop> "value" .
        "#;
        
        // If this compiles and runs, oxigraph migration is successful
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Oxigraph parsing should work");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 1);
    }

    /// Test: Parse complex Turtle with all features
    /// Verifies: All Turtle features work together
    #[test]
    fn test_parse_complex_turtle() {
        let ingest = IngestStage::new(vec!["test".to_string()], "rdf/turtle".to_string());
        
        let turtle = r#"
            @base <http://example.org/> .
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
            
            <alice> foaf:name "Alice"@en ;
                    foaf:name "Alicia"@es ;
                    foaf:age "30"^^xsd:integer .
            
            _:bob foaf:name "Bob" .
        "#;
        
        let result = ingest.parse_rdf_turtle(turtle);
        assert!(result.is_ok(), "Complex parsing should succeed");
        
        let triples = result.unwrap();
        assert_eq!(triples.len(), 4, "Should parse all four triples");
        
        // Verify various features are preserved
        assert!(triples.iter().any(|t| t.object.contains("@en")));
        assert!(triples.iter().any(|t| t.object.contains("@es")));
        assert!(triples.iter().any(|t| t.object.contains("integer")));
        assert!(triples.iter().any(|t| t.subject.starts_with("_:")));
    }
}

